//! Local JSONL transport for measuring a complete retrieval/classification/seed run.

use gennie_memory_retrieval::classifier::Classifier;
use gennie_memory_retrieval::model::{HybridRequest, RecallRequest, SearchRequest, SeedRequest};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufWriter, Write};
use std::time::Instant;

#[derive(Deserialize)]
#[serde(tag = "op", content = "request", rename_all = "snake_case")]
enum SessionRequest {
    MemWrite(gennie_memory_retrieval::reconciliation::WriteCommand),
    MemResolve(gennie_memory_retrieval::reconciliation::ResolveCommand),
    MemProposalGet(gennie_memory_retrieval::reconciliation::InspectRequest),
    MemAudit(gennie_memory_retrieval::reconciliation::InspectRequest),
    MemOperationGet(gennie_memory_retrieval::reconciliation::InspectRequest),
    MemLifecycle(gennie_memory_retrieval::lifecycle::LifecycleRequest),
    MemReviewList(gennie_memory_retrieval::lifecycle::ReviewListRequest),
    EmbeddingJob(gennie_memory_retrieval::lifecycle::JobRequest),
    SourceImportHttps(crate::source_import::ImportRequest),
    SourceRegister(gennie_memory_retrieval::source_verification::RegisterRequest),
    SourceVerify(gennie_memory_retrieval::source_verification::VerifyRequest),
    RecallV1(gennie_memory_retrieval::product::ProductRequest),
    SegmentText(gennie_memory_retrieval::segmentation::SegmentRequest),
    ProviderCapabilities,
    Search(SearchRequest),
    EntityResolve(gennie_memory_retrieval::entity::EntityRequest),
    Recall(RecallRequest),
    HybridSearch(HybridRequest),
    ComposeSeed(SeedRequest),
}

#[derive(Serialize)]
struct SessionResult {
    status: &'static str,
    elapsed_ms: f64,
    response: Option<serde_json::Value>,
    error: Option<String>,
}

fn handle(conn: &mut Connection, classifier: &impl Classifier, line: &str) -> SessionResult {
    let started = Instant::now();
    let result = (|| {
        if line.len() > 1_000_000 {
            return Err("la solicitud supera 1 MB".to_string());
        }
        let request: SessionRequest = serde_json::from_str(line).map_err(|e| e.to_string())?;
        match request {
            SessionRequest::MemWrite(request) => {
                gennie_memory_retrieval::reconciliation::write(conn, &request, classifier)
            }
            SessionRequest::MemResolve(request) => {
                gennie_memory_retrieval::reconciliation::resolve(conn, &request)
            }
            SessionRequest::MemProposalGet(request) => {
                gennie_memory_retrieval::reconciliation::proposal_get(conn, &request)
            }
            SessionRequest::MemAudit(request) => {
                gennie_memory_retrieval::reconciliation::audit(conn, &request)
            }
            SessionRequest::MemOperationGet(request) => {
                gennie_memory_retrieval::reconciliation::operation_get(conn, &request)
            }
            SessionRequest::MemLifecycle(request) => {
                gennie_memory_retrieval::lifecycle::apply(conn, &request)
            }
            SessionRequest::MemReviewList(request) => {
                gennie_memory_retrieval::lifecycle::review_list(conn, &request)
            }
            SessionRequest::EmbeddingJob(request) => {
                gennie_memory_retrieval::lifecycle::job(conn, &request)
            }
            SessionRequest::SourceImportHttps(request) => {
                crate::source_import::import(conn, request)
            }
            SessionRequest::SourceRegister(request) => serde_json::to_value(
                gennie_memory_retrieval::source_verification::register(conn, &request)?,
            )
            .map_err(|e| e.to_string()),
            SessionRequest::SourceVerify(request) => serde_json::to_value(
                gennie_memory_retrieval::source_verification::verify(conn, &request)?,
            )
            .map_err(|e| e.to_string()),
            SessionRequest::RecallV1(request) => {
                let serialized = serde_json::to_string(&request).map_err(|e| e.to_string())?;
                let response =
                    gennie_memory_retrieval::product::execute_json(conn, &serialized, classifier);
                serde_json::to_value(response).map_err(|e| e.to_string())
            }
            SessionRequest::SegmentText(request) => {
                match gennie_memory_retrieval::segmentation::segment(&request.text) {
                    Ok(result) => Ok(
                        serde_json::json!({"status":"completed", "segmentation":result,"issue":null}),
                    ),
                    Err(code) => Ok(
                        serde_json::json!({"status":"failed", "segmentation":null,"issue":{"code":code}}),
                    ),
                }
            }
            SessionRequest::ProviderCapabilities => {
                Ok(gennie_memory_retrieval::product::capabilities())
            }
            SessionRequest::EntityResolve(request) => {
                serde_json::to_value(gennie_memory_retrieval::entity::resolve(conn, &request)?)
                    .map_err(|e| e.to_string())
            }
            SessionRequest::Search(request) => {
                let response = gennie_memory_retrieval::service::search(conn, &request)?;
                serde_json::to_value(response).map_err(|e| e.to_string())
            }
            SessionRequest::Recall(request) => {
                let response =
                    gennie_memory_retrieval::service::recall(conn, &request, classifier)?;
                serde_json::to_value(response).map_err(|e| e.to_string())
            }
            SessionRequest::HybridSearch(request) => {
                let response = gennie_memory_retrieval::embedding::search_hybrid(conn, &request)?;
                serde_json::to_value(response).map_err(|e| e.to_string())
            }
            SessionRequest::ComposeSeed(request) => {
                let response = gennie_memory_retrieval::service::compose_seed(conn, &request)?;
                serde_json::to_value(response).map_err(|e| e.to_string())
            }
        }
    })();
    match result {
        Ok(response) => SessionResult {
            status: "complete",
            elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
            response: Some(response),
            error: None,
        },
        Err(error) => SessionResult {
            status: "failed",
            elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
            response: None,
            error: Some(error),
        },
    }
}

pub fn run(conn: &mut Connection, classifier: &impl Classifier) -> Result<(), String> {
    let input = std::io::stdin().lock();
    let mut output = BufWriter::new(std::io::stdout().lock());
    for line in input.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        serde_json::to_writer(&mut output, &handle(conn, classifier, &line))
            .map_err(|e| e.to_string())?;
        output.write_all(b"\n").map_err(|e| e.to_string())?;
        output.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_reuses_search_and_revalidates_the_seed() {
        let classifier = crate::openrouter::OpenRouterClassifier::new(None).unwrap();
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        crate::registry::migrate(&mut conn).unwrap();
        let fixtures: Vec<gennie_memory_retrieval::model::FixtureMemory> =
            serde_json::from_str(include_str!("../../tests/fixtures/conditional-corpus.json"))
                .unwrap();
        gennie_memory_retrieval::repo::load_fixtures(&mut conn, &fixtures).unwrap();

        let search = serde_json::json!({
            "query": "agenda reuniones remotas Proyecto Sintético",
            "scope_kind": "project",
            "scope_id": "synthetic_meetings",
            "entity_id": "project_synthetic_meetings",
            "as_of": "2026-10-15T12:00:00.000Z",
            "limit": 20
        });
        let found = handle(
            &mut conn,
            &classifier,
            &serde_json::json!({"op":"search","request":search}).to_string(),
        );
        assert_eq!(found.status, "complete");
        let candidates = found.response.unwrap()["candidates"]
            .as_array()
            .unwrap()
            .clone();
        let remote = candidates
            .iter()
            .find(|candidate| candidate["memory_id"] == "synthetic_meeting_remote")
            .unwrap();
        let selected = serde_json::json!({
            "memory_id": remote["memory_id"], "revision": remote["revision"]
        });
        let request = serde_json::json!({
            "search": search, "hybrid": null,
            "presented": [selected], "selected": [selected],
            "classifier": "test_choice", "classification_status": "complete"
        });
        let composed = handle(
            &mut conn,
            &classifier,
            &serde_json::json!({"op":"compose_seed","request":request}).to_string(),
        );
        assert_eq!(composed.status, "complete");
        let seed = composed.response.unwrap();
        assert_eq!(
            seed["memories"][0]["conditions"],
            "Solo para reuniones remotas"
        );
        assert_eq!(
            seed["memories"][0]["source"]["ref"],
            "synthetic:meeting-guide@v1"
        );
        assert!(
            seed["seed_text"]
                .as_str()
                .unwrap()
                .contains("Solo para reuniones remotas")
        );

        let recall = handle(
            &mut conn,
            &classifier,
            &serde_json::json!({"op":"recall","request":{"search":search}}).to_string(),
        );
        assert_eq!(recall.status, "complete");
        assert_eq!(recall.response.unwrap()["error_code"], "missing_api_key");

        let invalid = handle(&mut conn, &classifier, "not json");
        assert_eq!(invalid.status, "failed");
    }
}

#[cfg(test)]
mod product_session_tests {
    use super::*;
    #[test]
    fn versioned_operations_preserve_transport_and_validate_duplicate_fields() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::registry::migrate(&mut conn).unwrap();
        let classifier = crate::openrouter::OpenRouterClassifier::new(None).unwrap();
        let request = serde_json::json!({"op":"recall_v1","request":{"contract_version":1,"search":{"query":"sin recuerdos","scope_kind":"project","scope_id":"test","as_of":"2026-10-23T12:00:00.000Z"}}});
        let result = handle(&mut conn, &classifier, &request.to_string());
        assert_eq!(result.status, "complete");
        assert_eq!(result.response.unwrap()["outcome"], "no_candidates");
        let result = handle(
            &mut conn,
            &classifier,
            r#"{"op":"segment_text","request":{"text":"No enviar. Si llueve."}}"#,
        );
        let result = result.response.unwrap();
        let reconstructed = result["segmentation"]["segments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["text"].as_str().unwrap())
            .collect::<String>();
        assert_eq!(reconstructed, "No enviar. Si llueve.");
        let duplicated = request.to_string().replace(
            "\"contract_version\":1",
            "\"contract_version\":1,\"contract_version\":2",
        );
        assert_eq!(handle(&mut conn, &classifier, &duplicated).status, "failed");
    }
}
