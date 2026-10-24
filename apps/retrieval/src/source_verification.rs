//! Literal provenance against operator-registered local snapshots, never factual truth.
#[cfg(test)]
mod tests;

use crate::{
    model::{Candidate, SourceRef},
    service::valid_identifier,
    write::Scope,
};
use gennie_memory_kernel::digest::sha256_hex;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

pub const MAX_SOURCE_BYTES: usize = 262_144;
pub const PROFILE: &str = "local_snapshot_literal_v1";

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourcePolicy {
    #[default]
    Report,
    RequireVerified,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterRequest {
    pub schema_version: u32,
    #[serde(default)]
    pub actor_id: Option<String>,
    pub scope: Scope,
    pub source: SourceRef,
    pub version: String,
    pub text: String,
}
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub snapshot_id: String,
    pub version: String,
    pub text_sha256: String,
    pub registered_at: String,
    pub replayed: bool,
    pub trust_basis: &'static str,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyRequest {
    pub schema_version: u32,
    #[serde(default)]
    pub actor_id: Option<String>,
    pub scope: Scope,
    pub memory_id: String,
    pub revision: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCheck {
    pub check_id: String,
    pub memory_id: String,
    pub revision: i64,
    pub source: Option<SourceRef>,
    pub snapshot_id: Option<String>,
    pub snapshot_version: Option<String>,
    pub snapshot_sha256: Option<String>,
    pub observed_snapshot_sha256: Option<String>,
    pub original_sha256: String,
    pub start_byte: Option<usize>,
    pub end_byte: Option<usize>,
    pub status: String,
    pub reason_code: String,
    pub checked_at: String,
    pub profile: String,
    pub semantic_support: String,
}
#[derive(Debug, Serialize, Default)]
pub struct SourceReport {
    pub policy: SourcePolicy,
    pub checks: Vec<SourceCheck>,
}

fn validate_context(actor: &str, scope: &Scope) -> Result<(), String> {
    if !valid_identifier(actor)
        || !valid_identifier(&scope.id)
        || !matches!(
            scope.kind.as_str(),
            "user" | "agent" | "project" | "conversation"
        )
    {
        return Err("source_context_invalid".into());
    }
    Ok(())
}
fn snapshot_id(
    actor: &str,
    scope: &Scope,
    source: &SourceRef,
    version: &str,
    hash: &str,
) -> String {
    sha256_hex(
        serde_json::json!([
            PROFILE,
            actor,
            scope.kind,
            scope.id,
            source.kind,
            source.reference,
            version,
            hash
        ])
        .to_string()
        .as_bytes(),
    )
}

pub fn register(conn: &Connection, request: &RegisterRequest) -> Result<RegisterResponse, String> {
    let actor = request.actor_id.as_deref().unwrap_or("local");
    validate_context(actor, &request.scope)?;
    crate::write::validate_source(&request.source)?;
    if request.schema_version != 1
        || request.source.locator.is_some()
        || !valid_identifier(&request.version)
        || request.text.trim().is_empty()
        || request.text.len() > MAX_SOURCE_BYTES
    {
        return Err("source_snapshot_invalid".into());
    }
    let hash = sha256_hex(request.text.as_bytes());
    let id = snapshot_id(
        actor,
        &request.scope,
        &request.source,
        &request.version,
        &hash,
    );
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let previous:Option<(String,String,String,String)>=tx.query_row(
        "SELECT snapshot_id,text_sha256,source_text,registered_at FROM source_snapshots WHERE actor_id=?1 AND scope_kind=?2 AND scope_id=?3 AND source_kind=?4 AND source_ref=?5 AND version=?6",
        params![actor,request.scope.kind,request.scope.id,request.source.kind,request.source.reference,request.version],
        |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(|e|e.to_string())?;
    let replayed = previous.is_some();
    let timestamp = if let Some((old_id, old_hash, old_text, time)) = previous {
        if old_id != id || old_hash != hash || old_text != request.text {
            return Err("source_version_conflict".into());
        }
        time
    } else {
        let time = crate::repo::utc_now(&tx)?;
        tx.execute("INSERT INTO source_snapshots(snapshot_id,actor_id,scope_kind,scope_id,source_kind,source_ref,version,source_text,text_sha256,registered_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![id,actor,request.scope.kind,request.scope.id,request.source.kind,request.source.reference,request.version,request.text,hash,time]).map_err(|e|e.to_string())?;
        time
    };
    tx.commit().map_err(|e| e.to_string())?;
    Ok(RegisterResponse {
        snapshot_id: id,
        version: request.version.clone(),
        text_sha256: hash,
        registered_at: timestamp,
        replayed,
        trust_basis: "operator_registered_snapshot",
    })
}

struct Memory {
    candidate: Candidate,
    original: String,
}
fn memory(
    conn: &Connection,
    actor: &str,
    scope: &Scope,
    id: &str,
    revision: i64,
) -> Result<Memory, String> {
    conn.query_row("SELECT r.summary,r.original_text,r.state,r.conditions,r.source_kind,r.source_ref,r.source_locator FROM memories m JOIN memory_revisions r ON r.memory_id=m.id AND r.revision=m.current_revision WHERE m.actor_id=?1 AND m.scope_kind=?2 AND m.scope_id=?3 AND m.id=?4 AND r.revision=?5",params![actor,scope.kind,scope.id,id,revision],|r|{
        let kind:Option<String>=r.get(4)?;let reference:Option<String>=r.get(5)?;let locator:Option<String>=r.get(6)?;
        let source=match(kind,reference){(Some(kind),Some(reference))=>Some(SourceRef{kind,reference,locator}),(None,None) if locator.is_none()=>None,_=>return Err(rusqlite::Error::InvalidQuery)};
        Ok(Memory{candidate:Candidate{memory_id:id.into(),revision,summary:r.get(0)?,state:r.get(2)?,conditions:r.get(3)?,source,bm25:0.0},original:r.get(1)?})
    }).map_err(|_|"source_memory_unavailable".into())
}

pub fn verify(conn: &Connection, request: &VerifyRequest) -> Result<SourceCheck, String> {
    let actor = request.actor_id.as_deref().unwrap_or("local");
    validate_context(actor, &request.scope)?;
    if request.schema_version != 1 || !valid_identifier(&request.memory_id) || request.revision < 1
    {
        return Err("source_verify_invalid".into());
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let m = memory(
        &tx,
        actor,
        &request.scope,
        &request.memory_id,
        request.revision,
    )?;
    let checked = check(&tx, actor, &request.scope, &m)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(checked)
}

/// One consistent local snapshot and one connection; receipts are committed before return.
pub fn check_candidates(
    conn: &Connection,
    actor: &str,
    scope: &Scope,
    candidates: &[Candidate],
) -> Result<Vec<SourceCheck>, String> {
    validate_context(actor, scope)?;
    if candidates.len() > 20 {
        return Err("source_candidate_limit".into());
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let mut checks = Vec::new();
    for candidate in candidates {
        let m = memory(&tx, actor, scope, &candidate.memory_id, candidate.revision)?;
        if m.candidate.summary != candidate.summary
            || m.candidate.source != candidate.source
            || m.candidate.conditions != candidate.conditions
        {
            return Err("source_candidate_changed".into());
        }
        checks.push(check(&tx, actor, scope, &m)?);
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(checks)
}

struct Snapshot {
    id: String,
    version: String,
    hash: String,
    text: Option<String>,
}
fn check(conn: &Connection, actor: &str, scope: &Scope, m: &Memory) -> Result<SourceCheck, String> {
    let mut result = SourceCheck {
        check_id: String::new(),
        memory_id: m.candidate.memory_id.clone(),
        revision: m.candidate.revision,
        source: m.candidate.source.clone(),
        snapshot_id: None,
        snapshot_version: None,
        snapshot_sha256: None,
        observed_snapshot_sha256: None,
        original_sha256: sha256_hex(m.original.as_bytes()),
        start_byte: None,
        end_byte: None,
        status: "not_provided".into(),
        reason_code: "source_not_provided".into(),
        checked_at: String::new(),
        profile: PROFILE.into(),
        semantic_support: "not_assessed".into(),
    };
    if let Some(source) = m.candidate.source.as_ref() {
        let evidence = check_evidence(conn, scope, actor, source, &m.original)?;
        result.snapshot_id = evidence.snapshot_id;
        result.snapshot_version = evidence.snapshot_version;
        result.snapshot_sha256 = evidence.snapshot_sha256;
        result.observed_snapshot_sha256 = evidence.observed_snapshot_sha256;
        result.start_byte = evidence.start_byte;
        result.end_byte = evidence.end_byte;
        result.status = evidence.status;
        result.reason_code = evidence.reason_code;
    }
    // The stable key includes all observed evidence and outcome, excluding wall time.
    result.check_id = sha256_hex(
        serde_json::to_vec(&result)
            .map_err(|e| e.to_string())?
            .as_slice(),
    );
    let prior: Option<(String, String)> = conn
        .query_row(
            "SELECT report_json,checked_at FROM source_verification_checks WHERE check_id=?1",
            [&result.check_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some((report, time)) = prior {
        let mut expected = result.clone();
        expected.checked_at = time.clone();
        if serde_json::to_string(&expected).map_err(|e| e.to_string())? != report {
            return Err("source_receipt_integrity_mismatch".into());
        }
        result.checked_at = time;
    } else {
        result.checked_at = crate::repo::utc_now(conn)?;
        conn.execute("INSERT INTO source_verification_checks(check_id,memory_id,revision,snapshot_id,status,reason_code,report_json,checked_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",params![result.check_id,result.memory_id,result.revision,result.snapshot_id,result.status,result.reason_code,serde_json::to_string(&result).map_err(|e|e.to_string())?,result.checked_at]).map_err(|e|e.to_string())?;
    }
    Ok(result)
}

type Located = (&'static str, &'static str, Option<(usize, usize)>);
fn locate(text: &str, original: &str, locator: Option<&str>) -> Located {
    if original.is_empty() {
        return ("declared_unverified", "original_empty", None);
    }
    let range = if let Some(locator) = locator {
        let Some(bounds) = locator.strip_prefix("utf8:") else {
            return ("declared_unverified", "locator_unsupported", None);
        };
        let Some((start, end)) = bounds.split_once(':') else {
            return ("rejected", "source_range_invalid", None);
        };
        let (Ok(start), Ok(end)) = (start.parse::<usize>(), end.parse::<usize>()) else {
            return ("rejected", "source_range_invalid", None);
        };
        (start, end)
    } else {
        let Some(start) = text.find(original) else {
            return ("rejected", "source_quote_not_found", None);
        };
        let next = start + original.chars().next().unwrap().len_utf8();
        if text[next..].contains(original) {
            return ("declared_unverified", "source_quote_ambiguous", None);
        }
        (start, start + original.len())
    };
    if range.0 >= range.1 || text.get(range.0..range.1).is_none() {
        return ("rejected", "source_range_invalid", None);
    }
    if text.get(range.0..range.1) != Some(original) {
        return ("rejected", "source_quote_mismatch", None);
    }
    ("verified_literal", "literal_match", Some(range))
}

/// Literal evidence for a proposed write. No transaction or persistence is opened here.
/// Callers can pass their existing Transaction through its Connection deref.
#[derive(Debug, Clone, Serialize)]
pub struct LiteralEvidence {
    pub snapshot_id: Option<String>,
    pub snapshot_version: Option<String>,
    pub snapshot_sha256: Option<String>,
    pub observed_snapshot_sha256: Option<String>,
    pub original_sha256: String,
    pub start_byte: Option<usize>,
    pub end_byte: Option<usize>,
    pub status: String,
    pub reason_code: String,
    pub checked_at: String,
    pub profile: String,
    pub semantic_support: String,
}

pub fn check_evidence(
    conn: &Connection,
    scope: &Scope,
    actor: &str,
    source: &SourceRef,
    original: &str,
) -> Result<LiteralEvidence, String> {
    validate_context(actor, scope)?;
    if original.len() > MAX_SOURCE_BYTES {
        return Err("source_original_limit".into());
    }
    let mut result = LiteralEvidence {
        snapshot_id: None,
        snapshot_version: None,
        snapshot_sha256: None,
        observed_snapshot_sha256: None,
        original_sha256: sha256_hex(original.as_bytes()),
        start_byte: None,
        end_byte: None,
        status: "declared_unverified".into(),
        reason_code: "source_not_registered".into(),
        checked_at: crate::repo::utc_now(conn)?,
        profile: PROFILE.into(),
        semantic_support: "not_assessed".into(),
    };
    crate::write::validate_source(source)?;
    result.status = "declared_unverified".into();
    result.reason_code = "source_not_registered".into();
    let snapshot=conn.query_row("SELECT snapshot_id,version,text_sha256,CASE WHEN length(CAST(source_text AS BLOB)) <= ?6 THEN source_text ELSE NULL END FROM source_snapshots WHERE actor_id=?1 AND scope_kind=?2 AND scope_id=?3 AND source_kind=?4 AND source_ref=?5 ORDER BY sequence DESC LIMIT 1",params![actor,scope.kind,scope.id,source.kind,source.reference,MAX_SOURCE_BYTES as i64],|r|Ok(Snapshot{id:r.get(0)?,version:r.get(1)?,hash:r.get(2)?,text:r.get(3)?})).optional().map_err(|e|e.to_string())?;
    if let Some(snapshot) = snapshot {
        result.snapshot_id = Some(snapshot.id.clone());
        result.snapshot_version = Some(snapshot.version.clone());
        result.snapshot_sha256 = Some(snapshot.hash.clone());
        result.status = "rejected".into();
        result.reason_code = "snapshot_integrity_mismatch".into();
        if let Some(text) = snapshot.text {
            let observed = sha256_hex(text.as_bytes());
            result.observed_snapshot_sha256 = Some(observed.clone());
            if observed == snapshot.hash
                && snapshot_id(actor, scope, source, &snapshot.version, &observed) == snapshot.id
            {
                let (status, reason, range) = locate(&text, original, source.locator.as_deref());
                result.status = status.into();
                result.reason_code = reason.into();
                if let Some((start, end)) = range {
                    result.start_byte = Some(start);
                    result.end_byte = Some(end);
                }
            }
        }
    }
    Ok(result)
}
