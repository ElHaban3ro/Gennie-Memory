//! Local, declared identity registry. Labels never establish identity on their own.
use crate::{model::SourceRef, service::valid_identifier, write::Scope};
use gennie_memory_kernel::digest::sha256_hex;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EntityRequest {
    pub schema_version: u32,
    pub operation_id: String,
    pub scope: Scope,
    #[serde(default)]
    pub actor_id: Option<String>,
    pub kind: String,
    pub label: String,
    pub source: SourceRef,
    #[serde(default)]
    pub entity_id: Option<String>,
    #[serde(default)]
    pub create_new: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityCandidate {
    pub entity_id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityResponse {
    pub schema_version: u32,
    pub status: String,
    pub entity_id: Option<String>,
    pub candidates: Vec<EntityCandidate>,
    pub replayed: bool,
    pub provenance_status: String,
}

fn normalize(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub(crate) fn register_legacy(
    conn: &Connection,
    actor: &str,
    kind: &str,
    scope: &str,
    id: &str,
) -> Result<(), String> {
    conn.execute("INSERT OR IGNORE INTO entities(actor_id,scope_kind,scope_id,entity_id,kind,label) VALUES (?1,?2,?3,?4,'subject',?4)",params![actor,kind,scope,id]).map_err(|e|e.to_string())?;
    conn.execute("INSERT OR IGNORE INTO entity_aliases(actor_id,scope_kind,scope_id,entity_id,normalized_label,label) VALUES (?1,?2,?3,?4,?5,?4)",params![actor,kind,scope,id,normalize(id)]).map_err(|e|e.to_string())?;
    Ok(())
}

pub(crate) fn exists(
    conn: &Connection,
    actor: &str,
    scope: &Scope,
    id: &str,
) -> Result<bool, String> {
    conn.query_row("SELECT EXISTS(SELECT 1 FROM entities WHERE actor_id=?1 AND scope_kind=?2 AND scope_id=?3 AND entity_id=?4)", params![actor,scope.kind,scope.id,id], |r|r.get(0)).map_err(|e|e.to_string())
}

pub fn resolve(conn: &mut Connection, request: &EntityRequest) -> Result<EntityResponse, String> {
    if request.schema_version != 1
        || !crate::write::valid_operation_id(&request.operation_id)
        || !matches!(
            request.scope.kind.as_str(),
            "user" | "agent" | "project" | "conversation"
        )
        || !valid_identifier(&request.scope.id)
        || request
            .actor_id
            .as_deref()
            .is_some_and(|v| !valid_identifier(v))
        || !matches!(
            request.kind.as_str(),
            "person" | "place" | "event" | "company" | "subject"
        )
        || request.label.trim().is_empty()
        || request.label.chars().count() > 200
        || request.label.chars().any(char::is_control)
        || request
            .entity_id
            .as_deref()
            .is_some_and(|v| !valid_identifier(v))
        || (request.entity_id.is_some() && request.create_new)
    {
        return Err(
            "entity_request_invalid: identidad, nombre, ámbito u operación inválidos".into(),
        );
    }
    crate::write::validate_source(&request.source)?;
    let actor = request.actor_id.as_deref().unwrap_or("local");
    let mut canonical = request.clone();
    canonical.actor_id = Some(actor.to_owned());
    canonical.operation_id = request.operation_id.to_ascii_lowercase();
    let hash = sha256_hex(&serde_json::to_vec(&canonical).map_err(|e| e.to_string())?);
    let op = &canonical.operation_id;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let previous: Option<(String, String)> = tx
        .query_row(
            "SELECT request_sha256,response_json FROM entity_operations WHERE operation_id=?1",
            [op],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some((stored, response)) = previous {
        if stored != hash {
            return Err("idempotency_conflict: operación de entidad con otro contenido".into());
        }
        let mut response: EntityResponse =
            serde_json::from_str(&response).map_err(|e| e.to_string())?;
        response.replayed = true;
        return Ok(response);
    }
    let collision: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM memory_write_operations WHERE operation_id=?1)",
            [op],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if collision {
        return Err("idempotency_conflict: operación ya usada para escritura".into());
    }
    let mut candidates = Vec::new();
    if request.entity_id.is_none() && !request.create_new {
        let mut stmt = tx.prepare("SELECT e.entity_id,e.kind,e.label FROM entities e JOIN entity_aliases a USING(actor_id,scope_kind,scope_id,entity_id) WHERE e.actor_id=?1 AND e.scope_kind=?2 AND e.scope_id=?3 AND e.kind=?4 AND a.normalized_label=?5 ORDER BY e.entity_id LIMIT 101").map_err(|e|e.to_string())?;
        candidates = stmt
            .query_map(
                params![
                    actor,
                    request.scope.kind,
                    request.scope.id,
                    request.kind,
                    normalize(&request.label)
                ],
                |r| {
                    Ok(EntityCandidate {
                        entity_id: r.get(0)?,
                        kind: r.get(1)?,
                        label: r.get(2)?,
                    })
                },
            )
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if candidates.len() > 100 {
            return Err("entity_candidates_limit: acota la identidad mediante entity_id".into());
        }
    }
    let (status, id) = if let Some(id) = &request.entity_id {
        let kind: Option<String> = tx.query_row("SELECT kind FROM entities WHERE actor_id=?1 AND scope_kind=?2 AND scope_id=?3 AND entity_id=?4",params![actor,request.scope.kind,request.scope.id,id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        if kind.as_deref() != Some(&request.kind) {
            return Err(
                "entity_not_available: identidad o tipo no disponible en actor y ámbito".into(),
            );
        }
        ("resolved", Some(id.clone()))
    } else if !candidates.is_empty() {
        ("ambiguous", None)
    } else {
        let id = format!("ent-{}", &hash[..32]);
        tx.execute("INSERT INTO entities(actor_id,scope_kind,scope_id,entity_id,kind,label) VALUES (?1,?2,?3,?4,?5,?6)",params![actor,request.scope.kind,request.scope.id,id,request.kind,request.label]).map_err(|e|e.to_string())?;
        ("created", Some(id))
    };
    if let Some(id) = &id {
        tx.execute("INSERT OR IGNORE INTO entity_aliases(actor_id,scope_kind,scope_id,entity_id,normalized_label,label) VALUES (?1,?2,?3,?4,?5,?6)",params![actor,request.scope.kind,request.scope.id,id,normalize(&request.label),request.label]).map_err(|e|e.to_string())?;
    }
    let response = EntityResponse {
        schema_version: 1,
        status: status.into(),
        entity_id: id.clone(),
        candidates,
        replayed: false,
        provenance_status: "declared_unverified".into(),
    };
    tx.execute("INSERT INTO entity_operations(operation_id,request_sha256,response_json,committed_at) VALUES (?1,?2,?3,?4)",params![op,hash,serde_json::to_string(&response).map_err(|e|e.to_string())?,crate::repo::utc_now(&tx)?]).map_err(|e|e.to_string())?;
    if let Some(id) = id {
        tx.execute("INSERT INTO entity_evidence(operation_id,actor_id,scope_kind,scope_id,entity_id,source_json) VALUES (?1,?2,?3,?4,?5,?6)",params![op,actor,request.scope.kind,request.scope.id,id,serde_json::to_string(&request.source).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(response)
}
