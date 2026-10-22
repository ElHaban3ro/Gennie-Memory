//! Independent JSON audit of the opt-in proposal/isolated-verification contract.
use serde_json::{Value, json};

fn array<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    v[key].as_array().ok_or_else(|| format!("falta {key}"))
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("falta {key}"))
}
fn optional_tokens(v: &Value, key: &str) -> Result<Option<u64>, String> {
    match v.get(key) {
        Some(Value::Null) => Ok(None),
        Some(n) => n
            .as_u64()
            .map(Some)
            .ok_or_else(|| format!("{key} inválido")),
        None => Err(format!("falta {key}")),
    }
}
fn optional_cost(v: &Value, key: &str) -> Result<Option<f64>, String> {
    match v.get(key) {
        Some(Value::Null) => Ok(None),
        Some(n) => n
            .as_f64()
            .filter(|n| n.is_finite() && *n >= 0.0)
            .map(Some)
            .ok_or_else(|| format!("{key} inválido")),
        None => Err(format!("falta {key}")),
    }
}

pub fn inspect(response: &Value, proposer: &str) -> Result<Value, String> {
    inspect_mode(response, proposer, "per_memory_verified_v1")
}
pub fn inspect_corrections(response: &Value, proposer: &str) -> Result<Value, String> {
    inspect_mode(response, proposer, "per_memory_correction_verified_v1")
}
pub fn inspect_corrections_v2(response: &Value, proposer: &str) -> Result<Value, String> {
    inspect_mode(response, proposer, "per_memory_correction_verified_v2")
}
pub fn inspect_corrections_v3(response: &Value, proposer: &str) -> Result<Value, String> {
    inspect_mode(response, proposer, "per_memory_correction_verified_v3")
}
pub fn inspect_corrections_v4(response: &Value, proposer: &str) -> Result<Value, String> {
    inspect_mode(response, proposer, "per_memory_correction_verified_v4")
}
pub fn inspect_corrections_v5(response: &Value, proposer: &str) -> Result<Value, String> {
    inspect_mode(response, proposer, "per_memory_correction_verified_v5")
}
pub fn inspect_roles(response: &Value, proposer: &str) -> Result<Value, String> {
    inspect_mode(response, proposer, "per_memory_role_verified_v1")
}
pub fn inspect_roles_v2(response: &Value, proposer: &str) -> Result<Value, String> {
    inspect_mode(response, proposer, "per_memory_role_verified_v2")
}
pub fn inspect_role_batch(response: &Value, proposer: &str) -> Result<Value, String> {
    inspect_mode(response, proposer, "per_memory_role_batch_v1")
}
pub fn inspect_complete_premise(response: &Value, proposer: &str) -> Result<Value, String> {
    inspect_mode(response, proposer, "per_memory_complete_premise_v1")
}
fn inspect_mode(response: &Value, proposer: &str, policy: &str) -> Result<Value, String> {
    let complete = policy == "per_memory_complete_premise_v1";
    let corrections_only = policy != "per_memory_verified_v1";
    if response["status"] != "complete"
        || response["requested_model"] != proposer
        || response["seed"]["classifier"] != policy
    {
        return Err("política/modelo o estado de verificación incorrecto".into());
    }
    let candidates = array(&response["retrieval"], "candidates")?;
    if response["classification_status"] == "skipped_empty" {
        if !candidates.is_empty()
            || !response["per_memory"].is_null()
            || response["selected_ids"] != json!([])
            || !response["selected_id"].is_null()
            || response["seed"]["memories"] != json!([])
            || !response["served_model"].is_null()
            || optional_cost(response, "reported_cost_usd")?.is_some()
            || optional_tokens(response, "input_tokens")?.is_some()
            || optional_tokens(response, "output_tokens")?.is_some()
        {
            return Err("búsqueda vacía con selección, traza o consumo incompatible".into());
        }
        return Ok(
            json!({"skipped_empty":true,"proposed_ids":[],"accepted_ids":[],"rejected_ids":[],"http_requests":0,"stage_models":[],"usage":{"known_cost_usd":0.0,"reported_total_cost_usd":null,"missing_cost_receipts":0,"input_tokens":null,"output_tokens":null}}),
        );
    }
    if candidates.is_empty() {
        return Err("búsqueda vacía sin skipped_empty".into());
    }
    let trace = &response["per_memory"];
    let final_items = array(trace, "items")?;
    let verification = &trace["verification"];
    let proposal = array(verification, "proposal")?;
    let individual_checks = array(verification, "checks")?;
    let batched = policy == "per_memory_role_batch_v1";
    let batch = verification.get("batch").filter(|b| !b.is_null());
    if !batched && batch.is_some() || batched && !individual_checks.is_empty() {
        return Err("esquemas individuales/lote incompatibles".into());
    }
    let checks = if let Some(b) = batch {
        if b.as_object().is_none_or(|b| b.len() != 2) {
            return Err("metadatos de lote ajenos".into());
        }
        let verdicts = array(b, "verdicts")?;
        if verdicts.is_empty() {
            return Err("lote vacío".into());
        }
        verdicts
    } else {
        individual_checks
    };
    if proposal.len() != candidates.len().min(if corrections_only { 7 } else { 8 })
        || final_items.len() != proposal.len()
    {
        return Err("propuesta incompleta".into());
    }
    let kinds = if matches!(
        policy,
        "per_memory_role_verified_v1"
            | "per_memory_role_verified_v2"
            | "per_memory_role_batch_v1"
            | "per_memory_complete_premise_v1"
    ) {
        let rows = array(verification, "kind_proposal")?;
        if rows.len() != proposal.len()
            || !verification["scoped_proposal"].is_null()
            || !verification["role_proposal"].is_null()
        {
            return Err("tipos incompletos o mezclados".into());
        }
        Some(rows)
    } else {
        if !verification["kind_proposal"].is_null() {
            return Err("tipos en otra política".into());
        }
        None
    };
    let roles = if policy == "per_memory_correction_verified_v5" {
        let rows = array(verification, "role_proposal")?;
        if rows.len() != proposal.len() || !verification["scoped_proposal"].is_null() {
            return Err("roles incompletos o mezclados".into());
        }
        Some(rows)
    } else {
        if !verification["role_proposal"].is_null() {
            return Err("roles en política histórica".into());
        }
        None
    };
    let scoped = if corrections_only && roles.is_none() && kinds.is_none() {
        let rows = array(verification, "scoped_proposal")?;
        if rows.len() != proposal.len() {
            return Err("factores incompletos".into());
        }
        Some(rows)
    } else {
        if !verification["scoped_proposal"].is_null() {
            return Err("factores en otra política".into());
        }
        None
    };
    let mut positive = Vec::new();
    let mut direct = Vec::new();
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    let mut checked = 0;
    for (index, ((p, c), final_item)) in
        proposal.iter().zip(candidates).zip(final_items).enumerate()
    {
        if p["memory_id"] != c["memory_id"]
            || p["revision"] != c["revision"]
            || final_item["memory_id"] != p["memory_id"]
            || final_item["revision"] != p["revision"]
        {
            return Err("ID/revisión de propuesta o final distinto".into());
        }
        let fact = if let Some(kinds) = kinds {
            let row = &kinds[index];
            if row.as_object().is_none_or(|r| r.len() != 3) {
                return Err("metadatos de tipo ajenos".into());
            }
            let positive = match text(row, "evidence_kind")? {
                "requested_fact_or_partial" | "premise_correction" => true,
                "unrelated" => false,
                _ => return Err("tipo desconocido".into()),
            };
            if row["memory_id"] != p["memory_id"]
                || row["revision"] != p["revision"]
                || p["relation"]
                    != if positive {
                        "useful_evidence"
                    } else {
                        "unrelated"
                    }
            {
                return Err("tipo y propuesta incompatibles".into());
            }
            false
        } else if let Some(roles) = roles {
            let row = &roles[index];
            let context = row["context_for_fact"]
                .as_bool()
                .ok_or("contexto no binario")?;
            let (fact, correction) = match text(row, "evidence_kind")? {
                "requested_fact_or_partial" => (context, false),
                "premise_correction" => (false, true),
                "unrelated" => (false, false),
                _ => return Err("rol desconocido".into()),
            };
            if row["memory_id"] != p["memory_id"]
                || row["revision"] != p["revision"]
                || p["relation"]
                    != if fact || correction {
                        "useful_evidence"
                    } else {
                        "unrelated"
                    }
            {
                return Err("rol/contexto y propuesta incompatibles".into());
            }
            fact
        } else if let Some(scoped) = scoped {
            let row = &scoped[index];
            let fact = row["requested_fact_same_context"]
                .as_bool()
                .ok_or("dato no binario")?;
            let correction = row["premise_correction_same_context"]
                .as_bool()
                .ok_or("corrección no binaria")?;
            if row["memory_id"] != p["memory_id"]
                || row["revision"] != p["revision"]
                || p["relation"]
                    != if fact || correction {
                        "useful_evidence"
                    } else {
                        "unrelated"
                    }
            {
                return Err("factores y propuesta incompatibles".into());
            }
            fact
        } else {
            false
        };
        let expected = match text(p, "relation")? {
            "unrelated" => "unrelated",
            "useful_evidence" => {
                positive.push(p["memory_id"].clone());
                let useful = if fact {
                    direct.push(p["memory_id"].clone());
                    true
                } else {
                    let check = checks.get(checked).ok_or("falta verificación")?;
                    checked += 1;
                    if check["memory_id"] != p["memory_id"] || check["revision"] != p["revision"] {
                        return Err("verificación ajena o fuera de orden".into());
                    }
                    let useful = check["useful"].as_bool().ok_or("juicio no binario")?;
                    if batched
                        && check
                            .as_object()
                            .is_none_or(|c| c.len() != if check["paths"].is_null() { 3 } else { 4 })
                    {
                        return Err("recibo o campo ajeno dentro de veredicto de lote".into());
                    }
                    if complete {
                        if !check["paths"].is_null() {
                            return Err("vías históricas en comprobación completa".into());
                        }
                        let p = check["complete_paths"]
                            .as_object()
                            .filter(|p| p.len() == 4)
                            .ok_or("faltan cuatro vías completas")?;
                        let b = |k: &str| {
                            p.get(k)
                                .and_then(Value::as_bool)
                                .ok_or("vía completa no binaria")
                        };
                        let accepted = b("requested_fact_or_partial")?
                            || b("ordinary_correction")? && b("necessary_premise_incompatible")?
                            || b("explicit_context_contrast")?;
                        // Read every field even when OR short-circuits.
                        for k in [
                            "requested_fact_or_partial",
                            "ordinary_correction",
                            "necessary_premise_incompatible",
                            "explicit_context_contrast",
                        ] {
                            b(k)?;
                        }
                        if useful != accepted {
                            return Err("composición completa incorrecta".into());
                        }
                    } else if !check["complete_paths"].is_null() {
                        return Err("vías completas en otra política".into());
                    } else if matches!(
                        policy,
                        "per_memory_correction_verified_v4" | "per_memory_correction_verified_v5"
                    ) || kinds
                        .is_some_and(|r| r[index]["evidence_kind"] == "premise_correction")
                    {
                        let paths = check["paths"]
                            .as_object()
                            .filter(|p| p.len() == 2)
                            .ok_or("faltan vías tipadas")?;
                        let ordinary = paths
                            .get("ordinary_correction")
                            .and_then(Value::as_bool)
                            .ok_or("corrección no binaria")?;
                        let contrast = paths
                            .get("explicit_context_contrast")
                            .and_then(Value::as_bool)
                            .ok_or("contraste no binario")?;
                        if useful != (ordinary || contrast) {
                            return Err("composición de vías incorrecta".into());
                        }
                    } else if !check["paths"].is_null() {
                        return Err("vías en política histórica".into());
                    }
                    useful
                };
                if useful {
                    accepted.push(p["memory_id"].clone());
                    "useful_evidence"
                } else {
                    rejected.push(p["memory_id"].clone());
                    "unrelated"
                }
            }
            _ => return Err("propuesta no binaria".into()),
        };
        if final_item["relation"] != expected {
            return Err("resultado difiere de composición".into());
        }
    }
    if checks.len() != checked || response["selected_ids"] != json!(accepted) {
        return Err("verificaciones adicionales o semilla distinta".into());
    }
    let receipts: Vec<_> = std::iter::once(&verification["proposal_usage"])
        .chain(individual_checks.iter().map(|c| &c["usage"]))
        .chain(batch.into_iter().map(|b| &b["usage"]))
        .collect();
    let mut known_cost = None;
    let mut missing = 0;
    let mut input = 0_u64;
    let mut output = 0_u64;
    let mut input_complete = true;
    let mut output_complete = true;
    let mut models = Vec::new();
    let mut served = Vec::new();
    for (i, receipt) in receipts.iter().enumerate() {
        let requested = text(receipt, "requested_model")?;
        if requested
            != if i == 0 {
                proposer
            } else if complete {
                "perplexity/pplx-decider-v1.2"
            } else {
                "openai/gpt-6-luna-decisions"
            }
        {
            return Err("modelo solicitado de etapa incorrecto".into());
        }
        let model = match receipt.get("served_model") {
            Some(Value::Null) => None,
            Some(Value::String(s)) if !s.trim().is_empty() && s.len() <= 128 => Some(s.as_str()),
            _ => return Err("modelo servido inválido".into()),
        };
        if complete && i > 0 && model != Some("perplexity/pplx-decider-v1.2-20261016") {
            return Err("revisión de verificador completa distinta".into());
        }
        served.push(model);
        models.push(json!({"requested_model":requested,"served_model":model}));
        match optional_cost(receipt, "reported_cost_usd")? {
            Some(cost) => {
                let sum = known_cost.unwrap_or(0.0) + cost;
                if !sum.is_finite() {
                    return Err("desbordamiento coste".into());
                }
                known_cost = Some(sum);
            }
            None => missing += 1,
        }
        match optional_tokens(receipt, "input_tokens")? {
            Some(n) => input = input.checked_add(n).ok_or("desbordamiento tokens")?,
            None => input_complete = false,
        };
        match optional_tokens(receipt, "output_tokens")? {
            Some(n) => output = output.checked_add(n).ok_or("desbordamiento tokens")?,
            None => output_complete = false,
        };
    }
    let cost = if missing == 0 { known_cost } else { None };
    let input = input_complete.then_some(input);
    let output = output_complete.then_some(output);
    let same_model = if served.iter().all(|m| *m == served[0]) {
        served[0]
    } else {
        None
    };
    let actual_cost = optional_cost(response, "reported_cost_usd")?;
    let cost_matches = match (actual_cost, cost) {
        (None, None) => true,
        (Some(a), Some(b)) => (a - b).abs() <= 1e-12,
        _ => false,
    };
    if !cost_matches
        || optional_tokens(response, "input_tokens")? != input
        || optional_tokens(response, "output_tokens")? != output
        || response["served_model"] != json!(same_model)
    {
        return Err("agregado de recibos incorrecto o parcial presentado completo".into());
    }
    Ok(
        json!({"proposed_ids":positive,"direct_fact_ids":direct,"checked_ids":checks.iter().map(|c|c["memory_id"].clone()).collect::<Vec<_>>(),"accepted_ids":accepted,"rejected_ids":rejected,"http_requests":receipts.len(),"stage_models":models,
        "usage":{"known_cost_usd":known_cost,"reported_total_cost_usd":cost,"missing_cost_receipts":missing,"input_tokens":input,"output_tokens":output}}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revised_pilots_require_an_explicit_recognized_source_group() {
        assert!(!v4_regression(&json!({"policy":"per_memory_correction_verified_v1"})).unwrap());
        assert!(v4_regression(&json!({"policy":"per_memory_correction_verified_v2"})).is_err());
        assert!(
            v4_regression(
                &json!({"policy":"per_memory_correction_verified_v2","pilot_kind":"other"})
            )
            .is_err()
        );
        assert!(
            v4_regression(
                &json!({"policy":"per_memory_correction_verified_v2","pilot_kind":"v4_regression"})
            )
            .unwrap()
        );
        assert!(!v4_regression(&json!({"policy":"per_memory_correction_verified_v2","pilot_kind":"v5_preservation"})).unwrap());
    }
    #[test]
    fn only_absent_policy_uses_the_historical_default() {
        assert_eq!(pilot_policy(&json!({})).unwrap(), "per_memory_verified_v1");
        assert!(pilot_policy(&json!({"policy":null})).is_err());
        assert!(pilot_policy(&json!({"policy":3})).is_err());
        assert_eq!(
            pilot_policy(&json!({"policy":"per_memory_binary_v2"})).unwrap(),
            "per_memory_binary_v2"
        );
    }
    #[test]
    fn pilot_must_bind_every_required_source_not_only_supplied_hashes() {
        let hash = "a".repeat(64);
        let mut spec = json!({"source_cases":"source","corpus":"corpus","baseline_journal":"baseline","inputs":{"source":hash,"corpus":hash,"baseline":hash,"pilot/cases.jsonl":hash}});
        assert!(bound_inputs(&spec, std::path::Path::new("pilot")).is_ok());
        spec["inputs"].as_object_mut().unwrap().remove("baseline");
        assert!(bound_inputs(&spec, std::path::Path::new("pilot")).is_err());
        spec["inputs"] = json!({});
        assert!(bound_inputs(&spec, std::path::Path::new("pilot")).is_err());
    }
    #[test]
    fn empty_search_requires_no_selection_trace_or_receipt() {
        let mut response = json!({"status":"complete","classification_status":"skipped_empty","requested_model":"typesafe/jev-1.13","seed":{"classifier":"per_memory_verified_v1","memories":[]},"selected_ids":[],"selected_id":null,"served_model":null,"input_tokens":null,"output_tokens":null,"reported_cost_usd":null,"retrieval":{"candidates":[]},"per_memory":null});
        let audit = inspect(&response, "typesafe/jev-1.13").unwrap();
        assert_eq!(audit["http_requests"], 0);
        assert_eq!(audit["usage"]["missing_cost_receipts"], 0);
        response["selected_ids"] = json!(["a"]);
        assert!(inspect(&response, "typesafe/jev-1.13").is_err());
        response["selected_ids"] = json!([]);
        response["reported_cost_usd"] = json!(0);
        assert!(inspect(&response, "typesafe/jev-1.13").is_err());
    }
    #[test]
    fn role_audit_requires_all_fact_and_correction_checks_and_exclusive_path_types() {
        let u = |m: &str| json!({"requested_model":m,"served_model":m,"input_tokens":10,"output_tokens":0,"reported_cost_usd":0.001});
        let items = json!([{"memory_id":"a","revision":1,"relation":"useful_evidence"},{"memory_id":"b","revision":1,"relation":"useful_evidence"},{"memory_id":"c","revision":1,"relation":"unrelated"}]);
        let mut r = json!({"status":"complete","requested_model":"typesafe/jev-1.13","seed":{"classifier":"per_memory_role_verified_v1"},"selected_ids":["a","b"],"served_model":null,"input_tokens":30,"output_tokens":0,"reported_cost_usd":0.003,"retrieval":{"candidates":[{"memory_id":"a","revision":1},{"memory_id":"b","revision":1},{"memory_id":"c","revision":1}]},"per_memory":{"items":items.clone(),"verification":{"proposal":items,"kind_proposal":[{"memory_id":"a","revision":1,"evidence_kind":"requested_fact_or_partial"},{"memory_id":"b","revision":1,"evidence_kind":"premise_correction"},{"memory_id":"c","revision":1,"evidence_kind":"unrelated"}],"proposal_usage":u("typesafe/jev-1.13"),"checks":[{"memory_id":"a","revision":1,"useful":true,"usage":u("openai/gpt-6-luna-decisions")},{"memory_id":"b","revision":1,"useful":true,"paths":{"ordinary_correction":false,"explicit_context_contrast":true},"usage":u("openai/gpt-6-luna-decisions")}]}}});
        let a = inspect_roles(&r, "typesafe/jev-1.13").unwrap();
        assert_eq!(a["direct_fact_ids"], json!([]));
        assert_eq!(a["checked_ids"], json!(["a", "b"]));
        assert_eq!(a["http_requests"], 3);
        let mut complete = r.clone();
        complete["seed"]["classifier"] = json!("per_memory_complete_premise_v1");
        for check in complete["per_memory"]["verification"]["checks"]
            .as_array_mut()
            .unwrap()
        {
            check.as_object_mut().unwrap().remove("paths");
            check["complete_paths"] = json!({"requested_fact_or_partial":true,"ordinary_correction":false,"necessary_premise_incompatible":false,"explicit_context_contrast":false});
            check["usage"]["requested_model"] = json!("perplexity/pplx-decider-v1.2");
            check["usage"]["served_model"] = json!("perplexity/pplx-decider-v1.2-20261016");
        }
        assert!(inspect_complete_premise(&complete, "typesafe/jev-1.13").is_ok());
        let mut invalid = complete.clone();
        invalid["per_memory"]["verification"]["checks"][0]["complete_paths"]["requested_fact_or_partial"] =
            json!(false);
        assert!(inspect_complete_premise(&invalid, "typesafe/jev-1.13").is_err());
        let mut missing = complete.clone();
        missing["per_memory"]["verification"]["checks"][0]["complete_paths"]
            .as_object_mut()
            .unwrap()
            .remove("necessary_premise_incompatible");
        assert!(inspect_complete_premise(&missing, "typesafe/jev-1.13").is_err());
        let mut batch = r.clone();
        let mut verdicts = batch["per_memory"]["verification"]["checks"].clone();
        for c in verdicts.as_array_mut().unwrap() {
            c.as_object_mut().unwrap().remove("usage");
        }
        batch["seed"]["classifier"] = json!("per_memory_role_batch_v1");
        batch["per_memory"]["verification"]["batch"] =
            json!({"usage":u("openai/gpt-6-luna-decisions"),"verdicts":verdicts});
        assert!(inspect_role_batch(&batch, "typesafe/jev-1.13").is_err());
        batch["per_memory"]["verification"]["checks"] = json!([]);
        batch["input_tokens"] = json!(20);
        batch["reported_cost_usd"] = json!(0.002);
        let audit = inspect_role_batch(&batch, "typesafe/jev-1.13").unwrap();
        assert_eq!(audit["http_requests"], 2);
        assert_eq!(audit["checked_ids"], json!(["a", "b"]));
        assert!(inspect_roles(&batch, "typesafe/jev-1.13").is_err());
        batch["per_memory"]["verification"]["batch"]["verdicts"][0]["usage"] =
            u("openai/gpt-6-luna-decisions");
        assert!(inspect_role_batch(&batch, "typesafe/jev-1.13").is_err());
        assert!(inspect_roles_v2(&r, "typesafe/jev-1.13").is_err());
        r["seed"]["classifier"] = json!("per_memory_role_verified_v2");
        assert!(inspect_roles_v2(&r, "typesafe/jev-1.13").is_ok());
        assert!(inspect_roles(&r, "typesafe/jev-1.13").is_err());
        r["seed"]["classifier"] = json!("per_memory_role_verified_v1");
        let mut missing = r.clone();
        missing["per_memory"]["verification"]["checks"]
            .as_array_mut()
            .unwrap()
            .remove(0);
        assert!(inspect_roles(&missing, "typesafe/jev-1.13").is_err());
        r["per_memory"]["verification"]["checks"][0]["paths"] =
            json!({"ordinary_correction":true,"explicit_context_contrast":false});
        assert!(inspect_roles(&r, "typesafe/jev-1.13").is_err());
        r["per_memory"]["verification"]["checks"][0]["paths"] = Value::Null;
        r["per_memory"]["verification"]["kind_proposal"][0]["context_for_fact"] = json!(true);
        assert!(inspect_roles(&r, "typesafe/jev-1.13").is_err());
    }
    #[test]
    fn contextual_audit_rejects_wrong_context_facts_and_keeps_a_linked_correction() {
        let u = |m: &str| json!({"requested_model":m,"served_model":m,"input_tokens":10,"output_tokens":0,"reported_cost_usd":0.001});
        let items = json!([{"memory_id":"a","revision":1,"relation":"useful_evidence"},{"memory_id":"b","revision":1,"relation":"unrelated"},{"memory_id":"c","revision":1,"relation":"useful_evidence"}]);
        let mut r = json!({"status":"complete","requested_model":"typesafe/jev-1.13","seed":{"classifier":"per_memory_correction_verified_v5"},"selected_ids":["a","c"],"served_model":null,"input_tokens":20,"output_tokens":0,"reported_cost_usd":0.002,"retrieval":{"candidates":[{"memory_id":"a","revision":1},{"memory_id":"b","revision":1},{"memory_id":"c","revision":1}]},"per_memory":{"items":items.clone(),"verification":{"proposal":items,"role_proposal":[{"memory_id":"a","revision":1,"context_for_fact":true,"evidence_kind":"requested_fact_or_partial"},{"memory_id":"b","revision":1,"context_for_fact":false,"evidence_kind":"requested_fact_or_partial"},{"memory_id":"c","revision":1,"context_for_fact":false,"evidence_kind":"premise_correction"}],"proposal_usage":u("typesafe/jev-1.13"),"checks":[{"memory_id":"c","revision":1,"useful":true,"paths":{"ordinary_correction":false,"explicit_context_contrast":true},"usage":u("openai/gpt-6-luna-decisions")}]}}});
        let audit = inspect_corrections_v5(&r, "typesafe/jev-1.13").unwrap();
        assert_eq!(audit["direct_fact_ids"], json!(["a"]));
        assert_eq!(audit["checked_ids"], json!(["c"]));
        assert_eq!(audit["http_requests"], 2);
        r["per_memory"]["verification"]["role_proposal"][1]["context_for_fact"] = json!(true);
        assert!(inspect_corrections_v5(&r, "typesafe/jev-1.13").is_err());
        r["per_memory"]["verification"]["role_proposal"][1]["context_for_fact"] = json!(false);
        r["per_memory"]["verification"]["checks"][0]["paths"]["explicit_context_contrast"] =
            json!(false);
        assert!(inspect_corrections_v5(&r, "typesafe/jev-1.13").is_err());
    }
    #[test]
    fn correction_audit_checks_only_correction_proposals_and_counts_every_receipt() {
        let usage = |model: &str| json!({"requested_model":model,"served_model":model,"input_tokens":10,"output_tokens":0,"reported_cost_usd":0.001});
        let items = json!([{"memory_id":"a","revision":1,"relation":"useful_evidence"},{"memory_id":"b","revision":1,"relation":"useful_evidence"}]);
        let mut response = json!({"status":"complete","requested_model":"typesafe/jev-1.13","seed":{"classifier":"per_memory_correction_verified_v1"},"selected_ids":["a"],"served_model":null,"input_tokens":20,"output_tokens":0,"reported_cost_usd":0.002,"retrieval":{"candidates":[{"memory_id":"a","revision":1},{"memory_id":"b","revision":1}]},"per_memory":{"items":items.clone(),"verification":{"proposal":items,"scoped_proposal":[{"memory_id":"a","revision":1,"requested_fact_same_context":true,"premise_correction_same_context":true},{"memory_id":"b","revision":1,"requested_fact_same_context":false,"premise_correction_same_context":true}],"proposal_usage":usage("typesafe/jev-1.13"),"checks":[{"memory_id":"b","revision":1,"useful":false,"usage":usage("openai/gpt-6-luna-decisions")}]}}});
        response["per_memory"]["items"][1]["relation"] = json!("unrelated");
        let audit = inspect_corrections(&response, "typesafe/jev-1.13").unwrap();
        assert_eq!(audit["direct_fact_ids"], json!(["a"]));
        assert_eq!(audit["checked_ids"], json!(["b"]));
        assert_eq!(audit["http_requests"], 2);
        assert_eq!(audit["usage"]["known_cost_usd"], 0.002);
        assert!(inspect(&response, "typesafe/jev-1.13").is_err());
        assert!(inspect_corrections_v2(&response, "typesafe/jev-1.13").is_err());
        response["seed"]["classifier"] = json!("per_memory_correction_verified_v2");
        assert!(inspect_corrections_v2(&response, "typesafe/jev-1.13").is_ok());
        assert!(inspect_corrections(&response, "typesafe/jev-1.13").is_err());
        assert!(inspect_corrections_v3(&response, "typesafe/jev-1.13").is_err());
        response["seed"]["classifier"] = json!("per_memory_correction_verified_v3");
        assert!(inspect_corrections_v3(&response, "typesafe/jev-1.13").is_ok());
        assert!(inspect_corrections_v2(&response, "typesafe/jev-1.13").is_err());
        response["seed"]["classifier"] = json!("per_memory_correction_verified_v4");
        assert!(inspect_corrections_v4(&response, "typesafe/jev-1.13").is_err());
        response["per_memory"]["verification"]["checks"][0]["paths"] =
            json!({"ordinary_correction":false,"explicit_context_contrast":false});
        assert!(inspect_corrections_v4(&response, "typesafe/jev-1.13").is_ok());
        response["per_memory"]["verification"]["checks"][0]["paths"]["explicit_context_contrast"] =
            json!(true);
        assert!(inspect_corrections_v4(&response, "typesafe/jev-1.13").is_err());
        response["per_memory"]["verification"]["checks"][0]["paths"] = Value::Null;
        response["seed"]["classifier"] = json!("per_memory_correction_verified_v1");
        response["per_memory"]["verification"]["checks"][0]["memory_id"] = json!("a");
        assert!(inspect_corrections(&response, "typesafe/jev-1.13").is_err());
        response["per_memory"]["verification"]["checks"][0]["memory_id"] = json!("b");
        response["per_memory"]["verification"]["proposal_usage"]["reported_cost_usd"] = Value::Null;
        assert!(inspect_corrections(&response, "typesafe/jev-1.13").is_err());
        response["reported_cost_usd"] = Value::Null;
        let audit = inspect_corrections(&response, "typesafe/jev-1.13").unwrap();
        assert_eq!(audit["usage"]["known_cost_usd"], 0.001);
        assert_eq!(audit["usage"]["missing_cost_receipts"], 1);
    }
    #[test]
    fn incomplete_receipts_cannot_pass_as_full_cost_or_hide_wrong_checks() {
        let usage = |model: &str, cost: Value| json!({"requested_model":model,"served_model":model,"input_tokens":10,"output_tokens":0,"reported_cost_usd":cost});
        let item = json!({"memory_id":"a","revision":1,"relation":"useful_evidence"});
        let mut response = json!({"status":"complete","requested_model":"typesafe/jev-1.13","seed":{"classifier":"per_memory_verified_v1"},"selected_ids":["a"],"served_model":null,"input_tokens":20,"output_tokens":0,"reported_cost_usd":null,"retrieval":{"candidates":[{"memory_id":"a","revision":1}]},"per_memory":{"items":[item.clone()],"verification":{"proposal":[item],"proposal_usage":usage("typesafe/jev-1.13",json!(0.001)),"checks":[{"memory_id":"a","revision":1,"useful":true,"usage":usage("openai/gpt-6-luna-decisions",Value::Null)}]}}});
        let audit = inspect(&response, "typesafe/jev-1.13").unwrap();
        assert_eq!(audit["http_requests"], 2);
        assert_eq!(audit["usage"]["known_cost_usd"], 0.001);
        assert_eq!(audit["usage"]["missing_cost_receipts"], 1);
        response["reported_cost_usd"] = json!(0.001);
        assert!(inspect(&response, "typesafe/jev-1.13").is_err());
        response["reported_cost_usd"] = Value::Null;
        response["per_memory"]["verification"]["checks"][0]["revision"] = json!(2);
        assert!(inspect(&response, "typesafe/jev-1.13").is_err());
    }
}

fn read(path: &std::path::Path) -> Result<Value, String> {
    serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn lines(path: &std::path::Path) -> Result<Vec<Value>, String> {
    std::fs::read_to_string(path)
        .map_err(|e| e.to_string())?
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
        .collect()
}

fn bound_inputs(spec: &Value, directory: &std::path::Path) -> Result<(), String> {
    let inputs = spec["inputs"]
        .as_object()
        .ok_or("faltan entradas congeladas")?;
    let local_cases = directory.join("cases.jsonl");
    for path in [
        text(spec, "source_cases")?,
        text(spec, "corpus")?,
        text(spec, "baseline_journal")?,
        local_cases.to_str().ok_or("ruta inválida")?,
    ] {
        if !inputs
            .get(path)
            .is_some_and(|hash| hash.as_str().is_some_and(|h| h.len() == 64))
        {
            return Err(format!("fuente obligatoria sin huella: {path}"));
        }
    }
    Ok(())
}

fn pilot_policy(spec: &Value) -> Result<&str, String> {
    let policy = match spec.get("policy") {
        None => "per_memory_verified_v1",
        Some(value) => value
            .as_str()
            .ok_or("tipo de política de piloto inválido")?,
    };
    match policy {
        p @ ("per_memory_complete_premise_v1"
        | "per_memory_role_batch_v1"
        | "per_memory_role_verified_v2"
        | "per_memory_role_verified_v1"
        | "per_memory_correction_verified_v5"
        | "per_memory_correction_verified_v4"
        | "per_memory_correction_verified_v3"
        | "per_memory_correction_verified_v2"
        | "per_memory_correction_verified_v1"
        | "per_memory_verified_v1"
        | "per_memory_binary_v2"
        | "per_memory_binary_v3"
        | "per_memory_binary_v4"
        | "per_memory_binary_v5") => Ok(p),
        _ => Err("política de piloto no permitida".into()),
    }
}

fn structured_pilot(policy: &str) -> bool {
    policy.starts_with("per_memory_correction_verified_v")
        || matches!(
            policy,
            "per_memory_role_verified_v1"
                | "per_memory_role_verified_v2"
                | "per_memory_role_batch_v1"
                | "per_memory_complete_premise_v1"
        )
}
fn v4_regression(spec: &Value) -> Result<bool, String> {
    if !matches!(
        pilot_policy(spec)?,
        "per_memory_correction_verified_v2"
            | "per_memory_correction_verified_v3"
            | "per_memory_correction_verified_v4"
            | "per_memory_correction_verified_v5"
            | "per_memory_role_verified_v1"
            | "per_memory_role_verified_v2"
            | "per_memory_role_batch_v1"
            | "per_memory_complete_premise_v1"
    ) {
        return Ok(false);
    }
    match spec["pilot_kind"].as_str() {
        Some("v5_preservation") => Ok(false),
        Some("v4_regression") => Ok(true),
        _ => Err("tipo de pilotoV2ausente o inválido".into()),
    }
}

fn inspect_pilot(response: &Value, policy: &str) -> Result<Value, String> {
    if policy == "per_memory_complete_premise_v1" {
        return inspect_complete_premise(response, "typesafe/jev-1.13");
    }
    if policy == "per_memory_role_batch_v1" {
        return inspect_role_batch(response, "typesafe/jev-1.13");
    }
    if policy == "per_memory_role_verified_v2" {
        return inspect_roles_v2(response, "typesafe/jev-1.13");
    }
    if policy == "per_memory_role_verified_v1" {
        return inspect_roles(response, "typesafe/jev-1.13");
    }
    if policy == "per_memory_correction_verified_v5" {
        return inspect_corrections_v5(response, "typesafe/jev-1.13");
    }
    if policy == "per_memory_correction_verified_v4" {
        return inspect_corrections_v4(response, "typesafe/jev-1.13");
    }
    if policy == "per_memory_correction_verified_v3" {
        return inspect_corrections_v3(response, "typesafe/jev-1.13");
    }
    if policy == "per_memory_correction_verified_v2" {
        return inspect_corrections_v2(response, "typesafe/jev-1.13");
    }
    if policy == "per_memory_correction_verified_v1" {
        return inspect_corrections(response, "typesafe/jev-1.13");
    }
    if policy == "per_memory_verified_v1" {
        return inspect(response, "typesafe/jev-1.13");
    }
    if response["requested_model"] != "typesafe/jev-1.13"
        || !response["per_memory"]["verification"].is_null()
    {
        return Err("modelo o traza ajenos al piloto binario".into());
    }
    let empty = array(&response["retrieval"], "candidates")?.is_empty();
    if empty
        && (response["classification_status"] != "skipped_empty"
            || !response["reported_cost_usd"].is_null())
    {
        return Err("consumo de búsqueda vacía inválido".into());
    }
    let cost = optional_cost(response, "reported_cost_usd")?;
    Ok(
        json!({"http_requests":if empty {0} else {1},"rejected_ids":[],"stage_models":[{"requested_model":response["requested_model"],"served_model":response["served_model"]}],"usage":{"known_cost_usd":if empty {Some(0.0)} else {cost},"reported_total_cost_usd":cost,"missing_cost_receipts":if empty || cost.is_some(){0}else{1},"input_tokens":response["input_tokens"],"output_tokens":response["output_tokens"]}}),
    )
}

/// Validate the frozen known-case projection before spending any provider calls.
pub fn validate_pilot(directory: &std::path::Path) -> Result<(), String> {
    use crate::campaign::digest;
    let spec = read(&directory.join("pilot.json"))?;
    bound_inputs(&spec, directory)?;
    let policy = pilot_policy(&spec)?;
    if structured_pilot(policy) && !v4_regression(&spec)? {
        let historical = text(&spec, "preservation_journal")?;
        if !spec["inputs"]
            .as_object()
            .is_some_and(|i| i.contains_key(historical))
        {
            return Err("falta huella de evidencia útil anterior".into());
        }
    }
    let operations = match policy {
        "per_memory_correction_verified_v1" => 12,
        "per_memory_correction_verified_v2"
        | "per_memory_correction_verified_v3"
        | "per_memory_correction_verified_v4"
        | "per_memory_correction_verified_v5"
        | "per_memory_role_verified_v1"
        | "per_memory_role_verified_v2"
        | "per_memory_role_batch_v1"
        | "per_memory_complete_premise_v1" => {
            if v4_regression(&spec)? {
                5
            } else {
                13
            }
        }
        "per_memory_binary_v3" => 18,
        "per_memory_binary_v4" | "per_memory_binary_v5" => 34,
        _ => 10,
    };
    let max_http = if policy == "per_memory_role_batch_v1" {
        operations * 2
    } else if structured_pilot(policy) {
        operations * 8
    } else if policy == "per_memory_verified_v1" {
        90
    } else {
        operations
    };
    if spec["operations"] != operations
        || spec["max_http_requests"] != max_http
        || spec["promoted"] != false
    {
        return Err("límites del piloto alterados".into());
    }
    for (path, hash) in spec["inputs"]
        .as_object()
        .ok_or("faltan hashes de entradas")?
    {
        if digest(std::path::Path::new(path))? != hash.as_str().ok_or("hash inválido")? {
            return Err(format!("entrada cambió: {path}"));
        }
    }
    let originals = lines(std::path::Path::new(text(&spec, "source_cases")?))?;
    let cases = lines(&directory.join("cases.jsonl"))?;
    let mut ids = std::collections::BTreeSet::new();
    if cases.len() != operations {
        return Err("cantidad de casos distinta del protocolo".into());
    }
    if structured_pilot(policy) {
        let mut fixed = vec![
            "cev5q_066_01",
            "cev5q_067_06",
            "cev5q_071_04",
            "cev5q_073_03",
            "cev5q_068_05",
            "cev5q_069_06",
            "cev5q_070_05",
            "cev5q_066_07",
            "cev5q_069_07",
            "cev5q_073_08",
            "cev5q_068_03",
            "cev5q_069_05",
        ];
        if matches!(
            policy,
            "per_memory_correction_verified_v2"
                | "per_memory_correction_verified_v3"
                | "per_memory_correction_verified_v4"
                | "per_memory_correction_verified_v5"
                | "per_memory_role_verified_v1"
                | "per_memory_role_verified_v2"
                | "per_memory_role_batch_v1"
                | "per_memory_complete_premise_v1"
        ) {
            if v4_regression(&spec)? {
                fixed = vec![
                    "cev4q_065_04",
                    "cev4q_062_01",
                    "cev4q_061_06",
                    "cev4q_063_05",
                    "cev4q_059_08",
                ];
            } else {
                fixed.push("cev5q_066_03");
            }
        }
        if cases.len() != fixed.len() {
            return Err("roster fijo incompleto".into());
        }
        if cases
            .iter()
            .zip(fixed)
            .any(|(c, id)| c["original_case_id"] != id)
        {
            return Err("casos u orden distintos del protocolo fijo".into());
        }
    }
    for case in &cases {
        let original_id = text(case, "original_case_id")?;
        if !ids.insert(original_id) {
            return Err("caso repetido".into());
        }
        let mut expected = originals
            .iter()
            .find(|c| c["id"] == original_id)
            .ok_or("caso ajeno")?
            .clone();
        expected["id"] = json!(format!("{original_id}__{policy}"));
        expected["original_case_id"] = json!(original_id);
        expected["request"]["provider"] = json!("jev");
        expected["request"]["selection_policy"] = json!(policy);
        if expected != *case {
            return Err("proyección alteró consulta o gold".into());
        }
    }
    Ok(())
}

/// Small known diagnostic; historical comparisons are not paired latency evidence.
pub fn report_pilot(directory: &std::path::Path, output: &std::path::Path) -> Result<(), String> {
    use crate::campaign::digest;
    use std::io::Write;
    validate_pilot(directory)?;
    let spec = read(&directory.join("pilot.json"))?;
    let audit = read(&directory.join("audit.json"))?;
    let campaign = array(&audit, "campaigns")?
        .first()
        .ok_or("falta campaña auditada")?;
    if campaign["integrity"]["valid"] != true {
        return Err("campaña sin integridad".into());
    }
    for (field, path) in [
        ("cases", directory.join("cases.jsonl")),
        ("journal", directory.join("journal.jsonl")),
        ("corpus", std::path::PathBuf::from(text(&spec, "corpus")?)),
    ] {
        if campaign["sha256"][field] != digest(&path)? {
            return Err(format!("auditoría obsoleta: {field}"));
        }
    }
    let cases = lines(&directory.join("cases.jsonl"))?;
    let journal = lines(&directory.join("journal.jsonl"))?;
    let baseline = lines(std::path::Path::new(text(&spec, "baseline_journal")?))?;
    let preservation = if structured_pilot(pilot_policy(&spec)?) && !v4_regression(&spec)? {
        Some(lines(std::path::Path::new(text(
            &spec,
            "preservation_journal",
        )?))?)
    } else {
        None
    };
    let mut rows = Vec::new();
    let mut blocks = Vec::new();
    let mut known_cost = 0.0;
    let mut baseline_cost = 0.0;
    let mut complete_costs = true;
    let mut http_requests = 0_u64;
    let mut gains = Vec::new();
    for case in &cases {
        let matches: Vec<_> = journal
            .iter()
            .filter(|r| r["kind"] == "finished" && r["id"] == case["id"])
            .collect();
        if matches.len() != 1 {
            return Err("resultado ausente o duplicado".into());
        }
        let row = matches[0];
        if row["expected_ids"] != case["expected_ids"] {
            return Err("gold del diario cambió".into());
        }
        let response = &row["response"];
        let baseline_id = format!("{}__baseline", text(case, "original_case_id")?);
        let base = baseline
            .iter()
            .find(|r| r["kind"] == "finished" && r["id"] == baseline_id)
            .ok_or("falta base histórica")?;
        if base["response"]["status"] != "complete" {
            return Err("base histórica degradada".into());
        }
        if base["response"]["retrieval"] != response["retrieval"] {
            return Err("candidatos distintos de la base".into());
        }
        match base["response"]["reported_cost_usd"].as_f64() {
            Some(c) => baseline_cost += c,
            None => complete_costs = false,
        };
        if response["status"] != "complete" {
            complete_costs = false;
            known_cost += response["reported_cost_usd"].as_f64().unwrap_or(0.0);
            blocks.push(
                json!({"id":case["id"],"reason":"degraded","error_code":response["error_code"]}),
            );
            rows.push(json!({"id":case["id"],"status":"degraded","elapsed_ms":row["elapsed_ms"]}));
            continue;
        }
        let stages = inspect_pilot(response, pilot_policy(&spec)?)?;
        let classification = crate::evidence::per_memory_case(
            case,
            response,
            if structured_pilot(pilot_policy(&spec)?) {
                7
            } else {
                8
            },
            true,
        )?;
        known_cost += stages["usage"]["known_cost_usd"].as_f64().unwrap_or(0.0);
        complete_costs &= stages["usage"]["missing_cost_receipts"] == 0;
        http_requests += stages["http_requests"]
            .as_u64()
            .ok_or("faltan peticiones")?;
        let selected = array(response, "selected_ids")?;
        let expected = array(case, "expected_ids")?;
        let wrong: Vec<_> = selected
            .iter()
            .filter(|id| !expected.contains(id))
            .collect();
        let baseline_ids: Vec<_> = base["response"]["seed"]["memories"]
            .as_array()
            .ok_or("falta semilla base")?
            .iter()
            .map(|m| m["memory_id"].clone())
            .filter(|id| expected.contains(id))
            .collect();
        let base_selected = array(&base["response"]["seed"], "memories")?
            .iter()
            .map(|m| m["memory_id"].clone())
            .collect::<Vec<_>>();
        let gained: Vec<_> = selected
            .iter()
            .filter(|id| expected.contains(id) && !base_selected.contains(id))
            .collect();
        let removed: Vec<_> = base_selected
            .iter()
            .filter(|id| !expected.contains(id) && !selected.contains(id))
            .collect();
        if !gained.is_empty() || !removed.is_empty() {
            gains.push(json!({"id":case["id"],"gained_useful_ids":gained,"removed_unpermitted_ids":removed}));
        }
        let prior_ids = if let Some(prior) = &preservation {
            let key = format!(
                "{}__jev_per_memory_scoped_v1",
                text(case, "original_case_id")?
            );
            let row = prior
                .iter()
                .find(|r| r["kind"] == "finished" && r["id"] == key)
                .ok_or("falta selección scoped histórica")?;
            if row["response"]["status"] != "complete"
                || row["response"]["retrieval"] != response["retrieval"]
            {
                return Err("preservación con candidatos distintos o degradada".into());
            }
            array(&row["response"], "selected_ids")?
                .iter()
                .filter(|id| expected.contains(id))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let prior_lost: Vec<_> = prior_ids
            .iter()
            .filter(|id| !selected.contains(id))
            .collect();
        if !prior_lost.is_empty() {
            blocks.push(json!({"id":case["id"],"lost_previous_useful_ids":prior_lost}));
        }
        let lost: Vec<_> = baseline_ids
            .iter()
            .filter(|id| !selected.contains(id))
            .collect();
        if !wrong.is_empty() || !lost.is_empty() {
            blocks.push(json!({"id":case["id"],"wrong_ids":wrong,"lost_baseline_ids":lost}));
        }
        rows.push(json!({"id":case["id"],"original_case_id":case["original_case_id"],"selected_ids":selected,"baseline_permitted_ids":baseline_ids,"previous_permitted_ids":prior_ids,"lost_previous_useful_ids":prior_lost,"wrong_ids":wrong,"lost_baseline_ids":lost,"elapsed_ms":row["elapsed_ms"],"historical_baseline_elapsed_ms":base["elapsed_ms"],"verification":stages,"classification":classification}));
    }
    let ratio = if complete_costs && baseline_cost > 0.0 {
        Some(known_cost / baseline_cost)
    } else {
        None
    };
    let report = json!({"version":"per-memory-known-pilot-v1","policy":pilot_policy(&spec)?,"operations":cases.len(),"successful_http_requests":http_requests,"known_cost_usd":known_cost,"historical_baseline_subset_cost_usd":baseline_cost,"all_cost_receipts_complete":complete_costs,"historical_subset_cost_ratio":ratio,"passes_pilot_quality":blocks.is_empty(),"cost_expansion_blocked":ratio.is_none_or(|r|r>1.5),"historical_gains":gains,"passes_expansion_screen":blocks.is_empty() && (!gains.is_empty() || v4_regression(&spec)?) && ratio.is_some_and(|r|r<=1.5),"targeted_v4_regressions":v4_regression(&spec)?,"blocks":blocks,"cases":rows,"promoted":false,"fresh_validation":false,"warning":"Known selected cases; historical subset costs/times are diagnostic, not paired full-bank evidence. Failed HTTP counts may be unavailable.","journal_sha256":digest(&directory.join("journal.jsonl"))?,"audit_sha256":digest(&directory.join("audit.json"))?});
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    file.write_all(
        serde_json::to_string_pretty(&report)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
}
