// Synthetic mechanism controls; no automatic extraction or historical raw repair.
fn temporal_premise_case() -> (Value, &'static str, Value, &'static str) {
    let st = "Mara withdrew Helia in 2019, in setting A.";
    let qt = "In which year did Mara withdraw Helia? Setting A or B, 2019 or 2020?";
    let mut sd = doc(st, "source", &["Mara", "Helia"]);
    let mut qd = doc(qt, "question", &["Mara", "Helia"]);
    for (d, text, source) in [(&mut sd, st, true), (&mut qd, qt, false)] {
        let mut p = prop(atom(
            text,
            "action",
            vec![
                ("actor", reference("Mara")),
                (
                    "action_kind",
                    json!({"kind":"symbol","id":"withdraw","evidence":[quote(text,text)]}),
                ),
                ("object", reference("Helia")),
            ],
        ));
        p["status"] = json!("occurred");
        if source {
            p["qualifier_ids"] = json!(["when"]);
            d["qualifiers"] = json!([{"id":"when","scope_id":"root","kind":"time","body":{"role":"occurred_at","value":{"kind":"time","value":"2019","format":"year","evidence":[quote(text,"2019")]}},"resolution":"known","evidence":[quote(text,"2019")]}]);
        }
        d["claims"] = json!([claim(text, "withdrawal", p.clone())]);
        if !source {
            let mut r = request(text, "answer", "occurred_year", &p);
            r["premise_ids"] = json!(["withdrawal"]);
            d["requests"] = json!([r]);
        }
    }
    (sd, st, qd, qt)
}
#[test]
fn temporal_premise_requested_occurrence_year_does_not_block_its_own_premise() {
    let (sd, st, qd, qt) = temporal_premise_case();
    let r = pair(sd, st, qd, qt);
    assert_eq!(r["complete_answer"], true, "{r}");
    assert_eq!(
        r["requests"][0]["support"][0]["answer_terms"][0]["value"], "2019",
        "{r}"
    );
    assert_eq!(
        r["requests"][0]["support"][0]["qualifier_ids"],
        json!(["when"]),
        "{r}"
    );
}
#[test]
fn temporal_premise_keeps_other_context_and_explicit_time_constraints() {
    let (sd, st, qd, qt) = temporal_premise_case();
    for kind in [
        "configuration",
        "unknown_time",
        "wrong_time_role",
        "literal_year",
        "different_premise",
        "fixed_other_year",
        "planned",
    ] {
        let mut source = sd.clone();
        let mut question = qd.clone();
        match kind {
            "configuration" => {
                source["qualifiers"].as_array_mut().unwrap().push(json!({"id":"setting","scope_id":"root","kind":"configuration","body":{"key":"setting","value":{"kind":"text","value":"A","evidence":[quote(st,"A")]}},"resolution":"known","evidence":[quote(st,"A")]}));
                source["claims"][0]["body"]["value"]["qualifier_ids"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("setting"));
            }
            "unknown_time" => source["qualifiers"][0]["resolution"] = json!("unknown"),
            "wrong_time_role" => source["qualifiers"][0]["body"]["role"] = json!("holds_at"),
            "literal_year" => source["qualifiers"][0]["body"]["value"]["format"] = json!("literal"),
            "different_premise" => {
                question["claims"][0]["body"]["value"]["atom"]["polarity"] = json!("negative")
            }
            "fixed_other_year" => {
                let mut when = source["qualifiers"][0].clone();
                when["body"]["value"]["value"] = json!("2020");
                when["body"]["value"]["evidence"] = json!([quote(qt, "2020")]);
                when["evidence"] = json!([quote(qt, "2020")]);
                question["qualifiers"] = json!([when]);
                question["claims"][0]["body"]["value"]["qualifier_ids"] = json!(["when"]);
            }
            "planned" => source["claims"][0]["body"]["value"]["status"] = json!("planned"),
            _ => unreachable!(),
        }
        let r = pair(source, st, question, qt);
        assert_eq!(r["complete_answer"], false, "{kind}: {r}");
    }
}
#[test]
fn temporal_premise_retains_matching_configuration_and_blocks_branches() {
    let (mut sd, st, mut qd, qt) = temporal_premise_case();
    for (d, text) in [(&mut sd, st), (&mut qd, qt)] {
        d["qualifiers"].as_array_mut().unwrap().push(json!({"id":"setting","scope_id":"root","kind":"configuration","body":{"key":"setting","value":{"kind":"text","value":"A","evidence":[quote(text,"A")]}},"resolution":"known","evidence":[quote(text,"A")]}));
        d["claims"][0]["body"]["value"]["qualifier_ids"]
            .as_array_mut()
            .unwrap()
            .push(json!("setting"));
    }
    qd["requests"][0]["qualifier_ids"] = json!(["setting"]);
    qd["requests"][0]["target"]["qualifier_ids"] = json!(["setting"]);
    let r = pair(sd.clone(), st, qd.clone(), qt);
    assert_eq!(r["complete_answer"], true, "{r}");
    qd["qualifiers"][0]["body"]["value"]["value"] = json!("B");
    qd["qualifiers"][0]["body"]["value"]["evidence"] = json!([quote(qt, "B")]);
    let r = pair(sd.clone(), st, qd, qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    let (_, _, qd, _) = temporal_premise_case();
    sd["scopes"].as_array_mut().unwrap().push(json!({"id":"alternative","parent_id":"root","kind":"branch","branch_id":"choice","binding":null}));
    sd["claims"][0]["scope_id"] = json!("alternative");
    let r = pair(sd, st, qd, qt);
    assert_eq!(r["complete_answer"], false, "{r}");
}
#[test]
fn temporal_premise_respects_pair_scope_time_selector() {
    let (mut sd, st, mut qd, qt) = temporal_premise_case();
    dependencies(&mut sd);
    dependencies(&mut qd);
    let a = super::super::validate(&sd, st, &vocabulary());
    let b = super::super::validate(&qd, qt, &vocabulary());
    assert_eq!(a.report["status"], "complete");
    assert_eq!(b.report["status"], "complete");
    for (year, expected) in [("2019", true), ("2020", false)] {
        let scope = json!({"namespace":"fresh-controls","configuration":[],"time_selector":{"role":"occurred_at","value":{"kind":"time","value":year,"format":"year","evidence":[quote(qt,year)]}},"closed_world":false});
        let r = classify_pair(&a, &b, &vocabulary(), &scope);
        assert_eq!(r["complete_answer"], expected, "{year}: {r}");
    }
}
#[test]
fn temporal_premise_does_not_use_a_document_nested_under_a_vacuous_binder() {
    let (mut sd, st, qd, qt) = temporal_premise_case();
    let mut variable = sd["references"][0].clone();
    variable["id"] = json!("x");
    variable["name"] = Value::Null;
    variable["scope_id"] = json!("all");
    variable["binding_scope_id"] = json!("all");
    variable["declared_class"] = quote(st, "Helia");
    sd["references"].as_array_mut().unwrap().push(variable);
    let mut domain = sd["references"][1].clone();
    domain["id"] = json!("domain");
    domain["name"] = Value::Null;
    domain["declared_class"] = quote(st, "Helia");
    domain["denotation"] = json!("class");
    sd["references"].as_array_mut().unwrap().push(domain);
    sd["scopes"].as_array_mut().unwrap().extend([
        json!({"id":"all","parent_id":"root","kind":"binder","branch_id":null,"binding":{"variable":"x","quantifier":"forall","domain_ref":"domain","cardinality":null,"witness_relation":"unspecified","relation_closures":[],"evidence":[quote(st,st)]}}),
        json!({"id":"inside","parent_id":"all","kind":"document","branch_id":null,"binding":null})
    ]);
    let mut direct = sd["claims"][0].clone();
    direct["id"] = json!("observed");
    direct["body"]["value"]["basis"] = json!("observation");
    sd["claims"][0]["scope_id"] = json!("inside");
    sd["claims"].as_array_mut().unwrap().push(direct);
    let r = pair(sd, st, qd, qt);
    assert_eq!(r["complete_answer"], false, "{r}");
}
