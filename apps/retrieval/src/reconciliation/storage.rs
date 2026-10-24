use super::*;
use crate::write::{self, Scope};
use gennie_memory_kernel::digest::sha256_hex;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde_json::{Value, json};

pub(super) fn context(scope: &Scope) -> Result<(), String> {
    if !matches!(
        scope.kind.as_str(),
        "user" | "agent" | "project" | "conversation"
    ) || !crate::service::valid_identifier(&scope.id)
    {
        return Err("scope_invalid: ámbito inválido".into());
    }
    Ok(())
}
pub(super) fn identity(version: u32, op: &str) -> Result<String, String> {
    if version != 1 || !write::valid_operation_id(op) {
        return Err("operation_invalid: versión o UUID inválido".into());
    }
    Ok(op.to_ascii_lowercase())
}
pub(super) fn hash<T: serde::Serialize>(v: &T) -> Result<String, String> {
    Ok(sha256_hex(
        &serde_json::to_vec(v).map_err(|e| e.to_string())?,
    ))
}
pub(super) fn replay(conn: &Connection, op: &str, hash: &str) -> Result<Option<Value>, String> {
    let row:Option<(String,String)>=conn.query_row("SELECT request_sha256,response_json FROM reconciliation_operations WHERE operation_id=?1",[op],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
    if let Some((old, result)) = row {
        if old != hash {
            return Err("idempotency_conflict: UUID usado con otra solicitud".into());
        }
        let mut v: Value = serde_json::from_str(&result).map_err(|e| e.to_string())?;
        v["replayed"] = json!(true);
        return Ok(Some(v));
    }
    Ok(None)
}
pub(super) fn collision(conn: &Connection, op: &str) -> Result<(), String> {
    let yes:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM entity_operations WHERE operation_id=?1 UNION ALL SELECT 1 FROM memory_write_operations WHERE operation_id=?1)",[op],|r|r.get(0)).map_err(|e|e.to_string())?;
    if yes {
        return Err("idempotency_conflict: UUID ya usado en otra superficie".into());
    }
    Ok(())
}
pub(super) fn save(tx: &Transaction, op: &str, hash: &str, response: &Value) -> Result<(), String> {
    tx.execute(
        "INSERT INTO reconciliation_operations VALUES (?1,?2,?3,?4)",
        params![op, hash, response.to_string(), crate::repo::utc_now(tx)?],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
pub(super) fn load(
    conn: &Connection,
    scope: &Scope,
    id: &str,
) -> Result<RelationCandidate, String> {
    conn.query_row("SELECT m.current_revision,m.topic_key,r.original_text,r.summary,r.memory_type,r.conditions,r.valid_from,r.valid_until,r.source_kind,r.source_ref,r.source_locator FROM memories m JOIN memory_revisions r ON r.memory_id=m.id AND r.revision=m.current_revision WHERE m.id=?1 AND m.actor_id='local' AND m.scope_kind=?2 AND m.scope_id=?3",params![id,scope.kind,scope.id],|r|Ok(RelationCandidate{target:Target{memory_id:id.into(),revision:r.get(0)?},topic_key:r.get(1)?,memory:ExistingMemory{original_text:r.get(2)?,summary:r.get(3)?,memory_type:r.get(4)?,conditions:r.get(5)?,valid_from:r.get(6)?,valid_until:r.get(7)?,source:match (r.get::<_,Option<String>>(8)?,r.get::<_,Option<String>>(9)?){(Some(kind),Some(reference))=>Some(crate::model::SourceRef{kind,reference,locator:r.get(10)?}),(None,None)=>None,_=>return Err(rusqlite::Error::InvalidQuery)}}})).map_err(|_|"memory_not_available: recuerdo no disponible para el operador local en este ámbito".into())
}
pub(super) fn evidence_valid(e: &Evidence) -> Result<(), String> {
    write::validate_source(&e.source)?;
    if e.original_text.trim().is_empty() || e.original_text.chars().count() > 16000 {
        return Err("evidence_invalid: cita vacía o demasiado larga".into());
    }
    Ok(())
}
pub(super) fn audit_evidence(
    tx: &Transaction,
    op: &str,
    scope: &Scope,
    target: Option<&Target>,
    proposal: Option<&str>,
    action: &str,
    e: &Evidence,
) -> Result<Value, String> {
    evidence_valid(e)?;
    let provenance = crate::source_verification::check_evidence(
        tx,
        scope,
        "local",
        &e.source,
        &e.original_text,
    )?;
    let evidence = serde_json::to_value(e).map_err(|e| e.to_string())?;
    let p = serde_json::to_value(provenance).map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO memory_evidence_audit(operation_id,actor_id,scope_kind,scope_id,memory_id,revision,proposal_id,action,evidence_json,evidence_sha256,provenance_json,recorded_at) VALUES (?1,'local',?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![op,scope.kind,scope.id,target.map(|t|&t.memory_id),target.map(|t|t.revision),proposal,action,evidence.to_string(),hash(&evidence)?,p.to_string(),crate::repo::utc_now(tx)?]).map_err(|e|e.to_string())?;
    Ok(p)
}
pub(super) fn receipt(op: &str, outcome: &str, target: &Target) -> Value {
    json!({"schema_version":1,"operation_id":op,"status":"committed","outcome":outcome,"memory_id":target.memory_id,"revision":target.revision,"replayed":false,"indexing":"fts5_committed","warnings":[]})
}
pub(super) fn normalize_topic(topic: Option<&str>) -> Result<Option<String>, String> {
    topic
        .map(|t| {
            let v = t.to_lowercase();
            if v.chars().count() > 160
                || v.split('/')
                    .any(|p| p.is_empty() || p.trim() != p || p.chars().any(char::is_control))
            {
                return Err("topic_key_invalid".into());
            }
            Ok(v)
        })
        .transpose()
}
pub(super) fn legacy(c: &WriteCommand, scope: &Scope) -> Result<write::WriteRequest, String> {
    let r = write::WriteRequest {
        schema_version: c.schema_version,
        operation_id: c.operation_id.to_ascii_lowercase(),
        action: c.action.clone(),
        memory_id: c.memory_id.clone(),
        expected_revision: c.expected_revision,
        entity_id: c.entity_id.clone(),
        scope: scope.clone(),
        memory: c.memory.clone().ok_or("memory_required")?,
    };
    write::validate(&r)?;
    Ok(r)
}
pub(super) fn ensure_revision(c: &RelationCandidate, expected: Option<i64>) -> Result<(), String> {
    if expected != Some(c.target.revision) {
        Err("revision_conflict: vuelve a leer la revisión vigente".into())
    } else {
        Ok(())
    }
}
pub(super) fn link(
    tx: &Transaction,
    from: &Target,
    to: &Target,
    relation: &str,
    op: &str,
    e: &Value,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO memory_relations VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            from.memory_id,
            from.revision,
            to.memory_id,
            to.revision,
            relation,
            op,
            e.to_string()
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
