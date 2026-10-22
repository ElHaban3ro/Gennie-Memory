use super::*;
use serde_json::json;
use sha2::{Digest, Sha256};
const ST: &str =
    include_str!("../../../../tests/fixtures/documentary-nominal-possessive-v1/source.txt");
const QT: &str =
    include_str!("../../../../tests/fixtures/documentary-nominal-possessive-v1/question.txt");
fn fixture(name: &str) -> Value {
    serde_json::from_str(match name {
        "source" => {
            include_str!("../../../../tests/fixtures/documentary-nominal-possessive-v1/source.json")
        }
        "question" => include_str!(
            "../../../../tests/fixtures/documentary-nominal-possessive-v1/question.json"
        ),
        _ => include_str!(
            "../../../../tests/fixtures/documentary-nominal-possessive-v1/vocabulary.json"
        ),
    })
    .unwrap()
}
fn rehash(d: &mut Value, v: &Value) {
    d["vocabulary_sha256"] = json!(
        Sha256::digest(serde_json::to_vec(v).unwrap())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
}
fn validated(d: &Value, t: &str, v: &Value) -> DocumentResult {
    let r = validate(d, t, v);
    assert_eq!(r.report["status"], "complete", "{}", r.report);
    r
}
fn pair(s: &Value, q: &Value, v: &Value) -> Value {
    classify_pair_context(
        &validated(s, ST, v),
        &validated(q, QT, v),
        v,
        &json!({"namespace":"possessive-controls","configuration":[],"time_selector":null,"closed_world":false}),
    )
}
#[test]
fn possessive_complete_carrier_preserves_full_partial_and_open_slot() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    let q = fixture("question");
    let result = pair(&s, &q, &v);
    assert_eq!(result["complete_answer"], false, "{result}");
    let w = &result["requests"][0]["partial"][0];
    assert_eq!(w["kind"], "nominal_possessive_context", "{result}");
    for k in ["answer_terms", "answer_propositions", "bindings"] {
        assert_eq!(w[k], json!([]));
    }
    assert_eq!(w["source_document"], s);
    assert_eq!(w["question_document"], q);
    assert_eq!(w["source_constraint"], s["claims"][0]);
    for (a, b) in [
        ("source_references", "references"),
        ("source_scopes", "scopes"),
        ("source_qualifiers", "qualifiers"),
        ("source_dependencies", "dependencies"),
    ] {
        assert_eq!(w[a], s[b]);
    }
    assert_eq!(w["possessive_context"]["possessive_referent_proven"], false);
    assert_eq!(
        w["possessive_context"]["referent_resolution"],
        "source_indeterminate"
    );
    assert_eq!(
        w["source_references"][2]["links"].as_array().unwrap().len(),
        2
    );
}
#[test]
fn possessive_shape_scope_reused_target_and_semantic_use_rejected() {
    let v = fixture("vocabulary");
    for mutation in 0..7 {
        let mut s = fixture("source");
        match mutation {
            0 => s["references"][4]["mention"]["end_byte"] = json!(0),
            1 => s["references"][4]["scope_id"] = json!("missing"),
            2 => {
                let l = s["references"][2]["links"][1].clone();
                s["references"][2]["links"].as_array_mut().unwrap().push(l);
            }
            3 => {
                s["identity"] = json!([{"kind":"same","left":"determiner","right":"person","evidence":[s["references"][0]["mention"]]}])
            }
            4 => {
                s["claims"][0]["body"]["value"]["atom"]["arguments"][1]["value"]["id"] =
                    json!("determiner")
            }
            5 => s["references"][4]["name"] = s["references"][4]["mention"].clone(),
            _ => s["references"][2]["links"][0]["target"] = json!("determiner"),
        }
        assert_ne!(
            validate(&s, ST, &v).report["status"],
            "complete",
            "mutation {mutation}"
        );
    }
}
#[test]
fn possessive_requires_feature_and_absence_does_not_charge_new_walk() {
    let mut v = fixture("vocabulary");
    v["request_context_profile"]["features"] = json!(["nominal_complement_de"]);
    let mut s = fixture("source");
    rehash(&mut s, &v);
    assert_ne!(validate(&s, ST, &v).report["status"], "complete");
    context_budget::with_operation(&v, &json!({}), || {
        assert!(context_budget::charge(999_998, 0));
        assert!(nominal_possessive::blocked_nodes(&s).is_empty());
        assert!(context_budget::charge(1, 0));
    })
    .unwrap();
}
#[test]
fn possessive_implicit_normative_guard_and_alias_closures_are_cut() {
    let v = fixture("vocabulary");
    let mut source = fixture("source");
    let mut question = fixture("question");
    let mut alias = source["references"][2].clone();
    alias["id"] = json!("alias");
    alias["links"] = json!([]);
    source["references"].as_array_mut().unwrap().push(alias);
    source["identity"] = json!([{"kind":"same","left":"head","right":"alias","evidence":[source["references"][2]["mention"]]}]);
    source["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":"alias"},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
    let p = json!({"atom":{"predicate":"property","polarity":"positive","arguments":[{"role":"subject","value":{"kind":"ref","id":"regime"}},{"role":"attribute","value":{"kind":"symbol","id":"color","evidence":[source["coverage"][0]["span"]]}},{"role":"value","value":{"kind":"text","value":"socia","evidence":[source["references"][3]["mention"]]}}],"evidence":[source["coverage"][0]["span"]]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]});
    let guard = source["claims"][0]["body"]["value"].clone();
    source["qualifiers"] = json!([{"id":"guard","scope_id":"root","kind":"guard","body":guard,"resolution":"known","evidence":[source["coverage"][0]["span"]]}]);
    let mut scoped = p.clone();
    scoped["normative_scope"] = json!("alias");
    let evidence = source["coverage"][0]["span"].clone();
    source["claims"].as_array_mut().unwrap().extend([json!({"id":"scope_consumer","scope_id":"root","qualifier_ids":[],"evidence":[evidence],"body":{"kind":"proposition","value":scoped}}),json!({"id":"guard_consumer","scope_id":"root","qualifier_ids":["guard"],"evidence":[evidence],"body":{"kind":"proposition","value":p}})]);
    for (kind, node, refs) in [
        ("claim", "scope_consumer", vec!["regime", "alias"]),
        ("claim", "guard_consumer", vec!["regime"]),
    ] {
        source["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":kind,"id":node},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
        for rid in refs {
            source["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":kind,"id":node},"requires":{"kind":"reference","id":rid},"role":"identity"}));
        }
    }
    source["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":"guard_consumer"},"requires":{"kind":"qualifier","id":"guard"},"role":"guard"}));
    let mut target = p;
    target.as_object_mut().unwrap().remove("basis");
    target.as_object_mut().unwrap().remove("normative_scope");
    target["atom"]["arguments"].as_array_mut().unwrap().pop();
    target["atom"]["arguments"][1]["value"]["evidence"] = json!([question["coverage"][0]["span"]]);
    target["atom"]["evidence"] = json!([question["coverage"][0]["span"]]);
    question["requests"][0]["target"] = target;
    question["requests"][0]["mode"] = json!("answer");
    question["requests"][0]["slot"] = json!("value");
    question["requests"][0]["required_answer_extent"] = json!("one_value");
    question["requests"][0]["normative_scope_filter"] = Value::Null;
    question["dependencies"]
        .as_array_mut()
        .unwrap()
        .retain(|d| {
            d["consumer"]["kind"] != "request"
                || d["requires"]["kind"] != "reference"
                || d["requires"]["id"] == "regime"
        });
    let result = pair(&source, &question, &v);
    for channel in ["support", "refutation", "partial"] {
        assert_eq!(result["requests"][0][channel], json!([]), "{result}");
    }
    assert_eq!(result["complete_answer"], false);
    assert_eq!(result["selected_claim_ids"], json!([]));
    assert!(
        !result["requests"][0]["unresolved"]
            .as_array()
            .unwrap()
            .is_empty(),
        "{result}"
    );
}
#[test]
fn possessive_mismatching_actor_never_publishes_partial() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    let mut q: Value = serde_json::from_str(
        &serde_json::to_string(&fixture("question"))
            .unwrap()
            .replace("Nora", "Mira"),
    )
    .unwrap();
    let qt = QT.replace("Nora", "Mira");
    q["text_sha256"] = json!(
        Sha256::digest(qt.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let result = classify_pair_context(
        &validated(&s, ST, &v),
        &validated(&q, &qt, &v),
        &v,
        &json!({"namespace":"possessive-controls","configuration":[],"time_selector":null,"closed_world":false}),
    );
    assert_eq!(result["selected_claim_ids"], json!([]), "{result}");
    assert_eq!(result["complete_answer"], false);
    assert_eq!(result["requests"][0]["partial"], json!([]));
}
#[test]
fn possessive_accumulated_graph_budget_fails_whole_operation() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    let result = context_budget::with_operation(&v, &s, || {
        assert!(context_budget::charge(900_000, 0));
        for _ in 0..256 {
            nominal_possessive::blocked_nodes(&s);
        }
    });
    assert!(result.is_err());
}
#[test]
fn possessive_matching_su_still_never_proves_same_or_global_answer() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    let mut q = s.clone();
    q["kind"] = json!("question");
    let p = &s["claims"][0]["body"]["value"];
    let mut target = p.clone();
    target.as_object_mut().unwrap().remove("basis");
    target.as_object_mut().unwrap().remove("normative_scope");
    q["requests"] = fixture("question")["requests"].clone();
    q["requests"][0]["target"] = target;
    q["requests"][0]["evidence"] = s["claims"][0]["evidence"].clone();
    q["claims"] = json!([]);
    for d in q["dependencies"].as_array_mut().unwrap() {
        if d["consumer"]["kind"] == "claim" {
            d["consumer"]["kind"] = json!("request");
            d["consumer"]["id"] = json!("ask");
        }
    }
    let result = classify_pair_context(
        &validated(&s, ST, &v),
        &validated(&q, ST, &v),
        &v,
        &json!({"namespace":"possessive-controls","configuration":[],"time_selector":null,"closed_world":false}),
    );
    assert_eq!(result["complete_answer"], false, "{result}");
    assert_eq!(result["requests"][0]["support"], json!([]));
    assert_eq!(result["requests"][0]["refutation"], json!([]));
    assert_eq!(
        result["requests"][0]["partial"][0]["bindings"],
        json!([]),
        "{result}"
    );
}
#[test]
fn possessive_independent_claim_keeps_complete_answer() {
    let v = fixture("vocabulary");
    let mut s = fixture("source");
    let mut q = fixture("question");
    let st = format!("{ST} Lumen es azul.");
    let qt = format!("{QT} Color de Lumen.");
    let span = |text: &str, part: &str| {
        let start = text.find(part).unwrap();
        json!({"start_byte":start,"end_byte":start+part.len(),"quote":part})
    };
    let evidence = span(&st, "Lumen es azul.");
    let property = json!({"atom":{"predicate":"property","polarity":"positive","arguments":[{"role":"subject","value":{"kind":"ref","id":"regime"}},{"role":"attribute","value":{"kind":"symbol","id":"color","evidence":[evidence]}},{"role":"value","value":{"kind":"text","value":"azul","evidence":[span(&st,"azul")]}}],"evidence":[evidence]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]});
    s["claims"].as_array_mut().unwrap().push(json!({"id":"independent","scope_id":"root","qualifier_ids":[],"evidence":[evidence],"body":{"kind":"proposition","value":property}}));
    let mut target = property;
    target.as_object_mut().unwrap().remove("basis");
    target.as_object_mut().unwrap().remove("normative_scope");
    target["atom"]["arguments"].as_array_mut().unwrap().pop();
    target["atom"]["evidence"] = json!([span(&qt, "Color de Lumen.")]);
    target["atom"]["arguments"][1]["value"]["evidence"] = json!([span(&qt, "Color")]);
    let mut request = q["requests"][0].clone();
    request["id"] = json!("independent_ask");
    request["target"] = target;
    request["mode"] = json!("answer");
    request["required_answer_extent"] = json!("one_value");
    request["slot"] = json!("value");
    request["normative_scope_filter"] = Value::Null;
    request["evidence"] = json!([span(&qt, "Color de Lumen.")]);
    q["requests"].as_array_mut().unwrap().push(request);
    for (d, kind, node) in [
        (&mut s, "claim", "independent"),
        (&mut q, "request", "independent_ask"),
    ] {
        d["dependencies"].as_array_mut().unwrap().extend([json!({"consumer":{"kind":kind,"id":node},"requires":{"kind":"scope","id":"root"},"role":"binding"}),json!({"consumer":{"kind":kind,"id":node},"requires":{"kind":"reference","id":"regime"},"role":"identity"})]);
    }
    s["text_sha256"] = json!(
        Sha256::digest(st.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    q["text_sha256"] = json!(
        Sha256::digest(qt.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let result = classify_pair_context(
        &validated(&s, &st, &v),
        &validated(&q, &qt, &v),
        &v,
        &json!({"namespace":"possessive-controls","configuration":[],"time_selector":null,"closed_world":false}),
    );
    assert_eq!(result["requests"][1]["coverage"], "complete", "{result}");
    assert!(
        result["selected_claim_ids"]
            .as_array()
            .unwrap()
            .contains(&json!("independent"))
    );
}
#[test]
fn possessive_relevance_rows_cannot_be_spliced_into_an_undeclared_connection() {
    let mut v = fixture("vocabulary");
    v["predicate_rows"].as_array_mut().unwrap().push(json!({"id":"condition_context","roles":{"beneficiary":"ref","distractor":"ref","condition":"term"},"distributive":false}));
    let row = |id: &str, bindings: Value, carry: Value| json!({"id":id,"source_predicate":"condition_suffices","target_predicate":"condition_context","bindings":bindings,"carry_roles":carry,"conclusion":"partial_intermediate_only","max_hops":1,"allow_reverse":false,"allow_transitive_composition":false});
    v["relevance_links"] = json!([
        row(
            "incomplete",
            json!([["source.beneficiary", "target.beneficiary"]]),
            json!(["source.condition"])
        ),
        row(
            "complete_but_wrong_actor",
            json!([
                ["source.beneficiary", "target.distractor"],
                ["source.condition", "target.condition"]
            ]),
            json!(["source.condition"])
        )
    ]);
    let mut source = fixture("source");
    let mut question = fixture("question");
    rehash(&mut source, &v);
    rehash(&mut question, &v);
    question["requests"][0]["target"]["atom"]["predicate"] = json!("condition_context");
    question["requests"][0]["target"]["atom"]["arguments"]
        .as_array_mut()
        .unwrap()
        .push(json!({"role":"distractor","value":{"kind":"ref","id":"regime"}}));
    let result = pair(&source, &question, &v);
    for channel in ["support", "refutation", "partial"] {
        assert_eq!(result["requests"][0][channel], json!([]), "{result}");
    }
    assert_eq!(result["complete_answer"], false);
}
#[test]
fn possessive_determiner_must_be_a_whole_prenominal_token() {
    let v = fixture("vocabulary");
    let mut source = fixture("source");
    let removed = ST.find("su ficha").unwrap() + 2;
    let text = ST.replace("su ficha", "suficha");
    fn shift(value: &mut Value, removed: usize) {
        match value {
            Value::Object(object) => {
                for key in ["start_byte", "end_byte"] {
                    if let Some(offset) = object.get(key).and_then(Value::as_u64) {
                        if offset > removed as u64 {
                            object.insert(key.to_owned(), json!(offset - 1));
                        }
                    }
                }
                for value in object.values_mut() {
                    shift(value, removed);
                }
            }
            Value::Array(values) => {
                for value in values {
                    shift(value, removed);
                }
            }
            Value::String(value) => *value = value.replace("su ficha", "suficha"),
            _ => {}
        }
    }
    shift(&mut source, removed);
    source["text_sha256"] = json!(
        Sha256::digest(text.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let result = validate(&source, &text, &v);
    assert_eq!(result.report["status"], "invalid", "{}", result.report);
    assert!(
        result.report["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "invalid_shape" && issue["path"] == "/reference/head"),
        "{}",
        result.report
    );
    assert!(
        !result.report["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "invalid_quote"),
        "{}",
        result.report
    );
}
