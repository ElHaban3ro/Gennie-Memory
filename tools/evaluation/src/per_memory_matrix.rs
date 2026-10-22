//! Known-pair factorial diagnosis. Not a promotion gate or independent evaluation.
use crate::{campaign::digest, evidence};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

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
fn text<'a>(row: &'a Value, key: &str) -> Result<&'a str, String> {
    row[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("falta {key}"))
}

fn validate_cells(spec: &Value, cases: &[Value]) -> Result<(), String> {
    let operations = spec["operations"].as_array().ok_or("faltan operaciones")?;
    if operations.len() != 64 || cases.len() != 64 {
        return Err("se requieren64celdas antes de llamadas".into());
    }
    let case_ids: BTreeSet<_> = cases
        .iter()
        .map(|c| text(c, "id"))
        .collect::<Result<_, _>>()?;
    let mut ids = BTreeSet::new();
    let mut cells = BTreeSet::new();
    let mut searches = BTreeMap::new();
    for operation in operations {
        let id = text(operation, "id")?;
        let pair = text(operation, "pair_id")?;
        let task = text(operation, "task")?;
        let provider = text(operation, "provider")?;
        let context = text(operation, "context")?;
        if !ids.insert(id)
            || !cells.insert((pair, task, provider, context))
            || !["binary", "relations"].contains(&task)
            || !["jev", "decisions"].contains(&provider)
            || !["isolated", "batch"].contains(&context)
            || !operation["expected_useful"].is_boolean()
        {
            return Err("celda duplicada o tratamiento inválido".into());
        }
        text(operation, "target_id")?;
        let case = cases
            .iter()
            .find(|c| c["id"] == id)
            .ok_or("celda sin caso")?;
        if case["request"]["provider"] != provider
            || case["request"]["selection_policy"]
                != if task == "binary" {
                    "per_memory_binary_v1"
                } else {
                    "per_memory_v2"
                }
        {
            return Err("tratamiento de celda contradice request".into());
        }
        let search = case["request"]["search"].clone();
        if let Some(previous) = searches.insert((pair, context), search.clone()) {
            if previous != search {
                return Err("búsqueda cambia entre tareas o proveedores".into());
            }
        }
    }
    if ids != case_ids
        || case_ids.len() != 64
        || cells.iter().map(|c| c.0).collect::<BTreeSet<_>>().len() != 8
    {
        return Err("matriz sin ocho pares y ocho condiciones completas".into());
    }
    Ok(())
}

fn validate_design(
    spec: &Value,
    cases: &[Value],
    corpus: &[Value],
    original_cases: &[Value],
    original_memories: &[Value],
) -> Result<(), String> {
    let origins = spec["memory_provenance"]
        .as_array()
        .ok_or("falta procedencia")?;
    if origins.len() != corpus.len() {
        return Err("procedencia incompleta".into());
    }
    let mut seen = BTreeSet::new();
    for entry in origins {
        let id = text(entry, "memory_id")?;
        if !seen.insert(id) {
            return Err("procedencia duplicada".into());
        }
        let original = original_memories
            .iter()
            .find(|m| m["id"] == entry["original_memory_id"])
            .ok_or("fuente de recuerdo desconocida")?;
        let memory = corpus
            .iter()
            .find(|m| m["id"] == id)
            .ok_or("clon inexistente")?;
        if memory["scope_id"] != entry["diagnostic_scope_id"] {
            return Err("ámbito clonado distinto".into());
        }
        let mut restored = memory.clone();
        restored["id"] = original["id"].clone();
        restored["scope_id"] = original["scope_id"].clone();
        if restored != *original {
            return Err("texto o metadatos de clon difieren de fuente".into());
        }
    }
    let mut pairs = BTreeMap::new();
    for operation in spec["operations"].as_array().ok_or("faltan operaciones")? {
        let case = cases
            .iter()
            .find(|c| c["id"] == operation["id"])
            .ok_or("caso desconocido")?;
        let original = original_cases
            .iter()
            .find(|c| c["id"] == operation["source_case_id"])
            .ok_or("pregunta fuente desconocida")?;
        let mut search = case["request"]["search"].clone();
        search["scope_id"] = original["request"]["search"]["scope_id"].clone();
        if search != original["request"]["search"] {
            return Err("pregunta o parámetros distintos de la fuente".into());
        }
        let scope = &case["request"]["search"]["scope_id"];
        let size = corpus.iter().filter(|m| m["scope_id"] == *scope).count();
        if size
            != if operation["context"] == "isolated" {
                1
            } else {
                8
            }
        {
            return Err("corpus no corresponde al tamaño de contexto declarado".into());
        }
        let allowed = original["expected_ids"]
            .as_array()
            .ok_or("etiquetas fuente inválidas")?;
        let expected: BTreeSet<_> = origins
            .iter()
            .filter(|m| {
                m["diagnostic_scope_id"] == *scope && allowed.contains(&m["original_memory_id"])
            })
            .map(|m| text(m, "memory_id"))
            .collect::<Result<_, _>>()?;
        let declared: BTreeSet<_> = case["expected_ids"]
            .as_array()
            .ok_or("faltan etiquetas proyectadas")?
            .iter()
            .map(|v| v.as_str().ok_or("ID proyectado inválido"))
            .collect::<Result<_, _>>()?;
        if declared != expected {
            return Err("etiquetas proyectadas distintas de fuente".into());
        }
        let origin = origins
            .iter()
            .find(|m| m["memory_id"] == operation["target_id"])
            .ok_or("objetivo sin procedencia")?;
        let truth = allowed.contains(&operation["source_memory_id"]);
        if origin["original_memory_id"] != operation["source_memory_id"]
            || origin["diagnostic_scope_id"] != *scope
            || operation["expected_useful"] != truth
        {
            return Err("objetivo distinto de la fuente o de su etiqueta".into());
        }
        let signature = json!([
            operation["source_case_id"],
            operation["source_memory_id"],
            truth,
            original["request"]["search"]
        ]);
        if let Some(previous) = pairs.insert(text(operation, "pair_id")?, signature.clone()) {
            if previous != signature {
                return Err("par cambia pregunta/objetivo/verdad entre condiciones".into());
            }
        }
    }
    Ok(())
}

fn validate_auxiliary(
    cases: &[Value],
    corpus: &[Value],
    allow_missing: bool,
) -> Result<(), String> {
    for case in cases {
        let expected = case["expected_ids"].as_array().ok_or("faltan etiquetas")?;
        if expected.len() != 1 {
            continue;
        }
        let memory = corpus
            .iter()
            .find(|m| m["id"] == expected[0])
            .ok_or("ID esperado inexistente")?;
        let missing = ["expected_fact", "expected_condition", "expected_source"]
            .iter()
            .all(|key| case[key].is_null());
        if missing && allow_missing {
            continue;
        }
        if case["expected_fact"] != memory["summary"]
            || case["expected_condition"] != memory["conditions"]
            || case["expected_source"] != memory["source"]
        {
            return Err("metadatos auxiliares de etiqueta inválidos antes de llamadas".into());
        }
    }
    Ok(())
}

fn design_inputs(
    directory: &Path,
    allow_missing_auxiliary: bool,
) -> Result<(Value, Vec<Value>), String> {
    let spec = read(&directory.join("matrix.json"))?;
    let cases = lines(&directory.join("cases.jsonl"))?;
    validate_cells(&spec, &cases)?;
    let corpus = read(&directory.join("memories.json"))?;
    let corpus = corpus.as_array().ok_or("corpus inválido")?;
    validate_auxiliary(&cases, corpus, allow_missing_auxiliary)?;
    let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let hashes = spec["inputs_sha256"]
        .as_object()
        .ok_or("faltan huellas fuente")?;
    for (name, expected) in hashes {
        let path = Path::new(name);
        if path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            || *expected != digest(&project.join(path))?
        {
            return Err("entrada fuente modificada o ruta inválida".into());
        }
    }
    let source_cases = text(&spec, "source_cases_path")?;
    let source_memories = text(&spec, "source_memories_path")?;
    if !hashes.contains_key(source_cases) || !hashes.contains_key(source_memories) {
        return Err("fuentes no ligadas por hash".into());
    }
    let original_cases = lines(&project.join(source_cases))?;
    let original_memories = read(&project.join(source_memories))?;
    validate_design(
        &spec,
        &cases,
        corpus,
        &original_cases,
        original_memories
            .as_array()
            .ok_or("corpus fuente inválido")?,
    )?;
    if spec["cases_sha256"] != digest(&directory.join("cases.jsonl"))?
        || spec["corpus_sha256"] != digest(&directory.join("memories.json"))?
    {
        return Err("matriz modificada tras preparación".into());
    }
    Ok((spec, cases))
}

pub fn validate(directory: &Path) -> Result<(), String> {
    let (_, cases) = design_inputs(directory, false)?;
    println!(
        "{}",
        json!({"valid":true,"cases":cases.len(),"source_projection_verified":true,"semantic_truth_verified":false})
    );
    Ok(())
}

fn observation(spec: &Value, case: &Value, row: &Value) -> Result<Value, String> {
    let response = &row["response"];
    let binary = spec["task"] == "binary";
    if !["binary", "relations"].contains(&text(spec, "task")?)
        || !["isolated", "batch"].contains(&text(spec, "context")?)
        || !["jev", "decisions"].contains(&text(spec, "provider")?)
        || case["request"]["provider"] != spec["provider"]
        || case["request"]["selection_policy"]
            != if binary {
                "per_memory_binary_v1"
            } else {
                "per_memory_v2"
            }
        || response["requested_model"]
            != if spec["provider"] == "jev" {
                "typesafe/jev-1.13"
            } else {
                "openai/gpt-6-luna-decisions"
            }
    {
        return Err("condición ejecutada distinta de la matriz".into());
    }
    let id = text(spec, "target_id")?;
    let expected = case["expected_ids"]
        .as_array()
        .ok_or("faltan IDs permitidos")?;
    let truth = expected.contains(&json!(id));
    if spec["expected_useful"] != truth {
        return Err("objetivo binario difiere de etiquetas congeladas".into());
    }
    let mut out = spec.clone();
    out["status"] = response["status"].clone();
    out["elapsed_ms"] = row["elapsed_ms"].clone();
    out["cost_usd"] = response["reported_cost_usd"].clone();
    out["input_tokens"] = response["input_tokens"].clone();
    if response["status"] != "complete" {
        out["technical_failure"] = json!(true);
        out["predicted_useful"] = Value::Null;
        out["target_correct"] = Value::Null;
        return Ok(out);
    }
    evidence::per_memory_case(case, response, 8, binary)?;
    let candidates = response["retrieval"]["candidates"]
        .as_array()
        .ok_or("faltan candidatos")?;
    let expected_count = if spec["context"] == "isolated" { 1 } else { 8 };
    if candidates.len() != expected_count {
        return Err("tamaño de contexto distinto del tratamiento".into());
    }
    let position = candidates
        .iter()
        .position(|c| c["memory_id"] == id)
        .ok_or("objetivo no presentado")?;
    let assessment = &response["per_memory"]["items"][position];
    let predicted = assessment["relation"] != "unrelated";
    let selected = evidence::selected_set(response)?;
    out["technical_failure"] = json!(false);
    out["predicted_useful"] = json!(predicted);
    out["target_correct"] = json!(predicted == truth);
    out["target_relation"] = assessment["relation"].clone();
    out["target_position_one_based"] = json!(position + 1);
    out["selected_ids"] = json!(selected);
    out["unpermitted_ids"] = json!(
        selected
            .iter()
            .filter(|id| !expected.contains(&json!(id)))
            .collect::<Vec<_>>()
    );
    Ok(out)
}

pub fn run(directory: &Path, output: &Path) -> Result<(), String> {
    run_checked(directory, output, false)
}

pub fn run_projected(directory: &Path, output: &Path) -> Result<(), String> {
    run_checked(directory, output, true)
}

fn run_checked(directory: &Path, output: &Path, projected: bool) -> Result<(), String> {
    let (spec, cases) = design_inputs(directory, projected)?;
    let journal = lines(&directory.join("journal.jsonl"))?;
    let audit_path = directory.join(if projected {
        "audit-projection.json"
    } else {
        "audit.json"
    });
    let audit = if projected {
        let corpus = directory
            .join("memories.json")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let cases = directory
            .join("cases.jsonl")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let journal = directory
            .join("journal.jsonl")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let config = json!({"id":"matrix_diagnostic_projected_auxiliary","corpus":corpus,"cases":cases,"journal":journal,"allow_partial":false,"provenance":"Diagnostic-only source-bound projection after strict audit rejected all-null auxiliary label metadata. Original files untouched. Not a promotion or development quality pass."});
        let campaign = crate::closeout::audit_projected_source_metadata(Path::new(""), &config)?;
        let result = json!({"version":"per-memory-source-projection-audit-v1","matrix_sha256":digest(&directory.join("matrix.json"))?,"strict_original_audit_passed":false,"campaigns":[campaign]});
        if audit_path.exists() {
            if read(&audit_path)? != result {
                return Err("auditoría proyectada existente difiere".into());
            }
        } else {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&audit_path)
                .map_err(|e| e.to_string())?;
            file.write_all(
                (serde_json::to_string_pretty(&result).map_err(|e| e.to_string())? + "\n")
                    .as_bytes(),
            )
            .map_err(|e| e.to_string())?;
        }
        result
    } else {
        read(&audit_path)?
    };
    let manifest = journal.first().ok_or("falta manifiesto")?;
    let campaign = &audit["campaigns"][0];
    if audit["campaigns"].as_array().map(Vec::len) != Some(1)
        || campaign["integrity"]["valid"] != true
        || campaign["sha256"]["journal"] != digest(&directory.join("journal.jsonl"))?
        || campaign["sha256"]["cases"] != digest(&directory.join("cases.jsonl"))?
        || campaign["sha256"]["corpus"] != digest(&directory.join("memories.json"))?
        || manifest["provider"] != "per_case"
        || manifest["selection_policy"] != "per_case"
        || spec["cases_sha256"] != digest(&directory.join("cases.jsonl"))?
        || spec["corpus_sha256"] != digest(&directory.join("memories.json"))?
    {
        return Err("falta auditoría íntegra o entradas de matriz congeladas".into());
    }
    let operations = spec["operations"].as_array().ok_or("faltan operaciones")?;
    if operations.len() != cases.len() || cases.len() != 64 {
        return Err("matriz incompleta".into());
    }
    let finished: BTreeMap<_, _> = journal
        .iter()
        .filter(|r| r["kind"] == "finished")
        .map(|r| Ok((text(r, "id")?, r)))
        .collect::<Result<_, String>>()?;
    if finished.len() != cases.len() {
        return Err("faltan resultados de matriz".into());
    }
    let mut rows = Vec::new();
    let mut unique = BTreeSet::new();
    let mut retrievals = BTreeMap::new();
    for operation in operations {
        let id = text(operation, "id")?;
        let case = cases
            .iter()
            .find(|c| c["id"] == id)
            .ok_or("caso no encontrado")?;
        let row = finished.get(id).ok_or("operación no encontrada")?;
        let pair = text(operation, "pair_id")?;
        let context = text(operation, "context")?;
        let key = (
            pair,
            context,
            text(operation, "task")?,
            text(operation, "provider")?,
        );
        if !unique.insert(key) {
            return Err("celda de matriz duplicada".into());
        }
        let retrieval = row["response"]["retrieval"].clone();
        if let Some(previous) = retrievals.insert((pair, context), retrieval.clone()) {
            if previous != retrieval {
                return Err("candidatos distintos entre tareas/proveedores".into());
            }
        }
        rows.push(observation(operation, case, row)?);
    }
    if unique.iter().map(|k| k.0).collect::<BTreeSet<_>>().len() != 8 {
        return Err("se requieren ocho pares completos".into());
    }
    let mut cells = BTreeMap::<String, Value>::new();
    for row in &rows {
        let key = format!(
            "{}/{}/{}",
            text(row, "provider")?,
            text(row, "task")?,
            text(row, "context")?
        );
        let entry = cells.entry(key).or_insert(json!({"operations":0,"target_correct":0,"false_positive":0,"false_negative":0,"technical_failures":0,"known_cost_usd":0.0,"missing_cost_operations":0,"whole_seed_contaminated_cases":0}));
        for (counter, yes) in [
            ("operations", true),
            ("target_correct", row["target_correct"] == true),
            (
                "false_positive",
                row["predicted_useful"] == true && row["expected_useful"] == false,
            ),
            (
                "false_negative",
                row["predicted_useful"] == false && row["expected_useful"] == true,
            ),
            ("technical_failures", row["technical_failure"] == true),
            (
                "whole_seed_contaminated_cases",
                row["unpermitted_ids"]
                    .as_array()
                    .is_some_and(|v| !v.is_empty()),
            ),
        ] {
            if yes {
                entry[counter] = json!(entry[counter].as_u64().unwrap() + 1);
            }
        }
        match row["cost_usd"].as_f64() {
            Some(cost) => {
                entry["known_cost_usd"] = json!(entry["known_cost_usd"].as_f64().unwrap() + cost)
            }
            None => {
                entry["missing_cost_operations"] =
                    json!(entry["missing_cost_operations"].as_u64().unwrap() + 1)
            }
        }
    }
    let mut flips = Vec::new();
    for (i, a) in rows.iter().enumerate() {
        for b in &rows[i + 1..] {
            if a["pair_id"] != b["pair_id"]
                || a["technical_failure"] == true
                || b["technical_failure"] == true
            {
                continue;
            }
            let changed: Vec<_> = ["provider", "task", "context"]
                .into_iter()
                .filter(|k| a[k] != b[k])
                .collect();
            if changed.len() == 1 && a["predicted_useful"] != b["predicted_useful"] {
                flips.push(json!({"pair_id":a["pair_id"],"factor":changed[0],"first":a["id"],"second":b["id"],"first_prediction":a["predicted_useful"],"second_prediction":b["predicted_useful"]}));
            }
        }
    }
    let report = json!({"version":"per-memory-factorial-diagnostic-v1","matrix_sha256":digest(&directory.join("matrix.json"))?,"journal_sha256":digest(&directory.join("journal.jsonl"))?,"audit_sha256":digest(&audit_path)?,"auxiliary_metadata_projection_used":projected,"strict_original_audit_passed":!projected,"operations":rows.len(),"cells":cells,"rows":rows,"single_factor_prediction_flips":flips,"retrieval_identical_across_tasks_and_providers":true,"promoted":false,"independent_validation":false,"warning":"Eight deliberately chosen known pairs, one observation per cell, with related questions. Context changes other memories, target position and diagnostic scope. Flips are descriptive evidence, not internal cause or population superiority. Projection mode derives missing auxiliary metadata offline and is never a promotion gate."});
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    file.write_all(
        (serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n").as_bytes(),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_rejects_null_single_label_metadata_and_projection_rejects_wrong_supplied_fact() {
        let corpus = vec![
            json!({"id":"a","summary":"literal source","conditions":null,"source":{"kind":"user","ref":"synthetic://test"}}),
        ];
        let mut cases = vec![
            json!({"expected_ids":["a"],"expected_fact":null,"expected_condition":null,"expected_source":null}),
        ];
        assert!(validate_auxiliary(&cases, &corpus, false).is_err());
        assert!(validate_auxiliary(&cases, &corpus, true).is_ok());
        cases[0]["expected_fact"] = json!("Invented fact");
        assert!(validate_auxiliary(&cases, &corpus, true).is_err());
        cases[0]["expected_fact"] = corpus[0]["summary"].clone();
        cases[0]["expected_source"] = corpus[0]["source"].clone();
        assert!(validate_auxiliary(&cases, &corpus, false).is_ok());
    }

    #[test]
    fn preflight_rejects_missing_duplicate_or_misrouted_cells() {
        let mut operations = Vec::new();
        let mut cases = Vec::new();
        for pair in 0..8 {
            for context in ["isolated", "batch"] {
                for task in ["binary", "relations"] {
                    for provider in ["jev", "decisions"] {
                        let id = format!("{pair}-{context}-{task}-{provider}");
                        operations.push(json!({"id":id,"pair_id":format!("p{pair}"),"context":context,"task":task,"provider":provider,"expected_useful":true,"target_id":"a"}));
                        cases.push(json!({"id":id,"request":{"search":{"scope_id":format!("{pair}-{context}"),"query":"same"},"provider":provider,"selection_policy":if task=="binary"{"per_memory_binary_v1"}else{"per_memory_v2"}}}));
                    }
                }
            }
        }
        let spec = json!({"operations":operations});
        assert!(validate_cells(&spec, &cases).is_ok());
        let mut bad = spec.clone();
        bad["operations"] = json!([]);
        assert!(validate_cells(&bad, &cases).is_err());
        let mut bad = spec.clone();
        bad["operations"][1] = bad["operations"][0].clone();
        assert!(validate_cells(&bad, &cases).is_err());
        let mut bad = spec;
        bad["operations"][0]["provider"] = json!("decisions");
        assert!(validate_cells(&bad, &cases).is_err());
    }

    #[test]
    fn projected_pairs_cannot_change_the_question_text_or_gold() {
        let source_cases = vec![
            json!({"id":"q","expected_ids":["original"],"request":{"search":{"query":"Which?","scope_id":"source"}}}),
        ];
        let source_memories = vec![
            json!({"id":"original","scope_id":"source","summary":"literal source","revision":1}),
        ];
        let corpus = vec![
            json!({"id":"a","scope_id":"one","summary":"literal source","revision":1}),
            json!({"id":"b","scope_id":"two","summary":"literal source","revision":1}),
        ];
        let cases = vec![
            json!({"id":"one","expected_ids":["a"],"request":{"search":{"query":"Which?","scope_id":"one"}}}),
            json!({"id":"two","expected_ids":["b"],"request":{"search":{"query":"Which?","scope_id":"two"}}}),
        ];
        let spec = json!({"memory_provenance":[{"memory_id":"a","original_memory_id":"original","diagnostic_scope_id":"one"},{"memory_id":"b","original_memory_id":"original","diagnostic_scope_id":"two"}],"operations":[{"id":"one","pair_id":"pair","source_case_id":"q","source_memory_id":"original","target_id":"a","expected_useful":true,"context":"isolated"},{"id":"two","pair_id":"pair","source_case_id":"q","source_memory_id":"original","target_id":"b","expected_useful":true,"context":"isolated"}]});
        assert!(validate_design(&spec, &cases, &corpus, &source_cases, &source_memories).is_ok());
        let mut wrong = cases.clone();
        wrong[1]["request"]["search"]["query"] = json!("Different question");
        assert!(validate_design(&spec, &wrong, &corpus, &source_cases, &source_memories).is_err());
        let mut wrong = cases.clone();
        wrong[1]["expected_ids"] = json!([]);
        assert!(validate_design(&spec, &wrong, &corpus, &source_cases, &source_memories).is_err());
        let mut wrong = corpus;
        wrong[1]["summary"] = json!("Changed evidence");
        assert!(validate_design(&spec, &cases, &wrong, &source_cases, &source_memories).is_err());
    }
    #[test]
    fn target_success_cannot_hide_another_wrong_positive_or_wrong_task() {
        let spec = json!({"target_id":"a","expected_useful":true,"provider":"jev","task":"binary","context":"batch"});
        let case = json!({"id":"q","expected_ids":["a"],"request":{"provider":"jev","selection_policy":"per_memory_binary_v1"}});
        let candidates: Vec<_> = (0..8)
            .map(|i| json!({"memory_id":if i==0{"a".into()}else{format!("m{i}")},"revision":1}))
            .collect();
        let items:Vec<_>=candidates.iter().enumerate().map(|(i,c)|json!({"memory_id":c["memory_id"],"revision":1,"relation":if i<2{"useful_evidence"}else{"unrelated"}})).collect();
        let mut row = json!({"response":{"status":"complete","requested_model":"typesafe/jev-1.13","selected_id":"a","selected_ids":["a","m1"],"seed":{"memories":[{"memory_id":"a"},{"memory_id":"m1"}]},"retrieval":{"candidates":candidates},"per_memory":{"items":items}}});
        let out = observation(&spec, &case, &row).unwrap();
        assert_eq!(out["target_correct"], true);
        assert_eq!(out["unpermitted_ids"], json!(["m1"]));
        row["response"]["per_memory"]["items"][0]["relation"] = json!("supports_answer");
        assert!(observation(&spec, &case, &row).is_err());
    }
}
