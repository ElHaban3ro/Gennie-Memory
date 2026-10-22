//! Operator-aware context preserves the open slot and never publishes an answer.
use super::super::nominal_possessive::{self, KIND};
use super::*;
impl Engine<'_> {
    pub(super) fn possessive_reference(&self, doc: &Value, rid: &str) -> bool {
        let blocked = if std::ptr::eq(doc, self.question) {
            self.possessive_question
        } else {
            self.possessive_source
        };
        nominal_possessive::has(blocked, "reference", rid)
    }
    fn possessive_head_matches(&self, left: &str, right: &str) -> bool {
        let sd = &self.source_report["document"];
        let qd = &self.question_report["document"];
        let (Some(sa), Some(qa)) = (
            nominal_possessive::component(sd, left),
            nominal_possessive::component(qd, right),
        ) else {
            return false;
        };
        let mut found = false;
        for a in rows(sd, "references").iter().filter(|a| sa.contains(id(a))) {
            if !super::super::context_budget::charge(1, 0) {
                return false;
            }
            if a["denotation"] != "entity"
                || !a["binding_scope_id"].is_null()
                || !a["anaphora"].is_null()
            {
                return false;
            }
            for b in rows(qd, "references").iter().filter(|b| qa.contains(id(b))) {
                if !super::super::context_budget::charge(1, 0) {
                    return false;
                }
                if b["denotation"] != "entity"
                    || !b["binding_scope_id"].is_null()
                    || !b["anaphora"].is_null()
                {
                    return false;
                }
                if !a["declared_class"].is_null()
                    && a["declared_class"]["quote"] != b["declared_class"]["quote"]
                {
                    return false;
                }
                if !a["name"].is_null() && a["name"]["quote"] != b["name"]["quote"] {
                    return false;
                }
                if !a["semantic_type"].is_null()
                    && a["semantic_type"]["type_id"] != b["semantic_type"]["type_id"]
                {
                    return false;
                }
                found |= !a["declared_class"].is_null() || !a["name"].is_null();
            }
        }
        // Every link on every alias is retained and compared. Only the open
        // determiner versus a requested owned_by restriction carries identity debt.
        if !super::super::context_budget::charge(
            1,
            super::super::context_budget::serialized_bytes(&sd["references"])
                .unwrap_or(usize::MAX)
                .saturating_add(
                    super::super::context_budget::serialized_bytes(&qd["references"])
                        .unwrap_or(usize::MAX),
                ),
        ) {
            return false;
        }
        let source_links: Vec<_> = rows(sd, "references")
            .iter()
            .filter(|a| sa.contains(id(a)))
            .flat_map(|a| rows(a, "links"))
            .collect();
        let question_links: Vec<_> = rows(qd, "references")
            .iter()
            .filter(|b| qa.contains(id(b)))
            .flat_map(|b| rows(b, "links"))
            .collect();
        if !source_links.iter().any(|l| l["kind"] == KIND) {
            return false;
        }
        for (links, others, source_side) in [
            (&source_links, &question_links, true),
            (&question_links, &source_links, false),
        ] {
            for l in links.iter().copied() {
                if !super::super::context_budget::charge(1, 0) {
                    return false;
                }
                if l["kind"] == KIND {
                    continue;
                }
                if !source_side
                    && l["kind"] == "owned_by"
                    && !source_links.iter().any(|l| l["kind"] == "owned_by")
                {
                    let Some(owner) = self.reference(qd, s(l, "target")) else {
                        return false;
                    };
                    if owner["name"].is_null() || self.possessive_reference(qd, id(owner)) {
                        return false;
                    }
                    continue;
                }
                let matched = others.iter().any(|other| {
                    if !super::super::context_budget::charge(1, 0) || l["kind"] != other["kind"] {
                        return false;
                    }
                    let (a, b) = if source_side {
                        (l, *other)
                    } else {
                        (*other, l)
                    };
                    if l["kind"] == super::super::nominal_de::KIND {
                        let (Some(a), Some(b)) = (
                            super::super::nominal_de::descriptor_key(sd, s(a, "target")),
                            super::super::nominal_de::descriptor_key(qd, s(b, "target")),
                        ) else {
                            return false;
                        };
                        a == b
                    } else {
                        self.identity(sd, s(a, "target"), qd, s(b, "target")) == Relation::Same
                    }
                });
                if !matched {
                    return false;
                }
            }
        }
        found
    }
    /// One declared relevance row supplies the entire correlated assignment.
    /// Bindings from different rows can never be combined into a new rule.
    fn possessive_arguments_match(
        &self,
        sd: &Value,
        a: &Value,
        qd: &Value,
        b: &Value,
        link: Option<&Value>,
    ) -> bool {
        let mut external = false;
        let mut carrier = false;
        for arg in rows(a, "arguments") {
            if !super::super::context_budget::charge(1, 0) {
                return false;
            }
            let role = s(arg, "role");
            let other = if let Some(link) = link {
                let mut matches = rows(link, "bindings").iter().filter(|pair| {
                    super::super::context_budget::charge(1, 0)
                        && pair[0].as_str().and_then(|s| s.strip_prefix("source.")) == Some(role)
                });
                let Some(pair) = matches.next() else {
                    return false;
                };
                if matches.next().is_some() {
                    return false;
                }
                pair[1]
                    .as_str()
                    .and_then(|s| s.strip_prefix("target."))
                    .and_then(|role| argument(b, role))
            } else {
                argument(b, role)
            };
            let Some(other) = other else {
                return false;
            };
            let value = &arg["value"];
            if value["kind"] == "ref" && self.possessive_reference(sd, id(value)) {
                if other["kind"] != "ref" || !self.possessive_head_matches(id(value), id(other)) {
                    return false;
                }
                carrier = true;
            } else {
                if self.term(sd, value, qd, other) != Relation::Same {
                    return false;
                }
                external |= value["kind"] == "ref"
                    && self
                        .reference(sd, id(value))
                        .is_some_and(|r| !r["name"].is_null());
            }
        }
        carrier && external
    }
    fn possessive_partial(&self, c: &Value, r: &Value, out: &mut Evidence) -> bool {
        let sd = &self.source_report["document"];
        let qd = &self.question_report["document"];
        let p = &c["body"]["value"];
        let target = &r["target"];
        if !self.context_v2
            || c["body"]["kind"] != "proposition"
            || p["judgment"] != target["judgment"]
            || p["status"] != target["status"]
            || p["atom"]["polarity"] != target["atom"]["polarity"]
            || !self.premises(r).is_empty()
            || !self.branch(sd, s(c, "scope_id")).is_empty()
            || !self.branch(qd, s(r, "scope_id")).is_empty()
            || !self.binders(sd, s(c, "scope_id")).is_empty()
            || !self.binders(qd, s(r, "scope_id")).is_empty()
            || !self.dependency_missing(sd, "claim", id(c)).is_empty()
            || !self.dependency_missing(qd, "request", id(r)).is_empty()
            || profile_pending(self.source_report, "claim", id(c))
            || profile_pending(self.question_report, "request", id(r))
            || self.scope_match(r, p) != Relation::Same
            || self
                .context(
                    sd,
                    &self.claim_qualifiers(c, p),
                    qd,
                    &self.request_qualifiers(r),
                    true,
                )
                .0
                != Relation::Same
            || s(&r["basis_requirement"], "kind") == "exact"
                && r["basis_requirement"]["value"] != p["basis"]
        {
            return false;
        }
        for (report, kind, node) in [
            (self.source_report, "claim", id(c)),
            (self.question_report, "request", id(r)),
        ] {
            if rows(report, "candidates").iter().any(|candidate| {
                let applies = if kind == "claim" {
                    rows(candidate, "claim_ids")
                        .iter()
                        .any(|v| v.as_str() == Some(node))
                } else {
                    candidate["request_id"] == node
                };
                applies
                    && (!rows(candidate, "reason_codes").is_empty()
                        || !matches!(s(candidate, "state"), "eligible" | "conditional"))
            }) {
                return false;
            }
        }
        let a = &p["atom"];
        let b = &target["atom"];
        let compatible = if a["predicate"] == b["predicate"] {
            self.possessive_arguments_match(sd, a, qd, b, None)
        } else {
            rows(self.vocabulary, "relevance_links").iter().any(|link| {
                super::super::context_budget::charge(1, 0)
                    && link["source_predicate"] == a["predicate"]
                    && link["target_predicate"] == b["predicate"]
                    && self.possessive_arguments_match(sd, a, qd, b, Some(link))
            })
        };
        if !compatible {
            return false;
        }
        // Reserve all duplicated document arrays, full originals, constraints,
        // ordinary witness metadata and ID inventories before copying anything.
        let bytes = [sd, qd, c, r]
            .into_iter()
            .try_fold(4096usize, |n, v| {
                n.checked_add(super::super::context_budget::serialized_bytes(v)?.checked_mul(4)?)
            })
            .unwrap_or(usize::MAX);
        if !super::super::context_budget::charge(1, bytes) {
            return false;
        }
        let mut owners = Vec::new();
        let mut uses = Vec::new();
        for reference in rows(sd, "references") {
            for link in rows(reference, "links") {
                if link["kind"] == KIND {
                    owners.push(reference["id"].clone());
                    uses.push(link["target"].clone());
                }
            }
        }
        let mut w = self.witness(
            c,
            r,
            p,
            "nominal_possessive_context",
            vec![],
            vec!["nominal_possessive_referent".into(), "answer_slot".into()],
            &[],
        );
        w["bindings"] = json!([]);
        w["answer_terms"] = json!([]);
        w["answer_propositions"] = json!([]);
        w["source_constraint"] = c.clone();
        w["request_constraint"] = r.clone();
        w["source_document"] = sd.clone();
        w["question_document"] = qd.clone();
        for (outkey, doc, key) in [
            ("source_references", sd, "references"),
            ("question_references", qd, "references"),
            ("source_scopes", sd, "scopes"),
            ("question_scopes", qd, "scopes"),
            ("source_qualifiers", sd, "qualifiers"),
            ("question_qualifiers", qd, "qualifiers"),
            ("source_dependencies", sd, "dependencies"),
            ("question_dependencies", qd, "dependencies"),
        ] {
            w[outkey] = doc[key].clone();
        }
        w["possessive_context"] = json!({"version":"documentary_nominal_possessive_context_v1","source_owners":owners,"source_uses":uses,"referent_resolution":"source_indeterminate","possessive_referent_proven":false});
        out.partial(w);
        true
    }
    pub(super) fn possessive_candidates(&self, r: &Value, out: &mut Evidence) {
        if !super::super::context_budget::possessive_enabled() {
            return;
        }
        for c in self
            .pending_claims
            .iter()
            .filter(|c| nominal_possessive::has(self.possessive_source, "claim", id(c)))
        {
            if self.possessive_partial(c, r, out) {
                continue;
            }
            let body = &c["body"];
            let p = if body["kind"] == "rule" {
                &body["action"]
            } else {
                &body["value"]
            };
            let source_atom = if body["kind"] == "value_restriction" {
                &body["target"]
            } else {
                &p["atom"]
            };
            let connected = if body["kind"] == "event" {
                !body["event_kind"].is_null() && body["event_kind"] == r["target"]["event_kind"]
            } else {
                !source_atom["predicate"].is_null()
                    && source_atom["predicate"] == r["target"]["atom"]["predicate"]
            };
            if !connected {
                continue;
            }
            // Known mismatching external anchors or nominal classes do not
            // contaminate independent claims sharing the same predicate.
            let incompatible = rows(source_atom, "arguments").iter().any(|a| {
                let Some(b) = argument(&r["target"]["atom"], s(a, "role")) else {
                    return false;
                };
                if a["value"]["kind"] != "ref" || b["kind"] != "ref" {
                    return false;
                }
                let (Some(a), Some(b)) = (
                    self.reference(&self.source_report["document"], id(&a["value"])),
                    self.reference(self.question, id(b)),
                ) else {
                    return false;
                };
                (!a["name"].is_null()
                    && !b["name"].is_null()
                    && a["name"]["quote"] != b["name"]["quote"])
                    || (!a["declared_class"].is_null()
                        && !b["declared_class"].is_null()
                        && a["declared_class"]["quote"] != b["declared_class"]["quote"])
            });
            if incompatible {
                continue;
            }
            let bytes = [
                &self.source_report["document"],
                &self.question_report["document"],
                c,
                r,
            ]
            .into_iter()
            .try_fold(2048usize, |n, v| {
                n.checked_add(super::super::context_budget::serialized_bytes(v)?)
            })
            .unwrap_or(usize::MAX);
            if !super::super::context_budget::charge(1, bytes) {
                return;
            }
            let mut candidate = self.candidate(c, r, &["nominal_possessive_referent".into()]);
            candidate["source_constraint"] = c.clone();
            candidate["request_constraint"] = r.clone();
            candidate["source_document"] = self.source_report["document"].clone();
            candidate["question_document"] = self.question_report["document"].clone();
            out.unresolved.push(candidate);
        }
    }
    pub(super) fn possessive_request(&self, r: &Value) -> Value {
        let mut out = Evidence::default();
        self.possessive_candidates(r, &mut out);
        if out.partial.is_empty() && out.unresolved.is_empty() {
            if super::super::context_budget::charge(
                1,
                super::super::context_budget::serialized_bytes(r)
                    .unwrap_or(usize::MAX)
                    .saturating_add(2048),
            ) {
                out.unresolved.push(json!({"claim_ids":[],"request_id":id(r),"state":"unresolved","reason_codes":["nominal_possessive_referent"],"request_constraint":r}));
            }
        }
        json!({"request_id":id(r),"support":[],"refutation":[],"partial":out.partial,"unresolved":out.unresolved,"conflicts":[],"missing_constraints":["nominal_possessive_referent","answer_slot"],"coverage":if out.partial.is_empty(){"none"}else{"partial"}})
    }
}
