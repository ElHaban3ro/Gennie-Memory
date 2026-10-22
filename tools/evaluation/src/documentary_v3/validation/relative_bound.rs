//! Closed source-owned relative bound; the boundary is not an asserted event.
use super::*;
const KIND: &str = "relative_nonoccurrence_bound";
fn charge(c: &mut Check<'_>, p: &str) -> bool {
    if super::super::context_budget::charge(1, 0) {
        return true;
    }
    c.limit_exceeded = true;
    c.issue("request_context_limit", p);
    false
}
pub(super) fn node(c: &mut Check<'_>, t: &str, v: &Value, p: &str, depth: usize) -> bool {
    if t != "Qualifier" || v["kind"] != KIND {
        return false;
    }
    if !super::super::request_context_profile::permits(c.vocab, KIND)
        || !c.shape(
            v,
            p,
            &["id", "scope_id", "kind", "body", "resolution", "evidence"],
        )
    {
        c.issue("invalid_shape", p);
        return true;
    }
    for key in ["id", "scope_id"] {
        if !v[key]
            .as_str()
            .is_some_and(|s| !s.trim().is_empty() && s.len() <= 128)
        {
            c.issue("invalid_shape", &format!("{p}/{key}"));
        }
    }
    if v["resolution"] != "known"
        || !c.shape(
            &v["body"],
            &format!("{p}/body"),
            &["relation", "boundary_subject", "boundary_action"],
        )
        || v["body"]["relation"] != "until"
        || v["body"]["boundary_action"] != "switch_off"
        || !v["body"]["boundary_subject"]
            .as_str()
            .is_some_and(|s| !s.trim().is_empty() && s.len() <= 128)
    {
        c.issue("invalid_shape", p);
    }
    if !v["evidence"]
        .as_array()
        .is_some_and(|a| !a.is_empty() && a.len() <= 32)
    {
        c.issue("invalid_shape", p);
    }
    c.list(&v["evidence"], &format!("{p}/evidence"), "Evidence", depth);
    true
}
fn uses(c: &mut Check<'_>, value: &Value, qid: &Value, p: &str) -> usize {
    if !charge(c, p) {
        return 0;
    }
    match value {
        Value::Object(m) => m
            .iter()
            .map(|(key, child)| {
                if key == "qualifier_ids" {
                    array(child).iter().filter(|id| *id == qid).count()
                } else {
                    uses(c, child, qid, p)
                }
            })
            .sum(),
        Value::Array(a) => a.iter().map(|v| uses(c, v, qid, p)).sum(),
        _ => 0,
    }
}
pub(super) fn document(c: &mut Check<'_>, raw: &Value) {
    for q in array(&raw["qualifiers"]) {
        if !charge(c, "/qualifiers") {
            return;
        }
        if q["kind"] != KIND {
            continue;
        }
        let p = format!("/qualifier/{}", path_id(q["id"].as_str().unwrap_or("")));
        let owners: Vec<_> = array(&raw["claims"])
            .iter()
            .filter(|owner| charge(c, &p) && array(&owner["qualifier_ids"]).contains(&q["id"]))
            .collect();
        if raw["kind"] != "source" || owners.len() != 1 || uses(c, raw, &q["id"], &p) != 1 {
            c.issue("invalid_shape", &p);
            continue;
        }
        let owner = owners[0];
        let b = &owner["body"];
        let subject = array(&raw["references"])
            .iter()
            .find(|r| charge(c, &p) && r["id"] == q["body"]["boundary_subject"]);
        if b["kind"] != "event"
            || b["event_kind"] != "measurement"
            || b["status"] != "negated_occurrence"
            || b["aspect"] != "duration"
            || !b["result"].is_null()
            || b["subject"] != q["body"]["boundary_subject"]
            || q["scope_id"] != owner["scope_id"]
            || !subject.is_some_and(|r| r["scope_id"] == q["scope_id"])
        {
            c.issue("invalid_shape", &p);
        }
        let attached: Vec<_> = array(&raw["qualifiers"])
            .iter()
            .filter(|other| charge(c, &p) && array(&owner["qualifier_ids"]).contains(&other["id"]))
            .collect();
        if attached
            .iter()
            .filter(|other| other["kind"] == KIND)
            .count()
            != 1
            || attached
                .iter()
                .any(|other| other["body"]["key"] == "terminal_condition")
        {
            c.issue("invalid_shape", &p);
        }
        for (kind, cid, required_kind, rid, role) in [
            (
                "claim",
                &owner["id"],
                "qualifier",
                &q["id"],
                "configuration",
            ),
            (
                "qualifier",
                &q["id"],
                "reference",
                &q["body"]["boundary_subject"],
                "identity",
            ),
            ("qualifier", &q["id"], "scope", &q["scope_id"], "binding"),
        ] {
            if !array(&raw["dependencies"]).iter().any(|d| {
                charge(c, &p)
                    && d["consumer"]["kind"] == kind
                    && &d["consumer"]["id"] == cid
                    && d["requires"]["kind"] == required_kind
                    && &d["requires"]["id"] == rid
                    && d["role"] == role
            }) {
                c.issue(
                    "invalid_shape",
                    &format!("{p}/dependencies/{required_kind}/{role}"),
                );
            }
        }
    }
}
