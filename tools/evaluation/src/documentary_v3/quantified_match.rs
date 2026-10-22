//! Correspondence of bounded documentary patterns, never global entity identity.
//!
//! Call only on validated documents. The caller must first verify the complete
//! binder alpha-map (quantifier order, cardinality, witness relations, closures),
//! and still enforce eligibility, qualifiers, normative scope and conflicts.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn rows<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}
fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}
fn reference<'a>(doc: &'a Value, rid: &str) -> Option<&'a Value> {
    rows(doc, "references")
        .iter()
        .find(|r| text(r, "id") == rid)
}

/// Explicit local aliases share every restrictive association. No association
/// edge itself enters this closure or establishes identity.
fn local_component(doc: &Value, start: &str) -> BTreeSet<String> {
    let mut component = BTreeSet::from([start.to_owned()]);
    loop {
        let before = component.len();
        for edge in rows(doc, "identity") {
            if text(edge, "kind") == "same" {
                let (a, b) = (text(edge, "left"), text(edge, "right"));
                if component.contains(a) {
                    component.insert(b.to_owned());
                }
                if component.contains(b) {
                    component.insert(a.to_owned());
                }
            }
        }
        for r in rows(doc, "references") {
            if r["anaphora"]["status"] == "asserted"
                && rows(&r["anaphora"], "antecedent_ids").len() == 1
            {
                if let Some(antecedent) = r["anaphora"]["antecedent_ids"][0].as_str() {
                    if component.contains(text(r, "id")) {
                        component.insert(antecedent.to_owned());
                    }
                    if component.contains(antecedent) {
                        component.insert(text(r, "id").to_owned());
                    }
                }
            }
        }
        if before == component.len() {
            return component;
        }
    }
}
pub(super) fn associated_targets<'a>(doc: &'a Value, rid: &str) -> Option<Vec<&'a str>> {
    let component = local_component(doc, rid);
    if rows(doc, "identity").iter().any(|edge| {
        text(edge, "kind") == "different"
            && component.contains(text(edge, "left"))
            && component.contains(text(edge, "right"))
    }) {
        return None;
    }
    let mut seen = BTreeSet::new();
    let mut targets = Vec::new();
    for r in rows(doc, "references")
        .iter()
        .filter(|r| component.contains(text(r, "id")))
    {
        for link in rows(r, "links")
            .iter()
            .filter(|l| text(l, "kind") == "associated_with")
        {
            let target = text(link, "target");
            reference(doc, target)?;
            let aliases = local_component(doc, target);
            if seen.insert(aliases) {
                targets.push(target);
            }
        }
    }
    Some(targets)
}

/// Association anchors with additional qualifier dependencies require a future
/// context proof. Conservatively leave them pending instead of using their name.
pub(super) fn association_anchor_ready(doc: &Value, rid: &str) -> bool {
    let mut pending = local_component(doc, rid)
        .into_iter()
        .map(|id| ("reference".to_owned(), id))
        .collect::<Vec<_>>();
    let mut checked = BTreeSet::new();
    while let Some((kind, id)) = pending.pop() {
        if !checked.insert((kind.clone(), id.clone())) {
            continue;
        }
        if kind == "reference" && reference(doc, &id).is_none_or(|r| !r["anaphora"].is_null()) {
            return false;
        }
        for dep in rows(doc, "dependencies")
            .iter()
            .filter(|d| text(&d["consumer"], "kind") == kind && text(&d["consumer"], "id") == id)
        {
            let required = &dep["requires"];
            let rk = text(required, "kind");
            if rk == "qualifier" || rk == "claim" {
                return false;
            }
            pending.push((rk.to_owned(), text(required, "id").to_owned()));
        }
    }
    true
}

/// Structural correspondence inside an already bounded quantified pattern.
///
/// Anonymous descriptions require an explicit class and a nonempty chain of
/// `owned_by`/`part_of`/`associated_with` links ending in the same named anchor or a verified bound
/// variable. Bare equal classes, equal mention strings and IDs do not suffice.
/// This proves interchangeable *pattern constraints*, not real-world identity;
/// never use it to extend the engine's global `identity` relation.
pub(super) fn reference_pattern_matches(
    source: &Value,
    source_id: &str,
    question: &Value,
    question_id: &str,
    namespace: &str,
    verified_alpha_map: &BTreeMap<String, String>,
) -> bool {
    if namespace.is_empty()
        || text(source, "domain_namespace") != namespace
        || text(question, "domain_namespace") != namespace
    {
        return false;
    }
    fn compare(
        source: &Value,
        a_id: &str,
        question: &Value,
        b_id: &str,
        map: &BTreeMap<String, String>,
        visited: &mut BTreeSet<(String, String)>,
        depth: usize,
    ) -> bool {
        if depth > 16 || !visited.insert((a_id.to_owned(), b_id.to_owned())) {
            return false;
        }
        // The nominal carrier has no quantified correspondence contract yet.
        if super::context_budget::nominal_enabled()
            && (super::nominal_de::targets(source, a_id).is_none_or(|xs| !xs.is_empty())
                || super::nominal_de::targets(question, b_id).is_none_or(|xs| !xs.is_empty()))
        {
            return false;
        }
        let (Some(a), Some(b)) = (reference(source, a_id), reference(question, b_id)) else {
            return false;
        };
        if a["denotation"] != b["denotation"]
            || a["declared_class"]["quote"] != b["declared_class"]["quote"]
            || a["semantic_type"]["type_id"] != b["semantic_type"]["type_id"]
            || !a["anaphora"].is_null()
            || !b["anaphora"].is_null()
        {
            return false;
        }
        // A local explicit difference must not be overridden by a description.
        for (doc, rid) in [(source, a_id), (question, b_id)] {
            if rows(doc, "identity").iter().any(|edge| {
                text(edge, "kind") == "different"
                    && (text(edge, "left") == rid || text(edge, "right") == rid)
            }) {
                return false;
            }
        }
        let mapped = map.get(a_id);
        if mapped.is_some_and(|mapped| mapped != b_id)
            || mapped.is_some()
                && (!a["binding_scope_id"].is_string() || !b["binding_scope_id"].is_string())
        {
            return false;
        }
        let (Some(a_context), Some(b_context)) = (
            associated_targets(source, a_id),
            associated_targets(question, b_id),
        ) else {
            return false;
        };
        if a_context.len() != b_context.len() {
            return false;
        }
        let mut used_context = BTreeSet::new();
        for target in a_context {
            let matched = b_context.iter().enumerate().find(|(index, other)| {
                !used_context.contains(index)
                    && association_anchor_ready(source, target)
                    && association_anchor_ready(question, other)
                    && compare(
                        source,
                        target,
                        question,
                        other,
                        map,
                        &mut visited.clone(),
                        depth + 1,
                    )
            });
            let Some((index, _)) = matched else {
                return false;
            };
            used_context.insert(index);
        }
        if mapped.is_some() {
            return true;
        }
        let a_name = text(&a["name"], "quote");
        let b_name = text(&b["name"], "quote");
        if a_name.is_empty() != b_name.is_empty() || !a_name.is_empty() && a_name != b_name {
            return false;
        }
        let a_links = rows(a, "links");
        let b_links = rows(b, "links");
        if a_links.len() != b_links.len() {
            return false;
        }
        if a_name.is_empty()
            && (text(&a["declared_class"], "quote").is_empty() || a_links.is_empty())
        {
            return false;
        }
        let mut used = BTreeSet::new();
        for a_link in a_links {
            if !matches!(
                text(a_link, "kind"),
                "owned_by" | "part_of" | "associated_with"
            ) {
                return false;
            }
            let mut matched = None;
            for (index, b_link) in b_links.iter().enumerate() {
                if used.contains(&index) || a_link["kind"] != b_link["kind"] {
                    continue;
                }
                let mut next = visited.clone();
                if compare(
                    source,
                    text(a_link, "target"),
                    question,
                    text(b_link, "target"),
                    map,
                    &mut next,
                    depth + 1,
                ) {
                    matched = Some(index);
                    break;
                }
            }
            let Some(index) = matched else { return false };
            used.insert(index);
        }
        true
    }
    compare(
        source,
        source_id,
        question,
        question_id,
        verified_alpha_map,
        &mut BTreeSet::new(),
        0,
    )
}

fn one_universal<'a>(doc: &'a Value, scope_id: &str) -> Option<&'a Value> {
    let mut current = scope_id;
    let mut binder = None;
    let mut seen = BTreeSet::new();
    while !current.is_empty() {
        if !seen.insert(current.to_owned()) {
            return None;
        }
        let scope = rows(doc, "scopes")
            .iter()
            .find(|s| text(s, "id") == current)?;
        if !scope["branch_id"].is_null() {
            return None;
        }
        if !scope["binding"].is_null() {
            if binder.is_some() || scope["binding"]["quantifier"] != "forall" {
                return None;
            }
            binder = Some(&scope["binding"]);
        }
        current = text(scope, "parent_id");
    }
    binder
}
fn same_literal(a: &Value, b: &Value) -> bool {
    if a["kind"] != b["kind"] {
        return false;
    }
    match text(a, "kind") {
        "symbol" => a["id"] == b["id"],
        "text" => a["value"] == b["value"],
        "time" => a["value"] == b["value"] && a["format"] == b["format"],
        "quantity" => {
            a["decimal"] == b["decimal"] && a["unit"] == b["unit"] && a["bound"] == b["bound"]
        }
        _ => false,
    }
}

/// IDs of question `common` qualifiers discharged by a constant exact quantity
/// explicitly asserted under the same single universal domain.
///
/// A claim about B3, an existential, a branch, a varying reference, a bound,
/// or an unmatched universal domain cannot establish a common literal value.
/// Other qualifiers and all engine quality checks remain the caller's duty.
pub(super) fn common_quantity_qualifier_ids(
    source: &Value,
    claim: &Value,
    question: &Value,
    request: &Value,
    namespace: &str,
    verified_alpha_map: &BTreeMap<String, String>,
    described_matches: &dyn Fn(&Value, &Value) -> bool,
) -> Vec<String> {
    let fail = Vec::new;
    if request["mode"] != "answer" || claim["body"]["kind"] != "proposition" {
        return fail();
    }
    let Some(sb) = one_universal(source, text(claim, "scope_id")) else {
        return fail();
    };
    let Some(qb) = one_universal(question, text(request, "scope_id")) else {
        return fail();
    };
    if verified_alpha_map
        .get(text(sb, "variable"))
        .map(String::as_str)
        != Some(text(qb, "variable"))
        || sb["domain_ref"].is_null()
        || qb["domain_ref"].is_null()
        || !reference_pattern_matches(
            source,
            text(sb, "domain_ref"),
            question,
            text(qb, "domain_ref"),
            namespace,
            verified_alpha_map,
        )
    {
        return fail();
    }
    let p = &claim["body"]["value"];
    let target = &request["target"];
    let atom = &p["atom"];
    let pattern = &target["atom"];
    if p["judgment"] != "fact"
        || target["judgment"] != "fact"
        || p["status"] != target["status"]
        || atom["polarity"] != "positive"
        || pattern["polarity"] != "positive"
        || atom["predicate"] != pattern["predicate"]
    {
        return fail();
    }
    let slot = text(request, "slot");
    let Some(answer) = rows(atom, "arguments")
        .iter()
        .find(|a| text(a, "role") == slot)
        .map(|a| &a["value"])
    else {
        return fail();
    };
    if answer["kind"] != "quantity"
        || answer["bound"] != "exact"
        || text(answer, "decimal").is_empty()
        || rows(pattern, "arguments")
            .iter()
            .any(|a| text(a, "role") == slot)
        || rows(atom, "arguments").len() != rows(pattern, "arguments").len() + 1
    {
        return fail();
    }
    for arg in rows(pattern, "arguments") {
        let Some(source_arg) = rows(atom, "arguments")
            .iter()
            .find(|a| a["role"] == arg["role"])
        else {
            return fail();
        };
        let a = &source_arg["value"];
        let b = &arg["value"];
        let matches = if a["kind"] == "ref" && b["kind"] == "ref" {
            reference_pattern_matches(
                source,
                text(a, "id"),
                question,
                text(b, "id"),
                namespace,
                verified_alpha_map,
            )
        } else if a["kind"] == "described_ref" && b["kind"] == "described_ref" {
            described_matches(a, b)
        } else {
            same_literal(a, b)
        };
        if !matches {
            return fail();
        }
    }
    // A guard is conditional, even when its lexical quantity is constant.
    for qid in rows(claim, "qualifier_ids")
        .iter()
        .chain(rows(p, "qualifier_ids"))
    {
        let Some(q) = rows(source, "qualifiers").iter().find(|q| q["id"] == *qid) else {
            return fail();
        };
        if q["resolution"] != "known" || q["kind"] == "guard" {
            return fail();
        }
    }
    rows(request, "qualifier_ids")
        .iter()
        .filter_map(|qid| {
            let q = rows(question, "qualifiers")
                .iter()
                .find(|q| q["id"] == *qid)?;
            (q["kind"] == "configuration"
                && q["resolution"] == "known"
                && q["body"]["key"] == "quantity_extent"
                && q["body"]["value"]["kind"] == "text"
                && q["body"]["value"]["value"] == "común")
                .then(|| text(q, "id").to_owned())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn reference(
        id: &str,
        name: Value,
        class: Value,
        denotation: &str,
        links: Value,
        binding: Value,
    ) -> Value {
        json!({"id":id,"name":name,"declared_class":class,"denotation":denotation,"links":links,"semantic_type":null,"anaphora":null,"binding_scope_id":binding})
    }
    fn fixtures() -> (Value, Value, Value, Value, BTreeMap<String, String>) {
        let mut source = json!({"domain_namespace":"controls","identity":[],"references":[
            reference("anchor",json!({"quote":"Q4"}),Value::Null,"entity",json!([]),Value::Null),
            reference("domain",Value::Null,json!({"quote":"archivos crudos"}),"class",json!([{"kind":"part_of","target":"anchor"}]),Value::Null),
            reference("file",Value::Null,json!({"quote":"archivos crudos"}),"entity",json!([]),json!("all-files"))
        ],"scopes":[{"id":"root","parent_id":null,"branch_id":null,"binding":null},{"id":"all-files","parent_id":"root","branch_id":null,"binding":{"variable":"file","quantifier":"forall","domain_ref":"domain"}}],"qualifiers":[]});
        let question = source.clone();
        let claim = json!({"id":"c","scope_id":"all-files","qualifier_ids":[],"body":{"kind":"proposition","value":{"atom":{"predicate":"retention_for","polarity":"positive","arguments":[{"role":"subject","value":{"kind":"ref","id":"file"}},{"role":"duration","value":{"kind":"quantity","decimal":"18","unit":"días","bound":"exact"}}]},"judgment":"fact","status":"state","qualifier_ids":[]}}});
        let mut question = question;
        question["qualifiers"] = json!([{"id":"common","kind":"configuration","resolution":"known","body":{"key":"quantity_extent","value":{"kind":"text","value":"común"}}}]);
        let request = json!({"id":"r","scope_id":"all-files","mode":"answer","slot":"duration","target":{"atom":{"predicate":"retention_for","polarity":"positive","arguments":[{"role":"subject","value":{"kind":"ref","id":"file"}}]},"judgment":"fact","status":"state"},"qualifier_ids":["common"]});
        // The map is an input proven by the caller's complete alpha validation.
        let map = BTreeMap::from([("file".to_owned(), "file".to_owned())]);
        source["claims"] = json!([claim.clone()]);
        (source, question, claim, request, map)
    }

    #[test]
    fn same_anchored_universal_domain_is_a_pattern_correspondence() {
        let (source, question, _, _, map) = fixtures();
        assert!(reference_pattern_matches(
            &source, "domain", &question, "domain", "controls", &map
        ));
        assert!(!reference_pattern_matches(
            &source,
            "domain",
            &question,
            "domain",
            "other-namespace",
            &map
        ));
    }

    #[test]
    fn equal_classes_without_anchor_or_with_another_anchor_do_not_match() {
        let (source, mut question, _, _, map) = fixtures();
        question["references"][0]["name"]["quote"] = json!("Q5");
        assert!(!reference_pattern_matches(
            &source, "domain", &question, "domain", "controls", &map
        ));
        question["references"][1]["links"] = json!([]);
        assert!(!reference_pattern_matches(
            &source, "domain", &question, "domain", "controls", &map
        ));
    }

    #[test]
    fn associated_domain_pattern_is_directed_and_does_not_upgrade_relation_kind() {
        let (mut source, mut question, _, _, map) = fixtures();
        source["references"][1]["links"][0]["kind"] = json!("associated_with");
        question["references"][1]["links"][0]["kind"] = json!("associated_with");
        assert!(reference_pattern_matches(
            &source, "domain", &question, "domain", "controls", &map
        ));
        for stronger in ["owned_by", "part_of", "located_at", "member_of"] {
            question["references"][1]["links"][0]["kind"] = json!(stronger);
            assert!(!reference_pattern_matches(
                &source, "domain", &question, "domain", "controls", &map
            ));
        }
        question["references"][1]["links"][0]["kind"] = json!("associated_with");
        question["references"][0]["name"]["quote"] = json!("Q5");
        assert!(!reference_pattern_matches(
            &source, "domain", &question, "domain", "controls", &map
        ));
        question["references"][0]["name"]["quote"] = json!("Q4");
        question["references"][1]["links"] = json!([]);
        question["references"][0]["links"] = json!([{"kind":"associated_with","target":"domain"}]);
        assert!(!reference_pattern_matches(
            &source, "domain", &question, "domain", "controls", &map
        ));
    }
    #[test]
    fn bound_alpha_variable_cannot_ignore_context_and_cycles_do_not_prove_identity() {
        let (mut source, mut question, _, _, map) = fixtures();
        for doc in [&mut source, &mut question] {
            doc["references"][2]["links"] = json!([{"kind":"associated_with","target":"anchor"}]);
        }
        assert!(reference_pattern_matches(
            &source, "file", &question, "file", "controls", &map
        ));
        question["references"][0]["name"]["quote"] = json!("Q5");
        assert!(!reference_pattern_matches(
            &source, "file", &question, "file", "controls", &map
        ));
        question["references"][0]["name"]["quote"] = json!("Q4");
        for doc in [&mut source, &mut question] {
            doc["references"][0]["links"] = json!([{"kind":"associated_with","target":"anchor"}]);
        }
        assert!(!reference_pattern_matches(
            &source, "file", &question, "file", "controls", &map
        ));
    }
    #[test]
    fn literal_exact_quantity_under_a_matched_universal_discharges_only_common() {
        let (source, question, claim, request, map) = fixtures();
        assert_eq!(
            common_quantity_qualifier_ids(
                &source,
                &claim,
                &question,
                &request,
                "controls",
                &map,
                &|_, _| false
            ),
            vec!["common"]
        );
        let mut bounded = claim.clone();
        bounded["body"]["value"]["atom"]["arguments"][1]["value"]["bound"] = json!("at_least");
        assert!(
            common_quantity_qualifier_ids(
                &source,
                &bounded,
                &question,
                &request,
                "controls",
                &map,
                &|_, _| false
            )
            .is_empty()
        );
    }

    #[test]
    fn individual_or_existential_claim_cannot_prove_a_universal_common_quantity() {
        let (mut source, question, mut claim, request, map) = fixtures();
        claim["scope_id"] = json!("root");
        assert!(
            common_quantity_qualifier_ids(
                &source,
                &claim,
                &question,
                &request,
                "controls",
                &map,
                &|_, _| false
            )
            .is_empty()
        );
        claim["scope_id"] = json!("all-files");
        source["scopes"][1]["binding"]["quantifier"] = json!("exists");
        assert!(
            common_quantity_qualifier_ids(
                &source,
                &claim,
                &question,
                &request,
                "controls",
                &map,
                &|_, _| false
            )
            .is_empty()
        );
    }

    #[test]
    fn guarded_universal_quantity_does_not_become_unconditional_common() {
        let (mut source, question, mut claim, request, map) = fixtures();
        source["qualifiers"] = json!([{"id":"guard","kind":"guard","resolution":"known"}]);
        claim["qualifier_ids"] = json!(["guard"]);
        assert!(
            common_quantity_qualifier_ids(
                &source,
                &claim,
                &question,
                &request,
                "controls",
                &map,
                &|_, _| false
            )
            .is_empty()
        );
    }
}
