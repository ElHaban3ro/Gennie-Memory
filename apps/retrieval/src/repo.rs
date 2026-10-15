use crate::model::{Candidate, FixtureMemory, SearchRequest, SourceRef};
use rusqlite::{Connection, params};

pub fn load_fixtures(conn: &mut Connection, fixtures: &[FixtureMemory]) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    for fixture in fixtures {
        if fixture.id.is_empty()
            || fixture.scope_kind.is_empty()
            || fixture.scope_id.is_empty()
            || fixture.revision < 1
            || fixture
                .entity_id
                .as_ref()
                .is_some_and(|id| id.trim().is_empty() || id.len() > 128)
        {
            return Err("fixture: identidad, ámbito o revisión inválida".into());
        }
        if fixture.original_text.trim().is_empty()
            || fixture.original_text.chars().count() > 16000
            || fixture.summary.trim().is_empty()
            || fixture.summary.chars().count() > 1600
            || fixture
                .conditions
                .as_ref()
                .is_some_and(|value| value.trim().is_empty() || value.chars().count() > 1600)
        {
            return Err(format!(
                "fixture {} contiene texto o condiciones inválidos",
                fixture.id
            ));
        }
        if let Some(source) = &fixture.source {
            if !matches!(
                source.kind.as_str(),
                "user" | "document" | "tool" | "agent_discovery"
            ) || source.reference.trim().is_empty()
                || source.reference.chars().count() > 512
                || source.reference.chars().any(char::is_control)
                || source.locator.as_ref().is_some_and(|value| {
                    value.trim().is_empty()
                        || value.chars().count() > 256
                        || value.chars().any(char::is_control)
                })
            {
                return Err(format!(
                    "fixture {} contiene una fuente inválida",
                    fixture.id
                ));
            }
        }
        for date in [&fixture.valid_from, &fixture.valid_until]
            .into_iter()
            .flatten()
        {
            if !crate::service::valid_utc(date) {
                return Err(format!(
                    "fixture {} contiene una fecha UTC inválida",
                    fixture.id
                ));
            }
        }
        if let (Some(from), Some(until)) = (&fixture.valid_from, &fixture.valid_until) {
            if until <= from {
                return Err(format!(
                    "fixture {} tiene un intervalo inválido",
                    fixture.id
                ));
            }
        }
        let affected = tx.execute(
            "INSERT INTO memories(id, scope_kind, scope_id, current_revision, entity_id) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET current_revision = max(current_revision, excluded.current_revision)
             WHERE scope_kind = excluded.scope_kind AND scope_id = excluded.scope_id
               AND entity_id IS excluded.entity_id",
            params![fixture.id, fixture.scope_kind, fixture.scope_id, fixture.revision, fixture.entity_id],
        )
        .map_err(|e| e.to_string())?;
        if affected != 1 {
            return Err(format!("fixture {} cambia de ámbito o entidad", fixture.id));
        }
        tx.execute(
            "INSERT INTO memory_revisions(memory_id, revision, memory_type, original_text, summary, state, valid_from, valid_until, conditions, source_kind, source_ref, source_locator)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![fixture.id, fixture.revision, fixture.memory_type, fixture.original_text, fixture.summary, fixture.state, fixture.valid_from, fixture.valid_until,
                    fixture.conditions, fixture.source.as_ref().map(|source| &source.kind),
                    fixture.source.as_ref().map(|source| &source.reference),
                    fixture.source.as_ref().and_then(|source| source.locator.as_ref())],
        )
        .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO search_documents(memory_id, revision, original_text, summary) VALUES (?1, ?2, ?3, ?4)",
            params![fixture.id, fixture.revision, fixture.original_text, fixture.summary],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}

pub fn search(
    conn: &Connection,
    request: &SearchRequest,
    expression: &str,
) -> Result<Vec<Candidate>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT m.id, r.revision, r.summary, r.state, r.conditions,
                    r.source_kind, r.source_ref, r.source_locator,
                    bm25(memory_fts, 1.0, 1.5) AS rank
         FROM memory_fts
         JOIN search_documents d ON d.doc_id = memory_fts.rowid
         JOIN memories m ON m.id = d.memory_id
         JOIN memory_revisions r ON r.memory_id = d.memory_id AND r.revision = d.revision
         WHERE memory_fts MATCH ?1 AND m.scope_kind = ?2 AND m.scope_id = ?3
           AND d.revision = m.current_revision AND r.state = 'active'
           AND (r.valid_from IS NULL OR r.valid_from <= ?4)
           AND (r.valid_until IS NULL OR r.valid_until > ?4)
           AND (?5 IS NULL OR m.entity_id = ?5)
         ORDER BY rank ASC, m.id ASC LIMIT ?6",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            params![
                expression,
                request.scope_kind,
                request.scope_id,
                request.as_of,
                request.entity_id,
                request.limit.unwrap_or(8)
            ],
            |row| {
                let source_kind: Option<String> = row.get(5)?;
                let source_ref: Option<String> = row.get(6)?;
                let source_locator: Option<String> = row.get(7)?;
                let source = match (source_kind, source_ref) {
                    (Some(kind), Some(reference)) => Some(SourceRef {
                        kind,
                        reference,
                        locator: source_locator,
                    }),
                    (None, None) if source_locator.is_none() => None,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                Ok(Candidate {
                    memory_id: row.get(0)?,
                    revision: row.get(1)?,
                    summary: row.get(2)?,
                    state: row.get(3)?,
                    conditions: row.get(4)?,
                    source,
                    bm25: row.get(8)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
