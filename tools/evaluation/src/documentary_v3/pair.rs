//! Finite, documentary pair operations. No text parsing, world closure or rule recursion.
use super::DocumentResult;
#[path = "admission_request_context.rs"]
mod admission_request_context;
#[path = "admission_setting_context.rs"]
mod admission_setting_context;
#[path = "anonymous_normative_context.rs"]
mod anonymous_normative_context;
#[path = "configuration_slots.rs"]
mod configuration_slots;
#[path = "context_type_grounding.rs"]
mod context_type_grounding;
#[path = "described_match.rs"]
mod described_match;
#[path = "event_pattern_context.rs"]
mod event_pattern_context;
#[path = "linguistic_composition_matching.rs"]
mod linguistic_composition_matching;
#[path = "nominal_de_matching.rs"]
mod nominal_de_matching;
#[path = "nominal_event_context_matching.rs"]
mod nominal_event_context_matching;
#[path = "nominal_possessive_matching.rs"]
mod nominal_possessive_matching;
#[path = "partial_assignment_context.rs"]
mod partial_assignment_context;
#[path = "partial_documentary_context.rs"]
mod partial_documentary_context;
#[path = "pending_task_scope.rs"]
mod pending_task_scope;
#[path = "relative_bound_matching.rs"]
mod relative_bound_matching;
#[path = "request_context_matching.rs"]
mod request_context_matching;
#[path = "water_operand_context.rs"]
mod water_operand_context;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn rows<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}
fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}
fn id(v: &Value) -> &str {
    s(v, "id")
}
fn strings(v: &[Value]) -> Vec<String> {
    v.iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
fn unique(v: &mut Vec<String>) {
    v.sort();
    v.dedup();
}
fn semantic(v: &Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.iter()
                .filter(|(k, _)| {
                    !matches!(
                        k.as_str(),
                        "evidence" | "mention" | "name" | "declared_class" | "evidence_refs"
                    )
                })
                .map(|(k, v)| (k.clone(), semantic(v)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(semantic).collect()),
        _ => v.clone(),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Relation {
    Same,
    Different,
    Unknown,
    Conflict,
}
impl Relation {
    fn merge(self, other: Self) -> Self {
        use Relation::*;
        match (self, other) {
            (Conflict, _) | (_, Conflict) => Conflict,
            (Different, _) | (_, Different) => Different,
            (Unknown, _) | (_, Unknown) => Unknown,
            _ => Same,
        }
    }
}
struct Engine<'a> {
    source: &'a Value,
    question: &'a Value,
    source_report: &'a Value,
    pending_claims: &'a [Value],
    question_report: &'a Value,
    vocabulary: &'a Value,
    scope: &'a Value,
    context_v2: bool,
    possessive_source: &'a super::nominal_possessive::Nodes,
    lexical_source: &'a super::lexical_clause_context::Nodes,
    possessive_question: &'a super::nominal_possessive::Nodes,
}
impl<'a> Engine<'a> {
    fn is_source_document(&self, doc: &Value) -> bool {
        std::ptr::eq(doc, self.source) || std::ptr::eq(doc, &self.source_report["document"])
    }
    fn reference<'b>(&self, doc: &'b Value, rid: &str) -> Option<&'b Value> {
        rows(doc, "references")
            .iter()
            .find(|r| super::context_budget::charge(1, 0) && id(r) == rid)
    }
    fn local(&self, doc: &Value, left: &str, right: &str) -> Relation {
        if !super::context_budget::charge(1, 0) {
            return Relation::Unknown;
        }
        let component = |start: &str| {
            let mut found = BTreeSet::from([start.to_owned()]);
            loop {
                let before = found.len();
                for e in rows(doc, "identity") {
                    if !super::context_budget::charge(1, 0) {
                        return found;
                    }
                    if s(e, "kind") == "same" {
                        if found.contains(s(e, "left")) {
                            found.insert(s(e, "right").to_owned());
                        }
                        if found.contains(s(e, "right")) {
                            found.insert(s(e, "left").to_owned());
                        }
                    }
                }
                for reference in rows(doc, "references") {
                    if !super::context_budget::charge(1, 0) {
                        return found;
                    }
                    let anaphora = &reference["anaphora"];
                    if s(anaphora, "status") == "asserted"
                        && rows(anaphora, "antecedent_ids").len() == 1
                    {
                        if let Some(antecedent) = anaphora["antecedent_ids"][0].as_str() {
                            if found.contains(id(reference)) {
                                found.insert(antecedent.to_owned());
                            }
                            if found.contains(antecedent) {
                                found.insert(id(reference).to_owned());
                            }
                        }
                    }
                }
                if before == found.len() {
                    break;
                }
            }
            found
        };
        let a = component(left);
        let b = component(right);
        let same = !a.is_disjoint(&b);
        let different = rows(doc, "identity").iter().any(|e| {
            s(e, "kind") == "different"
                && ((a.contains(s(e, "left")) && b.contains(s(e, "right")))
                    || (b.contains(s(e, "left")) && a.contains(s(e, "right"))))
        });
        match (same, different) {
            (true, true) => Relation::Conflict,
            (true, false) => Relation::Same,
            (false, true) => Relation::Different,
            _ => Relation::Unknown,
        }
    }
    fn identity(&self, sd: &Value, left: &str, qd: &Value, right: &str) -> Relation {
        self.identity_depth(sd, left, qd, right, 0)
    }
    fn association_context_matches(
        &self,
        sd: &Value,
        left: &str,
        qd: &Value,
        right: &str,
        depth: usize,
    ) -> bool {
        if !self.nominal_context_matches(sd, left, qd, right, depth) {
            return false;
        }
        let (Some(a), Some(b)) = (
            super::quantified_match::associated_targets(sd, left),
            super::quantified_match::associated_targets(qd, right),
        ) else {
            return false;
        };
        if a.len() != b.len() {
            return false;
        }
        let mut used = BTreeSet::new();
        for target in a {
            let matched = b.iter().enumerate().find(|(index, other)| {
                !used.contains(index)
                    && super::quantified_match::association_anchor_ready(sd, target)
                    && super::quantified_match::association_anchor_ready(qd, other)
                    && self.identity_depth(sd, target, qd, other, depth + 1) == Relation::Same
            });
            let Some((index, _)) = matched else {
                return false;
            };
            used.insert(index);
        }
        true
    }
    fn identity_depth(
        &self,
        sd: &Value,
        left: &str,
        qd: &Value,
        right: &str,
        depth: usize,
    ) -> Relation {
        if super::context_budget::possessive_enabled()
            && (self.possessive_reference(sd, left) || self.possessive_reference(qd, right))
        {
            return Relation::Unknown;
        }

        if std::ptr::eq(sd, qd) {
            return self.local(sd, left, right);
        }
        if depth > 32 {
            return Relation::Unknown;
        }
        if self.local(sd, left, left) == Relation::Conflict
            || self.local(qd, right, right) == Relation::Conflict
        {
            return Relation::Conflict;
        }
        let namespace = s(self.scope, "namespace");
        if namespace.is_empty()
            || s(sd, "domain_namespace") != namespace
            || s(qd, "domain_namespace") != namespace
        {
            return Relation::Unknown;
        }
        if !self.association_context_matches(sd, left, qd, right, depth) {
            return Relation::Unknown;
        }
        let mut same = false;
        let mut different = false;
        // Explicit same/different propagate through every anchored local alias.
        for a in rows(sd, "references") {
            for b in rows(qd, "references") {
                if !super::context_budget::charge(1, 0) {
                    return Relation::Unknown;
                }
                let la = self.local(sd, left, id(a));
                let rb = self.local(qd, right, id(b));
                if matches!(la, Relation::Conflict) || matches!(rb, Relation::Conflict) {
                    continue;
                }
                if !matches!(la, Relation::Same | Relation::Different)
                    || !matches!(rb, Relation::Same | Relation::Different)
                {
                    continue;
                }
                let an = s(&a["name"], "quote");
                let bn = s(&b["name"], "quote");
                if an.is_empty() || an != bn || a["denotation"] != b["denotation"] {
                    continue;
                }
                let ac = s(&a["declared_class"], "quote");
                let bc = s(&b["declared_class"], "quote");
                if ac != bc {
                    continue;
                }
                if !self.association_context_matches(sd, id(a), qd, id(b), depth) {
                    continue;
                }
                let al: Vec<_> = rows(a, "links")
                    .iter()
                    .filter(|l| s(l, "kind") == "owned_by")
                    .collect();
                let bl: Vec<_> = rows(b, "links")
                    .iter()
                    .filter(|l| s(l, "kind") == "owned_by")
                    .collect();
                if al.len() != bl.len() {
                    continue;
                }
                if al.iter().any(|x| {
                    !bl.iter().any(|y| {
                        self.identity_depth(sd, s(x, "target"), qd, s(y, "target"), depth + 1)
                            == Relation::Same
                    })
                }) {
                    continue;
                }
                if la == Relation::Same && rb == Relation::Same {
                    same = true;
                } else if la != rb {
                    different = true;
                }
            }
        }
        match (same, different) {
            (true, true) => Relation::Conflict,
            (true, false) => Relation::Same,
            (false, true) => Relation::Different,
            _ => Relation::Unknown,
        }
    }
    // Applicability of a partial answer is weaker than full reference identity:
    // missing class evidence may stay partial, but competing named targets
    // need a documentary name anchor. This never proves Different or anonymous Same.
    fn partial_named_anchor(&self, actual: &str, wanted: &str, depth: usize) -> bool {
        if depth > 8
            || !self.nominal_context_matches(self.source, actual, self.question, wanted, depth)
            || self.local(self.source, actual, actual) == Relation::Conflict
            || self.local(self.question, wanted, wanted) == Relation::Conflict
            || s(self.source, "domain_namespace") != s(self.scope, "namespace")
            || s(self.question, "domain_namespace") != s(self.scope, "namespace")
        {
            return false;
        }
        let actual_aliases: Vec<_> = rows(self.source, "references")
            .iter()
            .filter(|r| self.local(self.source, actual, id(r)) == Relation::Same)
            .collect();
        let wanted_aliases: Vec<_> = rows(self.question, "references")
            .iter()
            .filter(|r| self.local(self.question, wanted, id(r)) == Relation::Same)
            .collect();
        if actual_aliases.is_empty() || wanted_aliases.is_empty() {
            return false;
        }
        if actual_aliases
            .iter()
            .all(|r| s(&r["name"], "quote").is_empty())
        {
            // In particular, a universally bound source variable must not need
            // the name of every individual in its domain. Preserve the existing
            // qualified context route; this is no proof of its applicability.
            return true;
        }
        let named: Vec<_> = wanted_aliases
            .iter()
            .filter(|r| !s(&r["name"], "quote").is_empty())
            .collect();
        if !named.is_empty()
            && !named.iter().any(|target| {
                actual_aliases.iter().any(|r| {
                    r["name"]["quote"] == target["name"]["quote"]
                        && r["denotation"] == target["denotation"]
                })
            })
        {
            return false;
        }
        for target in wanted_aliases {
            for link in rows(target, "links") {
                if link["kind"] == super::nominal_de::KIND {
                    continue;
                }
                if !actual_aliases.iter().any(|r| {
                    rows(r, "links").iter().any(|candidate| {
                        candidate["kind"] == link["kind"]
                            && self.partial_named_anchor(
                                s(candidate, "target"),
                                s(link, "target"),
                                depth + 1,
                            )
                    })
                }) {
                    return false;
                }
            }
        }
        true
    }
    fn required_association_ref(&self, actual: Option<&str>, wanted: &str) -> bool {
        if wanted.is_empty() {
            return true;
        }
        let targets = super::quantified_match::associated_targets(self.question, wanted);
        let nominal =
            if super::request_context_profile::permits(self.vocabulary, super::nominal_de::KIND) {
                super::nominal_de::targets(self.question, wanted)
            } else {
                Some(Vec::new())
            };
        if targets.as_ref().is_some_and(Vec::is_empty)
            && nominal.as_ref().is_some_and(Vec::is_empty)
        {
            return true;
        }
        actual.is_some_and(|actual| {
            self.identity(self.source, actual, self.question, wanted) == Relation::Same
        })
    }
    /// Query-required contextual identity must precede an answer witness. This
    /// does not suppress separately established corrections or relevance paths.
    fn required_associations_apply(&self, actual: &Value, request: &Value) -> bool {
        self.required_associations_with_map(actual, request, None)
    }
    fn required_association_with_map(
        &self,
        actual: Option<&str>,
        wanted: &str,
        map: Option<&BTreeMap<String, String>>,
    ) -> bool {
        if let (Some(actual), Some(map)) = (actual, map) {
            if self.description_alpha_roots(actual, wanted, map).is_some() {
                return true;
            }
        }
        self.required_association_ref(actual, wanted)
    }
    fn required_associations_with_map(
        &self,
        actual: &Value,
        request: &Value,
        map: Option<&BTreeMap<String, String>>,
    ) -> bool {
        // A closed-set exclusion may prove a truth request by Different rather
        // than Same. Its existing comparison must remain responsible for that.
        if s(request, "mode") != "truth"
            && !self.required_named_targets_with_map(actual, request, map)
        {
            return false;
        }
        let target = &request["target"];
        for arg in rows(&target["atom"], "arguments") {
            let wanted = &arg["value"];
            if matches!(s(wanted, "kind"), "ref" | "described_ref")
                && !self.required_association_with_map(
                    argument(&actual["atom"], s(arg, "role"))
                        .filter(|v| matches!(s(v, "kind"), "ref" | "described_ref"))
                        .map(|v| s(v, "id")),
                    s(wanted, "id"),
                    map,
                )
            {
                return false;
            }
        }
        for field in ["subject", "event_ref"] {
            if target[field].is_string()
                && !self.required_association_ref(actual[field].as_str(), s(target, field))
            {
                return false;
            }
        }
        if !self.required_association_ref(
            actual["normative_scope"].as_str(),
            s(request, "normative_scope_filter"),
        ) {
            return false;
        }
        for (selector, field) in [("method_selector", "method"), ("time_selector", "times")] {
            let wanted = &target[selector]["value"];
            if s(wanted, "kind") == "ref" {
                let wanted_id = s(wanted, "id");
                if super::quantified_match::associated_targets(self.question, wanted_id)
                    .as_ref()
                    .is_some_and(Vec::is_empty)
                {
                    continue;
                }
                let candidates = if field == "method" {
                    vec![&actual[field]["value"]]
                } else {
                    rows(actual, field)
                        .iter()
                        .filter(|t| t["role"] == target[selector]["role"])
                        .map(|t| &t["value"])
                        .collect()
                };
                if !candidates.iter().any(|v| {
                    self.required_association_ref(
                        (s(v, "kind") == "ref").then(|| s(v, "id")),
                        wanted_id,
                    )
                }) {
                    return false;
                }
            }
        }
        true
    }
    fn required_named_targets_apply(&self, actual: &Value, request: &Value) -> bool {
        self.required_named_targets_with_map(actual, request, None)
    }
    fn required_named_targets_with_map(
        &self,
        actual: &Value,
        request: &Value,
        map: Option<&BTreeMap<String, String>>,
    ) -> bool {
        let target = &request["target"];
        for arg in rows(&target["atom"], "arguments") {
            if matches!(s(&arg["value"], "kind"), "ref" | "described_ref")
                && !argument(&actual["atom"], s(arg, "role")).is_some_and(|v| {
                    matches!(s(v, "kind"), "ref" | "described_ref")
                        && if s(v, "kind") == "described_ref"
                            || s(&arg["value"], "kind") == "described_ref"
                        {
                            self.description_anchor_credited(s(v, "id"), s(&arg["value"], "id"))
                                || map.is_some_and(|map| {
                                    self.description_alpha_roots(
                                        s(v, "id"),
                                        s(&arg["value"], "id"),
                                        map,
                                    )
                                    .is_some()
                                        && self.mapped_term(v, &arg["value"], map) == Relation::Same
                                })
                        } else {
                            self.partial_named_anchor(s(v, "id"), s(&arg["value"], "id"), 0)
                        }
                })
            {
                return false;
            }
        }
        for field in ["subject", "event_ref"] {
            if target[field].is_string()
                && !actual[field]
                    .as_str()
                    .is_some_and(|v| self.partial_named_anchor(v, s(target, field), 0))
            {
                return false;
            }
        }
        true
    }
    fn term(&self, sd: &Value, a: &Value, qd: &Value, b: &Value) -> Relation {
        if !super::context_budget::charge(1, 0) {
            return Relation::Unknown;
        }
        if s(a, "kind") != s(b, "kind") {
            return Relation::Unknown;
        }
        match s(a, "kind") {
            "ref" => self.identity(sd, s(a, "id"), qd, s(b, "id")),
            "described_ref" => self.described_term(sd, a, qd, b, None),
            "quantity" => {
                if decimal_key(s(a, "decimal")) == decimal_key(s(b, "decimal"))
                    && a["unit"] == b["unit"]
                    && a["bound"] == b["bound"]
                {
                    Relation::Same
                } else if a["unit"] == b["unit"]
                    && s(a, "bound") == "exact"
                    && s(b, "bound") == "exact"
                    && decimal_key(s(a, "decimal")) != decimal_key(s(b, "decimal"))
                {
                    Relation::Different
                } else {
                    Relation::Unknown
                }
            }
            "time" => {
                if a["value"] == b["value"] && a["format"] == b["format"] {
                    Relation::Same
                } else {
                    Relation::Unknown
                }
            }
            "symbol" => {
                if a["id"] == b["id"] {
                    Relation::Same
                } else {
                    Relation::Different
                }
            }
            "text" => {
                if a["value"] == b["value"] {
                    Relation::Same
                } else if self.literal_disjoint(a, b) {
                    Relation::Different
                } else {
                    Relation::Unknown
                }
            }
            _ => Relation::Unknown,
        }
    }
    fn literal_disjoint(&self, a: &Value, b: &Value) -> bool {
        // Finite domains must explicitly declare exclusivity; fresh strings never suffice.
        let _ = (a, b);
        false // This vocabulary version declares no exclusive literal domains.
    }
    fn atom(
        &self,
        sd: &Value,
        a: &Value,
        qd: &Value,
        b: &Value,
        omitted: Option<&str>,
    ) -> Relation {
        if a["predicate"] != b["predicate"] {
            return Relation::Different;
        }
        let mut result = Relation::Same;
        for arg in rows(b, "arguments") {
            let role = s(arg, "role");
            if Some(role) == omitted {
                continue;
            }
            let Some(sa) = rows(a, "arguments").iter().find(|x| s(x, "role") == role) else {
                return Relation::Unknown;
            };
            if (super::descriptions::contains(a) || super::descriptions::contains(b))
                && s(&sa["value"], "kind") == "ref"
                && s(&arg["value"], "kind") == "ref"
                && !self.description_component_agreement(
                    sd,
                    s(&sa["value"], "id"),
                    qd,
                    s(&arg["value"], "id"),
                )
            {
                result = result.merge(Relation::Unknown);
            } else {
                result = result.merge(self.term(sd, &sa["value"], qd, &arg["value"]));
            }
        }
        // A missing non-slot role is never interpreted as a wildcard.
        for arg in rows(a, "arguments") {
            if Some(s(arg, "role")) != omitted
                && !rows(b, "arguments")
                    .iter()
                    .any(|x| x["role"] == arg["role"])
            {
                return Relation::Unknown;
            }
        }
        result
    }
    fn qualifier<'b>(&self, doc: &'b Value, qid: &str) -> Option<&'b Value> {
        rows(doc, "qualifiers").iter().find(|q| id(q) == qid)
    }
    fn qualifier_relation(&self, sd: &Value, a: &Value, qd: &Value, b: &Value) -> Relation {
        if s(a, "resolution") != "known" || s(b, "resolution") != "known" {
            return Relation::Unknown;
        }
        if a["kind"] != b["kind"] {
            return Relation::Unknown;
        }
        match s(a, "kind") {
            "configuration" => {
                if a["body"]["key"] != b["body"]["key"] {
                    Relation::Unknown
                } else {
                    self.term(sd, &a["body"]["value"], qd, &b["body"]["value"])
                }
            }
            "time" => {
                if a["body"]["role"] != b["body"]["role"] {
                    Relation::Unknown
                } else {
                    self.term(sd, &a["body"]["value"], qd, &b["body"]["value"])
                }
            }
            "guard" => self.proposition(sd, &a["body"], qd, &b["body"], None),
            _ => Relation::Unknown,
        }
    }
    fn proposition(
        &self,
        sd: &Value,
        a: &Value,
        qd: &Value,
        b: &Value,
        slot: Option<&str>,
    ) -> Relation {
        if a["judgment"] != b["judgment"]
            || a["status"] != b["status"]
            || a["basis"] != b["basis"]
            || a["atom"]["polarity"] != b["atom"]["polarity"]
        {
            return Relation::Different;
        }
        let mut r = self.atom(sd, &a["atom"], qd, &b["atom"], slot);
        if !a["normative_scope"].is_null() || !b["normative_scope"].is_null() {
            r = r.merge(self.identity(sd, s(a, "normative_scope"), qd, s(b, "normative_scope")));
        }
        r.merge(
            self.context(
                sd,
                rows(a, "qualifier_ids"),
                qd,
                rows(b, "qualifier_ids"),
                false,
            )
            .0,
        )
    }
    fn guard_satisfied(&self, doc: &Value, body: &Value) -> bool {
        let report = if self.is_source_document(doc) {
            self.source_report
        } else {
            self.question_report
        };
        let source_side = self.is_source_document(doc);
        let execution = if source_side { self.source } else { doc };
        let possessive_blocked = if source_side {
            self.possessive_source
        } else {
            self.possessive_question
        };
        rows(execution, "claims")
            .iter()
            .filter(|c| {
                super::context_budget::charge(1, 0)
                    && !super::nominal_possessive::has(possessive_blocked, "claim", id(c))
                    && s(&c["body"], "kind") == "proposition"
                    && self.claim_qualifiers(c, &c["body"]["value"]).is_empty()
                    && self.branch(doc, s(c, "scope_id")).is_empty()
            })
            .any(|c| {
                self.proposition(doc, &c["body"]["value"], doc, body, None) == Relation::Same
                    && rows(report, "candidates").iter().any(|candidate| {
                        s(candidate, "state") == "eligible"
                            && rows(candidate, "claim_ids")
                                .iter()
                                .any(|x| x.as_str() == Some(id(c)))
                    })
                    && !rows(doc, "dependencies").iter().any(|d| {
                        s(&d["consumer"], "kind") == "claim"
                            && s(&d["consumer"], "id") == id(c)
                            && s(&d["requires"], "kind") == "qualifier"
                    })
            })
    }
    fn context(
        &self,
        sd: &Value,
        source_ids: &[Value],
        qd: &Value,
        question_ids: &[Value],
        include_scope: bool,
    ) -> (Relation, Vec<String>) {
        let mut missing = vec![];
        let mut result = Relation::Same;
        let sources: Vec<_> = source_ids
            .iter()
            .filter_map(|x| x.as_str().and_then(|x| self.qualifier(sd, x)))
            .collect();
        let targets: Vec<_> = question_ids
            .iter()
            .filter_map(|x| x.as_str().and_then(|x| self.qualifier(qd, x)))
            .collect();
        for target in &targets {
            if !super::context_budget::charge(1, 0) {
                return (Relation::Unknown, vec!["request_context_limit".into()]);
            }
            let matches: Vec<_> = sources
                .iter()
                .map(|source| {
                    if super::context_budget::charge(1, 0) {
                        self.qualifier_relation(sd, source, qd, target)
                    } else {
                        Relation::Unknown
                    }
                })
                .collect();
            if matches.contains(&Relation::Same)
                && !matches.contains(&Relation::Different)
                && !matches.contains(&Relation::Conflict)
            {
                continue;
            }
            result = result.merge(
                if matches.contains(&Relation::Conflict)
                    || matches.contains(&Relation::Same) && matches.contains(&Relation::Different)
                {
                    Relation::Conflict
                } else if matches.contains(&Relation::Different) {
                    Relation::Different
                } else {
                    Relation::Unknown
                },
            );
            missing.push(context_code(s(target, "kind")).to_owned());
        }
        for source in sources {
            if !super::context_budget::charge(1, 0) {
                return (Relation::Unknown, vec!["request_context_limit".into()]);
            }
            if targets
                .iter()
                .any(|target| self.qualifier_relation(sd, source, qd, target) == Relation::Same)
            {
                continue;
            }
            let mut satisfied = false;
            if include_scope && s(source, "resolution") == "known" {
                match s(source, "kind") {
                    "configuration" => {
                        satisfied = rows(self.scope, "configuration").iter().any(|x| {
                            x["key"] == source["body"]["key"]
                                && self.term(sd, &source["body"]["value"], qd, &x["value"])
                                    == Relation::Same
                        })
                    }
                    "time" => {
                        satisfied = self.scope["time_selector"]["role"] == source["body"]["role"]
                            && self.term(
                                sd,
                                &source["body"]["value"],
                                qd,
                                &self.scope["time_selector"]["value"],
                            ) == Relation::Same
                    }
                    "guard" => satisfied = self.guard_satisfied(sd, &source["body"]),
                    _ => {}
                }
            }
            if !satisfied {
                result = result.merge(Relation::Unknown);
                missing.push(context_code(s(source, "kind")).to_owned());
            }
        }
        unique(&mut missing);
        (result, missing)
    }
    fn qualified_ids(&self, doc: &Value, c: &Value, p: &Value) -> Vec<Value> {
        let mut q = rows(c, "qualifier_ids").to_vec();
        q.extend(rows(p, "qualifier_ids").iter().cloned());
        let mut scopes = BTreeSet::new();
        let mut cursor = s(c, "scope_id");
        while scopes.insert(cursor.to_owned()) {
            if !super::context_budget::charge(1, 0) {
                break;
            }
            let Some(scope) = rows(doc, "scopes").iter().find(|scope| id(scope) == cursor) else {
                break;
            };
            cursor = s(scope, "parent_id");
        }
        for alternative in rows(doc, "claims")
            .iter()
            .filter(|a| s(&a["body"], "kind") == "alternatives")
        {
            if id(alternative) != id(c)
                && rows(&alternative["body"], "branches")
                    .iter()
                    .any(|branch| scopes.contains(s(branch, "scope_id")))
            {
                q.extend(rows(alternative, "qualifier_ids").iter().cloned());
            }
        }
        q.sort_by_key(Value::to_string);
        q.dedup();
        q
    }
    fn claim_qualifiers(&self, c: &Value, p: &Value) -> Vec<Value> {
        let doc = if rows(self.question, "claims")
            .iter()
            .any(|x| std::ptr::eq(x, c))
        {
            self.question
        } else {
            self.source
        };
        self.qualified_ids(doc, c, p)
    }
    fn request_qualifiers(&self, r: &Value) -> Vec<Value> {
        self.qualified_ids(self.question, r, &r["target"])
    }
    fn premise_qualifiers(&self, r: &Value, p: &Value) -> Vec<Value> {
        let mut q = self.claim_qualifiers(p, &p["body"]["value"]);
        q.extend(self.request_qualifiers(r));
        q.sort_by_key(Value::to_string);
        q.dedup();
        q
    }
    fn dependency_missing(&self, doc: &Value, kind: &str, cid: &str) -> Vec<String> {
        let mut missing = vec![];
        // Qualifiers own their operand references. Follow those implicit edges
        // plus declared dependencies without inventing or writing raw edges.
        fn operands(v: &Value, out: &mut BTreeSet<(String, String, String)>) {
            if matches!(s(v, "kind"), "ref" | "described_ref") {
                out.insert(("reference".into(), id(v).into(), "identity".into()));
            }
            if let Some(o) = v.as_object() {
                for (key, value) in o {
                    if key == "normative_scope" || key == "boundary_subject" {
                        if let Some(id) = value.as_str() {
                            out.insert(("reference".into(), id.into(), "normative_target".into()));
                        }
                    } else if key == "qualifier_ids" {
                        for id in value
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(Value::as_str)
                        {
                            out.insert(("qualifier".into(), id.into(), "configuration".into()));
                        }
                    }
                    operands(value, out);
                }
            } else if let Some(a) = v.as_array() {
                for value in a {
                    operands(value, out);
                }
            }
        }
        let report = if self.is_source_document(doc) {
            self.source_report
        } else {
            self.question_report
        };
        let mut prefixes = BTreeMap::new();
        for (table, node_kind) in [
            ("claims", "claim"),
            ("requests", "request"),
            ("references", "reference"),
            ("qualifiers", "qualifier"),
            ("scopes", "scope"),
        ] {
            for (index, row) in rows(&report["document"], table).iter().enumerate() {
                let node = (node_kind.to_owned(), id(row).to_owned());
                prefixes.insert(format!("/{table}/{index}"), node.clone());
                prefixes.insert(
                    format!(
                        "/{node_kind}/{}",
                        id(row).replace('~', "~0").replace('/', "~1")
                    ),
                    node,
                );
            }
        }
        let mut node_issues: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for issue in rows(report, "issues") {
            let prefix = s(issue, "path")
                .split('/')
                .take(3)
                .collect::<Vec<_>>()
                .join("/");
            if let Some(node) = prefixes.get(&prefix) {
                node_issues.entry(node.clone()).or_default().push(issue);
            }
        }
        let mut declared: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
        for dep in rows(doc, "dependencies") {
            if !super::context_budget::charge(1, 0) {
                return vec!["request_context_limit".into()];
            }
            declared
                .entry((
                    s(&dep["consumer"], "kind").to_owned(),
                    s(&dep["consumer"], "id").to_owned(),
                ))
                .or_default()
                .insert((
                    s(&dep["requires"], "kind").to_owned(),
                    s(&dep["requires"], "id").to_owned(),
                    s(dep, "role").to_owned(),
                ));
        }
        let mut pending = vec![(kind.to_owned(), cid.to_owned(), format!("{kind}:{cid}"))];
        let mut visited = BTreeSet::new();
        let mut graph: BTreeMap<(String, String), BTreeSet<(String, String)>> = BTreeMap::new();
        while let Some((owner_kind, owner_id, path)) = pending.pop() {
            if !super::context_budget::charge(1, 0) {
                missing.push("request_context_limit".into());
                break;
            }
            if !visited.insert((owner_kind.clone(), owner_id.clone())) {
                continue;
            }
            let node = (owner_kind.clone(), owner_id.clone());
            if let Some(issues) = node_issues.get(&node) {
                for issue in issues {
                    missing.push(format!(
                        "{path}->validation:{}:{}",
                        s(issue, "code"),
                        s(issue, "path")
                    ));
                }
            }
            let mut targets = declared.get(&node).cloned().unwrap_or_default();
            if super::request_context_profile::enabled(self.vocabulary) {
                let collection = match owner_kind.as_str() {
                    "claim" => "claims",
                    "request" => "requests",
                    _ => "",
                };
                if let Some(owner) = rows(doc, collection).iter().find(|o| id(o) == owner_id) {
                    for binding in rows(owner, "context_bindings") {
                        operands(binding, &mut targets);
                    }
                    if super::context_budget::composition_enabled()
                        && super::linguistic_composition::carries(owner)
                    {
                        operands(&owner["body"], &mut targets);
                        if let Some(scope) = owner["scope_id"].as_str() {
                            targets.insert(("scope".into(), scope.into(), "binding".into()));
                        }
                    }
                }
            }
            if owner_kind == "qualifier" {
                if let Some(q) = self.qualifier(doc, &owner_id) {
                    operands(&q["body"], &mut targets);
                }
            }
            for (required_kind, required_id, role) in targets {
                if !super::context_budget::charge(1, 0) {
                    missing.push("request_context_limit".into());
                    break;
                }
                let collection = match required_kind.as_str() {
                    "claim" => "claims",
                    "reference" => "references",
                    "qualifier" => "qualifiers",
                    "scope" => "scopes",
                    _ => "",
                };
                let target = rows(doc, collection).iter().find(|x| id(x) == required_id);
                let resolved = target.is_some_and(|t| {
                    if required_kind == "qualifier" {
                        s(t, "resolution") == "known"
                            && (s(t, "kind") != "guard" || self.guard_satisfied(doc, &t["body"]))
                    } else if required_kind == "reference" {
                        s(&t["anaphora"], "status") != "unresolved"
                    } else {
                        true
                    }
                });
                let next_path = format!("{path}->{required_kind}:{required_id}:{role}");
                if !resolved {
                    missing.push(next_path.clone());
                }
                if target.is_some() {
                    graph
                        .entry((owner_kind.clone(), owner_id.clone()))
                        .or_default()
                        .insert((required_kind.clone(), required_id.clone()));
                    graph
                        .entry((required_kind.clone(), required_id.clone()))
                        .or_default();
                    pending.push((required_kind, required_id, next_path));
                }
            }
        }
        // Explicit raw cycles are invalid already. Implicit operand edges can
        // expose another cycle; a finite traversal must not call it resolved.
        let mut indegree: BTreeMap<_, usize> = graph.keys().cloned().map(|n| (n, 0)).collect();
        for targets in graph.values() {
            for target in targets {
                *indegree.get_mut(target).unwrap() += 1;
            }
        }
        let mut ready: Vec<_> = indegree
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(n, _)| n.clone())
            .collect();
        let mut removed = 0;
        while let Some(node) = ready.pop() {
            if !super::context_budget::charge(1, 0) {
                missing.push("request_context_limit".into());
                break;
            }
            removed += 1;
            for target in &graph[&node] {
                let count = indegree.get_mut(target).unwrap();
                *count -= 1;
                if *count == 0 {
                    ready.push(target.clone());
                }
            }
        }
        if removed != graph.len() {
            missing.push(format!("{kind}:{cid}->dependency_cycle"));
        }
        let report = if self.is_source_document(doc) {
            self.source_report
        } else {
            self.question_report
        };
        for candidate in rows(report, "candidates") {
            if rows(candidate, "claim_ids")
                .iter()
                .any(|x| x.as_str() == Some(cid))
                && s(candidate, "state") == "unresolved"
            {
                missing.extend(strings(rows(candidate, "dependency_paths")));
            }
        }
        unique(&mut missing);
        missing
    }
    fn answer_association_context_matches(
        &self,
        source_id: &str,
        pattern_id: &str,
        depth: usize,
    ) -> bool {
        if depth > 8
            || !self.nominal_context_matches(
                self.source,
                source_id,
                self.question,
                pattern_id,
                depth,
            )
        {
            return false;
        }
        let (Some(actual), Some(wanted)) = (
            super::quantified_match::associated_targets(self.source, source_id),
            super::quantified_match::associated_targets(self.question, pattern_id),
        ) else {
            return false;
        };
        if actual.len() != wanted.len() {
            return false;
        }
        let mut used = BTreeSet::new();
        for target in wanted {
            let Some(pattern) = self.reference(self.question, target) else {
                return false;
            };
            let matched = actual.iter().enumerate().find(|(index, candidate)| {
                if used.contains(index)
                    || !super::quantified_match::association_anchor_ready(self.source, candidate)
                    || !super::quantified_match::association_anchor_ready(self.question, target)
                {
                    return false;
                }
                if pattern["name"].is_null() {
                    // Anonymous context is a bounded descriptive constraint,
                    // never a new cross-document identity anchor.
                    self.answer_reference_pattern(candidate, target, depth + 1) == Relation::Same
                } else {
                    self.identity(self.source, candidate, self.question, target) == Relation::Same
                }
            });
            let Some((index, _)) = matched else {
                return false;
            };
            used.insert(index);
        }
        true
    }
    // Satisfaction of a local answer pattern is not cross-document entity identity.
    fn answer_reference_pattern(
        &self,
        source_id: &str,
        pattern_id: &str,
        depth: usize,
    ) -> Relation {
        if !super::context_budget::charge(1, 0) {
            return Relation::Unknown;
        }
        if depth > 8 {
            return Relation::Unknown;
        }
        let (Some(source), Some(pattern)) = (
            self.reference(self.source, source_id),
            self.reference(self.question, pattern_id),
        ) else {
            return Relation::Unknown;
        };
        if !source["anaphora"].is_null() || !pattern["anaphora"].is_null() {
            return Relation::Unknown;
        }
        // A descriptive match does not turn an individual into a class or group.
        if source["denotation"] != pattern["denotation"] {
            return Relation::Unknown;
        }
        if self.local(self.source, source_id, source_id) == Relation::Conflict
            || self.local(self.question, pattern_id, pattern_id) == Relation::Conflict
        {
            return Relation::Conflict;
        }
        if !self.answer_association_context_matches(source_id, pattern_id, depth) {
            return Relation::Unknown;
        }
        let mut relation = if !pattern["name"].is_null() {
            self.identity(self.source, source_id, self.question, pattern_id)
        } else {
            Relation::Same
        };
        for (field, key) in [("declared_class", "quote"), ("semantic_type", "type_id")] {
            if !pattern[field].is_null() {
                relation = relation.merge(if source[field].is_null() {
                    Relation::Unknown
                } else if source[field][key] == pattern[field][key] {
                    Relation::Same
                } else {
                    Relation::Unknown
                });
            }
        }
        if pattern["declared_class"].is_null() && pattern["semantic_type"].is_null() {
            relation = relation.merge(Relation::Unknown);
        }
        for wanted in rows(pattern, "links") {
            if !super::context_budget::charge(1, 0) {
                return Relation::Unknown;
            }
            if matches!(
                s(wanted, "kind"),
                "associated_with" | "nominal_complement_de"
            ) {
                continue;
            }
            let matches: Vec<_> = rows(source, "links")
                .iter()
                .filter(|link| link["kind"] == wanted["kind"])
                .map(|link| {
                    self.answer_reference_pattern(s(link, "target"), s(wanted, "target"), depth + 1)
                })
                .collect();
            let linked = if matches.contains(&Relation::Same) {
                Relation::Same
            } else if matches.contains(&Relation::Conflict) {
                Relation::Conflict
            } else if matches.is_empty() || matches.contains(&Relation::Unknown) {
                Relation::Unknown
            } else {
                Relation::Different
            };
            relation = relation.merge(linked);
        }
        relation
    }
    fn answer_term_pattern(&self, term: &Value, pattern: &str) -> Relation {
        match s(term, "kind") {
            "ref" => self.answer_reference_pattern(s(term, "id"), pattern, 0),
            "described_ref" if self.descriptor_consistent(self.source, term) => {
                self.answer_reference_pattern(s(term, "id"), pattern, 0)
            }
            "described_ref" => Relation::Unknown,
            _ => Relation::Different,
        }
    }
    fn constrain_answer_pattern(&self, r: &Value, out: &mut Evidence) {
        if s(&r["class_filter"], "kind") != "reference_pattern" {
            return;
        }
        let pattern = s(&r["class_filter"], "reference_id");
        let mut filtered_partial = vec![];
        for key in 0..3 {
            let witnesses = match key {
                0 => &mut out.support,
                1 => &mut out.refutation,
                _ => &mut out.partial,
            };
            let mut retained = vec![];
            for witness in std::mem::take(witnesses) {
                if !super::context_budget::charge(1, 0) {
                    break;
                }
                // A finite relevance link exposes intermediate context, not a supplied answer.
                if matches!(
                    s(&witness, "kind"),
                    "partial_relevance" | "premise_refutation" | "partial_premise_refutation"
                ) {
                    retained.push(witness);
                    continue;
                }
                // This route transports source operands with explicit origin debt.
                // Its empty payload is context, never a candidate pipe answer.
                if key == 2
                    && self.context_v2
                    && s(&witness, "kind") == "partial_water_operand_context_v2"
                    && witness["answer_terms"] == json!([])
                    && witness["answer_propositions"] == json!([])
                    && witness["bindings"] == json!([])
                    && witness["context_transport"]["requested_origin_inferred"] == false
                    && witness["context_transport"]["identity_proven"] == false
                {
                    retained.push(witness);
                    continue;
                }
                let terms = rows(&witness, "answer_terms");
                let relation = if terms.is_empty() {
                    Relation::Unknown
                } else {
                    terms.iter().fold(Relation::Same, |relation, term| {
                        relation.merge(self.answer_term_pattern(term, pattern))
                    })
                };
                if relation != Relation::Same {
                    let matched: Vec<_> = terms
                        .iter()
                        .filter(|term| self.answer_term_pattern(term, pattern) == Relation::Same)
                        .cloned()
                        .collect();
                    if !matched.is_empty() {
                        // An uncertain member must not erase an independently matching
                        // answer. Keep only proven members and mark the set incomplete.
                        let mut partial = witness.clone();
                        partial["answer_terms"] = json!(matched);
                        let mut missing = strings(rows(&partial, "missing_constraints"));
                        missing.push("answer_reference_pattern".into());
                        unique(&mut missing);
                        partial["missing_constraints"] = json!(missing);
                        filtered_partial.push(partial);
                        out.missing.push("answer_reference_pattern".into());
                    }
                }
                match relation {
                    Relation::Same => retained.push(witness),
                    Relation::Different => (),
                    Relation::Unknown | Relation::Conflict => {
                        let mut candidate = witness;
                        candidate["state"] = json!("unresolved");
                        candidate["reason_codes"] = json!(["answer_reference_pattern"]);
                        out.unresolved.push(candidate);
                    }
                }
            }
            *witnesses = retained;
        }
        out.partial.extend(filtered_partial);
    }
    fn class_match(&self, term: &Value, filter: &Value) -> bool {
        if filter.is_null() {
            return true;
        }
        if !matches!(s(term, "kind"), "ref" | "described_ref")
            || s(term, "kind") == "described_ref" && !self.descriptor_consistent(self.source, term)
        {
            return false;
        }
        self.reference(self.source, s(term, "id"))
            .is_some_and(|r| match s(filter, "kind") {
                "lexical" => {
                    r["declared_class"]["quote"] == filter["class_quote"]["quote"]
                        && !r["declared_class"].is_null()
                }
                "semantic" => {
                    r["semantic_type"]["type_id"] == filter["type_id"]
                        && !r["semantic_type"].is_null()
                }
                _ => false,
            })
    }
    fn filters(&self, request: &Value, p: &Value, terms: &[Value]) -> Vec<String> {
        let mut missing = vec![];
        if terms
            .iter()
            .any(|t| s(t, "kind") == "described_ref" && !self.descriptor_consistent(self.source, t))
        {
            missing.push("description".to_owned());
        }
        if matches!(s(request, "mode"), "answer" | "event_slot")
            && terms
                .iter()
                .any(|t| s(t, "kind") == "quantity" && s(t, "bound") != "exact")
        {
            missing.push("quantity_bound".to_owned());
        }
        if super::temporal_slot(s(request, "slot")).is_some_and(|(_, year)| year)
            && (terms.is_empty()
                || terms
                    .iter()
                    .any(|t| s(t, "kind") != "time" || s(t, "format") != "year"))
        {
            missing.push("time_granularity".to_owned());
        }
        if s(&request["basis_requirement"], "kind") == "exact"
            && request["basis_requirement"]["value"] != p["basis"]
        {
            missing.push("basis".to_owned());
        }
        if !request["class_filter"].is_null()
            && s(&request["class_filter"], "kind") != "reference_pattern"
            && (terms.is_empty()
                || terms
                    .iter()
                    .any(|t| !self.class_match(t, &request["class_filter"])))
        {
            missing.push("class_filter".to_owned());
        }
        if !request["unit"].is_null()
            && (terms.is_empty()
                || terms
                    .iter()
                    .any(|t| s(t, "kind") != "quantity" || t["unit"] != request["unit"]["symbol"]))
        {
            missing.push("unit".to_owned());
        }
        missing
    }
    fn scope_match(&self, request: &Value, p: &Value) -> Relation {
        if request["normative_scope_filter"].is_null() {
            return Relation::Same;
        }
        if p["normative_scope"].is_null() {
            return Relation::Unknown;
        }
        self.identity(
            self.source,
            s(p, "normative_scope"),
            self.question,
            s(request, "normative_scope_filter"),
        )
    }
    fn branch(&self, doc: &Value, scope_id: &str) -> Vec<String> {
        let mut cur = scope_id;
        let mut found = vec![];
        let mut seen = BTreeSet::new();
        while seen.insert(cur.to_owned()) {
            if !super::context_budget::charge(1, 0) {
                break;
            }
            let Some(sc) = rows(doc, "scopes").iter().find(|x| id(x) == cur) else {
                break;
            };
            if let Some(b) = sc["branch_id"].as_str() {
                found.push(b.to_owned());
            }
            cur = s(sc, "parent_id");
        }
        found
    }
    fn candidate(&self, c: &Value, r: &Value, missing: &[String]) -> Value {
        let mut candidate = json!({"claim_ids":[id(c)],"request_id":id(r),"reference_ids":[],"state":"unresolved","reason_codes":missing,"dependency_paths":self.dependency_missing(self.source,"claim",id(c)),"evidence_refs":rows(c,"evidence")});
        if super::descriptions::contains(&c["body"]) {
            candidate["source_constraint"] = c["body"].clone();
        }
        candidate
    }
    fn witness(
        &self,
        c: &Value,
        r: &Value,
        p: &Value,
        kind: &str,
        terms: Vec<Value>,
        missing: Vec<String>,
        extra_claims: &[String],
    ) -> Value {
        let mut claim_ids = vec![id(c).to_owned()];
        claim_ids.extend_from_slice(extra_claims);
        unique(&mut claim_ids);
        let mut evidence = rows(c, "evidence").to_vec();
        for cid in extra_claims {
            if let Some(c) = rows(self.source, "claims").iter().find(|c| id(c) == cid) {
                evidence.extend(rows(c, "evidence").iter().cloned());
            }
        }
        let mut bindings = vec![];
        let alpha = self.alpha_map(c, r);
        for a in rows(&p["atom"], "arguments") {
            if s(&a["value"], "kind") == "described_ref"
                || rows(&r["target"]["atom"], "arguments")
                    .iter()
                    .any(|b| b["role"] == a["role"] && s(&b["value"], "kind") == "described_ref")
            {
                if let Some(b) = rows(&r["target"]["atom"], "arguments")
                    .iter()
                    .find(|b| b["role"] == a["role"])
                {
                    if matches!(s(&a["value"], "kind"), "ref" | "described_ref")
                        && matches!(s(&b["value"], "kind"), "ref" | "described_ref")
                    {
                        let relation = alpha.as_ref().map_or_else(
                            || self.term(self.source, &a["value"], self.question, &b["value"]),
                            |map| self.mapped_term(&a["value"], &b["value"], map),
                        );
                        if relation == Relation::Same
                            || self.description_anchor_credited(
                                s(&a["value"], "id"),
                                s(&b["value"], "id"),
                            )
                        {
                            let mut binding = json!({"source_ref":a["value"]["id"],"question_ref":b["value"]["id"],"relation":if relation==Relation::Same{"same_description"}else{"description_anchor"},"source_term":a["value"],"question_term":b["value"]});
                            if let Some(map) = alpha.as_ref() {
                                if let Some((source_variable, question_variable)) = self
                                    .description_alpha_roots(
                                        s(&a["value"], "id"),
                                        s(&b["value"], "id"),
                                        map,
                                    )
                                {
                                    if source_variable != s(&a["value"], "id")
                                        || question_variable != s(&b["value"], "id")
                                    {
                                        binding["alpha_alias_roots"] = json!({"source_variable":source_variable,"question_variable":question_variable,"relation":"local_same_to_verified_binder"});
                                    }
                                }
                            }
                            if relation != Relation::Same {
                                binding["anchor_evidence"] = self
                                    .description_anchor_evidence(
                                        s(&a["value"], "id"),
                                        s(&b["value"], "id"),
                                    )
                                    .unwrap_or(Value::Null);
                            }
                            bindings.push(binding);
                        }
                    }
                }
            }

            if s(&a["value"], "kind") == "ref" {
                for b in rows(&r["target"]["atom"], "arguments") {
                    if a["role"] == b["role"]
                        && s(&b["value"], "kind") == "ref"
                        && self.term(self.source, &a["value"], self.question, &b["value"])
                            == Relation::Same
                    {
                        bindings.push(json!({"source_ref":a["value"]["id"],"question_ref":b["value"]["id"],"relation":"same"}));
                    }
                }
            }
        }
        if let Some(map) = alpha {
            for (source_ref, question_ref) in map {
                bindings.push(json!({"source_ref":source_ref,"question_ref":question_ref,"relation":"alpha_pattern"}));
            }
        }
        let mut witness = json!({"id":format!("{}:{}:{}:{}",id(r),id(c),kind,claim_ids.join("+")),"kind":kind,"claim_ids":claim_ids,"request_id":id(r),"bindings":bindings,"evidence_refs":evidence,"qualifier_ids":self.claim_qualifiers(c,p),"branch_ids":self.branch(self.source,s(c,"scope_id")),"missing_constraints":missing,"answer_terms":terms,"answer_propositions":[],"requirement_states":[],"conclusion_judgment":p.get("judgment").cloned().unwrap_or(Value::Null),"normative_scope":p.get("normative_scope").cloned().unwrap_or(Value::Null)});
        if super::descriptions::contains(&c["body"]) || super::descriptions::contains(&r["target"])
        {
            witness["source_constraint"] = c["body"].clone();
            if !rows(&witness, "missing_constraints").is_empty() {
                witness["answer_terms"] = json!([]);
            }
        }
        if s(&c["body"], "kind") == "value_restriction" {
            if let Some(domain) = c["body"].get("answer_domain") {
                witness["restriction_domain"] = domain.clone();
            }
        }
        witness
    }
}
fn decimal_key(raw: &str) -> String {
    let negative = raw.starts_with('-');
    let unsigned = raw.trim_start_matches(['-', '+']);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let fraction = fraction.trim_end_matches('0');
    format!(
        "{}{}{}{}",
        if negative && (whole != "0" || !fraction.is_empty()) {
            "-"
        } else {
            ""
        },
        whole,
        if fraction.is_empty() { "" } else { "." },
        fraction
    )
}
fn context_code(kind: &str) -> &str {
    match kind {
        "configuration" => "configuration",
        "time" => "time_role",
        "guard" => "guard",
        _ => "dependency",
    }
}
fn argument<'a>(atom: &'a Value, role: &str) -> Option<&'a Value> {
    rows(atom, "arguments")
        .iter()
        .find(|a| s(a, "role") == role)
        .map(|a| &a["value"])
}
fn normative_conflict(a: &str, b: &str) -> bool {
    matches!(
        (a, b),
        ("allowed", "forbidden")
            | ("forbidden", "allowed")
            | ("required", "forbidden")
            | ("forbidden", "required")
    )
}

#[derive(Default)]
struct Evidence {
    support: Vec<Value>,
    refutation: Vec<Value>,
    partial: Vec<Value>,
    unresolved: Vec<Value>,
    conflicts: Vec<Value>,
    missing: Vec<String>,
    closed: bool,
    assignment_contrasts: Vec<Value>,
}
impl Evidence {
    fn publish(&mut self, w: Value, negative: bool) {
        if negative {
            self.refutation.push(w)
        } else {
            self.support.push(w)
        }
    }
    fn partial(&mut self, w: Value) {
        self.missing
            .extend(strings(rows(&w, "missing_constraints")));
        self.partial.push(w);
    }
}
impl Engine<'_> {
    fn common_quantity_scope_compatible(&self, c: &Value, r: &Value) -> bool {
        if !matches!(s(r, "mode"), "answer" | "event_slot" | "constraint") {
            return true;
        }
        let common = self
            .request_qualifiers(r)
            .iter()
            .filter_map(|id| self.qualifier(self.question, id.as_str()?))
            .any(|q| {
                s(q, "kind") == "configuration"
                    && s(q, "resolution") == "known"
                    && q["body"]["key"] == "quantity_extent"
                    && q["body"]["value"]["kind"] == "text"
                    && q["body"]["value"]["value"] == "común"
            });
        if !common {
            return true;
        }
        let universal: BTreeSet<_> = self
            .binders(self.question, s(r, "scope_id"))
            .into_iter()
            .filter(|b| s(b, "quantifier") == "forall")
            .map(|b| s(b, "variable"))
            .collect();
        let body = &c["body"];
        let empty = Value::Null;
        let (atom, normative_scope) = match s(body, "kind") {
            "proposition" => (
                &body["value"]["atom"],
                body["value"]["normative_scope"].as_str(),
            ),
            "rule" => (
                &body["action"]["atom"],
                body["action"]["normative_scope"].as_str(),
            ),
            "value_restriction" => (&body["target"], body["normative_scope"].as_str()),
            _ => (&empty, None),
        };
        let target = &r["target"];
        let mut pairs = vec![];
        for arg in rows(&target["atom"], "arguments") {
            if matches!(s(&arg["value"], "kind"), "ref" | "described_ref") {
                let actual = argument(atom, s(arg, "role"))
                    .filter(|v| matches!(s(v, "kind"), "ref" | "described_ref"))
                    .map(|v| s(v, "id"));
                pairs.push((s(&arg["value"], "id"), actual));
            }
        }
        for field in ["subject", "event_ref"] {
            if let Some(wanted) = target[field].as_str() {
                pairs.push((wanted, body[field].as_str()));
            }
        }
        if let Some(wanted) = r["normative_scope_filter"].as_str() {
            pairs.push((wanted, normative_scope));
        }
        let method = &target["method_selector"]["value"];
        if s(method, "kind") == "ref" {
            let actual = &body["method"]["value"];
            pairs.push((
                s(method, "id"),
                (s(actual, "kind") == "ref").then(|| s(actual, "id")),
            ));
        }
        let time = &target["time_selector"]["value"];
        if s(time, "kind") == "ref" {
            let actual: Vec<_> = rows(body, "times")
                .iter()
                .filter(|t| t["role"] == target["time_selector"]["role"])
                .collect();
            if actual.is_empty() {
                pairs.push((s(time, "id"), None));
            }
            for t in actual {
                let v = &t["value"];
                pairs.push((s(time, "id"), (s(v, "kind") == "ref").then(|| s(v, "id"))));
            }
        }
        let descriptive =
            super::descriptions::contains(body) || super::descriptions::contains(target);
        let required: Vec<_> = pairs
            .into_iter()
            .filter(|(wanted, _)| {
                universal.contains(wanted)
                    || descriptive
                        && universal.iter().any(|variable| {
                            self.local(self.question, wanted, variable) == Relation::Same
                        })
            })
            .collect();
        if required.is_empty() {
            return false;
        }
        let Some(map) = self.alpha_map(c, r) else {
            return false;
        };
        // A binder in the surrounding scope is not enough: the actual source
        // operand must be that variable (or an asserted local alias) in every
        // required target position. A constant under forall does not cover it.
        required.into_iter().all(|(wanted, actual)| {
            actual.is_some_and(|actual| {
                map.iter().any(|(source_variable, target_variable)| {
                    (target_variable.as_str() == wanted
                        || descriptive
                            && self.local(self.question, wanted, target_variable) == Relation::Same)
                        && self.local(self.source, actual, source_variable) == Relation::Same
                })
            })
        })
    }

    fn direct(
        &self,
        c: &Value,
        r: &Value,
        p: &Value,
        kind: &str,
        extra: &[String],
        out: &mut Evidence,
    ) {
        let target = &r["target"];
        let mode = s(r, "mode");
        if !matches!(mode, "answer" | "truth") {
            return;
        }
        if target.get("atom").is_none() {
            return;
        }
        let temporal_role = super::temporal_slot(s(r, "slot")).map(|(role, _)| role);
        let temporal = temporal_role.is_some();
        let slot = if mode == "truth" || temporal {
            None
        } else {
            Some(s(r, "slot"))
        };
        let relation = self.match_atom(c, r, p, slot);
        if relation == Relation::Different {
            return;
        }
        let judgment = s(p, "judgment");
        let requested = s(target, "judgment");
        let contrary_judgment = normative_conflict(judgment, requested)
            && s(&p["atom"], "polarity") == "positive"
            && s(&target["atom"], "polarity") == "positive";
        if judgment != requested && !contrary_judgment {
            return;
        }
        // A plan is not a denial of occurrence and execution is not inferred from permission.
        if p["status"] != target["status"] {
            return;
        }
        let scope_relation = self.scope_match(r, p);
        if scope_relation == Relation::Different {
            return;
        }
        let mut qids = self.request_qualifiers(r);
        qids.extend(rows(target, "qualifier_ids").iter().cloned());
        let mut source_qids = self.claim_qualifiers(c, p);
        let temporal_terms: Vec<Value> = if temporal {
            source_qids
                .iter()
                .filter_map(|qid| {
                    qid.as_str()
                        .and_then(|qid| self.qualifier(self.source, qid))
                })
                .filter(|q| {
                    s(q, "kind") == "time"
                        && s(q, "resolution") == "known"
                        && Some(s(&q["body"], "role")) == temporal_role
                })
                .map(|q| q["body"]["value"].clone())
                .collect()
        } else {
            vec![]
        };
        if temporal {
            source_qids.retain(|qid| {
                !qid.as_str()
                    .and_then(|qid| self.qualifier(self.source, qid))
                    .is_some_and(|q| {
                        s(q, "kind") == "time"
                            && Some(s(&q["body"], "role")) == temporal_role
                            && s(q, "resolution") == "known"
                    })
            });
        }
        if let Some(map) = self.alpha_map(c, r) {
            let discharged = super::quantified_match::common_quantity_qualifier_ids(
                self.source,
                c,
                self.question,
                r,
                s(self.scope, "namespace"),
                &map,
                &|a, b| self.mapped_term(a, b, &map) == Relation::Same,
            );
            qids.retain(|qid| {
                !qid.as_str()
                    .is_some_and(|id| discharged.iter().any(|x| x == id))
            });
        }
        let (context, mut missing) =
            self.context(self.source, &source_qids, self.question, &qids, true);
        if context == Relation::Different {
            return;
        }
        if relation != Relation::Same {
            missing.push("identity".to_owned());
        }
        if scope_relation != Relation::Same {
            missing.push("normative_target".to_owned());
        }
        let paths = self.dependency_missing(self.source, "claim", id(c));
        if !paths.is_empty() {
            missing.push("dependency".to_owned());
        }
        let terms: Vec<Value> = if temporal {
            temporal_terms
        } else {
            slot.and_then(|slot| argument(&p["atom"], slot))
                .cloned()
                .into_iter()
                .collect()
        };
        if temporal
            && (terms.is_empty()
                || (temporal_role == Some("occurred_at") && s(p, "status") != "occurred")
                || (temporal_role == Some("planned_for") && s(p, "status") != "planned"))
        {
            missing.push("time_role".into());
        }
        if slot.is_some() && terms.is_empty() {
            missing.push("answer_slot".to_owned());
        }
        missing.extend(self.filters(r, p, &terms));
        let branches = self.branch(self.source, s(c, "scope_id"));
        if !branches.is_empty() {
            missing.push("branch_choice".to_owned());
        }
        missing.extend(self.premises(r));
        unique(&mut missing);
        let negative = contrary_judgment || p["atom"]["polarity"] != target["atom"]["polarity"];
        if !negative && !self.common_quantity_scope_compatible(c, r) {
            out.unresolved
                .push(self.candidate(c, r, &["common_quantity_scope".into()]));
            return;
        }
        if relation == Relation::Conflict
            || context == Relation::Conflict
            || scope_relation == Relation::Conflict
        {
            out.conflicts.push(
                json!({"support_witness_ids":[],"refutation_witness_ids":[],"kind":"potential"}),
            );
        }
        let wk = if !branches.is_empty() {
            "conditional_branch"
        } else if missing.is_empty() {
            kind
        } else {
            "partial_slot"
        };
        let w = self.witness(c, r, p, wk, terms, missing.clone(), extra);
        if missing.is_empty() {
            out.publish(w, negative);
        } else {
            // Identity and unknown context can change whether this conclusion applies at all.
            if missing.iter().any(|m| {
                matches!(
                    m.as_str(),
                    "identity"
                        | "normative_target"
                        | "dependency"
                        | "guard"
                        | "configuration"
                        | "time_role"
                )
            }) {
                let mut candidate = self.candidate(c, r, &missing);
                candidate["reason_codes"] = json!(missing);
                out.unresolved.push(candidate);
            }
            // A requested association is a restrictive target, not a useful
            // answer slot with an optional identity warning.
            // Reuse only a complete map proven for this exact claim/request.
            // It is a local quantified correspondence, never an individual anchor.
            let alpha = if relation == Relation::Same
                && (super::descriptions::contains(&p["atom"])
                    || super::descriptions::contains(&r["target"]))
            {
                self.alpha_map(c, r)
            } else {
                None
            };
            let unmatched_association = !self.required_associations_with_map(p, r, alpha.as_ref());
            if !unmatched_association && self.required_named_targets_with_map(p, r, alpha.as_ref())
            {
                out.partial(w);
            }
        }
    }
    // An occurrence's requested time is the answer coordinate, not an
    // additional condition on whether that same occurrence happened. This
    // local premise proof never strips qualifiers from claims or witnesses.
    fn occurrence_premise_qualifiers(
        &self,
        c: &Value,
        r: &Value,
        premise: &Value,
        question_ids: &[Value],
    ) -> Vec<Value> {
        fn unquantified_document_chain(doc: &Value, owner: &Value) -> bool {
            let mut cursor = s(owner, "scope_id");
            let mut seen = BTreeSet::new();
            while seen.insert(cursor.to_owned()) {
                let Some(scope) = rows(doc, "scopes").iter().find(|scope| id(scope) == cursor)
                else {
                    return false;
                };
                if s(scope, "kind") != "document" || !scope["binding"].is_null() {
                    return false;
                }
                if scope["parent_id"].is_null() {
                    return true;
                }
                cursor = s(scope, "parent_id");
            }
            false
        }
        let ids = self.claim_qualifiers(c, &c["body"]["value"]);
        let Some((role, year)) = super::temporal_slot(s(r, "slot")) else {
            return ids;
        };
        let p = &premise["body"]["value"];
        if s(r, "mode") != "answer"
            || role != "occurred_at"
            || s(p, "judgment") != "fact"
            || s(&r["target"], "judgment") != "fact"
            || s(p, "status") != "occurred"
            || s(&r["target"], "status") != "occurred"
            || self.atom(
                self.question,
                &r["target"]["atom"],
                self.question,
                &p["atom"],
                None,
            ) != Relation::Same
            || [
                (self.source, c),
                (self.question, premise),
                (self.question, r),
            ]
            .iter()
            .any(|(doc, owner)| !unquantified_document_chain(doc, owner))
            || question_ids.iter().any(|qid| {
                qid.as_str()
                    .and_then(|id| self.qualifier(self.question, id))
                    .is_some_and(|q| s(q, "kind") == "time")
            })
        {
            return ids;
        }
        let times: Vec<_> = ids
            .iter()
            .filter(|qid| {
                qid.as_str()
                    .and_then(|id| self.qualifier(self.source, id))
                    .is_some_and(|q| {
                        s(q, "kind") == "time"
                            && s(q, "resolution") == "known"
                            && s(&q["body"], "role") == role
                            && s(&q["body"]["value"], "kind") == "time"
                            && (!year || s(&q["body"]["value"], "format") == "year")
                    })
            })
            .collect();
        if times.len() != 1 {
            return ids;
        }
        let answered = times[0].clone();
        if !self.scope["time_selector"].is_null() {
            let Some(q) = answered
                .as_str()
                .and_then(|id| self.qualifier(self.source, id))
            else {
                return ids;
            };
            if self.scope["time_selector"]["role"] != role
                || self.term(
                    self.source,
                    &q["body"]["value"],
                    self.question,
                    &self.scope["time_selector"]["value"],
                ) != Relation::Same
            {
                return ids;
            }
        }
        ids.into_iter().filter(|qid| *qid != answered).collect()
    }

    fn premises(&self, r: &Value) -> Vec<String> {
        for pid in rows(r, "premise_ids").iter().filter_map(Value::as_str) {
            let Some(p) = rows(self.question, "claims")
                .iter()
                .find(|c| id(c) == pid && s(&c["body"], "kind") == "proposition")
            else {
                return vec!["premise".into()];
            };
            if !self.branch(self.question, s(p, "scope_id")).is_empty()
                || !self
                    .dependency_missing(self.question, "claim", id(p))
                    .is_empty()
            {
                return vec!["premise".into()];
            }
            let qids = self.premise_qualifiers(r, p);
            let demonstrated = rows(self.source, "claims").iter().any(|c| {
                let source_ids = self.occurrence_premise_qualifiers(c, r, p, &qids);
                let mut source_proposition = c["body"]["value"].clone();
                if let Some(ids) = source_proposition["qualifier_ids"].as_array_mut() {
                    ids.retain(|qid| source_ids.contains(qid));
                }
                s(&c["body"], "kind") == "proposition"
                    && self.branch(self.source, s(c, "scope_id")).is_empty()
                    && self.proposition(
                        self.source,
                        &source_proposition,
                        self.question,
                        &p["body"]["value"],
                        None,
                    ) == Relation::Same
                    && self
                        .context(self.source, &source_ids, self.question, &qids, true)
                        .0
                        == Relation::Same
                    && self
                        .dependency_missing(self.source, "claim", id(c))
                        .is_empty()
                    && rows(self.source_report, "candidates")
                        .iter()
                        .any(|candidate| {
                            !matches!(s(candidate, "state"), "unresolved" | "blocked")
                                && rows(candidate, "claim_ids")
                                    .iter()
                                    .any(|x| x.as_str() == Some(id(c)))
                        })
            });
            if !demonstrated {
                return vec!["premise".into()];
            }
        }
        vec![]
    }
    fn weak_reference_anchor(&self, actual: &str, wanted: &str) -> bool {
        let relation = self.identity(self.source, actual, self.question, wanted);
        if relation == Relation::Same {
            return true;
        }
        relation == Relation::Unknown
            && self.partial_named_anchor(actual, wanted, 0)
            && rows(self.source, "references").iter().any(|a| {
                !s(&a["name"], "quote").is_empty()
                    && self.local(self.source, actual, id(a)) == Relation::Same
                    && rows(self.question, "references").iter().any(|b| {
                        self.local(self.question, wanted, id(b)) == Relation::Same
                            && a["name"]["quote"] == b["name"]["quote"]
                            && a["denotation"] == b["denotation"]
                    })
            })
    }
    fn weak_atom_anchors(&self, actual: &Value, wanted: &Value) -> bool {
        actual["predicate"] == wanted["predicate"]
            && rows(actual, "arguments").len() == rows(wanted, "arguments").len()
            && rows(wanted, "arguments").iter().all(|arg| {
                argument(actual, s(arg, "role")).is_some_and(|value| {
                    self.term(self.source, value, self.question, &arg["value"]) == Relation::Same
                        || s(value, "kind") == "ref"
                            && s(&arg["value"], "kind") == "ref"
                            && self.weak_reference_anchor(s(value, "id"), s(&arg["value"], "id"))
                })
            })
    }
    fn premise_refutations(&self, r: &Value, out: &mut Evidence) {
        for pid in rows(r, "premise_ids").iter().filter_map(Value::as_str) {
            let Some(premise) = rows(self.question, "claims")
                .iter()
                .find(|c| id(c) == pid && s(&c["body"], "kind") == "proposition")
            else {
                continue;
            };
            let target = &premise["body"]["value"];
            for c in rows(self.source, "claims")
                .iter()
                .filter(|c| s(&c["body"], "kind") == "proposition")
            {
                let p = &c["body"]["value"];
                if p["atom"]["polarity"] == target["atom"]["polarity"]
                    || p["judgment"] != target["judgment"]
                    || p["status"] != target["status"]
                {
                    continue;
                }
                let mut positive = p.clone();
                positive["atom"]["polarity"] = target["atom"]["polarity"].clone();
                if p["basis"] != target["basis"] {
                    continue;
                }
                // A potential correction needs compatible explicit anchors; different
                // named courses/people are not made equal by an unknown identity result.
                let anchors = rows(&positive["atom"], "arguments")
                    .iter()
                    .filter_map(|arg| {
                        let other = argument(&target["atom"], s(arg, "role"))?;
                        (s(&arg["value"], "kind") == "ref" && s(other, "kind") == "ref")
                            .then_some((s(&arg["value"], "id"), s(other, "id")))
                    })
                    .chain(
                        (!p["normative_scope"].is_null() && !target["normative_scope"].is_null())
                            .then_some((s(p, "normative_scope"), s(target, "normative_scope"))),
                    );
                if anchors.into_iter().any(|(a, b)| {
                    let an = self
                        .reference(self.source, a)
                        .map(|r| s(&r["name"], "quote"))
                        .unwrap_or("");
                    let bn = self
                        .reference(self.question, b)
                        .map(|r| s(&r["name"], "quote"))
                        .unwrap_or("");
                    !an.is_empty()
                        && !bn.is_empty()
                        && an != bn
                        && self.identity(self.source, a, self.question, b) != Relation::Same
                }) {
                    continue;
                }
                let atom_relation = self.atom(
                    self.source,
                    &positive["atom"],
                    self.question,
                    &target["atom"],
                    None,
                );
                if atom_relation != Relation::Same
                    && !self.weak_atom_anchors(&positive["atom"], &target["atom"])
                {
                    continue;
                }
                let scope_relation = if p["normative_scope"].is_null()
                    && target["normative_scope"].is_null()
                {
                    Relation::Same
                } else if p["normative_scope"].is_null() || target["normative_scope"].is_null() {
                    Relation::Unknown
                } else {
                    self.identity(
                        self.source,
                        s(p, "normative_scope"),
                        self.question,
                        s(target, "normative_scope"),
                    )
                };
                if scope_relation != Relation::Same {
                    let a = self.reference(self.source, s(p, "normative_scope"));
                    let b = self.reference(self.question, s(target, "normative_scope"));
                    let anchored = a.zip(b).is_some_and(|(a, b)| {
                        let name = s(&a["name"], "quote");
                        !name.is_empty()
                            && name == s(&b["name"], "quote")
                            && a["denotation"] == b["denotation"]
                            && s(self.source, "domain_namespace") == s(self.scope, "namespace")
                            && s(self.question, "domain_namespace") == s(self.scope, "namespace")
                    });
                    if !anchored
                        || matches!(scope_relation, Relation::Different | Relation::Conflict)
                    {
                        continue;
                    }
                }
                let relation =
                    self.proposition(self.source, &positive, self.question, target, None);
                let (context, context_missing) = self.context(
                    self.source,
                    &self.claim_qualifiers(c, p),
                    self.question,
                    &self.premise_qualifiers(r, premise),
                    true,
                );
                if matches!(relation, Relation::Different | Relation::Conflict)
                    || matches!(context, Relation::Different | Relation::Conflict)
                {
                    continue;
                }
                let mut missing = vec!["premise".into(), "answer_slot".into()];
                if relation != Relation::Same {
                    missing.push("identity".into());
                }
                if scope_relation != Relation::Same {
                    missing.push("normative_target".into());
                }
                missing.extend(context_missing);
                if !self
                    .dependency_missing(self.source, "claim", id(c))
                    .is_empty()
                {
                    missing.push("dependency".into());
                }
                if !self.branch(self.source, s(c, "scope_id")).is_empty() {
                    missing.push("branch_choice".into());
                }
                unique(&mut missing);
                let correction_proven = relation == Relation::Same
                    && context == Relation::Same
                    && missing
                        .iter()
                        .all(|reason| matches!(reason.as_str(), "premise" | "answer_slot"));
                let witness_kind = if correction_proven {
                    "premise_refutation"
                } else {
                    "partial_premise_refutation"
                };
                let mut witness = self.witness(c, r, p, witness_kind, vec![], missing.clone(), &[]);
                if atom_relation != Relation::Same {
                    witness["answer_propositions"] = json!([p]);
                }
                out.partial(witness);
                if !correction_proven {
                    out.unresolved.push(self.candidate(c, r, &missing));
                }
            }
        }
    }
    // Absence is unbounded only in historical vocabularies. Unknown is never
    // a license to close every value of a role in a new extraction.
    fn restriction_domain_known(&self, body: &Value) -> bool {
        match body.get("answer_domain") {
            None => {
                self.vocabulary["restriction_domain_profile"]["version"]
                    != "explicit_answer_domains_v1"
            }
            Some(domain) => matches!(s(domain, "kind"), "all" | "lexical" | "semantic"),
        }
    }
    fn restriction_domain_contains(&self, body: &Value, value: &Value) -> bool {
        if !self.restriction_domain_known(body) {
            return false;
        }
        let domain = &body["answer_domain"];
        if domain.is_null() || s(domain, "kind") == "all" {
            return true;
        }
        if !matches!(s(value, "kind"), "ref" | "described_ref")
            || s(value, "kind") == "described_ref"
                && !self.descriptor_consistent(self.question, value)
        {
            return false;
        }
        let Some(reference) = self.reference(self.question, s(value, "id")) else {
            return false;
        };
        if !reference["binding_scope_id"].is_null()
            || !reference["anaphora"].is_null()
            || self.local(self.question, id(reference), id(reference)) != Relation::Same
        {
            return false;
        }
        match s(domain, "kind") {
            "lexical" => {
                !reference["declared_class"].is_null()
                    && reference["declared_class"]["quote"] == domain["class_quote"]["quote"]
            }
            "semantic" => {
                !reference["semantic_type"].is_null()
                    && reference["semantic_type"]["type_id"] == domain["type_id"]
            }
            _ => false,
        }
    }
    fn restriction_domain_applies(&self, body: &Value, request: &Value) -> bool {
        if !self.restriction_domain_known(body) {
            return false;
        }
        let domain = &body["answer_domain"];
        if domain.is_null() || s(domain, "kind") == "all" {
            return true;
        }
        if let Some(value) = argument(&request["target"]["atom"], s(body, "answer_role")) {
            return self.restriction_domain_contains(body, value);
        }
        let filter = &request["class_filter"];
        s(request, "slot") == s(body, "answer_role")
            && domain["kind"] == filter["kind"]
            && match s(domain, "kind") {
                "lexical" => domain["class_quote"]["quote"] == filter["class_quote"]["quote"],
                "semantic" => domain["type_id"] == filter["type_id"],
                _ => false,
            }
    }
    fn restrictions(&self, c: &Value, r: &Value, out: &mut Evidence) {
        let body = &c["body"];
        let target = &r["target"];
        if target.get("atom").is_none() {
            return;
        }
        let role = s(body, "answer_role");
        let relation = self.atom(
            self.source,
            &body["target"],
            self.question,
            &target["atom"],
            Some(role),
        );
        if relation == Relation::Different
            || body["judgment"] != target["judgment"]
            || body["status"] != target["status"]
        {
            return;
        }
        if !self.restriction_domain_applies(body, r) {
            out.unresolved
                .push(self.candidate(c, r, &["restriction_domain".into()]));
            return;
        }
        if s(&body["target"], "polarity") == "positive"
            && !self.common_quantity_scope_compatible(c, r)
        {
            out.unresolved
                .push(self.candidate(c, r, &["common_quantity_scope".into()]));
            return;
        }
        let p = json!({"atom":body["target"],"judgment":body["judgment"],"status":body["status"],"basis":body["basis"],"normative_scope":body["normative_scope"],"qualifier_ids":c["qualifier_ids"]});
        if !self.required_associations_apply(&p, r) {
            out.unresolved
                .push(self.candidate(c, r, &["association_context".into()]));
            return;
        }
        if self.scope_match(r, &p) == Relation::Different {
            return;
        }
        let mut missing = self.filters(r, &p, rows(body, "members"));
        let mut qs = self.request_qualifiers(r);
        qs.extend(rows(target, "qualifier_ids").iter().cloned());
        let (context, context_missing) = self.context(
            self.source,
            &self.claim_qualifiers(c, &Value::Null),
            self.question,
            &qs,
            true,
        );
        if context == Relation::Different {
            return;
        }
        missing.extend(context_missing);
        if relation != Relation::Same {
            missing.push("identity".into());
        }
        if self.scope_match(r, &p) != Relation::Same {
            missing.push("normative_target".into());
        }
        if !self
            .dependency_missing(self.source, "claim", id(c))
            .is_empty()
        {
            missing.push("dependency".into());
        }
        if !self.branch(self.source, s(c, "scope_id")).is_empty() {
            missing.push("branch_choice".into());
        }
        if s(r, "mode") == "constraint" && s(r, "slot") == role {
            unique(&mut missing);
            let w = self.witness(
                c,
                r,
                &p,
                "direct",
                rows(body, "members").to_vec(),
                missing.clone(),
                &[],
            );
            if missing.is_empty() {
                out.support.push(w);
                out.closed = true;
            } else {
                out.partial(w);
                out.unresolved.push(self.candidate(c, r, &missing));
            }
            return;
        }
        if let Some(value) = argument(&target["atom"], role) {
            let comparisons: Vec<_> = rows(body, "members")
                .iter()
                .map(|m| self.term(self.source, m, self.question, value))
                .collect();
            if !comparisons.is_empty()
                && comparisons.iter().all(|x| *x == Relation::Different)
                && missing.is_empty()
            {
                let w = self.witness(
                    c,
                    r,
                    &p,
                    "restriction_exclusion",
                    vec![value.clone()],
                    vec![],
                    &[],
                );
                // Restriction target polarity names the limited judgment, never its opposite.
                out.publish(w, body["target"]["polarity"] == target["atom"]["polarity"]);
            } else {
                if comparisons
                    .iter()
                    .any(|x| matches!(x, Relation::Unknown | Relation::Conflict))
                {
                    missing.push("identity".into());
                    out.unresolved.push(self.candidate(c, r, &missing));
                }
                missing.push("answer_slot".into());
                unique(&mut missing);
                out.partial(self.witness(
                    c,
                    r,
                    &p,
                    "partial_slot",
                    rows(body, "members").to_vec(),
                    missing,
                    &[],
                ));
            }
        } else if s(r, "slot") == role {
            missing.push("answer_slot".into());
            unique(&mut missing);
            out.partial(self.witness(
                c,
                r,
                &p,
                "partial_slot",
                rows(body, "members").to_vec(),
                missing,
                &[],
            ));
        }
    }
    fn configuration_label<'b>(&self, doc: &'b Value, value: &'b Value) -> Option<&'b str> {
        match s(value, "kind") {
            "text" => value["value"].as_str(),
            "ref" => {
                let reference = self.reference(doc, s(value, "id"))?;
                if !reference["binding_scope_id"].is_null()
                    || !reference["anaphora"].is_null()
                    || self.local(doc, id(reference), id(reference)) != Relation::Same
                {
                    return None;
                }
                reference["name"]["quote"]
                    .as_str()
                    .filter(|name| !name.is_empty())
            }
            _ => None,
        }
    }
    fn configuration_label_matches(&self, doc: &Value, value: &Value, label: &str) -> bool {
        if s(value, "kind") == "text" {
            return s(value, "value") == label;
        }
        let Some(reference) = self.reference(doc, s(value, "id")) else {
            return false;
        };
        s(value, "kind") == "ref"
            && reference["binding_scope_id"].is_null()
            && reference["anaphora"].is_null()
            && self.local(doc, id(reference), id(reference)) == Relation::Same
            && rows(doc, "references").iter().any(|alias| {
                self.local(doc, id(reference), id(alias)) == Relation::Same
                    && s(&alias["name"], "quote") == label
            })
    }
    fn admission_context(&self, c: &Value, r: &Value, out: &mut Evidence) {
        let profile = &self.vocabulary["assignment_projection"];
        let target = &r["target"];
        let declared = rows(self.vocabulary, "relevance_links").iter().any(|link| {
            link["source_predicate"] == "admits"
                && link["target_predicate"] == "quantity"
                && link["bindings"] == json!([["source.subject", "target.subject"]])
                && rows(link, "carry_roles").contains(&json!("source.object"))
        });
        if !declared
            || profile["version"] != "uses_qualifier_assignment_v1"
            || s(r, "mode") != "answer"
            || s(r, "slot") != "amount"
            || s(&target["atom"], "predicate") != "quantity"
            || s(&target["atom"], "polarity") != "positive"
            || argument(&target["atom"], "amount").is_some()
            || s(&r["basis_requirement"], "kind") != "exact"
            || r["basis_requirement"]["value"] != "assignment"
        {
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
            || p["judgment"] != target["judgment"]
            || p["status"] != target["status"]
            || !(s(&p, "basis") == "description"
                || self.context_v2 && s(&p, "basis") == "capability")
        {
            return;
        }
        if closed {
            if s(body, "closure") != "closed"
                || s(&p["atom"], "polarity") != "positive"
                || s(body, "answer_role") != "object"
                || rows(body, "members").is_empty()
            {
                return;
            }
        } else if s(body, "kind") != "proposition" || s(&p["atom"], "polarity") != "negative" {
            return;
        }
        let (Some(subject), Some(wanted_subject)) = (
            argument(&p["atom"], "subject"),
            argument(&target["atom"], "subject"),
        ) else {
            return;
        };
        if s(subject, "kind") != "ref"
            || s(wanted_subject, "kind") != "ref"
            || !self.weak_reference_anchor(s(subject, "id"), s(wanted_subject, "id"))
        {
            return;
        }
        let ids = self.request_qualifiers(r);
        let desired: Vec<_> = ids
            .iter()
            .filter_map(|id| self.qualifier(self.question, id.as_str()?))
            .filter(|q| {
                s(q, "kind") == "configuration"
                    && rows(profile, "context_keys").contains(&q["body"]["key"])
            })
            .collect();
        if desired.len() != 1 || s(desired[0], "resolution") != "known" {
            return;
        }
        let desired = desired[0];
        if closed && !self.restriction_domain_contains(body, &desired["body"]["value"]) {
            out.unresolved
                .push(self.candidate(c, r, &["restriction_domain".into()]));
            return;
        }
        let Some(label) = self.configuration_label(self.question, &desired["body"]["value"]) else {
            return;
        };
        if closed {
            if rows(body, "members")
                .iter()
                .any(|member| self.configuration_label_matches(self.source, member, label))
            {
                return;
            }
            if rows(body, "members")
                .iter()
                .any(|member| self.configuration_label(self.source, member).is_none())
            {
                return;
            }
        } else if !argument(&p["atom"], "object")
            .is_some_and(|v| self.configuration_label_matches(self.source, v, label))
        {
            return;
        }
        if self.context_v2 {
            let wanted = &desired["body"]["value"];
            let compatible = if closed {
                rows(body, "members")
                    .iter()
                    .all(|member| self.admission_setting_context(member, wanted, false))
            } else {
                argument(&p["atom"], "object")
                    .is_some_and(|actual| self.admission_setting_context(actual, wanted, true))
            };
            if !compatible {
                out.unresolved
                    .push(self.candidate(c, r, &["admission_setting_context".into()]));
                return;
            }
        }
        // Keep every unmatched source condition. A closed set is not made
        // unconditional by treating its configuration qualifiers as data.
        let source_ids: Vec<_> = self
            .claim_qualifiers(c, &p)
            .into_iter()
            .filter(|qid| {
                !self
                    .qualifier(self.source, qid.as_str().unwrap_or(""))
                    .is_some_and(|q| {
                        s(q, "kind") == "configuration"
                            && q["body"]["key"] == desired["body"]["key"]
                            && s(q, "resolution") == "known"
                            && self.configuration_label_matches(
                                self.source,
                                &q["body"]["value"],
                                label,
                            )
                    })
            })
            .collect();
        let wanted_ids: Vec<_> = ids
            .iter()
            .filter(|qid| qid.as_str() != Some(id(desired)))
            .cloned()
            .collect();
        let (context, missing) =
            self.context(self.source, &source_ids, self.question, &wanted_ids, true);
        if context == Relation::Different || self.scope_match(r, &p) == Relation::Different {
            return;
        }
        let mut debt = vec![];
        if context != Relation::Same {
            debt.extend(missing);
        }
        if self.scope_match(r, &p) != Relation::Same {
            debt.push("normative_target".into());
        }
        if !self.branch(self.source, s(c, "scope_id")).is_empty() {
            debt.push("branch_choice".into());
        }
        if !self
            .dependency_missing(self.source, "claim", id(c))
            .is_empty()
        {
            debt.push("dependency".into());
        }
        debt.extend(self.premises(r));
        // Adapt only the subject for contextual checks; this is not an inferred
        // numeric proposition and is never published as an assignment.
        let subject_context = json!({"atom":{"arguments":[{"role":"subject","value":subject}]},"normative_scope":p["normative_scope"]});
        if !self.required_associations_apply(&subject_context, r) {
            debt.push("association_context".into());
        }
        if !debt.is_empty() {
            unique(&mut debt);
            out.unresolved.push(self.candidate(c, r, &debt));
            return;
        }
        let mut missing = vec![
            "admission_only".into(),
            "answer_slot".into(),
            "basis".into(),
        ];
        let identity = self.term(self.source, subject, self.question, wanted_subject);
        if identity != Relation::Same {
            missing.push("identity".into());
        }
        if closed {
            missing.push("admission_membership_unknown".into());
        }
        unique(&mut missing);
        let mut witness = self.witness(c, r, &p, "partial_relevance", vec![], missing.clone(), &[]);
        witness["admission_context"] = json!({"requested_setting":desired["body"]["value"],"actual_use_inferred":false,"effect":if closed {"closed_membership_unknown"}else{"negative_admission"}});
        if closed {
            witness["admission_constraint"] = body.clone();
            witness["conclusion_judgment"] = Value::Null;
        } else {
            witness["answer_propositions"] = json!([p]);
        }
        out.partial(witness);
        if closed || identity != Relation::Same {
            out.unresolved.push(self.candidate(c, r, &missing));
        }
    }
    fn scoped_assignment(&self, c: &Value, r: &Value, out: &mut Evidence) {
        let profile = &self.vocabulary["assignment_projection"];
        let p = &c["body"]["value"];
        let target = &r["target"];
        if profile["version"] != "uses_qualifier_assignment_v1"
            || s(r, "mode") != "answer"
            || s(r, "slot") != "amount"
            || s(&target["atom"], "predicate") != "quantity"
            || s(&target["atom"], "polarity") != "positive"
            || argument(&target["atom"], "amount").is_some()
            || s(&p["atom"], "predicate") != "uses"
            || s(&p["atom"], "polarity") != "positive"
            || s(p, "judgment") != "fact"
            || s(p, "status") != "state"
            || p["judgment"] != target["judgment"]
            || p["status"] != target["status"]
            || !matches!(s(p, "basis"), "description" | "assignment")
        {
            return;
        }
        let (Some(subject), Some(setting), Some(wanted_subject), Some(attribute)) = (
            argument(&p["atom"], "subject"),
            argument(&p["atom"], "object"),
            argument(&target["atom"], "subject"),
            argument(&target["atom"], "attribute"),
        ) else {
            return;
        };
        if s(subject, "kind") != "ref"
            || s(wanted_subject, "kind") != "ref"
            || s(attribute, "kind") != "symbol"
        {
            return;
        }
        let relation = self.term(self.source, subject, self.question, wanted_subject);
        let shared_name = rows(self.source, "references").iter().any(|a| {
            !s(&a["name"], "quote").is_empty()
                && self.local(self.source, s(subject, "id"), id(a)) == Relation::Same
                && rows(self.question, "references").iter().any(|b| {
                    self.local(self.question, s(wanted_subject, "id"), id(b)) == Relation::Same
                        && a["name"]["quote"] == b["name"]["quote"]
                        && a["denotation"] == b["denotation"]
                })
        });
        if relation != Relation::Same
            && !(relation == Relation::Unknown
                && shared_name
                && self.partial_named_anchor(s(subject, "id"), s(wanted_subject, "id"), 0))
        {
            return;
        }
        let source_ids = self.claim_qualifiers(c, p);
        let request_ids = self.request_qualifiers(r);
        let is_setting = |q: &Value| {
            s(q, "kind") == "configuration"
                && rows(profile, "context_keys").contains(&q["body"]["key"])
        };
        let requested: Vec<_> = request_ids
            .iter()
            .filter_map(|id| self.qualifier(self.question, id.as_str()?))
            .filter(|q| is_setting(q))
            .collect();
        if requested.len() != 1 || s(requested[0], "resolution") != "known" {
            return;
        }
        let desired = requested[0];
        let (Some(actual_label), Some(wanted_label)) = (
            self.configuration_label(self.source, setting),
            self.configuration_label(self.question, &desired["body"]["value"]),
        ) else {
            return;
        };
        let values: Vec<_> = source_ids
            .iter()
            .filter_map(|id| self.qualifier(self.source, id.as_str()?))
            .filter(|q| s(q, "kind") == "configuration" && q["body"]["key"] == attribute["id"])
            .collect();
        if values.is_empty() {
            return;
        }
        let mut debt = vec![];
        if values
            .iter()
            .any(|q| s(q, "resolution") != "known" || s(&q["body"]["value"], "kind") != "quantity")
        {
            debt.push("assignment_value".into());
        } else if values.iter().skip(1).any(|q| {
            self.term(
                self.source,
                &values[0]["body"]["value"],
                self.source,
                &q["body"]["value"],
            ) != Relation::Same
        }) {
            debt.push("assignment_values_conflict".into());
        }
        let settings: Vec<_> = source_ids
            .iter()
            .filter_map(|id| self.qualifier(self.source, id.as_str()?))
            .filter(|q| is_setting(q))
            .collect();
        if settings.iter().any(|q| {
            s(q, "resolution") != "known"
                || self.configuration_label(self.source, &q["body"]["value"]) != Some(actual_label)
        }) {
            debt.push("assignment_setting_conflict".into());
        }
        let source_context: Vec<_> = source_ids
            .iter()
            .filter(|qid| {
                self.qualifier(self.source, qid.as_str().unwrap_or(""))
                    .is_some_and(|q| {
                        !is_setting(q)
                            && !(s(q, "kind") == "configuration"
                                && q["body"]["key"] == attribute["id"])
                    })
            })
            .cloned()
            .collect();
        let wanted_context: Vec<_> = request_ids
            .iter()
            .filter(|qid| qid.as_str() != Some(id(desired)))
            .cloned()
            .collect();
        let (context, missing) = self.context(
            self.source,
            &source_context,
            self.question,
            &wanted_context,
            true,
        );
        if context == Relation::Different || self.scope_match(r, p) == Relation::Different {
            return;
        }
        if context != Relation::Same {
            debt.extend(missing);
        }
        if self.scope_match(r, p) != Relation::Same {
            debt.push("normative_target".into());
        }
        if !self.branch(self.source, s(c, "scope_id")).is_empty() {
            debt.push("branch_choice".into());
        }
        if !self
            .dependency_missing(self.source, "claim", id(c))
            .is_empty()
        {
            debt.push("dependency".into());
        }
        debt.extend(self.premises(r));
        if !debt.is_empty() {
            unique(&mut debt);
            out.unresolved.push(self.candidate(c, r, &debt));
            return;
        }
        let amount = values[0]["body"]["value"].clone();
        let derived = json!({"atom":{"predicate":"quantity","arguments":[{"role":"subject","value":subject},{"role":"attribute","value":{"kind":"symbol","id":attribute["id"],"evidence":values[0]["evidence"]}},{"role":"amount","value":amount}],"polarity":"positive","evidence":p["atom"]["evidence"]},"judgment":"fact","status":"state","basis":"assignment","normative_scope":p["normative_scope"],"qualifier_ids":[]});
        if !self.required_associations_apply(&derived, r) {
            out.unresolved
                .push(self.candidate(c, r, &["association_context".into()]));
            return;
        }
        let same_coordinate = actual_label == wanted_label
            && settings
                .iter()
                .all(|q| q["body"]["key"] == desired["body"]["key"]);
        let mut missing = self.filters(r, &derived, std::slice::from_ref(&amount));
        if relation != Relation::Same {
            missing.push("identity".into());
        }
        let (kind, terms) = if same_coordinate {
            ("scoped_assignment", vec![amount])
        } else {
            missing.extend(["configuration".into(), "answer_slot".into()]);
            ("partial_relevance", vec![])
        };
        unique(&mut missing);
        let mut witness = self.witness(c, r, &derived, kind, terms, missing.clone(), &[]);
        witness["answer_propositions"] = json!([p]);
        witness["assignment_context"] = json!({"key":desired["body"]["key"],"source_setting":setting,"requested_setting":desired["body"]["value"],"labels_match":same_coordinate,"quantity_qualifier_ids":values.iter().map(|q|id(q)).collect::<Vec<_>>()});
        if !same_coordinate {
            out.assignment_contrasts.push(witness);
        } else if missing.is_empty() {
            out.support.push(witness);
        } else {
            out.partial(witness);
        }
    }
    fn documentary_anchor_context(&self, c: &Value, r: &Value, out: &mut Evidence) {
        let p = &c["body"]["value"];
        let source = &p["atom"];
        let target = &r["target"]["atom"];
        if s(r, "mode") != "answer"
            || s(p, "judgment") != "fact"
            || s(p, "status") != "state"
            || s(source, "polarity") != "positive"
            || s(&r["target"], "judgment") != "fact"
            || s(target, "polarity") != "positive"
            || !matches!(s(p, "basis"), "description" | "observation")
            || !matches!(
                (s(target, "predicate"), s(r, "slot")),
                ("property", "value") | ("quantity", "amount")
            )
        {
            return;
        }
        let (Some(subject), Some(wanted), Some(attribute), Some(wanted_attribute)) = (
            argument(source, "subject"),
            argument(target, "subject"),
            argument(source, "attribute"),
            argument(target, "attribute"),
        ) else {
            return;
        };
        if s(subject, "kind") != "ref"
            || s(wanted, "kind") != "ref"
            || self.term(self.source, attribute, self.question, wanted_attribute) != Relation::Same
        {
            return;
        }
        let (Some(actual), Some(pattern)) = (
            self.reference(self.source, s(subject, "id")),
            self.reference(self.question, s(wanted, "id")),
        ) else {
            return;
        };
        if !actual["name"].is_null()
            || !pattern["name"].is_null()
            || s(actual, "denotation") != "group"
            || s(pattern, "denotation") != "group"
            || actual["declared_class"].is_null()
            || pattern["declared_class"].is_null()
            || actual["declared_class"]["quote"] != pattern["declared_class"]["quote"]
            || actual["semantic_type"]["type_id"] != pattern["semantic_type"]["type_id"]
            || !actual["anaphora"].is_null()
            || !pattern["anaphora"].is_null()
            || !actual["binding_scope_id"].is_null()
            || !pattern["binding_scope_id"].is_null()
            || rows(actual, "links").len() != 1
            || rows(pattern, "links").len() != 1
        {
            return;
        }
        let source_link = &actual["links"][0];
        let question_link = &pattern["links"][0];
        for row in rows(&self.vocabulary["documentary_anchor_context"], "rows") {
            let Some(link) = rows(self.vocabulary, "relevance_links")
                .iter()
                .find(|l| l["id"] == row["link_id"])
            else {
                continue;
            };
            if link["source_predicate"] != source["predicate"]
                || link["target_predicate"] != target["predicate"]
                || row["attribute_id"] != attribute["id"]
                || row["target_status"] != r["target"]["status"]
                || row["source_link_kind"] != source_link["kind"]
                || row["question_link_kind"] != question_link["kind"]
            {
                continue;
            }
            let source_anchor = json!({"kind":"ref","id":source_link["target"]});
            let question_anchor = json!({"kind":"ref","id":question_link["target"]});
            let (compatible, named) =
                self.qualified_relevance_binding(&source_anchor, &question_anchor);
            if !compatible || !named {
                continue;
            }
            let source_ids = self.claim_qualifiers(c, p);
            let target_ids = self.request_qualifiers(r);
            let Some(consumed) = self.known_configuration(
                self.source,
                &source_ids,
                rows(row, "source_configuration"),
            ) else {
                out.unresolved
                    .push(self.candidate(c, r, &["documentary_anchor_context".into()]));
                continue;
            };
            let remaining: Vec<_> = source_ids
                .iter()
                .filter(|v| {
                    if !consumed.contains(v.as_str().unwrap_or("")) {
                        return true;
                    }
                    let Some(q) = v.as_str().and_then(|id| self.qualifier(self.source, id)) else {
                        return true;
                    };
                    target_ids
                        .iter()
                        .filter_map(|v| self.qualifier(self.question, v.as_str()?))
                        .any(|wanted| {
                            s(wanted, "kind") == "configuration"
                                && wanted["body"]["key"] == q["body"]["key"]
                        })
                })
                .cloned()
                .collect();
            let (context, context_missing) =
                self.context(self.source, &remaining, self.question, &target_ids, true);
            let mut missing = vec![
                "answer_slot".into(),
                "documentary_state_only".into(),
                "identity".into(),
                "event_occurrence".into(),
            ];
            missing.extend(context_missing);
            let premises = self.premises(r);
            let basis = s(&r["basis_requirement"], "kind") != "exact"
                || r["basis_requirement"]["value"] == p["basis"];
            let blocked = context != Relation::Same
                || !basis
                || !premises.is_empty()
                || self.scope_match(r, p) != Relation::Same
                || !self
                    .dependency_missing(self.source, "claim", id(c))
                    .is_empty()
                || !self
                    .dependency_missing(self.question, "request", id(r))
                    .is_empty()
                || !self.branch(self.source, s(c, "scope_id")).is_empty()
                || !self.binders(self.source, s(c, "scope_id")).is_empty()
                || !self.binders(self.question, s(r, "scope_id")).is_empty()
                || rows(self.source_report, "candidates")
                    .iter()
                    .any(|candidate| {
                        !matches!(s(candidate, "state"), "eligible" | "conditional")
                            && rows(candidate, "claim_ids").contains(&c["id"])
                    });
            if blocked {
                missing.push("documentary_anchor_context".into());
                missing.extend(premises);
                if !basis {
                    missing.push("basis".into());
                }
                unique(&mut missing);
                out.unresolved.push(self.candidate(c, r, &missing));
                continue;
            }
            unique(&mut missing);
            let mut witness = self.witness(
                c,
                r,
                p,
                "partial_documentary_anchor_context",
                vec![],
                missing.clone(),
                &[],
            );
            witness["answer_propositions"] = json!([p]);
            witness["source_qualifiers"] = json!(
                source_ids
                    .iter()
                    .filter_map(|v| self.qualifier(self.source, v.as_str()?))
                    .collect::<Vec<_>>()
            );
            witness["documentary_anchor_context"] = json!({"profile_id":row["id"],"source_anchor_ref":source_link["target"],"question_anchor_ref":question_link["target"],"source_link_kind":source_link["kind"],"question_link_kind":question_link["kind"],"link_kinds_equated":false,"identity_proven":false,"requested_event_occurrence_inferred":false,"requested_value_inferred":false,"unknown_value_inferred":false});
            out.partial(witness);
            out.unresolved.push(self.candidate(c, r, &missing));
        }
    }
    fn documentary_context(&self, c: &Value, r: &Value, out: &mut Evidence) {
        self.documentary_anchor_context(c, r, out);
        let p = &c["body"]["value"];
        let source = &p["atom"];
        let target = &r["target"]["atom"];
        if s(r, "mode") != "answer"
            || !matches!(
                s(source, "predicate"),
                "documentary_availability" | "information_state"
            )
            || !matches!(
                (s(target, "predicate"), s(r, "slot")),
                ("property", "value") | ("quantity", "amount")
            )
            || s(source, "polarity") != "positive"
            || s(p, "judgment") != "fact"
            || s(p, "status") != "state"
            || p["judgment"] != r["target"]["judgment"]
            || p["status"] != r["target"]["status"]
        {
            return;
        }
        let (Some(subject), Some(wanted), Some(attribute), Some(wanted_attribute), Some(_state)) = (
            argument(source, "subject"),
            argument(target, "subject"),
            argument(source, "attribute"),
            argument(target, "attribute"),
            argument(source, "state"),
        ) else {
            return;
        };
        if s(subject, "kind") != "ref"
            || s(wanted, "kind") != "ref"
            || self.term(self.source, attribute, self.question, wanted_attribute) != Relation::Same
        {
            return;
        }
        let relation = self.term(self.source, subject, self.question, wanted);
        let shared_name = rows(self.source, "references").iter().any(|a| {
            !s(&a["name"], "quote").is_empty()
                && self.local(self.source, s(subject, "id"), id(a)) == Relation::Same
                && rows(self.question, "references").iter().any(|b| {
                    self.local(self.question, s(wanted, "id"), id(b)) == Relation::Same
                        && a["name"]["quote"] == b["name"]["quote"]
                        && a["denotation"] == b["denotation"]
                })
        });
        // Missing class proof under a shared name remains explicitly qualified.
        // Anonymous similarity or mere co-occurrence is not an anchor.
        let anchored = relation == Relation::Same
            || relation == Relation::Unknown
                && shared_name
                && self.partial_named_anchor(s(subject, "id"), s(wanted, "id"), 0);
        let mut missing = vec!["answer_slot".into(), "documentary_state_only".into()];
        if relation != Relation::Same {
            missing.push("identity".into());
        }
        let (context, context_missing) = self.context(
            self.source,
            &self.claim_qualifiers(c, p),
            self.question,
            &self.request_qualifiers(r),
            true,
        );
        if context == Relation::Different || self.scope_match(r, p) == Relation::Different {
            return;
        }
        missing.extend(context_missing);
        let mut blocked = !anchored || context != Relation::Same;
        if self.scope_match(r, p) != Relation::Same {
            missing.push("normative_target".into());
            blocked = true;
        }
        if !self.required_associations_apply(p, r) {
            missing.push("association_context".into());
            blocked = true;
        }
        if !self
            .dependency_missing(self.source, "claim", id(c))
            .is_empty()
        {
            missing.push("dependency".into());
            blocked = true;
        }
        if !self.branch(self.source, s(c, "scope_id")).is_empty() {
            missing.push("branch_choice".into());
            blocked = true;
        }
        if s(&r["basis_requirement"], "kind") == "exact"
            && r["basis_requirement"]["value"] != p["basis"]
        {
            missing.push("basis".into());
            blocked = true;
        }
        let premises = self.premises(r);
        if !premises.is_empty() {
            blocked = true;
            missing.extend(premises);
        }
        unique(&mut missing);
        if blocked {
            out.unresolved.push(self.candidate(c, r, &missing));
            return;
        }
        let mut witness = self.witness(c, r, p, "partial_relevance", vec![], missing.clone(), &[]);
        // Preserve the state and its record scope, not a fabricated requested value.
        witness["answer_propositions"] = json!([p]);
        out.partial(witness);
        if relation != Relation::Same {
            out.unresolved.push(self.candidate(c, r, &missing));
        }
    }
    // Compatibility for transporting the original proposition as context only.
    // Descriptive pattern satisfaction never establishes cross-document identity.
    fn qualified_relevance_binding(&self, a: &Value, b: &Value) -> (bool, bool) {
        if s(a, "kind") != "ref" || s(b, "kind") != "ref" {
            return (false, false);
        }
        let (Some(actual), Some(wanted)) = (
            self.reference(self.source, s(a, "id")),
            self.reference(self.question, s(b, "id")),
        ) else {
            return (false, false);
        };
        if !actual["anaphora"].is_null() || !wanted["anaphora"].is_null() {
            return (false, false);
        }
        for (field, key) in [("declared_class", "quote"), ("semantic_type", "type_id")] {
            if !actual[field].is_null()
                && !wanted[field].is_null()
                && actual[field][key] != wanted[field][key]
            {
                return (false, false);
            }
        }
        let named = rows(self.source, "references").iter().any(|source| {
            !s(&source["name"], "quote").is_empty()
                && self.local(self.source, s(a, "id"), id(source)) == Relation::Same
                && rows(self.question, "references").iter().any(|target| {
                    self.local(self.question, s(b, "id"), id(target)) == Relation::Same
                        && source["name"]["quote"] == target["name"]["quote"]
                        && source["denotation"] == target["denotation"]
                })
        }) && self.partial_named_anchor(s(a, "id"), s(b, "id"), 0);
        let anonymous = actual["name"].is_null()
            && wanted["name"].is_null()
            && self.answer_reference_pattern(s(a, "id"), s(b, "id"), 0) == Relation::Same;
        (named || anonymous, named)
    }
    fn relevance_role_values<'a>(&self, c: &'a Value, p: &'a Value, role: &str) -> Vec<&'a Value> {
        if s(&c["body"], "kind") == "value_restriction" && s(&c["body"], "answer_role") == role {
            rows(&c["body"], "members").iter().collect()
        } else {
            argument(&p["atom"], role).into_iter().collect()
        }
    }
    fn relevance_profile(&self, link: &Value, c: &Value, r: &Value, p: &Value) -> Option<Value> {
        let Some(profile) = rows(&self.vocabulary["relevance_profiles"], "rows")
            .iter()
            .find(|row| row["link_id"] == link["id"])
        else {
            return Some(Value::Null);
        };
        let target = &r["target"]["atom"];
        for guard in rows(profile, "alternatives") {
            let values = self.relevance_role_values(c, p, s(guard, "source_role"));
            let mut derived_type_proofs = vec![];
            let success = match s(guard, "kind") {
                "class_context" => {
                    let wanted = argument(target, s(guard, "target_role"))
                        .filter(|v| s(v, "kind") == "ref")
                        .and_then(|v| self.reference(self.question, s(v, "id")));
                    !values.is_empty()
                        && wanted.is_some_and(|v| {
                            !v["declared_class"].is_null()
                                && rows(guard, "target_classes")
                                    .contains(&v["declared_class"]["quote"])
                        })
                        && values.iter().all(|v| {
                            s(v, "kind") == "ref"
                                && self.reference(self.source, s(v, "id")).is_some_and(|v| {
                                    !v["declared_class"].is_null()
                                        && rows(guard, "source_classes")
                                            .contains(&v["declared_class"]["quote"])
                                })
                        })
                }
                "answer_filter" => {
                    !values.is_empty()
                        && !r["class_filter"].is_null()
                        && values.iter().all(|v| {
                            if s(&r["class_filter"], "kind") == "reference_pattern" {
                                s(v, "kind") == "ref"
                                    && self.answer_reference_pattern(
                                        s(v, "id"),
                                        s(&r["class_filter"], "reference_id"),
                                        0,
                                    ) == Relation::Same
                            } else {
                                if self.class_match(v, &r["class_filter"]) {
                                    true
                                } else if let Some(proof) = self
                                    .context_type_grounding_applicable(c, r, p)
                                    .then(|| self.context_type_grounding(v, &r["class_filter"]))
                                    .flatten()
                                {
                                    derived_type_proofs.push(proof);
                                    true
                                } else {
                                    false
                                }
                            }
                        })
                }
                "corroborating_claim" => {
                    for evidence in rows(self.source, "claims") {
                        let ep = &evidence["body"]["value"];
                        if id(evidence) == id(c)
                            || s(&evidence["body"], "kind") != "proposition"
                            || ep["atom"]["predicate"] != guard["predicate"]
                            || s(&ep["atom"], "polarity") != "positive"
                            || s(ep, "judgment") != "fact"
                            || s(ep, "status") != "state"
                            || !self.branch(self.source, s(evidence, "scope_id")).is_empty()
                            || !self
                                .binders(self.source, s(evidence, "scope_id"))
                                .is_empty()
                            || !self
                                .dependency_missing(self.source, "claim", id(evidence))
                                .is_empty()
                            || rows(self.source_report, "candidates")
                                .iter()
                                .any(|candidate| {
                                    s(candidate, "state") != "eligible"
                                        && rows(candidate, "claim_ids").contains(&evidence["id"])
                                })
                            || self.scope_match(r, ep) != Relation::Same
                            || self
                                .context(
                                    self.source,
                                    &self.claim_qualifiers(evidence, ep),
                                    self.question,
                                    &self.request_qualifiers(r),
                                    true,
                                )
                                .0
                                != Relation::Same
                            || self
                                .context(
                                    self.source,
                                    &self.claim_qualifiers(evidence, ep),
                                    self.source,
                                    &self.claim_qualifiers(c, p),
                                    true,
                                )
                                .0
                                != Relation::Same
                            || s(&r["basis_requirement"], "kind") == "exact"
                                && r["basis_requirement"]["value"] != ep["basis"]
                        {
                            continue;
                        }
                        let bound = rows(guard, "bindings").iter().all(|binding| {
                            let left = binding[0].as_str().unwrap_or("");
                            let right = binding[1]
                                .as_str()
                                .unwrap_or("")
                                .strip_prefix("evidence.")
                                .unwrap_or("");
                            let Some(actual) = argument(&ep["atom"], right) else {
                                return false;
                            };
                            if let Some(role) = left.strip_prefix("source.") {
                                if s(&c["body"], "kind") == "value_restriction"
                                    && s(&c["body"], "answer_role") == role
                                {
                                    return false;
                                }
                                argument(&p["atom"], role).is_some_and(|wanted| {
                                    self.term(self.source, actual, self.source, wanted)
                                        == Relation::Same
                                })
                            } else if let Some(role) = left.strip_prefix("target.") {
                                argument(target, role).is_some_and(|wanted| {
                                    self.term(self.source, actual, self.question, wanted)
                                        == Relation::Same
                                        || s(actual, "kind") == "ref"
                                            && s(wanted, "kind") == "ref"
                                            && self
                                                .reference(self.question, s(wanted, "id"))
                                                .is_some_and(|r| r["name"].is_null())
                                            && self.answer_reference_pattern(
                                                s(actual, "id"),
                                                s(wanted, "id"),
                                                0,
                                            ) == Relation::Same
                                })
                            } else {
                                false
                            }
                        });
                        if !bound {
                            continue;
                        }
                        let contradicted = rows(self.source, "claims").iter().any(|other| {
                            let op = &other["body"]["value"];
                            s(&other["body"], "kind") == "proposition"
                                && s(op, "judgment") == "fact"
                                && s(op, "status") == "state"
                                && s(&op["atom"], "polarity") == "negative"
                                && ep["normative_scope"] == op["normative_scope"]
                                && self.atom(
                                    self.source,
                                    &ep["atom"],
                                    self.source,
                                    &op["atom"],
                                    None,
                                ) == Relation::Same
                                && self
                                    .context(
                                        self.source,
                                        &self.claim_qualifiers(evidence, ep),
                                        self.source,
                                        &self.claim_qualifiers(other, op),
                                        true,
                                    )
                                    .0
                                    == Relation::Same
                        });
                        if !contradicted {
                            return Some(
                                json!({"link_id":link["id"],"guard_kind":"corroborating_claim","support_claim_ids":[evidence["id"]],"evidence_refs":evidence["evidence"],"requested_origin_inferred":false}),
                            );
                        }
                    }
                    false
                }
                _ => false,
            };
            if success {
                let mut proof = json!({"link_id":link["id"],"guard_kind":guard["kind"],"source_role":guard["source_role"],"source_reference_ids":values.iter().map(|v|v["id"].clone()).collect::<Vec<_>>(),"requested_origin_inferred":false});
                if !derived_type_proofs.is_empty() {
                    proof["context_type_grounding"] = json!(derived_type_proofs);
                }
                return Some(proof);
            }
        }
        None
    }
    fn relevance(&self, c: &Value, r: &Value, out: &mut Evidence) {
        let body = &c["body"];
        let restriction = (s(body, "kind") == "value_restriction").then_some(body);
        if restriction.is_some() && !self.restriction_domain_known(body) {
            return;
        }
        if restriction.is_some()
            && (s(body, "closure") != "closed"
                || s(body, "existence") != "asserted"
                || rows(body, "members").is_empty()
                || s(&body["target"], "polarity") != "positive"
                || s(body, "judgment") != "fact"
                || s(body, "status") != "state"
                || s(r, "mode") != "answer"
                || s(&r["target"], "judgment") != "fact")
        {
            return;
        }
        // This view is only for the applicability checks. Never publish it as
        // a proposition: existence in a set does not assert every member.
        let restriction_view = json!({"atom":body["target"],"judgment":body["judgment"],"status":body["status"],"basis":body["basis"],"normative_scope":body["normative_scope"],"qualifier_ids":[]});
        let p = match s(body, "kind") {
            "proposition" => &body["value"],
            "value_restriction" => &restriction_view,
            _ => return,
        };
        let target = &r["target"]["atom"];
        for link in rows(self.vocabulary, "relevance_links") {
            if link["source_predicate"] != p["atom"]["predicate"]
                || link["target_predicate"] != target["predicate"]
            {
                continue;
            }
            if let Some(constraint) = restriction {
                // The answer role ranges over the closed set; it cannot anchor
                // a fixed query referent. Reserved predicate paths stay separate.
                let member_role = format!("source.{}", s(constraint, "answer_role"));
                if rows(link, "bindings")
                    .iter()
                    .any(|binding| binding[0].as_str() == Some(member_role.as_str()))
                    || matches!(
                        s(&p["atom"], "predicate"),
                        "admits" | "documentary_availability" | "information_state"
                    )
                {
                    continue;
                }
            }
            if p["atom"]["predicate"] == "admits"
                && target["predicate"] == "property"
                && link["bindings"] == json!([["source.subject", "target.subject"]])
                && link["carry_roles"] == json!(["source.object"])
            {
                continue;
            }
            let Some(profile_proof) = self.relevance_profile(link, c, r, p) else {
                out.unresolved
                    .push(self.candidate(c, r, &["relevance_profile".into()]));
                continue;
            };
            if p["atom"]["predicate"] == "admits"
                && target["predicate"] == "quantity"
                && link["bindings"] == json!([["source.subject", "target.subject"]])
                && rows(link, "carry_roles").contains(&json!("source.object"))
            {
                self.admission_context(c, r, out);
                continue;
            }
            if matches!(
                s(&p["atom"], "predicate"),
                "documentary_availability" | "information_state"
            ) && rows(link, "bindings").len() == 2
                && rows(link, "bindings").contains(&json!(["source.subject", "target.subject"]))
                && rows(link, "bindings").contains(&json!(["source.attribute", "target.attribute"]))
                && rows(link, "carry_roles").contains(&json!("source.state"))
            {
                self.documentary_context(c, r, out);
                continue;
            }
            let mut relation = Relation::Same;
            let mut qualified_bindings = true;
            let mut named_binding = false;
            for binding in rows(link, "bindings") {
                let a = binding[0]
                    .as_str()
                    .unwrap_or("")
                    .strip_prefix("source.")
                    .unwrap_or("");
                let b = binding[1]
                    .as_str()
                    .unwrap_or("")
                    .strip_prefix("target.")
                    .unwrap_or("");
                let matched = match (argument(&p["atom"], a), argument(target, b)) {
                    (Some(a), Some(b)) => {
                        let strict = self.term(self.source, a, self.question, b);
                        let (compatible, named) = self.qualified_relevance_binding(a, b);
                        qualified_bindings &=
                            strict == Relation::Same || strict == Relation::Unknown && compatible;
                        named_binding |= named;
                        strict
                    }
                    _ => {
                        qualified_bindings = false;
                        Relation::Unknown
                    }
                };
                relation = relation.merge(matched);
            }
            if relation == Relation::Different {
                continue;
            }
            let (context, context_missing) = self.context(
                self.source,
                &self.claim_qualifiers(c, p),
                self.question,
                &self.request_qualifiers(r),
                true,
            );
            let scope_relation = self.scope_match(r, p);
            if context == Relation::Different || scope_relation == Relation::Different {
                continue;
            }
            let mut missing = vec!["answer_slot".into()];
            if restriction.is_some() {
                missing.push("source_constraint_only".into());
            }
            missing.extend(context_missing);
            let qualified = relation == Relation::Unknown
                && qualified_bindings
                && named_binding
                && s(r, "mode") == "answer"
                && s(p, "judgment") == "fact"
                && s(p, "status") == "state"
                && s(&p["atom"], "polarity") == "positive"
                && r["target"]["judgment"] == p["judgment"];
            // The declared bindings, including their reference patterns, were
            // checked above. Unbound target roles are missing context, not
            // claims that the source supplied them under same-named roles.
            let unbound_target_roles: Vec<_> = rows(target, "arguments")
                .iter()
                .filter(|arg| {
                    !rows(link, "bindings").iter().any(|binding| {
                        binding[1].as_str() == Some(format!("target.{}", s(arg, "role")).as_str())
                    })
                })
                .map(|arg| arg["role"].clone())
                .collect();
            if qualified && !unbound_target_roles.is_empty() {
                missing.push("unbound_target_context".into());
            }
            let mut applicability_missing = relation != Relation::Same && !qualified
                || context != Relation::Same
                || scope_relation != Relation::Same;
            if relation != Relation::Same {
                missing.push("identity".into());
            }
            if scope_relation != Relation::Same {
                missing.push("normative_target".into());
            }
            if s(&r["basis_requirement"], "kind") == "exact"
                && r["basis_requirement"]["value"] != p["basis"]
            {
                missing.push("basis".into());
                applicability_missing = true;
            }
            if !self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
            {
                missing.push("dependency".into());
                applicability_missing = true;
            }
            if !self.branch(self.source, s(c, "scope_id")).is_empty() {
                missing.push("branch_choice".into());
                applicability_missing = true;
            }
            missing.extend(self.premises(r));
            if missing.contains(&"premise".to_owned()) {
                applicability_missing = true;
            }
            if applicability_missing {
                unique(&mut missing);
                out.unresolved.push(self.candidate(c, r, &missing));
                continue;
            }
            let terms = rows(link, "carry_roles")
                .iter()
                .filter(|_| !qualified && restriction.is_none())
                .filter_map(|x| {
                    x.as_str()
                        .and_then(|x| x.strip_prefix("source."))
                        .and_then(|x| argument(&p["atom"], x))
                        .cloned()
                })
                .collect();
            unique(&mut missing);
            let mut witness =
                self.witness(c, r, p, "partial_relevance", terms, missing.clone(), &[]);
            if qualified {
                witness["answer_propositions"] = json!([p]);
                witness["context_identity_proven"] = json!(false);
                witness["unbound_target_roles"] = json!(unbound_target_roles);
                out.unresolved.push(self.candidate(c, r, &missing));
            }
            if let Some(constraint) = restriction {
                witness["answer_propositions"] = json!([]);
                witness["source_constraint"] = constraint.clone();
            }
            if !rows(&profile_proof, "context_type_grounding").is_empty() {
                witness["answer_terms"] = json!([]);
                if restriction.is_none() {
                    witness["answer_propositions"] = json!([p]);
                }
                witness["context_identity_proven"] = json!(false);
            }
            if !profile_proof.is_null() {
                witness["relevance_profile"] = profile_proof;
            }
            out.partial(witness);
        }
    }
    fn known_configuration(
        &self,
        doc: &Value,
        ids: &[Value],
        required: &[Value],
    ) -> Option<BTreeSet<String>> {
        let mut consumed = BTreeSet::new();
        for rule in required {
            let qualifiers: Vec<_> = ids
                .iter()
                .filter_map(|v| self.qualifier(doc, v.as_str()?))
                .filter(|q| s(q, "kind") == "configuration" && q["body"]["key"] == rule["key"])
                .collect();
            if qualifiers.is_empty()
                || qualifiers.iter().any(|q| {
                    s(q, "resolution") != "known"
                        || s(&q["body"]["value"], "kind") != "text"
                        || q["body"]["value"]["value"] != rule["value"]
                })
            {
                return None;
            }
            consumed.extend(qualifiers.iter().map(|q| id(q).to_owned()));
        }
        Some(consumed)
    }
    fn measurement_label(&self, doc: &Value, value: &Value) -> Option<String> {
        match s(value, "kind") {
            "text" => value["value"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
            "ref" => self
                .reference(doc, s(value, "id"))
                .filter(|r| r["anaphora"].is_null())
                .and_then(|r| r["name"]["quote"].as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
            _ => None,
        }
    }
    fn measurement_gap_context(&self, r: &Value, out: &mut Evidence) {
        let target = &r["target"]["atom"];
        if s(r, "mode") != "answer"
            || s(r, "slot") != "amount"
            || s(target, "predicate") != "quantity"
            || s(target, "polarity") != "positive"
            || s(&r["target"], "judgment") != "fact"
            || s(&r["target"], "status") != "state"
            || argument(target, "amount").is_some()
            || !r["normative_scope_filter"].is_null()
        {
            return;
        }
        let (Some(subject), Some(attribute)) =
            (argument(target, "subject"), argument(target, "attribute"))
        else {
            return;
        };
        for row in rows(&self.vocabulary["measurement_context"], "rows") {
            if attribute["id"] != row["target_attribute"] {
                continue;
            }
            for c in rows(self.source, "claims") {
                let body = &c["body"];
                if s(body, "kind") != "event"
                    || s(body, "event_kind") != "measurement"
                    || body["aspect"] != row["event_aspect"]
                    || s(body, "status") != "negated_occurrence"
                    || !body["result"].is_null()
                    || !matches!(s(body, "basis"), "description" | "observation")
                {
                    continue;
                }
                let actual = json!({"kind":"ref","id":body["subject"]});
                let relation = self.term(self.source, &actual, self.question, subject);
                let (compatible, named) = self.qualified_relevance_binding(&actual, subject);
                if relation != Relation::Same
                    && !(relation == Relation::Unknown && compatible && named)
                {
                    continue;
                }
                // This view supplies witness metadata only; it is never a value proposition.
                let p = json!({"judgment":"fact","status":"state","basis":body["basis"],"normative_scope":null,"qualifier_ids":[]});
                let source_ids = self.claim_qualifiers(c, &p);
                let target_ids = self.request_qualifiers(r);
                let (Some(mut source_used), Some(mut target_used)) = (
                    self.known_configuration(
                        self.source,
                        &source_ids,
                        rows(row, "source_configuration"),
                    ),
                    self.known_configuration(
                        self.question,
                        &target_ids,
                        rows(row, "target_configuration"),
                    ),
                ) else {
                    out.unresolved
                        .push(self.candidate(c, r, &["measurement_context".into()]));
                    continue;
                };
                let mut labels = vec![];
                let mut context_ok = true;
                for binding in rows(row, "context_bindings") {
                    let sources: Vec<_> = source_ids
                        .iter()
                        .filter_map(|v| self.qualifier(self.source, v.as_str()?))
                        .filter(|q| {
                            s(q, "kind") == "configuration" && q["body"]["key"] == binding[0]
                        })
                        .collect();
                    let targets: Vec<_> = target_ids
                        .iter()
                        .filter_map(|v| self.qualifier(self.question, v.as_str()?))
                        .filter(|q| {
                            s(q, "kind") == "configuration" && q["body"]["key"] == binding[1]
                        })
                        .collect();
                    if sources.is_empty() && targets.is_empty() {
                        continue;
                    }
                    let matched = !sources.is_empty()
                        && !targets.is_empty()
                        && sources.iter().all(|a| {
                            s(a, "resolution") == "known"
                                && targets.iter().all(|b| {
                                    let left =
                                        self.measurement_label(self.source, &a["body"]["value"]);
                                    s(b, "resolution") == "known"
                                        && left.is_some()
                                        && left
                                            == self.measurement_label(
                                                self.question,
                                                &b["body"]["value"],
                                            )
                                })
                        });
                    if !matched {
                        context_ok = false;
                        break;
                    }
                    source_used.extend(sources.iter().map(|q| id(q).to_owned()));
                    target_used.extend(targets.iter().map(|q| id(q).to_owned()));
                    labels.push(json!({"source_qualifier_ids":sources.iter().map(|q|id(q)).collect::<Vec<_>>(),"question_qualifier_ids":targets.iter().map(|q|id(q)).collect::<Vec<_>>(),"relation":"documentary_label","identity_proven":false}));
                }
                let source_remaining: Vec<_> = source_ids
                    .iter()
                    .filter(|v| !source_used.contains(v.as_str().unwrap_or("")))
                    .cloned()
                    .collect();
                let target_remaining: Vec<_> = target_ids
                    .iter()
                    .filter(|v| !target_used.contains(v.as_str().unwrap_or("")))
                    .cloned()
                    .collect();
                let (context, context_missing) = self.context(
                    self.source,
                    &source_remaining,
                    self.question,
                    &target_remaining,
                    true,
                );
                let mut missing = vec!["answer_slot".into(), "negative_measurement_only".into()];
                if relation != Relation::Same {
                    missing.push("identity".into());
                }
                missing.extend(context_missing);
                let basis = s(&r["basis_requirement"], "kind") != "exact"
                    || r["basis_requirement"]["value"] == body["basis"];
                let premises = self.premises(r);
                let blocked = !context_ok
                    || context != Relation::Same
                    || !basis
                    || !premises.is_empty()
                    || !self
                        .dependency_missing(self.source, "claim", id(c))
                        .is_empty()
                    || !self.branch(self.source, s(c, "scope_id")).is_empty()
                    || !self.binders(self.source, s(c, "scope_id")).is_empty()
                    || rows(self.source_report, "candidates")
                        .iter()
                        .any(|candidate| {
                            !matches!(s(candidate, "state"), "eligible" | "conditional")
                                && rows(candidate, "claim_ids").contains(&c["id"])
                        });
                if blocked {
                    missing.push("measurement_context".into());
                    missing.extend(premises);
                    if !basis {
                        missing.push("basis".into());
                    }
                    unique(&mut missing);
                    out.unresolved.push(self.candidate(c, r, &missing));
                    continue;
                }
                let mut witness = self.witness(
                    c,
                    r,
                    &p,
                    "partial_measurement_context",
                    vec![],
                    missing.clone(),
                    &[],
                );
                witness["source_event"] = body.clone();
                witness["source_qualifiers"] = json!(
                    source_ids
                        .iter()
                        .filter_map(|v| self.qualifier(self.source, v.as_str()?))
                        .collect::<Vec<_>>()
                );
                witness["measurement_context"] = json!({"profile_id":row["id"],"configuration_bindings":labels,"requested_value_inferred":false,"unknown_value_inferred":false});
                out.partial(witness);
                if relation != Relation::Same {
                    out.unresolved.push(self.candidate(c, r, &missing));
                }
            }
        }
    }
    fn event_peers(&self, c: &Value) -> Vec<&Value> {
        let b = &c["body"];
        rows(self.source, "claims")
            .iter()
            .filter(|other| {
                let ob = &other["body"];
                s(ob, "kind") == "event"
                    && self.local(self.source, s(b, "event_ref"), s(ob, "event_ref"))
                        == Relation::Same
                    && self.local(self.source, s(b, "subject"), s(ob, "subject")) == Relation::Same
                    && b["event_kind"] == ob["event_kind"]
                    && b["aspect"] == ob["aspect"]
                    && b["status"] == ob["status"]
                    && b["basis"] == ob["basis"]
                    && self.branch(self.source, s(c, "scope_id"))
                        == self.branch(self.source, s(other, "scope_id"))
                    && self
                        .context(
                            self.source,
                            &self.claim_qualifiers(c, &Value::Null),
                            self.source,
                            &self.claim_qualifiers(other, &Value::Null),
                            false,
                        )
                        .0
                        == Relation::Same
                    && self
                        .dependency_missing(self.source, "claim", id(other))
                        .is_empty()
            })
            .collect()
    }
    fn events(&self, r: &Value, out: &mut Evidence) {
        let target = &r["target"];
        if target.get("event_kind").is_none() {
            return;
        }
        let mut matches: Vec<(&Value, Relation, Vec<String>, Vec<String>)> = vec![];
        for c in rows(self.source, "claims")
            .iter()
            .filter(|c| s(&c["body"], "kind") == "event")
        {
            let b = &c["body"];
            if b["event_kind"] != target["event_kind"] {
                continue;
            }
            let mut relation = self.identity(
                self.source,
                s(b, "subject"),
                self.question,
                s(target, "subject"),
            );
            if relation == Relation::Different {
                continue;
            }
            if !target["event_ref"].is_null() {
                relation = relation.merge(self.identity(
                    self.source,
                    s(b, "event_ref"),
                    self.question,
                    s(target, "event_ref"),
                ));
            }
            if !target["aspect"].is_null() && b["aspect"] != target["aspect"] {
                continue;
            }
            let mut missing = vec![];
            let mut extra_claims = vec![];
            let peers = self.event_peers(c);
            let desired = s(target, "status");
            let actual = s(b, "status");
            if desired != "any" && actual != desired {
                if !(desired == "occurred"
                    && actual == "negated_occurrence"
                    && !target["event_ref"].is_null())
                {
                    continue;
                }
            }
            if actual != "negated_occurrence" && !self.common_quantity_scope_compatible(c, r) {
                out.unresolved
                    .push(self.candidate(c, r, &["common_quantity_scope".into()]));
                continue;
            }
            if !target["time_selector"].is_null() {
                let selector = &target["time_selector"];
                if let Some(peer) = peers.iter().find(|peer| {
                    rows(&peer["body"], "times").iter().any(|t| {
                        t["role"] == selector["role"]
                            && self.term(
                                self.source,
                                &t["value"],
                                self.question,
                                &selector["value"],
                            ) == Relation::Same
                    })
                }) {
                    if id(peer) != id(c) {
                        extra_claims.push(id(peer).to_owned());
                    }
                } else {
                    missing.push("time_role".into());
                }
            }
            if s(&target["method_selector"], "kind") == "specified" {
                if let Some(peer) = peers.iter().find(|peer| {
                    s(&peer["body"]["method"], "state") == "specified"
                        && self.term(
                            self.source,
                            &peer["body"]["method"]["value"],
                            self.question,
                            &target["method_selector"]["value"],
                        ) == Relation::Same
                }) {
                    if id(peer) != id(c) {
                        extra_claims.push(id(peer).to_owned());
                    }
                } else {
                    missing.push("method".into());
                }
            }
            if relation != Relation::Same {
                missing.push("identity".into());
            }
            if !self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
            {
                missing.push("dependency".into());
            }
            let (context, context_missing) = self.context(
                self.source,
                &self.claim_qualifiers(c, &Value::Null),
                self.question,
                &self.request_qualifiers(r),
                true,
            );
            if context == Relation::Different {
                continue;
            }
            missing.extend(context_missing);
            if !self.branch(self.source, s(c, "scope_id")).is_empty() {
                missing.push("branch_choice".into());
            }
            missing.extend(self.premises(r));
            if !self.required_associations_apply(b, r) {
                out.unresolved
                    .push(self.candidate(c, r, &["association_context".into()]));
                continue;
            }
            matches.push((c, relation, missing, extra_claims));
        }
        // Repeated events are not identified by subject/aspect. Same is the only merge proof.
        let event_roots: Vec<_> = matches
            .iter()
            .map(|(c, _, _, _)| s(&c["body"], "event_ref"))
            .collect();
        let ambiguous = event_roots.iter().enumerate().any(|(i, a)| {
            event_roots
                .iter()
                .skip(i + 1)
                .any(|b| self.local(self.source, a, b) != Relation::Same)
        });
        for (c, _relation, mut missing, extra_claims) in matches {
            let b = &c["body"];
            let slot = s(r, "slot");
            let mut terms = vec![];
            if ambiguous && s(r, "required_answer_extent") != "all_values" {
                missing.push("event_identity".into());
            }
            match slot {
                "status" => terms
                    .push(json!({"kind":"text","value":b["status"],"evidence":rows(c,"evidence")})),
                "aspect" => {
                    if !b["aspect"].is_null() {
                        terms.push(
                            json!({"kind":"symbol","id":b["aspect"],"evidence":rows(c,"evidence")}),
                        );
                    }
                }
                "method" => {
                    if s(&b["method"], "state") == "specified" {
                        terms.push(b["method"]["value"].clone());
                    } else {
                        missing.push("method".into());
                    }
                }
                "result" => {
                    if !b["result"].is_null() {
                        terms.push(b["result"].clone());
                    }
                }
                "planned_for" | "scheduled_at" | "occurred_at" | "planned_year"
                | "scheduled_year" | "occurred_year" => {
                    let slot = super::temporal_slot(slot).unwrap().0;
                    if (slot == "planned_for" && s(b, "status") != "planned")
                        || (slot == "occurred_at" && s(b, "status") != "occurred")
                    {
                        missing.push("time_role".into());
                    } else {
                        terms.extend(
                            rows(b, "times")
                                .iter()
                                .filter(|t| s(t, "role") == slot)
                                .map(|t| t["value"].clone()),
                        );
                    }
                }
                _ => {}
            }
            // A missing slot may be supplied only by a claim explicitly identifying this event.
            if terms.is_empty() {
                missing.push("answer_slot".into());
            }
            let p = json!({"judgment":"fact","status":b["status"],"basis":b["basis"],"normative_scope":null,"qualifier_ids":c["qualifier_ids"]});
            missing.extend(self.filters(r, &p, &terms));
            unique(&mut missing);
            let kind = if missing.is_empty() {
                "direct"
            } else if missing.contains(&"branch_choice".into()) {
                "conditional_branch"
            } else {
                "partial_slot"
            };
            let w = self.witness(c, r, &p, kind, terms, missing.clone(), &extra_claims);
            if missing.is_empty() {
                out.publish(w, s(b, "status") == "negated_occurrence");
            } else {
                if missing.iter().any(|x| {
                    matches!(
                        x.as_str(),
                        "event_identity"
                            | "identity"
                            | "dependency"
                            | "time_role"
                            | "guard"
                            | "configuration"
                    )
                }) {
                    out.unresolved.push(self.candidate(c, r, &missing));
                }
                out.partial(w);
            }
        }
    }
    fn rule_scope_resolved(&self, rule: &Value) -> bool {
        let body = &rule["body"];
        let action = &body["action"];
        (!matches!(s(action, "judgment"), "allowed" | "required" | "forbidden")
            || self.local(
                self.source,
                s(body, "normative_scope"),
                s(action, "normative_scope"),
            ) == Relation::Same)
            && self
                .dependency_missing(self.source, "claim", id(rule))
                .is_empty()
    }
    fn requirement_state(
        &self,
        rule: &Value,
        index: usize,
        waived: bool,
        unknown: bool,
        missing: &[String],
        exemptions: &[String],
    ) -> Value {
        let state = if unknown && !waived {
            "unresolved"
        } else if !missing.is_empty() {
            "conditional"
        } else if waived {
            "waived"
        } else {
            "known_active"
        };
        json!({"rule_id":id(rule),"normative_scope":rule["body"]["normative_scope"],"condition_index":index,"effective_state":state,"exemption_claim_ids":exemptions})
    }
    fn requirement_metadata(&self, witness: &mut Value, conditions: &[Value], states: Vec<Value>) {
        witness["answer_propositions"] = json!(conditions);
        witness["requirement_states"] = json!(states);
        // A requirement or waiver describes a rule; it never concludes that the action is permitted.
        witness["conclusion_judgment"] = Value::Null;
    }
    fn rule_conditions(&self, c: &Value, r: &Value, out: &mut Evidence) {
        let body = &c["body"];
        let action = &body["action"];
        let target = &r["target"];
        if target.get("atom").is_none() {
            return;
        }
        if self.scope_match(r, action) == Relation::Different
            || self
                .context(
                    self.source,
                    &self.claim_qualifiers(c, action),
                    self.question,
                    &self.request_qualifiers(r),
                    true,
                )
                .0
                == Relation::Different
        {
            return;
        }
        if !self.rule_scope_resolved(c) {
            if self.atom(
                self.source,
                &action["atom"],
                self.question,
                &target["atom"],
                if s(r, "mode") == "truth" {
                    None
                } else {
                    Some(s(r, "slot"))
                },
            ) != Relation::Different
            {
                out.unresolved
                    .push(self.candidate(c, r, &["normative_target".into()]));
            }
            return;
        }
        if s(r, "mode") == "rule_condition" {
            if self.atom(
                self.source,
                &action["atom"],
                self.question,
                &target["atom"],
                Some(s(r, "slot")),
            ) == Relation::Different
                || action["judgment"] != target["judgment"]
                || action["status"] != target["status"]
            {
                return;
            }
            if !self.required_associations_apply(action, r) {
                out.unresolved
                    .push(self.candidate(c, r, &["association_context".into()]));
                return;
            }
            let mut missing = vec![];
            if self.atom(
                self.source,
                &action["atom"],
                self.question,
                &target["atom"],
                Some(s(r, "slot")),
            ) != Relation::Same
            {
                missing.push("identity".into());
            }
            if self.scope_match(r, action) != Relation::Same {
                missing.push("normative_target".into());
            }
            let (context, context_missing) = self.context(
                self.source,
                &self.claim_qualifiers(c, action),
                self.question,
                &self.request_qualifiers(r),
                true,
            );
            if context == Relation::Different {
                return;
            }
            missing.extend(context_missing);
            if !self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
            {
                missing.push("dependency".into());
            }
            if s(body, "direction") != "necessary" {
                missing.push("normative_target".into());
            }
            if !self.branch(self.source, s(c, "scope_id")).is_empty() {
                missing.push("branch_choice".into());
            }
            let mut states = vec![];
            let mut exemption_claims = vec![];
            for (index, condition) in rows(body, "conditions").iter().enumerate() {
                let (waived, exemptions, unknown) = self.condition_waived(c, condition, r);
                if unknown && !waived {
                    missing.push("normative_target".into());
                }
                states.push(self.requirement_state(
                    c,
                    index,
                    waived,
                    unknown,
                    &missing,
                    &exemptions,
                ));
                exemption_claims.extend(exemptions);
            }
            missing.extend(self.filters(r, action, &[]));
            unique(&mut missing);
            let mut w = self.witness(
                c,
                r,
                action,
                "direct",
                vec![],
                missing.clone(),
                &exemption_claims,
            );
            self.requirement_metadata(&mut w, rows(body, "conditions"), states);
            if missing.is_empty() {
                out.support.push(w);
            } else {
                out.partial(w);
                if missing.iter().any(|m| m != "branch_choice") {
                    out.unresolved.push(self.candidate(c, r, &missing));
                }
            }
            return;
        }
        if s(body, "direction") != "sufficient" {
            return;
        }
        let mut inputs = vec![];
        let mut unresolved = false;
        for condition in rows(body, "conditions") {
            let matching: Vec<_> = rows(self.source, "claims")
                .iter()
                .filter(|x| {
                    s(&x["body"], "kind") == "proposition"
                        && self.proposition(
                            self.source,
                            &x["body"]["value"],
                            self.source,
                            condition,
                            None,
                        ) == Relation::Same
                        && self
                            .context(
                                self.source,
                                &self.claim_qualifiers(x, &x["body"]["value"]),
                                self.source,
                                &self.claim_qualifiers(c, condition),
                                true,
                            )
                            .0
                            == Relation::Same
                        && self.branch(self.source, s(x, "scope_id")).is_empty()
                        && self
                            .dependency_missing(self.source, "claim", id(x))
                            .is_empty()
                })
                .collect();
            if matching.is_empty() {
                unresolved = true;
            } else {
                inputs.extend(matching.iter().map(|x| id(x).to_owned()));
            }
        }
        if unresolved {
            if self.atom(
                self.source,
                &action["atom"],
                self.question,
                &target["atom"],
                if s(r, "mode") == "truth" {
                    None
                } else {
                    Some(s(r, "slot"))
                },
            ) != Relation::Different
            {
                out.unresolved.push(self.candidate(c, r, &["guard".into()]));
                out.partial(self.witness(
                    c,
                    r,
                    action,
                    "partial_slot",
                    vec![],
                    vec!["guard".into()],
                    &inputs,
                ));
            }
        } else {
            self.direct(c, r, action, "rule_step", &inputs, out);
        }
    }
    fn condition_waived(
        &self,
        rule: &Value,
        condition: &Value,
        r: &Value,
    ) -> (bool, Vec<String>, bool) {
        let mut exemptions = vec![];
        let mut unknown = false;
        if !self.rule_scope_resolved(rule) {
            return (false, vec![], true);
        }
        let beneficiary = argument(&rule["body"]["action"]["atom"], "actor")
            .or_else(|| argument(&rule["body"]["action"]["atom"], "subject"));
        for c in rows(self.source, "claims")
            .iter()
            .filter(|c| s(&c["body"], "kind") == "exemption")
        {
            let b = &c["body"];
            let relevant = rows(b, "rule_targets")
                .iter()
                .any(|x| x.as_str() == Some(id(rule)));
            if !relevant {
                continue;
            }
            if s(b, "target_resolution") != "exact" {
                unknown = true;
                continue;
            }
            if s(&rule["body"], "direction") != "necessary" {
                continue;
            }
            if self.local(
                self.source,
                s(b, "normative_scope"),
                s(&rule["body"], "normative_scope"),
            ) != Relation::Same
            {
                unknown = true;
                continue;
            }
            if !beneficiary.is_some_and(|x| {
                s(x, "kind") == "ref"
                    && self.local(self.source, s(x, "id"), s(b, "beneficiary")) == Relation::Same
            }) {
                unknown = true;
                continue;
            }
            if !self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
                || self
                    .context(
                        self.source,
                        &self.claim_qualifiers(c, &Value::Null),
                        self.question,
                        &self.request_qualifiers(r),
                        true,
                    )
                    .0
                    != Relation::Same
                || !self.branch(self.source, s(c, "scope_id")).is_empty()
            {
                unknown = true;
                continue;
            }
            if rows(b, "waived_conditions").iter().any(|a| {
                a["polarity"] == condition["atom"]["polarity"]
                    && self.atom(self.source, a, self.source, &condition["atom"], None)
                        == Relation::Same
            }) {
                exemptions.push(id(c).to_owned());
            }
        }
        (!exemptions.is_empty(), exemptions, unknown)
    }
    fn exemption_status(&self, r: &Value, out: &mut Evidence) {
        if s(r, "mode") != "exemption_status" {
            return;
        }
        let target = &r["target"];
        for rule in rows(self.source, "claims").iter().filter(|c| {
            s(&c["body"], "kind") == "rule" && s(&c["body"], "direction") == "necessary"
        }) {
            let action = &rule["body"]["action"];
            let relation = self.atom(
                self.source,
                &action["atom"],
                self.question,
                &target["atom"],
                if s(r, "slot") == "status" {
                    None
                } else {
                    Some(s(r, "slot"))
                },
            );
            if relation == Relation::Different || self.scope_match(r, action) == Relation::Different
            {
                continue;
            }
            if !self.required_associations_apply(action, r) {
                out.unresolved
                    .push(self.candidate(rule, r, &["association_context".into()]));
                continue;
            }
            for (index, condition) in rows(&rule["body"], "conditions").iter().enumerate() {
                let (waived, exemptions, unknown) = self.condition_waived(rule, condition, r);
                let mut missing = vec![];
                if relation != Relation::Same {
                    missing.push("identity".into());
                }
                if self.scope_match(r, action) != Relation::Same {
                    missing.push("normative_target".into());
                }
                let mut qids = self.request_qualifiers(r);
                qids.extend(rows(&r["target"], "qualifier_ids").iter().cloned());
                let (context, context_missing) = self.context(
                    self.source,
                    &self.claim_qualifiers(rule, action),
                    self.question,
                    &qids,
                    true,
                );
                if context == Relation::Different {
                    continue;
                }
                missing.extend(context_missing);
                if unknown || !self.rule_scope_resolved(rule) {
                    missing.push("normative_target".into());
                }
                if !self.branch(self.source, s(rule, "scope_id")).is_empty() {
                    missing.push("branch_choice".into());
                }
                if !self
                    .dependency_missing(self.source, "claim", id(rule))
                    .is_empty()
                {
                    missing.push("dependency".into());
                }
                let state =
                    self.requirement_state(rule, index, waived, unknown, &missing, &exemptions);
                let term = json!({"kind":"text","value":state["effective_state"],"evidence":rows(rule,"evidence")});
                let mut w = self.witness(
                    rule,
                    r,
                    action,
                    "exemption_status",
                    vec![term],
                    missing.clone(),
                    &exemptions,
                );
                self.requirement_metadata(&mut w, std::slice::from_ref(condition), vec![state]);
                if missing.is_empty() {
                    out.support.push(w);
                } else {
                    out.partial(w);
                    out.unresolved.push(self.candidate(rule, r, &missing));
                }
            }
        }
    }
    fn binders<'a>(&self, doc: &'a Value, scope: &str) -> Vec<&'a Value> {
        let mut chain = vec![];
        let mut cur = scope;
        let mut seen = BTreeSet::new();
        while seen.insert(cur.to_owned()) {
            if !super::context_budget::charge(1, 0) {
                break;
            }
            let Some(sc) = rows(doc, "scopes").iter().find(|sc| id(sc) == cur) else {
                break;
            };
            if !sc["binding"].is_null() {
                chain.push(&sc["binding"]);
            }
            cur = s(sc, "parent_id");
        }
        chain.reverse();
        chain
    }
    fn mapped_term(&self, a: &Value, b: &Value, map: &BTreeMap<String, String>) -> Relation {
        if s(a, "kind") == "described_ref" && s(b, "kind") == "described_ref" {
            return self.described_term(self.source, a, self.question, b, Some(map));
        }
        if s(a, "kind") == "ref"
            && s(b, "kind") == "ref"
            && map.get(s(a, "id")).is_some_and(|x| x == s(b, "id"))
        {
            let context_a = super::quantified_match::associated_targets(self.source, s(a, "id"));
            let context_b = super::quantified_match::associated_targets(self.question, s(b, "id"));
            if context_a.as_ref().is_none_or(|v| !v.is_empty())
                || context_b.as_ref().is_none_or(|v| !v.is_empty())
            {
                if super::quantified_match::reference_pattern_matches(
                    self.source,
                    s(a, "id"),
                    self.question,
                    s(b, "id"),
                    s(self.scope, "namespace"),
                    map,
                ) {
                    Relation::Same
                } else {
                    Relation::Unknown
                }
            } else {
                Relation::Same
            }
        } else {
            self.term(self.source, a, self.question, b)
        }
    }
    fn mapped_atom(
        &self,
        a: &Value,
        b: &Value,
        slot: Option<&str>,
        map: &BTreeMap<String, String>,
    ) -> Relation {
        if a["predicate"] != b["predicate"] {
            return Relation::Different;
        }
        let mut relation = Relation::Same;
        for arg in rows(b, "arguments") {
            if Some(s(arg, "role")) == slot {
                continue;
            }
            let Some(value) = argument(a, s(arg, "role")) else {
                return Relation::Unknown;
            };
            if (super::descriptions::contains(a) || super::descriptions::contains(b))
                && s(value, "kind") == "ref"
                && s(&arg["value"], "kind") == "ref"
                && !self.description_component_agreement(
                    self.source,
                    s(value, "id"),
                    self.question,
                    s(&arg["value"], "id"),
                )
            {
                relation = relation.merge(Relation::Unknown);
            } else {
                relation = relation.merge(self.mapped_term(value, &arg["value"], map));
            }
        }
        for arg in rows(a, "arguments") {
            if Some(s(arg, "role")) != slot && argument(b, s(arg, "role")).is_none() {
                return Relation::Unknown;
            }
        }
        relation
    }
    fn alpha_map(&self, c: &Value, r: &Value) -> Option<BTreeMap<String, String>> {
        let a = self.binders(self.source, s(c, "scope_id"));
        let b = self.binders(self.question, s(r, "scope_id"));
        if a.is_empty() || a.len() != b.len() {
            return None;
        }
        let descriptive = super::descriptions::contains(&c["body"])
            || super::descriptions::contains(&r["target"]);
        let map: BTreeMap<_, _> = a
            .iter()
            .zip(&b)
            .map(|(a, b)| (s(a, "variable").to_owned(), s(b, "variable").to_owned()))
            .collect();
        for (a, b) in a.iter().zip(b) {
            if a["quantifier"] != b["quantifier"] || a["witness_relation"] != b["witness_relation"]
            {
                return None;
            }
            let domain_a = &a["domain_ref"];
            let domain_b = &b["domain_ref"];
            if domain_a.is_null() != domain_b.is_null() {
                return None;
            }
            if descriptive
                && (!self.description_component_agreement(
                    self.source,
                    s(a, "variable"),
                    self.question,
                    s(b, "variable"),
                ) || !domain_a.is_null()
                    && !self.description_component_agreement(
                        self.source,
                        domain_a.as_str().unwrap_or(""),
                        self.question,
                        domain_b.as_str().unwrap_or(""),
                    ))
            {
                return None;
            }
            if !domain_a.is_null()
                && self.mapped_term(
                    &json!({"kind":"ref","id":domain_a}),
                    &json!({"kind":"ref","id":domain_b}),
                    &map,
                ) != Relation::Same
                && !super::quantified_match::reference_pattern_matches(
                    self.source,
                    domain_a.as_str().unwrap_or(""),
                    self.question,
                    domain_b.as_str().unwrap_or(""),
                    s(self.scope, "namespace"),
                    &map,
                )
            {
                return None;
            }
            let ca = &a["cardinality"];
            let cb = &b["cardinality"];
            if ca.is_null() != cb.is_null() {
                return None;
            }
            if descriptive
                && !ca.is_null()
                && !self.description_component_agreement(
                    self.source,
                    s(ca, "domain"),
                    self.question,
                    s(cb, "domain"),
                )
            {
                return None;
            }
            if !ca.is_null()
                && (ca["relation"] != cb["relation"]
                    || ca["value"] != cb["value"]
                    || self.mapped_term(
                        &json!({"kind":"ref","id":ca["domain"]}),
                        &json!({"kind":"ref","id":cb["domain"]}),
                        &map,
                    ) != Relation::Same
                        && !super::quantified_match::reference_pattern_matches(
                            self.source,
                            s(ca, "domain"),
                            self.question,
                            s(cb, "domain"),
                            s(self.scope, "namespace"),
                            &map,
                        ))
            {
                return None;
            }
            let ar = self.reference(self.source, s(a, "variable"))?;
            let br = self.reference(self.question, s(b, "variable"))?;
            if ar["denotation"] != br["denotation"]
                || ar["declared_class"]["quote"] != br["declared_class"]["quote"]
                || ar["semantic_type"]["type_id"] != br["semantic_type"]["type_id"]
            {
                return None;
            }
            if !super::quantified_match::reference_pattern_matches(
                self.source,
                s(a, "variable"),
                self.question,
                s(b, "variable"),
                s(self.scope, "namespace"),
                &map,
            ) && (!super::quantified_match::associated_targets(self.source, s(a, "variable"))?
                .is_empty()
                || !super::quantified_match::associated_targets(self.question, s(b, "variable"))?
                    .is_empty())
            {
                return None;
            }
            for link in rows(ar, "links") {
                if !rows(br, "links").iter().any(|l| {
                    l["kind"] == link["kind"]
                        && self.mapped_term(
                            &json!({"kind":"ref","id":link["target"]}),
                            &json!({"kind":"ref","id":l["target"]}),
                            &map,
                        ) == Relation::Same
                }) {
                    return None;
                }
            }
            if rows(ar, "links").len() != rows(br, "links").len() {
                return None;
            }
            // Relation closures are part of a quantified pattern, not decorations.
            if rows(a, "relation_closures").len() != rows(b, "relation_closures").len() {
                return None;
            }
            for ac in rows(a, "relation_closures") {
                if !rows(b, "relation_closures").iter().any(|bc| {
                    ac["closed_role"] == bc["closed_role"]
                        && self.mapped_term(
                            &json!({"kind":"ref","id":ac["domain_binding"]}),
                            &json!({"kind":"ref","id":bc["domain_binding"]}),
                            &map,
                        ) == Relation::Same
                        && self.mapped_term(
                            &json!({"kind":"ref","id":ac["object_binding"]}),
                            &json!({"kind":"ref","id":bc["object_binding"]}),
                            &map,
                        ) == Relation::Same
                        && self.mapped_atom(
                            &ac["target"]["atom"],
                            &bc["target"]["atom"],
                            None,
                            &map,
                        ) == Relation::Same
                        && ac["target"]["judgment"] == bc["target"]["judgment"]
                        && ac["target"]["status"] == bc["target"]["status"]
                        && ac["target"]["basis"] == bc["target"]["basis"]
                        && self
                            .context(
                                self.source,
                                rows(&ac["target"], "qualifier_ids"),
                                self.question,
                                rows(&bc["target"], "qualifier_ids"),
                                false,
                            )
                            .0
                            == Relation::Same
                }) {
                    return None;
                }
            }
        }
        Some(map)
    }
    fn match_atom(&self, c: &Value, r: &Value, p: &Value, slot: Option<&str>) -> Relation {
        let a = self.binders(self.source, s(c, "scope_id"));
        let b = self.binders(self.question, s(r, "scope_id"));
        if a.is_empty() && b.is_empty() {
            return self.atom(
                self.source,
                &p["atom"],
                self.question,
                &r["target"]["atom"],
                slot,
            );
        }
        // Quantified claims cannot be instantiated as a definite individual answer.
        if let Some(map) = self.alpha_map(c, r) {
            self.mapped_atom(&p["atom"], &r["target"]["atom"], slot, &map)
        } else if p["atom"]["predicate"] == r["target"]["atom"]["predicate"] {
            Relation::Unknown
        } else {
            Relation::Different
        }
    }
    fn shared_refutation(&self, c: &Value, r: &Value, out: &mut Evidence) {
        if s(r, "mode") != "truth" || s(&c["body"], "kind") != "proposition" {
            return;
        }
        let sb = self.binders(self.source, s(c, "scope_id"));
        let qb = self.binders(self.question, s(r, "scope_id"));
        if sb.len() != 2
            || qb.len() != 2
            || s(sb[0], "quantifier") != "forall"
            || s(sb[1], "quantifier") != "exists"
            || s(qb[0], "quantifier") != "exists"
            || s(qb[1], "quantifier") != "forall"
            || s(sb[1], "witness_relation") != "distinct_across_domain"
        {
            return;
        }
        if s(&sb[0]["cardinality"], "relation") != "exact"
            || sb[0]["cardinality"]["value"].as_u64().unwrap_or(0) < 2
        {
            return;
        }
        if self.identity(
            self.source,
            s(sb[0], "domain_ref"),
            self.question,
            s(qb[1], "domain_ref"),
        ) != Relation::Same
        {
            return;
        }
        let map = BTreeMap::from([
            (
                s(sb[0], "variable").to_owned(),
                s(qb[1], "variable").to_owned(),
            ),
            (
                s(sb[1], "variable").to_owned(),
                s(qb[0], "variable").to_owned(),
            ),
        ]);
        let p = &c["body"]["value"];
        let target = &r["target"];
        if self.mapped_atom(&p["atom"], &target["atom"], None, &map) != Relation::Same
            || p["judgment"] != target["judgment"]
            || p["status"] != target["status"]
            || p["atom"]["polarity"] != target["atom"]["polarity"]
            || self.scope_match(r, p) != Relation::Same
        {
            return;
        }
        let closed = sb
            .iter()
            .flat_map(|b| rows(b, "relation_closures"))
            .any(|cl| {
                s(cl, "domain_binding") == s(sb[0], "variable")
                    && s(cl, "object_binding") == s(sb[1], "variable")
                    && argument(&p["atom"], s(cl, "closed_role")).is_some_and(|t| {
                        s(t, "kind") == "ref" && s(t, "id") == s(sb[1], "variable")
                    })
                    && self.proposition(self.source, &cl["target"], self.source, p, None)
                        == Relation::Same
            });
        if !closed {
            return;
        }
        let mut request_qids = self.request_qualifiers(r);
        request_qids.extend(rows(&r["target"], "qualifier_ids").iter().cloned());
        let mut missing = self
            .context(
                self.source,
                &self.claim_qualifiers(c, p),
                self.question,
                &request_qids,
                true,
            )
            .1;
        missing.extend(self.filters(r, p, &[]));
        missing.extend(self.premises(r));
        if !self
            .dependency_missing(self.source, "claim", id(c))
            .is_empty()
        {
            missing.push("dependency".into());
        }
        if !self.branch(self.source, s(c, "scope_id")).is_empty() {
            missing.push("branch_choice".into());
        }
        if missing.is_empty() {
            out.refutation
                .push(self.witness(c, r, p, "direct", vec![], vec![], &[]));
        } else {
            out.unresolved.push(self.candidate(c, r, &missing));
        }
    }
    fn branches(&self, r: &Value, out: &mut Evidence) {
        let branch_partials = out.partial.clone();
        for alternatives in rows(self.source, "claims")
            .iter()
            .filter(|c| s(&c["body"], "kind") == "alternatives")
        {
            let body = &alternatives["body"];
            if s(body, "closure") != "closed" || rows(body, "branches").is_empty() {
                continue;
            }
            let mut per_branch: Vec<Vec<(&Value, &Value)>> = vec![];
            for branch in rows(body, "branches") {
                let mut witnesses = vec![];
                for w in &branch_partials {
                    if s(w, "kind") != "conditional_branch"
                        || !rows(w, "branch_ids")
                            .iter()
                            .any(|x| x.as_str() == Some(id(branch)))
                    {
                        continue;
                    }
                    let missing = strings(rows(w, "missing_constraints"));
                    if missing.iter().any(|m| m != "branch_choice") {
                        continue;
                    }
                    for cid in rows(w, "claim_ids").iter().filter_map(Value::as_str) {
                        if rows(branch, "claim_ids")
                            .iter()
                            .any(|x| x.as_str() == Some(cid))
                        {
                            if let Some(c) =
                                rows(self.source, "claims").iter().find(|c| id(c) == cid)
                            {
                                witnesses.push((w, c));
                            }
                        }
                    }
                }
                per_branch.push(witnesses);
            }
            if per_branch.iter().any(Vec::is_empty) {
                continue;
            }
            for (first_w, first_c) in &per_branch[0] {
                let mut selected = vec![(*first_w, *first_c)];
                for branch in per_branch.iter().skip(1) {
                    if let Some((w, c)) = branch.iter().find(|(_, c)| self.claim_equal(first_c, c))
                    {
                        selected.push((*w, *c));
                    }
                }
                if selected.len() != per_branch.len() {
                    continue;
                }
                let mut w = (*first_w).clone();
                let mut cids = vec![id(alternatives).to_owned()];
                let mut evidence = rows(alternatives, "evidence").to_vec();
                let mut bids = vec![];
                for (bw, bc) in selected {
                    cids.push(id(bc).to_owned());
                    evidence.extend(rows(bw, "evidence_refs").iter().cloned());
                    bids.extend(strings(rows(bw, "branch_ids")));
                }
                unique(&mut cids);
                unique(&mut bids);
                w["id"] = json!(format!("{}:{}:invariant", id(r), id(alternatives)));
                w["kind"] = json!("shared_invariant");
                w["claim_ids"] = json!(cids);
                w["evidence_refs"] = json!(evidence);
                w["branch_ids"] = json!(bids);
                w["missing_constraints"] = json!([]);
                let negative = if s(&first_c["body"], "kind") == "event" {
                    s(&first_c["body"], "status") == "negated_occurrence"
                } else {
                    let p = &first_c["body"]["value"];
                    let target = &r["target"];
                    p["atom"]["polarity"] != target["atom"]["polarity"]
                        || (normative_conflict(s(p, "judgment"), s(target, "judgment"))
                            && s(&p["atom"], "polarity") == "positive"
                            && s(&target["atom"], "polarity") == "positive")
                };
                out.publish(w, negative);
            }
        }
    }
    fn claim_equal(&self, a: &Value, b: &Value) -> bool {
        // Exact qualified contents only. Different local IDs are not erased.
        match s(&a["body"], "kind") {
            "proposition" => {
                if s(&b["body"], "kind") != "proposition"
                    || self.proposition(
                        self.source,
                        &a["body"]["value"],
                        self.source,
                        &b["body"]["value"],
                        None,
                    ) != Relation::Same
                {
                    return false;
                }
            }
            "event" => {
                if s(&b["body"], "kind") != "event"
                    || self.local(
                        self.source,
                        s(&a["body"], "event_ref"),
                        s(&b["body"], "event_ref"),
                    ) != Relation::Same
                    || self.local(
                        self.source,
                        s(&a["body"], "subject"),
                        s(&b["body"], "subject"),
                    ) != Relation::Same
                {
                    return false;
                }
                let mut ab = a["body"].clone();
                let mut bb = b["body"].clone();
                for key in ["event_ref", "subject"] {
                    ab[key] = Value::Null;
                    bb[key] = Value::Null;
                }
                if semantic(&ab) != semantic(&bb) {
                    return false;
                }
            }
            _ => return false,
        }
        let aq = self.claim_qualifiers(a, &a["body"]["value"]);
        let bq = self.claim_qualifiers(b, &b["body"]["value"]);
        if aq.len() != bq.len() {
            return false;
        }
        aq.iter().all(|q| {
            bq.iter().any(|x| {
                self.qualifier(self.source, q.as_str().unwrap_or(""))
                    .zip(self.qualifier(self.source, x.as_str().unwrap_or("")))
                    .is_some_and(|(a, b)| {
                        self.qualifier_relation(self.source, a, self.source, b) == Relation::Same
                    })
            })
        })
    }
    fn event_witness(&self, w: &Value) -> Option<&Value> {
        rows(self.source, "claims").iter().find(|c| {
            s(&c["body"], "kind") == "event"
                && rows(w, "claim_ids")
                    .iter()
                    .any(|x| x.as_str() == Some(id(c)))
        })
    }
    fn event_witness_relation(&self, a: &Value, b: &Value) -> Relation {
        let (Some(a), Some(b)) = (self.event_witness(a), self.event_witness(b)) else {
            return Relation::Unknown;
        };
        let ab = &a["body"];
        let bb = &b["body"];
        if ab["event_kind"] != bb["event_kind"] || ab["aspect"] != bb["aspect"] {
            return Relation::Different;
        }
        self.local(self.source, s(ab, "event_ref"), s(bb, "event_ref"))
            .merge(self.local(self.source, s(ab, "subject"), s(bb, "subject")))
            .merge(
                self.context(
                    self.source,
                    &self.claim_qualifiers(a, &Value::Null),
                    self.source,
                    &self.claim_qualifiers(b, &Value::Null),
                    false,
                )
                .0,
            )
    }
    fn conclusion(&self, w: &Value) -> Option<(&Value, &Value)> {
        // The borrow of the engine outlives the returned source nodes.
        let c = rows(self.source, "claims").iter().find(|c| {
            rows(w, "claim_ids")
                .iter()
                .any(|x| x.as_str() == Some(id(c)))
        })?;
        match s(&c["body"], "kind") {
            "proposition" => Some((c, &c["body"]["value"])),
            "rule" => Some((c, &c["body"]["action"])),
            _ => None,
        }
    }
    fn contradictions(&self, r: &Value, out: &mut Evidence) {
        for support in &out.support {
            for negative in &out.refutation {
                let relation = match (self.conclusion(support), self.conclusion(negative)) {
                    (Some((a, pa)), Some((b, pb))) => {
                        let mut rel = self.atom(
                            self.source,
                            &pa["atom"],
                            self.source,
                            &pb["atom"],
                            if s(r, "mode") == "truth" {
                                None
                            } else {
                                Some(s(r, "slot"))
                            },
                        );
                        if pa["status"] != pb["status"] {
                            continue;
                        }
                        rel = rel.merge(
                            self.context(
                                self.source,
                                &self.claim_qualifiers(a, pa),
                                self.source,
                                &self.claim_qualifiers(b, pb),
                                false,
                            )
                            .0,
                        );
                        if !pa["normative_scope"].is_null() || !pb["normative_scope"].is_null() {
                            rel = rel.merge(self.local(
                                self.source,
                                s(pa, "normative_scope"),
                                s(pb, "normative_scope"),
                            ));
                        }
                        rel
                    }
                    _ if self.event_witness(support).is_some()
                        && self.event_witness(negative).is_some() =>
                    {
                        self.event_witness_relation(support, negative)
                    }
                    _ => Relation::Same,
                };
                if relation != Relation::Different {
                    out.conflicts.push(json!({"support_witness_ids":[support["id"]],"refutation_witness_ids":[negative["id"]],"kind":if relation==Relation::Same{"direct"}else{"potential"}}));
                }
            }
        }
        // Multiple values are contradictory only if explicit identity/scalar evidence proves it.
        for i in 0..out.support.len() {
            for j in i + 1..out.support.len() {
                let a = &out.support[i];
                let b = &out.support[j];
                if s(r, "required_answer_extent") == "all_values" || s(r, "mode") == "constraint" {
                    continue;
                }
                let mut different = false;
                for av in rows(a, "answer_terms") {
                    for bv in rows(b, "answer_terms") {
                        if self.term(self.source, av, self.source, bv) == Relation::Different {
                            different = true;
                        }
                    }
                }
                if different
                    && (self.same_witness_context(a, b)
                        || self.event_witness(a).is_some()
                            && self.event_witness(b).is_some()
                            && self.event_witness_relation(a, b) == Relation::Same)
                {
                    out.conflicts.push(json!({"support_witness_ids":[a["id"],b["id"]],"refutation_witness_ids":[],"kind":"direct"}));
                }
            }
        }
        // Pending contrary propositions block a confident answer, without choosing a side.
        for pending in &out.unresolved {
            let pc = rows(self.source, "claims").iter().find(|c| {
                rows(pending, "claim_ids")
                    .iter()
                    .any(|x| x.as_str() == Some(id(c)))
            });
            let Some(pc) = pc else { continue };
            let pp = match s(&pc["body"], "kind") {
                "proposition" => &pc["body"]["value"],
                "rule" => &pc["body"]["action"],
                _ => continue,
            };
            for w in out.support.iter().chain(&out.refutation) {
                let Some((_, p)) = self.conclusion(w) else {
                    continue;
                };
                if pp["atom"]["predicate"] != p["atom"]["predicate"] || pp["status"] != p["status"]
                {
                    continue;
                }
                let contrary = pp["judgment"] == p["judgment"]
                    && pp["atom"]["polarity"] != p["atom"]["polarity"]
                    || normative_conflict(s(pp, "judgment"), s(p, "judgment"));
                if contrary {
                    out.conflicts.push(json!({"support_witness_ids":[w["id"]],"refutation_witness_ids":[],"kind":"potential"}));
                }
            }
        }
    }
    fn same_witness_context(&self, a: &Value, b: &Value) -> bool {
        let (Some((ac, ap)), Some((bc, bp))) = (self.conclusion(a), self.conclusion(b)) else {
            return false;
        };
        ap["status"] == bp["status"]
            && ap["judgment"] == bp["judgment"]
            && self
                .context(
                    self.source,
                    &self.claim_qualifiers(ac, ap),
                    self.source,
                    &self.claim_qualifiers(bc, bp),
                    false,
                )
                .0
                == Relation::Same
            && (ap["normative_scope"].is_null() && bp["normative_scope"].is_null()
                || self.local(
                    self.source,
                    s(ap, "normative_scope"),
                    s(bp, "normative_scope"),
                ) == Relation::Same)
    }
    fn coverage_candidates(&self, r: &Value, out: &mut Evidence) {
        for (doc, report) in [
            (self.source, self.source_report),
            (self.question, self.question_report),
        ] {
            for cov in rows(doc, "coverage")
                .iter()
                .filter(|c| s(c, "state") != "represented")
            {
                let owner = &cov["owner"];
                let pertinent = if owner.is_null() {
                    true
                } else {
                    match s(owner, "kind") {
                        "request" => std::ptr::eq(doc, self.question) && s(owner, "id") == id(r),
                        "claim" => rows(doc, "claims")
                            .iter()
                            .find(|c| id(c) == s(owner, "id"))
                            .is_some_and(|c| {
                                let b = &c["body"];
                                match s(b, "kind") {
                                    "proposition" => {
                                        b["value"]["atom"]["predicate"]
                                            == r["target"]["atom"]["predicate"]
                                    }
                                    "event" => b["event_kind"] == r["target"]["event_kind"],
                                    _ => true,
                                }
                            }),
                        "reference" => r.to_string().contains(&format!("\"{}\"", s(owner, "id"))),
                        _ => true,
                    }
                };
                if pertinent {
                    out.unresolved.push(json!({"claim_ids":if s(owner,"kind")=="claim"{vec![s(owner,"id")]}else{vec![]},"request_id":id(r),"reference_ids":[],"state":"unresolved","reason_codes":["dependency"],"dependency_paths":[format!("coverage:{}",s(cov,"issue_id"))],"evidence_refs":[cov["span"]]}));
                }
            }
            // Validator carries unknown vocabulary/dependency paths without discarding candidates.
            for candidate in rows(report, "candidates")
                .iter()
                .filter(|c| s(c, "state") == "unresolved")
            {
                if std::ptr::eq(doc, self.question) && candidate["request_id"] == r["id"] {
                    out.unresolved.push(candidate.clone());
                }
            }
        }
    }
    fn pending_profile_applies(&self, c: &Value, r: &Value) -> bool {
        let b = &c["body"];
        let p = match s(b, "kind") {
            "proposition" => b["value"].clone(),
            "rule" => b["action"].clone(),
            "value_restriction" => {
                json!({"atom":b["target"],"normative_scope":b["normative_scope"],"qualifier_ids":c["qualifier_ids"]})
            }
            "event" => {
                return r["target"].get("event_kind").is_some()
                    && b["event_kind"] == r["target"]["event_kind"]
                    && self.identity(
                        self.source,
                        s(b, "subject"),
                        self.question,
                        s(&r["target"], "subject"),
                    ) != Relation::Different
                    && (r["target"]["aspect"].is_null()
                        || b["aspect"].is_null()
                        || b["aspect"] == r["target"]["aspect"])
                    && self
                        .context(
                            self.source,
                            &self.claim_qualifiers(c, &Value::Null),
                            self.question,
                            &self.request_qualifiers(r),
                            true,
                        )
                        .0
                        != Relation::Different;
            }
            _ => return false,
        };
        if r["target"].get("atom").is_none() {
            return false;
        }
        let slot = if s(r, "mode") == "truth" || super::temporal_slot(s(r, "slot")).is_some() {
            None
        } else {
            Some(s(r, "slot"))
        };
        self.match_atom(c, r, &p, slot) != Relation::Different
            && self.scope_match(r, &p) != Relation::Different
            && self
                .context(
                    self.source,
                    &self.claim_qualifiers(c, &p),
                    self.question,
                    &self.request_qualifiers(r),
                    true,
                )
                .0
                != Relation::Different
    }
    fn request(&self, r: &Value) -> Value {
        if let Some(pending) = super::normative_indeterminacy::pending(
            self.question,
            r,
            self.question_report,
            self.vocabulary,
        ) {
            return pending;
        }
        if super::nominal_possessive::has(self.possessive_question, "request", id(r)) {
            return self.possessive_request(r);
        }
        self.relative_bound_request(r)
            .unwrap_or_else(|| self.request_without_relative_bound(r))
    }
    fn request_without_relative_bound(&self, r: &Value) -> Value {
        let result = self
            .request_context_request(r)
            .unwrap_or_else(|| self.request_legacy(r));
        self.pending_task_scope(r, result)
    }
    fn request_legacy(&self, r: &Value) -> Value {
        let mut out = Evidence::default();
        self.possessive_candidates(r, &mut out);
        if let Some(reason) = profile_pending_reason(self.question_report, "request", id(r)) {
            let mut candidate = rows(self.question_report, "candidates")
                .iter()
                .find(|c| c["request_id"] == r["id"])
                .cloned()
                .unwrap_or(Value::Null);
            candidate["state"] = json!("unresolved");
            let mut reasons = vec![reason.to_owned()];
            if rows(&candidate, "reason_codes").contains(&json!("unsupported_description")) {
                candidate["request_constraint"] = r.clone();
                reasons.push("unsupported_description".into());
            }
            unique(&mut reasons);
            return json!({"request_id":id(r),"support":[],"refutation":[],"partial":[],"unresolved":[candidate],"conflicts":[],"missing_constraints":reasons,"coverage":"none"});
        }
        for c in self.pending_claims.iter().filter(|c| {
            !super::nominal_possessive::has(self.possessive_source, "claim", id(c))
                && !self
                    .lexical_source
                    .contains(&("claim".to_owned(), id(c).to_owned()))
                && self.nominal_pending_applies(c, r)
        }) {
            let reason = profile_pending_reason(self.source_report, "claim", id(c))
                .unwrap_or("local_nominal_metadata");
            if !self.nominal_metadata_context(c, r, &mut out) {
                let nominal = super::request_context_profile::permits(
                    self.vocabulary,
                    super::nominal_de::KIND,
                );
                if nominal {
                    let bytes = [c, r, &self.source_report["document"]["references"]]
                        .into_iter()
                        .try_fold(2048usize, |n, v| {
                            n.checked_add(super::context_budget::serialized_bytes(v)?)
                        })
                        .unwrap_or(usize::MAX);
                    if !super::context_budget::charge(1, bytes) {
                        continue;
                    }
                }
                let mut candidate = self.candidate(c, r, &[reason.into()]);
                if nominal {
                    candidate["source_constraint"] = c.clone();
                    candidate["source_references"] =
                        self.source_report["document"]["references"].clone();
                    candidate["request_constraint"] = r.clone();
                }
                out.unresolved.push(candidate);
            }
        }
        if s(r, "mode") == "configuration_slot" {
            self.configuration_slot(r, &mut out);
        } else {
            self.measurement_gap_context(r, &mut out);
            self.events(r, &mut out);
            if self.context_v2 {
                self.event_pattern_context(r, &mut out);
                self.partial_negative_measurement_context(r, &mut out);
            }
            self.exemption_status(r, &mut out);
            for c in rows(self.source, "claims") {
                match s(&c["body"], "kind") {
                    "proposition" => {
                        if self.context_v2 {
                            self.admission_request_context(c, r, &mut out);
                            self.water_operand_context(c, r, &mut out);
                            self.anonymous_normative_context(c, r, &mut out);
                            self.partial_documentary_context(c, r, &mut out);
                            self.partial_assignment_context(c, r, &mut out);
                        }
                        let before = out.refutation.len();
                        self.shared_refutation(c, r, &mut out);
                        if out.refutation.len() == before {
                            self.direct(c, r, &c["body"]["value"], "direct", &[], &mut out);
                        }
                        self.relevance(c, r, &mut out);
                        self.scoped_assignment(c, r, &mut out);
                    }
                    "value_restriction" => {
                        if self.context_v2 {
                            self.admission_request_context(c, r, &mut out);
                        }
                        self.restrictions(c, r, &mut out);
                        self.admission_context(c, r, &mut out);
                        self.relevance(c, r, &mut out);
                    }
                    "rule" => self.rule_conditions(c, r, &mut out),
                    _ => {}
                }
            }
        }
        let assignment_values: Vec<_> = out
            .support
            .iter()
            .chain(&out.partial)
            .filter(|w| s(w, "kind") == "scoped_assignment")
            .flat_map(|w| rows(w, "answer_terms").iter().cloned())
            .collect();
        if assignment_values.first().is_some_and(|first| {
            assignment_values
                .iter()
                .skip(1)
                .any(|other| self.term(self.source, first, self.source, other) != Relation::Same)
        }) {
            let ids: Vec<_> = out
                .support
                .iter()
                .chain(&out.partial)
                .filter(|w| s(w, "kind") == "scoped_assignment")
                .map(|w| w["id"].clone())
                .collect();
            let mut retained = vec![];
            for mut witness in std::mem::take(&mut out.support) {
                if s(&witness, "kind") == "scoped_assignment" {
                    witness["missing_constraints"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!("assignment_occurrence"));
                    out.partial(witness);
                } else {
                    retained.push(witness);
                }
            }
            out.support = retained;
            for witness in &mut out.partial {
                if s(witness, "kind") == "scoped_assignment"
                    && !rows(witness, "missing_constraints")
                        .contains(&json!("assignment_occurrence"))
                {
                    witness["missing_constraints"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!("assignment_occurrence"));
                }
            }
            out.missing.push("assignment_occurrence".into());
            out.conflicts.push(json!({"kind":"potential","support_witness_ids":ids,"refutation_witness_ids":[],"reason":"assignment_occurrence"}));
        }
        if out.support.is_empty()
            && out.refutation.is_empty()
            && !out
                .partial
                .iter()
                .any(|w| s(w, "kind") == "scoped_assignment")
        {
            for witness in std::mem::take(&mut out.assignment_contrasts) {
                out.partial(witness);
            }
        }
        self.branches(r, &mut out);
        if s(r, "mode") == "answer"
            && super::temporal_slot(s(r, "slot")).is_some()
            && s(r, "required_answer_extent") != "all_values"
        {
            let witnesses: Vec<_> = out
                .support
                .iter()
                .chain(&out.partial)
                .filter(|w| {
                    s(w, "kind") != "partial_relevance"
                        && self.conclusion(w).is_some_and(|(_, p)| {
                            matches!(s(p, "status"), "planned" | "occurred")
                                && p["atom"]["predicate"] == r["target"]["atom"]["predicate"]
                        })
                })
                .collect();
            // Provenance inputs are not occurrence roots; one rule step is one conclusion.
            let mut ids: Vec<_> = witnesses
                .iter()
                .filter_map(|w| {
                    let (c, _) = self.conclusion(w)?;
                    let invariant = out.support.iter().find(|inv| {
                        s(inv, "kind") == "shared_invariant"
                            && rows(inv, "claim_ids")
                                .iter()
                                .any(|cid| cid.as_str() == Some(id(c)))
                    });
                    Some(invariant.map_or_else(|| id(c).to_owned(), |inv| id(inv).to_owned()))
                })
                .collect();
            unique(&mut ids);
            if ids.len() > 1 {
                out.unresolved.push(json!({"claim_ids":ids,"request_id":id(r),"reference_ids":[],"state":"unresolved","reason_codes":["occurrence_identity"],"dependency_paths":[],"evidence_refs":witnesses.iter().flat_map(|w|rows(w,"evidence_refs").iter().cloned()).collect::<Vec<_>>()}));
                out.missing.push("occurrence_identity".into());
            }
        }
        self.premise_refutations(r, &mut out);
        self.constrain_answer_pattern(r, &mut out);
        self.coverage_candidates(r, &mut out);
        self.contradictions(r, &mut out);
        let request_debt = self.dependency_missing(self.question, "request", id(r));
        if !request_debt.is_empty() {
            let claim_ids: BTreeSet<_> = out
                .support
                .iter()
                .chain(&out.refutation)
                .chain(&out.partial)
                .flat_map(|w| rows(w, "claim_ids").iter().filter_map(Value::as_str))
                .collect();
            let source_constraints: Vec<_> = rows(self.source, "claims")
                .iter()
                .filter(|c| claim_ids.contains(id(c)))
                .cloned()
                .collect();
            // Pending query context cannot publish an answer or selection. Keep
            // every proposed witness and its original source for later review.
            let candidate = json!({"claim_ids":[],"request_id":id(r),"reference_ids":[],"state":"unresolved","reason_codes":["dependency"],"dependency_paths":request_debt,"request_constraint":r,"source_constraints":source_constraints,"pending_evidence":{"support":out.support,"refutation":out.refutation,"partial":out.partial,"unresolved":out.unresolved,"conflicts":out.conflicts,"missing_constraints":out.missing},"evidence_refs":r["evidence"]});
            return json!({"request_id":id(r),"support":[],"refutation":[],"partial":[],"unresolved":[candidate],"conflicts":[],"missing_constraints":["dependency"],"coverage":"none"});
        }
        if !out.support.is_empty() || !out.refutation.is_empty() {
            if s(r, "required_answer_extent") == "all_values" && !out.closed {
                out.missing.push("answer_extent".into());
            }
        }
        // Fully discharged invariants supersede only the corresponding branch-choice debt.
        if out
            .support
            .iter()
            .chain(&out.refutation)
            .any(|w| s(w, "kind") == "shared_invariant")
        {
            out.missing.retain(|x| x != "branch_choice");
        }
        let complete = (!out.support.is_empty() || !out.refutation.is_empty())
            && out.conflicts.is_empty()
            && out.unresolved.is_empty()
            && !out.missing.contains(&"answer_extent".into());
        // The open construction is contextual evidence, not a debt in an
        // independent ordinary proof. Decide that proof before attaching it.
        // Full source owners remain in the global unresolved provenance too.
        let mut composition_context = Evidence::default();
        self.linguistic_composition_context(r, &mut composition_context);
        self.nominal_event_context(r, &mut composition_context);
        out.partial.extend(composition_context.partial);
        if !complete {
            out.unresolved.extend(composition_context.unresolved);
            out.missing.extend(composition_context.missing);
        }
        let useful =
            !out.support.is_empty() || !out.refutation.is_empty() || !out.partial.is_empty();
        let coverage = if !out.conflicts.is_empty() {
            "conflict"
        } else if complete {
            "complete"
        } else if useful {
            "partial"
        } else {
            "none"
        };
        if !complete && !useful && out.unresolved.is_empty() {
            out.missing.push("answer_slot".into());
        }
        unique(&mut out.missing);
        json!({"request_id":id(r),"support":out.support,"refutation":out.refutation,"partial":out.partial,"unresolved":out.unresolved,"conflicts":out.conflicts,"missing_constraints":out.missing,"coverage":coverage})
    }
}

fn profile_pending_reason<'a>(report: &'a Value, kind: &str, cid: &str) -> Option<&'a str> {
    rows(report, "candidates").iter().find_map(|c| {
        let matching = if kind == "claim" {
            rows(c, "claim_ids").iter().any(|x| x.as_str() == Some(cid))
        } else {
            c["request_id"].as_str() == Some(cid)
        };
        matching
            .then(|| {
                rows(c, "reason_codes")
                    .iter()
                    .filter_map(Value::as_str)
                    .find(|code| {
                        matches!(
                            *code,
                            "attribute_profile_mismatch"
                                | "unsupported_description"
                                | "unsupported_nominal_complement"
                                | "unsupported_nominal_possessive"
                                | "request_context_unsupported"
                                | "request_context_ambiguous"
                                | "request_context_extraction_unresolved"
                        )
                    })
            })
            .flatten()
    })
}
fn profile_pending(report: &Value, kind: &str, cid: &str) -> bool {
    profile_pending_reason(report, kind, cid).is_some()
}

/// Classifies two validator-owned results. JSON reports cannot impersonate validated documents.
pub fn classify_pair(
    source: &DocumentResult,
    question: &DocumentResult,
    vocabulary: &Value,
    pair_scope: &Value,
) -> Value {
    classify_pair_with_context(source, question, vocabulary, pair_scope, false)
}

/// Separate experimental context policy; the established classifier stays unchanged.
pub fn classify_pair_context(
    source: &DocumentResult,
    question: &DocumentResult,
    vocabulary: &Value,
    pair_scope: &Value,
) -> Value {
    classify_pair_with_context(source, question, vocabulary, pair_scope, true)
}

fn classify_pair_with_context(
    source: &DocumentResult,
    question: &DocumentResult,
    vocabulary: &Value,
    pair_scope: &Value,
    context_v2: bool,
) -> Value {
    if super::request_context_profile::enabled(vocabulary) && !super::context_budget::active() {
        return super::context_budget::with_pair_operation(vocabulary, &source.report["document"], &question.report["document"], ||
            classify_pair_with_context(source, question, vocabulary, pair_scope, context_v2))
            .unwrap_or_else(|reason|json!({"status":"invalid","requests":[],"selected_claim_ids":[],"unresolved_candidates":[{"reason_codes":[reason]}],"requires_review":true,"requires_qualification":true,"complete_answer":false}));
    }
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(serde_json::to_vec(vocabulary).unwrap_or_default())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let sd = &source.report["document"];
    let qd = &question.report["document"];
    let scope_ok = pair_scope.as_object().is_some_and(|m| {
        m.len() == 4
            && m.keys().all(|k| {
                matches!(
                    k.as_str(),
                    "namespace" | "configuration" | "time_selector" | "closed_world"
                )
            })
    }) && pair_scope["closed_world"] == false
        && (pair_scope["namespace"].is_null() || pair_scope["namespace"].is_string())
        && pair_scope["configuration"].is_array()
        && (pair_scope["time_selector"].is_null() || pair_scope["time_selector"].is_object());
    let invalid = s(&source.report, "status") == "invalid"
        || s(&question.report, "status") == "invalid"
        || source.vocabulary_sha256 != hash
        || question.vocabulary_sha256 != hash
        || s(sd, "version") != "documentary_evidence_v3"
        || s(qd, "version") != "documentary_evidence_v3"
        || s(sd, "kind") != "source"
        || s(qd, "kind") != "question"
        || rows(qd, "requests").is_empty()
        || !scope_ok
        || (super::request_context_profile::enabled(vocabulary) && !context_v2);
    if invalid {
        let mut candidates = rows(&source.report, "candidates").to_vec();
        candidates.extend(rows(&question.report, "candidates").iter().cloned());
        return json!({"status":"invalid","requests":rows(qd,"requests").iter().map(|r|json!({"request_id":id(r),"support":[],"refutation":[],"partial":[],"unresolved":[],"conflicts":[],"missing_constraints":["dependency"],"coverage":"none"})).collect::<Vec<_>>(),"selected_claim_ids":[],"unresolved_candidates":candidates,"requires_review":true,"requires_qualification":true,"complete_answer":false});
    }
    // This is a private execution view, not a new validated document or a raw repair.
    // Original reports retain every claim and issue, while unusable claims cannot
    // discharge guards, join events, or participate in branch invariants.
    let mut lexical_source = super::lexical_clause_context::blocked_nodes(sd);
    let composition_blocked = super::linguistic_composition::blocked_nodes(sd);
    if super::context_budget::composition_enabled() {
        if !super::context_budget::charge(
            composition_blocked.len(),
            composition_blocked
                .iter()
                .map(|(k, i)| k.len() + i.len() + 128)
                .sum(),
        ) {
            return json!({"status":"invalid","requests":[],"selected_claim_ids":[],"unresolved_candidates":[],"requires_review":true,"requires_qualification":true,"complete_answer":false});
        }
        lexical_source.extend(composition_blocked);
    }
    let possessive_source = super::nominal_possessive::blocked_nodes(sd);
    let possessive_question = super::nominal_possessive::blocked_nodes(qd);
    if super::context_budget::possessive_enabled()
        || super::context_budget::lexical_enabled()
        || super::context_budget::composition_enabled()
    {
        let bytes = super::context_budget::serialized_bytes(sd)
            .unwrap_or(usize::MAX)
            .saturating_mul(3);
        if !super::context_budget::charge(1, bytes) {
            // The operation boundary reports the shared technical failure and
            // retains inputs. Do not allocate an execution view after failure.
            return json!({"status":"invalid","requests":[],"selected_claim_ids":[],"unresolved_candidates":[],"requires_review":true,"requires_qualification":true,"complete_answer":false});
        }
    }
    let mut executable_source = sd.clone();
    let nominal_blocked =
        if super::request_context_profile::permits(vocabulary, super::nominal_de::KIND) {
            super::nominal_de::blocked_nodes(sd)
        } else {
            BTreeSet::new()
        };
    let pending_claims: Vec<_> = rows(sd, "claims")
        .iter()
        .filter(|c| {
            !lexical_source.contains(&("claim".to_owned(), id(c).to_owned()))
                && (profile_pending(&source.report, "claim", id(c))
                    || nominal_blocked.contains(&("claim".to_owned(), id(c).to_owned()))
                    || super::nominal_possessive::has(&possessive_source, "claim", id(c)))
        })
        .cloned()
        .collect();
    executable_source["claims"] = json!(
        rows(sd, "claims")
            .iter()
            .filter(|c| !profile_pending(&source.report, "claim", id(c))
                && !nominal_blocked.contains(&("claim".to_owned(), id(c).to_owned()))
                && !super::nominal_possessive::has(&possessive_source, "claim", id(c))
                && !lexical_source.contains(&("claim".to_owned(), id(c).to_owned())))
            .cloned()
            .collect::<Vec<_>>()
    );
    let engine = Engine {
        source: &executable_source,
        question: qd,
        source_report: &source.report,
        pending_claims: &pending_claims,
        question_report: &question.report,
        vocabulary,
        scope: pair_scope,
        context_v2,
        possessive_source: &possessive_source,
        lexical_source: &lexical_source,
        possessive_question: &possessive_question,
    };
    let requests: Vec<_> = rows(qd, "requests")
        .iter()
        .map(|r| engine.request(r))
        .collect();
    let mut selected = vec![];
    let mut unresolved = vec![];
    for r in &requests {
        for key in ["support", "refutation", "partial"] {
            for w in rows(r, key) {
                selected.extend(strings(rows(w, "claim_ids")));
            }
        }
        unresolved.extend(rows(r, "unresolved").iter().cloned());
    }
    if super::context_budget::lexical_enabled() || super::context_budget::composition_enabled() {
        for claim in rows(sd, "claims") {
            if !super::context_budget::charge(lexical_source.len().saturating_add(1), 0) {
                break;
            }
            if lexical_source.contains(&("claim".to_owned(), id(claim).to_owned())) {
                let bytes = super::context_budget::serialized_bytes(sd)
                    .unwrap_or(usize::MAX)
                    .saturating_add(
                        super::context_budget::serialized_bytes(claim).unwrap_or(usize::MAX),
                    )
                    .saturating_add(512);
                if !super::context_budget::charge(1, bytes) {
                    break;
                }
                unresolved.push(json!({"claim_ids":[id(claim)],"state":"unresolved","reason_codes":[if super::linguistic_composition::carries(claim) {"linguistic_composition_requires_intact_selection"}else{"lexical_clause_context_requires_intact_selection"}],"source_constraint":claim,"source_document":sd,"answer_terms":[],"answer_propositions":[],"bindings":[]}));
            }
        }
    }
    unique(&mut selected);
    let conflict = requests.iter().any(|r| !rows(r, "conflicts").is_empty());
    let complete = requests.iter().all(|r| s(r, "coverage") == "complete");
    let useful = requests.iter().any(|r| {
        ["support", "refutation", "partial"]
            .iter()
            .any(|k| !rows(r, k).is_empty())
    });
    let status = if conflict {
        "conflict"
    } else if !unresolved.is_empty() {
        "unresolved"
    } else if complete {
        "resolved"
    } else if useful {
        "partial"
    } else {
        "no_evidence"
    };
    let qualification = conflict
        || requests.iter().any(|r| {
            s(r, "coverage") != "complete"
                || !rows(r, "missing_constraints").is_empty()
                || rows(r, "support")
                    .iter()
                    .chain(rows(r, "refutation"))
                    .any(|w| {
                        !rows(w, "branch_ids").is_empty()
                            || !rows(w, "qualifier_ids").is_empty()
                            || w.get("request_context_witness").is_some()
                    })
        });
    json!({"status":status,"requests":requests,"selected_claim_ids":selected,"unresolved_candidates":unresolved,"requires_review":conflict||status=="unresolved","requires_qualification":qualification,"complete_answer":complete&&!conflict})
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    fn digest(v: &[u8]) -> String {
        Sha256::digest(v)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
    fn vocabulary() -> Value {
        json!({"version":"fresh-pair-controls-v1","predicate_rows":[
        {"id":"owns","roles":{"owner":"ref","object":"ref"},"distributive":true},
        {"id":"uses","roles":{"subject":"ref","object":"ref"},"distributive":true},
        {"id":"admits","roles":{"subject":"ref","object":"ref"},"distributive":true},
        {"id":"property","roles":{"subject":"ref","attribute":"attribute_id","value":"term"},"distributive":true},
        {"id":"quantity","roles":{"subject":"ref","attribute":"attribute_id","amount":"quantity"},"distributive":true},
        {"id":"documentary_availability","roles":{"subject":"ref","attribute":"attribute_id","state":"term","document_scope":"ref"},"distributive":true},
        {"id":"information_state","roles":{"subject":"ref","attribute":"attribute_id","state":"term"},"distributive":true},
        {"id":"action","roles":{"actor":"ref","action_kind":"action_id","object":"ref"},"distributive":true},
        {"id":"action_undergoer","roles":{"subject":"ref","action_kind":"action_id"},"distributive":true},
        {"id":"route_entry","roles":{"subject":"ref","via":"ref"},"distributive":true},
        {"id":"origin","roles":{"subject":"ref","origin":"ref"},"distributive":true}],"fixed_attribute_rows":["mass","pressure","validity_state"],"fixed_action_rows":["withdraw"],"semantic_type_rows":[],"relevance_links":[{"id":"route_context","source_predicate":"route_entry","target_predicate":"origin","bindings":[["source.subject","target.subject"]],"carry_roles":["source.via"],"conclusion":"partial_intermediate_only","max_hops":1,"allow_reverse":false,"allow_transitive_composition":false}, {"id":"documentary_context","source_predicate":"documentary_availability","target_predicate":"quantity","bindings":[["source.subject","target.subject"],["source.attribute","target.attribute"]],"carry_roles":["source.state","source.document_scope"],"conclusion":"partial_intermediate_only","max_hops":1,"allow_reverse":false,"allow_transitive_composition":false}, {"id":"information_context","source_predicate":"information_state","target_predicate":"quantity","bindings":[["source.subject","target.subject"],["source.attribute","target.attribute"]],"carry_roles":["source.state"],"conclusion":"partial_intermediate_only","max_hops":1,"allow_reverse":false,"allow_transitive_composition":false}]})
    }
    fn quote(text: &str, part: &str) -> Value {
        let start = text.find(part).unwrap();
        json!({"start_byte":start,"end_byte":start+part.len(),"quote":part})
    }
    fn doc(text: &str, kind: &str, names: &[&str]) -> Value {
        let q = quote(text, text);
        json!({"version":"documentary_evidence_v3","text_sha256":digest(text.as_bytes()),"kind":kind,"vocabulary_sha256":digest(&serde_json::to_vec(&vocabulary()).unwrap()),"domain_namespace":"fresh-controls","scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":names.iter().map(|name|json!({"id":name,"scope_id":"root","mention":quote(text,name),"name":quote(text,name),"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null})).collect::<Vec<_>>(),"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":[],"coverage":[{"span":q,"state":"represented","owner":null,"issue_id":null}]})
    }
    fn reference(r: &str) -> Value {
        json!({"kind":"ref","id":r})
    }
    fn atom(text: &str, predicate: &str, args: Vec<(&str, Value)>) -> Value {
        json!({"predicate":predicate,"arguments":args.into_iter().map(|(role,value)|json!({"role":role,"value":value})).collect::<Vec<_>>(),"polarity":"positive","evidence":[quote(text,text)]})
    }
    fn prop(atom: Value) -> Value {
        json!({"atom":atom,"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]})
    }
    fn claim(text: &str, cid: &str, p: Value) -> Value {
        json!({"id":cid,"scope_id":"root","qualifier_ids":[],"evidence":[quote(text,text)],"body":{"kind":"proposition","value":p}})
    }
    fn request(text: &str, mode: &str, slot: &str, p: &Value) -> Value {
        let mut target = p.clone();
        target.as_object_mut().unwrap().remove("basis");
        target.as_object_mut().unwrap().remove("normative_scope");
        json!({"id":"ask","scope_id":"root","mode":mode,"target":target,"slot":slot,"class_filter":null,"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":if mode=="truth"{"truth_status"}else{"one_value"},"normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[quote(text,text)]})
    }
    fn dependencies(d: &mut Value) {
        // Overdeclared explicit dependencies are deliberate test inputs: no hidden omission repairs.
        let mut deps = vec![];
        for rf in rows(d, "references") {
            deps.push(json!({"consumer":{"kind":"reference","id":id(rf)},"requires":{"kind":"scope","id":rf["scope_id"]},"role":"binding"}));
            for link in rows(rf, "links") {
                deps.push(json!({"consumer":{"kind":"reference","id":id(rf)},"requires":{"kind":"reference","id":link["target"]},"role":"identity"}));
            }
        }
        for (collection, kind) in [("claims", "claim"), ("requests", "request")] {
            for row in rows(d, collection) {
                deps.push(json!({"consumer":{"kind":kind,"id":id(row)},"requires":{"kind":"scope","id":row["scope_id"]},"role":"binding"}));
                for rf in rows(d, "references") {
                    deps.push(json!({"consumer":{"kind":kind,"id":id(row)},"requires":{"kind":"reference","id":id(rf)},"role":"identity"}));
                }
                for qualifier in rows(d, "qualifiers") {
                    deps.push(json!({"consumer":{"kind":kind,"id":id(row)},"requires":{"kind":"qualifier","id":id(qualifier)},"role":"configuration"}));
                }
                for premise in rows(row, "premise_ids") {
                    deps.push(json!({"consumer":{"kind":kind,"id":id(row)},"requires":{"kind":"claim","id":premise},"role":"premise"}));
                }
            }
        }
        d["dependencies"] = json!(deps);
    }
    fn pair(mut source: Value, st: &str, mut question: Value, qt: &str) -> Value {
        dependencies(&mut source);
        dependencies(&mut question);
        let a = super::super::validate(&source, st, &vocabulary());
        let b = super::super::validate(&question, qt, &vocabulary());
        assert_ne!(a.report["status"], "invalid", "source {}", a.report);
        assert_ne!(b.report["status"], "invalid", "question {}", b.report);
        classify_pair(
            &a,
            &b,
            &vocabulary(),
            &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
        )
    }
    fn ownership(st: &str, qt: &str) -> (Value, Value) {
        let mut sd = doc(st, "source", &["Mara", "Helia"]);
        let mut qd = doc(qt, "question", &["Mara", "Helia"]);
        let sp = prop(atom(
            st,
            "owns",
            vec![("owner", reference("Mara")), ("object", reference("Helia"))],
        ));
        let qp = prop(atom(
            qt,
            "owns",
            vec![("owner", reference("Mara")), ("object", reference("Helia"))],
        ));
        sd["claims"] = json!([claim(st, "ownership", sp)]);
        qd["requests"] = json!([request(qt, "truth", "status", &qp)]);
        (sd, qd)
    }
    fn associated_ownership() -> (Value, Value, &'static str, &'static str) {
        // A synthetic graph scaffold for deliberate mutations, not a source-fidelity
        // extraction fixture. The coherent event/constraint cases below construct
        // separate literal texts for matching and unmatched association contexts.
        let st = "Mara posee Helia; Helia se asocia con el grupo Faro y el grupo Muelle.";
        let qt = "¿Mara posee Helia asociada con el grupo Faro y el grupo Muelle?";
        let (mut source, mut question) = ownership(st, qt);
        for (d, text) in [(&mut source, st), (&mut question, qt)] {
            for anchor in ["Faro", "Muelle"] {
                let r = doc(text, "source", &[anchor])["references"][0].clone();
                d["references"].as_array_mut().unwrap().push(r);
            }
            d["references"][1]["links"] = json!([{"kind":"associated_with","target":"Faro","evidence":[quote(text,"Helia")]}]);
        }
        (source, question, st, qt)
    }
    #[test]
    fn association_matching_is_restrictive_and_unknown_never_selects_an_answer() {
        let (source, question, st, qt) = associated_ownership();
        assert_eq!(
            pair(source.clone(), st, question.clone(), qt)["complete_answer"],
            true
        );
        for target in [Some("Muelle"), None] {
            let mut changed = source.clone();
            changed["references"][1]["links"] = target.map_or_else(|| json!([]), |target| json!([{"kind":"associated_with","target":target,"evidence":[quote(st,"Helia")]}]));
            let result = pair(changed, st, question.clone(), qt);
            assert_eq!(result["complete_answer"], false, "{result}");
            assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
            assert!(
                !rows(&result, "unresolved_candidates").is_empty(),
                "{result}"
            );
        }
        // The extra source restriction preserves the existing useful partial
        // behavior when the question itself has no associated target.
        let mut bare = question;
        bare["references"][1]["links"] = json!([]);
        let result = pair(source, st, bare, qt);
        assert!(!rows(&result, "selected_claim_ids").is_empty(), "{result}");
        assert_eq!(result["complete_answer"], false);
    }
    #[test]
    fn local_aliases_cannot_bypass_associated_context_and_anonymous_is_not_identity() {
        let (mut source, question, st, qt) = associated_ownership();
        let mut alias = source["references"][1].clone();
        alias["id"] = json!("alias");
        alias["links"] = json!([]);
        source["references"].as_array_mut().unwrap().push(alias);
        source["identity"] =
            json!([{"kind":"same","left":"Helia","right":"alias","evidence":[quote(st,"Helia")]}]);
        source["references"][1]["links"][0]["target"] = json!("Muelle");
        assert!(
            rows(
                &pair(source.clone(), st, question.clone(), qt),
                "selected_claim_ids"
            )
            .is_empty()
        );
        source["references"][1]["links"][0]["target"] = json!("Faro");
        assert_eq!(
            pair(source.clone(), st, question.clone(), qt)["complete_answer"],
            true
        );
        let mut anonymous_question = question;
        source["references"][1]["name"] = Value::Null;
        source["references"][4]["name"] = Value::Null;
        anonymous_question["references"][1]["name"] = Value::Null;
        let anonymous = pair(source, st, anonymous_question, qt);
        assert_eq!(anonymous["complete_answer"], false);
        assert!(
            rows(&anonymous, "selected_claim_ids").is_empty(),
            "{anonymous}"
        );
    }
    #[test]
    fn association_is_bijective_and_pending_anchor_dependencies_do_not_disappear() {
        let (mut source, mut question, st, qt) = associated_ownership();
        source["references"][1]["links"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"associated_with","target":"Muelle","evidence":[quote(st,st)]}));
        question["references"][1]["links"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"associated_with","target":"Faro","evidence":[quote(qt,qt)]}));
        let result = pair(source, st, question, qt);
        assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
        let (mut source, mut question, st, qt) = associated_ownership();
        source["qualifiers"] = json!([{"id":"unknown-context","scope_id":"root","kind":"configuration","body":{"key":"context","value":{"kind":"text","value":"grupo","evidence":[quote(st,"grupo")]}},"resolution":"unknown","evidence":[quote(st,"grupo")]}]);
        dependencies(&mut source);
        dependencies(&mut question);
        source["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":"Faro"},"requires":{"kind":"qualifier","id":"unknown-context"},"role":"configuration"}));
        let a = super::super::validate(&source, st, &vocabulary());
        let b = super::super::validate(&question, qt, &vocabulary());
        assert_ne!(a.report["status"], "invalid", "{}", a.report);
        assert_ne!(b.report["status"], "invalid", "{}", b.report);
        let result = classify_pair(
            &a,
            &b,
            &vocabulary(),
            &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
        );
        assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
    }
    #[test]
    fn required_association_blocks_event_and_constraint_partial_answers() {
        let qt = "¿Cuál fue el resultado de la medición de presión de Helia asociada con Faro?";
        for context in ["Muelle", "Faro"] {
            let st = format!(
                "La medición Eira de presión de Helia asociada con {context} ocurrió y dio 7 bar."
            );
            let mut source = doc(&st, "source", &["Eira", "Helia", context]);
            let mut question = doc(qt, "question", &["Helia", "Faro"]);
            source["references"][1]["links"] = json!([{"kind":"associated_with","target":context,"evidence":[quote(&st,&format!("Helia asociada con {context}"))]}]);
            question["references"][0]["links"] = json!([{"kind":"associated_with","target":"Faro","evidence":[quote(qt,"Helia asociada con Faro")]}]);
            source["claims"] = json!([{"id":"measurement","scope_id":"root","qualifier_ids":[],"evidence":[quote(&st,&st)],"body":{"kind":"event","event_ref":"Eira","event_kind":"measurement","subject":"Helia","aspect":"pressure","status":"occurred","method":{"state":"unspecified"},"times":[],"result":{"kind":"quantity","decimal":"7","unit":"bar","bound":"exact","evidence":[quote(&st,"7 bar")]},"basis":"description"}}]);
            question["requests"] = json!([{"id":"ask","scope_id":"root","mode":"event_slot","target":{"event_ref":null,"event_kind":"measurement","subject":"Helia","aspect":"pressure","status":"occurred","time_selector":null,"method_selector":{"kind":"any"}},"slot":"result","class_filter":null,"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[quote(qt,qt)]}]);
            let result = pair(source, &st, question, qt);
            assert_eq!(result["complete_answer"], context == "Faro", "{result}");
            if context == "Muelle" {
                assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
                assert!(!rows(&result, "unresolved_candidates").is_empty());
            }
        }
        let qt = "¿Cuáles son todos los propietarios de Helia asociada con Faro?";
        for context in ["Muelle", "Faro"] {
            let st = format!("La única propietaria de Helia asociada con {context} es Mara.");
            let mut source = doc(&st, "source", &["Mara", "Helia", context]);
            let mut question = doc(qt, "question", &["Helia", "Faro"]);
            source["references"][1]["links"] = json!([{"kind":"associated_with","target":context,"evidence":[quote(&st,&format!("Helia asociada con {context}"))]}]);
            question["references"][0]["links"] = json!([{"kind":"associated_with","target":"Faro","evidence":[quote(qt,"Helia asociada con Faro")]}]);
            let target = atom(
                &st,
                "owns",
                vec![("object", reference("Helia")), ("owner", reference("Mara"))],
            );
            source["claims"] = json!([{"id":"owners","scope_id":"root","qualifier_ids":[],"evidence":[quote(&st,&st)],"body":{"kind":"value_restriction","target":target,"answer_role":"owner","closure":"closed","members":[reference("Mara")],"judgment":"fact","status":"state","basis":"description","normative_scope":null,"existence":"asserted"}}]);
            let desired = prop(atom(qt, "owns", vec![("object", reference("Helia"))]));
            let mut ask = request(qt, "constraint", "owner", &desired);
            ask["required_answer_extent"] = json!("all_values");
            question["requests"] = json!([ask]);
            let result = pair(source, &st, question, qt);
            assert_eq!(result["complete_answer"], context == "Faro", "{result}");
            if context == "Muelle" {
                assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
                assert!(!rows(&result, "unresolved_candidates").is_empty());
            }
        }
    }
    #[test]
    fn association_answer_pattern_matches_only_the_same_directed_kind() {
        let question = "¿De qué tubería de la red procede el agua del parque Olmo?";
        let (mut source, text, mut query) = network_filter_case("Olmo", question);
        // Both contexts remain anonymous descriptions of a network. This is
        // bounded answer-pattern satisfaction, not documentary entity identity.
        for d in [&mut source, &mut query] {
            d["references"][1]["links"][0]["kind"] = json!("associated_with");
        }
        assert_eq!(
            pair(source.clone(), &text, query.clone(), question)["complete_answer"],
            true
        );
        let mut missing = source.clone();
        missing["references"][1]["links"] = json!([]);
        let result = pair(missing, &text, query.clone(), question);
        assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
        for denotation in ["class", "group"] {
            let mut incompatible = source.clone();
            incompatible["references"][2]["denotation"] = json!(denotation);
            let result = pair(incompatible, &text, query.clone(), question);
            assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
        }
        for stronger in ["owned_by", "located_at", "member_of", "part_of"] {
            query["references"][1]["links"][0]["kind"] = json!(stronger);
            let result = pair(source.clone(), &text, query.clone(), question);
            assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
        }
    }
    #[test]
    fn named_answer_target_requires_anchoring_without_losing_class_partial() {
        // Deliberate graph mutation: omitting the quoted alias tests that text
        // alone is not treated as an operative identity edge.
        let st = "Mara posee Helia. Helia también se llama Kora.";
        for name in ["Helia", "Kora"] {
            let qt = format!("¿Quién posee el cilindro {name}?");
            let mut source = doc(st, "source", &["Mara", "Helia", "Kora"]);
            source["claims"] = json!([claim(
                st,
                "owner",
                prop(atom(
                    st,
                    "owns",
                    vec![("owner", reference("Mara")), ("object", reference("Helia"))]
                ))
            )]);
            let mut question = doc(&qt, "question", &[name]);
            question["references"][0]["declared_class"] = quote(&qt, "cilindro");
            question["requests"] = json!([request(
                &qt,
                "answer",
                "owner",
                &prop(atom(&qt, "owns", vec![("object", reference(name))]))
            )]);
            let result = pair(source.clone(), st, question.clone(), &qt);
            assert_eq!(
                !rows(&result, "selected_claim_ids").is_empty(),
                name == "Helia",
                "{result}"
            );
            assert_eq!(
                result["complete_answer"], false,
                "class is not established: {result}"
            );
            if name == "Kora" {
                assert!(
                    !rows(&result, "unresolved_candidates").is_empty(),
                    "{result}"
                );
                source["identity"] = json!([{"kind":"same","left":"Helia","right":"Kora","evidence":[quote(st,"Helia también se llama Kora")]}]);
                let aliased = pair(source, st, question, &qt);
                assert!(
                    !rows(&aliased, "selected_claim_ids").is_empty(),
                    "{aliased}"
                );
            }
        }
    }
    #[test]
    fn named_answer_target_keeps_universal_context_qualified() {
        let st = "Cada una de las dos guardianas de Ronda posee una llave.";
        let qt = "¿Qué llave posee Mara, miembro de Ronda?";
        let source = quantified(st, "source");
        let mut question = doc(qt, "question", &["Mara", "Ronda"]);
        question["references"][1]["denotation"] = json!("group");
        question["references"][0]["links"] = json!([{"kind":"member_of","target":"Ronda","evidence":[quote(qt,"miembro de Ronda")]}]);
        question["requests"] = json!([request(
            qt,
            "answer",
            "object",
            &prop(atom(qt, "owns", vec![("owner", reference("Mara"))]))
        )]);
        let result = pair(source, st, question, qt);
        assert!(!rows(&result, "selected_claim_ids").is_empty(), "{result}");
        assert_eq!(result["complete_answer"], false, "{result}");
        assert_eq!(result["requires_qualification"], true, "{result}");
        assert!(
            !rows(&result, "unresolved_candidates").is_empty(),
            "{result}"
        );
    }
    #[test]
    fn documentary_state_is_scoped_partial_context_without_a_requested_value() {
        let st = "La masa de Helia no consta en el registro.";
        for (name, attribute, word) in [
            ("Helia", "mass", "masa"),
            ("Orbe", "mass", "masa"),
            ("Helia", "pressure", "presión"),
        ] {
            let qt = format!("¿Qué {word} tiene el cilindro {name}?");
            let mut source = doc(st, "source", &["Helia", "registro"]);
            source["references"][1]["name"] = Value::Null;
            source["references"][1]["declared_class"] = quote(st, "registro");
            let p = prop(atom(
                st,
                "documentary_availability",
                vec![
                    ("subject", reference("Helia")),
                    (
                        "attribute",
                        json!({"kind":"symbol","id":"mass","evidence":[quote(st,"masa")]}),
                    ),
                    (
                        "state",
                        json!({"kind":"text","value":"no consta","evidence":[quote(st,"no consta")]}),
                    ),
                    ("document_scope", reference("registro")),
                ],
            ));
            source["claims"] = json!([claim(st, "availability", p.clone())]);
            let mut question = doc(&qt, "question", &[name]);
            question["references"][0]["declared_class"] = quote(&qt, "cilindro");
            question["requests"] = json!([request(
                &qt,
                "answer",
                "amount",
                &prop(atom(
                    &qt,
                    "quantity",
                    vec![
                        ("subject", reference(name)),
                        (
                            "attribute",
                            json!({"kind":"symbol","id":attribute,"evidence":[quote(&qt,word)]})
                        )
                    ]
                ))
            )]);
            let result = pair(source, st, question, &qt);
            let expected = name == "Helia" && attribute == "mass";
            assert_eq!(
                !rows(&result, "selected_claim_ids").is_empty(),
                expected,
                "{result}"
            );
            assert_eq!(result["complete_answer"], false, "{result}");
            if expected {
                let w = &result["requests"][0]["partial"][0];
                assert!(rows(w, "answer_terms").is_empty(), "{result}");
                assert_eq!(w["answer_propositions"], json!([p]), "{result}");
                assert!(rows(&result["requests"][0], "support").is_empty());
                assert!(rows(&result["requests"][0], "refutation").is_empty());
            }
        }
    }
    #[test]
    fn information_state_context_does_not_discharge_branches_or_basis() {
        let st = "La presión de Helia no consta.";
        let qt = "¿Qué presión tiene Helia?";
        for control in ["plain", "branch", "basis", "binding_order"] {
            let mut source = doc(st, "source", &["Helia"]);
            let p = prop(atom(
                st,
                "information_state",
                vec![
                    ("subject", reference("Helia")),
                    (
                        "attribute",
                        json!({"kind":"symbol","id":"pressure","evidence":[quote(st,"presión")]}),
                    ),
                    (
                        "state",
                        json!({"kind":"text","value":"no consta","evidence":[quote(st,"no consta")]}),
                    ),
                ],
            ));
            source["claims"] = json!([claim(st, "information", p.clone())]);
            let mut question = doc(qt, "question", &["Helia"]);
            question["requests"] = json!([request(
                qt,
                "answer",
                "amount",
                &prop(atom(
                    qt,
                    "quantity",
                    vec![
                        ("subject", reference("Helia")),
                        (
                            "attribute",
                            json!({"kind":"symbol","id":"pressure","evidence":[quote(qt,"presión")]})
                        )
                    ]
                ))
            )]);
            // Graph mutations test guards, not faithful extraction from this text.
            if control == "branch" {
                source["scopes"].as_array_mut().unwrap().push(json!({"id":"choice","parent_id":"root","kind":"branch","branch_id":"choice","binding":null}));
                source["claims"][0]["scope_id"] = json!("choice");
            } else if control == "basis" {
                question["requests"][0]["basis_requirement"] =
                    json!({"kind":"exact","value":"assignment"});
            }
            let result = if control == "binding_order" {
                let mut v = vocabulary();
                for link in v["relevance_links"].as_array_mut().unwrap() {
                    link["bindings"].as_array_mut().unwrap().reverse();
                }
                for d in [&mut source, &mut question] {
                    d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
                    dependencies(d);
                }
                let a = super::super::validate(&source, st, &v);
                let b = super::super::validate(&question, qt, &v);
                assert_eq!(a.report["status"], "complete", "{}", a.report);
                assert_eq!(b.report["status"], "complete", "{}", b.report);
                classify_pair(
                    &a,
                    &b,
                    &v,
                    &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
                )
            } else {
                pair(source, st, question, qt)
            };
            assert_eq!(
                !rows(&result, "selected_claim_ids").is_empty(),
                matches!(control, "plain" | "binding_order"),
                "{control}: {result}"
            );
            assert_eq!(result["complete_answer"], false, "{result}");
            if matches!(control, "plain" | "binding_order") {
                assert!(rows(&result["requests"][0]["partial"][0], "answer_terms").is_empty());
                assert_eq!(
                    result["requests"][0]["partial"][0]["answer_propositions"],
                    json!([p])
                );
            }
        }
    }
    #[test]
    fn scoped_assignment_keeps_subject_setting_and_quantity_together() {
        let base = "Helia usa ModoA con presión de 7 bar; Orbe usa ModoB con presión de 9 bar.";
        for control in [
            "match",
            "other_mode",
            "disabled",
            "planned",
            "missing_local_value",
            "conflicting_values",
            "admission",
            "negative",
            "setting_conflict",
            "branch",
            "matching_peer",
            "conflicting_peer",
            "negative_request",
        ] {
            let peer_mode = if control == "conflicting_peer" {
                "ModoA"
            } else {
                "ModoB"
            };
            let source_text = if matches!(control, "matching_peer" | "conflicting_peer") {
                format!("{base} Helia también usa {peer_mode} con presión de 11 bar.")
            } else {
                base.to_owned()
            };
            let st = source_text.as_str();
            let mode = if matches!(control, "other_mode" | "matching_peer") {
                "ModoB"
            } else {
                "ModoA"
            };
            let qt = format!("¿Qué presión usa Helia con el modo {mode}?");
            let mut v = vocabulary();
            if control != "disabled" {
                v["assignment_projection"] =
                    json!({"version":"uses_qualifier_assignment_v1","context_keys":["mode"]});
            }
            let mut source = doc(st, "source", &["Helia", "Orbe", "ModoA", "ModoB"]);
            let mut claims = vec![];
            let mut qualifiers = vec![];
            for (i, subject, setting, number) in
                [(0, "Helia", "ModoA", "7"), (1, "Orbe", "ModoB", "9")]
            {
                let value_id = format!("value{i}");
                let mode_id = format!("mode{i}");
                qualifiers.push(json!({"id":value_id,"scope_id":"root","kind":"configuration","body":{"key":"pressure","value":{"kind":"quantity","decimal":number,"unit":"bar","bound":"exact","evidence":[quote(st,&format!("{number} bar"))]}},"resolution":"known","evidence":[quote(st,&format!("{number} bar"))]}));
                qualifiers.push(json!({"id":mode_id,"scope_id":"root","kind":"configuration","body":{"key":"mode","value":reference(setting)},"resolution":"known","evidence":[quote(st,setting)]}));
                let mut c = claim(
                    st,
                    &format!("c{i}"),
                    prop(atom(
                        st,
                        "uses",
                        vec![
                            ("subject", reference(subject)),
                            ("object", reference(setting)),
                        ],
                    )),
                );
                c["qualifier_ids"] = json!([value_id, mode_id]);
                claims.push(c);
            }
            source["claims"] = json!(claims);
            source["qualifiers"] = json!(qualifiers);
            if matches!(control, "matching_peer" | "conflicting_peer") {
                let mut peer = source["claims"][0].clone();
                peer["id"] = json!("peer");
                peer["body"]["value"]["atom"]["arguments"][1]["value"] = reference(peer_mode);
                peer["qualifier_ids"] = json!([
                    if control == "conflicting_peer" {
                        "mode0"
                    } else {
                        "mode1"
                    },
                    "peer-value"
                ]);
                source["claims"].as_array_mut().unwrap().push(peer);
                source["qualifiers"].as_array_mut().unwrap().push(json!({"id":"peer-value","scope_id":"root","kind":"configuration","body":{"key":"pressure","value":{"kind":"quantity","decimal":"11","unit":"bar","bound":"exact","evidence":[quote(st,"11 bar")]}},"resolution":"known","evidence":[quote(st,"11 bar")]}));
            }
            // These graph mutations test mechanical guards, not source fidelity.
            if control == "planned" {
                source["claims"][0]["body"]["value"]["status"] = json!("planned");
            }
            if control == "missing_local_value" {
                source["claims"][0]["qualifier_ids"] = json!(["mode0"]);
            }
            if control == "conflicting_values" {
                source["claims"][0]["qualifier_ids"] = json!(["mode0", "value0", "value1"]);
            }
            if control == "admission" {
                source["claims"][0]["body"]["value"]["atom"]["predicate"] = json!("admits");
            }
            if control == "negative" {
                source["claims"][0]["body"]["value"]["atom"]["polarity"] = json!("negative");
            }
            if control == "setting_conflict" {
                source["qualifiers"][1]["body"]["value"] = reference("ModoB");
            }
            if control == "branch" {
                source["scopes"].as_array_mut().unwrap().push(json!({"id":"choice","parent_id":"root","kind":"branch","branch_id":"choice","binding":null}));
                source["claims"][0]["scope_id"] = json!("choice");
            }
            let mut question = doc(&qt, "question", &["Helia"]);
            question["qualifiers"] = json!([{"id":"setting","scope_id":"root","kind":"configuration","body":{"key":"mode","value":{"kind":"text","value":mode,"evidence":[quote(&qt,mode)]}},"resolution":"known","evidence":[quote(&qt,mode)]}]);
            let mut ask = request(
                &qt,
                "answer",
                "amount",
                &prop(atom(
                    &qt,
                    "quantity",
                    vec![
                        ("subject", reference("Helia")),
                        (
                            "attribute",
                            json!({"kind":"symbol","id":"pressure","evidence":[quote(&qt,"presión")]}),
                        ),
                    ],
                )),
            );
            ask["qualifier_ids"] = json!(["setting"]);
            ask["basis_requirement"] = json!({"kind":"exact","value":"assignment"});
            question["requests"] = json!([ask]);
            if control == "negative_request" {
                question["requests"][0]["target"]["atom"]["polarity"] = json!("negative");
            }
            for d in [&mut source, &mut question] {
                d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
                dependencies(d);
            }
            let a = super::super::validate(&source, st, &v);
            let b = super::super::validate(&question, &qt, &v);
            assert_eq!(a.report["status"], "complete", "{}", a.report);
            assert_eq!(b.report["status"], "complete", "{}", b.report);
            let result = classify_pair(
                &a,
                &b,
                &v,
                &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
            );
            let useful = matches!(
                control,
                "match" | "other_mode" | "matching_peer" | "conflicting_peer"
            );
            assert_eq!(
                !rows(&result, "selected_claim_ids").is_empty(),
                useful,
                "{control}: {result}"
            );
            assert!(
                !rows(&result, "selected_claim_ids").contains(&json!("c1")),
                "other device leaked: {result}"
            );
            assert_eq!(
                result["complete_answer"],
                matches!(control, "match" | "matching_peer"),
                "{control}: {result}"
            );
            if control == "match" {
                assert_eq!(
                    result["requests"][0]["support"][0]["answer_terms"][0]["decimal"],
                    "7"
                );
            }
            if control == "other_mode" {
                assert!(rows(&result["requests"][0]["partial"][0], "answer_terms").is_empty());
            }
            if control == "matching_peer" {
                assert!(
                    !rows(&result, "selected_claim_ids").contains(&json!("c0")),
                    "{result}"
                );
                assert_eq!(
                    result["requests"][0]["support"][0]["answer_terms"][0]["decimal"],
                    "11"
                );
            }
            if control == "conflicting_peer" {
                assert_eq!(result["status"], "conflict", "{result}");
                assert!(
                    rows(&result["requests"][0], "support").is_empty(),
                    "{result}"
                );
            }
        }
    }
    #[test]
    fn scoped_assignment_metadata_is_closed_and_typed() {
        let text = "Helia";
        for control in ["valid", "empty", "duplicate", "version", "extra", "roles"] {
            let mut v = vocabulary();
            v["assignment_projection"] =
                json!({"version":"uses_qualifier_assignment_v1","context_keys":["mode"]});
            match control {
                "empty" => v["assignment_projection"]["context_keys"] = json!([]),
                "duplicate" => v["assignment_projection"]["context_keys"] = json!(["mode", "mode"]),
                "version" => v["assignment_projection"]["version"] = json!("future-version"),
                "extra" => v["assignment_projection"]["extra"] = json!(true),
                "roles" => {
                    v["predicate_rows"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|r| r["id"] == "uses")
                        .unwrap()["roles"]["object"] = json!("term");
                }
                _ => {}
            }
            let mut raw = doc(text, "source", &["Helia"]);
            raw["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
            dependencies(&mut raw);
            let result = super::super::validate(&raw, text, &v);
            assert_eq!(
                result.report["status"] == "complete",
                control == "valid",
                "{control}: {}",
                result.report
            );
        }
    }
    #[test]
    fn named_premise_correction_keeps_missing_class_and_does_not_fill_date() {
        for control in ["same", "other_actor", "other_object", "positive", "alias"] {
            let actor = if control == "other_actor" {
                "Nara"
            } else {
                "Mara"
            };
            let object = if control == "other_object" {
                "Orbe"
            } else {
                "Helia"
            };
            let requested_actor = if control == "alias" { "Mora" } else { "Mara" };
            let st = format!(
                "{actor} {}retiró {object}.{}",
                if control == "positive" { "" } else { "no " },
                if control == "alias" {
                    " Mara también se llama Mora."
                } else {
                    ""
                }
            );
            let qt = format!("¿Cuándo retiró {requested_actor} el cilindro Helia?");
            let mut source = doc(&st, "source", &[actor, object]);
            if control == "alias" {
                source["references"]
                    .as_array_mut()
                    .unwrap()
                    .push(doc(&st, "source", &["Mora"])["references"][0].clone());
                source["identity"] = json!([{"kind":"same","left":"Mara","right":"Mora","evidence":[quote(&st,"Mara también se llama Mora")]}]);
            }
            let mut p = prop(atom(
                &st,
                "action",
                vec![
                    ("actor", reference(actor)),
                    (
                        "action_kind",
                        json!({"kind":"symbol","id":"withdraw","evidence":[quote(&st,"retiró")]}),
                    ),
                    ("object", reference(object)),
                ],
            ));
            p["status"] = json!("occurred");
            p["atom"]["polarity"] = json!(if control == "positive" {
                "positive"
            } else {
                "negative"
            });
            source["claims"] = json!([claim(&st, "denial", p.clone())]);
            let mut question = doc(&qt, "question", &[requested_actor, "Helia"]);
            question["references"][1]["declared_class"] = quote(&qt, "cilindro");
            let mut premise = prop(atom(
                &qt,
                "action",
                vec![
                    ("actor", reference(requested_actor)),
                    (
                        "action_kind",
                        json!({"kind":"symbol","id":"withdraw","evidence":[quote(&qt,"retiró")]}),
                    ),
                    ("object", reference("Helia")),
                ],
            ));
            premise["status"] = json!("occurred");
            question["claims"] = json!([claim(&qt, "premise", premise)]);
            let mut target = prop(atom(
                &qt,
                "action_undergoer",
                vec![
                    ("subject", reference("Helia")),
                    (
                        "action_kind",
                        json!({"kind":"symbol","id":"withdraw","evidence":[quote(&qt,"retiró")]}),
                    ),
                ],
            ));
            target["status"] = json!("occurred");
            let mut ask = request(&qt, "answer", "occurred_at", &target);
            ask["premise_ids"] = json!(["premise"]);
            question["requests"] = json!([ask]);
            let result = pair(source, &st, question, &qt);
            let useful = matches!(control, "same" | "alias");
            assert_eq!(
                !rows(&result, "selected_claim_ids").is_empty(),
                useful,
                "{control}: {result}"
            );
            assert_eq!(result["complete_answer"], false, "{result}");
            if useful {
                let w = &result["requests"][0]["partial"][0];
                assert_eq!(w["kind"], "partial_premise_refutation");
                assert!(rows(w, "answer_terms").is_empty());
                assert_eq!(w["answer_propositions"], json!([p]));
                assert!(rows(w, "missing_constraints").contains(&json!("identity")));
            }
        }
    }
    mod admission_request_context_tests {
        include!("admission_request_context_tests.rs");
    }
    mod admission_context_tests {
        include!("admission_context_tests.rs");
    }
    #[test]
    fn admission_caution_does_not_establish_actual_use_or_quantity() {
        for basis in ["description", "capability", "assignment", "specification"] {
            for control in [
                "negative",
                "closed",
                "positive",
                "matching_member",
                "other_device",
                "disabled",
                "conditional",
                "alias_member",
                "negative_closed",
            ] {
                let closed = matches!(
                    control,
                    "closed" | "matching_member" | "alias_member" | "negative_closed"
                );
                let subject = if control == "other_device" {
                    "Orbe"
                } else {
                    "Helia"
                };
                let object = if closed { "ModoA" } else { "ModoB" };
                let wanted = if control == "matching_member" {
                    "ModoA"
                } else {
                    "ModoB"
                };
                let st = if control == "negative_closed" {
                    format!("El único modo no admitido por {subject} es {object}.")
                } else if closed {
                    format!(
                        "El único modo admitido por {subject} es {object}.{}",
                        if control == "alias_member" {
                            " ModoA también se llama ModoB."
                        } else {
                            ""
                        }
                    )
                } else {
                    format!(
                        "{subject} {}admite {object}.",
                        if control == "positive" { "" } else { "no " }
                    )
                };
                let qt = format!("¿Qué presión usa Helia con el modo {wanted}?");
                let mut v = vocabulary();
                v["assignment_projection"] =
                    json!({"version":"uses_qualifier_assignment_v1","context_keys":["mode"]});
                if control != "disabled" {
                    v["relevance_links"].as_array_mut().unwrap().push(json!({"id":"admission_caution","source_predicate":"admits","target_predicate":"quantity","bindings":[["source.subject","target.subject"]],"carry_roles":["source.object"],"conclusion":"partial_intermediate_only","max_hops":1,"allow_reverse":false,"allow_transitive_composition":false}));
                }
                let mut source = doc(&st, "source", &[subject, object]);
                if control == "alias_member" {
                    source["references"]
                        .as_array_mut()
                        .unwrap()
                        .push(doc(&st, "source", &["ModoB"])["references"][0].clone());
                    source["identity"] = json!([{"kind":"same","left":"ModoA","right":"ModoB","evidence":[quote(&st,"ModoA también se llama ModoB")]}]);
                }
                let mut p = prop(atom(
                    &st,
                    "admits",
                    vec![
                        ("subject", reference(subject)),
                        ("object", reference(object)),
                    ],
                ));
                p["basis"] = json!(basis);
                p["atom"]["polarity"] = json!(if control != "negative_closed"
                    && (closed || control == "positive")
                {
                    "positive"
                } else {
                    "negative"
                });
                if closed {
                    source["claims"] = json!([{"id":"admission","scope_id":"root","qualifier_ids":[],"evidence":[quote(&st,&st)],"body":{"kind":"value_restriction","target":p["atom"],"answer_role":"object","closure":"closed","members":[reference(object)],"judgment":"fact","status":"state","basis":basis,"normative_scope":null,"existence":"asserted"}}]);
                } else {
                    source["claims"] = json!([claim(&st, "admission", p.clone())]);
                }
                // A conditional graph mutation checks applicability, not extraction fidelity.
                if control == "conditional" {
                    source["scopes"].as_array_mut().unwrap().push(json!({"id":"choice","parent_id":"root","kind":"branch","branch_id":"choice","binding":null}));
                    source["claims"][0]["scope_id"] = json!("choice");
                }
                let mut question = doc(&qt, "question", &["Helia"]);
                question["qualifiers"] = json!([{"id":"setting","scope_id":"root","kind":"configuration","body":{"key":"mode","value":{"kind":"text","value":wanted,"evidence":[quote(&qt,wanted)]}},"resolution":"known","evidence":[quote(&qt,wanted)]}]);
                let mut ask = request(
                    &qt,
                    "answer",
                    "amount",
                    &prop(atom(
                        &qt,
                        "quantity",
                        vec![
                            ("subject", reference("Helia")),
                            (
                                "attribute",
                                json!({"kind":"symbol","id":"pressure","evidence":[quote(&qt,"presión")]}),
                            ),
                        ],
                    )),
                );
                ask["qualifier_ids"] = json!(["setting"]);
                ask["basis_requirement"] = json!({"kind":"exact","value":"assignment"});
                question["requests"] = json!([ask]);
                for d in [&mut source, &mut question] {
                    d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
                    dependencies(d);
                }
                let a = super::super::validate(&source, &st, &v);
                let b = super::super::validate(&question, &qt, &v);
                assert_eq!(a.report["status"], "complete", "{}", a.report);
                assert_eq!(b.report["status"], "complete", "{}", b.report);
                for context_enabled in [false, true] {
                    let classify = if context_enabled {
                        classify_pair_context
                    } else {
                        classify_pair
                    };
                    let result = classify(
                        &a,
                        &b,
                        &v,
                        &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
                    );
                    let expected = (basis == "description"
                        || context_enabled && basis == "capability")
                        && matches!(control, "negative" | "closed");
                    assert_eq!(
                        !rows(&result, "selected_claim_ids").is_empty(),
                        expected,
                        "{basis}/{context_enabled}/{control}: {result}"
                    );
                    assert_eq!(result["complete_answer"], false, "{result}");
                    if expected {
                        let w = &result["requests"][0]["partial"][0];
                        assert!(rows(w, "answer_terms").is_empty());
                        assert_eq!(w["admission_context"]["actual_use_inferred"], false);
                        assert!(rows(&result["requests"][0], "support").is_empty());
                        assert!(rows(&result["requests"][0], "refutation").is_empty());
                        if closed {
                            assert_eq!(w["admission_constraint"], source["claims"][0]["body"]);
                        } else {
                            assert_eq!(w["answer_propositions"], json!([p]));
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn universal_data_scope_preserves_known_useful_directions() {
        let bank: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-universal-scope-controls-v1.json"
        ))
        .unwrap();
        for case in rows(&bank, "cases") {
            let input = &case["input"];
            let v = &input["vocabulary"];
            let a = super::super::validate(&input["source"]["raw"], s(&input["source"], "text"), v);
            let b =
                super::super::validate(&input["question"]["raw"], s(&input["question"], "text"), v);
            assert_eq!(a.report["status"], "complete", "{} {}", id(case), a.report);
            assert_eq!(b.report["status"], "complete", "{} {}", id(case), b.report);
            let result = classify_pair(&a, &b, v, &input["pair_scope"]);
            assert_eq!(
                !rows(&result, "selected_claim_ids").is_empty(),
                case["expected_selected"],
                "{}: {result}",
                id(case)
            );
            assert_eq!(
                result["complete_answer"],
                case["expected_complete"],
                "{}: {result}",
                id(case)
            );
        }
    }
    #[test]
    fn universal_common_scope_blocks_point_values_in_all_data_routes() {
        for route in ["proposition", "event", "restriction"] {
            let st = match route {
                "event" => "Se midió la presión de Helia de Ronda: 7 bar.",
                "restriction" => "Helia de Ronda solo puede tener una presión de 7 bar.",
                _ => "Helia de Ronda tiene una presión de 7 bar.",
            };
            let qt = "¿Qué presión común corresponde a todos los cilindros de Ronda?";
            let mut source = doc(st, "source", &["Helia", "Ronda"]);
            let amount = json!({"kind":"quantity","decimal":"7","unit":"bar","bound":"exact","evidence":[quote(st,"7 bar")]});
            let a = atom(
                st,
                "quantity",
                vec![
                    ("subject", reference("Helia")),
                    (
                        "attribute",
                        json!({"kind":"symbol","id":"pressure","evidence":[quote(st,"presión")]}),
                    ),
                    ("amount", amount.clone()),
                ],
            );
            source["claims"] = if route == "event" {
                json!([{"id":"point","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"event","event_ref":null,"event_kind":"measurement","subject":"Helia","aspect":"pressure","status":"occurred","method":{"state":"unspecified"},"times":[],"result":amount,"basis":"description"}}])
            } else if route == "restriction" {
                json!([{"id":"point","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"value_restriction","target":a,"answer_role":"amount","closure":"closed","members":[amount],"judgment":"fact","status":"state","basis":"description","normative_scope":null,"existence":"not_asserted"}}])
            } else {
                json!([claim(st, "point", prop(a))])
            };
            let mut question = doc(qt, "question", &["Ronda"]);
            question["references"].as_array_mut().unwrap().push(json!({"id":"cylinders","scope_id":"root","mention":quote(qt,"cilindros de Ronda"),"name":null,"declared_class":quote(qt,"cilindros"),"semantic_type":null,"denotation":"class","links":[{"kind":"associated_with","target":"Ronda","evidence":[quote(qt,"de Ronda")]}],"binding_scope_id":null,"anaphora":null}));
            question["scopes"].as_array_mut().unwrap().push(json!({"id":"all","parent_id":"root","kind":"binder","branch_id":null,"binding":{"variable":"member","quantifier":"forall","domain_ref":"cylinders","cardinality":null,"witness_relation":"unspecified","relation_closures":[],"evidence":[quote(qt,"todos los cilindros de Ronda")]}}));
            question["references"].as_array_mut().unwrap().push(json!({"id":"member","scope_id":"all","mention":quote(qt,"cilindros"),"name":null,"declared_class":quote(qt,"cilindros"),"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":"all","anaphora":null}));
            question["qualifiers"] = json!([{"id":"common","scope_id":"root","kind":"configuration","body":{"key":"quantity_extent","value":{"kind":"text","value":"común","evidence":[quote(qt,"común")]}},"resolution":"known","evidence":[quote(qt,"común")]}]);
            let target = prop(atom(
                qt,
                "quantity",
                vec![
                    ("subject", reference("member")),
                    (
                        "attribute",
                        json!({"kind":"symbol","id":"pressure","evidence":[quote(qt,"presión")]}),
                    ),
                ],
            ));
            let mut ask = request(
                qt,
                if route == "event" {
                    "event_slot"
                } else if route == "restriction" {
                    "constraint"
                } else {
                    "answer"
                },
                if route == "event" { "result" } else { "amount" },
                &target,
            );
            ask["scope_id"] = json!("all");
            ask["qualifier_ids"] = json!(["common"]);
            if route == "event" {
                ask["target"] = json!({"event_ref":null,"event_kind":"measurement","subject":"member","aspect":"pressure","status":"occurred","time_selector":null,"method_selector":{"kind":"any"}});
            }
            question["requests"] = json!([ask]);
            dependencies(&mut source);
            dependencies(&mut question);
            let a = super::super::validate(&source, st, &vocabulary());
            let b = super::super::validate(&question, qt, &vocabulary());
            assert_eq!(a.report["status"], "complete", "{route}: {}", a.report);
            assert_eq!(b.report["status"], "complete", "{route}: {}", b.report);
            let result = classify_pair(
                &a,
                &b,
                &vocabulary(),
                &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
            );
            assert!(
                rows(&result, "selected_claim_ids").is_empty(),
                "{route}: {result}"
            );
            assert_eq!(result["complete_answer"], false);
            assert!(
                rows(&result, "unresolved_candidates")
                    .iter()
                    .any(|c| rows(c, "reason_codes").contains(&json!("common_quantity_scope"))),
                "{route}: {result}"
            );

            assert!(
                !rows(&result, "unresolved_candidates").is_empty(),
                "{result}"
            );
        }
    }
    #[test]
    fn direct_truth_and_no_evidence_are_total() {
        let st = "Mara posee el cilindro Helia.";
        let qt = "¿Mara posee Helia?";
        let (sd, qd) = ownership(st, qt);
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "resolved");
        assert_eq!(result["complete_answer"], true);
        let mut empty = sd;
        empty["claims"] = json!([]);
        let result = pair(empty, st, qd, qt);
        assert_eq!(result["status"], "no_evidence");
        assert_eq!(result["complete_answer"], false);
        assert_eq!(rows(&result, "requests").len(), 1);
    }
    #[test]
    fn opposite_polarities_keep_both_traces() {
        let st = "Mara posee Helia; otra cláusula dice que Mara no posee Helia.";
        let qt = "¿Mara posee Helia?";
        let (mut sd, qd) = ownership(st, qt);
        let mut negative = sd["claims"][0].clone();
        negative["id"] = json!("negative");
        negative["body"]["value"]["atom"]["polarity"] = json!("negative");
        sd["claims"].as_array_mut().unwrap().push(negative);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "conflict");
        assert_eq!(rows(&result["requests"][0], "support").len(), 1);
        assert_eq!(rows(&result["requests"][0], "refutation").len(), 1);
    }
    #[test]
    fn anonymous_same_owner_is_not_named_identity() {
        let st = "Mara posee Helia. Una pieza de Mara permanece anónima.";
        let qt = "¿Mara posee Helia?";
        let (mut sd, qd) = ownership(st, qt);
        sd["references"][1]["name"] = Value::Null;
        sd["references"][1]["links"] =
            json!([{"kind":"owned_by","target":"Mara","evidence":[quote(st,st)]}]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert_eq!(result["complete_answer"], false);
    }
    #[test]
    fn unsupported_coverage_stays_in_denominator() {
        let st = "Mara posee Helia y se omite una condición desconocida.";
        let qt = "¿Mara posee Helia?";
        let (mut sd, qd) = ownership(st, qt);
        sd["coverage"][0]["state"] = json!("unsupported");
        sd["coverage"][0]["issue_id"] = json!("unparsed-condition");
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert_eq!(result["complete_answer"], false);
        assert!(!rows(&result, "selected_claim_ids").is_empty());
    }
    #[test]
    fn restricted_members_do_not_assert_ownership_and_unknown_name_is_not_exclusion() {
        let st = "Mara solo puede poseer Helia; Bruma es otro nombre sin enlace.";
        let qt = "¿Mara posee Bruma?";
        let mut sd = doc(st, "source", &["Mara", "Helia", "Bruma"]);
        let mut qd = doc(qt, "question", &["Mara", "Bruma"]);
        sd["claims"] = json!([{"id":"limit","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"value_restriction","target":atom(st,"owns",vec![("owner",reference("Mara")),("object",reference("Helia"))]),"answer_role":"object","closure":"closed","members":[reference("Helia")],"judgment":"fact","status":"state","basis":"description","normative_scope":null,"existence":"not_asserted"}}]);
        let p = prop(atom(
            qt,
            "owns",
            vec![("owner", reference("Mara")), ("object", reference("Bruma"))],
        ));
        qd["requests"] = json!([request(qt, "truth", "status", &p)]);
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "unresolved");
        assert!(rows(&result["requests"][0], "refutation").is_empty());
        sd["identity"] =
            json!([{"kind":"different","left":"Helia","right":"Bruma","evidence":[quote(st,st)]}]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "resolved");
        assert_eq!(
            s(&result["requests"][0]["refutation"][0], "kind"),
            "restriction_exclusion"
        );
    }
    #[test]
    fn exact_quantity_preserves_unit_and_basis_filters() {
        let st = "Nacar tiene una masa documental de 7 kg.";
        let qt = "¿Qué masa observada tiene Nacar en g?";
        let mut sd = doc(st, "source", &["Nacar"]);
        let mut qd = doc(qt, "question", &["Nacar"]);
        let amount = json!({"kind":"quantity","decimal":"7","unit":"kg","bound":"exact","evidence":[quote(st,"7 kg")]});
        let sp = prop(atom(
            st,
            "quantity",
            vec![
                ("subject", reference("Nacar")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"mass","evidence":[quote(st,"masa")]}),
                ),
                ("amount", amount),
            ],
        ));
        let qp = prop(atom(
            qt,
            "quantity",
            vec![
                ("subject", reference("Nacar")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"mass","evidence":[quote(qt,"masa")]}),
                ),
            ],
        ));
        sd["claims"] = json!([claim(st, "weight", sp)]);
        let mut rq = request(qt, "answer", "amount", &qp);
        rq["unit"] = json!({"symbol":"g","evidence":[quote(qt,"g")]});
        rq["basis_requirement"] = json!({"kind":"exact","value":"observation"});
        qd["requests"] = json!([rq]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "partial");
        assert_eq!(result["complete_answer"], false);
        assert!(rows(&result["requests"][0], "missing_constraints").contains(&json!("unit")));
        assert!(rows(&result["requests"][0], "missing_constraints").contains(&json!("basis")));
        assert_eq!(
            result["requests"][0]["partial"][0]["answer_terms"][0]["unit"],
            "kg"
        );
    }
    #[test]
    fn events_preserve_basis_and_do_not_choose_repeated_measurement() {
        let st = "Helia tiene dos mediciones de presión Eira y Orbe: 8 bar y 9 bar, descritas sin observación.";
        let qt = "¿Qué resultado observado de presión tiene Helia?";
        let mut sd = doc(st, "source", &["Helia", "Eira", "Orbe"]);
        let mut qd = doc(qt, "question", &["Helia"]);
        sd["claims"]=json!([("Eira","8"),("Orbe","9")].map(|(event,amount)|json!({"id":format!("measurement-{event}"),"scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"event","event_ref":event,"event_kind":"measurement","subject":"Helia","aspect":"pressure","status":"occurred","method":{"state":"unspecified"},"times":[],"result":{"kind":"quantity","decimal":amount,"unit":"bar","bound":"exact","evidence":[quote(st,st)]},"basis":"description"}})));
        qd["requests"] = json!([{"id":"ask","scope_id":"root","mode":"event_slot","target":{"event_ref":null,"event_kind":"measurement","subject":"Helia","aspect":"pressure","status":"occurred","time_selector":null,"method_selector":{"kind":"any"}},"slot":"result","class_filter":null,"unit":null,"basis_requirement":{"kind":"exact","value":"observation"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[quote(qt,qt)]}]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert_eq!(rows(&result["requests"][0], "partial").len(), 2);
        assert_eq!(result["complete_answer"], false);
    }
    #[test]
    fn documentary_anchor_context_preserves_record_without_equating_edges_or_delivery() {
        let controls: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-documentary-anchor-controls-v1.json"
        ))
        .unwrap();
        for case in rows(&controls, "cases") {
            let mut v = controls["profiled_vocabulary"].clone();
            if case["without_profile"] == true {
                v.as_object_mut()
                    .unwrap()
                    .remove("documentary_anchor_context");
            }
            let mut input = case["input"].clone();
            for side in ["source", "question"] {
                input[side]["raw"]["vocabulary_sha256"] =
                    json!(digest(&serde_json::to_vec(&v).unwrap()));
            }
            let sd =
                super::super::validate(&input["source"]["raw"], s(&input["source"], "text"), &v);
            let qd = super::super::validate(
                &input["question"]["raw"],
                s(&input["question"], "text"),
                &v,
            );
            assert_eq!(
                sd.report["status"],
                case["expected_source_status"]
                    .as_str()
                    .unwrap_or("complete"),
                "{}: {}",
                id(case),
                sd.report
            );
            assert_eq!(
                qd.report["status"],
                "complete",
                "{}: {}",
                id(case),
                qd.report
            );
            let result = classify_pair(&sd, &qd, &v, &input["pair_scope"]);
            assert_eq!(
                json!(!rows(&result, "selected_claim_ids").is_empty()),
                case["expected_selected"],
                "{}: {result}",
                id(case)
            );
            assert_eq!(result["complete_answer"], false);
            if case["expected_selected"] == true {
                for request in rows(&result, "requests") {
                    assert!(rows(request, "support").is_empty());
                    assert!(rows(request, "refutation").is_empty());
                }
                let w = &result["requests"][0]["partial"][0];
                assert_eq!(w["kind"], "partial_documentary_anchor_context");
                assert!(rows(w, "answer_terms").is_empty());
                assert_eq!(
                    w["answer_propositions"],
                    json!([input["source"]["raw"]["claims"][0]["body"]["value"]])
                );
                assert_eq!(w["source_qualifiers"], input["source"]["raw"]["qualifiers"]);
                for field in [
                    "link_kinds_equated",
                    "identity_proven",
                    "requested_event_occurrence_inferred",
                    "requested_value_inferred",
                    "unknown_value_inferred",
                ] {
                    assert_eq!(w["documentary_anchor_context"][field], false);
                }
                assert_eq!(
                    w["documentary_anchor_context"]["source_link_kind"],
                    "owned_by"
                );
                assert_eq!(
                    w["documentary_anchor_context"]["question_link_kind"],
                    "part_of"
                );
                assert!(rows(w, "missing_constraints").contains(&json!("identity")));
                assert!(rows(w, "missing_constraints").contains(&json!("event_occurrence")));
                assert!(rows(&result["requests"][1], "partial").is_empty());
            }
        }
    }
    #[test]
    fn documentary_anchor_profile_requires_explicit_link_status_and_configuration() {
        let controls: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-documentary-anchor-controls-v1.json"
        ))
        .unwrap();
        for mutation in [
            "version",
            "link",
            "attribute",
            "status",
            "link_kind",
            "empty",
            "duplicate",
            "extra",
        ] {
            let mut v = controls["profiled_vocabulary"].clone();
            let profile = &mut v["documentary_anchor_context"];
            match mutation {
                "version" => profile["version"] = json!("unknown"),
                "link" => profile["rows"][0]["link_id"] = json!("missing"),
                "attribute" => profile["rows"][0]["attribute_id"] = json!("missing"),
                "status" => profile["rows"][0]["target_status"] = json!("state"),
                "link_kind" => profile["rows"][0]["source_link_kind"] = json!("associated_with"),
                "empty" => profile["rows"][0]["source_configuration"] = json!([]),
                "duplicate" => {
                    let r = profile["rows"][0].clone();
                    profile["rows"].as_array_mut().unwrap().push(r);
                }
                _ => profile["rows"][0]["equate_link_kinds"] = json!(true),
            }
            let side = &controls["cases"][0]["input"]["source"];
            let mut raw = side["raw"].clone();
            raw["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
            let report = super::super::validate(&raw, s(side, "text"), &v).report;
            assert_eq!(report["status"], "invalid", "{mutation}: {report}");
            assert!(
                rows(&report, "issues")
                    .iter()
                    .any(|i| s(i, "path").starts_with("/vocabulary/documentary_anchor_context")),
                "{mutation}: {report}"
            );
        }
    }
    #[test]
    fn measurement_gap_keeps_event_conditions_without_inventing_value_or_ignorance() {
        let controls: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-measurement-gap-controls-v1.json"
        ))
        .unwrap();
        for case in rows(&controls, "cases") {
            let mut v = controls["profiled_vocabulary"].clone();
            if case["without_profile"] == true {
                v.as_object_mut().unwrap().remove("measurement_context");
            }
            let mut input = case["input"].clone();
            for side in ["source", "question"] {
                input[side]["raw"]["vocabulary_sha256"] =
                    json!(digest(&serde_json::to_vec(&v).unwrap()));
            }
            let sd =
                super::super::validate(&input["source"]["raw"], s(&input["source"], "text"), &v);
            let qd = super::super::validate(
                &input["question"]["raw"],
                s(&input["question"], "text"),
                &v,
            );
            assert_eq!(
                sd.report["status"],
                case["expected_source_status"]
                    .as_str()
                    .unwrap_or("complete"),
                "{}: {}",
                id(case),
                sd.report
            );
            assert_eq!(
                qd.report["status"],
                "complete",
                "{}: {}",
                id(case),
                qd.report
            );
            let result = classify_pair(&sd, &qd, &v, &input["pair_scope"]);
            assert_eq!(
                json!(!rows(&result, "selected_claim_ids").is_empty()),
                case["expected_selected"],
                "{}: {result}",
                id(case)
            );
            assert_eq!(result["complete_answer"], false, "{}: {result}", id(case));
            if case["expected_selected"] == true {
                for request in rows(&result, "requests") {
                    assert!(rows(request, "support").is_empty());
                    assert!(rows(request, "refutation").is_empty());
                    for w in rows(request, "partial") {
                        assert_eq!(w["kind"], "partial_measurement_context");
                        assert!(rows(w, "answer_terms").is_empty());
                        assert!(rows(w, "answer_propositions").is_empty());
                        let c = rows(&input["source"]["raw"], "claims")
                            .iter()
                            .find(|c| c["id"] == w["claim_ids"][0])
                            .unwrap();
                        assert_eq!(w["source_event"], c["body"]);
                        assert_eq!(w["measurement_context"]["requested_value_inferred"], false);
                        assert_eq!(w["measurement_context"]["unknown_value_inferred"], false);
                    }
                }
            }
        }
    }
    #[test]
    fn measurement_context_profile_rejects_ambiguous_or_unsupported_configuration() {
        let controls: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-measurement-gap-controls-v1.json"
        ))
        .unwrap();
        for mutation in [
            "version",
            "duplicate",
            "attribute",
            "target_empty",
            "binding_duplicate",
            "overlap",
            "extra",
            "bad_configuration",
        ] {
            let mut v = controls["profiled_vocabulary"].clone();
            let profile = &mut v["measurement_context"];
            match mutation {
                "version" => profile["version"] = json!("unknown"),
                "duplicate" => {
                    let r = profile["rows"][0].clone();
                    profile["rows"].as_array_mut().unwrap().push(r);
                }
                "attribute" => profile["rows"][0]["target_attribute"] = json!("missing"),
                "target_empty" => profile["rows"][0]["target_configuration"] = json!([]),
                "binding_duplicate" => {
                    profile["rows"][1]["context_bindings"] =
                        json!([["context", "route"], ["context", "other"]])
                }
                "overlap" => {
                    profile["rows"][0]["context_bindings"] =
                        json!([["terminal_condition", "other"]])
                }
                "extra" => profile["rows"][0]["unknown_value_inferred"] = json!(true),
                _ => profile["rows"][0]["source_configuration"] = json!(["bad"]),
            }
            let side = &controls["cases"][0]["input"]["source"];
            let mut raw = side["raw"].clone();
            raw["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
            let report = super::super::validate(&raw, s(side, "text"), &v).report;
            assert_eq!(report["status"], "invalid", "{mutation}: {report}");
            assert!(
                rows(&report, "issues")
                    .iter()
                    .any(|i| s(i, "path").starts_with("/vocabulary/measurement_context")),
                "{mutation}: {report}"
            );
        }
    }
    #[test]
    fn relevance_profiles_reject_same_destination_without_compatible_context() {
        let controls: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-relevance-domain-controls-v1.json"
        ))
        .unwrap();
        let v = &controls["profiled_vocabulary"];
        for case in rows(&controls, "cases") {
            let mut input = case["input"].clone();
            for side in ["source", "question"] {
                input[side]["raw"]["vocabulary_sha256"] =
                    json!(digest(&serde_json::to_vec(v).unwrap()));
            }
            let sd =
                super::super::validate(&input["source"]["raw"], s(&input["source"], "text"), v);
            let qd =
                super::super::validate(&input["question"]["raw"], s(&input["question"], "text"), v);
            assert_eq!(
                sd.report["status"],
                case["expected_source_status"]
                    .as_str()
                    .unwrap_or("complete"),
                "{}: {}",
                id(case),
                sd.report
            );
            assert_eq!(
                qd.report["status"],
                "complete",
                "{}: {}",
                id(case),
                qd.report
            );
            let result = classify_pair(&sd, &qd, v, &input["pair_scope"]);
            assert_eq!(
                json!(!rows(&result, "selected_claim_ids").is_empty()),
                case["expected_selected"],
                "{}: {result}",
                id(case)
            );
            assert_eq!(result["complete_answer"], false);
            if id(case) == "auxiliary_water" {
                let w = &result["requests"][0]["partial"][0];
                assert_eq!(w["relevance_profile"]["support_claim_ids"], json!(["c3"]));
                assert_eq!(w["relevance_profile"]["requested_origin_inferred"], false);
                assert!(rows(w, "answer_terms").is_empty());
                assert_eq!(
                    w["answer_propositions"],
                    json!([input["source"]["raw"]["claims"][0]["body"]["value"]])
                );
            }
        }
    }
    #[test]
    fn relevance_profiles_validate_roles_shape_and_source_target_proof_bindings() {
        let controls: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-relevance-domain-controls-v1.json"
        ))
        .unwrap();
        for mutation in [
            "version",
            "link",
            "role",
            "classes",
            "anchor",
            "duplicate",
            "extra",
            "evidence_role",
        ] {
            let mut v = controls["profiled_vocabulary"].clone();
            let profile = &mut v["relevance_profiles"];
            match mutation {
                "version" => profile["version"] = json!("unknown"),
                "link" => profile["rows"][0]["link_id"] = json!("missing"),
                "role" => profile["rows"][0]["alternatives"][0]["source_role"] = json!("missing"),
                "classes" => profile["rows"][0]["alternatives"][0]["source_classes"] = json!([]),
                "anchor" => {
                    profile["rows"][0]["alternatives"][1]["bindings"] =
                        json!([["target.substance", "evidence.substance"]])
                }
                "duplicate" => {
                    let row = profile["rows"][0].clone();
                    profile["rows"].as_array_mut().unwrap().push(row);
                }
                "extra" => profile["rows"][0]["alternatives"][0]["allow_transitive"] = json!(true),
                _ => {
                    profile["rows"][0]["alternatives"][1]["bindings"][0][1] =
                        json!("evidence.missing")
                }
            }
            let side = &controls["cases"][0]["input"]["source"];
            let mut raw = side["raw"].clone();
            raw["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
            let report = super::super::validate(&raw, s(side, "text"), &v).report;
            assert_eq!(report["status"], "invalid", "{mutation}: {report}");
            assert!(
                rows(&report, "issues")
                    .iter()
                    .any(|i| s(i, "path").starts_with("/vocabulary/relevance_profiles")),
                "{mutation}: {report}"
            );
        }
    }
    #[test]
    fn declared_supply_context_preserves_closed_constraints_without_member_claims() {
        let controls: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-supply-context-controls-v1.json"
        ))
        .unwrap();
        for case in rows(&controls, "cases") {
            let input = &case["input"];
            let source = &input["source"];
            let question = &input["question"];
            let v = &input["vocabulary"];
            let sd = super::super::validate(&source["raw"], s(source, "text"), v);
            let qd = super::super::validate(&question["raw"], s(question, "text"), v);
            assert_eq!(
                sd.report["status"],
                "complete",
                "{}: {}",
                id(case),
                sd.report
            );
            assert_eq!(
                qd.report["status"],
                "complete",
                "{}: {}",
                id(case),
                qd.report
            );
            let result = classify_pair(&sd, &qd, v, &input["pair_scope"]);
            assert_eq!(
                json!(!rows(&result, "selected_claim_ids").is_empty()),
                case["expected_selected"],
                "{}: {result}",
                id(case)
            );
            assert_eq!(result["complete_answer"], false, "{}: {result}", id(case));
            if matches!(id(case), "closed" | "multiple_members") {
                let request = &result["requests"][0];
                assert!(rows(request, "support").is_empty());
                assert!(rows(request, "refutation").is_empty());
                let witness = &request["partial"][0];
                assert!(rows(witness, "answer_terms").is_empty());
                assert!(rows(witness, "answer_propositions").is_empty());
                assert_eq!(
                    witness["source_constraint"],
                    source["raw"]["claims"][0]["body"]
                );
                assert!(
                    rows(witness, "missing_constraints").contains(&json!("source_constraint_only"))
                );
            }
        }
    }
    #[test]
    fn qualified_relevance_keeps_identity_pending_and_never_supplies_origin() {
        let controls: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-qualified-relevance-controls-v1.json"
        ))
        .unwrap();
        for case in rows(&controls, "cases") {
            let input = &case["input"];
            let source = &input["source"];
            let question = &input["question"];
            let v = &input["vocabulary"];
            let sd = super::super::validate(&source["raw"], s(source, "text"), v);
            let qd = super::super::validate(&question["raw"], s(question, "text"), v);
            assert_eq!(
                sd.report["status"],
                "complete",
                "{}: {}",
                id(case),
                sd.report
            );
            assert_eq!(
                qd.report["status"],
                "complete",
                "{}: {}",
                id(case),
                qd.report
            );
            let result = classify_pair(&sd, &qd, v, &input["pair_scope"]);
            let selected = !rows(&result, "selected_claim_ids").is_empty();
            assert_eq!(
                json!(selected),
                case["expected_selected"],
                "{}: {result}",
                id(case)
            );
            assert_eq!(result["complete_answer"], false, "{}: {result}", id(case));
            if id(case) == "qualified" {
                let request = &result["requests"][0];
                assert!(rows(request, "support").is_empty());
                assert!(rows(request, "refutation").is_empty());
                let witness = &request["partial"][0];
                assert!(rows(witness, "answer_terms").is_empty());
                assert_eq!(witness["context_identity_proven"], false);
                assert_eq!(
                    witness["answer_propositions"],
                    json!([source["raw"]["claims"][0]["body"]["value"]])
                );
                assert!(rows(witness, "missing_constraints").contains(&json!("identity")));
                assert!(!rows(&result, "unresolved_candidates").is_empty());
            }
        }
    }
    #[test]
    fn relevance_is_one_step_partial_and_keeps_original_judgment() {
        let st = "Helia entra por Bruma.";
        let qt = "¿Cuál es el origen de Helia?";
        let mut sd = doc(st, "source", &["Helia", "Bruma"]);
        let mut qd = doc(qt, "question", &["Helia"]);
        let sp = prop(atom(
            st,
            "route_entry",
            vec![("subject", reference("Helia")), ("via", reference("Bruma"))],
        ));
        sd["claims"] = json!([claim(st, "route", sp)]);
        let qp = prop(atom(qt, "origin", vec![("subject", reference("Helia"))]));
        qd["requests"] = json!([request(qt, "answer", "origin", &qp)]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "partial");
        assert!(rows(&result["requests"][0], "support").is_empty());
        assert_eq!(
            result["requests"][0]["partial"][0]["kind"],
            "partial_relevance"
        );
        assert_eq!(
            result["requests"][0]["partial"][0]["answer_terms"][0]["id"],
            "Bruma"
        );
    }
    #[test]
    fn normative_conflict_is_scoped_and_permission_never_implies_occurrence() {
        let st = "El código Cobalto permite y prohíbe a Mara retirar Helia.";
        let qt = "¿Cobalto permite a Mara retirar Helia?";
        let mut sd = doc(st, "source", &["Cobalto", "Mara", "Helia"]);
        let mut qd = doc(qt, "question", &["Cobalto", "Mara", "Helia"]);
        let mut sp = prop(atom(
            st,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(st,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        sp["judgment"] = json!("allowed");
        sp["normative_scope"] = json!("Cobalto");
        sp["basis"] = json!("normative");
        let mut qp = prop(atom(
            qt,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(qt,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        qp["judgment"] = json!("allowed");
        let mut forbidden = sp.clone();
        forbidden["judgment"] = json!("forbidden");
        sd["claims"] = json!([
            claim(st, "permission", sp.clone()),
            claim(st, "ban", forbidden)
        ]);
        let mut rq = request(qt, "truth", "status", &qp);
        rq["normative_scope_filter"] = json!("Cobalto");
        qd["requests"] = json!([rq]);
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "conflict");
        assert_eq!(
            result["requests"][0]["support"][0]["normative_scope"],
            "Cobalto"
        );
        qd["requests"][0]["target"]["judgment"] = json!("fact");
        qd["requests"][0]["target"]["status"] = json!("occurred");
        qd["requests"][0]["normative_scope_filter"] = Value::Null;
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "no_evidence");
    }
    #[test]
    fn one_step_sufficient_rule_requires_every_condition() {
        let st = "Cobalto permite a Mara retirar Helia si el sello de Helia está válido. El sello está válido.";
        let qt = "¿Cobalto permite a Mara retirar Helia?";
        let mut sd = doc(st, "source", &["Cobalto", "Mara", "Helia"]);
        let mut qd = doc(qt, "question", &["Cobalto", "Mara", "Helia"]);
        let condition = prop(atom(
            st,
            "property",
            vec![
                ("subject", reference("Helia")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"validity_state","evidence":[quote(st,"válido")]}),
                ),
                (
                    "value",
                    json!({"kind":"text","value":"válido","evidence":[quote(st,"válido")]}),
                ),
            ],
        ));
        let mut action = prop(atom(
            st,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(st,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        action["judgment"] = json!("allowed");
        action["normative_scope"] = json!("Cobalto");
        action["basis"] = json!("normative");
        sd["claims"] = json!([claim(st,"seal",condition.clone()),{"id":"rule","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"rule","rule_ref":"Cobalto","normative_scope":"Cobalto","direction":"sufficient","action":action,"conditions":[condition]}}]);
        let mut qp = prop(atom(
            qt,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(qt,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        qp["judgment"] = json!("allowed");
        let mut rq = request(qt, "truth", "status", &qp);
        rq["normative_scope_filter"] = json!("Cobalto");
        qd["requests"] = json!([rq]);
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "resolved");
        assert_eq!(result["requests"][0]["support"][0]["kind"], "rule_step");
        assert!(rows(&result["requests"][0]["support"][0], "claim_ids").contains(&json!("seal")));
        sd["claims"][1]["body"]["direction"] = json!("necessary");
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "no_evidence");
    }
    #[test]
    fn temporal_literal_difference_is_pending_not_disjointness() {
        let st = "Mara posee Helia durante el 21 de octubre.";
        let qt = "¿Mara posee Helia durante octubre?";
        let (mut sd, mut qd) = ownership(st, qt);
        sd["qualifiers"] = json!([{"id":"day","scope_id":"root","kind":"time","body":{"role":"holds_at","value":{"kind":"time","value":"21 de octubre","format":"literal","evidence":[quote(st,"21 de octubre")]}},"resolution":"known","evidence":[quote(st,"21 de octubre")]}]);
        sd["claims"][0]["qualifier_ids"] = json!(["day"]);
        qd["qualifiers"] = json!([{"id":"month","scope_id":"root","kind":"time","body":{"role":"holds_at","value":{"kind":"time","value":"octubre","format":"literal","evidence":[quote(qt,"octubre")]}},"resolution":"known","evidence":[quote(qt,"octubre")]}]);
        qd["requests"][0]["qualifier_ids"] = json!(["month"]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert!(rows(&result["requests"][0], "refutation").is_empty());
        assert!(!rows(&result["requests"][0], "partial").is_empty());
    }
    #[test]
    fn all_values_requires_closed_extent_and_namespace_must_be_explicit() {
        let st = "Mara posee Helia.";
        let qt = "¿Qué posee Mara? Helia se menciona como referencia.";
        let (mut sd, mut qd) = ownership(st, qt);
        let qp = prop(atom(qt, "owns", vec![("owner", reference("Mara"))]));
        let mut rq = request(qt, "answer", "object", &qp);
        rq["required_answer_extent"] = json!("all_values");
        qd["requests"] = json!([rq]);
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "partial");
        assert!(
            rows(&result["requests"][0], "missing_constraints").contains(&json!("answer_extent"))
        );
        sd["domain_namespace"] = Value::Null;
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert_eq!(result["complete_answer"], false);
    }
    #[test]
    fn closed_branches_publish_only_exact_invariant() {
        let st = "En ambas alternativas, Mara posee Helia.";
        let qt = "¿Mara posee Helia?";
        let (mut sd, qd) = ownership(st, qt);
        let original = sd["claims"][0].clone();
        let mut claims = vec![];
        for (bid, sid, cid) in [
            ("choice-one", "scope-one", "claim-one"),
            ("choice-two", "scope-two", "claim-two"),
        ] {
            sd["scopes"].as_array_mut().unwrap().push(
                json!({"id":sid,"parent_id":"root","kind":"branch","branch_id":bid,"binding":null}),
            );
            let mut c = original.clone();
            c["id"] = json!(cid);
            c["scope_id"] = json!(sid);
            claims.push(c);
        }
        claims.push(json!({"id":"choices","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"alternatives","closure":"closed","exclusive":"not_asserted","branches":[{"id":"choice-one","scope_id":"scope-one","claim_ids":["claim-one"]},{"id":"choice-two","scope_id":"scope-two","claim_ids":["claim-two"]}]}}));
        sd["claims"] = json!(claims);
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "resolved");
        assert_eq!(
            result["requests"][0]["support"][0]["kind"],
            "shared_invariant"
        );
        assert_eq!(
            rows(&result["requests"][0]["support"][0], "branch_ids").len(),
            2
        );
        sd["claims"][2]["body"]["closure"] = json!("open");
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "partial");
        assert_eq!(result["complete_answer"], false);
    }
    fn quantified(text: &str, kind: &str) -> Value {
        let mut d = doc(text, kind, &["Ronda"]);
        d["references"][0]["denotation"] = json!("group");
        let member = format!("{kind}-member");
        let object = format!("{kind}-object");
        for (rid, sid) in [(&member, "universal"), (&object, "existential")] {
            d["references"].as_array_mut().unwrap().push(json!({"id":rid,"scope_id":sid,"mention":quote(text,text),"name":null,"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":sid,"anaphora":null}));
        }
        d["scopes"].as_array_mut().unwrap().push(json!({"id":"universal","parent_id":"root","kind":"binder","branch_id":null,"binding":{"variable":member,"quantifier":"forall","domain_ref":"Ronda","cardinality":{"relation":"exact","value":2,"domain":"Ronda","evidence":[quote(text,text)]},"witness_relation":"unspecified","relation_closures":[],"evidence":[quote(text,text)]}}));
        d["scopes"].as_array_mut().unwrap().push(json!({"id":"existential","parent_id":"universal","kind":"binder","branch_id":null,"binding":{"variable":object,"quantifier":"exists","domain_ref":null,"cardinality":null,"witness_relation":"unspecified","relation_closures":[],"evidence":[quote(text,text)]}}));
        let p = prop(atom(
            text,
            "owns",
            vec![
                ("owner", reference(&member)),
                ("object", reference(&object)),
            ],
        ));
        if kind == "source" {
            let mut c = claim(text, "distributed", p);
            c["scope_id"] = json!("existential");
            d["claims"] = json!([c]);
        } else {
            let mut r = request(text, "truth", "status", &p);
            r["scope_id"] = json!("existential");
            d["requests"] = json!([r]);
        }
        d
    }
    #[test]
    fn alpha_pattern_preserves_quantifier_order_and_witness_relation() {
        let st = "Cada una de las dos guardianas de Ronda posee una llave.";
        let qt = "¿Cada una de las dos guardianas de Ronda posee una llave?";
        let sd = quantified(st, "source");
        let mut qd = quantified(qt, "question");
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "resolved");
        qd["scopes"][2]["binding"]["witness_relation"] = json!("distinct_across_domain");
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert_eq!(result["complete_answer"], false);
    }
    #[test]
    fn exact_exemption_waives_only_targeted_rule_and_never_grants_permission() {
        let st = "Cobalto exige sello válido a Mara para retirar Helia. Cobalto dispensa a Mara de esa condición.";
        let qt = "¿Cuál es el estado de la dispensa de Cobalto para Mara al retirar Helia?";
        let mut sd = doc(st, "source", &["Cobalto", "Mara", "Helia"]);
        let mut qd = doc(qt, "question", &["Cobalto", "Mara", "Helia"]);
        let condition = prop(atom(
            st,
            "property",
            vec![
                ("subject", reference("Helia")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"validity_state","evidence":[quote(st,"válido")]}),
                ),
                (
                    "value",
                    json!({"kind":"text","value":"válido","evidence":[quote(st,"válido")]}),
                ),
            ],
        ));
        let mut action = prop(atom(
            st,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(st,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        action["judgment"] = json!("allowed");
        action["normative_scope"] = json!("Cobalto");
        action["basis"] = json!("normative");
        sd["claims"] = json!([{"id":"rule","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"rule","rule_ref":"Cobalto","normative_scope":"Cobalto","direction":"necessary","action":action,"conditions":[condition.clone()]}},{"id":"waiver","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"exemption","rule_targets":["rule"],"normative_scope":"Cobalto","beneficiary":"Mara","waived_conditions":[condition["atom"]],"target_resolution":"exact"}}]);
        let mut qp = prop(atom(
            qt,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(qt,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        qp["judgment"] = json!("allowed");
        let mut rq = request(qt, "exemption_status", "status", &qp);
        rq["normative_scope_filter"] = json!("Cobalto");
        qd["requests"] = json!([rq]);
        dependencies(&mut sd);
        dependencies(&mut qd);
        sd["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":"waiver"},"requires":{"kind":"claim","id":"rule"},"role":"normative_target"}));
        let a = super::super::validate(&sd, st, &vocabulary());
        let b = super::super::validate(&qd, qt, &vocabulary());
        assert_ne!(a.report["status"], "invalid", "{}", a.report);
        assert_ne!(b.report["status"], "invalid", "{}", b.report);
        let result = classify_pair(
            &a,
            &b,
            &vocabulary(),
            &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
        );
        assert_eq!(result["status"], "resolved");
        assert_eq!(
            result["requests"][0]["support"][0]["answer_terms"][0]["value"],
            "waived"
        );
        let waiver_witness = &result["requests"][0]["support"][0];
        assert!(waiver_witness["conclusion_judgment"].is_null());
        assert_eq!(waiver_witness["requirement_states"][0]["rule_id"], "rule");
        assert_eq!(
            waiver_witness["requirement_states"][0]["normative_scope"],
            "Cobalto"
        );
        assert_eq!(
            waiver_witness["requirement_states"][0]["condition_index"],
            0
        );
        assert_eq!(
            waiver_witness["requirement_states"][0]["effective_state"],
            "waived"
        );
        assert!(
            rows(
                &waiver_witness["requirement_states"][0],
                "exemption_claim_ids"
            )
            .contains(&json!("waiver"))
        );
        assert_eq!(rows(waiver_witness, "answer_propositions").len(), 1);

        let mut branch_source = sd.clone();
        branch_source["claims"].as_array_mut().unwrap().pop();
        branch_source["scopes"].as_array_mut().unwrap().push(json!({"id":"rule-branch","parent_id":"root","kind":"branch","branch_id":"rule-assumption","binding":null}));
        branch_source["claims"][0]["scope_id"] = json!("rule-branch");
        let branch_result = pair(branch_source, st, qd.clone(), qt);
        assert_eq!(branch_result["complete_answer"], false);
        assert!(rows(&branch_result["requests"][0], "support").is_empty());
        assert!(
            rows(&branch_result["requests"][0]["partial"][0], "branch_ids")
                .contains(&json!("rule-assumption"))
        );
        qd["requests"][0]["mode"] = json!("rule_condition");
        let b = super::super::validate(&qd, qt, &vocabulary());
        let requirements = classify_pair(
            &a,
            &b,
            &vocabulary(),
            &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
        );
        assert_eq!(requirements["status"], "resolved");
        let w = &requirements["requests"][0]["support"][0];
        assert_eq!(rows(w, "answer_propositions").len(), 1);
        assert_eq!(w["requirement_states"][0]["effective_state"], "waived");
        assert!(w["conclusion_judgment"].is_null());
        assert!(rows(w, "answer_terms").is_empty());
        qd["requests"][0]["mode"] = json!("truth");
        qd["requests"][0]["required_answer_extent"] = json!("truth_status");
        let b = super::super::validate(&qd, qt, &vocabulary());
        let result = classify_pair(
            &a,
            &b,
            &vocabulary(),
            &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
        );
        assert_eq!(result["status"], "no_evidence");
    }

    #[test]
    fn shared_forbidden_branches_refute_requested_permission() {
        let st = "Según Cobalto, en ambas alternativas Mara tiene prohibido retirar Helia.";
        let qt = "¿Cobalto permite a Mara retirar Helia?";
        let mut sd = doc(st, "source", &["Cobalto", "Mara", "Helia"]);
        let mut qd = doc(qt, "question", &["Cobalto", "Mara", "Helia"]);
        let mut sp = prop(atom(
            st,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(st,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        sp["judgment"] = json!("forbidden");
        sp["normative_scope"] = json!("Cobalto");
        sp["basis"] = json!("normative");
        let mut claims = vec![];
        for (bid, sid, cid) in [
            ("ban-one", "ban-scope-one", "ban-claim-one"),
            ("ban-two", "ban-scope-two", "ban-claim-two"),
        ] {
            sd["scopes"].as_array_mut().unwrap().push(
                json!({"id":sid,"parent_id":"root","kind":"branch","branch_id":bid,"binding":null}),
            );
            let mut c = claim(st, cid, sp.clone());
            c["scope_id"] = json!(sid);
            claims.push(c);
        }
        claims.push(json!({"id":"bans","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"alternatives","closure":"closed","exclusive":"not_asserted","branches":[{"id":"ban-one","scope_id":"ban-scope-one","claim_ids":["ban-claim-one"]},{"id":"ban-two","scope_id":"ban-scope-two","claim_ids":["ban-claim-two"]}]}}));
        sd["claims"] = json!(claims);
        let mut qp = prop(atom(
            qt,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(qt,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        qp["judgment"] = json!("allowed");
        let mut rq = request(qt, "truth", "status", &qp);
        rq["normative_scope_filter"] = json!("Cobalto");
        qd["requests"] = json!([rq]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "resolved");
        assert!(rows(&result["requests"][0], "support").is_empty());
        assert_eq!(
            result["requests"][0]["refutation"][0]["kind"],
            "shared_invariant"
        );
        assert_eq!(
            result["requests"][0]["refutation"][0]["conclusion_judgment"],
            "forbidden"
        );
    }
    #[test]
    fn guard_fact_inside_branch_does_not_discharge_global_guard() {
        let st = "Si Helia es válida, Mara posee Helia. En una alternativa Helia es válida.";
        let qt = "¿Mara posee Helia?";
        let (mut sd, qd) = ownership(st, qt);
        let condition = prop(atom(
            st,
            "property",
            vec![
                ("subject", reference("Helia")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"validity_state","evidence":[quote(st,"válida")]}),
                ),
                (
                    "value",
                    json!({"kind":"text","value":"válida","evidence":[quote(st,"válida")]}),
                ),
            ],
        ));
        sd["qualifiers"] = json!([{"id":"guard","scope_id":"root","kind":"guard","body":condition.clone(),"resolution":"known","evidence":[quote(st,st)]}]);
        sd["claims"][0]["qualifier_ids"] = json!(["guard"]);
        sd["scopes"].as_array_mut().unwrap().push(json!({"id":"guard-branch","parent_id":"root","kind":"branch","branch_id":"conditional-validity","binding":null}));
        let mut c = claim(st, "branch-validity", condition);
        c["scope_id"] = json!("guard-branch");
        sd["claims"].as_array_mut().unwrap().push(c);
        dependencies(&mut sd);
        sd["dependencies"].as_array_mut().unwrap().retain(|d| {
            !(d["consumer"]["id"] == "branch-validity" && d["requires"]["kind"] == "qualifier")
        });
        let mut qd = qd;
        dependencies(&mut qd);
        let a = super::super::validate(&sd, st, &vocabulary());
        let b = super::super::validate(&qd, qt, &vocabulary());
        assert_ne!(a.report["status"], "invalid", "{}", a.report);
        assert_ne!(b.report["status"], "invalid", "{}", b.report);
        let result = classify_pair(
            &a,
            &b,
            &vocabulary(),
            &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
        );
        assert_eq!(result["status"], "unresolved");
        assert_eq!(result["complete_answer"], false);
        assert!(rows(&result["requests"][0], "support").is_empty());
    }

    #[test]
    fn pending_reference_cannot_be_used_as_guard_evidence() {
        let st = "Mara posee Helia si el sello es válido. Un registro dice que el sello es válido pero no resuelve de cuál se trata.";
        let qt = "¿Mara posee Helia?";
        let (mut sd, mut qd) = ownership(st, qt);
        let mut ghost = doc(st, "source", &["sello"])["references"][0].clone();
        ghost["name"] = Value::Null;
        ghost["anaphora"] =
            json!({"status":"unresolved","antecedent_ids":[],"evidence":[quote(st,"sello")]});
        sd["references"].as_array_mut().unwrap().push(ghost);
        let condition = prop(atom(
            st,
            "property",
            vec![
                ("subject", reference("sello")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"validity_state","evidence":[quote(st,"válido")]}),
                ),
                (
                    "value",
                    json!({"kind":"text","value":"válido","evidence":[quote(st,"válido")]}),
                ),
            ],
        ));
        sd["qualifiers"] = json!([{"id":"guard","scope_id":"root","kind":"guard","body":condition.clone(),"resolution":"known","evidence":[quote(st,st)]}]);
        sd["claims"][0]["qualifier_ids"] = json!(["guard"]);
        sd["claims"]
            .as_array_mut()
            .unwrap()
            .push(claim(st, "unresolved-seal", condition));
        dependencies(&mut sd);
        dependencies(&mut qd);
        sd["dependencies"].as_array_mut().unwrap().retain(|d| {
            !(d["consumer"]["id"] == "unresolved-seal" && d["requires"]["kind"] == "qualifier"
                || d["consumer"]["id"] == "ownership" && d["requires"]["id"] == "sello")
        });
        let a = super::super::validate(&sd, st, &vocabulary());
        let b = super::super::validate(&qd, qt, &vocabulary());
        assert_ne!(a.report["status"], "invalid", "{}", a.report);
        let result = classify_pair(
            &a,
            &b,
            &vocabulary(),
            &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
        );
        assert_eq!(result["status"], "unresolved");
        assert!(rows(&result["requests"][0], "support").is_empty());
    }
    #[test]
    fn explicit_same_events_combine_selector_and_slot_with_both_citations() {
        let st =
            "Helia tuvo la medición Eira ayer. Orbe es la misma medición y cita método óptico.";
        let qt = "¿Qué método tuvo la medición de Helia ayer?";
        let mut sd = doc(st, "source", &["Helia", "Eira", "Orbe"]);
        let mut qd = doc(qt, "question", &["Helia"]);
        sd["identity"] = json!([{"kind":"same","left":"Eira","right":"Orbe","evidence":[quote(st,"misma medición")]}]);
        sd["claims"] = json!([{"id":"dated-event","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,"Eira ayer")],"body":{"kind":"event","event_ref":"Eira","event_kind":"measurement","subject":"Helia","aspect":"pressure","status":"occurred","method":{"state":"unspecified"},"times":[{"role":"occurred_at","value":{"kind":"time","value":"ayer","format":"literal","evidence":[quote(st,"ayer")]}}],"result":null,"basis":"description"}},{"id":"method-event","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,"método óptico")],"body":{"kind":"event","event_ref":"Orbe","event_kind":"measurement","subject":"Helia","aspect":"pressure","status":"occurred","method":{"state":"specified","value":{"kind":"text","value":"óptico","evidence":[quote(st,"óptico")]}},"times":[],"result":null,"basis":"description"}}]);
        qd["requests"] = json!([{"id":"ask","scope_id":"root","mode":"event_slot","target":{"event_ref":null,"event_kind":"measurement","subject":"Helia","aspect":"pressure","status":"occurred","time_selector":{"role":"occurred_at","value":{"kind":"time","value":"ayer","format":"literal","evidence":[quote(qt,"ayer")]}},"method_selector":{"kind":"any"}},"slot":"method","class_filter":null,"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[quote(qt,qt)]}]);
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "resolved");
        let w = &result["requests"][0]["support"][0];
        assert_eq!(w["answer_terms"][0]["value"], "óptico");
        assert!(rows(w, "claim_ids").contains(&json!("dated-event")));
        assert!(rows(w, "claim_ids").contains(&json!("method-event")));
        sd["identity"] = json!([]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert_eq!(result["complete_answer"], false);
    }
    #[test]
    fn alternative_parent_guard_is_inherited_without_repeating_child_fields() {
        let st = "Solo si el sello de Helia es válido, en ambas alternativas Mara posee Helia.";
        let qt = "¿Mara posee Helia?";
        let (mut sd, qd) = ownership(st, qt);
        let original = sd["claims"][0].clone();
        let mut claims = vec![];
        let guard = prop(atom(
            st,
            "property",
            vec![
                ("subject", reference("Helia")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"validity_state","evidence":[quote(st,"válido")]}),
                ),
                (
                    "value",
                    json!({"kind":"text","value":"válido","evidence":[quote(st,"válido")]}),
                ),
            ],
        ));
        sd["qualifiers"] = json!([{"id":"parent-guard","scope_id":"root","kind":"guard","body":guard,"resolution":"known","evidence":[quote(st,st)]}]);
        for (bid, sid, cid) in [
            ("one", "scope-one", "claim-one"),
            ("two", "scope-two", "claim-two"),
        ] {
            sd["scopes"].as_array_mut().unwrap().push(
                json!({"id":sid,"parent_id":"root","kind":"branch","branch_id":bid,"binding":null}),
            );
            let mut c = original.clone();
            c["id"] = json!(cid);
            c["scope_id"] = json!(sid);
            claims.push(c);
        }
        claims.push(json!({"id":"guarded-alternatives","scope_id":"root","qualifier_ids":["parent-guard"],"evidence":[quote(st,st)],"body":{"kind":"alternatives","closure":"closed","exclusive":"not_asserted","branches":[{"id":"one","scope_id":"scope-one","claim_ids":["claim-one"]},{"id":"two","scope_id":"scope-two","claim_ids":["claim-two"]}]}}));
        sd["claims"] = json!(claims);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert!(rows(&result["requests"][0], "support").is_empty());
        assert!(
            rows(&result["requests"][0]["partial"][0], "qualifier_ids")
                .contains(&json!("parent-guard"))
        );
    }

    #[test]
    fn tiny_closed_refutation_honors_target_qualifier_ids() {
        let st = "Cada una de las dos guardianas de Ronda posee exclusivamente su propia llave y las llaves son distintas.";
        let qt = "¿Hay una llave compartida por las dos guardianas de Ronda en lluvia?";
        let mut sd = quantified(st, "source");
        let mut qd = quantified(qt, "question");
        sd["scopes"][2]["binding"]["witness_relation"] = json!("distinct_across_domain");
        let p = sd["claims"][0]["body"]["value"].clone();
        sd["scopes"][2]["binding"]["relation_closures"] = json!([{"id":"own-key-closure","scope_id":"existential","domain_binding":"source-member","object_binding":"source-object","target":p,"closed_role":"object","evidence":[quote(st,st)]}]);
        let outer = qd["scopes"][2]["binding"].clone();
        let inner = qd["scopes"][1]["binding"].clone();
        qd["scopes"][1] = json!({"id":"existential","parent_id":"root","kind":"binder","branch_id":null,"binding":outer});
        qd["scopes"][1]["binding"]["witness_relation"] = json!("same_across_domain");
        qd["scopes"][2] = json!({"id":"universal","parent_id":"existential","kind":"binder","branch_id":null,"binding":inner});
        qd["requests"][0]["scope_id"] = json!("universal");
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "resolved");
        assert!(!rows(&result["requests"][0], "refutation").is_empty());
        qd["qualifiers"] = json!([{"id":"rain","scope_id":"root","kind":"configuration","body":{"key":"weather","value":{"kind":"text","value":"lluvia","evidence":[quote(qt,"lluvia")]}},"resolution":"known","evidence":[quote(qt,"lluvia")]}]);
        qd["requests"][0]["target"]["qualifier_ids"] = json!(["rain"]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert!(rows(&result["requests"][0], "refutation").is_empty());
        assert_eq!(result["complete_answer"], false);
    }
    fn rename_ids(value: &mut Value, prefix: &str) {
        match value {
            Value::Object(object) => {
                let symbol = object.get("kind").and_then(Value::as_str) == Some("symbol");
                for (key, v) in object {
                    let scalar = (key == "id" && !symbol)
                        || matches!(
                            key.as_str(),
                            "scope_id"
                                | "parent_id"
                                | "branch_id"
                                | "binding_scope_id"
                                | "variable"
                                | "domain_ref"
                                | "domain"
                                | "domain_binding"
                                | "object_binding"
                                | "event_ref"
                                | "subject"
                                | "rule_ref"
                                | "normative_scope"
                                | "beneficiary"
                                | "normative_scope_filter"
                                | "left"
                                | "right"
                                | "target"
                        );
                    if scalar && v.is_string() {
                        *v = json!(format!("{prefix}{}", v.as_str().unwrap()));
                    } else if matches!(
                        key.as_str(),
                        "qualifier_ids"
                            | "premise_ids"
                            | "claim_ids"
                            | "rule_targets"
                            | "antecedent_ids"
                    ) {
                        if let Some(array) = v.as_array_mut() {
                            for id in array {
                                if let Some(idstr) = id.as_str() {
                                    *id = json!(format!("{prefix}{idstr}"));
                                }
                            }
                        }
                    } else {
                        rename_ids(v, prefix);
                    }
                }
            }
            Value::Array(array) => {
                for item in array {
                    rename_ids(item, prefix);
                }
            }
            _ => {}
        }
    }
    fn permute(value: &mut Value) {
        match value {
            Value::Object(object) => {
                for (key, v) in object {
                    if matches!(key.as_str(), "references" | "claims" | "arguments") {
                        if let Some(array) = v.as_array_mut() {
                            array.reverse();
                        }
                    }
                    permute(v);
                }
            }
            Value::Array(array) => {
                for item in array {
                    permute(item);
                }
            }
            _ => {}
        }
    }
    fn observable(result: &Value) -> Value {
        json!({"status":result["status"],"complete_answer":result["complete_answer"],"requires_review":result["requires_review"],"requires_qualification":result["requires_qualification"],"requests":rows(result,"requests").iter().map(|r|json!({"coverage":r["coverage"],"support":rows(r,"support").len(),"refutation":rows(r,"refutation").len(),"partial":rows(r,"partial").len(),"unresolved":rows(r,"unresolved").len(),"missing_constraints":r["missing_constraints"],"alpha_bindings":rows(r,"support").iter().flat_map(|w|rows(w,"bindings")).filter(|b|s(b,"relation")=="alpha_pattern").count()})).collect::<Vec<_>>()})
    }
    #[test]
    fn quantified_evidence_is_invariant_to_disjoint_ids_and_order_but_not_cardinality() {
        let st = "Cada una de las dos guardianas de Ronda posee una llave.";
        let qt = "¿Cada una de las dos guardianas de Ronda posee una llave?";
        let sd = quantified(st, "source");
        let qd = quantified(qt, "question");
        let expected = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(expected["status"], "resolved");
        let mut renamed_source = sd.clone();
        let mut renamed_question = qd.clone();
        rename_ids(&mut renamed_source, "s_");
        rename_ids(&mut renamed_question, "q_");
        permute(&mut renamed_source);
        permute(&mut renamed_question);
        assert_eq!(renamed_source["text_sha256"], sd["text_sha256"]);
        assert_eq!(renamed_question["text_sha256"], qd["text_sha256"]);
        let renamed = pair(renamed_source.clone(), st, renamed_question.clone(), qt);
        assert_eq!(observable(&renamed), observable(&expected));
        assert!(!rows(&renamed["requests"][0]["support"][0], "bindings").is_empty());
        let mut different_question = qd;
        different_question["scopes"][1]["binding"]["cardinality"]["value"] = json!(3);
        let changed = pair(sd, st, different_question.clone(), qt);
        assert_eq!(changed["status"], "unresolved");
        assert_eq!(changed["complete_answer"], false);
        rename_ids(&mut different_question, "q_");
        permute(&mut different_question);
        let changed_renamed = pair(renamed_source, st, different_question, qt);
        assert_eq!(observable(&changed_renamed), observable(&changed));
    }

    #[test]
    fn same_event_incompatible_exact_results_conflict_instead_of_completing() {
        let st =
            "Helia tiene dos registros Eira y Orbe de la misma medición de presión: 8 bar y 9 bar.";
        let qt = "¿Qué resultado de presión tuvo Helia?";
        let mut sd = doc(st, "source", &["Helia", "Eira", "Orbe"]);
        let mut qd = doc(qt, "question", &["Helia"]);
        sd["identity"] = json!([{"kind":"same","left":"Eira","right":"Orbe","evidence":[quote(st,"misma medición")]}]);
        sd["claims"]=json!([("Eira","8"),("Orbe","9")].map(|(event,amount)|json!({"id":format!("result-{event}"),"scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"event","event_ref":event,"event_kind":"measurement","subject":"Helia","aspect":"pressure","status":"occurred","method":{"state":"unspecified"},"times":[],"result":{"kind":"quantity","decimal":amount,"unit":"bar","bound":"exact","evidence":[quote(st,st)]},"basis":"description"}})));
        qd["requests"] = json!([{"id":"ask","scope_id":"root","mode":"event_slot","target":{"event_ref":null,"event_kind":"measurement","subject":"Helia","aspect":"pressure","status":"occurred","time_selector":null,"method_selector":{"kind":"any"}},"slot":"result","class_filter":null,"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[quote(qt,qt)]}]);
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "conflict");
        assert_eq!(result["complete_answer"], false);
        assert_eq!(rows(&result["requests"][0], "support").len(), 2);
        assert_eq!(rows(&result["requests"][0], "conflicts").len(), 1);
        sd["identity"] = json!([]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert!(rows(&result["requests"][0], "support").is_empty());
        assert_eq!(rows(&result["requests"][0], "partial").len(), 2);
    }
    #[test]
    fn premises_require_unconditional_source_and_keep_question_claim_context() {
        let st = "Mara posee Helia. Solo en una alternativa, el sello de Helia es válido.";
        let qt = "Dado que el sello de Helia es válido en lluvia, ¿Mara posee Helia?";
        let (mut sd, mut qd) = ownership(st, qt);
        let make_condition = |text: &str| {
            prop(atom(
                text,
                "property",
                vec![
                    ("subject", reference("Helia")),
                    (
                        "attribute",
                        json!({"kind":"symbol","id":"validity_state","evidence":[quote(text,"válido")]}),
                    ),
                    (
                        "value",
                        json!({"kind":"text","value":"válido","evidence":[quote(text,"válido")]}),
                    ),
                ],
            ))
        };
        let mut source_condition = claim(st, "seal-evidence", make_condition(st));
        source_condition["scope_id"] = json!("premise-branch");
        sd["scopes"].as_array_mut().unwrap().push(json!({"id":"premise-branch","parent_id":"root","kind":"branch","branch_id":"seal-only-conditional","binding":null}));
        sd["claims"].as_array_mut().unwrap().push(source_condition);
        qd["claims"] = json!([claim(qt, "question-premise", make_condition(qt))]);
        qd["requests"][0]["premise_ids"] = json!(["question-premise"]);
        let run = |mut source: Value, mut question: Value| {
            dependencies(&mut source);
            dependencies(&mut question);
            question["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"request","id":"ask"},"requires":{"kind":"claim","id":"question-premise"},"role":"premise"}));
            let a = super::super::validate(&source, st, &vocabulary());
            let b = super::super::validate(&question, qt, &vocabulary());
            assert_ne!(a.report["status"], "invalid", "{}", a.report);
            assert_ne!(b.report["status"], "invalid", "{}", b.report);
            classify_pair(
                &a,
                &b,
                &vocabulary(),
                &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
            )
        };
        let result = run(sd.clone(), qd.clone());
        assert_eq!(result["complete_answer"], false);
        assert!(rows(&result["requests"][0], "support").is_empty());
        assert!(rows(&result["requests"][0], "missing_constraints").contains(&json!("premise")));
        sd["claims"][1]["scope_id"] = json!("root");
        let result = run(sd.clone(), qd.clone());
        assert_eq!(result["status"], "resolved");
        qd["qualifiers"] = json!([{"id":"premise-weather","scope_id":"root","kind":"configuration","body":{"key":"weather","value":{"kind":"text","value":"lluvia","evidence":[quote(qt,"lluvia")]}},"resolution":"known","evidence":[quote(qt,"lluvia")]}]);
        qd["claims"][0]["qualifier_ids"] = json!(["premise-weather"]);
        let result = run(sd, qd);
        assert_eq!(result["complete_answer"], false);
        assert!(rows(&result["requests"][0], "support").is_empty());
        assert!(rows(&result["requests"][0], "missing_constraints").contains(&json!("premise")));
    }

    #[test]
    fn necessary_requirement_keeps_target_configuration_and_does_not_complete_other_context() {
        let st = "Cobalto permite a Mara retirar Helia solo si el sello es válido.";
        let qt = "¿Qué condición exige Cobalto a Mara para retirar Helia en lluvia?";
        let mut sd = doc(st, "source", &["Cobalto", "Mara", "Helia"]);
        let mut qd = doc(qt, "question", &["Cobalto", "Mara", "Helia"]);
        let condition = prop(atom(
            st,
            "property",
            vec![
                ("subject", reference("Helia")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"validity_state","evidence":[quote(st,"válido")]}),
                ),
                (
                    "value",
                    json!({"kind":"text","value":"válido","evidence":[quote(st,"válido")]}),
                ),
            ],
        ));
        let mut action = prop(atom(
            st,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(st,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        action["judgment"] = json!("allowed");
        action["normative_scope"] = json!("Cobalto");
        action["basis"] = json!("normative");
        sd["claims"] = json!([{"id":"rule","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"rule","rule_ref":"Cobalto","normative_scope":"Cobalto","direction":"necessary","action":action,"conditions":[condition]}}]);
        let mut qp = prop(atom(
            qt,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(qt,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        qp["judgment"] = json!("allowed");
        let mut rq = request(qt, "rule_condition", "status", &qp);
        rq["normative_scope_filter"] = json!("Cobalto");
        qd["requests"] = json!([rq]);
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["status"], "resolved");
        assert_eq!(
            rows(&result["requests"][0]["support"][0], "answer_propositions").len(),
            1
        );
        qd["qualifiers"] = json!([{"id":"weather","scope_id":"root","kind":"configuration","body":{"key":"weather","value":{"kind":"text","value":"lluvia","evidence":[quote(qt,"lluvia")]}},"resolution":"known","evidence":[quote(qt,"lluvia")]}]);
        qd["requests"][0]["target"]["qualifier_ids"] = json!(["weather"]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "unresolved");
        assert_eq!(result["complete_answer"], false);
        assert!(rows(&result["requests"][0], "support").is_empty());
        assert_eq!(
            rows(&result["requests"][0]["partial"][0], "answer_propositions").len(),
            1
        );
    }

    #[test]
    fn frozen_domain_controls_preserve_bounds_years_occurrences_and_applicability() {
        let bank: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-domain-controls-v1.json"
        ))
        .unwrap();
        for case in rows(&bank, "cases") {
            let input = &case["input"];
            let a = super::super::validate(
                &input["source"]["raw"],
                s(&input["source"], "text"),
                &input["vocabulary"],
            );
            let b = super::super::validate(
                &input["question"]["raw"],
                s(&input["question"], "text"),
                &input["vocabulary"],
            );
            assert_eq!(
                a.report["status"],
                case["expected"]["source_status"],
                "{} source {}",
                id(case),
                a.report
            );
            assert_eq!(
                b.report["status"],
                "complete",
                "{} question {}",
                id(case),
                b.report
            );
            let result = classify_pair(&a, &b, &input["vocabulary"], &input["pair_scope"]);
            assert_eq!(
                result["complete_answer"],
                case["expected"]["complete_answer"],
                "{} {}",
                id(case),
                result
            );
            assert_eq!(
                !rows(&result, "selected_claim_ids").is_empty(),
                case["expected"]["selected"].as_bool().unwrap(),
                "{} {}",
                id(case),
                result
            );
            if !case["expected"]["missing_constraint"].is_null() {
                assert!(
                    rows(&result["requests"][0], "missing_constraints")
                        .contains(&case["expected"]["missing_constraint"]),
                    "{} {}",
                    id(case),
                    result
                );
            }
        }
    }

    #[test]
    fn temporal_shared_invariant_is_one_occurrence_but_divergent_branches_are_not_complete() {
        let st = "Mara retiró Helia ayer; en otra lectura, hoy.";
        let qt = "¿Cuándo retiró Mara Helia?";
        let mut sd = doc(st, "source", &["Mara", "Helia"]);
        let mut qd = doc(qt, "question", &["Mara", "Helia"]);
        let mut sp = prop(atom(
            st,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(st,"retiró")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        sp["status"] = json!("occurred");
        sp["qualifier_ids"] = json!(["when"]);
        sd["qualifiers"] = json!([{"id":"when","scope_id":"root","kind":"time","body":{"role":"occurred_at","value":{"kind":"time","value":"ayer","format":"literal","evidence":[quote(st,"ayer")]}},"resolution":"known","evidence":[quote(st,"ayer")]}]);
        let mut a = claim(st, "a", sp.clone());
        a["scope_id"] = json!("left");
        let mut b = claim(st, "b", sp.clone());
        b["scope_id"] = json!("right");
        sd["scopes"].as_array_mut().unwrap().extend([
            json!({"id":"left","parent_id":"root","kind":"branch","branch_id":"L","binding":null}),
            json!({"id":"right","parent_id":"root","kind":"branch","branch_id":"R","binding":null}),
        ]);
        sd["claims"] = json!([a,b,{"id":"readings","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"alternatives","closure":"closed","exclusive":"asserted","branches":[{"id":"L","scope_id":"left","claim_ids":["a"]},{"id":"R","scope_id":"right","claim_ids":["b"]}]}}]);
        let mut qp = prop(atom(
            qt,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(qt,"retiró")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        qp["status"] = json!("occurred");
        qd["requests"] = json!([request(qt, "answer", "occurred_at", &qp)]);
        let result = pair(sd.clone(), st, qd.clone(), qt);
        assert_eq!(result["complete_answer"], true, "{result}");
        assert!(
            rows(&result["requests"][0], "support")
                .iter()
                .any(|w| s(w, "kind") == "shared_invariant")
        );
        sd["qualifiers"].as_array_mut().unwrap().push(json!({"id":"other-time","scope_id":"root","kind":"time","body":{"role":"occurred_at","value":{"kind":"time","value":"hoy","format":"literal","evidence":[quote(st,"hoy")]}},"resolution":"known","evidence":[quote(st,"hoy")]}));
        sd["claims"][1]["body"]["value"]["qualifier_ids"] = json!(["other-time"]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["complete_answer"], false, "{result}");
        assert!(rows(&result["requests"][0], "support").is_empty());
    }
    #[test]
    fn sufficient_rule_with_missing_guard_rejects_proven_different_context() {
        let st = "En el perfil Bruma, Cobalto permite a Mara retirar Helia si el sello de Helia está válido. Los perfiles Bruma y Lumen son distintos.";
        let qt = "¿Cobalto permite a Mara retirar Helia en Lumen?";
        let mut sd = doc(
            st,
            "source",
            &["Cobalto", "Mara", "Helia", "Bruma", "Lumen"],
        );
        let mut qd = doc(qt, "question", &["Cobalto", "Mara", "Helia", "Lumen"]);
        let condition = prop(atom(
            st,
            "property",
            vec![
                ("subject", reference("Helia")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"validity_state","evidence":[quote(st,"válido")]}),
                ),
                (
                    "value",
                    json!({"kind":"text","value":"válido","evidence":[quote(st,"válido")]}),
                ),
            ],
        ));
        let mut action = prop(atom(
            st,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(st,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        action["judgment"] = json!("allowed");
        action["normative_scope"] = json!("Cobalto");
        action["basis"] = json!("normative");
        sd["claims"] = json!([claim(st,"seal",condition.clone()),{"id":"rule","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"rule","rule_ref":"Cobalto","normative_scope":"Cobalto","direction":"sufficient","action":action,"conditions":[condition]}}]);
        let mut qp = prop(atom(
            qt,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(qt,"retirar")]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        qp["judgment"] = json!("allowed");
        let mut rq = request(qt, "truth", "status", &qp);
        rq["normative_scope_filter"] = json!("Cobalto");
        qd["requests"] = json!([rq]);
        sd["claims"].as_array_mut().unwrap().remove(0);
        sd["identity"] = json!([{"kind":"different","left":"Bruma","right":"Lumen","evidence":[quote(st,"Los perfiles Bruma y Lumen son distintos.")]}]);
        sd["qualifiers"] = json!([{"id":"profile","scope_id":"root","kind":"configuration","body":{"key":"profile","value":reference("Bruma")},"resolution":"known","evidence":[quote(st,"Bruma")]}]);
        sd["claims"][0]["qualifier_ids"] = json!(["profile"]);
        qd["qualifiers"] = json!([{"id":"profile","scope_id":"root","kind":"configuration","body":{"key":"profile","value":reference("Lumen")},"resolution":"known","evidence":[quote(qt,"Lumen")]}]);
        qd["requests"][0]["qualifier_ids"] = json!(["profile"]);
        let result = pair(sd, st, qd, qt);
        assert_eq!(result["status"], "no_evidence", "{result}");
        assert!(rows(&result, "selected_claim_ids").is_empty());
    }

    fn profiled_vocabulary() -> Value {
        let mut v = vocabulary();
        v["attribute_profiles"] = json!({"version":"documentary_attribute_profiles_v1","targets":["quantity.amount","property.value","event.result"],"rows":[{"attribute_id":"mass","target":"quantity.amount","term_kinds":["quantity"],"quantity_units":["kg","g"]},{"attribute_id":"mass","target":"property.value","term_kinds":["quantity"],"quantity_units":["kg","g"]},{"attribute_id":"pressure","target":"event.result","term_kinds":["quantity"],"quantity_units":["bar"]}]});
        v
    }
    fn profiled_documents(
        mut sd: Value,
        st: &str,
        mut qd: Value,
        qt: &str,
    ) -> (DocumentResult, DocumentResult, Value) {
        let v = profiled_vocabulary();
        for d in [&mut sd, &mut qd] {
            d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
            dependencies(d);
        }
        // premise dependencies are part of this typed input, not inferred from text.
        for rq in rows(&qd, "requests").to_vec() {
            for premise in rows(&rq, "premise_ids") {
                qd["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"request","id":rq["id"]},"requires":{"kind":"claim","id":premise},"role":"premise"}));
            }
        }
        let a = super::super::validate(&sd, st, &v);
        let b = super::super::validate(&qd, qt, &v);
        assert_ne!(a.report["status"], "invalid", "{}", a.report);
        assert_ne!(b.report["status"], "invalid", "{}", b.report);
        let result = classify_pair(
            &a,
            &b,
            &v,
            &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
        );
        (a, b, result)
    }
    fn mass_proposition(text: &str, unit: &str) -> Value {
        prop(atom(
            text,
            "quantity",
            vec![
                ("subject", reference("Vela")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"mass","evidence":[quote(text,"masa")]}),
                ),
                (
                    "amount",
                    json!({"kind":"quantity","decimal":"7","unit":unit,"bound":"exact","evidence":[quote(text,&format!("7 {unit}"))]}),
                ),
            ],
        ))
    }
    fn mass_question(text: &str) -> Value {
        prop(atom(
            text,
            "quantity",
            vec![
                ("subject", reference("Vela")),
                (
                    "attribute",
                    json!({"kind":"symbol","id":"mass","evidence":[quote(text,"masa")]}),
                ),
            ],
        ))
    }
    #[test]
    fn attribute_profile_mismatch_cannot_publish_and_does_not_erase_independent_fact() {
        let st = "Vela tiene masa 7 h. Mara posee Helia.";
        let qt = "¿Qué masa tiene Vela?";
        let mut sd = doc(st, "source", &["Vela", "Mara", "Helia"]);
        sd["claims"] = json!([
            claim(st, "bad-mass", mass_proposition(st, "h")),
            claim(
                st,
                "ownership",
                prop(atom(
                    st,
                    "owns",
                    vec![("owner", reference("Mara")), ("object", reference("Helia"))]
                ))
            )
        ]);
        let mut qd = doc(qt, "question", &["Vela"]);
        qd["requests"] = json!([request(qt, "answer", "amount", &mass_question(qt))]);
        let (a, b, result) = profiled_documents(sd.clone(), st, qd, qt);
        assert_eq!(a.report["status"], "partial");
        assert_eq!(b.report["status"], "complete");
        assert_eq!(result["status"], "unresolved", "{result}");
        assert!(rows(&result, "selected_claim_ids").is_empty());
        assert_eq!(
            rows(&a.report["document"], "claims").len(),
            2,
            "original validated report is intact"
        );
        let qt = "¿Mara posee Helia?";
        let mut qd = doc(qt, "question", &["Mara", "Helia"]);
        qd["requests"] = json!([request(
            qt,
            "truth",
            "status",
            &prop(atom(
                qt,
                "owns",
                vec![("owner", reference("Mara")), ("object", reference("Helia"))]
            ))
        )]);
        let (_, _, result) = profiled_documents(sd, st, qd, qt);
        assert_eq!(result["complete_answer"], true, "{result}");
        assert_eq!(result["selected_claim_ids"], json!(["ownership"]));
    }
    #[test]
    fn incompatible_request_unit_is_pending_and_allowed_units_do_not_authorize_conversion() {
        let st = "Vela tiene masa 7 kg.";
        let mut sd = doc(st, "source", &["Vela"]);
        sd["claims"] = json!([claim(st, "mass", mass_proposition(st, "kg"))]);
        for (unit, selected, question_status) in [("h", false, "partial"), ("g", true, "complete")]
        {
            let qt = format!("¿Qué masa en {unit} tiene Vela?");
            let mut qd = doc(&qt, "question", &["Vela"]);
            let mut rq = request(&qt, "answer", "amount", &mass_question(&qt));
            rq["unit"] = json!({"symbol":unit,"evidence":[quote(&qt,unit)]});
            qd["requests"] = json!([rq]);
            let (a, b, result) = profiled_documents(sd.clone(), st, qd, &qt);
            assert_eq!(a.report["status"], "complete");
            assert_eq!(b.report["status"], question_status);
            assert_eq!(
                !rows(&result, "selected_claim_ids").is_empty(),
                selected,
                "{result}"
            );
            assert_eq!(result["complete_answer"], false, "{result}");
            if selected {
                assert_eq!(
                    result["requests"][0]["partial"][0]["answer_terms"][0]["unit"],
                    "kg"
                );
            }
        }
    }
    #[test]
    fn restriction_member_unit_cannot_escape_the_attribute_profile() {
        let st = "Las opciones de masa de Vela incluyen 7 h en una tabla con referencia 7 kg.";
        let qt = "¿Qué masa tiene Vela?";
        let mut sd = doc(st, "source", &["Vela"]);
        let p = mass_proposition(st, "kg");
        sd["claims"] = json!([{"id":"restricted","scope_id":"root","qualifier_ids":[],"evidence":[quote(st,st)],"body":{"kind":"value_restriction","target":p["atom"],"answer_role":"amount","closure":"closed","members":[{"kind":"quantity","decimal":"7","unit":"h","bound":"exact","evidence":[quote(st,"7 h")]}],"judgment":"fact","status":"state","basis":"description","normative_scope":null,"existence":"asserted"}}]);
        let mut qd = doc(qt, "question", &["Vela"]);
        qd["requests"] = json!([request(qt, "answer", "amount", &mass_question(qt))]);
        let (a, _, result) = profiled_documents(sd, st, qd, qt);
        assert_eq!(a.report["status"], "partial");
        assert!(
            rows(&a.report, "issues")
                .iter()
                .any(|i| i["path"] == "/claims/0/body/members/0")
        );
        assert_eq!(result["status"], "unresolved", "{result}");
        assert!(rows(&result, "selected_claim_ids").is_empty());
    }
    #[test]
    fn question_premise_profile_failure_is_transitive_and_never_authorizes_selection() {
        let st = "Vela tiene masa 7 kg. Mara posee Helia.";
        let qt = "Si Vela tiene masa 7 h, ¿Mara posee Helia?";
        let mut sd = doc(st, "source", &["Vela", "Mara", "Helia"]);
        sd["claims"] = json!([
            claim(st, "mass", mass_proposition(st, "kg")),
            claim(
                st,
                "ownership",
                prop(atom(
                    st,
                    "owns",
                    vec![("owner", reference("Mara")), ("object", reference("Helia"))]
                ))
            )
        ]);
        let mut qd = doc(qt, "question", &["Vela", "Mara", "Helia"]);
        qd["claims"] = json!([claim(qt, "assumption", mass_proposition(qt, "h"))]);
        let mut rq = request(
            qt,
            "truth",
            "status",
            &prop(atom(
                qt,
                "owns",
                vec![("owner", reference("Mara")), ("object", reference("Helia"))],
            )),
        );
        rq["premise_ids"] = json!(["assumption"]);
        qd["requests"] = json!([rq]);
        let (_, b, result) = profiled_documents(sd, st, qd, qt);
        assert_eq!(b.report["status"], "partial");
        assert!(
            rows(&b.report, "candidates")
                .iter()
                .any(|c| c["request_id"] == "ask"
                    && rows(c, "reason_codes").contains(&json!("attribute_profile_mismatch")))
        );
        assert_eq!(result["status"], "unresolved", "{result}");
        assert!(rows(&result, "selected_claim_ids").is_empty());
    }
    fn network_filter_case(destination: &str, question: &str) -> (Value, String, Value) {
        let text = format!("{destination} recibe agua desde la tubería Tilo de la red.");
        let mut source = doc(&text, "source", &[destination, "Tilo"]);
        source["references"][1]["declared_class"] = quote(&text, "tubería");
        let mut network = source["references"][1].clone();
        network["id"] = json!("source-network");
        network["name"] = Value::Null;
        network["mention"] = quote(&text, "red");
        network["declared_class"] = quote(&text, "red");
        source["references"].as_array_mut().unwrap().push(network);
        source["references"][1]["links"] = json!([{"kind":"part_of","target":"source-network","evidence":[quote(&text,"tubería Tilo de la red")]}]);
        source["claims"] = json!([claim(
            &text,
            "origin-fact",
            prop(atom(
                &text,
                "origin",
                vec![
                    ("subject", reference(destination)),
                    ("origin", reference("Tilo"))
                ]
            ))
        )]);
        let mut query = doc(question, "question", &[destination]);
        let mut wanted = source["references"][1].clone();
        wanted["id"] = json!("wanted-pipe");
        wanted["name"] = Value::Null;
        wanted["mention"] = quote(question, "tubería de la red");
        wanted["declared_class"] = quote(question, "tubería");
        wanted["links"] = json!([{"kind":"part_of","target":"wanted-network","evidence":[quote(question,"tubería de la red")]}]);
        let mut network = wanted.clone();
        network["id"] = json!("wanted-network");
        network["mention"] = quote(question, "red");
        network["declared_class"] = quote(question, "red");
        network["links"] = json!([]);
        query["references"]
            .as_array_mut()
            .unwrap()
            .extend([wanted, network]);
        let mut rq = request(
            question,
            "answer",
            "origin",
            &prop(atom(
                question,
                "origin",
                vec![("subject", reference(destination))],
            )),
        );
        rq["class_filter"] = json!({"kind":"reference_pattern","reference_id":"wanted-pipe"});
        query["requests"] = json!([rq]);
        (source, text, query)
    }
    #[test]
    fn four_network_questions_require_explicit_pipe_and_network_pattern() {
        for (destination, question) in [
            (
                "Nardo",
                "¿De qué tubería de la red procede el agua de riego del huerto Nardo?",
            ),
            (
                "Malva",
                "¿De qué tubería de la red procede el agua de riego del huerto Malva?",
            ),
            (
                "Azucena",
                "¿De qué tubería de la red procede el agua de riego del huerto Azucena?",
            ),
            (
                "Olmo",
                "¿De qué tubería de la red procede el agua del parque Olmo?",
            ),
        ] {
            let (source, text, query) = network_filter_case(destination, question);
            let result = pair(source.clone(), &text, query.clone(), question);
            assert_eq!(result["complete_answer"], true, "{result}");
            let mut missing = source.clone();
            missing["references"][1]["links"] = json!([]);
            let result = pair(missing, &text, query.clone(), question);
            assert_eq!(result["complete_answer"], false, "{result}");
            assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
            assert!(
                !rows(&result["requests"][0], "unresolved").is_empty(),
                "{result}"
            );
            let mut other_class = source;
            other_class["references"][1]["declared_class"] = quote(&text, "agua");
            let result = pair(other_class, &text, query, question);
            assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
        }
    }
    #[test]
    fn network_filter_preserves_matching_member_of_a_mixed_closed_set() {
        let question = "¿De qué tubería de la red procede el agua del parque Olmo?";
        let (mut source, text, query) = network_filter_case("Olmo", question);
        let mut other = source["references"][1].clone();
        other["id"] = json!("other-origin");
        other["declared_class"] = quote(&text, "agua");
        other["links"] = json!([]);
        source["references"].as_array_mut().unwrap().push(other);
        let p = source["claims"][0]["body"]["value"].clone();
        source["claims"][0]["body"] = json!({
            "kind":"value_restriction","target":p["atom"],"answer_role":"origin",
            "closure":"closed","members":[reference("Tilo"),reference("other-origin")],
            "judgment":"fact","status":"state","basis":"description",
            "normative_scope":null,"existence":"asserted"
        });
        let result = pair(source, &text, query, question);
        assert_eq!(result["complete_answer"], false, "{result}");
        assert!(!rows(&result, "selected_claim_ids").is_empty(), "{result}");
        let partial = rows(&result["requests"][0], "partial");
        assert!(!partial.is_empty(), "{result}");
        assert!(
            partial
                .iter()
                .all(|w| rows(w, "answer_terms") == [reference("Tilo")]),
            "{result}"
        );
    }
    #[test]
    fn normative_premise_correction_preserves_person_course_and_unmet_guard() {
        fn scenario(person: &str, course: &str, guarded: bool) -> Value {
            let source_text = if guarded {
                format!("En {course} no se exige tutor para {person} si hay permiso.")
            } else {
                format!("En {course} no se exige tutor para {person}.")
            };
            let mut vocab = vocabulary();
            vocab["predicate_rows"].as_array_mut().unwrap().push(json!({"id":"requirement_class","roles":{"subject":"ref","class":"term"},"distributive":true}));
            vocab["fixed_action_rows"]
                .as_array_mut()
                .unwrap()
                .push(json!("accompany"));
            let question_text = "¿Qué tutor acompaña a Ina en el curso Faro?";
            let mut sd = doc(&source_text, "source", &[person, course]);
            let mut qd = doc(question_text, "question", &["Ina", "Faro"]);
            qd["references"][1]["declared_class"] = quote(question_text, "curso");
            let class = |text: &str| json!({"kind":"text","value":"tutor","evidence":[quote(text,"tutor")]});
            let mut negative = prop(atom(
                &source_text,
                "requirement_class",
                vec![
                    ("subject", reference(person)),
                    ("class", class(&source_text)),
                ],
            ));
            negative["atom"]["polarity"] = json!("negative");
            negative["basis"] = json!("normative");
            negative["normative_scope"] = json!(course);
            sd["claims"] = json!([claim(&source_text, "not-required", negative)]);
            let mut assumed = prop(atom(
                question_text,
                "requirement_class",
                vec![
                    ("subject", reference("Ina")),
                    ("class", class(question_text)),
                ],
            ));
            assumed["basis"] = json!("normative");
            assumed["normative_scope"] = json!("Faro");
            qd["claims"] = json!([claim(question_text, "assumed-requirement", assumed)]);
            let desired = prop(atom(
                question_text,
                "action",
                vec![
                    (
                        "action_kind",
                        json!({"kind":"symbol","id":"accompany","evidence":[quote(question_text,"acompaña")]}),
                    ),
                    ("object", reference("Ina")),
                ],
            ));
            let mut ask = request(question_text, "answer", "actor", &desired);
            ask["premise_ids"] = json!(["assumed-requirement"]);
            qd["requests"] = json!([ask]);
            if guarded {
                let condition = prop(atom(
                    &source_text,
                    "property",
                    vec![
                        ("subject", reference(person)),
                        (
                            "attribute",
                            json!({"kind":"symbol","id":"validity_state","evidence":[quote(&source_text,"permiso")]}),
                        ),
                        (
                            "value",
                            json!({"kind":"text","value":"permiso","evidence":[quote(&source_text,"permiso")]}),
                        ),
                    ],
                ));
                sd["qualifiers"] = json!([{"id":"permission-guard","scope_id":"root","kind":"guard","body":condition,"resolution":"known","evidence":[quote(&source_text,"si hay permiso")]}]);
                sd["claims"][0]["qualifier_ids"] = json!(["permission-guard"]);
            }
            dependencies(&mut sd);
            dependencies(&mut qd);
            qd["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"request","id":"ask"},"requires":{"kind":"claim","id":"assumed-requirement"},"role":"premise"}));
            sd["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&vocab).unwrap()));
            qd["vocabulary_sha256"] = sd["vocabulary_sha256"].clone();
            let source = super::super::validate(&sd, &source_text, &vocab);
            let question = super::super::validate(&qd, question_text, &vocab);
            assert_ne!(source.report["status"], "invalid", "{}", source.report);
            assert_ne!(question.report["status"], "invalid", "{}", question.report);
            classify_pair(
                &source,
                &question,
                &vocab,
                &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
            )
        }
        let relevant = scenario("Ina", "Faro", false);
        assert!(
            !rows(&relevant["requests"][0], "partial").is_empty(),
            "{relevant}"
        );
        assert_eq!(relevant["complete_answer"], false, "{relevant}");
        for unrelated in [
            scenario("Uma", "Faro", false),
            scenario("Ina", "Muelle", false),
        ] {
            assert!(
                rows(&unrelated, "selected_claim_ids").is_empty(),
                "{unrelated}"
            );
            assert!(
                rows(&unrelated["requests"][0], "partial").is_empty(),
                "{unrelated}"
            );
        }
        let guarded = scenario("Ina", "Faro", true);
        assert_eq!(guarded["complete_answer"], false, "{guarded}");
        assert!(
            rows(&guarded["requests"][0], "refutation").is_empty(),
            "{guarded}"
        );
        assert!(
            rows(&guarded["requests"][0], "partial")
                .iter()
                .all(|w| w["kind"] != "premise_refutation"),
            "{guarded}"
        );
        assert_eq!(guarded["requires_review"], true, "{guarded}");
    }
    #[test]
    fn closure_domains_limit_exclusions_and_preserve_independent_facts() {
        let bank: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-closure-domain-controls-v1.json"
        ))
        .unwrap();
        for case in rows(&bank, "cases") {
            let input = &case["input"];
            let v = &input["vocabulary"];
            let a = super::super::validate(&input["source"]["raw"], s(&input["source"], "text"), v);
            let b =
                super::super::validate(&input["question"]["raw"], s(&input["question"], "text"), v);
            assert_eq!(
                a.report["status"],
                case["expected_source_status"],
                "{} {}",
                id(case),
                a.report
            );
            assert_eq!(b.report["status"], "complete", "{} {}", id(case), b.report);
            let result = classify_pair(&a, &b, v, &input["pair_scope"]);
            let first = &result["requests"][0];
            for outcome in ["support", "refutation"] {
                assert_eq!(
                    !rows(first, outcome).is_empty(),
                    case["expected_first_outcome"] == outcome,
                    "{} {}",
                    id(case),
                    result
                );
            }
            if case["expected_first_outcome"] == "none" {
                assert!(rows(first, "partial").is_empty(), "{} {}", id(case), result);
            }
            for outcome in ["support", "refutation", "partial"] {
                for witness in rows(first, outcome) {
                    if let Some(domain) =
                        input["source"]["raw"]["claims"][0]["body"].get("answer_domain")
                    {
                        assert_eq!(
                            &witness["restriction_domain"],
                            domain,
                            "{} {}",
                            id(case),
                            result
                        );
                    }
                }
            }
            if case["independent_fact_must_survive"] == true {
                assert!(
                    !rows(&result["requests"][1], "support").is_empty(),
                    "{} {}",
                    id(case),
                    result
                );
                assert!(
                    rows(&result, "selected_claim_ids").contains(&json!("independent")),
                    "{} {}",
                    id(case),
                    result
                );
                assert!(
                    !rows(&result, "selected_claim_ids").contains(&json!("limited")),
                    "{} {}",
                    id(case),
                    result
                );
            }
        }
    }
    #[test]
    fn closure_domain_validation_distinguishes_unresolved_from_malformed() {
        let bank: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-closure-domain-controls-v1.json"
        ))
        .unwrap();
        for (mutation, expected) in [
            ("unknown_type", "partial"),
            ("bad_quote", "invalid"),
            ("extra_domain_field", "invalid"),
            ("bad_profile", "invalid"),
            ("scalar_domain", "partial"),
        ] {
            let mut input = bank["cases"][0]["input"].clone();
            match mutation {
                "unknown_type" => {
                    input["source"]["raw"]["claims"][0]["body"]["answer_domain"] = json!({"kind":"semantic","type_id":"unregistered","evidence":[input["source"]["raw"]["claims"][0]["body"]["answer_domain"]["class_quote"].clone()]});
                }
                "bad_quote" => {
                    input["source"]["raw"]["claims"][0]["body"]["answer_domain"]["class_quote"]["quote"] =
                        json!("invented");
                }
                "extra_domain_field" => {
                    input["source"]["raw"]["claims"][0]["body"]["answer_domain"]["untrusted"] =
                        json!(true);
                }
                "bad_profile" => {
                    input["vocabulary"]["restriction_domain_profile"]["version"] = json!("unknown");
                }
                "scalar_domain" => {
                    input["vocabulary"]["predicate_rows"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|r| r["id"] == "owns")
                        .unwrap()["roles"]["object"] = json!("term");
                }
                _ => unreachable!(),
            }
            let v = &input["vocabulary"];
            input["source"]["raw"]["vocabulary_sha256"] =
                json!(digest(&serde_json::to_vec(v).unwrap()));
            let result = super::super::validate(
                &input["source"]["raw"],
                s(&input["source"], "text"),
                &input["vocabulary"],
            );
            assert_eq!(
                result.report["status"], expected,
                "{mutation} {}",
                result.report
            );
            if expected == "partial" {
                assert!(
                    rows(&result.report, "candidates")
                        .iter()
                        .any(|c| rows(c, "claim_ids").contains(&json!("independent"))
                            && c["state"] == "eligible"),
                    "{mutation} {}",
                    result.report
                );
            }
        }
    }
    include!("described_tests.rs");
    include!("temporal_premise_tests.rs");
    include!("qualifier_dependency_tests.rs");
}
