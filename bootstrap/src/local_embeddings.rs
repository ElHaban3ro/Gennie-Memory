use gennie_memory_retrieval::local_embeddings::{Encoder, Settings, index};
use rusqlite::Connection;
use serde::Deserialize;
use std::{
    io::{BufRead, Write},
    time::Instant,
};

#[derive(Deserialize)]
#[serde(
    tag = "op",
    content = "request",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum Operation {
    Prepare(index::IndexRequest),
    Publish(index::PreparedIndex),
    PublishFile { path: String },
    LoadFixtures(Vec<gennie_memory_retrieval::model::FixtureMemory>),
    Index(index::IndexRequest),
    QueueRun(index::IndexRequest),
    Search(index::Search),
    TextSearch(gennie_memory_retrieval::model::SearchRequest),
    Segment { text: String },
}

pub fn run(conn: &mut Connection, settings: &Settings) -> Result<(), String> {
    let start = Instant::now();
    let encoder = Encoder::load(settings)?;
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    writeln!(output,"{}",serde_json::json!({"status":"ready","space_id":encoder.profile.space_id(),"model_load_ms":start.elapsed().as_secs_f64()*1000.0,"profile":encoder.profile})).map_err(|e|e.to_string())?;
    output.flush().map_err(|e| e.to_string())?;
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    loop {
        let mut line = String::new();
        let n = std::io::Read::take(&mut input, 64 * 1024 * 1024 + 1)
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        if n > 64 * 1024 * 1024 {
            return Err("embedding_session_input_limit: máximo 64 MiB por línea".into());
        }
        if line.trim().is_empty() {
            continue;
        }
        let start = Instant::now();
        let result: Result<serde_json::Value, String> = (|| {
            let op: Operation = serde_json::from_str(&line).map_err(|e| e.to_string())?;
            match op {
                Operation::QueueRun(request) => index::process_job(conn, &encoder, &request),
                Operation::Prepare(request) => {
                    serde_json::to_value(index::prepare(conn, &encoder, &request)?)
                        .map_err(|e| e.to_string())
                }
                Operation::Publish(request) => {
                    serde_json::to_value(index::publish(conn, &encoder, &request)?)
                        .map_err(|e| e.to_string())
                }
                Operation::PublishFile { path } => {
                    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
                    if file.metadata().map_err(|e| e.to_string())?.len() > 512 * 1024 * 1024 {
                        return Err("embedding_import_too_large".into());
                    }
                    let batch = serde_json::from_reader(std::io::BufReader::new(file))
                        .map_err(|e| e.to_string())?;
                    serde_json::to_value(index::publish(conn, &encoder, &batch)?)
                        .map_err(|e| e.to_string())
                }
                Operation::LoadFixtures(fixtures) => {
                    gennie_memory_retrieval::repo::load_fixtures(conn, &fixtures)?;
                    Ok(serde_json::json!({"loaded":fixtures.len()}))
                }
                Operation::Index(request) => {
                    let batch = index::prepare(conn, &encoder, &request)?;
                    serde_json::to_value(index::publish(conn, &encoder, &batch)?)
                        .map_err(|e| e.to_string())
                }
                Operation::Search(request) => {
                    serde_json::to_value(index::search(conn, &encoder, &request)?)
                        .map_err(|e| e.to_string())
                }
                Operation::TextSearch(request) => {
                    serde_json::to_value(gennie_memory_retrieval::service::search(conn, &request)?)
                        .map_err(|e| e.to_string())
                }
                Operation::Segment { text } => {
                    serde_json::to_value(encoder.fragments(&text)?).map_err(|e| e.to_string())
                }
            }
        })();
        let value = match result {
            Ok(response) => {
                serde_json::json!({"status":"complete","response":response,"elapsed_ms":start.elapsed().as_secs_f64()*1000.0})
            }
            Err(error) => {
                serde_json::json!({"status":"failed","error":error,"elapsed_ms":start.elapsed().as_secs_f64()*1000.0})
            }
        };
        writeln!(output, "{value}").map_err(|e| e.to_string())?;
        output.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}
