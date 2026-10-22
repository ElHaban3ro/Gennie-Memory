//! Evidence-bound opt-in context nodes. Unknown actors remain nonassertive patterns.
use super::*;
use crate::documentary_v3::request_context_profile as profile;

fn id(c: &mut Check<'_>, v: &Value, p: &str) {
    if !v
        .as_str()
        .is_some_and(|s| !s.trim().is_empty() && s.len() <= 128)
    {
        c.issue("invalid_shape", p);
    }
}
fn evidence(c: &mut Check<'_>, v: &Value, p: &str, depth: usize) {
    if !v.as_array().is_some_and(|a| !a.is_empty() && a.len() <= 32) {
        c.issue("invalid_shape", p);
    }
    c.list(v, p, "Evidence", depth);
}
fn simple_ref(c: &mut Check<'_>, v: &Value, p: &str, depth: usize) {
    if !c.shape(v, p, &["kind", "id"]) || v["kind"] != "ref" {
        c.issue("invalid_shape", p);
        return;
    }
    id(c, &v["id"], p);
    c.node("Term", v, p, depth + 1);
}
/// Called after Check has charged the current node. Children always reenter Check.
pub(super) fn node(c: &mut Check<'_>, t: &str, v: &Value, p: &str, depth: usize) -> bool {
    match t {
        "RequestContext" => {
            let kind = v["kind"].as_str().unwrap_or("");
            if !profile::permits(c.vocab, kind) {
                c.issue("invalid_shape", p);
                return true;
            }
            if kind == "lexical_clause_context" {
                if !c.shape(
                    v,
                    p,
                    &[
                        "id",
                        "kind",
                        "preposition",
                        "complement",
                        "complement_evidence",
                        "phrase_evidence",
                        "external_clause_evidence",
                        "internal_attachment",
                        "specialization",
                    ],
                ) {
                    return true;
                }
                id(c, &v["id"], &format!("{p}/id"));
                simple_ref(c, &v["complement"], &format!("{p}/complement"), depth);
                for key in [
                    "preposition",
                    "complement_evidence",
                    "phrase_evidence",
                    "external_clause_evidence",
                ] {
                    c.node("Evidence", &v[key], &format!("{p}/{key}"), depth + 1);
                }
                let a = &v["internal_attachment"];
                match a["kind"].as_str() {
                    Some("whole_clause_only") => {
                        c.shape(a, &format!("{p}/internal_attachment"), &["kind"]);
                    }
                    Some("source_underspecified") => {
                        c.shape(
                            a,
                            &format!("{p}/internal_attachment"),
                            &["kind", "boundary_evidence"],
                        );
                        evidence(
                            c,
                            &a["boundary_evidence"],
                            &format!("{p}/internal_attachment/boundary_evidence"),
                            depth,
                        );
                    }
                    _ => c.issue("invalid_shape", p),
                }
                if !v["specialization"].is_null()
                    || !super::super::lexical_clause_context::lexical_evidence_valid(c.text, v)
                {
                    c.issue("invalid_shape", p);
                }
                return true;
            }
            let fields = match kind {
                "nominal_during" => &["id", "kind", "anchor", "evidence"][..],
                "normative_action_relation" => {
                    &["id", "kind", "relation", "action", "evidence"][..]
                }
                _ => {
                    c.issue("invalid_shape", p);
                    return true;
                }
            };
            if !c.shape(v, p, fields) {
                return true;
            }
            id(c, &v["id"], &format!("{p}/id"));
            evidence(c, &v["evidence"], &format!("{p}/evidence"), depth);
            if kind == "nominal_during" {
                simple_ref(c, &v["anchor"], &format!("{p}/anchor"), depth);
            } else {
                c.node(
                    "ContextRelation",
                    &v["relation"],
                    &format!("{p}/relation"),
                    depth + 1,
                );
                c.node(
                    "SubordinateAction",
                    &v["action"],
                    &format!("{p}/action"),
                    depth + 1,
                );
            }
        }
        "ContextRelation" => {
            let unresolved = v["kind"] == "unresolved";
            let fields = if unresolved {
                &["kind", "alternatives", "evidence"][..]
            } else {
                &["kind", "evidence"][..]
            };
            if !c.shape(v, p, fields) {
                return true;
            }
            c.choice(
                &v["kind"],
                p,
                &["purpose", "necessary_condition", "unresolved"],
            );
            if unresolved {
                if v["alternatives"] != json!(["purpose", "necessary_condition"]) {
                    c.issue("invalid_shape", p);
                }
                c.issue("request_context_ambiguous", p);
            }
            evidence(c, &v["evidence"], &format!("{p}/evidence"), depth);
        }
        "SubordinateAction" => {
            if !c.shape(v, p, &["action_kind", "actor", "destination", "evidence"]) {
                return true;
            }
            let action = &v["action_kind"];
            if !c.shape(
                action,
                &format!("{p}/action_kind"),
                &["kind", "id", "evidence"],
            ) || action["kind"] != "symbol"
                || !array(&c.vocab["fixed_action_rows"]).contains(&action["id"])
            {
                c.issue("unknown_vocabulary", &format!("{p}/action_kind"));
            }
            id(c, &action["id"], &format!("{p}/action_kind/id"));
            c.node("Term", action, &format!("{p}/action_kind"), depth + 1);
            simple_ref(c, &v["destination"], &format!("{p}/destination"), depth);
            c.node(
                "ContextParticipant",
                &v["actor"],
                &format!("{p}/actor"),
                depth + 1,
            );
            evidence(c, &v["evidence"], &format!("{p}/evidence"), depth);
        }
        "ContextParticipant" => {
            let fields = match v["kind"].as_str() {
                Some("reference") => &["kind", "value", "evidence"][..],
                Some("owner_slot") => &["kind", "role", "evidence"][..],
                Some("unresolved") => &["kind", "reason", "evidence"][..],
                _ => {
                    c.issue("invalid_shape", p);
                    return true;
                }
            };
            if !c.shape(v, p, fields) {
                return true;
            }
            if v["kind"] == "reference" {
                simple_ref(c, &v["value"], &format!("{p}/value"), depth);
            } else if v["kind"] == "owner_slot" {
                c.choice(&v["role"], p, &["actor"]);
            } else {
                c.choice(
                    &v["reason"],
                    p,
                    &["unspecified", "ambiguous_control", "extraction_unresolved"],
                );
                match v["reason"].as_str() {
                    Some("ambiguous_control") => c.issue("request_context_ambiguous", p),
                    Some("extraction_unresolved") => {
                        c.issue("request_context_extraction_unresolved", p)
                    }
                    _ => {} // Faithful absence is complete documentary representation, not a binding.
                }
            }
            evidence(c, &v["evidence"], &format!("{p}/evidence"), depth);
        }
        "NormativeOpenRole" => {
            if !c.shape(v, p, &["role", "class_filter", "evidence"]) {
                return true;
            }
            c.choice(&v["role"], p, &["actor"]);
            c.node(
                "ClassFilter",
                &v["class_filter"],
                &format!("{p}/class_filter"),
                depth + 1,
            );
            evidence(c, &v["evidence"], &format!("{p}/evidence"), depth);
        }
        _ => return false,
    }
    true
}

pub(super) fn contexts(c: &mut Check<'_>, owner: &Value, p: &str, depth: usize) {
    let contexts = &owner["context_bindings"];
    if !contexts
        .as_array()
        .is_some_and(|a| !a.is_empty() && a.len() <= 8)
    {
        c.issue("invalid_shape", p);
    }
    let mut ids = BTreeSet::new();
    for (index, context) in array(contexts).iter().enumerate() {
        if !ids.insert(context["id"].to_string()) {
            c.issue("invalid_shape", p);
        }
        c.node(
            "RequestContext",
            context,
            &format!("{p}/{index}"),
            depth + 1,
        );
    }
}
fn argument<'a>(atom: &'a Value, role: &str) -> Option<&'a Value> {
    array(&atom["arguments"])
        .iter()
        .find(|a| a["role"] == role)
        .map(|a| &a["value"])
}
fn exact_roles(atom: &Value, roles: &[&str]) -> bool {
    array(&atom["arguments"]).len() == roles.len()
        && roles.iter().all(|r| argument(atom, r).is_some())
}
fn normative_target(target: &Value) -> bool {
    target["atom"]["predicate"] == "action"
        && target["atom"]["polarity"] == "positive"
        && target["judgment"] == "required"
        && target["status"] == "state"
}
/// All references owned solely by the new carrier have implicit graph edges.
pub(super) fn implicit_refs(owner: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if !charge_tree(&owner["context_bindings"]) {
        return out;
    }
    refs(&owner["context_bindings"], &mut out);
    if matches!(
        owner["body"]["kind"].as_str(),
        Some("normative_open_pattern" | "linguistic_composition")
    ) {
        if !charge_tree(&owner["body"]) {
            return out;
        }
        refs(&owner["body"], &mut out);
    }
    out
}
pub(super) fn has_carrier(owner: &Value) -> bool {
    owner.get("context_bindings").is_some()
        || matches!(
            owner["body"]["kind"].as_str(),
            Some("normative_open_pattern" | "linguistic_composition")
        )
}

pub(super) fn document(c: &mut Check<'_>, raw: &Value) {
    if !profile::enabled(c.vocab) {
        return;
    }
    for (table, kind) in [("claims", "claim"), ("requests", "request")] {
        for owner in array(&raw[table]) {
            if !has_carrier(owner)
                || (owner["body"]["kind"] == "linguistic_composition"
                    && owner.get("context_bindings").is_none())
            {
                continue;
            }
            let p = format!("/{kind}/{}", path_id(owner["id"].as_str().unwrap_or("")));
            if !super::super::context_budget::charge(1, 0) {
                c.limit_exceeded = true;
                c.issue("request_context_limit", &p);
                return;
            }
            if !super::super::context_budget::charge(
                array(&raw["references"]).len().saturating_mul(2),
                0,
            ) {
                c.limit_exceeded = true;
                c.issue("request_context_limit", &p);
                return;
            }
            let contexts = array(&owner["context_bindings"]);
            if contexts.len() != 1 {
                c.issue("request_context_unsupported", &p);
            }
            let open = owner["body"]["kind"] == "normative_open_pattern";
            if open
                && (raw["kind"] != "source" || !profile::permits(c.vocab, "normative_open_pattern"))
            {
                c.issue("invalid_shape", &p);
            }
            if open {
                let body = &owner["body"];
                if body["target"]["predicate"] != "action"
                    || body["target"]["polarity"] != "positive"
                    || !exact_roles(&body["target"], &["action_kind", "object"])
                    || body["judgment"] != "required"
                    || body["status"] != "state"
                    || body["basis"] != "normative"
                    || !body["normative_scope"].is_string()
                    || contexts
                        .first()
                        .is_none_or(|v| v["kind"] != "normative_action_relation")
                {
                    c.issue("request_context_unsupported", &p);
                }
                let filter = &body["open_role"]["class_filter"];
                if filter.is_null() {
                    c.issue("invalid_shape", &p);
                }
                if filter["kind"] == "reference_pattern" {
                    let rid = filter["reference_id"].as_str().unwrap_or("");
                    let pattern = array(&raw["references"]).iter().find(|r| r["id"] == rid);
                    let mut target_refs = BTreeSet::new();
                    refs(&body["target"], &mut target_refs);
                    if !pattern.is_some_and(|r| {
                        r["name"].is_null()
                            && r["denotation"] == "entity"
                            && r["binding_scope_id"].is_null()
                            && r["anaphora"].is_null()
                            && (!r["declared_class"].is_null() || !r["semantic_type"].is_null())
                    }) || target_refs.contains(rid)
                        || array(&raw["identity"])
                            .iter()
                            .any(|edge| edge["left"] == rid || edge["right"] == rid)
                    {
                        c.issue("invalid_shape", &p);
                    }
                }
            }
            for context in contexts {
                let target = if kind == "request" {
                    &owner["target"]
                } else {
                    &owner["body"]["value"]
                };
                match context["kind"].as_str() {
                    Some("lexical_clause_context") => {
                        if !super::super::lexical_clause_context::context_document_valid(
                            raw, owner, context,
                        ) {
                            c.issue("invalid_shape", &p);
                        } else if !super::super::lexical_clause_context::admitted(raw, owner) {
                            c.issue("request_context_unsupported", &p);
                        }
                    }
                    Some("nominal_during") => {
                        let ok = target["atom"]["predicate"] == "quantity"
                            && target["atom"]["polarity"] == "positive"
                            && target["judgment"] == "fact"
                            && target["status"] == "state"
                            && if kind == "request" {
                                raw["kind"] == "question"
                                    && owner["mode"] == "answer"
                                    && owner["slot"] == "amount"
                                    && exact_roles(&target["atom"], &["subject", "attribute"])
                            } else {
                                raw["kind"] == "source"
                                    && owner["body"]["kind"] == "proposition"
                                    && exact_roles(
                                        &target["atom"],
                                        &["subject", "attribute", "amount"],
                                    )
                            };
                        if !ok {
                            c.issue("request_context_unsupported", &p);
                        }
                        let anchor = context["anchor"]["id"].as_str().and_then(|rid| {
                            array(&raw["references"]).iter().find(|r| r["id"] == rid)
                        });
                        if !anchor.is_some_and(|r| !r["name"].is_null() && !r["mention"].is_null())
                        {
                            c.issue("request_context_unsupported", &p);
                        }
                    }
                    Some("normative_action_relation") => {
                        let ok = if open {
                            true
                        } else if kind == "request" {
                            raw["kind"] == "question"
                                && normative_target(target)
                                && exact_roles(&target["atom"], &["action_kind", "object"])
                                && owner["mode"] == "answer"
                                && owner["slot"] == "actor"
                                && !owner["class_filter"].is_null()
                                && owner["basis_requirement"]
                                    == json!({"kind":"exact","value":"normative"})
                                && owner["normative_scope_filter"].is_string()
                        } else {
                            raw["kind"] == "source"
                                && owner["body"]["kind"] == "proposition"
                                && normative_target(target)
                                && target["basis"] == "normative"
                                && target["normative_scope"].is_string()
                                && exact_roles(&target["atom"], &["actor", "action_kind", "object"])
                                && argument(&target["atom"], "actor")
                                    .and_then(|a| a["id"].as_str())
                                    .and_then(|rid| {
                                        array(&raw["references"]).iter().find(|r| r["id"] == rid)
                                    })
                                    .is_some_and(|r| !r["name"].is_null())
                        };
                        if !ok {
                            c.issue("request_context_unsupported", &p);
                        }
                        if kind == "claim"
                            && !open
                            && context["action"]["actor"]["kind"] == "owner_slot"
                        {
                            c.issue("request_context_unsupported", &p);
                        }
                    }
                    _ => {} // Shape check emits the unknown variant.
                }
            }
        }
    }
}

// Two visits are charged for helpers that inspect operands and then collect them.
fn charge_tree(v: &Value) -> bool {
    if !super::super::context_budget::charge(2, 0) {
        return false;
    }
    if let Some(o) = v.as_object() {
        o.values().all(charge_tree)
    } else if let Some(a) = v.as_array() {
        a.iter().all(charge_tree)
    } else {
        true
    }
}

/// Candidate traversal follows implicit operands without adding serialized dependency rows.
pub(super) fn implicit_dependencies(raw: &Value, kind: &str, id: &str) -> Vec<(String, String)> {
    let table = match kind {
        "claim" => "claims",
        "request" => "requests",
        "reference" => "references",
        "qualifier" => "qualifiers",
        "scope" => "scopes",
        _ => return vec![],
    };
    if !super::super::context_budget::charge(
        array(&raw[table])
            .len()
            .saturating_add(array(&raw["identity"]).len()),
        0,
    ) {
        return vec![];
    }
    let Some(row) = array(&raw[table]).iter().find(|r| r["id"] == id) else {
        return vec![];
    };
    let mut out = BTreeSet::new();
    if !charge_tree(row) {
        return vec![];
    }
    let mut used = BTreeSet::new();
    refs(row, &mut used);
    out.extend(used.into_iter().map(|r| ("reference".to_owned(), r)));
    for key in ["scope_id", "parent_id"] {
        if let Some(scope) = row[key].as_str() {
            out.insert(("scope".to_owned(), scope.to_owned()));
        }
    }
    fn qids(v: &Value, out: &mut BTreeSet<(String, String)>) {
        if let Some(obj) = v.as_object() {
            for (key, value) in obj {
                if key == "qualifier_ids" {
                    for q in array(value) {
                        if let Some(id) = q.as_str() {
                            out.insert(("qualifier".to_owned(), id.to_owned()));
                        }
                    }
                }
                qids(value, out);
            }
        } else if let Some(a) = v.as_array() {
            for value in a {
                qids(value, out);
            }
        }
    }
    qids(row, &mut out);
    if kind == "reference" {
        for edge in array(&raw["identity"]) {
            if edge["left"] == id {
                if let Some(other) = edge["right"].as_str() {
                    out.insert(("reference".into(), other.into()));
                }
            }
            if edge["right"] == id {
                if let Some(other) = edge["left"].as_str() {
                    out.insert(("reference".into(), other.into()));
                }
            }
        }
    }
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn quote(text: &str, value: &str) -> Value {
        let start = text.find(value).unwrap();
        json!({"start_byte":start,"end_byte":start+value.len(),"quote":value})
    }
    fn vocabulary() -> Value {
        json!({"version":"request-context-test-v1","predicate_rows":[
            {"id":"quantity","roles":{"subject":"ref","attribute":"attribute_id","amount":"quantity"},"distributive":true},
            {"id":"action","roles":{"actor":"ref","action_kind":"action_id","object":"ref"},"distributive":true}],
            "fixed_attribute_rows":["autonomy"],"fixed_action_rows":["accompany","enroll"],"semantic_type_rows":[],"relevance_links":[],
            "request_context_profile":{"version":profile::VERSION,"features":["nominal_during","normative_action_relation","normative_open_pattern"]}})
    }
    fn reference(text: &str, id: &str, name: &str) -> Value {
        json!({"id":id,"scope_id":"root","mention":quote(text,name),"name":quote(text,name),"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null})
    }
    fn base(text: &str, vocab: &Value) -> Value {
        json!({"version":"documentary_evidence_v3","text_sha256":digest(text.as_bytes()),"vocabulary_sha256":digest(&serde_json::to_vec(vocab).unwrap()),"kind":"question","domain_namespace":"synthetic-context-v1","scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":[reference(text,"device","D"),reference(text,"route","A"),reference(text,"person","P"),reference(text,"course","B")],"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":[],"coverage":[]})
    }
    fn add_dependencies(d: &mut Value) {
        let mut deps = vec![];
        for (table, kind) in [
            ("references", "reference"),
            ("requests", "request"),
            ("claims", "claim"),
        ] {
            for row in array(&d[table]) {
                deps.push(json!({"consumer":{"kind":kind,"id":row["id"]},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
                let mut refs_used = BTreeSet::new();
                refs(row, &mut refs_used);
                for rid in refs_used {
                    if row["id"] != rid {
                        deps.push(json!({"consumer":{"kind":kind,"id":row["id"]},"requires":{"kind":"reference","id":rid},"role":"identity"}));
                    }
                }
            }
        }
        d["dependencies"] = json!(deps);
    }
    fn quantity_request(text: &str) -> Value {
        let q = quote(text, text);
        json!({"id":"r","scope_id":"root","mode":"answer","target":{"atom":{"predicate":"quantity","arguments":[{"role":"subject","value":{"kind":"ref","id":"device"}},{"role":"attribute","value":{"kind":"symbol","id":"autonomy","evidence":[q]}}],"polarity":"positive","evidence":[q]},"judgment":"fact","status":"state","qualifier_ids":[]},"slot":"amount","class_filter":null,"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[q],"context_bindings":[{"id":"ctx","kind":"nominal_during","anchor":{"kind":"ref","id":"route"},"evidence":[q]}]})
    }
    fn normative_request(text: &str) -> Value {
        let mut r = quantity_request(text);
        let q = quote(text, text);
        r["target"] = json!({"atom":{"predicate":"action","arguments":[{"role":"action_kind","value":{"kind":"symbol","id":"accompany","evidence":[q]}},{"role":"object","value":{"kind":"ref","id":"person"}}],"polarity":"positive","evidence":[q]},"judgment":"required","status":"state","qualifier_ids":[]});
        r["slot"] = json!("actor");
        r["class_filter"] = json!({"kind":"lexical","class_quote":quote(text,"tutor")});
        r["basis_requirement"] = json!({"kind":"exact","value":"normative"});
        r["normative_scope_filter"] = json!("course");
        r["context_bindings"] = json!([{"id":"ctx","kind":"normative_action_relation","relation":{"kind":"purpose","evidence":[q]},"action":{"action_kind":{"kind":"symbol","id":"enroll","evidence":[q]},"actor":{"kind":"unresolved","reason":"unspecified","evidence":[q]},"destination":{"kind":"ref","id":"course"},"evidence":[q]},"evidence":[q]}]);
        r
    }
    fn report(d: &Value, text: &str, v: &Value) -> Value {
        serde_json::to_value(crate::documentary_v3::validate(d, text, v)).unwrap()
    }
    const TEXT: &str = "D A P B tutor";
    #[test]
    fn nominal_operands_are_implicit_but_still_validate_scope_and_quotes() {
        let v = vocabulary();
        let mut d = base(TEXT, &v);
        d["requests"] = json!([quantity_request(TEXT)]);
        add_dependencies(&mut d);
        d["dependencies"]
            .as_array_mut()
            .unwrap()
            .retain(|edge| !(edge["consumer"]["id"] == "r" && edge["requires"]["id"] == "route"));
        let r = report(&d, TEXT, &v);
        assert_eq!(r["status"], "complete", "{r}");
        assert_eq!(r["document"], d);
        d["requests"][0]["context_bindings"][0]["evidence"][0]["quote"] = json!("wrong");
        assert_eq!(report(&d, TEXT, &v)["status"], "invalid");
        d["requests"][0] = quantity_request(TEXT);
        d["scopes"].as_array_mut().unwrap().push(json!({"id":"branch","parent_id":"root","kind":"branch","branch_id":"br","binding":null}));
        d["references"][1]["scope_id"] = json!("branch");
        let r = report(&d, TEXT, &v);
        assert_eq!(r["status"], "invalid", "{r}");
        assert!(
            array(&r["issues"])
                .iter()
                .any(|i| i["code"] == "scope_escape")
        );
    }
    #[test]
    fn unknown_actor_reason_is_not_filled_or_treated_as_same_uncertainty() {
        let v = vocabulary();
        let mut d = base(TEXT, &v);
        d["requests"] = json!([normative_request(TEXT)]);
        add_dependencies(&mut d);
        let r = report(&d, TEXT, &v);
        assert_eq!(r["status"], "complete", "{r}");
        for (reason, code) in [
            ("ambiguous_control", "request_context_ambiguous"),
            (
                "extraction_unresolved",
                "request_context_extraction_unresolved",
            ),
        ] {
            d["requests"][0]["context_bindings"][0]["action"]["actor"]["reason"] = json!(reason);
            let r = report(&d, TEXT, &v);
            assert_eq!(r["status"], "partial", "{r}");
            assert_eq!(r["document"], d);
            assert!(
                array(&r["candidates"][0]["reason_codes"]).contains(&json!(code)),
                "{r}"
            );
        }
    }
    #[test]
    fn conjunctive_contexts_are_kept_unsupported_not_split_or_deleted() {
        let v = vocabulary();
        let mut d = base(TEXT, &v);
        d["requests"] = json!([quantity_request(TEXT)]);
        add_dependencies(&mut d);
        let mut second = d["requests"][0]["context_bindings"][0].clone();
        second["id"] = json!("second");
        d["requests"][0]["context_bindings"]
            .as_array_mut()
            .unwrap()
            .push(second);
        let r = report(&d, TEXT, &v);
        assert_eq!(r["status"], "partial", "{r}");
        assert_eq!(r["document"], d);
        assert!(
            array(&r["candidates"][0]["reason_codes"])
                .contains(&json!("request_context_unsupported"))
        );
    }
    #[test]
    fn source_open_pattern_has_no_individual_and_negative_owner_is_unsupported() {
        let v = vocabulary();
        let mut d = base(TEXT, &v);
        let r = normative_request(TEXT);
        d["kind"] = json!("source");
        d["claims"] = json!([{"id":"c","scope_id":"root","qualifier_ids":[],"evidence":[quote(TEXT,TEXT)],"context_bindings":r["context_bindings"],"body":{"kind":"normative_open_pattern","target":r["target"]["atom"],"open_role":{"role":"actor","class_filter":r["class_filter"],"evidence":[quote(TEXT,"tutor")]},"judgment":"required","status":"state","basis":"normative","normative_scope":"course"}}]);
        add_dependencies(&mut d);
        let r = report(&d, TEXT, &v);
        assert_eq!(r["status"], "complete", "{r}");
        assert_eq!(r["document"], d);
        d["claims"][0]["body"]["target"]["polarity"] = json!("negative");
        let r = report(&d, TEXT, &v);
        assert_eq!(r["status"], "partial", "{r}");
        assert_eq!(r["document"], d);
        assert!(
            array(&r["candidates"][0]["reason_codes"])
                .contains(&json!("request_context_unsupported"))
        );
    }
    #[test]
    fn feature_absence_and_extras_reject_new_carriers_without_weakening_legacy() {
        let mut v = vocabulary();
        let mut d = base(TEXT, &v);
        d["requests"] = json!([quantity_request(TEXT)]);
        add_dependencies(&mut d);
        d["requests"][0]["context_bindings"][0]["extra"] = json!(false);
        assert_eq!(report(&d, TEXT, &v)["status"], "invalid");
        d["requests"][0] = quantity_request(TEXT);
        v.as_object_mut().unwrap().remove("request_context_profile");
        d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
        assert_eq!(report(&d, TEXT, &v)["status"], "invalid");
        d["requests"][0]
            .as_object_mut()
            .unwrap()
            .remove("context_bindings");
        assert_eq!(report(&d, TEXT, &v)["status"], "complete");
    }
    #[test]
    fn implicit_reference_debt_reaches_owner_candidate_without_dependency_row() {
        let v = vocabulary();
        let mut d = base(TEXT, &v);
        d["requests"] = json!([quantity_request(TEXT)]);
        add_dependencies(&mut d);
        d["dependencies"]
            .as_array_mut()
            .unwrap()
            .retain(|edge| !(edge["consumer"]["id"] == "r" && edge["requires"]["id"] == "route"));
        d["references"][1]["anaphora"] =
            json!({"status":"unresolved","antecedent_ids":[],"evidence":[quote(TEXT,"A")]});
        let r = report(&d, TEXT, &v);
        assert_eq!(r["status"], "partial", "{r}");
        assert!(
            array(&r["candidates"][0]["reason_codes"]).contains(&json!("unresolved_target")),
            "{r}"
        );
    }
}
