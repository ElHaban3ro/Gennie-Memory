//! Explicit lifecycle transitions and a durable, fenced queue for complete local indexes.
use crate::write::Scope;
use gennie_memory_kernel::digest::sha256_hex;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleRequest {
    pub schema_version: u32,
    pub operation_id: String,
    pub action: String,
    pub memory_id: String,
    pub expected_revision: i64,
    pub scope: Scope,
    pub as_of: String,
    pub next_review_at: Option<String>,
    pub evidence: Option<crate::reconciliation::Evidence>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewListRequest {
    pub scope: Scope,
    pub as_of: String,
    pub limit: usize,
    #[serde(default)]
    pub after_id: Option<String>,
}
fn valid_scope(scope: &Scope) -> Result<(), String> {
    if !matches!(
        scope.kind.as_str(),
        "user" | "agent" | "project" | "conversation"
    ) || !crate::service::valid_identifier(&scope.id)
    {
        return Err("invalid_scope: ámbito inválido".into());
    }
    Ok(())
}
fn valid_date(date: &str) -> Result<(), String> {
    if !crate::service::valid_utc(date) {
        return Err("invalid_date: se requiere UTC RFC 3339 con milisegundos".into());
    }
    Ok(())
}

pub fn apply(conn: &mut Connection, request: &LifecycleRequest) -> Result<Value, String> {
    valid_scope(&request.scope)?;
    valid_date(&request.as_of)?;
    if request.schema_version != 1
        || !crate::write::valid_operation_id(&request.operation_id)
        || !crate::service::valid_identifier(&request.memory_id)
        || request.expected_revision < 1
        || !matches!(
            request.action.as_str(),
            "withdraw" | "restore" | "delete" | "review"
        )
    {
        return Err("invalid_lifecycle_request".into());
    }
    let evidence = request
        .evidence
        .as_ref()
        .ok_or("lifecycle_evidence_required")?;
    crate::write::validate_source(&evidence.source)?;
    if evidence.original_text.trim().is_empty() || evidence.original_text.chars().count() > 16000 {
        return Err("lifecycle_evidence_invalid".into());
    }
    if request.action == "review" {
        let next = request
            .next_review_at
            .as_deref()
            .ok_or("review_requires_next_review_at")?;
        valid_date(next)?;
        if next <= request.as_of.as_str() {
            return Err("next_review_at_must_be_future".into());
        }
    } else if request.next_review_at.is_some() {
        return Err("unexpected_next_review_at".into());
    }
    let mut canonical = request.clone();
    canonical.operation_id.make_ascii_lowercase();
    let hash = sha256_hex(&serde_json::to_vec(&canonical).map_err(|e| e.to_string())?);
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let previous:Option<(String,String)>=tx.query_row("SELECT request_sha256,response_json FROM memory_lifecycle_operations WHERE operation_id=?1",[&canonical.operation_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
    if let Some((saved, response)) = previous {
        if saved != hash {
            return Err("idempotency_conflict".into());
        }
        let mut value: Value = serde_json::from_str(&response).map_err(|e| e.to_string())?;
        value["replayed"] = json!(true);
        return Ok(value);
    }
    let collision:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM reconciliation_operations WHERE operation_id=?1 UNION ALL SELECT 1 FROM memory_write_operations WHERE operation_id=?1 UNION ALL SELECT 1 FROM entity_operations WHERE operation_id=?1)",[&canonical.operation_id],|r|r.get(0)).map_err(|e|e.to_string())?;
    if collision {
        return Err("idempotency_conflict".into());
    }
    let current:Option<(i64,String)>=tx.query_row("SELECT current_revision,lifecycle_state FROM memories WHERE id=?1 AND actor_id='local' AND scope_kind=?2 AND scope_id=?3",params![request.memory_id,request.scope.kind,request.scope.id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
    let (mut revision, state) = current.ok_or("memory_not_found")?;
    if revision != request.expected_revision {
        return Err("revision_conflict".into());
    }
    match request.action.as_str() {
        "withdraw" | "restore" => {
            let target = if request.action == "withdraw" {
                "withdrawn"
            } else {
                "active"
            };
            if state == target {
                return Err("invalid_lifecycle_transition".into());
            }
            let schedule:(Option<String>,Option<String>)=tx.query_row("SELECT next_review_at,reviewed_at FROM memory_review_schedule WHERE memory_id=?1",[&request.memory_id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
            let next = revision.checked_add(1).ok_or("revision_overflow")?;
            tx.execute("INSERT INTO memory_revisions(memory_id,revision,memory_type,original_text,summary,state,valid_from,valid_until,conditions,source_kind,source_ref,source_locator) SELECT memory_id,?3,memory_type,original_text,summary,state,valid_from,valid_until,conditions,source_kind,source_ref,source_locator FROM memory_revisions WHERE memory_id=?1 AND revision=?2",params![request.memory_id,revision,next]).map_err(|e|e.to_string())?;
            tx.execute("INSERT INTO search_documents(memory_id,revision,original_text,summary) SELECT memory_id,?3,original_text,summary FROM memory_revisions WHERE memory_id=?1 AND revision=?2",params![request.memory_id,revision,next]).map_err(|e|e.to_string())?;
            tx.execute("INSERT INTO memory_source_evidence(memory_id,revision,source_kind,source_ref,source_locator,evidence_text,verification_status,recorded_at) SELECT memory_id,?3,source_kind,source_ref,source_locator,evidence_text,verification_status,recorded_at FROM memory_source_evidence WHERE memory_id=?1 AND revision=?2",params![request.memory_id,revision,next]).map_err(|e|e.to_string())?;
            tx.execute(
                "UPDATE memories SET lifecycle_state=?2,current_revision=?3 WHERE id=?1",
                params![request.memory_id, target, next],
            )
            .map_err(|e| e.to_string())?;
            tx.execute(
                "UPDATE memory_content_keys SET revision=?2 WHERE memory_id=?1",
                params![request.memory_id, next],
            )
            .map_err(|e| e.to_string())?;
            tx.execute("UPDATE memory_review_schedule SET next_review_at=?2,reviewed_at=?3 WHERE memory_id=?1",params![request.memory_id,schedule.0,schedule.1]).map_err(|e|e.to_string())?;
            revision = next;
        }
        "review" => {
            if state != "active" {
                return Err("memory_withdrawn".into());
            }
            tx.execute("UPDATE memory_review_schedule SET reviewed_at=?2,next_review_at=?3 WHERE memory_id=?1 AND revision=?4",params![request.memory_id,request.as_of,request.next_review_at,revision]).map_err(|e|e.to_string())?;
        }
        "delete" => purge(&tx, &request.memory_id)?,
        _ => unreachable!(),
    }
    if request.action != "delete" {
        let provenance = crate::source_verification::check_evidence(
            &tx,
            &request.scope,
            "local",
            &evidence.source,
            &evidence.original_text,
        )?;
        let text = serde_json::to_string(evidence).map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO memory_evidence_audit(operation_id,actor_id,scope_kind,scope_id,memory_id,revision,action,evidence_json,evidence_sha256,provenance_json,recorded_at) VALUES(?1,'local',?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![canonical.operation_id,request.scope.kind,request.scope.id,request.memory_id,revision,request.action,text,sha256_hex(text.as_bytes()),serde_json::to_string(&provenance).map_err(|e|e.to_string())?,crate::repo::utc_now(&tx)?]).map_err(|e|e.to_string())?;
    }
    let response = json!({"schema_version":1,"status":"committed","action":request.action,"memory_id":request.memory_id,"revision":revision,"replayed":false});
    tx.execute(
        "INSERT INTO memory_lifecycle_operations VALUES(?1,?2,?3,?4)",
        params![
            canonical.operation_id,
            hash,
            response.to_string(),
            crate::repo::utc_now(&tx)?
        ],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(response)
}

fn purge(tx: &rusqlite::Transaction<'_>, id: &str) -> Result<(), String> {
    // Remove text-bearing dependents before deleting revisions; keep receipts without text.
    tx.execute("INSERT OR IGNORE INTO memory_erased_write_operations(operation_id,request_sha256,erased_at) SELECT operation_id,request_sha256,strftime('%Y-%m-%dT%H:%M:%fZ','now') FROM memory_write_operations WHERE memory_id=?1",[id]).map_err(|e|e.to_string())?;
    tx.execute("UPDATE reconciliation_operations SET response_json=json_object('schema_version',1,'operation_id',operation_id,'status','committed','outcome','content_deleted','memory_id',?1,'scope',json_extract(response_json,'$.scope'),'replayed',json('false')) WHERE json_extract(response_json,'$.memory_id')=?1 OR operation_id IN (SELECT operation_id FROM memory_evidence_audit WHERE memory_id=?1 OR proposal_id IN (SELECT proposal_id FROM memory_proposal_targets WHERE memory_id=?1) UNION SELECT operation_id FROM memory_relations WHERE from_memory_id=?1 OR to_memory_id=?1 UNION SELECT operation_id FROM memory_write_operations WHERE memory_id=?1)",[id]).map_err(|e|e.to_string())?;
    for sql in [
        "DELETE FROM memory_evidence_audit WHERE memory_id=?1 OR proposal_id IN (SELECT proposal_id FROM memory_proposal_targets WHERE memory_id=?1)",
        "DELETE FROM memory_proposals WHERE proposal_id IN (SELECT proposal_id FROM memory_proposal_targets WHERE memory_id=?1)",
        "DELETE FROM source_verification_checks WHERE memory_id=?1",
        "DELETE FROM local_embedding_indexes WHERE index_id IN (SELECT index_id FROM local_embedding_chunks WHERE memory_id=?1)",
        "DELETE FROM memory_write_operations WHERE memory_id=?1",
        "DELETE FROM memories WHERE id=?1",
    ] {
        tx.execute(sql, [id]).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn review_list(conn: &Connection, request: &ReviewListRequest) -> Result<Value, String> {
    valid_scope(&request.scope)?;
    valid_date(&request.as_of)?;
    if !(1..=100).contains(&request.limit)
        || request
            .after_id
            .as_deref()
            .is_some_and(|v| !crate::service::valid_identifier(v))
    {
        return Err("invalid_review_list_request".into());
    }
    let mut stmt=conn.prepare("SELECT m.id,m.current_revision,r.state,r.valid_from,r.valid_until,s.next_review_at,s.reviewed_at,EXISTS(SELECT 1 FROM memory_proposal_targets pt WHERE pt.memory_id=m.id) FROM memories m JOIN memory_revisions r ON r.memory_id=m.id AND r.revision=m.current_revision JOIN memory_review_schedule s ON s.memory_id=m.id AND s.revision=m.current_revision WHERE m.actor_id='local' AND m.scope_kind=?1 AND m.scope_id=?2 AND m.lifecycle_state='active' AND (?3 IS NULL OR m.id>?3) AND (s.next_review_at IS NULL OR s.next_review_at<=?4 OR (r.valid_until IS NOT NULL AND r.valid_until<=?4) OR EXISTS(SELECT 1 FROM memory_proposal_targets pt WHERE pt.memory_id=m.id)) ORDER BY m.id LIMIT ?5").map_err(|e|e.to_string())?;
    let rows=stmt.query_map(params![request.scope.kind,request.scope.id,request.after_id,request.as_of,request.limit as i64+1],|r| {
        let raw:String=r.get(2)?;
        let valid_from:Option<String>=r.get(3)?;
        let valid_until:Option<String>=r.get(4)?;
        let next_review_at:Option<String>=r.get(5)?;
        let conflict:bool=r.get(7)?;
        let expired=valid_until.as_deref().is_some_and(|v|v<=request.as_of.as_str());
        let due=next_review_at.as_deref().is_some_and(|v|v<=request.as_of.as_str());
        let mut reasons=Vec::new();
        if next_review_at.is_none(){reasons.push("not_scheduled");}
        if due{reasons.push("review_due");}
        if expired{reasons.push("validity_expired");}
        if conflict{reasons.push("pending_reconciliation");}
        let state=if expired{"expired"}else if due||conflict{"uncertain"}else{raw.as_str()};
        Ok(json!({"memory_id":r.get::<_,String>(0)?,"revision":r.get::<_,i64>(1)?,"state":state,"review_reasons":reasons,"applicable_now":state=="active" && valid_from.as_deref().is_none_or(|v|v<=request.as_of.as_str()),"valid_from":valid_from,"valid_until":valid_until,"next_review_at":next_review_at,"reviewed_at":r.get::<_,Option<String>>(6)?}))
    }).map_err(|e|e.to_string())?;
    let mut items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let more = items.len() > request.limit;
    items.truncate(request.limit);
    let next = if more {
        items.last().map(|v| v["memory_id"].clone())
    } else {
        None
    };
    Ok(json!({"schema_version":1,"items":items,"next_after_id":next}))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobRequest {
    pub action: String,
    pub scope: Scope,
    pub space_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobClaim {
    pub job_id: String,
    pub generation: i64,
    pub lease_token: String,
}

pub fn job(conn: &mut Connection, request: &JobRequest) -> Result<Value, String> {
    valid_scope(&request.scope)?;
    if !crate::service::valid_identifier(&request.space_id)
        || !matches!(
            request.action.as_str(),
            "enqueue" | "retry" | "cancel" | "get"
        )
    {
        return Err("invalid_job_request".into());
    }
    let id = sha256_hex(
        &serde_json::to_vec(&(
            "local",
            &request.scope.kind,
            &request.scope.id,
            &request.space_id,
        ))
        .map_err(|e| e.to_string())?,
    );
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    match request.action.as_str() {
        "enqueue" => {
            tx.execute("INSERT INTO embedding_jobs(job_id,actor_id,scope_kind,scope_id,space_id,status,updated_at) VALUES(?1,'local',?2,?3,?4,'pending',strftime('%Y-%m-%dT%H:%M:%fZ','now')) ON CONFLICT(job_id) DO NOTHING",params![id,request.scope.kind,request.scope.id,request.space_id]).map_err(|e|e.to_string())?;
        }
        "retry" => {
            let n=tx.execute("UPDATE embedding_jobs SET status='pending',generation=generation+1,lease_token=NULL,lease_until=NULL,error_code=NULL,index_id=NULL,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE job_id=?1 AND status IN ('failed','cancelled','completed')",[&id]).map_err(|e|e.to_string())?;
            if n != 1 {
                return Err("job_not_retryable".into());
            }
        }
        "cancel" => {
            let n=tx.execute("UPDATE embedding_jobs SET status='cancelled',lease_token=NULL,lease_until=NULL,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE job_id=?1 AND status IN ('pending','running','failed')",[&id]).map_err(|e|e.to_string())?;
            if n != 1 {
                return Err("job_not_cancellable".into());
            }
        }
        _ => {}
    }
    let result=tx.query_row("SELECT generation,status,attempts,error_code,index_id,lease_until FROM embedding_jobs WHERE job_id=?1",[&id],|r|Ok(json!({"job_id":id,"generation":r.get::<_,i64>(0)?,"status":r.get::<_,String>(1)?,"attempts":r.get::<_,i64>(2)?,"error_code":r.get::<_,Option<String>>(3)?,"index_id":r.get::<_,Option<String>>(4)?,"lease_until":r.get::<_,Option<String>>(5)?}))).optional().map_err(|e|e.to_string())?.ok_or("job_not_found")?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(result)
}

pub fn claim(conn: &mut Connection, request: &JobRequest) -> Result<JobClaim, String> {
    // Atomic acquisition also recovers an abandoned lease after a process crash.
    valid_scope(&request.scope)?;
    if !crate::service::valid_identifier(&request.space_id) {
        return Err("invalid_space_id".into());
    }
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let selected:Option<(String,i64)>=tx.query_row("UPDATE embedding_jobs SET status='running',attempts=attempts+1,lease_token=lower(hex(randomblob(16))),lease_until=strftime('%Y-%m-%dT%H:%M:%fZ','now','+10 minutes'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),error_code=NULL WHERE actor_id='local' AND scope_kind=?1 AND scope_id=?2 AND space_id=?3 AND (status='pending' OR (status='running' AND lease_until<=strftime('%Y-%m-%dT%H:%M:%fZ','now'))) RETURNING job_id,generation",params![request.scope.kind,request.scope.id,request.space_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
    let (job_id, generation) = selected.ok_or("job_not_available")?;
    let lease_token = tx
        .query_row(
            "SELECT lease_token FROM embedding_jobs WHERE job_id=?1",
            [&job_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(JobClaim {
        job_id,
        generation,
        lease_token,
    })
}

#[cfg(feature = "local-embeddings")]
pub(crate) fn complete(
    conn: &Connection,
    claim: &JobClaim,
    space: &str,
    scope: &Scope,
    actor: &str,
    index_id: &str,
) -> Result<(), String> {
    let n=conn.execute("UPDATE embedding_jobs SET status='completed',index_id=NULLIF(?4,''),lease_token=NULL,lease_until=NULL,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE job_id=?1 AND generation=?2 AND lease_token=?3 AND status='running' AND lease_until>strftime('%Y-%m-%dT%H:%M:%fZ','now') AND space_id=?5 AND actor_id=?6 AND scope_kind=?7 AND scope_id=?8",params![claim.job_id,claim.generation,claim.lease_token,index_id,space,actor,scope.kind,scope.id]).map_err(|e|e.to_string())?;
    if n != 1 {
        return Err("job_cancelled_stale_or_lease_expired".into());
    }
    Ok(())
}
#[cfg(feature = "local-embeddings")]
pub fn fail(conn: &Connection, claim: &JobClaim) -> Result<(), String> {
    conn.execute("UPDATE embedding_jobs SET status='failed',error_code='embedding_job_failed',lease_token=NULL,lease_until=NULL,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE job_id=?1 AND generation=?2 AND lease_token=?3 AND status='running'",params![claim.job_id,claim.generation,claim.lease_token]).map_err(|e|e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests;
