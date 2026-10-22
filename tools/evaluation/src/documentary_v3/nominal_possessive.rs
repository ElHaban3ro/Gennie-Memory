//! An open contextual determiner is a use, never an entity or an identity key.
use serde_json::Value;
use std::collections::BTreeSet;
pub(crate) const KIND: &str = "nominal_possessive";
pub(crate) type Nodes = BTreeSet<(String, String)>;
fn rows<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v[k].as_array().map(Vec::as_slice).unwrap_or(&[])
}
pub(crate) fn has(out: &Nodes, kind: &str, id: &str) -> bool {
    for (k, i) in out {
        if !super::context_budget::charge(1, 0) {
            return false;
        }
        if k == kind && i == id {
            return true;
        }
    }
    false
}
fn put(out: &mut Nodes, kind: &str, id: &str) -> bool {
    if has(out, kind, id) {
        return true;
    }
    if !super::context_budget::charge(1, kind.len().saturating_add(id.len()).saturating_add(128)) {
        return false;
    }
    out.insert((kind.to_owned(), id.to_owned()));
    true
}
/// Same aliases are traversed locally only; no lexical text becomes an anchor.
pub(crate) fn component(doc: &Value, id: &str) -> Option<BTreeSet<String>> {
    if !super::context_budget::possessive_enabled() {
        return None;
    }
    let mut nodes = Nodes::new();
    if !put(&mut nodes, "reference", id) {
        return None;
    }
    loop {
        let before = nodes.len();
        for e in rows(doc, "identity") {
            if !super::context_budget::charge(1, 0) {
                return None;
            }
            if e["kind"] == "same" {
                let (a, b) = (e["left"].as_str()?, e["right"].as_str()?);
                if has(&nodes, "reference", a) && !put(&mut nodes, "reference", b) {
                    return None;
                }
                if has(&nodes, "reference", b) && !put(&mut nodes, "reference", a) {
                    return None;
                }
            }
        }
        for r in rows(doc, "references") {
            if !super::context_budget::charge(1, 0) {
                return None;
            }
            if r["anaphora"]["status"] == "asserted"
                && rows(&r["anaphora"], "antecedent_ids").len() == 1
            {
                let (a, b) = (
                    r["id"].as_str()?,
                    r["anaphora"]["antecedent_ids"][0].as_str()?,
                );
                if has(&nodes, "reference", a) && !put(&mut nodes, "reference", b) {
                    return None;
                }
                if has(&nodes, "reference", b) && !put(&mut nodes, "reference", a) {
                    return None;
                }
            }
        }
        if before == nodes.len() {
            break;
        }
    }
    // Reserve the second representation before copying identifiers.
    if !super::context_budget::charge(nodes.len(), nodes.iter().map(|(_, id)| id.len() + 64).sum())
    {
        return None;
    }
    Some(nodes.into_iter().map(|(_, id)| id).collect())
}
fn nested_edges(v: &Value, out: &mut Nodes) {
    if !super::context_budget::charge(1, 0) {
        return;
    }
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                if matches!(k.as_str(), "claim_ids" | "premise_ids") {
                    for id in x.as_array().into_iter().flatten().filter_map(Value::as_str) {
                        if !put(out, "claim", id) {
                            return;
                        }
                    }
                }
                nested_edges(x, out);
            }
        }
        Value::Array(xs) => {
            for x in xs {
                nested_edges(x, out);
            }
        }
        _ => {}
    }
}
/// Quarantine the entire effective consumer closure before any inference route.
pub(crate) fn blocked_nodes(doc: &Value) -> Nodes {
    let mut blocked = Nodes::new();
    if !super::context_budget::possessive_enabled() {
        return blocked;
    }
    for r in rows(doc, "references") {
        if !super::context_budget::charge(1, 0) {
            return blocked;
        }
        if r["denotation"] == "lexical_use" || rows(r, "links").iter().any(|l| l["kind"] == KIND) {
            if !put(&mut blocked, "reference", r["id"].as_str().unwrap_or("")) {
                return blocked;
            }
        }
    }
    if blocked.is_empty() {
        return blocked;
    }
    let mut edges = Vec::new();
    for (table, kind) in [
        ("references", "reference"),
        ("claims", "claim"),
        ("requests", "request"),
        ("qualifiers", "qualifier"),
        ("scopes", "scope"),
    ] {
        for owner in rows(doc, table) {
            let id = owner["id"].as_str().unwrap_or("");
            // The shared collector allocates owned edges: reserve its bounded
            // full owner/identity input before invoking it.
            let bytes = super::context_budget::serialized_bytes(owner)
                .unwrap_or(usize::MAX)
                .saturating_add(
                    super::context_budget::serialized_bytes(&doc["identity"]).unwrap_or(usize::MAX),
                )
                .saturating_mul(16);
            if !super::context_budget::charge(1, bytes) {
                return blocked;
            }
            let mut required = Nodes::new();
            for (k, i) in super::validation::effective_context_dependencies(doc, kind, id) {
                if !put(&mut required, &k, &i) {
                    return blocked;
                }
            }
            nested_edges(owner, &mut required);
            for d in rows(doc, "dependencies") {
                if !super::context_budget::charge(1, 0) {
                    return blocked;
                }
                if d["consumer"]["kind"] == kind && d["consumer"]["id"] == id {
                    if !put(
                        &mut required,
                        d["requires"]["kind"].as_str().unwrap_or(""),
                        d["requires"]["id"].as_str().unwrap_or(""),
                    ) {
                        return blocked;
                    }
                }
            }
            if kind == "reference" {
                if let Some(aliases) = component(doc, id) {
                    for alias in aliases {
                        if !put(&mut required, "reference", &alias) {
                            return blocked;
                        }
                    }
                }
            }
            for node in required {
                if !super::context_budget::charge(1, id.len() + kind.len() + 128) {
                    return blocked;
                }
                edges.push(((kind.to_owned(), id.to_owned()), node));
            }
        }
    }
    loop {
        let before = blocked.len();
        for (consumer, required) in &edges {
            if !super::context_budget::charge(1, 0) {
                return blocked;
            }
            if blocked.contains(required) && !put(&mut blocked, &consumer.0, &consumer.1) {
                return blocked;
            }
        }
        if before == blocked.len() {
            return blocked;
        }
    }
}
pub(crate) fn reference_blocked(doc: &Value, id: &str) -> bool {
    super::context_budget::possessive_enabled() && has(&blocked_nodes(doc), "reference", id)
}
