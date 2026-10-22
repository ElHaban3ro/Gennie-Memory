//! Numeric assignment context for a property-shaped request, without answering it.
use super::*;

impl Engine<'_> {
    pub(super) fn partial_assignment_context(&self, c: &Value, r: &Value, out: &mut Evidence) {
        let p = &c["body"]["value"];
        let target = &r["target"]["atom"];
        if s(&c["body"], "kind") != "proposition"
            || s(r, "mode") != "answer"
            || s(r, "slot") != "value"
            || s(target, "predicate") != "property"
            || s(target, "polarity") != "positive"
            || argument(target, "value").is_some()
            || s(&p["atom"], "predicate") != "uses"
            || !self.branch(self.source, s(c, "scope_id")).is_empty()
            || !self.binders(self.source, s(c, "scope_id")).is_empty()
            || !self.branch(self.question, s(r, "scope_id")).is_empty()
            || !self.binders(self.question, s(r, "scope_id")).is_empty()
            || !self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
            || !self
                .dependency_missing(self.question, "request", id(r))
                .is_empty()
        {
            return;
        }
        let (Some(actual), Some(wanted), Some(attribute)) = (
            argument(&p["atom"], "subject"),
            argument(target, "subject"),
            argument(target, "attribute"),
        ) else {
            return;
        };
        if s(actual, "kind") != "ref"
            || s(wanted, "kind") != "ref"
            || s(attribute, "kind") != "symbol"
            || self
                .description_anchor_evidence(s(actual, "id"), s(wanted, "id"))
                .is_none()
            || rows(self.source_report, "candidates")
                .iter()
                .any(|candidate| {
                    s(candidate, "state") != "eligible"
                        && rows(candidate, "claim_ids").contains(&c["id"])
                })
        {
            return;
        }
        let mut view = r.clone();
        view["slot"] = json!("amount");
        view["target"]["atom"]["predicate"] = json!("quantity");
        // This asks only whether the literal source assignment is relevant.
        // The original premises remain debt; this is no uses/operation entailment.
        view["premise_ids"] = json!([]);
        let mut projected = Evidence::default();
        self.scoped_assignment(c, &view, &mut projected);
        for mut witness in projected.support.into_iter().chain(projected.partial) {
            if witness["assignment_context"]["labels_match"] != true {
                continue;
            }
            let mut missing = strings(rows(&witness, "missing_constraints"));
            missing.extend(self.premises(r));
            missing.extend([
                "answer_slot".into(),
                "property_quantity_context_only".into(),
            ]);
            if !rows(r, "premise_ids").is_empty() {
                missing.push("unproven_property_premise".into());
            }
            if s(&r["basis_requirement"], "kind") == "exact"
                && r["basis_requirement"]["value"] != p["basis"]
            {
                missing.push("basis".into());
            }
            unique(&mut missing);
            witness["kind"] = json!("partial_assignment_context_v2");
            witness["id"] = json!(format!("{}:{}:partial_assignment_context_v2", id(r), id(c)));
            witness["answer_terms"] = json!([]);
            witness["answer_propositions"] = json!([]);
            witness["source_proposition"] = p.clone();
            witness["source_qualifiers"] = json!(
                self.claim_qualifiers(c, p)
                    .iter()
                    .filter_map(|qid| self.qualifier(self.source, qid.as_str()?))
                    .collect::<Vec<_>>()
            );
            witness["request_target"] = r["target"].clone();
            witness["request_premise_ids"] = r["premise_ids"].clone();
            witness["missing_constraints"] = json!(missing);
            witness["context_transport"] = json!({"version":"assignment_property_context_v2","requested_value_inferred":false,"property_quantity_equivalence_asserted":false,"premises_discharged":false});
            out.partial(witness);
            out.unresolved.push(self.candidate(c, r, &missing));
        }
    }
}
#[cfg(test)]
#[path = "partial_assignment_context_tests.rs"]
mod tests;
