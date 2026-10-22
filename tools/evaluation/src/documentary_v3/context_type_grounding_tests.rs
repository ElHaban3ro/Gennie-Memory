use super::*;
use sha2::{Digest, Sha256};
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn evidence(text: &str, quote: &str) -> Value {
    let start = text.find(quote).unwrap();
    json!({"start_byte":start,"end_byte":start+quote.len(),"quote":quote})
}
fn fixture() -> (Value, Value, Value, &'static str, &'static str) {
    let st = "Laria recibe suministro de la cuenca azul Z. Otra clase es cuenca roja.";
    let qt = "¿Qué cuenca azul alimenta Laria? Otra clase es cuenca roja.";
    let v = json!({"version":"local-type-context-test-v1","predicate_rows":[{"id":"supplied_by","roles":{"destination":"ref","supplier":"ref"},"distributive":true},{"id":"receives_from","roles":{"destination":"ref","source":"ref"},"distributive":true}],"fixed_attribute_rows":[],"fixed_action_rows":[],"semantic_type_rows":[{"id":"blue_type","class_quotes":["cuenca azul"]},{"id":"red_type","class_quotes":["cuenca roja"]}],"relevance_links":[{"id":"typed_source_context","source_predicate":"supplied_by","target_predicate":"receives_from","bindings":[["source.destination","target.destination"]],"carry_roles":["source.supplier"],"conclusion":"partial_intermediate_only","max_hops":1,"allow_reverse":false,"allow_transitive_composition":false}],"relevance_profiles":{"version":"documentary_relevance_profiles_v1","rows":[{"link_id":"typed_source_context","alternatives":[{"kind":"answer_filter","source_role":"supplier"}]}]}});
    let doc = |text: &str, kind: &str| json!({"version":"documentary_evidence_v3","text_sha256":digest(text.as_bytes()),"kind":kind,"vocabulary_sha256":digest(&serde_json::to_vec(&v).unwrap()),"domain_namespace":"type-context-controls","scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":[{"id":"recipient","scope_id":"root","mention":evidence(text,"Laria"),"name":evidence(text,"Laria"),"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}],"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":[],"coverage":[{"span":evidence(text,text),"state":"represented","owner":null,"issue_id":null}]});
    let mut source = doc(st, "source");
    let mut question = doc(qt, "question");
    source["references"].as_array_mut().unwrap().push(json!({"id":"supplier","scope_id":"root","mention":evidence(st,"cuenca azul Z"),"name":evidence(st,"Z"),"declared_class":evidence(st,"cuenca azul"),"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}));
    source["claims"] = json!([{"id":"c1","scope_id":"root","qualifier_ids":[],"evidence":[evidence(st,st)],"body":{"kind":"proposition","value":{"atom":{"predicate":"supplied_by","arguments":[{"role":"destination","value":{"kind":"ref","id":"recipient"}},{"role":"supplier","value":{"kind":"ref","id":"supplier"}}],"polarity":"positive","evidence":[evidence(st,st)]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]}}}]);
    question["requests"] = json!([{"id":"ask","scope_id":"root","mode":"answer","target":{"atom":{"predicate":"receives_from","arguments":[{"role":"destination","value":{"kind":"ref","id":"recipient"}}],"polarity":"positive","evidence":[evidence(qt,qt)]},"judgment":"fact","status":"state","qualifier_ids":[]},"slot":"source","class_filter":{"kind":"semantic","type_id":"blue_type","evidence":[evidence(qt,"cuenca azul")]},"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[evidence(qt,qt)]}]);
    (source, question, v, st, qt)
}
fn validate(mut raw: Value, text: &str, v: &Value) -> super::super::super::DocumentResult {
    raw["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(v).unwrap()));
    let mut deps = vec![];
    for reference in rows(&raw, "references") {
        deps.push(json!({"consumer":{"kind":"reference","id":reference["id"]},"requires":{"kind":"scope","id":reference["scope_id"]},"role":"binding"}));
    }
    for (collection, kind) in [("claims", "claim"), ("requests", "request")] {
        for item in rows(&raw, collection) {
            deps.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"scope","id":item["scope_id"]},"role":"binding"}));
            for reference in rows(&raw, "references") {
                deps.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"reference","id":reference["id"]},"role":"identity"}));
            }
            for target in [
                &item["body"]["value"]["normative_scope"],
                &item["normative_scope_filter"],
            ] {
                if target.is_string() {
                    deps.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"reference","id":target},"role":"normative_target"}));
                }
            }
        }
    }
    raw["dependencies"] = json!(deps);
    let result = super::super::super::validate(&raw, text, v);
    let report = serde_json::to_value(&result).unwrap();
    assert_eq!(report["status"], "complete", "{report}");
    result
}
fn scope() -> Value {
    json!({"namespace":"type-context-controls","configuration":[],"time_selector":null,"closed_world":false})
}
#[test]
fn context_type_grounding_retains_original_null_type_and_vocabulary_evidence_only_as_partial() {
    let (s, q, v, st, qt) = fixture();
    let source = validate(s, st, &v);
    let question = validate(q, qt, &v);
    let before = serde_json::to_value(&source).unwrap();
    let old = super::super::super::classify_pair(&source, &question, &v, &scope());
    let new = super::super::super::classify_pair_context(&source, &question, &v, &scope());
    assert_eq!(old["selected_claim_ids"], json!([]), "{old}");
    assert_eq!(new["selected_claim_ids"], json!(["c1"]), "{new}");
    assert_eq!(new["complete_answer"], false);
    let witness = &new["requests"][0]["partial"][0];
    assert_eq!(witness["answer_terms"], json!([]));
    assert_eq!(
        witness["answer_propositions"][0],
        before["document"]["claims"][0]["body"]["value"]
    );
    let proof = &witness["relevance_profile"]["context_type_grounding"][0];
    assert_eq!(proof["vocabulary_row"]["id"], "blue_type");
    assert_eq!(
        proof["declared_class_evidence"],
        before["document"]["references"][1]["declared_class"]
    );
    assert_eq!(proof["identity_proven"], false);
    assert_eq!(before, serde_json::to_value(&source).unwrap());
    assert!(before["document"]["references"][1]["semantic_type"].is_null());
}
#[test]
fn context_type_grounding_rejects_wrong_type_ambiguous_quote_no_mapping_and_explicit_incompatible()
{
    for change in ["wrong_type", "ambiguous", "absent", "explicit_incompatible"] {
        let (mut s, mut q, mut v, st, qt) = fixture();
        match change {
            "wrong_type" => {
                q["requests"][0]["class_filter"]["type_id"] = json!("red_type");
                q["requests"][0]["class_filter"]["evidence"] = json!([evidence(qt, "cuenca roja")]);
            }
            "ambiguous" => v["semantic_type_rows"][1]["class_quotes"]
                .as_array_mut()
                .unwrap()
                .push(json!("cuenca azul")),
            "absent" => {
                v["semantic_type_rows"][0]["class_quotes"] = json!(["cuenca roja"]);
                q["requests"][0]["class_filter"]["evidence"] = json!([evidence(qt, "cuenca roja")]);
            }
            "explicit_incompatible" => {
                s["references"][1]["semantic_type"] =
                    json!({"type_id":"red_type","evidence":[evidence(st,"cuenca roja")]})
            }
            _ => unreachable!(),
        }
        let source = validate(s, st, &v);
        let question = validate(q, qt, &v);
        let result = super::super::super::classify_pair_context(&source, &question, &v, &scope());
        assert_eq!(
            result["selected_claim_ids"],
            json!([]),
            "{change}: {result}"
        );
        assert_eq!(result["complete_answer"], false);
    }
}
#[test]
fn context_type_grounding_does_not_change_global_class_match_or_inactive_mode() {
    let (s, q, v, st, qt) = fixture();
    let source = validate(s, st, &v);
    let question = validate(q, qt, &v);
    let sr = serde_json::to_value(&source).unwrap();
    let qr = serde_json::to_value(&question).unwrap();
    let scope = scope();
    for active in [false, true] {
        let engine = Engine {
            possessive_source: &std::collections::BTreeSet::new(),
            lexical_source: &std::collections::BTreeSet::new(),
            possessive_question: &std::collections::BTreeSet::new(),
            source: &sr["document"],
            question: &qr["document"],
            source_report: &sr,
            question_report: &qr,
            pending_claims: &[],
            vocabulary: &v,
            scope: &scope,
            context_v2: active,
        };
        let term = json!({"kind":"ref","id":"supplier"});
        let filter = &qr["document"]["requests"][0]["class_filter"];
        assert!(!engine.class_match(&term, filter));
        assert_eq!(
            engine.context_type_grounding(&term, filter).is_some(),
            active
        );
    }
}

fn add_scope_guard(doc: &mut Value, collection: &str, binder: bool, text: &str) {
    let scope = if binder { "bounded" } else { "branch" };
    let binding = if binder {
        json!({"variable":"bound","quantifier":"exists","domain_ref":null,"cardinality":null,"witness_relation":"unspecified","relation_closures":[],"evidence":[evidence(text,"cuenca azul")]})
    } else {
        Value::Null
    };
    doc["scopes"].as_array_mut().unwrap().push(json!({"id":scope,"parent_id":"root","kind":if binder{"binder"}else{"branch"},"branch_id":if binder{Value::Null}else{json!("b")},"binding":binding}));
    doc[collection][0]["scope_id"] = json!(scope);
    if binder {
        doc["references"].as_array_mut().unwrap().push(json!({"id":"bound","scope_id":scope,"mention":evidence(text,"cuenca azul"),"name":null,"declared_class":evidence(text,"cuenca azul"),"semantic_type":null,"denotation":"group","links":[],"binding_scope_id":scope,"anaphora":null}));
    }
}
fn add_norm_reference(doc: &mut Value, text: &str) {
    doc["references"].as_array_mut().unwrap().push(json!({"id":"norm","scope_id":"root","mention":evidence(text,"roja"),"name":evidence(text,"roja"),"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}));
}
#[test]
fn context_type_grounding_never_recovers_plans_negation_normative_or_bound_context() {
    for change in [
        "planned",
        "occurred",
        "negative",
        "allowed",
        "required",
        "query_planned",
        "query_negative",
        "query_allowed",
        "source_binder",
        "query_binder",
        "source_branch",
        "query_branch",
        "normative_basis",
        "capability_basis",
    ] {
        let (mut source, mut question, v, st, qt) = fixture();
        match change {
            "planned" => source["claims"][0]["body"]["value"]["status"] = json!("planned"),
            "occurred" => source["claims"][0]["body"]["value"]["status"] = json!("occurred"),
            "negative" => {
                source["claims"][0]["body"]["value"]["atom"]["polarity"] = json!("negative")
            }
            "allowed" | "required" => {
                add_norm_reference(&mut source, st);
                source["claims"][0]["body"]["value"]["judgment"] = json!(change);
                source["claims"][0]["body"]["value"]["normative_scope"] = json!("norm");
                source["claims"][0]["body"]["value"]["basis"] = json!("normative");
            }
            "query_planned" => question["requests"][0]["target"]["status"] = json!("planned"),
            "query_negative" => {
                question["requests"][0]["target"]["atom"]["polarity"] = json!("negative")
            }
            "query_allowed" => {
                add_norm_reference(&mut question, qt);
                question["requests"][0]["target"]["judgment"] = json!("allowed");
                question["requests"][0]["normative_scope_filter"] = json!("norm");
            }
            "source_binder" => add_scope_guard(&mut source, "claims", true, st),
            "query_binder" => add_scope_guard(&mut question, "requests", true, qt),
            "source_branch" => add_scope_guard(&mut source, "claims", false, st),
            "query_branch" => add_scope_guard(&mut question, "requests", false, qt),
            "normative_basis" => source["claims"][0]["body"]["value"]["basis"] = json!("normative"),
            "capability_basis" => {
                source["claims"][0]["body"]["value"]["basis"] = json!("capability")
            }
            _ => unreachable!(),
        }
        let source = validate(source, st, &v);
        let question = validate(question, qt, &v);
        let before = serde_json::to_value(&source).unwrap();
        let result = super::super::super::classify_pair_context(&source, &question, &v, &scope());
        assert_eq!(
            result["selected_claim_ids"],
            json!([]),
            "{change}: {result}"
        );
        assert_eq!(result["complete_answer"], false, "{change}: {result}");
        assert_eq!(before, serde_json::to_value(&source).unwrap());
    }
}
#[test]
fn context_type_grounding_does_not_choose_null_typed_alias_over_explicit_other_type() {
    let (mut source, question, v, st, qt) = fixture();
    source["references"].as_array_mut().unwrap().push(json!({"id":"alias","scope_id":"root","mention":evidence(st,"cuenca roja"),"name":null,"declared_class":evidence(st,"cuenca roja"),"semantic_type":{"type_id":"red_type","evidence":[evidence(st,"cuenca roja")]},"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}));
    source["identity"] =
        json!([{"kind":"same","left":"supplier","right":"alias","evidence":[evidence(st,st)]}]);
    let source = validate(source, st, &v);
    let question = validate(question, qt, &v);
    let result = super::super::super::classify_pair_context(&source, &question, &v, &scope());
    assert_eq!(result["selected_claim_ids"], json!([]), "{result}");
    assert_eq!(result["complete_answer"], false);
}
