//! Isolated Codex prompt simulation from frozen Rust recall results.

use crate::campaign::digest;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn read_jsonl(path: &Path) -> Result<Vec<Value>, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    BufReader::new(file)
        .lines()
        .enumerate()
        .filter_map(|(index, line)| match line {
            Ok(line) if line.trim().is_empty() => None,
            other => Some((index, other)),
        })
        .map(|(index, line)| {
            let line = line.map_err(|error| error.to_string())?;
            serde_json::from_str(&line).map_err(|error| format!("línea {}: {error}", index + 1))
        })
        .collect()
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("falta {key}"))
}

fn run_one(
    codex: &Path,
    prompt: &str,
    workdir: &Path,
    model: &str,
    engram: Option<(&Path, &Path)>,
) -> Result<(f64, String, Value, Vec<Value>, usize), String> {
    let mut command = Command::new(codex);
    command
        .args([
            "exec",
            "--json",
            "--ephemeral",
            "--ignore-user-config",
            "--ignore-rules",
            "--skip-git-repo-check",
            "-s",
            "read-only",
            "-m",
            model,
            "-C",
        ])
        .arg(workdir);
    if let Some((binary, data_dir)) = engram {
        command.arg("-c").arg(format!(
            "mcp_servers.engram.command={:?}",
            binary.to_string_lossy()
        ));
        command
            .arg("-c")
            .arg("mcp_servers.engram.args=[\"mcp\",\"--tools=agent\"]");
        command.arg("-c").arg(format!(
            "mcp_servers.engram.env.ENGRAM_DATA_DIR={:?}",
            data_dir.to_string_lossy()
        ));
        command.env("ENGRAM_DATA_DIR", data_dir);
    }
    let mut child = command
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("no se pudo iniciar Codex: {error}"))?;
    let started = Instant::now();
    let mut input = child.stdin.take().ok_or("falta entrada de Codex")?;
    input
        .write_all(prompt.as_bytes())
        .map_err(|error| error.to_string())?;
    drop(input);
    let stdout = child.stdout.take().ok_or("falta salida de Codex")?;
    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            if sender.send(Some((elapsed_ms, line))).is_err() {
                return;
            }
        }
        let _ = sender.send(None);
    });
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut events = Vec::new();
    let mut final_answer = None;
    let mut final_answer_ms = None;
    let mut usage = None;
    let mut tool_calls = 0;
    let mut forbidden_tools = 0;
    let result = (|| {
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let entry = receiver
                .recv_timeout(remaining)
                .map_err(|_| "Codex excedió 90 segundos o cerró la salida".to_string())?;
            let Some((elapsed_ms, line)) = entry else {
                break;
            };
            let line = line.map_err(|error| error.to_string())?;
            let event: Value = serde_json::from_str(&line).map_err(|error| error.to_string())?;
            if event.get("type").and_then(Value::as_str) == Some("turn.completed") {
                usage = event.get("usage").cloned();
            }
            if event.get("type").and_then(Value::as_str) == Some("item.completed") {
                let kind = event.pointer("/item/type").and_then(Value::as_str);
                if [
                    Some("mcp_tool_call"),
                    Some("command_execution"),
                    Some("web_search"),
                ]
                .contains(&kind)
                {
                    tool_calls += 1;
                    if kind != Some("mcp_tool_call") || engram.is_none() {
                        forbidden_tools += 1;
                    }
                }
                if kind == Some("agent_message") {
                    final_answer_ms = Some(elapsed_ms);
                    final_answer = event
                        .pointer("/item/text")
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                }
            }
            events.push(json!({"elapsed_ms":elapsed_ms,"event":event}));
        }
        Ok::<(), String>(())
    })();
    if result.is_err() {
        let _ = child.kill();
    }
    let exit = child.wait().map_err(|error| error.to_string())?;
    let _ = reader.join();
    result?;
    if !exit.success() || forbidden_tools != 0 || (engram.is_some() && tool_calls == 0) {
        return Err(format!(
            "turno Codex inválido: proceso {exit}, herramientas {tool_calls}, prohibidas {forbidden_tools}"
        ));
    }
    Ok((
        final_answer_ms.ok_or("falta respuesta final")?,
        final_answer.ok_or("falta texto final")?,
        usage.ok_or("falta uso informado")?,
        events,
        tool_calls,
    ))
}

pub fn run(
    journal: &Path,
    cases_path: &Path,
    selection: &Path,
    codex: &Path,
    output_dir: &Path,
    mode: &str,
    model: &str,
) -> Result<(), String> {
    if !["seed", "control"].contains(&mode) {
        return Err("modo debe ser seed o control".into());
    }
    let rows = read_jsonl(journal)?;
    let manifest = rows.first().ok_or("diario vacío")?;
    if text(manifest, "kind")? != "manifest" {
        return Err("falta manifiesto".into());
    }
    if manifest.get("cases_sha256").and_then(Value::as_str) != Some(digest(cases_path)?.as_str()) {
        return Err("las consultas no coinciden con el diario".into());
    }
    let mut recalled = HashMap::new();
    for row in rows
        .iter()
        .skip(1)
        .filter(|row| row.get("kind").and_then(Value::as_str) == Some("finished"))
    {
        let id = text(row, "id")?;
        if recalled.insert(id, row).is_some() {
            return Err(format!("resultado duplicado: {id}"));
        }
    }
    let mut cases = HashMap::new();
    for case in read_jsonl(cases_path)? {
        let id = text(&case, "id")?.to_owned();
        if cases.insert(id.clone(), case).is_some() {
            return Err(format!("caso duplicado: {id}"));
        }
    }
    let selection: Value =
        serde_json::from_slice(&fs::read(selection).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let ids = selection
        .get("case_ids")
        .and_then(Value::as_array)
        .ok_or("faltan case_ids")?;
    if ids.is_empty() || ids.len() > 50 {
        return Err("case_ids debe contener 1 a 50 elementos".into());
    }
    let mut seen = HashSet::new();
    for id in ids {
        let id = id.as_str().ok_or("ID inválido")?;
        if !seen.insert(id) || !cases.contains_key(id) || !recalled.contains_key(id) {
            return Err(format!("ID duplicado o ausente: {id}"));
        }
    }
    fs::create_dir(output_dir).map_err(|error| format!("usa un directorio nuevo: {error}"))?;
    let workdir = output_dir.join("workdir");
    fs::create_dir(&workdir).map_err(|error| error.to_string())?;
    let mut summary_rows = Vec::new();
    for case_id in ids {
        let id = case_id.as_str().ok_or("ID inválido")?;
        let case = cases.get(id).ok_or("caso ausente")?;
        let result = recalled.get(id).ok_or("recuerdo ausente")?;
        let response = result.get("response").ok_or("falta response")?;
        if response.get("status").and_then(Value::as_str) != Some("complete") {
            return Err(format!("recuerdo degradado en {id}"));
        }
        let seed = response
            .pointer("/seed/seed_text")
            .and_then(Value::as_str)
            .unwrap_or("");
        let seed = if mode == "control" { "" } else { seed };
        let search = case
            .pointer("/request/search")
            .or_else(|| case.get("request"))
            .ok_or("falta consulta")?;
        let prompt = format!(
            "Pregunta del usuario: {}\n\nMemoria semilla preparada antes de este turno:\n{}\n\nResponde brevemente usando solo los hechos de la semilla. Si la semilla no contiene el dato, dilo. No uses herramientas ni archivos.",
            text(search, "query")?,
            if seed.is_empty() {
                "No se encontró una memoria pertinente."
            } else {
                seed
            }
        );
        let (answer_ms, answer, usage, events, tool_calls) =
            run_one(codex, &prompt, &workdir, model, None)?;
        let event_path = output_dir.join(format!("{id}-events.jsonl"));
        let mut event_file = File::create(&event_path).map_err(|error| error.to_string())?;
        for event in events {
            serde_json::to_writer(&mut event_file, &event).map_err(|error| error.to_string())?;
            event_file
                .write_all(b"\n")
                .map_err(|error| error.to_string())?;
        }
        event_file.sync_data().map_err(|error| error.to_string())?;
        summary_rows.push(json!({"case_id":id,"expected_ids":case.get("expected_ids"),
            "selected_ids":response.pointer("/seed/memories").and_then(Value::as_array).ok_or("falta seed.memories")?
                .iter().map(|item| item.get("memory_id").cloned().unwrap_or(Value::Null)).collect::<Vec<_>>(),
            "seed_text_utf8_bytes":seed.len(),"seed_text_words":seed.split_whitespace().count(),
            "prompt_utf8_bytes":prompt.len(),"final_answer_ms":answer_ms,"final_answer":answer,
            "usage":usage,"tool_calls":tool_calls}));
        println!("{id}: {:.1} ms", answer_ms);
    }
    fs::remove_dir(&workdir).map_err(|error| error.to_string())?;
    let summary = json!({"kind":"rust_codex_initial_seed_prompt_simulation",
        "seed_injected":mode=="seed","codex_model":model,
        "case_ids":ids,"source":journal,"rows":summary_rows});
    fs::write(
        output_dir.join("summary.json"),
        serde_json::to_string_pretty(&summary).map_err(|error| error.to_string())? + "\n",
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn run_engram(
    cases_path: &Path,
    selection: &Path,
    codex: &Path,
    engram: &Path,
    data_dir: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    if !data_dir.exists() {
        return Err("la base Engram aislada no existe".into());
    }
    let data_dir = data_dir.canonicalize().map_err(|error| error.to_string())?;
    let engram = engram.canonicalize().map_err(|error| error.to_string())?;
    let mut cases = HashMap::new();
    for case in read_jsonl(cases_path)? {
        let id = text(&case, "id")?.to_owned();
        if cases.insert(id.clone(), case).is_some() {
            return Err(format!("caso duplicado: {id}"));
        }
    }
    let selection: Value =
        serde_json::from_slice(&fs::read(selection).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let ids = selection
        .get("case_ids")
        .and_then(Value::as_array)
        .ok_or("faltan case_ids")?;
    if ids.is_empty() || ids.len() > 20 {
        return Err("case_ids debe contener 1 a 20 elementos".into());
    }
    let mut seen = HashSet::new();
    for value in ids {
        let id = value.as_str().ok_or("ID inválido")?;
        if !seen.insert(id) || !cases.contains_key(id) {
            return Err(format!("ID duplicado o ausente: {id}"));
        }
    }
    fs::create_dir(output_dir).map_err(|error| format!("usa un directorio nuevo: {error}"))?;
    let workdir = output_dir.join("workdir");
    fs::create_dir(&workdir).map_err(|error| error.to_string())?;
    let mut summary_rows = Vec::new();
    for value in ids {
        let id = value.as_str().ok_or("ID inválido")?;
        let case = cases.get(id).ok_or("caso ausente")?;
        let search = case
            .pointer("/request/search")
            .or_else(|| case.get("request"))
            .ok_or("falta request")?;
        let owner = format!(
            "{}:{}",
            text(search, "scope_kind")?,
            text(search, "scope_id")?
        );
        let prompt = format!(
            "Pregunta del usuario: {}\n\nBusca en Engram mediante mem_search dentro del proyecto exacto {} y responde brevemente solo con la memoria encontrada. Si no hay una memoria pertinente, dilo. No uses archivos, shell ni otras herramientas.",
            text(search, "query")?,
            owner
        );
        let (answer_ms, answer, usage, events, tool_calls) = run_one(
            codex,
            &prompt,
            &workdir,
            "gpt-6.1-sol",
            Some((&engram, &data_dir)),
        )?;
        let event_path = output_dir.join(format!("{id}-events.jsonl"));
        let mut event_file = File::create(event_path).map_err(|error| error.to_string())?;
        for event in events {
            serde_json::to_writer(&mut event_file, &event).map_err(|error| error.to_string())?;
            event_file
                .write_all(b"\n")
                .map_err(|error| error.to_string())?;
        }
        event_file.sync_data().map_err(|error| error.to_string())?;
        summary_rows.push(json!({"case_id":id,"expected_ids":case.get("expected_ids"),
            "expected_fact":case.get("expected_fact"),"final_answer_ms":answer_ms,
            "final_answer":answer,"usage":usage,"tool_calls":tool_calls}));
        println!("{id}: {:.1} ms, {tool_calls} MCP", answer_ms);
    }
    fs::remove_dir(workdir).map_err(|error| error.to_string())?;
    let summary = json!({"kind":"rust_codex_engram_mcp_synthetic",
        "codex_model":"gpt-6.1-sol","cases_sha256":digest(cases_path)?,
        "engram_binary_sha256":digest(&engram)?,"case_ids":ids,"rows":summary_rows});
    fs::write(
        output_dir.join("summary.json"),
        serde_json::to_string_pretty(&summary).map_err(|error| error.to_string())? + "\n",
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn number(value: &Value, key: &str) -> Result<f64, String> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("falta número {key}"))
}

fn median(mut values: Vec<f64>) -> Result<f64, String> {
    if values.is_empty() {
        return Err("no hay muestras".into());
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    Ok(if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    })
}

pub fn paired_report(
    journal: &Path,
    seed_path: &Path,
    engram_path: &Path,
    control_path: &Path,
    output: &Path,
) -> Result<(), String> {
    let load = |path: &Path| -> Result<Value, String> {
        serde_json::from_slice(&fs::read(path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())
    };
    let seed = load(seed_path)?;
    let engram = load(engram_path)?;
    let control = load(control_path)?;
    let rows = |value: &Value| -> Result<HashMap<String, Value>, String> {
        let mut indexed = HashMap::new();
        for row in value
            .get("rows")
            .and_then(Value::as_array)
            .ok_or("faltan rows")?
        {
            let id = text(row, "case_id")?.to_owned();
            if indexed.insert(id.clone(), row.clone()).is_some() {
                return Err(format!("caso duplicado: {id}"));
            }
        }
        Ok(indexed)
    };
    let seed_rows = rows(&seed)?;
    let engram_rows = rows(&engram)?;
    let control_rows = rows(&control)?;
    if seed_rows.len() != engram_rows.len() || seed_rows.len() != control_rows.len() {
        return Err("las tres campañas no tienen los mismos casos".into());
    }
    let mut recall_rows = HashMap::new();
    for row in read_jsonl(journal)? {
        if row.get("kind").and_then(Value::as_str) == Some("finished") {
            let id = text(&row, "id")?.to_owned();
            if recall_rows.insert(id.clone(), row).is_some() {
                return Err(format!("recall duplicado: {id}"));
            }
        }
    }
    let mut ids = seed_rows.keys().cloned().collect::<Vec<_>>();
    ids.sort();
    let mut paired = Vec::new();
    let mut prep_ms = Vec::new();
    let mut seed_ms = Vec::new();
    let mut engram_ms = Vec::new();
    let mut calculated_ms = Vec::new();
    let mut token_deltas = Vec::new();
    for id in ids {
        let recall = recall_rows.get(&id).ok_or(format!("falta recall {id}"))?;
        let seed = seed_rows.get(&id).ok_or("falta seed")?;
        let engram = engram_rows.get(&id).ok_or(format!("falta Engram {id}"))?;
        let control = control_rows.get(&id).ok_or(format!("falta control {id}"))?;
        let prep = number(recall, "elapsed_ms")?;
        let codex_seed = number(seed, "final_answer_ms")?;
        let codex_engram = number(engram, "final_answer_ms")?;
        let seed_tokens = seed
            .pointer("/usage/input_tokens")
            .and_then(Value::as_i64)
            .ok_or("faltan tokens seed")?;
        let control_tokens = control
            .pointer("/usage/input_tokens")
            .and_then(Value::as_i64)
            .ok_or("faltan tokens control")?;
        let total = prep + codex_seed;
        prep_ms.push(prep);
        seed_ms.push(codex_seed);
        engram_ms.push(codex_engram);
        calculated_ms.push(total);
        token_deltas.push((seed_tokens - control_tokens) as f64);
        paired.push(
            json!({"case_id":id,"recall_ms":prep,"codex_seed_ms":codex_seed,
            "calculated_seed_total_ms":total,"codex_engram_ms":codex_engram,
            "seed_vs_control_input_token_delta":seed_tokens-control_tokens}),
        );
    }
    if output.exists() {
        return Err("el archivo de salida ya existe".into());
    }
    let report = json!({"kind":"rust_paired_flow_calculation",
        "measurement_note":"La suma recall + Codex con semilla combina dos ejecuciones; no mide una integración automática.",
        "sample_count":paired.len(),"median_recall_ms":median(prep_ms)?,
        "median_codex_seed_ms":median(seed_ms)?,"median_codex_engram_ms":median(engram_ms)?,
        "median_calculated_seed_total_ms":median(calculated_ms)?,
        "median_seed_vs_control_input_token_delta":median(token_deltas)?,
        "rows":paired,"sources":{
            "recall_sha256":digest(journal)?,"seed_sha256":digest(seed_path)?,
            "engram_sha256":digest(engram_path)?,"control_sha256":digest(control_path)?}});
    fs::write(
        output,
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())? + "\n",
    )
    .map_err(|error| error.to_string())
}
