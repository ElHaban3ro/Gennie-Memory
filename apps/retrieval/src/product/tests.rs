use super::*;
use crate::classifier::DecisionError;
use std::cell::RefCell;

enum Reply {
    Select,
    Abstain,
    Invalid,
    Fail(&'static str, Option<u16>),
    Relations,
    SuggestEquivalent,
    Late,
}
struct Fake {
    reply: Reply,
    queries: RefCell<Vec<String>>,
    equivalence_calls: RefCell<usize>,
}
impl Classifier for Fake {
    fn classify(
        &self,
        _: &str,
        _: &str,
        _: &[Candidate],
        _: ClassifierProvider,
        _: SelectionPolicy,
    ) -> Result<DecisionResult, DecisionError> {
        panic!("versioned flow must use deadline-aware entry");
    }
    fn classify_until(
        &self,
        query: &str,
        _: &str,
        candidates: &[Candidate],
        _: ClassifierProvider,
        _: SelectionPolicy,
        _: Instant,
    ) -> Result<DecisionResult, DecisionError> {
        self.queries.borrow_mut().push(query.into());
        if let Reply::Fail(code, http_status) = self.reply {
            return Err(DecisionError {
                diagnostic: None,
                code,
                http_status,
                partial_usage: Some(PartialDecisionUsage {
                    served_model: None,
                    input_tokens: Some(12),
                    output_tokens: None,
                    reported_cost_usd: Some(0.001),
                }),
            });
        }
        if matches!(self.reply, Reply::Late) {
            std::thread::sleep(Duration::from_millis(15));
        }
        Ok(DecisionResult {
            selected_id: match self.reply {
                Reply::Abstain => None,
                Reply::Invalid => Some("foreign-memory".into()),
                _ => Some(candidates[0].memory_id.clone()),
            },
            served_model: Some("test-model".into()),
            input_tokens: Some(12),
            output_tokens: Some(2),
            reported_cost_usd: Some(0.001),
            consensus: None,
            applicability: None,
            slot_evidence: None,
            per_memory: if matches!(self.reply, Reply::Relations) {
                Some(crate::per_memory::PerMemoryTrace {
                    items: candidates
                        .iter()
                        .enumerate()
                        .map(|(i, c)| crate::per_memory::MemoryAssessment {
                            memory_id: c.memory_id.clone(),
                            revision: c.revision,
                            relation: if i == 0 {
                                crate::per_memory::EvidenceRelation::RefutesPremise
                            } else {
                                crate::per_memory::EvidenceRelation::Unrelated
                            },
                        })
                        .collect(),
                    factor_checks: None,
                    scoped_checks: None,
                    choice_scores: None,
                    request_sha256: None,
                    verification: None,
                })
            } else {
                None
            },
        })
    }

    fn suggest_equivalent_until(
        &self,
        _: &str,
        anchor: &Candidate,
        candidates: &[Candidate],
        _: ClassifierProvider,
        _: Instant,
    ) -> Result<DecisionResult, DecisionError> {
        *self.equivalence_calls.borrow_mut() += 1;
        Ok(DecisionResult {
            selected_id: if matches!(self.reply, Reply::SuggestEquivalent) {
                candidates
                    .iter()
                    .find(|c| c.memory_id != anchor.memory_id)
                    .map(|c| c.memory_id.clone())
            } else {
                None
            },
            served_model: Some("test-model".into()),
            input_tokens: Some(5),
            output_tokens: Some(1),
            reported_cost_usd: Some(0.0005),
            consensus: None,
            applicability: None,
            slot_evidence: None,
            per_memory: None,
        })
    }
}
fn setup() -> (Connection, serde_json::Value) {
    let mut conn = Connection::open_in_memory().unwrap();
    gennie_memory_kernel::db::configure(&conn).unwrap();
    let modules: Vec<Box<dyn gennie_memory_kernel::core::AppModule>> =
        vec![Box::new(crate::module::RetrievalModule)];
    gennie_memory_kernel::db::migrate(&mut conn, &modules).unwrap();
    let memories: Vec<FixtureMemory> = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/conditional-corpus.json"
    ))
    .unwrap();
    crate::repo::load_fixtures(&mut conn, &memories).unwrap();
    (
        conn,
        serde_json::json!({"contract_version":1, "search":{"query":"agenda reuniones remotas Proyecto Sintético", "scope_kind":"project","scope_id":"synthetic_meetings","as_of":"2026-10-15T12:00:00.000Z"}}),
    )
}
fn run(conn: &Connection, r: &serde_json::Value, reply: Reply) -> (serde_json::Value, Vec<String>) {
    let fake = Fake {
        reply,
        queries: RefCell::new(vec![]),
        equivalence_calls: RefCell::new(0),
    };
    let out = execute_json(conn, &r.to_string(), &fake);
    (
        serde_json::to_value(out).unwrap(),
        fake.queries.into_inner(),
    )
}

#[test]
fn equivalent_memory_second_pass_is_a_review_hint_and_cost_is_aggregated() {
    let (conn, request) = setup();
    let fake = Fake {
        reply: Reply::SuggestEquivalent,
        queries: RefCell::new(vec![]),
        equivalence_calls: RefCell::new(0),
    };
    let out = execute_json(&conn, &request.to_string(), &fake);
    let out = serde_json::to_value(out).unwrap();
    assert_eq!(out["outcome"], "evidence_selected");
    assert_eq!(out["potential_equivalents"].as_array().unwrap().len(), 1);
    assert_eq!(
        out["potential_equivalents"][0]["decision"],
        "agent_review_required"
    );
    assert_eq!(
        out["seed"]["potential_equivalents"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        out["seed"]["seed_text"]
            .as_str()
            .unwrap()
            .contains("se mantienen separados")
    );
    assert_eq!(out["usage"]["classifier_invocations"], 2);
    assert_eq!(out["usage"]["input_tokens"], 17);
    assert_eq!(out["usage"]["reported_cost_usd"], 0.0015);
    assert_eq!(out["coverage"]["equivalence_review"], "complete");
}
#[test]
fn preserves_selection_source_and_does_not_fabricate_relations() {
    let (conn, r) = setup();
    let (out, calls) = run(&conn, &r, Reply::Select);
    assert_eq!(out["outcome"], "evidence_selected");
    assert_eq!(calls.len(), 1);
    assert_eq!(out["assessments"][0]["relation"], "unassessed");
    assert_eq!(
        out["assessments"][0]["provenance_status"],
        "declared_unverified"
    );
    assert!(out["assessments"][0]["reason"].is_null());
    assert!(!out["seed"]["memories"][0]["source"].is_null());
    assert_eq!(
        out["seed"]["memories"][0]["provenance_status"],
        "declared_unverified"
    );
    assert_eq!(out["coverage"]["question"], "unknown");
    assert_eq!(out["usage"]["receipt_status"], "complete");
    assert_eq!(out["effective_request"]["search"]["limit"], 20);
    assert!(out["started_at"].as_str().unwrap().ends_with('Z'));
}
#[test]
fn empty_abstention_invalid_and_technical_failure_are_distinct() {
    let (conn, mut r) = setup();
    let (out, _) = run(&conn, &r, Reply::Abstain);
    assert_eq!(out["outcome"], "abstained");
    let (out, _) = run(&conn, &r, Reply::Invalid);
    assert_eq!(out["issues"][0]["code"], "invalid_selection");
    assert!(out["seed"].is_null());
    for (code, status, expected) in [
        ("http_error", Some(402), "provider_payment_required"),
        ("http_error", Some(429), "provider_rate_limited"),
        ("request_timeout", None, "provider_timeout"),
        ("transport_error", None, "provider_transport_error"),
    ] {
        let (out, _) = run(&conn, &r, Reply::Fail(code, status));
        assert_eq!(out["issues"][0]["code"], expected);
        assert_eq!(out["outcome"], "not_evaluated");
        assert!(out["retrieval"]["candidates"].as_array().unwrap().len() > 0);
        assert_eq!(out["usage"]["reported_cost_usd"], 0.001);
        assert!(out["usage"]["output_tokens"].is_null());
    }
    r["search"]["query"] = "zzzzzzzz".into();
    let (out, calls) = run(&conn, &r, Reply::Select);
    assert_eq!(out["outcome"], "no_candidates");
    assert!(calls.is_empty());
}
#[test]
fn clarification_and_bad_input_never_call_provider() {
    let (conn, mut r) = setup();
    r["unresolved_references"] =
        serde_json::json!([{"start_byte":0,"end_byte":6,"required_field":"entity"}]);
    let (out, calls) = run(&conn, &r, Reply::Select);
    assert_eq!(out["outcome"], "clarification_required");
    assert!(calls.is_empty());
    r["unresolved_references"][0]["end_byte"] = 999.into();
    let (out, calls) = run(&conn, &r, Reply::Select);
    assert_eq!(out["execution_status"], "failed");
    assert!(calls.is_empty());
    r.as_object_mut().unwrap().remove("unresolved_references");
    r["contract_version"] = 2.into();
    let (out, calls) = run(&conn, &r, Reply::Select);
    assert_eq!(out["execution_status"], "failed");
    assert!(calls.is_empty());
}
#[test]
fn context_reaches_classifier_but_does_not_expand_search_scope() {
    let (conn, mut r) = setup();
    r["context"] = serde_json::json!([{"id":"turn1","source":"conversation:test","text":"Se trata de reuniones remotas; no de las presenciales."}]);
    let (out, calls) = run(&conn, &r, Reply::Select);
    let payload: serde_json::Value = serde_json::from_str(&calls[0]).unwrap();
    assert_eq!(payload["query"], r["search"]["query"]);
    assert_eq!(payload["context"], r["context"]);
    assert_eq!(out["context_segmentations"].as_array().unwrap().len(), 1);
    r["search"]["scope_id"] = "other_scope".into();
    let (out, calls) = run(&conn, &r, Reply::Select);
    assert_eq!(out["outcome"], "no_candidates");
    assert!(calls.is_empty());
}
#[test]
fn seed_and_deadline_limits_never_publish_truncated_or_late_seed() {
    let (conn, mut r) = setup();
    r["budgets"] = serde_json::json!({"seed_bytes":1});
    let (out, _) = run(&conn, &r, Reply::Select);
    assert_eq!(out["issues"][0]["code"], "seed_byte_limit");
    assert!(out["seed"].is_null());
    r["budgets"] = serde_json::json!({"deadline_ms":10});
    let (out, _) = run(&conn, &r, Reply::Late);
    assert!(out["seed"].is_null());
    assert_eq!(out["issues"][0]["code"], "operation_deadline");
}
#[test]
fn typed_roles_are_preserved_and_unsupported_policies_fail_before_call() {
    let (conn, mut r) = setup();
    r["selection_policy"] = "per_memory_v1".into();
    let (out, _) = run(&conn, &r, Reply::Relations);
    assert_eq!(out["outcome"], "evidence_selected");
    assert_eq!(out["assessments"][0]["relation"], "refutes_premise");
    r["selection_policy"] = "witness_v1".into();
    let (out, calls) = run(&conn, &r, Reply::Select);
    assert_eq!(out["issues"][0]["code"], "unsupported_capability");
    assert!(calls.is_empty());
}
