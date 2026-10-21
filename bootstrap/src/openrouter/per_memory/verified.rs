use super::{binary_payload, parse_binary_response};
use crate::openrouter::{OpenRouterClassifier, failure, merge_known_receipts};
use iromem_retrieval::classifier::{DecisionError, PartialDecisionUsage};
use iromem_retrieval::model::{Candidate, ClassifierProvider, DecisionResult};
use iromem_retrieval::per_memory::verification::{CheckedMemory, StageUsage, VerificationTrace};
use iromem_retrieval::per_memory::{EvidenceRelation, PerMemoryTrace};
use serde_json::Value;

pub(in crate::openrouter) fn classify(
    client: &OpenRouterClassifier,
    key: &str,
    query: &str,
    scope: &str,
    candidates: &[Candidate],
    provider: ClassifierProvider,
) -> Result<DecisionResult, DecisionError> {
    with_requests(query, scope, candidates, provider, &|payload| {
        client.request(key, payload)
    })
}

pub(super) fn receipt(result: &DecisionResult, provider: ClassifierProvider) -> StageUsage {
    receipt_for_model(result, provider.model_id())
}
pub(super) fn receipt_for_model(result: &DecisionResult, model: &str) -> StageUsage {
    StageUsage {
        requested_model: model.into(),
        served_model: result.served_model.clone(),
        input_tokens: result.input_tokens,
        output_tokens: result.output_tokens,
        reported_cost_usd: result.reported_cost_usd,
    }
}
pub(super) fn partial(result: &DecisionResult) -> PartialDecisionUsage {
    PartialDecisionUsage {
        served_model: result.served_model.clone(),
        input_tokens: result.input_tokens,
        output_tokens: result.output_tokens,
        reported_cost_usd: result.reported_cost_usd,
    }
}

fn with_requests(
    query: &str,
    scope: &str,
    candidates: &[Candidate],
    provider: ClassifierProvider,
    request: &(impl Fn(&Value) -> Result<Value, DecisionError> + Sync),
) -> Result<DecisionResult, DecisionError> {
    let candidates = &candidates[..candidates.len().min(8)];
    let body = request(&binary_payload(query, scope, candidates, provider)).map_err(|mut e| {
        e.code = "verified_proposal_failed";
        e
    })?;
    let proposal = parse_binary_response(body, candidates).map_err(|mut e| {
        e.code = "verified_proposal_invalid";
        e
    })?;
    let proposal_items = proposal
        .per_memory
        .as_ref()
        .expect("binary parser produces trace")
        .items
        .clone();
    let positive: Vec<_> = proposal_items
        .iter()
        .enumerate()
        .filter(|(_, p)| p.relation == EvidenceRelation::UsefulEvidence)
        .map(|(i, _)| i)
        .collect();
    let results = std::thread::scope(|threads| {
        let handles: Vec<_> = positive
            .iter()
            .map(|&i| {
                threads.spawn(move || {
                    let one = std::slice::from_ref(&candidates[i]);
                    let payload = binary_payload(query, scope, one, ClassifierProvider::Decisions);
                    request(&payload).and_then(|body| parse_binary_response(body, one))
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err(failure("verified_thread_failed")))
            })
            .collect::<Vec<_>>()
    });
    if results.iter().any(Result::is_err) {
        let mut known = Some(partial(&proposal));
        let mut status = None;
        for result in &results {
            known = merge_known_receipts(
                known,
                match result {
                    Ok(value) => Some(partial(value)),
                    Err(error) => {
                        if status.is_none() {
                            status = error.http_status;
                        }
                        error.partial_usage.clone()
                    }
                },
            );
        }
        if let Some(usage) = known.as_mut() {
            usage.served_model = None;
        }
        return Err(DecisionError {
            code: "verified_check_failed",
            http_status: status,
            partial_usage: known,
        });
    }
    let mut items = proposal_items.clone();
    let mut checks = Vec::with_capacity(positive.len());
    for (index, result) in positive.into_iter().zip(results) {
        let result = result.expect("all errors handled before composition");
        let useful = result.selected_id.is_some();
        if !useful {
            items[index].relation = EvidenceRelation::Unrelated;
        }
        checks.push(CheckedMemory {
            complete_paths: None,
            paths: None,
            memory_id: candidates[index].memory_id.clone(),
            revision: candidates[index].revision,
            useful,
            usage: receipt(&result, ClassifierProvider::Decisions),
        });
    }
    let verification = VerificationTrace {
        batch: None,
        kind_proposal: None,
        role_proposal: None,
        scoped_proposal: None,
        proposal: proposal_items,
        proposal_usage: receipt(&proposal, provider),
        checks,
    };
    let totals = verification
        .totals()
        .ok_or_else(|| failure("verified_usage_overflow"))?;
    let trace = PerMemoryTrace {
        choice_scores: None,
        scoped_checks: None,
        factor_checks: None,
        request_sha256: None,
        items,
        verification: Some(verification),
    };
    Ok(DecisionResult {
        selected_id: trace.selected_ids().first().map(|id| (*id).to_owned()),
        per_memory: Some(trace),
        consensus: None,
        applicability: None,
        slot_evidence: None,
        served_model: totals.served_model,
        input_tokens: totals.input_tokens,
        output_tokens: totals.output_tokens,
        reported_cost_usd: totals.reported_cost_usd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn run(mode: &str) -> (Result<DecisionResult, DecisionError>, usize) {
        let candidates: Vec<_> = (0..3)
            .map(|i| Candidate {
                memory_id: format!("m{i}"),
                revision: 1,
                summary: format!("fact-{i}"),
                conditions: None,
                source: None,
                state: "active".into(),
                bm25: 0.0,
            })
            .collect();
        let calls = AtomicUsize::new(0);
        let response = with_requests(
            "query",
            "scope",
            &candidates,
            ClassifierProvider::Jev,
            &|payload| {
                calls.fetch_add(1, Ordering::SeqCst);
                assert_eq!(payload["state"]["user_prompt"], "query");
                let questions = payload["questions"].as_object().unwrap();
                let proposing = payload["model"] == ClassifierProvider::Jev.model_id();
                let second = questions["memory_00"]["instructions"]
                    .as_str()
                    .unwrap()
                    .contains("fact-1");
                if !proposing {
                    assert_eq!(questions.len(), 1);
                    let text = questions["memory_00"]["instructions"].as_str().unwrap();
                    assert!(!(text.contains("fact-0") && text.contains("fact-1")));
                    if mode == "error" && second {
                        return Err(DecisionError {
                            code: "test",
                            http_status: Some(503),
                            partial_usage: Some(PartialDecisionUsage {
                                served_model: Some("verifier".into()),
                                input_tokens: Some(20),
                                output_tokens: Some(0),
                                reported_cost_usd: Some(0.003),
                            }),
                        });
                    }
                }
                let answers:serde_json::Map<String,Value>=questions.keys().enumerate().map(|(i,key)| {
                let useful=if proposing {mode!="none"&&i<2}else{!second};
                (key.clone(),json!({"type":"choice","choice":if useful{"A"}else{"none"},"probabilities":{"A":0.5,"none":0.5}}))
            }).collect();
                Ok(
                    json!({"answers":answers,"model":if proposing{"proposer"}else{"verifier"},"usage":{"input_tokens":if proposing{10}else{20},"output_tokens":0,"cost":if proposing{json!(0.001)}else if mode=="missing"&&second{Value::Null}else{json!(0.002)}}}),
                )
            },
        );
        (response, calls.load(Ordering::SeqCst))
    }

    #[test]
    fn each_proposal_is_checked_alone_and_receipts_are_counted_once() {
        let (result, calls) = run("normal");
        let result = result.unwrap();
        assert_eq!(calls, 3);
        assert_eq!(result.input_tokens, Some(50));
        assert!((result.reported_cost_usd.unwrap() - 0.005).abs() < 1e-12);
        assert!(result.served_model.is_none());
        let trace = result.per_memory.as_ref().unwrap();
        assert_eq!(trace.selected_ids(), ["m0"]);
        assert!(
            trace
                .verification
                .as_ref()
                .unwrap()
                .valid(&trace.items, ClassifierProvider::Jev.model_id())
        );
        assert!(trace.verification.as_ref().unwrap().usage_matches(&result));
        let (result, calls) = run("none");
        assert_eq!(calls, 1);
        assert!(
            result
                .unwrap()
                .per_memory
                .unwrap()
                .selected_ids()
                .is_empty()
        );
    }

    #[test]
    fn failed_check_is_atomic_and_missing_receipt_is_never_a_complete_cost() {
        let (result, calls) = run("error");
        let error = result.unwrap_err();
        assert_eq!(calls, 3);
        assert_eq!(error.http_status, Some(503));
        assert!((error.partial_usage.unwrap().reported_cost_usd.unwrap() - 0.006).abs() < 1e-12);
        let (result, calls) = run("missing");
        let result = result.unwrap();
        assert_eq!(calls, 3);
        assert!(result.reported_cost_usd.is_none());
        let totals = result
            .per_memory
            .unwrap()
            .verification
            .unwrap()
            .totals()
            .unwrap();
        assert!((totals.known_cost_usd.unwrap() - 0.003).abs() < 1e-12);
        assert_eq!(totals.missing_cost_receipts, 1);
    }
}
