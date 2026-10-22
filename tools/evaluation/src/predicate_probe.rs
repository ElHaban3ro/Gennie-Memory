//! Independent full-population audit of the property-relation hypothesis.
use crate::isolated_provider_probe::{cost, lines, read};
use serde_json::{Value, json};
use std::{collections::BTreeMap, io::Write, path::Path};
const MODELS: [(&str, &str); 3] = [
    (
        "perplexity/pplx-decider-v1.2",
        "perplexity/pplx-decider-v1.2-20261016",
    ),
    ("respan/span-02", "respan/span-02-20261015"),
    ("liquid/d1-ultra", "liquid/d1-ultra-20261015"),
];
fn choice(body: &Value) -> Result<&str, String> {
    let a = body["answers"]
        .as_object()
        .filter(|a| a.len() == 1)
        .ok_or("respuesta incompleta o extra")?;
    let x = a.get("relation").ok_or("falta relación")?;
    if x["type"] != "choice" {
        return Err("relación no choice".into());
    }
    x["choice"]
        .as_str()
        .filter(|s| matches!(*s, "A" | "B" | "C" | "none"))
        .ok_or("relación inválida".into())
}
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
    if pre["prepared_operations"] != 384
        || pre["max_http_requests"] != 384
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
        Path::new("tests/fixtures/predicate-relation-v1.json").to_path_buf(),
        Path::new("docs/evaluation/predicate-relation-probe-protocol-2026-10-19.md").to_path_buf(),
        Path::new("outputs/iromem/evaluation/decision-catalog-observation-2026-10-19.json")
            .to_path_buf(),
    ] {
        if !inputs.contains_key(p.to_str().unwrap()) {
            return Err("fuente obligatoria no vinculada".into());
        }
    }
    // Reconstruct source population against original CLI pilots, not a trusted status flag.
    let revalidation = dir.join(format!("source-revalidation-{}.json", std::process::id()));
    crate::full_prefix_probe::report(source, &revalidation)?;
    let checked = read(&revalidation)?;
    let original = read(&source.join("report.json"))?;
    if checked != original {
        return Err("auditoría fuente no reproducible; conservar diagnóstico".into());
    }
    std::fs::remove_file(&revalidation).map_err(|e| e.to_string())?;
    let originals = lines(&source.join("requests.jsonl"))?;
    let task = read(Path::new("tests/fixtures/predicate-relation-v1.json"))?;
    let catalog = read(Path::new(
        "outputs/iromem/evaluation/decision-catalog-observation-2026-10-19.json",
    ))?;
    if pre["models"] != catalog["models"] {
        return Err("modelos no vinculados".into());
    }
    let mut expected = Vec::new();
    for (i, (model, revision)) in MODELS.iter().enumerate() {
        if pre["models"][i]["id"] != *model || pre["models"][i]["canonical_slug"] != *revision {
            return Err("modelos/revisiones distintos".into());
        }
        for s in &originals {
            let mut r = s.clone();
            r["id"] = json!(format!(
                "model{i}_{}",
                s["id"].as_str().ok_or("ID fuente inválido")?
            ));
            r["source_request_id"] = s["id"].clone();
            r["model_id"] = json!(model);
            r["expected_served_model"] = json!(revision);
            r["payload"] =
                json!({"model":model,"state":s["payload"]["state"],"questions":{"relation":task}});
            expected.push(r);
        }
    }
    let requests = lines(&dir.join("requests.jsonl"))?;
    if requests != expected || requests.len() != 384 {
        return Err("roster, gold o payload distintos".into());
    }
    let events = lines(&dir.join("journal.jsonl"))?;
    if events.len() % 2 != 0 || events.len() / 2 > 384 {
        return Err("diario incierto o excedido; inspeccionar proceso sin reintentar".into());
    }
    let mut rows = Vec::new();
    let mut technical = false;
    for (r, pair) in requests.iter().zip(events.chunks_exact(2)) {
        if technical
            || pair[0]["kind"] != "started"
            || pair[0]["id"] != r["id"]
            || pair[1]["id"] != r["id"]
            || !matches!(pair[1]["kind"].as_str(), Some("response" | "failed"))
        {
            return Err("operación fuera de roster, repetida o tras fallo".into());
        }
        let e = &pair[1];
        let relation = if e["kind"] == "response" {
            choice(&e["body"]).ok()
        } else {
            None
        };
        let revision_matches = e["body"]["model"] == r["expected_served_model"];
        technical = relation.is_none() || !revision_matches;
        let selected = relation.map(|x| matches!(x, "A" | "B"));
        let label = r["expected_useful"].as_bool().ok_or("gold inválido")?;
        let classification = match selected {
            Some(true) if label => "true_positive",
            Some(true) => "false_positive",
            Some(false) if label => "false_negative",
            Some(false) => "true_negative",
            None => "technical_failure",
        };
        let fee = if e["kind"] == "response" {
            cost(&e["body"]["usage"], "cost")?
        } else {
            cost(&e["partial_usage"], "reported_cost_usd")?
        };
        let elapsed = e["elapsed_ms"]
            .as_f64()
            .filter(|n| n.is_finite() && *n >= 0.)
            .ok_or("tiempo ausente/inválido")?;
        rows.push(json!({"id":r["id"],"source_request_id":r["source_request_id"],"model_id":r["model_id"],"original_case_id":r["original_case_id"],"memory_id":r["memory_id"],"cohort":r["cohort"],"is_control":r["is_control"],"expected_useful":label,"selected":selected,"relation":relation,"classification":classification,"revision_matches":revision_matches,"known_cost_usd":fee,"elapsed_ms":elapsed}));
    }
    let mut summaries = Vec::new();
    let mut total = 0.;
    let mut unknown = 0;
    for (model, revision) in MODELS {
        let rr = rows
            .iter()
            .filter(|r| r["model_id"] == model)
            .collect::<Vec<_>>();
        let mut counts = BTreeMap::<String, usize>::new();
        let mut relations = BTreeMap::<String, usize>::new();
        let mut known = 0.;
        let mut missing = 0;
        let mut controls = 0;
        let mut times = Vec::new();
        let mut errors = Vec::new();
        for r in &rr {
            if let Some(f) = r["known_cost_usd"].as_f64() {
                known += f;
            } else {
                missing += 1;
            }
            times.push(r["elapsed_ms"].as_f64().unwrap());
            *relations
                .entry(r["relation"].as_str().unwrap_or("technical_failure").into())
                .or_default() += 1;
            if r["is_control"] == true {
                if r["selected"] == r["expected_useful"] && r["revision_matches"] == true {
                    controls += 1;
                }
            } else {
                let c = r["classification"].as_str().unwrap();
                *counts.entry(c.into()).or_default() += 1;
                if !c.starts_with("true_") {
                    errors.push((*r).clone());
                }
            }
        }
        if !known.is_finite() {
            return Err("coste desbordado".into());
        }
        total += known;
        unknown += missing;
        let passes = rr.len() == 128
            && missing == 0
            && controls == 2
            && rr.iter().all(|r| r["revision_matches"] == true)
            && counts.get("false_positive").copied().unwrap_or(0) == 0
            && counts.get("false_negative").copied().unwrap_or(0) == 0
            && counts.get("technical_failure").copied().unwrap_or(0) == 0;
        summaries.push(json!({"model_id":model,"expected_served_model":revision,"operations":rr.len(),"candidate_counts":counts,"relation_counts_including_controls":relations,"controls_correct":controls,"known_cost_usd":known,"unknown_cost_operations":missing,"individual_check_p50_ms":percentile(times.clone(),0.5),"individual_check_p95_ms":percentile(times,0.95),"passes_semantic_screen":passes,"errors":errors}));
    }
    let result = json!({"version":"predicate-relation-probe-v1","operations":rows.len(),"http_requests":rows.len(),"complete":rows.len()==384,"technical_stop":technical,"known_cost_usd":total,"unknown_cost_operations":unknown,"models":summaries,"observations":rows,"requests_sha256":pre["requests_sha256"],"journal_sha256":crate::campaign::digest(&dir.join("journal.jsonl"))?,"source_audit_reproduced":true,"original_gold_changed":false,"fresh_validation":false,"promoted":false,"warning":"Known full-prefix development, two author controls. C is related context, not factual answer or requested evidence. No model mixing; costs exclude Codex; check times are not end-to-end recall."});
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| e.to_string())?
        .write_all(&serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_context_category_and_rejects_missing_extra_or_invalid_answers() {
        for s in ["A", "B", "C", "none"] {
            assert_eq!(
                choice(&json!({"answers":{"relation":{"type":"choice","choice":s}}})).unwrap(),
                s
            );
        }
        for b in [
            json!({"answers":{}}),
            json!({"answers":{"relation":{"type":"choice","choice":"invalid"}}}),
            json!({"answers":{"relation":{"type":"choice","choice":"A"},"extra":{}}}),
        ] {
            assert!(choice(&b).is_err());
        }
    }
}
