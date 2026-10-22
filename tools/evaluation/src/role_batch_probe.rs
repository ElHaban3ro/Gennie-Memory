//! Independent raw-response audit of the fixed question-local isolation probe.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

pub fn report(directory: &Path, output: &Path) -> Result<(), String> {
    let requests_path = directory.join("requests.jsonl");
    let preflight: Value = serde_json::from_slice(
        &std::fs::read(directory.join("preflight.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let data = std::fs::read_to_string(&requests_path).map_err(|e| e.to_string())?;
    if preflight["requests_sha256"] != crate::campaign::digest(&requests_path)?
        || preflight["operations"] != 12
        || preflight["max_http_requests"] != 12
    {
        return Err("preflight de aislamiento alterado".into());
    }
    let requests: Vec<Value> = data
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let events: Vec<Value> = std::fs::read_to_string(directory.join("journal.jsonl"))
        .map_err(|e| e.to_string())?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let report = inspect(&requests, &events)?;
    let mut report = report;
    report["requests_sha256"] = preflight["requests_sha256"].clone();
    report["journal_sha256"] = json!(crate::campaign::digest(&directory.join("journal.jsonl"))?);
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn inspect(requests: &[Value], events: &[Value]) -> Result<Value, String> {
    if requests.len() != 12 || events.len() != 24 {
        return Err("diagnóstico incompleto o repetido".into());
    }
    let mut rows = Vec::new();
    let mut known = 0.0;
    let mut missing = 0;
    let mut correct = 0;
    let mut changes = 0;
    let mut singles = BTreeMap::new();
    let mut block = Vec::new();
    for (index, (r, pair)) in requests.iter().zip(events.chunks_exact(2)).enumerate() {
        let scene = [
            "fact_match",
            "fact_wrong_context",
            "correction_match",
            "correction_wrong_context",
        ][index / 3];
        let variant = ["single", "unrelated", "adversarial"][index % 3];
        let id = format!("{scene}_{variant}");
        let expected = matches!(scene, "fact_match" | "correction_match");
        if r["id"] != id
            || r["expected_target"] != expected
            || pair[0]["id"] != id
            || pair[1]["id"] != id
            || pair[0]["kind"] != "started"
            || r["payload"]["model"] != "openai/gpt-6-luna-decisions"
        {
            return Err("roster, expectativa o estado de operación alterado".into());
        }
        let event = &pair[1];
        let usage = if event["kind"] == "response" {
            &event["body"]["usage"]
        } else {
            &event["partial_usage"]
        };
        let cost = usage.get("cost").or_else(|| usage.get("reported_cost_usd"));
        let cost = cost
            .and_then(Value::as_f64)
            .filter(|c| c.is_finite() && *c >= 0.0);
        if let Some(c) = cost {
            known += c;
        } else {
            missing += 1;
        }
        if !known.is_finite() {
            return Err("coste inválido".into());
        }
        if event["kind"] != "response" {
            block.push(json!({"id":id,"technical_failure":true}));
            continue;
        }
        let questions = r["payload"]["questions"]
            .as_object()
            .ok_or("preguntas ausentes")?;
        let answers = event["body"]["answers"]
            .as_object()
            .ok_or("respuestas ausentes")?;
        if answers.len() != questions.len() {
            return Err("respuestas incompletas".into());
        }
        for (key, q) in questions {
            let a = answers.get(key).ok_or("respuesta ajena")?;
            let selected = a["choice"].as_str().ok_or("choice ausente")?;
            let criteria = q["criteria"].as_object().ok_or("criterios ausentes")?;
            if a["type"] != "choice" || !criteria.contains_key(selected) {
                return Err("choice inválida".into());
            }
        }
        let prefix = if variant == "single" { "" } else { "m0_" };
        let ordinary = answers
            .get(&format!("{prefix}relevance"))
            .ok_or("juicio objetivo ausente")?["choice"]
            == "A";
        let contrast = if scene.starts_with("correction") {
            answers
                .get(&format!("{prefix}context_contrast"))
                .ok_or("contraste objetivo ausente")?["choice"]
                == "A"
        } else {
            false
        };
        let selected = ordinary || contrast;
        if variant == "single" {
            singles.insert(scene, selected);
        } else if singles.get(scene) != Some(&selected) {
            changes += 1;
        }
        if selected == expected {
            correct += 1;
        } else {
            block.push(json!({"id":id,"expected":expected,"actual":selected}));
        }
        rows.push(json!({"id":id,"expected":expected,"selected":selected,"elapsed_ms":event["elapsed_ms"],"usage":usage,"served_model":event["body"]["model"]}));
    }
    Ok(
        json!({"version":"role-batch-isolation-v1","operations":12,"http_requests":12,"correct_targets":correct,"changed_targets_under_distraction":changes,"known_cost_usd":known,"unknown_cost_operations":missing,"all_cost_receipts_complete":missing==0,"passes_feasibility_screen":correct==12&&changes==0&&missing==0&&block.is_empty(),"blocks":block,"rows":rows,"fresh_validation":false,"promoted":false,"warning":"Fixed twelve-call synthetic feasibility screen, not full quality or isolation guarantee; each global receipt counted once."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_or_repeated_operation_cannot_look_like_a_completed_probe() {
        assert!(inspect(&[], &[]).is_err());
        assert!(inspect(&vec![json!({}); 12], &vec![json!({}); 24]).is_err());
    }
}
