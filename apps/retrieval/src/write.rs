//! Bounded, local create path for the v1 memory-write contract.
use crate::model::SourceRef;
use gennie_memory_kernel::digest::sha256_hex;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

const MAX_BATCH: usize = 2500;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WriteRequest {
    pub schema_version: u32,
    pub operation_id: String,
    pub action: String,
    #[serde(default)]
    pub memory_id: Option<String>,
    #[serde(default)]
    pub expected_revision: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    pub scope: Scope,
    pub memory: MemoryInput,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryInput {
    pub original_text: String,
    pub summary: String,
    #[serde(rename = "type")]
    pub memory_type: String,
    pub source: SourceRef,
    #[serde(default)]
    pub conditions: Option<String>,
    #[serde(default)]
    pub valid_from: Option<String>,
    #[serde(default)]
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WriteResponse {
    pub schema_version: u32,
    pub operation_id: String,
    pub status: &'static str,
    pub outcome: &'static str,
    pub memory_id: String,
    pub revision: i64,
    pub replayed: bool,
    pub evidence_added: bool,
    pub provenance_status: &'static str,
    pub indexing: &'static str,
    pub warnings: Vec<String>,
}

pub fn write_memory(
    conn: &mut Connection,
    request: &WriteRequest,
) -> Result<WriteResponse, String> {
    let tx = conn
        .transaction()
        .map_err(|_| "no se pudo iniciar la transacción de escritura")?;
    let collision: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM reconciliation_operations WHERE operation_id=?1)",
            [request.operation_id.to_ascii_lowercase()],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if collision {
        return Err("idempotency_conflict: UUID ya usado en reconciliación".into());
    }
    let result = write_in_transaction(&tx, request)?;
    tx.commit()
        .map_err(|_| "no se pudo confirmar la escritura")?;
    Ok(result)
}

/// Shares the caller's atomic write boundary; never commits independently.
pub(crate) fn write_in_transaction(
    tx: &rusqlite::Transaction<'_>,
    request: &WriteRequest,
) -> Result<WriteResponse, String> {
    validate(request)?;
    let erased: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM memory_erased_write_operations WHERE operation_id=?1)",
            [request.operation_id.to_ascii_lowercase()],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if erased {
        return Err(
            "content_deleted: esta operación fue eliminada y no puede recrear contenido".into(),
        );
    }
    let mut canonical_request = request.clone();
    canonical_request.operation_id = request.operation_id.to_ascii_lowercase();
    let normalized =
        serde_json::to_vec(&canonical_request).map_err(|_| "no se pudo normalizar la solicitud")?;
    let request_sha256 = sha256_hex(&normalized);
    let operation_id = request.operation_id.to_ascii_lowercase();

    let replay: Option<(String, String, i64, String, bool)> = tx
        .query_row(
            "SELECT request_sha256, memory_id, revision, outcome, evidence_added FROM memory_write_operations WHERE operation_id = ?1",
            [&operation_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i64>(4)? != 0)),
        )
        .optional()
        .map_err(|_| "no se pudo consultar la idempotencia de escritura")?;
    if let Some((existing_hash, memory_id, revision, outcome, evidence_added)) = replay {
        if existing_hash != request_sha256 {
            return Err(
                "idempotency_conflict: operation_id ya corresponde a otro contenido".into(),
            );
        }
        return Ok(response(
            &operation_id,
            memory_id,
            revision,
            true,
            &outcome,
            evidence_added,
        ));
    }

    let collision: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM entity_operations WHERE operation_id=?1)",
            [&operation_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if collision {
        return Err("idempotency_conflict: operación ya usada para entidad".into());
    }
    let mut effective = request.clone();
    if request.action == "revise" {
        let identity: Option<(String,Option<String>)> = tx.query_row("SELECT actor_id,entity_id FROM memories WHERE id=?1 AND scope_kind=?2 AND scope_id=?3",params![request.memory_id,request.scope.kind,request.scope.id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
        let Some((actor, entity)) = identity else {
            return Err("memory_not_found: recuerdo no disponible en el ámbito".into());
        };
        if actor != "local" {
            return Err("memory_not_available: recuerdo de otro actor".into());
        }
        if request.entity_id.is_some() && request.entity_id != entity {
            return Err("entity_mismatch: no se puede cambiar la entidad al revisar".into());
        }
        effective.entity_id = entity;
    }
    if let Some(id) = &effective.entity_id {
        if !crate::entity::exists(&tx, "local", &request.scope, id)? {
            return Err(
                "entity_not_available: resuelve la entidad en este ámbito antes de escribir".into(),
            );
        }
    }
    let request = &effective;
    let committed_at = tx
        .query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ', 'now')", [], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|_| "no se pudo obtener la hora UTC")?;
    let fingerprint = content_fingerprint(request);
    backfill_scope_keys(&tx, &request.scope.kind, &request.scope.id)?;
    if request.action == "revise" {
        return revise_memory(
            tx,
            request,
            &operation_id,
            &request_sha256,
            &fingerprint,
            &committed_at,
        );
    }
    let existing: Option<(String, i64)> = tx.query_row(
        "SELECT memory_id, revision FROM memory_content_keys WHERE scope_kind=?1 AND scope_id=?2 AND content_sha256=?3",
        params![request.scope.kind, request.scope.id, fingerprint],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional().map_err(|_| "no se pudo comprobar la huella de contenido")?;
    let (memory_id, revision, outcome) = if let Some((memory_id, revision)) = existing {
        let active: bool = tx
            .query_row(
                "SELECT lifecycle_state='active' FROM memories WHERE id=?1",
                [&memory_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if !active {
            return Err(
                "memory_withdrawn: restaura explícitamente el recuerdo antes de reutilizarlo"
                    .into(),
            );
        }
        (memory_id, revision, "deduplicated")
    } else {
        let memory_id = format!("mem-{}", &request_sha256[..32]);
        tx.execute(
            "INSERT INTO memories(id, scope_kind, scope_id, current_revision, entity_id) VALUES (?1, ?2, ?3, 1, ?4)",
            params![memory_id, request.scope.kind, request.scope.id, request.entity_id],
        )
        .map_err(|_| "no se pudo crear la identidad del recuerdo".to_owned())?;
        tx.execute(
            "INSERT INTO memory_revisions(memory_id, revision, memory_type, original_text, summary, state, valid_from, valid_until, conditions, source_kind, source_ref, source_locator)
             VALUES (?1, 1, ?2, ?3, ?4, 'active', ?5, ?6, ?7, ?8, ?9, ?10)",
            params![memory_id, request.memory.memory_type, request.memory.original_text,
                request.memory.summary, request.memory.valid_from, request.memory.valid_until,
                request.memory.conditions, request.memory.source.kind,
                request.memory.source.reference, request.memory.source.locator],
        ).map_err(|_| "no se pudo guardar la revisión del recuerdo")?;
        tx.execute(
            "INSERT INTO search_documents(memory_id, revision, original_text, summary) VALUES (?1, 1, ?2, ?3)",
            params![memory_id, request.memory.original_text, request.memory.summary],
        ).map_err(|_| "no se pudo indexar el recuerdo para búsqueda textual")?;
        tx.execute(
            "INSERT INTO memory_content_keys(scope_kind, scope_id, content_sha256, memory_id, revision, identity_version) VALUES (?1, ?2, ?3, ?4, 1, 1)",
            params![request.scope.kind, request.scope.id, fingerprint, memory_id],
        ).map_err(|_| "no se pudo registrar la huella exacta del recuerdo")?;
        (memory_id, 1, "created")
    };
    let evidence_added = add_source_evidence(&tx, request, &memory_id, revision, &committed_at)?;
    tx.execute(
        "INSERT INTO memory_write_operations(operation_id, request_sha256, memory_id, revision, committed_at, outcome, evidence_added) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![operation_id, request_sha256, memory_id, revision, committed_at, outcome, evidence_added as i64],
    )
    .map_err(|_| "no se pudo registrar la operación de escritura")?;
    Ok(response(
        &operation_id,
        memory_id,
        revision,
        false,
        outcome,
        evidence_added,
    ))
}

pub fn write_batch(
    conn: &mut Connection,
    requests: &[WriteRequest],
) -> Result<Vec<WriteResponse>, String> {
    if requests.is_empty() || requests.len() > MAX_BATCH {
        return Err(format!(
            "el lote debe contener entre 1 y {MAX_BATCH} operaciones"
        ));
    }
    let mut results = Vec::with_capacity(requests.len());
    for (index, request) in requests.iter().enumerate() {
        results.push(
            write_memory(conn, request)
                .map_err(|error| format!("operación {}: {error}", index + 1))?,
        );
    }
    Ok(results)
}

fn response(
    operation_id: &str,
    memory_id: String,
    revision: i64,
    replayed: bool,
    outcome: &str,
    evidence_added: bool,
) -> WriteResponse {
    WriteResponse {
        schema_version: 1,
        operation_id: operation_id.to_owned(),
        status: "committed",
        outcome: match outcome {
            "deduplicated" => "deduplicated",
            "revised" => "revised",
            _ => "created",
        },
        memory_id,
        revision,
        replayed,
        evidence_added,
        provenance_status: "declared_unverified",
        indexing: "fts5_committed",
        warnings: Vec::new(),
    }
}

#[derive(Serialize)]
struct ContentFingerprint<'a> {
    scope_kind: &'a str,
    scope_id: &'a str,
    memory_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actor_id: Option<&'a str>,
    original_text: String,
    summary: String,
    conditions: Option<String>,
    valid_from: Option<&'a str>,
    valid_until: Option<&'a str>,
}

fn normalized_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn content_fingerprint(request: &WriteRequest) -> String {
    let content = ContentFingerprint {
        scope_kind: &request.scope.kind,
        scope_id: &request.scope.id,
        memory_type: &request.memory.memory_type,
        entity_id: request.entity_id.as_deref(),
        actor_id: None,
        original_text: normalized_text(&request.memory.original_text),
        summary: normalized_text(&request.memory.summary),
        conditions: request.memory.conditions.as_deref().map(normalized_text),
        valid_from: request.memory.valid_from.as_deref(),
        valid_until: request.memory.valid_until.as_deref(),
    };
    sha256_hex(&serde_json::to_vec(&content).expect("fingerprint struct is serializable"))
}

fn fingerprint_from_row(
    actor_id: &str,
    entity_id: Option<&str>,
    scope_kind: &str,
    scope_id: &str,
    memory_type: &str,
    original_text: &str,
    summary: &str,
    conditions: Option<&str>,
    valid_from: Option<&str>,
    valid_until: Option<&str>,
) -> String {
    let content = ContentFingerprint {
        scope_kind,
        scope_id,
        memory_type,
        entity_id,
        actor_id: (actor_id != "local").then_some(actor_id),
        original_text: normalized_text(original_text),
        summary: normalized_text(summary),
        conditions: conditions.map(normalized_text),
        valid_from,
        valid_until,
    };
    sha256_hex(&serde_json::to_vec(&content).expect("fingerprint struct is serializable"))
}

pub(crate) fn backfill_scope_keys(
    tx: &rusqlite::Transaction<'_>,
    scope_kind: &str,
    scope_id: &str,
) -> Result<(), String> {
    // Refresh derived keys after upgrades/imports without rehashing every known
    // memory on each write. Identity is immutable within a memory.
    tx.execute(
        "DELETE FROM memory_content_keys WHERE scope_kind=?1 AND scope_id=?2
         AND (identity_version=0 OR NOT EXISTS (
             SELECT 1 FROM memories m JOIN memory_revisions r ON r.memory_id=m.id AND r.revision=m.current_revision
             WHERE m.id=memory_content_keys.memory_id AND m.current_revision=memory_content_keys.revision AND r.state='active'))",
        params![scope_kind, scope_id],
    )
    .map_err(|e| e.to_string())?;
    let mut stmt = tx.prepare(
        "SELECT m.id, m.current_revision, r.memory_type, r.original_text, r.summary, r.conditions, r.valid_from, r.valid_until, m.actor_id, m.entity_id
         FROM memories m JOIN memory_revisions r ON r.memory_id=m.id AND r.revision=m.current_revision
         WHERE m.scope_kind=?1 AND m.scope_id=?2 AND r.state='active'
           AND NOT EXISTS (SELECT 1 FROM memory_content_keys k WHERE k.memory_id=m.id AND k.revision=m.current_revision AND k.identity_version=1)
         ORDER BY m.id",
    ).map_err(|_| "no se pudieron leer recuerdos del ámbito para deduplicar")?;
    let rows = stmt
        .query_map(params![scope_kind, scope_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })
        .map_err(|_| "no se pudieron recorrer recuerdos del ámbito")?;
    for row in rows {
        let (memory_id, revision, kind, original, summary, conditions, from, until, actor, entity) =
            row.map_err(|_| "no se pudo leer una memoria existente")?;
        let fingerprint = fingerprint_from_row(
            &actor,
            entity.as_deref(),
            scope_kind,
            scope_id,
            &kind,
            &original,
            &summary,
            conditions.as_deref(),
            from.as_deref(),
            until.as_deref(),
        );
        tx.execute(
            "INSERT OR IGNORE INTO memory_content_keys(scope_kind,scope_id,content_sha256,memory_id,revision,identity_version) VALUES (?1,?2,?3,?4,?5,1)",
            params![scope_kind, scope_id, fingerprint, memory_id, revision],
        ).map_err(|_| "no se pudo indexar una huella existente")?;
    }
    Ok(())
}

pub(crate) fn add_source_evidence(
    tx: &rusqlite::Transaction<'_>,
    request: &WriteRequest,
    memory_id: &str,
    revision: i64,
    recorded_at: &str,
) -> Result<bool, String> {
    let source = &request.memory.source;
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM memory_source_evidence WHERE memory_id=?1 AND revision=?2 AND source_kind=?3 AND source_ref=?4 AND source_locator IS ?5 AND evidence_text=?6)",
        params![memory_id, revision, source.kind, source.reference, source.locator, request.memory.original_text],
        |row| row.get(0),
    ).map_err(|_| "no se pudo comprobar la evidencia de procedencia")?;
    if exists {
        return Ok(false);
    }
    tx.execute(
        "INSERT INTO memory_source_evidence(memory_id,revision,source_kind,source_ref,source_locator,evidence_text,verification_status,recorded_at) VALUES (?1,?2,?3,?4,?5,?6,'declared_unverified',?7)",
        params![memory_id, revision, source.kind, source.reference, source.locator, request.memory.original_text, recorded_at],
    ).map_err(|_| "no se pudo conservar la evidencia de procedencia")?;
    Ok(true)
}

fn revise_memory(
    tx: &rusqlite::Transaction<'_>,
    request: &WriteRequest,
    operation_id: &str,
    request_sha256: &str,
    fingerprint: &str,
    committed_at: &str,
) -> Result<WriteResponse, String> {
    let memory_id = request.memory_id.as_deref().expect("validated memory id");
    let expected_revision = request
        .expected_revision
        .expect("validated expected revision");
    let current: Option<(String, String, i64, String, String, String, Option<String>, Option<String>)> = tx.query_row(
        "SELECT m.scope_kind,m.scope_id,m.current_revision,r.memory_type,r.original_text,r.summary,r.conditions,r.valid_from FROM memories m JOIN memory_revisions r ON r.memory_id=m.id AND r.revision=m.current_revision WHERE m.id=?1",
        [memory_id],
        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?)),
    ).optional().map_err(|_| "no se pudo leer la revisión que se quiere editar")?;
    let Some((
        scope_kind,
        scope_id,
        current_revision,
        old_type,
        old_original,
        old_summary,
        old_conditions,
        old_from,
    )) = current
    else {
        return Err("memory_not_found: no existe el recuerdo que se quiere editar".into());
    };
    if scope_kind != request.scope.kind || scope_id != request.scope.id {
        return Err("scope_mismatch: no se puede mover el recuerdo a otro ámbito".into());
    }
    if current_revision != expected_revision {
        return Err(format!(
            "revision_conflict: se esperaba {expected_revision} y la revisión vigente es {current_revision}"
        ));
    }
    let old_until: Option<String> = tx
        .query_row(
            "SELECT valid_until FROM memory_revisions WHERE memory_id=?1 AND revision=?2",
            params![memory_id, current_revision],
            |row| row.get(0),
        )
        .map_err(|_| "no se pudo leer la vigencia de la revisión actual")?;
    let old_fingerprint = fingerprint_from_row(
        "local",
        request.entity_id.as_deref(),
        &scope_kind,
        &scope_id,
        &old_type,
        &old_original,
        &old_summary,
        old_conditions.as_deref(),
        old_from.as_deref(),
        old_until.as_deref(),
    );
    if old_fingerprint == fingerprint {
        return Err("no_change: el contenido nuevo es idéntico a la revisión vigente".into());
    }
    let duplicate: Option<String> = tx.query_row(
        "SELECT memory_id FROM memory_content_keys WHERE scope_kind=?1 AND scope_id=?2 AND content_sha256=?3",
        params![scope_kind,scope_id,fingerprint],
        |row| row.get(0),
    ).optional().map_err(|_| "no se pudo comprobar si el contenido ya existe")?;
    if duplicate.is_some_and(|existing| existing != memory_id) {
        return Err("exact_duplicate_requires_review: el contenido coincide con otro recuerdo; no se fusionó ni se sustituyó".into());
    }
    let next_revision = current_revision + 1;
    tx.execute(
        "INSERT INTO memory_revisions(memory_id,revision,memory_type,original_text,summary,state,valid_from,valid_until,conditions,source_kind,source_ref,source_locator) VALUES (?1,?2,?3,?4,?5,'active',?6,?7,?8,?9,?10,?11)",
        params![memory_id,next_revision,request.memory.memory_type,request.memory.original_text,request.memory.summary,request.memory.valid_from,request.memory.valid_until,request.memory.conditions,request.memory.source.kind,request.memory.source.reference,request.memory.source.locator],
    ).map_err(|_| "no se pudo guardar la nueva revisión")?;
    tx.execute(
        "INSERT INTO search_documents(memory_id,revision,original_text,summary) VALUES (?1,?2,?3,?4)",
        params![memory_id,next_revision,request.memory.original_text,request.memory.summary],
    ).map_err(|_| "no se pudo indexar la nueva revisión")?;
    tx.execute(
        "UPDATE memories SET current_revision=?2 WHERE id=?1",
        params![memory_id, next_revision],
    )
    .map_err(|_| "no se pudo activar la nueva revisión")?;
    tx.execute(
        "DELETE FROM memory_content_keys WHERE scope_kind=?1 AND scope_id=?2 AND content_sha256=?3 AND memory_id=?4",
        params![scope_kind,scope_id,old_fingerprint,memory_id],
    ).map_err(|_| "no se pudo retirar la huella anterior")?;
    tx.execute(
        "INSERT INTO memory_content_keys(scope_kind,scope_id,content_sha256,memory_id,revision,identity_version) VALUES (?1,?2,?3,?4,?5,1)",
        params![scope_kind,scope_id,fingerprint,memory_id,next_revision],
    ).map_err(|_| "no se pudo indexar el contenido revisado")?;
    let evidence_added = add_source_evidence(&tx, request, memory_id, next_revision, committed_at)?;
    tx.execute(
        "INSERT INTO memory_write_operations(operation_id,request_sha256,memory_id,revision,committed_at,outcome,evidence_added) VALUES (?1,?2,?3,?4,?5,'revised',?6)",
        params![operation_id,request_sha256,memory_id,next_revision,committed_at,evidence_added as i64],
    ).map_err(|_| "no se pudo registrar la operación de revisión")?;
    Ok(response(
        operation_id,
        memory_id.to_owned(),
        next_revision,
        false,
        "revised",
        evidence_added,
    ))
}

pub(crate) fn validate(request: &WriteRequest) -> Result<(), String> {
    if request.schema_version != 1 {
        return Err("schema_version debe ser 1".into());
    }
    if !matches!(request.action.as_str(), "create" | "revise") {
        return Err("mem_write admite action=create o action=revise".into());
    }
    if request.action == "create"
        && (request.memory_id.is_some() || request.expected_revision.is_some())
    {
        return Err("create no admite memory_id ni expected_revision".into());
    }
    if request.action == "revise"
        && (request
            .memory_id
            .as_ref()
            .is_none_or(|id| id.trim().is_empty() || id.len() > 128)
            || request
                .expected_revision
                .is_none_or(|revision| revision < 1))
    {
        return Err("revise requiere memory_id y expected_revision positivos".into());
    }
    if !valid_operation_id(&request.operation_id) {
        return Err("operation_id debe ser UUID con variante RFC 4122".into());
    }
    if !matches!(
        request.scope.kind.as_str(),
        "user" | "agent" | "project" | "conversation"
    ) || request.scope.id.trim().is_empty()
        || request.scope.id.chars().count() > 128
    {
        return Err("scope debe tener kind admitido e id de 1 a 128 caracteres".into());
    }
    if request
        .entity_id
        .as_deref()
        .is_some_and(|id| !crate::service::valid_identifier(id))
    {
        return Err("entity_id inválido".into());
    }
    let memory = &request.memory;
    if memory.original_text.trim().is_empty() || memory.original_text.chars().count() > 16000 {
        return Err("memory.original_text debe tener entre 1 y 16000 caracteres".into());
    }
    if memory.summary.trim().is_empty() || memory.summary.chars().count() > 1600 {
        return Err("memory.summary debe tener entre 1 y 1600 caracteres".into());
    }
    if !matches!(
        memory.memory_type.as_str(),
        "preference" | "fact" | "decision" | "event" | "procedure"
    ) {
        return Err("memory.type debe ser preference, fact, decision, event o procedure".into());
    }
    validate_source(&memory.source)?;
    if memory
        .conditions
        .as_ref()
        .is_some_and(|s| s.trim().is_empty() || s.chars().count() > 1600)
    {
        return Err("memory.conditions debe ser null o tener hasta 1600 caracteres".into());
    }
    for date in [&memory.valid_from, &memory.valid_until]
        .into_iter()
        .flatten()
    {
        if !crate::service::valid_utc(date) {
            return Err("las fechas deben usar UTC RFC 3339 con milisegundos".into());
        }
    }
    if memory
        .valid_from
        .as_ref()
        .zip(memory.valid_until.as_ref())
        .is_some_and(|(a, b)| b <= a)
    {
        return Err("valid_until debe ser posterior a valid_from".into());
    }
    Ok(())
}

pub(crate) fn valid_operation_id(value: &str) -> bool {
    let id = value.as_bytes();
    id.len() == 36
        && [8, 13, 18, 23].iter().all(|&i| id[i] == b'-')
        && id
            .iter()
            .enumerate()
            .all(|(i, b)| [8, 13, 18, 23].contains(&i) || b.is_ascii_hexdigit())
        && matches!(id[14].to_ascii_lowercase(), b'1'..=b'8')
        && matches!(id[19].to_ascii_lowercase(), b'8' | b'9' | b'a' | b'b')
}
pub(crate) fn validate_source(source: &SourceRef) -> Result<(), String> {
    if !matches!(
        source.kind.as_str(),
        "user" | "document" | "tool" | "agent_discovery"
    ) || source.reference.trim().is_empty()
        || source.reference.chars().count() > 512
        || source.reference.chars().any(char::is_control)
        || source.locator.as_ref().is_some_and(|s| {
            s.trim().is_empty() || s.chars().count() > 256 || s.chars().any(char::is_control)
        })
    {
        return Err("memory.source debe tener kind, ref y locator válidos".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::RetrievalModule;
    use gennie_memory_kernel::db;

    fn db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        db::migrate(&mut conn, &[Box::new(RetrievalModule)]).unwrap();
        conn
    }
    fn request(summary: &str) -> WriteRequest {
        WriteRequest {
            schema_version: 1,
            operation_id: "123e4567-e89b-42d3-a456-426614174000".into(),
            action: "create".into(),
            memory_id: None,
            expected_revision: None,
            entity_id: None,
            scope: Scope {
                kind: "user".into(),
                id: "person-a".into(),
            },
            memory: MemoryInput {
                original_text: "I walked my dogs this morning.".into(),
                summary: summary.into(),
                memory_type: "event".into(),
                source: SourceRef {
                    kind: "document".into(),
                    reference: "locomo.json".into(),
                    locator: Some("hmid=1".into()),
                },
                conditions: None,
                valid_from: None,
                valid_until: None,
            },
        }
    }
    #[test]
    fn create_persists_original_summary_source_and_fts_atomically() {
        let mut conn = db();
        let result = write_memory(&mut conn, &request("The narrator walked their dogs.")).unwrap();
        assert_eq!(result.status, "committed");
        assert!(!result.replayed);
        let row: (String, String, String, String) = conn.query_row(
            "SELECT r.original_text,r.summary,r.source_kind,r.source_locator FROM memory_revisions r WHERE r.memory_id=?1",
            [&result.memory_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)),
        ).unwrap();
        assert_eq!(
            row,
            (
                "I walked my dogs this morning.".into(),
                "The narrator walked their dogs.".into(),
                "document".into(),
                "hmid=1".into()
            )
        );
        let indexed: i64 = conn
            .query_row(
                "SELECT count(*) FROM memory_fts WHERE memory_fts MATCH 'dogs'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(indexed, 1);
    }
    #[test]
    fn repeated_operation_is_idempotent_and_conflicting_reuse_is_rejected() {
        let mut conn = db();
        let first = write_memory(&mut conn, &request("The narrator walked their dogs.")).unwrap();
        let replay = write_memory(&mut conn, &request("The narrator walked their dogs.")).unwrap();
        assert_eq!(first.memory_id, replay.memory_id);
        assert!(replay.replayed);
        assert!(
            write_memory(&mut conn, &request("A different summary."))
                .unwrap_err()
                .contains("idempotency_conflict")
        );
        let count: i64 = conn
            .query_row("SELECT count(*) FROM memories", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn exact_content_from_another_operation_reuses_memory_and_preserves_new_source() {
        let mut conn = db();
        let first = write_memory(&mut conn, &request("The narrator walked their dogs.")).unwrap();
        let mut second_request = request("The narrator walked their dogs.");
        second_request.operation_id = "223e4567-e89b-42d3-a456-426614174000".into();
        second_request.memory.source.reference = "another-export.csv".into();
        second_request.memory.source.locator = Some("hmid=22".into());
        let second = write_memory(&mut conn, &second_request).unwrap();

        assert_eq!(second.outcome, "deduplicated");
        assert_eq!(second.memory_id, first.memory_id);
        assert_eq!(second.revision, first.revision);
        assert!(second.evidence_added);
        assert_eq!(second.provenance_status, "declared_unverified");
        let memories: i64 = conn
            .query_row("SELECT count(*) FROM memories", [], |row| row.get(0))
            .unwrap();
        let evidence: i64 = conn
            .query_row("SELECT count(*) FROM memory_source_evidence", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(memories, 1);
        assert_eq!(evidence, 2);
    }

    #[test]
    fn exact_fingerprint_normalizes_whitespace_but_keeps_scope_and_content_fields() {
        let mut conn = db();
        let first = write_memory(&mut conn, &request("The narrator walked their dogs.")).unwrap();
        let mut same = request("The narrator walked their dogs.");
        same.operation_id = "223e4567-e89b-42d3-a456-426614174000".into();
        same.memory.original_text = "  I walked\n my  dogs this morning.  ".into();
        let duplicate = write_memory(&mut conn, &same).unwrap();
        assert_eq!(duplicate.outcome, "deduplicated");
        assert_eq!(duplicate.memory_id, first.memory_id);

        let mut other_scope = same.clone();
        other_scope.operation_id = "323e4567-e89b-42d3-a456-426614174000".into();
        other_scope.scope.id = "person-b".into();
        assert_eq!(
            write_memory(&mut conn, &other_scope).unwrap().outcome,
            "created"
        );

        let mut other_summary = same;
        other_summary.operation_id = "423e4567-e89b-42d3-a456-426614174000".into();
        other_summary.memory.summary = "This says something else.".into();
        assert_eq!(
            write_memory(&mut conn, &other_summary).unwrap().outcome,
            "created"
        );
    }

    #[test]
    fn revise_appends_history_and_rejects_stale_or_duplicate_content() {
        let mut conn = db();
        let first = write_memory(&mut conn, &request("The narrator walked their dogs.")).unwrap();
        let mut second = request("The narrator walked their dogs.");
        second.operation_id = "223e4567-e89b-42d3-a456-426614174000".into();
        second.memory.original_text = "I now walk my dogs at lunchtime.".into();
        second.memory.summary = "The narrator walks their dogs at lunchtime.".into();
        second.action = "revise".into();
        second.memory_id = Some(first.memory_id.clone());
        second.expected_revision = Some(1);
        let revised = write_memory(&mut conn, &second).unwrap();
        assert_eq!(revised.outcome, "revised");
        assert_eq!(revised.memory_id, first.memory_id);
        assert_eq!(revised.revision, 2);
        let replayed = write_memory(&mut conn, &second).unwrap();
        assert_eq!(replayed.outcome, "revised");
        assert!(replayed.replayed);
        let revisions: i64 = conn
            .query_row(
                "SELECT count(*) FROM memory_revisions WHERE memory_id=?1",
                [&first.memory_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(revisions, 2);

        let mut stale = second.clone();
        stale.operation_id = "323e4567-e89b-42d3-a456-426614174000".into();
        stale.expected_revision = Some(1);
        assert!(
            write_memory(&mut conn, &stale)
                .unwrap_err()
                .contains("revision_conflict")
        );

        let mut other = request("A separate fact.");
        other.operation_id = "423e4567-e89b-42d3-a456-426614174000".into();
        other.memory.original_text = "A separate fact.".into();
        other.memory.summary = "The narrator has another fact.".into();
        let other_created = write_memory(&mut conn, &other).unwrap();
        let mut duplicate = other;
        duplicate.operation_id = "523e4567-e89b-42d3-a456-426614174000".into();
        duplicate.action = "revise".into();
        duplicate.memory_id = Some(other_created.memory_id);
        duplicate.expected_revision = Some(1);
        duplicate.memory.original_text = second.memory.original_text.clone();
        duplicate.memory.summary = second.memory.summary.clone();
        assert!(
            write_memory(&mut conn, &duplicate)
                .unwrap_err()
                .contains("exact_duplicate_requires_review")
        );
    }

    #[test]
    fn invalid_request_is_rejected_before_any_write() {
        let mut conn = db();
        let mut input = request("A faithful short summary.");
        input.memory.summary = "  ".into();
        assert!(
            write_memory(&mut conn, &input)
                .unwrap_err()
                .contains("memory.summary")
        );
        let count: i64 = conn
            .query_row("SELECT count(*) FROM memories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn batch_cap_is_enforced_before_processing() {
        let mut conn = db();
        let input = request("A faithful short summary.");
        assert!(
            write_batch(&mut conn, &vec![input; MAX_BATCH + 1])
                .unwrap_err()
                .contains("2500")
        );
        let count: i64 = conn
            .query_row("SELECT count(*) FROM memories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
