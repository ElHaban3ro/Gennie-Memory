//! Exact vocabulary-backed type proof for a context-only relevance guard.
use super::{Engine, Relation, id, rows, s};
use serde_json::{Value, json};
impl Engine<'_> {
    pub(super) fn context_type_grounding_applicable(
        &self,
        c: &Value,
        r: &Value,
        p: &Value,
    ) -> bool {
        self.context_v2
            && s(p, "judgment") == "fact"
            && s(p, "status") == "state"
            && s(&p["atom"], "polarity") == "positive"
            && s(&r["target"], "judgment") == "fact"
            && s(&r["target"], "status") == "state"
            && s(&r["target"]["atom"], "polarity") == "positive"
            && matches!(s(p, "basis"), "description" | "assignment" | "observation")
            && (s(&r["basis_requirement"], "kind") == "any"
                || s(&r["basis_requirement"], "kind") == "exact"
                    && r["basis_requirement"]["value"] == p["basis"])
            && self.scope_match(r, p) == Relation::Same
            && self.binders(self.source, s(c, "scope_id")).is_empty()
            && self.binders(self.question, s(r, "scope_id")).is_empty()
            && self.branch(self.source, s(c, "scope_id")).is_empty()
            && self.branch(self.question, s(r, "scope_id")).is_empty()
    }
    pub(super) fn context_type_grounding(&self, term: &Value, filter: &Value) -> Option<Value> {
        if !self.context_v2 || s(term, "kind") != "ref" || s(filter, "kind") != "semantic" {
            return None;
        }
        let reference = self.reference(self.source, s(term, "id"))?;
        if s(reference, "denotation") != "entity"
            || !reference["semantic_type"].is_null()
            || !reference["anaphora"].is_null()
            || self.local(self.source, id(reference), id(reference)) == Relation::Conflict
        {
            return None;
        }
        // Do not select the convenient null-typed alias of a component that
        // already carries another explicit type. This asserts no type
        // exclusivity; it simply withholds this new lexical proof.
        if rows(self.source, "references").iter().any(|alias| {
            self.local(self.source, id(reference), id(alias)) == Relation::Same
                && !alias["semantic_type"].is_null()
                && alias["semantic_type"]["type_id"] != filter["type_id"]
        }) {
            return None;
        }
        let class = &reference["declared_class"];
        let quote = s(class, "quote");
        if quote.is_empty()
            || class["start_byte"]
                .as_u64()
                .zip(class["end_byte"].as_u64())
                .is_none_or(|(start, end)| end <= start || end - start != quote.len() as u64)
        {
            return None;
        }
        let matches: Vec<_> = rows(self.vocabulary, "semantic_type_rows")
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                rows(row, "class_quotes")
                    .iter()
                    .any(|value| value.as_str() == Some(quote))
            })
            .collect();
        if matches.len() != 1 {
            return None;
        }
        let (index, row) = matches[0];
        if row["id"] != filter["type_id"] {
            return None;
        }
        Some(
            json!({"kind":"exact_vocabulary_class_quote","source_reference_id":reference["id"],
            "requested_type_id":filter["type_id"],"declared_class_evidence":class,
            "vocabulary_row_index":index,"vocabulary_row":row,
            "source_semantic_type":reference["semantic_type"],"identity_proven":false,
            "answer_value_inferred":false}),
        )
    }
}
#[cfg(test)]
#[path = "context_type_grounding_tests.rs"]
mod tests;
