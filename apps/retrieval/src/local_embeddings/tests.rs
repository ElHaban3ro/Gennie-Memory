use super::*;
use crate::{
    model::{FixtureMemory, SearchRequest},
    module::RetrievalModule,
    repo,
    write::Scope,
};
use rusqlite::Connection;
use serde_json::json;

fn profile() -> Profile {
    Profile {
        schema_version: 1,
        model_id: "intfloat/multilingual-e5-small".into(),
        model_revision: "a".repeat(40),
        model_sha256: "b".repeat(64),
        config_sha256: "c".repeat(64),
        tokenizer_sha256: "d".repeat(64),
        license: "mit".into(),
        runtime: RUNTIME.into(),
        dimensions: 384,
        max_tokens: 512,
        query_prefix: "query: ".into(),
        document_prefix: "passage: ".into(),
    }
}
struct Fake(Profile);
impl TextEncoder for Fake {
    fn profile(&self) -> &Profile {
        &self.0
    }
    fn fragments(&self, text: &str) -> Result<Vec<Fragment>, String> {
        Ok(vec![Fragment {
            start_byte: 0,
            end_byte: text.len(),
            text_sha256: sha256_hex(text.as_bytes()),
            token_count: 8,
        }])
    }
    fn embed(&self, text: &str, _: bool, until: Instant) -> Result<Vec<f32>, String> {
        check_deadline(until)?;
        let mut v = vec![0.0; 384];
        v[usize::from(!text.contains("cobalt"))] = 1.0;
        Ok(v)
    }
}
fn database() -> Connection {
    let mut c = Connection::open_in_memory().unwrap();
    gennie_memory_kernel::db::configure(&c).unwrap();
    gennie_memory_kernel::db::migrate(&mut c, &[Box::new(RetrievalModule)]).unwrap();
    c
}
fn memory(id: &str, actor: &str, text: &str) -> FixtureMemory {
    serde_json::from_value(json!({"id":id,"actor_id":actor,"scope_kind":"project","scope_id":"demo","revision":1,"memory_type":"fact","original_text":text,"summary":text,"state":"active","valid_from":null,"valid_until":null})).unwrap()
}
fn spec() -> index::IndexRequest {
    index::IndexRequest {
        scope: Scope {
            kind: "project".into(),
            id: "demo".into(),
        },
        actor_id: None,
        deadline_ms: 30000,
    }
}
fn request(id: &str) -> index::Search {
    index::Search{index_id:id.into(),search:serde_json::from_value::<SearchRequest>(json!({"query":"cobalt","scope_kind":"project","scope_id":"demo","as_of":"2026-10-24T12:00:00.000Z","limit":1})).unwrap(),deadline_ms:30000,max_scored_chunks:100,semantic_only:true}
}

#[test]
fn profile_binds_model_tokenizer_runtime_prefixes_and_dimensions() {
    let p = profile();
    p.validate().unwrap();
    let mut other = p.clone();
    other.tokenizer_sha256 = "e".repeat(64);
    assert_ne!(p.space_id(), other.space_id());
    for change in ["dimension", "runtime", "prefix", "limit", "model"] {
        let mut bad = p.clone();
        match change {
            "dimension" => bad.dimensions = 768,
            "runtime" => bad.runtime = "unverified".into(),
            "prefix" => bad.query_prefix = "".into(),
            "limit" => bad.max_tokens = 1024,
            _ => bad.model_id = "unknown".into(),
        }
        assert!(bad.validate().is_err());
    }
}

#[test]
fn tokenizer_chunks_reconstruct_utf8_and_never_enable_silent_truncation() {
    use tokenizers::{
        models::wordlevel::WordLevel, pre_tokenizers::whitespace::Whitespace,
        processors::bert::BertProcessing,
    };
    let model = WordLevel::builder()
        .vocab(
            [
                ("[CLS]".into(), 0),
                ("[SEP]".into(), 1),
                ("[UNK]".into(), 2),
            ]
            .into_iter()
            .collect(),
        )
        .unk_token("[UNK]".into())
        .build()
        .unwrap();
    let mut tokenizer = Tokenizer::new(model);
    tokenizer.with_pre_tokenizer(Some(Whitespace));
    tokenizer.with_post_processor(Some(BertProcessing::new(
        ("[SEP]".into(), 1),
        ("[CLS]".into(), 0),
    )));
    let mut p = profile();
    p.max_tokens = 16;
    for text in [
        "  Bogotá cafe\u{301} 日本語 😀\n".repeat(20),
        "word ".repeat(100),
        "a\u{200b}b c d".into(),
    ] {
        let parts = fragments(&tokenizer, &p, &text).unwrap();
        assert!(!parts.is_empty());
        assert_eq!(
            parts
                .iter()
                .map(|f| &text[f.start_byte..f.end_byte])
                .collect::<String>(),
            text
        );
        let mut next = 0;
        for f in parts {
            assert_eq!(f.start_byte, next);
            next = f.end_byte;
            assert!(f.token_count <= 16);
            assert_eq!(
                sha256_hex(text[f.start_byte..f.end_byte].as_bytes()),
                f.text_sha256
            );
        }
    }
    assert!(fragments(&tokenizer, &p, "  ").is_err());
}

#[test]
fn complete_publication_filters_deduplicates_and_replays_without_mutation() {
    let mut conn = database();
    let encoder = Fake(profile());
    let mut other = memory("event", "local", "cobalt");
    other.memory_type = "event".into();
    repo::load_fixtures(
        &mut conn,
        &[
            memory("allowed", "local", "cobalt"),
            memory("foreign", "other", "cobalt"),
            other,
        ],
    )
    .unwrap();
    let batch = index::prepare(&conn, &encoder, &spec()).unwrap();
    assert_eq!(batch.sources.len(), 2);
    let receipt = index::publish(&mut conn, &encoder, &batch).unwrap();
    assert!(
        index::publish(&mut conn, &encoder, &batch)
            .unwrap()
            .replayed
    );
    let mut query = request(&receipt.index_id);
    query.search.memory_types = vec!["fact".into()];
    let found = index::search(&conn, &encoder, &query).unwrap();
    assert_eq!(found.candidates.len(), 1);
    assert_eq!(found.candidates[0].memory.memory_id, "allowed");
    assert_eq!(found.scored_chunks, 2);
    query.semantic_only = false;
    assert_eq!(
        index::search(&conn, &encoder, &query).unwrap().candidates[0].text_rank,
        Some(1)
    );
    query.search.actor_id = Some("other".into());
    assert!(
        index::search(&conn, &encoder, &query)
            .unwrap_err()
            .contains("not_available")
    );
}

#[test]
fn incomplete_tampered_or_orphaned_batches_never_publish() {
    let mut conn = database();
    let encoder = Fake(profile());
    repo::load_fixtures(&mut conn, &[memory("one", "local", "cobalt")]).unwrap();
    let batch = index::prepare(&conn, &encoder, &spec()).unwrap();
    for change in [
        "missing",
        "dimension",
        "nan",
        "zero",
        "hash",
        "range",
        "ordinal",
        "orphan",
        "source",
    ] {
        let mut bad = batch.clone();
        match change {
            "missing" => {
                bad.chunks.pop();
            }
            "dimension" => {
                bad.chunks[0].values.pop();
            }
            "nan" => bad.chunks[0].values[0] = f32::NAN,
            "zero" => bad.chunks[0].values.fill(0.0),
            "hash" => bad.chunks[0].fragment.text_sha256 = "wrong".into(),
            "range" => bad.chunks[0].fragment.end_byte -= 1,
            "ordinal" => bad.chunks[0].ordinal = 1,
            "orphan" => bad.chunks[0].memory_id = "other".into(),
            _ => bad.sources[0].original = "changed".into(),
        }
        assert!(
            index::publish(&mut conn, &encoder, &bad).is_err(),
            "{change}"
        );
        assert_eq!(
            conn.query_row("SELECT count(*) FROM local_embedding_indexes", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}

#[test]
fn revision_change_blocks_prepared_publication_and_serving_old_vectors() {
    let mut conn = database();
    let encoder = Fake(profile());
    let mut m = memory("one", "local", "cobalt");
    repo::load_fixtures(&mut conn, &[m.clone()]).unwrap();
    let batch = index::prepare(&conn, &encoder, &spec()).unwrap();
    let receipt = index::publish(&mut conn, &encoder, &batch).unwrap();
    m.revision = 2;
    m.original_text = "amber".into();
    m.summary = "amber".into();
    repo::load_fixtures(&mut conn, &[m]).unwrap();
    assert!(
        index::publish(&mut conn, &encoder, &batch)
            .unwrap_err()
            .contains("snapshot_changed")
    );
    assert!(
        index::search(&conn, &encoder, &request(&receipt.index_id))
            .unwrap_err()
            .contains("stale")
    );
    let fresh = index::prepare(&conn, &encoder, &spec()).unwrap();
    let fresh = index::publish(&mut conn, &encoder, &fresh).unwrap();
    assert_ne!(receipt.index_id, fresh.index_id);
    assert_eq!(
        index::search(&conn, &encoder, &request(&fresh.index_id))
            .unwrap()
            .candidates[0]
            .memory
            .revision,
        2
    );
}

#[test]
fn missing_chunks_wrong_profile_and_work_budget_fail_without_partial_results() {
    let mut conn = database();
    let encoder = Fake(profile());
    repo::load_fixtures(&mut conn, &[memory("one", "local", "cobalt")]).unwrap();
    let batch = index::prepare(&conn, &encoder, &spec()).unwrap();
    let receipt = index::publish(&mut conn, &encoder, &batch).unwrap();
    let mut query = request(&receipt.index_id);
    query.max_scored_chunks = 1;
    assert!(
        index::search(&conn, &encoder, &query)
            .unwrap_err()
            .contains("work_budget_exceeded")
    );
    query.max_scored_chunks = 100;
    let mut other = profile();
    other.model_revision = "e".repeat(40);
    assert!(
        index::search(&conn, &Fake(other), &query)
            .unwrap_err()
            .contains("space_mismatch")
    );
    conn.execute(
        "DELETE FROM local_embedding_chunks WHERE field='summary'",
        [],
    )
    .unwrap();
    assert!(
        index::search(&conn, &encoder, &query)
            .unwrap_err()
            .contains("incomplete")
    );
}

#[test]
fn deadline_and_artifact_absence_are_explicit_errors() {
    assert!(
        check_deadline(Instant::now() - Duration::from_millis(1))
            .unwrap_err()
            .contains("deadline_exceeded")
    );
    assert!(deadline(0).is_err());
    assert!(deadline(300001).is_err());
    let settings = Settings {
        profile: profile(),
        artifact_dir: "/nonexistent-gennie-test-model".into(),
    };
    assert!(
        Encoder::load(&settings)
            .err()
            .unwrap()
            .contains("artifact_unavailable")
    );
}

#[test]
fn publication_sql_failure_rolls_back_profile_index_and_all_chunks() {
    let mut conn = database();
    let encoder = Fake(profile());
    repo::load_fixtures(&mut conn, &[memory("one", "local", "cobalt")]).unwrap();
    let batch = index::prepare(&conn, &encoder, &spec()).unwrap();
    conn.execute_batch("CREATE TRIGGER reject_embedding BEFORE INSERT ON local_embedding_chunks BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    assert!(index::publish(&mut conn, &encoder, &batch).is_err());
    for table in [
        "local_embedding_profiles",
        "local_embedding_indexes",
        "local_embedding_chunks",
    ] {
        assert_eq!(
            conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}

#[test]
fn persisted_profile_or_vector_corruption_is_rejected() {
    let mut conn = database();
    let encoder = Fake(profile());
    repo::load_fixtures(&mut conn, &[memory("one", "local", "cobalt")]).unwrap();
    let batch = index::prepare(&conn, &encoder, &spec()).unwrap();
    let receipt = index::publish(&mut conn, &encoder, &batch).unwrap();
    conn.execute(
        "UPDATE local_embedding_chunks SET vector_blob=x'0000' WHERE field='original'",
        [],
    )
    .unwrap();
    assert!(
        index::search(&conn, &encoder, &request(&receipt.index_id))
            .unwrap_err()
            .contains("vector_format_invalid")
    );
    conn.execute("UPDATE local_embedding_profiles SET profile_json='{}'", [])
        .unwrap();
    assert!(index::search(&conn, &encoder, &request(&receipt.index_id)).is_err());
}

#[test]
fn artifact_hash_mismatch_is_detected_before_model_loading() {
    let path = std::env::temp_dir().join(format!(
        "gennie-model-hash-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&path).unwrap();
    std::fs::write(path.join("config.json"), b"{}").unwrap();
    let error = verified_file(&path, "config.json", &"a".repeat(64)).unwrap_err();
    std::fs::remove_dir_all(path).unwrap();
    assert!(error.contains("hash_mismatch"));
}

#[test]
fn rrf_never_adds_semantic_votes_outside_its_top_twenty() {
    let mut conn = database();
    let encoder = Fake(profile());
    let mut fixtures: Vec<_> = (0..24)
        .map(|n| memory(&format!("a-{n:02}"), "local", "amber"))
        .collect();
    fixtures.push(memory("zz-last", "local", "word"));
    repo::load_fixtures(&mut conn, &fixtures).unwrap();
    let batch = index::prepare(&conn, &encoder, &spec()).unwrap();
    let receipt = index::publish(&mut conn, &encoder, &batch).unwrap();
    let mut query = request(&receipt.index_id);
    query.max_scored_chunks = 100;
    query.search.query = "word".into();
    query.semantic_only = false;
    let found = index::search(&conn, &encoder, &query).unwrap();
    let first = &found.candidates[0];
    assert_eq!(first.memory.memory_id, "zz-last");
    assert_eq!(first.semantic_rank, 25);
    assert_eq!(first.text_rank, Some(1));
    assert!((first.rrf - 1.0 / 61.0).abs() < 1e-12);
}

#[test]
fn entity_and_validity_filters_run_before_scoring_or_limiting_chunks() {
    let mut conn = database();
    let encoder = Fake(profile());
    let mut allowed = memory("zz-allowed", "local", "cobalt");
    allowed.entity_id = Some("person-a".into());
    allowed.valid_from = Some("2026-10-24T12:00:00.000Z".into());
    let mut other = memory("a-other-entity", "local", "cobalt");
    other.entity_id = Some("person-b".into());
    let mut expired = memory("a-expired", "local", "cobalt");
    expired.entity_id = Some("person-a".into());
    expired.valid_until = Some("2026-10-24T12:00:00.000Z".into());
    let mut future = memory("a-future", "local", "cobalt");
    future.entity_id = Some("person-a".into());
    future.valid_from = Some("2026-10-24T12:00:00.001Z".into());
    repo::load_fixtures(&mut conn, &[allowed, other, expired, future]).unwrap();
    let batch = index::prepare(&conn, &encoder, &spec()).unwrap();
    assert_eq!(batch.sources.len(), 4);
    let receipt = index::publish(&mut conn, &encoder, &batch).unwrap();
    let mut query = request(&receipt.index_id);
    query.search.entity_id = Some("person-a".into());
    let found = index::search(&conn, &encoder, &query).unwrap();
    assert_eq!(found.eligible_memories, 1);
    assert_eq!(found.scored_chunks, 2);
    assert_eq!(found.candidates[0].memory.memory_id, "zz-allowed");
}
