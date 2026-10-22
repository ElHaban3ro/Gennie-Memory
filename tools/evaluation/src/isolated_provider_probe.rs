//! Raw paired judgments over frozen proposals; never a live recall quality claim.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) fn read(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
pub(super) fn lines(path: &Path) -> Result<Vec<Value>, String> {
    std::fs::read_to_string(path)
        .map_err(|e| e.to_string())?
        .lines()
        .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
        .collect()
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key].as_str().ok_or_else(|| format!("falta {key}"))
}
pub(super) fn ids(v: &Value) -> Result<BTreeSet<String>, String> {
    v.as_array()
        .ok_or("faltan IDs")?
        .iter()
        .map(|x| x.as_str().map(String::from).ok_or("ID no textual".into()))
        .collect()
}
fn completed(path: &Path) -> Result<BTreeMap<String, Value>, String> {
    let mut result = BTreeMap::new();
    for e in lines(path)? {
        if e["kind"] == "finished" {
            if result
                .insert(text(&e, "id")?.into(), e["response"].clone())
                .is_some()
            {
                return Err("respuesta histórica repetida".into());
            }
        }
    }
    Ok(result)
}
pub(super) fn cost(v: &Value, key: &str) -> Result<Option<f64>, String> {
    if v[key].is_null() {
        return Ok(None);
    }
    v[key]
        .as_f64()
        .filter(|n| n.is_finite() && *n >= 0.0)
        .map(Some)
        .ok_or("recibo inválido".into())
}
pub(super) fn choice(body: &Value, request: &Value, correction: bool) -> Result<bool, String> {
    let q = request["payload"]["questions"]
        .as_object()
        .ok_or("faltan preguntas")?;
    let a = body["answers"].as_object().ok_or("faltan respuestas")?;
    if q.len() != if correction { 2 } else { 1 } || a.len() != q.len() {
        return Err("cardinalidad de respuestas incorrecta".into());
    }
    for (k, v) in q {
        let answer = a.get(k).ok_or("respuesta ajena")?;
        let c = answer["choice"].as_str().ok_or("choice ausente")?;
        if answer["type"] != "choice"
            || !matches!(c, "A" | "none")
            || !v["criteria"].as_object().is_some_and(|q| q.contains_key(c))
        {
            return Err("choice inválida".into());
        }
    }
    if !a.contains_key("relevance") || correction && !a.contains_key("context_contrast") {
        return Err("tarea equivocada".into());
    }
    Ok(a["relevance"]["choice"] == "A" || correction && a["context_contrast"]["choice"] == "A")
}
fn percentile(mut values: Vec<f64>, p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let index = (values.len() - 1) as f64 * p;
    let lo = index.floor() as usize;
    let hi = index.ceil() as usize;
    Some(values[lo] + (values[hi] - values[lo]) * (index - lo as f64))
}

pub(super) fn evaluate(directory: &Path) -> Result<Value, String> {
    let preflight = read(&directory.join("preflight.json"))?;
    if preflight["operations"] != 86
        || preflight["max_http_requests"] != 86
        || preflight["frozen_positive_pairs"] != 43
        || preflight["requests_sha256"]
            != crate::campaign::digest(&directory.join("requests.jsonl"))?
    {
        return Err("preflight alterado".into());
    }
    for (path, hash) in preflight["historical_inputs_sha256"]
        .as_object()
        .ok_or("faltan fuentes vinculadas")?
    {
        if crate::campaign::digest(Path::new(path))? != hash.as_str().ok_or("hash inválido")? {
            return Err(format!("fuente cambió: {path}"));
        }
    }
    let requests = lines(&directory.join("requests.jsonl"))?;
    let events = lines(&directory.join("journal.jsonl"))?;
    if requests.len() != 86 || events.len() != 172 {
        return Err("diagnóstico incompleto o repetido".into());
    }
    let mut records = BTreeMap::new();
    for (r, e) in requests.iter().zip(events.chunks_exact(2)) {
        let id = text(r, "id")?;
        if e[0]["kind"] != "started"
            || e[0]["id"] != id
            || e[1]["id"] != id
            || !matches!(e[1]["kind"].as_str(), Some("response" | "failed"))
        {
            return Err("transición del diario inválida".into());
        }
        if records.insert(id.to_string(), (r, e[1].clone())).is_some() {
            return Err("petición repetida".into());
        }
    }
    let mut group_reports = BTreeMap::<String, Vec<Value>>::new();
    let mut comparisons = Vec::new();
    let mut used = BTreeSet::new();
    let mut total_known = 0.0;
    let mut unknown = 0;
    for (suffix, cohort, count) in [("pilot", "v5", 13), ("v4-targeted", "v4", 5)] {
        let source = PathBuf::from(format!(
            "outputs/iromem/evaluation/per-memory-role-verified-v2-{suffix}-2026-10-19"
        ));
        crate::verified::validate_pilot(&source)?;
        let spec = read(&source.join("pilot.json"))?;
        for f in [
            source.join("journal.jsonl"),
            source.join("cases.jsonl"),
            source.join("pilot.json"),
            PathBuf::from(text(&spec, "corpus")?),
            PathBuf::from(text(&spec, "source_cases")?),
        ] {
            if preflight["historical_inputs_sha256"][f.to_str().ok_or("ruta inválida")?].is_null()
            {
                return Err("fuente obligatoria no congelada".into());
            }
        }
        let cases = lines(&source.join("cases.jsonl"))?;
        if cases.len() != count {
            return Err("grupo histórico incompleto".into());
        }
        let proposal_responses = completed(&source.join("journal.jsonl"))?;
        let baseline = completed(Path::new(text(&spec, "baseline_journal")?))?;
        let preservation = if cohort == "v5" {
            Some(completed(Path::new(text(&spec, "preservation_journal")?))?)
        } else {
            None
        };
        let mut per_provider = BTreeMap::<String, Vec<Value>>::new();
        for case in cases {
            let original = text(&case, "original_case_id")?;
            let response = proposal_responses
                .get(text(&case, "id")?)
                .ok_or("propuesta histórica ausente")?;
            crate::verified::inspect_roles_v2(response, "typesafe/jev-1.13")?;
            let verification = &response["per_memory"]["verification"];
            let proposal_cost = cost(&verification["proposal_usage"], "reported_cost_usd")?
                .ok_or("propuesta sin recibo")?;
            let allowed = ids(&case["expected_ids"])?;
            let base = baseline
                .get(&format!("{original}__baseline"))
                .ok_or("base histórica ausente")?;
            let base_cost = cost(base, "reported_cost_usd")?.ok_or("base sin recibo")?;
            let base_allowed = ids(&base["selected_ids"])?
                .intersection(&allowed)
                .cloned()
                .collect::<BTreeSet<_>>();
            let previous = if let Some(p) = &preservation {
                ids(&p
                    .get(&format!("{original}__jev_per_memory_scoped_v1"))
                    .ok_or("preservación ausente")?["selected_ids"])?
                .intersection(&allowed)
                .cloned()
                .collect::<BTreeSet<_>>()
            } else {
                BTreeSet::new()
            };
            let mut judgments = BTreeMap::<String, Vec<Value>>::new();
            for k in verification["kind_proposal"]
                .as_array()
                .ok_or("tipos ausentes")?
            {
                let kind = text(k, "evidence_kind")?;
                if kind == "unrelated" {
                    continue;
                }
                if !matches!(kind, "requested_fact_or_partial" | "premise_correction") {
                    return Err("tipo desconocido".into());
                }
                let memory = text(k, "memory_id")?;
                let candidate = response["retrieval"]["candidates"]
                    .as_array()
                    .ok_or("candidatos ausentes")?
                    .iter()
                    .find(|c| c["memory_id"] == memory && c["revision"] == k["revision"])
                    .ok_or("candidato ajeno")?;
                let mut pair = Vec::new();
                for provider in ["decisions", "jev"] {
                    let id = format!("{original}_{memory}_{provider}");
                    let (request, event) = records.get(&id).ok_or("par ausente")?;
                    used.insert(id);
                    let model = if provider == "jev" {
                        "typesafe/jev-1.13"
                    } else {
                        "openai/gpt-6-luna-decisions"
                    };
                    if request["original_case_id"] != original
                        || request["cohort"] != cohort
                        || request["provider"] != provider
                        || request["memory_id"] != memory
                        || request["revision"] != k["revision"]
                        || request["evidence_kind"] != kind
                        || request["payload"]["model"] != model
                        || request["payload"]["state"]
                            != json!({"query":case["request"]["search"]["query"],"saved_memory":candidate["summary"]})
                    {
                        return Err("par difiere de la propuesta congelada".into());
                    }
                    pair.push(request["payload"].clone());
                    let (selected, technical_error) = if event["kind"] == "response" {
                        match choice(&event["body"], request, kind == "premise_correction") {
                            Ok(b) => (Some(b), None),
                            Err(e) => (None, Some(e)),
                        }
                    } else {
                        (None, Some("HTTP failed".into()))
                    };
                    let usage = if event["kind"] == "response" {
                        &event["body"]["usage"]
                    } else {
                        &event["partial_usage"]
                    };
                    let c = cost(
                        usage,
                        if event["kind"] == "response" {
                            "cost"
                        } else {
                            "reported_cost_usd"
                        },
                    )?;
                    if let Some(c) = c {
                        total_known += c;
                    } else {
                        unknown += 1;
                    }
                    judgments.entry(provider.into()).or_default().push(json!({"memory_id":memory,"revision":k["revision"],"evidence_kind":kind,"selected":selected,"technical_error":technical_error,"known_cost_usd":c,"elapsed_ms":event["elapsed_ms"],"usage":usage,"served_model":event["body"]["model"]}));
                }
                pair[1]["model"] = pair[0]["model"].clone();
                if pair[0] != pair[1] {
                    return Err("cambiaron campos distintos del modelo".into());
                }
            }
            let mut summary = BTreeMap::new();
            for provider in ["decisions", "jev"] {
                let checks = judgments.remove(provider).unwrap_or_default();
                let degraded = checks.iter().any(|c| c["selected"].is_null());
                let selected = if degraded {
                    BTreeSet::new()
                } else {
                    checks
                        .iter()
                        .filter(|c| c["selected"] == true)
                        .map(|c| c["memory_id"].as_str().unwrap().to_string())
                        .collect::<BTreeSet<_>>()
                };
                let wrong = selected.difference(&allowed).cloned().collect::<Vec<_>>();
                let lost_base = base_allowed
                    .difference(&selected)
                    .cloned()
                    .collect::<Vec<_>>();
                let lost_previous = previous.difference(&selected).cloned().collect::<Vec<_>>();
                let gained = selected
                    .intersection(&allowed)
                    .filter(|id| !base_allowed.contains(*id))
                    .cloned()
                    .collect::<Vec<_>>();
                let missing = checks
                    .iter()
                    .filter(|c| c["known_cost_usd"].is_null())
                    .count();
                let sum = checks
                    .iter()
                    .filter_map(|c| c["known_cost_usd"].as_f64())
                    .sum::<f64>();
                let row = json!({"original_case_id":original,"selected_ids":selected,"allowed_ids":allowed,"baseline_permitted_ids":base_allowed,"previous_permitted_ids":previous,"wrong_ids":wrong,"lost_baseline_ids":lost_base,"lost_previous_useful_ids":lost_previous,"gained_useful_ids":gained,"omitted_allowed_ids":allowed.difference(&selected).collect::<Vec<_>>(),"degraded":degraded,"all_allowed_selected":selected==allowed,"historical_baseline_cost_usd":base_cost,"historical_proposal_cost_usd":proposal_cost,"new_verification_known_cost_usd":sum,"missing_cost_receipts":missing,"checks":checks});
                summary.insert(provider, row["selected_ids"].clone());
                per_provider.entry(provider.into()).or_default().push(row);
            }
            comparisons.push(json!({"original_case_id":original,"cohort":cohort,"decisions_ids":summary["decisions"],"jev_ids":summary["jev"],"same_selection":summary["decisions"]==summary["jev"]}));
        }
        for (provider, cases) in per_provider {
            let baseline = cases
                .iter()
                .map(|c| c["historical_baseline_cost_usd"].as_f64().unwrap())
                .sum::<f64>();
            let proposals = cases
                .iter()
                .map(|c| c["historical_proposal_cost_usd"].as_f64().unwrap())
                .sum::<f64>();
            let checks = cases
                .iter()
                .map(|c| c["new_verification_known_cost_usd"].as_f64().unwrap())
                .sum::<f64>();
            let missing = cases
                .iter()
                .map(|c| c["missing_cost_receipts"].as_u64().unwrap())
                .sum::<u64>();
            let quality = cases.iter().all(|c| {
                c["wrong_ids"] == json!([])
                    && c["lost_baseline_ids"] == json!([])
                    && c["lost_previous_useful_ids"] == json!([])
                    && c["degraded"] == false
            }) && if cohort == "v4" {
                cases.iter().all(|c| c["all_allowed_selected"] == true)
            } else {
                cases.iter().any(|c| c["gained_useful_ids"] != json!([]))
            };
            let times = cases
                .iter()
                .flat_map(|c| c["checks"].as_array().unwrap())
                .filter_map(|c| c["elapsed_ms"].as_f64())
                .collect::<Vec<_>>();
            let ratio = (proposals + checks) / baseline;
            group_reports.entry(provider).or_default().push(json!({"cohort":cohort,"cases":cases,"historical_baseline_cost_usd":baseline,"historical_proposal_cost_usd":proposals,"new_verification_known_cost_usd":checks,"estimated_pipeline_known_cost_usd":proposals+checks,"estimated_pipeline_cost_ratio":ratio,"missing_cost_receipts":missing,"passes_replay_quality":quality,"passes_estimated_cost_screen":missing==0&&ratio<=1.5,"verification_p50_ms":percentile(times.clone(),0.5),"verification_p95_ms":percentile(times,0.95)}));
        }
    }
    if used.len() != 86 || records.len() != 86 || !total_known.is_finite() {
        return Err("peticiones no vinculadas o total inválido".into());
    }
    let mut providers = BTreeMap::new();
    for (name, groups) in group_reports {
        let passes = groups.iter().all(|g| {
            g["passes_replay_quality"] == true && g["passes_estimated_cost_screen"] == true
        });
        providers.insert(
            name,
            json!({"groups":groups,"passes_bounded_implementation_screen":passes}),
        );
    }
    let report = json!({"version":"isolated-role-provider-probe-v1","operations":86,"http_requests":86,"paired_positive_judgments":43,"known_cost_usd":total_known,"unknown_cost_operations":unknown,"all_new_cost_receipts_complete":unknown==0,"requests_sha256":preflight["requests_sha256"],"journal_sha256":crate::campaign::digest(&directory.join("journal.jsonl"))?,"providers":providers,"case_comparisons":comparisons,"fresh_validation":false,"promoted":false,"warning":"Frozen historical proposals with newly measured isolated verifier stages. Estimated pipeline cost reuses historical proposal receipts; it is not contemporaneous recall cost or end-to-end latency. Both full-bank and fresh validation gates remain pending."});
    Ok(report)
}

pub fn report(directory: &Path, output: &Path) -> Result<(), String> {
    let report = evaluate(directory)?;
    use std::io::Write;
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| e.to_string())?
        .write_all(&serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correction_or_requires_both_real_answers_and_rejects_foreign_options() {
        let request = json!({"payload":{"questions":{"relevance":{"criteria":{"A":"yes","none":"no"}},"context_contrast":{"criteria":{"A":"yes","none":"no"}}}}});
        let a = |c| json!({"type":"choice","choice":c});
        let mut body = json!({"answers":{"relevance":a("none"),"context_contrast":a("A")}});
        assert_eq!(choice(&body, &request, true).unwrap(), true);
        body["answers"]["context_contrast"]["choice"] = json!("none");
        assert!(!choice(&body, &request, true).unwrap());
        body["answers"]["context_contrast"]["choice"] = json!("B");
        assert!(choice(&body, &request, true).is_err());
        body["answers"]
            .as_object_mut()
            .unwrap()
            .remove("context_contrast");
        assert!(choice(&body, &request, true).is_err());
    }
}
