//! Independent audit of all retrieved candidates plus declared development controls.
use crate::isolated_provider_probe::{cost, ids, lines, read};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const MODEL: &str = "perplexity/pplx-decider-v1.2";
const REVISION: &str = "perplexity/pplx-decider-v1.2-20261016";
fn choice(body: &Value) -> Result<(bool, Value), String> {
    let a = body["answers"]
        .as_object()
        .filter(|a| a.len() == 5)
        .ok_or("faltan cinco respuestas")?;
    let mut b = serde_json::Map::new();
    for k in [
        "fact",
        "ordinary_correction",
        "context_contrast",
        "necessary_premise_incompatible",
        "asked_dimension",
    ] {
        let x = a.get(k).ok_or("respuesta ajena")?;
        if x["type"] != "choice" {
            return Err("respuesta no choice".into());
        }
        b.insert(
            k.into(),
            json!(match x["choice"].as_str() {
                Some("A") => true,
                Some("none") => false,
                _ => return Err("opción desconocida".into()),
            }),
        );
    }
    let selected = b["fact"] == true && b["asked_dimension"] == true
        || b["ordinary_correction"] == true && b["necessary_premise_incompatible"] == true
        || b["context_contrast"] == true;
    Ok((selected, json!(b)))
}
fn percentile(mut v: Vec<f64>, p: f64) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let i = (v.len() - 1) as f64 * p;
    Some(v[i.floor() as usize] + (v[i.ceil() as usize] - v[i.floor() as usize]) * (i - i.floor()))
}

pub fn report(directory: &Path, output: &Path) -> Result<(), String> {
    let pre = read(&directory.join("preflight.json"))?;
    if pre["prepared_operations"] != 128
        || pre["max_http_requests"] != 128
        || pre["candidate_judgments"] != 126
        || pre["development_controls"] != 2
        || pre["model_id"] != MODEL
        || pre["expected_served_model"] != REVISION
        || pre["requests_sha256"] != crate::campaign::digest(&directory.join("requests.jsonl"))?
    {
        return Err("preflight alterado".into());
    }
    let inputs = pre["inputs_sha256"].as_object().ok_or("faltan fuentes")?;
    for (p, h) in inputs {
        if crate::campaign::digest(Path::new(p))? != h.as_str().ok_or("hash inválido")? {
            return Err(format!("fuente cambió: {p}"));
        }
    }
    let reference = Path::new("outputs/iromem/evaluation/role-complete-premise-probe-2026-10-19");
    let refs = lines(&reference.join("requests.jsonl"))?;
    let rp = read(&reference.join("preflight.json"))?;
    if refs.len() != 43
        || rp["requests_sha256"] != crate::campaign::digest(&reference.join("requests.jsonl"))?
    {
        return Err("referencia de preguntas no vinculada".into());
    }
    let mut questions = refs[0]["payload"]["questions"].clone();
    if refs.iter().any(|r| r["payload"]["questions"] != questions)
        || questions["necessary_premise_incompatible"]
            != read(Path::new("tests/fixtures/premise-incompatibility-v1.json"))?
    {
        return Err("templates anteriores distintos".into());
    }
    questions["asked_dimension"] = read(Path::new("tests/fixtures/asked-dimension-v1.json"))?;
    for p in [
        "tests/fixtures/asked-dimension-v1.json",
        "tests/fixtures/asked-dimension-controls-v1.json",
        "tests/fixtures/premise-incompatibility-v1.json",
        "docs/evaluation/full-prefix-dimension-probe-protocol-2026-10-19.md",
    ] {
        if !inputs.contains_key(p) {
            return Err("fixture/protocolo obligatorio sin vínculo".into());
        }
    }
    for p in [
        reference.join("requests.jsonl"),
        reference.join("preflight.json"),
    ] {
        if !inputs.contains_key(p.to_str().unwrap()) {
            return Err("referencia de preguntas no congelada".into());
        }
    }
    let mut expected = Vec::new();
    let mut case_metadata = BTreeMap::new();
    for (suffix, cohort, count) in [("pilot", "v5", 13), ("v4-targeted", "v4", 5)] {
        let source = PathBuf::from(format!(
            "outputs/iromem/evaluation/per-memory-complete-premise-v1-{suffix}-2026-10-19"
        ));
        crate::verified::validate_pilot(&source)?;
        for p in [
            source.join("cases.jsonl"),
            source.join("journal.jsonl"),
            source.join("pilot.json"),
            source.join("audit.json"),
        ] {
            if !inputs.contains_key(p.to_str().unwrap()) {
                return Err("fuente de candidatos no congelada".into());
            }
        }
        let audit = read(&source.join("audit.json"))?;
        if audit["campaigns"][0]["integrity"]["valid"] != true {
            return Err("fuente sin auditoría de integridad".into());
        }
        let cases = lines(&source.join("cases.jsonl"))?;
        let events = lines(&source.join("journal.jsonl"))?;
        let finished = events
            .iter()
            .filter(|e| e["kind"] == "finished")
            .collect::<Vec<_>>();
        if finished.len() != count {
            return Err("grupo fuente incompleto".into());
        }
        for e in finished {
            crate::verified::inspect_complete_premise(&e["response"], "typesafe/jev-1.13")?;
            let case = cases
                .iter()
                .find(|c| c["id"] == e["id"])
                .ok_or("caso fuente ajeno")?;
            let original = case["original_case_id"]
                .as_str()
                .ok_or("caso original ausente")?;
            let allowed = ids(&case["expected_ids"])?;
            let candidates = e["response"]["retrieval"]["candidates"]
                .as_array()
                .filter(|c| c.len() >= 7)
                .ok_or("prefijo incompleto")?;
            let mut seen = BTreeSet::new();
            for (rank, c) in candidates.iter().take(7).enumerate() {
                let memory = c["memory_id"].as_str().ok_or("ID ausente")?;
                if !seen.insert(memory) {
                    return Err("candidato repetido".into());
                }
                expected.push(json!({"id":format!("{original}_{memory}"),"original_case_id":original,"source_original_case_id":original,"memory_id":memory,"revision":c["revision"],"cohort":cohort,"presented_rank":rank,"is_control":false,"expected_useful":allowed.contains(memory),"payload":{"model":MODEL,"state":{"query":case["request"]["search"]["query"],"saved_memory":c["summary"]},"questions":questions}}));
            }
            case_metadata.insert(
                original.to_string(),
                json!({"cohort":cohort,"allowed_ids":allowed,"question_relation":case["relation"]}),
            );
        }
    }
    if expected.len() != 126 || case_metadata.len() != 18 {
        return Err("población fuente distinta".into());
    }
    let controls = read(Path::new("tests/fixtures/asked-dimension-controls-v1.json"))?;
    if controls["independent_review"] != false
        || controls["cases"].as_array().is_none_or(|c| c.len() != 2)
    {
        return Err("controles de desarrollo distintos".into());
    }
    for control in controls["cases"].as_array().unwrap() {
        let source = expected
            .iter()
            .find(|r| {
                r["original_case_id"] == control["source_original_case_id"]
                    && r["memory_id"] == control["source_memory_id"]
            })
            .ok_or("control sin recuerdo original")?;
        let mut r = source.clone();
        r["id"] = control["id"].clone();
        r["original_case_id"] = control["id"].clone();
        r["cohort"] = json!("development_controls");
        r["presented_rank"] = Value::Null;
        r["is_control"] = json!(true);
        r["expected_useful"] = control["expected_useful"].clone();
        r["payload"]["state"]["query"] = control["query"].clone();
        expected.push(r);
    }
    let requests = lines(&directory.join("requests.jsonl"))?;
    if requests != expected {
        return Err("solicitudes alteran roster, gold o payload, o filtran propuestas".into());
    }
    let events = lines(&directory.join("journal.jsonl"))?;
    if events.len() % 2 != 0 || events.len() / 2 > 128 {
        return Err("diario sin terminal o excedido; verificar proceso sin reintentar".into());
    }
    let mut rows = Vec::new();
    let mut known = 0.0;
    let mut missing = 0;
    let mut technical = false;
    let mut candidate_counts = BTreeMap::<String, usize>::new();
    let mut control_correct = 0;
    let mut times = Vec::new();
    for (r, pair) in requests.iter().zip(events.chunks_exact(2)) {
        if technical {
            return Err("se continuó tras fallo técnico/revisión distinta".into());
        }
        if pair[0]["kind"] != "started"
            || pair[0]["id"] != r["id"]
            || pair[1]["id"] != r["id"]
            || !matches!(pair[1]["kind"].as_str(), Some("response" | "failed"))
        {
            return Err("operación fuera de roster o repetida".into());
        }
        let e = &pair[1];
        let (selected, paths, error) = if e["kind"] == "response" {
            match choice(&e["body"]) {
                Ok((s, p)) => (Some(s), p, None),
                Err(s) => (None, Value::Null, Some(s)),
            }
        } else {
            (None, Value::Null, Some("HTTP failed".into()))
        };
        let revision = e["body"]["model"] == REVISION;
        technical = selected.is_none() || !revision;
        let usage = if e["kind"] == "response" {
            &e["body"]["usage"]
        } else {
            &e["partial_usage"]
        };
        let fee = cost(
            usage,
            if e["kind"] == "response" {
                "cost"
            } else {
                "reported_cost_usd"
            },
        )?;
        if let Some(f) = fee {
            known += f;
        } else {
            missing += 1;
        }
        if let Some(t) = e["elapsed_ms"]
            .as_f64()
            .filter(|t| t.is_finite() && *t >= 0.0)
        {
            times.push(t);
        }
        let label = r["expected_useful"].as_bool().ok_or("gold no binario")?;
        let classification = match selected {
            Some(true) if label => "true_positive",
            Some(true) => "false_positive",
            Some(false) if label => "false_negative",
            Some(false) => "true_negative",
            None => "technical_failure",
        };
        if r["is_control"] == true {
            if selected == Some(label) && revision {
                control_correct += 1;
            }
        } else {
            *candidate_counts.entry(classification.into()).or_default() += 1;
        }
        rows.push(json!({"id":r["id"],"original_case_id":r["original_case_id"],"source_original_case_id":r["source_original_case_id"],"memory_id":r["memory_id"],"revision":r["revision"],"cohort":r["cohort"],"is_control":r["is_control"],"expected_useful":label,"selected":selected,"classification":classification,"paths":paths,"revision_matches":revision,"technical_error":error,"known_cost_usd":fee,"usage":usage,"served_model":e["body"]["model"],"elapsed_ms":e["elapsed_ms"]}));
    }
    if !known.is_finite() {
        return Err("coste desbordado".into());
    }
    let mut cases = Vec::new();
    for (id, meta) in case_metadata {
        let observations = rows
            .iter()
            .filter(|r| r["is_control"] == false && r["original_case_id"] == id)
            .collect::<Vec<_>>();
        let prefix = requests
            .iter()
            .filter(|r| r["is_control"] == false && r["original_case_id"] == id)
            .map(|r| r["memory_id"].as_str().unwrap().to_string())
            .collect::<BTreeSet<_>>();
        let allowed = ids(&meta["allowed_ids"])?;
        let selected = observations
            .iter()
            .filter(|r| r["selected"] == true)
            .map(|r| r["memory_id"].as_str().unwrap().to_string())
            .collect::<BTreeSet<_>>();
        cases.push(json!({"original_case_id":id,"cohort":meta["cohort"],"question_relation":meta["question_relation"],"observations":observations.len(),"presented_ids":prefix,"selected_ids":selected,"wrong_ids":selected.difference(&allowed).collect::<Vec<_>>(),"omitted_allowed_within_prefix":allowed.intersection(&prefix).filter(|id|!selected.contains(*id)).collect::<Vec<_>>(),"allowed_outside_prefix":allowed.difference(&prefix).collect::<Vec<_>>()}));
    }
    let complete = rows.len() == 128;
    let passes = complete
        && !technical
        && missing == 0
        && control_correct == 2
        && candidate_counts.get("false_positive").copied().unwrap_or(0) == 0
        && candidate_counts.get("false_negative").copied().unwrap_or(0) == 0;
    let result = json!({"version":"full-prefix-dimension-probe-v1","prepared_operations":128,"operations":rows.len(),"http_requests":rows.len(),"candidate_observations":rows.iter().filter(|r|r["is_control"]==false).count(),"control_observations":rows.iter().filter(|r|r["is_control"]==true).count(),"candidate_utility_counts":candidate_counts,"controls_correct":control_correct,"complete":complete,"technical_stop":technical,"known_cost_usd":known,"unknown_cost_operations":missing,"all_new_cost_receipts_complete":missing==0,"individual_check_p50_ms":percentile(times.clone(),0.5),"individual_check_p95_ms":percentile(times,0.95),"passes_isolated_semantic_screen":passes,"cases":cases,"observations":rows,"requests_sha256":pre["requests_sha256"],"journal_sha256":crate::campaign::digest(&directory.join("journal.jsonl"))?,"fresh_validation":false,"promoted":false,"warning":"All126known prefix candidates plus2author development controls. Costs/times are isolated research checks, not efficient runtime or end-to-end recall. Original gold unchanged; no guarantee outside prefix."});
    use std::io::Write;
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
    fn dimension_gate_cannot_veto_valid_correction_and_missing_answer_is_not_abstention() {
        let a = |s| json!({"type":"choice","choice":s});
        let mut b = json!({"answers":{"fact":a("A"),"ordinary_correction":a("none"),"context_contrast":a("none"),"necessary_premise_incompatible":a("none"),"asked_dimension":a("none")}});
        assert!(!choice(&b).unwrap().0);
        b["answers"]["ordinary_correction"]["choice"] = json!("A");
        b["answers"]["necessary_premise_incompatible"]["choice"] = json!("A");
        assert!(choice(&b).unwrap().0);
        b["answers"]
            .as_object_mut()
            .unwrap()
            .remove("asked_dimension");
        assert!(choice(&b).is_err());
    }
}
