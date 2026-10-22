//! An inseparable open linguistic construction. Its negative core is never a fact.
use super::{context_budget as budget, lexical_clause_context as graph};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;

pub(crate) const FEATURE: &str = "linguistic_composition_until_measurement_v1";
pub(crate) const KIND: &str = "linguistic_composition";
const CONSTRUCTION: &str = "negative_se_measurement_with_until_v1";
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
#[serde(tag = "kind", deny_unknown_fields)]
enum Owner {
    #[serde(rename = "explicit_owner")]
    Explicit {
        relation: String,
        term: RefTerm,
        relation_evidence: Evidence,
        mention_evidence: Evidence,
    },
    #[serde(rename = "possessive_argument")]
    Possessive {
        id: String,
        semantic_type: String,
        grammatical_person: String,
        possessed_head: String,
        determiner_evidence: Evidence,
        resolution: Value,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Description {
    id: String,
    kind: String,
    attribute: String,
    head_evidence: Evidence,
    owner: Owner,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Voice {
    kind: String,
    evidence: Evidence,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Measurement {
    id: String,
    kind: String,
    predicate: String,
    head_evidence: Evidence,
    voice: Voice,
    object: Description,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Negation {
    id: String,
    kind: String,
    evidence: Evidence,
    operand: Measurement,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Attachment {
    kind: String,
    region_root_id: String,
    resolution: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Argument {
    id: String,
    kind: String,
    semantic_type: String,
    realization_evidence: Evidence,
    resolution: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Boundary {
    id: String,
    kind: String,
    predicate: String,
    head_evidence: Evidence,
    form: String,
    subject: Argument,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Operator {
    id: String,
    kind: String,
    relation: String,
    connective_evidence: Evidence,
    phrase_evidence: Evidence,
    attachment: Attachment,
    boundary: Boundary,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Composition {
    kind: String,
    construction: String,
    clause_evidence: Evidence,
    core: Negation,
    operators: Vec<Operator>,
}
fn rows<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v[k].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn step() -> bool {
    budget::charge(1, 0)
}
fn visit(v: &Value, depth: usize) -> bool {
    if depth > 64 || !step() {
        budget::charge(usize::MAX, 0);
        return false;
    }
    match v {
        Value::Array(a) => a.iter().all(|x| visit(x, depth + 1)),
        Value::Object(m) => m.values().all(|x| visit(x, depth + 1)),
        _ => true,
    }
}
pub(crate) fn reserve(v: &Value, copies: usize) -> bool {
    visit(v, 0)
        && budget::serialized_bytes(v).is_some_and(|n| budget::charge(1, n.saturating_mul(copies)))
}
fn local_id(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 128
}
impl Evidence {
    fn valid(&self, text: &str) -> bool {
        budget::charge(self.quote.len().saturating_add(1), 0)
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
    fn equals(&self, other: &Self) -> bool {
        step()
            && self.start_byte == other.start_byte
            && self.end_byte == other.end_byte
            && self.quote == other.quote
    }
}
/// Validates the closed typed tree and its local incidence, without resolving any hole.
pub(crate) fn valid(body: &Value, text: &str, doc: Option<&Value>) -> bool {
    if !reserve(body, 2) {
        return false;
    }
    let Ok(c) = serde_json::from_value::<Composition>(body.clone()) else {
        return false;
    };
    if c.kind != KIND || c.construction != CONSTRUCTION || c.operators.len() != 1 {
        return false;
    }
    let m = &c.core.operand;
    let q = &m.object;
    let o = &c.operators[0];
    let b = &o.boundary;
    let s = &b.subject;
    if c.core.kind != "negation"
        || m.kind != "measurement_predicate"
        || m.predicate != "measure"
        || m.voice.kind != "nonagentive_se"
        || q.kind != "quantity_description"
        || q.attribute != "duration"
        || o.kind != "lexical_temporal_boundary"
        || o.relation != "until"
        || o.attachment.kind != "source_underspecified"
        || o.attachment.region_root_id != c.core.id
        || !o.attachment.resolution.is_null()
        || b.kind != "event_predicate"
        || b.predicate != "switch_off"
        || b.form != "reflexive_infinitive"
        || s.kind != "implicit_argument"
        || s.semantic_type != "entity"
        || !s.resolution.is_null()
    {
        return false;
    }
    let mut ids = BTreeSet::new();
    for id in [&c.core.id, &m.id, &q.id, &o.id, &b.id, &s.id] {
        if !budget::charge(ids.len() + 1, id.len() + 64)
            || !local_id(id)
            || !ids.insert(id.as_str())
        {
            return false;
        }
    }
    let evidence = [
        &c.core.evidence,
        &m.head_evidence,
        &m.voice.evidence,
        &q.head_evidence,
        &o.connective_evidence,
        &o.phrase_evidence,
        &b.head_evidence,
        &s.realization_evidence,
    ];
    if !c.clause_evidence.valid(text)
        || evidence
            .iter()
            .any(|e| !e.valid(text) || !c.clause_evidence.contains(e))
        || !s.realization_evidence.equals(&b.head_evidence)
        || c.clause_evidence.start_byte != c.core.evidence.start_byte
        || c.clause_evidence.end_byte != o.phrase_evidence.end_byte
        || o.phrase_evidence.start_byte != o.connective_evidence.start_byte
        || o.phrase_evidence.end_byte != b.head_evidence.end_byte
        || o.connective_evidence.end_byte >= b.head_evidence.start_byte
        || c.core.evidence.end_byte > m.voice.evidence.start_byte
        || m.voice.evidence.end_byte > m.head_evidence.start_byte
        || m.head_evidence.end_byte > q.head_evidence.start_byte
        || q.head_evidence.end_byte > o.connective_evidence.start_byte
    {
        return false;
    }
    let gap = text
        .get(o.connective_evidence.end_byte as usize..b.head_evidence.start_byte as usize)
        .unwrap_or("");
    if !budget::charge(gap.len(), 0) || !gap.chars().all(char::is_whitespace) {
        return false;
    }
    match &q.owner {
        Owner::Explicit {
            relation,
            term,
            relation_evidence,
            mention_evidence,
        } => {
            if relation != "duration_bearer"
                || term.kind != "ref"
                || !local_id(&term.id)
                || !relation_evidence.valid(text)
                || !mention_evidence.valid(text)
                || !c.clause_evidence.contains(relation_evidence)
                || !c.clause_evidence.contains(mention_evidence)
                || q.head_evidence.end_byte > relation_evidence.start_byte
                || relation_evidence.end_byte > mention_evidence.start_byte
                || mention_evidence.end_byte > o.connective_evidence.start_byte
            {
                return false;
            }
            if let Some(doc) = doc {
                let Some(reference) = rows(doc, "references")
                    .iter()
                    .find(|r| step() && r["id"] == term.id)
                else {
                    return false;
                };
                let mention = &reference["mention"];
                if mention["start_byte"] != mention_evidence.start_byte
                    || mention["end_byte"] != mention_evidence.end_byte
                    || mention["quote"] != mention_evidence.quote
                {
                    return false;
                }
            }
        }
        Owner::Possessive {
            id,
            semantic_type,
            grammatical_person,
            possessed_head,
            determiner_evidence,
            resolution,
        } => {
            if !budget::charge(ids.len() + 1, id.len() + 64)
                || !local_id(id)
                || !ids.insert(id.as_str())
                || semantic_type != "entity"
                || grammatical_person != "third"
                || possessed_head != "duration"
                || !resolution.is_null()
                || !determiner_evidence.valid(text)
                || !c.clause_evidence.contains(determiner_evidence)
                || m.head_evidence.end_byte > determiner_evidence.start_byte
                || determiner_evidence.end_byte > q.head_evidence.start_byte
            {
                return false;
            }
        }
    }
    !budget::exhausted()
}
pub(crate) fn carries(claim: &Value) -> bool {
    claim["body"]["kind"] == KIND
}
pub(crate) fn blocked_nodes(doc: &Value) -> graph::Nodes {
    let mut seeds = graph::Nodes::new();
    if !budget::composition_enabled() {
        return seeds;
    }
    for c in rows(doc, "claims") {
        if !step() {
            return seeds;
        }
        if carries(c) {
            if !budget::charge(1, c["id"].as_str().map_or(0, str::len) + 128) {
                return seeds;
            }
            seeds.insert(("claim".into(), c["id"].as_str().unwrap_or("").to_owned()));
            super::nominal_event_context::seeds(doc, &c["body"], &mut seeds);
        }
    }
    graph::consumer_closure(doc, seeds)
}
/// Internal IDs are incidence-local addresses, not semantic constants or referents.
pub(crate) fn canonical_ast(body: &Value) -> Option<Value> {
    if super::nominal_event_context::carries(body) {
        return super::nominal_event_context::canonical_ast(body);
    }
    if body["kind"] != KIND || !reserve(body, 2) {
        return None;
    }
    let mut out = body.clone();
    for (pointer, name) in [
        ("/core/id", "core"),
        ("/core/operand/id", "measure"),
        ("/core/operand/object/id", "duration"),
        ("/operators/0/id", "until"),
        ("/operators/0/boundary/id", "switch_off"),
        ("/operators/0/boundary/subject/id", "implicit_subject"),
        ("/operators/0/attachment/region_root_id", "core"),
    ] {
        *out.pointer_mut(pointer)? = Value::String(name.into());
    }
    if out["core"]["operand"]["object"]["owner"]["kind"] == "possessive_argument" {
        *out.pointer_mut("/core/operand/object/owner/id")? =
            Value::String("possessive_owner".into());
    }
    Some(out)
}
