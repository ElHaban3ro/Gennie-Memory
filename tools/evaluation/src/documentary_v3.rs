//! Pure, evidence-bound documentary V3 validation and pair classification.
use serde::{Serialize, Serializer};
use serde_json::Value;
mod attribute_profiles;
mod configuration_profile;
pub(crate) mod context_budget;
mod descriptions;
pub(crate) mod lexical_clause_context;
pub(crate) mod linguistic_composition;
pub(crate) mod nominal_de;
pub(crate) mod nominal_event_context;
pub(crate) mod nominal_possessive;
pub(crate) mod normative_indeterminacy;
mod pair;
mod quantified_match;
pub(crate) mod request_context_profile;
mod validation;
pub use pair::{classify_pair, classify_pair_context};

/// Only validation can construct this result. Serialization preserves the public report shape.
pub struct DocumentResult {
    report: Value,
    vocabulary_sha256: String,
}
impl Serialize for DocumentResult {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.report.serialize(serializer)
    }
}
pub fn validate(raw: &Value, text: &str, vocabulary: &Value) -> DocumentResult {
    if !request_context_profile::enabled(vocabulary) {
        return validation::validate(raw, text, vocabulary);
    }
    context_budget::with_operation(vocabulary, raw, || validation::validate(raw, text, vocabulary))
        .unwrap_or_else(|_| {
            use sha2::{Digest, Sha256};
            let hash = Sha256::digest(serde_json::to_vec(vocabulary).unwrap_or_default())
                .iter().map(|b| format!("{b:02x}")).collect();
            // Error provenance is a bounded copy of the received raw, not an executable document.
            // Very deep/large Value callers retain their original input and receive its text hash.
            let mut stack = vec![(raw, 0usize)];
            let mut nodes = 0usize;
            let mut bounded = true;
            while let Some((value, depth)) = stack.pop() {
                nodes += 1;
                if nodes > 100_000 || depth > 64 { bounded = false; break; }
                match value {
                    Value::Array(values) => stack.extend(values.iter().map(|v| (v, depth + 1))),
                    Value::Object(values) => stack.extend(values.values().map(|v| (v, depth + 1))),
                    _ => {}
                }
            }
            bounded = bounded && context_budget::serialized_bytes(raw).is_some_and(|n| n <= 16 * 1024 * 1024);
            let mut report = serde_json::json!({"status":"invalid","document":null,"candidates":[],"issues":[{"id":"issue-1","code":"request_context_limit","path":"","evidence_refs":[]}],"raw_candidate":null,"received_document_sha256":null});
            if bounded {
                struct HashWriter(Sha256);
                impl std::io::Write for HashWriter {
                    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> { self.0.update(bytes); Ok(bytes.len()) }
                    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
                }
                let mut writer = HashWriter(Sha256::new());
                if serde_json::to_writer(&mut writer, raw).is_ok() {
                    report["received_document_sha256"] = Value::String(writer.0.finalize().iter().map(|b|format!("{b:02x}")).collect());
                    report["raw_candidate"] = raw.clone();
                }
            } else {
                report["raw_candidate_unavailable_reason"] = Value::String("input_limit".into());
                report["received_text_sha256"] = Value::String(Sha256::digest(text.as_bytes()).iter().map(|b|format!("{b:02x}")).collect());
            }
            DocumentResult { report, vocabulary_sha256: hash }
        })
}

/// Explicit requested granularity; a year slot never derives a calendar projection.
fn temporal_slot(slot: &str) -> Option<(&str, bool)> {
    match slot {
        "holds_at" | "scheduled_at" | "planned_for" | "occurred_at" => Some((slot, false)),
        "holds_year" => Some(("holds_at", true)),
        "scheduled_year" => Some(("scheduled_at", true)),
        "planned_year" => Some(("planned_for", true)),
        "occurred_year" => Some(("occurred_at", true)),
        _ => None,
    }
}

#[cfg(test)]
mod nominal_de_tests;
#[cfg(test)]
mod nominal_possessive_tests;

#[cfg(test)]
mod normative_indeterminacy_tests;

#[cfg(test)]
mod linguistic_composition_tests;
#[cfg(test)]
mod nominal_event_context_tests;
