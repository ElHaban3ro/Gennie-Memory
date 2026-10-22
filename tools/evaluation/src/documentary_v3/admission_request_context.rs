//! Admission constraints retained as context, with the actual-use premise still pending.
use super::*;
impl Engine<'_> {
    fn admission_request_clean_ref(&self, doc: &Value, rid: &str) -> bool {
        self.reference(doc, rid).is_some()
            && self.local(doc, rid, rid) == Relation::Same
            && rows(doc, "references")
                .iter()
                .filter(|r| self.local(doc, rid, id(r)) == Relation::Same)
                .all(|r| {
                    r["anaphora"].is_null()
                        && r["binding_scope_id"].is_null()
                        && self.branch(doc, s(r, "scope_id")).is_empty()
                        && self.binders(doc, s(r, "scope_id")).is_empty()
                        && self.dependency_missing(doc, "reference", id(r)).is_empty()
                })
    }
    fn admission_request_mode_domain(&self, doc: &Value, term: &Value) -> bool {
        if s(term, "kind") != "ref" || !self.admission_request_clean_ref(doc, id(term)) {
            return false;
        }
        let mut lexical = false;
        for r in rows(doc, "references")
            .iter()
            .filter(|r| self.local(doc, id(term), id(r)) == Relation::Same)
        {
            if !r["declared_class"].is_null() {
                if s(&r["declared_class"], "quote") != "modo" {
                    return false;
                }
                lexical = true;
            }
            if !r["semantic_type"].is_null() {
                let types: Vec<_> = rows(self.vocabulary, "semantic_type_rows")
                    .iter()
                    .filter(|t| t["id"] == r["semantic_type"]["type_id"])
                    .collect();
                if types.len() != 1 || !rows(types[0], "class_quotes").contains(&json!("modo")) {
                    return false;
                }
                if rows(self.vocabulary, "semantic_type_rows")
                    .iter()
                    .filter(|t| rows(t, "class_quotes").contains(&json!("modo")))
                    .count()
                    != 1
                {
                    return false;
                }
            }
        }
        lexical
    }
    fn admission_request_member(&self, member: &Value) -> Option<Value> {
        if s(member, "kind") == "ref" {
            return Some(member.clone());
        }
        if s(member, "kind") != "described_ref"
            || !self.descriptor_consistent(self.source, member)
            || rows(member, "constraints").len() != 1
        {
            return None;
        }
        let p = &rows(member, "constraints")[0];
        let a = &p["atom"];
        if s(a, "predicate") != "quantity"
            || s(a, "polarity") != "positive"
            || s(p, "judgment") != "fact"
            || s(p, "status") != "state"
            || s(p, "basis") != "description"
            || !p["normative_scope"].is_null()
            || !rows(p, "qualifier_ids").is_empty()
            || rows(a, "arguments").len() != 3
        {
            return None;
        }
        let root = argument(a, "subject")?;
        let attribute = argument(a, "attribute")?;
        let amount = argument(a, "amount")?;
        if s(root, "kind") != "ref"
            || id(root) != id(member)
            || s(attribute, "kind") != "symbol"
            || id(attribute) != "resolution"
            || s(amount, "kind") != "quantity"
            || s(amount, "bound") != "exact"
            || s(amount, "unit").trim().is_empty()
            || !s(amount, "decimal")
                .parse::<f64>()
                .is_ok_and(f64::is_finite)
        {
            return None;
        }
        Some(json!({"kind":"ref","id":member["id"]}))
    }
    pub(super) fn admission_request_context(&self, c: &Value, r: &Value, out: &mut Evidence) {
        if !self.context_v2 {
            return;
        }
        let target = &r["target"];
        let atom = &target["atom"];
        if s(r, "mode") != "answer"
            || s(atom, "predicate") != "property"
            || s(atom, "polarity") != "positive"
            || s(target, "judgment") != "fact"
            || s(target, "status") != "state"
            || argument(atom, "value").is_some()
            || !r["normative_scope_filter"].is_null()
            || r["basis_requirement"] != json!({"kind":"exact","value":"assignment"})
        {
            return;
        }
        let profile = &self.vocabulary["admission_request_context"];
        if profile["version"] != "admission_request_context_v1" {
            return;
        }
        let Some(attribute) = argument(atom, "attribute") else {
            return;
        };
        let profiles: Vec<_> = rows(profile, "rows")
            .iter()
            .filter(|p| p["target_attribute"] == attribute["id"] && p["request_slot"] == r["slot"])
            .collect();
        if profiles.len() != 1 {
            return;
        }
        let row = profiles[0];
        if !rows(self.vocabulary, "relevance_links").iter().any(|l| {
            l["id"] == row["link_id"]
                && l["source_predicate"] == "admits"
                && l["target_predicate"] == "property"
                && l["bindings"] == json!([["source.subject", "target.subject"]])
                && l["carry_roles"] == json!(["source.object"])
                && l["conclusion"] == "partial_intermediate_only"
                && l["max_hops"] == 1
                && l["allow_reverse"] == false
                && l["allow_transitive_composition"] == false
        }) {
            return;
        }
        let body = &c["body"];
        let closed = s(body, "kind") == "value_restriction";
        let p = if closed {
            json!({"atom":body["target"],"judgment":body["judgment"],"status":body["status"],"basis":body["basis"],"normative_scope":body["normative_scope"],"qualifier_ids":c["qualifier_ids"]})
        } else {
            body["value"].clone()
        };
        if s(&p["atom"], "predicate") != "admits"
            || s(&p, "judgment") != "fact"
            || s(&p, "status") != "state"
            || !matches!(s(&p, "basis"), "description" | "capability")
            || !p["normative_scope"].is_null()
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
        if closed {
            if body["closure"] != "closed"
                || body["existence"] != "asserted"
                || body["answer_role"] != "object"
                || p["atom"]["polarity"] != "positive"
                || rows(body, "members").is_empty()
            {
                return;
            }
            if body["answer_domain"]["kind"] != "lexical"
                || body["answer_domain"]["class_quote"]["quote"] != "modo"
            {
                out.unresolved
                    .push(self.candidate(c, r, &["admission_member_domain".into()]));
                return;
            }
        } else if body["kind"] != "proposition" || p["atom"]["polarity"] != "negative" {
            return;
        }
        let (Some(subject), Some(wanted_subject)) =
            (argument(&p["atom"], "subject"), argument(atom, "subject"))
        else {
            return;
        };
        if s(subject, "kind") != "ref"
            || s(wanted_subject, "kind") != "ref"
            || !self.admission_request_clean_ref(self.source, id(subject))
            || !self.admission_request_clean_ref(self.question, id(wanted_subject))
            || !self.weak_reference_anchor(id(subject), id(wanted_subject))
            || !self.admission_setting_context(subject, wanted_subject, true)
        {
            return;
        }
        let qids = self.request_qualifiers(r);
        let settings: Vec<_> = qids
            .iter()
            .filter_map(|q| self.qualifier(self.question, q.as_str()?))
            .filter(|q| q["kind"] == "configuration" && q["body"]["key"] == row["setting_key"])
            .collect();
        if settings.len() != 1
            || settings[0]["resolution"] != "known"
            || !rows(target, "qualifier_ids").contains(&settings[0]["id"])
        {
            return;
        }
        let setting = settings[0];
        let wanted = &setting["body"]["value"];
        if s(wanted, "kind") != "ref"
            || !self.admission_request_clean_ref(self.question, id(wanted))
        {
            return;
        }
        let mut members = vec![];
        if closed {
            if !self.admission_request_mode_domain(self.question, wanted) {
                out.unresolved
                    .push(self.candidate(c, r, &["admission_member_domain".into()]));
                return;
            }
            for member in rows(body, "members") {
                let Some(base) = self.admission_request_member(member) else {
                    out.unresolved.push(self.candidate(
                        c,
                        r,
                        &["unsupported_admission_descriptor".into()],
                    ));
                    return;
                };
                if !self.admission_request_mode_domain(self.source, &base) {
                    out.unresolved
                        .push(self.candidate(c, r, &["admission_member_domain".into()]));
                    return;
                }
                let Some(label) = self.configuration_label(self.question, wanted) else {
                    return;
                };
                if self.term(self.source, &base, self.question, wanted) == Relation::Same
                    || self.configuration_label(self.source, &base).is_none()
                    || self.configuration_label_matches(self.source, &base, label)
                    || !self.admission_setting_context(&base, wanted, false)
                {
                    return;
                }
                members.push(base);
            }
        } else if !argument(&p["atom"], "object").is_some_and(|a| {
            self.admission_request_clean_ref(self.source, id(a))
                && self.admission_setting_context(a, wanted, true)
        }) {
            return;
        }
        // Exactly this documentary premise, not a general permission to ignore premises.
        if rows(r, "premise_ids").len() != 1 {
            return;
        }
        let pid = &rows(r, "premise_ids")[0];
        let premises: Vec<_> = rows(self.question, "claims")
            .iter()
            .filter(|p| p["id"] == *pid)
            .collect();
        if premises.len() != 1 {
            return;
        }
        let premise = premises[0];
        let pp = &premise["body"]["value"];
        let pa = &pp["atom"];
        let edges: Vec<_> = rows(self.question, "dependencies")
            .iter()
            .filter(|e| {
                e["consumer"]["kind"] == "request"
                    && e["consumer"]["id"] == r["id"]
                    && e["role"] == "premise"
            })
            .collect();
        if edges.len() != 1
            || edges[0]["requires"] != json!({"kind":"claim","id":pid})
            || premise["body"]["kind"] != "proposition"
            || pa["predicate"] != row["premise_predicate"]
            || pa["polarity"] != "positive"
            || pp["judgment"] != "fact"
            || pp["status"] != "state"
            || pp["basis"] != "assignment"
            || !pp["normative_scope"].is_null()
            || !self
                .branch(self.question, s(premise, "scope_id"))
                .is_empty()
            || !self
                .binders(self.question, s(premise, "scope_id"))
                .is_empty()
            || !self
                .dependency_missing(self.question, "claim", id(premise))
                .is_empty()
        {
            return;
        }
        let same_question = |a: &Value, b: &Value| {
            s(a, "kind") == "ref"
                && s(b, "kind") == "ref"
                && self.admission_request_clean_ref(self.question, id(a))
                && self.local(self.question, id(a), id(b)) == Relation::Same
        };
        if !argument(pa, s(row, "premise_subject_role"))
            .is_some_and(|a| same_question(a, wanted_subject))
            || !argument(pa, s(row, "premise_setting_role"))
                .is_some_and(|a| same_question(a, wanted))
        {
            return;
        }
        let premise_ids = self.claim_qualifiers(premise, pp);
        let without_setting = |ids: Vec<Value>| {
            ids.into_iter()
                .filter(|q| q != &setting["id"])
                .collect::<Vec<_>>()
        };
        if self
            .context(
                self.question,
                &without_setting(premise_ids),
                self.question,
                &without_setting(qids.clone()),
                false,
            )
            .0
            != Relation::Same
        {
            return;
        }
        let source_ids = self.claim_qualifiers(c, &p);
        // Consume only the selected request setting; every source condition still needs proof.
        let source_has_setting = source_ids
            .iter()
            .filter_map(|q| self.qualifier(self.source, q.as_str()?))
            .any(|a| {
                self.qualifier_relation(self.source, a, self.question, setting) == Relation::Same
            });
        let target_ids = if source_has_setting {
            qids.clone()
        } else {
            without_setting(qids)
        };
        if self
            .context(self.source, &source_ids, self.question, &target_ids, true)
            .0
            != Relation::Same
        {
            return;
        }
        let mut missing = vec![
            "admission_only".into(),
            "answer_slot".into(),
            "basis".into(),
            "premise".into(),
        ];
        if closed {
            missing.push("admission_membership_unknown".into());
        }
        if self.term(self.source, subject, self.question, wanted_subject) != Relation::Same {
            missing.push("identity".into());
        }
        unique(&mut missing);
        let mut witness = self.witness(
            c,
            r,
            &p,
            "partial_admission_request_context",
            vec![],
            missing.clone(),
            &[],
        );
        for key in ["answer_terms", "answer_propositions", "bindings"] {
            witness[key] = json!([]);
        }
        witness["source_claim_body"] = body.clone();
        witness["source_qualifiers"] = json!(
            source_ids
                .iter()
                .filter_map(|qid| self.qualifier(self.source, qid.as_str()?))
                .collect::<Vec<_>>()
        );
        witness["source_scopes"] = self.source["scopes"].clone();
        witness["pending_question_premise"] = json!({"premise_id":pid,"claim_body":premise["body"],"request_dependency":edges[0],"evidence":premise["evidence"]});
        witness["admission_request_context"] = json!({"profile_row":row,"requested_setting":wanted,"actual_use_inferred":false,"requested_value_inferred":false,"premise_satisfied":false,"premise_refuted":false,"property_quantity_equated":false,"membership_exclusion_inferred":false,"descriptor_value_promoted":false});
        out.partial(witness);
        out.unresolved.push(self.candidate(c, r, &missing));
    }
}
