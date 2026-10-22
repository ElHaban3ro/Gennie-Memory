fn qualifier_dependency_case() -> (Value, &'static str, Value, &'static str) {
    let st = "Mara owns Helia in profile Sora.";
    let qt = "Does Mara own Helia in profile Sora?";
    let (mut sd, mut qd) = ownership(st, qt);
    for (d, text) in [(&mut sd, st), (&mut qd, qt)] {
        d["references"]
            .as_array_mut()
            .unwrap()
            .push(doc(text, "source", &["Sora"])["references"][0].clone());
        d["qualifiers"] = json!([{"id":"profile","scope_id":"root","kind":"configuration","body":{"key":"profile","value":reference("Sora")},"resolution":"known","evidence":[quote(text,"profile Sora")]}]);
        if d["kind"] == "source" {
            d["claims"][0]["qualifier_ids"] = json!(["profile"]);
        } else {
            d["requests"][0]["qualifier_ids"] = json!(["profile"]);
        }
        dependencies(d);
        d["dependencies"].as_array_mut().unwrap().retain(|x| {
            !(x["consumer"]["kind"] != "reference"
                && x["requires"]["kind"] == "reference"
                && x["requires"]["id"] == "Sora")
        });
    }
    (sd, st, qd, qt)
}
fn qualifier_dependency_result(sd: Value, st: &str, qd: Value, qt: &str) -> Value {
    let v = vocabulary();
    let a = super::super::validate(&sd, st, &v);
    let b = super::super::validate(&qd, qt, &v);
    assert_ne!(a.report["status"], "invalid", "{}", a.report);
    assert_ne!(b.report["status"], "invalid", "{}", b.report);
    classify_pair(
        &a,
        &b,
        &v,
        &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
    )
}
#[test]
fn qualifier_operand_uses_its_own_dependencies_without_redundant_owner_edge() {
    let (sd, st, qd, qt) = qualifier_dependency_case();
    let r = qualifier_dependency_result(sd.clone(), st, qd.clone(), qt);
    assert_eq!(r["complete_answer"], true, "{r}");
    for source in [true, false] {
        let mut s = sd.clone();
        let mut q = qd.clone();
        let (d, text) = if source { (&mut s, st) } else { (&mut q, qt) };
        d["qualifiers"].as_array_mut().unwrap().push(json!({"id":"pending","scope_id":"root","kind":"configuration","body":{"key":"context","value":{"kind":"text","value":"profile","evidence":[quote(text,"profile")]}},"resolution":"unknown","evidence":[quote(text,"profile")]}));
        d["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":"Sora"},"requires":{"kind":"qualifier","id":"pending"},"role":"configuration"}));
        let r = qualifier_dependency_result(s, st, q, qt);
        assert_eq!(r["complete_answer"], false, "source={source}: {r}");
    }
}
#[test]
fn qualifier_operand_cycle_is_pending_and_candidate_payload_is_preserved() {
    let (sd, st, mut qd, qt) = qualifier_dependency_case();
    qd["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":"Sora"},"requires":{"kind":"qualifier","id":"profile"},"role":"configuration"}));
    let r = qualifier_dependency_result(sd.clone(), st, qd.clone(), qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
    let candidate = &r["requests"][0]["unresolved"][0];
    assert_eq!(candidate["request_constraint"], qd["requests"][0], "{r}");
    assert_eq!(candidate["source_constraints"], sd["claims"], "{r}");
    assert!(
        !rows(&candidate["pending_evidence"], "support").is_empty(),
        "{r}"
    );
}
#[test]
fn qualifier_operand_validation_issues_are_local_and_propagate_on_both_sides() {
    let (sd, st, qd, qt) = qualifier_dependency_case();
    for source in [true, false] {
        let mut s = sd.clone();
        let mut q = qd.clone();
        let (d, text) = if source { (&mut s, st) } else { (&mut q, qt) };
        d["references"][2]["semantic_type"] =
            json!({"type_id":"not_registered","evidence":[quote(text,"Sora")]});
        let r = qualifier_dependency_result(s, st, q, qt);
        assert_eq!(r["complete_answer"], false, "source={source}: {r}");
    }
    let mut q = qd;
    let mut unused = q["references"][2].clone();
    unused["id"] = json!("unused");
    unused["semantic_type"] = json!({"type_id":"not_registered","evidence":[quote(qt,"Sora")]});
    q["references"].as_array_mut().unwrap().push(unused);
    q["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":"unused"},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
    let r = qualifier_dependency_result(sd, st, q, qt);
    assert_eq!(r["complete_answer"], true, "{r}");
}
