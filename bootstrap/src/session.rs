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
    Search(SearchRequest),
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

fn handle(conn: &Connection, classifier: &impl Classifier, line: &str) -> SessionResult {
    let started = Instant::now();
    let result = (|| {
        if line.len() > 1_000_000 {
            return Err("la solicitud supera 1 MB".to_string());
        }
        let request: SessionRequest = serde_json::from_str(line).map_err(|e| e.to_string())?;
        match request {
            SessionRequest::Search(request) => {
                let response = gennie_memory_retrieval::service::search(conn, &request)?;
                serde_json::to_value(response).map_err(|e| e.to_string())
            }
            SessionRequest::Recall(request) => {
                let response = if let Some(reviewed) =
                    gennie_memory_dashboard::service::reviewed_recall(conn, &request)?
                {
                    reviewed
                } else {
                    gennie_memory_retrieval::service::recall(conn, &request, classifier)?
                };
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

pub fn run(conn: &Connection, classifier: &impl Classifier) -> Result<(), String> {
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
        gennie_memory_kernel::db::migrate(&mut conn, &crate::registry::modules().unwrap()).unwrap();
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
            &conn,
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
            &conn,
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
            &conn,
            &classifier,
            &serde_json::json!({"op":"recall","request":{"search":search}}).to_string(),
        );
        assert_eq!(recall.status, "complete");
        assert_eq!(recall.response.unwrap()["error_code"], "missing_api_key");

        let review_search: SearchRequest = serde_json::from_value(search.clone()).unwrap();
        let review_view = gennie_memory_dashboard::service::view(
            &conn,
            &review_search,
            gennie_memory_dashboard::model::ViewMode::Search,
        )
        .unwrap();
        let review_request: gennie_memory_dashboard::model::ReviewRequest = serde_json::from_value(
            serde_json::json!({
                "case_id": null, "search": search, "view_mode": "search",
                "candidates": review_view.candidates.iter().map(|item| serde_json::json!({"memory_id": item.memory_id, "revision": item.revision})).collect::<Vec<_>>(),
                "decisions": [{"memory_id": "synthetic_meeting_remote", "revision": 1, "verdict": "match"}],
                "outcome": "matches", "note": null
            }),
        ).unwrap();
        gennie_memory_dashboard::service::save(&mut conn, review_request).unwrap();
        let reviewed = handle(
            &conn,
            &classifier,
            &serde_json::json!({"op":"recall","request":{"search":review_search}}).to_string(),
        );
        assert_eq!(reviewed.status, "complete");
        assert_eq!(
            reviewed.response.unwrap()["classification_status"],
            "review_reused"
        );

        let invalid = handle(&conn, &classifier, "not json");
        assert_eq!(invalid.status, "failed");
    }
}
