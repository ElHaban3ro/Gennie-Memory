//! Serial, warm-process retrieval benchmark using the Rust JSONL session protocol.

use crate::campaign::digest;
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

fn get_search(case: &Value, mode: &str) -> Result<Value, String> {
    let request = case.get("request").ok_or("falta request")?;
    let search = request.get("search").unwrap_or(request);
    let mut search = search.as_object().ok_or("request.search inválido")?.clone();
    search.insert("query_mode".into(), json!(mode));
    Ok(Value::Object(search))
}

fn exchange(
    input: &mut impl Write,
    output: &mut impl BufRead,
    search: &Value,
) -> Result<Value, String> {
    serde_json::to_writer(&mut *input, &json!({"op":"search","request":search}))
        .map_err(|error| error.to_string())?;
    input.write_all(b"\n").map_err(|error| error.to_string())?;
    input.flush().map_err(|error| error.to_string())?;
    let mut line = String::new();
    if output
        .read_line(&mut line)
        .map_err(|error| error.to_string())?
        == 0
    {
        return Err("la sesión terminó antes de responder".into());
    }
    let envelope: Value = serde_json::from_str(&line).map_err(|error| error.to_string())?;
    if envelope.get("status").and_then(Value::as_str) != Some("complete") {
        return Err(format!(
            "búsqueda fallida: {}",
            envelope
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("sin detalle")
        ));
    }
    envelope
        .get("response")
        .cloned()
        .ok_or("falta response".into())
}

pub fn run(
    cases_path: &Path,
    binary: &Path,
    database: &Path,
    output_path: &Path,
    split: &str,
    mode: &str,
) -> Result<(), String> {
    if !["keywords", "natural"].contains(&mode) {
        return Err("modo desconocido".into());
    }
    let source = File::open(cases_path).map_err(|error| error.to_string())?;
    let mut cases = Vec::new();
    let mut seen = HashSet::new();
    for (index, line) in BufReader::new(source).lines().enumerate() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let case: Value =
            serde_json::from_str(&line).map_err(|error| format!("línea {}: {error}", index + 1))?;
        if case.get("split").and_then(Value::as_str) != Some(split) {
            return Err(format!("partición inesperada en línea {}", index + 1));
        }
        let id = case.get("id").and_then(Value::as_str).ok_or("falta id")?;
        if !seen.insert(id.to_owned()) {
            return Err(format!("ID duplicado: {id}"));
        }
        get_search(&case, mode)?;
        cases.push(case);
    }
    if cases.is_empty() {
        return Err("no hay casos".into());
    }
    let manifest = json!({"kind":"search_benchmark", "mode":mode, "split":split,
        "case_count":cases.len(), "cases_sha256":digest(cases_path)?,
        "binary_sha256":digest(binary)?, "database_sha256":digest(database)?});
    let manifest_path = output_path.with_extension("manifest.json");
    if output_path.exists() || manifest_path.exists() {
        return Err("elige rutas de salida nuevas".into());
    }
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)
        .map_err(|error| error.to_string())?;
    let mut writer = BufWriter::new(file);
    let mut child = Command::new(binary)
        .arg("session")
        .env("IROMEM_DATABASE_PATH", database)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| error.to_string())?;
    let mut input = child.stdin.take().ok_or("falta entrada de sesión")?;
    let mut output = BufReader::new(child.stdout.take().ok_or("falta salida de sesión")?);
    let result = (|| {
        let warmup = get_search(&cases[0], mode)?;
        exchange(&mut input, &mut output, &warmup)?;
        for case in &cases {
            let search = get_search(case, mode)?;
            let started = Instant::now();
            let response = exchange(&mut input, &mut output, &search)?;
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            let mut result = Map::new();
            for name in [
                "id",
                "split",
                "segment",
                "expected_ids",
                "expected_fact",
                "expected_condition",
                "expected_source",
                "forbidden_ids",
                "label_source",
            ] {
                if let Some(value) = case.get(name) {
                    result.insert(name.into(), value.clone());
                }
            }
            result.insert("elapsed_ms".into(), json!(elapsed_ms));
            result.insert("response".into(), response);
            serde_json::to_writer(&mut writer, &result).map_err(|error| error.to_string())?;
            writer.write_all(b"\n").map_err(|error| error.to_string())?;
        }
        writer.flush().map_err(|error| error.to_string())?;
        writer
            .get_ref()
            .sync_data()
            .map_err(|error| error.to_string())?;
        Ok::<(), String>(())
    })();
    drop(input);
    if result.is_err() {
        let _ = child.kill();
    }
    let exit = child.wait().map_err(|error| error.to_string())?;
    result?;
    if !exit.success() {
        return Err(format!("la sesión terminó con estado {exit}"));
    }
    let mut manifest_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&manifest_path)
        .map_err(|error| error.to_string())?;
    serde_json::to_writer_pretty(&mut manifest_file, &manifest)
        .map_err(|error| error.to_string())?;
    manifest_file
        .write_all(b"\n")
        .map_err(|error| error.to_string())?;
    manifest_file
        .sync_data()
        .map_err(|error| error.to_string())?;
    println!("{} búsquedas; {}", cases.len(), output_path.display());
    Ok(())
}
