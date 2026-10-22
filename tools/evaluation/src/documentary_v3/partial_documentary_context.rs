//! Context-only transport. It never discharges the requested value or its basis.
use super::*;

impl Engine<'_> {
    fn context_named_anchor(&self, actual: &Value, wanted: &Value) -> bool {
        if s(actual, "kind") != "ref" || s(wanted, "kind") != "ref" {
            return false;
        }
        self.description_anchor_evidence(s(actual, "id"), s(wanted, "id"))
            .is_some()
    }

    pub(super) fn partial_documentary_context(&self, c: &Value, r: &Value, out: &mut Evidence) {
        let p = &c["body"]["value"];
        let atom = &p["atom"];
        let target = &r["target"]["atom"];
        if s(&c["body"], "kind") != "proposition"
            || !matches!(
                s(atom, "predicate"),
                "documentary_availability" | "information_state"
            )
            || s(atom, "polarity") != "positive"
            || s(p, "judgment") != "fact"
            || s(p, "status") != "state"
            || !matches!(s(p, "basis"), "description" | "observation")
            || s(r, "mode") != "answer"
            || !matches!(
                (s(target, "predicate"), s(r, "slot")),
                ("property", "value") | ("quantity", "amount")
            )
            || s(target, "polarity") != "positive"
            || s(&r["target"], "judgment") != "fact"
            || s(&r["target"], "status") != "state"
            || self.scope_match(r, p) != Relation::Same
            || !self.premises(r).is_empty()
            || !self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
            || !self
                .dependency_missing(self.question, "request", id(r))
                .is_empty()
            || !self.branch(self.source, s(c, "scope_id")).is_empty()
            || !self.branch(self.question, s(r, "scope_id")).is_empty()
            || !self.binders(self.source, s(c, "scope_id")).is_empty()
            || !self.binders(self.question, s(r, "scope_id")).is_empty()
        {
            return;
        }
        let (Some(subject), Some(wanted), Some(attribute), Some(wanted_attribute), Some(_)) = (
            argument(atom, "subject"),
            argument(target, "subject"),
            argument(atom, "attribute"),
            argument(target, "attribute"),
            argument(atom, "state"),
        ) else {
            return;
        };
        if !self.context_named_anchor(subject, wanted)
            || self.term(self.source, attribute, self.question, wanted_attribute) != Relation::Same
            || !self.required_associations_apply(p, r)
        {
            return;
        }
        // A preservation qualifier describes the documentary support, not the
        // unknown value. No other unmatched applicability condition is removed.
        let source_ids = self.claim_qualifiers(c, p);
        let target_ids = self.request_qualifiers(r);
        let mut support_state = Vec::new();
        let mut remaining = Vec::new();
        for qid in &source_ids {
            let Some(q) = self.qualifier(self.source, qid.as_str().unwrap_or("")) else {
                return;
            };
            let document_state = s(atom, "predicate") == "documentary_availability"
                && argument(atom, "document_scope").is_some()
                && s(q, "kind") == "configuration"
                && s(q, "resolution") == "known"
                && s(&q["body"], "key") == "document_preservation_state"
                && s(&q["body"]["value"], "kind") == "text"
                && !s(&q["body"]["value"], "value").is_empty();
            let requested = target_ids
                .iter()
                .filter_map(|id| self.qualifier(self.question, id.as_str()?))
                .any(|other| other["body"]["key"] == q["body"]["key"]);
            if document_state && !requested {
                support_state.push(q.clone());
            } else {
                remaining.push(qid.clone());
            }
        }
        let (context, _) = self.context(self.source, &remaining, self.question, &target_ids, true);
        if context != Relation::Same {
            return;
        }
        let basis_debt = s(&r["basis_requirement"], "kind") == "exact"
            && r["basis_requirement"]["value"] != p["basis"];
        // Existing routes already transport the equal-basis/equal-context case.
        if !basis_debt && support_state.is_empty() {
            return;
        }
        if rows(self.source_report, "candidates")
            .iter()
            .any(|candidate| {
                !matches!(s(candidate, "state"), "eligible" | "conditional")
                    && rows(candidate, "claim_ids").contains(&c["id"])
            })
        {
            return;
        }
        let mut missing = vec!["answer_slot".into(), "documentary_state_only".into()];
        if basis_debt {
            missing.push("basis".into());
        }
        if !support_state.is_empty() {
            missing.push("document_support_context".into());
        }
        let mut w = self.witness(
            c,
            r,
            p,
            "partial_documentary_context_v2",
            vec![],
            missing.clone(),
            &[],
        );
        w["answer_propositions"] = json!([p]);
        w["source_qualifiers"] = json!(
            source_ids
                .iter()
                .filter_map(|id| self.qualifier(self.source, id.as_str()?))
                .collect::<Vec<_>>()
        );
        w["request_target"] = r["target"].clone();
        w["context_transport"] = json!({"version":"documentary_context_transport_v2","requested_value_inferred":false,"requested_basis_proven":!basis_debt,"document_support_qualifiers":support_state,"named_anchor":self.description_anchor_evidence(s(subject,"id"),s(wanted,"id"))});
        out.partial(w);
        out.unresolved.push(self.candidate(c, r, &missing));
    }

    pub(super) fn partial_negative_measurement_context(&self, r: &Value, out: &mut Evidence) {
        if s(&r["basis_requirement"], "kind") != "exact"
            || !self.branch(self.question, s(r, "scope_id")).is_empty()
            || !self.binders(self.question, s(r, "scope_id")).is_empty()
            || !self
                .dependency_missing(self.question, "request", id(r))
                .is_empty()
            || !self.premises(r).is_empty()
        {
            return;
        }
        let Some(wanted) = argument(&r["target"]["atom"], "subject") else {
            return;
        };
        let mut contextual_request = r.clone();
        contextual_request["basis_requirement"] = json!({"kind":"any"});
        let mut context = Evidence::default();
        // Reuse every existing profile, configuration, association, scope,
        // branch and dependency check. Only the requested answer basis changes.
        self.measurement_gap_context(&contextual_request, &mut context);
        for mut w in context.partial {
            let Some(cid) = rows(&w, "claim_ids")
                .first()
                .and_then(Value::as_str)
                .map(str::to_owned)
            else {
                continue;
            };
            let Some(c) = rows(self.source, "claims")
                .iter()
                .find(|c| id(c) == cid.as_str())
            else {
                continue;
            };
            let body = &c["body"];
            let actual = json!({"kind":"ref","id":body["subject"]});
            if r["basis_requirement"]["value"] == body["basis"]
                || !self.context_named_anchor(&actual, wanted)
            {
                continue;
            }
            let mut missing = strings(rows(&w, "missing_constraints"));
            missing.push("basis".into());
            unique(&mut missing);
            w["kind"] = json!("partial_negative_measurement_context_v2");
            w["id"] = json!(format!(
                "{}:{}:partial_negative_measurement_context_v2",
                id(r),
                cid
            ));
            w["missing_constraints"] = json!(missing);
            w["answer_terms"] = json!([]);
            w["answer_propositions"] = json!([]);
            w["request_target"] = r["target"].clone();
            w["context_transport"] = json!({"version":"documentary_context_transport_v2","requested_value_inferred":false,"requested_basis_proven":false});
            out.partial(w);
            out.unresolved.push(self.candidate(c, r, &missing));
        }
    }
}

#[cfg(test)]
#[path = "partial_documentary_context_tests.rs"]
mod tests;
