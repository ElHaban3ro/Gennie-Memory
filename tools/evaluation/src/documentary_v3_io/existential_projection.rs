//! Existential introduction over complete local witnesses. Never a reference identity.
//! The original validated matrix is the authoritative proof and residual store.
use super::{arr, fields, leaves, reference_key};
use crate::documentary_v3::{context_budget as budget, request_context_profile};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn enabled(v: &Value) -> bool {
    request_context_profile::permits(v, "existential_conclusion_projection")
}
type Failure = &'static str;
type Fallible<T> = Result<T, Failure>;
fn exhaust() -> Failure {
    let _ = budget::charge(usize::MAX, 0);
    "limit"
}

fn charge(v: &Value) -> Fallible<()> {
    let n = budget::serialized_bytes(v).ok_or_else(exhaust)?;
    if budget::charge(1, n) {
        Ok(())
    } else {
        Err(exhaust())
    }
}
fn copied(v: &Value) -> Fallible<Value> {
    charge(v)?;
    Ok(v.clone())
}
fn text(v: &Value) -> Fallible<&str> {
    v.as_str().ok_or("not_supported")
}
fn step() -> Fallible<()> {
    if budget::charge(1, 0) {
        Ok(())
    } else {
        Err(exhaust())
    }
}
fn exact(v: &Value, keys: &[&str]) -> Fallible<()> {
    if fields(v, keys) {
        Ok(())
    } else {
        Err("not_supported")
    }
}
fn comparisons(count: usize) -> Fallible<()> {
    if count > 4096 {
        return Err(exhaust());
    }
    let levels = usize::BITS.saturating_sub(count.leading_zeros()) as usize;
    // Reserve a conservative O(n log n) comparison allowance before sorting.
    if budget::charge(count.saturating_mul(levels).saturating_mul(4), 0) {
        Ok(())
    } else {
        Err(exhaust())
    }
}
fn sorted(xs: Vec<Value>) -> Fallible<Vec<Value>> {
    comparisons(xs.len())?;
    let mut keyed = Vec::new();
    for value in xs {
        charge(&value)?;
        if !budget::charge(1, 128) {
            return Err(exhaust());
        }
        let key = value.to_string();
        keyed.push((key, value));
    }
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    keyed.dedup_by(|a, b| a.0 == b.0);
    Ok(keyed.into_iter().map(|(_, v)| v).collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Address {
    record_kind: String,
    local_id: String,
}
fn address(kind: &str, id: &str) -> Address {
    Address {
        record_kind: kind.into(),
        local_id: id.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Edge {
    edge_id: String,
    kind: String,
    from: Address,
    to: Address,
    role: Option<String>,
    link_kind: Option<String>,
    identity_kind: Option<String>,
    dependency_role: Option<String>,
    dependency_origin: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeOrigin {
    node_id: String,
    document_side: String,
    reference_ids: Vec<String>,
    canonical_reference_key: Option<Value>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordOrigin {
    record_kind: String,
    local_record_id: String,
    document_side: String,
    document_pointers: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OriginMap {
    source_reading_id: String,
    question_reading_id: String,
    matrix_cell_index: usize,
    request_index: usize,
    support_witness_index: usize,
    node_origins: Vec<NodeOrigin>,
    record_origins: Vec<RecordOrigin>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Diagnostic {
    code: String,
    source_reading_id: Option<String>,
    question_reading_id: Option<String>,
    request_id: String,
    support_witness_index: Option<usize>,
    document_pointers: Option<Vec<String>>,
}
struct Candidate {
    request: Value,
    body: Value,
    origin: OriginMap,
    anchor_keys: Value,
}
struct Graph<'a> {
    docs: [&'a Value; 2],
    vocabulary: &'a Value,
    maps: [BTreeMap<String, String>; 2],
    visiting: BTreeSet<(usize, String)>,
    nodes: Vec<Value>,
    edges: Vec<Value>,
    origins: Vec<NodeOrigin>,
    records: Vec<RecordOrigin>,
    scopes: Vec<Value>,
    named: BTreeMap<String, String>,
    answer: BTreeSet<String>,
    closed: BTreeSet<(usize, String, String)>,
    credited: BTreeMap<String, String>,
}
impl<'a> Graph<'a> {
    fn new(
        source: &'a Value,
        question: &'a Value,
        vocabulary: &'a Value,
        answer: BTreeSet<String>,
    ) -> Self {
        Self {
            docs: [source, question],
            vocabulary,
            maps: Default::default(),
            visiting: Default::default(),
            nodes: vec![],
            edges: vec![],
            origins: vec![],
            records: vec![],
            scopes: vec![],
            named: Default::default(),
            answer,
            closed: Default::default(),
            credited: Default::default(),
        }
    }
    fn component(&self, side: usize, id: &str) -> Fallible<BTreeSet<String>> {
        if !budget::charge(1, id.len() + 64) {
            return Err(exhaust());
        }
        let mut out = BTreeSet::from([id.to_owned()]);
        loop {
            let n = out.len();
            for e in arr(self.docs[side], "identity") {
                step()?;
                if e["kind"] == "same" {
                    let a = text(&e["left"])?;
                    let b = text(&e["right"])?;
                    if out.contains(a) || out.contains(b) {
                        if !budget::charge(1, a.len() + b.len() + 128) {
                            return Err(exhaust());
                        }
                        out.insert(a.into());
                        out.insert(b.into());
                    }
                }
            }
            if n == out.len() {
                break;
            }
        }
        for e in arr(self.docs[side], "identity") {
            step()?;
            if e["kind"] == "different"
                && out.contains(text(&e["left"])?)
                && out.contains(text(&e["right"])?)
            {
                return Err("relevant_conflict");
            }
        }
        Ok(out)
    }
    fn record(&mut self, side: usize, kind: &str, local: &str, pointer: String) -> Fallible<()> {
        if !budget::charge(1, pointer.len() + local.len() + 128) {
            return Err(exhaust());
        }
        if self.docs[side].pointer(&pointer).is_none() {
            return Err("invalid_origin");
        }
        let r = RecordOrigin {
            record_kind: kind.into(),
            local_record_id: local.into(),
            document_side: if side == 0 { "source" } else { "question" }.into(),
            document_pointers: vec![pointer],
        };
        if !self.records.contains(&r) {
            self.records.push(r)
        }
        Ok(())
    }
    fn edge(
        &mut self,
        kind: &str,
        from: Address,
        to: Address,
        label: Option<&str>,
        origin: Option<&str>,
    ) -> Fallible<()> {
        if !budget::charge(1, 256) {
            return Err(exhaust());
        }
        let e = Edge {
            edge_id: String::new(),
            kind: kind.into(),
            from,
            to,
            role: if kind == "role" {
                label.map(str::to_owned)
            } else {
                None
            },
            link_kind: if kind == "reference_link" {
                label.map(str::to_owned)
            } else {
                None
            },
            identity_kind: if kind == "identity" {
                label.map(str::to_owned)
            } else {
                None
            },
            dependency_role: if kind == "dependency" {
                label.map(str::to_owned)
            } else {
                None
            },
            dependency_origin: origin.map(str::to_owned),
        };
        self.edges
            .push(serde_json::to_value(e).map_err(|_| "not_supported")?);
        Ok(())
    }
    fn scope(&mut self, side: usize, id: &str) -> Fallible<String> {
        let index = arr(self.docs[side], "scopes")
            .iter()
            .position(|s| s["id"] == id)
            .ok_or("unresolved_dependency")?;
        let s = &arr(self.docs[side], "scopes")[index];
        charge(s)?;
        exact(s, &["id", "kind", "parent_id", "binding", "branch_id"])?;
        if s["kind"] != "document"
            || !s["parent_id"].is_null()
            || !s["binding"].is_null()
            || !s["branch_id"].is_null()
        {
            return Err("not_supported");
        }
        let out =
            json!({"id":"s0","kind":"document","parent_id":null,"binding":null,"branch_id":null});
        if self.scopes.is_empty() {
            self.scopes.push(out)
        }
        self.record(side, "scope", "s0", format!("/scopes/{index}"))?;
        self.dependencies(side, "scope", id, address("scope", "s0"))?;
        Ok("s0".into())
    }
    fn descriptor(&mut self, side: usize, r: &Value) -> Fallible<Value> {
        if r["denotation"] == "lexical_use"
            || crate::documentary_v3::nominal_possessive::reference_blocked(
                self.docs[side],
                r["id"].as_str().unwrap_or(""),
            )
        {
            return Err("not_supported");
        }
        exact(
            r,
            &[
                "id",
                "scope_id",
                "mention",
                "name",
                "declared_class",
                "semantic_type",
                "denotation",
                "links",
                "binding_scope_id",
                "anaphora",
            ],
        )?;
        if !r["binding_scope_id"].is_null() || !r["anaphora"].is_null() {
            return Err("not_supported");
        }
        let scope = self.scope(side, text(&r["scope_id"])?)?;
        let class = if r["declared_class"].is_null() {
            Value::Null
        } else {
            json!({"quote":text(&r["declared_class"]["quote"])?})
        };
        let semantic = if r["semantic_type"].is_null() {
            Value::Null
        } else {
            exact(&r["semantic_type"], &["type_id", "evidence"])?;
            json!({"type_id":text(&r["semantic_type"]["type_id"])?})
        };
        Ok(
            json!({"denotation":r["denotation"],"declared_class":class,"semantic_type":semantic,"scope_id":scope}),
        )
    }
    fn reference(
        &mut self,
        side: usize,
        id: &str,
        nominal_owner: Option<&str>,
    ) -> Fallible<String> {
        step()?;
        if let Some(n) = self.maps[side].get(id) {
            if !budget::charge(1, n.len()) {
                return Err(exhaust());
            }
            return Ok(n.clone());
        }
        if self.visiting.len() > 64 {
            return Err(exhaust());
        }
        if !budget::charge(1, id.len() + 64) {
            return Err(exhaust());
        }
        if !self.visiting.insert((side, id.into())) {
            return Err("unresolved_dependency");
        }
        let component = self.component(side, id)?;
        let mut refs = Vec::new();
        let mut descriptors = Vec::new();
        let mut names = Vec::new();
        for member in &component {
            step()?;
            let index = arr(self.docs[side], "references")
                .iter()
                .position(|r| r["id"] == *member)
                .ok_or("unresolved_dependency")?;
            let r = &arr(self.docs[side], "references")[index];
            charge(r)?;
            descriptors.push(self.descriptor(side, r)?);
            if !r["name"].is_null() {
                names.push(json!({"quote":text(&r["name"]["quote"])?}));
            }
            refs.push((index, r));
        }
        descriptors = sorted(descriptors)?;
        names = sorted(names)?;
        if descriptors.is_empty() {
            return Err("not_supported");
        }
        // Same aliases may repeat a descriptor but cannot silently change denotation/class/type.
        if descriptors.iter().any(|d| {
            d["denotation"] != descriptors[0]["denotation"]
                || d["declared_class"] != descriptors[0]["declared_class"]
                || d["semantic_type"] != descriptors[0]["semantic_type"]
        }) {
            return Err("relevant_conflict");
        }
        let bound = side == 0 && component == self.answer;
        let key = if names.is_empty() {
            None
        } else {
            Some(reference_key(self.docs[side], id, &mut BTreeSet::new()).ok_or("not_supported")?)
        };
        if bound && (!names.is_empty() || descriptors[0]["denotation"] != "entity") {
            return Err("not_supported");
        }
        if !bound && names.is_empty() && nominal_owner.is_none() {
            return Err("not_supported");
        }
        if nominal_owner.is_some() {
            if !request_context_profile::permits(self.vocabulary, "nominal_complement_de")
                || !names.is_empty()
                || descriptors[0]["denotation"] != "entity"
                || crate::documentary_v3::nominal_de::descriptor_key(self.docs[side], id).is_none()
            {
                return Err("not_supported");
            }
        }
        let mut descriptor = copied(&descriptors[0])?;
        descriptor["alias_descriptors"] = json!(descriptors);
        if !names.is_empty() {
            descriptor["domain_namespace"] = copied(&self.docs[side]["domain_namespace"])?;
            descriptor["name_anchors"] = json!(names)
        }
        let named_key = if key.is_some() {
            charge(&descriptor)?;
            if let Some(key) = &key {
                charge(key)?;
            }
            let value = json!([side, descriptor, key]);
            charge(&value)?;
            Some(value.to_string())
        } else {
            None
        };
        if !budget::charge(1, 128) {
            return Err(exhaust());
        }
        let credited = if side == 1 {
            component
                .iter()
                .filter_map(|id| self.credited.get(id))
                .filter_map(|id| self.maps[0].get(id))
                .next()
                .cloned()
        } else {
            None
        };
        let existing =
            credited.or_else(|| named_key.as_ref().and_then(|k| self.named.get(k)).cloned());
        let node_id = if bound {
            "n0".into()
        } else if let Some(n) = existing {
            n
        } else {
            format!("n{}", self.nodes.len() + 1)
        };
        if !budget::charge(
            1,
            node_id.len() + component.iter().map(|s| s.len() + 64).sum::<usize>(),
        ) {
            return Err(exhaust());
        }
        for m in &component {
            self.maps[side].insert(m.clone(), node_id.clone());
        }
        let node = if bound {
            json!({"kind":"bound_variable","node_id":node_id,"variable":"v0","descriptor":descriptor})
        } else if let Some(owner) = nominal_owner {
            json!({"kind":"local_nominal_lexical_use","node_id":node_id,"owner_node_id":owner,"descriptor":descriptor})
        } else {
            json!({"kind":"named_anchor","node_id":node_id,"descriptor":descriptor})
        };
        if let Some(existing) = self.nodes.iter().find(|n| n["node_id"] == node_id) {
            if existing["descriptor"] != node["descriptor"] {
                return Err("not_supported");
            }
        } else {
            self.nodes.push(node)
        }
        // Component equality is intrinsic, independent of raw alias cardinality.
        self.edge(
            "identity",
            address("node", &node_id),
            address("node", &node_id),
            Some("same"),
            None,
        )?;
        if let Some(k) = named_key {
            self.named.insert(k, node_id.clone());
        }
        self.origins.push(NodeOrigin {
            node_id: node_id.clone(),
            document_side: if side == 0 { "source" } else { "question" }.into(),
            reference_ids: component.iter().cloned().collect(),
            canonical_reference_key: key,
        });
        let mut links = Vec::new();
        for (index, r) in &refs {
            self.edge(
                "scope_owner",
                address("node", &node_id),
                address("scope", "s0"),
                None,
                None,
            )?;
            for (li, link) in arr(r, "links").iter().enumerate() {
                step()?;
                exact(link, &["kind", "target", "evidence"])?;
                let target = text(&link["target"])?;
                let target_ref = arr(self.docs[side], "references")
                    .iter()
                    .find(|r| r["id"] == target)
                    .ok_or("unresolved_dependency")?;
                charge(link)?;
                charge(target_ref)?;
                let sort_value = json!([
                    link["kind"],
                    target_ref["name"]["quote"],
                    target_ref["declared_class"]["quote"],
                    target_ref["semantic_type"]
                ]);
                charge(&sort_value)?;
                let sort_key = sort_value.to_string();
                if !budget::charge(1, sort_key.len() + 128) {
                    return Err(exhaust());
                }
                links.push((sort_key, target, link, index, li));
            }
        }
        comparisons(links.len())?;
        links.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, target, link, index, li) in links {
            let kind = text(&link["kind"])?;
            if component.contains(target) || self.visiting.contains(&(side, target.to_owned())) {
                return Err("unresolved_dependency");
            }
            let target_node = self.reference(
                side,
                target,
                if kind == "nominal_complement_de" {
                    Some(&node_id)
                } else {
                    None
                },
            )?;
            self.edge(
                "reference_link",
                address("node", &node_id),
                address("node", &target_node),
                Some(kind),
                None,
            )?;
            self.record(
                side,
                "edge",
                &format!("{node_id}:link"),
                format!("/references/{index}/links/{li}"),
            )?;
        }
        for (_, r) in refs {
            self.dependencies(
                side,
                "reference",
                text(&r["id"])?,
                address("node", &node_id),
            )?;
        }
        let identities = arr(self.docs[side], "identity");
        for (i, e) in identities.iter().enumerate() {
            step()?;
            let left = text(&e["left"])?;
            let right = text(&e["right"])?;
            if !component.contains(left) && !component.contains(right) {
                continue;
            }
            self.record(
                side,
                "edge",
                &format!("{node_id}:identity"),
                format!("/identity/{i}"),
            )?;
            if e["kind"] == "different" {
                let other = if component.contains(left) {
                    right
                } else {
                    left
                };
                let target = self.reference(side, other, None)?;
                if self
                    .nodes
                    .iter()
                    .any(|n| n["node_id"] == target && n["kind"] == "local_nominal_lexical_use")
                {
                    return Err("not_supported");
                }
                self.edge(
                    "identity",
                    address("node", &node_id),
                    address("node", &target),
                    Some("different"),
                    None,
                )?;
            }
        }
        self.visiting.remove(&(side, id.into()));
        Ok(node_id)
    }
    fn term(&mut self, side: usize, t: &Value) -> Fallible<Value> {
        charge(t)?;
        match t["kind"].as_str() {
            Some("ref") => {
                exact(t, &["kind", "id"])?;
                let node = self.reference(side, text(&t["id"])?, None)?;
                if self
                    .nodes
                    .iter()
                    .any(|n| n["node_id"] == node && n["kind"] == "local_nominal_lexical_use")
                {
                    return Err("not_supported");
                }
                Ok(json!({"node_id":node}))
            }
            Some("symbol") => {
                exact(t, &["kind", "id", "evidence"])?;
                Ok(json!({"kind":"symbol","id":t["id"]}))
            }
            Some("text") => {
                exact(t, &["kind", "value", "evidence"])?;
                Ok(json!({"kind":"text","value":t["value"]}))
            }
            Some("quantity") => {
                exact(t, &["kind", "decimal", "unit", "bound", "evidence"])?;
                Ok(
                    json!({"kind":"quantity","decimal":t["decimal"],"unit":t["unit"],"bound":t["bound"]}),
                )
            }
            _ => Err("not_supported"),
        }
    }
    fn atom(&mut self, side: usize, a: &Value, owner: Address) -> Fallible<Value> {
        exact(a, &["predicate", "arguments", "polarity", "evidence"])?;
        let predicate = arr(self.vocabulary, "predicate_rows")
            .iter()
            .find(|p| p["id"] == a["predicate"])
            .ok_or("not_supported")?;
        let mut args = arr(a, "arguments").iter().collect::<Vec<_>>();
        comparisons(args.len())?;
        args.sort_by(|a, b| a["role"].as_str().cmp(&b["role"].as_str()));
        let mut out = Vec::new();
        for arg in args {
            exact(arg, &["role", "value"])?;
            let role = text(&arg["role"])?;
            let typ = predicate["roles"][role].as_str().ok_or("not_supported")?;
            if typ == "ref" && arg["value"]["kind"] != "ref" {
                return Err("not_supported");
            }
            let value = self.term(side, &arg["value"])?;
            if let Some(node) = value["node_id"].as_str() {
                self.edge(
                    "role",
                    owner.clone(),
                    address("node", node),
                    Some(role),
                    None,
                )?;
                self.edge(
                    "dependency",
                    owner.clone(),
                    address("node", node),
                    Some("implicit_operand"),
                    Some("implicit"),
                )?;
            }
            out.push(json!({"role":role,"value":value}));
        }
        Ok(json!({"predicate":a["predicate"],"arguments":out,"polarity":a["polarity"]}))
    }
    fn dependencies(&mut self, side: usize, kind: &str, id: &str, from: Address) -> Fallible<()> {
        if !budget::charge(1, id.len() + kind.len() + 128) {
            return Err(exhaust());
        }
        if !self.closed.insert((side, kind.into(), id.into())) {
            return Ok(());
        }
        for (i, d) in arr(self.docs[side], "dependencies").iter().enumerate() {
            step()?;
            if d["consumer"]["kind"] != kind || d["consumer"]["id"] != id {
                continue;
            }
            let role = text(&d["role"])?;
            if !matches!(
                role,
                "identity"
                    | "guard"
                    | "configuration"
                    | "temporal"
                    | "binding"
                    | "premise"
                    | "normative_target"
                    | "branch_membership"
            ) {
                return Err("not_supported");
            }
            let target = text(&d["requires"]["id"])?;
            let to = match d["requires"]["kind"].as_str() {
                Some("reference") => {
                    // A lexical target is only legal when the owning nominal incidence already established it.
                    let node = if let Some(n) = self.maps[side].get(target) {
                        n.clone()
                    } else {
                        self.reference(side, target, None)?
                    };
                    address("node", &node)
                }
                Some("scope") => address("scope", &self.scope(side, target)?),
                // No second claim or unrepresented qualifier can be discarded as residual.
                Some("claim" | "request" | "qualifier") => return Err("not_supported"),
                _ => return Err("unresolved_dependency"),
            };
            self.edge("dependency", from.clone(), to, Some(role), Some("explicit"))?;
            self.record(
                side,
                "edge",
                &format!("{}:dependency", from.local_id),
                format!("/dependencies/{i}"),
            )?;
        }
        Ok(())
    }
}

fn candidate(
    input: &Value,
    matrix: &[Value],
    cell_index: usize,
    request_index: usize,
    witness_index: usize,
) -> Fallible<Candidate> {
    step()?;
    let cell = matrix.get(cell_index).ok_or("invalid_origin")?;
    let source_leaves = leaves(&input["source"]["raw"]);
    let question_leaves = leaves(&input["question"]["raw"]);
    let source = source_leaves
        .iter()
        .find(|(id, _)| Some(*id) == cell["source_reading_id"].as_str())
        .map(|(_, d)| *d)
        .ok_or("invalid_origin")?;
    let question = question_leaves
        .iter()
        .find(|(id, _)| Some(*id) == cell["question_reading_id"].as_str())
        .map(|(_, d)| *d)
        .ok_or("invalid_origin")?;
    let row = arr(&cell["result"], "requests")
        .get(request_index)
        .ok_or("incomplete_request")?;
    if row["coverage"] != "complete" {
        return Err("incomplete_request");
    }
    if !arr(row, "conflicts").is_empty() {
        return Err("relevant_conflict");
    }
    if !arr(row, "unresolved").is_empty() || !arr(row, "missing_constraints").is_empty() {
        return Err("unresolved_dependency");
    }
    let request = arr(question, "requests")
        .get(request_index)
        .ok_or("invalid_origin")?;
    if row["request_id"] != request["id"] {
        return Err("invalid_origin");
    }
    exact(
        request,
        &[
            "id",
            "scope_id",
            "mode",
            "target",
            "slot",
            "class_filter",
            "unit",
            "basis_requirement",
            "required_answer_extent",
            "normative_scope_filter",
            "qualifier_ids",
            "premise_ids",
            "evidence",
        ],
    )?;
    if request["mode"] != "answer"
        || request["required_answer_extent"] != "one_value"
        || !request["normative_scope_filter"].is_null()
        || !arr(request, "qualifier_ids").is_empty()
        || !arr(request, "premise_ids").is_empty()
    {
        return Err("not_supported");
    }
    exact(
        &request["target"],
        &["atom", "judgment", "status", "qualifier_ids"],
    )?;
    if request["target"]["judgment"] != "fact"
        || request["target"]["status"] != "state"
        || request["target"]["atom"]["polarity"] != "positive"
        || !arr(&request["target"], "qualifier_ids").is_empty()
    {
        return Err("not_supported");
    }
    let predicate = arr(&input["vocabulary"], "predicate_rows")
        .iter()
        .find(|p| p["id"] == request["target"]["atom"]["predicate"])
        .ok_or("not_supported")?;
    let slot = text(&request["slot"])?;
    if predicate["roles"][slot] != "ref"
        || arr(&request["target"]["atom"], "arguments")
            .iter()
            .any(|a| a["role"] == slot)
    {
        return Err("not_supported");
    }
    let w = arr(row, "support")
        .get(witness_index)
        .ok_or("incomplete_witness")?;
    charge(w)?;
    if w["kind"] != "direct"
        || w["request_id"] != request["id"]
        || arr(w, "claim_ids").len() != 1
        || arr(w, "answer_terms").len() != 1
        || !arr(w, "missing_constraints").is_empty()
        || !arr(w, "qualifier_ids").is_empty()
        || !arr(w, "branch_ids").is_empty()
        || !arr(w, "requirement_states").is_empty()
        || !w["normative_scope"].is_null()
    {
        return Err("incomplete_witness");
    }
    let answer = &arr(w, "answer_terms")[0];
    exact(answer, &["kind", "id"])?;
    if answer["kind"] != "ref" {
        return Err("not_supported");
    }
    let claim_index = arr(source, "claims")
        .iter()
        .position(|c| c["id"] == arr(w, "claim_ids")[0])
        .ok_or("invalid_origin")?;
    let claim = &arr(source, "claims")[claim_index];
    if crate::documentary_v3::linguistic_composition::blocked_nodes(source)
        .contains(&("claim".to_owned(), text(&claim["id"])?.to_owned()))
    {
        return Err("not_supported");
    }
    if crate::documentary_v3::lexical_clause_context::blocked_nodes(source)
        .contains(&("claim".to_owned(), text(&claim["id"])?.to_owned()))
    {
        return Err("not_supported");
    }
    charge(claim)?;
    exact(
        claim,
        &["id", "scope_id", "qualifier_ids", "evidence", "body"],
    )?;
    exact(&claim["body"], &["kind", "value"])?;
    let p = &claim["body"]["value"];
    exact(
        p,
        &[
            "atom",
            "judgment",
            "status",
            "basis",
            "normative_scope",
            "qualifier_ids",
        ],
    )?;
    if claim["body"]["kind"] != "proposition"
        || p["judgment"] != "fact"
        || p["status"] != "state"
        || p["basis"] != "description"
        || p["atom"]["polarity"] != "positive"
        || !p["normative_scope"].is_null()
        || !arr(p, "qualifier_ids").is_empty()
        || !arr(claim, "qualifier_ids").is_empty()
    {
        return Err("not_supported");
    }
    if !arr(&p["atom"], "arguments")
        .iter()
        .any(|a| a["role"] == slot && a["value"] == *answer)
    {
        return Err("invalid_origin");
    }
    let mut g = Graph::new(source, question, &input["vocabulary"], BTreeSet::new());
    g.answer = g.component(0, text(&answer["id"])?)?;
    for binding in arr(w, "bindings") {
        if binding["relation"] == "same" {
            let source = text(&binding["source_ref"])?;
            let question = text(&binding["question_ref"])?;
            if !budget::charge(1, source.len() + question.len() + 128) {
                return Err(exhaust());
            }
            g.credited.insert(question.into(), source.into());
        }
    }
    // Establish the bound component once before any occurrence is normalized.
    g.reference(0, text(&answer["id"])?, None)?;
    let atom = g.atom(0, &p["atom"], address("claim", "c0"))?;
    let target_atom = g.atom(
        1,
        &request["target"]["atom"],
        address("request", "request0"),
    )?;
    // Every question participant must retain its already-credited source counterpart.
    for a in arr(&request["target"]["atom"], "arguments") {
        if a["value"]["kind"] == "ref" {
            let b = arr(&p["atom"], "arguments")
                .iter()
                .find(|b| b["role"] == a["role"])
                .ok_or("not_supported")?;
            let sid = text(&b["value"]["id"])?;
            let qid = text(&a["value"]["id"])?;
            if !arr(w, "bindings").iter().any(|b| {
                b["source_ref"] == sid && b["question_ref"] == qid && b["relation"] == "same"
            }) {
                return Err("not_supported");
            }
            if g.maps[0].get(sid) != g.maps[1].get(qid) {
                return Err("not_supported");
            }
        }
    }
    if !g.nodes.iter().any(|n| n["kind"] == "named_anchor") {
        return Err("not_supported");
    }
    let root = g.scope(0, text(&claim["scope_id"])?)?;
    let request_scope = g.scope(1, text(&request["scope_id"])?)?;
    g.edge(
        "scope_owner",
        address("claim", "c0"),
        address("scope", &root),
        None,
        None,
    )?;
    g.edge(
        "scope_owner",
        address("request", "request0"),
        address("scope", &request_scope),
        None,
        None,
    )?;
    g.dependencies(0, "claim", text(&claim["id"])?, address("claim", "c0"))?;
    g.dependencies(
        1,
        "request",
        text(&request["id"])?,
        address("request", "request0"),
    )?;
    g.record(0, "claim", "c0", format!("/claims/{claim_index}"))?;
    g.record(
        1,
        "request",
        "request0",
        format!("/requests/{request_index}"),
    )?;
    let class_filter = if request["class_filter"].is_null() {
        Value::Null
    } else {
        let filter = &request["class_filter"];
        match filter["kind"].as_str() {
            Some("lexical") => {
                exact(filter, &["kind", "class_quote"])?;
                json!({"kind":"lexical","class_quote":{"quote":text(&filter["class_quote"]["quote"])?}})
            }
            Some("semantic") => {
                exact(filter, &["kind", "type_id", "evidence"])?;
                json!({"kind":"semantic","type_id":filter["type_id"]})
            }
            _ => return Err("not_supported"),
        }
    };
    if request["basis_requirement"]["kind"] == "any" {
        exact(&request["basis_requirement"], &["kind"])?;
    } else {
        exact(&request["basis_requirement"], &["kind", "value"])?;
        if request["basis_requirement"]["kind"] != "exact"
            || request["basis_requirement"]["value"] != "description"
        {
            return Err("not_supported");
        }
    }
    let unit = if request["unit"].is_null() {
        Value::Null
    } else {
        exact(&request["unit"], &["symbol", "evidence"])?;
        json!({"symbol":request["unit"]["symbol"]})
    };
    let normalized_request = json!({"scope_id":request_scope,"mode":request["mode"],"target":{"atom":target_atom,"judgment":request["target"]["judgment"],"status":request["target"]["status"],"qualifier_ids":[]},"slot":slot,"class_filter":class_filter,"unit":unit,"basis_requirement":request["basis_requirement"],"required_answer_extent":request["required_answer_extent"],"normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[]});
    // These selectors are closed by the pair validator; reject reference-bearing contexts until competent normalization exists.
    exact(
        &input["pair_scope"],
        &[
            "namespace",
            "configuration",
            "time_selector",
            "closed_world",
        ],
    )?;
    if !arr(&input["pair_scope"], "configuration").is_empty()
        || !input["pair_scope"]["time_selector"].is_null()
    {
        return Err("not_supported");
    }
    let context = copied(&input["pair_scope"])?;
    comparisons(g.nodes.len())?;
    g.nodes
        .sort_by(|a, b| a["node_id"].as_str().cmp(&b["node_id"].as_str()));
    g.edges = sorted(g.edges)?;
    for (i, e) in g.edges.iter_mut().enumerate() {
        e["edge_id"] = json!(format!("e{i}"))
    }
    // Each record origin addresses a real normalized edge, retaining all deduplicated raw incidences.
    for record in &mut g.records {
        if record.record_kind == "edge" {
            let ptr = &record.document_pointers[0];
            let side = if record.document_side == "source" {
                0
            } else {
                1
            };
            let raw = g.docs[side].pointer(ptr).ok_or("invalid_origin")?;
            let matching = g.edges.iter().find(|e| {
                if ptr.starts_with("/dependencies/") {
                    let kind = raw["consumer"]["kind"].as_str().unwrap_or("");
                    let id = raw["consumer"]["id"].as_str().unwrap_or("");
                    let consumer = if kind == "reference" {
                        g.maps[side].get(id).map(String::as_str)
                    } else if kind == "claim" {
                        Some("c0")
                    } else if kind == "request" {
                        Some("request0")
                    } else {
                        Some("s0")
                    };
                    let target_kind = raw["requires"]["kind"].as_str().unwrap_or("");
                    let target_id = raw["requires"]["id"].as_str().unwrap_or("");
                    let target = if target_kind == "reference" {
                        g.maps[side].get(target_id).map(String::as_str)
                    } else {
                        Some("s0")
                    };
                    e["kind"] == "dependency"
                        && e["dependency_origin"] == "explicit"
                        && e["dependency_role"] == raw["role"]
                        && e["from"]["local_id"].as_str() == consumer
                        && e["to"]["local_id"].as_str() == target
                } else if ptr.starts_with("/identity/") {
                    let left = raw["left"]
                        .as_str()
                        .and_then(|id| g.maps[side].get(id))
                        .map(String::as_str);
                    let right = raw["right"]
                        .as_str()
                        .and_then(|id| g.maps[side].get(id))
                        .map(String::as_str);
                    e["kind"] == "identity"
                        && e["identity_kind"] == raw["kind"]
                        && e["from"]["local_id"].as_str() == left
                        && e["to"]["local_id"].as_str() == right
                } else {
                    let target = raw["target"]
                        .as_str()
                        .and_then(|id| g.maps[side].get(id))
                        .map(String::as_str);
                    let owner_id = record.local_record_id.split(':').next().unwrap_or("");
                    e["kind"] == "reference_link"
                        && e["link_kind"] == raw["kind"]
                        && e["from"]["local_id"] == owner_id
                        && e["to"]["local_id"].as_str() == target
                }
            });
            // Same is represented by the node quotient, not an invented global identity edge.
            record.local_record_id = matching
                .and_then(|e| e["edge_id"].as_str())
                .ok_or("invalid_origin")?
                .to_owned();
        }
    }
    comparisons(g.records.len())?;
    g.records.sort_by(|a, b| {
        (
            &a.record_kind,
            &a.local_record_id,
            &a.document_side,
            &a.document_pointers,
        )
            .cmp(&(
                &b.record_kind,
                &b.local_record_id,
                &b.document_side,
                &b.document_pointers,
            ))
    });
    comparisons(g.origins.len())?;
    g.origins.sort_by(|a, b| {
        (&a.node_id, &a.document_side, &a.reference_ids).cmp(&(
            &b.node_id,
            &b.document_side,
            &b.reference_ids,
        ))
    });
    let body = json!({"atom":atom,"judgment":"fact","status":"state","basis":"description","root_scope":root,"nodes":g.nodes,"scopes":g.scopes,"qualifiers":[],"restrictions":[],"context":context,"edges":g.edges});
    charge(&body)?;
    charge(&normalized_request)?;
    let mut anchor_rows = Vec::new();
    for origin in &g.origins {
        if let Some(key) = &origin.canonical_reference_key {
            charge(key)?;
            if !budget::charge(1, origin.node_id.len() + origin.document_side.len() + 128) {
                return Err(exhaust());
            }
            anchor_rows.push(
                json!({"node_id":origin.node_id,"document_side":origin.document_side,"key":key}),
            );
        }
    }
    let anchor_keys = json!(sorted(anchor_rows)?);
    Ok(Candidate {
        request: normalized_request,
        body,
        anchor_keys,
        origin: OriginMap {
            source_reading_id: text(&cell["source_reading_id"])?.into(),
            question_reading_id: text(&cell["question_reading_id"])?.into(),
            matrix_cell_index: cell_index,
            request_index,
            support_witness_index: witness_index,
            node_origins: g.origins,
            record_origins: g.records,
        },
    })
}

fn diagnostic(code: &str, id: &str, cell: Option<&Value>, witness: Option<usize>) -> Value {
    serde_json::to_value(Diagnostic {
        code: code.into(),
        source_reading_id: cell
            .and_then(|c| c["source_reading_id"].as_str())
            .map(str::to_owned),
        question_reading_id: cell
            .and_then(|c| c["question_reading_id"].as_str())
            .map(str::to_owned),
        request_id: id.into(),
        support_witness_index: witness,
        document_pointers: cell.map(|_| vec![]),
    })
    .unwrap()
}
fn equivalent(a: &Candidate, b: &Candidate) -> Fallible<bool> {
    charge(&a.request)?;
    charge(&a.body)?;
    charge(&b.request)?;
    charge(&b.body)?;
    charge(&a.anchor_keys)?;
    charge(&b.anchor_keys)?;
    Ok(a.request == b.request && a.body == b.body && a.anchor_keys == b.anchor_keys)
}
/// Consumer gate: rebuild every proof from this operation's immutable raw leaves and matrix.
/// A matching body without exact binder, request, cell and complete origins is insufficient.
fn consume(
    input: &Value,
    matrix: &[Value],
    request_index: usize,
    conclusion: &Value,
) -> Fallible<()> {
    exact(
        conclusion,
        &[
            "version",
            "conclusion_id",
            "request_id",
            "binder",
            "request",
            "body",
            "identity_proven",
            "origin_maps",
        ],
    )?;
    if conclusion["version"] != "documentary_existential_conclusion_v1"
        || conclusion["identity_proven"] != false
        || conclusion["binder"]
            != json!({"quantifier":"exists","variables":["v0"],"answer_variable":"v0"})
        || !conclusion["conclusion_id"].is_string()
    {
        return Err("invalid_origin");
    }
    let origins = arr(conclusion, "origin_maps");
    if origins.len() != matrix.len() || matrix.is_empty() {
        return Err("invalid_origin");
    }
    let mut seen = BTreeSet::new();
    let mut common_anchor_keys: Option<Value> = None;
    for value in origins {
        charge(value)?;
        let origin: OriginMap =
            serde_json::from_value(copied(value)?).map_err(|_| "invalid_origin")?;
        if origin.request_index != request_index || !seen.insert(origin.matrix_cell_index) {
            return Err("invalid_origin");
        }
        let cell = matrix
            .get(origin.matrix_cell_index)
            .ok_or("invalid_origin")?;
        if arr(&cell["result"], "requests")
            .get(request_index)
            .is_none_or(|r| r["request_id"] != conclusion["request_id"])
        {
            return Err("invalid_origin");
        }
        let candidate = candidate(
            input,
            matrix,
            origin.matrix_cell_index,
            request_index,
            origin.support_witness_index,
        )?;
        if candidate.request != conclusion["request"]
            || candidate.body != conclusion["body"]
            || candidate.origin != origin
        {
            return Err("invalid_origin");
        }
        if common_anchor_keys
            .as_ref()
            .is_some_and(|keys| *keys != candidate.anchor_keys)
        {
            return Err("invalid_origin");
        }
        common_anchor_keys = Some(candidate.anchor_keys);
    }
    Ok(())
}

pub(super) fn upgrade(input: &Value, report: &mut Value, eligible: bool) {
    if !enabled(&input["vocabulary"]) {
        return;
    }
    report["version"] = json!("documentary_reading_pair_report_v2");
    if let Ok(source_text) = copied(&input["source"]["text"]) {
        report["source"]["text"] = source_text;
    }
    if let Ok(question_text) = copied(&input["question"]["text"]) {
        report["question"]["text"] = question_text;
    }
    if let Ok(source_raw) = copied(&input["source"]["raw"]) {
        report["source"]["original_raw"] = source_raw;
    }
    if let Ok(question_raw) = copied(&input["question"]["raw"]) {
        report["question"]["original_raw"] = question_raw;
    }
    let matrix = report["result"]["reading_matrix"].take();
    let cells = matrix.as_array().map(Vec::as_slice).unwrap_or(&[]);
    if let Some(requests) = report["result"]["requests"].as_array_mut() {
        for (ri, row) in requests.iter_mut().enumerate() {
            let id = row["request_id"].as_str().unwrap_or("").to_owned();
            row["existential_conclusions"] = json!([]);
            row["projection_diagnostics"] = json!([]);
            if row["complete_answer"] == true {
                row["completion_kind"] = json!("identity_answer");
                continue;
            }
            row["completion_kind"] = copied(&row["coverage"]).unwrap_or(json!("none"));
            if !eligible || cells.is_empty() {
                row["projection_diagnostics"] = json!([diagnostic(
                    if input["source"]["raw"]["closure"] == "open"
                        || input["question"]["raw"]["closure"] == "open"
                    {
                        "open_reading_set"
                    } else {
                        "ineligible_document"
                    },
                    &id,
                    None,
                    None
                )]);
                continue;
            }
            let mut all: Vec<Vec<Candidate>> = Vec::new();
            let mut diagnostics = Vec::new();
            for (ci, cell) in cells.iter().enumerate() {
                let count = arr(&cell["result"], "requests")
                    .get(ri)
                    .map(|r| arr(r, "support").len())
                    .unwrap_or(0);
                let mut candidates = Vec::new();
                if count == 0 {
                    diagnostics.push(diagnostic("incomplete_request", &id, Some(cell), None))
                }
                for wi in 0..count {
                    match candidate(input, cells, ci, ri, wi) {
                        Ok(c) => candidates.push(c),
                        Err(code) => diagnostics.push(diagnostic(code, &id, Some(cell), Some(wi))),
                    }
                }
                all.push(candidates);
            }
            let mut common = Vec::new();
            if let Some(first) = all.first() {
                for base in first {
                    let mut origins = Vec::new();
                    let mut yes = true;
                    for candidates in &all {
                        let mut found = None;
                        for c in candidates {
                            match equivalent(base, c) {
                                Ok(true) => {
                                    found = Some(c);
                                    break;
                                }
                                Ok(false) => {}
                                Err(code) => {
                                    diagnostics.push(diagnostic(code, &id, None, None));
                                    yes = false;
                                    break;
                                }
                            }
                        }
                        if let Some(c) = found {
                            origins.push(serde_json::to_value(&c.origin).unwrap())
                        } else {
                            yes = false;
                            break;
                        }
                    }
                    if yes {
                        let conclusion = json!({"version":"documentary_existential_conclusion_v1","conclusion_id":"pending","request_id":id,"binder":{"quantifier":"exists","variables":["v0"],"answer_variable":"v0"},"request":base.request,"body":base.body,"identity_proven":false,"origin_maps":origins});
                        if charge(&conclusion).is_ok() {
                            common.push(conclusion)
                        }
                    }
                }
            }
            common = match sorted(common) {
                Ok(sorted) => sorted,
                Err(code) => {
                    diagnostics.push(diagnostic(code, &id, None, None));
                    vec![]
                }
            };
            common.dedup_by(|a, b| a["request"] == b["request"] && a["body"] == b["body"]);
            for (i, c) in common.iter_mut().enumerate() {
                c["conclusion_id"] = json!(format!("conclusion{i}"))
            }
            // Multiple non-equivalent answers remain conditional for one_value; no arbitrary choice.
            if common.len() == 1 {
                match consume(input, cells, ri, &common[0]) {
                    Ok(()) => {
                        row["completion_kind"] = json!("existential_conclusion");
                        row["complete_answer"] = json!(true);
                        row["coverage"] = json!("complete");
                        row["existential_conclusions"] = json!(common)
                    }
                    Err(code) => diagnostics.push(diagnostic(code, &id, None, None)),
                }
            } else if diagnostics.is_empty() {
                diagnostics.push(diagnostic("different_conclusion", &id, None, None))
            }
            row["projection_diagnostics"] = json!(diagnostics);
        }
    }
    report["result"]["reading_matrix"] = matrix;
    let complete = eligible
        && !arr(&report["result"], "requests").is_empty()
        && arr(&report["result"], "requests")
            .iter()
            .all(|r| r["complete_answer"] == true);
    report["result"]["complete_answer"] = json!(complete);
    report["result"]["requires_review"] = json!(!complete);
}

#[cfg(test)]
mod tests {
    use super::*;
    const CONTROL: &str = include_str!(
        "../../../../tests/fixtures/documentary-existential-projection-v1/known-nominal-control.json"
    );
    fn input() -> Value {
        serde_json::from_str(CONTROL).unwrap()
    }
    fn report(input: &Value) -> Value {
        super::super::classify_with_context(input, true)
    }
    fn rehash(input: &mut Value) {
        let hash = super::super::hash(&serde_json::to_vec(&input["vocabulary"]).unwrap());
        for side in ["source", "question"] {
            let raw = &mut input[side]["raw"];
            raw["vocabulary_sha256"] = json!(hash);
            if let Some(readings) = raw.get_mut("readings").and_then(Value::as_array_mut) {
                for r in readings {
                    r["document"]["vocabulary_sha256"] = json!(hash)
                }
            }
        }
    }
    #[test]
    fn common_nominal_body_has_one_binder_and_complete_per_cell_origins() {
        let input = input();
        let result = report(&input);
        assert_eq!(
            result["version"], "documentary_reading_pair_report_v2",
            "{result}"
        );
        let row = &result["result"]["requests"][0];
        assert_eq!(row["completion_kind"], "existential_conclusion", "{row}");
        let c = &row["existential_conclusions"][0];
        assert_eq!(
            arr(&c["body"], "nodes")
                .iter()
                .filter(|n| n["kind"] == "bound_variable")
                .count(),
            1
        );
        assert!(
            arr(&c["body"], "nodes")
                .iter()
                .any(|n| n["kind"] == "local_nominal_lexical_use"
                    && n["descriptor"]["declared_class"]["quote"] == "papel")
        );
        assert_eq!(arr(c, "origin_maps").len(), 2);
        assert_eq!(c["body"]["context"], input["pair_scope"]);
        assert!(
            budget::with_operation(&input["vocabulary"], &input, || consume(
                &input,
                arr(&result["result"], "reading_matrix"),
                0,
                c
            ))
            .unwrap()
            .is_ok()
        );
        let mut corrupt = c.clone();
        corrupt["origin_maps"][0]["matrix_cell_index"] = json!(1);
        assert_eq!(
            budget::with_operation(&input["vocabulary"], &input, || consume(
                &input,
                arr(&result["result"], "reading_matrix"),
                0,
                &corrupt
            ))
            .unwrap(),
            Err("invalid_origin")
        );
    }
    #[test]
    fn absent_feature_retains_historical_v1_and_invalid_profile_does_not_enable_v2() {
        let mut input = input();
        input["vocabulary"]["request_context_profile"]["features"]
            .as_array_mut()
            .unwrap()
            .retain(|f| f != "existential_conclusion_projection");
        rehash(&mut input);
        assert!(!enabled(&input["vocabulary"]));
        let result = report(&input);
        assert_eq!(result["version"], "documentary_reading_pair_report_v1");
        assert!(
            result["result"]["requests"][0]
                .get("completion_kind")
                .is_none()
        );
        input["vocabulary"]["request_context_profile"]["features"] =
            json!(["existential_conclusion_projection", "unknown"]);
        rehash(&mut input);
        assert!(!enabled(&input["vocabulary"]));
        assert_ne!(
            report(&input)["version"],
            "documentary_reading_pair_report_v2"
        );
    }
    #[test]
    fn alpha_renaming_and_semantic_row_permutation_preserve_content() {
        let alpha: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-existential-projection-v1/alpha-control.json"
        ))
        .unwrap();
        let a = report(&input());
        let b = report(&alpha);
        assert_eq!(b["result"]["complete_answer"], true, "{b}");
        let a = &a["result"]["requests"][0]["existential_conclusions"][0];
        let b = &b["result"]["requests"][0]["existential_conclusions"][0];
        assert_eq!(a["body"], b["body"]);
        assert_eq!(a["request"], b["request"]);
    }
    #[test]
    fn shared_budget_exhaustion_cannot_publish_a_favorable_subset() {
        let input = input();
        let v = &input["vocabulary"];
        assert!(
            budget::with_operation(v, &input, || {
                assert!(!budget::charge(usize::MAX, 0));
                let result = report(&input);
                assert_eq!(result["result"]["complete_answer"], false);
            })
            .is_err()
        );
    }
    #[test]
    fn repeated_answer_incidence_is_correlated_and_lexical_tail_is_never_physical() {
        let input = input();
        let validated = report(&input);
        assert_eq!(validated["source"]["status"], "complete");
        assert_eq!(validated["question"]["status"], "complete");
        budget::with_operation(&input["vocabulary"], &input, || {
            let source = &input["source"]["raw"]["readings"][0]["document"];
            let question = &input["question"]["raw"];
            let mut graph = Graph::new(source, question, &input["vocabulary"], BTreeSet::new());
            graph.answer = graph.component(0, "tray").unwrap();
            let first = graph.term(0, &json!({"kind":"ref","id":"tray"})).unwrap();
            let repeated = graph.term(0, &json!({"kind":"ref","id":"tray"})).unwrap();
            let distinct = graph.term(0, &json!({"kind":"ref","id":"senda"})).unwrap();
            assert_eq!(first, repeated);
            assert_ne!(first, distinct);
            assert_eq!(first["node_id"], "n0");
            assert_eq!(
                graph.term(0, &json!({"kind":"ref","id":"paper"})),
                Err("not_supported")
            );
        })
        .unwrap();
    }
    #[test]
    fn linguistic_composition_keeps_independent_existential_conclusion_and_blocks_its_consumer() {
        let mut i = input();
        i["vocabulary"]["request_context_profile"]["features"]
            .as_array_mut()
            .unwrap()
            .push(json!("linguistic_composition_until_measurement_v1"));
        assert!(arr(&i["vocabulary"], "fixed_attribute_rows").contains(&json!("duration")));
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-until-composition-v1/source.json"
        ))
        .unwrap();
        let added_text =
            include_str!("../../../../tests/fixtures/documentary-until-composition-v1/source.txt");
        let original_text = i["source"]["text"].as_str().unwrap().to_owned();
        let offset = original_text.len() + 1;
        let expanded = format!("{original_text} {added_text}");
        let text_hash = super::super::hash(expanded.as_bytes());
        fn shift(v: &mut Value, offset: usize) {
            if let Some(m) = v.as_object_mut() {
                if m.contains_key("start_byte") && m.contains_key("end_byte") {
                    for key in ["start_byte", "end_byte"] {
                        m[key] = json!(m[key].as_u64().unwrap() + offset as u64);
                    }
                }
                for child in m.values_mut() {
                    shift(child, offset);
                }
            } else if let Some(a) = v.as_array_mut() {
                for child in a {
                    shift(child, offset);
                }
            }
        }
        let mut claim = fixture["claims"][0].clone();
        let mut reference = fixture["references"][0].clone();
        shift(&mut claim, offset);
        shift(&mut reference, offset);
        claim["id"] = json!("until_composition");
        claim["body"]["core"]["operand"]["object"]["owner"]["term"]["id"] = json!("until_player");
        reference["id"] = json!("until_player");
        i["source"]["text"] = json!(expanded);
        i["source"]["raw"]["text_sha256"] = json!(text_hash);
        for reading in i["source"]["raw"]["readings"].as_array_mut().unwrap() {
            let doc = &mut reading["document"];
            doc["text_sha256"] = json!(text_hash);
            doc["claims"].as_array_mut().unwrap().push(claim.clone());
            doc["references"]
                .as_array_mut()
                .unwrap()
                .push(reference.clone());
            doc["dependencies"].as_array_mut().unwrap().extend([
                json!({"consumer":{"kind":"reference","id":"until_player"},"requires":{"kind":"scope","id":"root"},"role":"binding"}),
                json!({"consumer":{"kind":"claim","id":"until_composition"},"requires":{"kind":"scope","id":"root"},"role":"binding"}),
                json!({"consumer":{"kind":"claim","id":"until_composition"},"requires":{"kind":"reference","id":"until_player"},"role":"identity"})
            ]);
            doc["coverage"].as_array_mut().unwrap().push(json!({"span":claim["evidence"][0],"state":"represented","owner":{"kind":"claim","id":"until_composition"},"issue_id":null}));
        }
        rehash(&mut i);
        let r = report(&i);
        assert_eq!(r["source"]["status"], "complete", "{r}");
        assert_eq!(r["question"]["status"], "complete", "{r}");
        let row = &r["result"]["requests"][0];
        assert_eq!(row["completion_kind"], "existential_conclusion", "{r}");
        assert!(!arr(row, "existential_conclusions").is_empty());
        assert!(
            !row["existential_conclusions"][0]
                .to_string()
                .contains("until_composition")
        );
        for reading in i["source"]["raw"]["readings"].as_array_mut().unwrap() {
            reading["document"]["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":"possession"},"requires":{"kind":"claim","id":"until_composition"},"role":"premise"}));
        }
        let blocked = report(&i);
        assert_eq!(blocked["source"]["status"], "complete", "{blocked}");
        assert_eq!(blocked["question"]["status"], "complete", "{blocked}");
        assert!(
            arr(&blocked["result"]["requests"][0], "existential_conclusions").is_empty(),
            "{blocked}"
        );
        assert_ne!(
            blocked["result"]["requests"][0]["completion_kind"],
            "existential_conclusion"
        );
    }
}

#[cfg(test)]
mod operation_cap_tests {
    use super::*;
    #[test]
    fn local_sort_cap_exhausts_entire_operation() {
        let vocabulary = json!({"request_context_profile":{"version":"documentary_request_context_v1","features":["existential_conclusion_projection"]}});
        let result = budget::with_operation(&vocabulary, &json!({}), || {
            assert_eq!(comparisons(4097), Err("limit"));
            assert!(budget::exhausted());
            assert!(!budget::charge(0, 0));
        });
        assert_eq!(result, Err("request_context_limit".into()));
    }
}
