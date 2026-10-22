// Synthetic schema/meaning controls; these are not extraction-fidelity measurements.
fn term_role_case(closed: bool) -> (Value, Value, Value) {
    let mut v = vocabulary();
    v["term_description_profile"] = json!({"version":"local_reference_descriptions_v1"});
    v["described_term_role_profile"] = json!({"version":"described_reference_term_roles_v1"});
    v["predicate_rows"].as_array_mut().unwrap().push(json!({"id":"property","roles":{"subject":"ref","attribute":"attribute_id","value":"term"},"distributive":true}));
    v["fixed_attribute_rows"] = json!(["water", "mass"]);
    let mut d = useful();
    // Distinct literal names prevent the all-"a" helper from proving identity.
    let text = "Mara Helia Sora Orbe garden home water mass";
    d["text_sha256"] = json!(digest(text.as_bytes()));
    d["domain_namespace"] = json!("term-role-controls");
    d["references"] = json!(["Mara","Helia","Sora","Orbe"].map(|id| {
        let e = role_quote(text, id);
        json!({"id":id,"scope_id":"root","mention":e,"name":e,"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null})
    }));
    let e = role_quote(text, text);
    let p = json!({"atom":{"predicate":"owns","arguments":[{"role":"owner","value":{"kind":"ref","id":"Sora"}},{"role":"object","value":{"kind":"ref","id":"Helia"}}],"polarity":"positive","evidence":[e]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]});
    let term = json!({"kind":"described_ref","id":"Helia","constraints":[p],"resolution":"known","evidence":[e]});
    let atom = json!({"predicate":"property","arguments":[{"role":"subject","value":{"kind":"ref","id":"Mara"}},{"role":"attribute","value":{"kind":"symbol","id":"water","evidence":[role_quote(text,"water")]}},{"role":"value","value":term}],"polarity":"positive","evidence":[e]});
    let prop = json!({"atom":atom,"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]});
    let body = if closed {
        // A restriction target is an Atom (all declared roles), not an AtomPattern.
        let target = atom.clone();
        json!({"kind":"value_restriction","target":target,"answer_role":"value","closure":"closed","members":[term],"judgment":"fact","status":"state","basis":"description","normative_scope":null,"existence":"asserted"})
    } else {
        json!({"kind":"proposition","value":prop})
    };
    d["claims"] =
        json!([{"id":"c","scope_id":"root","qualifier_ids":[],"evidence":[e],"body":body}]);
    role_dependencies(&mut d, "claim", "c");
    let mut q = d.clone();
    q["kind"] = json!("question");
    q["claims"] = json!([]);
    let mut target = prop;
    target.as_object_mut().unwrap().remove("basis");
    target.as_object_mut().unwrap().remove("normative_scope");
    target["atom"]["arguments"].as_array_mut().unwrap().pop();
    q["requests"] = json!([{"id":"r","scope_id":"root","mode":if closed {"constraint"} else {"answer"},"slot":"value","target":target,"qualifier_ids":[],"premise_ids":[],"class_filter":null,"basis_requirement":{"kind":"any"},"normative_scope_filter":null,"required_answer_extent":if closed {"all_values"} else {"one_value"},"unit":null,"evidence":[e]}]);
    role_dependencies(&mut q, "request", "r");
    role_hash(&mut d, &v);
    role_hash(&mut q, &v);
    (v, d, q)
}
fn role_quote(text: &str, quote: &str) -> Value {
    let start = text.find(quote).unwrap();
    json!({"start_byte":start,"end_byte":start+quote.len(),"quote":quote})
}
fn role_hash(d: &mut Value, v: &Value) {
    d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(v).unwrap()));
}
fn role_dependencies(d: &mut Value, kind: &str, id: &str) {
    let mut deps = vec![];
    for r in array(&d["references"]) {
        deps.push(json!({"consumer":{"kind":"reference","id":r["id"]},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
        deps.push(json!({"consumer":{"kind":kind,"id":id},"requires":{"kind":"reference","id":r["id"]},"role":"identity"}));
    }
    deps.push(json!({"consumer":{"kind":kind,"id":id},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
    d["dependencies"] = json!(deps);
}
fn role_pair(v: &Value, d: &Value, q: &Value) -> Value {
    let text = "Mara Helia Sora Orbe garden home water mass";
    let a = validate(d, text, v);
    let b = validate(q, text, v);
    assert_eq!(a.report["status"], "complete", "{}", a.report);
    assert_eq!(b.report["status"], "complete", "{}", b.report);
    super::super::classify_pair(
        &a,
        &b,
        v,
        &json!({"namespace":"term-role-controls","configuration":[],"time_selector":null,"closed_world":false}),
    )
}
#[test]
fn described_term_roles_opt_in_preserves_members_and_property_value() {
    for closed in [false, true] {
        let (mut v, mut d, mut q) = term_role_case(closed);
        let report = role_pair(&v, &d, &q);
        assert_eq!(report["complete_answer"], true, "{report}");
        let wanted = if closed {
            d["claims"][0]["body"]["members"].clone()
        } else {
            json!([d["claims"][0]["body"]["value"]["atom"]["arguments"][2]["value"]])
        };
        assert_eq!(
            report["requests"][0]["support"][0]["answer_terms"], wanted,
            "{report}"
        );
        v.as_object_mut()
            .unwrap()
            .remove("described_term_role_profile");
        role_hash(&mut d, &v);
        role_hash(&mut q, &v);
        assert_ne!(
            validate(&d, "Mara Helia Sora Orbe garden home water mass", &v).report["status"],
            "complete"
        );
    }
}
#[test]
fn described_term_roles_reject_invalid_or_missing_prerequisite_profile() {
    let (v, d, _) = term_role_case(false);
    for profile in [
        json!(null),
        json!({"version":"other"}),
        json!({"version":"described_reference_term_roles_v1","extra":true}),
    ] {
        let mut v = v.clone();
        v["described_term_role_profile"] = profile;
        let mut d = d.clone();
        role_hash(&mut d, &v);
        assert_eq!(
            validate(&d, "Mara Helia Sora Orbe garden home water mass", &v).report["status"],
            "invalid"
        );
    }
    let mut v = v;
    v.as_object_mut()
        .unwrap()
        .remove("term_description_profile");
    let mut d = d;
    role_hash(&mut d, &v);
    assert_eq!(
        validate(&d, "Mara Helia Sora Orbe garden home water mass", &v).report["status"],
        "invalid"
    );
}
#[test]
fn described_term_roles_do_not_widen_symbol_quantity_or_method_carriers() {
    let (v, d, _) = term_role_case(false);
    let term = d["claims"][0]["body"]["value"]["atom"]["arguments"][2]["value"].clone();
    for role_type in ["attribute_id", "action_id", "quantity"] {
        let mut v = v.clone();
        v["predicate_rows"][1]["roles"]["value"] = json!(role_type);
        let mut d = d.clone();
        role_hash(&mut d, &v);
        assert_ne!(
            validate(&d, "Mara Helia Sora Orbe garden home water mass", &v).report["status"],
            "complete"
        );
    }
    // Existing non-Argument term carriers remain forbidden even with both profiles.
    let mut check = Check {
        text: "Mara Helia Sora Orbe garden home water mass",
        vocab: &v,
        issues: vec![],
        nodes: 0,
        limit_exceeded: false,
    };
    check.node(
        "Method",
        &json!({"state":"specified","value":term}),
        "/method",
        0,
    );
    assert!(check.issues.iter().any(|i| i["code"] == "invalid_shape"));
}
#[test]
fn described_term_roles_preserve_context_and_pending_description_debt() {
    let (v, mut d, mut q) = term_role_case(false);
    let text = "Mara Helia Sora Orbe garden home water mass";
    for doc in [&mut d, &mut q] {
        doc["qualifiers"] = json!([{"id":"purpose","scope_id":"root","kind":"configuration","body":{"key":"purpose","value":{"kind":"text","value":"garden","evidence":[role_quote(text,"garden")]}},"resolution":"known","evidence":[role_quote(text,"garden")]}]);
        let table = if doc["kind"] == "source" {
            "claims"
        } else {
            "requests"
        };
        let id = if table == "claims" { "c" } else { "r" };
        doc[table][0]["qualifier_ids"] = json!(["purpose"]);
        doc["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":if table=="claims" {"claim"} else {"request"},"id":id},"requires":{"kind":"qualifier","id":"purpose"},"role":"configuration"}));
    }
    let report = role_pair(&v, &d, &q);
    assert_eq!(report["complete_answer"], true, "{report}");
    q["qualifiers"][0]["body"]["value"] =
        json!({"kind":"text","value":"home","evidence":[role_quote(text,"home")]});
    let report = role_pair(&v, &d, &q);
    assert_eq!(report["complete_answer"], false, "{report}");
    d["claims"][0]["body"]["value"]["atom"]["arguments"][2]["value"]["resolution"] =
        json!("unknown");
    let report = validate(&d, text, &v).report;
    assert_eq!(report["status"], "partial", "{report}");
    assert_eq!(report["document"]["claims"], d["claims"]);
}
#[test]
fn described_term_roles_never_promote_inner_ownership_or_other_attribute() {
    let (v, d, mut q) = term_role_case(false);
    q["requests"][0]["target"]["atom"]["arguments"][1]["value"]["id"] = json!("mass");
    let report = role_pair(&v, &d, &q);
    assert_eq!(report["complete_answer"], false, "{report}");
    q["requests"][0]["mode"] = json!("truth");
    q["requests"][0]["slot"] = json!("status");
    q["requests"][0]["required_answer_extent"] = json!("truth_status");
    let p =
        d["claims"][0]["body"]["value"]["atom"]["arguments"][2]["value"]["constraints"][0].clone();
    let mut target = p;
    target.as_object_mut().unwrap().remove("basis");
    target.as_object_mut().unwrap().remove("normative_scope");
    q["requests"][0]["target"] = target;
    let report = role_pair(&v, &d, &q);
    assert_eq!(report["complete_answer"], false, "{report}");
    q["requests"][0]["target"]["atom"]["arguments"][0]["value"]["id"] = json!("Orbe");
    let report = role_pair(&v, &d, &q);
    assert_eq!(report["complete_answer"], false, "{report}");
}

#[test]
fn described_term_roles_match_whole_description_not_dropped_provider() {
    let (v, d, mut q) = term_role_case(false);
    q["requests"][0]["mode"] = json!("truth");
    q["requests"][0]["slot"] = json!("status");
    q["requests"][0]["required_answer_extent"] = json!("truth_status");
    q["requests"][0]["target"]["atom"] = d["claims"][0]["body"]["value"]["atom"].clone();
    let report = role_pair(&v, &d, &q);
    assert_eq!(report["complete_answer"], true, "{report}");
    q["requests"][0]["target"]["atom"]["arguments"][2]["value"]["constraints"][0]["atom"]["arguments"]
        [0]["value"]["id"] = json!("Orbe");
    let report = role_pair(&v, &d, &q);
    assert_eq!(report["complete_answer"], false, "{report}");
    assert!(
        array(&report["requests"][0]["support"]).is_empty(),
        "{report}"
    );
    assert!(
        array(&report["requests"][0]["refutation"]).is_empty(),
        "{report}"
    );
    // A name-anchored partial may retain both incompatible descriptions; it
    // must not publish the requested provider or discard the source constraint.
    for partial in array(&report["requests"][0]["partial"]) {
        assert!(array(&partial["answer_terms"]).is_empty(), "{report}");
        assert!(
            array(&partial["answer_propositions"]).is_empty(),
            "{report}"
        );
        assert_eq!(partial["source_constraint"], d["claims"][0]["body"]);
    }
}
