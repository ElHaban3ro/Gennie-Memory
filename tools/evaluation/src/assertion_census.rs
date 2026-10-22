//! Audit of automatic assertion projection, including failures and all in-flight charges.
use crate::{
    isolated_provider_probe::{cost, lines, read},
    projection_sets as projection,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::Path,
};
fn percentile(mut v: Vec<f64>, p: f64) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let i = (v.len() - 1) as f64 * p;
    Some(v[i.floor() as usize] + (v[i.ceil() as usize] - v[i.floor() as usize]) * (i - i.floor()))
}
fn canonical_query(q: &Value) -> Option<(String, String)> {
    let mut c = q.clone();
    let sort_refs = |v: &mut Value| -> Option<()> {
        let a = v.as_array_mut()?;
        if a.iter().any(|v| !v.is_string()) {
            return None;
        }
        a.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        Some(())
    };
    sort_refs(&mut c["identity_token_indices"])?;
    let identity = serde_json::to_string(&c["identity_token_indices"]).ok()?;
    for k in ["requested_token_groups", "premise_token_groups"] {
        let groups = c[k].as_array_mut()?;
        for group in groups.iter_mut() {
            sort_refs(group)?;
        }
        groups.sort_by_key(|g| g.to_string());
    }
    Some((identity, serde_json::to_string(&c).ok()?))
}
fn query_variation(requests: &[Value], events: &[Value]) -> Value {
    let lookup = requests
        .iter()
        .filter_map(|r| r["id"].as_str().map(|id| (id, r)))
        .collect::<BTreeMap<_, _>>();
    let mut groups = BTreeMap::<String, (usize, BTreeSet<String>, BTreeSet<String>)>::new();
    for e in events.iter().filter(|e| e["kind"] == "response") {
        let Some(r) = e["id"].as_str().and_then(|id| lookup.get(id)) else {
            continue;
        };
        let Some(text) = e["body"]["choices"][0]["message"]["content"].as_str() else {
            continue;
        };
        let Ok(w) = serde_json::from_str::<Value>(text) else {
            continue;
        };
        if let Some((identity, full)) = canonical_query(&w["query"]) {
            let g = groups
                .entry(r["original_case_id"].as_str().unwrap().into())
                .or_default();
            g.0 += 1;
            g.1.insert(identity);
            g.2.insert(full);
        }
    }
    let changed = groups
        .values()
        .filter(|(n, ids, _)| *n > 1 && ids.len() > 1)
        .count();
    json!({"posthoc":true,"new_provider_calls":0,"queries_with_multiple_candidates_and_varying_identity_membership":changed,"scope":"All parsed query frames, including invalid claim extractions. Ignores inner token order and group order; preserves membership. Variation is a diagnostic, not proof that every syntactic difference changes meaning or a quality gain.","queries":groups.into_iter().map(|(id,(n,identities,full))|json!({"original_case_id":id,"parsed_frames":n,"distinct_identity_sets":identities.len(),"distinct_query_frames_ignoring_order":full.len(),"identity_sets":identities.into_iter().map(|s|serde_json::from_str::<Value>(&s).unwrap()).collect::<Vec<_>>()})).collect::<Vec<_>>()})
}
pub fn report(dir: &Path, output: &Path) -> Result<(), String> {
    let pre = read(&dir.join("preflight.json"))?;
    if pre["prepared_operations"] != 128
        || pre["max_http_requests"] != 128
        || pre["max_concurrent"] != 4
        || pre["requests_sha256"] != crate::campaign::digest(&dir.join("requests.jsonl"))?
    {
        return Err("preflight alterado".into());
    }
    if pre["stop_on_extraction_error"] != false {
        return Err("censo sin política de continuación explícita".into());
    }
    let inputs = pre["inputs_sha256"].as_object().ok_or("fuentes ausentes")?;
    for (p, h) in inputs {
        if crate::campaign::digest(Path::new(p))? != h.as_str().ok_or("hash inválido")? {
            return Err(format!("fuente cambió: {p}"));
        }
    }
    let source = Path::new("outputs/iromem/evaluation/full-prefix-dimension-probe-2026-10-19");
    for p in [
        source.join("requests.jsonl"),
        source.join("preflight.json"),
        source.join("journal.jsonl"),
        source.join("report.json"),
        Path::new("tests/fixtures/assertion-projection-sets-v4.json").to_path_buf(),
        Path::new("docs/evaluation/assertion-projection-census-protocol-2026-10-19.md")
            .to_path_buf(),
        Path::new("tools/evaluation/src/projection_sets.rs").to_path_buf(),
        Path::new("tools/evaluation/src/projection_ids.rs").to_path_buf(),
        Path::new("tests/fixtures/assertion-projection-ids-v3.json").to_path_buf(),
        Path::new("tools/evaluation/src/projection_tokens.rs").to_path_buf(),
        Path::new("tests/fixtures/assertion-projection-tokens-v2.json").to_path_buf(),
    ] {
        if !inputs.contains_key(p.to_str().unwrap()) {
            return Err("fuente obligatoria sin vínculo".into());
        }
    }
    let revalidation = dir.join(format!("source-revalidation-{}.json", std::process::id()));
    crate::full_prefix_probe::report(source, &revalidation)?;
    if read(&revalidation)? != read(&source.join("report.json"))? {
        return Err("fuente no reproducible".into());
    }
    std::fs::remove_file(revalidation).map_err(|e| e.to_string())?;
    let originals = lines(&source.join("requests.jsonl"))?;
    let mut expected = Vec::new();
    for s in originals {
        let mut r = s.clone();
        r["payload"] = projection::payload(
            s["payload"]["state"]["query"]
                .as_str()
                .ok_or("falta consulta")?,
            s["payload"]["state"]["saved_memory"]
                .as_str()
                .ok_or("falta resumen")?,
        );
        expected.push(r);
    }
    let requests = lines(&dir.join("requests.jsonl"))?;
    if requests != expected || requests.len() != 128 {
        return Err("roster, gold o payload distintos".into());
    }
    let events = lines(&dir.join("journal.jsonl"))?;
    if events.len() % 8 != 0 || events.len() > 256 {
        return Err("lote incierto/incompleto: verificar proceso sin reintentar".into());
    }
    let mut rows = Vec::new();
    let mut stopped = false;
    let mut counts = BTreeMap::<String, usize>::new();
    let mut known = 0.;
    let mut unknown = 0;
    let mut controls = 0;
    let mut times = Vec::new();
    let mut invalid = 0;
    let mut unestablished = 0;
    for (batch_no, events) in events.chunks_exact(8).enumerate() {
        if stopped {
            return Err("nuevo lote después de error técnico".into());
        }
        let batch = &requests[batch_no * 4..batch_no * 4 + 4];
        let mut batch_failed = false;
        for i in 0..4 {
            let r = &batch[i];
            let start = &events[i];
            let e = &events[i + 4];
            if start["kind"] != "started"
                || start["id"] != r["id"]
                || e["id"] != r["id"]
                || !matches!(e["kind"].as_str(), Some("response" | "failed"))
            {
                return Err("diario repetido o fuera de roster".into());
            }
            let input: Value = serde_json::from_str(
                r["payload"]["messages"][1]["content"]
                    .as_str()
                    .ok_or("entrada inválida")?,
            )
            .map_err(|e| e.to_string())?;
            batch_failed |= fatal_transport(e);
            let result = if e["kind"] == "response"
                && e["http_status"]
                    .as_u64()
                    .is_some_and(|s| (200..300).contains(&s))
            {
                projection::inspect_response(
                    input["query"].as_str().unwrap(),
                    input["saved_memory"].as_str().unwrap(),
                    &e["body"],
                )
            } else {
                Err("HTTP o transporte fallido".into())
            };
            let (check, error) = match result {
                Ok(v) => (v, None),
                Err(s) => {
                    invalid += 1;
                    (Value::Null, Some(s))
                }
            };
            let selected = check["check"]["selected"].as_bool();
            let label = r["expected_useful"].as_bool().ok_or("gold inválido")?;
            let classification = match selected {
                Some(true) if label => "true_positive",
                Some(true) => "false_positive",
                Some(false) if label => "false_negative",
                Some(false) => "true_negative",
                None => "technical_failure",
            };
            unestablished += check["check"]["unestablished_useful_claim_indices"]
                .as_array()
                .map_or(0, Vec::len);
            let fee = cost(&e["body"]["usage"], "cost")?;
            if let Some(f) = fee {
                known += f;
            } else {
                unknown += 1;
            }
            let elapsed = e["elapsed_ms"]
                .as_f64()
                .filter(|n| n.is_finite() && *n >= 0.)
                .ok_or("tiempo ausente/inválido")?;
            times.push(elapsed);
            if r["is_control"] == true {
                if selected == Some(label) {
                    controls += 1;
                }
            } else {
                *counts.entry(classification.into()).or_default() += 1;
            }
            rows.push(json!({"id":r["id"],"original_case_id":r["original_case_id"],"memory_id":r["memory_id"],"cohort":r["cohort"],"is_control":r["is_control"],"expected_useful":label,"selected":selected,"classification":classification,"extraction":check,"technical_error":error,"served_model":e["body"]["model"],"provider":e["body"]["provider"],"known_cost_usd":fee,"usage":e["body"]["usage"],"elapsed_ms":elapsed}));
        }
        stopped = batch_failed;
    }
    if !known.is_finite() {
        return Err("coste desbordado".into());
    }
    let complete = rows.len() == 128;
    let passes = complete
        && invalid == 0
        && unknown == 0
        && controls == 2
        && counts.get("false_positive").copied().unwrap_or(0) == 0
        && counts.get("false_negative").copied().unwrap_or(0) == 0;
    let result = json!({"version":"assertion-projection-sets-census-v1","operations":rows.len(),"http_requests":rows.len(),"prepared_operations":128,"complete":complete,"technical_stop":stopped,"invalid_extractions":invalid,"unestablished_useful_claims":unestablished,"candidate_counts":counts,"controls_correct":controls,"known_cost_usd":known,"unknown_cost_operations":unknown,"individual_extraction_p50_ms":percentile(times.clone(),0.5),"individual_extraction_p95_ms":percentile(times,0.95),"passes_semantic_screen":passes,"extraction_errors_continue_for_measurement_only":true,"invalid_extractions_on_expected_useful":rows.iter().filter(|r|r["is_control"]==false && r["expected_useful"]==true && r["selected"].is_null()).count(),"observations":rows,"requests_sha256":pre["requests_sha256"],"journal_sha256":crate::campaign::digest(&dir.join("journal.jsonl"))?,"source_audit_reproduced":true,"posthoc_query_frame_variation":query_variation(&requests,&events),"original_gold_changed":false,"fresh_validation":false,"promoted":false,"warning":"Known128pairs including2authorcontrols. Quote/target integrity is checked; same-case and relation remain fallible model judgments. Different model plus representation; no isolated causal claim. Alias not proof of immutable model revision. Diagnostic extraction latency/cost is not efficient recall or Codex cost."});
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| e.to_string())?
        .write_all(&serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn fatal_transport(e: &Value) -> bool {
    e["kind"] != "response"
        || e["http_status"]
            .as_u64()
            .is_none_or(|s| !(200..300).contains(&s))
        || !projection::contract()["accepted_served_models"]
            .as_array()
            .unwrap()
            .contains(&e["body"]["model"])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_variation_ignores_order_but_preserves_identity_membership() {
        let a = json!({"identity_token_indices":["q2","q1"],"requested_token_groups":[["q3","q0"],["q4"]],"premise_token_groups":[]});
        let mut b = a.clone();
        b["identity_token_indices"] = json!(["q1", "q2"]);
        b["requested_token_groups"] = json!([["q4"], ["q0", "q3"]]);
        assert_eq!(canonical_query(&a), canonical_query(&b));
        b["identity_token_indices"] = json!(["q1"]);
        assert_ne!(canonical_query(&a), canonical_query(&b));
    }
    #[test]
    fn census_continues_invalid_extraction_but_not_transport_or_model_errors() {
        let ok = json!({"kind":"response","http_status":200,"body":{"model":"openai/gpt-4.1-mini","choices":[]}});
        assert!(!fatal_transport(&ok));
        let mut bad = ok.clone();
        bad["http_status"] = json!(500);
        assert!(fatal_transport(&bad));
        bad = ok;
        bad["body"]["model"] = json!("other");
        assert!(fatal_transport(&bad));
        assert!(fatal_transport(&json!({"kind":"failed"})));
    }
}
