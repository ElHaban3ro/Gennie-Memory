//! Closed nominal syntax; its lexical tail never denotes a global paper object.
use super::*;
use crate::documentary_v3::nominal_de::{self, KIND};
pub(super) fn node(c: &mut Check<'_>, t: &str, v: &Value, p: &str, depth: usize) -> bool {
    if t != "Link" || v["kind"] != KIND {
        return false;
    }
    if !super::super::request_context_profile::permits(c.vocab, KIND)
        || !c.shape(v, p, &["kind", "target", "evidence"])
    {
        c.issue("invalid_shape", p);
        return true;
    }
    c.string(&v["target"], &format!("{p}/target"));
    if !v["evidence"].as_array().is_some_and(|a| a.len() == 1) {
        c.issue("invalid_shape", p);
    }
    c.list(&v["evidence"], &format!("{p}/evidence"), "Evidence", depth);
    true
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
        .is_some_and(|((a, b), (c, d))| a <= c && d <= b && c < d)
}
pub(super) fn document(c: &mut Check<'_>, raw: &Value) {
    if !super::super::request_context_profile::permits(c.vocab, KIND) {
        return;
    }
    let has = array(&raw["references"])
        .iter()
        .any(|r| array(&r["links"]).iter().any(|l| l["kind"] == KIND));
    if !has {
        return;
    }
    for r in charged(array(&raw["references"])) {
        for link in charged(array(&r["links"])) {
            if link["kind"] != KIND {
                continue;
            }
            let p = format!(
                "/reference/{}/links",
                path_id(r["id"].as_str().unwrap_or(""))
            );
            let target = charged(array(&raw["references"])).find(|t| t["id"] == link["target"]);
            let Some(target) = target else {
                c.issue("dangling_id", &p);
                continue;
            };
            if target["denotation"] != "entity"
                || !target["name"].is_null()
                || !target["declared_class"]["quote"]
                    .as_str()
                    .is_some_and(|s| !s.trim().is_empty())
            {
                c.issue("invalid_shape", &p);
            }
            let head = if r["name"].is_null() {
                &r["declared_class"]
            } else {
                &r["name"]
            };
            let tail = &target["declared_class"];
            let evidence = &link["evidence"][0];
            let construction = span(head)
                .zip(span(tail))
                .is_some_and(|((hs, he), (ts, te))| {
                    hs < he
                        && he < ts
                        && ts < te
                        && span(evidence) == Some((hs, te))
                        && c.text.get(he..ts).is_some_and(|between| {
                            let trimmed = between.trim_matches(char::is_whitespace);
                            trimmed == "de"
                                && between.starts_with(char::is_whitespace)
                                && between.ends_with(char::is_whitespace)
                        })
                        && c.text.get(hs..te) == evidence["quote"].as_str()
                });
            if r["id"] == target["id"]
                || r["scope_id"] != target["scope_id"]
                || !contains(&r["mention"], evidence)
                || !contains(&target["mention"], tail)
                || !construction
            {
                c.issue("invalid_shape", &p);
            }
            if nominal_de::descriptor_key(raw, target["id"].as_str().unwrap_or("")).is_none() {
                c.issue("unsupported_nominal_complement", &p);
            }
            if !r["binding_scope_id"].is_null()
                || array(&raw["scopes"])
                    .iter()
                    .any(|s| s["id"] == r["scope_id"] && s["kind"] != "document")
            {
                c.issue("unsupported_nominal_complement", &p);
            }
        }
    }
    // General claims on a lexical tail would strengthen the source. Only local
    // qualitative metadata has a validated shape; execution keeps it contextual.
    for (table, kind) in [("claims", "claim"), ("requests", "request")] {
        for owner in charged(array(&raw[table])) {
            let mut used = BTreeSet::new();
            refs(owner, &mut used);
            if used
                .iter()
                .any(|rid| nominal_de::targets(raw, rid).is_none_or(|xs| !xs.is_empty()))
                && array(&raw["scopes"])
                    .iter()
                    .any(|s| s["id"] == owner["scope_id"] && s["kind"] != "document")
            {
                c.issue(
                    "unsupported_nominal_complement",
                    &format!("/{kind}/{}", path_id(owner["id"].as_str().unwrap_or(""))),
                );
            }
            if used.iter().any(|rid| nominal_de::is_tail(raw, rid))
                && !(kind == "claim" && nominal_de::lexical_metadata(raw, owner)
                    || kind == "request" && nominal_de::local_metadata_request(raw, owner))
            {
                c.issue(
                    "unsupported_nominal_complement",
                    &format!("/{kind}/{}", path_id(owner["id"].as_str().unwrap_or(""))),
                );
            }
        }
    }
}
