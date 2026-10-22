//! Positive contextual preservation of the complete construction, never its core.
use super::super::{lexical_clause_context as graph, linguistic_composition as composition};
use super::*;
impl Engine<'_> {
    pub(super) fn composition_anchor(&self, sd: &Value, left: &str, right: &str) -> bool {
        if self.identity(sd, left, self.question, right) != Relation::Same
            || !self.context_clean_reference(sd, left)
            || !self.context_clean_reference(self.question, right)
        {
            return false;
        }
        let aliases = |doc: &Value, rid: &str| -> Vec<String> {
            rows(doc, "references")
                .iter()
                .filter(|r| {
                    super::super::context_budget::charge(1, 0)
                        && self.local(doc, rid, id(r)) == Relation::Same
                })
                .map(|r| id(r).to_owned())
                .collect()
        };
        if !composition::reserve(&sd["references"], 2)
            || !composition::reserve(&self.question["references"], 2)
        {
            return false;
        }
        let a = aliases(sd, left);
        let b = aliases(self.question, right);
        for (doc, own, otherdoc, other) in
            [(sd, &a, self.question, &b), (self.question, &b, sd, &a)]
        {
            for alias in own {
                let Some(reference) = self.reference(doc, alias) else {
                    return false;
                };
                // A type/class restriction on an alias cannot disappear behind another name anchor.
                for key in ["declared_class", "semantic_type"] {
                    if reference[key].is_null() {
                        continue;
                    }
                    let field = if key == "declared_class" {
                        "quote"
                    } else {
                        "type_id"
                    };
                    if !other
                        .iter()
                        .filter_map(|r| self.reference(otherdoc, r))
                        .any(|r| {
                            super::super::context_budget::charge(1, 0)
                                && r[key][field] == reference[key][field]
                        })
                    {
                        return false;
                    }
                }
                for link in rows(reference, "links") {
                    if !super::super::context_budget::charge(1, 0) {
                        return false;
                    }
                    let matched = other
                        .iter()
                        .filter_map(|r| self.reference(otherdoc, r))
                        .flat_map(|r| rows(r, "links"))
                        .any(|candidate| {
                            if !super::super::context_budget::charge(1, 0)
                                || candidate["kind"] != link["kind"]
                            {
                                return false;
                            }
                            if link["kind"] == super::super::nominal_de::KIND {
                                return super::super::nominal_de::descriptor_key(
                                    doc,
                                    s(link, "target"),
                                )
                                .zip(super::super::nominal_de::descriptor_key(
                                    otherdoc,
                                    s(candidate, "target"),
                                ))
                                .is_some_and(|(a, b)| a == b);
                            }
                            self.identity(doc, s(link, "target"), otherdoc, s(candidate, "target"))
                                == Relation::Same
                        });
                    if !matched {
                        return false;
                    }
                }
            }
        }
        true
    }
    pub(super) fn composition_guards_consistent(&self, doc: &Value, ids: &[Value]) -> bool {
        let execution = if self.is_source_document(doc) {
            self.source
        } else {
            doc
        };
        for qualifier in ids
            .iter()
            .filter_map(|q| q.as_str().and_then(|id| self.qualifier(doc, id)))
        {
            if !super::super::context_budget::charge(1, 0) {
                return false;
            }
            if qualifier["kind"] != "guard" {
                continue;
            }
            let guard = &qualifier["body"];
            if !composition::reserve(guard, 2) {
                return false;
            }
            let mut opposite = guard.clone();
            opposite["atom"]["polarity"] = json!(if guard["atom"]["polarity"] == "positive" {
                "negative"
            } else {
                "positive"
            });
            if rows(execution, "claims").iter().any(|c| {
                super::super::context_budget::charge(1, 0)
                    && c["body"]["kind"] == "proposition"
                    && self.claim_qualifiers(c, &c["body"]["value"]).is_empty()
                    && self.branch(doc, s(c, "scope_id")).is_empty()
                    && self.proposition(doc, &c["body"]["value"], doc, &opposite, None)
                        == Relation::Same
            }) {
                return false;
            }
        }
        true
    }
    pub(super) fn composition_pair_context(&self, sd: &Value, source_ids: &[Value]) -> bool {
        let mut keys = BTreeSet::new();
        for configuration in rows(self.scope, "configuration") {
            if !super::super::context_budget::charge(keys.len() + 1, 128)
                || !keys.insert(s(configuration, "key"))
            {
                return false;
            }
            if !super::super::context_budget::charge(1, 0) {
                return false;
            }
            if !source_ids
                .iter()
                .filter_map(|q| q.as_str().and_then(|q| self.qualifier(sd, q)))
                .any(|q| {
                    super::super::context_budget::charge(1, 0)
                        && q["kind"] == "configuration"
                        && q["resolution"] == "known"
                        && q["body"]["key"] == configuration["key"]
                        && self.term(
                            sd,
                            &q["body"]["value"],
                            self.question,
                            &configuration["value"],
                        ) == Relation::Same
                })
            {
                return false;
            }
        }
        if !self.scope["time_selector"].is_null()
            && !source_ids
                .iter()
                .filter_map(|q| q.as_str().and_then(|q| self.qualifier(sd, q)))
                .any(|q| {
                    super::super::context_budget::charge(1, 0)
                        && q["kind"] == "time"
                        && q["resolution"] == "known"
                        && q["body"]["role"] == self.scope["time_selector"]["role"]
                        && self.term(
                            sd,
                            &q["body"]["value"],
                            self.question,
                            &self.scope["time_selector"]["value"],
                        ) == Relation::Same
                })
        {
            return false;
        }
        true
    }
    pub(super) fn composition_coverage_pending(&self, doc: &Value, cl: &graph::Nodes) -> bool {
        rows(doc, "coverage").iter().any(|row| {
            if !super::super::context_budget::charge(cl.len() + 1, 0) {
                return true;
            }
            if row["state"] == "represented" {
                return false;
            }
            let owner = &row["owner"];
            owner.is_null() || cl.contains(&(s(owner, "kind").to_owned(), id(owner).to_owned()))
        })
    }
    pub(super) fn linguistic_composition_context(&self, r: &Value, out: &mut Evidence) {
        if !self.context_v2
            || !super::super::request_context_profile::permits(
                self.vocabulary,
                composition::FEATURE,
            )
        {
            return;
        }
        let target = &r["target"];
        let atom = &target["atom"];
        let sd = &self.source_report["document"];
        let exact_request = r["mode"] == "answer"
            && r["slot"] == "amount"
            && atom["predicate"] == "quantity"
            && atom["polarity"] == "positive"
            && target["judgment"] == "fact"
            && target["status"] == "state"
            && rows(atom, "arguments").len() == 2
            && argument(atom, "amount").is_none()
            && argument(atom, "attribute")
                .is_some_and(|a| a["kind"] == "symbol" && a["id"] == "duration")
            && argument(atom, "subject").is_some_and(|a| a["kind"] == "ref");
        for c in rows(sd, "claims") {
            if !super::super::context_budget::charge(1, 0) {
                return;
            }
            if !composition::carries(c) || c["body"].get("constructor").is_some() {
                continue;
            }
            // Full copies are reserved before any candidate/witness allocation.
            if !composition::reserve(sd, 3)
                || !composition::reserve(self.question, 3)
                || !composition::reserve(r, 2)
            {
                return;
            }
            let owner = &c["body"]["core"]["operand"]["object"]["owner"];
            let mut debt = vec![
                "answer_slot".to_owned(),
                "until_attachment".to_owned(),
                "until_negation_scope".to_owned(),
                "boundary_subject".to_owned(),
                "measurement_episode".to_owned(),
            ];
            if owner["kind"] != "explicit_owner" {
                debt.push("possessive_owner".into());
            }
            let actual = s(&owner["term"], "id");
            let wanted = argument(atom, "subject").map(|a| id(a)).unwrap_or("");
            let mut compatible = exact_request && owner["kind"] == "explicit_owner";
            if compatible && !self.composition_anchor(sd, actual, wanted) {
                compatible = false;
                debt.push("identity".into());
            }
            if !exact_request {
                compatible = false;
                debt.push("unsupported_request".into());
            }
            if !self.dependency_missing(sd, "claim", id(c)).is_empty()
                || !self
                    .dependency_missing(self.question, "request", id(r))
                    .is_empty()
            {
                compatible = false;
                debt.push("dependency".into());
            }
            if !self.branch(sd, s(c, "scope_id")).is_empty()
                || !self.binders(sd, s(c, "scope_id")).is_empty()
                || !self.branch(self.question, s(r, "scope_id")).is_empty()
                || !self.binders(self.question, s(r, "scope_id")).is_empty()
            {
                compatible = false;
                debt.push("scope".into());
            }
            if r.get("context_bindings").is_some()
                || c.get("context_bindings").is_some()
                || !self.premises(r).is_empty()
                || !r["normative_scope_filter"].is_null()
            {
                compatible = false;
                debt.push("request_context".into());
            }
            let source_closure = graph::closure(sd, "claim", id(c));
            let request_closure = graph::closure(self.question, "request", id(r));
            if self.composition_coverage_pending(sd, &source_closure) {
                compatible = false;
                debt.push("source_coverage".into());
            }
            if self.composition_coverage_pending(self.question, &request_closure) {
                compatible = false;
                debt.push("request_coverage".into());
            }
            let source_ids = self.qualified_ids(sd, c, &Value::Null);
            let request_ids = self.request_qualifiers(r);
            let (context, missing) =
                self.context(sd, &source_ids, self.question, &request_ids, true);
            if context != Relation::Same
                || !self.composition_pair_context(sd, &source_ids)
                || !self.composition_guards_consistent(sd, &source_ids)
                || !self.composition_guards_consistent(self.question, &request_ids)
            {
                compatible = false;
                debt.extend(missing);
                debt.push("effective_context".into());
            }
            if r["basis_requirement"]["kind"] == "exact" {
                debt.push("basis".into());
            }
            if !r["class_filter"].is_null() {
                debt.push("answer_reference_pattern".into());
            }
            if !r["unit"].is_null() {
                debt.push("answer_unit".into());
            }
            unique(&mut debt);
            let mut candidate = self.candidate(c, r, &debt);
            candidate["source_constraint"] = c.clone();
            candidate["source_document"] = sd.clone();
            candidate["request_constraint"] = r.clone();
            candidate["answer_terms"] = json!([]);
            candidate["answer_propositions"] = json!([]);
            candidate["bindings"] = json!([]);
            out.unresolved.push(candidate);
            if !compatible {
                continue;
            }
            if !super::super::context_budget::charge(
                source_closure.len() + request_closure.len(),
                (source_closure.len() + request_closure.len()).saturating_mul(512),
            ) {
                return;
            }
            let endpoints = |nodes: graph::Nodes| {
                nodes
                    .into_iter()
                    .map(|(kind, id)| json!({"kind":kind,"id":id}))
                    .collect::<Vec<_>>()
            };
            let mut witness = self.witness(
                c,
                r,
                &Value::Null,
                "partial_relevance",
                vec![],
                debt.clone(),
                &[],
            );
            witness["bindings"] = json!([]);
            witness["answer_terms"] = json!([]);
            witness["answer_propositions"] = json!([]);
            witness["source_constraint"] = c["body"].clone();
            witness["source_owner"] = c.clone();
            witness["source_document"] = sd.clone();
            witness["question_document"] = self.question.clone();
            witness["request_constraint"] = r.clone();
            witness["source_dependencies"] = sd["dependencies"].clone();
            witness["source_qualifiers"] = sd["qualifiers"].clone();
            witness["request_dependencies"] = self.question["dependencies"].clone();
            witness["request_qualifiers"] = self.question["qualifiers"].clone();
            witness["effective_source_closure"] = json!(endpoints(source_closure));
            witness["effective_request_closure"] = json!(endpoints(request_closure));
            witness["context_transport"] = json!({"version":"documentary_context_transport_v2","construction":c["body"]["construction"],"original_context":"fixed_source_occurrence","requested_value_inferred":false,"requested_basis_proven":false,"core_independently_asserted":false,"measurement_occurrence_inferred":false,"boundary_occurrence_inferred":false,"internal_attachment_resolved":false,"boundary_subject_resolved":false});
            out.partial(witness);
        }
    }
}
