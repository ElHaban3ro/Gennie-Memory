//! Frozen wrong-proposal guard screen, not a fresh-quality or latency campaign.
use super::*;
use std::collections::BTreeMap;
const FAMILY: &str = "outputs/iromem/evaluation/scoped-role-family-contrast-2026-10-20";
const PROTOCOL: &str = "docs/evaluation/fast-critic-replacement-screen-v1-protocol-2026-10-20.md";
const MODELS: [&str; 2] = ["typesafe/jev-1.13", "openai/gpt-6-luna-decisions"];
const REVISIONS: [&str; 2] = [
    "typesafe/jev-1.13-20260917",
    "openai/gpt-6-luna-decisions-20261006",
];
pub(crate) fn screen_inputs() -> Result<(Vec<Value>, Value), String> {
    let (source, mut hashes) = super::inputs()?;
    let archive = read(Path::new(
        "outputs/iromem/evaluation/quality-goal-2026-10-19/snapshots/032/source-provenance.json",
    ))?;
    for file in [
        "requests.jsonl",
        "journal.jsonl",
        "inputs.jsonl",
        "report-v2.json",
    ] {
        let p = Path::new(FAMILY).join(file);
        let h = digest(&p)?;
        if archive["artifacts_sha256"][p.to_str().unwrap()] != h {
            return Err("propuestas históricas alteradas".into());
        }
        hashes[p.to_str().unwrap()] = json!(h);
    }
    hashes[PROTOCOL] = json!(digest(Path::new(PROTOCOL))?);
    let requests = lines(&Path::new(FAMILY).join("requests.jsonl"))?;
    let events = lines(&Path::new(FAMILY).join("journal.jsonl"))?;
    let mut result = Vec::new();
    for mut row in source {
        let key = format!("{}__family_02_candidate", row["id"].as_str().unwrap());
        let r = requests
            .iter()
            .find(|r| r["id"] == key)
            .ok_or("sin propuesta fuente")?;
        let e = events
            .iter()
            .find(|e| e["kind"] == "response" && e["id"] == key)
            .ok_or("sin cuerpo fuente")?;
        let candidates = row["candidates"].as_array().unwrap();
        let mut expected =
            super::payload(&row["role_payload"], candidates, "inception/mercury-decide");
        if r["payload"] != expected {
            return Err("payload de propuesta no reproducible".into());
        }
        let (selected, _) = selected_model(
            &e["body"],
            &r["payload"],
            candidates,
            "inception/mercury-decide-20260930",
        )?;
        let b = candidates
            .iter()
            .enumerate()
            .filter(|(n, c)| {
                selected.contains(c["memory_id"].as_str().unwrap())
                    && e["body"]["answers"][format!("memory_{n:02}_relation")]["choice"] == "B"
            })
            .map(|(_, c)| c["memory_id"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        if b.is_empty() {
            continue;
        }
        // Frozen source decisions select the roster; gold is read only by report().
        expected["model"] = json!("typesafe/jev-1.13");
        row["frozen_proposal_body"] = e["body"].clone();
        row["frozen_proposal_payload"] = r["payload"].clone();
        row["frozen_proposal_event_id"] = json!(key);
        row["proposed_b_ids"] = json!(b);
        result.push(row);
    }
    if result.len() != 6
        || result
            .iter()
            .map(|r| r["proposed_b_ids"].as_array().unwrap().len())
            .sum::<usize>()
            != 11
    {
        return Err("roster histórico distinto de seis/once".into());
    }
    Ok((result, hashes))
}
fn request_rows(inputs: &[Value]) -> Result<Vec<Value>, String> {
    let mut result = Vec::new();
    for (n, row) in inputs.iter().enumerate() {
        let b = ids(&row["proposed_b_ids"])?;
        let candidates = row["candidates"].as_array().unwrap();
        for slot in if n % 2 == 0 { [0, 1] } else { [1, 0] } {
            let mut p = super::payload(&row["role_payload"], candidates, MODELS[slot]);
            p["questions"].as_object_mut().unwrap().retain(|key, _| {
                key == "case_anchor"
                    || candidates.iter().enumerate().any(|(i, c)| {
                        key == &format!("memory_{i:02}_relation")
                            && b.contains(c["memory_id"].as_str().unwrap())
                    })
            });
            result.push(json!({"id":format!("{}__critic_{slot}",row["id"].as_str().unwrap()),"case_id":row["id"],"slot":slot,"expected_served_model":REVISIONS[slot],"payload":p}));
        }
    }
    Ok(result)
}
pub fn prepare(d: &Path) -> Result<(), String> {
    let (rows, hashes) = screen_inputs()?;
    let requests = request_rows(&rows)?;
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    for (name, data) in [("inputs.jsonl", &rows), ("requests.jsonl", &requests)] {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(d.join(name))
            .map_err(|e| e.to_string())?;
        for row in data {
            writeln!(f, "{row}").map_err(|e| e.to_string())?;
        }
        f.sync_all().map_err(|e| e.to_string())?;
    }
    write_new(
        &d.join("preparation.json"),
        &json!({"source_hashes":hashes,"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"requests_sha256":digest(&d.join("requests.jsonl"))?,"max_http_requests":12,"known_queries":6,"frozen_proposals":11,"fresh_validation":false,"runtime_latency_measured":false,"promoted":false}),
    )
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    let (rows, hashes) = screen_inputs()?;
    let requests = request_rows(&rows)?;
    let prep = read(&d.join("preparation.json"))?;
    let pre = read(&d.join("preflight.json"))?;
    if lines(&d.join("inputs.jsonl"))? != rows
        || lines(&d.join("requests.jsonl"))? != requests
        || prep["source_hashes"] != hashes
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || prep["requests_sha256"] != digest(&d.join("requests.jsonl"))?
        || pre["preparation_sha256"] != digest(&d.join("preparation.json"))?
    {
        return Err("filtro no ligado a preparación".into());
    }
    for (p, h) in pre["sources_sha256"]
        .as_object()
        .ok_or("sin fuentes de ejecución")?
    {
        if digest(&d.join("source-executed").join(p))? != *h {
            return Err("fuente ejecutada alterada".into());
        }
    }
    let events = lines(&d.join("journal.jsonl"))?;
    let mut end = BTreeMap::new();
    let mut started = BTreeSet::new();
    let mut cursor = 0;
    let mut total_cost = 0.;
    let mut unknown = 0;
    for e in &events {
        let id = e["id"].as_str().ok_or("sin id")?;
        if e["kind"] == "started" {
            if cursor >= requests.len() || e["id"] != requests[cursor]["id"] || !started.insert(id)
            {
                return Err("inicio repetido o fuera de orden".into());
            }
            cursor += 1;
        } else if e["kind"] == "response" || e["kind"] == "failed" {
            if !started.contains(id) || end.insert(id, e).is_some() {
                return Err("respuesta sin inicio o repetida".into());
            }
            let c = if e["kind"] == "response" {
                e["body"]["usage"]["cost"].as_f64()
            } else {
                e["partial_usage"]["reported_cost_usd"].as_f64()
            };
            if let Some(c) = c.filter(|v| v.is_finite() && *v >= 0.) {
                total_cost += c;
            } else {
                unknown += 1;
            }
        } else {
            return Err("evento desconocido".into());
        }
    }
    unknown += started.len() - end.len();
    let mut observations = Vec::new();
    let mut valid = [0; 2];
    let mut selected = BTreeMap::new();
    let mut individual_wrong = [0; 2];
    let mut individual_lost = [0; 2];
    for r in &requests {
        let id = r["id"].as_str().unwrap();
        let slot = r["slot"].as_u64().unwrap() as usize;
        let row = rows.iter().find(|row| row["id"] == r["case_id"]).unwrap();
        let proposed = ids(&row["proposed_b_ids"])?;
        let gold = ids(&row["case"]["expected_ids"])?;
        let decoded = end
            .get(id)
            .filter(|e| e["kind"] == "response")
            .and_then(|e| {
                super::validated_subset(
                    &e["body"],
                    &r["payload"],
                    row["candidates"].as_array().unwrap(),
                    r["expected_served_model"].as_str().unwrap(),
                )
                .ok()
            });
        if let Some((s, trace)) = decoded {
            if !s.is_subset(&proposed) {
                return Err("juicio fuera de propuestas".into());
            }
            let bad = s.difference(&gold).cloned().collect::<Vec<_>>();
            let missing = proposed
                .intersection(&gold)
                .filter(|id| !s.contains(*id))
                .cloned()
                .collect::<Vec<_>>();
            individual_wrong[slot] += bad.len();
            individual_lost[slot] += missing.len();
            valid[slot] += 1;
            selected.insert(id.to_owned(), s.clone());
            observations.push(json!({"id":id,"valid":true,"selected_proposed_b_ids":s,"unpermitted_ids":bad,"lost_permitted_b_ids":missing,"trace":trace}));
        } else {
            observations.push(json!({"id":id,"valid":false,"selection":null}));
        }
    }
    let mut cases = Vec::new();
    let mut wrong = 0;
    let mut lost = 0;
    let mut retained = 0;
    let mut rejected = 0;
    let mut compared = 0;
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        let a = selected.get(&format!("{id}__critic_0"));
        let b = selected.get(&format!("{id}__critic_1"));
        let delta = if let Some((a, b)) = a.zip(b) {
            compared += 1;
            let s = a.intersection(b).cloned().collect::<BTreeSet<_>>();
            let gold = ids(&row["case"]["expected_ids"])?;
            let proposed = ids(&row["proposed_b_ids"])?;
            let bad = s.difference(&gold).cloned().collect::<Vec<_>>();
            let missing = proposed
                .intersection(&gold)
                .filter(|id| !s.contains(*id))
                .cloned()
                .collect::<Vec<_>>();
            wrong += bad.len();
            lost += missing.len();
            retained += s.intersection(&gold).count();
            rejected += proposed
                .difference(&gold)
                .filter(|id| !s.contains(*id))
                .count();
            json!({"selected_b_ids":s,"unpermitted_ids":bad,"lost_permitted_b_ids":missing})
        } else {
            Value::Null
        };
        cases.push(json!({"id":id,"delta":delta}));
    }
    let passed = valid == [6, 6]
        && compared == 6
        && unknown == 0
        && wrong == 0
        && lost == 0
        && retained == 5
        && rejected == 6;
    write_new(
        out,
        &json!({"version":"fast-critic-replacement-screen-v1","http_requests":started.len(),"ended_http_requests":end.len(),"uncertain_http_requests":started.len()-end.len(),"known_cost_usd":total_cost,"unknown_cost_operations":unknown,"valid_operations":valid,"compared_questions":compared,"combined_unpermitted_ids":if compared==0{None}else{Some(wrong)},"combined_lost_permitted_b_ids":if compared==0{None}else{Some(lost)},"retained_permitted_b_ids":retained,"rejected_unpermitted_b_ids":rejected,"individual_wrong_ids":individual_wrong,"individual_lost_permitted_ids":individual_lost,"screen_passed":passed,"fresh_validation":false,"runtime_latency_measured":false,"promoted":false,"journal_sha256":digest(&d.join("journal.jsonl"))?,"observations":observations,"cases":cases}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn critic_replacement_payload_diff_is_model_only_and_questions_are_frozen_proposals() {
        let row = json!({"id":"q","candidates":[{"memory_id":"a","revision":1,"summary":"one"},{"memory_id":"b","revision":2,"summary":"two"}],"proposed_b_ids":["b"],"role_payload":{"state":{"user_prompt":"q","scope":"s","memories":{},"rules":"r"},"questions":{}}});
        let r = request_rows(&[row]).unwrap();
        let mut a = r[0]["payload"].clone();
        let b = &r[1]["payload"];
        a["model"] = b["model"].clone();
        assert_eq!(a, *b);
        assert_eq!(a["state"]["memories"].as_object().unwrap().len(), 2);
        assert_eq!(a["questions"].as_object().unwrap().len(), 1);
        assert!(a["questions"].get("memory_01_relation").is_some());
        assert!(a["questions"].get("memory_00_relation").is_none());
    }
}

pub(super) fn archived_guard(report: &Value, rows: &[Value]) -> Result<Value, String> {
    let mut compared = 0;
    let mut lost = 0;
    let mut wrong = 0;
    let mut retained = 0;
    let mut rejected = 0;
    let mut cases = Vec::new();
    for row in rows {
        let observed = report["cases"]
            .as_array()
            .ok_or("sin casos")?
            .iter()
            .find(|c| c["id"] == row["id"])
            .and_then(|c| c["operations"].as_array())
            .and_then(|ops| {
                ops.iter()
                    .find(|o| o["variant"] == "candidate" && o["status"] == "complete")
            });
        if let Some(op) = observed {
            compared += 1;
            let selected = ids(&op["selected_ids"])?;
            let gold = ids(&row["case"]["expected_ids"])?;
            let b = ids(&row["proposed_b_ids"])?;
            let bad = b
                .difference(&gold)
                .filter(|id| selected.contains(*id))
                .cloned()
                .collect::<Vec<_>>();
            let missing = b
                .intersection(&gold)
                .filter(|id| !selected.contains(*id))
                .cloned()
                .collect::<Vec<_>>();
            wrong += bad.len();
            lost += missing.len();
            retained += b
                .intersection(&gold)
                .filter(|id| selected.contains(*id))
                .count();
            rejected += b
                .difference(&gold)
                .filter(|id| !selected.contains(*id))
                .count();
            cases.push(json!({"id":row["id"],"accepted_unpermitted_archived_b":bad,"lost_permitted_archived_b":missing}));
        } else {
            cases.push(json!({"id":row["id"],"unavailable":true}));
        }
    }
    Ok(
        json!({"compared_queries":compared,"unavailable_queries":rows.len()-compared,"unpermitted_archived_b_accepted":wrong,"permitted_archived_b_lost":lost,"retained_permitted_b":retained,"rejected_unpermitted_b":rejected,"passed":compared==6&&wrong==0&&lost==0&&retained==5&&rejected==6,"cases":cases}),
    )
}
