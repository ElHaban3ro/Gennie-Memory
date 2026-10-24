use super::{storage::*, *};
use crate::{
    classifier::Classifier,
    source_verification::SourcePolicy,
    write::{self, Scope},
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde_json::{Value, json};

fn scope_for(conn: &Connection, c: &WriteCommand) -> Result<Scope, String> {
    if let Some(s) = &c.scope {
        context(s)?;
        return Ok(s.clone());
    }
    if c.action == "create" {
        return Err("scope_required".into());
    }
    conn.query_row(
        "SELECT scope_kind,scope_id FROM memories WHERE id=?1 AND actor_id='local'",
        [&c.memory_id],
        |r| {
            Ok(Scope {
                kind: r.get(0)?,
                id: r.get(1)?,
            })
        },
    )
    .map_err(|_| "memory_not_available".into())
}
fn validate(conn: &Connection, c: &WriteCommand, scope: &Scope) -> Result<(), String> {
    context(scope)?;
    if !matches!(c.action.as_str(), "create" | "revise" | "confirm") {
        return Err(
            "action_invalid: usa create, revise o confirm; ciclo de vida usa mem-lifecycle".into(),
        );
    }
    if c.action == "confirm" {
        if c.memory.is_some()
            || c.topic_key.is_some()
            || c.entity_id.is_some()
            || c.relation_provider.is_some()
            || c.memory_id
                .as_deref()
                .is_none_or(|s| !crate::service::valid_identifier(s))
            || c.expected_revision.is_none_or(|r| r < 1)
        {
            return Err("confirm_fields_invalid".into());
        }
        evidence_valid(c.evidence.as_ref().ok_or("evidence_required")?)?;
        if let Some(Some(date)) = &c.review_after {
            if !crate::service::valid_utc(date) || date <= &crate::repo::utc_now(conn)? {
                return Err("review_after_invalid".into());
            }
        }
    } else {
        if c.evidence.is_some()
            || c.review_after.is_some()
            || (c.action != "create" && (c.topic_key.is_some() || c.relation_provider.is_some()))
        {
            return Err("action_fields_invalid".into());
        }
        legacy(c, scope)?;
    }
    Ok(())
}
fn candidates(
    conn: &Connection,
    c: &WriteCommand,
    scope: &Scope,
) -> Result<Vec<RelationCandidate>, String> {
    let mut ids = Vec::<String>::new();
    if let Some(topic) = &c.topic_key {
        let id:Option<String>=conn.query_row("SELECT id FROM memories WHERE actor_id='local' AND scope_kind=?1 AND scope_id=?2 AND topic_key=?3",params![scope.kind,scope.id,topic],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        if let Some(id) = id {
            ids.push(id)
        }
    }
    if c.relation_provider.is_some() {
        let m = c.memory.as_ref().ok_or("memory_required")?;
        let query = crate::model::SearchRequest {
            query: m.summary.clone(),
            scope_kind: scope.kind.clone(),
            scope_id: scope.id.clone(),
            entity_id: c.entity_id.clone(),
            actor_id: Some("local".into()),
            as_of: crate::repo::utc_now(conn)?,
            limit: Some(20),
            query_mode: None,
            memory_types: vec![m.memory_type.clone()],
            bm25_weights: None,
        };
        for candidate in crate::service::search(conn, &query)?.candidates {
            if !ids.contains(&candidate.memory_id) {
                ids.push(candidate.memory_id)
            }
        }
    }
    if ids.len() > 20 {
        return Err("relation_candidates_limit: acota el tema o la entidad".into());
    }
    ids.iter().map(|id| load(conn, scope, id)).collect()
}
fn judgments(
    c: &WriteCommand,
    cs: &[RelationCandidate],
    classifier: &impl Classifier,
) -> (Vec<RelationJudgment>, Option<Value>, Option<Value>) {
    if cs.is_empty() {
        return (vec![], None, None);
    }
    let fallback = || {
        cs.iter()
            .map(|v| RelationJudgment {
                target: v.target.clone(),
                relation: Relation::Undetermined,
                reason: "explicit_resolution_required".into(),
                incoming_quote: c.memory.as_ref().unwrap().original_text.clone(),
                existing_quote: v.memory.original_text.clone(),
            })
            .collect()
    };
    let Some(provider) = c.relation_provider else {
        return (fallback(), None, None);
    };
    match classifier.classify_relations(c.memory.as_ref().unwrap(),cs,provider){
        Ok(report)=>{
            let mut seen=std::collections::BTreeSet::new();
            let valid=report.judgments.len()==cs.len()&&report.judgments.iter().all(|j|{
                cs.iter().any(|v|v.target==j.target&&!j.incoming_quote.is_empty()&&!j.existing_quote.is_empty()&&c.memory.as_ref().unwrap().original_text.contains(&j.incoming_quote)&&v.memory.original_text.contains(&j.existing_quote))&&seen.insert(j.target.memory_id.clone())&&!j.reason.trim().is_empty()&&j.reason.len()<=1600
            });
            if valid{(report.judgments,report.usage,None)}else{(fallback(),report.usage,Some(json!({"code":"model_output_invalid","stage":"relation_classification","http_status":null})))}
        },
        Err(e)=>(fallback(),e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd,"receipt_status":"partial"})),Some(json!({"code":e.code,"stage":"relation_classification","http_status":e.http_status,"provider_code":e.diagnostic.as_ref().map(|d|d.code),"retry_after_ms":e.diagnostic.as_ref().and_then(|d|d.retry_after_ms)})))
    }
}

pub fn write(
    conn: &mut Connection,
    command: &WriteCommand,
    classifier: &impl Classifier,
) -> Result<Value, String> {
    let mut c = command.clone();
    c.operation_id = identity(c.schema_version, &c.operation_id)?;
    c.topic_key = normalize_topic(c.topic_key.as_deref())?;
    let request_hash = hash(&c)?;
    if let Some(r) = replay(conn, &c.operation_id, &request_hash)? {
        return Ok(r);
    }
    let scope = scope_for(conn, &c)?;
    validate(conn, &c, &scope)?;
    if let Some(entity) = &c.entity_id {
        if !crate::entity::exists(conn, "local", &scope, entity)? {
            return Err("entity_not_available".into());
        }
    }
    // Snapshot candidates before provider I/O. No SQL transaction spans the call.
    let cs = if c.action == "create" {
        candidates(conn, &c, &scope)?
    } else if c.action == "revise" {
        vec![load(
            conn,
            &scope,
            c.memory_id.as_deref().ok_or("memory_id_required")?,
        )?]
    } else {
        vec![]
    };
    let exact_before_call = if c.action == "create" {
        let r = legacy(&c, &scope)?;
        conn.query_row("SELECT EXISTS(SELECT 1 FROM memory_content_keys WHERE scope_kind=?1 AND scope_id=?2 AND content_sha256=?3)",params![scope.kind,scope.id,write::content_fingerprint(&r)],|r|r.get::<_,bool>(0)).map_err(|e|e.to_string())?
    } else {
        false
    };
    let (relations, usage, classification_error) = if exact_before_call {
        (vec![], None, None)
    } else {
        judgments(&c, &cs, classifier)
    };
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    if let Some(r) = replay(&tx, &c.operation_id, &request_hash)? {
        return Ok(r);
    }
    // Legacy receipts remain replayable with their original canonical hash.
    let legacy_exists: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM memory_write_operations WHERE operation_id=?1)",
            [&c.operation_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if legacy_exists
        && matches!(c.action.as_str(), "create" | "revise")
        && c.topic_key.is_none()
        && c.relation_provider.is_none()
        && matches!(c.source_policy, SourcePolicy::Report)
    {
        let result = write::write_in_transaction(&tx, &legacy(&c, &scope)?)?;
        let value = serde_json::to_value(result).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        return Ok(value);
    }
    collision(&tx, &c.operation_id)?;
    for old in &cs {
        ensure_revision(
            &load(&tx, &scope, &old.target.memory_id)?,
            Some(old.target.revision),
        )?;
    }
    // Detect a topic inserted while the provider was running, too.
    if c.action == "create" && c.topic_key.is_some() {
        let current = candidates_topic(&tx, &scope, c.topic_key.as_deref().unwrap())?;
        if current
            .iter()
            .any(|id| !cs.iter().any(|v| &v.target.memory_id == id))
        {
            return Err("revision_conflict: tema cambió durante comparación".into());
        }
    }
    let mut result = if c.action == "confirm" {
        confirm(&tx, &c, &scope)?
    } else {
        let mut legacy = legacy(&c, &scope)?;
        if c.action == "revise" {
            let old = load(&tx, &scope, c.memory_id.as_ref().unwrap())?;
            ensure_revision(&old, c.expected_revision)?;
            legacy.entity_id = old_entity(&tx, &old.target.memory_id)?;
            if c.entity_id.is_some() && c.entity_id != legacy.entity_id {
                return Err("entity_mismatch".into());
            }
        }
        let evidence = Evidence {
            original_text: legacy.memory.original_text.clone(),
            source: legacy.memory.source.clone(),
        };
        let provenance = crate::source_verification::check_evidence(
            &tx,
            &scope,
            "local",
            &evidence.source,
            &evidence.original_text,
        )?;
        let p = serde_json::to_value(&provenance).map_err(|e| e.to_string())?;
        let strict = matches!(c.source_policy, SourcePolicy::RequireVerified)
            && p["status"] != "verified_literal";
        // Exact identity wins over uncertain semantic labels, retaining new provenance.
        write::backfill_scope_keys(&tx, &scope.kind, &scope.id)?;
        let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM memory_content_keys WHERE scope_kind=?1 AND scope_id=?2 AND content_sha256=?3)",params![scope.kind,scope.id,write::content_fingerprint(&legacy)],|r|r.get(0)).map_err(|e|e.to_string())?;
        let needs_relation = c.action == "create" && !exact && (!cs.is_empty());
        if strict || needs_relation {
            let reason = if strict {
                "unverified_source"
            } else if c.topic_key.is_some() {
                "topic_conflict"
            } else {
                "relation_review_required"
            };
            propose(
                &tx,
                &c,
                &scope,
                &relations,
                reason,
                usage,
                classification_error,
            )?
        } else {
            let result = write::write_in_transaction(&tx, &legacy)?;
            let target = Target {
                memory_id: result.memory_id.clone(),
                revision: result.revision,
            };
            if let Some(topic) = &c.topic_key {
                let existing: Option<String> = tx
                    .query_row(
                        "SELECT topic_key FROM memories WHERE id=?1",
                        [&target.memory_id],
                        |r| r.get(0),
                    )
                    .map_err(|e| e.to_string())?;
                if existing.is_some() && existing.as_ref() != Some(topic) {
                    return Err("topic_mismatch: identidad ya tiene otro tema".into());
                }
                tx.execute(
                    "UPDATE memories SET topic_key=?2 WHERE id=?1",
                    params![target.memory_id, topic],
                )
                .map_err(|e| e.to_string())?;
            }
            audit_evidence(
                &tx,
                &c.operation_id,
                &scope,
                Some(&target),
                None,
                &c.action,
                &evidence,
            )?;
            if c.action == "revise" {
                link(
                    &tx,
                    &target,
                    &Target {
                        memory_id: target.memory_id.clone(),
                        revision: c.expected_revision.unwrap(),
                    },
                    "correction",
                    &c.operation_id,
                    &json!(evidence),
                )?
            }
            let mut value = serde_json::to_value(result).map_err(|e| e.to_string())?;
            value["source_check"] = p;
            value
        }
    };
    result["scope"] = json!(scope);
    save(&tx, &c.operation_id, &request_hash, &result)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(result)
}
fn candidates_topic(conn: &Connection, scope: &Scope, topic: &str) -> Result<Vec<String>, String> {
    let mut s=conn.prepare("SELECT id FROM memories WHERE actor_id='local' AND scope_kind=?1 AND scope_id=?2 AND topic_key=?3").map_err(|e|e.to_string())?;
    s.query_map(params![scope.kind, scope.id, topic], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
fn old_entity(conn: &Connection, id: &str) -> Result<Option<String>, String> {
    conn.query_row("SELECT entity_id FROM memories WHERE id=?1", [id], |r| {
        r.get(0)
    })
    .map_err(|e| e.to_string())
}
fn propose(
    tx: &Transaction,
    c: &WriteCommand,
    scope: &Scope,
    relations: &[RelationJudgment],
    reason: &str,
    usage: Option<Value>,
    error: Option<Value>,
) -> Result<Value, String> {
    let id = format!("prop-{}", &hash(c)?[..32]);
    tx.execute(
        "INSERT INTO memory_proposals VALUES (?1,'local',?2,?3,?4,?5,?6)",
        params![
            id,
            scope.kind,
            scope.id,
            serde_json::to_string(c).map_err(|e| e.to_string())?,
            reason,
            crate::repo::utc_now(tx)?
        ],
    )
    .map_err(|e| e.to_string())?;
    for j in relations {
        tx.execute(
            "INSERT INTO memory_proposal_targets VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                id,
                j.target.memory_id,
                j.target.revision,
                j.relation.as_str(),
                j.reason,
                json!({"incoming_quote":j.incoming_quote,"existing_quote":j.existing_quote})
                    .to_string()
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    let m = c.memory.as_ref().unwrap();
    let provenance = audit_evidence(
        tx,
        &c.operation_id,
        scope,
        None,
        Some(&id),
        "propose",
        &Evidence {
            original_text: m.original_text.clone(),
            source: m.source.clone(),
        },
    )?;
    Ok(
        json!({"schema_version":1,"operation_id":c.operation_id,"status":"needs_resolution","proposal_id":id,"reason":reason,"candidates":relations.iter().map(|j|&j.target).collect::<Vec<_>>(),"relations":relations.iter().map(|j|json!({"target":j.target,"relation":j.relation,"reason":j.reason})).collect::<Vec<_>>(),"classification_error":error.as_ref().map(|e|e["code"].clone()),"classification_diagnostic":error,"usage":usage,"source_check":provenance,"replayed":false,"warnings":[]}),
    )
}
fn confirm(tx: &Transaction, c: &WriteCommand, scope: &Scope) -> Result<Value, String> {
    let old = load(tx, scope, c.memory_id.as_ref().unwrap())?;
    ensure_revision(&old, c.expected_revision)?;
    let evidence = c.evidence.as_ref().unwrap();
    let provenance = crate::source_verification::check_evidence(
        tx,
        scope,
        "local",
        &evidence.source,
        &evidence.original_text,
    )?;
    if matches!(c.source_policy, SourcePolicy::RequireVerified)
        && provenance.status != "verified_literal"
    {
        return Err("unverified_source: confirmación requiere evidencia verificable".into());
    }
    let schedule: Option<String> = tx
        .query_row(
            "SELECT next_review_at FROM memory_review_schedule WHERE memory_id=?1",
            [&old.target.memory_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let target = Target {
        memory_id: old.target.memory_id.clone(),
        revision: old.target.revision + 1,
    };
    tx.execute("INSERT INTO memory_revisions(memory_id,revision,memory_type,original_text,summary,state,valid_from,valid_until,conditions,source_kind,source_ref,source_locator) SELECT memory_id,?3,memory_type,original_text,summary,state,valid_from,valid_until,conditions,source_kind,source_ref,source_locator FROM memory_revisions WHERE memory_id=?1 AND revision=?2",params![target.memory_id,old.target.revision,target.revision]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO search_documents(memory_id,revision,original_text,summary) VALUES (?1,?2,?3,?4)",params![target.memory_id,target.revision,old.memory.original_text,old.memory.summary]).map_err(|e|e.to_string())?;
    tx.execute(
        "UPDATE memories SET current_revision=?2 WHERE id=?1",
        params![target.memory_id, target.revision],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE memory_content_keys SET revision=?2 WHERE memory_id=?1",
        params![target.memory_id, target.revision],
    )
    .map_err(|e| e.to_string())?;
    // Original provenance remains on the revision; confirmation has its own evidence.
    tx.execute("INSERT INTO memory_source_evidence(memory_id,revision,source_kind,source_ref,source_locator,evidence_text,verification_status,recorded_at) SELECT memory_id,?3,source_kind,source_ref,source_locator,evidence_text,verification_status,recorded_at FROM memory_source_evidence WHERE memory_id=?1 AND revision=?2",params![target.memory_id,old.target.revision,target.revision]).map_err(|e|e.to_string())?;
    let input = write::MemoryInput {
        original_text: evidence.original_text.clone(),
        summary: old.memory.summary.clone(),
        memory_type: old.memory.memory_type.clone(),
        conditions: old.memory.conditions.clone(),
        valid_from: old.memory.valid_from.clone(),
        valid_until: old.memory.valid_until.clone(),
        source: evidence.source.clone(),
    };
    let r = write::WriteRequest {
        schema_version: 1,
        operation_id: c.operation_id.clone(),
        action: "revise".into(),
        memory_id: Some(target.memory_id.clone()),
        expected_revision: Some(old.target.revision),
        entity_id: None,
        scope: scope.clone(),
        memory: input,
    };
    write::add_source_evidence(
        tx,
        &r,
        &target.memory_id,
        target.revision,
        &crate::repo::utc_now(tx)?,
    )?;
    // Lifecycle owns the schedule. Preserve omitted date, clear/set explicitly.
    let next = c.review_after.as_ref().unwrap_or(&schedule);
    tx.execute(
        "UPDATE memory_review_schedule SET next_review_at=?2,reviewed_at=?3 WHERE memory_id=?1",
        params![target.memory_id, next, crate::repo::utc_now(tx)?],
    )
    .map_err(|e| e.to_string())?;
    audit_evidence(
        tx,
        &c.operation_id,
        scope,
        Some(&target),
        None,
        "confirm",
        evidence,
    )?;
    let mut result = receipt(&c.operation_id, "confirmed", &target);
    result["source_check"] = json!(provenance);
    Ok(result)
}

pub fn write_batch(
    conn: &mut Connection,
    cs: &[WriteCommand],
    classifier: &impl Classifier,
) -> Result<Vec<Value>, String> {
    if cs.is_empty() || cs.len() > 2500 {
        return Err("batch_size_invalid: entre 1 y 2500 operaciones".into());
    }
    cs.iter()
        .enumerate()
        .map(|(i, c)| write(conn, c, classifier).map_err(|e| format!("operación {}: {e}", i + 1)))
        .collect()
}

pub fn resolve(conn: &mut Connection, command: &ResolveCommand) -> Result<Value, String> {
    let mut c = command.clone();
    c.operation_id = identity(c.schema_version, &c.operation_id)?;
    c.topic_key = normalize_topic(c.topic_key.as_deref())?;
    c.expected_targets
        .sort_by(|a, b| a.memory_id.cmp(&b.memory_id));
    if !crate::service::valid_identifier(&c.proposal_id)
        || !matches!(
            c.decision.as_str(),
            "keep_existing" | "replace_existing" | "create_separate"
        )
        || c.expected_targets.len() > 20
        || c.expected_targets
            .windows(2)
            .any(|w| w[0].memory_id == w[1].memory_id)
        || c.expected_targets
            .iter()
            .any(|t| t.revision < 1 || !crate::service::valid_identifier(&t.memory_id))
    {
        return Err("resolution_invalid".into());
    }
    evidence_valid(&c.evidence)?;
    if c.decision == "keep_existing"
        && (c.target_memory_id.is_some()
            || c.scope.is_some()
            || c.memory.is_some()
            || c.topic_key.is_some())
        || c.decision == "replace_existing"
            && (c.target_memory_id.is_none()
                || c.memory.is_none()
                || c.scope.is_some()
                || c.topic_key.is_some())
        || c.decision == "create_separate"
            && (c.target_memory_id.is_some() || c.scope.is_none() || c.memory.is_none())
    {
        return Err("resolution_fields_invalid".into());
    }
    let request_hash = hash(&c)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    if let Some(r) = replay(&tx, &c.operation_id, &request_hash)? {
        return Ok(r);
    }
    collision(&tx, &c.operation_id)?;
    let row:Option<(String,String,String)>=tx.query_row("SELECT scope_kind,scope_id,request_json FROM memory_proposals WHERE proposal_id=?1 AND actor_id='local'",[&c.proposal_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(|e|e.to_string())?;
    let (kind, id, request) = row.ok_or("proposal_not_available")?;
    let scope = Scope { kind, id };
    let original: WriteCommand = serde_json::from_str(&request).map_err(|e| e.to_string())?;
    let mut stmt = tx
        .prepare(
            "SELECT memory_id FROM memory_proposal_targets WHERE proposal_id=?1 ORDER BY memory_id",
        )
        .map_err(|e| e.to_string())?;
    let ids: Vec<String> = stmt
        .query_map([&c.proposal_id], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);
    if ids
        != c.expected_targets
            .iter()
            .map(|t| t.memory_id.clone())
            .collect::<Vec<_>>()
    {
        return Err("expected_targets_mismatch: debes incluir todos los destinos".into());
    }
    for t in &c.expected_targets {
        ensure_revision(&load(&tx, &scope, &t.memory_id)?, Some(t.revision))?;
    }
    // Resolution evidence always needs a literal, authorized snapshot.
    let checked = crate::source_verification::check_evidence(
        &tx,
        &scope,
        "local",
        &c.evidence.source,
        &c.evidence.original_text,
    )?;
    if c.decision != "keep_existing" && checked.status != "verified_literal" {
        return Err("unverified_source: registra y verifica evidencia antes de resolver".into());
    }
    let mut result = if c.decision == "keep_existing" {
        json!({"schema_version":1,"operation_id":c.operation_id,"status":"committed","outcome":"discarded","proposal_id":c.proposal_id,"replayed":false,"warnings":[]})
    } else {
        let memory = c.memory.clone().unwrap();
        let checked = crate::source_verification::check_evidence(
            &tx,
            &scope,
            "local",
            &memory.source,
            &memory.original_text,
        )?;
        if checked.status != "verified_literal" {
            return Err(
                "unverified_source: el contenido resultante también requiere fuente verificable"
                    .into(),
            );
        }
        let (action, memory_id, expected_revision, entity) = if c.decision == "replace_existing" {
            let id = c.target_memory_id.clone().unwrap();
            let t = c
                .expected_targets
                .iter()
                .find(|t| t.memory_id == id)
                .ok_or("target_memory_id_invalid")?;
            {
                let entity = old_entity(&tx, &id)?;
                // Resolving a proposal must never transfer a claim to another entity.
                let original_entity = if original.action == "revise" {
                    old_entity(
                        &tx,
                        original.memory_id.as_deref().ok_or("memory_id_required")?,
                    )?
                } else {
                    original.entity_id.clone()
                };
                if entity != original_entity {
                    return Err(
                        "entity_mismatch: propuesta y destino pertenecen a entidades distintas"
                            .into(),
                    );
                }
                ("revise", Some(id.clone()), Some(t.revision), entity)
            }
        } else {
            let supplied = c.scope.as_ref().unwrap();
            if supplied.kind != scope.kind || supplied.id != scope.id {
                return Err("scope_mismatch".into());
            }
            let topic = c.topic_key.as_ref().or(original.topic_key.as_ref());
            if let Some(topic) = topic {
                if !candidates_topic(&tx, &scope, topic)?.is_empty() {
                    return Err("topic_conflict: create_separate requiere un tema distinto".into());
                }
            }
            if !c.expected_targets.is_empty()
                && memory
                    .conditions
                    .as_deref()
                    .is_none_or(|v| v.trim().is_empty())
            {
                return Err(
                    "conditions_required: explica la distinción del recuerdo separado".into(),
                );
            }
            ("create", None, None, original.entity_id.clone())
        };
        let r = write::WriteRequest {
            schema_version: 1,
            operation_id: c.operation_id.clone(),
            action: action.into(),
            memory_id,
            expected_revision,
            entity_id: entity,
            scope: scope.clone(),
            memory,
        };
        let written = write::write_in_transaction(&tx, &r)?;
        if c.decision == "create_separate" && written.outcome == "deduplicated" {
            return Err("exact_duplicate_requires_review: no se creó una memoria separada".into());
        }
        let target = Target {
            memory_id: written.memory_id.clone(),
            revision: written.revision,
        };
        if action == "create" {
            if let Some(topic) = c.topic_key.as_ref().or(original.topic_key.as_ref()) {
                tx.execute(
                    "UPDATE memories SET topic_key=?2 WHERE id=?1",
                    params![target.memory_id, topic],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        for old in &c.expected_targets {
            let relation = if action == "revise" && old.memory_id == target.memory_id {
                "correction"
            } else {
                "unrelated"
            };
            link(
                &tx,
                &target,
                old,
                relation,
                &c.operation_id,
                &json!(c.evidence),
            )?;
        }
        audit_evidence(
            &tx,
            &c.operation_id,
            &scope,
            Some(&target),
            None,
            "resolve",
            &c.evidence,
        )?;
        audit_evidence(
            &tx,
            &c.operation_id,
            &scope,
            Some(&target),
            None,
            "resolved_content",
            &Evidence {
                original_text: r.memory.original_text.clone(),
                source: r.memory.source.clone(),
            },
        )?;
        // Transfer proposal provenance to the accepted revision before purging its payload.
        tx.execute("UPDATE memory_evidence_audit SET memory_id=?2,revision=?3,proposal_id=NULL WHERE proposal_id=?1",params![c.proposal_id,target.memory_id,target.revision]).map_err(|e|e.to_string())?;
        serde_json::to_value(written).map_err(|e| e.to_string())?
    };
    if c.decision == "keep_existing" {
        // The discard receipt carries no original proposal or rejected content.
        tx.execute(
            "DELETE FROM memory_evidence_audit WHERE proposal_id=?1",
            [&c.proposal_id],
        )
        .map_err(|e| e.to_string())?;
        if c.expected_targets.is_empty() {
            audit_evidence(
                &tx,
                &c.operation_id,
                &scope,
                None,
                None,
                "discard",
                &c.evidence,
            )?;
        } else {
            for target in &c.expected_targets {
                audit_evidence(
                    &tx,
                    &c.operation_id,
                    &scope,
                    Some(target),
                    None,
                    "discard",
                    &c.evidence,
                )?;
            }
        }
    }
    tx.execute(
        "DELETE FROM memory_proposals WHERE proposal_id=?1",
        [&c.proposal_id],
    )
    .map_err(|e| e.to_string())?;
    result["proposal_id"] = json!(c.proposal_id);
    result["scope"] = json!(scope);
    save(&tx, &c.operation_id, &request_hash, &result)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(result)
}

fn inspect_scope(r: &InspectRequest) -> Result<(), String> {
    if r.schema_version != 1 {
        return Err("schema_version_invalid".into());
    }
    context(&r.scope)
}
pub fn proposal_get(conn: &Connection, r: &InspectRequest) -> Result<Value, String> {
    inspect_scope(r)?;
    let id = r.proposal_id.as_ref().ok_or("proposal_id_required")?;
    let row:Option<(String,String)>=conn.query_row("SELECT request_json,reason FROM memory_proposals WHERE proposal_id=?1 AND actor_id='local' AND scope_kind=?2 AND scope_id=?3",params![id,r.scope.kind,r.scope.id],|v|Ok((v.get(0)?,v.get(1)?))).optional().map_err(|e|e.to_string())?;
    let (request, reason) = row.ok_or("proposal_not_available")?;
    let mut stmt=conn.prepare("SELECT memory_id,revision,relation,reason,evidence_json FROM memory_proposal_targets WHERE proposal_id=?1 ORDER BY memory_id").map_err(|e|e.to_string())?;
    let targets=stmt.query_map([id],|v|Ok(json!({"memory_id":v.get::<_,String>(0)?,"revision":v.get::<_,i64>(1)?,"relation":v.get::<_,String>(2)?,"reason":v.get::<_,String>(3)?,"evidence":serde_json::from_str::<Value>(&v.get::<_,String>(4)?).unwrap_or(Value::Null)}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    Ok(
        json!({"schema_version":1,"proposal_id":id,"reason":reason,"request":serde_json::from_str::<Value>(&request).map_err(|e|e.to_string())?,"targets":targets}),
    )
}
pub fn audit(conn: &Connection, r: &InspectRequest) -> Result<Value, String> {
    inspect_scope(r)?;
    let id = r.memory_id.as_ref().ok_or("memory_id_required")?;
    load(conn, &r.scope, id)?;
    let mut stmt=conn.prepare("SELECT audit_id,operation_id,revision,action,evidence_json,evidence_sha256,provenance_json,recorded_at FROM memory_evidence_audit WHERE actor_id='local' AND scope_kind=?1 AND scope_id=?2 AND memory_id=?3 ORDER BY audit_id LIMIT 1001").map_err(|e|e.to_string())?;
    let rows=stmt.query_map(params![r.scope.kind,r.scope.id,id],|v|Ok(json!({"audit_id":v.get::<_,i64>(0)?,"operation_id":v.get::<_,String>(1)?,"revision":v.get::<_,i64>(2)?,"action":v.get::<_,String>(3)?,"evidence":serde_json::from_str::<Value>(&v.get::<_,String>(4)?).unwrap_or(Value::Null),"evidence_sha256":v.get::<_,String>(5)?,"observed_evidence_sha256":gennie_memory_kernel::digest::sha256_hex(v.get::<_,String>(4)?.as_bytes()),"provenance":serde_json::from_str::<Value>(&v.get::<_,String>(6)?).unwrap_or(Value::Null),"recorded_at":v.get::<_,String>(7)?}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    if rows.len() > 1000 {
        return Err("audit_limit: inspección supera 1000 entradas".into());
    }
    for row in &rows {
        if row["evidence_sha256"] != row["observed_evidence_sha256"] {
            return Err("audit_integrity_mismatch".into());
        }
    }
    Ok(json!({"schema_version":1,"memory_id":id,"entries":rows}))
}
pub fn operation_get(conn: &Connection, r: &InspectRequest) -> Result<Value, String> {
    inspect_scope(r)?;
    let op = identity(1, r.operation_id.as_deref().ok_or("operation_id_required")?)?;
    let text: String = conn
        .query_row(
            "SELECT response_json FROM reconciliation_operations WHERE operation_id=?1",
            [op],
            |v| v.get(0),
        )
        .map_err(|_| "operation_not_available")?;
    let value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    if value["scope"]["kind"] != r.scope.kind || value["scope"]["id"] != r.scope.id {
        return Err("operation_not_available".into());
    }
    Ok(value)
}
