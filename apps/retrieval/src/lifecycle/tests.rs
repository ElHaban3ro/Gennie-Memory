use super::*;
fn db() -> Connection {
    let mut c = Connection::open_in_memory().unwrap();
    gennie_memory_kernel::db::configure(&c).unwrap();
    gennie_memory_kernel::db::migrate(&mut c, &[Box::new(crate::module::RetrievalModule)]).unwrap();
    let f: Vec<crate::model::FixtureMemory>=serde_json::from_value(json!([{"id":"one","actor_id":"local","scope_kind":"project","scope_id":"demo","revision":1,"memory_type":"fact","original_text":"cobalt","summary":"cobalt","state":"active"}])).unwrap();
    crate::repo::load_fixtures(&mut c, &f).unwrap();
    c
}
fn scope() -> Scope {
    Scope {
        kind: "project".into(),
        id: "demo".into(),
    }
}
fn req(action: &str, n: u8) -> LifecycleRequest {
    LifecycleRequest {
        schema_version: 1,
        operation_id: format!("123e4567-e89b-42d3-a456-4266141740{n:02}"),
        action: action.into(),
        memory_id: "one".into(),
        expected_revision: 1,
        scope: scope(),
        as_of: "2026-10-24T12:00:00.000Z".into(),
        next_review_at: None,
        evidence: Some(serde_json::from_value(json!({"original_text":"The operator explicitly requested this lifecycle action.","source":{"kind":"user","ref":"synthetic-request"}})).unwrap()),
    }
}
fn search(c: &Connection) -> usize {
    let r=serde_json::from_value(json!({"query":"cobalt","scope_kind":"project","scope_id":"demo","as_of":"2026-10-24T12:00:00.000Z","limit":20})).unwrap();
    crate::service::search(c, &r).unwrap().candidates.len()
}
fn jobreq(action: &str) -> JobRequest {
    JobRequest {
        action: action.into(),
        scope: scope(),
        space_id: "test-space".into(),
    }
}
#[test]
fn withdrawal_restore_delete_and_replay_preserve_visibility() {
    let mut c = db();
    assert_eq!(search(&c), 1);
    let r = req("withdraw", 1);
    apply(&mut c, &r).unwrap();
    assert_eq!(search(&c), 0);
    assert_eq!(apply(&mut c, &r).unwrap()["replayed"], true);
    assert!(
        c.execute("UPDATE memories SET current_revision=2 WHERE id='one'", [])
            .is_err()
    );
    assert!(
        apply(&mut c, &req("restore", 9))
            .unwrap_err()
            .contains("revision_conflict")
    );
    let mut restore = req("restore", 2);
    restore.expected_revision = 2;
    assert_eq!(apply(&mut c, &restore).unwrap()["revision"], 3);
    assert_eq!(search(&c), 1);
    let mut delete = req("delete", 3);
    delete.expected_revision = 3;
    apply(&mut c, &delete).unwrap();
    assert_eq!(search(&c), 0);
    for table in [
        "memory_revisions",
        "search_documents",
        "memory_review_schedule",
        "memory_source_evidence",
    ] {
        let n: i64 = c
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0, "{table}");
    }
    assert_eq!(apply(&mut c, &delete).unwrap()["replayed"], true);
    assert!(apply(&mut c, &req("restore", 4)).is_err());
}
#[test]
fn review_due_stale_scope_and_generation() {
    let mut c = db();
    let list = ReviewListRequest {
        scope: scope(),
        as_of: req("review", 1).as_of,
        limit: 10,
        after_id: None,
    };
    assert_eq!(
        review_list(&c, &list).unwrap()["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let mut r = req("review", 1);
    r.next_review_at = Some("2026-11-24T12:00:00.000Z".into());
    apply(&mut c, &r).unwrap();
    assert!(
        review_list(&c, &list).unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let mut wrong = req("withdraw", 2);
    wrong.scope.id = "other".into();
    assert!(
        apply(&mut c, &wrong)
            .unwrap_err()
            .contains("memory_not_found")
    );
    wrong = req("withdraw", 3);
    wrong.expected_revision = 2;
    assert!(
        apply(&mut c, &wrong)
            .unwrap_err()
            .contains("revision_conflict")
    );
    assert_eq!(search(&c), 1);
    job(&mut c, &jobreq("enqueue")).unwrap();
    let a = claim(&mut c, &jobreq("get")).unwrap();
    assert!(claim(&mut c, &jobreq("get")).is_err());
    apply(&mut c, &req("withdraw", 4)).unwrap();
    let b = claim(&mut c, &jobreq("get")).unwrap();
    assert!(b.generation > a.generation);
    job(&mut c, &jobreq("cancel")).unwrap();
    assert!(claim(&mut c, &jobreq("get")).is_err());
    job(&mut c, &jobreq("retry")).unwrap();
    assert!(claim(&mut c, &jobreq("get")).is_ok());
}
#[test]
fn abandoned_lease_recovers_with_new_fence() {
    let mut c = db();
    job(&mut c, &jobreq("enqueue")).unwrap();
    let a = claim(&mut c, &jobreq("get")).unwrap();
    c.execute(
        "UPDATE embedding_jobs SET lease_until='2000-01-01T00:00:00.000Z'",
        [],
    )
    .unwrap();
    let b = claim(&mut c, &jobreq("get")).unwrap();
    assert_eq!(a.generation, b.generation);
    assert_ne!(a.lease_token, b.lease_token);
}

#[test]
fn deletion_purges_dependents_and_rolls_back_as_one_operation() {
    let mut c = db();
    c.execute("INSERT INTO memory_write_operations(operation_id,request_sha256,memory_id,revision,committed_at) VALUES('write-id',?1,'one',1,'2026-10-24T00:00:00.000Z')",["a".repeat(64)]).unwrap();
    c.execute("INSERT INTO memory_proposals VALUES('proposal','local','project','demo','sensitive proposed text','reason','2026-10-24T00:00:00.000Z')",[]).unwrap();
    c.execute("INSERT INTO memory_proposal_targets VALUES('proposal','one',1,'contradiction','reason','sensitive evidence')",[]).unwrap();
    c.execute("INSERT INTO memory_evidence_audit(operation_id,actor_id,scope_kind,scope_id,memory_id,revision,proposal_id,action,evidence_json,evidence_sha256,provenance_json,recorded_at) VALUES('audit-op','local','project','demo','one',1,'proposal','propose','sensitive evidence',?1,'sensitive provenance','2026-10-24T00:00:00.000Z')",["b".repeat(64)]).unwrap();
    c.execute("INSERT INTO source_verification_checks VALUES(?1,'one',1,NULL,'rejected','test','sensitive report','2026-10-24T00:00:00.000Z')",["c".repeat(64)]).unwrap();
    c.execute("INSERT INTO reconciliation_operations VALUES('audit-op',?1,'{\"memory_id\":\"one\",\"reason\":\"sensitive text\"}','2026-10-24T00:00:00.000Z')",["d".repeat(64)]).unwrap();
    c.execute_batch("CREATE TRIGGER reject_test_delete BEFORE DELETE ON memories BEGIN SELECT RAISE(ABORT,'injected_failure'); END;").unwrap();
    assert!(apply(&mut c, &req("delete", 1)).is_err());
    assert_eq!(search(&c), 1);
    let n: i64 = c
        .query_row("SELECT count(*) FROM memory_proposals", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
    c.execute_batch("DROP TRIGGER reject_test_delete;").unwrap();
    apply(&mut c, &req("delete", 1)).unwrap();
    for t in [
        "memory_proposals",
        "memory_proposal_targets",
        "memory_evidence_audit",
        "source_verification_checks",
        "memory_write_operations",
    ] {
        let n: i64 = c
            .query_row(&format!("SELECT count(*) FROM {t}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0, "{t}");
    }
    let receipt: String = c
        .query_row(
            "SELECT response_json FROM reconciliation_operations WHERE operation_id='audit-op'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!receipt.contains("sensitive"));
}

#[test]
fn erased_legacy_write_cannot_resurrect_on_retry() {
    let mut c = db();
    let w:crate::write::WriteRequest=serde_json::from_value(json!({"schema_version":1,"operation_id":"123e4567-e89b-42d3-a456-426614174090","action":"create","scope":{"kind":"project","id":"demo"},"memory":{"type":"fact","original_text":"sensitive unique content","summary":"unique","source":{"kind":"document","ref":"synthetic"}}})).unwrap();
    let saved = crate::write::write_memory(&mut c, &w).unwrap();
    let mut r = req("delete", 91);
    r.memory_id = saved.memory_id;
    apply(&mut c, &r).unwrap();
    assert!(
        crate::write::write_memory(&mut c, &w)
            .unwrap_err()
            .contains("content_deleted")
    );
    let n: i64 = c
        .query_row("SELECT count(*) FROM memories", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
}
