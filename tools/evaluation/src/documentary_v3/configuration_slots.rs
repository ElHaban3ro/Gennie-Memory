//! Explicit same-proposition configuration values; no property or event inference.
use super::{Engine, Evidence, Relation, id, rows, s, unique};
use serde_json::{Value, json};
impl Engine<'_> {
    pub(super) fn configuration_slot(&self, r: &Value, out: &mut Evidence) {
        let key = s(r, "slot");
        if s(r, "mode") != "configuration_slot"
            || !super::super::configuration_profile::permits(self.vocabulary, key)
            || s(self.scope, "namespace").is_empty()
            || self.source["domain_namespace"] != self.scope["namespace"]
            || self.question["domain_namespace"] != self.scope["namespace"]
        {
            return;
        }
        let wanted_key = |doc: &Value, qid: &Value| {
            self.qualifier(doc, qid.as_str().unwrap_or(""))
                .is_some_and(|q| s(q, "kind") == "configuration" && s(&q["body"], "key") == key)
        };
        // Requested-key restrictions directly on a question are rejected by
        // validation. Inherited conditions, including that key, remain intact.
        let target_other = self.request_qualifiers(r);
        let mut ready: Vec<(&Value, &Value, Value, Vec<Value>)> = vec![];
        // Earlier profile/dependency candidates already prevent this request
        // from publishing values. Retain them without exposing answer terms.
        let mut blocked = !out.unresolved.is_empty() || !out.conflicts.is_empty();
        let mut observed_values: Vec<Value> = vec![];
        for claim in rows(self.source, "claims") {
            if s(&claim["body"], "kind") != "proposition" {
                continue;
            }
            let proposition = &claim["body"]["value"];
            if s(proposition, "judgment") != "fact"
                || s(&proposition["atom"], "polarity") != "positive"
                || proposition["status"] != r["target"]["status"]
                || s(&r["basis_requirement"], "kind") == "exact"
                    && r["basis_requirement"]["value"] != proposition["basis"]
            {
                continue;
            }
            let relation = self.atom(
                self.source,
                &proposition["atom"],
                self.question,
                &r["target"]["atom"],
                None,
            );
            if relation == Relation::Different || !self.required_named_targets_apply(proposition, r)
            {
                continue;
            }
            let qualifiers = self.claim_qualifiers(claim, proposition);
            let mut direct_ids = rows(claim, "qualifier_ids").to_vec();
            direct_ids.extend(rows(proposition, "qualifier_ids").iter().cloned());
            direct_ids.sort_by_key(Value::to_string);
            direct_ids.dedup();
            let others: Vec<_> = qualifiers
                .iter()
                .filter(|id| !direct_ids.contains(id) || !wanted_key(self.source, id))
                .cloned()
                .collect();
            let (context, mut missing) =
                self.context(self.source, &others, self.question, &target_other, true);
            if context == Relation::Different
                || self.scope_match(r, proposition) == Relation::Different
            {
                continue;
            }
            if relation != Relation::Same || !self.required_associations_apply(proposition, r) {
                missing.push("identity".into());
            }
            if self.scope_match(r, proposition) != Relation::Same {
                missing.push("normative_target".into());
            }
            if !self
                .dependency_missing(self.source, "claim", id(claim))
                .is_empty()
                || !self
                    .dependency_missing(self.question, "request", id(r))
                    .is_empty()
            {
                missing.push("dependency".into());
            }
            if !self.branch(self.source, s(claim, "scope_id")).is_empty()
                || !self.branch(self.question, s(r, "scope_id")).is_empty()
                || !self.binders(self.source, s(claim, "scope_id")).is_empty()
                || !self.binders(self.question, s(r, "scope_id")).is_empty()
            {
                missing.push("branch_choice".into());
            }
            missing.extend(self.premises(r));
            let values: Vec<_> = direct_ids
                .iter()
                .filter_map(|qid| self.qualifier(self.source, qid.as_str()?))
                .filter(|q| s(q, "kind") == "configuration" && s(&q["body"], "key") == key)
                .collect();
            if values.is_empty() {
                missing.push("answer_slot".into());
            }
            if values.iter().any(|q| s(q, "resolution") != "known") {
                missing.push("configuration_value".into());
            }
            if context != Relation::Same {
                missing.push("configuration".into());
            }
            // Uniqueness precedes answer filters. A conflicting unit/type must
            // not disappear merely because another value satisfies the query.
            if relation == Relation::Same
                && context == Relation::Same
                && self.scope_match(r, proposition) == Relation::Same
                && (s(&r["basis_requirement"], "kind") != "exact"
                    || r["basis_requirement"]["value"] == proposition["basis"])
            {
                observed_values.extend(
                    values
                        .iter()
                        .filter(|q| s(q, "resolution") == "known")
                        .map(|q| q["body"]["value"].clone()),
                );
            }
            for q in &values {
                let value = &q["body"]["value"];
                missing.extend(self.filters(r, proposition, std::slice::from_ref(value)));
                if s(value, "kind") == "quantity" && s(value, "bound") != "exact" {
                    missing.push("quantity_bound".into());
                }
            }
            unique(&mut missing);
            if !missing.is_empty() {
                let mut candidate = self.candidate(claim, r, &missing);
                candidate["source_proposition"] = proposition.clone();
                candidate["source_qualifiers"] = json!(values);
                out.unresolved.push(candidate);
                blocked = true;
                continue;
            }
            for q in values {
                ready.push((
                    claim,
                    proposition,
                    q["body"]["value"].clone(),
                    vec![q.clone()],
                ));
            }
        }
        // Negative assertions do not donate values. A compatible explicit
        // negation of the same operation/value must nevertheless prevent a
        // positive answer from closing merely because other modes are skipped.
        for claim in rows(self.source, "claims") {
            if ready.is_empty() || s(&claim["body"], "kind") != "proposition" {
                continue;
            }
            let proposition = &claim["body"]["value"];
            if s(proposition, "judgment") != "fact"
                || s(&proposition["atom"], "polarity") != "negative"
                || proposition["status"] != r["target"]["status"]
                || !self.required_named_targets_apply(proposition, r)
                || !self.branch(self.source, s(claim, "scope_id")).is_empty()
                || !self.binders(self.source, s(claim, "scope_id")).is_empty()
                || s(&r["basis_requirement"], "kind") == "exact"
                    && r["basis_requirement"]["value"] != proposition["basis"]
            {
                continue;
            }
            let mut applicability = proposition["atom"].clone();
            applicability["polarity"] = json!("positive");
            if self.atom(
                self.source,
                &applicability,
                self.question,
                &r["target"]["atom"],
                None,
            ) != Relation::Same
                || self.scope_match(r, proposition) != Relation::Same
            {
                continue;
            }
            let mut direct = rows(claim, "qualifier_ids").to_vec();
            direct.extend(rows(proposition, "qualifier_ids").iter().cloned());
            direct.sort_by_key(Value::to_string);
            direct.dedup();
            let all = self.claim_qualifiers(claim, proposition);
            let others: Vec<_> = all
                .iter()
                .filter(|qid| !direct.contains(qid) || !wanted_key(self.source, qid))
                .cloned()
                .collect();
            let (negative_context, _) =
                self.context(self.source, &others, self.question, &target_other, true);
            let negative_debt = !self
                .dependency_missing(self.source, "claim", id(claim))
                .is_empty();
            if negative_context == Relation::Different
                || negative_context != Relation::Same && !negative_debt
            {
                continue;
            }
            let values: Vec<_> = direct
                .iter()
                .filter_map(|qid| self.qualifier(self.source, qid.as_str()?))
                .filter(|q| s(q, "kind") == "configuration" && s(&q["body"], "key") == key)
                .collect();
            // An unresolved condition may still apply. Preserve its debt
            // rather than treating uncertainty as proof of inapplicability.
            let pending = negative_context != Relation::Same
                || values.iter().any(|q| s(q, "resolution") != "known")
                || negative_debt;
            let incompatible = values.is_empty()
                || pending
                || values.iter().any(|q| {
                    observed_values.iter().any(|value| {
                        self.term(self.source, &q["body"]["value"], self.source, value)
                            == Relation::Same
                    })
                });
            if !incompatible {
                continue;
            }
            blocked = true;
            let reason = if pending {
                "configuration_negative_pending"
            } else {
                "configuration_polarity_conflict"
            };
            let mut candidate = self.candidate(claim, r, &[reason.into()]);
            candidate["source_proposition"] = proposition.clone();
            candidate["source_qualifiers"] = json!(values);
            out.unresolved.push(candidate);
            if !pending {
                out.conflicts.push(json!({"kind":"potential","support_witness_ids":[],"refutation_witness_ids":[],"reason":reason}));
            }
        }
        let conflicting = observed_values.iter().enumerate().any(|(index, a)| {
            observed_values
                .iter()
                .skip(index + 1)
                .any(|b| self.term(self.source, a, self.source, b) != Relation::Same)
        });
        if conflicting {
            blocked = true;
            out.conflicts.push(json!({"kind":"potential","support_witness_ids":[],"refutation_witness_ids":[],"reason":"configuration_value_conflict"}));
        }
        for (claim, proposition, value, qualifiers) in ready {
            if blocked {
                let mut candidate = self.candidate(
                    claim,
                    r,
                    &[if conflicting {
                        "configuration_value_conflict".into()
                    } else {
                        "configuration_value_pending".into()
                    }],
                );
                candidate["source_proposition"] = proposition.clone();
                candidate["source_qualifiers"] = json!(qualifiers);
                out.unresolved.push(candidate);
                continue;
            }
            let mut witness = self.witness(
                claim,
                r,
                proposition,
                "configuration_slot",
                vec![value],
                vec![],
                &[],
            );
            witness["source_proposition"] = proposition.clone();
            witness["source_qualifiers"] = json!(qualifiers);
            witness["configuration_key"] = json!(key);
            out.publish(witness, false);
        }
    }
}
#[cfg(test)]
#[path = "configuration_slots_tests.rs"]
mod tests;
