//! Source-bound nominal event; the core and attachment are never projected as facts.
use super::{
    context_budget as budget, lexical_clause_context as graph,
    linguistic_composition as composition,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
pub(crate) const FEATURE: &str = "nonagentive_nominal_event_with_contexts_v1";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    start_byte: u64,
    end_byte: u64,
    quote: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RefTerm {
    kind: String,
    id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Core {
    id: String,
    action_id: String,
    action_evidence: Evidence,
    construction: String,
    construction_evidence: Evidence,
    actuality: String,
    temporal_resolution: String,
    patient: RefTerm,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Region {
    id: String,
    kind: String,
    core_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Attachment {
    kind: String,
    resolution: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    id: String,
    kind: String,
    preposition: Evidence,
    complement: RefTerm,
    complement_evidence: Evidence,
    phrase_evidence: Evidence,
    region_id: String,
    attachment: Attachment,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Body {
    kind: String,
    constructor: String,
    composition_id: String,
    evidence: Evidence,
    core: Core,
    region: Region,
    contexts: Vec<Context>,
}
fn rows<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn step() -> bool {
    budget::charge(1, 0)
}
fn local_id(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 128
}
impl Evidence {
    fn valid(&self, text: &str) -> bool {
        budget::charge(self.quote.len() + 1, 0)
            && self.start_byte < self.end_byte
            && self.end_byte <= u32::MAX as u64
            && text.get(self.start_byte as usize..self.end_byte as usize)
                == Some(self.quote.as_str())
    }
    fn contains(&self, other: &Self) -> bool {
        step()
            && self.start_byte <= other.start_byte
            && other.start_byte < other.end_byte
            && other.end_byte <= self.end_byte
    }
    fn matches(&self, v: &Value) -> bool {
        step()
            && v["start_byte"] == self.start_byte
            && v["end_byte"] == self.end_byte
            && v["quote"] == self.quote
    }
}
pub(crate) fn carries(body: &Value) -> bool {
    body["kind"] == composition::KIND && body["constructor"] == FEATURE
}
/// A pattern is a local compound description, never Same of two physical patients.
pub(crate) fn patient_key(doc: &Value, rid: &str) -> Option<Value> {
    let r = rows(doc, "references")
        .iter()
        .find(|r| step() && r["id"] == rid)?;
    let links = rows(r, "links");
    if r["denotation"] != "entity"
        || !r["name"].is_null()
        || !r["binding_scope_id"].is_null()
        || !r["anaphora"].is_null()
        || !r["semantic_type"].is_null()
        || links.len() != 1
        || links[0]["kind"] != super::nominal_de::KIND
        || !r["declared_class"]["quote"]
            .as_str()
            .is_some_and(|s| !s.trim().is_empty())
    {
        return None;
    }
    if rows(doc, "identity")
        .iter()
        .any(|e| !step() || e["left"] == rid || e["right"] == rid)
    {
        return None;
    }
    // Metadata, aliases and other links may not be erased by this first subset.
    for other in rows(doc, "references") {
        if !step() {
            return None;
        }
        if other["anaphora"]["antecedent_ids"]
            .as_array()
            .is_some_and(|a| a.iter().any(|x| x == rid))
        {
            return None;
        }
    }
    let tail = links[0]["target"].as_str()?;
    if rows(doc, "identity")
        .iter()
        .any(|e| !step() || e["left"] == tail || e["right"] == tail)
    {
        return None;
    }
    let descriptor = super::nominal_de::descriptor_key(doc, tail)?;
    if super::nominal_de::component(doc, tail)?.len() != 1 {
        return None;
    }
    if !composition::reserve(r, 1) || !composition::reserve(&descriptor, 1) {
        return None;
    }
    Some(json!({"head":r["declared_class"]["quote"],"descriptor":descriptor}))
}
/// A lexical shape cannot erase independent restrictions asserted on that patient.
pub(crate) fn pattern_constraints_clean(doc: &Value, rid: &str, own_claim: Option<&str>) -> bool {
    let closure = graph::closure(doc, "reference", rid);
    if closure
        .iter()
        .any(|(kind, _)| !matches!(kind.as_str(), "reference" | "scope"))
    {
        return false;
    }
    let mut stack = Vec::new();
    for c in rows(doc, "claims") {
        if !step() {
            return false;
        }
        if own_claim == c["id"].as_str() {
            continue;
        }
        if !budget::charge(1, 64) {
            return false;
        }
        stack.push(&c["body"]);
    }
    while let Some(v) = stack.pop() {
        if !budget::charge(1, 64) {
            return false;
        }
        if v["kind"] == "ref"
            && closure
                .iter()
                .any(|(kind, id)| step() && kind == "reference" && v["id"] == *id)
        {
            return false;
        }
        match v {
            Value::Object(m) => {
                if !budget::charge(m.len(), m.len().saturating_mul(64)) {
                    return false;
                }
                stack.extend(m.values());
            }
            Value::Array(a) => {
                if !budget::charge(a.len(), a.len().saturating_mul(64)) {
                    return false;
                }
                stack.extend(a.iter());
            }
            _ => {}
        }
    }
    !budget::exhausted()
}
pub(crate) fn valid(body: &Value, text: &str, doc: Option<&Value>) -> bool {
    if !composition::reserve(body, 2) {
        return false;
    }
    let Ok(c) = serde_json::from_value::<Body>(body.clone()) else {
        return false;
    };
    if c.kind != composition::KIND
        || c.constructor != FEATURE
        || !(1..=2).contains(&c.contexts.len())
        || c.core.action_id != "remove"
        || c.core.construction != "nonagentive_se"
        || c.core.actuality != "occurred"
        || c.core.temporal_resolution != "source_past"
        || c.core.patient.kind != "ref"
        || !local_id(&c.core.patient.id)
        || c.region.kind != "core_with_patient_nominal_closure"
        || c.region.core_id != c.core.id
    {
        return false;
    }
    let mut ids = BTreeSet::new();
    for id in [&c.composition_id, &c.core.id, &c.region.id]
        .into_iter()
        .chain(c.contexts.iter().map(|p| &p.id))
    {
        if !budget::charge(ids.len() + 1, id.len() + 64)
            || !local_id(id)
            || !ids.insert(id.as_str())
        {
            return false;
        }
    }
    if !c.evidence.valid(text)
        || c.evidence.quote.contains(';')
        || !c.core.action_evidence.valid(text)
        || !c.core.construction_evidence.valid(text)
        || !c.evidence.contains(&c.core.action_evidence)
        || !c.evidence.contains(&c.core.construction_evidence)
        || c.core.construction_evidence.quote != "se"
        || c.core.construction_evidence.end_byte >= c.core.action_evidence.start_byte
    {
        return false;
    }
    let mut end = 0;
    let mut complements = BTreeSet::new();
    for p in &c.contexts {
        if !step()
            || p.kind != "source_bound_prepositional_context"
            || p.region_id != c.region.id
            || p.attachment.kind != "original_source_context"
            || p.attachment.resolution != "not_published"
            || p.complement.kind != "ref"
            || !local_id(&p.complement.id)
            || !budget::charge(1, p.complement.id.len() + 64)
            || !complements.insert(p.complement.id.as_str())
            || !p.preposition.valid(text)
            || !p.complement_evidence.valid(text)
            || !p.phrase_evidence.valid(text)
            || !c.evidence.contains(&p.phrase_evidence)
            || !p.phrase_evidence.contains(&p.preposition)
            || !p.phrase_evidence.contains(&p.complement_evidence)
            || p.phrase_evidence.start_byte != p.preposition.start_byte
            || p.phrase_evidence.end_byte != p.complement_evidence.end_byte
            || p.preposition.end_byte >= p.complement_evidence.start_byte
            || p.phrase_evidence.start_byte < end
            || (p.phrase_evidence.start_byte < c.core.construction_evidence.end_byte
                && c.core.construction_evidence.start_byte < p.phrase_evidence.end_byte)
            || (p.phrase_evidence.start_byte < c.core.action_evidence.end_byte
                && c.core.action_evidence.start_byte < p.phrase_evidence.end_byte)
            || !p.preposition.quote.chars().all(char::is_alphabetic)
        {
            return false;
        }
        let gap = text
            .get(p.preposition.end_byte as usize..p.complement_evidence.start_byte as usize)
            .unwrap_or("");
        if !budget::charge(gap.len() + 1, 0)
            || gap.is_empty()
            || !gap.chars().all(char::is_whitespace)
        {
            return false;
        }
        end = p.phrase_evidence.end_byte;
    }
    if let Some(doc) = doc {
        let Some(patient) = rows(doc, "references")
            .iter()
            .find(|r| step() && r["id"] == c.core.patient.id)
        else {
            return false;
        };
        let Some(key) = patient_key(doc, &c.core.patient.id) else {
            return false;
        };
        if key.is_null()
            || patient["mention"]["start_byte"]
                .as_u64()
                .is_none_or(|n| n <= c.core.action_evidence.end_byte)
        {
            return false;
        }
        if !composition::reserve(&patient["mention"], 2) {
            return false;
        }
        let Ok(mention) = serde_json::from_value::<Evidence>(patient["mention"].clone()) else {
            return false;
        };
        if !mention.valid(text) || !c.evidence.contains(&mention) {
            return false;
        }
        let Some(tail) = rows(patient, "links")
            .first()
            .and_then(|l| l["target"].as_str())
            .and_then(|id| {
                rows(doc, "references")
                    .iter()
                    .find(|r| step() && r["id"] == id)
            })
        else {
            return false;
        };
        if !composition::reserve(&tail["mention"], 2) {
            return false;
        }
        let Ok(tail_mention) = serde_json::from_value::<Evidence>(tail["mention"].clone()) else {
            return false;
        };
        if !tail_mention.valid(text) || !mention.contains(&tail_mention) {
            return false;
        }
        let Some(root) = rows(doc, "scopes")
            .iter()
            .find(|s| step() && s["id"] == patient["scope_id"])
        else {
            return false;
        };
        if root["kind"] != "document"
            || !root["parent_id"].is_null()
            || !root["binding"].is_null()
            || !root["branch_id"].is_null()
        {
            return false;
        }
        for p in &c.contexts {
            let Some(r) = rows(doc, "references")
                .iter()
                .find(|r| step() && r["id"] == p.complement.id)
            else {
                return false;
            };
            if r["scope_id"] != patient["scope_id"]
                || r["denotation"] != "entity"
                || r["name"].is_null()
                || !r["binding_scope_id"].is_null()
                || !r["anaphora"].is_null()
                || !rows(r, "links").is_empty()
                || (!p.complement_evidence.matches(&r["mention"])
                    && !p.complement_evidence.matches(&r["name"]))
                || p.complement.id == c.core.patient.id
                || p.phrase_evidence.start_byte < mention.end_byte
                    && mention.start_byte < p.phrase_evidence.end_byte
            {
                return false;
            }
        }
        let mut found = false;
        for claim in rows(doc, "claims") {
            if !step() {
                return false;
            }
            let comparison = budget::serialized_bytes(&claim["body"]).unwrap_or(usize::MAX);
            if !budget::charge(comparison, 0) {
                return false;
            }
            if claim["body"] != *body {
                continue;
            }
            found = true;
            if doc["kind"] != "source"
                || claim["scope_id"] != patient["scope_id"]
                || claim.get("context_bindings").is_some()
            {
                return false;
            }
            let closure = graph::closure(doc, "claim", claim["id"].as_str().unwrap_or(""));
            if closure.iter().any(|(kind, _)| kind == "qualifier") {
                return false;
            }
        }
        if !found {
            return false;
        }
    }
    !budget::exhausted()
}
pub(crate) fn seeds(doc: &Value, body: &Value, nodes: &mut graph::Nodes) {
    if !carries(body) {
        return;
    }
    let h = body["core"]["patient"]["id"].as_str().unwrap_or("");
    if !budget::charge(2, h.len().saturating_mul(2) + 256) {
        return;
    }
    nodes.insert(("reference".into(), h.into()));
    // Include the full nominal closure and aliases even on malformed DTOs.
    let mut pending = vec![h.to_owned()];
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !budget::charge(2, id.len().saturating_mul(2) + 256) {
            return;
        }
        if !seen.insert(id.clone()) {
            continue;
        }
        nodes.insert(("reference".into(), id.clone()));
        for r in rows(doc, "references") {
            if !step() {
                return;
            }
            if r["id"] == id {
                for l in rows(r, "links") {
                    if !step() {
                        return;
                    }
                    if l["kind"] == super::nominal_de::KIND {
                        if let Some(t) = l["target"].as_str() {
                            if !budget::charge(1, t.len() + 64) {
                                return;
                            }
                            pending.push(t.to_owned());
                        }
                    }
                }
            }
        }
    }
}
pub(crate) fn canonical_ast(body: &Value) -> Option<Value> {
    if !carries(body) || !composition::reserve(body, 2) {
        return None;
    }
    let mut out = body.clone();
    for (p, id) in [
        ("/composition_id", "composition"),
        ("/core/id", "core"),
        ("/region/id", "region"),
        ("/region/core_id", "core"),
    ] {
        *out.pointer_mut(p)? = json!(id);
    }
    for (i, p) in out["contexts"].as_array_mut()?.iter_mut().enumerate() {
        if !budget::charge(1, 128) {
            return None;
        }
        p["id"] = json!(format!("context_{i}"));
        p["region_id"] = json!("region");
    }
    Some(out)
}
