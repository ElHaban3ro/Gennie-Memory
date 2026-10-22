//! Occurrence-local reference descriptions. Constraints never become document claims.
use serde_json::Value;

fn visit() -> bool {
    super::context_budget::charge(1, 0)
}
fn charged<I: IntoIterator>(iter: I) -> impl Iterator<Item = I::Item> {
    iter.into_iter().take_while(|_| visit())
}
fn reserve_path(path: &str, key: &str) -> bool {
    super::context_budget::charge(
        1,
        path.len()
            .saturating_add(key.len().saturating_mul(6))
            .saturating_add(2),
    )
}

pub(super) const PROFILE: &str = "local_reference_descriptions_v1";
pub(super) fn enabled(v: &Value) -> bool {
    v["term_description_profile"]["version"] == PROFILE
}
pub(super) const TERM_ROLES_PROFILE: &str = "described_reference_term_roles_v1";
/// This additional opt-in widens only declared `term` argument/member roles.
pub(super) fn term_roles_enabled(v: &Value) -> bool {
    enabled(v) && v["described_term_role_profile"]["version"] == TERM_ROLES_PROFILE
}
pub(super) fn contains(v: &Value) -> bool {
    if !visit() {
        return false;
    }
    match v {
        Value::Object(o) => {
            v["kind"] == "described_ref" || o.values().take_while(|_| visit()).any(contains)
        }
        Value::Array(a) => a.iter().take_while(|_| visit()).any(contains),
        _ => false,
    }
}
fn semantic(v: &Value) -> Value {
    if !super::context_budget::charge(1, 2) {
        return Value::Null;
    }
    match v {
        Value::Object(o) => Value::Object(
            o.iter()
                .take_while(|_| visit())
                .filter(|(k, _)| *k != "evidence")
                .filter_map(|(k, v)| {
                    if !super::context_budget::charge(
                        1,
                        k.len().saturating_mul(6).saturating_add(2),
                    ) {
                        return None;
                    }
                    Some((k.clone(), semantic(v)))
                })
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().take_while(|_| visit()).map(semantic).collect()),
        _ => {
            let bytes = super::context_budget::serialized_bytes(v).unwrap_or(usize::MAX);
            if super::context_budget::charge(1, bytes) {
                v.clone()
            } else {
                Value::Null
            }
        }
    }
}
/// Shape and vocabulary validation recurse through existing Proposition/Term nodes.
/// This check adds local ownership, bounded non-recursion and explicit resolution.
pub(super) fn issues(term: &Value, vocabulary: &Value) -> Vec<&'static str> {
    let mut out = vec![];
    if !enabled(vocabulary) {
        out.push("unknown_vocabulary");
    }
    if term["evidence"].as_array().is_none_or(|a| a.is_empty()) {
        out.push("invalid_shape");
    }
    let Some(constraints) = term["constraints"].as_array() else {
        return vec!["invalid_shape"];
    };
    if constraints.is_empty() || constraints.len() > 16 {
        out.push("invalid_shape");
    }
    if term["resolution"] == "unknown" {
        out.push("unsupported_combination");
    }
    let mut seen = std::collections::BTreeSet::new();
    for p in charged(constraints) {
        let rooted = p["atom"]["arguments"].as_array().is_some_and(|a| {
            a.iter()
                .take_while(|_| visit())
                .any(|a| a["value"]["kind"] == "ref" && a["value"]["id"] == term["id"])
        });
        if !rooted
            || contains(p)
            || p["judgment"] != "fact"
            || !matches!(p["basis"].as_str(), Some("description" | "specification"))
            || !p["normative_scope"].is_null()
        {
            out.push("invalid_shape");
        }
        let key = semantic(p);
        let bytes = super::context_budget::serialized_bytes(&key).unwrap_or(usize::MAX);
        if !super::context_budget::charge(1, bytes) {
            return out;
        }
        if !seen.insert(key.to_string()) {
            out.push("invalid_shape");
        }
    }
    if !super::context_budget::charge(
        out.len()
            .saturating_mul(out.len().checked_ilog2().unwrap_or(0).saturating_add(1) as usize),
        0,
    ) {
        return vec![];
    }
    out.sort();
    out.dedup();
    out
}

/// V1 internal context supports time/configuration. A guard is retained pending;
/// it must not be discharged as a world fact or recursively compare itself.
pub(super) fn document_issues(raw: &Value) -> Vec<(String, String)> {
    fn qids(v: &Value, out: &mut Vec<String>) {
        if !visit() {
            return;
        }
        match v {
            Value::Object(o) => {
                for (k, x) in charged(o) {
                    if k == "qualifier_ids" {
                        out.extend(
                            x.as_array()
                                .into_iter()
                                .flatten()
                                .take_while(|_| visit())
                                .filter_map(Value::as_str)
                                .filter(|id| super::context_budget::charge(1, id.len()))
                                .map(str::to_owned),
                        );
                    }
                    qids(x, out);
                }
            }
            Value::Array(a) => {
                for x in charged(a) {
                    qids(x, out)
                }
            }
            _ => (),
        }
    }
    fn walk(v: &Value, path: &str, raw: &Value, out: &mut Vec<(String, String)>) {
        if !visit() {
            return;
        }
        if v["kind"] == "described_ref" {
            let mut ids = vec![];
            qids(&v["constraints"], &mut ids);
            for id in charged(ids) {
                if raw["qualifiers"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .take_while(|_| visit())
                    .find(|q| q["id"] == id)
                    .is_some_and(|q| q["kind"] == "guard")
                {
                    if !super::context_budget::charge(
                        2,
                        path.len().saturating_mul(2).saturating_add(64),
                    ) {
                        return;
                    }
                    out.push(("unsupported_combination".into(), path.into()));
                    out.push(("unsupported_description".into(), path.into()));
                }
            }
        }
        match v {
            Value::Object(o) => {
                for (k, x) in charged(o) {
                    if !reserve_path(path, k) {
                        return;
                    }
                    walk(
                        x,
                        &format!("{path}/{}", k.replace('~', "~0").replace('/', "~1")),
                        raw,
                        out,
                    );
                }
            }
            Value::Array(a) => {
                for (i, x) in a.iter().take_while(|_| visit()).enumerate() {
                    if !reserve_path(path, "array-index") {
                        return;
                    }
                    walk(x, &format!("{path}/{i}"), raw, out);
                }
            }
            _ => (),
        }
    }
    let mut out = vec![];
    walk(raw, "", raw, &mut out);
    if !super::context_budget::charge(
        out.len()
            .saturating_mul(out.len().checked_ilog2().unwrap_or(0).saturating_add(1) as usize),
        0,
    ) {
        return vec![];
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn recursive_description_walk_and_semantic_copies_share_operation_budget() {
        let vocabulary = json!({"request_context_profile":{"version":"documentary_request_context_v1","features":["nominal_during"]}});
        let raw = json!({"children":vec![json!({"value":"literal"});128]});
        assert!(
            super::super::context_budget::with_operation(&vocabulary, &json!({}), || {
                let (remaining_steps, _) =
                    super::super::context_budget::remaining_for_test().unwrap();
                assert!(super::super::context_budget::charge(
                    remaining_steps.checked_sub(100).unwrap(),
                    0
                ));
                document_issues(&raw)
            })
            .is_err()
        );
        let value = json!({"value":"a literal that must be copied"});
        assert!(
            super::super::context_budget::with_operation(&vocabulary, &json!({}), || {
                let (_, remaining_bytes) =
                    super::super::context_budget::remaining_for_test().unwrap();
                assert!(super::super::context_budget::charge(
                    0,
                    remaining_bytes.checked_sub(8).unwrap()
                ));
                semantic(&value)
            })
            .is_err()
        );
        assert!(document_issues(&raw).is_empty()); // No active operation preserves legacy behavior.
    }
}
