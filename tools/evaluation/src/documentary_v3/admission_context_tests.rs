use super::*;
#[test]
fn context_admission_setting_context_preserves_all_reference_links() {
    for basis in ["capability", "description"] {
        for kind in ["owned_by", "associated_with", "part_of", "member_of"] {
            for same in [false, true] {
                for nested in [false, true] {
                    let st = "Helia no admite ModoB. ModoB de Orbe. Orbe en Eira.";
                    let qt = if same {
                        "¿Qué presión usa Helia con ModoB de Orbe? Orbe en Eira."
                    } else if nested {
                        "¿Qué presión usa Helia con ModoB de Orbe? Orbe en Helia. Eira."
                    } else {
                        "¿Qué presión usa Helia con ModoB de Helia? Orbe en Eira."
                    };
                    let mut v = vocabulary();
                    v["assignment_projection"] =
                        json!({"version":"uses_qualifier_assignment_v1","context_keys":["mode"]});
                    v["relevance_links"].as_array_mut().unwrap().push(json!({"id":"admission_caution","source_predicate":"admits","target_predicate":"quantity","bindings":[["source.subject","target.subject"]],"carry_roles":["source.object"],"conclusion":"partial_intermediate_only","max_hops":1,"allow_reverse":false,"allow_transitive_composition":false}));
                    let mut source = doc(st, "source", &["Helia", "ModoB", "Orbe", "Eira"]);
                    let mut question = doc(qt, "question", &["Helia", "ModoB", "Orbe", "Eira"]);
                    source["references"][1]["links"] = json!([{"kind":kind,"target":"Orbe","evidence":[quote(st,"ModoB de Orbe")]}]);
                    question["references"][1]["links"] = json!([{"kind":kind,"target":if same || nested {"Orbe"}else{"Helia"},"evidence":[quote(qt,if same || nested {"ModoB de Orbe"}else{"ModoB de Helia"})]}]);
                    if nested {
                        source["references"][2]["links"] = json!([{"kind":"part_of","target":"Eira","evidence":[quote(st,"Orbe en Eira")]}]);
                        question["references"][2]["links"] = json!([{"kind":"part_of","target":if same {"Eira"}else{"Helia"},"evidence":[quote(qt,if same {"Orbe en Eira"}else{"Orbe en Helia"})]}]);
                    }
                    let mut p = prop(atom(
                        st,
                        "admits",
                        vec![
                            ("subject", reference("Helia")),
                            ("object", reference("ModoB")),
                        ],
                    ));
                    p["atom"]["polarity"] = json!("negative");
                    p["basis"] = json!(basis);
                    source["claims"] = json!([claim(st, "admission", p.clone())]);
                    question["qualifiers"] = json!([{"id":"setting","scope_id":"root","kind":"configuration","body":{"key":"mode","value":reference("ModoB")},"resolution":"known","evidence":[quote(qt,"ModoB")]}]);
                    let mut ask = request(
                        qt,
                        "answer",
                        "amount",
                        &prop(atom(
                            qt,
                            "quantity",
                            vec![
                                ("subject", reference("Helia")),
                                (
                                    "attribute",
                                    json!({"kind":"symbol","id":"pressure","evidence":[quote(qt,"presión")]}),
                                ),
                            ],
                        )),
                    );
                    ask["qualifier_ids"] = json!(["setting"]);
                    ask["basis_requirement"] = json!({"kind":"exact","value":"assignment"});
                    question["requests"] = json!([ask]);
                    for d in [&mut source, &mut question] {
                        d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&v).unwrap()));
                        dependencies(d);
                    }
                    let a = super::super::super::validate(&source, st, &v);
                    let b = super::super::super::validate(&question, qt, &v);
                    assert_eq!(a.report["status"], "complete", "{}", a.report);
                    assert_eq!(b.report["status"], "complete", "{}", b.report);
                    let out = classify_pair_context(
                        &a,
                        &b,
                        &v,
                        &json!({"namespace":"fresh-controls","configuration":[],"time_selector":null,"closed_world":false}),
                    );
                    assert_eq!(
                        !rows(&out, "selected_claim_ids").is_empty(),
                        same,
                        "{kind}/{same}: {out}"
                    );
                    assert_eq!(out["complete_answer"], false);
                    if same {
                        let w = &out["requests"][0]["partial"][0];
                        assert_eq!(w["answer_terms"], json!([]));
                        assert_eq!(w["answer_propositions"], json!([p]));
                    }
                }
            }
        }
    }
}
