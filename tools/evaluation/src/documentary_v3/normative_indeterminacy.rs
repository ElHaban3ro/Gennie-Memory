//! Opt-in representational completeness is not operational resolution.
//!
//! Frozen DTO additions on enabled question reports:
//! representation_disposition = {version, state: complete|partial|invalid,
//! admitted_request_ids: [Id]}; resolution_disposition = {version,
//! state: blocked|not_blocked, debts: [{request_id, kind, path, provenance,
//! issue: original Issue|null}], actor_inferred:false, normative_scope_inferred:false}.
//! kind = ambiguous_subordinate_actor|unexplicit_normative_scope|existing_validation_issue.
//! Pair pending DTO additionally preserves request_constraint and question_document,
//! answer_terms/answer_propositions/bindings=[], resolution_eligible:false.
//! No issue is deleted, renamed, or used alone as an eligibility whitelist.
use serde_json::{Value, json};
use std::collections::BTreeSet;
pub(crate) const FEATURE: &str = "normative_question_indeterminacy";
const VERSION: &str = "documentary_normative_indeterminacy_v1";
fn rows<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn charge(v: &Value) -> bool {
    charge_depth(v, 0)
}
fn charge_depth(v: &Value, depth: usize) -> bool {
    if depth > 64 {
        super::context_budget::charge(usize::MAX, 0);
        return false;
    }
    if !super::context_budget::charge(1, 0) {
        return false;
    }
    match v {
        Value::Object(m) => m.values().all(|v| charge_depth(v, depth + 1)),
        Value::Array(a) => a.iter().all(|v| charge_depth(v, depth + 1)),
        _ => true,
    }
}
fn copied(v: &Value) -> Value {
    if !charge(v)
        || !super::context_budget::charge(
            1,
            super::context_budget::serialized_bytes(v).unwrap_or(usize::MAX),
        )
    {
        return Value::Null;
    }
    v.clone()
}
fn escape(id: &str) -> String {
    id.replace('~', "~0").replace('/', "~1")
}
fn ambiguous(r: &Value) -> bool {
    rows(r, "context_bindings").iter().any(|c| {
        super::context_budget::charge(1, 0)
            && c["kind"] == "normative_action_relation"
            && c["action"]["actor"]["kind"] == "unresolved"
            && c["action"]["actor"]["reason"] == "ambiguous_control"
    })
}
fn seed(r: &Value) -> bool {
    rows(r, "context_bindings")
        .iter()
        .any(|c| super::context_budget::charge(1, 0) && c["kind"] == "normative_action_relation")
        && (ambiguous(r) || r["normative_scope_filter"].is_null())
}
/// Shape is reconstructed, while ordinary validation separately proves quotes,
/// required fields, vocabulary, dependency graph and reference scope integrity.
fn admitted(doc: &Value, r: &Value) -> bool {
    let contexts = rows(r, "context_bindings");
    let t = &r["target"];
    let args = rows(&t["atom"], "arguments");
    let scope = rows(doc, "scopes")
        .iter()
        .find(|s| super::context_budget::charge(1, 0) && s["id"] == r["scope_id"]);
    !rows(r, "evidence").is_empty()
        && !rows(&t["atom"], "evidence").is_empty()
        && args
            .iter()
            .filter(|a| a["role"] == "action_kind")
            .all(|a| !rows(&a["value"], "evidence").is_empty())
        && r["mode"] == "answer"
        && r["slot"] == "actor"
        && r["required_answer_extent"] == "one_value"
        && r["unit"].is_null()
        && r["class_filter"].is_object()
        && rows(r, "qualifier_ids").is_empty()
        && rows(r, "premise_ids").is_empty()
        && rows(t, "qualifier_ids").is_empty()
        && r["basis_requirement"] == json!({"kind":"exact","value":"normative"})
        && (r["normative_scope_filter"].is_null() || r["normative_scope_filter"].is_string())
        && t["judgment"] == "required"
        && t["status"] == "state"
        && t["atom"]["predicate"] == "action"
        && t["atom"]["polarity"] == "positive"
        && args.len() == 2
        && ["action_kind", "object"]
            .iter()
            .all(|role| args.iter().filter(|a| a["role"] == *role).count() == 1)
        && scope.is_some_and(|s| {
            s["kind"] == "document"
                && s["binding"].is_null()
                && s["branch_id"].is_null()
                && s["parent_id"].is_null()
        })
        && contexts.len() == 1
        && contexts[0]["kind"] == "normative_action_relation"
        && !rows(&contexts[0]["action"]["action_kind"], "evidence").is_empty()
        && matches!(
            contexts[0]["relation"]["kind"].as_str(),
            Some("purpose" | "necessary_condition")
        )
        && (contexts[0]["action"]["actor"]["kind"] != "unresolved"
            || matches!(
                contexts[0]["action"]["actor"]["reason"].as_str(),
                Some("ambiguous_control" | "unspecified")
            ))
}
fn expected(r: &Value, index: usize) -> Vec<(String, String)> {
    let mut out = vec![];
    if !super::context_budget::charge(
        1,
        r["id"]
            .as_str()
            .map_or(0, str::len)
            .saturating_mul(4)
            .saturating_add(1024),
    ) {
        return out;
    }
    // Partial multi-context inputs retain the true provenance as well. Only
    // admitted() may grant representational completeness, and it requires one.
    for (context_index, context) in rows(r, "context_bindings").iter().enumerate() {
        if !super::context_budget::charge(1, 0) {
            return out;
        }
        if context["kind"] == "normative_action_relation"
            && context["action"]["actor"]["kind"] == "unresolved"
            && context["action"]["actor"]["reason"] == "ambiguous_control"
        {
            if !super::context_budget::charge(1, 512) {
                return out;
            }
            out.push((
                "request_context_ambiguous".into(),
                format!("/requests/{index}/context_bindings/{context_index}/action/actor"),
            ));
        }
    }
    if r["normative_scope_filter"].is_null() {
        out.push((
            "request_context_unsupported".into(),
            format!("/request/{}", escape(r["id"].as_str().unwrap_or(""))),
        ));
    }
    out
}
fn debts(doc: &Value, issues: &[Value]) -> Vec<Value> {
    let mut out = vec![];
    for (i, r) in rows(doc, "requests").iter().enumerate() {
        if !seed(r) {
            continue;
        }
        for (code, path) in expected(r, i) {
            let issue = issues.iter().find(|issue| {
                super::context_budget::charge(1, 0)
                    && issue["code"] == code
                    && issue["path"] == path
            });
            let kind = if code == "request_context_ambiguous" {
                "ambiguous_subordinate_actor"
            } else {
                "unexplicit_normative_scope"
            };
            let bytes = path.len()
                + r["id"].as_str().map_or(0, str::len)
                + 512
                + issue
                    .and_then(super::context_budget::serialized_bytes)
                    .unwrap_or(0);
            if !super::context_budget::charge(1, bytes) {
                return out;
            }
            out.push(json!({"request_id":r["id"],"kind":kind,"path":path,"provenance":"validated_request_content","issue":issue}));
        }
    }
    // All remaining validation debts remain visible, including extraction
    // failures and dependencies that this feature never admits as complete.
    for issue in issues {
        if out
            .iter()
            .any(|d| charge(&d["issue"]) && charge(issue) && d["issue"] == *issue)
        {
            continue;
        }
        if !super::context_budget::charge(
            1,
            super::context_budget::serialized_bytes(issue)
                .unwrap_or(usize::MAX)
                .saturating_add(512),
        ) {
            return out;
        }
        out.push(json!({"request_id":null,"kind":"existing_validation_issue","path":issue["path"],"provenance":"validation_issue","issue":issue}));
    }
    out
}
pub(crate) fn decorate(report: &mut Value, doc: &Value, vocabulary: &Value) {
    if !super::request_context_profile::permits(vocabulary, FEATURE) || doc["kind"] != "question" {
        return;
    }
    if !charge(doc) || !charge(&report["issues"]) {
        return;
    }
    let bytes = super::context_budget::serialized_bytes(doc)
        .unwrap_or(usize::MAX)
        .saturating_add(
            super::context_budget::serialized_bytes(&report["issues"]).unwrap_or(usize::MAX),
        )
        .saturating_mul(8);
    if !super::context_budget::charge(1, bytes) {
        return;
    }
    let issues = rows(report, "issues");
    let mut admitted_ids = vec![];
    let mut allowed = BTreeSet::new();
    for (i, r) in rows(doc, "requests").iter().enumerate() {
        if seed(r) && admitted(doc, r) {
            if !super::context_budget::charge(1, r["id"].as_str().map_or(0, str::len) + 256) {
                return;
            }
            admitted_ids.push(r["id"].clone());
            allowed.extend(expected(r, i));
        }
    }
    let all_represented = !rows(doc, "coverage").is_empty()
        && rows(doc, "coverage")
            .iter()
            .all(|c| super::context_budget::charge(1, 0) && c["state"] == "represented");
    let complete = report["status"] != "invalid"
        && all_represented
        && issues.iter().all(|i| {
            super::context_budget::charge(
                1,
                i["code"]
                    .as_str()
                    .map_or(0, str::len)
                    .saturating_add(i["path"].as_str().map_or(0, str::len))
                    .saturating_add(64),
            ) && allowed.contains(&(
                i["code"].as_str().unwrap_or("").to_owned(),
                i["path"].as_str().unwrap_or("").to_owned(),
            ))
        });
    let debt = debts(doc, issues);
    let invalid = report["status"] == "invalid";
    if complete {
        report["status"] = json!("complete");
    }
    if !super::context_budget::charge(1, 1024) {
        return;
    }
    report["representation_disposition"] = json!({"version":VERSION,"state":if invalid {"invalid"} else if complete {"complete"} else {"partial"},"admitted_request_ids":admitted_ids});
    report["resolution_disposition"] = json!({"version":VERSION,"state":if debt.is_empty() && !invalid {"not_blocked"}else{"blocked"},"debts":debt,"actor_inferred":false,"normative_scope_inferred":false});
}
/// Derive the full effective consumer closure; caller invokes before every route.
/// A missing report issue or a partial label can never reopen the request.
pub(crate) fn pending(
    doc: &Value,
    request: &Value,
    report: &Value,
    vocabulary: &Value,
) -> Option<Value> {
    if !super::request_context_profile::permits(vocabulary, FEATURE) {
        return None;
    }
    if !charge(doc) {
        return None;
    }
    let mut blocked = BTreeSet::new();
    for r in rows(doc, "requests") {
        if seed(r) {
            if !super::context_budget::charge(1, r["id"].as_str().map_or(0, str::len) + 128) {
                return None;
            }
            blocked.insert((
                "request".to_owned(),
                r["id"].as_str().unwrap_or("").to_owned(),
            ));
        }
    }
    if blocked.is_empty() {
        return None;
    }
    let mut edges = vec![];
    for (table, kind) in [
        ("requests", "request"),
        ("claims", "claim"),
        ("references", "reference"),
        ("qualifiers", "qualifier"),
        ("scopes", "scope"),
    ] {
        for row in rows(doc, table) {
            let id = row["id"].as_str().unwrap_or("");
            // The shared collector owns identifiers; reserve before invoking it.
            let bytes = super::context_budget::serialized_bytes(row)
                .unwrap_or(usize::MAX)
                .saturating_add(
                    super::context_budget::serialized_bytes(&doc["identity"]).unwrap_or(usize::MAX),
                )
                .saturating_mul(16);
            if !super::context_budget::charge(1, bytes) {
                return None;
            }
            let mut deps = super::validation::effective_context_dependencies(doc, kind, id);
            for d in rows(doc, "dependencies") {
                if !super::context_budget::charge(1, 0) {
                    return None;
                }
                if d["consumer"]["kind"] == kind && d["consumer"]["id"] == id {
                    let k = d["requires"]["kind"].as_str().unwrap_or("");
                    let i = d["requires"]["id"].as_str().unwrap_or("");
                    if !super::context_budget::charge(1, k.len() + i.len() + 128) {
                        return None;
                    }
                    deps.push((k.to_owned(), i.to_owned()));
                }
            }
            for dep in deps {
                if !super::context_budget::charge(1, id.len() + kind.len() + 128) {
                    return None;
                }
                edges.push(((kind.to_owned(), id.to_owned()), dep));
            }
        }
    }
    loop {
        let before = blocked.len();
        for (consumer, required) in &edges {
            if !super::context_budget::charge(1, 0) {
                return None;
            }
            if blocked.contains(required) && !blocked.contains(consumer) {
                if !super::context_budget::charge(1, consumer.0.len() + consumer.1.len() + 128) {
                    return None;
                }
                blocked.insert(consumer.clone());
            }
        }
        if before == blocked.len() {
            break;
        }
    }
    if !blocked.contains(&(
        "request".into(),
        request["id"].as_str().unwrap_or("").into(),
    )) {
        return None;
    }
    if !charge(&report["issues"]) {
        return None;
    }
    let debt = debts(doc, rows(report, "issues"));
    let request_copy = copied(request);
    let document_copy = copied(doc);
    if super::context_budget::exhausted() {
        return None;
    }
    Some(
        json!({"request_id":request["id"],"support":[],"refutation":[],"partial":[],"conflicts":[],"missing_constraints":[FEATURE],"coverage":"none","unresolved":[{"kind":FEATURE,"request_id":request["id"],"claim_ids":[],"request_constraint":request_copy,"question_document":document_copy,"debts":debt,"answer_terms":[],"answer_propositions":[],"bindings":[],"resolution_eligible":false,"actor_inferred":false,"normative_scope_inferred":false}]}),
    )
}
