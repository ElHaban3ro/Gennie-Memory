//! Context-only event correspondence. Pattern proof never establishes event identity.
use super::{Engine, Evidence, Relation, id, rows, s};
use serde_json::{Value, json};
use std::collections::BTreeMap;

impl Engine<'_> {
    // Unlike the global alpha matcher, a binder's anonymous class domain may be
    // compared structurally here. Its variable must still have a grounded path,
    // checked by reference_pattern_matches below. This map proves no individuals.
    fn event_context_map(&self, c: &Value, r: &Value) -> Option<(BTreeMap<String, String>, bool)> {
        let left = self.binders(self.source, s(c, "scope_id"));
        let right = self.binders(self.question, s(r, "scope_id"));
        if left.len() != right.len() || left.len() > 8 {
            return None;
        }
        let mut map = BTreeMap::new();
        let mut normalized_carrier = false;
        for (a, b) in left.iter().zip(&right) {
            let carrier_difference = a["domain_ref"].is_null() != b["domain_ref"].is_null()
                || (a["cardinality"]["domain"] == a["variable"])
                    != (b["cardinality"]["domain"] == b["variable"]);
            if carrier_difference {
                // The grammar permits cardinality over either the explicit
                // class or its bound group. Compare only their counting class;
                // this neither identifies class/group nor discharges identity.
                if left.len() != 1
                    || s(a, "quantifier") != "exists"
                    || s(b, "quantifier") != "exists"
                    || s(&a["cardinality"], "relation") != "exact"
                    || s(&b["cardinality"], "relation") != "exact"
                    || a["cardinality"]["value"].as_u64().is_none_or(|v| v == 0)
                {
                    return None;
                }
                for (doc, binding) in [(self.source, *a), (self.question, *b)] {
                    let variable = self.reference(doc, s(binding, "variable"))?;
                    if s(variable, "denotation") != "group"
                        || !variable["name"].is_null()
                        || s(&variable["declared_class"], "quote").is_empty()
                    {
                        return None;
                    }
                    if !binding["domain_ref"].is_null() {
                        let domain = self.reference(doc, s(binding, "domain_ref"))?;
                        if s(domain, "denotation") != "class"
                            || !domain["name"].is_null()
                            || !domain["anaphora"].is_null()
                            || !domain["binding_scope_id"].is_null()
                            || !rows(domain, "links").is_empty()
                            || domain["declared_class"]["quote"]
                                != variable["declared_class"]["quote"]
                            || domain["semantic_type"]["type_id"]
                                != variable["semantic_type"]["type_id"]
                            || rows(doc, "identity").iter().any(|edge| {
                                edge["left"] == domain["id"] || edge["right"] == domain["id"]
                            })
                            || rows(doc, "dependencies").iter().any(|dep| {
                                s(&dep["consumer"], "kind") == "reference"
                                    && dep["consumer"]["id"] == domain["id"]
                                    && s(&dep["requires"], "kind") != "scope"
                            })
                        {
                            return None;
                        }
                    }
                }
                normalized_carrier = true;
            }
            if a["quantifier"] != b["quantifier"]
                || a["witness_relation"] != b["witness_relation"]
                || !rows(a, "relation_closures").is_empty()
                || !rows(b, "relation_closures").is_empty()
                || a["cardinality"].is_null() != b["cardinality"].is_null()
            {
                return None;
            }
            if !a["cardinality"].is_null()
                && (a["cardinality"]["relation"] != b["cardinality"]["relation"]
                    || a["cardinality"]["value"] != b["cardinality"]["value"])
            {
                return None;
            }
            for (ar, br) in [
                (s(a, "variable"), s(b, "variable")),
                (s(a, "domain_ref"), s(b, "domain_ref")),
                (
                    s(&a["cardinality"], "domain"),
                    s(&b["cardinality"], "domain"),
                ),
            ] {
                if carrier_difference && (ar != s(a, "variable") || br != s(b, "variable")) {
                    continue;
                }
                if ar.is_empty() && br.is_empty() {
                    continue;
                }
                let (Some(av), Some(bv)) = (
                    self.reference(self.source, ar),
                    self.reference(self.question, br),
                ) else {
                    return None;
                };
                if av["denotation"] != bv["denotation"]
                    || s(&av["declared_class"], "quote").is_empty()
                    || av["declared_class"]["quote"] != bv["declared_class"]["quote"]
                    || av["semantic_type"]["type_id"] != bv["semantic_type"]["type_id"]
                    || ar != s(a, "variable") && (!av["name"].is_null() || !bv["name"].is_null())
                    || !av["anaphora"].is_null()
                    || !bv["anaphora"].is_null()
                    || !rows(av, "links").is_empty() && ar != s(a, "variable")
                    || !rows(bv, "links").is_empty() && br != s(b, "variable")
                {
                    return None;
                }
            }
            if map
                .insert(s(a, "variable").to_owned(), s(b, "variable").to_owned())
                .is_some()
            {
                return None;
            }
        }
        // A mapped variable is not permission to ignore its own links. Check
        // every variable's complete pattern without that root's alpha shortcut.
        // Other mapped roots have their own independently checked pattern.
        for (source_variable, question_variable) in &map {
            let mut remaining = map.clone();
            remaining.remove(source_variable);
            if !super::super::quantified_match::reference_pattern_matches(
                self.source,
                source_variable,
                self.question,
                question_variable,
                s(self.scope, "namespace"),
                &remaining,
            ) {
                return None;
            }
        }
        Some((map, normalized_carrier))
    }

    pub(super) fn event_pattern_context(&self, r: &Value, out: &mut Evidence) {
        let target = &r["target"];
        if s(r, "mode") != "event_slot" || target.get("event_kind").is_none() {
            return;
        }
        for c in rows(self.source, "claims") {
            let body = &c["body"];
            if s(body, "kind") != "event"
                || body["event_kind"] != target["event_kind"]
                || !matches!(s(body, "status"), "occurred" | "planned")
                // A plan is not documentary evidence of completion. The
                // generic status selector must not recover it for a truth
                // status question through this context-only route.
                || s(r, "slot") == "status"
                    && s(r, "required_answer_extent") == "truth_status"
                    && s(target, "status") == "any"
                    && s(body, "status") != "occurred"
                || s(target, "status") != "any" && target["status"] != body["status"]
                || !target["aspect"].is_null() && target["aspect"] != body["aspect"]
                || !self.branch(self.source, s(c, "scope_id")).is_empty()
                || !self.branch(self.question, s(r, "scope_id")).is_empty()
                || !self
                    .dependency_missing(self.source, "claim", id(c))
                    .is_empty()
                || !self
                    .dependency_missing(self.question, "request", id(r))
                    .is_empty()
                || !self.common_quantity_scope_compatible(c, r)
                || s(&r["basis_requirement"], "kind") == "exact"
                    && r["basis_requirement"]["value"] != body["basis"]
            {
                continue;
            }
            // A named event is a restrictive referent. This route handles
            // anonymous event descriptions only; names retain strict identity.
            if [
                (self.source, &body["event_ref"]),
                (self.question, &target["event_ref"]),
            ]
            .iter()
            .any(|(doc, event)| {
                !event.is_null()
                    && event
                        .as_str()
                        .and_then(|id| self.reference(doc, id))
                        .is_none_or(|reference| !reference["name"].is_null())
            }) {
                continue;
            }
            let Some((map, normalized_carrier)) = self.event_context_map(c, r) else {
                continue;
            };
            let Some(subject) = self.reference(self.source, s(body, "subject")) else {
                continue;
            };
            let Some(wanted) = self.reference(self.question, s(target, "subject")) else {
                continue;
            };
            // No bare class/mention matching and no competing named subject.
            if !subject["name"].is_null()
                || !wanted["name"].is_null()
                || rows(subject, "links").is_empty()
                || rows(wanted, "links").is_empty()
                || !super::super::quantified_match::reference_pattern_matches(
                    self.source,
                    s(body, "subject"),
                    self.question,
                    s(target, "subject"),
                    s(self.scope, "namespace"),
                    &map,
                )
            {
                continue;
            }
            let (context, _) = self.context(
                self.source,
                &self.claim_qualifiers(c, &Value::Null),
                self.question,
                &self.request_qualifiers(r),
                true,
            );
            if context != Relation::Same {
                continue;
            }
            // Explicit selectors are restrictive; absence is not permission to
            // claim the same event or use a different method/time.
            if !target["time_selector"].is_null()
                && !rows(body, "times").iter().any(|t| {
                    t["role"] == target["time_selector"]["role"]
                        && self.term(
                            self.source,
                            &t["value"],
                            self.question,
                            &target["time_selector"]["value"],
                        ) == Relation::Same
                })
                || s(&target["method_selector"], "kind") == "specified"
                    && (s(&body["method"], "state") != "specified"
                        || self.term(
                            self.source,
                            &body["method"]["value"],
                            self.question,
                            &target["method_selector"]["value"],
                        ) != Relation::Same)
            {
                continue;
            }
            // Existing stronger/partial routes retain ownership when present.
            if out
                .support
                .iter()
                .chain(&out.refutation)
                .chain(&out.partial)
                .any(|w| rows(w, "claim_ids").contains(&c["id"]))
            {
                continue;
            }
            let mut missing = vec![
                "identity".to_owned(),
                "event_identity".to_owned(),
                "answer_slot".to_owned(),
            ];
            if normalized_carrier {
                missing.push("cardinality_carrier_identity".to_owned());
            }
            if !rows(r, "premise_ids").is_empty() {
                missing.push("premise".to_owned());
            }
            let mut witness = self.witness(
                c,
                r,
                &json!({"judgment":"fact"}),
                "partial_event_context",
                vec![],
                missing,
                &[],
            );
            witness["source_event"] = body.clone();
            witness["source_qualifiers"] = json!(
                self.claim_qualifiers(c, &Value::Null)
                    .iter()
                    .filter_map(|qid| self.qualifier(self.source, qid.as_str()?))
                    .collect::<Vec<_>>()
            );
            witness["bindings"] = json!([]);
            witness["event_context"] = json!({"context_pattern_proven":true,
                "event_identity_proven":false,"requested_value_inferred":false,
                "cardinality_carrier_normalized":normalized_carrier,"cardinality_carrier_identity_proven":false,
                "source_subject":body["subject"],"question_subject":target["subject"],
                "alpha_pattern":map});
            out.partial(witness);
        }
    }
}

#[cfg(test)]
#[path = "event_pattern_context_tests.rs"]
mod tests;
