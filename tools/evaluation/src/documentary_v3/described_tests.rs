// Included in pair's test module to reuse documentary builders without runtime code.
fn desc_vocab() -> Value {
    let mut v = vocabulary();
    v["term_description_profile"] = json!({"version":"local_reference_descriptions_v1"});
    v
}
fn desc_quantity(text: &str, root: &str, n: &str) -> Value {
    prop(atom(
        text,
        "quantity",
        vec![
            ("subject", reference(root)),
            (
                "attribute",
                json!({"kind":"symbol","id":"mass","evidence":[quote(text,text)]}),
            ),
            (
                "amount",
                json!({"kind":"quantity","decimal":n,"unit":"kg","bound":"exact","evidence":[quote(text,&format!("{n} kg"))]}),
            ),
        ],
    ))
}
fn desc_term(text: &str, root: &str, n: &str, origin: &str) -> Value {
    json!({"kind":"described_ref","id":root,"constraints":[desc_quantity(text,root,n),prop(atom(text,"origin",vec![("subject",reference(root)),("origin",reference(origin))]))],"resolution":"known","evidence":[quote(text,text)]})
}
fn desc_case(n: &str) -> (Value, String, Value, String) {
    let st = "Mara admits only Helia of 6 kg from Sora. Helia is a camera. Orbe is another camera."
        .to_owned();
    let qt = format!("Does Mara admit Helia of {n} kg from Sora? Helia is a camera.");
    let mut sd = doc(&st, "source", &["Mara", "Helia", "Sora", "Orbe"]);
    let mut qd = doc(&qt, "question", &["Mara", "Helia", "Sora"]);
    for d in [&mut sd, &mut qd] {
        d["references"][1]["declared_class"] =
            quote(if d["kind"] == "source" { &st } else { &qt }, "camera");
    }
    let target = atom(
        &st,
        "admits",
        vec![
            ("subject", reference("Mara")),
            ("object", reference("Helia")),
        ],
    );
    sd["claims"] = json!([{"id":"limit","scope_id":"root","qualifier_ids":[],"evidence":[quote(&st,&st)],"body":{"kind":"value_restriction","target":target,"answer_role":"object","answer_domain":{"kind":"lexical","class_quote":quote(&st,"camera")},"closure":"closed","members":[desc_term(&st,"Helia","6","Sora")],"judgment":"fact","status":"state","basis":"description","normative_scope":null,"existence":"not_asserted"}}]);
    let p = prop(atom(
        &qt,
        "admits",
        vec![
            ("subject", reference("Mara")),
            ("object", desc_term(&qt, "Helia", n, "Sora")),
        ],
    ));
    qd["requests"] = json!([request(&qt, "truth", "status", &p)]);
    (sd, st, qd, qt)
}
fn desc_result(mut sd: Value, st: &str, mut qd: Value, qt: &str) -> Value {
    let v = desc_vocab();
    dependencies(&mut sd);
    dependencies(&mut qd);
    for d in [&mut sd, &mut qd] {
        d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
    }
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
fn descriptions_closed_member_mass_exclusion_requires_domain_and_no_existential_gain() {
    let (sd, st, qd, qt) = desc_case("8");
    let r = desc_result(sd.clone(), &st, qd.clone(), &qt);
    assert_eq!(r["complete_answer"], true, "{r}");
    assert_eq!(
        r["requests"][0]["refutation"][0]["kind"], "restriction_exclusion",
        "{r}"
    );
    let mut no_domain = qd;
    no_domain["references"][1]["declared_class"] = Value::Null;
    let r = desc_result(sd, &st, no_domain, &qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r["requests"][0], "refutation").is_empty(), "{r}");
    let (sd, st, qd, qt) = desc_case("6");
    let r = desc_result(sd, &st, qd, &qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r["requests"][0], "support").is_empty(), "{r}");
}
#[test]
fn descriptions_constraint_answer_keeps_whole_member_and_never_publishes_properties() {
    let (sd, st, mut qd, qt) = desc_case("6");
    qd["requests"][0]["mode"] = json!("constraint");
    qd["requests"][0]["slot"] = json!("object");
    qd["requests"][0]["required_answer_extent"] = json!("all_values");
    qd["requests"][0]["class_filter"] = json!({"kind":"lexical","class_quote":quote(&qt,"camera")});
    qd["requests"][0]["target"]["atom"]["arguments"]
        .as_array_mut()
        .unwrap()
        .pop();
    let r = desc_result(sd.clone(), &st, qd.clone(), &qt);
    assert_eq!(r["complete_answer"], true, "{r}");
    assert_eq!(
        r["requests"][0]["support"][0]["answer_terms"], sd["claims"][0]["body"]["members"],
        "{r}"
    );
    assert!(rows(&r["requests"][0]["support"][0], "answer_propositions").is_empty());
    qd["requests"][0] = request(&qt, "truth", "status", &desc_quantity(&qt, "Helia", "6"));
    let r = desc_result(sd, &st, qd, &qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
}
#[test]
fn descriptions_plain_missing_unknown_and_units_never_prove_exclusion() {
    for variant in ["bare", "unknown", "missing_constraint", "other_unit"] {
        let (sd, st, mut qd, qt) = desc_case("8");
        let t = &mut qd["requests"][0]["target"]["atom"]["arguments"][1]["value"];
        match variant {
            "bare" => *t = reference("Helia"),
            "unknown" => t["resolution"] = json!("unknown"),
            "missing_constraint" => {
                t["constraints"].as_array_mut().unwrap().remove(0);
            }
            _ => t["constraints"][0]["atom"]["arguments"][2]["value"]["unit"] = json!("g"),
        }
        let r = desc_result(sd, &st, qd, &qt);
        assert_eq!(r["complete_answer"], false, "{variant}: {r}");
        assert!(
            rows(&r["requests"][0], "refutation").is_empty(),
            "{variant}: {r}"
        );
        for w in rows(&r["requests"][0], "partial") {
            assert!(rows(w, "answer_terms").is_empty(), "{r}");
            assert!(w.get("source_constraint").is_some(), "{r}");
        }
    }
}
#[test]
fn descriptions_negative_use_does_not_assert_origin_or_discharge_guard() {
    let (mut sd, st, mut qd, qt) = desc_use_case(true);
    let p = sd["claims"][0]["body"]["value"].clone();
    let r = desc_result(sd.clone(), &st, qd.clone(), &qt);
    assert_eq!(r["complete_answer"], true, "{r}");
    let origin = prop(atom(
        &qt,
        "origin",
        vec![
            ("subject", reference("Helia")),
            ("origin", reference("Sora")),
        ],
    ));
    qd["requests"] = json!([request(&qt, "truth", "status", &origin)]);
    let r = desc_result(sd.clone(), &st, qd, &qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
    let mut independent = claim(
        &st,
        "real",
        prop(atom(
            &st,
            "owns",
            vec![("owner", reference("Mara")), ("object", reference("Orbe"))],
        )),
    );
    independent["qualifier_ids"] = json!(["guard"]);
    sd["qualifiers"] = json!([{"id":"guard","scope_id":"root","kind":"guard","body":p["atom"]["arguments"][1]["value"]["constraints"][1],"resolution":"known","evidence":[quote(&st,&st)]}]);
    sd["claims"].as_array_mut().unwrap().push(independent);
    let mut qd = doc(&st, "question", &["Mara", "Helia", "Sora", "Orbe"]);
    let target = prop(atom(
        &st,
        "owns",
        vec![("owner", reference("Mara")), ("object", reference("Orbe"))],
    ));
    qd["requests"] = json!([request(&st, "truth", "status", &target)]);
    let r = desc_result(sd, &st, qd, &st);
    assert_eq!(r["complete_answer"], false, "{r}");
}
#[test]
fn descriptions_order_and_reference_ids_do_not_change_equivalence() {
    let (sd, st, mut qd, qt) = desc_use_case(false);
    qd["requests"][0]["target"]["atom"]["arguments"][1]["value"]["constraints"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let r = desc_result(sd.clone(), &st, qd.clone(), &qt);
    assert_eq!(r["complete_answer"], true, "{r}");
    fn rename(v: &mut Value) {
        match v {
            Value::Object(o) => {
                for (k, v) in o {
                    if (k == "id" || k == "target") && v == "Helia" {
                        *v = json!("renamed-object");
                    } else {
                        rename(v);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    rename(v)
                }
            }
            _ => (),
        }
    }
    rename(&mut qd);
    let r = desc_result(sd, &st, qd, &qt);
    assert_eq!(r["complete_answer"], true, "{r}");
}
#[test]
fn descriptions_internal_incompatible_masses_stay_unresolved() {
    let (mut sd, st, mut qd, qt) = desc_case("6");
    // Both literals are explicit; this manual contradictory lowering must not resolve.
    let text = st.replace("6 kg", "6 kg and 8 kg");
    sd["text_sha256"] = json!(digest(text.as_bytes()));
    fn requote(v: &mut Value, text: &str) {
        match v {
            Value::Object(o) => {
                if o.contains_key("start_byte") && o.contains_key("quote") {
                    let q = o["quote"].as_str().unwrap().to_owned();
                    if let Some(i) = text.find(&q) {
                        o.insert("start_byte".into(), json!(i));
                        o.insert("end_byte".into(), json!(i + q.len()));
                    } else {
                        o.insert("start_byte".into(), json!(0));
                        o.insert("end_byte".into(), json!(text.len()));
                        o.insert("quote".into(), json!(text));
                    }
                } else {
                    for v in o.values_mut() {
                        requote(v, text);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    requote(v, text)
                }
            }
            _ => (),
        }
    }
    requote(&mut sd, &text);
    sd["claims"][0]["body"]["members"][0]["constraints"]
        .as_array_mut()
        .unwrap()
        .push(desc_quantity(&text, "Helia", "8"));
    qd["requests"][0]["mode"] = json!("constraint");
    qd["requests"][0]["slot"] = json!("object");
    qd["requests"][0]["class_filter"] = json!({"kind":"lexical","class_quote":quote(&qt,"camera")});
    qd["requests"][0]["target"]["atom"]["arguments"]
        .as_array_mut()
        .unwrap()
        .pop();
    let r = desc_result(sd, &text, qd, &qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    for w in rows(&r["requests"][0], "partial") {
        assert!(rows(w, "answer_terms").is_empty(), "{r}");
    }
}
#[test]
fn descriptions_profile_shape_scope_and_locality_are_enforced() {
    let (mut sd, st, _, _) = desc_case("6");
    dependencies(&mut sd);
    for variant in [
        "profile_absent",
        "profile_wrong",
        "empty",
        "unrooted",
        "nested",
        "bad_quote",
        "dangling",
        "duplicate",
        "scalar_slot",
        "scope_escape",
    ] {
        let mut d = sd.clone();
        let mut v = desc_vocab();
        let t = &mut d["claims"][0]["body"]["members"][0];
        match variant {
            "profile_absent" => {
                v.as_object_mut()
                    .unwrap()
                    .remove("term_description_profile");
            }
            "profile_wrong" => v["term_description_profile"]["version"] = json!("future"),
            "empty" => t["constraints"] = json!([]),
            "unrooted" => t["id"] = json!("Orbe"),
            "nested" => {
                let x = t.clone();
                t["constraints"][0]["atom"]["arguments"][0]["value"] = x;
            }
            "bad_quote" => t["evidence"][0]["quote"] = json!("invented"),
            "dangling" => t["id"] = json!("missing"),
            "scope_escape" => {
                d["scopes"].as_array_mut().unwrap().push(json!({"id":"hidden","parent_id":"root","kind":"branch","branch_id":"unresolved-branch","binding":null}));
                d["references"][2]["scope_id"] = json!("hidden");
            }
            "duplicate" => {
                let x = t["constraints"][0].clone();
                t["constraints"].as_array_mut().unwrap().push(x);
            }
            _ => {
                let t = t.clone();
                d["claims"][0]["body"]["target"] = desc_quantity(&st, "Helia", "6")["atom"].clone();
                d["claims"][0]["body"]["answer_role"] = json!("amount");
                d["claims"][0]["body"]["members"] = json!([t]);
            }
        }
        d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
        let r = super::super::validate(&d, &st, &v);
        assert_ne!(r.report["status"], "complete", "{variant}: {}", r.report);
        assert!(
            rows(&r.report, "candidates")
                .iter()
                .all(|c| c["state"] != "eligible"),
            "{variant}: {}",
            r.report
        );
    }
}
fn desc_use_case(negative: bool) -> (Value, String, Value, String) {
    let st = format!(
        "Mara {} Helia of 6 kg from Sora. Helia is a camera. Orbe is a camera. Mara owns Orbe. Another reservation requires Helia to come from Sora. During reservation R, Helia has mass 6 kg.",
        if negative { "does not use" } else { "uses" }
    );
    let qt = format!(
        "Does Mara {} Helia of 6 kg from Sora? Helia is a camera. During reservation R.",
        if negative { "not use" } else { "use" }
    );
    let mut sd = doc(&st, "source", &["Mara", "Helia", "Sora", "Orbe"]);
    let mut qd = doc(&qt, "question", &["Mara", "Helia", "Sora"]);
    let mut p = prop(atom(
        &st,
        "uses",
        vec![
            ("subject", reference("Mara")),
            ("object", desc_term(&st, "Helia", "6", "Sora")),
        ],
    ));
    let mut qp = prop(atom(
        &qt,
        "uses",
        vec![
            ("subject", reference("Mara")),
            ("object", desc_term(&qt, "Helia", "6", "Sora")),
        ],
    ));
    if negative {
        p["atom"]["polarity"] = json!("negative");
        qp["atom"]["polarity"] = json!("negative");
    }
    sd["claims"] = json!([claim(&st, "use", p)]);
    qd["requests"] = json!([request(&qt, "truth", "status", &qp)]);
    (sd, st, qd, qt)
}
#[test]
fn descriptions_internal_time_is_not_host_time_or_a_current_fact() {
    let (mut sd, st, mut qd, qt) = desc_use_case(false);
    let qualifier = |text: &str| json!({"id":"past","scope_id":"root","kind":"time","body":{"role":"holds_at","value":{"kind":"time","value":"reservation R","format":"literal","evidence":[quote(text,"reservation R")]}},"resolution":"known","evidence":[quote(text,text)]});
    sd["qualifiers"] = json!([qualifier(&st)]);
    qd["qualifiers"] = json!([qualifier(&qt)]);
    sd["claims"][0]["body"]["value"]["atom"]["arguments"][1]["value"]["constraints"][0]["qualifier_ids"] =
        json!(["past"]);
    qd["requests"][0]["target"]["atom"]["arguments"][1]["value"]["constraints"][0]["qualifier_ids"] =
        json!(["past"]);
    // Dependencies explicitly require the qualifier; put it on the host too so
    // this test distinguishes internal versus external use without hidden discharge.
    sd["claims"][0]["qualifier_ids"] = json!(["past"]);
    qd["requests"][0]["qualifier_ids"] = json!(["past"]);
    let r = desc_result(sd.clone(), &st, qd.clone(), &qt);
    assert_eq!(r["complete_answer"], true, "{r}");
    qd["requests"][0]["target"]["atom"]["arguments"][1]["value"]["constraints"][0]["qualifier_ids"] =
        json!([]);
    let r = desc_result(sd, &st, qd, &qt);
    assert_eq!(r["complete_answer"], false, "{r}");
}
#[test]
fn descriptions_multiple_members_keep_mass_and_origin_attached() {
    let st = "Mara admits only Helia of 6 kg from Sora or Orbe of 9 kg from Tara. Helia is a camera. Orbe is a camera.";
    let qt = "Does Mara admit Helia of 9 kg from Tara? Helia is a camera.";
    let mut sd = doc(st, "source", &["Mara", "Helia", "Sora", "Orbe", "Tara"]);
    let (source_template, _, _, _) = desc_case("6");
    sd["claims"] = source_template["claims"].clone();
    // Refresh all fixture evidence for this explicit, independent multi-member text.
    fn evidence(v: &mut Value, text: &str) {
        match v {
            Value::Object(o) => {
                if o.contains_key("quote") && o.contains_key("start_byte") {
                    let q = o["quote"].as_str().unwrap();
                    let q = if text.contains(q) { q } else { text };
                    let e = quote(text, q);
                    *o = e.as_object().unwrap().clone();
                } else {
                    for v in o.values_mut() {
                        evidence(v, text);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    evidence(v, text)
                }
            }
            _ => (),
        }
    }
    evidence(&mut sd["claims"], st);
    sd["claims"][0]["body"]["members"] = json!([
        desc_term(st, "Helia", "6", "Sora"),
        desc_term(st, "Orbe", "9", "Tara")
    ]);
    let mut qd = doc(qt, "question", &["Mara", "Helia", "Tara"]);
    qd["references"][1]["declared_class"] = quote(qt, "camera");
    qd["requests"] = json!([request(
        qt,
        "truth",
        "status",
        &prop(atom(
            qt,
            "admits",
            vec![
                ("subject", reference("Mara")),
                ("object", desc_term(qt, "Helia", "9", "Tara"))
            ]
        ))
    )]);
    let r = desc_result(sd, st, qd, qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r["requests"][0], "support").is_empty(), "{r}");
}
#[test]
fn descriptions_binding_map_does_not_erase_descriptors() {
    let (mut sd, st, mut qd, qt) = desc_use_case(false);
    dependencies(&mut sd);
    dependencies(&mut qd);
    let v = desc_vocab();
    for d in [&mut sd, &mut qd] {
        d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
    }
    let sr = super::super::validate(&sd, &st, &v);
    let qr = super::super::validate(&qd, &qt, &v);
    let engine = Engine {
            possessive_source: &BTreeSet::new(),
            lexical_source: &BTreeSet::new(),
            possessive_question: &BTreeSet::new(),
        source: &sd,
        question: &qd,
        source_report: &sr.report,
        question_report: &qr.report,
        pending_claims: &[],
        vocabulary: &v,
        scope: &json!({"namespace":"fresh-controls"}),
        context_v2: false,
    };
    let a = desc_term(&st, "Helia", "6", "Sora");
    let b = desc_term(&qt, "Helia", "6", "Sora");
    let map = BTreeMap::from([("Helia".to_owned(), "Helia".to_owned())]);
    assert_eq!(engine.mapped_term(&a, &b, &map), Relation::Same);
    let mut other = b.clone();
    other["constraints"][0]["atom"]["arguments"][2]["value"]["decimal"] = json!("8");
    assert_eq!(engine.mapped_term(&a, &other, &map), Relation::Different);
    other = b;
    other["resolution"] = json!("unknown");
    assert_eq!(engine.mapped_term(&a, &other, &map), Relation::Unknown);
}
#[test]
fn descriptions_unknown_host_does_not_block_independent_assertion() {
    let (mut sd, st, _, qt) = desc_use_case(false);
    sd["claims"][0]["body"]["value"]["atom"]["arguments"][1]["value"]["resolution"] =
        json!("unknown");
    // A second explicit source statement, distinct from the descriptive operand.
    let p = prop(atom(
        &st,
        "owns",
        vec![("owner", reference("Mara")), ("object", reference("Orbe"))],
    ));
    sd["claims"]
        .as_array_mut()
        .unwrap()
        .push(claim(&st, "ownership", p));
    let mut qd = doc(&st, "question", &["Mara", "Orbe"]);
    let p = prop(atom(
        &st,
        "owns",
        vec![("owner", reference("Mara")), ("object", reference("Orbe"))],
    ));
    qd["requests"] = json!([request(&st, "truth", "status", &p)]);
    let r = desc_result(sd, &st, qd, &st);
    assert_eq!(r["complete_answer"], true, "{qt}: {r}");
    assert_eq!(r["selected_claim_ids"], json!(["ownership"]), "{r}");
}
#[test]
fn descriptions_internal_guard_stays_pending_without_world_discharge() {
    let (mut sd, st, mut qd, qt) = desc_use_case(false);
    for (d, text, source) in [(&mut sd, &st, true), (&mut qd, &qt, false)] {
        let guard = json!({"id":"internal-guard","scope_id":"root","kind":"guard","body":prop(atom(text,"origin",vec![("subject",reference("Helia")),("origin",reference("Sora"))])),"resolution":"known","evidence":[quote(text,text)]});
        d["qualifiers"] = json!([guard]);
        let p = if source {
            &mut d["claims"][0]["body"]["value"]
        } else {
            &mut d["requests"][0]["target"]
        };
        p["atom"]["arguments"][1]["value"]["constraints"][0]["qualifier_ids"] =
            json!(["internal-guard"]);
    }
    let r = desc_result(sd, &st, qd, &qt);
    assert_eq!(r["complete_answer"], false, "{r}");
}
#[test]
fn descriptions_opposite_polarity_constraints_cannot_complete_a_claim() {
    let (mut sd, st, mut qd, qt) = desc_use_case(false);
    for (d, source) in [(&mut sd, true), (&mut qd, false)] {
        let p = if source {
            &mut d["claims"][0]["body"]["value"]
        } else {
            &mut d["requests"][0]["target"]
        };
        let constraints = p["atom"]["arguments"][1]["value"]["constraints"]
            .as_array_mut()
            .unwrap();
        let mut contrary = constraints[1].clone();
        contrary["atom"]["polarity"] = json!("negative");
        constraints.push(contrary);
    }
    let r = desc_result(sd, &st, qd, &qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r["requests"][0], "support").is_empty(), "{r}");
}
#[test]
fn descriptions_circular_internal_guard_returns_pending_before_recursing() {
    let (mut sd, st, mut qd, qt) = desc_use_case(false);
    for (d, text, source) in [(&mut sd, &st, true), (&mut qd, &qt, false)] {
        let p = if source {
            &mut d["claims"][0]["body"]["value"]
        } else {
            &mut d["requests"][0]["target"]
        };
        p["atom"]["arguments"][1]["value"]["constraints"][0]["qualifier_ids"] = json!(["circular"]);
        let mut body = p.clone();
        body["basis"] = json!("description");
        body["normative_scope"] = Value::Null;
        d["qualifiers"] = json!([{"id":"circular","scope_id":"root","kind":"guard","body":body,"resolution":"known","evidence":[quote(text,text)]}]);
    }
    let r = desc_result(sd, &st, qd, &qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
    assert!(
        rows(&r["requests"][0], "missing_constraints").contains(&json!("unsupported_description")),
        "{r}"
    );
    assert!(rows(&r["requests"][0], "support").is_empty(), "{r}");
}
#[test]
fn descriptions_reference_pattern_retains_complete_described_answer() {
    let (mut sd, st, _, _) = desc_use_case(false);
    sd["references"][1]["declared_class"] = quote(&st, "camera");
    let qt = "Which camera does Mara use?";
    let mut qd = doc(qt, "question", &["Mara", "camera"]);
    qd["references"][1]["id"] = json!("pattern");
    qd["references"][1]["name"] = Value::Null;
    qd["references"][1]["declared_class"] = quote(qt, "camera");
    qd["requests"] = json!([request(
        qt,
        "answer",
        "object",
        &prop(atom(qt, "uses", vec![("subject", reference("Mara"))]))
    )]);
    qd["requests"][0]["class_filter"] =
        json!({"kind":"reference_pattern","reference_id":"pattern"});
    let r = desc_result(sd.clone(), &st, qd, &qt);
    assert_eq!(r["complete_answer"], true, "{r}");
    assert_eq!(
        r["requests"][0]["support"][0]["answer_terms"][0],
        sd["claims"][0]["body"]["value"]["atom"]["arguments"][1]["value"],
        "{r}"
    );
}
#[test]
fn descriptions_mixed_carriers_preserve_qualified_context_and_bindings() {
    for (source_description, question_description) in [(true, false), (false, true), (true, true)] {
        let (mut sd, st, mut qd, qt) = desc_use_case(false);
        if !source_description {
            sd["claims"][0]["body"]["value"]["atom"]["arguments"][1]["value"] = reference("Helia");
        }
        if !question_description {
            qd["requests"][0]["target"]["atom"]["arguments"][1]["value"] = reference("Helia");
        }
        let r = desc_result(sd, &st, qd, &qt);
        let rr = &r["requests"][0];
        if source_description && question_description {
            assert_eq!(r["complete_answer"], true, "{r}");
            assert!(
                rows(&rr["support"][0], "bindings")
                    .iter()
                    .any(|b| b["relation"] == "same_description"),
                "{r}"
            );
        } else {
            assert_eq!(r["complete_answer"], false, "{r}");
            assert_eq!(r["selected_claim_ids"], json!(["use"]), "{r}");
            assert!(rows(&rr["partial"][0], "answer_terms").is_empty(), "{r}");
            assert!(
                rows(&rr["partial"][0], "bindings")
                    .iter()
                    .any(|b| b["relation"] == "description_anchor"
                        && b["anchor_evidence"]["source_name"]["quote"] == "Helia"
                        && b["anchor_evidence"]["question_name"]["quote"] == "Helia"),
                "{r}"
            );
        }
    }
}
fn desc_universal_case() -> (Value, &'static str, Value, &'static str) {
    let st =
        "Para toda cámara del grupo Faro, la masa es 6 kg. Se describe su procedencia como Sora.";
    let qt = "¿Qué masa común tiene toda cámara del grupo Faro descrita como procedente de Sora?";
    let mut sd = doc(st, "source", &["cámara", "Faro", "Sora"]);
    let mut qd = doc(qt, "question", &["cámara", "Faro", "Sora"]);
    for (d, text, var) in [(&mut sd, st, "x"), (&mut qd, qt, "y")] {
        let r = &mut d["references"][0];
        r["id"] = json!(var);
        r["name"] = Value::Null;
        r["declared_class"] = quote(text, "cámara");
        r["scope_id"] = json!("all");
        r["binding_scope_id"] = json!("all");
        let mut domain = r.clone();
        domain["id"] = json!("domain");
        domain["scope_id"] = json!("root");
        domain["binding_scope_id"] = Value::Null;
        domain["denotation"] = json!("class");
        domain["links"] =
            json!([{"kind":"part_of","target":"Faro","evidence":[quote(text,"grupo Faro")]}]);
        d["references"].as_array_mut().unwrap().push(domain);
        d["scopes"].as_array_mut().unwrap().push(json!({"id":"all","parent_id":"root","kind":"binder","branch_id":null,"binding":{"variable":var,"quantifier":"forall","domain_ref":"domain","cardinality":null,"witness_relation":"unspecified","relation_closures":[],"evidence":[quote(text,text)]}}));
    }
    let described = |text: &str, var: &str| json!({"kind":"described_ref","id":var,"constraints":[prop(atom(text,"origin",vec![("subject",reference(var)),("origin",reference("Sora"))]))],"resolution":"known","evidence":[quote(text,text)]});
    let mut p = desc_quantity(st, "x", "6");
    p["atom"]["arguments"][0]["value"] = described(st, "x");
    let mut c = claim(st, "mass", p);
    c["scope_id"] = json!("all");
    sd["claims"] = json!([c]);
    let p = prop(atom(
        qt,
        "quantity",
        vec![
            ("subject", described(qt, "y")),
            (
                "attribute",
                json!({"kind":"symbol","id":"mass","evidence":[quote(qt,"masa")]}),
            ),
        ],
    ));
    let mut r = request(qt, "answer", "amount", &p);
    r["scope_id"] = json!("all");
    r["qualifier_ids"] = json!(["common"]);
    qd["requests"] = json!([r]);
    qd["qualifiers"] = json!([{"id":"common","scope_id":"root","kind":"configuration","body":{"key":"quantity_extent","value":{"kind":"text","value":"común","evidence":[quote(qt,"común")]}},"resolution":"known","evidence":[quote(qt,"común")]}]);
    (sd, st, qd, qt)
}
#[test]
fn descriptions_common_quantity_keeps_verified_universal_operand() {
    let (mut sd, st, mut qd, qt) = desc_universal_case();
    let original_question = qd.clone();
    let result = desc_result(sd.clone(), st, qd.clone(), qt);
    assert_eq!(result["complete_answer"], true, "{result}");
    assert_eq!(
        result["requests"][0]["support"][0]["answer_terms"][0]["decimal"], "6",
        "{result}"
    );
    // An unrelated origin or a constant under forall cannot discharge common.
    qd["requests"][0]["target"]["atom"]["arguments"][0]["value"]["constraints"][0]["atom"]["arguments"]
        [1]["value"] = reference("Faro");
    let result = desc_result(sd.clone(), st, qd, qt);
    assert_eq!(result["complete_answer"], false, "{result}");
    sd["claims"][0]["body"]["value"]["atom"]["arguments"][0]["value"]["id"] = json!("Faro");
    sd["claims"][0]["body"]["value"]["atom"]["arguments"][0]["value"]["constraints"][0]["atom"]["arguments"]
        [0]["value"] = reference("Faro");
    // Retain the genuine question variable; the source constant does not cover it.
    let result = desc_result(sd, st, original_question, qt);
    assert_eq!(result["complete_answer"], false, "{result}");
}
#[test]
fn descriptions_contradictory_closed_member_never_becomes_answer_even_with_alias() {
    let (mut sd, st, mut qd, qt) = desc_case("6");
    let mut alias = sd["references"][2].clone();
    alias["id"] = json!("origin-alias");
    sd["references"].as_array_mut().unwrap().push(alias);
    sd["identity"] = json!([{"kind":"same","left":"Sora","right":"origin-alias","evidence":[quote(&st,"Sora")]}]);
    let c = &mut sd["claims"][0]["body"]["members"][0]["constraints"];
    let mut contrary = c[1].clone();
    contrary["atom"]["polarity"] = json!("negative");
    contrary["atom"]["arguments"][1]["value"] = reference("origin-alias");
    c.as_array_mut().unwrap().push(contrary);
    qd["requests"][0]["mode"] = json!("constraint");
    qd["requests"][0]["slot"] = json!("object");
    qd["requests"][0]["class_filter"] = json!({"kind":"lexical","class_quote":quote(&qt,"camera")});
    qd["requests"][0]["target"]["atom"]["arguments"]
        .as_array_mut()
        .unwrap()
        .pop();
    let result = desc_result(sd, &st, qd, &qt);
    assert_eq!(result["complete_answer"], false, "{result}");
    assert!(
        rows(&result["requests"][0], "support").is_empty(),
        "{result}"
    );
    for w in rows(&result["requests"][0], "partial") {
        assert!(rows(w, "answer_terms").is_empty(), "{result}");
        assert!(w.get("source_constraint").is_some(), "{result}");
    }
}
#[test]
fn descriptions_pattern_preserves_mixed_independent_answers_without_stripping() {
    let (mut sd, st, _, _) = desc_use_case(false);
    sd["references"][1]["declared_class"] = quote(&st, "camera");
    sd["references"][3]["declared_class"] = quote(&st, "camera");
    sd["claims"].as_array_mut().unwrap().push(claim(
        &st,
        "other-use",
        prop(atom(
            &st,
            "uses",
            vec![
                ("subject", reference("Mara")),
                ("object", reference("Orbe")),
            ],
        )),
    ));
    let qt = "Which camera does Mara use?";
    let mut qd = doc(qt, "question", &["Mara", "camera"]);
    qd["references"][1]["id"] = json!("pattern");
    qd["references"][1]["name"] = Value::Null;
    qd["references"][1]["declared_class"] = quote(qt, "camera");
    qd["requests"] = json!([request(
        qt,
        "answer",
        "object",
        &prop(atom(qt, "uses", vec![("subject", reference("Mara"))]))
    )]);
    qd["requests"][0]["class_filter"] =
        json!({"kind":"reference_pattern","reference_id":"pattern"});
    let result = desc_result(sd.clone(), &st, qd.clone(), qt);
    assert!(
        rows(&result, "selected_claim_ids").contains(&json!("use")),
        "{result}"
    );
    assert!(
        rows(&result, "selected_claim_ids").contains(&json!("other-use")),
        "{result}"
    );
    assert!(
        rows(&result["requests"][0], "support")
            .iter()
            .any(|w| rows(w, "answer_terms")
                .iter()
                .any(|t| t["kind"] == "described_ref")),
        "{result}"
    );
    sd["references"][3]["declared_class"] = Value::Null;
    let result = desc_result(sd, &st, qd, qt);
    assert!(
        rows(&result, "selected_claim_ids").contains(&json!("use")),
        "{result}"
    );
    assert!(
        !rows(&result, "selected_claim_ids").contains(&json!("other-use")),
        "{result}"
    );
}
#[test]
fn descriptions_partial_anchor_does_not_admit_another_named_root() {
    let (sd, st, mut qd, qt) = desc_use_case(false);
    qd["requests"][0]["target"]["atom"]["arguments"][1]["value"] =
        desc_term(&qt, "Sora", "6", "Helia");
    let result = desc_result(sd, &st, qd, &qt);
    assert_eq!(result["complete_answer"], false, "{result}");
    assert!(rows(&result, "selected_claim_ids").is_empty(), "{result}");
}
#[test]
fn descriptions_partial_associations_accept_both_carriers_only_with_real_context() {
    for (source_description, question_description) in
        [(true, false), (false, true), (true, true), (false, false)]
    {
        let (mut sd, st, mut qd, qt) = desc_use_case(false);
        if !source_description {
            sd["claims"][0]["body"]["value"]["atom"]["arguments"][1]["value"] = reference("Helia");
        }
        if !question_description {
            qd["requests"][0]["target"]["atom"]["arguments"][1]["value"] = reference("Helia");
        }
        for (d, text) in [(&mut sd, &st), (&mut qd, &qt)] {
            d["references"][1]["links"] =
                json!([{"kind":"associated_with","target":"Sora","evidence":[quote(text,"Sora")]}]);
        }
        let r = desc_result(sd.clone(), &st, qd.clone(), &qt);
        assert!(
            rows(&r, "selected_claim_ids").contains(&json!("use")),
            "{source_description}/{question_description}: {r}"
        );
        qd["references"][1]["links"][0]["target"] = json!("Mara");
        let r = desc_result(sd, &st, qd, &qt);
        assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
    }
}
#[test]
fn descriptions_unanchored_anonymous_source_is_not_qualified_evidence() {
    let (mut sd, st, mut qd, qt) = desc_use_case(false);
    sd["references"][1]["name"] = Value::Null;
    sd["references"][1]["declared_class"] = quote(&st, "camera");
    qd["references"][1]["links"] =
        json!([{"kind":"associated_with","target":"Sora","evidence":[quote(&qt,"Sora")]}]);
    let r = desc_result(sd, &st, qd, &qt);
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
    assert_eq!(r["complete_answer"], false, "{r}");
}
#[test]
fn descriptions_deferred_request_preserves_payload_and_both_failure_reasons() {
    let (mut sd, st, mut qd, qt) = desc_use_case(false);
    qd["qualifiers"] = json!([{"id":"internal","scope_id":"root","kind":"guard","body":prop(atom(&qt,"origin",vec![("subject",reference("Helia")),("origin",reference("Sora"))])),"resolution":"known","evidence":[quote(&qt,&qt)]}]);
    let t = &mut qd["requests"][0]["target"]["atom"]["arguments"][1]["value"];
    t["constraints"][0]["qualifier_ids"] = json!(["internal"]);
    t["constraints"][0]["atom"]["arguments"][2]["value"]["unit"] = json!("g");
    let mut v = desc_vocab();
    v["attribute_profiles"] = json!({"version":"documentary_attribute_profiles_v1","targets":["quantity.amount"],"rows":[{"attribute_id":"mass","target":"quantity.amount","term_kinds":["quantity"],"quantity_units":["kg"]}]});
    for d in [&mut sd, &mut qd] {
        dependencies(d);
        d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
    }
    let a = super::super::validate(&sd, &st, &v);
    let b = super::super::validate(&qd, &qt, &v);
    assert_eq!(b.report["status"], "partial", "{}", b.report);
    let r = classify_pair(
        &a,
        &b,
        &v,
        &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
    );
    let rr = &r["requests"][0];
    assert_eq!(r["complete_answer"], false, "{r}");
    assert_eq!(
        rr["unresolved"][0]["request_constraint"], qd["requests"][0],
        "{r}"
    );
    for code in ["attribute_profile_mismatch", "unsupported_description"] {
        assert!(
            rows(rr, "missing_constraints").contains(&json!(code)),
            "{r}"
        );
    }
}
#[test]
fn descriptions_verified_quantified_partial_survives_basis_debt() {
    let (sd, st, mut qd, qt) = desc_universal_case();
    qd["requests"][0]["basis_requirement"] = json!({"kind":"exact","value":"observation"});
    let r = desc_result(sd.clone(), st, qd.clone(), qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert_eq!(r["selected_claim_ids"], json!(["mass"]), "{r}");
    let w = &r["requests"][0]["partial"][0];
    assert!(rows(w, "answer_terms").is_empty(), "{r}");
    assert_eq!(w["source_constraint"], sd["claims"][0]["body"], "{r}");
    assert!(
        rows(w, "missing_constraints").contains(&json!("basis")),
        "{r}"
    );
    assert!(
        rows(w, "bindings")
            .iter()
            .any(|b| b["relation"] == "alpha_pattern"),
        "{r}"
    );
    let mut sd = sd;
    sd["scopes"][1]["binding"]["quantifier"] = json!("exists");
    let r = desc_result(sd, st, qd, qt);
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
}
fn desc_alias_component_case(
    source_class: &str,
    question_class: &str,
) -> (Value, String, Value, String) {
    let st = format!(
        "Mara uses Helia of 6 kg from Sora. Helia is a {source_class}. H1 is another label for the same Helia."
    );
    let qt = format!(
        "Does Mara use Helia of 6 kg from Sora? Helia is a {question_class}. H1 is another label for the same Helia."
    );
    let mut sd = doc(&st, "source", &["Mara", "Helia", "Sora"]);
    let mut qd = doc(&qt, "question", &["Mara", "Helia", "Sora"]);
    for (d, t, class) in [(&mut sd, &st, source_class), (&mut qd, &qt, question_class)] {
        d["references"][1]["declared_class"] = quote(t, class);
        let mut alias = d["references"][1].clone();
        alias["id"] = json!("alias");
        alias["name"] = quote(t, "H1");
        alias["mention"] = quote(t, "H1");
        alias["declared_class"] = Value::Null;
        d["references"].as_array_mut().unwrap().push(alias);
        d["identity"] = json!([{"kind":"same","left":"Helia","right":"alias","evidence":[quote(t,"H1 is another label for the same Helia")]}]);
    }
    sd["claims"] = json!([claim(
        &st,
        "use",
        prop(atom(
            &st,
            "uses",
            vec![
                ("subject", reference("Mara")),
                ("object", desc_term(&st, "Helia", "6", "Sora"))
            ]
        ))
    )]);
    qd["requests"] = json!([request(
        &qt,
        "truth",
        "status",
        &prop(atom(
            &qt,
            "uses",
            vec![
                ("subject", reference("Mara")),
                ("object", desc_term(&qt, "Helia", "6", "Sora"))
            ]
        ))
    )]);
    (sd, st, qd, qt)
}
#[test]
fn descriptions_alias_without_class_cannot_hide_divergent_root_classes() {
    let (sd, st, qd, qt) = desc_alias_component_case("battery", "camera");
    let r = desc_result(sd, &st, qd, &qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
    let (sd, st, qd, qt) = desc_alias_component_case("camera", "camera");
    let r = desc_result(sd, &st, qd, &qt);
    assert_eq!(r["complete_answer"], true, "{r}");
}
#[test]
fn descriptions_quantified_domains_cannot_hide_semantic_type_disagreement() {
    let (mut sd, st, mut qd, qt) = desc_universal_case();
    qd["requests"][0]["qualifier_ids"] = json!([]);
    qd["qualifiers"] = json!([]);
    let mut v = desc_vocab();
    v["semantic_type_rows"] = json!([{"id":"type-a","class_quotes":["cámara"]},{"id":"type-b","class_quotes":["cámara"]}]);
    for (d, text, typ) in [(&mut sd, st, "type-a"), (&mut qd, qt, "type-b")] {
        d["references"][3]["name"] = quote(text, "Faro");
        d["references"][3]["semantic_type"] =
            json!({"type_id":typ,"evidence":[quote(text,"cámara")]});
        dependencies(d);
        d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
    }
    let a = super::super::validate(&sd, st, &v);
    let b = super::super::validate(&qd, qt, &v);
    assert_eq!(a.report["status"], "complete", "{}", a.report);
    assert_eq!(b.report["status"], "complete", "{}", b.report);
    let r = classify_pair(
        &a,
        &b,
        &v,
        &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
    );
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
}
#[test]
fn descriptions_quantified_alias_representatives_preserve_partial_proof() {
    let (mut sd, st, mut qd, qt) = desc_universal_case();
    qd["requests"][0]["basis_requirement"] = json!({"kind":"exact","value":"observation"});
    for (d, text, var, alias, source) in [
        (&mut sd, st, "x", "alias-x", true),
        (&mut qd, qt, "y", "alias-y", false),
    ] {
        let mut r = d["references"][0].clone();
        r["id"] = json!(alias);
        r["binding_scope_id"] = Value::Null;
        d["references"].as_array_mut().unwrap().push(r);
        d["identity"] =
            json!([{"kind":"same","left":var,"right":alias,"evidence":[quote(text,"cámara")]}]);
        let p = if source {
            &mut d["claims"][0]["body"]["value"]
        } else {
            &mut d["requests"][0]["target"]
        };
        let t = &mut p["atom"]["arguments"][0]["value"];
        t["id"] = json!(alias);
        t["constraints"][0]["atom"]["arguments"][0]["value"] = reference(alias);
    }
    let r = desc_result(sd.clone(), st, qd.clone(), qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert_eq!(r["selected_claim_ids"], json!(["mass"]), "{r}");
    assert_eq!(
        r["requests"][0]["partial"][0]["source_constraint"], sd["claims"][0]["body"],
        "{r}"
    );
    assert!(
        rows(&r["requests"][0]["partial"][0], "bindings")
            .iter()
            .any(|b| b["alpha_alias_roots"]["source_variable"] == "x"
                && b["alpha_alias_roots"]["question_variable"] == "y"),
        "{r}"
    );
    let reset = |d: &mut Value, source: bool, var: &str, alias: &str| {
        d["identity"] = json!([]);
        d["references"]
            .as_array_mut()
            .unwrap()
            .retain(|r| r["id"] != alias);
        let p = if source {
            &mut d["claims"][0]["body"]["value"]
        } else {
            &mut d["requests"][0]["target"]
        };
        p["atom"]["arguments"][0]["value"]["id"] = json!(var);
        p["atom"]["arguments"][0]["value"]["constraints"][0]["atom"]["arguments"][0]["value"] =
            reference(var);
    };
    let mut one = qd.clone();
    reset(&mut one, false, "y", "alias-y");
    let one_result = desc_result(sd.clone(), st, one, qt);
    assert_eq!(
        one_result["selected_claim_ids"],
        json!(["mass"]),
        "{one_result}"
    );
    let mut one = sd.clone();
    reset(&mut one, true, "x", "alias-x");
    let one_result = desc_result(one, st, qd.clone(), qt);
    assert_eq!(
        one_result["selected_claim_ids"],
        json!(["mass"]),
        "{one_result}"
    );
    sd["identity"] = json!([]);
    let r = desc_result(sd, st, qd, qt);
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
}
#[test]
fn descriptions_domain_alias_denotation_cannot_be_hidden_by_nominal_anchor() {
    let (mut sd, st, mut qd, qt) = desc_universal_case();
    qd["requests"][0]["qualifier_ids"] = json!([]);
    qd["qualifiers"] = json!([]);
    for (d, t) in [(&mut sd, st), (&mut qd, qt)] {
        d["references"][3]["name"] = quote(t, "Faro");
    }
    let mut alias = sd["references"][3].clone();
    alias["id"] = json!("domain-alias");
    alias["denotation"] = json!("group");
    sd["references"].as_array_mut().unwrap().push(alias);
    sd["identity"] = json!([{"kind":"same","left":"domain","right":"domain-alias","evidence":[quote(st,"grupo Faro")]}]);
    let r = desc_result(sd, st, qd, qt);
    assert_eq!(r["complete_answer"], false, "{r}");
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
}
#[test]
fn descriptions_quantified_alias_inherits_associations_without_duplicating_links() {
    let (mut sd, st, mut qd, qt) = desc_universal_case();
    qd["requests"][0]["basis_requirement"] = json!({"kind":"exact","value":"observation"});
    for (d, text) in [(&mut sd, st), (&mut qd, qt)] {
        d["references"][0]["links"] =
            json!([{"kind":"associated_with","target":"Faro","evidence":[quote(text,"Faro")]}]);
    }
    let base = desc_result(sd.clone(), st, qd.clone(), qt);
    assert_eq!(base["selected_claim_ids"], json!(["mass"]), "{base}");
    let mut alias = sd["references"][0].clone();
    alias["id"] = json!("alias-x");
    alias["binding_scope_id"] = Value::Null;
    alias["links"] = json!([]);
    sd["references"].as_array_mut().unwrap().push(alias);
    sd["identity"] =
        json!([{"kind":"same","left":"x","right":"alias-x","evidence":[quote(st,"cámara")]}]);
    let t = &mut sd["claims"][0]["body"]["value"]["atom"]["arguments"][0]["value"];
    t["id"] = json!("alias-x");
    t["constraints"][0]["atom"]["arguments"][0]["value"] = reference("alias-x");
    let r = desc_result(sd.clone(), st, qd.clone(), qt);
    assert_eq!(r["selected_claim_ids"], base["selected_claim_ids"], "{r}");
    assert_eq!(r["complete_answer"], false, "{r}");
    assert_eq!(
        r["requests"][0]["partial"][0]["source_constraint"], sd["claims"][0]["body"],
        "{r}"
    );
    assert!(
        rows(&r["requests"][0]["partial"][0], "answer_terms").is_empty(),
        "{r}"
    );
    // A genuinely additional inherited restriction must still block the match.
    sd["references"][4]["links"] =
        json!([{"kind":"associated_with","target":"Sora","evidence":[quote(st,"Sora")]}]);
    let r = desc_result(sd.clone(), st, qd.clone(), qt);
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
    sd["references"][4]["links"] = json!([]);
    qd["references"][0]["links"][0]["target"] = json!("Sora");
    qd["references"][0]["links"][0]["evidence"] = json!([quote(qt, "Sora")]);
    let r = desc_result(sd.clone(), st, qd.clone(), qt);
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
    qd["references"][0]["links"][0]["target"] = json!("Faro");
    qd["references"][0]["links"][0]["evidence"] = json!([quote(qt, "Faro")]);
    sd["identity"] = json!([]);
    let r = desc_result(sd, st, qd, qt);
    assert!(rows(&r, "selected_claim_ids").is_empty(), "{r}");
}
