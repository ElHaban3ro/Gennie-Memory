use super::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
fn fixture() -> Value {
    serde_json::from_str(include_str!("nominal_event_context_fixture.json")).unwrap()
}
fn hash(v: &Value) -> String {
    Sha256::digest(serde_json::to_vec(v).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn rehash(a: &mut Value) {
    let h = hash(&a["vocabulary"]);
    for side in ["source", "question"] {
        a[side]["raw"]["vocabulary_sha256"] = json!(h);
    }
}
fn complete(a: &Value, side: &str) -> DocumentResult {
    let r = validate(
        &a[side]["raw"],
        a[side]["text"].as_str().unwrap(),
        &a["vocabulary"],
    );
    assert_eq!(r.report["status"], "complete", "{}", r.report);
    r
}
fn pair(a: &Value) -> Value {
    classify_pair_context(
        &complete(a, "source"),
        &complete(a, "question"),
        &a["vocabulary"],
        &a["pair_scope"],
    )
}
fn partial(r: &Value) -> &[Value] {
    r["requests"][0]["partial"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
}
fn quote(t: &str, part: &str) -> Value {
    let a = t.find(part).unwrap();
    json!({"start_byte":a,"end_byte":a+part.len(),"quote":part})
}
#[test]
fn nominal_event_context_positive_preserves_all_operands_and_unanswered_debts() {
    let a = fixture();
    let r = pair(&a);
    assert_eq!(partial(&r).len(), 1, "{r}");
    let w = &partial(&r)[0];
    assert_eq!(w["kind"], "source_bound_linguistic_context");
    assert_eq!(w["source_constraint"], a["source"]["raw"]["claims"][0]);
    assert_eq!(w["source_body"], a["source"]["raw"]["claims"][0]["body"]);
    assert_eq!(w["vocabulary"], a["vocabulary"]);
    assert_eq!(w["question_document"], a["question"]["raw"]);
    assert_eq!(
        w["nominal_event_context"]["context_ids"],
        json!(["place-pp", "person-pp"])
    );
    for k in [
        "patient_identity_proven",
        "requested_role_proven",
        "core_independently_asserted",
        "full_answer_supported",
    ] {
        assert_eq!(w["nominal_event_context"][k], false);
    }
    for k in [
        "bindings",
        "answer_terms",
        "answer_propositions",
        "requirement_states",
    ] {
        assert_eq!(w[k], json!([]));
    }
    for d in [
        "answer_slot",
        "patient_identity",
        "context_attachment",
        "requested_role",
    ] {
        assert!(
            w["missing_constraints"]
                .as_array()
                .unwrap()
                .contains(&json!(d))
        );
    }
    assert_eq!(r["complete_answer"], false);
    assert!(r["requests"][0]["support"].as_array().unwrap().is_empty());
}
#[test]
fn nominal_event_context_closed_schema_rejects_attachment_region_duplicates_and_utf8() {
    for i in 0..10 {
        let mut a = fixture();
        let b = &mut a["source"]["raw"]["claims"][0]["body"];
        match i {
            0 => b["region"]["core_id"] = json!("foreign"),
            1 => b["contexts"][0]["region_id"] = json!("foreign"),
            2 => b["contexts"][0]["attachment"]["resolution"] = json!("event"),
            3 => b["contexts"][1]["id"] = b["contexts"][0]["id"].clone(),
            4 => b["construction"] = json!("negative_se_measurement_with_until_v1"),
            5 => b["contexts"][1] = b["contexts"][0].clone(),
            6 => b["core"]["action_id"] = json!("grant"),
            7 => b["core"]["action_evidence"]["start_byte"] = json!(20),
            8 => b["core"]["actuality"] = json!("planned"),
            _ => b["contexts"][0]["attachment"]["target"] = json!("patient"),
        };
        let r = validate(
            &a["source"]["raw"],
            a["source"]["text"].as_str().unwrap(),
            &a["vocabulary"],
        );
        assert_ne!(r.report["status"], "complete", "case {i}: {}", r.report);
    }
}
#[test]
fn nominal_event_context_permission_is_separate_and_requires_nominal_and_remove() {
    for i in 0..4 {
        let mut a = fixture();
        match i {
            0 => {
                a["vocabulary"]["request_context_profile"]["features"] =
                    json!(["nominal_complement_de"])
            }
            1 => {
                a["vocabulary"]["request_context_profile"]["features"] =
                    json!(["nonagentive_nominal_event_with_contexts_v1"])
            }
            2 => a["vocabulary"]["fixed_action_rows"] = json!(["grant"]),
            _ => {
                a["vocabulary"]["request_context_profile"]["features"] = json!([
                    "nominal_complement_de",
                    "linguistic_composition_until_measurement_v1"
                ]);
                a["vocabulary"]["fixed_attribute_rows"] = json!(["duration"]);
            }
        }
        rehash(&mut a);
        assert_ne!(
            validate(
                &a["source"]["raw"],
                a["source"]["text"].as_str().unwrap(),
                &a["vocabulary"]
            )
            .report["status"],
            "complete",
            "case {i}"
        );
    }
}
#[test]
fn nominal_event_context_other_action_namespace_or_required_basis_does_not_select() {
    for i in 0..4 {
        let mut a = fixture();
        match i {
            0 => {
                a["question"]["raw"]["requests"][0]["target"]["atom"]["arguments"][1]["value"]["id"] =
                    json!("grant")
            }
            1 => a["question"]["raw"]["domain_namespace"] = json!("other"),
            2 => {
                a["question"]["raw"]["requests"][0]["basis_requirement"] =
                    json!({"kind":"exact","value":"description"})
            }
            _ => {
                a["question"]["raw"]["requests"][0]["target"]["atom"]["polarity"] =
                    json!("negative")
            }
        };
        let r = pair(&a);
        assert!(partial(&r).is_empty(), "case {i}: {r}");
        assert_eq!(r["selected_claim_ids"], json!([]));
    }
}
#[test]
fn nominal_event_context_other_name_or_nominal_pattern_does_not_select() {
    for (before, after) in [
        ("Paz", "Sol"),
        ("acceso", "egreso"),
        ("restricción", "autorización"),
    ] {
        let mut a = fixture();
        let old = a["question"].to_string();
        let text = old.replace(before, after);
        a["question"] = serde_json::from_str(&text).unwrap();
        let t = a["question"]["text"].as_str().unwrap().to_owned();
        // Rebuild offsets after a different-length lexical replacement.
        fn fix(v: &mut Value, t: &str) {
            if let Some(o) = v.as_object_mut() {
                if o.contains_key("start_byte")
                    && o.contains_key("end_byte")
                    && o.contains_key("quote")
                {
                    let part = o["quote"].as_str().unwrap().to_owned();
                    let start = t.find(&part).unwrap();
                    o.insert("start_byte".into(), json!(start));
                    o.insert("end_byte".into(), json!(start + part.len()));
                } else {
                    for x in o.values_mut() {
                        fix(x, t);
                    }
                }
            } else if let Some(xs) = v.as_array_mut() {
                for x in xs {
                    fix(x, t);
                }
            }
        }
        fix(&mut a["question"]["raw"], &t);
        a["question"]["raw"]["text_sha256"] = json!(
            Sha256::digest(t.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        let r = pair(&a);
        assert!(partial(&r).is_empty(), "{before}: {r}");
    }
}
#[test]
fn nominal_event_context_canonical_internal_ids_do_not_resolve_patient_or_attachment() {
    let a = fixture();
    let b = &a["source"]["raw"]["claims"][0]["body"];
    let mut renamed = b.clone();
    renamed["composition_id"] = json!("patient");
    renamed["core"]["id"] = json!("person");
    renamed["region"]["id"] = json!("place");
    renamed["region"]["core_id"] = json!("person");
    for (i, p) in renamed["contexts"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        p["id"] = json!(format!("renamed-{i}"));
        p["region_id"] = json!("place");
    }
    assert_eq!(
        linguistic_composition::canonical_ast(b),
        linguistic_composition::canonical_ast(&renamed)
    );
    let mut changed = renamed;
    changed["core"]["patient"]["id"] = json!("person");
    assert_ne!(
        linguistic_composition::canonical_ast(b),
        linguistic_composition::canonical_ast(&changed)
    );
}
#[test]
fn nominal_event_context_consumer_cut_seeds_patient_and_tail_not_named_pps() {
    let a = fixture();
    let v = &a["vocabulary"];
    let d = &a["source"]["raw"];
    context_budget::with_operation(v, d, || {
        let blocked = linguistic_composition::blocked_nodes(d);
        for id in ["patient", "tail"] {
            assert!(
                blocked.contains(&("reference".into(), id.into())),
                "{blocked:?}"
            );
        }
        for id in ["place", "person"] {
            assert!(
                !blocked.contains(&("reference".into(), id.into())),
                "{blocked:?}"
            );
        }
    })
    .unwrap();
}
#[test]
fn nominal_event_context_scalar_evidence_without_feature_ten_and_late_work_share_budget() {
    let a = fixture();
    let v = &a["vocabulary"];
    let d = &a["source"]["raw"];
    assert!(!request_context_profile::permits(
        v,
        "lexical_clause_context"
    ));
    assert!(
        context_budget::with_operation(v, d, || {
            assert!(context_budget::composition_enabled());
            let _ = pair(&a);
            assert!(!context_budget::charge(1, 16 * 1024 * 1024));
        })
        .is_err()
    );
    let mut absent = v.clone();
    absent
        .as_object_mut()
        .unwrap()
        .remove("request_context_profile");
    context_budget::with_operation(&absent, d, || {
        assert!(!context_budget::composition_enabled());
        assert!(linguistic_composition::blocked_nodes(d).is_empty());
    })
    .unwrap();
}
#[test]
fn nominal_event_context_same_patient_consumer_is_cut_but_named_pp_fact_answers() {
    let mut a = fixture();
    let source_text = format!("{} Paz pesa 8.", a["source"]["text"].as_str().unwrap());
    let question_text = format!("{} ¿Pesa Paz 8?", a["question"]["text"].as_str().unwrap());
    for (side, text) in [("source", &source_text), ("question", &question_text)] {
        a[side]["text"] = json!(text);
        a[side]["raw"]["text_sha256"] = json!(
            Sha256::digest(text.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
    }
    let atom = |text: &str, rid: &str| json!({"predicate":"quantity","arguments":[{"role":"subject","value":{"kind":"ref","id":rid}},{"role":"attribute","value":{"kind":"symbol","id":"mass","evidence":[quote(text,"pesa")] }},{"role":"amount","value":{"kind":"quantity","decimal":"8","unit":null,"bound":"exact","evidence":[quote(text,"8")]}}],"polarity":"positive","evidence":[quote(text,"Paz pesa 8")]});
    let source_atom = atom(&source_text, "person");
    let fact = json!({"id":"independent","scope_id":"root","qualifier_ids":[],"evidence":[quote(&source_text,"Paz pesa 8")],"body":{"kind":"proposition","value":{"atom":source_atom,"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]}}});
    let mut dependent = fact.clone();
    dependent["id"] = json!("patient-consumer");
    dependent["body"]["value"]["atom"]["arguments"][0]["value"]["id"] = json!("patient");
    a["source"]["raw"]["claims"]
        .as_array_mut()
        .unwrap()
        .extend([fact, dependent]);
    // The added rows use effective implicit reference edges rather than a claim-to-claim hint.
    for cid in ["independent", "patient-consumer"] {
        let rid = if cid == "independent" {
            "person"
        } else {
            "patient"
        };
        a["source"]["raw"]["dependencies"].as_array_mut().unwrap().extend([json!({"consumer":{"kind":"claim","id":cid},"requires":{"kind":"reference","id":rid},"role":"identity"}),json!({"consumer":{"kind":"claim","id":cid},"requires":{"kind":"scope","id":"root"},"role":"binding"})]);
    }
    let mut question_atom = atom(&source_text, "person");
    question_atom["evidence"] = json!([quote(&question_text, "Paz 8")]);
    question_atom["arguments"][1]["value"]["evidence"] = json!([quote(&question_text, "Pesa")]);
    question_atom["arguments"][2]["value"]["evidence"] = json!([quote(&question_text, "8")]);
    let ask = &mut a["question"]["raw"]["requests"][0];
    ask["target"] =
        json!({"atom":question_atom,"judgment":"fact","status":"state","qualifier_ids":[]});
    ask["evidence"] = json!([quote(&question_text, "Pesa Paz 8")]);
    let r = pair(&a);
    assert_eq!(r["complete_answer"], true, "{r}");
    assert_eq!(r["selected_claim_ids"], json!(["independent"]), "{r}");
    context_budget::with_operation(&a["vocabulary"], &a["source"]["raw"], || {
        let blocked = linguistic_composition::blocked_nodes(&a["source"]["raw"]);
        assert!(blocked.contains(&("claim".into(), "patient-consumer".into())));
        assert!(!blocked.contains(&("claim".into(), "independent".into())));
    })
    .unwrap();
}
#[test]
fn nominal_event_context_prepositional_phrase_cannot_absorb_se_or_action() {
    for complement in ["Orbe se", "Orbe se eliminó"] {
        let mut a = fixture();
        let text = a["source"]["text"].as_str().unwrap().to_owned();
        let mention = quote(&text, complement);
        let r = a["source"]["raw"]["references"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["id"] == "place")
            .unwrap();
        r["mention"] = mention.clone();
        r["name"] = mention.clone();
        let pp = &mut a["source"]["raw"]["claims"][0]["body"]["contexts"][0];
        pp["complement_evidence"] = mention;
        pp["phrase_evidence"] = quote(&text, &format!("En {complement}"));
        let result = validate(&a["source"]["raw"], &text, &a["vocabulary"]);
        assert_eq!(result.report["status"], "invalid", "{}", result.report);
        let pair = classify_pair_context(
            &result,
            &complete(&a, "question"),
            &a["vocabulary"],
            &a["pair_scope"],
        );
        assert_eq!(pair["selected_claim_ids"], json!([]));
        assert!(partial(&pair).is_empty());
    }
}
