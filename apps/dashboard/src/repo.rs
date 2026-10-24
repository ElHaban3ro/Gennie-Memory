use crate::model::{CandidateDecision, ReviewOutcome, SavedReview};
use gennie_memory_retrieval::model::Candidate;
use rusqlite::{Connection, OptionalExtension, params};

pub struct StoredReview {
    pub saved: SavedReview,
    pub candidates: Vec<Candidate>,
}

pub fn latest(conn: &Connection, query_key: &str) -> Result<Option<StoredReview>, String> {
    conn.query_row(
        "SELECT id, case_id, candidates_json, decisions_json, outcome, note, created_at
         FROM dashboard_reviews WHERE query_key = ?1 ORDER BY id DESC LIMIT 1",
        [query_key],
        |row| {
            let candidates: String = row.get(2)?;
            let decisions: String = row.get(3)?;
            let outcome: String = row.get(4)?;
            Ok((
                row.get(0)?,
                row.get(1)?,
                candidates,
                decisions,
                outcome,
                row.get(5)?,
                row.get(6)?,
            ))
        },
    )
    .optional()
    .map_err(|e| e.to_string())?
    .map(
        |(id, case_id, candidates, decisions, outcome, note, created_at): (
            i64,
            Option<String>,
            String,
            String,
            String,
            Option<String>,
            String,
        )| {
            Ok(StoredReview {
                candidates: serde_json::from_str(&candidates).map_err(|e| e.to_string())?,
                saved: SavedReview {
                    id,
                    case_id,
                    decisions: serde_json::from_str::<Vec<CandidateDecision>>(&decisions)
                        .map_err(|e| e.to_string())?,
                    outcome: serde_json::from_str::<ReviewOutcome>(&format!("\"{outcome}\""))
                        .map_err(|e| e.to_string())?,
                    note,
                    created_at,
                },
            })
        },
    )
    .transpose()
}

pub fn insert(
    conn: &mut Connection,
    case_id: Option<&str>,
    query_key: &str,
    search: &gennie_memory_retrieval::model::SearchRequest,
    view_mode: &str,
    candidates: &[Candidate],
    decisions: &[CandidateDecision],
    outcome: ReviewOutcome,
    note: Option<&str>,
) -> Result<SavedReview, String> {
    let candidates_json = serde_json::to_string(candidates).map_err(|e| e.to_string())?;
    let decisions_json = serde_json::to_string(decisions).map_err(|e| e.to_string())?;
    let outcome_text = match outcome {
        ReviewOutcome::Matches => "matches",
        ReviewOutcome::None => "none",
        ReviewOutcome::Missing => "missing",
    };
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO dashboard_reviews (case_id, query_key, query, scope_kind, scope_id, entity_id, as_of, query_mode, candidates_json, decisions_json, outcome, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
        params![case_id, query_key, search.query, search.scope_kind, search.scope_id, search.entity_id,
            search.as_of, view_mode, candidates_json, decisions_json, outcome_text, note],
    ).map_err(|e| e.to_string())?;
    let id = tx.last_insert_rowid();
    let created_at: String = tx
        .query_row(
            "SELECT created_at FROM dashboard_reviews WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(SavedReview {
        id,
        case_id: case_id.map(str::to_owned),
        decisions: decisions.to_vec(),
        outcome,
        note: note.map(str::to_owned),
        created_at,
    })
}

pub fn reviewed_case_ids(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT case_id FROM dashboard_reviews WHERE case_id IS NOT NULL")
        .map_err(|e| e.to_string())?;
    stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())
}

pub fn export_rows(conn: &Connection) -> Result<Vec<serde_json::Value>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, case_id, query, scope_kind, scope_id, entity_id, as_of, query_mode,
                candidates_json, decisions_json, outcome, note, created_at
         FROM dashboard_reviews ORDER BY id DESC LIMIT 500",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?, "case_id": row.get::<_, Option<String>>(1)?,
            "query": row.get::<_, String>(2)?, "scope_kind": row.get::<_, String>(3)?,
            "scope_id": row.get::<_, String>(4)?, "entity_id": row.get::<_, Option<String>>(5)?,
            "as_of": row.get::<_, String>(6)?, "view_mode": row.get::<_, String>(7)?,
            "candidates": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(8)?).unwrap_or_default(),
            "decisions": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(9)?).unwrap_or_default(),
            "outcome": row.get::<_, String>(10)?, "note": row.get::<_, Option<String>>(11)?,
            "created_at": row.get::<_, String>(12)?
        }))
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}
