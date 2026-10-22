//! Offline structural/consistency checker for annotated per-slot evidence.
//! It does not extract evidence or establish semantic truth.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("missing_{key}"))
}

fn strings(v: &Value, key: &str) -> Result<BTreeSet<String>, String> {
    let values = v[key].as_array().ok_or_else(|| format!("missing_{key}"))?;
    let mut result = BTreeSet::new();
    for value in values {
        let value = value
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| format!("invalid_{key}"))?;
        if !result.insert(value.to_owned()) {
            return Err(format!("duplicate_{key}"));
        }
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Observation {
    memory_id: String,
    slot_id: String,
    status: String,
    value: Option<String>,
    fact_ids: Vec<String>,
}

fn validate_case(case: &Value) -> Result<(), String> {
    let case_id = string(case, "id")?;
    let query = &case["query"];
    let query_scope = string(query, "scope")?;
    let slots = strings(query, "slots")?;
    let premises = strings(query, "premises")?;
    let anchors = strings(query, "identity_anchors")?;
    if slots.is_empty() {
        return Err(format!("{case_id}:empty_slots"));
    }

    let candidates = case["candidate_set"]
        .as_array()
        .filter(|rows| !rows.is_empty())
        .ok_or_else(|| format!("{case_id}:empty_candidate_set"))?;
    let mut memories = BTreeMap::new();
    for memory in candidates {
        let id = string(memory, "memory_id")?.to_owned();
        if memories.insert(id.clone(), memory).is_some() {
            return Err(format!("{case_id}:duplicate_memory_id:{id}"));
        }
        string(memory, "text")?;
        let coverage = memory["source_coverage"]
            .as_array()
            .ok_or_else(|| format!("{case_id}:missing_source_coverage:{id}"))?;
        let mut coverage_keys = BTreeSet::new();
        for row in coverage {
            let slot = string(row, "slot_id")?;
            if !slots.contains(slot) {
                return Err(format!("{case_id}:coverage_unknown_slot:{id}:{slot}"));
            }
            let scope = string(row, "scope")?;
            let status = string(row, "status")?;
            if !["complete", "incomplete", "unknown"].contains(&status) {
                return Err(format!("{case_id}:invalid_coverage_status:{id}:{status}"));
            }
            if !coverage_keys.insert((slot.to_owned(), scope.to_owned())) {
                return Err(format!("{case_id}:duplicate_coverage:{id}:{slot}:{scope}"));
            }
        }
    }

    let facts = case["facts"]
        .as_array()
        .ok_or_else(|| format!("{case_id}:missing_facts"))?;
    let mut fact_ids = BTreeSet::new();
    let mut facts_by_memory_slot: BTreeMap<(String, String), Vec<&Value>> = BTreeMap::new();
    let mut premise_observations = BTreeSet::new();
    for fact in facts {
        let fact_id = string(fact, "fact_id")?;
        if !fact_ids.insert(fact_id.to_owned()) {
            return Err(format!("{case_id}:duplicate_fact_id:{fact_id}"));
        }
        let memory_id = string(fact, "memory_id")?.to_owned();
        let memory = memories
            .get(&memory_id)
            .ok_or_else(|| format!("{case_id}:unknown_fact_memory:{memory_id}"))?;
        let quote = string(fact, "source_quote")?;
        let source = string(memory, "text")?;
        if !source.contains(quote) {
            return Err(format!("{case_id}:ungrounded_quote:{fact_id}"));
        }
        let subject = string(fact, "subject_ref")?;
        let identity = string(fact, "identity_link")?;
        if !["match", "mismatch", "unresolved"].contains(&identity) {
            return Err(format!("{case_id}:invalid_identity_link:{fact_id}"));
        }
        if identity == "match" && !anchors.contains(subject) {
            return Err(format!("{case_id}:undeclared_subject_anchor:{fact_id}"));
        }
        let scope = string(fact, "scope_status")?;
        if !["compatible", "different", "unresolved"].contains(&scope) {
            return Err(format!("{case_id}:invalid_scope_status:{fact_id}"));
        }

        if !fact["slot_id"].is_null() {
            let slot_id = string(fact, "slot_id")?.to_owned();
            if !slots.contains(slot_id.as_str()) {
                return Err(format!("{case_id}:undeclared_slot:{fact_id}"));
            }
            let status = string(fact, "slot_status")?;
            if ![
                "answers",
                "partial",
                "explicit_unmeasured",
                "not_applicable",
            ]
            .contains(&status)
            {
                return Err(format!("{case_id}:invalid_slot_status:{fact_id}"));
            }
            if ["answers", "partial", "explicit_unmeasured"].contains(&status)
                && (identity == "mismatch" || scope == "different")
            {
                return Err(format!("{case_id}:unsupported_slot_link:{fact_id}"));
            }
            if status == "explicit_unmeasured"
                && !fact["explicit_state_quote"]
                    .as_str()
                    .is_some_and(|q| !q.is_empty() && source.contains(q) && quote.contains(q))
            {
                return Err(format!(
                    "{case_id}:unmeasured_without_grounded_assertion:{fact_id}"
                ));
            }
            if status == "answers" && fact["value"].as_str().is_none() {
                return Err(format!("{case_id}:answer_without_value:{fact_id}"));
            }
            facts_by_memory_slot
                .entry((memory_id.clone(), slot_id))
                .or_default()
                .push(fact);
        }

        if !fact["premise_id"].is_null() {
            let premise_id = string(fact, "premise_id")?;
            if !premises.contains(premise_id) {
                return Err(format!("{case_id}:undeclared_premise:{fact_id}"));
            }
            let relation = string(fact, "premise_relation")?;
            if !["supports", "refutes", "unrelated"].contains(&relation) {
                return Err(format!("{case_id}:invalid_premise_relation:{fact_id}"));
            }
            if ["supports", "refutes"].contains(&relation)
                && (identity != "match" || scope != "compatible")
            {
                return Err(format!("{case_id}:unsupported_premise_link:{fact_id}"));
            }
            premise_observations.insert((premise_id.to_owned(), relation.to_owned()));
        }
    }

    let mut actual_observations = Vec::new();
    for (memory_id, memory) in &memories {
        for slot_id in &slots {
            let facts_for_slot = facts_by_memory_slot
                .get(&(memory_id.clone(), slot_id.clone()))
                .cloned()
                .unwrap_or_default();
            let eligible: Vec<_> = facts_for_slot
                .iter()
                .copied()
                .filter(|f| f["identity_link"] == "match" && f["scope_status"] == "compatible")
                .collect();
            let unresolved: Vec<_> = facts_for_slot
                .iter()
                .copied()
                .filter(|f| f["identity_link"] == "unresolved")
                .collect();
            let selected_statuses: Vec<_> = eligible
                .iter()
                .filter_map(|f| f["slot_status"].as_str())
                .filter(|s| ["answers", "partial", "explicit_unmeasured"].contains(s))
                .collect();
            let (status, value, ids) = if !selected_statuses.is_empty() {
                let status = if selected_statuses.contains(&"answers") {
                    "answers"
                } else if selected_statuses.contains(&"partial") {
                    "partial"
                } else {
                    "explicit_unmeasured"
                };
                let value = eligible
                    .iter()
                    .find(|f| f["slot_status"] == status && f["value"].is_string())
                    .and_then(|f| f["value"].as_str())
                    .map(str::to_owned);
                let ids = eligible
                    .iter()
                    .filter(|f| f["slot_status"] == status)
                    .filter_map(|f| f["fact_id"].as_str().map(str::to_owned))
                    .collect();
                (status.to_owned(), value, ids)
            } else if !unresolved.is_empty() {
                (
                    "ambiguous".to_owned(),
                    None,
                    unresolved
                        .iter()
                        .filter_map(|f| f["fact_id"].as_str().map(str::to_owned))
                        .collect(),
                )
            } else {
                let coverage_status = memory["source_coverage"]
                    .as_array()
                    .and_then(|rows| {
                        rows.iter()
                            .find(|r| r["slot_id"] == slot_id.as_str() && r["scope"] == query_scope)
                    })
                    .and_then(|r| r["status"].as_str())
                    .unwrap_or("unknown");
                let status = if coverage_status == "complete" {
                    "not_observed_in_this_memory"
                } else {
                    "unknown"
                };
                (status.to_owned(), None, Vec::new())
            };
            actual_observations.push(Observation {
                memory_id: memory_id.clone(),
                slot_id: slot_id.clone(),
                status,
                value,
                fact_ids: ids,
            });
        }
    }

    let expected = case["expected"]
        .as_object()
        .ok_or_else(|| format!("{case_id}:missing_expected"))?;
    let expected_rows = expected["observations"]
        .as_array()
        .ok_or_else(|| format!("{case_id}:missing_expected_observations"))?;
    let mut expected_observations = Vec::new();
    let mut expected_keys = BTreeSet::new();
    for row in expected_rows {
        let memory_id = string(row, "memory_id")?.to_owned();
        let slot_id = string(row, "slot_id")?.to_owned();
        if !memories.contains_key(&memory_id) || !slots.contains(&slot_id) {
            return Err(format!("{case_id}:expected_reference_not_in_case"));
        }
        if !expected_keys.insert((memory_id.clone(), slot_id.clone())) {
            return Err(format!("{case_id}:duplicate_expected_observation"));
        }
        expected_observations.push(Observation {
            memory_id,
            slot_id,
            status: string(row, "status")?.to_owned(),
            value: row["value"].as_str().map(str::to_owned),
            fact_ids: row["fact_ids"]
                .as_array()
                .ok_or_else(|| format!("{case_id}:missing_expected_fact_ids"))?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| format!("{case_id}:invalid_expected_fact_id"))
                })
                .collect::<Result<_, _>>()?,
        });
    }
    actual_observations.sort_by(|a, b| (&a.memory_id, &a.slot_id).cmp(&(&b.memory_id, &b.slot_id)));
    expected_observations
        .sort_by(|a, b| (&a.memory_id, &a.slot_id).cmp(&(&b.memory_id, &b.slot_id)));
    if actual_observations != expected_observations {
        return Err(format!("{case_id}:frozen_slot_expectations_mismatch"));
    }

    let expected_premises = expected["premise_relations"]
        .as_array()
        .ok_or_else(|| format!("{case_id}:missing_expected_premise_relations"))?;
    let mut expected_premise_set = BTreeSet::new();
    for row in expected_premises {
        let premise_id = string(row, "premise_id")?;
        let relation = string(row, "relation")?;
        if !premises.contains(premise_id) {
            return Err(format!("{case_id}:expected_unknown_premise"));
        }
        expected_premise_set.insert((premise_id.to_owned(), relation.to_owned()));
    }
    if premise_observations != expected_premise_set {
        return Err(format!("{case_id}:frozen_premise_expectations_mismatch"));
    }

    // Aggregate across requested slots, not across memory-slot rows: a complete
    // memory with no observation must not erase explicit evidence from another.
    let slot_statuses: Vec<_> = slots
        .iter()
        .map(|slot| {
            let rows: Vec<_> = actual_observations
                .iter()
                .filter(|o| &o.slot_id == slot)
                .collect();
            if rows.iter().any(|o| o.status == "ambiguous") {
                "ambiguous"
            } else if rows
                .iter()
                .any(|o| ["answers", "partial", "explicit_unmeasured"].contains(&o.status.as_str()))
            {
                "answered"
            } else if rows.iter().any(|o| o.status == "unknown") {
                "unknown"
            } else {
                "unanswered"
            }
        })
        .collect();
    let actual_aggregate = if slot_statuses.contains(&"ambiguous") {
        "ambiguous"
    } else if slot_statuses.iter().all(|s| *s == "answered") {
        "complete"
    } else if slot_statuses.contains(&"answered") {
        "partial"
    } else if slot_statuses.contains(&"unknown") {
        "unknown"
    } else {
        "unanswered"
    };
    if expected["aggregate"].as_str() != Some(actual_aggregate) {
        return Err(format!("{case_id}:aggregate_mismatch"));
    }
    Ok(())
}

pub fn validate(bank: &Value) -> Result<usize, String> {
    if bank["version"] != "slot-alignment-contract-v1" || bank["synthetic"] != true {
        return Err("invalid_slot_contract_bank".into());
    }
    if bank["annotation_only"] != true {
        return Err("slot_contract_requires_annotation_only".into());
    }
    let cases = bank["cases"]
        .as_array()
        .filter(|rows| rows.len() == 12)
        .ok_or("slot_contract_requires_12_frozen_cases")?;
    let mut ids = BTreeSet::new();
    for case in cases {
        let id = string(case, "id")?;
        if !ids.insert(id.to_owned()) {
            return Err("duplicate_case_id".into());
        }
        validate_case(case)?;
    }
    Ok(cases.len())
}

pub fn run(input: &Path, output: &Path) -> Result<(), String> {
    let bytes = std::fs::read(input).map_err(|e| e.to_string())?;
    let bank: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let cases = validate(&bank)?;
    let report = json!({
        "version": "slot-alignment-contract-report-v1",
        "input_sha256": crate::campaign::digest(input)?,
        "cases": cases,
        "passed": cases,
        "failed": 0,
        "annotation_only": true,
        "semantic_truth_verified": false,
        "automatic_extraction_tested": false,
        "retrieval_quality_measured": false,
        "runtime_behavior_measured": false,
        "remote_calls": 0,
        "reported_remote_cost_usd": 0.0
    });
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    file.write_all(
        (serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n").as_bytes(),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_development_matrix_is_structurally_consistent() {
        let bank: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-alignment-v1/development.json"
        ))
        .unwrap();
        assert_eq!(validate(&bank).unwrap(), 12);
    }

    #[test]
    fn rejects_silence_without_complete_source_coverage() {
        let mut bank: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-alignment-v1/development.json"
        ))
        .unwrap();
        bank["cases"][0]["candidate_set"][0]["source_coverage"][0]["status"] = json!("incomplete");
        assert!(
            validate(&bank)
                .unwrap_err()
                .contains("frozen_slot_expectations_mismatch")
        );
    }

    #[test]
    fn rejects_quotes_that_do_not_belong_to_their_memory() {
        let mut bank: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-alignment-v1/development.json"
        ))
        .unwrap();
        bank["cases"][1]["facts"][0]["source_quote"] = json!("not in M02");
        assert!(validate(&bank).unwrap_err().contains("ungrounded_quote"));
    }

    #[test]
    fn rejects_unmeasured_states_without_explicit_grounded_quote() {
        let mut bank: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-alignment-v1/development.json"
        ))
        .unwrap();
        bank["cases"][9]["facts"][1]["explicit_state_quote"] = json!("the mean was measured");
        assert!(
            validate(&bank)
                .unwrap_err()
                .contains("unmeasured_without_grounded_assertion")
        );
    }

    #[test]
    fn identity_context_does_not_satisfy_a_requested_slot() {
        let mut bank: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-alignment-v1/development.json"
        ))
        .unwrap();
        bank["cases"][0]["expected"]["aggregate"] = json!("complete");
        assert!(validate(&bank).unwrap_err().contains("aggregate_mismatch"));
    }
}
