use crate::classifier::Classifier;
use crate::model::{
    QueryMode, RecallRequest, RecallResponse, RecallTimings, SearchRequest, SearchResponse,
    SeedMemory, SeedRequest, SeedResponse, SelectedCandidate,
};
use rusqlite::Connection;
use std::collections::HashSet;
use std::time::Instant;

pub fn search(conn: &Connection, request: &SearchRequest) -> Result<SearchResponse, String> {
    validate_search_request(request)?;
    search_validated(conn, request)
}

pub(crate) fn validate_search_request(request: &SearchRequest) -> Result<(), String> {
    if request.query.trim().is_empty() || request.query.chars().count() > 8000 {
        return Err("query debe tener entre 1 y 8000 caracteres".into());
    }
    if request.scope_kind.trim().is_empty() || request.scope_id.trim().is_empty() {
        return Err("scope_kind y scope_id son obligatorios".into());
    }
    if request
        .entity_id
        .as_ref()
        .is_some_and(|id| id.trim().is_empty() || id.len() > 128)
    {
        return Err("entity_id debe tener entre 1 y 128 bytes".into());
    }
    if !valid_utc(&request.as_of) {
        return Err("as_of debe usar UTC RFC 3339 con milisegundos".into());
    }
    if !(1..=20).contains(&request.limit.unwrap_or(8)) {
        return Err("limit debe estar entre 1 y 20".into());
    }
    Ok(())
}

fn search_validated(conn: &Connection, request: &SearchRequest) -> Result<SearchResponse, String> {
    let (expression, method) = match request.query_mode.unwrap_or(QueryMode::Keywords) {
        QueryMode::Natural => (fts_expression(&request.query), "fts5"),
        QueryMode::Keywords => (keyword_expression(&request.query), "fts5_keywords"),
    };
    if expression.is_empty() {
        return Ok(SearchResponse {
            status: "complete",
            method,
            candidates: vec![],
            semantic_coverage: "disabled",
        });
    }
    let candidates = crate::repo::search(conn, request, &expression)?;
    Ok(SearchResponse {
        status: "complete",
        method,
        candidates,
        semantic_coverage: "disabled",
    })
}

fn witness_valid(
    query: &str,
    candidates: &[crate::model::Candidate],
    policy: crate::model::SelectionPolicy,
    decision: &crate::model::DecisionResult,
) -> bool {
    if policy != crate::model::SelectionPolicy::WitnessV1 {
        return true;
    }
    let Some(trace) = &decision.applicability else {
        return decision.selected_id.is_none();
    };
    let Some(candidate) = candidates.iter().find(|c| c.memory_id == trace.proposal_id) else {
        return false;
    };
    let Ok((outcome, reasons)) =
        crate::applicability::check(query, &candidate.summary, &trace.witness)
    else {
        return false;
    };
    outcome == trace.outcome
        && reasons == trace.reasons
        && if outcome == "accepted" {
            decision.selected_id.as_ref() == Some(&trace.proposal_id)
        } else {
            decision.selected_id.is_none()
        }
}

fn slot_evidence_valid(
    query: &str,
    candidates: &[crate::model::Candidate],
    policy: crate::model::SelectionPolicy,
    decision: &crate::model::DecisionResult,
) -> bool {
    if !matches!(
        policy,
        crate::model::SelectionPolicy::SlotEvidenceV1
            | crate::model::SelectionPolicy::SlotEvidenceV2
            | crate::model::SelectionPolicy::SlotEvidenceV3
    ) {
        return decision.slot_evidence.is_none();
    }
    decision.slot_evidence.as_ref().is_some_and(|trace| {
        trace.validate(query, candidates)
            && decision.per_memory.is_none()
            && decision.selected_id.is_none()
    })
}

fn normalize_slot_evidence(
    query: &str,
    candidates: &[crate::model::Candidate],
    decision: &mut crate::model::DecisionResult,
) {
    if let Some(trace) = decision.slot_evidence.as_mut() {
        trace.demote_mismatched_target_attributions();
        trace.demote_ambiguous_attributions(query);
        trace.qualify_temporal_evidence(query, candidates);
    }
}

fn consensus_valid(
    candidates: &[crate::model::Candidate],
    policy: crate::model::SelectionPolicy,
    decision: &crate::model::DecisionResult,
) -> bool {
    let Some(limit) = policy.consensus_candidate_limit() else {
        return decision.consensus.is_none();
    };
    let Some(trace) = &decision.consensus else {
        return false;
    };
    let count = candidates.len().min(limit);
    if trace.classified_candidate_count != count
        || trace
            .votes
            .iter()
            .flatten()
            .any(|id| !candidates[..count].iter().any(|c| &c.memory_id == id))
    {
        return false;
    }
    decision.selected_id.as_deref() == policy.resolve_votes(&trace.votes)
}

pub fn recall(
    conn: &Connection,
    request: &RecallRequest,
    classifier: &impl Classifier,
) -> Result<RecallResponse, String> {
    let started = Instant::now();
    let mut search_request = request.search.clone();
    if search_request.limit.is_none() {
        search_request.limit = Some(20);
    }
    let retrieval = search(conn, &search_request)?;
    let search_ms = started.elapsed().as_secs_f64() * 1000.0;
    let presented: Vec<SelectedCandidate> = retrieval
        .candidates
        .iter()
        .map(|item| SelectedCandidate {
            memory_id: item.memory_id.clone(),
            revision: item.revision,
        })
        .collect();
    let classifier_started = Instant::now();
    let (classification_status, error_code, provider_http_status, decision, partial_usage) =
        if presented.is_empty() {
            ("skipped_empty", None, None, None, None)
        } else {
            let classified = classifier
                .classify(
                    &search_request.query,
                    &search_request.scope_id,
                    &retrieval.candidates,
                    request.provider,
                    request.selection_policy,
                )
                .map(|mut decision| {
                    // Provider adapters may already normalize the trace. Repeat the
                    // fail-closed transform at the service boundary so every
                    // Classifier implementation obeys the same retrieval policy.
                    normalize_slot_evidence(
                        &search_request.query,
                        &retrieval.candidates,
                        &mut decision,
                    );
                    decision
                });
            match classified {
                Ok(decision)
                    if witness_valid(
                        &search_request.query,
                        &retrieval.candidates,
                        request.selection_policy,
                        &decision,
                    ) && slot_evidence_valid(
                        &search_request.query,
                        &retrieval.candidates,
                        request.selection_policy,
                        &decision,
                    ) && consensus_valid(
                        &retrieval.candidates,
                        request.selection_policy,
                        &decision,
                    ) && match request.selection_policy.per_memory_candidate_limit() {
                        Some(_limit)
                            if matches!(
                                request.selection_policy,
                                crate::model::SelectionPolicy::SlotEvidenceV1
                                    | crate::model::SelectionPolicy::SlotEvidenceV2
                                    | crate::model::SelectionPolicy::SlotEvidenceV3
                            ) =>
                        {
                            decision.per_memory.is_none()
                                && decision.slot_evidence.is_some()
                                && decision.applicability.is_none()
                        }
                        Some(limit) => decision.per_memory.as_ref().is_some_and(|trace| {
                            trace.relations_match_task(matches!(
                                request.selection_policy,
                                crate::model::SelectionPolicy::PerMemoryBinaryV1
                                    | crate::model::SelectionPolicy::PerMemoryBinaryV2
                                    | crate::model::SelectionPolicy::PerMemoryBinaryV3
                                    | crate::model::SelectionPolicy::PerMemoryBinaryV4
                                    | crate::model::SelectionPolicy::PerMemoryBinaryV5
                                    | crate::model::SelectionPolicy::PerMemoryExamplesV1
                                    | crate::model::SelectionPolicy::PerMemoryFactorsV1
                                    | crate::model::SelectionPolicy::PerMemoryScopedV1
                                    | crate::model::SelectionPolicy::PerMemoryScopedV2
                                    | crate::model::SelectionPolicy::PerMemoryScopedV3
                                    | crate::model::SelectionPolicy::PerMemoryScopedAnchorV1
                                    | crate::model::SelectionPolicy::PerMemoryScoredV1
                                    | crate::model::SelectionPolicy::PerMemoryVerifiedV1
                                    | crate::model::SelectionPolicy::PerMemoryCorrectionVerifiedV1
                                    | crate::model::SelectionPolicy::PerMemoryCorrectionVerifiedV2
                                    | crate::model::SelectionPolicy::PerMemoryCorrectionVerifiedV3
                                    | crate::model::SelectionPolicy::PerMemoryCorrectionVerifiedV4
                                    | crate::model::SelectionPolicy::PerMemoryCorrectionVerifiedV5
                                    | crate::model::SelectionPolicy::PerMemoryRoleVerifiedV1
                                    | crate::model::SelectionPolicy::PerMemoryRoleVerifiedV2
                                    | crate::model::SelectionPolicy::PerMemoryRoleBatchV1
                                    | crate::model::SelectionPolicy::PerMemoryCompletePremiseV1
                            )) && (trace.scoped_checks.is_some()
                                == (matches!(
                                    request.selection_policy,
                                    crate::model::SelectionPolicy::PerMemoryScopedV1
                                        | crate::model::SelectionPolicy::PerMemoryScopedV2
                                        | crate::model::SelectionPolicy::PerMemoryScopedV3
                                        | crate::model::SelectionPolicy::PerMemoryScopedAnchorV1
                                )))
                                && (trace.factor_checks.is_some()
                                    == (request.selection_policy
                                        == crate::model::SelectionPolicy::PerMemoryFactorsV1))
                                && (trace.request_sha256.is_some()
                                    == (request.selection_policy
                                        == crate::model::SelectionPolicy::PerMemoryScoredV1))
                                && (trace.choice_scores.is_some()
                                    == (request.selection_policy
                                        == crate::model::SelectionPolicy::PerMemoryScoredV1))
                                && (if request.selection_policy.uses_verification() {
                                    trace.verification.as_ref().is_some_and(|v| {
                                        v.matches_policy(request.selection_policy)
                                            && v.valid_for_policy(
                                                &trace.items,
                                                request.provider.model_id(),
                                                request.selection_policy,
                                            )
                                            && v.usage_matches(&decision)
                                    })
                                } else {
                                    trace.verification.is_none()
                                })
                                && trace.anchors_valid(
                                    &search_request.query,
                                    &retrieval.candidates[..retrieval.candidates.len().min(limit)],
                                    request.selection_policy
                                        == crate::model::SelectionPolicy::PerMemoryScopedAnchorV1,
                                )
                                && trace.valid(
                                    &retrieval.candidates[..retrieval.candidates.len().min(limit)],
                                    decision.selected_id.as_deref(),
                                )
                        }),
                        None => decision.per_memory.is_none(),
                    } && decision.selected_id.as_ref().is_none_or(|id| {
                        presented.iter().any(|candidate| candidate.memory_id == *id)
                    }) =>
                {
                    ("complete", None, None, Some(decision), None)
                }
                Ok(decision) => (
                    "failed",
                    Some(
                        if request
                            .selection_policy
                            .per_memory_candidate_limit()
                            .is_some()
                        {
                            "invalid_per_memory"
                        } else if request
                            .selection_policy
                            .consensus_candidate_limit()
                            .is_some()
                        {
                            "invalid_consensus"
                        } else if request.selection_policy
                            == crate::model::SelectionPolicy::WitnessV1
                        {
                            "invalid_applicability"
                        } else {
                            "invalid_choice"
                        },
                    ),
                    None,
                    None,
                    Some(crate::classifier::PartialDecisionUsage {
                        served_model: decision.served_model,
                        input_tokens: decision.input_tokens,
                        output_tokens: decision.output_tokens,
                        reported_cost_usd: decision.reported_cost_usd,
                    }),
                ),
                Err(error) => (
                    "failed",
                    Some(error.code),
                    error.http_status,
                    None,
                    error.partial_usage,
                ),
            }
        };
    let classifier_ms = classifier_started.elapsed().as_secs_f64() * 1000.0;
    let ids: Vec<String> = decision.as_ref().map_or_else(Vec::new, |item| {
        if let Some(trace) = &item.slot_evidence {
            return trace
                .selected_ids(&retrieval.candidates, &search_request.query)
                .iter()
                .cloned()
                .collect();
        }
        item.per_memory.as_ref().map_or_else(
            || item.selected_id.clone().into_iter().collect(),
            |trace| {
                trace
                    .selected_ids()
                    .into_iter()
                    .map(str::to_owned)
                    .collect()
            },
        )
    });
    let selected = presented
        .iter()
        .filter(|candidate| ids.contains(&candidate.memory_id))
        .map(|candidate| SelectedCandidate {
            memory_id: candidate.memory_id.clone(),
            revision: candidate.revision,
        })
        .collect();
    let composition_started = Instant::now();
    let seed = compose_seed_with_limit(
        conn,
        &SeedRequest {
            search: Some(search_request),
            hybrid: None,
            presented,
            selected,
            classifier: if request.selection_policy
                == crate::model::SelectionPolicy::PerMemoryVerifiedV1
            {
                "per_memory_verified_v1".into()
            } else if let Some(version) = request.selection_policy.correction_verification_version()
            {
                format!("per_memory_correction_verified_v{version}")
            } else if request.selection_policy
                == crate::model::SelectionPolicy::PerMemoryCompletePremiseV1
            {
                "per_memory_complete_premise_v1".into()
            } else if request.selection_policy
                == crate::model::SelectionPolicy::PerMemoryRoleBatchV1
            {
                "per_memory_role_batch_v1".into()
            } else if let Some(version) = request.selection_policy.role_verification_version() {
                format!("per_memory_role_verified_v{version}")
            } else {
                request.provider.model_id().into()
            },
            classification_status: if classification_status == "failed" {
                "failed"
            } else {
                "complete"
            }
            .into(),
        },
        request
            .selection_policy
            .per_memory_candidate_limit()
            .unwrap_or(8),
    )?;
    let composition_ms = composition_started.elapsed().as_secs_f64() * 1000.0;
    Ok(RecallResponse {
        selected_ids: seed.memories.iter().map(|m| m.memory_id.clone()).collect(),
        per_memory: decision.as_ref().and_then(|d| d.per_memory.clone()),
        consensus: decision.as_ref().and_then(|d| d.consensus.clone()),
        applicability: decision.as_ref().and_then(|d| d.applicability.clone()),
        slot_evidence: decision.as_ref().and_then(|d| d.slot_evidence.clone()),
        status: seed.status,
        retrieval,
        classification_status,
        requested_model: request.provider.model_id(),
        served_model: decision
            .as_ref()
            .and_then(|item| item.served_model.clone())
            .or_else(|| {
                partial_usage
                    .as_ref()
                    .and_then(|item| item.served_model.clone())
            }),
        selected_id: decision.as_ref().and_then(|item| item.selected_id.clone()),
        seed,
        input_tokens: decision
            .as_ref()
            .and_then(|item| item.input_tokens)
            .or_else(|| partial_usage.as_ref().and_then(|item| item.input_tokens)),
        output_tokens: decision
            .as_ref()
            .and_then(|item| item.output_tokens)
            .or_else(|| partial_usage.as_ref().and_then(|item| item.output_tokens)),
        reported_cost_usd: decision
            .as_ref()
            .and_then(|item| item.reported_cost_usd)
            .or_else(|| {
                partial_usage
                    .as_ref()
                    .and_then(|item| item.reported_cost_usd)
            }),
        error_code,
        provider_http_status,
        timings: RecallTimings {
            search_ms,
            classifier_ms,
            composition_ms,
            total_ms: started.elapsed().as_secs_f64() * 1000.0,
        },
    })
}

pub fn compose_seed(conn: &Connection, request: &SeedRequest) -> Result<SeedResponse, String> {
    compose_seed_with_limit(conn, request, 8)
}

pub(crate) fn compose_seed_with_limit(
    conn: &Connection,
    request: &SeedRequest,
    selected_limit: usize,
) -> Result<SeedResponse, String> {
    if request.classifier.trim().is_empty() || request.classifier.chars().count() > 128 {
        return Err("classifier debe tener entre 1 y 128 caracteres".into());
    }
    if request.classification_status != "complete" && request.classification_status != "failed" {
        return Err("classification_status debe ser complete o failed".into());
    }
    let (method, semantic_coverage, retrieval_status, candidates): (
        &'static str,
        &'static str,
        &'static str,
        Vec<SeedMemory>,
    ) = match (&request.search, &request.hybrid) {
        (Some(text_request), None) => {
            let result = search(conn, text_request)?;
            let candidates = result
                .candidates
                .into_iter()
                .map(|candidate| {
                    let provenance_status = if candidate.source.is_some() {
                        "declared_unverified"
                    } else {
                        "not_provided"
                    };
                    SeedMemory {
                        memory_id: candidate.memory_id,
                        revision: candidate.revision,
                        scope_kind: text_request.scope_kind.clone(),
                        scope_id: text_request.scope_id.clone(),
                        summary: candidate.summary,
                        conditions: candidate.conditions,
                        source: candidate.source,
                        provenance_status,
                        state: candidate.state,
                    }
                })
                .collect();
            (
                "fts5_external_classifier",
                result.semantic_coverage,
                result.status,
                candidates,
            )
        }
        (None, Some(hybrid_request)) => {
            let result = crate::embedding::search_hybrid(conn, hybrid_request)?;
            let candidates = result
                .candidates
                .into_iter()
                .map(|candidate| {
                    let provenance_status = if candidate.source.is_some() {
                        "declared_unverified"
                    } else {
                        "not_provided"
                    };
                    SeedMemory {
                        memory_id: candidate.memory_id,
                        revision: candidate.revision,
                        scope_kind: hybrid_request.search.scope_kind.clone(),
                        scope_id: hybrid_request.search.scope_id.clone(),
                        summary: candidate.summary,
                        conditions: candidate.conditions,
                        source: candidate.source,
                        provenance_status,
                        state: candidate.state,
                    }
                })
                .collect();
            (
                "hybrid_external_classifier",
                result.semantic_coverage,
                result.status,
                candidates,
            )
        }
        _ => return Err("indica exactamente una búsqueda: search o hybrid".into()),
    };
    if request.presented.len() > 20 {
        return Err("presented no puede contener más de 20 candidatos".into());
    }
    let mut presented_ids = HashSet::new();
    for presented in &request.presented {
        if !presented_ids.insert(&presented.memory_id) {
            return Err("presented contiene duplicados".into());
        }
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.memory_id == presented.memory_id)
            .ok_or_else(|| {
                format!(
                    "presented_id {} no pertenece a los candidatos autorizados",
                    presented.memory_id
                )
            })?;
        if candidate.revision != presented.revision {
            return Err(format!(
                "presented_id {} tiene una revisión obsoleta",
                presented.memory_id
            ));
        }
    }
    if request.classification_status == "failed" {
        if !request.selected.is_empty() {
            return Err("un clasificador fallido no puede seleccionar memorias".into());
        }
        return Ok(SeedResponse {
            status: "degraded",
            method,
            classifier: request.classifier.clone(),
            semantic_coverage,
            memories: vec![],
            potential_equivalents: vec![],
            seed_text: None,
        });
    }
    if request.selected.len() > selected_limit {
        return Err(format!(
            "la semilla no puede contener más de {selected_limit} memorias"
        ));
    }
    let mut seen = HashSet::new();
    let mut memories = Vec::with_capacity(request.selected.len());
    for selected in &request.selected {
        if !seen.insert(&selected.memory_id) {
            return Err("selected contiene duplicados".into());
        }
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.memory_id == selected.memory_id)
            .ok_or_else(|| {
                format!(
                    "selected_id {} no pertenece a los candidatos autorizados",
                    selected.memory_id
                )
            })?;
        if candidate.revision != selected.revision {
            return Err(format!(
                "selected_id {} tiene una revisión obsoleta",
                selected.memory_id
            ));
        }
        if !presented_ids.contains(&selected.memory_id) {
            return Err(format!(
                "selected_id {} no se presentó al clasificador",
                selected.memory_id
            ));
        }
        memories.push(candidate.clone());
    }
    let seed_text = if memories.is_empty() {
        None
    } else {
        Some(
            memories
                .iter()
                .map(|item| {
                    let mut text =
                        format!("[{}@{}] {}", item.memory_id, item.revision, item.summary);
                    if let Some(conditions) = &item.conditions {
                        text.push_str(&format!("\n  Condiciones: {conditions}"));
                    }
                    if let Some(source) = &item.source {
                        text.push_str(&format!(
                            "\n  Fuente declarada: {} / {}",
                            source.kind, source.reference
                        ));
                        if let Some(locator) = &source.locator {
                            text.push_str(&format!(" / {locator}"));
                        }
                    }
                    text
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
    };
    Ok(SeedResponse {
        status: retrieval_status,
        method,
        classifier: request.classifier.clone(),
        semantic_coverage,
        memories,
        potential_equivalents: vec![],
        seed_text,
    })
}

pub(crate) fn valid_utc(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !(bytes.len() == 24
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'.'
        && bytes[23] == b'Z'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, byte)| matches!(i, 4 | 7 | 10 | 13 | 16 | 19 | 23) || byte.is_ascii_digit()))
    {
        return false;
    }
    let number = |start: usize, end: usize| value[start..end].parse::<u32>().unwrap_or(0);
    let year = number(0, 4);
    let month = number(5, 7);
    let day = number(8, 10);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    year > 0
        && day > 0
        && day <= days
        && number(11, 13) < 24
        && number(14, 16) < 60
        && number(17, 19) < 60
}

fn fts_expression(query: &str) -> String {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .map(|word| format!("\"{}\"", word.replace('"', "\"\"")))
        .collect();
    terms.join(" OR ")
}

fn keyword_expression(query: &str) -> String {
    // This fixed, label-independent list matches the Engram comparison's
    // development-only query normalization. The indexed memories are unchanged.
    const STOPWORDS: &[&str] = &[
        "a", "al", "algo", "como", "con", "cual", "cuando", "cuál", "cuáles", "cuándo", "cómo",
        "de", "del", "donde", "dónde", "el", "ella", "en", "es", "esta", "este", "esto", "fue",
        "ha", "hay", "la", "las", "le", "lo", "los", "mi", "mis", "no", "nos", "para", "por",
        "que", "quien", "quién", "qué", "se", "ser", "si", "son", "su", "sus", "te", "tiene", "un",
        "una", "unas", "unos", "y", "ya",
    ];
    let mut seen = HashSet::new();
    query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| word.chars().count() >= 3)
        .map(str::to_lowercase)
        .filter(|word| !STOPWORDS.contains(&word.as_str()) && seen.insert(word.clone()))
        .map(|word| {
            if word.chars().count() >= 7 {
                let prefix = word
                    .chars()
                    .take(word.chars().count() - 2)
                    .collect::<String>();
                format!("\"{word}\" OR \"{prefix}\"*")
            } else {
                format!("\"{word}\"")
            }
        })
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::{DecisionError, PartialDecisionUsage};
    use crate::model::{Candidate, ClassifierProvider, DecisionResult, SelectionPolicy};
    use std::cell::Cell;

    struct FixedClassifier {
        choice: Option<&'static str>,
        fail: bool,
        partial_usage: Option<PartialDecisionUsage>,
        calls: Cell<usize>,
        observed_policy: Cell<SelectionPolicy>,
    }

    impl Classifier for FixedClassifier {
        fn classify(
            &self,
            _query: &str,
            _scope_id: &str,
            _candidates: &[Candidate],
            _provider: ClassifierProvider,
            selection_policy: SelectionPolicy,
        ) -> Result<DecisionResult, DecisionError> {
            self.calls.set(self.calls.get() + 1);
            self.observed_policy.set(selection_policy);
            if self.fail {
                return Err(DecisionError {
                    diagnostic: None,
                    code: if self.partial_usage.is_some() {
                        "support_gate_transport_error"
                    } else {
                        "transport_error"
                    },
                    http_status: None,
                    partial_usage: self.partial_usage.clone(),
                });
            }
            Ok(DecisionResult {
                per_memory: None,
                consensus: selection_policy.consensus_candidate_limit().map(|limit| {
                    crate::model::ConsensusTrace {
                        transport: None,
                        classified_candidate_count: _candidates.len().min(limit),
                        votes: [
                            self.choice.map(str::to_owned),
                            self.choice.map(str::to_owned),
                        ],
                    }
                }),
                applicability: None,
                slot_evidence: None,
                selected_id: self.choice.map(str::to_owned),
                served_model: Some("synthetic-model".into()),
                input_tokens: Some(10),
                output_tokens: Some(0),
                reported_cost_usd: Some(0.001),
            })
        }
    }

    #[test]
    fn retrieval_gate_demotes_ambiguous_v2_attribution_from_any_classifier() {
        use crate::model::{Candidate, DecisionResult};
        use crate::slot_evidence::{
            CandidateEvidence, EvidenceRelation, IdentityAssessment, IdentityStatus, Proposition,
            QueryAnchor, QueryTarget, SlotEvidenceTrace,
        };

        let query = "Which city for employee Ana Ruiz E-17?";
        let quote = "Employee Ana Ruiz E-17 told Bob B-29 that the office was closed.";
        let candidate = Candidate {
            memory_id: "m1".into(),
            revision: 1,
            summary: quote.into(),
            conditions: None,
            source: None,
            state: "active".into(),
            bm25: 0.0,
        };
        let trace = SlotEvidenceTrace {
            version: "slot-evidence-v2".into(),
            query_anchors: vec![QueryAnchor {
                id: "person".into(),
                kind: "employee".into(),
                subject: Some("Ana Ruiz".into()),
                stable_id: "E-17".into(),
                quote: "employee Ana Ruiz E-17".into(),
            }],
            slots: vec![QueryTarget {
                id: "city".into(),
                dimension: "requested_slot".into(),
                quote: "Which city".into(),
            }],
            premises: vec![],
            candidates: vec![CandidateEvidence {
                memory_id: "m1".into(),
                revision: 1,
                identity: IdentityAssessment {
                    status: IdentityStatus::Match,
                    anchor_id: Some("person".into()),
                    stable_id_quote: Some("E-17".into()),
                    subject_quote: Some("Ana Ruiz E-17".into()),
                },
                propositions: vec![Proposition {
                    occurrence_id: "p1".into(),
                    quote: quote.into(),
                    subject_quote: "Ana Ruiz E-17".into(),
                    identity_anchor_id: Some("person".into()),
                    predicate_quote: String::new(),
                    value: None,
                    polarity_quote: None,
                    modality_quote: None,
                    event_quote: None,
                    time_quote: None,
                    conditions_quote: None,
                    relation: EvidenceRelation::Answers,
                    target_id: "city".into(),
                }],
            }],
        };
        let mut decision = DecisionResult {
            per_memory: None,
            consensus: None,
            applicability: None,
            slot_evidence: Some(trace),
            selected_id: None,
            served_model: Some("synthetic-model".into()),
            input_tokens: Some(10),
            output_tokens: Some(10),
            reported_cost_usd: Some(0.001),
        };

        assert_eq!(
            decision.slot_evidence.as_ref().unwrap().candidates[0].propositions[0].relation,
            EvidenceRelation::Answers
        );
        normalize_slot_evidence(query, std::slice::from_ref(&candidate), &mut decision);
        assert!(slot_evidence_valid(
            query,
            &[candidate.clone()],
            SelectionPolicy::SlotEvidenceV2,
            &decision,
        ));
        let trace = decision.slot_evidence.as_ref().unwrap();
        assert_eq!(
            trace.candidates[0].propositions[0].relation,
            EvidenceRelation::UnresolvedAttribution
        );
        assert!(trace.candidates[0].propositions[0].target_id.is_empty());
        assert!(trace.selected_ids(&[candidate], query).is_empty());

        // The same service normalization must also block a different named
        // subject inside a reported clause, even when that person has no ID.
        let report_query = "Which city does employee Ana Ruiz E-17 live in?";
        let report_quote = "Employee Ana Ruiz E-17 reported that Bob Ruiz lives in Quito.";
        let report_candidate = Candidate {
            memory_id: "m2".into(),
            revision: 1,
            summary: report_quote.into(),
            conditions: None,
            source: None,
            state: "active".into(),
            bm25: 0.0,
        };
        let mut report_trace = decision.slot_evidence.clone().unwrap();
        report_trace.query_anchors[0].quote = "employee Ana Ruiz E-17".into();
        report_trace.slots[0].quote = "Which city".into();
        report_trace.candidates[0].memory_id = "m2".into();
        report_trace.candidates[0].identity.subject_quote = Some("Ana Ruiz E-17".into());
        report_trace.candidates[0].propositions[0].quote = report_quote.into();
        report_trace.candidates[0].propositions[0].subject_quote = "Ana Ruiz E-17".into();
        report_trace.candidates[0].propositions[0].relation = EvidenceRelation::Answers;
        report_trace.candidates[0].propositions[0].target_id = "city".into();
        let mut report_decision = DecisionResult {
            per_memory: None,
            consensus: None,
            applicability: None,
            slot_evidence: Some(report_trace),
            selected_id: None,
            served_model: Some("synthetic-model".into()),
            input_tokens: Some(10),
            output_tokens: Some(10),
            reported_cost_usd: Some(0.001),
        };
        normalize_slot_evidence(
            report_query,
            std::slice::from_ref(&report_candidate),
            &mut report_decision,
        );
        assert!(slot_evidence_valid(
            report_query,
            std::slice::from_ref(&report_candidate),
            SelectionPolicy::SlotEvidenceV2,
            &report_decision,
        ));
        let report_trace = report_decision.slot_evidence.as_ref().unwrap();
        assert_eq!(
            report_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::UnresolvedAttribution
        );
        assert!(
            report_trace
                .selected_ids(&[report_candidate], report_query)
                .is_empty()
        );
    }

    #[test]
    fn recall_normalizes_foreign_subjects_and_temporal_claims_before_seed() {
        use crate::model::{
            DecisionResult, FixtureMemory, QueryMode, RecallRequest, SearchRequest,
        };
        use crate::slot_evidence::{
            CandidateEvidence, EvidenceRelation, IdentityAssessment, IdentityStatus, Proposition,
            QueryAnchor, QueryTarget, SlotEvidenceTrace,
        };

        struct ReportedFact;
        impl Classifier for ReportedFact {
            fn classify(
                &self,
                _query: &str,
                _scope_id: &str,
                candidates: &[Candidate],
                _provider: ClassifierProvider,
                _selection_policy: SelectionPolicy,
            ) -> Result<DecisionResult, DecisionError> {
                let candidate = &candidates[0];
                let quote = if candidate.summary.contains("earlier draft")
                    && candidate.summary.contains("final version")
                {
                    "Employee Ana Ruiz E-17's earlier draft lists Bogotá.".into()
                } else {
                    candidate.summary.clone()
                };
                Ok(DecisionResult {
                    per_memory: None,
                    consensus: None,
                    applicability: None,
                    slot_evidence: Some(SlotEvidenceTrace {
                        version: "slot-evidence-v2".into(),
                        query_anchors: vec![QueryAnchor {
                            id: "person".into(),
                            kind: "employee".into(),
                            subject: Some("Ana Ruiz".into()),
                            stable_id: "E-17".into(),
                            quote: "employee Ana Ruiz E-17".into(),
                        }],
                        slots: vec![QueryTarget {
                            id: "city".into(),
                            dimension: "requested_slot".into(),
                            quote: "Which city".into(),
                        }],
                        premises: vec![],
                        candidates: vec![CandidateEvidence {
                            memory_id: candidate.memory_id.clone(),
                            revision: candidate.revision,
                            identity: IdentityAssessment {
                                status: IdentityStatus::Match,
                                anchor_id: Some("person".into()),
                                stable_id_quote: Some("E-17".into()),
                                subject_quote: Some("Ana Ruiz E-17".into()),
                            },
                            propositions: vec![Proposition {
                                occurrence_id: "p1".into(),
                                quote,
                                subject_quote: "Ana Ruiz E-17".into(),
                                identity_anchor_id: Some("person".into()),
                                predicate_quote: "lives in".into(),
                                value: Some("Quito".into()),
                                polarity_quote: None,
                                modality_quote: None,
                                event_quote: None,
                                time_quote: None,
                                conditions_quote: None,
                                relation: EvidenceRelation::Answers,
                                target_id: "city".into(),
                            }],
                        }],
                    }),
                    selected_id: None,
                    served_model: Some("synthetic-model".into()),
                    input_tokens: Some(10),
                    output_tokens: Some(10),
                    reported_cost_usd: Some(0.001),
                })
            }
        }

        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        let cases = [
            (
                "c1",
                "Employee Ana Ruiz E-17 reported that Bob Ruiz lives in Quito.",
                false,
            ),
            (
                "c2",
                "Employee Ana Ruiz E-17 reported that Ana Ruiz's colleague Bob Torres lives in Quito.",
                false,
            ),
            (
                "c3",
                "Employee Ana Ruiz E-17 reported that the well-known Ana Ruiz lives in Quito.",
                true,
            ),
            (
                "c4",
                "Employee Ana Ruiz E-17's earlier draft lists Bogotá. The final version for employee Ana Ruiz E-17 lists Quito.",
                true,
            ),
            (
                "c5",
                "Employee Ana Ruiz E-17's earlier draft lists Bogotá.",
                true,
            ),
            (
                "c6",
                "Employee Ana Ruiz E-17's earlier draft lists Bogotá.",
                true,
            ),
            (
                "c7",
                "The final version for employee Ana Ruiz E-17 lists Quito.",
                true,
            ),
            (
                "c8",
                "For employee Ana Ruiz E-17, the current version lists the city as Quito.",
                true,
            ),
        ];
        crate::repo::load_fixtures(
            &mut conn,
            &cases
                .iter()
                .map(|(scope, summary, _)| FixtureMemory {
                    id: format!("reported-fact-{scope}"),
                    scope_kind: "conversation".into(),
                    scope_id: (*scope).into(),
                    entity_id: None,
                    revision: 1,
                    memory_type: "conversation".into(),
                    original_text: (*summary).into(),
                    summary: (*summary).into(),
                    conditions: None,
                    source: None,
                    state: "active".into(),
                    valid_from: None,
                    valid_until: None,
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let mut request = RecallRequest {
            search: SearchRequest {
                query: "Which city for employee Ana Ruiz E-17?".into(),
                scope_kind: "conversation".into(),
                scope_id: "c1".into(),
                entity_id: None,
                as_of: "2026-10-23T12:00:00.000Z".into(),
                limit: None,
                query_mode: Some(QueryMode::Natural),
            },
            provider: ClassifierProvider::Decisions,
            selection_policy: SelectionPolicy::SlotEvidenceV2,
        };
        let default_query = request.search.query.clone();
        for (scope, _, expected_selected) in cases {
            request.search.scope_id = scope.into();
            request.search.query = match scope {
                "c4" | "c5" => {
                    "Which city is in the final version for employee Ana Ruiz E-17?".into()
                }
                "c6" => "Which city was in the earlier draft for employee Ana Ruiz E-17?".into(),
                "c7" => "Which city is current for employee Ana Ruiz E-17?".into(),
                "c8" => "Which city is in the current version for employee Ana Ruiz E-17?".into(),
                _ => default_query.clone(),
            };
            let response = recall(&conn, &request, &ReportedFact).unwrap();
            assert_eq!(response.status, "complete");
            assert_eq!(
                !response.seed.memories.is_empty(),
                expected_selected,
                "unexpected selection for {scope}"
            );
            let relation = response.slot_evidence.unwrap().candidates[0].propositions[0].relation;
            let expected_relation = match scope {
                "c1" | "c2" => EvidenceRelation::UnresolvedAttribution,
                "c3" => EvidenceRelation::Answers,
                "c4" | "c5" | "c7" => EvidenceRelation::Partial,
                "c6" | "c8" => EvidenceRelation::Answers,
                _ => unreachable!("test fixture case"),
            };
            assert_eq!(
                relation, expected_relation,
                "unexpected relation for {scope}"
            );
        }
    }

    #[test]
    fn recall_preserves_explicit_premise_refutation_separately_from_requested_slots() {
        use crate::model::{FixtureMemory, QueryMode, RecallRequest, SearchRequest};
        use crate::slot_evidence::{
            CandidateEvidence, EvidenceRelation, IdentityAssessment, IdentityStatus, Proposition,
            QueryAnchor, QueryTarget, SlotEvidenceTrace,
        };

        struct CorrectedPremise;
        impl Classifier for CorrectedPremise {
            fn classify(
                &self,
                _query: &str,
                _scope_id: &str,
                candidates: &[Candidate],
                _provider: ClassifierProvider,
                _selection_policy: SelectionPolicy,
            ) -> Result<DecisionResult, DecisionError> {
                let candidate = &candidates[0];
                let quote = candidate.summary.clone();
                Ok(DecisionResult {
                    per_memory: None,
                    consensus: None,
                    applicability: None,
                    slot_evidence: Some(SlotEvidenceTrace {
                        version: "slot-evidence-v2".into(),
                        query_anchors: vec![QueryAnchor {
                            id: "person".into(),
                            kind: "employee".into(),
                            subject: Some("Ana Ruiz".into()),
                            stable_id: "E-17".into(),
                            quote: "employee Ana Ruiz E-17".into(),
                        }],
                        slots: vec![],
                        premises: vec![QueryTarget {
                            id: "premise-approval".into(),
                            dimension: "premise".into(),
                            quote: "employee Ana Ruiz E-17 approve item PR-77".into(),
                        }],
                        candidates: vec![CandidateEvidence {
                            memory_id: candidate.memory_id.clone(),
                            revision: candidate.revision,
                            identity: IdentityAssessment {
                                status: IdentityStatus::Match,
                                anchor_id: Some("person".into()),
                                stable_id_quote: Some("E-17".into()),
                                subject_quote: Some("Ana Ruiz E-17".into()),
                            },
                            propositions: vec![Proposition {
                                occurrence_id: "p1".into(),
                                quote,
                                subject_quote: "Ana Ruiz E-17".into(),
                                identity_anchor_id: Some("person".into()),
                                predicate_quote: String::new(),
                                value: None,
                                polarity_quote: None,
                                modality_quote: None,
                                event_quote: None,
                                time_quote: None,
                                conditions_quote: None,
                                relation: EvidenceRelation::Refutes,
                                target_id: "premise-approval".into(),
                            }],
                        }],
                    }),
                    selected_id: None,
                    served_model: Some("synthetic-model".into()),
                    input_tokens: Some(12),
                    output_tokens: Some(8),
                    reported_cost_usd: Some(0.001),
                })
            }
        }

        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        crate::repo::load_fixtures(
            &mut conn,
            &[FixtureMemory {
                id: "ana-approval".into(),
                scope_kind: "conversation".into(),
                scope_id: "premise-correction".into(),
                entity_id: None,
                revision: 1,
                memory_type: "conversation".into(),
                original_text: "Employee Ana Ruiz E-17 did not approve item PR-77.".into(),
                summary: "Employee Ana Ruiz E-17 did not approve item PR-77.".into(),
                conditions: None,
                source: None,
                state: "active".into(),
                valid_from: None,
                valid_until: None,
            }],
        )
        .unwrap();
        let request = RecallRequest {
            search: SearchRequest {
                query: "Did employee Ana Ruiz E-17 approve item PR-77?".into(),
                scope_kind: "conversation".into(),
                scope_id: "premise-correction".into(),
                entity_id: None,
                as_of: "2026-10-23T12:00:00.000Z".into(),
                limit: None,
                query_mode: Some(QueryMode::Natural),
            },
            provider: ClassifierProvider::Decisions,
            selection_policy: SelectionPolicy::SlotEvidenceV2,
        };

        let response = recall(&conn, &request, &CorrectedPremise).unwrap();

        assert_eq!(response.status, "complete");
        assert_eq!(response.seed.memories.len(), 1);
        assert_eq!(response.seed.memories[0].memory_id, "ana-approval");
        let trace = response.slot_evidence.unwrap();
        assert!(trace.slots.is_empty());
        assert_eq!(trace.premises.len(), 1);
        assert_eq!(trace.candidates[0].identity.status, IdentityStatus::Match);
        assert_eq!(
            trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Refutes
        );
        assert_eq!(
            trace.candidates[0].propositions[0].target_id,
            "premise-approval"
        );
    }

    #[test]
    fn per_memory_recall_composes_all_parts_and_rejects_incomplete_trace() {
        use crate::per_memory::{EvidenceRelation, MemoryAssessment, PerMemoryTrace};
        struct Multi {
            invalid: bool,
        }
        impl Classifier for Multi {
            fn classify(
                &self,
                _q: &str,
                _s: &str,
                candidates: &[Candidate],
                _p: ClassifierProvider,
                policy: SelectionPolicy,
            ) -> Result<DecisionResult, DecisionError> {
                assert!(policy.per_memory_candidate_limit().is_some());
                assert!(!candidates.is_empty());
                let mut trace = PerMemoryTrace {
                    choice_scores: None,
                    scoped_checks: None,
                    factor_checks: None,
                    request_sha256: None,
                    verification: None,
                    items: candidates
                        .iter()
                        .take(policy.per_memory_candidate_limit().unwrap())
                        .map(|c| MemoryAssessment {
                            memory_id: c.memory_id.clone(),
                            revision: c.revision,
                            relation: if matches!(
                                policy,
                                SelectionPolicy::PerMemoryBinaryV1
                                    | SelectionPolicy::PerMemoryBinaryV2
                                    | SelectionPolicy::PerMemoryBinaryV3
                                    | SelectionPolicy::PerMemoryBinaryV4
                                    | SelectionPolicy::PerMemoryBinaryV5
                                    | SelectionPolicy::PerMemoryExamplesV1
                                    | SelectionPolicy::PerMemoryFactorsV1
                                    | SelectionPolicy::PerMemoryScopedV1
                                    | SelectionPolicy::PerMemoryScopedV2
                                    | SelectionPolicy::PerMemoryScopedV3
                                    | SelectionPolicy::PerMemoryScopedAnchorV1
                                    | SelectionPolicy::PerMemoryScoredV1
                                    | SelectionPolicy::PerMemoryVerifiedV1
                                    | SelectionPolicy::PerMemoryCorrectionVerifiedV1
                                    | SelectionPolicy::PerMemoryCorrectionVerifiedV2
                                    | SelectionPolicy::PerMemoryCorrectionVerifiedV3
                                    | SelectionPolicy::PerMemoryCorrectionVerifiedV4
                                    | SelectionPolicy::PerMemoryCorrectionVerifiedV5
                                    | SelectionPolicy::PerMemoryRoleVerifiedV1
                                    | SelectionPolicy::PerMemoryRoleVerifiedV2
                                    | crate::model::SelectionPolicy::PerMemoryRoleBatchV1
                                    | crate::model::SelectionPolicy::PerMemoryCompletePremiseV1
                            ) {
                                EvidenceRelation::UsefulEvidence
                            } else {
                                EvidenceRelation::PendingOrPartial
                            },
                        })
                        .collect(),
                };
                if matches!(
                    policy,
                    SelectionPolicy::PerMemoryScopedV1
                        | SelectionPolicy::PerMemoryScopedV2
                        | SelectionPolicy::PerMemoryScopedV3
                        | SelectionPolicy::PerMemoryScopedAnchorV1
                ) {
                    trace.scoped_checks = Some(
                        trace
                            .items
                            .iter()
                            .map(|i| crate::per_memory::factors::ScopedEvidenceChecks {
                                literal_anchor: (policy
                                    == SelectionPolicy::PerMemoryScopedAnchorV1)
                                    .then(|| crate::per_memory::anchor::LiteralAnchor {
                                        query_token: None,
                                        present_in_source: None,
                                    }),
                                memory_id: i.memory_id.clone(),
                                revision: i.revision,
                                requested_fact_same_context: true,
                                premise_correction_same_context: false,
                            })
                            .collect(),
                    );
                }
                if policy == SelectionPolicy::PerMemoryFactorsV1 {
                    trace.factor_checks = Some(
                        trace
                            .items
                            .iter()
                            .map(|i| crate::per_memory::factors::MemoryFactorChecks {
                                memory_id: i.memory_id.clone(),
                                revision: i.revision,
                                same_context: true,
                                requested_fact: true,
                                premise_correction: false,
                            })
                            .collect(),
                    );
                }
                if policy == SelectionPolicy::PerMemoryScoredV1 {
                    trace.request_sha256 = Some("a".repeat(64));
                    trace.choice_scores = Some(
                        trace
                            .items
                            .iter()
                            .map(|item| crate::per_memory::MemoryChoiceScore {
                                memory_id: item.memory_id.clone(),
                                revision: item.revision,
                                a: 0.8,
                                none: 0.2,
                            })
                            .collect(),
                    );
                }
                let first = trace.selected_ids().first().map(|id| (*id).to_owned());
                if self.invalid {
                    trace.items.pop();
                }
                if matches!(
                    policy,
                    SelectionPolicy::PerMemoryVerifiedV1
                        | SelectionPolicy::PerMemoryCorrectionVerifiedV1
                        | SelectionPolicy::PerMemoryCorrectionVerifiedV2
                        | SelectionPolicy::PerMemoryCorrectionVerifiedV3
                        | SelectionPolicy::PerMemoryCorrectionVerifiedV4
                        | SelectionPolicy::PerMemoryCorrectionVerifiedV5
                        | SelectionPolicy::PerMemoryRoleVerifiedV1
                        | SelectionPolicy::PerMemoryRoleVerifiedV2
                        | crate::model::SelectionPolicy::PerMemoryRoleBatchV1
                        | crate::model::SelectionPolicy::PerMemoryCompletePremiseV1
                ) {
                    use crate::per_memory::verification::{
                        CheckedMemory, StageUsage, VERIFIER_MODEL, VerificationTrace,
                    };
                    let usage = |model: &str| StageUsage {
                        requested_model: model.into(),
                        served_model: Some(model.into()),
                        input_tokens: Some(10),
                        output_tokens: Some(0),
                        reported_cost_usd: Some(0.001),
                    };
                    trace.verification = Some(VerificationTrace {
                        batch: None,
                        kind_proposal: if policy.uses_kind_proposal() {
                            Some(trace.items.iter().map(|m|crate::per_memory::contextual::EvidenceKindProposal {memory_id:m.memory_id.clone(),revision:m.revision,evidence_kind:crate::per_memory::contextual::ContextualEvidenceKind::RequestedFactOrPartial}).collect())
                        } else {
                            None
                        },
                        role_proposal: if policy.correction_verification_version() == Some(5) {
                            Some(trace.items.iter().map(|m|crate::per_memory::contextual::ContextualRoleProposal {memory_id:m.memory_id.clone(),revision:m.revision,context_for_fact:true,evidence_kind:crate::per_memory::contextual::ContextualEvidenceKind::RequestedFactOrPartial}).collect())
                        } else {
                            None
                        },
                        scoped_proposal: if matches!(
                            policy.correction_verification_version(),
                            Some(1..=4)
                        ) {
                            Some(
                                trace
                                    .items
                                    .iter()
                                    .map(|i| crate::per_memory::factors::ScopedEvidenceChecks {
                                        literal_anchor: None,
                                        memory_id: i.memory_id.clone(),
                                        revision: i.revision,
                                        requested_fact_same_context: true,
                                        premise_correction_same_context: false,
                                    })
                                    .collect(),
                            )
                        } else {
                            None
                        },
                        proposal: trace.items.clone(),
                        proposal_usage: usage(_p.model_id()),
                        checks: trace
                            .items
                            .iter()
                            .filter(|_| policy.correction_verification_version().is_none())
                            .map(|m| CheckedMemory {
                                complete_paths: None,
                                paths: None,
                                memory_id: m.memory_id.clone(),
                                revision: m.revision,
                                useful: true,
                                usage: usage(VERIFIER_MODEL),
                            })
                            .collect(),
                    });
                }
                if policy == SelectionPolicy::PerMemoryRoleBatchV1 {
                    use crate::per_memory::verification::{BatchVerdict, BatchVerification};
                    let v = trace.verification.as_mut().unwrap();
                    let usage = v.checks[0].usage.clone();
                    let verdicts = v
                        .checks
                        .drain(..)
                        .map(|c| BatchVerdict {
                            memory_id: c.memory_id,
                            revision: c.revision,
                            useful: c.useful,
                            paths: c.paths,
                        })
                        .collect();
                    v.batch = Some(BatchVerification { usage, verdicts });
                }
                if policy == SelectionPolicy::PerMemoryCompletePremiseV1 {
                    use crate::per_memory::verification::{
                        CompleteEvidencePaths, PREMISE_VERIFIER_MODEL, PREMISE_VERIFIER_REVISION,
                    };
                    for c in &mut trace.verification.as_mut().unwrap().checks {
                        c.complete_paths = Some(CompleteEvidencePaths {
                            requested_fact_or_partial: true,
                            ordinary_correction: false,
                            necessary_premise_incompatible: false,
                            explicit_context_contrast: false,
                        });
                        c.usage.requested_model = PREMISE_VERIFIER_MODEL.into();
                        c.usage.served_model = Some(PREMISE_VERIFIER_REVISION.into());
                    }
                }
                let totals = trace.verification.as_ref().map(|v| v.totals().unwrap());
                Ok(DecisionResult {
                    served_model: totals
                        .as_ref()
                        .map_or_else(|| Some("fake".into()), |t| t.served_model.clone()),
                    input_tokens: totals.as_ref().map_or(Some(100), |t| t.input_tokens),
                    output_tokens: totals.as_ref().map_or(Some(0), |t| t.output_tokens),
                    reported_cost_usd: totals.as_ref().map_or(Some(0.001), |t| t.reported_cost_usd),
                    per_memory: Some(trace),
                    consensus: None,
                    applicability: None,
                    slot_evidence: None,
                    selected_id: first,
                })
            }
        }
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        for sql in [
            include_str!("../migrations/0001_initial.sql"),
            include_str!("../migrations/0003_entity_filter.sql"),
            include_str!("../migrations/0004_seed_provenance.sql"),
        ] {
            conn.execute_batch(sql).unwrap();
        }
        let make = |id: &str, text: &str| FixtureMemory {
            id: id.into(),
            scope_kind: "conversation".into(),
            scope_id: "parts".into(),
            entity_id: None,
            revision: 1,
            memory_type: "fact".into(),
            original_text: text.into(),
            summary: text.into(),
            conditions: None,
            source: None,
            state: "active".into(),
            valid_from: None,
            valid_until: None,
        };
        crate::repo::load_fixtures(
            &mut conn,
            &[
                make(
                    "humidity",
                    "Aurora resiste humedad dos días; aplastamiento pendiente.",
                ),
                make("seal", "Aurora tiene sellado hermético."),
            ],
        )
        .unwrap();
        let mut request:RecallRequest=serde_json::from_str(r#"{"search":{"query":"Aurora","scope_kind":"conversation","scope_id":"parts","as_of":"2026-10-19T12:00:00.000Z","limit":20},"selection_policy":"per_memory_v1"}"#).unwrap();
        let response = recall(&conn, &request, &Multi { invalid: false }).unwrap();
        assert_eq!(response.status, "complete");
        assert_eq!(response.seed.memories.len(), 2);
        assert_eq!(
            response.selected_ids,
            response
                .seed
                .memories
                .iter()
                .map(|m| m.memory_id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(response.selected_id.as_ref(), response.selected_ids.first());
        let text = response.seed.seed_text.unwrap();
        assert!(text.contains("aplastamiento pendiente"));
        assert!(text.contains("sellado hermético"));
        let extras: Vec<_> = (2..16)
            .map(|i| {
                make(
                    &format!("part-{i}"),
                    &format!("Aurora parte adicional {i}."),
                )
            })
            .collect();
        crate::repo::load_fixtures(&mut conn, &extras).unwrap();
        for limit in [9, 16] {
            request.search.limit = Some(limit);
            let many = recall(&conn, &request, &Multi { invalid: false }).unwrap();
            assert_eq!(many.seed.memories.len(), limit as usize);
            assert_eq!(many.selected_ids.len(), limit as usize);
        }
        let many = recall(&conn, &request, &Multi { invalid: false }).unwrap();
        let presented = many
            .retrieval
            .candidates
            .iter()
            .map(|c| SelectedCandidate {
                memory_id: c.memory_id.clone(),
                revision: c.revision,
            })
            .collect();
        let selected = many.retrieval.candidates[..9]
            .iter()
            .map(|c| SelectedCandidate {
                memory_id: c.memory_id.clone(),
                revision: c.revision,
            })
            .collect();
        let legacy = SeedRequest {
            search: Some(request.search.clone()),
            hybrid: None,
            presented,
            selected,
            classifier: "manual".into(),
            classification_status: "complete".into(),
        };
        assert!(compose_seed(&conn, &legacy).unwrap_err().contains("8"));
        for policy in [
            SelectionPolicy::PerMemoryV2,
            SelectionPolicy::PerMemoryV3,
            SelectionPolicy::PerMemoryV4,
            SelectionPolicy::PerMemoryBinaryV1,
            SelectionPolicy::PerMemoryBinaryV2,
            SelectionPolicy::PerMemoryBinaryV3,
            SelectionPolicy::PerMemoryBinaryV4,
            SelectionPolicy::PerMemoryBinaryV5,
            SelectionPolicy::PerMemoryExamplesV1,
            SelectionPolicy::PerMemoryFactorsV1,
            SelectionPolicy::PerMemoryScopedV1,
            SelectionPolicy::PerMemoryScopedV2,
            SelectionPolicy::PerMemoryScopedV3,
            SelectionPolicy::PerMemoryScopedAnchorV1,
            SelectionPolicy::PerMemoryScoredV1,
            SelectionPolicy::PerMemoryVerifiedV1,
            SelectionPolicy::PerMemoryCorrectionVerifiedV1,
            SelectionPolicy::PerMemoryCorrectionVerifiedV2,
            SelectionPolicy::PerMemoryCorrectionVerifiedV3,
            SelectionPolicy::PerMemoryCorrectionVerifiedV4,
            SelectionPolicy::PerMemoryCorrectionVerifiedV5,
            SelectionPolicy::PerMemoryRoleVerifiedV1,
            SelectionPolicy::PerMemoryRoleVerifiedV2,
            SelectionPolicy::PerMemoryRoleBatchV1,
            SelectionPolicy::PerMemoryCompletePremiseV1,
        ] {
            request.selection_policy = policy;
            let compact = recall(&conn, &request, &Multi { invalid: false }).unwrap();
            assert_eq!(compact.status, "complete");
            assert_eq!(
                compact.seed.memories.len(),
                policy.per_memory_candidate_limit().unwrap()
            );
            assert_eq!(
                compact.per_memory.unwrap().items.len(),
                policy.per_memory_candidate_limit().unwrap()
            );
        }
        request.selection_policy = SelectionPolicy::PerMemoryV1;
        let invalid = recall(&conn, &request, &Multi { invalid: true }).unwrap();
        assert_eq!(invalid.status, "degraded");
        assert_eq!(invalid.error_code, Some("invalid_per_memory"));
        assert!(invalid.seed.memories.is_empty());
        assert!(invalid.selected_ids.is_empty());
        assert_eq!(invalid.reported_cost_usd, Some(0.001));
        request.search.query = "zzyyxxnotpresent".into();
        let empty = recall(&conn, &request, &Multi { invalid: false }).unwrap();
        assert_eq!(empty.classification_status, "skipped_empty");
        assert!(empty.per_memory.is_none());
    }

    #[test]
    fn recall_chooses_a_scoped_memory_and_degrades_on_provider_failure() {
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        let memory = |id: &str, scope: &str| FixtureMemory {
            id: id.into(),
            scope_kind: "user".into(),
            scope_id: scope.into(),
            entity_id: None,
            revision: 1,
            memory_type: "preference".into(),
            original_text: "Prefiere formato breve".into(),
            summary: "Prefiere formato breve".into(),
            conditions: None,
            source: None,
            state: "active".into(),
            valid_from: None,
            valid_until: None,
        };
        crate::repo::load_fixtures(&mut conn, &[memory("wanted", "u1"), memory("other", "u2")])
            .unwrap();
        let mut request = RecallRequest {
            search: SearchRequest {
                query: "¿Qué formato prefiero?".into(),
                scope_kind: "user".into(),
                scope_id: "u1".into(),
                entity_id: None,
                as_of: "2026-10-15T12:00:00.000Z".into(),
                limit: None,
                query_mode: None,
            },
            provider: ClassifierProvider::Decisions,
            selection_policy: SelectionPolicy::Baseline,
        };
        let good = FixedClassifier {
            choice: Some("wanted"),
            fail: false,
            partial_usage: None,
            calls: Cell::new(0),
            observed_policy: Cell::new(SelectionPolicy::Baseline),
        };
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.status, "complete");
        assert_eq!(response.retrieval.method, "fts5_keywords");
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert_eq!(response.input_tokens, Some(10));
        assert_eq!(good.calls.get(), 1);
        assert_eq!(good.observed_policy.get(), SelectionPolicy::Baseline);

        request.selection_policy = SelectionPolicy::WitnessV1;
        let unproven = recall(&conn, &request, &good).unwrap();
        assert_eq!(unproven.status, "degraded");
        assert_eq!(unproven.error_code, Some("invalid_applicability"));
        assert!(unproven.seed.memories.is_empty());
        assert_eq!(unproven.input_tokens, Some(10));

        assert_eq!(
            serde_json::from_str::<SelectionPolicy>("\"roles_fallback_v1\"").unwrap(),
            SelectionPolicy::RolesFallbackV1
        );
        request.selection_policy = SelectionPolicy::RolesFallbackV1;
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert_eq!(good.observed_policy.get(), SelectionPolicy::RolesFallbackV1);

        request.selection_policy = SelectionPolicy::RolesConsensusV1;
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        let mut candidates = vec![response.retrieval.candidates[0].clone(); 20];
        for (i, c) in candidates.iter_mut().enumerate() {
            c.memory_id = format!("candidate-{i}");
        }
        let mut proof = crate::model::DecisionResult {
            per_memory: None,
            consensus: Some(crate::model::ConsensusTrace {
                transport: None,
                classified_candidate_count: 16,
                votes: [Some("candidate-19".into()), Some("candidate-19".into())],
            }),
            applicability: None,
            slot_evidence: None,
            selected_id: Some("candidate-19".into()),
            served_model: None,
            input_tokens: None,
            output_tokens: None,
            reported_cost_usd: None,
        };
        assert!(!consensus_valid(
            &candidates,
            request.selection_policy,
            &proof
        ));
        proof.consensus.as_mut().unwrap().votes =
            [Some("candidate-0".into()), Some("candidate-0".into())];
        proof.selected_id = Some("candidate-0".into());
        assert!(consensus_valid(
            &candidates,
            request.selection_policy,
            &proof
        ));
        proof.consensus.as_mut().unwrap().votes[1] = None;
        assert!(!consensus_valid(
            &candidates,
            request.selection_policy,
            &proof
        ));
        proof.selected_id = None;
        assert!(consensus_valid(
            &candidates,
            request.selection_policy,
            &proof
        ));
        proof.consensus.as_mut().unwrap().classified_candidate_count = 20;
        proof.consensus.as_mut().unwrap().votes =
            [Some("candidate-19".into()), Some("candidate-19".into())];
        proof.selected_id = Some("candidate-19".into());
        assert!(consensus_valid(
            &candidates,
            SelectionPolicy::RolesConsensusSharedV1,
            &proof
        ));
        assert!(!consensus_valid(
            &candidates,
            SelectionPolicy::RolesConsensusV1,
            &proof
        ));
        assert_eq!(
            good.observed_policy.get(),
            SelectionPolicy::RolesConsensusV1
        );

        proof.consensus.as_mut().unwrap().classified_candidate_count = 4;
        proof.consensus.as_mut().unwrap().votes =
            [Some("candidate-4".into()), Some("candidate-4".into())];
        proof.selected_id = Some("candidate-4".into());
        assert!(!consensus_valid(
            &candidates,
            SelectionPolicy::RolesConsensusCompactV1,
            &proof
        ));
        proof.consensus.as_mut().unwrap().votes =
            [Some("candidate-3".into()), Some("candidate-3".into())];
        proof.selected_id = Some("candidate-3".into());
        assert!(consensus_valid(
            &candidates,
            SelectionPolicy::RolesConsensusCompactV1,
            &proof
        ));
        request.selection_policy = serde_json::from_str::<RecallRequest>(
            r#"{"search":{"query":"wanted","scope_kind":"conversation","scope_id":"session","as_of":"2026-10-19T12:00:00.000Z","limit":20},"selection_policy":"roles_consensus_compact_v1"}"#
        ).unwrap().selection_policy;
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert_eq!(
            good.observed_policy.get(),
            SelectionPolicy::RolesConsensusCompactV1
        );

        request.selection_policy =
            serde_json::from_str::<SelectionPolicy>(r#""roles_consensus_mirrored_v1""#).unwrap();
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert_eq!(
            good.observed_policy.get(),
            SelectionPolicy::RolesConsensusMirroredV1
        );

        let mut asymmetric = proof;
        asymmetric
            .consensus
            .as_mut()
            .unwrap()
            .classified_candidate_count = 16;
        asymmetric.consensus.as_mut().unwrap().votes = [Some("candidate-3".into()), None];
        asymmetric.selected_id = Some("candidate-3".into());
        assert!(consensus_valid(
            &candidates,
            SelectionPolicy::RolesPrimaryVetoV1,
            &asymmetric
        ));
        assert!(!consensus_valid(
            &candidates,
            SelectionPolicy::RolesConsensusMirroredV1,
            &asymmetric
        ));
        asymmetric.consensus.as_mut().unwrap().votes = [Some("candidate-19".into()), None];
        asymmetric.selected_id = Some("candidate-19".into());
        assert!(!consensus_valid(
            &candidates,
            SelectionPolicy::RolesPrimaryVetoV1,
            &asymmetric
        ));
        request.selection_policy =
            serde_json::from_str::<SelectionPolicy>(r#""roles_primary_veto_v1""#).unwrap();
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert_eq!(
            good.observed_policy.get(),
            SelectionPolicy::RolesPrimaryVetoV1
        );

        request.selection_policy = SelectionPolicy::RolesConsensusBatchV1;
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert!(response.consensus.is_some());

        request.selection_policy = SelectionPolicy::RolesConsensusSharedV1;
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");

        request.selection_policy = SelectionPolicy::RolesV2;
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert_eq!(good.observed_policy.get(), SelectionPolicy::RolesV2);

        request.selection_policy = SelectionPolicy::RolesV1;
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert_eq!(good.observed_policy.get(), SelectionPolicy::RolesV1);

        request.selection_policy = SelectionPolicy::UtilityV2;
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert_eq!(good.observed_policy.get(), SelectionPolicy::UtilityV2);

        request.selection_policy = SelectionPolicy::UtilityV1;
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert_eq!(good.observed_policy.get(), SelectionPolicy::UtilityV1);

        request.selection_policy = SelectionPolicy::EvidenceV1;
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.seed.memories[0].memory_id, "wanted");
        assert_eq!(good.observed_policy.get(), SelectionPolicy::EvidenceV1);

        let failed = FixedClassifier {
            choice: None,
            fail: true,
            partial_usage: None,
            calls: Cell::new(0),
            observed_policy: Cell::new(SelectionPolicy::Baseline),
        };
        let response = recall(&conn, &request, &failed).unwrap();
        assert_eq!(response.status, "degraded");
        assert_eq!(response.error_code, Some("transport_error"));
        assert!(response.seed.seed_text.is_none());
        assert_eq!(response.retrieval.candidates.len(), 1);
        assert_eq!(response.reported_cost_usd, None);

        request.selection_policy = SelectionPolicy::SupportGateV1;
        let partial_failed = FixedClassifier {
            partial_usage: Some(PartialDecisionUsage {
                served_model: Some("synthetic-model".into()),
                input_tokens: Some(10),
                output_tokens: Some(0),
                reported_cost_usd: Some(0.001),
            }),
            ..failed
        };
        let response = recall(&conn, &request, &partial_failed).unwrap();
        assert_eq!(response.status, "degraded");
        assert_eq!(response.classification_status, "failed");
        assert_eq!(response.error_code, Some("support_gate_transport_error"));
        assert_eq!(response.served_model.as_deref(), Some("synthetic-model"));
        assert_eq!(response.input_tokens, Some(10));
        assert_eq!(response.output_tokens, Some(0));
        assert_eq!(response.reported_cost_usd, Some(0.001));
        assert!(response.selected_id.is_none());
        assert!(response.seed.memories.is_empty());
        assert!(response.seed.seed_text.is_none());

        let calls_before_empty_search = good.calls.get();
        request.search.query = "palabra_sin_coincidencia".into();
        let response = recall(&conn, &request, &good).unwrap();
        assert_eq!(response.classification_status, "skipped_empty");
        assert_eq!(good.calls.get(), calls_before_empty_search);
    }
    use crate::model::FixtureMemory;

    #[test]
    fn keyword_search_removes_question_words_but_keeps_scoped_memory() {
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        let fixture = |id: &str, scope: &str| FixtureMemory {
            id: id.into(),
            scope_kind: "user".into(),
            scope_id: scope.into(),
            entity_id: None,
            revision: 1,
            memory_type: "fact".into(),
            original_text: "El nombre completo es Clara Luz".into(),
            summary: "Nombre completo Clara Luz".into(),
            conditions: None,
            source: None,
            state: "active".into(),
            valid_from: None,
            valid_until: None,
        };
        let mut invitation = fixture("invitation", "u1");
        invitation.original_text = "Invitaciones de calendario con agenda previa".into();
        invitation.summary = "Quiere invitaciones de calendario con agenda previa".into();
        crate::repo::load_fixtures(
            &mut conn,
            &[fixture("wanted", "u1"), fixture("other", "u2"), invitation],
        )
        .unwrap();
        let request = SearchRequest {
            query: "¿Cuál es mi nombre completo?".into(),
            scope_kind: "user".into(),
            scope_id: "u1".into(),
            entity_id: None,
            as_of: "2026-10-15T12:00:00.000Z".into(),
            limit: Some(20),
            query_mode: Some(QueryMode::Keywords),
        };
        let response = search(&conn, &request).unwrap();
        assert_eq!(response.method, "fts5_keywords");
        assert_eq!(response.candidates.len(), 1);
        assert_eq!(response.candidates[0].memory_id, "wanted");
        let default_request: SearchRequest = serde_json::from_str(
            r#"{"query":"¿Cuál es mi nombre completo?","scope_kind":"user","scope_id":"u1","as_of":"2026-10-15T12:00:00.000Z","limit":20}"#,
        )
        .unwrap();
        assert_eq!(
            search(&conn, &default_request).unwrap().method,
            "fts5_keywords"
        );
        let natural_request: SearchRequest = serde_json::from_str(
            r#"{"query":"¿Cuál es mi nombre completo?","scope_kind":"user","scope_id":"u1","as_of":"2026-10-15T12:00:00.000Z","limit":20,"query_mode":"natural"}"#,
        )
        .unwrap();
        assert_eq!(search(&conn, &natural_request).unwrap().method, "fts5");
        let mut singular_request = request;
        singular_request.query = "¿Qué incluir en la invitación?".into();
        let expanded = search(&conn, &singular_request).unwrap();
        assert_eq!(expanded.candidates.len(), 1);
        assert_eq!(expanded.candidates[0].memory_id, "invitation");
        singular_request.query_mode = Some(QueryMode::Natural);
        assert!(
            search(&conn, &singular_request)
                .unwrap()
                .candidates
                .is_empty()
        );
        assert_eq!(
            keyword_expression("¿Cuál es mi nombre completo?"),
            "\"nombre\" OR \"completo\" OR \"comple\"*"
        );
        assert!(keyword_expression("¿Y si no?").is_empty());
    }

    #[test]
    fn search_filters_scope_and_old_revision() {
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        let fixtures = vec![
            FixtureMemory {
                id: "m1".into(),
                scope_kind: "user".into(),
                scope_id: "u1".into(),
                entity_id: None,
                revision: 1,
                memory_type: "preference".into(),
                original_text: "Prefiero Fer".into(),
                summary: "Llamar Fer".into(),
                conditions: None,
                source: None,
                state: "corrected".into(),
                valid_from: None,
                valid_until: None,
            },
            FixtureMemory {
                id: "m1".into(),
                scope_kind: "user".into(),
                scope_id: "u1".into(),
                entity_id: None,
                revision: 2,
                memory_type: "preference".into(),
                original_text: "Prefiero Fernando".into(),
                summary: "Llamar Fernando".into(),
                conditions: None,
                source: None,
                state: "active".into(),
                valid_from: None,
                valid_until: None,
            },
            FixtureMemory {
                id: "m2".into(),
                scope_kind: "user".into(),
                scope_id: "u2".into(),
                entity_id: None,
                revision: 1,
                memory_type: "preference".into(),
                original_text: "Prefiero Fernando".into(),
                summary: "Llamar Fernando".into(),
                conditions: None,
                source: None,
                state: "active".into(),
                valid_from: None,
                valid_until: None,
            },
        ];
        crate::repo::load_fixtures(&mut conn, &fixtures).unwrap();
        let response = search(
            &conn,
            &SearchRequest {
                query: "Fernando".into(),
                scope_kind: "user".into(),
                scope_id: "u1".into(),
                entity_id: None,
                as_of: "2026-10-15T12:00:00.000Z".into(),
                limit: None,
                query_mode: Some(QueryMode::Natural),
            },
        )
        .unwrap();
        assert_eq!(response.candidates.len(), 1);
        assert_eq!(response.candidates[0].memory_id, "m1");
        assert_eq!(response.candidates[0].revision, 2);
    }

    #[test]
    fn fts_input_is_not_an_operator() {
        assert_eq!(fts_expression("Fer OR x*"), "\"Fer\" OR \"OR\" OR \"x\"");
    }

    #[test]
    fn invalid_calendar_day_is_rejected() {
        assert!(!valid_utc("2026-02-30T12:00:00.000Z"));
        assert!(valid_utc("2024-02-29T12:00:00.000Z"));
    }

    #[test]
    fn seed_rechecks_selection_against_authorized_candidates() {
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        crate::repo::load_fixtures(
            &mut conn,
            &[
                FixtureMemory {
                    id: "allowed".into(),
                    scope_kind: "project".into(),
                    scope_id: "a".into(),
                    entity_id: None,
                    revision: 1,
                    memory_type: "fact".into(),
                    original_text: "Formato breve".into(),
                    summary: "Usar formato breve".into(),
                    conditions: None,
                    source: None,
                    state: "active".into(),
                    valid_from: None,
                    valid_until: None,
                },
                FixtureMemory {
                    id: "foreign".into(),
                    scope_kind: "project".into(),
                    scope_id: "b".into(),
                    entity_id: None,
                    revision: 1,
                    memory_type: "fact".into(),
                    original_text: "Formato extenso".into(),
                    summary: "Usar formato extenso".into(),
                    conditions: None,
                    source: None,
                    state: "active".into(),
                    valid_from: None,
                    valid_until: None,
                },
            ],
        )
        .unwrap();
        let mut request = SeedRequest {
            search: Some(SearchRequest {
                query: "formato".into(),
                scope_kind: "project".into(),
                scope_id: "a".into(),
                entity_id: None,
                as_of: "2026-10-15T12:00:00.000Z".into(),
                limit: Some(8),
                query_mode: Some(QueryMode::Natural),
            }),
            hybrid: None,
            presented: vec![crate::model::SelectedCandidate {
                memory_id: "allowed".into(),
                revision: 1,
            }],
            selected: vec![crate::model::SelectedCandidate {
                memory_id: "foreign".into(),
                revision: 1,
            }],
            classifier: "synthetic-test".into(),
            classification_status: "complete".into(),
        };
        assert!(
            compose_seed(&conn, &request)
                .unwrap_err()
                .contains("autorizados")
        );
        request.selected = vec![crate::model::SelectedCandidate {
            memory_id: "allowed".into(),
            revision: 0,
        }];
        assert!(
            compose_seed(&conn, &request)
                .unwrap_err()
                .contains("obsoleta")
        );
        request.selected[0].revision = 1;
        let response = compose_seed(&conn, &request).unwrap();
        assert_eq!(response.memories.len(), 1);
        assert_eq!(
            response.seed_text.as_deref(),
            Some("[allowed@1] Usar formato breve")
        );
        request.presented.clear();
        assert!(
            compose_seed(&conn, &request)
                .unwrap_err()
                .contains("no se presentó")
        );
    }

    #[test]
    fn seed_preserves_conditions_and_declared_source() {
        use crate::model::{FixtureMemory, SelectedCandidate, SourceRef};
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::configure(&conn).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0003_entity_filter.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/0004_seed_provenance.sql"))
            .unwrap();
        let source = SourceRef {
            kind: "document".into(),
            reference: "synthetic:manual@v2".into(),
            locator: Some("section-4".into()),
        };
        let fixture = FixtureMemory {
            id: "conditional".into(),
            scope_kind: "project".into(),
            scope_id: "synthetic".into(),
            entity_id: None,
            revision: 1,
            memory_type: "procedure".into(),
            original_text: "Si el cliente pide urgencia, usar canal rojo.".into(),
            summary: "Si el cliente pide urgencia, usar canal rojo.".into(),
            conditions: Some("Solo si el cliente pide urgencia".into()),
            source: Some(source.clone()),
            state: "active".into(),
            valid_from: None,
            valid_until: None,
        };
        crate::repo::load_fixtures(&mut conn, &[fixture.clone()]).unwrap();
        let search_request = SearchRequest {
            query: "urgencia".into(),
            scope_kind: "project".into(),
            scope_id: "synthetic".into(),
            entity_id: None,
            as_of: "2026-10-15T12:00:00.000Z".into(),
            limit: Some(8),
            query_mode: Some(QueryMode::Natural),
        };
        let candidates = search(&conn, &search_request).unwrap().candidates;
        assert_eq!(candidates[0].conditions, fixture.conditions);
        assert_eq!(candidates[0].source, Some(source.clone()));
        let selected = SelectedCandidate {
            memory_id: "conditional".into(),
            revision: 1,
        };
        let response = compose_seed(
            &conn,
            &SeedRequest {
                search: Some(search_request),
                hybrid: None,
                presented: vec![SelectedCandidate {
                    memory_id: selected.memory_id.clone(),
                    revision: selected.revision,
                }],
                selected: vec![selected],
                classifier: "synthetic-test".into(),
                classification_status: "complete".into(),
            },
        )
        .unwrap();
        assert_eq!(response.memories[0].conditions, fixture.conditions);
        assert_eq!(response.memories[0].source, Some(source));
        let text = response.seed_text.unwrap();
        assert!(text.contains("Condiciones: Solo si el cliente pide urgencia"));
        assert!(text.contains("Fuente declarada: document / synthetic:manual@v2 / section-4"));

        let mut invalid = fixture;
        invalid.id = "invalid".into();
        invalid.source.as_mut().unwrap().kind = "unknown".into();
        assert!(crate::repo::load_fixtures(&mut conn, &[invalid]).is_err());
    }
}
