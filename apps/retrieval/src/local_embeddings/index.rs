use super::{Fragment, Profile, TextEncoder, check_deadline, deadline, vector_norm};
use crate::{
    model::{Candidate, SearchRequest, SourceRef},
    write::Scope,
};
use gennie_memory_kernel::digest::sha256_hex;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceDocument {
    pub memory_id: String,
    pub revision: i64,
    pub original: String,
    pub summary: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chunk {
    pub memory_id: String,
    pub revision: i64,
    pub field: String,
    pub ordinal: usize,
    pub source_sha256: String,
    pub fragment: Fragment,
    pub values: Vec<f32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedIndex {
    pub schema_version: u32,
    pub profile: Profile,
    pub actor_id: String,
    pub scope: Scope,
    pub sources: Vec<SourceDocument>,
    pub chunks: Vec<Chunk>,
}
#[derive(Debug, Serialize)]
pub struct IndexReceipt {
    pub status: &'static str,
    pub index_id: String,
    pub space_id: String,
    pub memories: usize,
    pub chunks: usize,
    pub replayed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexRequest {
    pub scope: Scope,
    #[serde(default)]
    pub actor_id: Option<String>,
    pub deadline_ms: u64,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Search {
    pub index_id: String,
    pub search: SearchRequest,
    pub deadline_ms: u64,
    #[serde(default = "max_chunks")]
    pub max_scored_chunks: usize,
    #[serde(default)]
    pub semantic_only: bool,
}
fn max_chunks() -> usize {
    100_000
}
#[derive(Debug, Clone, Serialize)]
pub struct Match {
    #[serde(flatten)]
    pub memory: Candidate,
    pub cosine: f64,
    pub field: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub semantic_rank: usize,
    pub text_rank: Option<usize>,
    pub rrf: f64,
}
#[derive(Debug, Serialize)]
pub struct Response {
    pub status: &'static str,
    pub method: &'static str,
    pub index_id: String,
    pub space_id: String,
    pub eligible_memories: usize,
    pub scored_chunks: usize,
    pub semantic_coverage: &'static str,
    pub candidate_pool_truncated: bool,
    pub candidates: Vec<Match>,
    pub query_embedding_ms: f64,
    pub exact_search_ms: f64,
    pub total_ms: f64,
}

fn scope_valid(actor: &str, scope: &Scope) -> Result<(), String> {
    if !crate::service::valid_identifier(actor)
        || !crate::service::valid_identifier(&scope.id)
        || !matches!(
            scope.kind.as_str(),
            "user" | "agent" | "project" | "conversation"
        )
    {
        return Err("embedding_scope_invalid".into());
    }
    Ok(())
}
fn sources(
    conn: &Connection,
    actor: &str,
    scope: &Scope,
    until: Instant,
) -> Result<Vec<SourceDocument>, String> {
    scope_valid(actor, scope)?;
    check_deadline(until)?;
    let mut stmt=conn.prepare("SELECT m.id,m.current_revision,r.original_text,r.summary FROM memories m JOIN memory_revisions r ON r.memory_id=m.id AND r.revision=m.current_revision WHERE m.actor_id=?1 AND m.scope_kind=?2 AND m.scope_id=?3 AND m.lifecycle_state='active' AND r.state='active' ORDER BY m.id LIMIT 10001").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map(params![actor, scope.kind, scope.id], |r| {
            Ok(SourceDocument {
                memory_id: r.get(0)?,
                revision: r.get(1)?,
                original: r.get(2)?,
                summary: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        check_deadline(until)?;
        result.push(row.map_err(|e| e.to_string())?);
        if result.len() > 10000 {
            return Err("embedding_scope_limit: máximo 10000 recuerdos por índice".into());
        }
    }
    Ok(result)
}
fn manifest(sources: &[SourceDocument]) -> String {
    sha256_hex(&serde_json::to_vec(sources).expect("source serializable"))
}

pub fn prepare(
    conn: &Connection,
    encoder: &impl TextEncoder,
    request: &IndexRequest,
) -> Result<PreparedIndex, String> {
    let until = deadline(request.deadline_ms)?;
    encoder.profile().validate()?;
    let actor = request.actor_id.as_deref().unwrap_or("local");
    let sources = sources(conn, actor, &request.scope, until)?;
    if sources.is_empty() {
        return Err("embedding_scope_empty".into());
    }
    let mut chunks = Vec::new();
    for source in &sources {
        for (field, text) in [("original", &source.original), ("summary", &source.summary)] {
            check_deadline(until)?;
            for (ordinal, fragment) in encoder.fragments(text)?.into_iter().enumerate() {
                if chunks.len() >= 100_000 {
                    return Err("embedding_chunk_limit".into());
                }
                let values =
                    encoder.embed(&text[fragment.start_byte..fragment.end_byte], false, until)?;
                chunks.push(Chunk {
                    memory_id: source.memory_id.clone(),
                    revision: source.revision,
                    field: field.into(),
                    ordinal,
                    source_sha256: sha256_hex(text.as_bytes()),
                    fragment,
                    values,
                });
            }
        }
    }
    check_deadline(until)?;
    Ok(PreparedIndex {
        schema_version: 1,
        profile: encoder.profile().clone(),
        actor_id: actor.into(),
        scope: request.scope.clone(),
        sources,
        chunks,
    })
}

pub fn publish(
    conn: &mut Connection,
    encoder: &impl TextEncoder,
    batch: &PreparedIndex,
) -> Result<IndexReceipt, String> {
    publish_guarded(conn, encoder, batch, None)
}

pub(crate) fn publish_guarded(
    conn: &mut Connection,
    encoder: &impl TextEncoder,
    batch: &PreparedIndex,
    claim: Option<&crate::lifecycle::JobClaim>,
) -> Result<IndexReceipt, String> {
    let until = deadline(30_000)?;
    batch.profile.validate()?;
    if encoder.profile() != &batch.profile {
        return Err("embedding_space_mismatch".into());
    }
    scope_valid(&batch.actor_id, &batch.scope)?;
    if batch.schema_version != 1
        || batch.sources.is_empty()
        || batch.sources.len() > 10000
        || batch.chunks.is_empty()
        || batch.chunks.len() > 100_000
    {
        return Err("embedding_batch_invalid".into());
    }
    let mut groups: HashMap<(&str, i64, &str), Vec<&Chunk>> = HashMap::new();
    for chunk in &batch.chunks {
        check_deadline(until)?;
        vector_norm(&chunk.values, batch.profile.dimensions)?;
        groups
            .entry((&chunk.memory_id, chunk.revision, &chunk.field))
            .or_default()
            .push(chunk);
    }
    let mut ids = HashSet::new();
    for source in &batch.sources {
        if !ids.insert((&source.memory_id, source.revision)) {
            return Err("embedding_source_duplicate".into());
        }
        for (field, text) in [("original", &source.original), ("summary", &source.summary)] {
            let mut parts = groups
                .remove(&(source.memory_id.as_str(), source.revision, field))
                .ok_or("embedding_index_incomplete")?;
            parts.sort_by_key(|c| c.ordinal);
            let expected = encoder.fragments(text)?;
            if expected.len() != parts.len()
                || expected.iter().zip(&parts).any(|(a, b)| {
                    a.start_byte != b.fragment.start_byte
                        || a.end_byte != b.fragment.end_byte
                        || a.token_count != b.fragment.token_count
                        || a.text_sha256 != b.fragment.text_sha256
                })
            {
                return Err("embedding_chunk_tokenization_mismatch".into());
            }
            let mut end = 0;
            let source_hash = sha256_hex(text.as_bytes());
            for (ordinal, chunk) in parts.iter().enumerate() {
                let f = &chunk.fragment;
                if chunk.ordinal != ordinal
                    || f.start_byte != end
                    || f.end_byte <= end
                    || f.end_byte > text.len()
                    || !text.is_char_boundary(f.start_byte)
                    || !text.is_char_boundary(f.end_byte)
                    || chunk.source_sha256 != source_hash
                    || sha256_hex(text[f.start_byte..f.end_byte].as_bytes()) != f.text_sha256
                    || f.token_count == 0
                    || f.token_count > batch.profile.max_tokens
                {
                    return Err("embedding_chunk_invalid".into());
                }
                end = f.end_byte;
            }
            if end != text.len() {
                return Err("embedding_index_incomplete".into());
            }
        }
    }
    if !groups.is_empty() {
        return Err("embedding_chunk_orphan".into());
    }
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    if sources(&tx, &batch.actor_id, &batch.scope, until)? != batch.sources {
        return Err("embedding_snapshot_changed: no se publicó el índice".into());
    }
    let space_id = batch.profile.space_id();
    let profile_json = serde_json::to_string(&batch.profile).map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT OR IGNORE INTO local_embedding_profiles(space_id,profile_json) VALUES (?1,?2)",
        params![space_id, profile_json],
    )
    .map_err(|e| e.to_string())?;
    let actual: String = tx
        .query_row(
            "SELECT profile_json FROM local_embedding_profiles WHERE space_id=?1",
            [&space_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if actual != profile_json {
        return Err("embedding_profile_conflict".into());
    }
    let index_id = format!(
        "idx-{}",
        sha256_hex(&serde_json::to_vec(batch).map_err(|e| e.to_string())?)
    );
    let existing: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM local_embedding_indexes WHERE index_id=?1)",
            [&index_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if !existing {
        tx.execute("INSERT INTO local_embedding_indexes(index_id,space_id,actor_id,scope_kind,scope_id,manifest_sha256,memory_count,chunk_count,published_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![index_id,space_id,batch.actor_id,batch.scope.kind,batch.scope.id,manifest(&batch.sources),batch.sources.len() as i64,batch.chunks.len() as i64,crate::repo::utc_now(&tx)?]).map_err(|e|e.to_string())?;
        for chunk in &batch.chunks {
            check_deadline(until)?;
            tx.execute("INSERT INTO local_embedding_chunks(index_id,memory_id,revision,field,ordinal,start_byte,end_byte,source_sha256,chunk_sha256,token_count,vector_blob,norm) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![index_id,chunk.memory_id,chunk.revision,chunk.field,chunk.ordinal as i64,chunk.fragment.start_byte as i64,chunk.fragment.end_byte as i64,chunk.source_sha256,chunk.fragment.text_sha256,chunk.fragment.token_count as i64,chunk.values.iter().flat_map(|v|v.to_le_bytes()).collect::<Vec<_>>(),vector_norm(&chunk.values,batch.profile.dimensions)?]).map_err(|e|e.to_string())?;
        }
    }
    check_deadline(until)?;
    if let Some(claim) = claim {
        crate::lifecycle::complete(
            &tx,
            claim,
            &space_id,
            &batch.scope,
            &batch.actor_id,
            &index_id,
        )?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(IndexReceipt {
        status: "published_complete",
        index_id,
        space_id,
        memories: batch.sources.len(),
        chunks: batch.chunks.len(),
        replayed: existing,
    })
}

pub fn search(
    conn: &Connection,
    encoder: &impl TextEncoder,
    request: &Search,
) -> Result<Response, String> {
    let start = Instant::now();
    let until = deadline(request.deadline_ms)?;
    crate::service::validate_search_request(&request.search)?;
    if request.max_scored_chunks == 0 || request.max_scored_chunks > 1_000_000 {
        return Err("embedding_work_budget_invalid".into());
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let actor = request.search.actor_id.as_deref().unwrap_or("local");
    let scope = Scope {
        kind: request.search.scope_kind.clone(),
        id: request.search.scope_id.clone(),
    };
    let record:Option<(String,String,i64,i64,String)>=tx.query_row("SELECT i.space_id,i.manifest_sha256,i.memory_count,i.chunk_count,p.profile_json FROM local_embedding_indexes i JOIN local_embedding_profiles p ON p.space_id=i.space_id WHERE i.index_id=?1 AND i.actor_id=?2 AND i.scope_kind=?3 AND i.scope_id=?4",params![request.index_id,actor,scope.kind,scope.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(|e|e.to_string())?;
    let (space, expected, memory_count, chunk_count, profile_json) =
        record.ok_or("embedding_index_not_available")?;
    if space != encoder.profile().space_id()
        || serde_json::from_str::<Profile>(&profile_json).map_err(|e| e.to_string())?
            != *encoder.profile()
    {
        return Err("embedding_space_mismatch".into());
    }
    let current = sources(&tx, actor, &scope, until)?;
    if current.len() as i64 != memory_count || manifest(&current) != expected {
        return Err(
            "embedding_index_stale: reconstruye el índice para las revisiones actuales".into(),
        );
    }
    let count: i64 = tx
        .query_row(
            "SELECT count(*) FROM local_embedding_chunks WHERE index_id=?1",
            [&request.index_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if count != chunk_count {
        return Err("embedding_index_incomplete".into());
    }
    let lookup: HashMap<_, _> = current
        .iter()
        .map(|s| ((s.memory_id.as_str(), s.revision), s))
        .collect();
    let query_start = Instant::now();
    let query = encoder.embed(&request.search.query, true, until)?;
    let query_norm = vector_norm(&query, encoder.profile().dimensions)?;
    let query_embedding_ms = query_start.elapsed().as_secs_f64() * 1000.0;
    let exact = Instant::now();
    let mut stmt=tx.prepare("SELECT c.memory_id,c.revision,c.field,c.start_byte,c.end_byte,c.chunk_sha256,c.vector_blob,c.norm,r.summary,r.conditions,r.source_kind,r.source_ref,r.source_locator,CASE WHEN EXISTS(SELECT 1 FROM memory_proposal_targets pt WHERE pt.memory_id=m.id) OR EXISTS(SELECT 1 FROM memory_review_schedule ms WHERE ms.memory_id=m.id AND ms.revision=m.current_revision AND ms.next_review_at IS NOT NULL AND ms.next_review_at<=?5) THEN 'uncertain' ELSE r.state END FROM local_embedding_chunks c JOIN memories m ON m.id=c.memory_id JOIN memory_revisions r ON r.memory_id=m.id AND r.revision=c.revision WHERE c.index_id=?1 AND m.current_revision=c.revision AND m.actor_id=?2 AND m.scope_kind=?3 AND m.scope_id=?4 AND m.lifecycle_state='active' AND r.state='active' AND (r.valid_from IS NULL OR r.valid_from<=?5) AND (r.valid_until IS NULL OR r.valid_until>?5) AND (?6 IS NULL OR m.entity_id=?6) AND (json_array_length(?7)=0 OR r.memory_type IN (SELECT value FROM json_each(?7))) ORDER BY c.memory_id,c.field,c.ordinal").map_err(|e|e.to_string())?;
    let mut rows = stmt
        .query(params![
            request.index_id,
            actor,
            scope.kind,
            scope.id,
            request.search.as_of,
            request.search.entity_id,
            serde_json::to_string(&request.search.memory_types).map_err(|e| e.to_string())?
        ])
        .map_err(|e| e.to_string())?;
    let mut best: HashMap<String, Match> = HashMap::new();
    let mut scored = 0;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        check_deadline(until)?;
        scored += 1;
        if scored > request.max_scored_chunks {
            return Err("embedding_work_budget_exceeded".into());
        }
        let id: String = row.get(0).map_err(|e| e.to_string())?;
        let revision: i64 = row.get(1).map_err(|e| e.to_string())?;
        let field: String = row.get(2).map_err(|e| e.to_string())?;
        let from: i64 = row.get(3).map_err(|e| e.to_string())?;
        let to: i64 = row.get(4).map_err(|e| e.to_string())?;
        let source = lookup
            .get(&(id.as_str(), revision))
            .ok_or("embedding_source_missing")?;
        let text = match field.as_str() {
            "original" => &source.original,
            "summary" => &source.summary,
            _ => return Err("embedding_field_invalid".into()),
        };
        if from < 0
            || to <= from
            || to as usize > text.len()
            || !text.is_char_boundary(from as usize)
            || !text.is_char_boundary(to as usize)
        {
            return Err("embedding_chunk_invalid".into());
        }
        if sha256_hex(text[from as usize..to as usize].as_bytes())
            != row.get::<_, String>(5).map_err(|e| e.to_string())?
        {
            return Err("embedding_chunk_hash_mismatch".into());
        }
        let bytes: Vec<u8> = row.get(6).map_err(|e| e.to_string())?;
        if bytes.len() != encoder.profile().dimensions * 4 {
            return Err("embedding_vector_format_invalid".into());
        }
        let values: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().expect("four bytes")))
            .collect();
        let norm = vector_norm(&values, encoder.profile().dimensions)?;
        let stored: f64 = row.get(7).map_err(|e| e.to_string())?;
        if !stored.is_finite() || (stored - norm).abs() > 1e-5 {
            return Err("embedding_norm_invalid".into());
        }
        let cosine = (values
            .iter()
            .zip(&query)
            .map(|(a, b)| f64::from(*a) * f64::from(*b))
            .sum::<f64>()
            / (norm * query_norm))
            .clamp(-1.0, 1.0);
        if best.get(&id).is_some_and(|m| m.cosine >= cosine) {
            continue;
        }
        let kind: Option<String> = row.get(10).map_err(|e| e.to_string())?;
        let reference: Option<String> = row.get(11).map_err(|e| e.to_string())?;
        let locator: Option<String> = row.get(12).map_err(|e| e.to_string())?;
        let source = match (kind, reference) {
            (Some(kind), Some(reference)) => Some(SourceRef {
                kind,
                reference,
                locator,
            }),
            (None, None) if locator.is_none() => None,
            _ => return Err("embedding_source_invalid".into()),
        };
        best.insert(
            id.clone(),
            Match {
                memory: Candidate {
                    memory_id: id,
                    revision,
                    summary: row.get(8).map_err(|e| e.to_string())?,
                    conditions: row.get(9).map_err(|e| e.to_string())?,
                    source,
                    state: row.get(13).map_err(|e| e.to_string())?,
                    bm25: 0.0,
                },
                cosine,
                field,
                start_byte: from as usize,
                end_byte: to as usize,
                semantic_rank: 0,
                text_rank: None,
                rrf: 0.0,
            },
        );
    }
    drop(rows);
    drop(stmt);
    let eligible = best.len();
    let mut candidates: Vec<_> = best.into_values().collect();
    candidates.sort_by(|a, b| {
        b.cosine
            .total_cmp(&a.cosine)
            .then_with(|| a.memory.memory_id.cmp(&b.memory.memory_id))
    });
    for (i, c) in candidates.iter_mut().enumerate() {
        c.semantic_rank = i + 1;
        c.rrf = 1.0 / (60.0 + c.semantic_rank as f64);
    }
    let exact_search_ms = exact.elapsed().as_secs_f64() * 1000.0;
    if !request.semantic_only {
        let mut text = request.search.clone();
        text.limit = Some(20);
        let found = crate::service::search(&tx, &text)?;
        if found.status != "complete" {
            return Err("embedding_text_index_incomplete".into());
        }
        let text: HashMap<_, _> = found
            .candidates
            .iter()
            .enumerate()
            .map(|(i, c)| (c.memory_id.as_str(), i + 1))
            .collect();
        for c in &mut candidates {
            if c.semantic_rank > 20 {
                c.rrf = 0.0;
            }
            c.text_rank = text.get(c.memory.memory_id.as_str()).copied();
            if let Some(rank) = c.text_rank {
                c.rrf += 1.0 / (60.0 + rank as f64);
            }
        }
        candidates.retain(|c| c.semantic_rank <= 20 || c.text_rank.is_some());
        candidates.sort_by(|a, b| {
            b.rrf
                .total_cmp(&a.rrf)
                .then_with(|| {
                    a.text_rank
                        .unwrap_or(usize::MAX)
                        .cmp(&b.text_rank.unwrap_or(usize::MAX))
                })
                .then_with(|| a.semantic_rank.cmp(&b.semantic_rank))
                .then_with(|| a.memory.memory_id.cmp(&b.memory.memory_id))
        });
    }
    let truncated = eligible > request.search.limit.unwrap_or(8) as usize;
    candidates.truncate(request.search.limit.unwrap_or(8) as usize);
    check_deadline(until)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(Response {
        status: "complete",
        method: if request.semantic_only {
            "local_exact_cosine_v1"
        } else {
            "local_fts5_cosine_rrf_v1"
        },
        index_id: request.index_id.clone(),
        space_id: space,
        eligible_memories: eligible,
        scored_chunks: scored,
        semantic_coverage: "complete",
        candidate_pool_truncated: truncated,
        candidates,
        query_embedding_ms,
        exact_search_ms,
        total_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Run one explicitly queued complete-index job without loading a background daemon.
pub fn process_job(
    conn: &mut Connection,
    encoder: &impl TextEncoder,
    request: &IndexRequest,
) -> Result<serde_json::Value, String> {
    if request.actor_id.as_deref().unwrap_or("local") != "local" {
        return Err("queue_requires_local_actor".into());
    }
    let job = crate::lifecycle::JobRequest {
        action: "get".into(),
        scope: request.scope.clone(),
        space_id: encoder.profile().space_id(),
    };
    let claim = crate::lifecycle::claim(conn, &job)?;
    let result = (|| {
        // Empty scopes still finish, purging previous complete indexes atomically.
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        if sources(&tx, "local", &request.scope, deadline(request.deadline_ms)?)?.is_empty() {
            tx.execute("DELETE FROM local_embedding_indexes WHERE actor_id='local' AND scope_kind=?1 AND scope_id=?2 AND space_id=?3",params![request.scope.kind,request.scope.id,job.space_id]).map_err(|e|e.to_string())?;
            crate::lifecycle::complete(&tx, &claim, &job.space_id, &request.scope, "local", "")?;
            tx.commit().map_err(|e| e.to_string())?;
            return Ok(
                serde_json::json!({"status":"completed_empty","job_id":claim.job_id,"index_id":null}),
            );
        }
        tx.commit().map_err(|e| e.to_string())?;
        let batch = prepare(conn, encoder, request)?;
        serde_json::to_value(publish_guarded(conn, encoder, &batch, Some(&claim))?)
            .map_err(|e| e.to_string())
    })();
    if result.is_err() {
        crate::lifecycle::fail(conn, &claim)?;
    }
    result
}
