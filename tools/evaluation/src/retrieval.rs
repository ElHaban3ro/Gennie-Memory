//! Offline validation of frozen retrieval results; no database or provider access.

use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

fn read_jsonl(path: &Path) -> Result<Vec<Value>, String> {
    let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line).map_err(|error| format!("línea {}: {error}", index + 1))
        })
        .collect()
}

fn required_str<'a>(row: &'a Value, field: &str) -> Result<&'a str, String> {
    row.get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("falta {field}"))
}

fn ids(row: &Value, field: &str) -> Result<Vec<String>, String> {
    row.get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("falta {field}"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{field} debe contener IDs"))
        })
        .collect()
}

fn quantile(values: &[f64], fraction: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    ordered[((ordered.len() - 1) as f64 * fraction) as usize]
}

fn ratio(numerator: usize, denominator: usize) -> Value {
    if denominator == 0 {
        Value::Null
    } else {
        json!(numerator as f64 / denominator as f64)
    }
}

#[derive(Clone)]
struct Row {
    split: String,
    segment: String,
    scope_id: String,
    expected: Vec<String>,
    forbidden: Vec<String>,
    candidates: Vec<String>,
    elapsed_ms: f64,
    status: String,
    coverage: String,
}

fn summarize(rows: &[Row]) -> Value {
    let positive: Vec<&Row> = rows.iter().filter(|row| !row.expected.is_empty()).collect();
    let negative: Vec<&Row> = rows.iter().filter(|row| row.expected.is_empty()).collect();
    let recall = positive
        .iter()
        .filter(|row| row.expected.iter().any(|id| row.candidates.contains(id)))
        .count();
    let recall_at_5 = positive
        .iter()
        .filter(|row| {
            row.expected.iter().any(|id| {
                row.candidates
                    .iter()
                    .take(5)
                    .any(|candidate| candidate == id)
            })
        })
        .count();
    let complete_recall_at_5 = positive
        .iter()
        .filter(|row| {
            row.expected.iter().all(|id| {
                row.candidates
                    .iter()
                    .take(5)
                    .any(|candidate| candidate == id)
            })
        })
        .count();
    let complete_recall_at_20 = positive
        .iter()
        .filter(|row| row.expected.iter().all(|id| row.candidates.contains(id)))
        .count();
    let top1 = positive
        .iter()
        .filter(|row| {
            row.candidates
                .first()
                .is_some_and(|candidate| row.expected.contains(candidate))
        })
        .count();
    let empty = negative
        .iter()
        .filter(|row| row.candidates.is_empty())
        .count();
    let forbidden = rows
        .iter()
        .filter(|row| row.forbidden.iter().any(|id| row.candidates.contains(id)))
        .count();
    let latencies: Vec<f64> = rows.iter().map(|row| row.elapsed_ms).collect();
    json!({
        "cases": rows.len(), "positive": positive.len(), "negative": negative.len(),
        "recall_at_5": ratio(recall_at_5, positive.len()),
        "recall_at_20": ratio(recall, positive.len()),
        "all_evidence_recall_at_5": ratio(complete_recall_at_5, positive.len()),
        "all_evidence_recall_at_20": ratio(complete_recall_at_20, positive.len()),
        "top_1_accuracy": ratio(top1, positive.len()),
        "negative_empty_rate": ratio(empty, negative.len()),
        "forbidden_leak_count": forbidden,
        "latency_ms": {"p50": quantile(&latencies, 0.5), "p95": quantile(&latencies, 0.95),
                       "p99": quantile(&latencies, 0.99)}
    })
}

fn evaluate(queries: &[Value], results: &[Value]) -> Result<Value, String> {
    let mut by_id = HashMap::new();
    for query in queries {
        let id = required_str(query, "id")?;
        if by_id.insert(id, query).is_some() {
            return Err("IDs de consulta duplicados".into());
        }
    }
    let mut seen = HashSet::new();
    let mut rows = Vec::new();
    let mut method: Option<&str> = None;
    for result in results {
        let id = required_str(result, "id")?;
        if !seen.insert(id) {
            return Err(format!("resultado duplicado: {id}"));
        }
        let query = by_id
            .get(id)
            .ok_or_else(|| format!("resultado sin consulta: {id}"))?;
        for field in [
            "split",
            "segment",
            "expected_ids",
            "forbidden_ids",
            "label_source",
            "expected_condition",
            "expected_source",
        ] {
            if result.get(field) != query.get(field) {
                return Err(format!("etiqueta {field} distinta en {id}"));
            }
        }
        let response = result
            .get("response")
            .ok_or_else(|| format!("falta response en {id}"))?;
        let status = required_str(response, "status")?;
        if status != "complete" && status != "degraded" {
            return Err(format!("estado inválido en {id}"));
        }
        let current_method = required_str(response, "method")?;
        if ![
            "fts5",
            "fts5_keywords",
            "fts5_semantic_rrf",
            "engram_any",
            "basic_memory_fts",
            "lodis_hybrid",
        ]
        .contains(&current_method)
        {
            return Err(format!("método inválido en {id}"));
        }
        if method.is_some_and(|previous| previous != current_method) {
            return Err("el archivo mezcla métodos de búsqueda".into());
        }
        method = Some(current_method);
        let candidates: Vec<String> = response
            .get("candidates")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("faltan candidatos en {id}"))?
            .iter()
            .map(|item| required_str(item, "memory_id").map(str::to_owned))
            .collect::<Result<_, _>>()?;
        let elapsed_ms = result
            .get("elapsed_ms")
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite() && *value >= 0.0)
            .ok_or_else(|| format!("latencia inválida en {id}"))?;
        rows.push(Row {
            split: required_str(result, "split")?.into(),
            segment: required_str(result, "segment")?.into(),
            scope_id: query
                .pointer("/request/search/scope_id")
                .or_else(|| query.pointer("/request/scope_id"))
                .and_then(Value::as_str)
                .unwrap_or("unspecified")
                .into(),
            expected: ids(result, "expected_ids")?,
            forbidden: ids(result, "forbidden_ids")?,
            candidates,
            elapsed_ms,
            status: status.into(),
            coverage: required_str(response, "semantic_coverage")?.into(),
        });
    }
    if seen.len() != by_id.len() {
        return Err("faltan resultados para algunas consultas".into());
    }
    let mut grouped: BTreeMap<String, Vec<Row>> = BTreeMap::new();
    for row in &rows {
        grouped
            .entry(format!("{}/{}", row.split, row.segment))
            .or_default()
            .push(row.clone());
    }
    let mut by_split_segment = Map::new();
    for (key, values) in grouped {
        by_split_segment.insert(key, summarize(&values));
    }
    let mut scopes: BTreeMap<String, Vec<Row>> = BTreeMap::new();
    for row in &rows {
        scopes
            .entry(row.scope_id.clone())
            .or_default()
            .push(row.clone());
    }
    let mut by_scope = Map::new();
    for (key, values) in scopes {
        by_scope.insert(key, summarize(&values));
    }
    let mut coverage = Map::new();
    for name in ["disabled", "complete", "partial", "missing"] {
        coverage.insert(
            name.into(),
            json!(rows.iter().filter(|row| row.coverage == name).count()),
        );
    }
    Ok(json!({
        "method": method.ok_or("no hay resultados")?,
        "overall": summarize(&rows),
        "by_split_segment": by_split_segment,
        "by_scope": by_scope,
        "degraded_cases": rows.iter().filter(|row| row.status == "degraded").count(),
        "semantic_coverage": coverage,
    }))
}

pub fn run(results: &Path, queries: &Path, output: Option<&Path>) -> Result<(), String> {
    let report = evaluate(&read_jsonl(queries)?, &read_jsonl(results)?)?;
    let encoded = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())? + "\n";
    if let Some(path) = output {
        fs::write(path, encoded).map_err(|error| error.to_string())
    } else {
        print!("{encoded}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_rejects_missing_results_and_mismatched_labels() {
        let query = json!({"id":"case-1", "split":"development", "segment":"positive",
            "expected_ids":["memory-1"], "forbidden_ids":[], "label_source":"fixture"});
        let result = json!({"id":"case-1", "split":"development", "segment":"positive",
            "expected_ids":["memory-1"], "forbidden_ids":[], "label_source":"fixture",
            "elapsed_ms":2.0, "response":{"status":"complete", "method":"fts5_keywords",
            "semantic_coverage":"disabled", "candidates":[{"memory_id":"memory-1"}]}});
        let report = evaluate(std::slice::from_ref(&query), std::slice::from_ref(&result)).unwrap();
        assert_eq!(report["overall"]["recall_at_20"], 1.0);
        assert_eq!(report["overall"]["recall_at_5"], 1.0);
        assert!(evaluate(std::slice::from_ref(&query), &[]).is_err());
        let mut changed = result;
        changed["expected_ids"] = json!([]);
        assert!(evaluate(&[query], &[changed]).is_err());
    }
}
