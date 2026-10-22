//! Relevance of an intact source-bound nominal construction, not its semantic role.
use super::super::{
    context_budget as budget, lexical_clause_context as graph,
    linguistic_composition as composition, nominal_event_context as nominal,
};
use super::*;
impl Engine<'_> {
    pub(super) fn nominal_event_context(&self, r: &Value, out: &mut Evidence) {
        if !self.context_v2
            || !super::super::request_context_profile::permits(self.vocabulary, nominal::FEATURE)
        {
            return;
        }
        let sd = &self.source_report["document"];
        let target = &r["target"];
        let atom = &target["atom"];
        let exact = r["mode"] == "truth"
            && r["slot"] == "status"
            && r["required_answer_extent"] == "truth_status"
            && atom["predicate"] == "action_beneficiary"
            && atom["polarity"] == "positive"
            && target["judgment"] == "fact"
            && target["status"] == "occurred"
            && rows(atom, "arguments").len() == 3
            && argument(atom, "subject").is_some_and(|a| a["kind"] == "ref")
            && argument(atom, "beneficiary").is_some_and(|a| a["kind"] == "ref")
            && argument(atom, "action_kind")
                .is_some_and(|a| a["kind"] == "symbol" && a["id"] == "remove")
            && self.premises(r).is_empty()
            && r["class_filter"].is_null()
            && r["unit"].is_null()
            && r["normative_scope_filter"].is_null()
            && target["normative_scope"].is_null()
            && r.get("context_bindings").is_none();
        for c in rows(sd, "claims") {
            if !budget::charge(1, 0) {
                return;
            }
            let body = &c["body"];
            if !nominal::carries(body) {
                continue;
            }
            if !composition::reserve(sd, 5)
                || !composition::reserve(self.question, 5)
                || !composition::reserve(self.vocabulary, 2)
                || !composition::reserve(r, 3)
            {
                return;
            }
            let mut debt = vec![
                "answer_slot".to_owned(),
                "patient_identity".to_owned(),
                "context_attachment".to_owned(),
                "requested_role".to_owned(),
            ];
            let mut compatible = exact;
            let source_patient = s(&body["core"]["patient"], "id");
            let wanted_patient = argument(atom, "subject").map(id).unwrap_or("");
            let wanted_beneficiary = argument(atom, "beneficiary").map(id).unwrap_or("");
            if !exact {
                debt.push("unsupported_request".into());
            }
            if !nominal::patient_key(sd, source_patient)
                .zip(nominal::patient_key(self.question, wanted_patient))
                .is_some_and(|(a, b)| a == b)
                || !nominal::pattern_constraints_clean(sd, source_patient, Some(id(c)))
                || !nominal::pattern_constraints_clean(self.question, wanted_patient, None)
            {
                compatible = false;
                debt.push("nominal_pattern".into());
            }
            if !rows(body, "contexts").iter().any(|p| {
                budget::charge(1, 0)
                    && self.composition_anchor(sd, s(&p["complement"], "id"), wanted_beneficiary)
            }) {
                compatible = false;
                debt.push("identity".into());
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
            // This constructor publishes no evidential basis; an exact demand is not a wildcard.
            if r["basis_requirement"]["kind"] == "exact" {
                compatible = false;
                debt.push("basis".into());
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
            if !budget::charge(
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
            let mut w = self.witness(
                c,
                r,
                &Value::Null,
                "source_bound_linguistic_context",
                vec![],
                debt,
                &[],
            );
            w["bindings"] = json!([]);
            w["answer_terms"] = json!([]);
            w["answer_propositions"] = json!([]);
            w["requirement_states"] = json!([]);
            w["source_constraint"] = c.clone();
            w["source_body"] = body.clone();
            w["request_constraint"] = r.clone();
            w["source_document"] = sd.clone();
            w["question_document"] = self.question.clone();
            w["vocabulary"] = self.vocabulary.clone();
            for key in ["references", "scopes", "qualifiers", "dependencies"] {
                w[format!("source_{key}")] = sd[key].clone();
                w[format!("question_{key}")] = self.question[key].clone();
            }
            w["effective_source_closure"] = json!(endpoints(source_closure));
            w["effective_request_closure"] = json!(endpoints(request_closure));
            w["nominal_event_context"] = json!({"version":"documentary_nominal_event_context_v1","constructor":nominal::FEATURE,"composition_id":body["composition_id"],"core_id":body["core"]["id"],"region_id":body["region"]["id"],"context_ids":rows(body,"contexts").iter().map(|p|p["id"].clone()).collect::<Vec<_>>(),"attachment_resolution":"not_published","patient_identity_proven":false,"requested_role_proven":false,"core_independently_asserted":false,"full_answer_supported":false});
            out.partial(w);
        }
    }
}
