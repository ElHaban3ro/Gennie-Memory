use crate::{embedding, entity, model::*, module::RetrievalModule, repo, service, write};
use gennie_memory_kernel::{core::AppModule, db};
use rusqlite::Connection;
use serde_json::json;

fn database() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    db::configure(&conn).unwrap();
    db::migrate(&mut conn, &[Box::new(RetrievalModule)]).unwrap();
    conn
}
fn memory(id: &str, actor: &str, kind: &str, text: &str) -> FixtureMemory {
    serde_json::from_value(json!({"id":id,"scope_kind":"project","scope_id":"demo","actor_id":actor,"revision":1,"memory_type":kind,"original_text":text,"summary":text,"state":"active","valid_from":null,"valid_until":null})).unwrap()
}
fn search(query: &str) -> SearchRequest {
    serde_json::from_value(json!({"query":query,"scope_kind":"project","scope_id":"demo","as_of":"2026-10-24T12:00:00.000Z","limit":1})).unwrap()
}
fn vectors(conn: &mut Connection, memories: &[FixtureMemory]) {
    let batch = EmbeddingBatch {
        space_id: "test".into(),
        model_id: "test".into(),
        model_sha256: "a".repeat(64),
        tokenizer_sha256: "b".repeat(64),
        procedure: "test".into(),
        dimensions: 2,
        vectors: memories
            .iter()
            .map(|m| FixtureVector {
                memory_id: m.id.clone(),
                revision: m.revision,
                values: if m.id == "allowed" {
                    vec![0.1, 1.0]
                } else {
                    vec![1.0, 0.0]
                },
            })
            .collect(),
    };
    embedding::load_batch(conn, &batch).unwrap();
}

#[test]
fn actor_type_and_scope_filters_precede_both_rankings_and_scope_list() {
    let mut conn = database();
    let mut memories = vec![
        memory("allowed", "local", "fact", "cobalt"),
        memory("foreign", "other", "fact", "cobalt cobalt cobalt"),
        memory("wrong-type", "local", "event", "cobalt cobalt"),
        memory("wrong-scope", "local", "fact", "cobalt"),
    ];
    memories[3].scope_id = "elsewhere".into();
    repo::load_fixtures(&mut conn, &memories).unwrap();
    vectors(&mut conn, &memories);
    let mut request = search("cobalt");
    request.memory_types = vec!["fact".into()];
    let result = service::search(&conn, &request).unwrap();
    assert_eq!(
        result
            .candidates
            .iter()
            .map(|c| c.memory_id.as_str())
            .collect::<Vec<_>>(),
        ["allowed"]
    );
    assert_eq!(result.coverage.eligible_memories, 1);
    let hybrid = embedding::search_hybrid(
        &conn,
        &HybridRequest {
            search: request.clone(),
            space_id: "test".into(),
            query_vector: vec![1.0, 0.0],
        },
    )
    .unwrap();
    assert_eq!(hybrid.candidates.len(), 1);
    assert_eq!(hybrid.candidates[0].memory_id, "allowed");
    assert_eq!((hybrid.eligible_memories, hybrid.indexed_memories), (1, 1));
    assert_eq!(
        repo::list_scope(&conn, &request).unwrap()[0].memory_id,
        "allowed"
    );
    request.actor_id = Some("other".into());
    assert_eq!(
        service::search(&conn, &request).unwrap().candidates[0].memory_id,
        "foreign"
    );
    request.actor_id = Some("unknown".into());
    assert!(
        service::search(&conn, &request)
            .unwrap()
            .candidates
            .is_empty()
    );
}

#[test]
fn composition_cannot_reintroduce_a_foreign_actor_or_type() {
    let mut conn = database();
    repo::load_fixtures(
        &mut conn,
        &[
            memory("allowed", "local", "fact", "cobalt"),
            memory("foreign", "other", "fact", "cobalt"),
            memory("event", "local", "event", "cobalt"),
        ],
    )
    .unwrap();
    for id in ["foreign", "event"] {
        let mut query = search("cobalt");
        query.memory_types = vec!["fact".into()];
        let request = SeedRequest {
            search: Some(query),
            hybrid: None,
            presented: vec![SelectedCandidate {
                memory_id: id.into(),
                revision: 1,
            }],
            selected: vec![SelectedCandidate {
                memory_id: id.into(),
                revision: 1,
            }],
            classifier: "test".into(),
            classification_status: "complete".into(),
        };
        assert!(service::compose_seed(&conn, &request).is_err());
    }
}

#[test]
fn fixture_revisions_cannot_change_actor_and_failed_batch_rolls_back() {
    let mut conn = database();
    repo::load_fixtures(&mut conn, &[memory("one", "local", "fact", "cobalt")]).unwrap();
    let mut changed = memory("one", "other", "fact", "cobalt");
    changed.revision = 2;
    assert!(
        repo::load_fixtures(
            &mut conn,
            &[memory("new", "local", "fact", "cobalt"), changed]
        )
        .is_err()
    );
    assert_eq!(
        conn.query_row("SELECT count(*) FROM memories", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}

fn entity_request(op: u32, label: &str) -> entity::EntityRequest {
    serde_json::from_value(json!({"schema_version":1,"operation_id":format!("123e4567-e89b-42d3-a456-{op:012}"),"scope":{"kind":"project","id":"demo"},"kind":"person","label":label,"source":{"kind":"document","ref":"synthetic-people","locator":format!("person:{op}")}})).unwrap()
}
fn write_request(op: u32, entity: Option<&str>) -> write::WriteRequest {
    serde_json::from_value(json!({"schema_version":1,"operation_id":format!("223e4567-e89b-42d3-a456-{op:012}"),"action":"create","entity_id":entity,"scope":{"kind":"project","id":"demo"},"memory":{"type":"fact","original_text":"Ana uses cobalt.","summary":"Ana uses cobalt.","source":{"kind":"document","ref":"synthetic-people"}}})).unwrap()
}

#[test]
fn entity_aliases_homonyms_and_operation_replays_preserve_identity() {
    let mut conn = database();
    let first = entity_request(1, "Ana María");
    let created = entity::resolve(&mut conn, &first).unwrap();
    assert_eq!(created.status, "created");
    assert!(entity::resolve(&mut conn, &first).unwrap().replayed);
    let same_name = entity::resolve(&mut conn, &entity_request(2, "ANA   MARÍA")).unwrap();
    assert_eq!(same_name.status, "ambiguous");
    assert_eq!(same_name.entity_id, None);
    assert_eq!(same_name.candidates.len(), 1);
    let mut explicit = entity_request(3, "Anita");
    explicit.entity_id = created.entity_id.clone();
    assert_eq!(
        entity::resolve(&mut conn, &explicit).unwrap().status,
        "resolved"
    );
    let alias = entity::resolve(&mut conn, &entity_request(4, "anita")).unwrap();
    assert_eq!(
        alias.candidates[0].entity_id,
        created.entity_id.clone().unwrap()
    );
    let mut homonym = entity_request(5, "Ana María");
    homonym.create_new = true;
    let second = entity::resolve(&mut conn, &homonym).unwrap();
    assert_ne!(created.entity_id, second.entity_id);
    let ambiguous = entity::resolve(&mut conn, &entity_request(6, "Ana María")).unwrap();
    assert_eq!(ambiguous.candidates.len(), 2);
    let mut conflict = first;
    conflict.label = "Bea".into();
    assert!(
        entity::resolve(&mut conn, &conflict)
            .unwrap_err()
            .contains("idempotency_conflict")
    );
    assert_eq!(
        conn.query_row("SELECT count(*) FROM entity_evidence", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
}

#[test]
fn entity_ids_and_candidates_never_cross_actor_scope_or_kind() {
    let mut conn = database();
    let created = entity::resolve(&mut conn, &entity_request(1, "Ana")).unwrap();
    for (index, dimension) in ["actor", "scope", "kind"].into_iter().enumerate() {
        let mut request = entity_request(index as u32 + 2, "Ana");
        request.entity_id = created.entity_id.clone();
        match dimension {
            "actor" => request.actor_id = Some("other".into()),
            "scope" => request.scope.id = "elsewhere".into(),
            _ => request.kind = "company".into(),
        }
        assert!(
            entity::resolve(&mut conn, &request)
                .unwrap_err()
                .contains("entity_not_available")
        );
        request.entity_id = None;
        assert_eq!(
            entity::resolve(&mut conn, &request).unwrap().status,
            "created"
        );
    }
}

#[test]
fn resolved_entity_round_trip_and_exact_dedup_keep_distinct_homonyms() {
    let mut conn = database();
    let one = entity::resolve(&mut conn, &entity_request(1, "Ana"))
        .unwrap()
        .entity_id
        .unwrap();
    let mut second = entity_request(2, "Ana");
    second.create_new = true;
    let two = entity::resolve(&mut conn, &second)
        .unwrap()
        .entity_id
        .unwrap();
    let first = write::write_memory(&mut conn, &write_request(1, Some(&one))).unwrap();
    let repeat = write::write_memory(&mut conn, &write_request(2, Some(&one))).unwrap();
    assert_eq!(repeat.outcome, "deduplicated");
    assert_eq!(first.memory_id, repeat.memory_id);
    let other = write::write_memory(&mut conn, &write_request(3, Some(&two))).unwrap();
    assert_ne!(first.memory_id, other.memory_id);
    let mut query = search("cobalt");
    query.entity_id = Some(two.clone());
    assert_eq!(
        service::search(&conn, &query).unwrap().candidates[0].memory_id,
        other.memory_id
    );
    let mut revision = write_request(4, None);
    revision.action = "revise".into();
    revision.memory_id = Some(first.memory_id.clone());
    revision.expected_revision = Some(1);
    revision.memory.summary = "Ana now uses cobalt daily.".into();
    write::write_memory(&mut conn, &revision).unwrap();
    query.entity_id = Some(one);
    assert_eq!(
        service::search(&conn, &query).unwrap().candidates[0].revision,
        2
    );
    let mut illegal = revision;
    illegal.operation_id = "223e4567-e89b-42d3-a456-000000000005".into();
    illegal.expected_revision = Some(2);
    illegal.entity_id = Some(two);
    assert!(
        write::write_memory(&mut conn, &illegal)
            .unwrap_err()
            .contains("entity_mismatch")
    );
}

#[test]
fn write_rejects_unknown_entities_and_other_actor_without_side_effects() {
    let mut conn = database();
    assert!(
        write::write_memory(&mut conn, &write_request(1, Some("unknown")))
            .unwrap_err()
            .contains("entity_not_available")
    );
    repo::load_fixtures(
        &mut conn,
        &[memory("foreign", "other", "fact", "Ana uses cobalt.")],
    )
    .unwrap();
    let mut request = write_request(2, None);
    request.action = "revise".into();
    request.memory_id = Some("foreign".into());
    request.expected_revision = Some(1);
    assert!(
        write::write_memory(&mut conn, &request)
            .unwrap_err()
            .contains("memory_not_available")
    );
    request = write_request(3, None);
    let created = write::write_memory(&mut conn, &request).unwrap();
    assert_eq!(created.outcome, "created");
    assert_ne!(created.memory_id, "foreign");
}

#[test]
fn operation_ids_cannot_be_reused_across_entity_and_memory_writes() {
    let mut conn = database();
    let entity = entity_request(1, "Ana");
    entity::resolve(&mut conn, &entity).unwrap();
    let mut memory = write_request(1, None);
    memory.operation_id = entity.operation_id;
    assert!(
        write::write_memory(&mut conn, &memory)
            .unwrap_err()
            .contains("idempotency_conflict")
    );
    memory.operation_id = "323e4567-e89b-42d3-a456-000000000001".into();
    write::write_memory(&mut conn, &memory).unwrap();
    let mut entity = entity_request(2, "Bea");
    entity.operation_id = memory.operation_id;
    assert!(
        entity::resolve(&mut conn, &entity)
            .unwrap_err()
            .contains("idempotency_conflict")
    );
}

#[test]
fn phrase_queries_keep_order_accents_and_treat_operators_as_data() {
    let mut conn = database();
    repo::load_fixtures(
        &mut conn,
        &[
            memory(
                "phrase",
                "local",
                "fact",
                "La reunión ocurre en Bogotá central.",
            ),
            memory(
                "separate",
                "local",
                "fact",
                "Bogotá tiene un parque central.",
            ),
        ],
    )
    .unwrap();
    for mode in [QueryMode::Natural, QueryMode::Keywords] {
        let mut query = search("\"bogota central\"");
        query.query_mode = Some(mode);
        query.limit = Some(20);
        let found = service::search(&conn, &query).unwrap();
        assert_eq!(found.candidates.len(), 1);
        assert_eq!(found.candidates[0].memory_id, "phrase");
        for hostile in [
            "\"bogota central",
            "\" OR * : NEAR( ) { }",
            "' ; DROP TABLE memories; --",
            "summary:bogota NOT central",
        ] {
            query.query = hostile.into();
            service::search(&conn, &query).unwrap();
        }
    }
    assert_eq!(
        conn.query_row("SELECT count(*) FROM memories", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        2
    );
}

#[test]
fn bm25_weights_change_ranking_but_not_eligibility() {
    let mut conn = database();
    let mut original = memory("original", "local", "fact", "cobalt");
    original.summary = "plain".into();
    let mut summary = memory("summary", "local", "fact", "plain");
    summary.summary = "cobalt".into();
    repo::load_fixtures(&mut conn, &[original, summary]).unwrap();
    let mut query = search("cobalt");
    query.bm25_weights = Some([10.0, 0.0]);
    assert_eq!(
        service::search(&conn, &query).unwrap().candidates[0].memory_id,
        "original"
    );
    query.bm25_weights = Some([0.0, 10.0]);
    let found = service::search(&conn, &query).unwrap();
    assert_eq!(found.candidates[0].memory_id, "summary");
    assert_eq!(found.coverage.matched_memories, 2);
    for weights in [[0.0, 0.0], [-1.0, 1.0], [f64::NAN, 1.0], [101.0, 1.0]] {
        query.bm25_weights = Some(weights);
        assert!(service::search(&conn, &query).is_err());
    }
}

#[test]
fn coverage_separates_eligible_indexed_matched_limited_and_empty_queries() {
    let mut conn = database();
    repo::load_fixtures(
        &mut conn,
        &[
            memory("one", "local", "fact", "cobalt"),
            memory("two", "local", "fact", "cobalt"),
            memory("unmatched", "local", "fact", "amber"),
            memory("foreign", "other", "fact", "cobalt"),
        ],
    )
    .unwrap();
    let mut query = search("cobalt");
    let found = service::search(&conn, &query).unwrap();
    let coverage = found.coverage;
    assert_eq!(
        (
            coverage.eligible_memories,
            coverage.indexed_memories,
            coverage.matched_memories,
            coverage.returned_memories
        ),
        (3, 3, 2, 1)
    );
    assert!(coverage.truncated);
    assert_eq!(coverage.retrieval, "limited");
    query.limit = Some(20);
    let found = service::search(&conn, &query).unwrap();
    assert_eq!(found.coverage.retrieval, "complete");
    assert_eq!(found.candidates.len(), 2);
    query.query = "no y si".into();
    let empty = service::search(&conn, &query).unwrap();
    assert_eq!(empty.coverage.query, "empty_after_normalization");
    assert_eq!(empty.coverage.retrieval, "not_evaluated");
    query.query = "cobalt".into();
    conn.execute("DELETE FROM search_documents WHERE memory_id='two'", [])
        .unwrap();
    let partial = service::search(&conn, &query).unwrap();
    assert_eq!(partial.status, "degraded");
    assert_eq!(partial.coverage.fts_index, "partial");
    assert_eq!(partial.coverage.indexed_memories, 2);
}

#[test]
fn fts_projection_updates_deletes_and_current_revision_are_consistent() {
    let mut conn = database();
    let mut fixture = memory("one", "local", "fact", "cobalt");
    repo::load_fixtures(&mut conn, &[fixture.clone()]).unwrap();
    fixture.revision = 2;
    fixture.original_text = "amber".into();
    fixture.summary = "amber".into();
    repo::load_fixtures(&mut conn, &[fixture]).unwrap();
    assert!(
        service::search(&conn, &search("cobalt"))
            .unwrap()
            .candidates
            .is_empty()
    );
    assert_eq!(
        service::search(&conn, &search("amber")).unwrap().candidates[0].revision,
        2
    );
    conn.execute(
        "UPDATE search_documents SET original_text='sapphire',summary='sapphire' WHERE revision=2",
        [],
    )
    .unwrap();
    assert!(
        service::search(&conn, &search("amber"))
            .unwrap()
            .candidates
            .is_empty()
    );
    assert_eq!(
        service::search(&conn, &search("sapphire"))
            .unwrap()
            .candidates
            .len(),
        1
    );
    conn.execute(
        "INSERT INTO memory_fts(memory_fts,rank) VALUES ('integrity-check',1)",
        [],
    )
    .unwrap();
    conn.execute("DELETE FROM search_documents WHERE revision=2", [])
        .unwrap();
    assert!(
        service::search(&conn, &search("sapphire"))
            .unwrap()
            .candidates
            .is_empty()
    );
    conn.execute(
        "INSERT INTO memory_fts(memory_fts,rank) VALUES ('integrity-check',1)",
        [],
    )
    .unwrap();
}

#[test]
fn upgrade_preserves_history_and_checksums_and_registers_legacy_entities() {
    struct PriorModule;
    impl AppModule for PriorModule {
        fn name(&self) -> &'static str {
            "retrieval"
        }
        fn migrations(&self) -> Vec<gennie_memory_kernel::core::Migration> {
            RetrievalModule.migrations().into_iter().take(7).collect()
        }
    }
    let mut conn = Connection::open_in_memory().unwrap();
    db::configure(&conn).unwrap();
    db::migrate(&mut conn, &[Box::new(PriorModule)]).unwrap();
    let old_checksums: Vec<String> = conn
        .prepare("SELECT checksum FROM schema_migrations ORDER BY version")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    conn.execute_batch("INSERT INTO memories(id,scope_kind,scope_id,current_revision,entity_id) VALUES ('old','project','demo',2,'known');
        INSERT INTO memory_revisions(memory_id,revision,memory_type,original_text,summary,state,source_kind,source_ref) VALUES ('old',1,'fact','cobalt','cobalt','active','document','original-source');
        INSERT INTO memory_revisions(memory_id,revision,memory_type,original_text,summary,state,source_kind,source_ref) VALUES ('old',2,'fact','amber','amber','active','document','new-source');
        INSERT INTO search_documents(memory_id,revision,original_text,summary) VALUES ('old',1,'cobalt','cobalt'),('old',2,'amber','amber');").unwrap();
    db::migrate(&mut conn, &[Box::new(RetrievalModule)]).unwrap();
    let new_checksums: Vec<String> = conn
        .prepare("SELECT checksum FROM schema_migrations WHERE version<=7 ORDER BY version")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(old_checksums, new_checksums);
    assert_eq!(
        conn.query_row(
            "SELECT source_ref FROM memory_revisions WHERE memory_id='old' AND revision=1",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "original-source"
    );
    assert!(
        service::search(&conn, &search("cobalt"))
            .unwrap()
            .candidates
            .is_empty()
    );
    let found = service::search(&conn, &search("amber")).unwrap();
    assert_eq!(found.candidates[0].revision, 2);
    assert_eq!(
        found.candidates[0].source.as_ref().unwrap().reference,
        "new-source"
    );
    assert_eq!(
        conn.query_row("SELECT actor_id FROM memories WHERE id='old'", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap(),
        "local"
    );
    assert!(
        entity::exists(
            &conn,
            "local",
            &write::Scope {
                kind: "project".into(),
                id: "demo".into()
            },
            "known"
        )
        .unwrap()
    );
    let mut conn = database();
    db::migrate(&mut conn, &[Box::new(RetrievalModule)]).unwrap();
    conn.execute(
        "UPDATE schema_migrations SET checksum='altered' WHERE module='retrieval' AND version=8",
        [],
    )
    .unwrap();
    assert!(
        db::migrate(&mut conn, &[Box::new(RetrievalModule)])
            .unwrap_err()
            .contains("retrieval:8")
    );
}

#[test]
fn validation_and_time_boundaries_do_not_expand_the_result() {
    let mut conn = database();
    let mut expired = memory("expired", "local", "fact", "cobalt");
    expired.valid_until = Some("2026-10-24T12:00:00.000Z".into());
    let mut future = memory("future", "local", "fact", "cobalt");
    future.valid_from = Some("2026-10-24T12:00:00.001Z".into());
    let mut active = memory("active", "local", "fact", "cobalt");
    active.valid_from = Some("2026-10-24T12:00:00.000Z".into());
    repo::load_fixtures(&mut conn, &[expired, future, active]).unwrap();
    let mut query = search("cobalt");
    assert_eq!(
        service::search(&conn, &query).unwrap().candidates[0].memory_id,
        "active"
    );
    query.actor_id = Some(" ".into());
    assert!(service::search(&conn, &query).is_err());
    query.actor_id = None;
    query.memory_types = vec![" ".into()];
    assert!(service::search(&conn, &query).is_err());
}

#[test]
fn entity_failure_rolls_back_identity_alias_operation_and_evidence() {
    let mut conn = database();
    conn.execute_batch("CREATE TRIGGER fail_entity_evidence BEFORE INSERT ON entity_evidence BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    assert!(entity::resolve(&mut conn, &entity_request(1, "Ana")).is_err());
    for table in [
        "entities",
        "entity_aliases",
        "entity_operations",
        "entity_evidence",
    ] {
        let count: i64 = conn
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "{table}");
    }
}

#[test]
fn hybrid_fusion_returns_one_current_memory_per_identity_despite_two_ranks() {
    let mut conn = database();
    let memories = [memory("allowed", "local", "fact", "cobalt")];
    repo::load_fixtures(&mut conn, &memories).unwrap();
    vectors(&mut conn, &memories);
    let found = embedding::search_hybrid(
        &conn,
        &HybridRequest {
            search: search("cobalt"),
            space_id: "test".into(),
            query_vector: vec![1.0, 0.0],
        },
    )
    .unwrap();
    assert_eq!(found.candidates.len(), 1);
    let candidate = &found.candidates[0];
    assert_eq!(candidate.text_rank, Some(1));
    assert_eq!(candidate.semantic_rank, Some(1));
    assert_eq!(candidate.revision, 1);
    assert!(!found.candidate_pool_truncated);
}

#[test]
fn legacy_write_receipts_keep_their_request_hash_and_replay_after_upgrade() {
    #[derive(serde::Serialize)]
    struct LegacyRequest<'a> {
        schema_version: u32,
        operation_id: &'a str,
        action: &'a str,
        memory_id: Option<&'a str>,
        expected_revision: Option<i64>,
        scope: &'a write::Scope,
        memory: &'a write::MemoryInput,
    }
    let mut conn = database();
    let request = write_request(1, None);
    let legacy = LegacyRequest {
        schema_version: request.schema_version,
        operation_id: &request.operation_id,
        action: &request.action,
        memory_id: None,
        expected_revision: None,
        scope: &request.scope,
        memory: &request.memory,
    };
    let hash = gennie_memory_kernel::digest::sha256_hex(&serde_json::to_vec(&legacy).unwrap());
    let written = write::write_memory(&mut conn, &request).unwrap();
    conn.execute(
        "UPDATE memory_write_operations SET request_sha256=?1 WHERE operation_id=?2",
        rusqlite::params![hash, request.operation_id],
    )
    .unwrap();
    let replay = write::write_memory(&mut conn, &request).unwrap();
    assert!(replay.replayed);
    assert_eq!(written.memory_id, replay.memory_id);
}

#[test]
fn fixture_import_after_a_write_cannot_leave_stale_exact_content_keys() {
    let mut conn = database();
    let first = write::write_memory(&mut conn, &write_request(1, None)).unwrap();
    let fixture = memory("imported", "local", "fact", "New imported fact.");
    repo::load_fixtures(&mut conn, &[fixture]).unwrap();
    let mut request = write_request(2, None);
    request.memory.original_text = "New imported fact.".into();
    request.memory.summary = "New imported fact.".into();
    let duplicate = write::write_memory(&mut conn, &request).unwrap();
    assert_eq!(duplicate.outcome, "deduplicated");
    assert_eq!(duplicate.memory_id, "imported");
    assert_ne!(first.memory_id, duplicate.memory_id);
}
