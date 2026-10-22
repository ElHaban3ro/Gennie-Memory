//! Source-only supply operands, never an inferred final upstream origin.
use super::*;
impl Engine<'_> {
    fn water_clean_ref(&self, doc: &Value, term: &Value, anonymous: bool) -> bool {
        if s(term, "kind") != "ref"
            || !self.description_component_agreement(doc, s(term, "id"), doc, s(term, "id"))
        {
            return false;
        }
        rows(doc, "references")
            .iter()
            .filter(|rf| self.local(doc, s(term, "id"), id(rf)) == Relation::Same)
            .all(|rf| {
                rf["anaphora"].is_null()
                    && rf["binding_scope_id"].is_null()
                    && self.branch(doc, s(rf, "scope_id")).is_empty()
                    && self.binders(doc, s(rf, "scope_id")).is_empty()
                    && (!anonymous || rf["name"].is_null())
                    && self.dependency_missing(doc, "reference", id(rf)).is_empty()
            })
    }
    fn water_local_links(&self, doc: &Value, term: &Value, destination: Option<&Value>) -> bool {
        self.water_clean_ref(doc, term, false)
            && rows(doc, "references")
                .iter()
                .filter(|rf| self.local(doc, s(term, "id"), id(rf)) == Relation::Same)
                .all(|rf| {
                    rows(rf, "links").iter().all(|link| {
                        destination.is_some_and(|d| {
                            s(link, "kind") == "associated_with"
                                && self.local(doc, s(link, "target"), s(d, "id")) == Relation::Same
                        })
                    })
                })
    }
    fn water_claim_ready(&self, c: &Value) -> bool {
        let p = &c["body"]["value"];
        s(&c["body"], "kind") == "proposition"
            && s(p, "judgment") == "fact"
            && s(p, "status") == "state"
            && s(&p["atom"], "polarity") == "positive"
            && p["normative_scope"].is_null()
            && matches!(s(p, "basis"), "assignment" | "description" | "observation")
            && self.branch(self.source, s(c, "scope_id")).is_empty()
            && self.binders(self.source, s(c, "scope_id")).is_empty()
            && self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
            && !rows(self.source_report, "candidates").iter().any(|v| {
                !matches!(s(v, "state"), "eligible" | "conditional")
                    && rows(v, "claim_ids").contains(&c["id"])
            })
            && !rows(self.source, "claims").iter().any(|other| {
                let op = &other["body"]["value"];
                s(&other["body"], "kind") == "proposition"
                    && s(op, "judgment") == "fact"
                    && s(op, "status") == "state"
                    && s(&op["atom"], "polarity") == "negative"
                    && op["basis"] == p["basis"]
                    && op["normative_scope"] == p["normative_scope"]
                    && self.branch(self.source, s(other, "scope_id")).is_empty()
                    && self.binders(self.source, s(other, "scope_id")).is_empty()
                    && self
                        .dependency_missing(self.source, "claim", id(other))
                        .is_empty()
                    && self.atom(self.source, &p["atom"], self.source, &op["atom"], None)
                        == Relation::Same
                    && self
                        .context(
                            self.source,
                            &self.claim_qualifiers(c, p),
                            self.source,
                            &self.claim_qualifiers(other, op),
                            true,
                        )
                        .0
                        == Relation::Same
            })
    }
    fn water_operand_pair(&self, a: &Value, b: &Value, ad: &Value, bd: &Value) -> bool {
        if !self.water_clean_ref(self.source, a, true)
            || !self.water_clean_ref(self.question, b, true)
            || !self.water_local_links(self.source, a, Some(ad))
            || !self.water_local_links(self.question, b, Some(bd))
            || !self.description_component_agreement(
                self.source,
                s(a, "id"),
                self.question,
                s(b, "id"),
            )
        {
            return false;
        }
        let (Some(ar), Some(br)) = (
            self.reference(self.source, s(a, "id")),
            self.reference(self.question, s(b, "id")),
        ) else {
            return false;
        };
        !s(&ar["declared_class"], "quote").is_empty()
            && ar["declared_class"]["quote"] == br["declared_class"]["quote"]
    }
    fn water_conditions(&self, c: &Value, r: &Value) -> Option<Vec<Value>> {
        let p = &c["body"]["value"];
        let source = self.claim_qualifiers(c, p);
        let mut target = vec![];
        let mut pending = vec![];
        for qid in self.request_qualifiers(r) {
            let q = self.qualifier(self.question, qid.as_str()?)?;
            if s(q, "kind") == "configuration" && s(&q["body"], "key") == "purpose" {
                if s(q, "resolution") != "known"
                    || s(&q["body"]["value"], "kind") != "text"
                    || s(&q["body"]["value"], "value").is_empty()
                {
                    return None;
                }
                // If the source states purpose, it must match. Only absence is debt.
                if source
                    .iter()
                    .filter_map(|x| self.qualifier(self.source, x.as_str()?))
                    .any(|sq| {
                        s(sq, "kind") == "configuration" && sq["body"]["key"] == q["body"]["key"]
                    })
                {
                    target.push(qid);
                } else {
                    pending.push(q.clone());
                }
            } else {
                target.push(qid);
            }
        }
        (self
            .context(self.source, &source, self.question, &target, true)
            .0
            == Relation::Same)
            .then_some(pending)
    }
    pub(super) fn water_operand_context(&self, c: &Value, r: &Value, out: &mut Evidence) {
        let p = &c["body"]["value"];
        let atom = &p["atom"];
        let target = &r["target"]["atom"];
        if !self.context_v2
            || !self.water_claim_ready(c)
            || s(r, "mode") != "answer"
            || s(r, "slot") != "origin"
            || s(target, "predicate") != "upstream_origin"
            || s(target, "polarity") != "positive"
            || s(&r["target"], "judgment") != "fact"
            || s(&r["target"], "status") != "state"
            || !r["normative_scope_filter"].is_null()
            || !self.branch(self.question, s(r, "scope_id")).is_empty()
            || !self.binders(self.question, s(r, "scope_id")).is_empty()
            || !self
                .dependency_missing(self.question, "request", id(r))
                .is_empty()
            || self.scope_match(r, p) != Relation::Same
        {
            return;
        }
        let (Some(destination), Some(wanted_destination), Some(water)) = (
            argument(atom, "destination"),
            argument(target, "destination"),
            argument(target, "substance"),
        ) else {
            return;
        };
        if !self.water_clean_ref(self.source, destination, false)
            || !self.water_clean_ref(self.question, wanted_destination, false)
            || self
                .description_anchor_evidence(s(destination, "id"), s(wanted_destination, "id"))
                .is_none()
            || !self.admission_setting_context(destination, wanted_destination, false)
            || !self.water_clean_ref(self.question, water, true)
            || !self.water_local_links(self.question, water, Some(wanted_destination))
        {
            return;
        }
        let Some(water_ref) = self.reference(self.question, s(water, "id")) else {
            return;
        };
        // The domain is declared by the vocabulary, never guessed from a supplier.
        let Some(profile) = rows(&self.vocabulary["relevance_profiles"], "rows")
            .iter()
            .find(|p| s(p, "link_id") == "receiver_context_for_origin")
        else {
            return;
        };
        let domain_guards: Vec<_> = rows(profile, "alternatives")
            .iter()
            .filter(|g| {
                s(g, "kind") == "class_context"
                    && s(g, "source_role") == "source"
                    && s(g, "target_role") == "substance"
                    && rows(g, "target_classes").contains(&water_ref["declared_class"]["quote"])
            })
            .collect();
        if domain_guards.is_empty() {
            return;
        }
        let (link_id, provider_role) = match s(atom, "predicate") {
            "supplied_by" => ("supply_context_for_origin", "supplier"),
            "receives_from" => ("receiver_context_for_origin", "source"),
            _ => return,
        };
        let Some(link) = rows(self.vocabulary, "relevance_links")
            .iter()
            .find(|link| {
                s(link, "id") == link_id
                    && link["source_predicate"] == atom["predicate"]
                    && link["target_predicate"] == target["predicate"]
            })
        else {
            return;
        };
        let expected = if provider_role == "supplier" {
            json!([
                ["source.substance", "target.substance"],
                ["source.destination", "target.destination"]
            ])
        } else {
            json!([["source.destination", "target.destination"]])
        };
        if rows(link, "bindings").len() != expected.as_array().unwrap().len()
            || !expected
                .as_array()
                .unwrap()
                .iter()
                .all(|binding| rows(link, "bindings").contains(binding))
            || link["carry_roles"] != json!([format!("source.{provider_role}")])
            || link["conclusion"] != "partial_intermediate_only"
            || link["allow_transitive_composition"] != false
            || link["allow_reverse"] != false
            || link["max_hops"] != 1
        {
            return;
        }
        let Some(provider) = argument(atom, provider_role) else {
            return;
        };
        if !self.water_local_links(self.source, provider, None) {
            return;
        }
        let Some(pending_purpose) = self.water_conditions(c, r) else {
            return;
        };
        let mut request_premises = vec![];
        for pid in rows(r, "premise_ids") {
            let Some(pc) = rows(self.question, "claims")
                .iter()
                .find(|x| x["id"] == *pid)
            else {
                return;
            };
            let pp = &pc["body"]["value"];
            let Some(object) = argument(&pp["atom"], "object") else {
                return;
            };
            let Some(purpose) = argument(&pp["atom"], "purpose") else {
                return;
            };
            if s(&pc["body"], "kind") != "proposition"
                || s(&pp["atom"], "predicate") != "for_purpose"
                || s(&pp["atom"], "polarity") != "positive"
                || s(pp, "judgment") != "fact"
                || s(pp, "status") != "state"
                || !pp["normative_scope"].is_null()
                || s(object, "kind") != "ref"
                || self.local(self.question, s(object, "id"), s(water, "id")) != Relation::Same
                || !self.water_local_links(self.question, purpose, None)
                || !self.claim_qualifiers(pc, pp).is_empty()
                || !self.branch(self.question, s(pc, "scope_id")).is_empty()
                || !self.binders(self.question, s(pc, "scope_id")).is_empty()
                || !self
                    .dependency_missing(self.question, "claim", id(pc))
                    .is_empty()
            {
                return;
            }
            request_premises.push(pc.clone());
        }
        let mut participating = vec![c];
        if provider_role == "supplier" {
            let Some(actual_water) = argument(atom, "substance") else {
                return;
            };
            if !self.water_operand_pair(actual_water, water, destination, wanted_destination) {
                return;
            }
        } else {
            let Some(provider_ref) = self.reference(self.source, s(provider, "id")) else {
                return;
            };
            let class_proof = domain_guards.iter().any(|g| {
                rows(g, "source_classes").contains(&provider_ref["declared_class"]["quote"])
            });
            if !class_proof {
                let expected = json!([
                    ["source.source", "evidence.destination"],
                    ["target.substance", "evidence.substance"]
                ]);
                if !rows(profile, "alternatives").iter().any(|guard| {
                    s(guard, "kind") == "corroborating_claim"
                        && s(guard, "predicate") == "upstream_origin"
                        && rows(guard, "bindings").len() == 2
                        && expected
                            .as_array()
                            .unwrap()
                            .iter()
                            .all(|binding| rows(guard, "bindings").contains(binding))
                }) {
                    return;
                }
                let Some(aux) = rows(self.source, "claims").iter().find(|aux| {
                    let ap = &aux["body"]["value"];
                    let aa = &ap["atom"];
                    if !self.water_claim_ready(aux)
                        || s(aa, "predicate") != "upstream_origin"
                        || self.water_conditions(aux, r).is_none()
                    {
                        return false;
                    }
                    let (Some(ad), Some(aw), Some(origin)) = (
                        argument(aa, "destination"),
                        argument(aa, "substance"),
                        argument(aa, "origin"),
                    ) else {
                        return false;
                    };
                    s(ad, "kind") == "ref"
                        && self.local(self.source, s(ad, "id"), s(provider, "id")) == Relation::Same
                        && self.water_operand_pair(aw, water, ad, wanted_destination)
                        && self.water_local_links(self.source, origin, None)
                        && self
                            .context(
                                self.source,
                                &self.claim_qualifiers(c, p),
                                self.source,
                                &self.claim_qualifiers(aux, ap),
                                true,
                            )
                            .0
                            == Relation::Same
                }) else {
                    return;
                };
                participating.push(aux);
            }
        }
        let mut missing = vec![
            "answer_slot".into(),
            "identity".into(),
            "source_origin_not_proven".into(),
        ];
        if !pending_purpose.is_empty() {
            missing.push("purpose".into());
        }
        if !request_premises.is_empty() {
            missing.push("premise".into());
        }
        if s(&r["basis_requirement"], "kind") == "exact"
            && r["basis_requirement"]["value"] != p["basis"]
        {
            missing.push("basis".into());
        }
        unique(&mut missing);
        let extra: Vec<_> = participating
            .iter()
            .skip(1)
            .map(|x| id(x).to_owned())
            .collect();
        let mut w = self.witness(
            c,
            r,
            p,
            "partial_water_operand_context_v2",
            vec![],
            missing.clone(),
            &extra,
        );
        w["answer_terms"] = json!([]);
        w["answer_propositions"] = json!([]);
        w["bindings"] = json!([]);
        w["source_propositions"] = json!(
            participating
                .iter()
                .map(|c| &c["body"]["value"])
                .collect::<Vec<_>>()
        );
        w["source_scopes"] = self.source["scopes"].clone();
        w["source_qualifiers"] = json!(
            participating
                .iter()
                .flat_map(|c| self.claim_qualifiers(c, &c["body"]["value"]))
                .filter_map(|qid| self.qualifier(self.source, qid.as_str()?))
                .collect::<Vec<_>>()
        );
        w["request_premises"] = json!(request_premises);
        w["request_target"] = r["target"].clone();
        w["context_transport"] = json!({"version":"water_operand_context_v1","link_id":link_id,"identity_proven":false,"requested_origin_inferred":false,"purpose_proven":false,"pending_purpose_qualifiers":pending_purpose,"named_anchor":self.description_anchor_evidence(s(destination,"id"),s(wanted_destination,"id")),"local_operand_binding_only":true});
        out.partial(w);
        out.unresolved.push(self.candidate(c, r, &missing));
    }
}
#[cfg(test)]
#[path = "water_operand_context_tests.rs"]
mod tests;
