//! Conditional alternative task on the complete frozen rejection roster.
use crate::isolated_provider_probe::{choice, cost, evaluate, ids, lines, read};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn combined(first: Option<bool>, opposite: Option<bool>) -> Option<bool> {
    match first {
        Some(true) => Some(true),
        Some(false) => opposite,
        None => None,
    }
}

pub fn report(base: &Path, directory: &Path, output: &Path) -> Result<(), String> {
    let pre = read(&directory.join("preflight.json"))?;
    if pre["operations"] != 25
        || pre["max_http_requests"] != 25
        || pre["requests_sha256"] != crate::campaign::digest(&directory.join("requests.jsonl"))?
    {
        return Err("preflight de vía alternativa alterado".into());
    }
    let hashes = pre["inputs_sha256"].as_object().ok_or("faltan fuentes")?;
    for (p, h) in hashes {
        if crate::campaign::digest(Path::new(p))? != h.as_str().ok_or("hash inválido")? {
            return Err(format!("fuente cambió: {p}"));
        }
    }
    for p in [
        base.join("requests.jsonl"),
        base.join("journal.jsonl"),
        base.join("report-audited.json"),
    ] {
        if !hashes.contains_key(p.to_str().ok_or("ruta inválida")?) {
            return Err("fuente de elegibilidad sin congelar".into());
        }
    }
    let baseline = evaluate(base)?;
    let original_requests = lines(&base.join("requests.jsonl"))?;
    let requests = lines(&directory.join("requests.jsonl"))?;
    let events = lines(&directory.join("journal.jsonl"))?;
    if requests.len() != 25 || events.len() != 50 {
        return Err("roster incompleto o repetido".into());
    }
    let mut records = BTreeMap::new();
    for (r, pair) in requests.iter().zip(events.chunks_exact(2)) {
        let id = r["id"].as_str().ok_or("ID ausente")?;
        if pair[0]["kind"] != "started"
            || pair[0]["id"] != id
            || pair[1]["id"] != id
            || !matches!(pair[1]["kind"].as_str(), Some("response" | "failed"))
        {
            return Err("diario inválido".into());
        }
        if records.insert(id.to_string(), (r, &pair[1])).is_some() {
            return Err("operación repetida".into());
        }
    }
    let mut used = BTreeSet::new();
    let mut total_new = 0.0;
    let mut missing_new = 0;
    let mut group_reports = Vec::new();
    for group in baseline["providers"]["decisions"]["groups"]
        .as_array()
        .ok_or("faltan grupos")?
    {
        let mut rows = Vec::new();
        let mut group_new = 0.0;
        let mut group_missing = 0;
        let mut rescued_ids = Vec::new();
        for case in group["cases"].as_array().ok_or("faltan casos")? {
            let original = case["original_case_id"]
                .as_str()
                .ok_or("ID de caso ausente")?;
            let allowed = ids(&case["allowed_ids"])?;
            let base_allowed = ids(&case["baseline_permitted_ids"])?;
            let previous = ids(&case["previous_permitted_ids"])?;
            let mut selected = BTreeSet::new();
            let mut degraded = false;
            let mut checked = Vec::new();
            for c in case["checks"]
                .as_array()
                .ok_or("faltan primeras comprobaciones")?
            {
                let first = c["selected"]
                    .as_bool()
                    .ok_or("primera etapa fallida; no elegibilidad fiable")?;
                let memory = c["memory_id"].as_str().ok_or("memoria ausente")?;
                let id = format!("{original}_{memory}_decisions");
                let source = original_requests
                    .iter()
                    .find(|r| r["id"] == id)
                    .ok_or("petición original ausente")?;
                if first {
                    selected.insert(memory.to_string());
                    checked.push(json!({"memory_id":memory,"first_selected":true,"opposite_asked":false,"selected":true}));
                    continue;
                }
                let new_id = format!("{id}_opposite");
                let (r, e) = records.get(&new_id).ok_or("rechazo elegible omitido")?;
                used.insert(new_id);
                let correction = c["evidence_kind"] == "requested_fact_or_partial";
                let opposite_kind = if correction {
                    "premise_correction"
                } else {
                    "requested_fact_or_partial"
                };
                let template = original_requests
                    .iter()
                    .find(|r| r["provider"] == "decisions" && r["evidence_kind"] == opposite_kind)
                    .ok_or("template ausente")?;
                if r["base_request_id"] != id
                    || r["original_case_id"] != original
                    || r["cohort"] != group["cohort"]
                    || r["memory_id"] != memory
                    || r["revision"] != c["revision"]
                    || r["original_evidence_kind"] != c["evidence_kind"]
                    || r["correction_task"] != correction
                    || r["payload"]["model"] != "openai/gpt-6-luna-decisions"
                    || r["payload"]["state"] != source["payload"]["state"]
                    || r["payload"]["questions"] != template["payload"]["questions"]
                {
                    return Err("vía alternativa alteró identidad, template o elegibilidad".into());
                }
                let (second, error) = if e["kind"] == "response" {
                    match choice(&e["body"], r, correction) {
                        Ok(v) => (Some(v), None),
                        Err(s) => (None, Some(s)),
                    }
                } else {
                    (None, Some("HTTP failed".into()))
                };
                let value = combined(Some(first), second);
                if value.is_none() {
                    degraded = true;
                }
                if value == Some(true) {
                    selected.insert(memory.to_string());
                    rescued_ids.push(json!({"original_case_id":original,"memory_id":memory,"permitted":allowed.contains(memory)}));
                }
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
                    group_new += f;
                    total_new += f;
                } else {
                    group_missing += 1;
                    missing_new += 1;
                }
                checked.push(json!({"memory_id":memory,"first_selected":false,"opposite_asked":true,"opposite_correction_task":correction,"opposite_selected":second,"selected":value,"technical_error":error,"new_cost_usd":fee,"usage":usage,"elapsed_ms":e["elapsed_ms"],"served_model":e["body"]["model"]}));
            }
            if degraded {
                selected.clear();
            }
            rows.push(json!({"original_case_id":original,"selected_ids":selected,"allowed_ids":allowed,"wrong_ids":selected.difference(&allowed).collect::<Vec<_>>(),"lost_baseline_ids":base_allowed.difference(&selected).collect::<Vec<_>>(),"lost_previous_useful_ids":previous.difference(&selected).collect::<Vec<_>>(),"omitted_allowed_ids":allowed.difference(&selected).collect::<Vec<_>>(),"gained_useful_ids":selected.intersection(&allowed).filter(|id|!base_allowed.contains(*id)).collect::<Vec<_>>(),"degraded":degraded,"all_allowed_selected":selected==allowed,"checks":checked}));
        }
        let quality = rows.iter().all(|c| {
            c["wrong_ids"] == json!([])
                && c["lost_baseline_ids"] == json!([])
                && c["lost_previous_useful_ids"] == json!([])
                && c["degraded"] == false
        }) && if group["cohort"] == "v4" {
            rows.iter().all(|c| c["all_allowed_selected"] == true)
        } else {
            rows.iter().any(|c| c["gained_useful_ids"] != json!([]))
        };
        let first = group["new_verification_known_cost_usd"]
            .as_f64()
            .ok_or("coste previo ausente")?;
        let proposals = group["historical_proposal_cost_usd"]
            .as_f64()
            .ok_or("coste de propuesta ausente")?;
        let base_cost = group["historical_baseline_cost_usd"]
            .as_f64()
            .ok_or("coste de base ausente")?;
        let ratio = (proposals + first + group_new) / base_cost;
        group_reports.push(json!({"cohort":group["cohort"],"cases":rows,"rescued_ids":rescued_ids,"historical_proposal_cost_usd":proposals,"historical_first_verification_cost_usd":first,"new_opposite_task_cost_usd":group_new,"estimated_pipeline_known_cost_usd":proposals+first+group_new,"historical_baseline_cost_usd":base_cost,"estimated_pipeline_cost_ratio":ratio,"missing_new_cost_receipts":group_missing,"passes_replay_quality":quality,"passes_estimated_cost_screen":group_missing==0&&ratio<=1.5}));
    }
    if used.len() != 25 || records.len() != 25 || !total_new.is_finite() {
        return Err("roster no vinculado o coste inválido".into());
    }
    let passes = group_reports
        .iter()
        .all(|g| g["passes_replay_quality"] == true && g["passes_estimated_cost_screen"] == true);
    let result = json!({"version":"opposite-task-probe-v1","operations":25,"http_requests":25,"new_known_cost_usd":total_new,"new_unknown_cost_operations":missing_new,"all_new_cost_receipts_complete":missing_new==0,"passes_bounded_implementation_screen":passes,"groups":group_reports,"requests_sha256":pre["requests_sha256"],"journal_sha256":crate::campaign::digest(&directory.join("journal.jsonl"))?,"base_journal_sha256":crate::campaign::digest(&base.join("journal.jsonl"))?,"fresh_validation":false,"promoted":false,"warning":"Conditional OR replay on all frozen Decisions rejections; only opposite checks are new charges. First verification and proposal costs are reused historical receipts, not new end-to-end measurement."});
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
    fn conditional_or_never_turns_a_technical_failure_into_a_reliable_rejection() {
        assert_eq!(combined(Some(false), Some(true)), Some(true));
        assert_eq!(combined(Some(false), Some(false)), Some(false));
        assert_eq!(combined(Some(false), None), None);
        assert_eq!(combined(None, Some(true)), None);
        assert_eq!(combined(Some(true), None), Some(true));
    }
}
