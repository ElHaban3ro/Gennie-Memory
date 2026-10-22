//! Context-only preservation of a qualified, anonymous normative source.
use super::*;
impl Engine<'_> {
    // A semantic type alone cannot erase an unsupported restrictive class phrase.
    fn normative_registered_class(&self, reference: &Value) -> Option<&Value> {
        let quote = s(&reference["declared_class"], "quote");
        if quote.is_empty() {
            return None;
        }
        let mut matches = rows(self.vocabulary, "semantic_type_rows")
            .iter()
            .filter(|row| {
                rows(row, "class_quotes")
                    .iter()
                    .any(|q| q.as_str() == Some(quote))
            });
        let row = matches.next()?;
        if matches.next().is_some()
            || (!reference["semantic_type"].is_null()
                && reference["semantic_type"]["type_id"] != row["id"])
        {
            return None;
        }
        Some(row)
    }
    fn normative_class_pair(&self, actual: &Value, wanted: &Value) -> bool {
        match (
            self.normative_registered_class(actual),
            self.normative_registered_class(wanted),
        ) {
            (Some(a), Some(b)) => a["id"] == b["id"],
            _ => false,
        }
    }
    fn normative_clean_component(
        &self,
        doc: &Value,
        rid: &str,
        anonymous: bool,
        unbound: bool,
    ) -> bool {
        let aliases: Vec<_> = rows(doc, "references")
            .iter()
            .filter(|rf| self.local(doc, rid, id(rf)) == Relation::Same)
            .collect();
        let Some(primary) = self.reference(doc, rid) else {
            return false;
        };
        !aliases.is_empty()
            && self.local(doc, rid, rid) == Relation::Same
            && aliases.iter().all(|rf| {
                (!anonymous || rf["name"].is_null())
                    && (!unbound || rf["binding_scope_id"].is_null())
                    && rf["anaphora"].is_null()
                    && self.dependency_missing(doc, "reference", id(rf)).is_empty()
                    && (rf["declared_class"].is_null() || self.normative_class_pair(primary, rf))
                    && (rf["semantic_type"].is_null()
                        || self
                            .normative_registered_class(primary)
                            .is_some_and(|row| row["id"] == rf["semantic_type"]["type_id"]))
            })
    }
    fn normative_unrestricted_variable(&self, doc: &Value, rid: &str) -> bool {
        let Some(primary) = self.reference(doc, rid) else {
            return false;
        };
        self.normative_clean_component(doc, rid, true, false)
            && rows(doc, "references")
                .iter()
                .filter(|rf| self.local(doc, rid, id(rf)) == Relation::Same)
                .all(|rf| {
                    rows(rf, "links").is_empty()
                        && (rf["binding_scope_id"].is_null()
                            || rf["binding_scope_id"] == primary["binding_scope_id"])
                })
    }
    fn normative_named_context(&self, actual: &Value, wanted: &Value) -> bool {
        if !self.admission_setting_context(actual, wanted, false) {
            return false;
        }
        // At least one real named anchor is mandatory. The helper separately
        // checks all other restrictions, including their alias/nested contexts.
        rows(self.source, "references")
            .iter()
            .filter(|rf| self.local(self.source, s(actual, "id"), id(rf)) == Relation::Same)
            .any(|rf| {
                rows(rf, "links").iter().any(|link| {
                    let Some(anchor) = self.reference(self.source, s(link, "target")) else {
                        return false;
                    };
                    !s(&anchor["name"], "quote").is_empty()
                        && rows(self.question, "references")
                            .iter()
                            .filter(|rf| {
                                self.local(self.question, s(wanted, "id"), id(rf)) == Relation::Same
                            })
                            .any(|rf| {
                                rows(rf, "links").iter().any(|other| {
                                    link["kind"] == other["kind"]
                                        && self.identity(
                                            self.source,
                                            id(anchor),
                                            self.question,
                                            s(other, "target"),
                                        ) == Relation::Same
                                })
                            })
                })
            })
    }
    fn normative_named_role(&self, actual: &Value, wanted: &Value) -> bool {
        if s(actual, "kind") != "ref" || s(wanted, "kind") != "ref" {
            return self.term(self.source, actual, self.question, wanted) == Relation::Same;
        }
        let (Some(a), Some(b)) = (
            self.reference(self.source, s(actual, "id")),
            self.reference(self.question, s(wanted, "id")),
        ) else {
            return false;
        };
        !s(&a["name"], "quote").is_empty()
            && !s(&b["name"], "quote").is_empty()
            && self.normative_clean_component(self.source, id(a), false, true)
            && self.normative_clean_component(self.question, id(b), false, true)
            && self.term(self.source, actual, self.question, wanted) == Relation::Same
            && self.admission_setting_context(actual, wanted, true)
    }
    pub(super) fn anonymous_normative_context(&self, c: &Value, r: &Value, out: &mut Evidence) {
        let p = &c["body"]["value"];
        let atom = &p["atom"];
        let target = &r["target"]["atom"];
        if !self.context_v2
            || s(&c["body"], "kind") != "proposition"
            || s(p, "judgment") != "fact"
            || s(p, "status") != "state"
            || s(p, "basis") != "normative"
            || s(atom, "polarity") != "positive"
            || s(r, "mode") != "answer"
            || s(&r["target"], "judgment") != "fact"
            || s(&r["target"], "status") != "state"
            || s(target, "polarity") != "positive"
            || atom["predicate"] != target["predicate"]
            || s(&r["basis_requirement"], "kind") != "exact"
            || s(&r["basis_requirement"], "value") != "normative"
            || !self.branch(self.source, s(c, "scope_id")).is_empty()
            || !self.branch(self.question, s(r, "scope_id")).is_empty()
            || !self.premises(r).is_empty()
            || !self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
            || !self
                .dependency_missing(self.question, "request", id(r))
                .is_empty()
            || !self.common_quantity_scope_compatible(c, r)
        {
            return;
        }
        let scope_id = s(p, "normative_scope");
        let wanted_scope = s(r, "normative_scope_filter");
        if scope_id.is_empty() || wanted_scope.is_empty() {
            return;
        }
        let roles: Vec<_> = rows(atom, "arguments")
            .iter()
            .filter(|a| s(&a["value"], "kind") == "ref" && s(&a["value"], "id") == scope_id)
            .collect();
        if roles.len() != 1 {
            return;
        }
        let role = s(roles[0], "role");
        let Some(wanted) = argument(target, role) else {
            return;
        };
        if s(wanted, "kind") != "ref" || s(wanted, "id") != wanted_scope || role == s(r, "slot") {
            return;
        }
        let (Some(a), Some(b)) = (
            self.reference(self.source, scope_id),
            self.reference(self.question, wanted_scope),
        ) else {
            return;
        };
        let actual = &roles[0]["value"];
        if a["denotation"] != b["denotation"]
            || !self.normative_class_pair(a, b)
            || !self.normative_clean_component(self.source, scope_id, true, true)
            || !self.normative_clean_component(self.question, wanted_scope, true, true)
            || !self.normative_named_context(actual, wanted)
        {
            return;
        }
        let source_binders = self.binders(self.source, s(c, "scope_id"));
        let question_binders = self.binders(self.question, s(r, "scope_id"));
        if source_binders
            .iter()
            .chain(&question_binders)
            .any(|b| s(b, "quantifier") != "forall")
        {
            return;
        }
        let map = self.alpha_map(c, r);
        if !question_binders.is_empty() && map.is_none() {
            return;
        }
        let mut membership_debt = false;
        let mut role_proofs = vec![];
        for arg in rows(target, "arguments") {
            let other_role = s(arg, "role");
            if other_role == role {
                continue;
            }
            if other_role == s(r, "slot") {
                return;
            }
            let Some(av) = argument(atom, other_role) else {
                return;
            };
            let bv = &arg["value"];
            if let Some(binding) = source_binders
                .iter()
                .find(|binding| s(av, "kind") == "ref" && s(av, "id") == s(binding, "variable"))
            {
                let Some(variable) = self.reference(self.source, s(av, "id")) else {
                    return;
                };
                if !self.normative_unrestricted_variable(self.source, id(variable)) {
                    return;
                }
                let Some(domain) = self.reference(self.source, s(binding, "domain_ref")) else {
                    return;
                };
                if !self.normative_class_pair(variable, domain)
                    || !self.normative_clean_component(self.source, id(domain), false, true)
                {
                    return;
                }
                if !question_binders.is_empty() {
                    let Some(map) = map.as_ref() else {
                        return;
                    };
                    if self.mapped_term(av, bv, map) != Relation::Same {
                        return;
                    }
                    let Some(qbinding) = question_binders
                        .iter()
                        .find(|b| s(b, "variable") == s(bv, "id"))
                    else {
                        return;
                    };
                    let (Some(qvar), Some(qdomain)) = (
                        self.reference(self.question, s(bv, "id")),
                        self.reference(self.question, s(qbinding, "domain_ref")),
                    ) else {
                        return;
                    };
                    if !self.normative_class_pair(variable, qvar)
                        || !self.normative_class_pair(domain, qdomain)
                        || !self.normative_class_pair(qvar, qdomain)
                        || !self.normative_clean_component(self.question, id(qdomain), false, true)
                        || !self.normative_unrestricted_variable(self.question, id(qvar))
                        || !self.normative_named_context(
                            &json!({"kind":"ref","id":id(domain)}),
                            &json!({"kind":"ref","id":id(qdomain)}),
                        )
                    {
                        return;
                    }
                    role_proofs.push(json!({"role":other_role,"kind":"verified_universal_alpha_context","source_binding":binding,"question_binding":qbinding,"source_domain":domain,"question_domain":qdomain}));
                } else {
                    let Some(individual) = self.reference(self.question, s(bv, "id")) else {
                        return;
                    };
                    if source_binders.len() != 1
                        || !self.normative_clean_component(self.source, id(domain), true, true)
                        || s(bv, "kind") != "ref"
                        || s(&individual["name"], "quote").is_empty()
                        || !binding["cardinality"].is_null()
                        || !rows(binding, "relation_closures").is_empty()
                        || !rows(variable, "links").is_empty()
                        || !self.normative_class_pair(domain, individual)
                        || !self.normative_clean_component(
                            self.question,
                            id(individual),
                            false,
                            true,
                        )
                        || !self.normative_named_context(&json!({"kind":"ref","id":id(domain)}), bv)
                    {
                        return;
                    }
                    membership_debt = true;
                    role_proofs.push(json!({"role":other_role,"kind":"universal_source_context_only","source_binding":binding,"source_domain":domain,"question_reference":individual,"domain_membership_proven":false,"individual_instantiation":false}));
                }
            } else if !self.normative_named_role(av, bv) {
                return;
            }
        }
        if rows(atom, "arguments")
            .iter()
            .any(|arg| s(arg, "role") != s(r, "slot") && argument(target, s(arg, "role")).is_none())
            || argument(atom, s(r, "slot")).is_none()
        {
            return;
        }
        // Never carry an unused binder as if it covered a constant operand.
        if source_binders.iter().any(|b| {
            !rows(atom, "arguments").iter().any(|a| {
                s(&a["value"], "kind") == "ref" && s(&a["value"], "id") == s(b, "variable")
            })
        }) {
            return;
        }
        let source_ids = self.claim_qualifiers(c, p);
        let target_ids = self.request_qualifiers(r);
        let common = |q: &Value| {
            s(q, "kind") == "configuration"
                && s(q, "resolution") == "known"
                && q["body"]["key"] == "quantity_extent"
                && q["body"]["value"]["kind"] == "text"
                && q["body"]["value"]["value"] == "común"
        };
        let target_context: Vec<_> = target_ids
            .iter()
            .filter(|qid| {
                !self
                    .qualifier(self.question, qid.as_str().unwrap_or(""))
                    .is_some_and(common)
            })
            .cloned()
            .collect();
        if self
            .context(
                self.source,
                &source_ids,
                self.question,
                &target_context,
                false,
            )
            .0
            != Relation::Same
        {
            return;
        }
        let mut missing = vec![
            "identity".into(),
            "normative_target".into(),
            "answer_slot".into(),
        ];
        if membership_debt {
            missing.push("domain_membership".into());
        }
        let mut witness = self.witness(
            c,
            r,
            p,
            "partial_anonymous_normative_context_v2",
            vec![],
            missing.clone(),
            &[],
        );
        witness["bindings"] = json!([]);
        witness["answer_terms"] = json!([]);
        witness["answer_propositions"] = json!([]);
        witness["source_proposition"] = p.clone();
        witness["source_qualifiers"] = json!(
            source_ids
                .iter()
                .filter_map(|qid| self.qualifier(self.source, qid.as_str()?))
                .collect::<Vec<_>>()
        );
        witness["source_scopes"] = json!(rows(self.source, "scopes"));
        witness["request_target"] = r["target"].clone();
        witness["context_transport"] = json!({"version":"anonymous_normative_context_v1","identity_proven":false,"normative_scope_equal_proven":false,"requested_value_inferred":false,"domain_membership_proven":false,"normative_role":role,"source_scope_reference":a,"question_scope_reference":b,"registered_class_row":self.normative_registered_class(a),"restrictive_context_equal":true,"role_context_proofs":role_proofs,"alpha_map":map,"common_quantity_scope_guard":true});
        out.partial(witness);
        out.unresolved.push(self.candidate(c, r, &missing));
    }
}
#[cfg(test)]
#[path = "anonymous_normative_context_tests.rs"]
mod tests;
