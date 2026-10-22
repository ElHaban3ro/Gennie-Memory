//! Lexical-use constraints are compared only under their owning nominal link.
//! This key is deliberately not a reference identity or an existence witness.
use serde_json::{Value, json};
use std::collections::BTreeSet;
pub(crate) const KIND: &str = "nominal_complement_de";
fn rows<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn store_id(ids: &mut BTreeSet<String>, id: &str) -> Option<()> {
    if ids.contains(id) {
        return Some(());
    }
    if !super::context_budget::charge(1, id.len().saturating_add(64)) {
        return None;
    }
    ids.insert(id.to_owned());
    Some(())
}
fn store_ref<'a>(ids: &mut BTreeSet<&'a str>, id: &'a str) -> Option<()> {
    if ids.contains(id) {
        return Some(());
    }
    if !super::context_budget::charge(1, 64) {
        return None;
    }
    ids.insert(id);
    Some(())
}
pub(super) fn component(doc: &Value, rid: &str) -> Option<BTreeSet<String>> {
    if !super::context_budget::nominal_enabled() {
        return None;
    }
    let mut ids = BTreeSet::new();
    store_id(&mut ids, rid)?;
    loop {
        let before = ids.len();
        for edge in rows(doc, "identity") {
            if !super::context_budget::charge(1, 0) {
                return None;
            }
            if edge["kind"] == "same" {
                let a = edge["left"].as_str()?;
                let b = edge["right"].as_str()?;
                if ids.contains(a) {
                    store_id(&mut ids, b)?;
                }
                if ids.contains(b) {
                    store_id(&mut ids, a)?;
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
                let alias = r["id"].as_str()?;
                let antecedent = r["anaphora"]["antecedent_ids"][0].as_str()?;
                if ids.contains(alias) {
                    store_id(&mut ids, antecedent)?;
                }
                if ids.contains(antecedent) {
                    store_id(&mut ids, alias)?;
                }
            }
        }
        if before == ids.len() {
            break;
        }
    }
    for edge in rows(doc, "identity") {
        if !super::context_budget::charge(1, 0) {
            return None;
        }
        if edge["kind"] == "different"
            && ids.contains(edge["left"].as_str()?)
            && ids.contains(edge["right"].as_str()?)
        {
            return None;
        }
    }
    Some(ids)
}
pub(super) fn targets<'a>(doc: &'a Value, rid: &str) -> Option<Vec<&'a str>> {
    if !super::context_budget::nominal_enabled() {
        return Some(Vec::new());
    }
    let ids = component(doc, rid)?;
    let mut targets = Vec::new();
    let mut seen = BTreeSet::new();
    for r in rows(doc, "references") {
        if !super::context_budget::charge(1, 0) {
            return None;
        }
        if !ids.contains(r["id"].as_str()?) {
            continue;
        }
        for link in rows(r, "links") {
            if !super::context_budget::charge(1, 0) {
                return None;
            }
            if link["kind"] == KIND {
                let target = link["target"].as_str()?;
                let aliases = component(doc, target)?;
                if !seen.contains(&aliases) {
                    if !super::context_budget::charge(1, 128) {
                        return None;
                    }
                    seen.insert(aliases);
                    targets.push(target);
                }
            }
        }
    }
    Some(targets)
}
pub(crate) fn descriptor_key(doc: &Value, rid: &str) -> Option<Value> {
    if !super::context_budget::nominal_enabled() {
        return None;
    }
    let ids = component(doc, rid)?;
    let mut classes = BTreeSet::new();
    let mut types = BTreeSet::new();
    let mut scopes = BTreeSet::new();
    let mut count = 0;
    for r in rows(doc, "references") {
        if !super::context_budget::charge(1, 0) {
            return None;
        }
        if !ids.contains(r["id"].as_str()?) {
            continue;
        }
        count += 1;
        if r["denotation"] != "entity"
            || !r["name"].is_null()
            || !r["binding_scope_id"].is_null()
            || !r["anaphora"].is_null()
            || !rows(r, "links").is_empty()
        {
            return None;
        }
        let class = r["declared_class"]["quote"].as_str()?;
        if class.trim().is_empty() || !super::context_budget::charge(0, class.len()) {
            return None;
        }
        store_ref(&mut classes, class)?;
        if !r["semantic_type"].is_null() {
            store_ref(&mut types, r["semantic_type"]["type_id"].as_str()?)?;
        }
        store_ref(&mut scopes, r["scope_id"].as_str()?)?;
    }
    if count != ids.len() || classes.len() != 1 || types.len() > 1 || scopes.len() != 1 {
        return None;
    }
    // A lexical-use descriptor is local to exactly one owner component.
    let mut owners = BTreeSet::new();
    for r in rows(doc, "references") {
        if !super::context_budget::charge(1, 0) {
            return None;
        }
        for link in rows(r, "links") {
            if !super::context_budget::charge(1, 0) {
                return None;
            }
            if link["kind"] == KIND && ids.contains(link["target"].as_str()?) {
                if r["scope_id"].as_str() != scopes.first().copied() {
                    return None;
                }
                let owner = component(doc, r["id"].as_str()?)?;
                if !owners.contains(&owner) {
                    if !super::context_budget::charge(1, 128) {
                        return None;
                    }
                    owners.insert(owner);
                }
            }
        }
    }
    if owners.len() != 1 || owners.first()?.iter().any(|id| ids.contains(id)) {
        return None;
    }
    for d in rows(doc, "dependencies") {
        if !super::context_budget::charge(1, 0) {
            return None;
        }
        if d["consumer"]["kind"] == "reference"
            && ids.contains(d["consumer"]["id"].as_str()?)
            && (d["requires"]["kind"] != "scope"
                || d["requires"]["id"].as_str() != scopes.first().copied())
        {
            return None;
        }
    }
    if !super::context_budget::charge(
        1,
        classes
            .first()?
            .len()
            .saturating_add(types.first().map_or(0, |t| t.len()))
            .saturating_add(160),
    ) {
        return None;
    }
    Some(
        json!({"kind":"local_nominal_lexical_use","class":classes.first()?,"semantic_type":types.first(),"denotation":"entity"}),
    )
}
pub(super) fn is_tail(doc: &Value, rid: &str) -> bool {
    if !super::context_budget::nominal_enabled() {
        return false;
    }
    let Some(ids) = component(doc, rid) else {
        return false;
    };
    rows(doc, "references").iter().any(|r| {
        super::context_budget::charge(1, 0)
            && rows(r, "links").iter().any(|l| {
                super::context_budget::charge(1, 0)
                    && l["kind"] == KIND
                    && l["target"].as_str().is_some_and(|t| ids.contains(t))
            })
    })
}
pub(super) fn lexical_metadata(doc: &Value, claim: &Value) -> bool {
    if !super::context_budget::nominal_enabled() {
        return false;
    }
    let p = &claim["body"]["value"];
    let args = rows(&p["atom"], "arguments");
    claim["body"]["kind"] == "proposition"
        && p["atom"]["predicate"] == "property"
        && p["atom"]["polarity"] == "positive"
        && p["judgment"] == "fact"
        && p["status"] == "state"
        && p["basis"] == "description"
        && p["normative_scope"].is_null()
        && args.len() == 3
        && args.iter().any(|a| {
            a["role"] == "subject"
                && a["value"]["kind"] == "ref"
                && a["value"]["id"].as_str().is_some_and(|id| is_tail(doc, id))
        })
        && args.iter().any(|a| {
            a["role"] == "attribute"
                && a["value"]["kind"] == "symbol"
                && a["value"]["id"] == "nominal_qualitative_descriptor"
        })
        && args.iter().any(|a| {
            a["role"] == "value"
                && a["value"]["kind"] == "text"
                && a["value"]["value"]
                    .as_str()
                    .is_some_and(|v| !v.trim().is_empty())
        })
}
/// Quarantine every effective consumer so lexical metadata cannot discharge a
/// global rule, guard or identity through a second inference path.
pub(super) fn blocked_nodes(doc: &Value) -> BTreeSet<(String, String)> {
    if !super::context_budget::nominal_enabled() {
        return BTreeSet::new();
    }
    let mut blocked = BTreeSet::new();
    for c in rows(doc, "claims") {
        if lexical_metadata(doc, c) {
            let id = c["id"].as_str().unwrap_or("");
            if !super::context_budget::charge(1, id.len().saturating_add(128)) {
                return blocked;
            }
            blocked.insert(("claim".to_owned(), id.to_owned()));
        }
    }
    if blocked.is_empty() {
        return blocked;
    }
    let mut edges = Vec::new();
    for d in rows(doc, "dependencies") {
        let bytes = [
            &d["consumer"]["kind"],
            &d["consumer"]["id"],
            &d["requires"]["kind"],
            &d["requires"]["id"],
        ]
        .into_iter()
        .fold(128usize, |n, v| {
            n.saturating_add(v.as_str().map_or(0, str::len))
        });
        if !super::context_budget::charge(1, bytes) {
            return blocked;
        }
        edges.push((
            (
                d["consumer"]["kind"].as_str().unwrap_or("").to_owned(),
                d["consumer"]["id"].as_str().unwrap_or("").to_owned(),
            ),
            (
                d["requires"]["kind"].as_str().unwrap_or("").to_owned(),
                d["requires"]["id"].as_str().unwrap_or("").to_owned(),
            ),
        ));
    }
    for (table, kind) in [
        ("claims", "claim"),
        ("qualifiers", "qualifier"),
        ("references", "reference"),
        ("scopes", "scope"),
    ] {
        for owner in rows(doc, table) {
            let id = owner["id"].as_str().unwrap_or("");
            fn nested_claims(value: &Value, ids: &mut BTreeSet<String>) {
                if !super::context_budget::charge(1, 0) {
                    return;
                }
                match value {
                    Value::Object(m) => {
                        for (key, child) in m {
                            if matches!(key.as_str(), "claim_ids" | "premise_ids") {
                                for id in child
                                    .as_array()
                                    .into_iter()
                                    .flatten()
                                    .filter_map(Value::as_str)
                                {
                                    if store_id(ids, id).is_none() {
                                        return;
                                    }
                                }
                            }
                            nested_claims(child, ids);
                        }
                    }
                    Value::Array(a) => {
                        for child in a {
                            nested_claims(child, ids);
                        }
                    }
                    _ => {}
                }
            }
            let mut nested = BTreeSet::new();
            nested_claims(owner, &mut nested);
            for claim in nested {
                if !super::context_budget::charge(
                    1,
                    id.len().saturating_add(kind.len()).saturating_add(128),
                ) {
                    return blocked;
                }
                edges.push((
                    (kind.to_owned(), id.to_owned()),
                    ("claim".to_owned(), claim),
                ));
            }
            let owned_bytes = super::context_budget::serialized_bytes(owner)
                .unwrap_or(usize::MAX)
                .saturating_add(
                    super::context_budget::serialized_bytes(&doc["identity"]).unwrap_or(usize::MAX),
                )
                .saturating_mul(16);
            if !super::context_budget::charge(1, owned_bytes) {
                return blocked;
            }
            for required in super::validation::effective_context_dependencies(doc, kind, id) {
                if !super::context_budget::charge(
                    1,
                    id.len().saturating_add(kind.len()).saturating_add(128),
                ) {
                    return blocked;
                }
                edges.push(((kind.to_owned(), id.to_owned()), required));
            }
        }
    }
    for r in rows(doc, "references") {
        let id = r["id"].as_str().unwrap_or("");
        if let Some(aliases) = component(doc, id) {
            for alias in aliases {
                if !super::context_budget::charge(1, id.len().saturating_add(128)) {
                    return blocked;
                }
                edges.push((
                    ("reference".into(), id.to_owned()),
                    ("reference".into(), alias),
                ));
            }
        }
    }
    loop {
        let before = blocked.len();
        for (consumer, required) in &edges {
            if !super::context_budget::charge(1, 0) {
                return blocked;
            }
            if blocked.contains(required) && !blocked.contains(consumer) {
                if !super::context_budget::charge(
                    1,
                    consumer
                        .0
                        .len()
                        .saturating_add(consumer.1.len())
                        .saturating_add(128),
                ) {
                    return blocked;
                }
                blocked.insert(consumer.clone());
            }
        }
        if before == blocked.len() {
            return blocked;
        }
    }
}
pub(super) fn local_metadata_request(doc: &Value, r: &Value) -> bool {
    if !super::context_budget::nominal_enabled() {
        return false;
    }
    let p = &r["target"];
    let args = rows(&p["atom"], "arguments");
    r["mode"] == "answer"
        && r["slot"] == "value"
        && p["atom"]["predicate"] == "property"
        && p["atom"]["polarity"] == "positive"
        && p["judgment"] == "fact"
        && p["status"] == "state"
        && args.iter().any(|a| {
            a["role"] == "subject"
                && a["value"]["kind"] == "ref"
                && a["value"]["id"].as_str().is_some_and(|id| is_tail(doc, id))
        })
        && args.iter().any(|a| {
            a["role"] == "attribute"
                && a["value"]["kind"] == "symbol"
                && a["value"]["id"] == "nominal_qualitative_descriptor"
        })
}
