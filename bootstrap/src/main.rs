#[cfg(test)]
#[path = "../../tools/evaluation/src/independent_projection.rs"]
mod independent_projection;
mod openrouter;
#[cfg(test)]
#[path = "../../tools/evaluation/src/projection_contract.rs"]
mod projection_contract;
#[cfg(test)]
#[path = "../../tools/evaluation/src/projection_ids.rs"]
mod projection_ids;
#[cfg(test)]
#[path = "../../tools/evaluation/src/projection_sets.rs"]
mod projection_sets;
#[cfg(test)]
#[path = "../../tools/evaluation/src/projection_tokens.rs"]
mod projection_tokens;
mod registry;
mod session;
mod witness;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::Instant;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchCase {
    id: String,
    split: String,
    segment: String,
    request: iromem_retrieval::model::SearchRequest,
    expected_ids: Vec<String>,
    expected_fact: Option<String>,
    #[serde(default)]
    expected_condition: Option<String>,
    #[serde(default)]
    expected_source: Option<iromem_retrieval::model::SourceRef>,
    forbidden_ids: Vec<String>,
    label_source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HybridBatchCase {
    id: String,
    split: String,
    segment: String,
    request: iromem_retrieval::model::HybridRequest,
    expected_ids: Vec<String>,
    expected_fact: Option<String>,
    #[serde(default)]
    expected_condition: Option<String>,
    #[serde(default)]
    expected_source: Option<iromem_retrieval::model::SourceRef>,
    forbidden_ids: Vec<String>,
    label_source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedBatchCase {
    id: String,
    split: String,
    segment: String,
    request: iromem_retrieval::model::SeedRequest,
    expected_ids: Vec<String>,
    expected_fact: Option<String>,
    #[serde(default)]
    expected_condition: Option<String>,
    #[serde(default)]
    expected_source: Option<iromem_retrieval::model::SourceRef>,
    forbidden_ids: Vec<String>,
    label_source: String,
}

#[derive(Serialize)]
struct BatchResult<'a, R: Serialize> {
    id: &'a str,
    split: &'a str,
    segment: &'a str,
    expected_ids: &'a [String],
    expected_fact: &'a Option<String>,
    expected_condition: &'a Option<String>,
    expected_source: &'a Option<iromem_retrieval::model::SourceRef>,
    forbidden_ids: &'a [String],
    label_source: &'a str,
    elapsed_ms: f64,
    response: R,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("iromem: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let command = args
        .next()
        .ok_or("uso: iromem <init-db|load-fixtures|load-embeddings|search|recall|hybrid-search|search-batch|hybrid-search-batch|compose-seed|compose-seed-batch|session|dashboard> [archivo]")?;
    let config = iromem_kernel::config::load()?;
    let mut conn = iromem_kernel::db::connect(&config.database_path)?;
    let modules = registry::modules()?;
    iromem_kernel::db::migrate(&mut conn, &modules)?;
    match command.as_str() {
        "dashboard" => {
            let port: u16 = args
                .next()
                .unwrap_or_else(|| "8787".into())
                .parse()
                .map_err(|_| "puerto inválido")?;
            let case_path = args.next();
            let bind_ip: std::net::Ipv4Addr = args
                .next()
                .unwrap_or_else(|| "127.0.0.1".into())
                .parse()
                .map_err(|_| "IP del dashboard inválida")?;
            if args.next().is_some() {
                return Err("uso: iromem dashboard [puerto] [casos.jsonl] [IP-local]".into());
            }
            let dist = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../apps/dashboard/web/dist");
            iromem_dashboard::web::run(
                conn,
                port,
                case_path.as_deref().map(std::path::Path::new),
                dist,
                bind_ip,
            )?;
        }
        "session" => {
            let secrets = iromem_kernel::config::load_secrets()?;
            let classifier = if let Some(response_content_path) =
                config.evaluation_response_content_path.as_deref()
            {
                openrouter::OpenRouterClassifier::new_with_evaluation_capture_and_content(
                    secrets.openrouter_api_key,
                    config.evaluation_capture_path.as_deref(),
                    Some(response_content_path),
                )?
            } else {
                openrouter::OpenRouterClassifier::new_with_evaluation_capture(
                    secrets.openrouter_api_key,
                    config.evaluation_capture_path.as_deref(),
                )?
            };
            session::run(&conn, &classifier)?;
        }
        "init-db" => println!("SQLite listo: {}", config.database_path.display()),
        "load-fixtures" => {
            let path = args.next().ok_or("falta archivo JSON de fixtures")?;
            let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let fixtures: Vec<iromem_retrieval::model::FixtureMemory> =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            iromem_retrieval::repo::load_fixtures(&mut conn, &fixtures)?;
            println!("{} revisiones cargadas", fixtures.len());
        }
        "load-embeddings" => {
            let path = args.next().ok_or("falta archivo JSON de embeddings")?;
            let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let batch: iromem_retrieval::model::EmbeddingBatch =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            let loaded = iromem_retrieval::embedding::load_batch(&mut conn, &batch)?;
            println!("{loaded} vectores cargados en {}", batch.space_id);
        }
        "search" => {
            let path = args.next().ok_or("falta archivo JSON de consulta")?;
            let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let request: iromem_retrieval::model::SearchRequest =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            let response = iromem_retrieval::service::search(&conn, &request)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())?
            );
        }
        "recall" => {
            let path = args
                .next()
                .ok_or("falta archivo JSON de consulta de recuerdo")?;
            let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
            if data.len() > 1_000_000 {
                return Err("la solicitud supera 1 MB".into());
            }
            let request: iromem_retrieval::model::RecallRequest =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            let response = if let Some(reviewed) =
                iromem_dashboard::service::reviewed_recall(&conn, &request)?
            {
                reviewed
            } else {
                let secrets = iromem_kernel::config::load_secrets()?;
                let classifier = if let Some(response_content_path) =
                    config.evaluation_response_content_path.as_deref()
                {
                    openrouter::OpenRouterClassifier::new_with_evaluation_capture_and_content(
                        secrets.openrouter_api_key,
                        config.evaluation_capture_path.as_deref(),
                        Some(response_content_path),
                    )?
                } else {
                    openrouter::OpenRouterClassifier::new_with_evaluation_capture(
                        secrets.openrouter_api_key,
                        config.evaluation_capture_path.as_deref(),
                    )?
                };
                iromem_retrieval::service::recall(&conn, &request, &classifier)?
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())?
            );
        }
        "hybrid-search" => {
            let path = args
                .next()
                .ok_or("falta archivo JSON de consulta híbrida")?;
            let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let request: iromem_retrieval::model::HybridRequest =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            let response = iromem_retrieval::embedding::search_hybrid(&conn, &request)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())?
            );
        }
        "search-batch" => {
            let path = args.next().ok_or("falta archivo JSONL de consultas")?;
            let file = fs::File::open(path).map_err(|e| e.to_string())?;
            let mut output = BufWriter::new(std::io::stdout().lock());
            for (line_number, line) in BufReader::new(file).lines().enumerate() {
                let line = line.map_err(|e| e.to_string())?;
                if line.trim().is_empty() {
                    continue;
                }
                let case: BatchCase = serde_json::from_str(&line)
                    .map_err(|e| format!("línea {}: {e}", line_number + 1))?;
                let started = Instant::now();
                let response = iromem_retrieval::service::search(&conn, &case.request)
                    .map_err(|e| format!("caso {}: {e}", case.id))?;
                let result = BatchResult {
                    id: &case.id,
                    split: &case.split,
                    segment: &case.segment,
                    expected_ids: &case.expected_ids,
                    expected_fact: &case.expected_fact,
                    expected_condition: &case.expected_condition,
                    expected_source: &case.expected_source,
                    forbidden_ids: &case.forbidden_ids,
                    label_source: &case.label_source,
                    elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
                    response,
                };
                serde_json::to_writer(&mut output, &result).map_err(|e| e.to_string())?;
                output.write_all(b"\n").map_err(|e| e.to_string())?;
            }
            output.flush().map_err(|e| e.to_string())?;
        }
        "hybrid-search-batch" => {
            let path = args
                .next()
                .ok_or("falta archivo JSONL de consultas híbridas")?;
            let file = fs::File::open(path).map_err(|e| e.to_string())?;
            let mut output = BufWriter::new(std::io::stdout().lock());
            for (line_number, line) in BufReader::new(file).lines().enumerate() {
                let line = line.map_err(|e| e.to_string())?;
                if line.trim().is_empty() {
                    continue;
                }
                let case: HybridBatchCase = serde_json::from_str(&line)
                    .map_err(|e| format!("línea {}: {e}", line_number + 1))?;
                let started = Instant::now();
                let response = iromem_retrieval::embedding::search_hybrid(&conn, &case.request)
                    .map_err(|e| format!("caso {}: {e}", case.id))?;
                let result = BatchResult {
                    id: &case.id,
                    split: &case.split,
                    segment: &case.segment,
                    expected_ids: &case.expected_ids,
                    expected_fact: &case.expected_fact,
                    expected_condition: &case.expected_condition,
                    expected_source: &case.expected_source,
                    forbidden_ids: &case.forbidden_ids,
                    label_source: &case.label_source,
                    elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
                    response,
                };
                serde_json::to_writer(&mut output, &result).map_err(|e| e.to_string())?;
                output.write_all(b"\n").map_err(|e| e.to_string())?;
            }
            output.flush().map_err(|e| e.to_string())?;
        }
        "compose-seed" => {
            let path = args.next().ok_or("falta archivo JSON de selección")?;
            let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let request: iromem_retrieval::model::SeedRequest =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            let response = iromem_retrieval::service::compose_seed(&conn, &request)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())?
            );
        }
        "compose-seed-batch" => {
            let path = args.next().ok_or("falta archivo JSONL de selecciones")?;
            let file = fs::File::open(path).map_err(|e| e.to_string())?;
            let mut output = BufWriter::new(std::io::stdout().lock());
            for (line_number, line) in BufReader::new(file).lines().enumerate() {
                let line = line.map_err(|e| e.to_string())?;
                if line.trim().is_empty() {
                    continue;
                }
                let case: SeedBatchCase = serde_json::from_str(&line)
                    .map_err(|e| format!("línea {}: {e}", line_number + 1))?;
                let started = Instant::now();
                let response = iromem_retrieval::service::compose_seed(&conn, &case.request)
                    .map_err(|e| format!("caso {}: {e}", case.id))?;
                let result = BatchResult {
                    id: &case.id,
                    split: &case.split,
                    segment: &case.segment,
                    expected_ids: &case.expected_ids,
                    expected_fact: &case.expected_fact,
                    expected_condition: &case.expected_condition,
                    expected_source: &case.expected_source,
                    forbidden_ids: &case.forbidden_ids,
                    label_source: &case.label_source,
                    elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
                    response,
                };
                serde_json::to_writer(&mut output, &result).map_err(|e| e.to_string())?;
                output.write_all(b"\n").map_err(|e| e.to_string())?;
            }
            output.flush().map_err(|e| e.to_string())?;
        }
        _ => {
            return Err(
                "comando desconocido: use init-db, load-fixtures, load-embeddings, search, recall, hybrid-search, search-batch, hybrid-search-batch, compose-seed, compose-seed-batch o session".into(),
            );
        }
    }
    Ok(())
}
