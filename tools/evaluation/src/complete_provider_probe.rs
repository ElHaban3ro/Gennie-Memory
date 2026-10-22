//! Independent stage, revision, template, utility and cost audit of three models.
use crate::isolated_provider_probe::{cost, evaluate, ids, lines, read};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[cfg(test)]
fn choice(body: &Value) -> Result<(bool, Value), String> {
    choice_mode(body, false)
}
fn choice_mode(body: &Value, premise_gate: bool) -> Result<(bool, Value), String> {
    let answers = body["answers"]
        .as_object()
        .filter(|a| a.len() == if premise_gate { 4 } else { 3 })
        .ok_or("faltan tres respuestas")?;
    let mut paths = serde_json::Map::new();
    let mut keys = vec!["fact", "ordinary_correction", "context_contrast"];
    if premise_gate {
        keys.push("necessary_premise_incompatible");
    }
    for k in keys {
        let a = answers.get(k).ok_or("respuesta ajena")?;
        if a["type"] != "choice" {
            return Err("respuesta no choice".into());
        }
        let yes = match a["choice"].as_str() {
            Some("A") => true,
            Some("none") => false,
            _ => return Err("opción desconocida".into()),
        };
        paths.insert(k.into(), json!(yes));
    }
    let positive = paths["fact"] == true
        || paths["context_contrast"] == true
        || paths["ordinary_correction"] == true
            && (!premise_gate || paths["necessary_premise_incompatible"] == true);
    Ok((positive, json!(paths)))
}

fn atomic_payloads_match(requests: &[Value], question: &Value) -> bool {
    requests.len() == 43
        && requests
            .iter()
            .all(|r| r["payload"]["questions"]["necessary_premise_incompatible"] == *question)
}

pub fn report(directory: &Path, output: &Path) -> Result<(), String> {
    report_mode(directory, output, false)
}
pub fn report_premise(directory: &Path, output: &Path) -> Result<(), String> {
    report_mode(directory, output, true)
}
fn report_mode(directory: &Path, output: &Path, premise_gate: bool) -> Result<(), String> {
    let model_count = if premise_gate { 1 } else { 3 };
    let spec_path = Path::new("tests/fixtures/role-complete-provider-screen-v1.json");
    let spec = read(spec_path)?;
    let pre = read(&directory.join("preflight.json"))?;
    if pre["prepared_operations"] != model_count * 43
        || pre["max_http_requests"] != model_count * 43
        || pre["initial_screen_requests"] != model_count * 12
        || pre["requests_sha256"] != crate::campaign::digest(&directory.join("requests.jsonl"))?
    {
        return Err("preflight alterado".into());
    }
    let inputs = pre["inputs_sha256"].as_object().ok_or("fuentes ausentes")?;
    for (p, h) in inputs {
        if crate::campaign::digest(Path::new(p))? != h.as_str().ok_or("hash inválido")? {
            return Err(format!("fuente cambió: {p}"));
        }
    }
    let base = Path::new(spec["base_directory"].as_str().ok_or("falta fuente")?);
    for p in [
        spec_path.to_path_buf(),
        base.join("requests.jsonl"),
        base.join("journal.jsonl"),
        base.join("report-audited.json"),
    ] {
        if !inputs.contains_key(p.to_str().ok_or("ruta inválida")?) {
            return Err("fuente obligatoria no vinculada".into());
        }
    }
    let source = evaluate(base)?;
    let original = lines(&base.join("requests.jsonl"))?
        .into_iter()
        .filter(|r| r["provider"] == "decisions")
        .collect::<Vec<_>>();
    let fixture_models = spec["models"]
        .as_array()
        .filter(|m| m.len() == 3)
        .ok_or("modelos incompletos")?;
    let fixture_models = &fixture_models[..model_count];
    let catalog_path = Path::new(
        spec["catalog_observation"]
            .as_str()
            .ok_or("catálogo ausente")?,
    );
    if !inputs.contains_key(catalog_path.to_str().ok_or("ruta inválida")?) {
        return Err("catálogo observado no vinculado".into());
    }
    let catalog = read(catalog_path)?;
    let mut model_ids = BTreeSet::new();
    for m in fixture_models {
        let id = m["id"].as_str().ok_or("ID de modelo ausente")?;
        let observed = catalog["models"]
            .as_array()
            .ok_or("catálogo vacío")?
            .iter()
            .find(|o| o["id"] == id)
            .ok_or("modelo fuera del catálogo congelado")?;
        if !model_ids.insert(id) || observed["canonical_slug"] != m["expected_served_model"] {
            return Err("modelo repetido o revisión distinta del catálogo".into());
        }
    }
    let screen = spec["screen"]
        .as_array()
        .filter(|s| s.len() == 12)
        .ok_or("filtro incompleto")?;
    let mut ordered = Vec::new();
    for pair in screen {
        let r = original
            .iter()
            .find(|r| r["original_case_id"] == pair[0] && r["memory_id"] == pair[1])
            .ok_or("control fuera de fuente")?;
        if ordered.iter().any(|x: &&Value| x["id"] == r["id"]) {
            return Err("control repetido".into());
        }
        ordered.push(r);
    }
    for r in &original {
        if !ordered.iter().any(|x| x["id"] == r["id"]) {
            ordered.push(r);
        }
    }
    if ordered.len() != 43 {
        return Err("propuestas incompletas".into());
    }
    let f = original
        .iter()
        .find(|r| r["evidence_kind"] == "requested_fact_or_partial")
        .ok_or("falta tarea de datos")?;
    let c = original
        .iter()
        .find(|r| r["evidence_kind"] == "premise_correction")
        .ok_or("falta tarea de corrección")?;
    let mut questions = json!({"fact":f["payload"]["questions"]["relevance"],"ordinary_correction":c["payload"]["questions"]["relevance"],"context_contrast":c["payload"]["questions"]["context_contrast"]});
    let mut atomic_direct_entry = None;
    if premise_gate {
        let p = Path::new("tests/fixtures/premise-incompatibility-v1.json");
        atomic_direct_entry = Some(inputs.contains_key(p.to_str().unwrap()));
        questions["necessary_premise_incompatible"] = read(p)?;
    }
    let requests = lines(&directory.join("requests.jsonl"))?;
    if requests.len() != model_count * 43 {
        return Err("solicitudes no congeladas completamente".into());
    }
    let mut allowed_by_case = BTreeMap::new();
    for g in source["providers"]["decisions"]["groups"]
        .as_array()
        .ok_or("grupos ausentes")?
    {
        for c in g["cases"].as_array().ok_or("casos ausentes")? {
            allowed_by_case.insert(
                c["original_case_id"].as_str().unwrap().to_string(),
                ids(&c["allowed_ids"])?,
            );
        }
    }
    let mut indices = BTreeMap::new();
    for (mi, m) in fixture_models.iter().enumerate() {
        for (i, old) in ordered.iter().enumerate() {
            let r = &requests[mi * 43 + i];
            let expected = allowed_by_case[old["original_case_id"].as_str().unwrap()]
                .contains(old["memory_id"].as_str().unwrap());
            let expected_id = format!(
                "{}_{}_model{mi}",
                old["original_case_id"].as_str().unwrap(),
                old["memory_id"].as_str().unwrap()
            );
            if r["id"] != expected_id
                || r["model_index"] != mi
                || r["model_id"] != m["id"]
                || r["expected_served_model"] != m["expected_served_model"]
                || r["original_case_id"] != old["original_case_id"]
                || r["memory_id"] != old["memory_id"]
                || r["revision"] != old["revision"]
                || r["cohort"] != old["cohort"]
                || r["screen"] != (i < 12)
                || r["expected_useful"] != expected
                || r["payload"]
                    != json!({"model":m["id"],"state":old["payload"]["state"],"questions":questions})
            {
                return Err("petición difiere de roster, template, modelo o fuente".into());
            }
            if indices.insert(expected_id, mi * 43 + i).is_some() {
                return Err("solicitud repetida".into());
            }
        }
    }
    let events = lines(&directory.join("journal.jsonl"))?;
    if events.len() % 2 != 0 {
        return Err(
            "operación todavía sin terminal; inspeccionar proceso antes de reintentar".into(),
        );
    }
    let mut replies = BTreeMap::new();
    let mut last = None;
    for pair in events.chunks_exact(2) {
        let id = pair[0]["id"].as_str().ok_or("ID ausente")?;
        let index = *indices.get(id).ok_or("operación no preparada")?;
        if pair[0]["kind"] != "started"
            || pair[1]["id"] != id
            || !matches!(pair[1]["kind"].as_str(), Some("response" | "failed"))
            || last.is_some_and(|n| index <= n)
        {
            return Err("diario repetido o fuera de orden".into());
        }
        last = Some(index);
        if replies.insert(id.to_string(), pair[1].clone()).is_some() {
            return Err("respuesta repetida".into());
        }
    }
    let mut model_reports = Vec::new();
    let mut known = 0.0;
    let mut missing = 0;
    for (mi, m) in fixture_models.iter().enumerate() {
        let rs = &requests[mi * 43..mi * 43 + 43];
        let mut measured = Vec::new();
        let mut screen_correct = 0;
        let mut pass = true;
        let mut technical_at = None;
        let mut model_fee = 0.0;
        let mut model_missing = 0;
        for (i, r) in rs.iter().enumerate() {
            let Some(e) = replies.get(r["id"].as_str().unwrap()) else {
                if rs[i + 1..]
                    .iter()
                    .any(|r| replies.contains_key(r["id"].as_str().unwrap()))
                {
                    return Err("hueco de operación preparada".into());
                }
                break;
            };
            if technical_at.is_some() {
                return Err("se continuó tras fallo técnico del modelo".into());
            }
            if i == 12 && !pass {
                return Err("ampliación sin pasar doce controles".into());
            }
            let (selected, paths, error) = if e["kind"] == "response" {
                match choice_mode(&e["body"], premise_gate) {
                    Ok((b, p)) => (Some(b), p, None),
                    Err(s) => (None, Value::Null, Some(s)),
                }
            } else {
                (None, Value::Null, Some("HTTP failed".into()))
            };
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
                model_fee += f;
            } else {
                missing += 1;
                model_missing += 1;
            }
            let revision = e["body"]["model"] == m["expected_served_model"];
            if i < 12 {
                if selected == r["expected_useful"].as_bool()
                    && selected.is_some()
                    && fee.is_some()
                    && revision
                {
                    screen_correct += 1;
                } else {
                    pass = false;
                }
            }
            if selected.is_none() {
                technical_at = Some(i);
                pass = false;
            }
            measured.push(json!({"id":r["id"],"original_case_id":r["original_case_id"],"memory_id":r["memory_id"],"screen":r["screen"],"expected_useful":r["expected_useful"],"selected":selected,"paths":paths,"technical_error":error,"served_model":e["body"]["model"],"revision_matches":revision,"usage":usage,"known_cost_usd":fee,"elapsed_ms":e["elapsed_ms"]}));
        }
        let n = measured.len();
        let screen_pass = screen_correct == 12 && n >= 12;
        let complete = n == 43;
        let mut groups = Vec::new();
        if complete {
            for g in source["providers"]["decisions"]["groups"]
                .as_array()
                .unwrap()
            {
                let mut cases = Vec::new();
                let mut total = 0.0;
                let mut unknown = 0;
                for c in g["cases"].as_array().unwrap() {
                    let checks = measured
                        .iter()
                        .filter(|r| r["original_case_id"] == c["original_case_id"])
                        .cloned()
                        .collect::<Vec<_>>();
                    let degraded = checks
                        .iter()
                        .any(|r| r["selected"].is_null() || r["revision_matches"] != true);
                    let selected = if degraded {
                        BTreeSet::new()
                    } else {
                        checks
                            .iter()
                            .filter(|r| r["selected"] == true)
                            .map(|r| r["memory_id"].as_str().unwrap().to_string())
                            .collect::<BTreeSet<_>>()
                    };
                    let allowed = ids(&c["allowed_ids"])?;
                    let base_ids = ids(&c["baseline_permitted_ids"])?;
                    let previous = ids(&c["previous_permitted_ids"])?;
                    for r in &checks {
                        if let Some(f) = r["known_cost_usd"].as_f64() {
                            total += f;
                        } else {
                            unknown += 1;
                        }
                    }
                    cases.push(json!({"original_case_id":c["original_case_id"],"selected_ids":selected,"allowed_ids":allowed,"wrong_ids":selected.difference(&allowed).collect::<Vec<_>>(),"lost_baseline_ids":base_ids.difference(&selected).collect::<Vec<_>>(),"lost_previous_useful_ids":previous.difference(&selected).collect::<Vec<_>>(),"omitted_allowed_ids":allowed.difference(&selected).collect::<Vec<_>>(),"gained_useful_ids":selected.intersection(&allowed).filter(|id|!base_ids.contains(*id)).collect::<Vec<_>>(),"degraded":degraded,"all_allowed_selected":selected==allowed,"checks":checks}));
                }
                let quality = cases.iter().all(|c| {
                    c["wrong_ids"] == json!([])
                        && c["lost_baseline_ids"] == json!([])
                        && c["lost_previous_useful_ids"] == json!([])
                        && c["degraded"] == false
                }) && if g["cohort"] == "v4" {
                    cases.iter().all(|c| c["all_allowed_selected"] == true)
                } else {
                    cases.iter().any(|c| c["gained_useful_ids"] != json!([]))
                };
                let proposals = g["historical_proposal_cost_usd"]
                    .as_f64()
                    .ok_or("coste de propuesta ausente")?;
                let baseline = g["historical_baseline_cost_usd"]
                    .as_f64()
                    .ok_or("coste de base ausente")?;
                let ratio = (proposals + total) / baseline;
                groups.push(json!({"cohort":g["cohort"],"cases":cases,"new_verification_known_cost_usd":total,"historical_proposal_cost_usd":proposals,"historical_baseline_cost_usd":baseline,"estimated_pipeline_known_cost_usd":proposals+total,"estimated_pipeline_cost_ratio":ratio,"missing_cost_receipts":unknown,"passes_replay_quality":quality,"passes_estimated_cost_screen":unknown==0&&ratio<=1.5}));
            }
        }
        let qualified = screen_pass
            && complete
            && groups.iter().all(|g| {
                g["passes_replay_quality"] == true && g["passes_estimated_cost_screen"] == true
            });
        model_reports.push(json!({"model_id":m["id"],"expected_served_model":m["expected_served_model"],"operations":n,"screen_correct":screen_correct,"screen_cases_executed":n.min(12),"passes_screen":screen_pass,"technical_stop_at":technical_at,"full_replay_complete":complete,"known_cost_usd":model_fee,"unknown_cost_operations":model_missing,"passes_bounded_implementation_screen":qualified,"groups":groups,"observations":measured}));
    }
    if replies.len()
        != model_reports
            .iter()
            .map(|r| r["operations"].as_u64().unwrap())
            .sum::<u64>() as usize
        || !known.is_finite()
    {
        return Err("operaciones no vinculadas o coste inválido".into());
    }
    // Every request was already bound to preflight.requests_sha256, and every
    // full payload above was compared against `questions`, including the atomic
    // question. This proves its pre-call value even if its separate file entry
    // was omitted. Keep that omission visible; never mutate the old preflight.
    if premise_gate
        && !atomic_payloads_match(&requests, &questions["necessary_premise_incompatible"])
    {
        return Err("pregunta atómica difiere de solicitudes congeladas".into());
    }
    let binding = if premise_gate {
        json!({"direct_fixture_entry_present":atomic_direct_entry,"all_frozen_payloads_match_atomic_fixture":true,"binding":"preflight.requests_sha256 -> all43payloads -> exact atomic question","original_direct_fixture_preflight_passed":atomic_direct_entry==Some(true),"fixture_sha256":crate::campaign::digest(Path::new("tests/fixtures/premise-incompatibility-v1.json"))?})
    } else {
        Value::Null
    };
    let result = json!({"version":if premise_gate{"role-complete-premise-probe-v1"}else{"role-complete-provider-probe-v1"},"premise_gate":premise_gate,"atomic_question_binding":binding,"prepared_operations":model_count*43,"operations":replies.len(),"http_requests":replies.len(),"known_cost_usd":known,"unknown_cost_operations":missing,"all_new_cost_receipts_complete":missing==0,"models":model_reports,"requests_sha256":pre["requests_sha256"],"journal_sha256":crate::campaign::digest(&directory.join("journal.jsonl"))?,"fresh_validation":false,"promoted":false,"warning":"Staged known-proposal replay; the12case screen is not full quality. Unexecuted prepared requests are not charged operations. Estimated pipeline costs reuse historical proposals; full current-flow and fresh validation remain pending."});
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
    fn all_three_real_choices_are_required_for_role_complete_or() {
        let a = |s| json!({"type":"choice","choice":s});
        let mut b = json!({"answers":{"fact":a("none"),"ordinary_correction":a("none"),"context_contrast":a("A")}});
        assert!(choice(&b).unwrap().0);
        b["answers"]["context_contrast"]["choice"] = json!("none");
        assert!(!choice(&b).unwrap().0);
        b["answers"].as_object_mut().unwrap().remove("fact");
        assert!(choice(&b).is_err());
    }
    #[test]
    fn atomic_premise_is_mandatory_but_cannot_veto_fact_or_linked_contrast() {
        let a = |s| json!({"type":"choice","choice":s});
        let mut b = json!({"answers":{"fact":a("none"),"ordinary_correction":a("A"),"context_contrast":a("none"),"necessary_premise_incompatible":a("none")}});
        assert!(!choice_mode(&b, true).unwrap().0);
        b["answers"]["necessary_premise_incompatible"]["choice"] = json!("A");
        assert!(choice_mode(&b, true).unwrap().0);
        b["answers"]["necessary_premise_incompatible"]["choice"] = json!("none");
        b["answers"]["fact"]["choice"] = json!("A");
        assert!(choice_mode(&b, true).unwrap().0);
        b["answers"]["fact"]["choice"] = json!("none");
        b["answers"]["context_contrast"]["choice"] = json!("A");
        assert!(choice_mode(&b, true).unwrap().0);
        b["answers"]
            .as_object_mut()
            .unwrap()
            .remove("necessary_premise_incompatible");
        assert!(choice_mode(&b, true).is_err());
    }
    #[test]
    fn indirect_question_binding_requires_every_pre_call_payload_to_match() {
        let q = json!({"type":"choice","instructions":"frozen","criteria":{"A":"yes","none":"no"}});
        let mut requests =
            vec![json!({"payload":{"questions":{"necessary_premise_incompatible":q}}}); 43];
        assert!(atomic_payloads_match(&requests, &q));
        requests[42]["payload"]["questions"]["necessary_premise_incompatible"]["instructions"] =
            json!("changed");
        assert!(!atomic_payloads_match(&requests, &q));
        requests.pop();
        assert!(!atomic_payloads_match(&requests, &q));
    }
}
