//! Resumable synthetic recall campaign driven by the Rust CLI session protocol.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

pub(crate) fn digest(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn read_lines(path: &Path) -> Result<Vec<Value>, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    BufReader::new(file)
        .lines()
        .enumerate()
        .filter_map(|(index, line)| match line {
            Ok(line) if line.trim().is_empty() => None,
            other => Some((index, other)),
        })
        .map(|(index, line)| {
            let line = line.map_err(|error| error.to_string())?;
            serde_json::from_str(&line).map_err(|error| format!("línea {}: {error}", index + 1))
        })
        .collect()
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("falta {key}"))
}

fn ids(value: &Value, key: &str) -> Result<Vec<String>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("falta {key}"))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{key} debe contener IDs"))
        })
        .collect()
}

fn load_cases(path: &Path, split: &str) -> Result<Vec<Value>, String> {
    let cases = read_lines(path)?;
    if cases.is_empty() {
        return Err("el banco no contiene casos".into());
    }
    let mut seen = HashSet::new();
    let mut label_source: Option<&str> = None;
    for case in &cases {
        let id = field(case, "id")?;
        if !seen.insert(id) {
            return Err(format!("ID repetido: {id}"));
        }
        let source = field(case, "label_source")?;
        if field(case, "split")? != split
            || ![
                "deterministic_fixture_rule",
                "agent_synthetic_unreviewed",
                "blind_synthetic_adjudication",
            ]
            .contains(&source)
            || label_source.is_some_and(|previous| previous != source)
        {
            return Err(format!("etiqueta de banco incompatible: {id}"));
        }
        label_source = Some(source);
        ids(case, "expected_ids")?;
        ids(case, "forbidden_ids")?;
        let request = case
            .get("request")
            .ok_or_else(|| format!("falta request en {id}"))?;
        let search = request.get("search").unwrap_or(request);
        for key in ["query", "scope_kind", "scope_id", "as_of"] {
            field(search, key)?;
        }
    }
    Ok(cases)
}

fn append(file: &mut File, event: &Value) -> Result<(), String> {
    serde_json::to_writer(&mut *file, event).map_err(|error| error.to_string())?;
    file.write_all(b"\n").map_err(|error| error.to_string())?;
    file.sync_data().map_err(|error| error.to_string())
}

fn read_journal(
    journal: &Path,
    manifest: &Value,
    cases: &[Value],
) -> Result<HashSet<String>, String> {
    let rows = read_lines(journal)?;
    if rows.first() != Some(manifest) {
        return Err("el diario no coincide con banco, binario, base, proveedor o partición".into());
    }
    let valid: HashMap<&str, &Value> = cases
        .iter()
        .map(|case| Ok((field(case, "id")?, case)))
        .collect::<Result<_, String>>()?;
    let mut started = HashSet::new();
    let mut finished = HashSet::new();
    for event in rows.iter().skip(1) {
        let id = field(event, "id")?;
        let case = valid
            .get(id)
            .ok_or_else(|| format!("evento de caso ajeno: {id}"))?;
        match field(event, "kind")? {
            "started" if started.insert(id.to_owned()) && !finished.contains(id) => {}
            "finished" if started.contains(id) && finished.insert(id.to_owned()) => {
                if event.get("segment") != case.get("segment")
                    || event.get("expected_ids") != case.get("expected_ids")
                    || event.get("forbidden_ids") != case.get("forbidden_ids")
                    || event.get("response").is_none()
                {
                    return Err(format!("etiquetas o respuesta distintas en {id}"));
                }
            }
            _ => return Err(format!("secuencia de eventos inválida: {id}")),
        }
    }
    if let Some(id) = started.difference(&finished).next() {
        return Err(format!(
            "llamada de resultado incierto: {id}; no se reintentará automáticamente"
        ));
    }
    Ok(finished)
}

fn case_policy<'a>(case: &'a Value, policy: &'a str) -> Result<&'a str, String> {
    if policy != "per_case"
        && case
            .pointer("/request/selection_policy")
            .and_then(Value::as_str)
            .is_some_and(|declared| declared != policy)
    {
        return Err("política global contradice caso explícito; use per_case".into());
    }
    let policy = if policy == "per_case" {
        case.pointer("/request/selection_policy")
            .and_then(Value::as_str)
            .ok_or("falta política explícita por caso")?
    } else {
        policy
    };
    if ![
        "baseline",
        "evidence_v1",
        "support_gate_v1",
        "utility_v1",
        "utility_v2",
        "witness_v1",
        "slot_evidence_v1",
        "slot_evidence_v2",
        "slot_evidence_v3",
        "roles_v1",
        "roles_fallback_v1",
        "roles_v2",
        "roles_consensus_v1",
        "roles_consensus_batch_v1",
        "roles_consensus_shared_v1",
        "roles_consensus_compact_v1",
        "roles_consensus_mirrored_v1",
        "roles_primary_veto_v1",
        "epistemic_partial_single_pass_v1",
        "per_memory_v1",
        "per_memory_v2",
        "per_memory_v3",
        "per_memory_v4",
        "per_memory_binary_v1",
        "per_memory_binary_v2",
        "per_memory_binary_v3",
        "per_memory_binary_v4",
        "per_memory_binary_v5",
        "per_memory_examples_v1",
        "per_memory_factors_v1",
        "per_memory_scoped_v1",
        "per_memory_scoped_v2",
        "per_memory_scoped_v3",
        "per_memory_scoped_anchor_v1",
        "per_memory_scored_v1",
        "per_memory_verified_v1",
        "per_memory_correction_verified_v1",
        "per_memory_correction_verified_v2",
        "per_memory_correction_verified_v3",
        "per_memory_correction_verified_v4",
        "per_memory_correction_verified_v5",
        "per_memory_role_verified_v1",
        "per_memory_role_verified_v2",
        "per_memory_role_batch_v1",
        "per_memory_complete_premise_v1",
    ]
    .contains(&policy)
    {
        return Err("política de selección inválida".into());
    }
    Ok(policy)
}

fn recall_message(case: &Value, provider: &str, policy: &str) -> Result<Value, String> {
    let request = case.get("request").ok_or("falta request")?;
    let search = request.get("search").unwrap_or(request);
    Ok(json!({
        "op":"recall",
        "request":{
            "search":search,
            "provider":case_provider(case, provider)?,
            "selection_policy":case_policy(case, policy)?,
        }
    }))
}
fn case_provider<'a>(case: &'a Value, provider: &'a str) -> Result<&'a str, String> {
    if provider != "per_case"
        && case
            .pointer("/request/provider")
            .and_then(Value::as_str)
            .is_some_and(|declared| declared != provider)
    {
        return Err("proveedor global contradice caso explícito; use per_case".into());
    }
    let provider = if provider == "per_case" {
        case.pointer("/request/provider")
            .and_then(Value::as_str)
            .ok_or("falta proveedor por caso")?
    } else {
        provider
    };
    if !["decisions", "jev", "jev114", "gemma3_4b"].contains(&provider) {
        return Err("proveedor por caso inválido".into());
    }
    Ok(provider)
}

pub fn run(
    cases_path: &Path,
    binary: &Path,
    database: &Path,
    journal: &Path,
    provider: &str,
    split: &str,
    max_calls: usize,
    selection_policy: &str,
) -> Result<(), String> {
    if !["decisions", "jev", "jev114", "gemma3_4b", "per_case"].contains(&provider)
        || ![
            "baseline",
            "evidence_v1",
            "support_gate_v1",
            "utility_v1",
            "utility_v2",
            "witness_v1",
            "slot_evidence_v1",
            "slot_evidence_v2",
            "slot_evidence_v3",
            "roles_v1",
            "roles_fallback_v1",
            "roles_v2",
            "roles_consensus_v1",
            "roles_consensus_batch_v1",
            "roles_consensus_shared_v1",
            "roles_consensus_compact_v1",
            "roles_consensus_mirrored_v1",
            "roles_primary_veto_v1",
            "epistemic_partial_single_pass_v1",
            "per_memory_v1",
            "per_memory_v2",
            "per_memory_v3",
            "per_memory_v4",
            "per_memory_binary_v1",
            "per_memory_binary_v2",
            "per_memory_binary_v3",
            "per_memory_binary_v4",
            "per_memory_binary_v5",
            "per_memory_examples_v1",
            "per_memory_factors_v1",
            "per_memory_scoped_v1",
            "per_memory_scoped_v2",
            "per_memory_scoped_v3",
            "per_memory_scoped_anchor_v1",
            "per_memory_scored_v1",
            "per_memory_verified_v1",
            "per_memory_correction_verified_v1",
            "per_memory_correction_verified_v2",
            "per_memory_correction_verified_v3",
            "per_memory_correction_verified_v4",
            "per_memory_correction_verified_v5",
            "per_memory_role_verified_v1",
            "per_memory_role_verified_v2",
            "per_memory_role_batch_v1",
            "per_memory_complete_premise_v1",
            "per_case",
        ]
        .contains(&selection_policy)
        || max_calls == 0
    {
        return Err("proveedor, política o máximo de llamadas inválido".into());
    }
    let cases = load_cases(cases_path, split)?;
    for case in &cases {
        case_policy(case, selection_policy)?;
        case_provider(case, provider)?;
    }
    let manifest = json!({
        "kind": "manifest", "format": 1, "cases_sha256": digest(cases_path)?,
        "binary_sha256": digest(binary)?, "database_sha256": digest(database)?,
        "case_count": cases.len(), "provider": provider, "selection_policy":selection_policy, "split": split,
        "label_source": field(&cases[0], "label_source")?,
    });
    let finished = if journal.exists() {
        read_journal(journal, &manifest, &cases)?
    } else {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(journal)
            .map_err(|error| error.to_string())?;
        append(&mut file, &manifest)?;
        HashSet::new()
    };
    let mut journal_file = OpenOptions::new()
        .append(true)
        .open(journal)
        .map_err(|error| error.to_string())?;
    let mut child = Command::new(binary)
        .arg("session")
        .env("IROMEM_DATABASE_PATH", database)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("no se pudo iniciar iromem: {error}"))?;
    let mut input = child.stdin.take().ok_or("falta entrada de sesión")?;
    let mut output = BufReader::new(child.stdout.take().ok_or("falta salida de sesión")?);
    let result = (|| {
        let mut attempted = 0;
        for case in &cases {
            let id = field(case, "id")?;
            if finished.contains(id) {
                continue;
            }
            if attempted == max_calls {
                break;
            }
            let message = recall_message(case, provider, selection_policy)?;
            append(&mut journal_file, &json!({"kind":"started","id":id}))?;
            let started = Instant::now();
            serde_json::to_writer(&mut input, &message).map_err(|error| error.to_string())?;
            input.write_all(b"\n").map_err(|error| error.to_string())?;
            input.flush().map_err(|error| error.to_string())?;
            let mut line = String::new();
            if output
                .read_line(&mut line)
                .map_err(|error| error.to_string())?
                == 0
            {
                return Err(format!(
                    "la sesión terminó durante {id}; la llamada queda incierta"
                ));
            }
            let envelope: Value = serde_json::from_str(&line).map_err(|_| {
                format!("respuesta inválida durante {id}; la llamada queda incierta")
            })?;
            if field(&envelope, "status")? != "complete" {
                return Err(format!(
                    "la sesión falló durante {id}; la llamada queda incierta"
                ));
            }
            let response = envelope.get("response").ok_or("falta response")?;
            append(
                &mut journal_file,
                &json!({
                    "kind":"finished", "id":id, "segment":field(case,"segment")?,
                    "expected_ids":ids(case,"expected_ids")?,
                    "forbidden_ids":ids(case,"forbidden_ids")?,
                    "elapsed_ms":started.elapsed().as_secs_f64()*1000.0,
                    "response":response,
                }),
            )?;
            attempted += 1;
        }
        Ok::<usize, String>(attempted)
    })();
    drop(input);
    if result.is_err() {
        let _ = child.kill();
    }
    let exit = child.wait().map_err(|error| error.to_string())?;
    let attempted = result?;
    if !exit.success() {
        return Err(format!("la sesión terminó con estado {exit}"));
    }
    println!(
        "{} llamadas nuevas; diario {}",
        attempted,
        journal.display()
    );
    Ok(())
}

pub fn report(journal: &Path, output: Option<&Path>) -> Result<(), String> {
    let rows = read_lines(journal)?;
    let manifest = rows.first().ok_or("diario vacío")?;
    if field(manifest, "kind")? != "manifest" {
        return Err("falta manifiesto".into());
    }
    let count = manifest
        .get("case_count")
        .and_then(Value::as_u64)
        .ok_or("falta case_count")?;
    let mut finished = HashMap::new();
    let mut started = HashSet::new();
    for row in rows.iter().skip(1) {
        let id = field(row, "id")?;
        match field(row, "kind")? {
            "started" if started.insert(id.to_owned()) => {}
            "finished" if started.contains(id) && finished.insert(id.to_owned(), row).is_none() => {
            }
            _ => return Err(format!("diario inválido en {id}")),
        }
    }
    if finished.len() as u64 != count || started.len() != finished.len() {
        return Err(format!(
            "campaña incompleta: {} de {count} casos terminados",
            finished.len()
        ));
    }
    let mut groups: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut correct = 0;
    let mut positive = 0;
    let mut positive_correct = 0;
    let mut negative = 0;
    let mut negative_empty = 0;
    let mut degraded = 0;
    let mut forbidden = 0;
    let mut candidate_recall = 0;
    let mut negative_candidate_empty = 0;
    let mut elapsed = Vec::new();
    let mut search = Vec::new();
    let mut classifier = Vec::new();
    let mut service_total = Vec::new();
    let mut cost = 0.0;
    let mut cost_reported_cases = 0;
    let mut input_tokens = Vec::new();
    let mut output_tokens = Vec::new();
    for row in finished.values() {
        let expected = ids(row, "expected_ids")?;
        let forbidden_ids = ids(row, "forbidden_ids")?;
        let response = row.get("response").ok_or("falta response")?;
        let memories = response
            .pointer("/seed/memories")
            .and_then(Value::as_array)
            .ok_or("falta seed.memories")?;
        let selected: Vec<String> = memories
            .iter()
            .map(|item| field(item, "memory_id").map(str::to_owned))
            .collect::<Result<_, _>>()?;
        let candidates = response
            .pointer("/retrieval/candidates")
            .and_then(Value::as_array)
            .ok_or("faltan candidatos")?;
        let candidate_ids: Vec<String> = candidates
            .iter()
            .map(|item| field(item, "memory_id").map(str::to_owned))
            .collect::<Result<_, _>>()?;
        let is_correct = response.get("status").and_then(Value::as_str) == Some("complete")
            && selected == expected;
        correct += usize::from(is_correct);
        if expected.is_empty() {
            negative += 1;
            negative_empty += usize::from(is_correct);
            negative_candidate_empty += usize::from(candidate_ids.is_empty());
        } else {
            positive += 1;
            positive_correct += usize::from(is_correct);
            candidate_recall += usize::from(expected.iter().any(|id| candidate_ids.contains(id)));
        }
        degraded += usize::from(response.get("status").and_then(Value::as_str) == Some("degraded"));
        forbidden += usize::from(
            forbidden_ids
                .iter()
                .any(|id| candidate_ids.contains(id) || selected.contains(id)),
        );
        let group = groups.entry(field(row, "segment")?.to_owned()).or_default();
        group.0 += 1;
        group.1 += usize::from(is_correct);
        elapsed.push(
            row.get("elapsed_ms")
                .and_then(Value::as_f64)
                .ok_or("falta elapsed_ms")?,
        );
        search.push(
            response
                .pointer("/timings/search_ms")
                .and_then(Value::as_f64)
                .ok_or("falta search_ms")?,
        );
        classifier.push(
            response
                .pointer("/timings/classifier_ms")
                .and_then(Value::as_f64)
                .ok_or("falta classifier_ms")?,
        );
        service_total.push(
            response
                .pointer("/timings/total_ms")
                .and_then(Value::as_f64)
                .ok_or("falta total_ms")?,
        );
        if let Some(value) = response.get("reported_cost_usd").and_then(Value::as_f64) {
            cost += value;
            cost_reported_cases += 1;
        }
        if let Some(value) = response.get("input_tokens").and_then(Value::as_u64) {
            input_tokens.push(value);
        }
        if let Some(value) = response.get("output_tokens").and_then(Value::as_u64) {
            output_tokens.push(value);
        }
    }
    let quantile = |values: &mut Vec<f64>, fraction: f64| {
        values.sort_by(f64::total_cmp);
        let n = values.len();
        let position = (n - 1) as f64 * fraction;
        let lower = position.floor() as usize;
        let upper = position.ceil() as usize;
        values[lower] + (values[upper] - values[lower]) * position.fract()
    };
    let mut by_segment = serde_json::Map::new();
    for (name, (cases, correct)) in groups {
        by_segment.insert(name, json!({"cases":cases, "correct":correct}));
    }
    let mut input_values: Vec<f64> = input_tokens.iter().map(|value| *value as f64).collect();
    let mut output_values: Vec<f64> = output_tokens.iter().map(|value| *value as f64).collect();
    let report = json!({
        "status":"complete", "provider":manifest["provider"], "split":manifest["split"],
        "cases":count, "correct":correct, "positive":positive, "positive_correct":positive_correct,
        "negative":negative, "negative_empty":negative_empty, "degraded":degraded,
        "candidate_recall":candidate_recall, "negative_candidate_empty":negative_candidate_empty,
        "forbidden_leak_count":forbidden, "by_segment":by_segment,
        "elapsed_p50_ms":quantile(&mut elapsed,0.5), "elapsed_p95_ms":quantile(&mut elapsed,0.95),
        "search_p50_ms":quantile(&mut search,0.5), "search_p95_ms":quantile(&mut search,0.95),
        "classifier_p50_ms":quantile(&mut classifier,0.5), "classifier_p95_ms":quantile(&mut classifier,0.95),
        "service_total_p50_ms":quantile(&mut service_total,0.5),
        "service_total_p95_ms":quantile(&mut service_total,0.95),
        "reported_cost_usd":cost, "cost_reported_cases":cost_reported_cases,
        "input_tokens_reported_cases":input_tokens.len(),
        "input_tokens_p50":if input_values.is_empty(){None}else{Some(quantile(&mut input_values,0.5))},
        "output_tokens_reported_cases":output_tokens.len(),
        "output_tokens_p50":if output_values.is_empty(){None}else{Some(quantile(&mut output_values,0.5))},
        "manifest":manifest,
    });
    let encoded = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())? + "\n";
    if let Some(path) = output {
        fs::write(path, encoded).map_err(|error| error.to_string())
    } else {
        print!("{encoded}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_case_policy_is_explicit_and_cannot_recurse() {
        assert_eq!(
            case_provider(&json!({"request":{"provider":"jev114"}}), "per_case").unwrap(),
            "jev114"
        );
        assert!(case_provider(&json!({"request":{"provider":"jev114"}}), "jev").is_err());
        assert_eq!(
            case_provider(&json!({"request":{"provider":"jev"}}), "per_case").unwrap(),
            "jev"
        );
        assert_eq!(
            case_provider(&json!({"request":{"provider":"gemma3_4b"}}), "per_case").unwrap(),
            "gemma3_4b"
        );
        assert!(case_provider(&json!({}), "per_case").is_err());
        assert!(case_provider(&json!({"request":{"provider":"per_case"}}), "per_case").is_err());
        assert_eq!(
            case_policy(
                &json!({"request":{"selection_policy":"utility_v1"}}),
                "per_case"
            )
            .unwrap(),
            "utility_v1"
        );
        assert_eq!(
            case_policy(
                &json!({"request":{"selection_policy":"slot_evidence_v2"}}),
                "per_case"
            )
            .unwrap(),
            "slot_evidence_v2"
        );
        assert!(case_policy(&json!({"request":{}}), "per_case").is_err());
        assert!(
            case_policy(
                &json!({"request":{"selection_policy":"per_case"}}),
                "per_case"
            )
            .is_err()
        );
        assert!(
            case_policy(
                &json!({"request":{"selection_policy":"utility_v1"}}),
                "baseline"
            )
            .is_err()
        );
        assert!(case_provider(&json!({"request":{"provider":"jev"}}), "decisions").is_err());
        assert_eq!(
            case_policy(
                &json!({"request":{"selection_policy":"baseline"}}),
                "baseline"
            )
            .unwrap(),
            "baseline"
        );
    }

    #[test]
    fn recall_message_always_pins_baseline_and_requested_policy() {
        let case = json!({"request":{"search":{"query":"q","scope_kind":"conversation",
            "scope_id":"s","as_of":"2026-10-23T00:00:00.000Z"}}});
        let baseline = recall_message(&case, "decisions", "baseline").unwrap();
        assert_eq!(
            baseline.pointer("/request/selection_policy"),
            Some(&json!("baseline"))
        );
        let single_pass =
            recall_message(&case, "decisions", "epistemic_partial_single_pass_v1").unwrap();
        assert_eq!(
            single_pass.pointer("/request/selection_policy"),
            Some(&json!("epistemic_partial_single_pass_v1"))
        );
        assert_eq!(
            single_pass.pointer("/request/provider"),
            Some(&json!("decisions"))
        );
    }
    #[test]
    fn campaign_preserves_one_declared_label_source() {
        let file = std::env::temp_dir().join(format!(
            "iromem-campaign-label-source-{}",
            std::process::id()
        ));
        let case = |id: &str, source: &str| {
            json!({"id":id,"split":"development","label_source":source,
                "expected_ids":[],"forbidden_ids":[],
                "request":{"search":{"query":"question","scope_kind":"conversation",
                    "scope_id":"scope","as_of":"2026-10-17T12:00:00.000Z"}}})
        };
        fs::write(
            &file,
            format!("{}\n", case("case-1", "agent_synthetic_unreviewed")),
        )
        .unwrap();
        assert_eq!(load_cases(&file, "development").unwrap().len(), 1);
        fs::write(
            &file,
            format!("{}\n", case("case-1", "blind_synthetic_adjudication")),
        )
        .unwrap();
        assert_eq!(load_cases(&file, "development").unwrap().len(), 1);
        fs::write(
            &file,
            format!(
                "{}\n{}\n",
                case("case-1", "agent_synthetic_unreviewed"),
                case("case-2", "deterministic_fixture_rule")
            ),
        )
        .unwrap();
        assert!(load_cases(&file, "development").is_err());
        fs::remove_file(file).unwrap();
    }

    #[test]
    fn journal_rejects_uncertain_call_before_any_retry() {
        let file =
            std::env::temp_dir().join(format!("iromem-campaign-test-{}", std::process::id()));
        let manifest = json!({"kind":"manifest","case_count":1});
        fs::write(
            &file,
            format!(
                "{}\n{}\n",
                manifest,
                json!({"kind":"started","id":"case-1"})
            ),
        )
        .unwrap();
        let cases = vec![json!({"id":"case-1"})];
        let error = read_journal(&file, &manifest, &cases).unwrap_err();
        assert!(error.contains("incierto"));
        fs::remove_file(file).unwrap();
    }
}
