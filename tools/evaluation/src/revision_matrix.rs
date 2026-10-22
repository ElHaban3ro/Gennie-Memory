//! Blind review packet preparation and scoring for the synthetic revision matrix.
//! The packet builder never reads frozen annotations; the scorer opens them only
//! after blinded judgments have been finalized.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

const ARMS: [&str; 3] = ["D0", "G0", "G2"];
const RELATIONS: [&str; 6] = [
    "answers",
    "partial",
    "explicit_unmeasured",
    "refutes",
    "irrelevant",
    "ambiguous",
];
const OUTCOMES: [&str; 5] = [
    "selected",
    "abstained",
    "invalid_output",
    "provider_error",
    "service_error",
];

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|e| format!("read_{}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("json_{}: {e}", path.display()))
}

fn read_jsonl(path: &Path) -> Result<Vec<Value>, String> {
    let file = File::open(path).map_err(|e| format!("open_{}: {e}", path.display()))?;
    BufReader::new(file)
        .lines()
        .enumerate()
        .filter_map(|(i, line)| match line {
            Ok(line) if line.trim().is_empty() => None,
            Ok(line) => Some(
                serde_json::from_str(&line)
                    .map_err(|e| format!("jsonl_{}_line_{}: {e}", path.display(), i + 1)),
            ),
            Err(e) => Some(Err(format!("read_{}_line_{}: {e}", path.display(), i + 1))),
        })
        .collect()
}

fn write_json(path: &Path, value: &Value, private: bool) -> Result<(), String> {
    let mut opts = OpenOptions::new();
    opts.create_new(true).write(true);
    if private {
        opts.mode(0o600);
    }
    let mut file = opts
        .open(path)
        .map_err(|e| format!("create_{}: {e}", path.display()))?;
    serde_json::to_writer_pretty(&mut file, value).map_err(|e| format!("write_json: {e}"))?;
    file.write_all(b"\n")
        .map_err(|e| format!("write_json: {e}"))?;
    file.sync_all()
        .map_err(|e| format!("sync_{}: {e}", path.display()))
}

fn write_jsonl(path: &Path, rows: &[Value]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| format!("create_{}: {e}", path.display()))?;
    for row in rows {
        serde_json::to_writer(&mut file, row).map_err(|e| format!("write_jsonl: {e}"))?;
        file.write_all(b"\n")
            .map_err(|e| format!("write_jsonl: {e}"))?;
    }
    file.sync_all()
        .map_err(|e| format!("sync_{}: {e}", path.display()))
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn nonempty<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("missing_{key}"))
}

fn random_alias(prefix: &str) -> Result<String, String> {
    let mut bytes = [0u8; 12];
    File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|e| format!("secure_random_unavailable: {e}"))?;
    let digest = sha256(&bytes);
    Ok(format!("{prefix}-{}", &digest[..20]))
}

fn hash_file(path: &Path) -> Result<String, String> {
    Ok(sha256(
        &fs::read(path).map_err(|e| format!("read_{}: {e}", path.display()))?,
    ))
}

fn validate_preflight_hashes(preflight: &Path) -> Result<Value, String> {
    let manifest = read_json(&preflight.join("manifest.json"))?;
    let inputs = manifest["inputs"]
        .as_object()
        .ok_or("manifest_inputs_missing")?;
    for (path, expected) in inputs {
        let actual = hash_file(&preflight.join(path))?;
        if expected.as_str() != Some(actual.as_str()) {
            return Err(format!("preflight_input_hash_mismatch:{path}"));
        }
    }
    let requests = manifest["requests"]
        .as_array()
        .ok_or("manifest_requests_missing")?;
    for row in requests {
        let path = nonempty(row, "path")?;
        if hash_file(&preflight.join(path))? != nonempty(row, "sha256")? {
            return Err(format!("preflight_request_hash_mismatch:{path}"));
        }
    }
    Ok(manifest)
}

fn ensure_new_dir(path: &Path, private: bool) -> Result<(), String> {
    fs::create_dir(path).map_err(|e| format!("create_dir_{}: {e}", path.display()))?;
    if private {
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("chmod_{}: {e}", path.display()))?;
    }
    Ok(())
}

fn scan_for_sensitive_fields(v: &Value, sensitive_values: &BTreeSet<String>) -> Result<(), String> {
    let banned = [
        "arm",
        "provider",
        "model",
        "policy",
        "selection_policy",
        "trace",
        "schema",
        "error",
        "timing",
        "cost",
        "expected_ids",
        "expected_selected",
        "answer_boundary",
        "annotation",
        "memory_id",
        "request_id",
        "case_id",
        "served_model",
        "requested_model",
        "input_tokens",
        "output_tokens",
        "reported_cost_usd",
        "elapsed_ms",
        "error_code",
    ];
    match v {
        Value::Object(map) => {
            for (key, value) in map {
                let normalized = key.to_ascii_lowercase();
                let tokenized = normalized.split('_').collect::<BTreeSet<_>>();
                let technical = [
                    "arm",
                    "provider",
                    "model",
                    "policy",
                    "trace",
                    "schema",
                    "error",
                    "timing",
                    "cost",
                    "annotation",
                    "tokens",
                ];
                if banned.contains(&key.as_str())
                    || technical.iter().any(|token| tokenized.contains(token))
                    || normalized.starts_with("expected_")
                {
                    return Err(format!("blind_packet_sensitive_field:{key}"));
                }
                scan_for_sensitive_fields(value, sensitive_values)?;
            }
        }
        Value::Array(rows) => {
            for row in rows {
                scan_for_sensitive_fields(row, sensitive_values)?;
            }
        }
        Value::String(value) => {
            let lower = value.to_lowercase();
            for sensitive in sensitive_values {
                let needle = sensitive.to_lowercase();
                if needle.is_empty() {
                    continue;
                }
                let mut start = 0;
                while let Some(offset) = lower[start..].find(&needle) {
                    let from = start + offset;
                    let to = from + needle.len();
                    let boundary = |c: Option<char>| {
                        c.is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
                    };
                    let before = lower[..from].chars().next_back();
                    let after = lower[to..].chars().next();
                    if boundary(before) && boundary(after) {
                        return Err("blind_packet_sensitive_value".into());
                    }
                    start = to;
                    if start >= lower.len() {
                        break;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Clone)]
struct Candidate {
    memory_id: String,
    summary: String,
}

fn candidate_roster_sha256(candidates: &[Candidate]) -> Result<String, String> {
    let canonical: Vec<Value> = candidates
        .iter()
        .map(|candidate| json!({"memory_id":candidate.memory_id,"summary":candidate.summary}))
        .collect();
    let bytes = serde_json::to_vec(&canonical).map_err(|e| format!("roster_serialize: {e}"))?;
    Ok(sha256(&bytes))
}

fn arm_name(path: &str) -> Result<(&str, &str), String> {
    let name = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("invalid_request_path")?;
    let (case, arm) = name.rsplit_once('-').ok_or("invalid_request_filename")?;
    if !ARMS.contains(&arm) || !case.starts_with("revision-") {
        return Err("invalid_request_filename".into());
    }
    Ok((case, arm))
}

/// Build a reviewer-only packet and a separately protected mapping.
/// `arm-results.jsonl` accepts normalized rows with `case_id`, `arm`,
/// `runtime_status`, `failure_class`, `selected_ids`, request/binary/harness/prompt/schema hashes,
/// served model, timings, tokens and cost receipt fields.
pub fn prepare_blind_review(
    preflight: &Path,
    arm_results: &Path,
    output: &Path,
) -> Result<(), String> {
    if output.exists() {
        return Err("output_already_exists".into());
    }
    let manifest = validate_preflight_hashes(preflight)?;
    let requests = manifest["requests"]
        .as_array()
        .ok_or("manifest_requests_missing")?;
    let mut cases: BTreeMap<String, (Value, BTreeMap<String, Value>)> = BTreeMap::new();
    let mut expected_request_hashes = BTreeMap::new();
    let corpus_sha256 = manifest["inputs"]["corpus.json"]
        .as_str()
        .ok_or("corpus_hash_missing")?
        .to_owned();
    let search_results_sha256 = manifest["inputs"]["search-results.jsonl"]
        .as_str()
        .ok_or("search_results_hash_missing")?
        .to_owned();
    let mut sensitive_values: BTreeSet<String> =
        ARMS.iter().map(|arm| arm.to_lowercase()).collect();
    for request_row in requests {
        let path = nonempty(request_row, "path")?;
        let (case_id, arm) = arm_name(path)?;
        let request_path = preflight.join(path);
        let request_hash = nonempty(request_row, "sha256")?.to_owned();
        expected_request_hashes.insert((case_id.to_owned(), arm.to_owned()), request_hash);
        let request = read_json(&request_path)?;
        if request["provider"].as_str().is_none()
            || request["selection_policy"].as_str().is_none()
            || request["search"].is_null()
        {
            return Err(format!("invalid_request:{case_id}:{arm}"));
        }
        sensitive_values.insert(request["provider"].as_str().unwrap().to_owned());
        sensitive_values.insert(request["selection_policy"].as_str().unwrap().to_owned());
        let entry = cases
            .entry(case_id.to_owned())
            .or_insert_with(|| (request["search"].clone(), BTreeMap::new()));
        if entry.0 != request["search"] {
            return Err(format!("search_mismatch:{case_id}"));
        }
        if entry.1.insert(arm.to_owned(), request).is_some() {
            return Err(format!("duplicate_arm:{case_id}:{arm}"));
        }
    }
    if cases.is_empty() || cases.values().any(|(_, arms)| arms.len() != ARMS.len()) {
        return Err("expected_complete_arm_triplets".into());
    }
    let case_count = cases.len();
    let expected_result_count = case_count * ARMS.len();

    let mut candidates_by_case = BTreeMap::new();
    for row in read_jsonl(&preflight.join("search-results.jsonl"))? {
        let case_id = nonempty(&row, "id")?.to_owned();
        let candidate_rows = row["response"]["candidates"]
            .as_array()
            .ok_or_else(|| format!("missing_candidates:{case_id}"))?;
        let mut candidates = Vec::new();
        let mut seen = BTreeSet::new();
        for c in candidate_rows {
            let memory_id = nonempty(c, "memory_id")?.to_owned();
            let summary = nonempty(c, "summary")?.to_owned();
            if !seen.insert(memory_id.clone()) {
                return Err(format!("duplicate_candidate:{case_id}"));
            }
            candidates.push(Candidate { memory_id, summary });
        }
        sensitive_values.insert(case_id.clone());
        for candidate in &candidates {
            sensitive_values.insert(candidate.memory_id.clone());
        }
        if candidates_by_case
            .insert(case_id.clone(), candidates)
            .is_some()
        {
            return Err(format!("duplicate_search_case:{case_id}"));
        }
    }
    if candidates_by_case.len() != case_count {
        return Err("search_case_count_mismatch".into());
    }

    let results = read_jsonl(arm_results)?;
    if results.len() != expected_result_count {
        return Err(format!("expected_{expected_result_count}_arm_results"));
    }
    let mut result_by_key = BTreeMap::new();
    for result in results {
        let case = nonempty(&result, "case_id")?.to_owned();
        let arm = nonempty(&result, "arm")?.to_owned();
        if !ARMS.contains(&arm.as_str()) {
            return Err(format!("invalid_arm:{arm}"));
        }
        let outcome = derive_outcome(&result, &case, &arm)?;
        let selected = result["selected_ids"]
            .as_array()
            .ok_or("selected_ids_missing")?;
        let mut selected_seen = BTreeSet::new();
        for id in selected {
            let id = id
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or("invalid_selected_id")?;
            if !selected_seen.insert(id.to_owned()) {
                return Err(format!("duplicate_selected_id:{case}:{arm}"));
            }
        }
        if outcome != "selected" && !selected.is_empty() {
            return Err(format!("nonselected_with_ids:{case}:{arm}"));
        }
        validate_execution_fingerprint(&result, &case, &arm)?;
        for field in [
            "corpus_sha256",
            "search_results_sha256",
            "candidate_roster_sha256",
        ] {
            let digest = nonempty(&result, field)?;
            if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(format!("invalid_{field}:{case}:{arm}"));
            }
        }
        if result["corpus_sha256"].as_str() != Some(corpus_sha256.as_str())
            || result["search_results_sha256"].as_str() != Some(search_results_sha256.as_str())
        {
            return Err(format!("shared_input_hash_mismatch:{case}:{arm}"));
        }
        for field in ["requested_model", "served_model"] {
            if let Some(value) = result[field].as_str().filter(|value| !value.is_empty()) {
                sensitive_values.insert(value.to_owned());
            }
        }
        let expected_hash = expected_request_hashes
            .get(&(case.clone(), arm.clone()))
            .ok_or_else(|| format!("request_fingerprint_missing:{case}:{arm}"))?;
        if result["request_sha256"].as_str() != Some(expected_hash.as_str()) {
            return Err(format!("execution_request_hash_mismatch:{case}:{arm}"));
        }
        if result_by_key
            .insert((case.clone(), arm.clone()), result)
            .is_some()
        {
            return Err(format!("duplicate_arm_result:{case}:{arm}"));
        }
    }
    for case in cases.keys() {
        for arm in ARMS {
            if !result_by_key.contains_key(&(case.clone(), arm.to_owned())) {
                return Err(format!("missing_arm_result:{case}:{arm}"));
            }
        }
        let roster_hash =
            candidate_roster_sha256(candidates_by_case.get(case).ok_or("search_case_missing")?)?;
        for arm in ARMS {
            let result = &result_by_key[&(case.clone(), arm.to_owned())];
            if result["candidate_roster_sha256"].as_str() != Some(roster_hash.as_str()) {
                return Err(format!("candidate_roster_hash_mismatch:{case}:{arm}"));
            }
        }
        if result_by_key[&(case.clone(), "G0".into())]["requested_model"]
            != result_by_key[&(case.clone(), "G2".into())]["requested_model"]
        {
            return Err(format!("paired_model_mismatch:{case}"));
        }
    }

    let mut reviewer_cases = Vec::new();
    let mut private_cases = Vec::new();
    for (case_id, (search, _arms)) in cases {
        let case_alias = random_alias("case")?;
        let candidates = candidates_by_case
            .get(&case_id)
            .ok_or_else(|| format!("search_case_missing:{case_id}"))?;
        let candidate_aliases: Vec<String> = (0..candidates.len())
            .map(|i| format!("M{}", i + 1))
            .collect();
        let mut public_candidates = Vec::new();
        let mut private_candidate_map = serde_json::Map::new();
        let roster: BTreeSet<String> = candidates.iter().map(|c| c.memory_id.clone()).collect();
        for (alias, candidate) in candidate_aliases.iter().zip(candidates) {
            public_candidates.push(json!({
                "candidate_alias": alias,
                "summary": candidate.summary
            }));
            private_candidate_map.insert(alias.clone(), json!(candidate.memory_id));
        }
        let mut conditions = Vec::new();
        let mut private_conditions = serde_json::Map::new();
        let mut arms_order = ARMS.to_vec();
        secure_shuffle(&mut arms_order)?;
        for arm in arms_order {
            let condition_alias = random_alias("condition")?;
            let result = &result_by_key[&(case_id.clone(), arm.to_owned())];
            let selected_ids: BTreeSet<String> = result["selected_ids"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect();
            let selected_aliases: Vec<String> = candidates
                .iter()
                .zip(&candidate_aliases)
                .filter(|(c, _)| selected_ids.contains(&c.memory_id))
                .map(|(_, alias)| alias.clone())
                .collect();
            conditions.push(json!({
                "condition_alias": condition_alias,
                "selected_candidate_aliases": selected_aliases
            }));
            private_conditions.insert(
                condition_alias,
                json!({
                    "arm": arm,
                "outcome": derive_outcome(result, &case_id, &arm)?,
                    "unpresented_selected_ids": selected_ids.difference(&roster).collect::<Vec<_>>()
                }),
            );
        }
        let scope_kind = search["scope_kind"].as_str().ok_or("scope_kind_missing")?;
        let review_case = json!({
            "case_alias": case_alias,
            "query": search["query"],
            "scope_kind": scope_kind,
            "as_of": search["as_of"],
            "candidates": public_candidates,
            "conditions": conditions
        });
        scan_for_sensitive_fields(&review_case, &sensitive_values)?;
        reviewer_cases.push(review_case);
        private_cases.push(json!({
            "case_id": case_id,
            "case_alias": case_alias,
            "candidate_aliases": private_candidate_map,
            "conditions": private_conditions
        }));
    }

    let review_dir = output.join("review");
    let private_dir = output.join("private");
    ensure_new_dir(output, false)?;
    ensure_new_dir(&review_dir, false)?;
    ensure_new_dir(&private_dir, true)?;
    let reviewer_path = review_dir.join("packet.jsonl");
    write_jsonl(&reviewer_path, &reviewer_cases)?;
    let reviewer_manifest = json!({
        "version": "revision-matrix-v1-blind-review-packet-v1",
        "synthetic": true,
        "cases": reviewer_cases.len(),
        "sha256": hash_file(&reviewer_path)?
    });
    write_json(&review_dir.join("manifest.json"), &reviewer_manifest, false)?;
    let private_key = json!({
        "version": "revision-matrix-v1-private-map-v1",
        "review_manifest_sha256": hash_file(&review_dir.join("manifest.json"))?,
        "preflight_manifest_sha256": hash_file(&preflight.join("manifest.json"))?,
        "arm_results_sha256": hash_file(arm_results)?,
        "cases": private_cases
    });
    write_json(&private_dir.join("mapping.json"), &private_key, true)?;
    Ok(())
}

fn secure_shuffle<T>(values: &mut [T]) -> Result<(), String> {
    let mut random = [0u8; 8];
    for i in (1..values.len()).rev() {
        File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut random))
            .map_err(|e| format!("secure_random_unavailable: {e}"))?;
        let n = u64::from_ne_bytes(random) as usize % (i + 1);
        values.swap(i, n);
    }
    Ok(())
}

fn validate_execution_fingerprint(row: &Value, case: &str, arm: &str) -> Result<(), String> {
    for field in [
        "request_sha256",
        "binary_sha256",
        "harness_sha256",
        "prompt_sha256",
        "schema_sha256",
        "response_sha256",
    ] {
        let value = nonempty(row, field)?;
        if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("invalid_{field}:{case}:{arm}"));
        }
    }
    for field in ["elapsed_classifier_ms", "elapsed_service_ms"] {
        if row[field]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .is_none()
        {
            return Err(format!("invalid_{field}:{case}:{arm}"));
        }
    }
    nonempty(row, "requested_model")?;
    if !row.get("served_model").is_some()
        || !row.get("input_tokens").is_some()
        || !row.get("output_tokens").is_some()
        || !row.get("receipt_sha256").is_some()
    {
        return Err(format!("execution_receipt_fields_missing:{case}:{arm}"));
    }
    for field in ["input_tokens", "output_tokens"] {
        if !row[field].is_null() && row[field].as_u64().is_none() {
            return Err(format!("invalid_{field}:{case}:{arm}"));
        }
    }
    if !row.get("reported_cost_usd").is_some() {
        return Err(format!("reported_cost_field_missing:{case}:{arm}"));
    }
    if !row.get("error_code").is_some() || !row.get("provider_http_status").is_some() {
        return Err(format!("execution_error_fields_missing:{case}:{arm}"));
    }
    if !row["error_code"].is_null()
        && row["error_code"]
            .as_str()
            .is_none_or(|value| value.trim().is_empty())
    {
        return Err(format!("invalid_error_code:{case}:{arm}"));
    }
    if !row["provider_http_status"].is_null()
        && row["provider_http_status"]
            .as_u64()
            .is_none_or(|status| status > u16::MAX as u64)
    {
        return Err(format!("invalid_provider_http_status:{case}:{arm}"));
    }
    if let Some(receipt) = row["receipt_sha256"].as_str() {
        if receipt.len() != 64 || !receipt.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("invalid_receipt_sha256:{case}:{arm}"));
        }
    }
    if row["reported_cost_usd"].is_number() && row["receipt_sha256"].as_str().is_none() {
        return Err(format!("cost_without_receipt:{case}:{arm}"));
    }
    if row["reported_cost_usd"].is_null() && row["receipt_sha256"].is_string() {
        // A receipt can omit provider pricing; keep the cost explicitly unknown.
    }
    Ok(())
}

fn derive_outcome(row: &Value, case: &str, arm: &str) -> Result<&'static str, String> {
    let runtime_status = nonempty(row, "runtime_status")?;
    let failure_class = nonempty(row, "failure_class")?;
    let selected = row["selected_ids"]
        .as_array()
        .ok_or("selected_ids_missing")?;
    let outcome = match (runtime_status, failure_class) {
        ("complete", "none") if selected.is_empty() => "abstained",
        ("complete", "none") => "selected",
        ("degraded", "invalid_output") => "invalid_output",
        ("degraded", "provider_error") => "provider_error",
        ("degraded", "service_error") => "service_error",
        _ => return Err(format!("inconsistent_runtime_outcome:{case}:{arm}")),
    };
    if !OUTCOMES.contains(&outcome) {
        return Err(format!("invalid_outcome:{case}:{arm}"));
    }
    Ok(outcome)
}

#[derive(Clone)]
struct Judgment {
    relation: String,
    evidence_quote: String,
    evidence_valid: bool,
    also_refutes_premise: bool,
    rationale: String,
}

#[derive(Clone)]
struct BlindnessGuess {
    guessed_arm: Option<String>,
    confidence: String,
    rationale: String,
}

fn judgment_json(judgment: &Judgment) -> Value {
    json!({
        "relation":judgment.relation,
        "evidence_quote":judgment.evidence_quote,
        "evidence_valid":judgment.evidence_valid,
        "also_refutes_premise":judgment.also_refutes_premise,
        "rationale":judgment.rationale
    })
}

fn judgment_rows(path: &Path) -> Result<BTreeMap<(String, String, String), Judgment>, String> {
    let mut rows = BTreeMap::new();
    for row in read_jsonl(path)? {
        if row["record_type"].as_str() == Some("blindness_guess") {
            continue;
        }
        if row.get("record_type").is_some() && row["record_type"].as_str() != Some("judgment") {
            return Err("invalid_reviewer_record_type".into());
        }
        let case = nonempty(&row, "case_alias")?.to_owned();
        let condition = nonempty(&row, "condition_alias")?.to_owned();
        let candidate = nonempty(&row, "candidate_alias")?.to_owned();
        let relation = nonempty(&row, "relation")?.to_owned();
        if !RELATIONS.contains(&relation.as_str()) {
            return Err(format!("invalid_relation:{case}:{condition}:{candidate}"));
        }
        let evidence_quote = nonempty(&row, "evidence_quote")?.to_owned();
        let evidence_valid = row["evidence_valid"]
            .as_bool()
            .ok_or("evidence_valid_missing")?;
        let also_refutes_premise = row["also_refutes_premise"]
            .as_bool()
            .ok_or("also_refutes_missing")?;
        let rationale = nonempty(&row, "rationale")?.to_owned();
        let judgment = Judgment {
            relation,
            evidence_quote,
            evidence_valid,
            also_refutes_premise,
            rationale,
        };
        if rows
            .insert(
                (case.clone(), condition.clone(), candidate.clone()),
                judgment,
            )
            .is_some()
        {
            return Err(format!("duplicate_judgment:{case}:{condition}:{candidate}"));
        }
    }
    Ok(rows)
}

fn blindness_guesses(path: &Path) -> Result<BTreeMap<(String, String), BlindnessGuess>, String> {
    let mut guesses = BTreeMap::new();
    let mut judgments = Vec::new();
    let mut guess_records = Vec::new();
    let mut guesses_started = false;
    for row in read_jsonl(path)? {
        if row["record_type"].as_str() != Some("blindness_guess") {
            if guesses_started {
                return Err("judgment_after_blindness_guess".into());
            }
            if row.get("record_type").is_some() && row["record_type"].as_str() != Some("judgment") {
                return Err("invalid_reviewer_record_type".into());
            }
            judgments.push(row);
            continue;
        }
        guesses_started = true;
        guess_records.push(row);
    }
    let judgment_digest = serde_json::to_vec(&judgments)
        .map(|bytes| sha256(&bytes))
        .map_err(|e| format!("reviewer_judgments_serialize: {e}"))?;
    for row in guess_records {
        if row["judgment_records_sha256"].as_str() != Some(judgment_digest.as_str()) {
            return Err("blindness_guess_judgments_hash_mismatch".into());
        }
        let case = nonempty(&row, "case_alias")?.to_owned();
        let condition = nonempty(&row, "condition_alias")?.to_owned();
        let guessed_arm = match row.get("guessed_arm") {
            Some(Value::Null) => None,
            Some(Value::String(value)) if ["D0", "G0", "G2"].contains(&value.as_str()) => {
                Some(value.clone())
            }
            _ => return Err("invalid_blindness_guess_arm".into()),
        };
        let confidence = nonempty(&row, "guess_confidence")?.to_owned();
        if !["low", "medium", "high"].contains(&confidence.as_str()) {
            return Err("invalid_blindness_guess_confidence".into());
        }
        let rationale = nonempty(&row, "guess_rationale")?.to_owned();
        if guesses
            .insert(
                (case.clone(), condition.clone()),
                BlindnessGuess {
                    guessed_arm,
                    confidence,
                    rationale,
                },
            )
            .is_some()
        {
            return Err(format!("duplicate_blindness_guess:{case}:{condition}"));
        }
    }
    Ok(guesses)
}

fn as_set(value: &Value, field: &str) -> Result<BTreeSet<String>, String> {
    value[field]
        .as_array()
        .ok_or_else(|| format!("missing_{field}"))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("invalid_{field}"))
        })
        .collect()
}

fn p50_p95(values: &[f64]) -> Value {
    if values.is_empty() {
        return json!({"n": 0, "p50": null, "p95": null});
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let nearest_rank = |p: f64| -> f64 {
        let index = ((p * sorted.len() as f64).ceil() as usize).saturating_sub(1);
        sorted[index]
    };
    json!({"n":sorted.len(),"p50":nearest_rank(0.50),"p95":nearest_rank(0.95)})
}

/// Score only after both blinded reviewer files and adjudications are frozen.
pub fn score_blind_review(
    preflight: &Path,
    arm_results: &Path,
    mapping_path: &Path,
    reviewer_a_path: &Path,
    reviewer_b_path: &Path,
    adjudication_path: &Path,
    output: &Path,
) -> Result<(), String> {
    if output.exists() {
        return Err("output_already_exists".into());
    }
    validate_preflight_hashes(preflight)?;
    let mapping = read_json(mapping_path)?;
    let preflight_manifest_sha256 = hash_file(&preflight.join("manifest.json"))?;
    if mapping["preflight_manifest_sha256"].as_str() != Some(preflight_manifest_sha256.as_str()) {
        return Err("preflight_manifest_binding_mismatch".into());
    }
    let output_dir = mapping_path
        .parent()
        .and_then(Path::parent)
        .ok_or("private_mapping_path_invalid")?;
    let review_manifest_path = output_dir.join("review/manifest.json");
    let review_manifest_sha256 = hash_file(&review_manifest_path)?;
    if mapping["review_manifest_sha256"].as_str() != Some(review_manifest_sha256.as_str()) {
        return Err("review_manifest_binding_mismatch".into());
    }
    let review_manifest = read_json(&review_manifest_path)?;
    let review_packet_sha256 = hash_file(&output_dir.join("review/packet.jsonl"))?;
    if review_manifest["sha256"].as_str() != Some(review_packet_sha256.as_str()) {
        return Err("review_packet_hash_mismatch".into());
    }
    let arm_results_sha256 = hash_file(arm_results)?;
    if mapping["arm_results_sha256"].as_str() != Some(arm_results_sha256.as_str()) {
        return Err("arm_results_hash_mismatch".into());
    }
    let reviewer_a = judgment_rows(reviewer_a_path)?;
    let reviewer_b = judgment_rows(reviewer_b_path)?;
    let guesses_a = blindness_guesses(reviewer_a_path)?;
    let guesses_b = blindness_guesses(reviewer_b_path)?;
    let adjudications = judgment_rows(adjudication_path)?;
    let annotations: BTreeMap<String, Value> = read_jsonl(&preflight.join("annotations.jsonl"))?
        .into_iter()
        .map(|row| Ok((nonempty(&row, "id")?.to_owned(), row)))
        .collect::<Result<_, String>>()?;
    let search_rows: BTreeMap<String, Value> = read_jsonl(&preflight.join("search-results.jsonl"))?
        .into_iter()
        .map(|row| Ok((nonempty(&row, "id")?.to_owned(), row)))
        .collect::<Result<_, String>>()?;
    let mut results = BTreeMap::new();
    for row in read_jsonl(arm_results)? {
        let key = (
            nonempty(&row, "case_id")?.to_owned(),
            nonempty(&row, "arm")?.to_owned(),
        );
        if results.insert(key.clone(), row).is_some() {
            return Err(format!("duplicate_arm_result:{}:{}", key.0, key.1));
        }
    }
    let case_count = mapping["cases"]
        .as_array()
        .ok_or("mapping_cases_missing")?
        .len();
    let expected_result_count = case_count * ARMS.len();
    if case_count == 0 || results.len() != expected_result_count {
        return Err(format!("expected_{expected_result_count}_scoring_results"));
    }

    let mut case_reports = Vec::new();
    let mut required_judgment_keys = BTreeSet::new();
    let mut required_adjudication_keys = BTreeSet::new();
    let mut required_guess_keys = BTreeSet::new();
    let mut blindness_audit = Vec::new();
    let mut per_arm: BTreeMap<String, Vec<Value>> =
        ARMS.iter().map(|a| (a.to_string(), Vec::new())).collect();
    for case in mapping["cases"].as_array().ok_or("mapping_cases_missing")? {
        let case_id = nonempty(case, "case_id")?;
        let case_alias = nonempty(case, "case_alias")?;
        let annotation = annotations.get(case_id).ok_or("annotation_case_missing")?;
        let expected = as_set(annotation, "expected_selected")?;
        let search = search_rows.get(case_id).ok_or("search_case_missing")?;
        let roster: BTreeSet<String> = search["response"]["candidates"]
            .as_array()
            .ok_or("search_candidates_missing")?
            .iter()
            .map(|v| nonempty(v, "memory_id").map(str::to_owned))
            .collect::<Result<_, _>>()?;
        let candidates = case["candidate_aliases"]
            .as_object()
            .ok_or("candidate_map_missing")?;
        let mut private_candidate_by_alias = BTreeMap::new();
        for (alias, value) in candidates {
            private_candidate_by_alias.insert(
                alias.clone(),
                value.as_str().ok_or("candidate_map_invalid")?.to_owned(),
            );
        }
        let condition_map = case["conditions"]
            .as_object()
            .ok_or("condition_map_missing")?;
        for (condition_alias, private) in condition_map {
            let arm = nonempty(private, "arm")?.to_owned();
            required_guess_keys.insert((case_alias.to_owned(), condition_alias.clone()));
            let guess_a = guesses_a
                .get(&(case_alias.to_owned(), condition_alias.clone()))
                .ok_or_else(|| format!("reviewer_a_blindness_guess_missing:{case_id}:{arm}"))?;
            let guess_b = guesses_b
                .get(&(case_alias.to_owned(), condition_alias.clone()))
                .ok_or_else(|| format!("reviewer_b_blindness_guess_missing:{case_id}:{arm}"))?;
            blindness_audit.push(json!({
                "case_id":case_id,
                "condition_alias":condition_alias,
                "actual_arm":arm,
                "reviewer_a":{"guessed_arm":guess_a.guessed_arm,"correct":guess_a.guessed_arm.as_deref()==Some(arm.as_str()),"confidence":guess_a.confidence,"rationale":guess_a.rationale},
                "reviewer_b":{"guessed_arm":guess_b.guessed_arm,"correct":guess_b.guessed_arm.as_deref()==Some(arm.as_str()),"confidence":guess_b.confidence,"rationale":guess_b.rationale}
            }));
            let output_row = results
                .get(&(case_id.to_owned(), arm.clone()))
                .ok_or("arm_result_missing")?;
            let selected: BTreeSet<String> = as_set(output_row, "selected_ids")?;
            let expected_recalled: BTreeSet<String> =
                selected.intersection(&expected).cloned().collect();
            let unexpected: BTreeSet<String> = selected.difference(&roster).cloned().collect();
            let non_expected_roster: BTreeSet<String> = selected
                .intersection(&roster)
                .filter(|id| !expected.contains(*id))
                .cloned()
                .collect();
            let mut category_counts: BTreeMap<String, usize> = BTreeMap::new();
            let mut blind_useful = 0;
            let mut ambiguous = 0;
            let mut adjudicated_disagreements = 0;
            let mut selected_detail = Vec::new();
            for (candidate_alias, memory_id) in &private_candidate_by_alias {
                if !selected.contains(memory_id) {
                    continue;
                }
                let key = (
                    case_alias.to_owned(),
                    condition_alias.clone(),
                    candidate_alias.clone(),
                );
                required_judgment_keys.insert(key.clone());
                let a = reviewer_a.get(&key).ok_or_else(|| {
                    format!("reviewer_a_missing:{}:{}:{}", case_id, arm, candidate_alias)
                })?;
                let b = reviewer_b.get(&key).ok_or_else(|| {
                    format!("reviewer_b_missing:{}:{}:{}", case_id, arm, candidate_alias)
                })?;
                let summary = search["response"]["candidates"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|c| c["memory_id"].as_str() == Some(memory_id.as_str()))
                    .and_then(|c| c["summary"].as_str())
                    .ok_or("candidate_summary_missing")?;
                if !summary.contains(&a.evidence_quote) || !summary.contains(&b.evidence_quote) {
                    return Err(format!(
                        "review_quote_not_literal:{}:{}:{}",
                        case_id, arm, candidate_alias
                    ));
                }
                let disagreement = a.relation != b.relation
                    || a.evidence_quote != b.evidence_quote
                    || a.evidence_valid != b.evidence_valid
                    || a.also_refutes_premise != b.also_refutes_premise;
                let final_judgment = if disagreement {
                    required_adjudication_keys.insert(key.clone());
                    adjudications.get(&key).ok_or_else(|| {
                        format!(
                            "adjudication_required:{}:{}:{}",
                            case_id, arm, candidate_alias
                        )
                    })?
                } else {
                    a
                };
                let literal_quote_valid = summary.contains(&final_judgment.evidence_quote);
                let effective_relation = if !final_judgment.evidence_valid || !literal_quote_valid {
                    "ambiguous"
                } else {
                    final_judgment.relation.as_str()
                };
                *category_counts
                    .entry(effective_relation.to_owned())
                    .or_default() += 1;
                if ["answers", "partial", "explicit_unmeasured", "refutes"]
                    .contains(&effective_relation)
                {
                    blind_useful += 1;
                }
                if effective_relation == "ambiguous" {
                    ambiguous += 1;
                }
                if disagreement {
                    adjudicated_disagreements += 1;
                }
                selected_detail.push(json!({
                    "candidate_id":memory_id,
                    "reviewer_a":judgment_json(a),
                    "reviewer_b":judgment_json(b),
                    "agreement_before_adjudication":!disagreement,
                    "adjudication":if disagreement {Some(judgment_json(final_judgment))} else {None},
                    "final_relation":effective_relation,
                    "literal_quote_valid":literal_quote_valid,
                    "disagreement_adjudicated":disagreement
                }));
            }
            let outcome = derive_outcome(output_row, case_id, &arm)?;
            let expected_recall = if expected.is_empty() {
                None
            } else {
                Some(expected_recalled.len() as f64 / expected.len() as f64)
            };
            let adjudicated_selection_count = selected_detail.len();
            let blind_useful_selection_rate = if adjudicated_selection_count == 0 {
                Value::Null
            } else {
                json!(blind_useful as f64 / adjudicated_selection_count as f64)
            };
            let row_report = json!({
                "case_id":case_id,"arm":arm,"outcome":outcome,
                "runtime_status":output_row["runtime_status"],
                "failure_class":output_row["failure_class"],
                "error_code":output_row["error_code"],
                "provider_http_status":output_row["provider_http_status"],
                "expected_ids":expected,"selected_ids":selected,
                "expected_id_recall":expected_recall,
                "exact_set_match":selected==expected,
                "unexpected_selected_ids":unexpected,
                "non_expected_roster_ids":non_expected_roster,
                "blind_useful_selected":blind_useful,
                "adjudicated_selection_count":adjudicated_selection_count,
                "blind_useful_selection_rate":blind_useful_selection_rate,
                "ambiguous_selected":ambiguous,
                "adjudicated_disagreement_count":adjudicated_disagreements,
                "blind_category_counts":category_counts,
                "selected_judgments":selected_detail,
                "elapsed_classifier_ms":output_row["elapsed_classifier_ms"],
                "elapsed_service_ms":output_row["elapsed_service_ms"],
                "input_tokens":output_row["input_tokens"],"output_tokens":output_row["output_tokens"],
                "reported_cost_usd":output_row["reported_cost_usd"],"receipt_sha256":output_row["receipt_sha256"]
            });
            per_arm.get_mut(&arm).unwrap().push(row_report.clone());
            case_reports.push(row_report);
        }
    }
    if reviewer_a.keys().cloned().collect::<BTreeSet<_>>() != required_judgment_keys
        || reviewer_b.keys().cloned().collect::<BTreeSet<_>>() != required_judgment_keys
    {
        return Err("reviewer_judgment_coverage_mismatch".into());
    }
    if adjudications.keys().cloned().collect::<BTreeSet<_>>() != required_adjudication_keys {
        return Err("adjudication_coverage_mismatch".into());
    }
    if guesses_a.keys().cloned().collect::<BTreeSet<_>>() != required_guess_keys
        || guesses_b.keys().cloned().collect::<BTreeSet<_>>() != required_guess_keys
    {
        return Err("blindness_guess_coverage_mismatch".into());
    }
    let mut arm_summary = serde_json::Map::new();
    for arm in ARMS {
        let rows = &per_arm[arm];
        let exact = rows.iter().filter(|r| r["exact_set_match"] == true).count();
        let all_expected: usize = rows
            .iter()
            .map(|r| r["expected_ids"].as_array().map_or(0, Vec::len))
            .sum();
        let recovered: usize = rows
            .iter()
            .map(|r| {
                r["selected_ids"].as_array().map_or(0, |xs| {
                    xs.iter()
                        .filter(|x| r["expected_ids"].as_array().unwrap().contains(x))
                        .count()
                })
            })
            .sum();
        let lat_classifier: Vec<f64> = rows
            .iter()
            .filter_map(|r| r["elapsed_classifier_ms"].as_f64())
            .collect();
        let lat_service: Vec<f64> = rows
            .iter()
            .filter_map(|r| r["elapsed_service_ms"].as_f64())
            .collect();
        let known_cost: f64 = rows
            .iter()
            .filter_map(|r| r["reported_cost_usd"].as_f64())
            .sum();
        let unknown_costs = rows
            .iter()
            .filter(|r| r["reported_cost_usd"].is_null())
            .count();
        let adjudicated_selection_count: usize = rows
            .iter()
            .map(|r| r["adjudicated_selection_count"].as_u64().unwrap_or(0) as usize)
            .sum();
        let blind_useful_selected: u64 = rows
            .iter()
            .map(|r| r["blind_useful_selected"].as_u64().unwrap_or(0))
            .sum();
        let blind_useful_selection_rate = if adjudicated_selection_count == 0 {
            Value::Null
        } else {
            json!(blind_useful_selected as f64 / adjudicated_selection_count as f64)
        };
        arm_summary.insert(arm.into(), json!({
            "n_cases":rows.len(),"exact_set_match_count":exact,"exact_set_match_denominator":rows.len(),
            "expected_id_recall":{"numerator":recovered,"denominator":all_expected,"value":if all_expected==0 {Value::Null} else {json!(recovered as f64/all_expected as f64)}},
            "unexpected_selected_id_count":rows.iter().map(|r|r["unexpected_selected_ids"].as_array().map_or(0,Vec::len)).sum::<usize>(),
            "blind_useful_selected":blind_useful_selected,
            "adjudicated_selection_count":adjudicated_selection_count,
            "blind_useful_selection_rate":{"numerator":blind_useful_selected,"denominator":adjudicated_selection_count,"value":blind_useful_selection_rate},
            "blind_irrelevant_selected":rows.iter().map(|r|r["blind_category_counts"]["irrelevant"].as_u64().unwrap_or(0)).sum::<u64>(),
            "blind_ambiguous_selected":rows.iter().map(|r|r["blind_category_counts"]["ambiguous"].as_u64().unwrap_or(0)).sum::<u64>(),
            "classifier_latency_ms_nearest_rank":p50_p95(&lat_classifier),"service_latency_ms_nearest_rank":p50_p95(&lat_service),
            "known_reported_cost_usd":known_cost,"unknown_cost_operations":unknown_costs,
            "input_tokens_known":rows.iter().filter_map(|r|r["input_tokens"].as_u64()).sum::<u64>(),
            "output_tokens_known":rows.iter().filter_map(|r|r["output_tokens"].as_u64()).sum::<u64>(),
            "error_code_counts":rows.iter().fold(serde_json::Map::new(), |mut acc, r| { let key=r["error_code"].as_str().unwrap_or("none").to_owned(); let count=acc.get(&key).and_then(Value::as_u64).unwrap_or(0)+1; acc.insert(key,json!(count)); acc }),
            "outcome_counts":rows.iter().fold(serde_json::Map::new(), |mut acc, r| { let key=r["outcome"].as_str().unwrap_or("unknown").to_owned(); let count=acc.get(&key).and_then(Value::as_u64).unwrap_or(0)+1; acc.insert(key,json!(count)); acc })
        }));
    }
    let pairs = pairwise_deltas(&per_arm)?;
    let report = json!({
        "version":"revision-matrix-v1-blind-score-v1",
        "case_count":case_count,
        "review_manifest_sha256":review_manifest_sha256,
        "review_packet_sha256":review_packet_sha256,
        "blindness_audit":{
            "n_conditions":blindness_audit.len(),
            "reviewer_a_correct_guesses":blindness_audit.iter().filter(|r|r["reviewer_a"]["correct"]==true).count(),
            "reviewer_b_correct_guesses":blindness_audit.iter().filter(|r|r["reviewer_b"]["correct"]==true).count(),
            "chance_reference_for_three_arms":1.0/3.0,
            "guesses":blindness_audit
        },
        "arm_summary":arm_summary,
        "case_results":case_reports,
        "paired_deltas":pairs,
        "frozen_annotations_modified":false,
        "semantic_truth_claimed":false
    });
    let mut out = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| format!("create_{}: {e}", output.display()))?;
    serde_json::to_writer_pretty(&mut out, &report).map_err(|e| format!("write_score: {e}"))?;
    out.write_all(b"\n")
        .map_err(|e| format!("write_score: {e}"))?;
    out.sync_all().map_err(|e| format!("sync_score: {e}"))?;
    Ok(())
}

fn pairwise_deltas(per_arm: &BTreeMap<String, Vec<Value>>) -> Result<Value, String> {
    let mut out = Vec::new();
    for comparator in ["G0", "D0"] {
        let candidate = &per_arm["G2"];
        let base = &per_arm[comparator];
        let base_by: BTreeMap<&str, &Value> = base
            .iter()
            .map(|r| (r["case_id"].as_str().unwrap(), r))
            .collect();
        for row in candidate {
            let id = row["case_id"].as_str().ok_or("case_id_missing")?;
            let before = base_by.get(id).ok_or("paired_case_missing")?;
            let cand_expected: BTreeSet<String> = as_set(row, "selected_ids")?
                .intersection(&as_set(row, "expected_ids")?)
                .cloned()
                .collect();
            let base_expected: BTreeSet<String> = as_set(before, "selected_ids")?
                .intersection(&as_set(before, "expected_ids")?)
                .cloned()
                .collect();
            out.push(json!({
                "case_id":id,"comparison":format!("G2_minus_{comparator}"),
                "gained_expected_ids":cand_expected.difference(&base_expected).collect::<Vec<_>>(),
                "lost_expected_ids":base_expected.difference(&cand_expected).collect::<Vec<_>>(),
                "g2_non_expected_roster_ids":row["non_expected_roster_ids"],
                "comparator_non_expected_roster_ids":before["non_expected_roster_ids"],
                "g2_ids_outside_roster":row["unexpected_selected_ids"],
                "comparator_ids_outside_roster":before["unexpected_selected_ids"],
                "g2_outcome":row["outcome"],"comparator_outcome":before["outcome"],
                "g2_blind_categories":row["blind_category_counts"],"comparator_blind_categories":before["blind_category_counts"]
            }));
        }
    }
    Ok(Value::Array(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn put(path: &Path, text: &str) {
        fs::write(path, text).unwrap();
    }

    fn jsonl(rows: &[Value]) -> String {
        rows.iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .join("\n")
            + "\n"
    }

    fn fixture_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("iromem-revision-matrix-test-{nonce}"))
    }

    #[test]
    fn blind_packet_rejects_direct_arm_metadata_and_gold_fields() {
        let sensitive = BTreeSet::from(["gemma3_4b".to_owned(), "revision-01".to_owned()]);
        assert!(
            scan_for_sensitive_fields(
                &json!({"query":"What is Q?","conditions":[{"selected_candidate_aliases":["M1"]}]}),
                &sensitive
            )
            .is_ok()
        );
        for field in [
            "arm",
            "provider",
            "selection_policy",
            "expected_ids",
            "reported_cost_usd",
            "cost_usd",
            "error_code",
        ] {
            assert!(
                scan_for_sensitive_fields(&json!({field:"hidden"}), &sensitive).is_err(),
                "{field}"
            );
        }
        assert!(
            scan_for_sensitive_fields(
                &json!({"query":"Use gemma3_4b for revision-01"}),
                &sensitive
            )
            .is_err()
        );
        assert!(scan_for_sensitive_fields(&json!({"query":"mygemma3_4b"}), &sensitive).is_ok());
    }

    #[test]
    fn nearest_rank_percentiles_are_explicit_and_small_sample_safe() {
        assert_eq!(p50_p95(&[]), json!({"n":0,"p50":null,"p95":null}));
        assert_eq!(
            p50_p95(&[1.0, 2.0, 3.0, 4.0]),
            json!({"n":4,"p50":2.0,"p95":4.0})
        );
    }

    #[test]
    fn private_aliases_are_unique_enough_for_one_case() {
        let aliases: BTreeSet<String> = (0..100).map(|_| random_alias("case").unwrap()).collect();
        assert_eq!(aliases.len(), 100);
        assert!(
            aliases
                .iter()
                .all(|s| s.starts_with("case-") && s.len() == 25)
        );
    }

    #[test]
    fn blind_packet_and_score_join_are_end_to_end_and_fail_closed() {
        let root = fixture_root();
        let preflight = root.join("preflight");
        fs::create_dir_all(preflight.join("requests")).unwrap();
        let mut requests = Vec::new();
        let mut search = Vec::new();
        let mut annotations = Vec::new();
        let mut result_rows = Vec::new();
        let corpus_sha256;
        let search_results_sha256;
        let case_count = 64;
        for index in 1..=case_count {
            let case_id = format!("revision-{index:02}");
            let memory_id = format!("memory-{index:02}");
            let query = json!({
                "query": format!("What is the final date for project P-{index}?"),
                "scope_kind":"project","scope_id":format!("scope-{index}"),
                "entity_id":null,"as_of":"2026-10-23T12:00:00.000Z","limit":20,"query_mode":"keywords"
            });
            let mut request_hashes = BTreeMap::new();
            for arm in ARMS {
                let provider = if arm == "D0" {
                    "decisions"
                } else {
                    "gemma3_4b"
                };
                let policy = if arm == "G2" {
                    "slot_evidence_v2"
                } else {
                    "baseline"
                };
                let request =
                    json!({"search":query.clone(),"provider":provider,"selection_policy":policy});
                let path = format!("requests/{case_id}-{arm}.json");
                let bytes = serde_json::to_vec_pretty(&request).unwrap();
                fs::write(preflight.join(&path), &bytes).unwrap();
                let hash = sha256(&bytes);
                request_hashes.insert(arm.to_owned(), hash.clone());
                requests.push(json!({"path":path,"sha256":hash,"provider":provider,"selection_policy":policy}));
                result_rows.push(json!({
                    "case_id":case_id.clone(),"arm":arm,"runtime_status":"complete","failure_class":"none","selected_ids":[memory_id.clone()],
                    "corpus_sha256":"0".repeat(64),"search_results_sha256":"0".repeat(64),"candidate_roster_sha256":"0".repeat(64),
                    "request_sha256":hash,"binary_sha256":"a".repeat(64),"harness_sha256":"b".repeat(64),
                    "prompt_sha256":"c".repeat(64),"schema_sha256":"d".repeat(64),"response_sha256":"e".repeat(64),
                    "receipt_sha256":"f".repeat(64),"requested_model":"test/model","served_model":"test/model",
                    "error_code":null,"provider_http_status":null,
                    "elapsed_classifier_ms":2.0,"elapsed_service_ms":3.0,"input_tokens":12,"output_tokens":4,"reported_cost_usd":0.0001
                }));
            }
            search.push(json!({
                "id":case_id.clone(),"expected_ids":[memory_id.clone()],"response":{"candidates":[{"memory_id":memory_id.clone(),"revision":1,"summary":format!("For project P-{index}, the final date is 19 May.")} ]}
            }));
            annotations.push(json!({"id":case_id,"relation":"answers","expected_selected":[memory_id],"answer_boundary":"19 May"}));
            assert_eq!(request_hashes.len(), 3);
        }
        put(&preflight.join("corpus.json"), "{}\n");
        put(&preflight.join("search-cases.jsonl"), "{}\n");
        put(&preflight.join("annotations.jsonl"), &jsonl(&annotations));
        put(&preflight.join("search-results.jsonl"), &jsonl(&search));
        corpus_sha256 = hash_file(&preflight.join("corpus.json")).unwrap();
        search_results_sha256 = hash_file(&preflight.join("search-results.jsonl")).unwrap();
        for row in &mut result_rows {
            let case = row["case_id"].as_str().unwrap();
            let index: usize = case.strip_prefix("revision-").unwrap().parse().unwrap();
            let candidate = Candidate {
                memory_id: format!("memory-{index:02}"),
                summary: format!("For project P-{index}, the final date is 19 May."),
            };
            row["corpus_sha256"] = json!(corpus_sha256);
            row["search_results_sha256"] = json!(search_results_sha256);
            row["candidate_roster_sha256"] = json!(candidate_roster_sha256(&[candidate]).unwrap());
        }
        let mut inputs = serde_json::Map::new();
        for name in [
            "corpus.json",
            "search-cases.jsonl",
            "annotations.jsonl",
            "search-results.jsonl",
        ] {
            inputs.insert(
                name.to_owned(),
                json!(hash_file(&preflight.join(name)).unwrap()),
            );
        }
        put(
            &preflight.join("manifest.json"),
            &serde_json::to_string(&json!({"inputs":inputs,"requests":requests})).unwrap(),
        );
        let arm_results = root.join("arm-results.jsonl");
        put(&arm_results, &jsonl(&result_rows));
        let mut mismatched_model = result_rows.clone();
        mismatched_model
            .iter_mut()
            .find(|row| row["case_id"] == "revision-01" && row["arm"] == "G2")
            .unwrap()["requested_model"] = json!("other/model");
        let mismatched_model_path = root.join("mismatched-model.jsonl");
        put(&mismatched_model_path, &jsonl(&mismatched_model));
        assert!(
            prepare_blind_review(
                &preflight,
                &mismatched_model_path,
                &root.join("rejected-model-output")
            )
            .unwrap_err()
            .contains("paired_model_mismatch")
        );
        let mut mismatched_corpus = result_rows.clone();
        mismatched_corpus[0]["corpus_sha256"] = json!("9".repeat(64));
        let mismatched_corpus_path = root.join("mismatched-corpus.jsonl");
        put(&mismatched_corpus_path, &jsonl(&mismatched_corpus));
        assert!(
            prepare_blind_review(
                &preflight,
                &mismatched_corpus_path,
                &root.join("rejected-corpus-output")
            )
            .unwrap_err()
            .contains("shared_input_hash_mismatch")
        );
        let mut mismatched_roster = result_rows.clone();
        mismatched_roster
            .iter_mut()
            .find(|row| row["case_id"] == "revision-01" && row["arm"] == "D0")
            .unwrap()["candidate_roster_sha256"] = json!("8".repeat(64));
        let mismatched_roster_path = root.join("mismatched-roster.jsonl");
        put(&mismatched_roster_path, &jsonl(&mismatched_roster));
        assert!(
            prepare_blind_review(
                &preflight,
                &mismatched_roster_path,
                &root.join("rejected-roster-output")
            )
            .unwrap_err()
            .contains("candidate_roster_hash_mismatch:revision-01:D0")
        );
        let output = root.join("review-output");
        prepare_blind_review(&preflight, &arm_results, &output).unwrap();
        let packet = fs::read_to_string(output.join("review/packet.jsonl")).unwrap();
        assert!(!packet.contains("gemma3_4b"));
        assert!(!packet.contains("decisions"));
        assert!(!packet.contains("expected_ids"));
        assert!(!packet.contains("memory-01"));
        assert!(packet.contains("M1"));
        let mapping_path = output.join("private/mapping.json");
        assert_eq!(
            fs::metadata(&mapping_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let mapping = read_json(&mapping_path).unwrap();
        let mut reviewer_a_rows = Vec::new();
        let mut reviewer_b_rows = Vec::new();
        let mut reviewer_a_guesses = Vec::new();
        let mut reviewer_b_guesses = Vec::new();
        let mut adjudication_rows = Vec::new();
        for case in mapping["cases"].as_array().unwrap() {
            let case_alias = case["case_alias"].as_str().unwrap();
            let candidate_alias = case["candidate_aliases"]
                .as_object()
                .unwrap()
                .keys()
                .next()
                .unwrap()
                .clone();
            for (condition_alias, private) in case["conditions"].as_object().unwrap() {
                reviewer_a_guesses.push(json!({
                    "record_type":"blindness_guess","case_alias":case_alias,"condition_alias":condition_alias,
                    "guessed_arm":"G0","guess_confidence":"low","guess_rationale":"weak stylistic impression"
                }));
                reviewer_b_guesses.push(json!({
                    "record_type":"blindness_guess","case_alias":case_alias,"condition_alias":condition_alias,
                    "guessed_arm":null,"guess_confidence":"low","guess_rationale":"cannot distinguish"
                }));
                let base_relation = if case["case_id"] == "revision-02" && private["arm"] == "G2" {
                    "irrelevant"
                } else {
                    "answers"
                };
                let mut relation_b = base_relation;
                if case["case_id"] == "revision-01" && private["arm"] == "G2" {
                    relation_b = "partial";
                }
                let base = json!({
                    "case_alias":case_alias,"condition_alias":condition_alias,"candidate_alias":candidate_alias,
                    "relation":base_relation,"evidence_quote":"19 May","evidence_valid":true,
                    "also_refutes_premise":false,"rationale":"literal date"
                });
                reviewer_a_rows.push(base.clone());
                let mut other = base.clone();
                other["relation"] = json!(relation_b);
                reviewer_b_rows.push(other);
                if relation_b != base_relation {
                    let mut resolved = base;
                    resolved["rationale"] = json!("the final date is stated explicitly");
                    adjudication_rows.push(resolved);
                }
            }
        }
        for (judgments, guesses) in [
            (&reviewer_a_rows, &mut reviewer_a_guesses),
            (&reviewer_b_rows, &mut reviewer_b_guesses),
        ] {
            let digest = sha256(&serde_json::to_vec(judgments).unwrap());
            for guess in guesses {
                guess["judgment_records_sha256"] = json!(digest);
            }
        }
        reviewer_a_rows.extend(reviewer_a_guesses);
        reviewer_b_rows.extend(reviewer_b_guesses);
        let reviewer_a = root.join("reviewer-a.jsonl");
        let reviewer_b = root.join("reviewer-b.jsonl");
        let adjudications = root.join("adjudications.jsonl");
        put(&reviewer_a, &jsonl(&reviewer_a_rows));
        put(&reviewer_b, &jsonl(&reviewer_b_rows));
        put(&adjudications, &jsonl(&adjudication_rows));
        let mut changed_after_freeze = reviewer_a_rows.clone();
        changed_after_freeze[0]["rationale"] = json!("edited after the arm guess");
        let changed_after_freeze_path = root.join("reviewer-a-changed-after-freeze.jsonl");
        put(&changed_after_freeze_path, &jsonl(&changed_after_freeze));
        assert!(
            score_blind_review(
                &preflight,
                &arm_results,
                &mapping_path,
                &changed_after_freeze_path,
                &reviewer_b,
                &adjudications,
                &root.join("changed-after-freeze-score.json")
            )
            .unwrap_err()
            .contains("blindness_guess_judgments_hash_mismatch")
        );
        let report_path = root.join("score.json");
        score_blind_review(
            &preflight,
            &arm_results,
            &mapping_path,
            &reviewer_a,
            &reviewer_b,
            &adjudications,
            &report_path,
        )
        .unwrap();
        let report = read_json(&report_path).unwrap();
        assert_eq!(report["case_count"], case_count);
        for arm in ARMS {
            assert_eq!(
                report["arm_summary"][arm]["exact_set_match_count"],
                case_count
            );
            assert_eq!(
                report["arm_summary"][arm]["expected_id_recall"]["value"],
                1.0
            );
        }
        assert_eq!(
            report["arm_summary"]["G0"]["blind_useful_selected"],
            case_count
        );
        assert_eq!(
            report["arm_summary"]["G0"]["adjudicated_selection_count"],
            case_count
        );
        assert_eq!(
            report["arm_summary"]["G0"]["blind_useful_selection_rate"]["value"],
            1.0
        );
        assert_eq!(
            report["arm_summary"]["G2"]["blind_useful_selected"],
            case_count - 1
        );
        assert_eq!(report["arm_summary"]["G2"]["blind_irrelevant_selected"], 1);
        assert_eq!(
            report["arm_summary"]["G2"]["blind_useful_selection_rate"]["numerator"],
            case_count - 1
        );
        assert_eq!(
            report["arm_summary"]["G2"]["blind_useful_selection_rate"]["denominator"],
            case_count
        );
        assert_eq!(
            report["arm_summary"]["G2"]["blind_useful_selection_rate"]["value"],
            (case_count - 1) as f64 / case_count as f64
        );
        assert_eq!(
            report["paired_deltas"].as_array().unwrap().len(),
            case_count * (ARMS.len() - 1)
        );
        assert_eq!(
            report["blindness_audit"]["n_conditions"],
            case_count * ARMS.len()
        );
        assert_eq!(
            report["blindness_audit"]["reviewer_a_correct_guesses"],
            case_count
        );
        assert_eq!(report["blindness_audit"]["reviewer_b_correct_guesses"], 0);
        let adjudicated = report["case_results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["case_id"] == "revision-01" && row["arm"] == "G2")
            .unwrap();
        let detail = adjudicated["selected_judgments"]
            .as_array()
            .unwrap()
            .first()
            .unwrap();
        assert_eq!(detail["reviewer_a"]["rationale"], "literal date");
        assert_eq!(detail["reviewer_b"]["relation"], "partial");
        assert_eq!(
            detail["adjudication"]["rationale"],
            "the final date is stated explicitly"
        );
        let wrong_mapping_results = root.join("changed-results.jsonl");
        let changed = result_rows
            .into_iter()
            .map(|mut r| {
                r["elapsed_service_ms"] = json!(99.0);
                r
            })
            .collect::<Vec<_>>();
        put(&wrong_mapping_results, &jsonl(&changed));
        assert!(
            score_blind_review(
                &preflight,
                &wrong_mapping_results,
                &mapping_path,
                &reviewer_a,
                &reviewer_b,
                &adjudications,
                &root.join("should-not-exist.json")
            )
            .unwrap_err()
            .contains("arm_results_hash_mismatch")
        );
        put(
            &output.join("review/packet.jsonl"),
            &format!("{}\n", packet),
        );
        assert!(
            score_blind_review(
                &preflight,
                &arm_results,
                &mapping_path,
                &reviewer_a,
                &reviewer_b,
                &adjudications,
                &root.join("tampered-packet-score.json")
            )
            .unwrap_err()
            .contains("review_packet_hash_mismatch")
        );
        fs::remove_dir_all(root).unwrap();
    }
}
