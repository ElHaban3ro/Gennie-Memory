//! Description comparison is local to an operand; it never enriches identity or facts.
use super::*;
impl Engine<'_> {
    // Agreement is required for descriptive matching, not a declaration that
    // distinct lexical classes are disjoint. Unreconciled labels stay pending.
    pub(super) fn description_component_agreement(
        &self,
        sd: &Value,
        left: &str,
        qd: &Value,
        right: &str,
    ) -> bool {
        let mut classes = BTreeSet::new();
        let mut types = BTreeSet::new();
        let mut denotations = BTreeSet::new();
        for (doc, root) in [(sd, left), (qd, right)] {
            if self.local(doc, root, root) != Relation::Same {
                return false;
            }
            let component: Vec<_> = rows(doc, "references")
                .iter()
                .filter(|r| self.local(doc, root, id(r)) == Relation::Same)
                .collect();
            if component.is_empty() {
                return false;
            }
            for r in component {
                if s(&r["anaphora"], "status") == "unresolved" {
                    return false;
                }
                let class = s(&r["declared_class"], "quote");
                if !class.is_empty() {
                    classes.insert(class.to_owned());
                }
                let typ = s(&r["semantic_type"], "type_id");
                if !typ.is_empty() {
                    types.insert(typ.to_owned());
                }
                denotations.insert(s(r, "denotation").to_owned());
            }
        }
        classes.len() <= 1 && types.len() <= 1 && denotations.len() == 1
    }

    // This records documentary name correspondence, not description equality.
    // The legacy anonymous/universal shortcut is not a new positive root anchor.
    pub(super) fn description_anchor_credited(&self, actual: &str, wanted: &str) -> bool {
        self.description_anchor_evidence(actual, wanted).is_some()
    }
    pub(super) fn description_anchor_evidence(&self, actual: &str, wanted: &str) -> Option<Value> {
        if !self.description_component_agreement(self.source, actual, self.question, wanted)
            || !self.partial_named_anchor(actual, wanted, 0)
        {
            return None;
        }
        for a in rows(self.source, "references")
            .iter()
            .filter(|a| self.local(self.source, actual, id(a)) == Relation::Same)
        {
            let name = s(&a["name"], "quote");
            if name.is_empty() {
                continue;
            }
            for b in rows(self.question, "references")
                .iter()
                .filter(|b| self.local(self.question, wanted, id(b)) == Relation::Same)
            {
                if name == s(&b["name"], "quote")
                    && a["denotation"] == b["denotation"]
                    && (a["declared_class"].is_null()
                        || b["declared_class"].is_null()
                        || a["declared_class"]["quote"] == b["declared_class"]["quote"])
                    && (a["semantic_type"].is_null()
                        || b["semantic_type"].is_null()
                        || a["semantic_type"]["type_id"] == b["semantic_type"]["type_id"])
                {
                    return Some(
                        json!({"source_reference_id":id(a),"question_reference_id":id(b),"source_name":a["name"],"question_name":b["name"]}),
                    );
                }
            }
        }
        None
    }

    // Every alias inherits restrictive links from its entire explicit Same
    // component. Repeated mentions of the same edge add no new restriction.
    fn description_effective_links<'a>(&self, doc: &'a Value, root: &str) -> Vec<&'a Value> {
        let mut links: Vec<&Value> = Vec::new();
        for r in rows(doc, "references")
            .iter()
            .filter(|r| self.local(doc, root, id(r)) == Relation::Same)
        {
            for link in rows(r, "links") {
                if !links.iter().any(|old| {
                    old["kind"] == link["kind"]
                        && self.local(doc, s(old, "target"), s(link, "target")) == Relation::Same
                }) {
                    links.push(link);
                }
            }
        }
        links
    }

    // Resolve only a unique pair of already verified bound roots. Alias edges
    // remain local proof, never an expanded global identity map.
    pub(super) fn description_alpha_roots(
        &self,
        actual: &str,
        wanted: &str,
        map: &BTreeMap<String, String>,
    ) -> Option<(String, String)> {
        if !self.description_component_agreement(self.source, actual, self.question, wanted) {
            return None;
        }
        let left: Vec<_> = map
            .iter()
            .filter(|(a, _)| self.local(self.source, actual, a) == Relation::Same)
            .collect();
        let right: Vec<_> = map
            .iter()
            .filter(|(_, b)| self.local(self.question, wanted, b) == Relation::Same)
            .collect();
        if left.len() != 1 || right.len() != 1 || left[0] != right[0] {
            return None;
        }
        let (a, b) = left[0];
        if self.mapped_term(
            &json!({"kind":"ref","id":a}),
            &json!({"kind":"ref","id":b}),
            map,
        ) != Relation::Same
        {
            return None;
        }
        if actual != a || wanted != b {
            let ar = self.reference(self.source, actual)?;
            let br = self.reference(self.question, wanted)?;
            let source_links = self.description_effective_links(self.source, actual);
            let question_links = self.description_effective_links(self.question, wanted);
            if ar["name"]["quote"] != br["name"]["quote"]
                || source_links.len() != question_links.len()
            {
                return None;
            }
            let mut used = BTreeSet::new();
            for link in source_links {
                let found = question_links.iter().enumerate().find(|(index, other)| {
                    !used.contains(index)
                        && link["kind"] == other["kind"]
                        && self.description_component_agreement(
                            self.source,
                            s(link, "target"),
                            self.question,
                            s(other, "target"),
                        )
                        && self.mapped_term(
                            &json!({"kind":"ref","id":link["target"]}),
                            &json!({"kind":"ref","id":other["target"]}),
                            map,
                        ) == Relation::Same
                });
                let (index, _) = found?;
                used.insert(index);
            }
        }
        Some((a.clone(), b.clone()))
    }

    fn descriptor_atom(
        &self,
        sd: &Value,
        a: &Value,
        qd: &Value,
        b: &Value,
        map: &BTreeMap<String, String>,
        omit: Option<&str>,
    ) -> Relation {
        if a["predicate"] != b["predicate"] || a["polarity"] != b["polarity"] {
            return Relation::Unknown;
        }
        let aa = rows(a, "arguments");
        let ba = rows(b, "arguments");
        if aa.len() != ba.len() {
            return Relation::Unknown;
        }
        let mut result = Relation::Same;
        for x in aa {
            if Some(s(x, "role")) == omit {
                continue;
            }
            let Some(y) = ba.iter().find(|y| y["role"] == x["role"]) else {
                return Relation::Unknown;
            };
            let av = &x["value"];
            let bv = &y["value"];
            if s(av, "kind") == "ref"
                && s(bv, "kind") == "ref"
                && !self.description_component_agreement(sd, s(av, "id"), qd, s(bv, "id"))
            {
                return Relation::Unknown;
            }
            let rel = if s(av, "kind") == "ref"
                && s(bv, "kind") == "ref"
                && map.get(s(av, "id")).is_some_and(|id| id == s(bv, "id"))
            {
                if self.local(sd, s(av, "id"), s(av, "id")) == Relation::Conflict
                    || self.local(qd, s(bv, "id"), s(bv, "id")) == Relation::Conflict
                {
                    Relation::Conflict
                } else {
                    Relation::Same
                }
            } else {
                self.term(sd, av, qd, bv)
            };
            result = result.merge(rel);
        }
        result
    }
    fn descriptor_context(&self, sd: &Value, a: &Value, qd: &Value, b: &Value) -> bool {
        // Reject unsupported internal guards before traversing their bodies: IDs
        // can form cycles even when the JSON tree and dependency graph are finite.
        if rows(a, "qualifier_ids").iter().any(|id| {
            !id.as_str()
                .and_then(|id| self.qualifier(sd, id))
                .is_some_and(|q| {
                    s(q, "resolution") == "known"
                        && matches!(s(q, "kind"), "time" | "configuration")
                })
        }) || rows(b, "qualifier_ids").iter().any(|id| {
            !id.as_str()
                .and_then(|id| self.qualifier(qd, id))
                .is_some_and(|q| {
                    s(q, "resolution") == "known"
                        && matches!(s(q, "kind"), "time" | "configuration")
                })
        }) {
            return false;
        }
        ["judgment", "status", "basis", "normative_scope"]
            .iter()
            .all(|k| a[*k] == b[*k])
            && self
                .context(
                    sd,
                    rows(a, "qualifier_ids"),
                    qd,
                    rows(b, "qualifier_ids"),
                    false,
                )
                .0
                == Relation::Same
    }
    // Distinct exact quantities on the same coordinate prove disjoint descriptions.
    // Other predicate differences, origins and absent constraints do not.
    fn descriptor_disjoint(
        &self,
        sd: &Value,
        a: &Value,
        qd: &Value,
        b: &Value,
        map: &BTreeMap<String, String>,
    ) -> bool {
        if !self.descriptor_context(sd, a, qd, b) {
            return false;
        }
        if a["atom"]["predicate"] == b["atom"]["predicate"]
            && a["atom"]["polarity"] != b["atom"]["polarity"]
        {
            let mut positive = b["atom"].clone();
            positive["polarity"] = a["atom"]["polarity"].clone();
            if self.descriptor_atom(sd, &a["atom"], qd, &positive, map, None) == Relation::Same {
                return true;
            }
        }
        if s(&a["atom"], "predicate") != "quantity"
            || s(&b["atom"], "predicate") != "quantity"
            || s(&a["atom"], "polarity") != "positive"
            || s(&b["atom"], "polarity") != "positive"
        {
            return false;
        }
        let (Some(x), Some(y)) = (
            argument(&a["atom"], "amount"),
            argument(&b["atom"], "amount"),
        ) else {
            return false;
        };
        s(x, "kind") == "quantity"
            && s(y, "kind") == "quantity"
            && self.term(sd, x, qd, y) == Relation::Different
            && self.descriptor_atom(sd, &a["atom"], qd, &b["atom"], map, Some("amount"))
                == Relation::Same
    }
    pub(super) fn descriptor_consistent(&self, doc: &Value, t: &Value) -> bool {
        s(t, "resolution") == "known"
            && self.description_component_agreement(doc, s(t, "id"), doc, s(t, "id"))
            && !rows(t, "constraints").is_empty()
            && rows(t, "constraints").iter().all(|p| {
                rows(p, "qualifier_ids").iter().all(|id| {
                    id.as_str()
                        .and_then(|id| self.qualifier(doc, id))
                        .is_some_and(|q| {
                            s(q, "resolution") == "known"
                                && matches!(s(q, "kind"), "time" | "configuration")
                        })
                })
            })
            && !rows(t, "constraints").iter().enumerate().any(|(i, a)| {
                rows(t, "constraints")[i + 1..]
                    .iter()
                    .any(|b| self.descriptor_disjoint(doc, a, doc, b, &BTreeMap::new()))
            })
    }
    pub(super) fn described_term(
        &self,
        sd: &Value,
        a: &Value,
        qd: &Value,
        b: &Value,
        map: Option<&BTreeMap<String, String>>,
    ) -> Relation {
        if !self.descriptor_consistent(sd, a)
            || !self.descriptor_consistent(qd, b)
            || !self.description_component_agreement(sd, s(a, "id"), qd, s(b, "id"))
        {
            return Relation::Unknown;
        }
        let root_a = json!({"kind":"ref","id":a["id"]});
        let root_b = json!({"kind":"ref","id":b["id"]});
        let root = if let Some(map) = map {
            if self
                .description_alpha_roots(s(a, "id"), s(b, "id"), map)
                .is_some()
            {
                Relation::Same
            } else {
                self.term(sd, &root_a, qd, &root_b)
            }
        } else {
            self.term(sd, &root_a, qd, &root_b)
        };
        if root != Relation::Same {
            return root;
        }
        let mut local = map.cloned().unwrap_or_default();
        local.insert(s(a, "id").into(), s(b, "id").into());
        let aa = rows(a, "constraints");
        let bb = rows(b, "constraints");
        if aa.iter().any(|x| {
            bb.iter()
                .any(|y| self.descriptor_disjoint(sd, x, qd, y, &local))
        }) {
            return Relation::Different;
        }
        if aa.len() != bb.len() {
            return Relation::Unknown;
        }
        let mut used = BTreeSet::new();
        for x in aa {
            let found = bb.iter().enumerate().find(|(i, y)| {
                !used.contains(i)
                    && self.descriptor_context(sd, x, qd, y)
                    && self.descriptor_atom(sd, &x["atom"], qd, &y["atom"], &local, None)
                        == Relation::Same
            });
            let Some((index, _)) = found else {
                return Relation::Unknown;
            };
            used.insert(index);
        }
        Relation::Same
    }
}
