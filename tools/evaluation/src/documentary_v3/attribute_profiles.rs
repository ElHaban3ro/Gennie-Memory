//! Literal attribute value profiles. Membership is not unit conversion or semantic fidelity.
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
use std::collections::BTreeSet;

type Issue = (String, String);
const TARGETS: [&str; 3] = ["quantity.amount", "property.value", "event.result"];
const KINDS: [&str; 5] = ["ref", "text", "quantity", "time", "symbol"];

fn shape(v: &Value, keys: &[&str]) -> bool {
    v.as_object().is_some_and(|o| {
        o.len() == keys.len()
            && keys
                .iter()
                .take_while(|_| visit())
                .all(|k| o.contains_key(*k))
    })
}
fn strings(v: &Value) -> Option<BTreeSet<&str>> {
    let a = v.as_array()?;
    let s: BTreeSet<_> = a
        .iter()
        .take_while(|_| visit())
        .map(Value::as_str)
        .collect::<Option<_>>()?;
    (s.len() == a.len() && s.iter().take_while(|_| visit()).all(|x| !x.is_empty())).then_some(s)
}
fn rows(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn issue(code: &str, path: String) -> Issue {
    if !super::context_budget::charge(1, code.len()) {
        return ("request_context_limit".into(), String::new());
    }
    (code.to_owned(), path)
}
fn escaped(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// A malformed profile invalidates the entire vocabulary. Legacy vocabularies have no profile.
pub(super) fn vocabulary_issues(v: &Value) -> Vec<Issue> {
    let Some(p) = v.get("attribute_profiles") else {
        return vec![];
    };
    let base = "/vocabulary/attribute_profiles";
    let mut out = vec![];
    if !shape(p, &["version", "targets", "rows"])
        || p["version"] != "documentary_attribute_profiles_v1"
    {
        return vec![issue("unknown_vocabulary", base.into())];
    }
    let Some(targets) = strings(&p["targets"]) else {
        return vec![issue("unknown_vocabulary", format!("{base}/targets"))];
    };
    if targets.is_empty()
        || !targets
            .iter()
            .take_while(|_| visit())
            .all(|t| TARGETS.contains(t))
    {
        out.push(issue("unknown_vocabulary", format!("{base}/targets")));
    }
    if !p["rows"].is_array() || rows(&p["rows"]).is_empty() {
        out.push(issue("unknown_vocabulary", format!("{base}/rows")));
    }
    let mut seen = BTreeSet::new();
    for (i, r) in rows(&p["rows"]).iter().take_while(|_| visit()).enumerate() {
        if !reserve_path(base, "rows/index") {
            return out;
        }
        let path = format!("{base}/rows/{i}");
        if !shape(
            r,
            &["attribute_id", "target", "term_kinds", "quantity_units"],
        ) {
            out.push(issue("unknown_vocabulary", path));
            continue;
        }
        let attr = r["attribute_id"].as_str().unwrap_or("");
        let target = r["target"].as_str().unwrap_or("");
        let kinds = strings(&r["term_kinds"]);
        let units = strings(&r["quantity_units"]);
        let basic = !attr.is_empty()
            && rows(&v["fixed_attribute_rows"])
                .iter()
                .take_while(|_| visit())
                .any(|x| x == attr)
            && targets.contains(target)
            && seen.insert((attr, target))
            && kinds.as_ref().is_some_and(|s| {
                !s.is_empty() && s.iter().take_while(|_| visit()).all(|x| KINDS.contains(x))
            })
            && units.as_ref().is_some_and(|s| {
                kinds.as_ref().is_some_and(|k| k.contains("quantity")) == !s.is_empty()
            });
        let carrier = target == "event.result"
            || target.split_once('.').is_some_and(|(pred, role)| {
                rows(&v["predicate_rows"])
                    .iter()
                    .take_while(|_| visit())
                    .any(|p| {
                        p["id"] == pred
                            && p["roles"]["attribute"] == "attribute_id"
                            && match p["roles"][role].as_str() {
                                Some("quantity") => kinds
                                    .as_ref()
                                    .is_some_and(|s| s.len() == 1 && s.contains("quantity")),
                                Some("term") => {
                                    kinds.as_ref().is_some_and(|s| !s.contains("symbol"))
                                }
                                Some("ref") => kinds
                                    .as_ref()
                                    .is_some_and(|s| s.len() == 1 && s.contains("ref")),
                                _ => false,
                            }
                    })
            });
        if !basic || !carrier {
            out.push(issue("unknown_vocabulary", path));
        }
    }
    out
}

fn check_value(
    p: &Value,
    target: &str,
    attr: Option<&str>,
    value: Option<&Value>,
    unit: Option<&str>,
    path: String,
    out: &mut BTreeSet<Issue>,
) {
    if !visit() {
        return;
    }
    if !rows(&p["targets"])
        .iter()
        .take_while(|_| visit())
        .any(|t| t == target)
    {
        return;
    }
    if value.is_none() && unit.is_none() {
        return;
    }
    let profile = rows(&p["rows"])
        .iter()
        .take_while(|_| visit())
        .find(|r| r["target"] == target && r["attribute_id"].as_str() == attr && attr.is_some());
    let accepted = profile.is_some_and(|r| {
        let value_ok = value.is_none_or(|t| {
            rows(&r["term_kinds"])
                .iter()
                .take_while(|_| visit())
                .any(|k| Some(k) == t.get("kind") || k == "ref" && t["kind"] == "described_ref")
                && (t["kind"] != "quantity"
                    || rows(&r["quantity_units"])
                        .iter()
                        .take_while(|_| visit())
                        .any(|u| Some(u) == t.get("unit")))
        });
        let unit_ok = unit.is_none_or(|u| {
            rows(&r["term_kinds"])
                .iter()
                .take_while(|_| visit())
                .any(|k| k == "quantity")
                && rows(&r["quantity_units"])
                    .iter()
                    .take_while(|_| visit())
                    .any(|x| x == u)
        });
        value_ok && unit_ok
    });
    if !accepted {
        out.insert(issue("attribute_profile_mismatch", path));
    }
}
fn attribute(atom: &Value) -> Option<&str> {
    rows(&atom["arguments"])
        .iter()
        .take_while(|_| visit())
        .find(|a| a["role"] == "attribute" && a["value"]["kind"] == "symbol")
        .and_then(|a| a["value"]["id"].as_str())
}
fn walk(v: &Value, path: &str, p: &Value, out: &mut BTreeSet<Issue>) {
    if !visit() {
        return;
    }
    if let (Some(pred), Some(args)) = (v["predicate"].as_str(), v["arguments"].as_array()) {
        for (i, a) in args.iter().take_while(|_| visit()).enumerate() {
            if !reserve_path(path, "arguments/value") {
                return;
            }
            if let Some(role) = a["role"].as_str() {
                if !super::context_budget::charge(
                    1,
                    pred.len().saturating_add(role.len()).saturating_add(1),
                ) {
                    return;
                }
                check_value(
                    p,
                    &format!("{pred}.{role}"),
                    attribute(v),
                    a.get("value"),
                    None,
                    format!("{path}/arguments/{i}/value"),
                    out,
                );
            }
        }
    }
    if v["kind"] == "event" && !v["result"].is_null() {
        if !reserve_path(path, "result") {
            return;
        }
        check_value(
            p,
            "event.result",
            v["aspect"].as_str(),
            v.get("result"),
            None,
            format!("{path}/result"),
            out,
        );
    }
    // Restriction members carry the value of answer_role outside the target atom.
    // Validate each member at its own location, retaining unrelated valid carriers.
    if v["kind"] == "value_restriction" {
        if let (Some(pred), Some(role)) =
            (v["target"]["predicate"].as_str(), v["answer_role"].as_str())
        {
            for (i, member) in rows(&v["members"])
                .iter()
                .take_while(|_| visit())
                .enumerate()
            {
                if !reserve_path(path, "members") {
                    return;
                }
                if !super::context_budget::charge(
                    1,
                    pred.len().saturating_add(role.len()).saturating_add(1),
                ) {
                    return;
                }
                check_value(
                    p,
                    &format!("{pred}.{role}"),
                    attribute(&v["target"]),
                    Some(member),
                    None,
                    format!("{path}/members/{i}"),
                    out,
                );
            }
        }
    }
    // Units constrain answer slots even when the pattern intentionally omits that value.
    if v.get("mode").is_some() {
        if let Some(unit) = v["unit"]["symbol"].as_str() {
            if !reserve_path(path, "unit") {
                return;
            }
            if v["mode"] == "event_slot" && v["slot"] == "result" {
                check_value(
                    p,
                    "event.result",
                    v["target"]["aspect"].as_str(),
                    None,
                    Some(unit),
                    format!("{path}/unit"),
                    out,
                );
            } else if let (Some(pred), Some(slot)) = (
                v["target"]["atom"]["predicate"].as_str(),
                v["slot"].as_str(),
            ) {
                if !super::context_budget::charge(
                    1,
                    pred.len().saturating_add(slot.len()).saturating_add(1),
                ) {
                    return;
                }
                check_value(
                    p,
                    &format!("{pred}.{slot}"),
                    attribute(&v["target"]["atom"]),
                    None,
                    Some(unit),
                    format!("{path}/unit"),
                    out,
                );
            }
        }
    }
    match v {
        Value::Object(o) => {
            for (k, x) in charged(o) {
                if !reserve_path(path, k) {
                    return;
                }
                walk(x, &format!("{path}/{}", escaped(k)), p, out);
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().take_while(|_| visit()).enumerate() {
                if !reserve_path(path, "array-index") {
                    return;
                }
                walk(x, &format!("{path}/{i}"), p, out);
            }
        }
        _ => (),
    }
}
/// Incompatibility is local pending information, never an invented conversion or attribute.
pub(super) fn document_issues(raw: &Value, v: &Value) -> Vec<Issue> {
    let Some(p) = v.get("attribute_profiles") else {
        return vec![];
    };
    if !vocabulary_issues(v).is_empty() {
        return vec![];
    }
    let mut out = BTreeSet::new();
    walk(raw, "", p, &mut out);
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn vocab() -> Value {
        json!({"version":"test","predicate_rows":[{"id":"quantity","roles":{"subject":"ref","attribute":"attribute_id","amount":"quantity"},"distributive":true},{"id":"property","roles":{"subject":"ref","attribute":"attribute_id","value":"term"},"distributive":true}],"fixed_attribute_rows":["mass","duration"],"fixed_action_rows":[],"semantic_type_rows":[],"relevance_links":[],"attribute_profiles":{"version":"documentary_attribute_profiles_v1","targets":["quantity.amount","property.value","event.result"],"rows":[{"attribute_id":"mass","target":"quantity.amount","term_kinds":["quantity"],"quantity_units":["kg","g"]},{"attribute_id":"mass","target":"property.value","term_kinds":["quantity"],"quantity_units":["kg","g"]},{"attribute_id":"mass","target":"event.result","term_kinds":["quantity"],"quantity_units":["kg","g"]}]}})
    }
    fn atom(unit: &str) -> Value {
        json!({"predicate":"quantity","arguments":[{"role":"attribute","value":{"kind":"symbol","id":"mass"}},{"role":"amount","value":{"kind":"quantity","decimal":"3","unit":unit,"bound":"exact","evidence":[]}}]})
    }
    #[test]
    fn repeated_profile_scans_consume_the_shared_budget_without_legacy_change() {
        let mut v = vocab();
        v["request_context_profile"] =
            json!({"version":"documentary_request_context_v1","features":["nominal_during"]});
        let raw = json!({"values":vec![atom("kg");128]});
        assert!(
            super::super::context_budget::with_operation(&v, &json!({}), || {
                assert!(super::super::context_budget::charge(999_900, 0));
                document_issues(&raw, &v)
            })
            .is_err()
        );
        assert!(document_issues(&raw, &v).is_empty());
    }
    #[test]
    fn profile_rejects_hours_for_mass_and_accepts_literal_kilograms() {
        let v = vocab();
        assert!(vocabulary_issues(&v).is_empty());
        assert_eq!(
            document_issues(&atom("h"), &v),
            vec![issue(
                "attribute_profile_mismatch",
                "/arguments/1/value".into()
            )]
        );
        assert!(document_issues(&atom("kg"), &v).is_empty());
        assert!(!document_issues(&atom("kilograms"), &v).is_empty());
        let mut x = atom("g");
        x["arguments"][1]["value"]["decimal"] = json!("3000");
        assert!(document_issues(&x, &v).is_empty()); // Membership only, no equation with 3 kg.
    }
    #[test]
    fn legacy_and_missing_row_have_distinct_meanings() {
        let mut v = vocab();
        v.as_object_mut().unwrap().remove("attribute_profiles");
        assert!(vocabulary_issues(&v).is_empty());
        assert!(document_issues(&atom("h"), &v).is_empty());
        v = vocab();
        let mut a = atom("h");
        a["arguments"][0]["value"]["id"] = json!("duration");
        assert!(!document_issues(&a, &v).is_empty());
    }
    #[test]
    fn duplicate_target_row_and_invalid_carrier_are_global_errors() {
        let mut v = vocab();
        let r = v["attribute_profiles"]["rows"][0].clone();
        v["attribute_profiles"]["rows"]
            .as_array_mut()
            .unwrap()
            .push(r);
        assert_eq!(vocabulary_issues(&v)[0].0, "unknown_vocabulary");
        v = vocab();
        v["attribute_profiles"]["targets"] = json!(["quantity.amount", "quantity.amount"]);
        assert!(!vocabulary_issues(&v).is_empty());
        v = vocab();
        v["predicate_rows"][0]["roles"]["attribute"] = json!("ref");
        assert!(!vocabulary_issues(&v).is_empty());
    }
    #[test]
    fn request_missing_answer_slot_is_valid_but_incompatible_unit_is_pending() {
        let v = vocab();
        let mut a = atom("kg");
        a["arguments"].as_array_mut().unwrap().pop();
        let mut r = json!({"mode":"answer","target":{"atom":a},"slot":"amount","unit":null});
        assert!(document_issues(&r, &v).is_empty());
        r["unit"] = json!({"symbol":"h"});
        assert_eq!(
            document_issues(&r, &v),
            vec![issue("attribute_profile_mismatch", "/unit".into())]
        );
        r["unit"] = json!({"symbol":"kg"});
        assert!(document_issues(&r, &v).is_empty());
    }
    #[test]
    fn event_result_requires_literal_profile_and_known_aspect() {
        let v = vocab();
        let mut e = json!({"kind":"event","aspect":"mass","result":{"kind":"quantity","unit":"kg","decimal":"7"}});
        assert!(document_issues(&e, &v).is_empty());
        e["result"]["unit"] = json!("h");
        assert!(!document_issues(&e, &v).is_empty());
        e["aspect"] = Value::Null;
        assert!(!document_issues(&e, &v).is_empty());
        e["result"] = Value::Null;
        assert!(document_issues(&e, &v).is_empty());
    }
    #[test]
    fn restriction_members_use_target_attribute_and_answer_role_with_exact_paths() {
        let v = vocab();
        let mut target = atom("kg");
        target["arguments"].as_array_mut().unwrap().pop();
        let good =
            json!({"kind":"quantity","decimal":"7","unit":"kg","bound":"at_most","evidence":[]});
        let bad =
            json!({"kind":"quantity","decimal":"3","unit":"h","bound":"at_least","evidence":[]});
        let mut d = json!({"claims":[{"body":{"kind":"value_restriction","target":target,"answer_role":"amount","members":[good,bad]}}]});
        assert_eq!(
            document_issues(&d, &v),
            vec![issue(
                "attribute_profile_mismatch",
                "/claims/0/body/members/1".into()
            )]
        );
        // Compatible bounds remain intact, with no fabricated conversion or evidence rewrite.
        assert_eq!(d["claims"][0]["body"]["members"][0]["bound"], "at_most");
        assert_eq!(d["claims"][0]["body"]["members"][1]["unit"], "h");
        d["claims"][0]["body"]["members"][1]["unit"] = json!("g");
        assert!(document_issues(&d, &v).is_empty());
        d["claims"][0]["body"]["target"]["arguments"][0]["value"]["id"] = json!("duration");
        assert_eq!(document_issues(&d, &v).len(), 2);
        d["claims"][0]["body"]["members"] = json!([]);
        assert!(document_issues(&d, &v).is_empty());
    }
    #[test]
    fn restriction_property_values_reject_wrong_kind_and_preserve_legacy_behavior() {
        let mut v = vocab();
        v["attribute_profiles"]["rows"][1]["term_kinds"] = json!(["text"]);
        v["attribute_profiles"]["rows"][1]["quantity_units"] = json!([]);
        let d = json!({"claims":[{"body":{"kind":"value_restriction","target":{"predicate":"property","arguments":[{"role":"attribute","value":{"kind":"symbol","id":"mass"}}]},"answer_role":"value","members":[{"kind":"text","value":"heavy"},{"kind":"quantity","unit":"kg","decimal":"7","bound":"exact"}]}}]});
        assert_eq!(
            document_issues(&d, &v),
            vec![issue(
                "attribute_profile_mismatch",
                "/claims/0/body/members/1".into()
            )]
        );
        v.as_object_mut().unwrap().remove("attribute_profiles");
        assert!(document_issues(&d, &v).is_empty());
    }
    #[test]
    fn nonquantity_profile_has_no_units_and_rejects_quantity_values() {
        let mut v = vocab();
        v["attribute_profiles"]["rows"][1]["term_kinds"] = json!(["text"]);
        v["attribute_profiles"]["rows"][1]["quantity_units"] = json!([]);
        assert!(vocabulary_issues(&v).is_empty());
        let a = json!({"predicate":"property","arguments":[{"role":"attribute","value":{"kind":"symbol","id":"mass"}},{"role":"value","value":{"kind":"text","value":"heavy"}}]});
        assert!(document_issues(&a, &v).is_empty());
        let mut b = a;
        b["arguments"][1]["value"] = json!({"kind":"quantity","unit":"kg"});
        assert!(!document_issues(&b, &v).is_empty());
        v["attribute_profiles"]["rows"][1]["quantity_units"] = json!(["kg"]);
        assert!(!vocabulary_issues(&v).is_empty());
    }
}
