use super::*;
use crate::{
    classifier::{Classifier, DecisionError},
    model::{Candidate, ClassifierProvider, DecisionResult, SelectionPolicy},
    module::RetrievalModule,
    source_verification,
    write::Scope,
};
use gennie_memory_kernel::db;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::cell::Cell;
struct Judge {
    calls: Cell<usize>,
    invalid: bool,
}
impl Judge {
    fn new() -> Self {
        Self {
            calls: Cell::new(0),
            invalid: false,
        }
    }
}
impl Classifier for Judge {
    fn classify(
        &self,
        _: &str,
        _: &str,
        _: &[Candidate],
        _: ClassifierProvider,
        _: SelectionPolicy,
    ) -> Result<DecisionResult, DecisionError> {
        panic!("recall not expected")
    }
    fn classify_relations(
        &self,
        m: &crate::write::MemoryInput,
        cs: &[RelationCandidate],
        _: ClassifierProvider,
    ) -> Result<RelationReport, DecisionError> {
        self.calls.set(self.calls.get() + 1);
        Ok(RelationReport {
            judgments: cs
                .iter()
                .map(|c| RelationJudgment {
                    target: c.target.clone(),
                    relation: Relation::Contradiction,
                    reason: "mock declared incompatible claim".into(),
                    incoming_quote: if self.invalid {
                        "invented quote".into()
                    } else {
                        m.original_text.clone()
                    },
                    existing_quote: c.memory.original_text.clone(),
                })
                .collect(),
            usage: Some(json!({"reported_cost_usd":0.001})),
        })
    }
}
fn database() -> Connection {
    let mut c = Connection::open_in_memory().unwrap();
    db::configure(&c).unwrap();
    db::migrate(&mut c, &[Box::new(RetrievalModule)]).unwrap();
    c
}
fn op(i: u32) -> String {
    format!("123e4567-e89b-42d3-a456-{i:012}")
}
fn command(i: u32, text: &str) -> WriteCommand {
    serde_json::from_value(json!({"schema_version":1,"operation_id":op(i),"action":"create","scope":{"kind":"project","id":"demo"},"memory":{"original_text":text,"summary":text,"type":"preference","source":{"kind":"user","ref":format!("msg:{i}")}}})).unwrap()
}
fn register(c: &Connection, m: &crate::write::MemoryInput) {
    source_verification::register(
        c,
        &source_verification::RegisterRequest {
            schema_version: 1,
            actor_id: None,
            scope: Scope {
                kind: "project".into(),
                id: "demo".into(),
            },
            source: crate::model::SourceRef {
                locator: None,
                ..m.source.clone()
            },
            version: "v1".into(),
            text: m.original_text.clone(),
        },
    )
    .unwrap();
}
fn evidence(c: &Connection, text: &str) -> Evidence {
    let m = command(90, text).memory.unwrap();
    register(c, &m);
    Evidence {
        original_text: m.original_text,
        source: m.source,
    }
}
fn inspect(id: &str) -> InspectRequest {
    serde_json::from_value(
        json!({"schema_version":1,"scope":{"kind":"project","id":"demo"},"memory_id":id}),
    )
    .unwrap()
}
fn resolution(
    c: &Connection,
    proposal: &Value,
    targets: Vec<Target>,
    decision: &str,
) -> ResolveCommand {
    ResolveCommand {
        schema_version: 1,
        operation_id: op(50),
        proposal_id: proposal["proposal_id"].as_str().unwrap().into(),
        expected_targets: targets,
        decision: decision.into(),
        evidence: evidence(c, "Please apply this correction."),
        target_memory_id: None,
        scope: None,
        memory: None,
        topic_key: None,
    }
}
fn count(c: &Connection, table: &str) -> i64 {
    c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn committed_writes_replay_receipts_and_audit_sources_without_new_calls() {
    let mut c = database();
    let j = Judge::new();
    let request = command(1, "Use blue paper.");
    let first = write(&mut c, &request, &j).unwrap();
    assert_eq!(first["status"], "committed");
    assert_eq!(first["source_check"]["status"], "declared_unverified");
    let again = write(&mut c, &request, &j).unwrap();
    assert_eq!(again["memory_id"], first["memory_id"]);
    assert_eq!(again["replayed"], true);
    assert_eq!(count(&c, "memory_evidence_audit"), 1);
    let mut changed = request.clone();
    changed.memory.as_mut().unwrap().summary = "Other summary".into();
    assert!(
        write(&mut c, &changed, &j)
            .unwrap_err()
            .contains("idempotency_conflict")
    );
    assert_eq!(j.calls.get(), 0);
    let rows = audit(&c, &inspect(first["memory_id"].as_str().unwrap())).unwrap();
    assert_eq!(
        rows["entries"][0]["evidence"]["original_text"],
        "Use blue paper."
    );
    let mut inspect = inspect(first["memory_id"].as_str().unwrap());
    inspect.operation_id = Some(op(1));
    assert_eq!(operation_get(&c, &inspect).unwrap(), first);
    c.execute("UPDATE memory_evidence_audit SET evidence_json='{}'", [])
        .unwrap();
    assert!(audit(&c, &inspect).unwrap_err().contains("integrity"));
}
#[test]
fn exact_fingerprint_retains_different_source_and_does_not_merge_negation() {
    let mut c = database();
    let j = Judge::new();
    let first = write(&mut c, &command(1, "Use blue paper."), &j).unwrap();
    let second = write(&mut c, &command(2, "Use blue paper."), &j).unwrap();
    assert_eq!(second["outcome"], "deduplicated");
    assert_eq!(second["memory_id"], first["memory_id"]);
    assert_eq!(count(&c, "memory_source_evidence"), 2);
    write(&mut c, &command(3, "Do not use blue paper."), &j).unwrap();
    assert_eq!(count(&c, "memories"), 2);
}
#[test]
fn strict_unverified_source_stays_out_of_fts_and_can_be_resolved_after_snapshot() {
    let mut c = database();
    let j = Judge::new();
    let mut r = command(1, "Use blue paper.");
    r.source_policy = source_verification::SourcePolicy::RequireVerified;
    let p = write(&mut c, &r, &j).unwrap();
    assert_eq!(p["status"], "needs_resolution");
    assert_eq!(count(&c, "memories"), 0);
    assert_eq!(count(&c, "search_documents"), 0);
    assert_eq!(write(&mut c, &r, &j).unwrap()["replayed"], true);
    let mut resolution = resolution(&c, &p, vec![], "create_separate");
    resolution.scope = r.scope.clone();
    resolution.memory = r.memory.clone();
    assert!(
        resolve(&mut c, &resolution)
            .unwrap_err()
            .contains("unverified_source")
    );
    assert_eq!(count(&c, "memory_proposals"), 1);
    register(&c, r.memory.as_ref().unwrap());
    let accepted = resolve(&mut c, &resolution).unwrap();
    assert_eq!(accepted["status"], "committed");
    assert_eq!(count(&c, "memory_proposals"), 0);
    assert_eq!(count(&c, "memories"), 1);
    assert_eq!(resolve(&mut c, &resolution).unwrap()["replayed"], true);
    assert!(
        audit(&c, &inspect(accepted["memory_id"].as_str().unwrap())).unwrap()["entries"]
            .as_array()
            .unwrap()
            .len()
            >= 3
    );
}
#[test]
fn topic_conflict_requires_all_targets_and_an_explicit_revision() {
    let mut c = database();
    let j = Judge::new();
    let mut a = command(1, "Use blue paper.");
    a.topic_key = Some("Paper/Color".into());
    let first = write(&mut c, &a, &j).unwrap();
    let mut b = command(2, "Use red paper.");
    b.topic_key = Some("paper/color".into());
    let p = write(&mut c, &b, &j).unwrap();
    assert_eq!(p["reason"], "topic_conflict");
    assert_eq!(count(&c, "memories"), 1);
    let target = Target {
        memory_id: first["memory_id"].as_str().unwrap().into(),
        revision: 1,
    };
    let mut r = resolution(&c, &p, vec![], "replace_existing");
    r.target_memory_id = Some(target.memory_id.clone());
    r.memory = b.memory.clone();
    register(&c, r.memory.as_ref().unwrap());
    assert!(
        resolve(&mut c, &r)
            .unwrap_err()
            .contains("expected_targets_mismatch")
    );
    r.expected_targets = vec![target.clone()];
    let result = resolve(&mut c, &r).unwrap();
    assert_eq!(result["revision"], 2);
    assert_eq!(count(&c, "memory_relations"), 1);
    assert_eq!(count(&c, "memory_proposals"), 0);
    let old: String = c
        .query_row(
            "SELECT original_text FROM memory_revisions WHERE memory_id=?1 AND revision=1",
            [&target.memory_id],
            |v| v.get(0),
        )
        .unwrap();
    assert_eq!(old, "Use blue paper.");
}
#[test]
fn stale_proposals_require_current_targets_and_resolution_rollback_is_complete() {
    let mut c = database();
    let j = Judge::new();
    let mut a = command(1, "Use blue paper.");
    a.topic_key = Some("paper/color".into());
    let first = write(&mut c, &a, &j).unwrap();
    let mut b = command(2, "Use red paper.");
    b.topic_key = a.topic_key.clone();
    let p = write(&mut c, &b, &j).unwrap();
    let mut revision = command(3, "Use green paper.");
    revision.action = "revise".into();
    revision.memory_id = Some(first["memory_id"].as_str().unwrap().into());
    revision.expected_revision = Some(1);
    write(&mut c, &revision, &j).unwrap();
    let mut r = resolution(
        &c,
        &p,
        vec![Target {
            memory_id: revision.memory_id.clone().unwrap(),
            revision: 1,
        }],
        "keep_existing",
    );
    assert!(
        resolve(&mut c, &r)
            .unwrap_err()
            .contains("revision_conflict")
    );
    assert_eq!(count(&c, "memory_proposals"), 1);
    r.expected_targets[0].revision = 2;
    resolve(&mut c, &r).unwrap();
    assert_eq!(count(&c, "memory_proposals"), 0);
    assert_eq!(count(&c, "memory_proposal_targets"), 0);
    let original_payload: String = c
        .query_row(
            "SELECT response_json FROM reconciliation_operations WHERE operation_id=?1",
            [op(2)],
            |v| v.get(0),
        )
        .unwrap();
    assert!(!original_payload.contains("Use red paper."));
}
#[test]
fn confirm_preserves_content_and_source_and_advances_revision_with_separate_evidence() {
    let mut c = database();
    let j = Judge::new();
    let first = write(&mut c, &command(1, "Use blue paper."), &j).unwrap();
    let id = first["memory_id"].as_str().unwrap();
    c.execute("UPDATE memory_review_schedule SET next_review_at='2030-01-01T00:00:00.000Z' WHERE memory_id=?1",[id]).unwrap();
    let e = evidence(&c, "I still prefer blue paper.");
    let confirm:WriteCommand=serde_json::from_value(json!({"schema_version":1,"operation_id":op(2),"action":"confirm","memory_id":id,"expected_revision":1,"evidence":e})).unwrap();
    let result = write(&mut c, &confirm, &j).unwrap();
    assert_eq!(result["outcome"], "confirmed");
    assert_eq!(result["revision"], 2);
    let row:(String,String,String)=c.query_row("SELECT r.original_text,r.source_ref,s.next_review_at FROM memory_revisions r JOIN memory_review_schedule s USING(memory_id,revision) WHERE r.memory_id=?1 AND r.revision=2",[id],|v|Ok((v.get(0)?,v.get(1)?,v.get(2)?))).unwrap();
    assert_eq!(
        row,
        (
            "Use blue paper.".into(),
            "msg:1".into(),
            "2030-01-01T00:00:00.000Z".into()
        )
    );
    assert!(
        write(
            &mut c,
            &WriteCommand {
                operation_id: op(3),
                ..confirm
            },
            &j
        )
        .unwrap_err()
        .contains("revision_conflict")
    );
}
#[test]
fn classifier_judgments_never_replace_and_invalid_quotes_degrade_to_review() {
    let mut c = database();
    let j = Judge::new();
    write(&mut c, &command(1, "Use blue paper."), &j).unwrap();
    let mut b = command(2, "Use red paper.");
    b.relation_provider = Some(ClassifierProvider::Decisions);
    let p = write(&mut c, &b, &j).unwrap();
    assert_eq!(j.calls.get(), 1);
    assert_eq!(p["status"], "needs_resolution");
    assert_eq!(p["relations"][0]["relation"], "contradiction");
    assert_eq!(count(&c, "memories"), 1);
    let mut bad = Judge::new();
    bad.invalid = true;
    b.operation_id = op(3);
    let p = write(&mut c, &b, &bad).unwrap();
    assert_eq!(p["classification_error"], "model_output_invalid");
    assert_eq!(p["relations"][0]["relation"], "undetermined");
}
#[test]
fn transaction_failure_rolls_back_revision_fts_evidence_and_receipt() {
    let mut c = database();
    let j = Judge::new();
    c.execute_batch("CREATE TRIGGER reject_audit BEFORE INSERT ON memory_evidence_audit BEGIN SELECT RAISE(ABORT,'injected fault'); END;").unwrap();
    assert!(write(&mut c, &command(1, "Use blue paper."), &j).is_err());
    assert_eq!(count(&c, "memories"), 0);
    assert_eq!(count(&c, "search_documents"), 0);
    assert_eq!(count(&c, "memory_write_operations"), 0);
    assert_eq!(count(&c, "reconciliation_operations"), 0);
}
#[test]
fn operation_namespace_entity_and_scope_are_not_client_controlled() {
    let mut c = database();
    let j = Judge::new();
    let mut a = command(1, "Use blue paper.");
    a.topic_key = Some("paper/color".into());
    let first = write(&mut c, &a, &j).unwrap();
    let mut r = inspect(first["memory_id"].as_str().unwrap());
    r.scope.id = "other".into();
    assert!(audit(&c, &r).is_err());
    let entity=serde_json::from_value(json!({"schema_version":1,"operation_id":op(1),"scope":{"kind":"project","id":"demo"},"kind":"person","label":"Ana","source":{"kind":"user","ref":"msg:1"}})).unwrap();
    assert!(
        crate::entity::resolve(&mut c, &entity)
            .unwrap_err()
            .contains("idempotency_conflict")
    );
    let mut json = serde_json::to_value(&a).unwrap();
    json["actor_id"] = json!("other");
    assert!(serde_json::from_value::<WriteCommand>(json).is_err());
    a.operation_id = op(2);
    a.memory.as_mut().unwrap().summary = "Updated summary".into();
    let p = write(&mut c, &a, &j).unwrap();
    let mut get = inspect("");
    get.memory_id = None;
    get.proposal_id = Some(p["proposal_id"].as_str().unwrap().into());
    get.scope.id = "other".into();
    assert!(proposal_get(&c, &get).is_err());
}

#[test]
fn pending_proposal_is_visible_as_uncertainty_until_explicit_discard() {
    let mut c = database();
    let j = Judge::new();
    let mut a = command(1, "Use blue paper.");
    a.topic_key = Some("paper/color".into());
    let first = write(&mut c, &a, &j).unwrap();
    let mut b = command(2, "Use red paper.");
    b.topic_key = a.topic_key.clone();
    let p = write(&mut c, &b, &j).unwrap();
    let q:crate::model::SearchRequest=serde_json::from_value(json!({"query":"paper","scope_kind":"project","scope_id":"demo","as_of":"2030-01-01T00:00:00.000Z","limit":20})).unwrap();
    let search = crate::service::search(&c, &q).unwrap();
    assert_eq!(search.candidates[0].state, "uncertain");
    let selection = json!([{"memory_id":first["memory_id"],"revision":1}]);
    let seed_request=serde_json::from_value(json!({"search":q,"presented":selection,"selected":selection,"classifier":"mock","classification_status":"complete"})).unwrap();
    let seed = crate::service::compose_seed(&c, &seed_request).unwrap();
    assert!(seed.seed_text.unwrap().contains("propuesta pendiente"));
    let r = resolution(
        &c,
        &p,
        vec![Target {
            memory_id: first["memory_id"].as_str().unwrap().into(),
            revision: 1,
        }],
        "keep_existing",
    );
    resolve(&mut c, &r).unwrap();
    assert_eq!(
        crate::service::search(&c, &q).unwrap().candidates[0].state,
        "active"
    );
}

#[test]
fn confirm_omitted_and_null_schedule_are_distinct_idempotent_requests() {
    let mut c = database();
    let j = Judge::new();
    let first = write(&mut c, &command(1, "Use blue paper."), &j).unwrap();
    let id = first["memory_id"].as_str().unwrap();
    let e = evidence(&c, "Still blue paper.");
    let base = json!({"schema_version":1,"operation_id":op(2),"action":"confirm","memory_id":id,"expected_revision":1,"evidence":e});
    let omitted: WriteCommand = serde_json::from_value(base.clone()).unwrap();
    write(&mut c, &omitted, &j).unwrap();
    let mut explicit = base;
    explicit["review_after"] = Value::Null;
    let clear: WriteCommand = serde_json::from_value(explicit).unwrap();
    assert!(
        write(&mut c, &clear, &j)
            .unwrap_err()
            .contains("idempotency_conflict")
    );
    let mut next = clear;
    next.operation_id = op(3);
    next.expected_revision = Some(2);
    write(&mut c, &next, &j).unwrap();
    let scheduled: Option<String> = c
        .query_row(
            "SELECT next_review_at FROM memory_review_schedule WHERE memory_id=?1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(scheduled.is_none());
}

#[test]
fn relation_transport_holds_no_transaction_and_rechecks_concurrent_revisions() {
    struct ConcurrentWriter {
        path: std::path::PathBuf,
        memory_id: String,
    }
    impl Classifier for ConcurrentWriter {
        fn classify(
            &self,
            _: &str,
            _: &str,
            _: &[Candidate],
            _: ClassifierProvider,
            _: SelectionPolicy,
        ) -> Result<DecisionResult, DecisionError> {
            unreachable!()
        }
        fn classify_relations(
            &self,
            m: &crate::write::MemoryInput,
            cs: &[RelationCandidate],
            p: ClassifierProvider,
        ) -> Result<RelationReport, DecisionError> {
            // A second connection is test-only, simulating another process during I/O.
            let mut external = Connection::open(&self.path).unwrap();
            db::configure(&external).unwrap();
            let mut update = command(3, "Use green paper.");
            update.action = "revise".into();
            update.memory_id = Some(self.memory_id.clone());
            update.expected_revision = Some(1);
            let request = super::storage::legacy(&update, update.scope.as_ref().unwrap()).unwrap();
            crate::write::write_memory(&mut external, &request).unwrap();
            Judge::new().classify_relations(m, cs, p)
        }
    }
    let path =
        std::env::temp_dir().join(format!("gennie-write-race-{}.sqlite", std::process::id()));
    assert!(!path.exists());
    let mut c = Connection::open(&path).unwrap();
    db::configure(&c).unwrap();
    db::migrate(&mut c, &[Box::new(RetrievalModule)]).unwrap();
    let mut first = command(1, "Use blue paper.");
    first.topic_key = Some("paper/color".into());
    let created = write(&mut c, &first, &Judge::new()).unwrap();
    let id = created["memory_id"].as_str().unwrap().to_owned();
    let mut incoming = command(2, "Use red paper.");
    incoming.topic_key = first.topic_key;
    incoming.relation_provider = Some(ClassifierProvider::Decisions);
    let writer = ConcurrentWriter {
        path: path.clone(),
        memory_id: id.clone(),
    };
    let result = write(&mut c, &incoming, &writer);
    assert!(result.unwrap_err().contains("revision_conflict"));
    assert_eq!(count(&c, "memory_proposals"), 0);
    assert_eq!(count(&c, "memories"), 1);
    let revision: i64 = c
        .query_row(
            "SELECT current_revision FROM memories WHERE id=?1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(revision, 2);
    drop(c);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn audit_accepts_exact_evidence_bytes_written_by_lifecycle() {
    let mut c = database();
    let first = write(&mut c, &command(1, "Use blue paper."), &Judge::new()).unwrap();
    let id = first["memory_id"].as_str().unwrap();
    let request=serde_json::from_value(json!({"schema_version":1,"operation_id":op(2),"action":"withdraw","memory_id":id,"expected_revision":1,"scope":{"kind":"project","id":"demo"},"as_of":"2030-01-01T00:00:00.000Z","evidence":{"original_text":"Please withdraw this preference.","source":{"kind":"user","ref":"control:1"}}})).unwrap();
    crate::lifecycle::apply(&mut c, &request).unwrap();
    let report = audit(&c, &inspect(id)).unwrap();
    assert_eq!(report["entries"][1]["action"], "withdraw");
    assert!(
        write(&mut c, &command(3, "Use blue paper."), &Judge::new())
            .unwrap_err()
            .contains("memory_withdrawn")
    );
}
