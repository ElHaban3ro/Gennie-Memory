//! Paired candidate-budget experiment; labels are preserved and truth is not inferred.

use crate::campaign::digest;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("falta {key}"))
}
fn read(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn lines(path: &Path) -> Result<Vec<Value>, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())?
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| serde_json::from_str(s).map_err(|e| e.to_string()))
        .collect()
}
fn quantile(values: &[f64], fraction: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    let p = (values.len() - 1) as f64 * fraction;
    Some(
        values[p.floor() as usize]
            + (values[p.ceil() as usize] - values[p.floor() as usize]) * p.fract(),
    )
}
fn distribution(values: &[f64]) -> Value {
    json!({"count":values.len(),"sum":if values.is_empty(){None}else{Some(values.iter().sum::<f64>())},
        "p50":quantile(values,0.5),"p95":quantile(values,0.95),"p99":quantile(values,0.99)})
}

fn paired_cases(original: &[Value]) -> Result<Vec<Value>, String> {
    let mut pairs = Vec::new();
    for (index, case) in original.iter().enumerate() {
        let id = text(case, "id")?;
        let order = if index % 2 == 0 { [20, 10] } else { [10, 20] };
        for limit in order {
            let mut row = case.clone();
            row["id"] = json!(format!("{id}__k{limit}"));
            row["original_case_id"] = json!(id);
            row["candidate_limit"] = json!(limit);
            row["request"]["search"]["limit"] = json!(limit);
            row["request"]["search"]["query_mode"] = json!("keywords");
            pairs.push(row);
        }
    }
    Ok(pairs)
}

pub fn prepare(suite: &Path, output: &Path) -> Result<(), String> {
    if output.exists() {
        return Err("elige un directorio nuevo".into());
    }
    let config = read(suite)?;
    if text(&config, "version")? != "candidate-budget-paired-v1" {
        return Err("versión no admitida".into());
    }
    let root = suite
        .parent()
        .unwrap_or(Path::new("."))
        .join(text(&config, "base_dir")?);
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let mut sources = Vec::new();
    for dataset in config["datasets"].as_array().ok_or("faltan bancos")? {
        let id = text(dataset, "id")?;
        let cases = root.join(text(dataset, "cases")?);
        let corpus = root.join(text(dataset, "corpus")?);
        let original = lines(&cases)?;
        let pairs = paired_cases(&original)?;
        let path = output.join(format!("{id}-cases.jsonl"));
        fs::write(
            &path,
            pairs
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
                + "\n",
        )
        .map_err(|e| e.to_string())?;
        sources.push(json!({"dataset":id,"original_cases_sha256":digest(&cases)?,"corpus_sha256":digest(&corpus)?,
            "paired_cases_sha256":digest(&path)?,"original_count":original.len(),"paired_count":pairs.len()}));
    }
    fs::write(output.join("preparation.json"),serde_json::to_string_pretty(&json!({"suite_sha256":digest(suite)?,
        "sources":sources,"labels_rewritten":false,"order":"20 then 10 on even source positions; 10 then 20 on odd positions"})).map_err(|e|e.to_string())?+"\n").map_err(|e|e.to_string())
}

fn selected(record: &Value) -> Result<Option<&str>, String> {
    match record.pointer("/response/selected_id") {
        Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        _ => Err("falta selección".into()),
    }
}

fn category(case: &Value, record: &Value, annotation: Option<&Value>) -> Result<String, String> {
    if record.pointer("/response/status").and_then(Value::as_str) != Some("complete") {
        return Ok("degraded".into());
    }
    let Some(id) = selected(record)? else {
        return Ok("empty".into());
    };
    if case["expected_ids"]
        .as_array()
        .ok_or("faltan etiquetas originales")?
        .iter()
        .any(|v| v.as_str() == Some(id))
    {
        return Ok("supports_answer".into());
    }
    if let Some(annotation) = annotation {
        if annotation["evidence_memory_ids"]
            .as_array()
            .ok_or("falta evidencia")?
            .iter()
            .any(|v| v.as_str() == Some(id))
        {
            return Ok(text(annotation, "relation")?.to_owned());
        }
    }
    Ok("unreviewed_or_unrelated_selection".into())
}

fn verdict(left: &str, right: &str) -> &'static str {
    if right == "degraded" && left != "degraded" {
        return "new_degradation";
    }
    if ["supports_answer", "refutes_premise", "pending_or_partial"].contains(&left) && right != left
    {
        return "lost_previously_selected_evidence";
    }
    if right == "unreviewed_or_unrelated_selection" && left != right {
        return "new_unreviewed_selection";
    }
    "no_automatic_block"
}

pub fn report(suite: &Path, directory: &Path, output: &Path) -> Result<(), String> {
    let config = read(suite)?;
    if text(&config, "version")? != "candidate-budget-paired-v1" {
        return Err("versión no admitida".into());
    }
    let root = suite
        .parent()
        .unwrap_or(Path::new("."))
        .join(text(&config, "base_dir")?);
    let overlay_path = root.join(text(&config, "overlay")?);
    let overlay: BTreeMap<String, Value> = lines(&overlay_path)?
        .into_iter()
        .map(|row| Ok((text(&row, "case_id")?.to_owned(), row)))
        .collect::<Result<_, String>>()?;
    let preparation = read(&directory.join("preparation.json"))?;
    if preparation["suite_sha256"].as_str() != Some(digest(suite)?.as_str()) {
        return Err("protocolo modificado".into());
    }
    let mut reports = Vec::new();
    for dataset in config["datasets"].as_array().ok_or("faltan bancos")? {
        let dataset_id = text(dataset, "id")?;
        let original = lines(&root.join(text(dataset, "cases")?))?;
        let cases_path = directory.join(format!("{dataset_id}-cases.jsonl"));
        let journal_path = directory.join(format!("{dataset_id}-journal.jsonl"));
        let cases = lines(&cases_path)?;
        if cases != paired_cases(&original)? {
            return Err("casos emparejados cambiados".into());
        }
        let journal = lines(&journal_path)?;
        let manifest = journal.first().ok_or("diario vacío")?;
        if manifest["cases_sha256"].as_str() != Some(digest(&cases_path)?.as_str()) {
            return Err("hash de campaña distinto".into());
        }
        let mut started = BTreeSet::new();
        let mut finished = BTreeMap::new();
        for row in journal.iter().skip(1) {
            let id = text(row, "id")?.to_owned();
            match text(row, "kind")? {
                "started" if started.insert(id.clone()) => {}
                "finished"
                    if started.contains(&id) && finished.insert(id.clone(), row).is_none() => {}
                _ => return Err("evento duplicado o incoherente".into()),
            }
        }
        if finished.len() != cases.len() || started.len() != finished.len() {
            return Err("campaña incompleta; no imputar abstención".into());
        }
        let mut aggregates = BTreeMap::<u64, Value>::new();
        let mut changes = Vec::new();
        let mut blocks = Vec::new();
        let mut deltas = Vec::new();
        let mut by_class = BTreeMap::<u64, BTreeMap<String, usize>>::new();
        for limit in [20, 10] {
            let mut input = Vec::new();
            let mut cost = Vec::new();
            let mut elapsed = Vec::new();
            let mut models = BTreeMap::<String, usize>::new();
            let mut skipped = 0;
            for case in cases
                .iter()
                .filter(|c| c["candidate_limit"].as_u64() == Some(limit))
            {
                let record = finished.get(text(case, "id")?).ok_or("caso ajeno")?;
                if record["expected_ids"] != case["expected_ids"] {
                    return Err("etiqueta cambiada".into());
                }
                let original_id = text(case, "original_case_id")?;
                let class = category(case, record, overlay.get(original_id))?;
                *by_class.entry(limit).or_default().entry(class).or_default() += 1;
                let response = &record["response"];
                skipped += usize::from(
                    response["classification_status"].as_str() == Some("skipped_empty"),
                );
                if let Some(n) = response["input_tokens"].as_f64() {
                    input.push(n);
                }
                if let Some(n) = response["reported_cost_usd"].as_f64() {
                    cost.push(n);
                }
                elapsed.push(record["elapsed_ms"].as_f64().ok_or("falta tiempo")?);
                *models
                    .entry(
                        response["served_model"]
                            .as_str()
                            .unwrap_or("not_reported")
                            .into(),
                    )
                    .or_default() += 1;
            }
            aggregates.insert(limit,json!({"input_tokens":distribution(&input),"reported_cost_usd":distribution(&cost),
                "runner_elapsed_ms":distribution(&elapsed),"cases":elapsed.len(),"skipped_empty":skipped,
                "models":models,"selection_categories":by_class[&limit]}));
        }
        for case in &original {
            let id = text(case, "id")?;
            let a = finished[&format!("{id}__k20")];
            let b = finished[&format!("{id}__k10")];
            let ac = a
                .pointer("/response/retrieval/candidates")
                .and_then(Value::as_array)
                .ok_or("faltan candidatos")?;
            let bc = b
                .pointer("/response/retrieval/candidates")
                .and_then(Value::as_array)
                .ok_or("faltan candidatos")?;
            if bc != &ac.iter().take(10).cloned().collect::<Vec<_>>() {
                return Err("listas no son prefijos comparables".into());
            }
            let left = category(case, a, overlay.get(id))?;
            let right = category(case, b, overlay.get(id))?;
            let decision = verdict(&left, &right);
            if decision != "no_automatic_block" {
                blocks.push(json!({"case_id":id,"reason":decision,"baseline_category":left,"candidate_category":right}));
            }
            if selected(a)? != selected(b)? || left != right {
                changes.push(json!({"case_id":id,"query":case.pointer("/request/search/query"),
                    "baseline_category":left,"candidate_category":right,"baseline_selected_id":selected(a)?,"candidate_selected_id":selected(b)?,
                    "baseline_seed":a.pointer("/response/seed/seed_text"),"candidate_seed":b.pointer("/response/seed/seed_text")}));
            }
            if left != "degraded" && right != "degraded" {
                deltas.push(b["elapsed_ms"].as_f64().unwrap() - a["elapsed_ms"].as_f64().unwrap());
            }
        }
        reports.push(json!({"dataset":dataset_id,"pairs":original.len(),"paired_cases_sha256":digest(&cases_path)?,
            "journal_sha256":digest(&journal_path)?,"manifest":manifest,"configurations":aggregates,
            "paired_elapsed_delta_k10_minus_k20_ms":distribution(&deltas),"changes":changes,"blocks":blocks,
            "passes_development_selection_screen":blocks.is_empty(),"promoted":false}));
    }
    fs::write(output,serde_json::to_string_pretty(&json!({"version":"candidate-budget-paired-report-v1",
        "suite_sha256":digest(suite)?,"overlay_sha256":digest(&overlay_path)?,"datasets":reports,
        "answer_truth_measured":false,"human_review":false,"reserved_data_used":false,
        "warning":"Known synthetic development; paired order controls some drift but one trial per case is not a population speed or truth guarantee."})).map_err(|e|e.to_string())?+"\n").map_err(|e|e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pairs_alternate_order_without_changing_labels_or_question() {
        let rows = vec![
            json!({"id":"a","expected_ids":["m1"],"request":{"search":{"query":"q1","limit":20}}}),
            json!({"id":"b","expected_ids":[],"request":{"search":{"query":"q2","limit":20}}}),
        ];
        let pairs = paired_cases(&rows).unwrap();
        assert_eq!(
            pairs
                .iter()
                .map(|x| x["candidate_limit"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            vec![20, 10, 10, 20]
        );
        assert_eq!(pairs[0]["expected_ids"], pairs[1]["expected_ids"]);
        assert_eq!(
            pairs[0]["request"]["search"]["query"],
            pairs[1]["request"]["search"]["query"]
        );
    }
    #[test]
    fn losing_a_refutation_is_not_a_safety_improvement() {
        assert_eq!(
            verdict("refutes_premise", "empty"),
            "lost_previously_selected_evidence"
        );
        assert_eq!(
            verdict("supports_answer", "empty"),
            "lost_previously_selected_evidence"
        );
        assert_eq!(verdict("empty", "refutes_premise"), "no_automatic_block");
        assert_eq!(
            verdict("empty", "unreviewed_or_unrelated_selection"),
            "new_unreviewed_selection"
        );
    }
}
