//! Audit of role matching on fixed independently projected sources; shared preparation billed once.
use crate::{
    independent_projection as base, independent_projection_audit as parent_audit,
    independent_projection_roles as role,
    isolated_provider_probe::{cost, lines, read},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, io::Write, path::Path};
fn save(path: &Path, v: &Value) -> Result<(), String> {
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?
        .write_all(&serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn data(parent: &Path, dir: &Path) -> Result<Value, String> {
    let check = dir.join(format!("parent-revalidation-{}.json", std::process::id()));
    parent_audit::report(parent, &check)?;
    let r = read(&check)?;
    if r != read(&parent.join("report.json"))? {
        return Err("padre no reproducible".into());
    }
    std::fs::remove_file(check).map_err(|e| e.to_string())?;
    let original = lines(Path::new(
        "outputs/iromem/evaluation/full-prefix-dimension-probe-2026-10-19/requests.jsonl",
    ))?;
    let plan = base::prepare(&original)?;
    let pp = lines(&parent.join("projection.requests.jsonl"))?;
    let pe = lines(&parent.join("projection.journal.jsonl"))?;
    let (te, fatal) = parent_audit::terminals(&pp, &pe)?;
    if fatal || te.len() != 125 {
        return Err("preparación padre incompleta".into());
    }
    let mut frames = BTreeMap::new();
    let mut projections = Vec::new();
    let mut invalid = 0;
    for (req, e) in pp.iter().zip(&te) {
        let result = role::check_projection(
            req["kind"].as_str().unwrap(),
            req["source_text"].as_str().unwrap(),
            &e["body"],
        );
        let (p, error) = match result {
            Ok(p) => {
                frames.insert(
                    req["id"].as_str().unwrap().to_string(),
                    p["decoded"].clone(),
                );
                (p, None)
            }
            Err(s) => {
                invalid += 1;
                (Value::Null, Some(s))
            }
        };
        projections.push(json!({"id":req["id"],"kind":req["kind"],"projection":p,"error":error}));
    }
    let (requests, skipped) = role::matching_requests(&original, &plan, &frames)?;
    let previous = lines(&parent.join("matching.requests.jsonl"))?;
    let mut shared = 0;
    for old in &previous {
        let new = requests
            .iter()
            .find(|r| r["id"] == old["id"])
            .ok_or("se perdió par compartido")?;
        let mut a: Value =
            serde_json::from_str(old["payload"]["messages"][1]["content"].as_str().unwrap())
                .map_err(|e| e.to_string())?;
        a.as_object_mut().unwrap().remove("allowed_edges");
        let b: Value =
            serde_json::from_str(new["payload"]["messages"][1]["content"].as_str().unwrap())
                .map_err(|e| e.to_string())?;
        if a != b {
            return Err("cambió texto/proyección en par compartido".into());
        }
        shared += 1;
    }
    Ok(
        json!({"requests":requests,"skipped":skipped,"projections":projections,"invalid_projections":invalid,"shared_pairs_preserved":shared,"source_preparation_cost_usd":r["preparation_cost_usd"],"original":original}),
    )
}
fn inputs(parent: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = [
        "preflight.json",
        "plan.json",
        "projection.requests.jsonl",
        "projection.journal.jsonl",
        "matching.preflight.json",
        "matching.requests.jsonl",
        "matching.journal.jsonl",
        "matching.skipped.json",
        "report.json",
    ]
    .into_iter()
    .map(|p| parent.join(p))
    .collect::<Vec<_>>();
    for p in [
        "tools/evaluation/src/independent_projection.rs",
        "tools/evaluation/src/independent_projection_roles.rs",
        "tests/fixtures/independent-projection-v1.json",
        "tests/fixtures/independent-role-matcher-v2.json",
        "docs/evaluation/independent-role-matcher-v2-protocol-2026-10-19.md",
    ] {
        paths.push(Path::new(p).to_path_buf());
    }
    paths
}
pub fn prepare(parent: &Path, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let d = data(parent, dir)?;
    let requests = d["requests"].as_array().unwrap();
    let serialized = requests
        .iter()
        .map(|r| r.to_string() + "\n")
        .collect::<String>();
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("requests.jsonl"))
        .map_err(|e| e.to_string())?
        .write_all(serialized.as_bytes())
        .map_err(|e| e.to_string())?;
    save(&dir.join("projections.json"), &d["projections"])?;
    save(&dir.join("skipped.json"), &d["skipped"])?;
    let mut hashes = serde_json::Map::new();
    for p in inputs(parent) {
        hashes.insert(
            p.to_str().unwrap().into(),
            json!(crate::campaign::digest(&p)?),
        );
    }
    save(
        &dir.join("preflight.json"),
        &json!({"version":"independent-role-matcher-v2","parent_directory":parent,"prepared_matching_operations":requests.len(),"max_http_requests":128,"max_concurrent":4,"inputs_sha256":hashes,"requests_sha256":crate::campaign::digest(&dir.join("requests.jsonl"))?,"projections_sha256":crate::campaign::digest(&dir.join("projections.json"))?,"skipped_sha256":crate::campaign::digest(&dir.join("skipped.json"))?,"shared_pairs_preserved":d["shared_pairs_preserved"],"source_preparation_cost_usd":d["source_preparation_cost_usd"],"fresh_validation":false,"promoted":false}),
    )
}
fn percentile(mut a: Vec<f64>, p: f64) -> Option<f64> {
    if a.is_empty() {
        return None;
    }
    a.sort_by(f64::total_cmp);
    let i = (a.len() - 1) as f64 * p;
    Some(a[i.floor() as usize] + (a[i.ceil() as usize] - a[i.floor() as usize]) * (i - i.floor()))
}
pub fn report(dir: &Path, output: &Path) -> Result<(), String> {
    let pre = read(&dir.join("preflight.json"))?;
    if pre["version"] != "independent-role-matcher-v2"
        || pre["max_http_requests"] != 128
        || pre["max_concurrent"] != 4
    {
        return Err("preflight inválido".into());
    }
    let parent = Path::new(pre["parent_directory"].as_str().ok_or("padre ausente")?);
    let frozen = pre["inputs_sha256"].as_object().ok_or("fuentes ausentes")?;
    for (p, h) in frozen {
        if crate::campaign::digest(Path::new(p))? != h.as_str().ok_or("hash inválido")? {
            return Err(format!("fuente cambió: {p}"));
        }
    }
    for p in inputs(parent) {
        if !frozen.contains_key(p.to_str().unwrap()) {
            return Err("fuente requerida no vinculada".into());
        }
    }
    for (f, k) in [
        ("requests.jsonl", "requests_sha256"),
        ("projections.json", "projections_sha256"),
        ("skipped.json", "skipped_sha256"),
    ] {
        if pre[k] != crate::campaign::digest(&dir.join(f))? {
            return Err("artefacto alterado".into());
        }
    }
    let d = data(parent, dir)?;
    let requests = lines(&dir.join("requests.jsonl"))?;
    if json!(requests) != d["requests"]
        || read(&dir.join("projections.json"))? != d["projections"]
        || read(&dir.join("skipped.json"))? != d["skipped"]
        || pre["prepared_matching_operations"] != requests.len()
        || pre["shared_pairs_preserved"] != d["shared_pairs_preserved"]
        || pre["source_preparation_cost_usd"] != d["source_preparation_cost_usd"]
    {
        return Err("normalización o payloads no reproducibles".into());
    }
    let execution = read(&dir.join("execution-preflight.json"))?;
    if execution["preflight_sha256"] != crate::campaign::digest(&dir.join("preflight.json"))?
        || execution["requests_sha256"] != pre["requests_sha256"]
        || execution["test_binary_sha256"]
            .as_str()
            .is_none_or(|s| s.len() != 64)
    {
        return Err("ejecución no vinculada".into());
    }
    let events = lines(&dir.join("journal.jsonl"))?;
    let (te, fatal) = parent_audit::terminals(&requests, &events)?;
    if te.len() != requests.len() && !fatal {
        return Err("ejecución incompleta; verificar proceso".into());
    }
    let mut actual = BTreeMap::new();
    let mut newcost = 0.;
    let mut missing = 0;
    let mut times = Vec::new();
    let mut invalid = 0;
    for (r, e) in requests.iter().zip(&te) {
        let parsed = if e["kind"] == "response"
            && e["http_status"]
                .as_u64()
                .is_some_and(|s| (200..300).contains(&s))
        {
            role::check_match(&e["body"], &Value::Null, &Value::Null)
        } else {
            Err("HTTP/transporte inválido".into())
        };
        let (check, error) = match parsed {
            Ok(c) => (c, None),
            Err(s) => {
                invalid += 1;
                (Value::Null, Some(s))
            }
        };
        let fee = cost(&e["body"]["usage"], "cost")?;
        if let Some(f) = fee {
            newcost += f;
        } else {
            missing += 1;
        }
        let time = parent_audit::elapsed(e)?;
        times.push(time);
        actual.insert(r["id"].as_str().unwrap().to_string(),json!({"id":r["id"],"selected":check["selected"],"check":check,"error":error,"known_cost_usd":fee,"elapsed_ms":time,"served_model":e["body"]["model"],"usage":e["body"]["usage"]}));
    }
    if !newcost.is_finite() {
        return Err("coste desbordado".into());
    }
    let mut counts = BTreeMap::<String, usize>::new();
    let mut observations = Vec::new();
    let mut controls = 0;
    for r in d["original"].as_array().unwrap() {
        let id = r["id"].as_str().unwrap();
        let a = actual.get(id);
        let selected = a.and_then(|r| r["selected"].as_bool());
        let expected = r["expected_useful"].as_bool().unwrap();
        let skipped = d["skipped"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["id"] == id);
        let class = match selected {
            Some(true) if expected => "true_positive",
            Some(true) => "false_positive",
            Some(false) if expected => "false_negative",
            Some(false) => "true_negative",
            None if skipped => "unavailable_projection",
            None if a.is_some() => "invalid_match",
            None => "not_attempted_after_transport",
        };
        if r["is_control"] == true {
            if selected == Some(expected) {
                controls += 1;
            }
        } else {
            *counts.entry(class.into()).or_default() += 1;
        }
        observations.push(json!({"id":id,"original_case_id":r["original_case_id"],"memory_id":r["memory_id"],"is_control":r["is_control"],"expected_useful":expected,"selected":selected,"classification":class,"match":a}));
    }
    let complete = te.len() == requests.len() && !fatal;
    let passes = complete
        && d["invalid_projections"] == 0
        && d["skipped"].as_array().unwrap().is_empty()
        && requests.len() == 128
        && invalid == 0
        && missing == 0
        && controls == 2
        && counts.get("false_positive").copied().unwrap_or(0) == 0
        && counts.get("false_negative").copied().unwrap_or(0) == 0;
    let result = json!({"version":"independent-role-matcher-v2","new_operations":te.len(),"new_http_requests":te.len(),"new_known_cost_usd":newcost,"unknown_new_cost_operations":missing,"reused_preparation_operations":125,"reused_preparation_cost_usd":d["source_preparation_cost_usd"],"logical_full_candidate_operations":125+te.len(),"logical_full_candidate_cost_usd":newcost+d["source_preparation_cost_usd"].as_f64().unwrap(),"double_count_preparation_in_global_ledger":false,"shared_pairs_preserved":d["shared_pairs_preserved"],"invalid_projections_after_exact_normalization":d["invalid_projections"],"skipped_pairs":d["skipped"].as_array().unwrap().len(),"invalid_matches":invalid,"complete":complete,"fatal_transport":fatal,"candidate_counts":counts,"controls_correct":controls,"individual_match_p50_ms":percentile(times.clone(),0.5),"individual_match_p95_ms":percentile(times,0.95),"passes_semantic_screen":passes,"observations":observations,"journal_sha256":crate::campaign::digest(&dir.join("journal.jsonl"))?,"original_gold_changed":false,"fresh_validation":false,"promoted":false,"warning":"Known128pairs, fixed projections; exact-reference duplicates normalized with raw trace. Matcher interface/prompt changed, not proof of option-count causality. Shared preparation included in logical cost, billed once globally; not recall latency."});
    save(output, &result)
}
