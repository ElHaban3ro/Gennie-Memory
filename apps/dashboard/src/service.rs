use crate::model::{
    ReviewCandidate, ReviewOutcome, ReviewRequest, ReviewView, SavedReview, Verdict, ViewMode,
};
use gennie_memory_retrieval::model::{
    Candidate, QueryMode, RecallRequest, RecallResponse, RecallTimings, SearchRequest, SeedRequest,
    SelectedCandidate,
};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::time::Instant;

fn key(search: &SearchRequest, view_mode: ViewMode) -> String {
    let normalized = search
        .query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let raw = serde_json::json!([
        search.scope_kind,
        search.scope_id,
        search.entity_id,
        normalized,
        match search.query_mode.unwrap_or(QueryMode::Keywords) {
            QueryMode::Keywords => "keywords",
            QueryMode::Natural => "natural",
        },
        search.limit.unwrap_or(20),
        match view_mode {
            ViewMode::Search => "search",
            ViewMode::Scope => "scope",
        }
    ]);
    Sha256::digest(raw.to_string().as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn candidates(
    conn: &Connection,
    search: &SearchRequest,
    view_mode: ViewMode,
) -> Result<Vec<Candidate>, String> {
    // Search validates all common fields before either view uses the same scope filters.
    let searched = gennie_memory_retrieval::service::search(conn, search)?;
    match view_mode {
        ViewMode::Search => Ok(searched.candidates),
        ViewMode::Scope => gennie_memory_retrieval::repo::list_scope(conn, search),
    }
}

fn signature(items: &[Candidate]) -> Vec<ReviewCandidate> {
    items
        .iter()
        .map(|item| ReviewCandidate {
            memory_id: item.memory_id.clone(),
            revision: item.revision,
        })
        .collect()
}

fn eligible(review: &SavedReview, available: &[Candidate]) -> bool {
    if review.outcome == ReviewOutcome::Missing {
        return false;
    }
    let selected: Vec<_> = review
        .decisions
        .iter()
        .filter(|item| item.verdict == Verdict::Match)
        .collect();
    selected.len() <= 8
        && selected.iter().all(|item| {
            available.iter().any(|candidate| {
                candidate.memory_id == item.memory_id && candidate.revision == item.revision
            })
        })
}

pub fn view(
    conn: &Connection,
    search: &SearchRequest,
    view_mode: ViewMode,
) -> Result<ReviewView, String> {
    let searched = gennie_memory_retrieval::service::search(conn, search)?.candidates;
    let current = match view_mode {
        ViewMode::Search => searched.clone(),
        ViewMode::Scope => gennie_memory_retrieval::repo::list_scope(conn, search)?,
    };
    let previous = crate::repo::latest(conn, &key(search, view_mode))?;
    let matching = previous.filter(|row| signature(&row.candidates) == signature(&current));
    let reusable = matching.as_ref().is_some_and(|row| {
        let available = if view_mode == ViewMode::Search {
            &current
        } else {
            &searched
        };
        eligible(&row.saved, available)
    });
    Ok(ReviewView {
        candidates: current,
        review: matching.map(|row| row.saved),
        reusable,
    })
}

pub fn save(conn: &mut Connection, request: ReviewRequest) -> Result<SavedReview, String> {
    if request
        .case_id
        .as_ref()
        .is_some_and(|id| id.is_empty() || id.len() > 128)
        || request
            .note
            .as_ref()
            .is_some_and(|note| note.chars().count() > 2000)
    {
        return Err("ID o nota demasiado largos".into());
    }
    let current = candidates(conn, &request.search, request.view_mode)?;
    let presented = signature(&current);
    if request.candidates != presented {
        return Err("los candidatos cambiaron; vuelve a cargar la pregunta".into());
    }
    let mut seen = HashSet::new();
    for decision in &request.decisions {
        if !seen.insert(&decision.memory_id)
            || !presented.iter().any(|item| {
                item.memory_id == decision.memory_id && item.revision == decision.revision
            })
        {
            return Err("la decisión contiene un recuerdo duplicado u obsoleto".into());
        }
    }
    let positive = request
        .decisions
        .iter()
        .filter(|item| item.verdict == Verdict::Match)
        .count();
    match request.outcome {
        ReviewOutcome::Matches if positive == 0 => {
            return Err("marca al menos un recuerdo que coincide".into());
        }
        ReviewOutcome::None | ReviewOutcome::Missing if positive != 0 => {
            return Err("una respuesta ausente no puede incluir coincidencias".into());
        }
        _ => {}
    }
    let key = key(&request.search, request.view_mode);
    crate::repo::insert(
        conn,
        request.case_id.as_deref(),
        &key,
        &request.search,
        match request.view_mode {
            ViewMode::Search => "search",
            ViewMode::Scope => "scope",
        },
        &current,
        &request.decisions,
        request.outcome,
        request.note.as_deref(),
    )
}

pub fn reviewed_recall(
    conn: &Connection,
    request: &RecallRequest,
) -> Result<Option<RecallResponse>, String> {
    let started = Instant::now();
    let mut search = request.search.clone();
    if search.limit.is_none() {
        search.limit = Some(20);
    }
    let search_review = crate::repo::latest(conn, &key(&search, ViewMode::Search))?;
    let scope_review = crate::repo::latest(conn, &key(&search, ViewMode::Scope))?;
    if search_review.is_none() && scope_review.is_none() {
        return Ok(None);
    }
    let retrieval = gennie_memory_retrieval::service::search(conn, &search)?;
    let search_ms = started.elapsed().as_secs_f64() * 1000.0;
    let search_review = search_review.filter(|row| {
        signature(&row.candidates) == signature(&retrieval.candidates)
            && eligible(&row.saved, &retrieval.candidates)
    });
    let previous = if let Some(review) = search_review {
        review
    } else {
        let Some(review) = scope_review else {
            return Ok(None);
        };
        let scope_now = gennie_memory_retrieval::repo::list_scope(conn, &search)?;
        if signature(&review.candidates) != signature(&scope_now)
            || !eligible(&review.saved, &retrieval.candidates)
        {
            return Ok(None);
        }
        review
    };
    let selected: Vec<SelectedCandidate> = previous
        .saved
        .decisions
        .iter()
        .filter(|item| item.verdict == Verdict::Match)
        .map(|item| SelectedCandidate {
            memory_id: item.memory_id.clone(),
            revision: item.revision,
        })
        .collect();
    if selected.len() > 8 {
        return Ok(None);
    }
    let presented = signature(&retrieval.candidates)
        .into_iter()
        .map(|item| SelectedCandidate {
            memory_id: item.memory_id,
            revision: item.revision,
        })
        .collect();
    let composition_started = Instant::now();
    let seed = gennie_memory_retrieval::service::compose_seed(
        conn,
        &SeedRequest {
            search: Some(search),
            hybrid: None,
            presented,
            selected,
            classifier: "local_review".into(),
            classification_status: "complete".into(),
        },
    )?;
    let composition_ms = composition_started.elapsed().as_secs_f64() * 1000.0;
    let selected_id = if seed.memories.len() == 1 {
        Some(seed.memories[0].memory_id.clone())
    } else {
        None
    };
    Ok(Some(RecallResponse {
        selected_ids: seed.memories.iter().map(|m| m.memory_id.clone()).collect(),
        per_memory: None,
        consensus: None,
        applicability: None,
        slot_evidence: None,
        status: seed.status,
        retrieval,
        classification_status: "review_reused",
        requested_model: request.provider.model_id(),
        served_model: Some("local_review".into()),
        selected_id,
        seed,
        input_tokens: None,
        output_tokens: None,
        reported_cost_usd: None,
        error_code: None,
        provider_http_status: None,
        timings: RecallTimings {
            search_ms,
            classifier_ms: 0.0,
            composition_ms,
            total_ms: started.elapsed().as_secs_f64() * 1000.0,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{CandidateDecision, ReviewRequest},
        module::DashboardModule,
    };
    use gennie_memory_kernel::core::AppModule;
    use gennie_memory_retrieval::{model::FixtureMemory, module::RetrievalModule};

    fn setup() -> (Connection, SearchRequest, Vec<FixtureMemory>) {
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        let modules: Vec<Box<dyn AppModule>> =
            vec![Box::new(RetrievalModule), Box::new(DashboardModule)];
        gennie_memory_kernel::db::migrate(&mut conn, &modules).unwrap();
        let fixtures: Vec<FixtureMemory> =
            serde_json::from_str(include_str!("../../../tests/fixtures/small-corpus.json"))
                .unwrap();
        gennie_memory_retrieval::repo::load_fixtures(&mut conn, &fixtures).unwrap();
        let recall: serde_json::Value =
            serde_json::from_str(include_str!("../../../tests/fixtures/small-recall.json"))
                .unwrap();
        let search: SearchRequest = serde_json::from_value(recall["search"].clone()).unwrap();
        (conn, search, fixtures)
    }

    #[test]
    fn human_review_is_reused_then_invalidated_by_memory_revision() {
        let (mut conn, search, fixtures) = setup();
        let view = view(&conn, &search, ViewMode::Scope).unwrap();
        assert_eq!(view.candidates.len(), 1);
        let candidate = &view.candidates[0];
        let identity = ReviewCandidate {
            memory_id: candidate.memory_id.clone(),
            revision: candidate.revision,
        };
        let saved = save(
            &mut conn,
            ReviewRequest {
                case_id: Some("case_1".into()),
                search: search.clone(),
                view_mode: ViewMode::Scope,
                candidates: vec![identity.clone()],
                decisions: vec![CandidateDecision {
                    memory_id: identity.memory_id.clone(),
                    revision: identity.revision,
                    verdict: Verdict::Match,
                }],
                outcome: ReviewOutcome::Matches,
                note: None,
            },
        )
        .unwrap();
        assert_eq!(saved.id, 1);
        let recall: RecallRequest =
            serde_json::from_value(serde_json::json!({"search": search})).unwrap();
        let reused = reviewed_recall(&conn, &recall).unwrap().unwrap();
        assert_eq!(reused.classification_status, "review_reused");
        assert_eq!(reused.seed.memories[0].memory_id, identity.memory_id);
        assert_eq!(reused.reported_cost_usd, None);
        let mut newer = fixtures[0].clone();
        newer.revision = 2;
        newer.summary = "Marina ahora prefiere informes detallados los lunes.".into();
        newer.original_text = newer.summary.clone();
        gennie_memory_retrieval::repo::load_fixtures(&mut conn, &[newer]).unwrap();
        assert!(reviewed_recall(&conn, &recall).unwrap().is_none());
        assert!(
            save(
                &mut conn,
                ReviewRequest {
                    case_id: None,
                    search,
                    view_mode: ViewMode::Scope,
                    candidates: vec![identity],
                    decisions: vec![],
                    outcome: ReviewOutcome::None,
                    note: None,
                }
            )
            .unwrap_err()
            .contains("cambiaron")
        );
    }

    #[test]
    fn missing_memory_is_recorded_but_never_silently_reused() {
        let (mut conn, search, _) = setup();
        let view = view(&conn, &search, ViewMode::Search).unwrap();
        save(
            &mut conn,
            ReviewRequest {
                case_id: None,
                search: search.clone(),
                view_mode: ViewMode::Search,
                candidates: signature(&view.candidates),
                decisions: vec![],
                outcome: ReviewOutcome::Missing,
                note: Some("Falta la decisión reciente".into()),
            },
        )
        .unwrap();
        let recall: RecallRequest =
            serde_json::from_value(serde_json::json!({"search": search})).unwrap();
        assert!(reviewed_recall(&conn, &recall).unwrap().is_none());
    }
}
