use crate::model::{EmbeddingBatch, HybridCandidate, HybridRequest, HybridResponse, SourceRef};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::{HashMap, HashSet};

const RRF_K: f64 = 60.0;
const RANK_LIMIT: usize = 20;

fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn norm(values: &[f32], dimensions: usize) -> Result<f64, String> {
    if values.len() != dimensions || values.iter().any(|value| !value.is_finite()) {
        return Err("vector con dimensión o valores inválidos".into());
    }
    let norm = values
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || norm <= 0.0 {
        return Err("vector de norma cero o inválida".into());
    }
    Ok(norm)
}

pub fn load_batch(conn: &mut Connection, batch: &EmbeddingBatch) -> Result<usize, String> {
    if batch.space_id.trim().is_empty()
        || batch.model_id.trim().is_empty()
        || batch.procedure.trim().is_empty()
        || !valid_hash(&batch.model_sha256)
        || !valid_hash(&batch.tokenizer_sha256)
        || !(1..=4096).contains(&batch.dimensions)
        || batch.vectors.is_empty()
    {
        return Err("perfil de embeddings inválido".into());
    }
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT OR IGNORE INTO embedding_spaces
         (space_id, model_id, model_sha256, tokenizer_sha256, procedure, dimensions, vector_format)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'json-f32-v1')",
        params![
            batch.space_id,
            batch.model_id,
            batch.model_sha256,
            batch.tokenizer_sha256,
            batch.procedure,
            batch.dimensions as i64
        ],
    )
    .map_err(|e| e.to_string())?;
    let actual: (String, String, String, String, i64) = tx
        .query_row(
            "SELECT model_id, model_sha256, tokenizer_sha256, procedure, dimensions
             FROM embedding_spaces WHERE space_id = ?1",
            [&batch.space_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?;
    if actual
        != (
            batch.model_id.clone(),
            batch.model_sha256.clone(),
            batch.tokenizer_sha256.clone(),
            batch.procedure.clone(),
            batch.dimensions as i64,
        )
    {
        return Err("space_id ya existe con otro perfil de embeddings".into());
    }
    let mut seen = HashSet::new();
    for item in &batch.vectors {
        if !seen.insert((&item.memory_id, item.revision)) {
            return Err("vector duplicado en el lote".into());
        }
        let vector_norm = norm(&item.values, batch.dimensions)?;
        let current: Option<(i64, String)> = tx
            .query_row(
                "SELECT m.current_revision, r.state FROM memories m
                 JOIN memory_revisions r ON r.memory_id = m.id AND r.revision = ?2
                 WHERE m.id = ?1",
                params![item.memory_id, item.revision],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if !matches!(current, Some((revision, ref state)) if revision == item.revision && state == "active")
        {
            return Err(format!(
                "vector {}@{} no corresponde a una revisión activa y vigente",
                item.memory_id, item.revision
            ));
        }
        let vector_json = serde_json::to_string(&item.values).map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO memory_embeddings(memory_id, revision, space_id, vector_json, norm)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                item.memory_id,
                item.revision,
                batch.space_id,
                vector_json,
                vector_norm
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(batch.vectors.len())
}

struct SemanticMatch {
    memory_id: String,
    revision: i64,
    summary: String,
    conditions: Option<String>,
    source: Option<SourceRef>,
    state: String,
    cosine: f64,
}

pub fn search_hybrid(conn: &Connection, request: &HybridRequest) -> Result<HybridResponse, String> {
    // Reuse the same input validation as the textual service, but collect a wider
    // candidate pool before RRF and final output limiting.
    let text_request = crate::model::SearchRequest {
        query: request.search.query.clone(),
        scope_kind: request.search.scope_kind.clone(),
        scope_id: request.search.scope_id.clone(),
        entity_id: request.search.entity_id.clone(),
        as_of: request.search.as_of.clone(),
        limit: Some(RANK_LIMIT as u32),
        query_mode: Some(
            request
                .search
                .query_mode
                .unwrap_or(crate::model::QueryMode::Natural),
        ),
    };
    let text = crate::service::search(conn, &text_request)?;
    if !(1..=RANK_LIMIT as u32).contains(&request.search.limit.unwrap_or(8)) {
        return Err("limit debe estar entre 1 y 20".into());
    }
    let dimensions: i64 = conn
        .query_row(
            "SELECT dimensions FROM embedding_spaces WHERE space_id = ?1",
            [&request.space_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or("space_id desconocido")?;
    let query_norm = norm(&request.query_vector, dimensions as usize)?;
    let mut stmt = conn
        .prepare(
            "SELECT m.id, r.revision, r.summary, r.state, r.conditions,
                    r.source_kind, r.source_ref, r.source_locator, e.vector_json, e.norm
             FROM memories m
             JOIN memory_revisions r ON r.memory_id = m.id AND r.revision = m.current_revision
             LEFT JOIN memory_embeddings e ON e.memory_id = m.id AND e.revision = r.revision AND e.space_id = ?1
             WHERE m.scope_kind = ?2 AND m.scope_id = ?3 AND r.state = 'active'
               AND (r.valid_from IS NULL OR r.valid_from <= ?4)
               AND (r.valid_until IS NULL OR r.valid_until > ?4)
               AND (?5 IS NULL OR m.entity_id = ?5)",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            params![
                request.space_id,
                request.search.scope_kind,
                request.search.scope_id,
                request.search.as_of,
                request.search.entity_id
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
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    source,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<f64>>(9)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?;
    let mut eligible = 0;
    let mut indexed = 0;
    let mut semantic = Vec::new();
    for row in rows {
        let (memory_id, revision, summary, state, conditions, source, vector_json, stored_norm) =
            row.map_err(|e| e.to_string())?;
        eligible += 1;
        if let (Some(vector_json), Some(stored_norm)) = (vector_json, stored_norm) {
            let vector: Vec<f32> = serde_json::from_str(&vector_json).map_err(|e| e.to_string())?;
            let checked_norm = norm(&vector, dimensions as usize)?;
            if !stored_norm.is_finite() || (stored_norm - checked_norm).abs() > 1e-5 {
                return Err("norma de vector almacenada inválida".into());
            }
            let dot = vector
                .iter()
                .zip(&request.query_vector)
                .map(|(left, right)| f64::from(*left) * f64::from(*right))
                .sum::<f64>();
            semantic.push(SemanticMatch {
                memory_id,
                revision,
                summary,
                conditions,
                source,
                state,
                cosine: (dot / (checked_norm * query_norm)).clamp(-1.0, 1.0),
            });
            indexed += 1;
        }
    }
    semantic.sort_by(|left, right| {
        right
            .cosine
            .total_cmp(&left.cosine)
            .then_with(|| left.memory_id.cmp(&right.memory_id))
    });
    let mut fused: HashMap<(String, i64), HybridCandidate> = HashMap::new();
    for (position, candidate) in text.candidates.iter().enumerate() {
        let key = (candidate.memory_id.clone(), candidate.revision);
        fused.insert(
            key,
            HybridCandidate {
                memory_id: candidate.memory_id.clone(),
                revision: candidate.revision,
                summary: candidate.summary.clone(),
                conditions: candidate.conditions.clone(),
                source: candidate.source.clone(),
                state: candidate.state.clone(),
                text_rank: Some(position + 1),
                semantic_rank: None,
                cosine: None,
                rrf: 1.0 / (RRF_K + (position + 1) as f64),
            },
        );
    }
    for (position, candidate) in semantic.iter().take(RANK_LIMIT).enumerate() {
        let key = (candidate.memory_id.clone(), candidate.revision);
        let entry = fused.entry(key).or_insert_with(|| HybridCandidate {
            memory_id: candidate.memory_id.clone(),
            revision: candidate.revision,
            summary: candidate.summary.clone(),
            conditions: candidate.conditions.clone(),
            source: candidate.source.clone(),
            state: candidate.state.clone(),
            text_rank: None,
            semantic_rank: None,
            cosine: None,
            rrf: 0.0,
        });
        entry.semantic_rank = Some(position + 1);
        entry.cosine = Some(candidate.cosine);
        entry.rrf += 1.0 / (RRF_K + (position + 1) as f64);
    }
    let mut candidates: Vec<_> = fused.into_values().collect();
    candidates.sort_by(|left, right| {
        right
            .rrf
            .total_cmp(&left.rrf)
            .then_with(|| {
                left.text_rank
                    .unwrap_or(usize::MAX)
                    .cmp(&right.text_rank.unwrap_or(usize::MAX))
            })
            .then_with(|| {
                left.semantic_rank
                    .unwrap_or(usize::MAX)
                    .cmp(&right.semantic_rank.unwrap_or(usize::MAX))
            })
            .then_with(|| left.memory_id.cmp(&right.memory_id))
    });
    candidates.truncate(request.search.limit.unwrap_or(8) as usize);
    let coverage = if indexed == eligible {
        "complete"
    } else if indexed == 0 {
        "missing"
    } else {
        "partial"
    };
    Ok(HybridResponse {
        status: if coverage == "complete" {
            "complete"
        } else {
            "degraded"
        },
        method: "fts5_semantic_rrf",
        semantic_coverage: coverage,
        eligible_memories: eligible,
        indexed_memories: indexed,
        candidates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FixtureMemory, FixtureVector, SearchRequest, SourceRef};

    fn memory(id: &str, scope: &str, summary: &str) -> FixtureMemory {
        FixtureMemory {
            id: id.into(),
            scope_kind: "project".into(),
            scope_id: scope.into(),
            entity_id: None,
            revision: 1,
            memory_type: "fact".into(),
            original_text: summary.into(),
            summary: summary.into(),
            conditions: None,
            source: None,
            state: "active".into(),
            valid_from: None,
            valid_until: None,
        }
    }

    fn batch(vectors: Vec<FixtureVector>) -> EmbeddingBatch {
        EmbeddingBatch {
            space_id: "synthetic-space-v1".into(),
            model_id: "synthetic-model".into(),
            model_sha256: "a".repeat(64),
            tokenizer_sha256: "b".repeat(64),
            procedure: "test-vector-v1".into(),
            dimensions: 2,
            vectors,
        }
    }

    #[test]
    fn semantic_search_respects_scope_and_reports_partial_coverage() {
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0002_embeddings.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        let mut wanted = memory("wanted", "a", "Recuerdo del faro");
        wanted.conditions = Some("Solo al navegar de noche".into());
        wanted.source = Some(SourceRef {
            kind: "document".into(),
            reference: "synthetic:logbook".into(),
            locator: None,
        });
        crate::repo::load_fixtures(
            &mut conn,
            &[
                wanted,
                memory("other", "a", "Recuerdo de la duna"),
                memory("foreign", "b", "Recuerdo del faro"),
            ],
        )
        .unwrap();
        load_batch(
            &mut conn,
            &batch(vec![FixtureVector {
                memory_id: "wanted".into(),
                revision: 1,
                values: vec![1.0, 0.0],
            }]),
        )
        .unwrap();
        let request = HybridRequest {
            search: SearchRequest {
                query: "una paráfrasis sin coincidencia".into(),
                scope_kind: "project".into(),
                scope_id: "a".into(),
                entity_id: None,
                as_of: "2026-10-15T12:00:00.000Z".into(),
                limit: Some(8),
                query_mode: None,
            },
            space_id: "synthetic-space-v1".into(),
            query_vector: vec![1.0, 0.0],
        };
        let result = search_hybrid(&conn, &request).unwrap();
        assert_eq!(result.status, "degraded");
        assert_eq!(result.semantic_coverage, "partial");
        assert_eq!((result.eligible_memories, result.indexed_memories), (2, 1));
        assert_eq!(result.candidates[0].memory_id, "wanted");
        assert_eq!(
            result.candidates[0].conditions.as_deref(),
            Some("Solo al navegar de noche")
        );
        assert_eq!(
            result.candidates[0]
                .source
                .as_ref()
                .map(|value| value.reference.as_str()),
            Some("synthetic:logbook")
        );
        assert!(
            !result
                .candidates
                .iter()
                .any(|item| item.memory_id == "foreign")
        );
        assert!(
            load_batch(
                &mut conn,
                &batch(vec![FixtureVector {
                    memory_id: "foreign".into(),
                    revision: 1,
                    values: vec![f32::NAN, 0.0],
                }]),
            )
            .is_err()
        );
        let mut revised = memory("wanted", "a", "Recuerdo revisado");
        revised.revision = 2;
        crate::repo::load_fixtures(&mut conn, &[revised]).unwrap();
        let result = search_hybrid(&conn, &request).unwrap();
        assert_eq!(result.semantic_coverage, "missing");
        assert_eq!(result.indexed_memories, 0);
        assert!(
            load_batch(
                &mut conn,
                &batch(vec![FixtureVector {
                    memory_id: "wanted".into(),
                    revision: 1,
                    values: vec![1.0, 0.0],
                }]),
            )
            .unwrap_err()
            .contains("revisión activa")
        );
    }

    #[test]
    fn rrf_tie_prefers_text_rank_over_identifier_order() {
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0002_embeddings.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        crate::repo::load_fixtures(
            &mut conn,
            &[
                memory("z_wanted", "a", "alpha beacon"),
                memory("a_other", "a", "alpha"),
            ],
        )
        .unwrap();
        load_batch(
            &mut conn,
            &batch(vec![
                FixtureVector {
                    memory_id: "z_wanted".into(),
                    revision: 1,
                    values: vec![0.0, 1.0],
                },
                FixtureVector {
                    memory_id: "a_other".into(),
                    revision: 1,
                    values: vec![1.0, 0.0],
                },
            ]),
        )
        .unwrap();
        let result = search_hybrid(
            &conn,
            &HybridRequest {
                search: SearchRequest {
                    query: "alpha beacon".into(),
                    scope_kind: "project".into(),
                    scope_id: "a".into(),
                    entity_id: None,
                    as_of: "2026-10-15T12:00:00.000Z".into(),
                    limit: Some(2),
                    query_mode: Some(crate::model::QueryMode::Natural),
                },
                space_id: "synthetic-space-v1".into(),
                query_vector: vec![1.0, 0.0],
            },
        )
        .unwrap();
        assert_eq!(result.candidates[0].memory_id, "z_wanted");
        assert_eq!(result.candidates[0].text_rank, Some(1));
        assert_eq!(result.candidates[0].semantic_rank, Some(2));
    }

    #[test]
    fn entity_filter_precedes_text_and_semantic_ranking() {
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0002_embeddings.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        let mut wanted = memory("wanted", "shared", "alpha instruction");
        wanted.entity_id = Some("project_one".into());
        let mut distractor = memory("distractor", "shared", "alpha instruction");
        distractor.entity_id = Some("project_two".into());
        crate::repo::load_fixtures(&mut conn, &[wanted.clone(), distractor]).unwrap();
        load_batch(
            &mut conn,
            &batch(vec![
                FixtureVector {
                    memory_id: "wanted".into(),
                    revision: 1,
                    values: vec![0.0, 1.0],
                },
                FixtureVector {
                    memory_id: "distractor".into(),
                    revision: 1,
                    values: vec![1.0, 0.0],
                },
            ]),
        )
        .unwrap();
        let search = SearchRequest {
            query: "alpha instruction".into(),
            scope_kind: "project".into(),
            scope_id: "shared".into(),
            entity_id: Some("project_one".into()),
            as_of: "2026-10-15T12:00:00.000Z".into(),
            limit: Some(8),
            query_mode: Some(crate::model::QueryMode::Natural),
        };
        let textual = crate::service::search(&conn, &search).unwrap();
        assert_eq!(textual.candidates.len(), 1);
        assert_eq!(textual.candidates[0].memory_id, "wanted");
        let hybrid = search_hybrid(
            &conn,
            &HybridRequest {
                search,
                space_id: "synthetic-space-v1".into(),
                query_vector: vec![1.0, 0.0],
            },
        )
        .unwrap();
        assert_eq!((hybrid.eligible_memories, hybrid.indexed_memories), (1, 1));
        assert_eq!(hybrid.candidates.len(), 1);
        assert_eq!(hybrid.candidates[0].memory_id, "wanted");
        wanted.revision = 2;
        wanted.entity_id = Some("project_two".into());
        assert!(
            crate::repo::load_fixtures(&mut conn, &[wanted])
                .unwrap_err()
                .contains("cambia de ámbito")
        );
    }
}
