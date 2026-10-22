//! Audit of automatic assertion projection, including failures and all in-flight charges.
use crate::{
    isolated_provider_probe::{cost, lines, read},
    projection_ids as projection,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, io::Write, path::Path};
fn percentile(mut v: Vec<f64>, p: f64) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let i = (v.len() - 1) as f64 * p;
    Some(v[i.floor() as usize] + (v[i.ceil() as usize] - v[i.floor() as usize]) * (i - i.floor()))
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
        Path::new("tests/fixtures/assertion-projection-ids-v3.json").to_path_buf(),
        Path::new("docs/evaluation/assertion-projection-ids-v3-protocol-2026-10-19.md")
            .to_path_buf(),
        Path::new("tools/evaluation/src/projection_ids.rs").to_path_buf(),
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
                    batch_failed = true;
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
    let result = json!({"version":"assertion-projection-ids-probe-v3","operations":rows.len(),"http_requests":rows.len(),"prepared_operations":128,"complete":complete,"technical_stop":stopped,"invalid_extractions":invalid,"unestablished_useful_claims":unestablished,"candidate_counts":counts,"controls_correct":controls,"known_cost_usd":known,"unknown_cost_operations":unknown,"individual_extraction_p50_ms":percentile(times.clone(),0.5),"individual_extraction_p95_ms":percentile(times,0.95),"passes_semantic_screen":passes,"observations":rows,"requests_sha256":pre["requests_sha256"],"journal_sha256":crate::campaign::digest(&dir.join("journal.jsonl"))?,"source_audit_reproduced":true,"original_gold_changed":false,"fresh_validation":false,"promoted":false,"warning":"Known128pairs including2authorcontrols. Quote/target integrity is checked; same-case and relation remain fallible model judgments. Different model plus representation; no isolated causal claim. Alias not proof of immutable model revision. Diagnostic extraction latency/cost is not efficient recall or Codex cost."});
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| e.to_string())?
        .write_all(&serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
