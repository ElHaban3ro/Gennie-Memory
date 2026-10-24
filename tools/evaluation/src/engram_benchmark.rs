//! Reproducible Engram MCP comparison on a synthetic, isolated imported store.

use crate::campaign::digest;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;

const STOPWORDS: &[&str] = &[
    "a", "al", "algo", "como", "con", "cual", "cuando", "cuál", "cuáles", "cuándo", "cómo", "de",
    "del", "donde", "dónde", "el", "ella", "en", "es", "esta", "este", "esto", "fue", "ha", "hay",
    "la", "las", "le", "lo", "los", "mi", "mis", "no", "nos", "para", "por", "que", "quien",
    "quién", "qué", "se", "ser", "si", "son", "su", "sus", "te", "tiene", "un", "una", "unas",
    "unos", "y", "ya",
];

fn keywords(input: &str) -> String {
    let lowered = input.to_lowercase();
    let mut words = Vec::new();
    let mut current = String::new();
    for ch in lowered.chars().chain(std::iter::once(' ')) {
        if ch.is_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            if current.chars().count() >= 3 && !STOPWORDS.contains(&current.as_str()) {
                words.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
        }
    }
    words.join(" ")
}

fn required<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("falta {field}"))
}

fn read_cases(path: &Path, split: &str) -> Result<Vec<Value>, String> {
    let source = File::open(path).map_err(|error| error.to_string())?;
    let mut cases = Vec::new();
    let mut ids = HashSet::new();
    for (index, line) in BufReader::new(source).lines().enumerate() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let case: Value =
            serde_json::from_str(&line).map_err(|error| format!("línea {}: {error}", index + 1))?;
        if required(&case, "split")? != split {
            return Err(format!("partición inesperada en línea {}", index + 1));
        }
        if !ids.insert(required(&case, "id")?.to_owned()) {
            return Err("ID de caso duplicado".into());
        }
        let search = case
            .get("request")
            .and_then(|value| value.get("search"))
            .ok_or("falta request.search")?;
        required(search, "query")?;
        required(search, "scope_kind")?;
        required(search, "scope_id")?;
        cases.push(case);
    }
    if cases.is_empty() {
        return Err("no hay casos".into());
    }
    Ok(cases)
}

fn project(scope_kind: &str, scope_id: &str) -> String {
    format!("{scope_kind}:{scope_id}")
}

fn sync_id(id: &str) -> String {
    let hash = Sha256::digest(id.as_bytes());
    let prefix = hash[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("obs-{prefix}")
}

fn import_fixture(
    memories: &Path,
    binary: &Path,
    data_dir: &Path,
) -> Result<HashMap<String, String>, String> {
    let items: Vec<Value> =
        serde_json::from_slice(&fs::read(memories).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    if items.is_empty() {
        return Err("fixture sin recuerdos".into());
    }
    let mut projects = BTreeSet::new();
    let mut observations = Vec::with_capacity(items.len());
    let mut mapping = HashMap::new();
    for (index, memory) in items.iter().enumerate() {
        let id = required(memory, "id")?;
        let owner = project(
            required(memory, "scope_kind")?,
            required(memory, "scope_id")?,
        );
        let session = format!("import-{owner}");
        let sync = sync_id(id);
        if mapping.insert(sync.clone(), id.to_owned()).is_some() {
            return Err("sync ID duplicado".into());
        }
        projects.insert(owner.clone());
        observations.push(json!({
            "id": index+1, "sync_id":sync, "session_id":session, "type":"manual",
            "title":required(memory,"summary")?, "content":required(memory,"original_text")?,
            "project":owner, "scope":"project", "revision_count":1, "duplicate_count":1,
            "last_seen_at":"2026-10-15 12:00:00", "created_at":"2026-10-15 12:00:00",
            "updated_at":"2026-10-15 12:00:00", "pinned":false
        }));
    }
    let sessions: Vec<Value> = projects
        .iter()
        .map(|owner| {
            json!({
                "id":format!("import-{owner}"), "project":owner, "ownership_mode":"project_owned",
                "directory":"/synthetic/gennie-memory", "started_at":"2026-10-15 12:00:00.000000000"
            })
        })
        .collect();
    let export = json!({"version":"0.2.0", "exported_at":"2026-10-15 12:00:00.000000000",
        "sessions":sessions,"observations":observations,"prompts":Value::Null});
    let import_path = data_dir.with_extension("import.json");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&import_path)
        .map_err(|error| error.to_string())?;
    serde_json::to_writer(&mut file, &export).map_err(|error| error.to_string())?;
    file.sync_data().map_err(|error| error.to_string())?;
    let result = Command::new(binary)
        .arg("import")
        .arg(&import_path)
        .env("ENGRAM_DATA_DIR", data_dir)
        .output()
        .map_err(|error| error.to_string())?;
    if !result.status.success() {
        return Err(format!(
            "Engram import: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    Ok(mapping)
}

struct Mcp {
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next_id: u64,
}

impl Mcp {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.next_id += 1;
        let id = self.next_id;
        serde_json::to_writer(
            &mut self.input,
            &json!({"jsonrpc":"2.0", "id":id,
            "method":method,"params":params}),
        )
        .map_err(|error| error.to_string())?;
        self.input
            .write_all(b"\n")
            .map_err(|error| error.to_string())?;
        self.input.flush().map_err(|error| error.to_string())?;
        let mut line = String::new();
        if self
            .output
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            return Err("Engram MCP terminó antes de responder".into());
        }
        let response: Value = serde_json::from_str(&line).map_err(|error| error.to_string())?;
        if response.get("id").and_then(Value::as_u64) != Some(id) {
            return Err("ID MCP inesperado".into());
        }
        if response.get("error").is_some() {
            return Err(format!("error MCP: {}", response["error"]));
        }
        response
            .get("result")
            .cloned()
            .ok_or("falta resultado MCP".into())
    }

    fn search(
        &mut self,
        query: &str,
        owner: &str,
        mapping: &HashMap<String, String>,
    ) -> Result<Vec<String>, String> {
        let result = self.call("tools/call",json!({"name":"mem_search","arguments":{
            "query":query,"project":owner,"match_mode":"any","limit":20,"response_format":"compact"
        }}))?;
        if result.get("isError").and_then(Value::as_bool) == Some(true) {
            return Err(format!("mem_search: {result}"));
        }
        let encoded = result
            .get("content")
            .and_then(Value::as_array)
            .and_then(|parts| {
                parts
                    .iter()
                    .find_map(|part| part.get("text").and_then(Value::as_str))
            })
            .ok_or("respuesta mem_search sin texto")?;
        let body: Value = serde_json::from_str(encoded)
            .map_err(|error| format!("respuesta mem_search: {error}"))?;
        let results = body
            .get("results")
            .and_then(Value::as_array)
            .ok_or("mem_search sin results")?;
        results
            .iter()
            .map(|item| {
                let sync = required(item, "sync_id")?;
                mapping
                    .get(sync)
                    .cloned()
                    .ok_or_else(|| format!("resultado Engram desconocido: {sync}"))
            })
            .collect()
    }
}

pub fn run(
    cases_path: &Path,
    memories: &Path,
    binary: &Path,
    data_dir: &Path,
    output_path: &Path,
    split: &str,
) -> Result<(), String> {
    let absolute_data_dir = if data_dir.is_absolute() {
        data_dir.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(data_dir)
    };
    let data_dir = absolute_data_dir.as_path();
    if data_dir.exists()
        || output_path.exists()
        || output_path.with_extension("manifest.json").exists()
    {
        return Err("elige directorio de datos y salida nuevos".into());
    }
    let cases = read_cases(cases_path, split)?;
    let manifest = json!({"kind":"engram_benchmark","split":split,"cases":cases.len(),
        "cases_sha256":digest(cases_path)?,"memories_sha256":digest(memories)?,
        "binary_sha256":digest(binary)?,"method":"engram_any","query":"keywords"});
    fs::create_dir(data_dir).map_err(|error| error.to_string())?;
    let mapping = import_fixture(memories, binary, data_dir)?;
    let mut child = Command::new(binary)
        .args(["mcp", "--tools=agent"])
        .env("ENGRAM_DATA_DIR", data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| error.to_string())?;
    let mut mcp = Mcp {
        input: child.stdin.take().ok_or("falta stdin MCP")?,
        output: BufReader::new(child.stdout.take().ok_or("falta stdout MCP")?),
        next_id: 0,
    };
    let result = (|| {
        mcp.call(
            "initialize",
            json!({"protocolVersion":"2025-06-18","capabilities":{},
            "clientInfo":{"name":"gennie-memory-eval","version":"0.1"}}),
        )?;
        let first = &cases[0]["request"]["search"];
        mcp.search(
            &keywords(required(first, "query")?),
            &project(required(first, "scope_kind")?, required(first, "scope_id")?),
            &mapping,
        )?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output_path)
            .map_err(|error| error.to_string())?;
        let mut writer = BufWriter::new(file);
        for case in &cases {
            let search = &case["request"]["search"];
            let normalized = keywords(required(search, "query")?);
            let owner = project(
                required(search, "scope_kind")?,
                required(search, "scope_id")?,
            );
            let started = Instant::now();
            let ids = mcp.search(&normalized, &owner, &mapping)?;
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            let mut row = case.as_object().ok_or("caso inválido")?.clone();
            row.remove("request");
            row.insert("elapsed_ms".into(), json!(elapsed_ms));
            row.insert("normalized_query".into(), json!(normalized));
            row.insert("response".into(),json!({"status":"complete","method":"engram_any",
                "semantic_coverage":"disabled","candidates":ids.iter().map(|id|json!({"memory_id":id})).collect::<Vec<_>>() }));
            serde_json::to_writer(&mut writer, &row).map_err(|error| error.to_string())?;
            writer.write_all(b"\n").map_err(|error| error.to_string())?;
        }
        writer.flush().map_err(|error| error.to_string())?;
        writer
            .get_ref()
            .sync_data()
            .map_err(|error| error.to_string())?;
        let manifest_path = output_path.with_extension("manifest.json");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(manifest_path)
            .map_err(|error| error.to_string())?;
        serde_json::to_writer_pretty(&mut file, &manifest).map_err(|error| error.to_string())?;
        file.write_all(b"\n").map_err(|error| error.to_string())?;
        file.sync_data().map_err(|error| error.to_string())?;
        Ok::<(), String>(())
    })();
    drop(mcp);
    if result.is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    result?;
    println!(
        "{} búsquedas Engram MCP; {}",
        cases.len(),
        output_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyword_normalization_matches_frozen_contract() {
        assert_eq!(
            keywords("¿Cuál es el nombre completo de user ficticio 002?"),
            "nombre completo user ficticio 002"
        );
    }
}
