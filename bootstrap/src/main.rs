#[cfg(test)]
#[path = "../../tools/evaluation/src/independent_projection.rs"]
mod independent_projection;
#[cfg(feature = "local-embeddings")]
mod local_embeddings;
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
mod retired_migrations;
mod session;
mod witness;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::time::Instant;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchCase {
    id: String,
    split: String,
    segment: String,
    request: gennie_memory_retrieval::model::SearchRequest,
    expected_ids: Vec<String>,
    expected_fact: Option<String>,
    #[serde(default)]
    expected_condition: Option<String>,
    #[serde(default)]
    expected_source: Option<gennie_memory_retrieval::model::SourceRef>,
    forbidden_ids: Vec<String>,
    label_source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HybridBatchCase {
    id: String,
    split: String,
    segment: String,
    request: gennie_memory_retrieval::model::HybridRequest,
    expected_ids: Vec<String>,
    expected_fact: Option<String>,
    #[serde(default)]
    expected_condition: Option<String>,
    #[serde(default)]
    expected_source: Option<gennie_memory_retrieval::model::SourceRef>,
    forbidden_ids: Vec<String>,
    label_source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedBatchCase {
    id: String,
    split: String,
    segment: String,
    request: gennie_memory_retrieval::model::SeedRequest,
    expected_ids: Vec<String>,
    expected_fact: Option<String>,
    #[serde(default)]
    expected_condition: Option<String>,
    #[serde(default)]
    expected_source: Option<gennie_memory_retrieval::model::SourceRef>,
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
    expected_source: &'a Option<gennie_memory_retrieval::model::SourceRef>,
    forbidden_ids: &'a [String],
    label_source: &'a str,
    elapsed_ms: f64,
    response: R,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("gennie-memory: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let command = args
        .next()
        .ok_or("uso: gennie-memory <init-db|mem-lifecycle|mem-review-list|embedding-job|source-register|source-verify|entity-resolve|mem-resolve|mem-proposal-get|mem-audit|mem-operation-get|mem-write|mem-write-batch|load-fixtures|load-embeddings|search|recall|recall-v1|segment-text|provider-capabilities|hybrid-search|search-batch|hybrid-search-batch|compose-seed|compose-seed-batch|session> [archivo]")?;
    if command == "segment-text" {
        let path = args
            .next()
            .ok_or("uso: gennie-memory segment-text <archivo-texto-UTF8>")?;
        if args.next().is_some() {
            return Err("segment-text admite un archivo de texto".into());
        }
        let text = read_bounded(&path, gennie_memory_retrieval::segmentation::MAX_TEXT_BYTES)?;
        let result = gennie_memory_retrieval::segmentation::segment(&text);
        let output = match result {
            Ok(segmentation) => {
                serde_json::json!({"status":"completed", "segmentation":segmentation, "issue":null})
            }
            Err(code) => {
                serde_json::json!({"status":"failed", "segmentation":null, "issue":{"code":code,"message":"El texto no se pudo segmentar íntegramente dentro de los límites."}})
            }
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .map_err(|_| "no se pudo serializar la segmentación")?
        );
        return Ok(());
    }
    if command == "provider-capabilities" {
        println!(
            "{}",
            serde_json::to_string_pretty(&gennie_memory_retrieval::product::capabilities())
                .map_err(|_| "no se pudo serializar el catálogo")?
        );
        return Ok(());
    }
    let config = gennie_memory_kernel::config::load()?;
    let mut conn = gennie_memory_kernel::db::connect(&config.database_path)?;
    registry::migrate(&mut conn)?;
    match command.as_str() {
        #[cfg(feature = "local-embeddings")]
        "embedding-session" => {
            let path = args
                .next()
                .ok_or("uso: gennie-memory embedding-session <settings.json>")?;
            if args.next().is_some() {
                return Err("embedding-session admite un archivo de configuración".into());
            }
            let settings =
                serde_json::from_str(&read_bounded(&path, 32_768)?).map_err(|e| e.to_string())?;
            local_embeddings::run(&mut conn, &settings)?;
        }
        "entity-resolve" => {
            let path = args
                .next()
                .ok_or("uso: gennie-memory entity-resolve <solicitud.json>")?;
            if args.next().is_some() {
                return Err("entity-resolve admite un archivo JSON".into());
            }
            let input = read_bounded(&path, 32_768)?;
            let request = serde_json::from_str(&input).map_err(|e| e.to_string())?;
            let result = gennie_memory_retrieval::entity::resolve(&mut conn, &request)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?
            );
        }
        "recall-v1" => {
            let path = args
                .next()
                .ok_or("uso: gennie-memory recall-v1 <solicitud.json>")?;
            if args.next().is_some() {
                return Err("recall-v1 admite un archivo JSON".into());
            }
            let input = read_bounded(&path, 131_072)?;
            let secrets = gennie_memory_kernel::config::load_secrets()?;
            let classifier =
                openrouter::OpenRouterClassifier::new_with_evaluation_capture_and_content(
                    secrets.openrouter_api_key,
                    config.evaluation_capture_path.as_deref(),
                    config.evaluation_response_content_path.as_deref(),
                )?;
            let response =
                gennie_memory_retrieval::product::execute_json(&conn, &input, &classifier);
            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .map_err(|_| "no se pudo serializar la respuesta")?
            );
        }
        "session" => {
            let secrets = gennie_memory_kernel::config::load_secrets()?;
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
            session::run(&mut conn, &classifier)?;
        }
        "mem-lifecycle" | "mem-review-list" | "embedding-job" => {
            let path = args.next().ok_or("indica un archivo JSON")?;
            if args.next().is_some() {
                return Err("el comando admite un archivo JSON".into());
            }
            let data = read_bounded(&path, 65_536)?;
            let response = match command.as_str() {
                "mem-lifecycle" => gennie_memory_retrieval::lifecycle::apply(
                    &mut conn,
                    &serde_json::from_str(&data).map_err(|e| format!("solicitud inválida: {e}"))?,
                )?,
                "mem-review-list" => gennie_memory_retrieval::lifecycle::review_list(
                    &conn,
                    &serde_json::from_str(&data).map_err(|e| format!("solicitud inválida: {e}"))?,
                )?,
                _ => gennie_memory_retrieval::lifecycle::job(
                    &mut conn,
                    &serde_json::from_str(&data).map_err(|e| format!("solicitud inválida: {e}"))?,
                )?,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())?
            );
        }
        "init-db" => println!("SQLite listo: {}", config.database_path.display()),
        "source-register" | "source-verify" => {
            let path = args.next().ok_or("indica un archivo JSON de fuente")?;
            if args.next().is_some() {
                return Err("el comando de fuente admite un archivo JSON".into());
            }
            let data = read_bounded(
                &path,
                if command == "source-register" {
                    2_097_152
                } else {
                    65_536
                },
            )?;
            let response = if command == "source-register" {
                let request = serde_json::from_str(&data)
                    .map_err(|_| "solicitud source-register inválida")?;
                serde_json::to_value(gennie_memory_retrieval::source_verification::register(
                    &conn, &request,
                )?)
            } else {
                let request =
                    serde_json::from_str(&data).map_err(|_| "solicitud source-verify inválida")?;
                serde_json::to_value(gennie_memory_retrieval::source_verification::verify(
                    &conn, &request,
                )?)
            }
            .map_err(|_| "no se pudo serializar la comprobación de fuente")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .map_err(|_| "no se pudo serializar la fuente")?
            );
        }
        "mem-resolve" | "mem-proposal-get" | "mem-audit" | "mem-operation-get" => {
            let path = args.next().ok_or("falta solicitud JSON")?;
            if args.next().is_some() {
                return Err("se admite un archivo JSON".into());
            }
            let data = read_bounded(&path, 131_072)?;
            let response = if command == "mem-resolve" {
                let request =
                    serde_json::from_str(&data).map_err(|e| format!("solicitud inválida: {e}"))?;
                gennie_memory_retrieval::reconciliation::resolve(&mut conn, &request)?
            } else {
                let request =
                    serde_json::from_str(&data).map_err(|e| format!("solicitud inválida: {e}"))?;
                match command.as_str() {
                    "mem-proposal-get" => {
                        gennie_memory_retrieval::reconciliation::proposal_get(&conn, &request)?
                    }
                    "mem-audit" => gennie_memory_retrieval::reconciliation::audit(&conn, &request)?,
                    _ => gennie_memory_retrieval::reconciliation::operation_get(&conn, &request)?,
                }
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())?
            );
        }
        "mem-write" => {
            let path = args
                .next()
                .ok_or("uso: gennie-memory mem-write <solicitud.json>")?;
            if args.next().is_some() {
                return Err("mem-write admite un archivo JSON".into());
            }
            let data = read_bounded(&path, 65_536)?;
            let request: gennie_memory_retrieval::reconciliation::WriteCommand =
                serde_json::from_str(&data)
                    .map_err(|e| format!("solicitud mem-write inválida: {e}"))?;
            let secrets = gennie_memory_kernel::config::load_secrets()?;
            let classifier = openrouter::OpenRouterClassifier::new(secrets.openrouter_api_key)?;
            let response =
                gennie_memory_retrieval::reconciliation::write(&mut conn, &request, &classifier)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .map_err(|_| "no se pudo serializar el resultado")?
            );
        }
        "mem-write-batch" => {
            let path = args
                .next()
                .ok_or("uso: gennie-memory mem-write-batch <solicitudes.jsonl>")?;
            if args.next().is_some() {
                return Err("mem-write-batch admite un archivo JSONL".into());
            }
            let data = read_bounded(&path, 67_108_864)?;
            let requests = data
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| {
                    serde_json::from_str::<gennie_memory_retrieval::reconciliation::WriteCommand>(
                        line,
                    )
                    .map_err(|e| format!("solicitud JSONL inválida: {e}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let secrets = gennie_memory_kernel::config::load_secrets()?;
            let classifier = openrouter::OpenRouterClassifier::new(secrets.openrouter_api_key)?;
            let results = gennie_memory_retrieval::reconciliation::write_batch(
                &mut conn,
                &requests,
                &classifier,
            )?;
            for result in results {
                println!(
                    "{}",
                    serde_json::to_string(&result)
                        .map_err(|_| "no se pudo serializar el resultado")?
                );
            }
        }
        "load-fixtures" => {
            let path = args.next().ok_or("falta archivo JSON de fixtures")?;
            let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let fixtures: Vec<gennie_memory_retrieval::model::FixtureMemory> =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            gennie_memory_retrieval::repo::load_fixtures(&mut conn, &fixtures)?;
            println!("{} revisiones cargadas", fixtures.len());
        }
        "load-embeddings" => {
            let path = args.next().ok_or("falta archivo JSON de embeddings")?;
            let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let batch: gennie_memory_retrieval::model::EmbeddingBatch =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            let loaded = gennie_memory_retrieval::embedding::load_batch(&mut conn, &batch)?;
            println!("{loaded} vectores cargados en {}", batch.space_id);
        }
        "search" => {
            let path = args.next().ok_or("falta archivo JSON de consulta")?;
            let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let request: gennie_memory_retrieval::model::SearchRequest =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            let response = gennie_memory_retrieval::service::search(&conn, &request)?;
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
            let request: gennie_memory_retrieval::model::RecallRequest =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            let secrets = gennie_memory_kernel::config::load_secrets()?;
            let classifier =
                openrouter::OpenRouterClassifier::new_with_evaluation_capture_and_content(
                    secrets.openrouter_api_key,
                    config.evaluation_capture_path.as_deref(),
                    config.evaluation_response_content_path.as_deref(),
                )?;
            let response = gennie_memory_retrieval::service::recall(&conn, &request, &classifier)?;
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
            let request: gennie_memory_retrieval::model::HybridRequest =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            let response = gennie_memory_retrieval::embedding::search_hybrid(&conn, &request)?;
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
                let response = gennie_memory_retrieval::service::search(&conn, &case.request)
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
                let response =
                    gennie_memory_retrieval::embedding::search_hybrid(&conn, &case.request)
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
            let request: gennie_memory_retrieval::model::SeedRequest =
                serde_json::from_str(&data).map_err(|e| e.to_string())?;
            let response = gennie_memory_retrieval::service::compose_seed(&conn, &request)?;
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
                let response = gennie_memory_retrieval::service::compose_seed(&conn, &case.request)
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
                "comando desconocido: use init-db, source-register, source-verify, entity-resolve, mem-lifecycle, mem-review-list, embedding-job, mem-resolve, mem-proposal-get, mem-audit, mem-operation-get, mem-write, mem-write-batch, load-fixtures, load-embeddings, search, recall, recall-v1, segment-text, provider-capabilities, hybrid-search, search-batch, hybrid-search-batch, compose-seed, compose-seed-batch o session".into(),
            );
        }
    }
    Ok(())
}

fn read_bounded(path: &str, limit: usize) -> Result<String, String> {
    let mut text = String::new();
    fs::File::open(path)
        .map_err(|_| "no se pudo abrir la entrada")?
        .take(limit as u64 + 1)
        .read_to_string(&mut text)
        .map_err(|_| "no se pudo leer la entrada UTF-8")?;
    if text.len() > limit {
        return Err(format!("la entrada supera {limit} bytes"));
    }
    Ok(text)
}
