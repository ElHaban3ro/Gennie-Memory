//! Bounded local retrieval configuration screen; no selection or truth claims.

use crate::{campaign::digest, search_benchmark};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("falta {key}"))
}

fn read(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)
        .map_err(|e| e.to_string())
}

fn lines(path: &Path) -> Result<Vec<Value>, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(|e| e.to_string()))
        .collect()
}

fn quantile(values: &[f64], fraction: f64) -> f64 {
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    let position = (ordered.len() - 1) as f64 * fraction;
    let low = position.floor() as usize;
    let high = position.ceil() as usize;
    ordered[low] + (ordered[high] - ordered[low]) * position.fract()
}

fn summarize(
    cases: &[Value],
    rows: &[Value],
    overlay: &BTreeMap<String, Value>,
) -> Result<Value, String> {
    let by_id: BTreeMap<&str, &Value> = rows
        .iter()
        .map(|row| Ok((text(row, "id")?, row)))
        .collect::<Result<_, String>>()?;
    if rows.len() != cases.len() || by_id.len() != cases.len() {
        return Err("casos duplicados o ausentes".into());
    }
    let mut positive = 0;
    let mut positive_misses = Vec::new();
    let mut relation_totals = BTreeMap::<String, usize>::new();
    let mut relation_misses = BTreeMap::<String, Vec<String>>::new();
    let mut count = 0;
    let mut bytes = Vec::new();
    let mut elapsed = Vec::new();
    for case in cases {
        let id = text(case, "id")?;
        let row = by_id.get(id).ok_or("falta resultado")?;
        if row["expected_ids"] != case["expected_ids"] {
            return Err("etiquetas cambiadas".into());
        }
        let candidates = row
            .pointer("/response/candidates")
            .and_then(Value::as_array)
            .ok_or("faltan candidatos")?;
        if row.pointer("/response/status").and_then(Value::as_str) != Some("complete") {
            return Err("búsqueda degradada".into());
        }
        let ids: BTreeSet<&str> = candidates
            .iter()
            .map(|c| text(c, "memory_id"))
            .collect::<Result<_, _>>()?;
        let expected = case["expected_ids"]
            .as_array()
            .ok_or("faltan IDs esperados")?;
        if !expected.is_empty() {
            positive += 1;
            if !expected
                .iter()
                .any(|x| x.as_str().is_some_and(|x| ids.contains(x)))
            {
                positive_misses.push(id.to_owned());
            }
        } else if let Some(annotation) = overlay.get(id) {
            let relation = text(annotation, "relation")?;
            *relation_totals.entry(relation.to_owned()).or_default() += 1;
            let evidence = annotation["evidence_memory_ids"]
                .as_array()
                .ok_or("falta evidencia revisada")?;
            if !evidence.is_empty()
                && !evidence
                    .iter()
                    .any(|x| x.as_str().is_some_and(|x| ids.contains(x)))
            {
                relation_misses
                    .entry(relation.to_owned())
                    .or_default()
                    .push(id.to_owned());
            }
        } else {
            return Err(format!("falta anotación negativa: {id}"));
        }
        count += candidates.len();
        bytes.push(
            candidates
                .iter()
                .map(|c| text(c, "summary").map(str::len))
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .sum::<usize>() as f64,
        );
        elapsed.push(row["elapsed_ms"].as_f64().ok_or("falta latencia")?);
    }
    Ok(
        json!({"cases":cases.len(),"positive":positive,"positive_covered":positive-positive_misses.len(),
        "positive_misses":positive_misses,"negative_relation_totals":relation_totals,"evidence_misses":relation_misses,
        "candidate_count":count,"candidate_summary_utf8_bytes_sum":bytes.iter().sum::<f64>(),
        "candidate_summary_utf8_bytes_p50":quantile(&bytes,0.5),
        "runner_roundtrip_ms":{"p50":quantile(&elapsed,0.5),"p95":quantile(&elapsed,0.95),"p99":quantile(&elapsed,0.99)},
        "selection_quality_measured":false,"answer_truth_measured":false,"tokens_measured":false}),
    )
}

fn misses(metrics: &Value) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    if let Some(ids) = metrics["positive_misses"].as_array() {
        result.extend(ids.iter().filter_map(Value::as_str).map(str::to_owned));
    }
    if let Some(relations) = metrics["evidence_misses"].as_object() {
        for ids in relations.values() {
            if let Some(ids) = ids.as_array() {
                result.extend(ids.iter().filter_map(Value::as_str).map(str::to_owned));
            }
        }
    }
    result
}

fn compare(base: &Value, variant: &Value) -> Value {
    let lost: Vec<_> = misses(variant).difference(&misses(base)).cloned().collect();
    let base_bytes = base["candidate_summary_utf8_bytes_sum"].as_f64().unwrap();
    let bytes = variant["candidate_summary_utf8_bytes_sum"]
        .as_f64()
        .unwrap();
    json!({"lost_previously_covered_case_ids":lost,"coverage_preserved":lost.is_empty(),
        "summary_bytes_ratio":bytes/base_bytes,"local_candidate_only":lost.is_empty() && bytes < base_bytes,
        "promoted":false,"cost_or_remote_latency_gain_measured":false})
}

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn cli(binary: &Path, database: &Path, args: &[&str]) -> Result<(), String> {
    let result = Command::new(binary)
        .args(args)
        .env("IROMEM_DATABASE_PATH", database)
        .output()
        .map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into_owned());
    }
    Ok(())
}

pub fn run(suite: &Path, binary: &Path, output: &Path) -> Result<(), String> {
    if output.exists() {
        return Err("elige un directorio de resultados nuevo".into());
    }
    let config = read(suite)?;
    if text(&config, "version")? != "search-config-screen-v1" {
        return Err("versión no admitida".into());
    }
    let root = suite
        .parent()
        .unwrap_or(Path::new("."))
        .join(text(&config, "base_dir")?);
    let binary = binary.canonicalize().map_err(|e| e.to_string())?;
    let overlay_path = root.join(text(&config, "overlay")?);
    let overlay: BTreeMap<String, Value> = lines(&overlay_path)?
        .into_iter()
        .map(|row| Ok((text(&row, "case_id")?.to_owned(), row)))
        .collect::<Result<_, String>>()?;
    if overlay.len() != 37 {
        return Err("anotación negativa incompleta".into());
    }
    let datasets = config["datasets"].as_array().ok_or("faltan bancos")?;
    let temporary = Temporary(std::env::temp_dir().join(
        format!("iromem-config-screen-{}-{}",std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos()),
    ));
    fs::create_dir(&temporary.0).map_err(|e| e.to_string())?;
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let mut reports = Vec::new();
    let configs = [
        ("keywords", 5),
        ("keywords", 10),
        ("keywords", 20),
        ("natural", 5),
        ("natural", 10),
        ("natural", 20),
    ];
    for dataset in datasets {
        let id = text(dataset, "id")?;
        let corpus = root.join(text(dataset, "corpus")?);
        let cases_path = root.join(text(dataset, "cases")?);
        let original = lines(&cases_path)?;
        let database = temporary.0.join(format!("{id}.db"));
        cli(&binary, &database, &["init-db"])?;
        cli(
            &binary,
            &database,
            &["load-fixtures", corpus.to_str().ok_or("ruta inválida")?],
        )?;
        let database_hash = digest(&database)?;
        let mut paths = BTreeMap::new();
        for limit in [5, 10, 20] {
            let path = output.join(format!("{id}-cases-k{limit}.jsonl"));
            let derived = original
                .iter()
                .map(|case| {
                    let mut row = case.clone();
                    row["request"]["search"]["limit"] = json!(limit);
                    row
                })
                .collect::<Vec<_>>();
            fs::write(
                &path,
                derived
                    .iter()
                    .map(Value::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\n",
            )
            .map_err(|e| e.to_string())?;
            paths.insert(limit, (path, derived));
        }
        let mut runs = Vec::new();
        let mut metrics_by_config = BTreeMap::<String, Vec<Value>>::new();
        for round in 0..3 {
            let mut order = configs.to_vec();
            if round == 1 {
                order.reverse();
            }
            if round == 2 {
                order.rotate_left(2);
            }
            for (position, (mode, limit)) in order.iter().enumerate() {
                let (cases_path, cases) = &paths[limit];
                let path = output.join(format!("{id}-{mode}-k{limit}-r{round}.jsonl"));
                search_benchmark::run(cases_path, &binary, &database, &path, "development", mode)?;
                if digest(&database)? != database_hash {
                    return Err("la búsqueda modificó la DB".into());
                }
                let metrics = summarize(cases, &lines(&path)?, &overlay)?;
                metrics_by_config
                    .entry(format!("{mode}-k{limit}"))
                    .or_default()
                    .push(metrics.clone());
                runs.push(
                    json!({"round":round,"position":position,"mode":mode,"limit":limit,
                    "results":path,"sha256":digest(&path)?,"metrics":metrics}),
                );
            }
        }
        let baseline = metrics_by_config["keywords-k20"][0].clone();
        let mut summaries = Vec::new();
        for (name, metrics) in metrics_by_config {
            if metrics.iter().any(|m| {
                misses(m) != misses(&metrics[0])
                    || m["candidate_summary_utf8_bytes_sum"]
                        != metrics[0]["candidate_summary_utf8_bytes_sum"]
            }) {
                return Err("resultados no deterministas entre repeticiones".into());
            }
            let p50 = metrics
                .iter()
                .map(|m| m["runner_roundtrip_ms"]["p50"].as_f64().unwrap())
                .collect::<Vec<_>>();
            let p95 = metrics
                .iter()
                .map(|m| m["runner_roundtrip_ms"]["p95"].as_f64().unwrap())
                .collect::<Vec<_>>();
            summaries.push(
                json!({"configuration":name,"repetitions":3,"metrics":metrics[0],
                "median_run_p50_ms":quantile(&p50,0.5),"median_run_p95_ms":quantile(&p95,0.5),
                "run_p95_ms":p95,"versus_baseline":compare(&baseline,&metrics[0])}),
            );
        }
        reports.push(
            json!({"dataset":id,"case_count":original.len(),"corpus_sha256":digest(&corpus)?,
            "original_cases_sha256":digest(&cases_path)?,"database_sha256":database_hash,
            "configurations":summaries,"runs":runs}),
        );
    }
    fs::write(output.join("report.json"),serde_json::to_string_pretty(&json!({"version":"search-config-screen-v1",
        "suite_sha256":digest(suite)?,"binary_sha256":digest(&binary)?,"overlay_sha256":digest(&overlay_path)?,
        "provider_calls":0,"labels_rewritten":false,"reserved_data_used":false,"datasets":reports,
        "limits":["Known synthetic development and retrospective evidence annotations, not independent validation.",
            "Summary UTF-8 bytes are not tokens, price or remote latency.","No selector or generated answer quality measured.",
            "Three serial repetitions; no concurrency or population performance claim."]})).map_err(|e|e.to_string())?+"\n").map_err(|e|e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smaller_context_is_rejected_when_it_loses_refutation() {
        let baseline = json!({"positive_misses":[],"evidence_misses":{},"candidate_summary_utf8_bytes_sum":1000});
        let trimmed = json!({"positive_misses":[],"evidence_misses":{"refutes_premise":["q-neg"]},"candidate_summary_utf8_bytes_sum":300});
        let result = compare(&baseline, &trimmed);
        assert_eq!(result["local_candidate_only"], false);
        assert_eq!(result["lost_previously_covered_case_ids"], json!(["q-neg"]));
    }
    #[test]
    fn preserving_coverage_is_only_a_local_candidate() {
        let baseline = json!({"positive_misses":[],"evidence_misses":{},"candidate_summary_utf8_bytes_sum":1000});
        let trimmed = json!({"positive_misses":[],"evidence_misses":{},"candidate_summary_utf8_bytes_sum":600});
        let result = compare(&baseline, &trimmed);
        assert_eq!(result["local_candidate_only"], true);
        assert_eq!(result["promoted"], false);
        assert_eq!(result["cost_or_remote_latency_gain_measured"], false);
    }
}
