//! Restrictive nominal construction compatibility, never anonymous identity.
use super::*;
impl Engine<'_> {
    pub(super) fn nominal_context_matches(
        &self,
        sd: &Value,
        left: &str,
        qd: &Value,
        right: &str,
        depth: usize,
    ) -> bool {
        if !super::super::request_context_profile::permits(
            self.vocabulary,
            super::super::nominal_de::KIND,
        ) {
            return true;
        }
        let (Some(a), Some(b)) = (
            super::super::nominal_de::targets(sd, left),
            super::super::nominal_de::targets(qd, right),
        ) else {
            return false;
        };
        if a.is_empty() && b.is_empty() {
            return true;
        }
        if depth > 32 {
            return false;
        }
        if a.len() != b.len() {
            return false;
        }
        let mut used = BTreeSet::new();
        for target in a {
            let Some(ak) = super::super::nominal_de::descriptor_key(sd, target) else {
                return false;
            };
            let matched = b.iter().enumerate().find(|(i, other)| {
                if !super::super::context_budget::charge(1, 0) || used.contains(i) {
                    return false;
                }
                super::super::nominal_de::descriptor_key(qd, other).is_some_and(|bk| {
                    ak["class"] == bk["class"]
                        && (ak["semantic_type"].is_null()
                            || bk["semantic_type"].is_null()
                            || ak["semantic_type"] == bk["semantic_type"])
                })
            });
            let Some((i, _)) = matched else {
                return false;
            };
            if !super::super::context_budget::charge(1, 64) {
                return false;
            }
            used.insert(i);
        }
        // Once the nominal construction participates, every additional
        // restrictive link on every alias participates as well.
        fn links<'a>(doc: &'a Value, rid: &str) -> Option<Vec<&'a Value>> {
            let component = super::super::nominal_de::component(doc, rid)?;
            let mut links = Vec::new();
            let mut seen = BTreeSet::new();
            for r in rows(doc, "references") {
                if !super::super::context_budget::charge(1, 0) {
                    return None;
                }
                if !component.contains(id(r)) {
                    continue;
                }
                for link in rows(r, "links") {
                    if !super::super::context_budget::charge(1, 0) {
                        return None;
                    }
                    if link["kind"] != super::super::nominal_de::KIND {
                        let target = super::super::nominal_de::component(doc, s(link, "target"))?;
                        if !seen
                            .iter()
                            .any(|(kind, refs)| kind == s(link, "kind") && refs == &target)
                        {
                            if !super::super::context_budget::charge(
                                1,
                                s(link, "kind").len().saturating_add(144),
                            ) {
                                return None;
                            }
                            seen.insert((s(link, "kind").to_owned(), target));
                            links.push(link);
                        }
                    }
                }
            }
            Some(links)
        }
        let (Some(a), Some(b)) = (links(sd, left), links(qd, right)) else {
            return false;
        };
        if a.len() != b.len() {
            return false;
        }
        let mut used = BTreeSet::new();
        for link in a {
            let matched = b.iter().enumerate().find(|(i, other)| {
                !used.contains(i)
                    && link["kind"] == other["kind"]
                    && self.identity_depth(
                        sd,
                        s(&link, "target"),
                        qd,
                        s(other, "target"),
                        depth + 1,
                    ) == Relation::Same
            });
            let Some((i, _)) = matched else {
                return false;
            };
            if !super::super::context_budget::charge(1, 64) {
                return false;
            }
            used.insert(i);
        }
        true
    }
}
impl Engine<'_> {
    pub(super) fn nominal_metadata_context(
        &self,
        c: &Value,
        r: &Value,
        out: &mut Evidence,
    ) -> bool {
        if !super::super::request_context_profile::permits(
            self.vocabulary,
            super::super::nominal_de::KIND,
        ) {
            return false;
        }
        let original = &self.source_report["document"];
        if !super::super::nominal_de::lexical_metadata(original, c)
            || !super::super::nominal_de::local_metadata_request(self.question, r)
            || !self.context_v2
            || !self.premises(r).is_empty()
            || !self.binders(self.source, s(c, "scope_id")).is_empty()
            || !self.binders(self.question, s(r, "scope_id")).is_empty()
            || !self.branch(self.source, s(c, "scope_id")).is_empty()
            || !self.branch(self.question, s(r, "scope_id")).is_empty()
            || !self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
            || !self
                .dependency_missing(self.question, "request", id(r))
                .is_empty()
            || profile_pending(self.source_report, "claim", id(c))
            || profile_pending(self.question_report, "request", id(r))
        {
            return false;
        }
        let p = &c["body"]["value"];
        let Some(a) = argument(&p["atom"], "subject") else {
            return false;
        };
        let Some(b) = argument(&r["target"]["atom"], "subject") else {
            return false;
        };
        let (Some(ak), Some(bk)) = (
            super::super::nominal_de::descriptor_key(original, id(a)),
            super::super::nominal_de::descriptor_key(self.question, id(b)),
        ) else {
            return false;
        };
        if ak != bk
            || self.scope_match(r, p) != Relation::Same
            || self
                .context(
                    self.source,
                    &self.claim_qualifiers(c, p),
                    self.question,
                    &self.request_qualifiers(r),
                    true,
                )
                .0
                != Relation::Same
        {
            return false;
        }
        fn owners<'a>(doc: &'a Value, tail: &str) -> Vec<&'a Value> {
            let Some(aliases) = super::super::nominal_de::component(doc, tail) else {
                return vec![];
            };
            let mut owners = Vec::new();
            for rf in rows(doc, "references") {
                if !super::super::context_budget::charge(1, 0) {
                    return vec![];
                }
                if rows(rf, "links").iter().any(|l| {
                    super::super::context_budget::charge(1, 0)
                        && l["kind"] == super::super::nominal_de::KIND
                        && aliases.contains(s(l, "target"))
                }) {
                    if !super::super::context_budget::charge(1, std::mem::size_of::<&Value>()) {
                        return vec![];
                    }
                    owners.push(rf);
                }
            }
            owners
        }
        let source_owners = owners(original, id(a));
        let question_owners = owners(self.question, id(b));
        if source_owners.is_empty()
            || question_owners.is_empty()
            || !source_owners.iter().all(|head| {
                !head["name"].is_null()
                    && self
                        .dependency_missing(self.source, "reference", id(head))
                        .is_empty()
                    && question_owners.iter().any(|wanted| {
                        !wanted["name"].is_null()
                            && self
                                .dependency_missing(self.question, "reference", id(wanted))
                                .is_empty()
                            && self.identity(self.source, id(head), self.question, id(wanted))
                                == Relation::Same
                    })
            })
        {
            return false;
        }
        // Reserve the complete retained payload before any Value copies.
        let values = [
            c,
            c,
            r,
            r,
            &original["references"],
            &self.question["references"],
            &original["scopes"],
            &self.question["scopes"],
            &original["qualifiers"],
            &self.question["qualifiers"],
        ];
        let bytes = values
            .into_iter()
            .chain(source_owners.iter().copied())
            .chain(question_owners.iter().copied())
            .try_fold(4096usize, |n, v| {
                n.checked_add(super::super::context_budget::serialized_bytes(v)?)
            })
            .unwrap_or(usize::MAX);
        if !super::super::context_budget::charge(1, bytes) {
            return false;
        }
        let mut w = self.witness(
            c,
            r,
            p,
            "local_nominal_metadata_context",
            vec![],
            vec!["local_nominal_metadata".into(), "answer_slot".into()],
            &[],
        );
        w["bindings"] = json!([]);
        w["answer_terms"] = json!([]);
        w["answer_propositions"] = json!([]);
        w["source_constraint"] = c.clone();
        w["request_constraint"] = r.clone();
        w["source_references"] = original["references"].clone();
        w["question_references"] = self.question["references"].clone();
        w["source_scopes"] = original["scopes"].clone();
        w["question_scopes"] = self.question["scopes"].clone();
        w["source_qualifiers"] = original["qualifiers"].clone();
        w["question_qualifiers"] = self.question["qualifiers"].clone();
        w["nominal_context"] = json!({"kind":super::super::nominal_de::KIND,"source_owners":source_owners,"question_owners":question_owners,"identity_proven":false,"requested_value_inferred":false,"global_property_inferred":false});
        out.partial(w);
        true
    }
}

impl Engine<'_> {
    pub(super) fn nominal_pending_applies(&self, c: &Value, r: &Value) -> bool {
        if !super::super::request_context_profile::permits(
            self.vocabulary,
            super::super::nominal_de::KIND,
        ) {
            return self.pending_profile_applies(c, r);
        }
        let original = &self.source_report["document"];
        let p = &c["body"]["value"];
        let target = &r["target"]["atom"];
        if super::super::nominal_de::lexical_metadata(original, c) {
            if target["predicate"] != p["atom"]["predicate"] {
                return false;
            }
            let (Some(a), Some(b)) = (
                argument(&p["atom"], "attribute"),
                argument(target, "attribute"),
            ) else {
                return false;
            };
            if a["kind"] != b["kind"] || a["id"] != b["id"] {
                return false;
            }
            let (Some(a), Some(b)) = (argument(&p["atom"], "subject"), argument(target, "subject"))
            else {
                return false;
            };
            let (Some(a), Some(b)) = (
                self.reference(original, id(a)),
                self.reference(self.question, id(b)),
            ) else {
                return false;
            };
            return !a["declared_class"].is_null()
                && a["declared_class"]["quote"] == b["declared_class"]["quote"];
        }
        for a in rows(&p["atom"], "arguments") {
            if a["value"]["kind"] != "ref" {
                continue;
            }
            if let Some(b) = argument(target, s(a, "role")) {
                if b["kind"] == "ref" && !self.partial_named_anchor(s(&a["value"], "id"), id(b), 0)
                {
                    return false;
                }
            }
        }
        self.pending_profile_applies(c, r)
    }
}
