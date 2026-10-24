//! Observable product contracts with synthetic data; no provider inference.
use gennie_memory_kernel::{core::AppModule, db, digest::sha256_hex};
use gennie_memory_retrieval::{
    classifier::{BudgetedDecision, Classifier, DecisionError, PartialDecisionUsage},
    model::*,
    module::RetrievalModule,
    product, repo, segmentation, service, token_budget,
};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{cell::RefCell, time::Instant};

const CORPUS: &str = include_str!("../../../tests/fixtures/product-coverage-v1/corpus.json");
const CASES: &str = include_str!("../../../tests/fixtures/product-coverage-v1/cases.json");
const MANIFEST: &str = include_str!("../../../tests/fixtures/product-coverage-v1/manifest.json");

#[derive(Deserialize)]
struct Case {
    id: String,
    search: SearchRequest,
    expected_ids: Vec<String>,
}

fn database() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    db::configure(&conn).unwrap();
    let modules: Vec<Box<dyn AppModule>> = vec![Box::new(RetrievalModule)];
    db::migrate(&mut conn, &modules).unwrap();
    repo::load_fixtures(
        &mut conn,
        &serde_json::from_str::<Vec<FixtureMemory>>(CORPUS).unwrap(),
    )
    .unwrap();
    conn
}

fn cases() -> Vec<Case> {
    serde_json::from_str(CASES).unwrap()
}

fn request(id: &str) -> Value {
    let case = cases().into_iter().find(|c| c.id == id).unwrap();
    json!({"contract_version":1,"search":case.search})
}

fn ids(result: &SearchResponse) -> Vec<String> {
    let mut ids: Vec<_> = result
        .candidates
        .iter()
        .map(|c| c.memory_id.clone())
        .collect();
    ids.sort();
    ids
}

#[test]
fn versioned_matrix_checks_both_query_modes_and_exact_allowed_sets() {
    let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
    assert_eq!(
        manifest["sha256"]["corpus.json"],
        sha256_hex(CORPUS.as_bytes())
    );
    assert_eq!(
        manifest["sha256"]["cases.json"],
        sha256_hex(CASES.as_bytes())
    );
    let conn = database();
    let cases = cases();
    assert_eq!(manifest["cases"], cases.len());
    for case in cases {
        for mode in [QueryMode::Natural, QueryMode::Keywords] {
            let mut search = case.search.clone();
            search.query_mode = Some(mode);
            let result = service::search(&conn, &search).unwrap();
            let mut expected = case.expected_ids.clone();
            expected.sort();
            assert_eq!(ids(&result), expected, "{} / {mode:?}", case.id);
            assert_eq!(result.coverage.retrieval, "complete", "{}", case.id);
            assert_eq!(result.coverage.matched_memories, expected.len());
            assert!(!result.coverage.truncated);
        }
    }
}

#[test]
fn removing_entity_and_type_filters_exposes_only_the_expected_controls() {
    let conn = database();
    let mut search: SearchRequest =
        serde_json::from_value(request("private-intersection")["search"].clone()).unwrap();
    assert_eq!(
        ids(&service::search(&conn, &search).unwrap()),
        ["private-allowed"]
    );
    search.entity_id = None;
    assert_eq!(
        ids(&service::search(&conn, &search).unwrap()),
        ["private-allowed", "private-entity"]
    );
    search.memory_types.clear();
    assert_eq!(
        ids(&service::search(&conn, &search).unwrap()),
        ["private-allowed", "private-entity", "private-type"]
    );
    // Actor and scope remain enforced even after both optional filters are removed.
    search.actor_id = Some("other".into());
    assert_eq!(
        ids(&service::search(&conn, &search).unwrap()),
        ["private-actor"]
    );
    search.actor_id = Some("local".into());
    search.scope_id = "private".into();
    assert_eq!(
        ids(&service::search(&conn, &search).unwrap()),
        ["private-scope"]
    );
}

#[test]
fn zero_bm25_weight_changes_scoring_without_removing_field_matches() {
    let conn = database();
    let mut search: SearchRequest =
        serde_json::from_value(request("long-original-tail")["search"].clone()).unwrap();
    // Give the summary a distinct term so this test diagnoses which field matched.
    conn.execute(
        "UPDATE search_documents SET summary='Resumen sin el dato terminal' WHERE memory_id='long'",
        [],
    )
    .unwrap();
    let original = service::search(&conn, &search).unwrap();
    assert_eq!(ids(&original), ["long"]);
    search.query = "terminal".into();
    assert_eq!(ids(&service::search(&conn, &search).unwrap()), ["long"]);
    // BM25 zero weights affect ranking, not FTS MATCH eligibility.
    search.bm25_weights = Some([0.0, 1.0]);
    search.query = "zafirofinal".into();
    assert_eq!(ids(&service::search(&conn, &search).unwrap()), ["long"]);
}

#[test]
fn sixty_four_revisions_keep_current_projection_and_reject_stale_seed_selection() {
    let mut conn = database();
    let mut memory = serde_json::from_str::<Vec<FixtureMemory>>(CORPUS)
        .unwrap()
        .into_iter()
        .find(|m| m.id == "private-allowed")
        .unwrap();
    let mut search: SearchRequest =
        serde_json::from_value(request("private-intersection")["search"].clone()).unwrap();
    search.query = "revisionmarker".into();
    for revision in 2..=65 {
        memory.revision = revision;
        memory.original_text = format!("revisionmarker ciclo{revision} conserva condiciones.");
        memory.summary = memory.original_text.clone();
        repo::load_fixtures(&mut conn, &[memory.clone()]).unwrap();
        let found = service::search(&conn, &search).unwrap();
        assert_eq!(ids(&found), ["private-allowed"]);
        assert_eq!(found.candidates[0].revision, revision);
        assert_eq!(found.coverage.retrieval, "complete");
        assert_eq!(
            found.coverage.eligible_memories,
            found.coverage.indexed_memories
        );
        // Both IDs are still authorized, but the presented revision is stale.
        let stale: SeedRequest = serde_json::from_value(json!({
            "search":search,"presented":[{"memory_id":"private-allowed","revision":revision-1}],
            "selected":[{"memory_id":"private-allowed","revision":revision-1}],
            "classifier":"synthetic-contract-double","classification_status":"complete"
        }))
        .unwrap();
        assert!(service::compose_seed(&conn, &stale).is_err());
        if revision % 8 == 0 {
            let mut uncommitted = memory.clone();
            uncommitted.revision += 1;
            uncommitted.summary = "SYNTHETIC_UNCOMMITTED_TEXT".into();
            // First row would succeed; second duplicates an existing revision.
            assert!(repo::load_fixtures(&mut conn, &[uncommitted, memory.clone()]).is_err());
            let after = service::search(&conn, &search).unwrap();
            assert_eq!(after.candidates[0].revision, revision);
            assert_eq!(after.candidates[0].summary, memory.summary);
        }
    }
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM memory_revisions WHERE memory_id='private-allowed'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        65
    );
}

#[test]
fn unicode_segmentation_reconstructs_all_bytes_and_negated_conditions() {
    for text in serde_json::from_str::<Vec<FixtureMemory>>(CORPUS)
        .unwrap()
        .into_iter()
        .map(|m| m.original_text)
        .chain([
            "\r\nNo enviar.\r\n\r\nSolo si dice «sí. mañana». 👩🏽‍💻 cafe\u{301}".into(),
            "```sql\nSELECT 'no. borrar';\n```\nNo ejecutar.".into(),
        ])
    {
        let result = segmentation::segment(&text).unwrap();
        assert_eq!(result.coverage, "complete");
        assert_eq!(result.original_sha256, sha256_hex(text.as_bytes()));
        let mut offset = 0;
        let mut reconstructed = String::new();
        for part in result.segments {
            assert_eq!(part.start_byte, offset);
            assert_eq!(
                text.get(part.start_byte..part.end_byte),
                Some(part.text.as_str())
            );
            offset = part.end_byte;
            reconstructed.push_str(&part.text);
        }
        assert_eq!(offset, text.len());
        assert_eq!(reconstructed, text);
    }
}

enum Reply<'a> {
    Select(&'a str),
    Abstain,
    Fail,
}

struct Scripted<'a> {
    reply: Reply<'a>,
    review_fails: bool,
    seen: RefCell<Vec<Vec<Candidate>>>,
}

fn receipt(id: Option<&str>) -> DecisionResult {
    DecisionResult {
        selected_id: id.map(str::to_owned),
        served_model: Some("synthetic-contract-double".into()),
        input_tokens: Some(17),
        output_tokens: Some(2),
        reported_cost_usd: Some(0.001),
        per_memory: None,
        consensus: None,
        applicability: None,
        slot_evidence: None,
    }
}

fn failure() -> DecisionError {
    DecisionError {
        code: "provider_unavailable",
        http_status: Some(503),
        diagnostic: None,
        partial_usage: Some(PartialDecisionUsage {
            served_model: Some("synthetic-contract-double".into()),
            input_tokens: Some(7),
            output_tokens: None,
            reported_cost_usd: None,
        }),
    }
}

fn bounded(
    remaining: usize,
    stage: &'static str,
    candidates: &[Candidate],
    result: Result<DecisionResult, DecisionError>,
) -> BudgetedDecision {
    let mut budget = token_budget::InputBudget::new(remaining);
    let payload = json!({"synthetic_contract_double":true,"candidates":candidates}).to_string();
    let result = match budget.inspect(stage, payload.as_bytes()) {
        Ok(()) => result,
        Err(code) => Err(DecisionError {
            code,
            http_status: None,
            diagnostic: None,
            partial_usage: None,
        }),
    };
    BudgetedDecision {
        result,
        requests: budget.requests,
    }
}

impl Classifier for Scripted<'_> {
    fn classify(
        &self,
        _: &str,
        _: &str,
        _: &[Candidate],
        _: ClassifierProvider,
        _: SelectionPolicy,
    ) -> Result<DecisionResult, DecisionError> {
        panic!("product must use the bounded entry point")
    }
    fn classify_budgeted_until(
        &self,
        _: &str,
        _: &str,
        candidates: &[Candidate],
        _: ClassifierProvider,
        _: SelectionPolicy,
        _: Instant,
        remaining: usize,
    ) -> BudgetedDecision {
        self.seen.borrow_mut().push(candidates.to_vec());
        bounded(
            remaining,
            "classification",
            candidates,
            match self.reply {
                Reply::Select(id) => Ok(receipt(Some(id))),
                Reply::Abstain => Ok(receipt(None)),
                Reply::Fail => Err(failure()),
            },
        )
    }
    fn suggest_equivalent_budgeted_until(
        &self,
        _: &str,
        _: &Candidate,
        candidates: &[Candidate],
        _: ClassifierProvider,
        _: Instant,
        remaining: usize,
    ) -> BudgetedDecision {
        bounded(
            remaining,
            "equivalence_review",
            candidates,
            if self.review_fails {
                Err(failure())
            } else {
                Ok(receipt(None))
            },
        )
    }
}

fn run(
    conn: &Connection,
    input: &Value,
    reply: Reply<'_>,
    review_fails: bool,
) -> (Value, Vec<Vec<Candidate>>) {
    let classifier = Scripted {
        reply,
        review_fails,
        seen: RefCell::new(vec![]),
    };
    let response =
        serde_json::to_value(product::execute_json(conn, &input.to_string(), &classifier)).unwrap();
    (response, classifier.seen.into_inner())
}

#[test]
fn product_preserves_unicode_conditions_and_declared_sources_in_selected_seed() {
    let conn = database();
    for (case, id) in [
        ("spanish-diacritics", "es"),
        ("arabic-literal", "ar"),
        ("japanese-literal", "ja"),
        ("unicode-combining", "unicode"),
        ("long-original-tail", "long"),
    ] {
        let (out, seen) = run(&conn, &request(case), Reply::Select(id), false);
        assert_eq!(out["outcome"], "evidence_selected", "{case}: {out}");
        assert_eq!(seen[0].len(), 1);
        assert_eq!(out["seed"]["memories"][0]["summary"], seen[0][0].summary);
        assert_eq!(out["assessments"][0]["relation"], "unassessed");
        let text = out["seed"]["seed_text"].as_str().unwrap();
        assert!(text.contains(&seen[0][0].summary));
        if let Some(conditions) = &seen[0][0].conditions {
            assert!(text.contains(conditions));
        }
        if let Some(source) = &seen[0][0].source {
            assert!(text.contains(&source.reference));
            assert_eq!(
                out["seed"]["memories"][0]["provenance_status"],
                "declared_unverified"
            );
        }
    }
}

#[test]
fn contradictions_remain_distinct_and_review_failure_preserves_initial_evidence() {
    let conn = database();
    let (out, seen) = run(
        &conn,
        &request("contradiction-kept"),
        Reply::Select("contradiction-no"),
        true,
    );
    assert_eq!(seen[0].len(), 2);
    assert_eq!(out["execution_status"], "degraded");
    assert_eq!(out["outcome"], "evidence_selected");
    assert_eq!(out["coverage"]["equivalence_review"], "unavailable");
    assert_eq!(out["seed"]["memories"].as_array().unwrap().len(), 1);
    assert_eq!(out["seed"]["memories"][0]["memory_id"], "contradiction-no");
    let text = out["seed"]["seed_text"].as_str().unwrap();
    assert!(text.contains("no está abierta"));
    assert!(text.contains("Solo si la luz está roja."));
    assert_eq!(out["usage"]["classifier_invocations"], 2);
    assert_eq!(out["usage"]["input_tokens"], 24);
    assert_eq!(out["usage"]["reported_cost_usd"], 0.001);
}

#[test]
fn forged_selection_cannot_publish_sensitive_text_and_failure_does_not_poison_next_call() {
    let conn = database();
    let input = request("private-intersection");
    for reply in [
        Reply::Select("private-actor"),
        Reply::Fail,
        Reply::Abstain,
        Reply::Select("private-allowed"),
    ] {
        let (out, seen) = run(&conn, &input, reply, false);
        assert_eq!(seen[0].len(), 1);
        assert_eq!(seen[0][0].memory_id, "private-allowed");
        let serialized = out.to_string();
        for forbidden in [
            "SYNTHETIC_PRIVATE_ACTOR_782",
            "SYNTHETIC_PRIVATE_SCOPE_349",
            "SYNTHETIC_PRIVATE_ENTITY_516",
            "SYNTHETIC_PRIVATE_TYPE_625",
        ] {
            assert!(!serialized.contains(forbidden));
        }
        match out["outcome"].as_str().unwrap() {
            "evidence_selected" => {
                assert_eq!(out["seed"]["memories"][0]["memory_id"], "private-allowed")
            }
            "abstained" => assert!(out["seed"]["memories"].as_array().unwrap().is_empty()),
            _ => {
                assert_eq!(out["execution_status"], "degraded");
                assert!(out["seed"].is_null());
            }
        }
    }
}

#[test]
fn incomplete_failure_receipt_stays_unknown_without_retry_or_fabricated_zero_cost() {
    let conn = database();
    let (out, seen) = run(&conn, &request("private-intersection"), Reply::Fail, false);
    assert_eq!(seen.len(), 1);
    assert_eq!(out["execution_status"], "degraded");
    assert!(out["seed"].is_null());
    assert_eq!(out["usage"]["classifier_invocations"], 1);
    assert_eq!(out["usage"]["receipt_status"], "partial");
    assert_eq!(out["usage"]["input_tokens"], 7);
    assert!(out["usage"]["output_tokens"].is_null());
    assert!(out["usage"]["reported_cost_usd"].is_null());
    assert_eq!(out["usage"]["cost_status"], "unknown");
    assert_eq!(out["retrieval"]["candidates"].as_array().unwrap().len(), 1);
}

#[test]
fn empty_scope_ambiguity_and_oversized_context_do_not_reach_classifier() {
    let conn = database();
    let (out, seen) = run(&conn, &request("empty-scope"), Reply::Fail, false);
    assert!(seen.is_empty());
    assert_eq!(out["outcome"], "no_candidates");
    let mut input = request("homonym-unresolved");
    input["unresolved_references"] =
        json!([{"start_byte":0,"end_byte":4,"required_field":"entity"}]);
    let (out, seen) = run(&conn, &input, Reply::Fail, false);
    assert!(seen.is_empty());
    assert_eq!(out["outcome"], "clarification_required");
    let mut input = request("spanish-diacritics");
    input["context"] = json!([{"id":"large","source":"synthetic","text":"é".repeat(20000)}]);
    let (out, seen) = run(&conn, &input, Reply::Fail, false);
    assert!(seen.is_empty());
    assert!(out["seed"].is_null());
    assert_ne!(out["outcome"], "no_candidates");
}
