use super::*;
use crate::classifier::DecisionError;
use std::cell::RefCell;

enum Reply {
    Select,
    Abstain,
    Invalid,
    Fail(&'static str, Option<u16>),
    Relations,
    Factors,
    SuggestEquivalent,
    Late,
}
struct Fake {
    reply: Reply,
    queries: RefCell<Vec<String>>,
    equivalence_calls: RefCell<usize>,
}
fn bounded_fake(
    remaining: usize,
    stage: &'static str,
    payload: serde_json::Value,
    call: impl FnOnce() -> Result<DecisionResult, DecisionError>,
) -> crate::classifier::BudgetedDecision {
    let mut budget = crate::token_budget::InputBudget::new(remaining);
    let checked = budget.inspect(stage, payload.to_string().as_bytes());
    let result = match checked {
        Ok(()) => call(),
        Err(code) => Err(DecisionError {
            diagnostic: None,
            code,
            http_status: None,
            partial_usage: None,
        }),
    };
    crate::classifier::BudgetedDecision {
        result,
        requests: budget.requests,
    }
}
impl Classifier for Fake {
    fn factor_profile_sha256(&self) -> Option<String> {
        Some("a".repeat(64))
    }
    fn classify_budgeted_until(
        &self,
        query: &str,
        scope: &str,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        policy: SelectionPolicy,
        deadline: Instant,
        remaining: usize,
    ) -> crate::classifier::BudgetedDecision {
        bounded_fake(
            remaining,
            "classification",
            serde_json::json!({"test_adapter_only":true,"query":query,"scope":scope,"candidates":candidates}),
            || self.classify_until(query, scope, candidates, provider, policy, deadline),
        )
    }
    fn suggest_equivalent_budgeted_until(
        &self,
        scope: &str,
        anchor: &Candidate,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        deadline: Instant,
        remaining: usize,
    ) -> crate::classifier::BudgetedDecision {
        bounded_fake(
            remaining,
            "equivalence_review",
            serde_json::json!({"test_adapter_only":true,"scope":scope,"anchor":anchor,"candidates":candidates}),
            || self.suggest_equivalent_until(scope, anchor, candidates, provider, deadline),
        )
    }
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
            served_model: Some(if matches!(self.reply, Reply::Factors) {
                ClassifierProvider::Decisions.model_id().into()
            } else {
                "test-model".into()
            }),
            input_tokens: Some(12),
            output_tokens: Some(2),
            reported_cost_usd: Some(0.001),
            consensus: None,
            applicability: None,
            slot_evidence: None,
            per_memory: if matches!(self.reply, Reply::Relations | Reply::Factors) {
                Some(crate::per_memory::PerMemoryTrace {
                    items: candidates
                        .iter()
                        .enumerate()
                        .map(|(i, c)| crate::per_memory::MemoryAssessment {
                            memory_id: c.memory_id.clone(),
                            revision: c.revision,
                            relation: if i == 0 {
                                if matches!(self.reply, Reply::Factors) {
                                    crate::per_memory::EvidenceRelation::UsefulEvidence
                                } else {
                                    crate::per_memory::EvidenceRelation::RefutesPremise
                                }
                            } else {
                                crate::per_memory::EvidenceRelation::Unrelated
                            },
                        })
                        .collect(),
                    factor_checks: matches!(self.reply, Reply::Factors).then(|| {
                        candidates
                            .iter()
                            .enumerate()
                            .map(|(i, c)| crate::per_memory::factors::MemoryFactorChecks {
                                memory_id: c.memory_id.clone(),
                                revision: c.revision,
                                same_context: i == 0,
                                requested_fact: true,
                                premise_correction: false,
                                scores: Some(crate::per_memory::factors::MemoryFactorScores {
                                    same_context: if i == 0 { 0.9 } else { 0.1 },
                                    requested_fact: 0.8,
                                    premise_correction: 0.2,
                                }),
                            })
                            .collect()
                    }),
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
fn isolation_filters_reach_product_and_empty_normalization_is_not_no_candidates() {
    let (conn, mut request) = setup();
    request["search"]["actor_id"] = serde_json::json!("another-actor");
    let (out, calls) = run(&conn, &request, Reply::Select);
    assert_eq!(out["outcome"], "no_candidates");
    assert!(calls.is_empty());
    request["search"]
        .as_object_mut()
        .unwrap()
        .remove("actor_id");
    request["search"]["memory_types"] = serde_json::json!(["unknown-type"]);
    let (out, calls) = run(&conn, &request, Reply::Select);
    assert_eq!(out["outcome"], "no_candidates");
    assert!(calls.is_empty());
    request["search"]
        .as_object_mut()
        .unwrap()
        .remove("memory_types");
    request["search"]["query"] = serde_json::json!("no y si");
    let (out, calls) = run(&conn, &request, Reply::Select);
    assert_eq!(out["outcome"], "not_evaluated");
    assert_eq!(out["coverage"]["retrieval"], "not_evaluated");
    assert!(calls.is_empty());
}

#[test]
fn missing_text_projection_degrades_product_before_classification() {
    let (conn, request) = setup();
    conn.execute("DELETE FROM search_documents", []).unwrap();
    let (out, calls) = run(&conn, &request, Reply::Select);
    assert_eq!(out["execution_status"], "degraded");
    assert_eq!(out["outcome"], "not_evaluated");
    assert_eq!(out["issues"][0]["code"], "fts_index_incomplete");
    assert!(calls.is_empty());
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

#[test]
fn token_input_boundary_blocks_before_adapter_and_preserves_candidates() {
    let (conn, mut r) = setup();
    let (baseline, _) = run(&conn, &r, Reply::Select);
    let n = baseline["token_accounting"]["requests"][0]["json_tokens"]
        .as_u64()
        .unwrap();
    r["budgets"] = serde_json::json!({"input_tokens":n-1});
    let (blocked, calls) = run(&conn, &r, Reply::Select);
    assert!(calls.is_empty());
    assert!(blocked["retrieval"]["candidates"].as_array().unwrap().len() > 1);
    assert_eq!(blocked["issues"][0]["code"], "input_token_limit");
    assert_eq!(blocked["issues"][0]["component"], "gennie-memory");
    assert_eq!(blocked["usage"]["receipt_status"], "not_called");
    assert_eq!(blocked["token_accounting"]["accepted_input_tokens"], 0);
    r["budgets"]["input_tokens"] = n.into();
    let (partial, calls) = run(&conn, &r, Reply::SuggestEquivalent);
    assert_eq!(calls.len(), 1);
    assert!(partial["seed"].is_object());
    assert_eq!(partial["coverage"]["equivalence_review"], "omitted_budget");
    assert_eq!(partial["usage"]["classifier_invocations"], 1);
    assert_eq!(partial["usage"]["reported_cost_usd"], 0.001);
    assert_eq!(
        partial["token_accounting"]["requests"][1]["accepted"],
        false
    );
}
#[test]
fn cumulative_input_and_final_seed_have_exact_inclusive_limits() {
    let (conn, mut r) = setup();
    let (baseline, _) = run(&conn, &r, Reply::SuggestEquivalent);
    let input = baseline["token_accounting"]["accepted_input_tokens"]
        .as_u64()
        .unwrap();
    let seed = baseline["token_accounting"]["seed_tokens"]
        .as_u64()
        .unwrap();
    let text = baseline["seed"]["seed_text"].as_str().unwrap();
    assert!(text.contains("Revisión requerida"));
    assert_eq!(crate::token_budget::count(text).unwrap() as u64, seed);
    r["budgets"] = serde_json::json!({"input_tokens":input,"seed_tokens":seed});
    let (exact, _) = run(&conn, &r, Reply::SuggestEquivalent);
    assert_eq!(exact["seed"], baseline["seed"]);
    assert_eq!(exact["token_accounting"]["seed_status"], "complete");
    r["budgets"]["seed_tokens"] = (seed - 1).into();
    let (blocked, _) = run(&conn, &r, Reply::SuggestEquivalent);
    assert!(blocked["seed"].is_null());
    assert_eq!(blocked["issues"][0]["code"], "seed_token_limit");
    assert!(
        blocked["assessments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["selected"] == true)
    );
    assert!(
        blocked["assessments"]
            .as_array()
            .unwrap()
            .iter()
            .all(|a| a["included_in_seed"] == false)
    );
    assert_eq!(blocked["token_accounting"]["seed_tokens"], seed);
}
#[test]
fn invalid_token_budgets_fail_before_classification() {
    let (conn, mut r) = setup();
    for field in ["input_tokens", "seed_tokens"] {
        for value in [0, 65537] {
            r["budgets"] = serde_json::json!({field:value});
            let (out, calls) = run(&conn, &r, Reply::Select);
            assert!(calls.is_empty());
            assert_eq!(out["execution_status"], "failed");
        }
    }
}

#[test]
fn adapters_without_budget_support_fail_closed() {
    struct Unsupported;
    impl Classifier for Unsupported {
        fn classify(
            &self,
            _: &str,
            _: &str,
            _: &[Candidate],
            _: ClassifierProvider,
            _: SelectionPolicy,
        ) -> Result<DecisionResult, DecisionError> {
            panic!("unbounded adapter must never be called")
        }
    }
    let (conn, r) = setup();
    let out = execute_json(&conn, &r.to_string(), &Unsupported);
    assert_eq!(out.issues[0].code, "token_budget_not_supported");
    assert_eq!(out.issues[0].component, "gennie-memory");
    assert!(out.seed.is_none());
    assert_eq!(out.usage.classifier_invocations, 0);
}

fn registered_product_sources(
    conn: &Connection,
    r: &serde_json::Value,
) -> crate::source_verification::RegisterRequest {
    conn.execute("UPDATE memory_revisions SET source_kind='document',source_ref='synthetic-product-source',source_locator=NULL",[]).unwrap();
    let mut stmt=conn.prepare("SELECT original_text FROM memory_revisions r JOIN memories m ON m.id=r.memory_id AND m.current_revision=r.revision WHERE m.scope_id=?1 ORDER BY m.id").unwrap();
    let text = stmt
        .query_map([r["search"]["scope_id"].as_str().unwrap()], |row| {
            row.get::<_, String>(0)
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n---\n");
    let registration=serde_json::from_value(serde_json::json!({"schema_version":1,"scope":{"kind":"project","id":r["search"]["scope_id"]},"source":{"kind":"document","ref":"synthetic-product-source"},"version":"v1","text":text})).unwrap();
    crate::source_verification::register(conn, &registration).unwrap();
    registration
}
#[test]
fn source_policy_reports_unknowns_and_strict_mode_blocks_only_the_seed() {
    let (conn, mut r) = setup();
    let (reported, _) = run(&conn, &r, Reply::Select);
    assert!(reported["seed"].is_object());
    assert!(
        reported["source_verification"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["status"] != "verified_literal")
    );
    r["source_policy"] = "require_verified".into();
    let (strict, calls) = run(&conn, &r, Reply::Select);
    assert!(!calls.is_empty());
    assert!(strict["seed"].is_null());
    assert_eq!(strict["issues"][0]["code"], "source_verification_blocked");
    assert!(
        strict["assessments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["selected"] == true)
    );
    assert_eq!(strict["usage"]["receipt_status"], "complete");
    assert!(
        strict["assessments"]
            .as_array()
            .unwrap()
            .iter()
            .all(|a| a["included_in_seed"] == false)
    );
}
#[test]
fn verified_sources_and_equivalent_hints_are_rechecked_and_counted_in_final_seed() {
    let (conn, mut r) = setup();
    registered_product_sources(&conn, &r);
    r["source_policy"] = "require_verified".into();
    let (out, _) = run(&conn, &r, Reply::SuggestEquivalent);
    assert_eq!(out["execution_status"], "completed");
    assert!(out["seed"]["memories"].as_array().unwrap().len() > 1);
    assert!(
        out["seed"]["memories"]
            .as_array()
            .unwrap()
            .iter()
            .all(|m| m["provenance_status"] == "verified_literal")
    );
    let text = out["seed"]["seed_text"].as_str().unwrap();
    assert!(text.contains("Procedencia local"));
    assert!(text.contains("veracidad, resumen y condiciones añadidas sin evaluar"));
    assert_eq!(
        out["token_accounting"]["seed_tokens"],
        crate::token_budget::count(text).unwrap()
    );
    let before: i64 = conn
        .query_row(
            "SELECT count(*) FROM source_verification_checks",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let (again, _) = run(&conn, &r, Reply::SuggestEquivalent);
    assert_eq!(again["source_verification"], out["source_verification"]);
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM source_verification_checks",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        before
    );
}
#[test]
fn rejected_registered_sources_never_reach_a_published_seed_in_report_mode() {
    let (conn, r) = setup();
    let mut registration = registered_product_sources(&conn, &r);
    registration.version = "v2".into();
    registration.text = "Documento sintético retirado; no contiene la cita.".into();
    crate::source_verification::register(&conn, &registration).unwrap();
    let (out, _) = run(&conn, &r, Reply::Select);
    assert!(out["seed"].is_null());
    assert_eq!(out["issues"][0]["code"], "source_verification_blocked");
    assert!(
        out["source_verification"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["status"] == "rejected")
    );
    assert_eq!(out["usage"]["cost_status"], "reported");
}
#[test]
fn source_updated_during_classification_is_revalidated_before_seed() {
    struct Updating<'a> {
        conn: &'a Connection,
        registration: crate::source_verification::RegisterRequest,
        inner: Fake,
    }
    impl Classifier for Updating<'_> {
        fn classify(
            &self,
            _: &str,
            _: &str,
            _: &[Candidate],
            _: ClassifierProvider,
            _: SelectionPolicy,
        ) -> Result<DecisionResult, DecisionError> {
            panic!("must use bounded entry")
        }
        fn classify_budgeted_until(
            &self,
            q: &str,
            s: &str,
            c: &[Candidate],
            p: ClassifierProvider,
            policy: SelectionPolicy,
            d: Instant,
            n: usize,
        ) -> crate::classifier::BudgetedDecision {
            let result = self.inner.classify_budgeted_until(q, s, c, p, policy, d, n);
            crate::source_verification::register(self.conn, &self.registration).unwrap();
            result
        }
        fn suggest_equivalent_budgeted_until(
            &self,
            s: &str,
            a: &Candidate,
            c: &[Candidate],
            p: ClassifierProvider,
            d: Instant,
            n: usize,
        ) -> crate::classifier::BudgetedDecision {
            self.inner
                .suggest_equivalent_budgeted_until(s, a, c, p, d, n)
        }
    }
    let (conn, r) = setup();
    let mut registration = registered_product_sources(&conn, &r);
    registration.version = "v2".into();
    registration.text = "La cita ya no está en esta versión sintética.".into();
    let updating = Updating {
        conn: &conn,
        registration,
        inner: Fake {
            reply: Reply::Select,
            queries: RefCell::new(vec![]),
            equivalence_calls: RefCell::new(0),
        },
    };
    let out = execute_json(&conn, &r.to_string(), &updating);
    assert!(out.seed.is_none());
    assert_eq!(out.issues[0].code, "source_verification_blocked");
    assert!(
        out.source_verification
            .checks
            .iter()
            .any(|c| c.snapshot_version.as_deref() == Some("v2") && c.status == "rejected")
    );
}

#[test]
fn factors_keep_applicability_separate_from_usefulness_and_apply_only_bound_calibration() {
    let (conn, mut r) = setup();
    r["selection_policy"] = "per_memory_factors_v1".into();
    let (out, _) = run(&conn, &r, Reply::Factors);
    assert_eq!(out["execution_status"], "completed");
    assert_eq!(out["assessments"][0]["applicability"], true);
    assert_eq!(out["assessments"][1]["applicability"], false);
    assert_eq!(out["assessments"][1]["relevance"], true);
    assert_eq!(out["assessments"][1]["selected"], false);
    assert_eq!(
        out["assessments"][0]["calibration_status"],
        "not_calibrated"
    );
    let bin = serde_json::json!([{"upper":1.0,"positives":6,"n":8}]);
    r["calibration"] = serde_json::json!({"schema_version":1,"profile":"factor_empirical_bins_v1","profile_sha256":"a".repeat(64),"provider":"decisions","selection_policy":"per_memory_factors_v1","requested_model":ClassifierProvider::Decisions.model_id(),"served_model":ClassifierProvider::Decisions.model_id(),"training_sha256":"b".repeat(64),"factors":{"same_context":bin,"requested_fact":bin,"premise_correction":bin}});
    let (calibrated, _) = run(&conn, &r, Reply::Factors);
    assert_eq!(calibrated["execution_status"], "completed");
    assert_eq!(
        calibrated["assessments"][0]["calibrated_factor_scores"]["same_context"],
        0.7
    );
    assert_eq!(
        calibrated["assessments"][0]["calibration_status"],
        "empirical_profile_bound"
    );
    assert_eq!(calibrated["seed"]["memories"], out["seed"]["memories"]);
    r["calibration"]["profile_sha256"] = "c".repeat(64).into();
    let (rejected, calls) = run(&conn, &r, Reply::Factors);
    assert_eq!(
        rejected["issues"][0]["code"],
        "calibration_profile_mismatch"
    );
    assert!(calls.is_empty());
}
#[test]
fn factor_policy_rejects_a_trace_without_independent_factors() {
    let (conn, mut r) = setup();
    r["selection_policy"] = "per_memory_factors_v1".into();
    let (out, _) = run(&conn, &r, Reply::Relations);
    assert_eq!(out["issues"][0]["code"], "invalid_selection");
    assert!(out["seed"].is_null());
}

#[test]
fn calibration_never_silently_accepts_missing_scores_or_another_served_model() {
    struct Missing {
        model_mismatch: bool,
    }
    impl Classifier for Missing {
        fn factor_profile_sha256(&self) -> Option<String> {
            Some("a".repeat(64))
        }
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
        fn classify_budgeted_until(
            &self,
            query: &str,
            scope: &str,
            candidates: &[Candidate],
            provider: ClassifierProvider,
            policy: SelectionPolicy,
            deadline: Instant,
            remaining: usize,
        ) -> crate::classifier::BudgetedDecision {
            let fake = Fake {
                reply: Reply::Factors,
                queries: RefCell::new(vec![]),
                equivalence_calls: RefCell::new(0),
            };
            let mut result = fake.classify_budgeted_until(
                query, scope, candidates, provider, policy, deadline, remaining,
            );
            let decision = result.result.as_mut().unwrap();
            if self.model_mismatch {
                decision.served_model = Some("other-model".into());
            } else {
                decision
                    .per_memory
                    .as_mut()
                    .unwrap()
                    .factor_checks
                    .as_mut()
                    .unwrap()[0]
                    .scores = None;
            }
            result
        }
    }
    let (conn, mut r) = setup();
    r["selection_policy"] = "per_memory_factors_v1".into();
    let bin = serde_json::json!([{"upper":1.0,"positives":6,"n":8}]);
    r["calibration"] = serde_json::json!({"schema_version":1,"profile":"factor_empirical_bins_v1","profile_sha256":"a".repeat(64),"provider":"decisions","selection_policy":"per_memory_factors_v1","requested_model":ClassifierProvider::Decisions.model_id(),"served_model":ClassifierProvider::Decisions.model_id(),"training_sha256":"b".repeat(64),"factors":{"same_context":bin,"requested_fact":bin,"premise_correction":bin}});
    for (model_mismatch, code) in [
        (true, "calibration_model_mismatch"),
        (false, "calibration_scores_unavailable"),
    ] {
        let out = serde_json::to_value(execute_json(
            &conn,
            &r.to_string(),
            &Missing { model_mismatch },
        ))
        .unwrap();
        assert_eq!(out["issues"][0]["code"], code);
        assert!(out["seed"].is_null());
        assert_eq!(out["usage"]["reported_cost_usd"], 0.001);
    }
}

#[test]
fn calibration_binds_the_observed_snapshot_and_never_substitutes_its_alias() {
    struct Snapshot;
    impl Classifier for Snapshot {
        fn factor_profile_sha256(&self) -> Option<String> {
            Some("a".repeat(64))
        }
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
        fn classify_budgeted_until(
            &self,
            query: &str,
            scope: &str,
            candidates: &[Candidate],
            provider: ClassifierProvider,
            policy: SelectionPolicy,
            deadline: Instant,
            remaining: usize,
        ) -> crate::classifier::BudgetedDecision {
            let fake = Fake {
                reply: Reply::Factors,
                queries: RefCell::new(vec![]),
                equivalence_calls: RefCell::new(0),
            };
            let mut result = fake.classify_budgeted_until(
                query, scope, candidates, provider, policy, deadline, remaining,
            );
            result.result.as_mut().unwrap().served_model =
                Some("openai/gpt-6-luna-decisions-20261006".into());
            result
        }
    }
    let (conn, mut r) = setup();
    r["selection_policy"] = "per_memory_factors_v1".into();
    let bin = serde_json::json!([{"upper":1.0,"positives":6,"n":8}]);
    r["calibration"] = serde_json::json!({"schema_version":1,"profile":"factor_empirical_bins_v1","profile_sha256":"a".repeat(64),"provider":"decisions","selection_policy":"per_memory_factors_v1","requested_model":ClassifierProvider::Decisions.model_id(),"served_model":"openai/gpt-6-luna-decisions-20261006","training_sha256":"b".repeat(64),"factors":{"same_context":bin,"requested_fact":bin,"premise_correction":bin}});
    let out = serde_json::to_value(execute_json(&conn, &r.to_string(), &Snapshot)).unwrap();
    assert_eq!(out["execution_status"], "completed");
    assert_eq!(
        out["assessments"][0]["calibration_status"],
        "empirical_profile_bound"
    );
    let (wrong, _) = run(&conn, &r, Reply::Factors);
    assert_eq!(wrong["issues"][0]["code"], "calibration_model_mismatch");
    r["calibration"]["served_model"] = "other-provider/model".into();
    let (wrong, calls) = run(&conn, &r, Reply::Factors);
    assert_eq!(wrong["issues"][0]["code"], "calibration_profile_mismatch");
    assert!(calls.is_empty());
}
