//! Local selection of an intact, positively qualified source occurrence.
//! The separately supplied runner manifest authorizes a review, never linguistic truth.
use super::{context_budget as budget, request_context_profile as profile};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub(crate) const KIND: &str = "lexical_clause_context";
pub(crate) type Nodes = BTreeSet<(String, String)>;
fn rows<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn step() -> bool {
    budget::charge(1, 0)
}
fn walk(v: &Value, depth: usize) -> bool {
    if depth > 64 || !step() {
        budget::charge(usize::MAX, 0);
        return false;
    }
    match v {
        Value::Object(m) => m.values().all(|x| walk(x, depth + 1)),
        Value::Array(a) => a.iter().all(|x| walk(x, depth + 1)),
        _ => true,
    }
}
fn reserve(v: &Value, copies: usize) -> bool {
    walk(v, 0)
        && budget::serialized_bytes(v).is_some_and(|n| budget::charge(1, n.saturating_mul(copies)))
}
fn closed(v: &Value, keys: &[&str]) -> bool {
    step()
        && v.as_object().is_some_and(|m| {
            m.len() == keys.len() && keys.iter().all(|k| step() && m.contains_key(*k))
        })
}
fn id(v: &Value) -> bool {
    v.as_str()
        .is_some_and(|s| !s.trim().is_empty() && s.len() <= 128)
}
fn sha(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}
pub(crate) fn digest(v: &Value) -> String {
    if !reserve(v, 1) {
        return String::new();
    }
    Sha256::digest(serde_json::to_vec(v).unwrap_or_default())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn span(v: &Value) -> Option<(usize, usize)> {
    Some((
        usize::try_from(v["start_byte"].as_u64()?).ok()?,
        usize::try_from(v["end_byte"].as_u64()?).ok()?,
    ))
}
fn contains(a: &Value, b: &Value) -> bool {
    span(a)
        .zip(span(b))
        .is_some_and(|((a, b), (c, d))| a <= c && c < d && d <= b)
}
fn evidence(v: &Value, text: &str) -> bool {
    closed(v, &["start_byte", "end_byte", "quote"])
        && span(v).is_some_and(|(s, e)| {
            s < e && e <= u32::MAX as usize && text.get(s..e) == v["quote"].as_str()
        })
}
fn evidences(v: &Value, text: &str) -> bool {
    v.as_array()
        .is_some_and(|a| !a.is_empty() && a.len() <= 32 && a.iter().all(|e| evidence(e, text)))
}
pub(crate) fn carries(owner: &Value) -> bool {
    rows(owner, "context_bindings")
        .iter()
        .any(|c| step() && c["kind"] == KIND)
}
pub(crate) fn lexical_evidence_valid(text: &str, context: &Value) -> bool {
    let p = &context["preposition"];
    let c = &context["complement_evidence"];
    let phrase = &context["phrase_evidence"];
    if !p["quote"].as_str().is_some_and(|s| {
        budget::charge(s.len(), 0) && !s.is_empty() && s.chars().all(char::is_alphabetic)
    }) {
        return false;
    }
    span(p)
        .zip(span(c))
        .zip(span(phrase))
        .is_some_and(|(((a, b), (c, d)), (e, f))| {
            a == e
                && d == f
                && b < c
                && text.get(b..c).is_some_and(|gap| {
                    budget::charge(gap.len(), 0) && gap.chars().all(char::is_whitespace)
                })
        })
        && contains(&context["external_clause_evidence"], phrase)
}
fn table(kind: &str) -> Option<&'static str> {
    match kind {
        "claim" => Some("claims"),
        "request" => Some("requests"),
        "reference" => Some("references"),
        "qualifier" => Some("qualifiers"),
        "scope" => Some("scopes"),
        _ => None,
    }
}
fn node<'a>(doc: &'a Value, kind: &str, local: &str) -> Option<&'a Value> {
    rows(doc, table(kind)?)
        .iter()
        .find(|v| step() && v["id"] == local)
}
fn put(out: &mut Nodes, kind: &str, local: &str) -> bool {
    if !budget::charge(
        out.len().saturating_add(1),
        kind.len().saturating_add(local.len()).saturating_add(128),
    ) {
        return false;
    }
    out.insert((kind.to_owned(), local.to_owned()));
    true
}
fn nested(v: &Value, out: &mut Nodes) {
    if !step() {
        return;
    }
    match v {
        Value::Object(m) => {
            for (key, x) in m {
                if matches!(key.as_str(), "claim_ids" | "premise_ids") {
                    for x in x.as_array().into_iter().flatten() {
                        if let Some(s) = x.as_str() {
                            put(out, "claim", s);
                        }
                    }
                }
                nested(x, out);
            }
        }
        Value::Array(a) => {
            for x in a {
                nested(x, out);
            }
        }
        _ => {}
    }
}
fn requirements(doc: &Value, kind: &str, local: &str) -> Nodes {
    let mut out = Nodes::new();
    let Some(owner) = node(doc, kind, local) else {
        return out;
    };
    if !reserve(owner, 16) || !reserve(&doc["identity"], 16) {
        return out;
    }
    for (k, i) in super::validation::effective_context_dependencies(doc, kind, local) {
        put(&mut out, &k, &i);
    }
    nested(owner, &mut out);
    for dep in rows(doc, "dependencies") {
        if !step() {
            return out;
        }
        if dep["consumer"]["kind"] == kind && dep["consumer"]["id"] == local {
            if let (Some(k), Some(i)) = (
                dep["requires"]["kind"].as_str(),
                dep["requires"]["id"].as_str(),
            ) {
                put(&mut out, k, i);
            }
        }
    }
    // Same and asserted singular anaphora are local edges, never cross-document IDs.
    if kind == "reference" {
        for edge in rows(doc, "identity") {
            if !step() {
                return out;
            }
            if edge["kind"] == "same" {
                if edge["left"] == local {
                    if let Some(i) = edge["right"].as_str() {
                        put(&mut out, "reference", i);
                    }
                }
                if edge["right"] == local {
                    if let Some(i) = edge["left"].as_str() {
                        put(&mut out, "reference", i);
                    }
                }
            }
        }
        for r in rows(doc, "references") {
            if !step() {
                return out;
            }
            if r["anaphora"]["status"] == "asserted"
                && rows(&r["anaphora"], "antecedent_ids").len() == 1
            {
                let a = r["id"].as_str().unwrap_or("");
                let b = r["anaphora"]["antecedent_ids"][0].as_str().unwrap_or("");
                if a == local {
                    put(&mut out, "reference", b);
                }
                if b == local {
                    put(&mut out, "reference", a);
                }
            }
        }
    }
    out
}
pub(crate) fn closure(doc: &Value, kind: &str, local: &str) -> Nodes {
    let mut out = Nodes::new();
    if !put(&mut out, kind, local) {
        return out;
    }
    loop {
        let before = out.len();
        let mut add = Nodes::new();
        for (k, i) in &out {
            for (k, i) in requirements(doc, k, i) {
                put(&mut add, &k, &i);
            }
        }
        for (k, i) in add {
            put(&mut out, &k, &i);
        }
        if before == out.len() || budget::exhausted() {
            return out;
        }
    }
}
/// Reverse the full explicit/implicit graph before any ordinary inference.
pub(crate) fn blocked_nodes(doc: &Value) -> Nodes {
    let mut blocked = Nodes::new();
    if !budget::lexical_enabled() {
        return blocked;
    }
    for c in rows(doc, "claims") {
        if carries(c) {
            put(&mut blocked, "claim", c["id"].as_str().unwrap_or(""));
        }
    }
    consumer_closure(doc, blocked)
}
/// Shared graph machinery; the caller owns feature admission and semantic seeds.
pub(crate) fn consumer_closure(doc: &Value, mut blocked: Nodes) -> Nodes {
    if blocked.is_empty() {
        return blocked;
    }
    let mut edges = Vec::new();
    for (tab, kind) in [
        ("claims", "claim"),
        ("requests", "request"),
        ("references", "reference"),
        ("qualifiers", "qualifier"),
        ("scopes", "scope"),
    ] {
        for n in rows(doc, tab) {
            if !step() {
                return blocked;
            }
            let local = n["id"].as_str().unwrap_or("");
            for required in requirements(doc, kind, local) {
                if !budget::charge(1, local.len() + kind.len() + 256) {
                    return blocked;
                }
                edges.push(((kind.to_owned(), local.to_owned()), required));
            }
        }
    }
    loop {
        let before = blocked.len();
        for (consumer, required) in &edges {
            if !budget::charge(blocked.len().saturating_add(1), 0) {
                return blocked;
            }
            if blocked.contains(required) {
                put(&mut blocked, &consumer.0, &consumer.1);
            }
        }
        if before == blocked.len() || budget::exhausted() {
            return blocked;
        }
    }
}
fn simple_nominal(r: &Value) -> bool {
    !r.is_null()
        && r["denotation"] == "entity"
        && !r["mention"].is_null()
        && r["binding_scope_id"].is_null()
        && r["anaphora"].is_null()
}
pub(crate) fn context_document_valid(doc: &Value, owner: &Value, context: &Value) -> bool {
    let Some(local) = context["complement"]["id"].as_str() else {
        return false;
    };
    let Some(r) = node(doc, "reference", local) else {
        return false;
    };
    if context["complement_evidence"] != r["mention"] && context["complement_evidence"] != r["name"]
    {
        return false;
    }
    // Every own host evidence, including internally nested qualifiers, lies in the external clause.
    fn inside(v: &Value, external: &Value) -> bool {
        if !step() {
            return false;
        }
        match v {
            Value::Object(m) => m.iter().all(|(k, x)| {
                if k == "context_bindings" {
                    true
                } else if k == "evidence" {
                    x.as_array()
                        .is_some_and(|a| a.iter().all(|e| step() && contains(external, e)))
                } else {
                    inside(x, external)
                }
            }),
            Value::Array(a) => a.iter().all(|x| inside(x, external)),
            _ => true,
        }
    }
    if !inside(owner, &context["external_clause_evidence"]) {
        return false;
    }
    for qid in rows(owner, "qualifier_ids")
        .iter()
        .chain(rows(&owner["body"]["value"], "qualifier_ids"))
    {
        let Some(q) = qid.as_str().and_then(|i| node(doc, "qualifier", i)) else {
            return false;
        };
        if !inside(q, &context["external_clause_evidence"]) {
            return false;
        }
    }
    true
}
pub(crate) fn admitted(doc: &Value, owner: &Value) -> bool {
    let contexts = rows(owner, "context_bindings");
    let p = &owner["body"]["value"];
    if doc["kind"] != "source"
        || owner["body"]["kind"] != "proposition"
        || contexts.len() != 1
        || contexts[0]["kind"] != KIND
        || p["atom"]["polarity"] != "positive"
        || p["judgment"] != "fact"
        || p["status"] != "occurred"
        || p["basis"] != "description"
        || !p["normative_scope"].is_null()
    {
        return false;
    }
    let Some(complement) = contexts[0]["complement"]["id"]
        .as_str()
        .and_then(|i| node(doc, "reference", i))
    else {
        return false;
    };
    if !simple_nominal(complement)
        || !rows(complement, "links").is_empty()
        || rows(doc, "identity")
            .iter()
            .any(|e| step() && (e["left"] == complement["id"] || e["right"] == complement["id"]))
    {
        return false;
    }
    let cl = closure(doc, "claim", owner["id"].as_str().unwrap_or(""));
    for (kind, local) in cl {
        let Some(n) = node(doc, &kind, &local) else {
            return false;
        };
        match kind.as_str() {
            "scope" => {
                if n["kind"] != "document"
                    || !n["parent_id"].is_null()
                    || !n["binding"].is_null()
                    || !n["branch_id"].is_null()
                {
                    return false;
                }
            }
            "reference" => {
                if !simple_nominal(n) {
                    return false;
                }
            }
            "qualifier" => {
                if !matches!(n["kind"].as_str(), Some("configuration" | "time"))
                    || n["resolution"] != "known"
                {
                    return false;
                }
            }
            "claim" => {
                if n["body"]["kind"] != "proposition"
                    || n["body"]["value"]["atom"]["polarity"] != "positive"
                    || n["body"]["value"]["judgment"] != "fact"
                    || !matches!(
                        n["body"]["value"]["status"].as_str(),
                        Some("state" | "occurred")
                    )
                    || n["body"]["value"]["basis"] != "description"
                    || !n["body"]["value"]["normative_scope"].is_null()
                {
                    return false;
                }
            }
            _ => return false,
        }
    }
    !budget::exhausted()
}
fn endpoint(v: &Value, doc: &Value) -> bool {
    closed(v, &["kind", "id"])
        && id(&v["id"])
        && v["kind"]
            .as_str()
            .zip(v["id"].as_str())
            .is_some_and(|(k, i)| node(doc, k, i).is_some())
}
fn result_error(code: &str) -> Value {
    json!({"kind":"error","category":if code=="request_context_limit"{"operation"}else{"structural"},"code":code})
}
fn triple(v: &Value) -> bool {
    closed(v, &["vocabulary", "text", "raw"]) && v["text"].is_string()
}
fn feature_delta(base: &Value, derived: &Value) -> bool {
    if !reserve(derived, 2) {
        return false;
    }
    let mut back = derived.clone();
    let Some(features) = back["request_context_profile"]["features"].as_array_mut() else {
        return false;
    };
    if features.iter().filter(|v| *v == KIND).count() != 1 {
        return false;
    }
    features.retain(|v| v != KIND);
    if base.get("request_context_profile").is_none() {
        if !features.is_empty() {
            return false;
        }
        back.as_object_mut()
            .unwrap()
            .remove("request_context_profile");
    }
    walk(base, 0) && back == *base
}
fn provenance(input: &Value) -> bool {
    let base = &input["base"];
    let derived = &input["derived"];
    if !triple(base)
        || !triple(derived)
        || base["text"] != derived["text"]
        || base["raw"]["domain_namespace"] != input["domain_namespace"]
        || derived["raw"]["domain_namespace"] != input["domain_namespace"]
        || !feature_delta(&base["vocabulary"], &derived["vocabulary"])
    {
        return false;
    }
    if !reserve(&derived["raw"], 2) {
        return false;
    }
    let mut back = derived["raw"].clone();
    let Some(owner) = back["claims"]
        .as_array_mut()
        .and_then(|a| a.iter_mut().find(|c| c["id"] == input["owner_claim_id"]))
    else {
        return false;
    };
    let Some(original) = rows(&base["raw"], "claims")
        .iter()
        .find(|c| step() && c["id"] == input["owner_claim_id"])
    else {
        return false;
    };
    if original
        .get("context_bindings")
        .is_some_and(|x| !x.as_array().is_some_and(Vec::is_empty))
    {
        return false;
    }
    if let Some(old) = original.get("context_bindings") {
        owner["context_bindings"] = old.clone();
    } else {
        owner.as_object_mut().unwrap().remove("context_bindings");
    }
    back["vocabulary_sha256"] = base["raw"]["vocabulary_sha256"].clone();
    walk(&base["raw"], 0) && back == base["raw"]
}
fn nominal_intent(r: &Value) -> bool {
    simple_nominal(r)
}
fn intent_valid(intent: &Value, derived: &Value, hash: &str) -> bool {
    if !closed(
        intent,
        &[
            "version",
            "derived_input_sha256",
            "predicate",
            "grounded_role_constraints",
        ],
    ) || intent["version"] != "lexical_clause_evidence_selection_v1"
        || intent["derived_input_sha256"] != hash
        || !id(&intent["predicate"])
    {
        return false;
    }
    let Some(pred) = rows(&derived["vocabulary"], "predicate_rows")
        .iter()
        .find(|p| step() && p["id"] == intent["predicate"])
    else {
        return false;
    };
    let Some(xs) = intent["grounded_role_constraints"].as_array() else {
        return false;
    };
    if xs.is_empty() || xs.len() > pred["roles"].as_object().map_or(0, serde_json::Map::len) {
        return false;
    }
    let mut roles = BTreeSet::new();
    let mut anchored = false;
    for x in xs {
        if !closed(x, &["role", "term"])
            || !id(&x["role"])
            || !roles.insert(x["role"].as_str().unwrap_or(""))
        {
            return false;
        }
        let t = &x["term"];
        let typ = pred["roles"][x["role"].as_str().unwrap_or("")].as_str();
        if !matches!(
            t["kind"].as_str(),
            Some("ref" | "symbol" | "quantity" | "time" | "text")
        ) || !super::validation::selection_term_valid(
            t,
            derived["text"].as_str().unwrap_or(""),
            &derived["vocabulary"],
        ) {
            return false;
        }
        let valid = match typ {
            Some("ref") => t["kind"] == "ref",
            Some("quantity") => t["kind"] == "quantity",
            Some("attribute_id") => {
                t["kind"] == "symbol"
                    && rows(&derived["vocabulary"], "fixed_attribute_rows").contains(&t["id"])
            }
            Some("action_id") => {
                t["kind"] == "symbol"
                    && rows(&derived["vocabulary"], "fixed_action_rows").contains(&t["id"])
            }
            Some("term") => t["kind"] != "symbol",
            _ => false,
        };
        if !valid || (matches!(t["kind"].as_str(), Some("ref" | "symbol")) && !id(&t["id"])) {
            return false;
        }
        if t["kind"] == "ref" {
            let Some(r) = t["id"]
                .as_str()
                .and_then(|i| node(&derived["raw"], "reference", i))
            else {
                return false;
            };
            if !nominal_intent(r) {
                return false;
            }
            anchored = true;
        }
    }
    anchored
}
fn manifest_valid(trust: &Value) -> bool {
    if !closed(
        trust,
        &[
            "version",
            "authorized_source_input_sha256",
            "review_anchors",
        ],
    ) || trust["version"] != "lexical_clause_trust_manifest_v1"
        || (!trust["authorized_source_input_sha256"].is_null()
            && !sha(&trust["authorized_source_input_sha256"]))
    {
        return false;
    }
    let Some(anchors) = trust["review_anchors"].as_array() else {
        return false;
    };
    if anchors.len() > 100_000 {
        return false;
    }
    let mut seen = BTreeSet::new();
    anchors.iter().all(|a| {
        closed(
            a,
            &[
                "review_anchor_id",
                "review_sha256",
                "source_input_sha256",
                "derived_input_sha256",
                "owner_claim_id",
                "context_id",
            ],
        ) && ["review_anchor_id", "owner_claim_id", "context_id"]
            .iter()
            .all(|k| id(&a[*k]))
            && [
                "review_sha256",
                "source_input_sha256",
                "derived_input_sha256",
            ]
            .iter()
            .all(|k| sha(&a[*k]))
            && seen.insert(a["review_anchor_id"].as_str().unwrap_or(""))
    })
}
fn review_valid(
    review: &Value,
    input: &Value,
    source_hash: &str,
    derived_hash: &str,
    cl: &Nodes,
) -> Option<(Vec<String>, bool)> {
    let doc = &input["derived"]["raw"];
    let text = input["derived"]["text"].as_str()?;
    if !closed(
        review,
        &[
            "version",
            "source_input_sha256",
            "derived_input_sha256",
            "review_anchor_id",
            "owner_claim_id",
            "context_id",
            "reviewed_internal_attachment",
            "host_fidelity",
            "external_attachment",
            "internal_attachment",
            "complement_fidelity",
            "incidences",
        ],
    ) || review["version"] != "lexical_clause_semantic_review_v1"
        || review["source_input_sha256"] != source_hash
        || review["derived_input_sha256"] != derived_hash
        || review["owner_claim_id"] != input["owner_claim_id"]
        || review["context_id"] != input["context_id"]
        || !id(&review["review_anchor_id"])
    {
        return None;
    }
    let owner = node(doc, "claim", input["owner_claim_id"].as_str()?)?;
    let context = rows(owner, "context_bindings")
        .iter()
        .find(|c| c["id"] == input["context_id"])?;
    if review["reviewed_internal_attachment"] != context["internal_attachment"]["kind"] {
        return None;
    }
    let mut all_accepted = true;
    for key in [
        "host_fidelity",
        "external_attachment",
        "internal_attachment",
        "complement_fidelity",
    ] {
        if !matches!(review[key].as_str(), Some("accepted" | "pending")) {
            return None;
        }
        all_accepted &= review[key] == "accepted";
    }
    let incidences = review["incidences"].as_array()?;
    if incidences.len() > 100_000 {
        return None;
    }
    let mut seen = BTreeSet::new();
    let mut covered = BTreeSet::new();
    let mut effective = Vec::new();
    for inc in incidences {
        if !closed(
            inc,
            &[
                "id",
                "affected_owner",
                "affected_context_id",
                "origin",
                "disposition",
            ],
        ) || !id(&inc["id"])
            || !seen.insert(inc["id"].as_str()?)
            || !endpoint(&inc["affected_owner"], doc)
        {
            return None;
        }
        if !inc["affected_context_id"].is_null() {
            if !id(&inc["affected_context_id"]) || inc["affected_owner"]["kind"] != "claim" {
                return None;
            }
            let affected = node(doc, "claim", inc["affected_owner"]["id"].as_str()?)?;
            if rows(affected, "context_bindings")
                .iter()
                .filter(|c| step() && c["id"] == inc["affected_context_id"])
                .count()
                != 1
            {
                return None;
            }
        }
        let origin = &inc["origin"];
        match origin["kind"].as_str() {
            Some("coverage") => {
                if !closed(origin, &["kind", "owner", "span"])
                    || (!origin["owner"].is_null() && !endpoint(&origin["owner"], doc))
                    || !evidence(&origin["span"], text)
                {
                    return None;
                }
                let matches: Vec<_> = rows(doc, "coverage")
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| {
                        step()
                            && c["owner"] == origin["owner"]
                            && c["span"] == origin["span"]
                            && matches!(c["state"].as_str(), Some("unresolved" | "unsupported"))
                    })
                    .map(|(i, _)| i)
                    .collect();
                if matches.len() != 1 || !covered.insert(matches[0]) {
                    return None;
                }
            }
            Some("review") => {
                if !closed(origin, &["kind", "evidence"]) || !evidences(&origin["evidence"], text) {
                    return None;
                }
            }
            _ => return None,
        }
        let disposition = &inc["disposition"];
        match disposition["kind"].as_str() {
            Some("unresolved") => {
                if !closed(disposition, &["kind"]) {
                    return None;
                }
            }
            Some("resolved") => {
                if !closed(disposition, &["kind", "evidence", "justification"])
                    || !evidences(&disposition["evidence"], text)
                    || !disposition["justification"]
                        .as_str()
                        .is_some_and(|s| !s.trim().is_empty() && s.len() <= 4096)
                {
                    return None;
                }
            }
            _ => return None,
        }
        let owner_key = (
            inc["affected_owner"]["kind"].as_str()?.to_owned(),
            inc["affected_owner"]["id"].as_str()?.to_owned(),
        );
        if !budget::charge(cl.len() + 1, owner_key.0.len() + owner_key.1.len() + 128) {
            return None;
        }
        if cl.contains(&owner_key) && disposition["kind"] == "unresolved" {
            effective.push(inc["id"].as_str()?.to_owned());
        }
    }
    for (i, cov) in rows(doc, "coverage").iter().enumerate() {
        if !step() {
            return None;
        }
        if matches!(cov["state"].as_str(), Some("unresolved" | "unsupported"))
            && !covered.contains(&i)
        {
            return None;
        }
    }
    if all_accepted && !effective.is_empty() {
        return None;
    }
    if !budget::charge(effective.len().saturating_mul(effective.len()), 0) {
        return None;
    }
    effective.sort();
    Some((effective, all_accepted))
}
fn identity_component(doc: &Value, local: &str) -> Option<BTreeSet<String>> {
    if !budget::charge(1, local.len() + 64) {
        return None;
    }
    let mut out = BTreeSet::from([local.to_owned()]);
    loop {
        let before = out.len();
        for edge in rows(doc, "identity") {
            if !budget::charge(out.len().saturating_add(1), 0) {
                return None;
            }
            if edge["kind"] != "same" {
                continue;
            }
            let (a, b) = (edge["left"].as_str()?, edge["right"].as_str()?);
            if out.contains(a) || out.contains(b) {
                if !budget::charge(1, a.len() + b.len() + 128) {
                    return None;
                }
                out.insert(a.to_owned());
                out.insert(b.to_owned());
            }
        }
        // Only identity edges establish this first local nominal selection.
        // Anaphoric anchors are excluded by admission, not reinterpreted as aliases.
        if before == out.len() {
            break;
        }
    }
    for edge in rows(doc, "identity") {
        if !budget::charge(out.len().saturating_add(1), 0) {
            return None;
        }
        if edge["kind"] == "different"
            && out.contains(edge["left"].as_str()?)
            && out.contains(edge["right"].as_str()?)
        {
            return None;
        }
    }
    Some(out)
}
fn term_same(doc: &Value, a: &Value, b: &Value) -> bool {
    if !walk(a, 0) || !walk(b, 0) || a["kind"] != b["kind"] {
        return false;
    }
    match a["kind"].as_str() {
        Some("ref") => {
            let Some(local) = a["id"].as_str() else {
                return false;
            };
            identity_component(doc, local)
                .is_some_and(|c| c.contains(b["id"].as_str().unwrap_or("")))
        }
        Some("symbol") => a["id"] == b["id"],
        Some("quantity") => {
            let same_unit = match (&a["unit"], &b["unit"]) {
                (Value::String(a), Value::String(b)) => a == b,
                (Value::Null, Value::Null) => true,
                _ => false,
            };
            a["decimal"] == b["decimal"] && same_unit && a["bound"] == b["bound"]
        }
        Some("time") => a["value"] == b["value"] && a["format"] == b["format"],
        Some("text") => a["value"] == b["value"],
        _ => false,
    }
}
fn closure_has_issues(report: &Value, doc: &Value, cl: &Nodes) -> bool {
    for issue in rows(report, "issues") {
        if !step() {
            return true;
        }
        let path = issue["path"].as_str().unwrap_or("");
        // Coverage debt is checked exhaustively against the anchored review below.
        if path == "/coverage" {
            continue;
        }
        for (kind, local) in cl {
            if !step() {
                return true;
            }
            let escaped = local.replace('~', "~0").replace('/', "~1");
            let singular = format!("/{kind}/{escaped}");
            let indexed = table(kind).and_then(|tab| {
                rows(doc, tab)
                    .iter()
                    .position(|row| step() && row["id"] == *local)
                    .map(|i| format!("/{tab}/{i}"))
            });
            if path.is_empty()
                || path.starts_with("/vocabulary")
                || path == singular
                || path.starts_with(&(singular + "/"))
                || indexed.is_some_and(|p| path == p || path.starts_with(&(p + "/")))
            {
                return true;
            }
        }
    }
    false
}
fn select_inner(input: &Value, trust: &Value) -> Value {
    if !closed(
        input,
        &[
            "version",
            "base",
            "derived",
            "owner_claim_id",
            "context_id",
            "domain_namespace",
            "selection",
            "semantic_review",
        ],
    ) || input["version"] != "lexical_clause_selection_input_v1"
        || !["owner_claim_id", "context_id", "domain_namespace"]
            .iter()
            .all(|k| id(&input[*k]))
        || !triple(&input["base"])
        || !triple(&input["derived"])
    {
        return result_error("invalid_document");
    }
    let derived = &input["derived"];
    let base = &input["base"];
    let vocab = &derived["vocabulary"];
    let doc = &derived["raw"];
    if !profile::permits(vocab, KIND) {
        return result_error("invalid_document");
    }
    let base_result = super::validate(
        &base["raw"],
        base["text"].as_str().unwrap_or(""),
        &base["vocabulary"],
    );
    let derived_result = super::validate(doc, derived["text"].as_str().unwrap_or(""), vocab);
    if base_result.report["status"] == "invalid"
        || derived_result.report["status"] == "invalid"
        || doc["kind"] != "source"
    {
        return result_error("invalid_document");
    }
    let Some(owner) = node(doc, "claim", input["owner_claim_id"].as_str().unwrap_or("")) else {
        return result_error("invalid_document");
    };
    let Some(context) = rows(owner, "context_bindings")
        .iter()
        .find(|c| step() && c["id"] == input["context_id"] && c["kind"] == KIND)
    else {
        return result_error("invalid_document");
    };
    let source_hash = digest(base);
    let derived_hash = digest(derived);
    if !intent_valid(&input["selection"], derived, &derived_hash) {
        return result_error("invalid_intent");
    }
    if !manifest_valid(trust)
        || !provenance(input)
        || (!trust["authorized_source_input_sha256"].is_null()
            && trust["authorized_source_input_sha256"] != source_hash)
    {
        return result_error("invalid_provenance");
    }
    let cl = closure(doc, "claim", input["owner_claim_id"].as_str().unwrap_or(""));
    let review = &input["semantic_review"];
    let mut effective = Vec::new();
    let mut accepted = false;
    let mut authorized = false;
    if !review.is_null() {
        let Some((debts, ok)) = review_valid(review, input, &source_hash, &derived_hash, &cl)
        else {
            return result_error("invalid_review");
        };
        effective = debts;
        accepted = ok;
        if let Some(anchor) = rows(trust, "review_anchors")
            .iter()
            .find(|a| step() && a["review_anchor_id"] == review["review_anchor_id"])
        {
            if anchor["review_sha256"] != digest(review)
                || anchor["source_input_sha256"] != source_hash
                || anchor["derived_input_sha256"] != derived_hash
                || anchor["owner_claim_id"] != input["owner_claim_id"]
                || anchor["context_id"] != input["context_id"]
            {
                return result_error("invalid_review");
            }
            authorized = true;
        }
    }
    if !admitted(doc, owner) || closure_has_issues(&derived_result.report, doc, &cl) {
        return json!({"kind":"unsupported","derived_input_sha256":derived_hash,"owner_claim_id":input["owner_claim_id"],"context_id":input["context_id"],"reason":"unsupported_scope"});
    }
    let mut pending = Vec::new();
    if trust["authorized_source_input_sha256"].is_null() {
        pending.push("unverified_provenance");
    }
    if !authorized {
        pending.push("unverified_review");
        effective.clear();
    }
    if authorized && (!accepted || !effective.is_empty()) {
        pending.push("semantic_pending");
    }
    if !pending.is_empty() {
        pending.sort();
        return json!({"kind":"pending","derived_input_sha256":derived_hash,"owner_claim_id":input["owner_claim_id"],"context_id":input["context_id"],"reasons":pending,"effective_incidence_ids":effective});
    }
    let atom = &owner["body"]["value"]["atom"];
    let no_match = |reason: &str| json!({"kind":"no_match","derived_input_sha256":derived_hash,"owner_claim_id":input["owner_claim_id"],"context_id":input["context_id"],"reason":reason});
    if atom["predicate"] != input["selection"]["predicate"] {
        return no_match("predicate_mismatch");
    }
    let mut matched = Vec::new();
    for constraint in rows(&input["selection"], "grounded_role_constraints") {
        let Some(arg) = rows(atom, "arguments")
            .iter()
            .find(|a| step() && a["role"] == constraint["role"])
        else {
            return no_match("role_constraint_mismatch");
        };
        if !term_same(doc, &arg["value"], &constraint["term"]) {
            return no_match("role_constraint_mismatch");
        }
        matched.push(constraint["role"].as_str().unwrap_or("").to_owned());
    }
    if !budget::charge(matched.len().saturating_mul(matched.len()), 0) {
        return result_error("request_context_limit");
    }
    matched.sort();
    if !reserve(derived, 4) || !reserve(review, 2) || !reserve(&input["selection"], 2) {
        return result_error("request_context_limit");
    }
    let endpoints: Vec<_> = cl
        .into_iter()
        .map(|(kind, id)| json!({"kind":kind,"id":id}))
        .collect();
    if !budget::charge(endpoints.len(), endpoints.len().saturating_mul(512)) {
        return result_error("request_context_limit");
    }
    let witness = json!({"version":"lexical_clause_partial_witness_v1","source_input_sha256":source_hash,"derived_input_sha256":derived_hash,"domain_namespace":input["domain_namespace"],"owner_claim_id":input["owner_claim_id"],"context_id":input["context_id"],"source_text":derived["text"],"vocabulary":vocab,"document":doc,"owner_claim":owner,"effective_closure":endpoints,"selection":input["selection"],"matched_roles":matched,"semantic_review":review,"status":"selected_partial_contextual_evidence","answer_terms":[],"answer_propositions":[],"full_answer_supported":false,"full_text_coverage_claimed":false});
    let _ = context;
    json!({"kind":"selected","witness":witness})
}
/// The caller supplies trust separately; the candidate never chooses or embeds its authority.
pub(crate) fn select(input: &Value, trust: &Value) -> Value {
    let vocab = &input["derived"]["vocabulary"];
    budget::with_operation(vocab, input, || {
        // Additional operands share both node/evidence preflight and all work budgets.
        if !budget::preflight(trust) {
            budget::charge(usize::MAX, 0);
            return result_error("request_context_limit");
        }
        let result = select_inner(input, trust);
        if !reserve(&result, 2) {
            return result_error("request_context_limit");
        }
        result
    })
    .unwrap_or_else(|_| result_error("request_context_limit"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn quote(text: &str, part: &str) -> Value {
        let start = text.find(part).unwrap();
        json!({"start_byte":start,"end_byte":start+part.len(),"quote":part})
    }
    fn reference(text: &str, local: &str, name: &str) -> Value {
        json!({"id":local,"scope_id":"root","mention":quote(text,name),"name":quote(text,name),"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null})
    }
    fn hashes(input: &mut Value, trust: &mut Value) {
        // Fixtures obey the same mandatory graph contract as CLI inputs.
        // Do not add an edge solely for the new context: that operand is implicit.
        fn operands(v: &Value, refs: &mut BTreeSet<String>, quals: &mut BTreeSet<String>) {
            match v {
                Value::Object(m) => {
                    if v["kind"] == "ref" {
                        if let Some(id) = v["id"].as_str() {
                            refs.insert(id.to_owned());
                        }
                    }
                    for (k, x) in m {
                        if k == "context_bindings" {
                            continue;
                        }
                        if k == "qualifier_ids" {
                            for q in x.as_array().into_iter().flatten().filter_map(Value::as_str) {
                                quals.insert(q.to_owned());
                            }
                        }
                        operands(x, refs, quals);
                    }
                }
                Value::Array(a) => {
                    for x in a {
                        operands(x, refs, quals);
                    }
                }
                _ => {}
            }
        }
        for side in ["base", "derived"] {
            let raw = &mut input[side]["raw"];
            let mut edges = rows(raw, "dependencies").to_vec();
            for (table, kind) in [("references", "reference"), ("claims", "claim")] {
                for owner in rows(raw, table) {
                    let mut targets = vec![("scope".to_owned(), "root".to_owned(), "binding")];
                    let mut refs = BTreeSet::new();
                    let mut quals = BTreeSet::new();
                    operands(owner, &mut refs, &mut quals);
                    targets.extend(
                        refs.into_iter()
                            .map(|id| ("reference".to_owned(), id, "identity")),
                    );
                    targets.extend(
                        quals
                            .into_iter()
                            .map(|id| ("qualifier".to_owned(), id, "configuration")),
                    );
                    for (required, id, role) in targets {
                        let edge = json!({"consumer":{"kind":kind,"id":owner["id"]},"requires":{"kind":required,"id":id},"role":role});
                        if !edges.contains(&edge) {
                            edges.push(edge);
                        }
                    }
                }
            }
            raw["dependencies"] = json!(edges);
        }
        input["base"]["raw"]["vocabulary_sha256"] = json!(digest(&input["base"]["vocabulary"]));
        input["derived"]["raw"]["vocabulary_sha256"] =
            json!(digest(&input["derived"]["vocabulary"]));
        let source = digest(&input["base"]);
        let derived = digest(&input["derived"]);
        input["selection"]["derived_input_sha256"] = json!(derived);
        input["semantic_review"]["source_input_sha256"] = json!(source);
        input["semantic_review"]["derived_input_sha256"] = json!(derived);
        trust["authorized_source_input_sha256"] = json!(source);
        trust["review_anchors"] = json!([{"review_anchor_id":"fixture-review","review_sha256":digest(&input["semantic_review"]),"source_input_sha256":source,"derived_input_sha256":derived,"owner_claim_id":"action","context_id":"context"}]);
    }
    fn fixture() -> (Value, Value) {
        let text = "Lira actuó en Orbe.";
        let vocab = json!({"version":"lexical-test-v1","predicate_rows":[{"id":"action","roles":{"actor":"ref","action_kind":"action_id"},"distributive":false}],"fixed_attribute_rows":[],"fixed_action_rows":["operate"],"semantic_type_rows":[],"relevance_links":[]});
        let raw = json!({"version":"documentary_evidence_v3","kind":"source","text_sha256":Sha256::digest(text.as_bytes()).iter().map(|b|format!("{b:02x}")).collect::<String>(),"vocabulary_sha256":"","domain_namespace":"lexical-controls","scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":[reference(text,"actor","Lira"),reference(text,"complement","Orbe")],"identity":[],"qualifiers":[],"claims":[{"id":"action","scope_id":"root","qualifier_ids":[],"evidence":[quote(text,"Lira actuó")],"body":{"kind":"proposition","value":{"atom":{"predicate":"action","arguments":[{"role":"actor","value":{"kind":"ref","id":"actor"}},{"role":"action_kind","value":{"kind":"symbol","id":"operate","evidence":[quote(text,"actuó")]}}],"polarity":"positive","evidence":[quote(text,"Lira actuó")]},"judgment":"fact","status":"occurred","basis":"description","normative_scope":null,"qualifier_ids":[]}}}],"requests":[],"dependencies":[],"coverage":[{"span":quote(text,"Lira actuó"),"state":"represented","owner":{"kind":"claim","id":"action"},"issue_id":null},{"span":quote(text,"en Orbe"),"state":"unresolved","owner":null,"issue_id":"unresolved-pp"}]});
        let base = json!({"vocabulary":vocab,"text":text,"raw":raw});
        let mut derived = base.clone();
        derived["vocabulary"]["request_context_profile"] =
            json!({"version":profile::VERSION,"features":[KIND]});
        derived["raw"]["claims"][0]["context_bindings"] = json!([{"id":"context","kind":KIND,"preposition":quote(text,"en"),"complement":{"kind":"ref","id":"complement"},"complement_evidence":quote(text,"Orbe"),"phrase_evidence":quote(text,"en Orbe"),"external_clause_evidence":quote(text,"Lira actuó en Orbe"),"internal_attachment":{"kind":"whole_clause_only"},"specialization":null}]);
        let review = json!({"version":"lexical_clause_semantic_review_v1","source_input_sha256":"","derived_input_sha256":"","review_anchor_id":"fixture-review","owner_claim_id":"action","context_id":"context","reviewed_internal_attachment":"whole_clause_only","host_fidelity":"accepted","external_attachment":"accepted","internal_attachment":"accepted","complement_fidelity":"accepted","incidences":[{"id":"pp-debt","affected_owner":{"kind":"claim","id":"action"},"affected_context_id":"context","origin":{"kind":"coverage","owner":null,"span":quote(text,"en Orbe")},"disposition":{"kind":"resolved","evidence":[quote(text,"en Orbe")],"justification":"Synthetic mechanism control; no linguistic quality judgment."}}]});
        let mut input = json!({"version":"lexical_clause_selection_input_v1","base":base,"derived":derived,"owner_claim_id":"action","context_id":"context","domain_namespace":"lexical-controls","selection":{"version":"lexical_clause_evidence_selection_v1","derived_input_sha256":"","predicate":"action","grounded_role_constraints":[{"role":"actor","term":{"kind":"ref","id":"actor"}}]},"semantic_review":review});
        let mut trust = json!({"version":"lexical_clause_trust_manifest_v1","authorized_source_input_sha256":null,"review_anchors":[]});
        hashes(&mut input, &mut trust);
        (input, trust)
    }
    #[test]
    fn positive_selection_preserves_qualified_occurrence_and_resolved_debt() {
        let (input, trust) = fixture();
        for side in ["base", "derived"] {
            let d = &input[side];
            let report =
                super::super::validate(&d["raw"], d["text"].as_str().unwrap(), &d["vocabulary"]);
            assert_ne!(
                report.report["status"], "invalid",
                "{side}: {}",
                report.report
            );
        }
        let result = select(&input, &trust);
        assert_eq!(result["kind"], "selected", "{result}");
        let w = &result["witness"];
        assert_eq!(w["document"], input["derived"]["raw"]);
        assert_eq!(w["owner_claim"], input["derived"]["raw"]["claims"][0]);
        assert_eq!(
            w["semantic_review"]["incidences"][0]["disposition"]["kind"],
            "resolved"
        );
        assert_eq!(w["answer_terms"], json!([]));
        assert_eq!(w["answer_propositions"], json!([]));
        assert_eq!(w["full_answer_supported"], false);
        assert_eq!(w["matched_roles"], json!(["actor"]));
    }
    #[test]
    fn absent_authority_is_pending_and_tampered_anchor_is_error() {
        let (input, mut trust) = fixture();
        trust["review_anchors"] = json!([]);
        trust["authorized_source_input_sha256"] = Value::Null;
        assert_eq!(
            select(&input, &trust)["reasons"],
            json!(["unverified_provenance", "unverified_review"])
        );
        let (input, mut trust) = fixture();
        trust["review_anchors"][0]["review_sha256"] = json!("0".repeat(64));
        assert_eq!(select(&input, &trust)["code"], "invalid_review");
    }
    #[test]
    fn debt_and_attachment_mismatch_never_become_positive() {
        let (mut input, mut trust) = fixture();
        input["semantic_review"]["incidences"][0]["disposition"] = json!({"kind":"unresolved"});
        hashes(&mut input, &mut trust);
        assert_eq!(select(&input, &trust)["code"], "invalid_review");
        input["semantic_review"]["external_attachment"] = json!("pending");
        hashes(&mut input, &mut trust);
        let r = select(&input, &trust);
        assert_eq!(r["kind"], "pending", "{r}");
        assert_eq!(r["effective_incidence_ids"], json!(["pp-debt"]));
        input["semantic_review"]["reviewed_internal_attachment"] = json!("source_underspecified");
        hashes(&mut input, &mut trust);
        assert_eq!(select(&input, &trust)["code"], "invalid_review");
    }
    #[test]
    fn local_intent_and_exact_provenance_are_required() {
        let (mut input, trust) = fixture();
        input["selection"]["derived_input_sha256"] = json!("0".repeat(64));
        assert_eq!(select(&input, &trust)["code"], "invalid_intent");
        let (mut input, trust) = fixture();
        input["selection"]["grounded_role_constraints"][0]["term"]["id"] = json!("outside");
        assert_eq!(select(&input, &trust)["code"], "invalid_intent");
        let (mut input, mut trust) = fixture();
        input["derived"]["raw"]["coverage"][0]["issue_id"] = json!("undeclared-delta");
        hashes(&mut input, &mut trust);
        assert_eq!(select(&input, &trust)["code"], "invalid_provenance");
    }
    #[test]
    fn dependency_and_difference_are_not_identity_but_same_is() {
        let (mut input, mut trust) = fixture();
        let mut other = input["base"]["raw"]["references"][0].clone();
        other["id"] = json!("other");
        for side in ["base", "derived"] {
            input[side]["raw"]["references"]
                .as_array_mut()
                .unwrap()
                .push(other.clone());
        }
        input["selection"]["grounded_role_constraints"][0]["term"]["id"] = json!("other");
        for relation in ["dependency", "different", "same"] {
            for side in ["base", "derived"] {
                input[side]["raw"]["identity"] = if relation == "dependency" {
                    json!([])
                } else {
                    json!([{"kind":relation,"left":"actor","right":"other","evidence":[other["mention"]]}])
                };
                input[side]["raw"]["dependencies"] = if relation == "dependency" {
                    json!([{"consumer":{"kind":"reference","id":"actor"},"requires":{"kind":"reference","id":"other"},"role":"identity"}])
                } else {
                    json!([])
                };
            }
            hashes(&mut input, &mut trust);
            let r = select(&input, &trust);
            assert_eq!(
                r["kind"],
                if relation == "same" {
                    "selected"
                } else {
                    "no_match"
                },
                "{relation}: {r}"
            );
        }
    }
    #[test]
    fn ordinary_routes_quarantine_owner_and_effective_consumers_only() {
        let (input, _) = fixture();
        let v = &input["derived"]["vocabulary"];
        let mut doc = input["derived"]["raw"].clone();
        let mut consumer = doc["claims"][0].clone();
        consumer["id"] = json!("consumer");
        consumer.as_object_mut().unwrap().remove("context_bindings");
        let mut independent = consumer.clone();
        independent["id"] = json!("independent");
        doc["claims"]
            .as_array_mut()
            .unwrap()
            .extend([consumer, independent]);
        doc["dependencies"] = json!([{"consumer":{"kind":"claim","id":"consumer"},"requires":{"kind":"claim","id":"action"},"role":"premise"}]);
        budget::with_operation(v, &doc, || {
            let blocked = blocked_nodes(&doc);
            assert!(blocked.contains(&("claim".into(), "action".into())));
            assert!(blocked.contains(&("claim".into(), "consumer".into())));
            assert!(!blocked.contains(&("claim".into(), "independent".into())));
        })
        .unwrap();
    }
    #[test]
    fn known_configuration_qualifier_is_preserved_without_becoming_an_answer() {
        let (mut input, mut trust) = fixture();
        let e = input["base"]["raw"]["references"][1]["mention"].clone();
        for side in ["base", "derived"] {
            input[side]["raw"]["claims"][0]["qualifier_ids"] = json!(["configuration"]);
            input[side]["raw"]["qualifiers"] = json!([{"id":"configuration","scope_id":"root","kind":"configuration","body":{"key":"context","value":{"kind":"text","value":"Orbe","evidence":[e]}},"resolution":"known","evidence":[e]}]);
        }
        hashes(&mut input, &mut trust);
        let r = select(&input, &trust);
        assert_eq!(r["kind"], "selected", "{r}");
        assert_eq!(
            r["witness"]["document"]["qualifiers"],
            input["derived"]["raw"]["qualifiers"]
        );
    }
    #[test]
    fn unresolved_effective_type_blocks_without_vetoing_an_independent_reference() {
        for effective in [true, false] {
            let (mut input, mut trust) = fixture();
            for side in ["base", "derived"] {
                let mut r = input[side]["raw"]["references"][0].clone();
                r["semantic_type"] = json!({"type_id":"unregistered","evidence":[r["mention"]]});
                if effective {
                    input[side]["raw"]["references"][0] = r;
                } else {
                    r["id"] = json!("independent");
                    input[side]["raw"]["references"]
                        .as_array_mut()
                        .unwrap()
                        .push(r);
                }
            }
            hashes(&mut input, &mut trust);
            let r = select(&input, &trust);
            assert_eq!(
                r["kind"],
                if effective { "unsupported" } else { "selected" },
                "effective={effective}: {r}"
            );
        }
    }
    #[test]
    fn absent_feature_adds_no_walk_and_combined_evidence_cap_is_atomic() {
        let v = json!({"request_context_profile":{"version":profile::VERSION,"features":["nominal_during"]}});
        budget::with_operation(&v, &json!({}), || {
            assert!(budget::charge(999_998, 0));
            assert!(blocked_nodes(&json!({"claims":[{}]})).is_empty());
            assert!(budget::charge(1, 0));
        })
        .unwrap();
        let v = json!({"request_context_profile":{"version":profile::VERSION,"features":[KIND]}});
        let many = json!(
            (0..2050)
                .map(|_| json!({"start_byte":0,"end_byte":1,"quote":"x"}))
                .collect::<Vec<_>>()
        );
        assert!(
            budget::with_operation(&v, &many, || {
                assert!(!budget::preflight(&many));
                budget::charge(usize::MAX, 0);
            })
            .is_err()
        );
    }
    #[test]
    fn lexical_clause_quantity_selection_uses_term_unit_strings_without_request_unit_conversion() {
        for (source_unit, wanted_unit, selected) in [
            (json!("min"), json!("min"), true),
            (json!("min"), json!("s"), false),
            (Value::Null, Value::Null, true),
            (json!("min"), Value::Null, false),
        ] {
            let (mut input, mut trust) = fixture();
            let text = "Lira actuó dos minutos en Orbe.";
            let amount = json!({"kind":"quantity","decimal":"2","unit":source_unit,"bound":"exact","evidence":[quote(text,"dos minutos")]});
            for side in ["base", "derived"] {
                input[side]["text"] = json!(text);
                input[side]["vocabulary"]["predicate_rows"][0]["roles"]["amount"] =
                    json!("quantity");
                let doc = &mut input[side]["raw"];
                doc["text_sha256"] = json!(
                    Sha256::digest(text.as_bytes())
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
                doc["references"][1]["mention"] = quote(text, "Orbe");
                doc["references"][1]["name"] = quote(text, "Orbe");
                doc["claims"][0]["evidence"] = json!([quote(text, "Lira actuó dos minutos")]);
                doc["claims"][0]["body"]["value"]["atom"]["evidence"] =
                    json!([quote(text, "Lira actuó dos minutos")]);
                doc["claims"][0]["body"]["value"]["atom"]["arguments"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"role":"amount","value":amount}));
                doc["coverage"][0]["span"] = quote(text, "Lira actuó dos minutos");
                doc["coverage"][1]["span"] = quote(text, "en Orbe");
                if side == "derived" {
                    let c = &mut doc["claims"][0]["context_bindings"][0];
                    c["preposition"] = quote(text, "en");
                    c["complement_evidence"] = quote(text, "Orbe");
                    c["phrase_evidence"] = quote(text, "en Orbe");
                    c["external_clause_evidence"] = quote(text, "Lira actuó dos minutos en Orbe");
                }
            }
            let mut requested = amount.clone();
            requested["unit"] = wanted_unit;
            input["selection"]["grounded_role_constraints"]
                .as_array_mut()
                .unwrap()
                .push(json!({"role":"amount","term":requested}));
            let incidence = &mut input["semantic_review"]["incidences"][0];
            incidence["origin"]["span"] = quote(text, "en Orbe");
            incidence["disposition"]["evidence"] = json!([quote(text, "en Orbe")]);
            hashes(&mut input, &mut trust);
            for side in ["base", "derived"] {
                let d = &input[side];
                let r = super::super::validate(&d["raw"], text, &d["vocabulary"]);
                assert_eq!(r.report["status"], "partial", "{side}: {}", r.report);
                assert_eq!(r.report["issues"][0]["path"], "/coverage");
                assert_eq!(r.report["issues"].as_array().unwrap().len(), 1);
            }
            assert!(super::super::validation::selection_term_valid(
                &requested,
                text,
                &input["derived"]["vocabulary"]
            ));
            let result = select(&input, &trust);
            assert_eq!(
                result["kind"],
                if selected { "selected" } else { "no_match" },
                "{result}"
            );
        }
    }
}
