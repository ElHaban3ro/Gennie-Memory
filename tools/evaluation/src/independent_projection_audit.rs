//! Source/transport audit of staged, input-isolated projections; semantic judgments remain fallible.
use crate::{
    independent_projection as projection,
    isolated_provider_probe::{cost, lines, read},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, io::Write, path::Path};
fn fatal(e: &Value) -> bool {
    e["kind"] != "response"
        || e["http_status"]
            .as_u64()
            .is_none_or(|s| !(200..300).contains(&s))
        || !projection::spec()["accepted_served_models"]
            .as_array()
            .unwrap()
            .contains(&e["body"]["model"])
}
pub(crate) fn terminals(
    requests: &[Value],
    events: &[Value],
) -> Result<(Vec<Value>, bool), String> {
    let mut out = Vec::new();
    let mut cursor = 0;
    let mut stopped = false;
    for batch in requests.chunks(4) {
        if cursor == events.len() {
            break;
        }
        if stopped || events.len() - cursor < batch.len() * 2 {
            return Err(
                "lote incierto o posterior a fallo; inspeccionar proceso sin reintentar".into(),
            );
        }
        for (i, r) in batch.iter().enumerate() {
            let a = &events[cursor + i];
            let e = &events[cursor + batch.len() + i];
            if a["kind"] != "started"
                || a["id"] != r["id"]
                || e["id"] != r["id"]
                || !matches!(e["kind"].as_str(), Some("response" | "failed"))
            {
                return Err("diario fuera de roster, repetido o sin terminal".into());
            }
            stopped |= fatal(e);
            out.push(e.clone());
        }
        cursor += batch.len() * 2;
    }
    if cursor != events.len() {
        return Err("eventos excedidos".into());
    }
    Ok((out, stopped))
}
fn percentile(mut v: Vec<f64>, p: f64) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let i = (v.len() - 1) as f64 * p;
    Some(v[i.floor() as usize] + (v[i.ceil() as usize] - v[i.floor() as usize]) * (i - i.floor()))
}
pub(crate) fn fee(e: &Value) -> Result<Option<f64>, String> {
    cost(&e["body"]["usage"], "cost")
}
pub(crate) fn elapsed(e: &Value) -> Result<f64, String> {
    e["elapsed_ms"]
        .as_f64()
        .filter(|n| n.is_finite() && *n >= 0.)
        .ok_or("tiempo inválido".into())
}
pub fn report(dir: &Path, output: &Path) -> Result<(), String> {
    let pre = read(&dir.join("preflight.json"))?;
    if pre["projection_operations_planned"] != 125
        || pre["query_projections"] != 20
        || pre["memory_projections"] != 105
        || pre["matching_pairs"] != 128
        || pre["max_http_requests"] != 253
        || pre["max_concurrent"] != 4
        || pre["plan_sha256"] != crate::campaign::digest(&dir.join("plan.json"))?
        || pre["projection_requests_sha256"]
            != crate::campaign::digest(&dir.join("projection.requests.jsonl"))?
    {
        return Err("preflight alterado".into());
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
        Path::new("tests/fixtures/independent-projection-v1.json").to_path_buf(),
        Path::new("tools/evaluation/src/independent_projection.rs").to_path_buf(),
        Path::new("docs/evaluation/independent-projection-v1-protocol-2026-10-19.md").to_path_buf(),
    ] {
        if !inputs.contains_key(p.to_str().unwrap()) {
            return Err("fuente obligatoria no vinculada".into());
        }
    }
    let revalidation = dir.join(format!("source-revalidation-{}.json", std::process::id()));
    crate::full_prefix_probe::report(source, &revalidation)?;
    if read(&revalidation)? != read(&source.join("report.json"))? {
        return Err("fuente no reproducible".into());
    }
    std::fs::remove_file(revalidation).map_err(|e| e.to_string())?;
    let original = lines(&source.join("requests.jsonl"))?;
    let plan = projection::prepare(&original)?;
    let requests = lines(&dir.join("projection.requests.jsonl"))?;
    if read(&dir.join("plan.json"))? != plan
        || plan["projection_requests"] != json!(requests)
        || requests.len() != 125
        || requests.iter().filter(|r| r["kind"] == "query").count() != 20
    {
        return Err("plan o payloads de proyección distintos".into());
    }
    let events = lines(&dir.join("projection.journal.jsonl"))?;
    let (terminal, projection_fatal) = terminals(&requests, &events)?;
    if terminal.len() != 125 && !projection_fatal {
        return Err("fase1incompleta sin fallo terminal; verificar proceso".into());
    }
    let mut frames = BTreeMap::new();
    let mut projections = Vec::new();
    let mut known = 0.;
    let mut missing = 0;
    let mut projection_fees = 0.;
    let mut matching_fees = 0.;
    let mut qtimes = Vec::new();
    let mut mtimes = Vec::new();
    let mut atimes = Vec::new();
    let mut qvalid = 0;
    let mut mvalid = 0;
    let mut qinvalid = 0;
    let mut minvalid = 0;
    for (r, e) in requests.iter().zip(&terminal) {
        let result = if fatal(e) {
            Err("HTTP/transporte/modelo inválido".into())
        } else {
            projection::check_projection(
                r["kind"].as_str().unwrap(),
                r["source_text"].as_str().unwrap(),
                &e["body"],
            )
        };
        let (parsed, error) = match result {
            Ok(p) => {
                frames.insert(r["id"].as_str().unwrap().to_string(), p["decoded"].clone());
                (p, None)
            }
            Err(s) => (Value::Null, Some(s)),
        };
        let good = error.is_none();
        let time = elapsed(e)?;
        if r["kind"] == "query" {
            qtimes.push(time);
            if good {
                qvalid += 1;
            } else {
                qinvalid += 1;
            }
        } else {
            mtimes.push(time);
            if good {
                mvalid += 1;
            } else {
                minvalid += 1;
            }
        }
        let fee = fee(e)?;
        if let Some(f) = fee {
            known += f;
            projection_fees += f;
        } else {
            missing += 1;
        }
        projections.push(json!({"id":r["id"],"kind":r["kind"],"valid":good,"error":error,"projection":parsed,"known_cost_usd":fee,"elapsed_ms":time,"served_model":e["body"]["model"],"provider":e["body"]["provider"],"usage":e["body"]["usage"]}));
    }
    let mut matched = BTreeMap::new();
    let mut match_rows = Vec::new();
    let mut matching_fatal = false;
    let mut skips = Vec::new();
    let mut match_total = 0;
    if projection_fatal {
        if dir.join("matching.preflight.json").exists() {
            return Err("matching iniciado tras fallo de proyección".into());
        }
    } else {
        let (expected, unavailable) = projection::matching_requests(&original, &plan, &frames)?;
        let mr = lines(&dir.join("matching.requests.jsonl"))?;
        let mp = read(&dir.join("matching.preflight.json"))?;
        if mr != expected
            || read(&dir.join("matching.skipped.json"))? != json!(unavailable)
            || mp["matching_operations_planned"] != mr.len()
            || mp["skipped_pairs"] != unavailable.len()
            || mp["matching_requests_sha256"]
                != crate::campaign::digest(&dir.join("matching.requests.jsonl"))?
            || mp["projection_journal_sha256"]
                != crate::campaign::digest(&dir.join("projection.journal.jsonl"))?
            || mp["phase1_preflight_sha256"]
                != crate::campaign::digest(&dir.join("preflight.json"))?
            || mp["skipped_sha256"] != crate::campaign::digest(&dir.join("matching.skipped.json"))?
        {
            return Err("fase2no ligada a proyecciones reales o payloads alterados".into());
        }
        skips = unavailable;
        match_total = mr.len();
        let me = lines(&dir.join("matching.journal.jsonl"))?;
        let (te, fatal) = terminals(&mr, &me)?;
        matching_fatal = fatal;
        if te.len() != mr.len() && !fatal {
            return Err("fase2incompleta sin fallo terminal; verificar proceso".into());
        }
        for (r, e) in mr.iter().zip(&te) {
            let q = &frames[r["query_projection_id"].as_str().unwrap()];
            let m = &frames[r["memory_projection_id"].as_str().unwrap()];
            let result = if fatal_transport(e) {
                Err("HTTP/transporte/modelo inválido".into())
            } else {
                projection::check_match(&e["body"], q, m)
            };
            let (check, error) = match result {
                Ok(c) => (c, None),
                Err(s) => (Value::Null, Some(s)),
            };
            let fee = fee(e)?;
            if let Some(f) = fee {
                known += f;
                matching_fees += f;
            } else {
                missing += 1;
            }
            let time = elapsed(e)?;
            atimes.push(time);
            let row = json!({"id":r["id"],"query_projection_id":r["query_projection_id"],"memory_projection_id":r["memory_projection_id"],"selected":check["selected"],"match_check":check,"error":error,"known_cost_usd":fee,"elapsed_ms":time,"served_model":e["body"]["model"],"provider":e["body"]["provider"],"usage":e["body"]["usage"]});
            matched.insert(r["id"].as_str().unwrap().to_string(), row.clone());
            match_rows.push(row);
        }
    }
    if !known.is_finite() {
        return Err("coste desbordado".into());
    }
    let mut counts = BTreeMap::<String, usize>::new();
    let mut observations = Vec::new();
    let mut controls = 0;
    for r in &original {
        let id = r["id"].as_str().unwrap();
        let row = matched.get(id);
        let selected = row.and_then(|r| r["selected"].as_bool());
        let label = r["expected_useful"].as_bool().ok_or("gold inválido")?;
        let unavailable = skips.iter().any(|s| s["id"] == id);
        let class = match selected {
            Some(true) if label => "true_positive",
            Some(true) => "false_positive",
            Some(false) if label => "false_negative",
            Some(false) => "true_negative",
            None if unavailable => "unavailable_projection",
            None if row.is_some() => "invalid_match",
            None => "not_attempted_after_transport",
        };
        if r["is_control"] == true {
            if selected == Some(label) {
                controls += 1;
            }
        } else {
            *counts.entry(class.into()).or_default() += 1;
        }
        observations.push(json!({"id":id,"original_case_id":r["original_case_id"],"memory_id":r["memory_id"],"is_control":r["is_control"],"expected_useful":label,"selected":selected,"classification":class,"match":row}));
    }
    let invalid_matches = match_rows.iter().filter(|r| !r["error"].is_null()).count();
    let complete = terminal.len() == 125
        && !projection_fatal
        && !matching_fatal
        && match_rows.len() == match_total;
    let passes = complete
        && qinvalid == 0
        && minvalid == 0
        && skips.is_empty()
        && invalid_matches == 0
        && missing == 0
        && controls == 2
        && counts.get("false_positive").copied().unwrap_or(0) == 0
        && counts.get("false_negative").copied().unwrap_or(0) == 0;
    let result = json!({"version":"independent-projection-diagnostic-v1","operations":terminal.len()+match_rows.len(),"http_requests":terminal.len()+match_rows.len(),"projection_operations":terminal.len(),"matching_operations":match_rows.len(),"matching_planned":match_total,"skipped_pairs":skips.len(),"complete":complete,"projection_fatal_transport":projection_fatal,"matching_fatal_transport":matching_fatal,"query_projections_valid":qvalid,"query_projections_invalid":qinvalid,"memory_projections_valid":mvalid,"memory_projections_invalid":minvalid,"invalid_matches":invalid_matches,"candidate_counts":counts,"controls_correct":controls,"known_cost_usd":known,"preparation_cost_usd":projection_fees,"matching_cost_usd":matching_fees,"unknown_cost_operations":missing,"query_projection_p50_ms":percentile(qtimes.clone(),0.5),"query_projection_p95_ms":percentile(qtimes,0.95),"memory_projection_p50_ms":percentile(mtimes.clone(),0.5),"memory_projection_p95_ms":percentile(mtimes,0.95),"individual_match_p50_ms":percentile(atimes.clone(),0.5),"individual_match_p95_ms":percentile(atimes,0.95),"passes_semantic_screen":passes,"projections":projections,"matching_observations":match_rows,"observations":observations,"projection_journal_sha256":crate::campaign::digest(&dir.join("projection.journal.jsonl"))?,"matching_journal_sha256":if dir.join("matching.journal.jsonl").exists(){Some(crate::campaign::digest(&dir.join("matching.journal.jsonl"))?)}else{None},"source_audit_reproduced":true,"fresh_validation":false,"original_gold_changed":false,"promoted":false,"warning":"Same128knownpairs. Preparation costs counted; stage latencies are not recall. Input isolation and source links do not certify semantic correctness or independent model family. No retrospective labels or model mixing."});
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| e.to_string())?
        .write_all(&serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn fatal_transport(e: &Value) -> bool {
    fatal(e)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_tail_batch_is_valid_and_uncertain_batch_is_not() {
        let rr = vec![json!({"id":"only"})];
        let ev = vec![
            json!({"kind":"started","id":"only"}),
            json!({"kind":"response","id":"only","http_status":200,"body":{"model":"openai/gpt-4.1-mini"}}),
        ];
        assert_eq!(terminals(&rr, &ev).unwrap().0.len(), 1);
        assert!(terminals(&rr, &ev[..1]).is_err());
    }
}
