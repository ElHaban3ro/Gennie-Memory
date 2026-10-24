use super::*;
use crate::{lifecycle, module::RetrievalModule, write::Scope};
use rusqlite::Connection;
use serde_json::json;
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
    fn embed(&self, _: &str, _: bool, _: Instant) -> Result<Vec<f32>, String> {
        let mut v = vec![0.; 384];
        v[0] = 1.;
        Ok(v)
    }
}
fn encoder() -> Fake {
    Fake(Profile {
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
    })
}
fn setup() -> (Connection, Fake, index::IndexRequest, lifecycle::JobRequest) {
    let mut c = Connection::open_in_memory().unwrap();
    gennie_memory_kernel::db::configure(&c).unwrap();
    gennie_memory_kernel::db::migrate(&mut c, &[Box::new(RetrievalModule)]).unwrap();
    let f:Vec<crate::model::FixtureMemory>=serde_json::from_value(json!([{"id":"one","scope_kind":"project","scope_id":"demo","revision":1,"memory_type":"fact","original_text":"cobalt","summary":"cobalt","state":"active"}])).unwrap();
    crate::repo::load_fixtures(&mut c, &f).unwrap();
    let e = encoder();
    let r = index::IndexRequest {
        scope: Scope {
            kind: "project".into(),
            id: "demo".into(),
        },
        actor_id: None,
        deadline_ms: 30000,
    };
    let j = lifecycle::JobRequest {
        action: "enqueue".into(),
        scope: r.scope.clone(),
        space_id: e.profile().space_id(),
    };
    lifecycle::job(&mut c, &j).unwrap();
    (c, e, r, j)
}
#[test]
fn cancelled_or_superseded_jobs_never_publish_prepared_vectors() {
    let (mut c, e, r, mut j) = setup();
    let claim = lifecycle::claim(&mut c, &j).unwrap();
    let batch = index::prepare(&c, &e, &r).unwrap();
    j.action = "cancel".into();
    lifecycle::job(&mut c, &j).unwrap();
    assert!(
        index::publish_guarded(&mut c, &e, &batch, Some(&claim))
            .unwrap_err()
            .contains("job_cancelled")
    );
    let n: i64 = c
        .query_row("SELECT count(*) FROM local_embedding_indexes", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(n, 0);
    j.action = "retry".into();
    lifecycle::job(&mut c, &j).unwrap();
    let current = lifecycle::claim(&mut c, &j).unwrap();
    assert!(current.generation > claim.generation);
    // A stale worker must not mark a replacement failed.
    lifecycle::fail(&c, &claim).unwrap();
    j.action = "get".into();
    assert_eq!(lifecycle::job(&mut c, &j).unwrap()["status"], "running");
    let result = index::publish_guarded(&mut c, &e, &batch, Some(&current)).unwrap();
    assert_eq!(result.memories, 1);
    assert_eq!(lifecycle::job(&mut c, &j).unwrap()["status"], "completed");
}
#[test]
fn worker_publishes_complete_scope_then_handles_empty_scope() {
    let (mut c, e, r, mut j) = setup();
    let receipt = index::process_job(&mut c, &e, &r).unwrap();
    assert_eq!(receipt["chunks"], 2);
    c.execute(
        "UPDATE memories SET lifecycle_state='withdrawn' WHERE id='one'",
        [],
    )
    .unwrap();
    assert_eq!(
        index::process_job(&mut c, &e, &r).unwrap()["status"],
        "completed_empty"
    );
    j.action = "get".into();
    let done = lifecycle::job(&mut c, &j).unwrap();
    assert_eq!(done["status"], "completed");
    assert!(done["index_id"].is_null());
    let n: i64 = c
        .query_row("SELECT count(*) FROM local_embedding_chunks", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(n, 0);
}
#[test]
fn expired_lease_fences_publication_and_new_claim_recovers() {
    let (mut c, e, r, j) = setup();
    let old = lifecycle::claim(&mut c, &j).unwrap();
    let batch = index::prepare(&c, &e, &r).unwrap();
    c.execute(
        "UPDATE embedding_jobs SET lease_until='2000-01-01T00:00:00.000Z'",
        [],
    )
    .unwrap();
    assert!(index::publish_guarded(&mut c, &e, &batch, Some(&old)).is_err());
    let new = lifecycle::claim(&mut c, &j).unwrap();
    assert_ne!(new.lease_token, old.lease_token);
    index::publish_guarded(&mut c, &e, &batch, Some(&new)).unwrap();
}

#[test]
fn restoring_same_content_cannot_publish_an_older_generation() {
    let (mut c, e, r, j) = setup();
    let claim = lifecycle::claim(&mut c, &j).unwrap();
    let batch = index::prepare(&c, &e, &r).unwrap();
    c.execute(
        "UPDATE memories SET lifecycle_state='withdrawn' WHERE id='one'",
        [],
    )
    .unwrap();
    c.execute(
        "UPDATE memories SET lifecycle_state='active' WHERE id='one'",
        [],
    )
    .unwrap();
    assert!(
        index::publish_guarded(&mut c, &e, &batch, Some(&claim))
            .unwrap_err()
            .contains("job_cancelled")
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM local_embedding_indexes", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert!(index::process_job(&mut c, &e, &r).is_ok());
}
