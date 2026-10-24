use super::*;
use crate::model::FixtureMemory;
use serde_json::json;

fn fixture() -> FixtureMemory {
    serde_json::from_value(json!({"id":"m1","scope_kind":"project","scope_id":"demo","revision":1,"memory_type":"fact","original_text":"No enviar si llueve en Bogotá 😀.","summary":"La entrega depende del clima.","state":"active","source":{"kind":"document","ref":"synthetic-weather"}})).unwrap()
}
fn setup() -> (Connection, RegisterRequest, VerifyRequest) {
    let mut conn = Connection::open_in_memory().unwrap();
    gennie_memory_kernel::db::configure(&conn).unwrap();
    let modules: Vec<Box<dyn gennie_memory_kernel::core::AppModule>> =
        vec![Box::new(crate::module::RetrievalModule)];
    gennie_memory_kernel::db::migrate(&mut conn, &modules).unwrap();
    crate::repo::load_fixtures(&mut conn, &[fixture()]).unwrap();
    let register=serde_json::from_value(json!({"schema_version":1,"scope":{"kind":"project","id":"demo"},"source":{"kind":"document","ref":"synthetic-weather"},"version":"v1","text":"Encabezado\nNo enviar si llueve en Bogotá 😀.\nFin."})).unwrap();
    let verify=serde_json::from_value(json!({"schema_version":1,"scope":{"kind":"project","id":"demo"},"memory_id":"m1","revision":1})).unwrap();
    (conn, register, verify)
}
fn locator(conn: &Connection, value: Option<&str>) {
    conn.execute(
        "UPDATE memory_revisions SET source_locator=?1 WHERE memory_id='m1'",
        [value],
    )
    .unwrap();
}
fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn literal_provenance_is_versioned_idempotent_and_does_not_certify_summary() {
    let (conn, request, target) = setup();
    let first = register(&conn, &request).unwrap();
    assert!(!first.replayed);
    assert!(register(&conn, &request).unwrap().replayed);
    let checked = verify(&conn, &target).unwrap();
    assert_eq!(checked.status, "verified_literal");
    assert_eq!(checked.semantic_support, "not_assessed");
    assert_eq!(
        &request.text[checked.start_byte.unwrap()..checked.end_byte.unwrap()],
        fixture().original_text
    );
    assert_eq!(
        checked.snapshot_id.as_deref(),
        Some(first.snapshot_id.as_str())
    );
    assert_eq!(
        checked.checked_at,
        verify(&conn, &target).unwrap().checked_at
    );
    assert_eq!(count(&conn, "source_verification_checks"), 1);
    assert_eq!(
        checked.original_sha256,
        sha256_hex(fixture().original_text.as_bytes())
    );
    let json = serde_json::to_string(&checked).unwrap();
    assert!(!json.contains(&fixture().original_text));
}
#[test]
fn missing_sources_and_unsupported_or_ambiguous_locators_remain_unverified() {
    let (conn, mut request, target) = setup();
    assert_eq!(
        verify(&conn, &target).unwrap().reason_code,
        "source_not_registered"
    );
    request.text = format!("{}\n{}", fixture().original_text, fixture().original_text);
    register(&conn, &request).unwrap();
    assert_eq!(
        verify(&conn, &target).unwrap().reason_code,
        "source_quote_ambiguous"
    );
    locator(&conn, Some("line=4"));
    let checked = verify(&conn, &target).unwrap();
    assert_eq!(checked.status, "declared_unverified");
    assert_eq!(checked.reason_code, "locator_unsupported");
    conn.execute(
        "UPDATE memory_revisions SET source_kind=NULL,source_ref=NULL,source_locator=NULL",
        [],
    )
    .unwrap();
    assert_eq!(verify(&conn, &target).unwrap().status, "not_provided");
}
#[test]
fn unicode_byte_ranges_are_exact_and_never_normalized() {
    let (conn, request, target) = setup();
    register(&conn, &request).unwrap();
    let original = fixture().original_text;
    let start = request.text.find(&original).unwrap();
    let end = start + original.len();
    locator(&conn, Some(&format!("utf8:{start}:{end}")));
    assert_eq!(verify(&conn, &target).unwrap().status, "verified_literal");
    for range in [
        "utf8:0:999999",
        "utf8:0:1",
        "utf8:1:0",
        "utf8:x:5",
        "utf8:0:0",
        "utf8:1:2:3",
    ] {
        locator(&conn, Some(range));
        assert_eq!(verify(&conn, &target).unwrap().status, "rejected");
    }
    // Split the emoji's four UTF-8 bytes.
    let emoji = request.text.find('😀').unwrap();
    locator(&conn, Some(&format!("utf8:{}:{}", emoji + 1, end)));
    assert_eq!(
        verify(&conn, &target).unwrap().reason_code,
        "source_range_invalid"
    );
    locator(&conn, None);
    conn.execute(
        "UPDATE memory_revisions SET original_text=replace(original_text,'Bogotá','Bogota')",
        [],
    )
    .unwrap();
    assert_eq!(
        verify(&conn, &target).unwrap().reason_code,
        "source_quote_not_found"
    );
}
#[test]
fn overlapping_quotes_require_an_explicit_range() {
    assert_eq!(locate("aaa", "aa", None).1, "source_quote_ambiguous");
    assert_eq!(locate("aaa", "aa", Some("utf8:1:3")).0, "verified_literal");
    assert_eq!(locate("ááá", "áá", None).1, "source_quote_ambiguous");
}
#[test]
fn versions_are_immutable_and_replaying_old_versions_does_not_reactivate_them() {
    let (conn, mut request, target) = setup();
    let v1 = register(&conn, &request).unwrap();
    let checked_v1 = verify(&conn, &target).unwrap();
    request.text = "No se autoriza ninguna entrega.".into();
    assert_eq!(
        register(&conn, &request).unwrap_err(),
        "source_version_conflict"
    );
    request.version = "v2".into();
    let v2 = register(&conn, &request).unwrap();
    let checked = verify(&conn, &target).unwrap();
    assert_eq!(checked.status, "rejected");
    assert_ne!(checked.check_id, checked_v1.check_id);
    assert_eq!(checked.snapshot_id, Some(v2.snapshot_id.clone()));
    request.version = "v1".into();
    request.text = "Encabezado\nNo enviar si llueve en Bogotá 😀.\nFin.".into();
    assert_eq!(
        register(&conn, &request).unwrap().snapshot_id,
        v1.snapshot_id
    );
    assert_eq!(
        verify(&conn, &target).unwrap().snapshot_id,
        Some(v2.snapshot_id)
    );
    assert_eq!(count(&conn, "source_verification_checks"), 2);
}
#[test]
fn snapshot_and_receipt_tampering_cannot_reuse_a_verified_flag() {
    let (conn, request, target) = setup();
    register(&conn, &request).unwrap();
    let checked = verify(&conn, &target).unwrap();
    conn.execute(
        "UPDATE source_verification_checks SET report_json='{}' WHERE check_id=?1",
        [&checked.check_id],
    )
    .unwrap();
    assert_eq!(
        verify(&conn, &target).unwrap_err(),
        "source_receipt_integrity_mismatch"
    );
    conn.execute(
        "UPDATE source_snapshots SET source_text='contenido alterado'",
        [],
    )
    .unwrap();
    assert_eq!(
        verify(&conn, &target).unwrap().reason_code,
        "snapshot_integrity_mismatch"
    );
}
#[test]
fn actor_scope_and_revision_are_checked_before_loading_source() {
    let (conn, mut request, mut target) = setup();
    request.actor_id = Some("other".into());
    register(&conn, &request).unwrap();
    assert_eq!(
        verify(&conn, &target).unwrap().reason_code,
        "source_not_registered"
    );
    target.actor_id = Some("other".into());
    assert_eq!(
        verify(&conn, &target).unwrap_err(),
        "source_memory_unavailable"
    );
    target.actor_id = None;
    target.scope.id = "another".into();
    assert!(verify(&conn, &target).is_err());
    target.scope.id = "demo".into();
    target.revision = 2;
    assert!(verify(&conn, &target).is_err());
    request.actor_id = None;
    request.scope.id = "another".into();
    register(&conn, &request).unwrap();
    target.revision = 1;
    assert_eq!(
        verify(&conn, &target).unwrap().reason_code,
        "source_not_registered"
    );
}
#[test]
fn newer_memory_revisions_require_fresh_literal_evidence() {
    let (mut conn, request, target) = setup();
    register(&conn, &request).unwrap();
    verify(&conn, &target).unwrap();
    let mut m = fixture();
    m.revision = 2;
    m.original_text = "Sí enviar aunque llueva.".into();
    crate::repo::load_fixtures(&mut conn, &[m]).unwrap();
    assert_eq!(
        verify(&conn, &target).unwrap_err(),
        "source_memory_unavailable"
    );
    let target = VerifyRequest {
        revision: 2,
        ..target
    };
    assert_eq!(
        verify(&conn, &target).unwrap().reason_code,
        "source_quote_not_found"
    );
}
#[test]
fn checking_a_changed_candidate_rolls_back_all_new_receipts() {
    let (conn, request, _) = setup();
    register(&conn, &request).unwrap();
    let scope = Scope {
        kind: "project".into(),
        id: "demo".into(),
    };
    let m = memory(&conn, "local", &scope, "m1", 1).unwrap();
    let mut changed = m.candidate.clone();
    changed.summary = "alterada".into();
    assert_eq!(
        check_candidates(&conn, "local", &scope, &[m.candidate, changed]).unwrap_err(),
        "source_candidate_changed"
    );
    assert_eq!(count(&conn, "source_verification_checks"), 0);
}
#[test]
fn invalid_or_oversized_registration_leaves_no_snapshot() {
    let (conn, mut request, _) = setup();
    request.text = "x".repeat(MAX_SOURCE_BYTES + 1);
    assert!(register(&conn, &request).is_err());
    request.text = "x".into();
    request.source.locator = Some("utf8:0:1".into());
    assert!(register(&conn, &request).is_err());
    request.source.locator = None;
    request.scope.kind = "unknown".into();
    assert!(register(&conn, &request).is_err());
    assert_eq!(count(&conn, "source_snapshots"), 0);
}

#[test]
fn proposed_evidence_can_be_checked_inside_a_caller_transaction_without_receipts() {
    let (conn, request, _) = setup();
    register(&conn, &request).unwrap();
    let tx = conn.unchecked_transaction().unwrap();
    let result = check_evidence(
        &tx,
        &request.scope,
        "local",
        &request.source,
        &fixture().original_text,
    )
    .unwrap();
    assert_eq!(result.status, "verified_literal");
    assert_eq!(result.semantic_support, "not_assessed");
    assert_eq!(count(&tx, "source_verification_checks"), 0);
    assert!(
        check_evidence(
            &tx,
            &request.scope,
            "other",
            &request.source,
            &fixture().original_text
        )
        .unwrap()
        .snapshot_id
        .is_none()
    );
    tx.rollback().unwrap();
}

#[test]
fn derived_candidate_uncertainty_does_not_change_literal_provenance() {
    let (conn, request, _) = setup();
    register(&conn, &request).unwrap();
    let mut candidate = memory(&conn, "local", &request.scope, "m1", 1)
        .unwrap()
        .candidate;
    candidate.state = "uncertain".into();
    let checks = check_candidates(&conn, "local", &request.scope, &[candidate]).unwrap();
    assert_eq!(checks[0].status, "verified_literal");
    assert_eq!(checks[0].semantic_support, "not_assessed");
}
