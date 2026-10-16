//! Derive a scoped, unseen cohort from the frozen synthetic fixture in Rust.

use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn run(
    queries: &Path,
    memories: &Path,
    first: u32,
    last: u32,
    output: &Path,
) -> Result<(), String> {
    if first == 0 || last < first || last > 999 || output.exists() {
        return Err("rango o archivo de salida inválido".into());
    }
    let memories: Vec<Value> =
        serde_json::from_slice(&fs::read(memories).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let mut by_id = HashMap::new();
    for memory in &memories {
        let id = memory
            .get("id")
            .and_then(Value::as_str)
            .ok_or("memoria sin ID")?;
        if by_id.insert(id, memory).is_some() {
            return Err(format!("memoria duplicada: {id}"));
        }
    }
    let input = File::open(queries).map_err(|error| error.to_string())?;
    let mut cases = Vec::new();
    let mut seen = HashSet::new();
    let mut scopes = HashSet::new();
    let split = format!("holdout_{first:03}_{last:03}");
    for line in BufReader::new(input).lines() {
        let line = line.map_err(|error| error.to_string())?;
        let query: Value = serde_json::from_str(&line).map_err(|error| error.to_string())?;
        let scope = query
            .get("scope_id")
            .and_then(Value::as_str)
            .ok_or("consulta sin scope_id")?;
        let number = scope
            .rsplit('_')
            .next()
            .ok_or("scope_id inválido")?
            .parse::<u32>()
            .map_err(|_| "scope_id sin número".to_string())?;
        if !(first..=last).contains(&number) {
            continue;
        }
        let id = query
            .get("case_id")
            .and_then(Value::as_str)
            .ok_or("consulta sin case_id")?;
        if !seen.insert(id.to_owned()) {
            return Err(format!("caso duplicado: {id}"));
        }
        let kind = query
            .get("scope_kind")
            .and_then(Value::as_str)
            .ok_or("consulta sin scope_kind")?;
        if scope != format!("synthetic_{kind}_{number:03}") {
            return Err(format!("scope incoherente en {id}"));
        }
        scopes.insert(scope.to_owned());
        let expected = query
            .get("expected_memory_ids")
            .and_then(Value::as_array)
            .ok_or("faltan etiquetas")?;
        if expected.len() > 1 {
            return Err(format!("más de un esperado en {id}"));
        }
        let label = query
            .get("label")
            .and_then(Value::as_str)
            .ok_or("falta label")?;
        if (expected.is_empty() && label != "negative")
            || (!expected.is_empty() && label != "positive")
        {
            return Err(format!("etiqueta incoherente en {id}"));
        }
        let referenced = expected
            .first()
            .map(|value| -> Result<&Value, String> {
                let memory_id = value.as_str().ok_or("ID esperado inválido")?;
                let memory = by_id
                    .get(memory_id)
                    .copied()
                    .ok_or("memoria esperada ausente")?;
                if memory.get("scope_kind") != query.get("scope_kind")
                    || memory.get("scope_id") != query.get("scope_id")
                {
                    return Err(format!("memoria esperada fuera de ámbito en {id}"));
                }
                Ok(memory)
            })
            .transpose()?;
        let segment = format!(
            "{kind}/{}",
            query
                .get("topic")
                .and_then(Value::as_str)
                .ok_or("falta topic")?
        );
        cases.push(json!({"id":id,"split":split,"segment":segment,
            "expected_ids":expected,"forbidden_ids":[],"label_source":"deterministic_fixture_rule",
            "expected_fact":referenced.and_then(|memory|memory.get("summary")),
            "expected_condition":referenced.and_then(|memory|memory.get("conditions")),
            "expected_source":referenced.and_then(|memory|memory.get("source")),
            "request":{"search":{"query":query.get("query"),"scope_kind":kind,
                "scope_id":scope,"entity_id":query.get("entity_id"),"as_of":query.get("as_of"),
                "limit":query.get("limit")}}}));
    }
    if scopes.len() != ((last - first + 1) * 4) as usize || cases.is_empty() {
        return Err(format!(
            "cohorte incompleta: {} ámbitos, {} consultas",
            scopes.len(),
            cases.len()
        ));
    }
    let mut file = File::create_new(output).map_err(|error| error.to_string())?;
    for case in &cases {
        serde_json::to_writer(&mut file, case).map_err(|error| error.to_string())?;
        file.write_all(b"\n").map_err(|error| error.to_string())?;
    }
    file.sync_data().map_err(|error| error.to_string())?;
    println!("{} consultas; {} ámbitos", cases.len(), scopes.len());
    Ok(())
}
