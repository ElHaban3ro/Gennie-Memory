use super::*;
use sha2::{Digest, Sha256};
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn quote(text: &str, part: &str) -> Value {
    let start = text.find(part).unwrap();
    json!({"start_byte":start,"end_byte":start+part.len(),"quote":part})
}
fn fixture() -> (Value, Value, Value, &'static str) {
    let text = "faros Bruma Delta medición martes viernes parejas seis";
    let vocabulary = json!({"version":"event-pattern-test-v1","predicate_rows":[{"id":"property","roles":{"subject":"ref","attribute":"attribute_id","value":"term"},"distributive":true}],"fixed_attribute_rows":["autonomy"],"fixed_action_rows":[],"semantic_type_rows":[],"relevance_links":[]});
    let mut source = json!({"version":"documentary_evidence_v3","text_sha256":digest(text.as_bytes()),"kind":"source","vocabulary_sha256":digest(&serde_json::to_vec(&vocabulary).unwrap()),"domain_namespace":"event-pattern-controls","scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":[],"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":[],"coverage":[{"span":quote(text,text),"state":"represented","owner":null,"issue_id":null}]});
    source["references"] = json!([
      {"id":"owner","scope_id":"root","mention":quote(text,"Bruma"),"name":quote(text,"Bruma"),"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null},
      {"id":"lights","scope_id":"root","mention":quote(text,"faros"),"name":null,"declared_class":quote(text,"faros"),"semantic_type":null,"denotation":"group","links":[{"kind":"associated_with","target":"owner","evidence":[quote(text,"faros Bruma")]}],"binding_scope_id":null,"anaphora":null},
      {"id":"event","scope_id":"root","mention":quote(text,"medición"),"name":null,"declared_class":quote(text,"medición"),"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}]);
    source["claims"] = json!([{"id":"c1","scope_id":"root","qualifier_ids":[],"evidence":[quote(text,text)],"body":{"kind":"event","event_ref":"event","event_kind":"measurement","subject":"lights","aspect":"autonomy","status":"planned","method":{"state":"unspecified"},"times":[{"role":"scheduled_at","value":{"kind":"time","value":"martes","format":"literal","evidence":[quote(text,"martes")]}}],"result":null,"basis":"observation"}}]);
    let mut question = source.clone();
    question["kind"] = json!("question");
    question["claims"] = json!([]);
    question["requests"] = json!([{"id":"ask","scope_id":"root","mode":"event_slot","target":{"event_kind":"measurement","subject":"lights","aspect":"autonomy","status":"planned","event_ref":"event","method_selector":{"kind":"any"},"time_selector":null},"slot":"scheduled_at","class_filter":null,"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[quote(text,text)]}]);
    (source, question, vocabulary, text)
}
fn execute(mut source: Value, mut question: Value, vocabulary: Value, text: &str) -> Value {
    for d in [&mut source, &mut question] {
        let mut deps = rows(d, "dependencies").to_vec();
        for r in rows(d, "references") {
            deps.push(json!({"consumer":{"kind":"reference","id":r["id"]},"requires":{"kind":"scope","id":r["scope_id"]},"role":"binding"}));
            for l in rows(r, "links") {
                deps.push(json!({"consumer":{"kind":"reference","id":r["id"]},"requires":{"kind":"reference","id":l["target"]},"role":"identity"}));
            }
        }
        for (collection, kind) in [("claims", "claim"), ("requests", "request")] {
            for item in rows(d, collection) {
                deps.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"scope","id":item["scope_id"]},"role":"binding"}));
                for r in rows(d, "references") {
                    deps.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"reference","id":r["id"]},"role":"identity"}));
                }
                for q in rows(d, "qualifiers") {
                    deps.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"qualifier","id":q["id"]},"role":"configuration"}));
                }
            }
        }
        d["dependencies"] = json!(deps);
    }
    let s = super::super::super::validate(&source, text, &vocabulary);
    let q = super::super::super::validate(&question, text, &vocabulary);
    let sr = serde_json::to_value(&s).unwrap();
    let qr = serde_json::to_value(&q).unwrap();
    assert_eq!(sr["status"], "complete", "{sr}");
    assert_eq!(qr["status"], "complete", "{qr}");
    super::super::super::classify_pair_context(
        &s,
        &q,
        &vocabulary,
        &json!({"namespace":"event-pattern-controls","configuration":[],"time_selector":null,"closed_world":false}),
    )
}
#[test]
fn event_pattern_preserves_source_event_without_proving_identity_or_requested_time() {
    let (s, q, v, t) = fixture();
    let original = s["claims"][0]["body"].clone();
    let r = execute(s, q, v, t);
    assert_eq!(r["selected_claim_ids"], json!(["c1"]), "{r}");
    assert_eq!(r["complete_answer"], false);
    let w = &r["requests"][0]["partial"][0];
    assert_eq!(w["kind"], "partial_event_context");
    assert_eq!(w["source_event"], original);
    assert_eq!(w["answer_terms"], json!([]));
    assert_eq!(w["bindings"], json!([]));
    assert_eq!(w["event_context"]["event_identity_proven"], false);
}
#[test]
fn event_pattern_rejects_changed_owner_class_status_configuration_and_negative_event() {
    for kind in [
        "owner",
        "class",
        "status",
        "configuration",
        "negation",
        "bare_class",
    ] {
        let (mut s, mut q, v, t) = fixture();
        match kind {
            "owner" => q["references"][0]["name"] = quote(t, "Delta"),
            "class" => q["references"][1]["declared_class"] = quote(t, "parejas"),
            "status" => s["claims"][0]["body"]["status"] = json!("occurred"),
            "negation" => s["claims"][0]["body"]["status"] = json!("negated_occurrence"),
            "bare_class" => {
                s["references"][1]["links"] = json!([]);
                q["references"][1]["links"] = json!([]);
            }
            "configuration" => {
                s["qualifiers"] = json!([{"id":"setting","scope_id":"root","kind":"configuration","body":{"key":"setting","value":{"kind":"text","value":"martes","evidence":[quote(t,"martes")]}},"resolution":"known","evidence":[quote(t,"martes")]}]);
                s["claims"][0]["qualifier_ids"] = json!(["setting"]);
            }
            _ => unreachable!(),
        }
        let r = execute(s, q, v, t);
        assert!(
            rows(&r["requests"][0], "partial")
                .iter()
                .all(|w| w["kind"] != "partial_event_context"),
            "{kind}: {r}"
        );
    }
}
#[test]
fn event_pattern_does_not_fuse_separate_events() {
    let (mut s, q, v, t) = fixture();
    let mut second = s["claims"][0].clone();
    second["id"] = json!("c2");
    second["body"]["event_ref"] = Value::Null;
    second["body"]["times"][0]["value"] =
        json!({"kind":"time","value":"viernes","format":"literal","evidence":[quote(t,"viernes")]});
    s["claims"].as_array_mut().unwrap().push(second);
    let r = execute(s, q, v, t);
    let witnesses = rows(&r["requests"][0], "partial");
    assert_eq!(witnesses.len(), 2, "{r}");
    for w in witnesses {
        assert_eq!(rows(w, "claim_ids").len(), 1);
        assert_eq!(w["answer_terms"], json!([]));
    }
    assert_eq!(r["complete_answer"], false);
}

fn quantified_fixture() -> (Value, Value, Value, &'static str) {
    let (mut source, mut question, vocabulary, text) = fixture();
    for doc in [&mut source, &mut question] {
        doc["scopes"].as_array_mut().unwrap().push(json!({"id":"six","parent_id":"root","kind":"binder","branch_id":null,"binding":{"variable":"couples","quantifier":"exists","domain_ref":"couple-class","cardinality":{"relation":"exact","value":6,"domain":"couple-class","evidence":[quote(text,"seis")]},"witness_relation":"unspecified","relation_closures":[],"evidence":[quote(text,"seis")]}}));
        doc["references"].as_array_mut().unwrap().extend([
            json!({"id":"couple-class","scope_id":"root","mention":quote(text,"parejas"),"name":null,"declared_class":quote(text,"parejas"),"semantic_type":null,"denotation":"class","links":[],"binding_scope_id":null,"anaphora":null}),
            json!({"id":"couples","scope_id":"six","mention":quote(text,"parejas"),"name":null,"declared_class":quote(text,"parejas"),"semantic_type":null,"denotation":"group","links":[{"kind":"associated_with","target":"owner","evidence":[quote(text,text)]}],"binding_scope_id":"six","anaphora":null}),
        ]);
        doc["references"][1]["links"][0]["target"] = json!("couples");
        doc["references"][1]["scope_id"] = json!("six");
        for key in ["claims", "requests"] {
            for row in doc[key].as_array_mut().unwrap() {
                row["scope_id"] = json!("six");
            }
        }
    }
    (source, question, vocabulary, text)
}
#[test]
fn event_pattern_quantified_context_keeps_cardinality_and_quantifier_restrictive() {
    let (s, q, v, t) = quantified_fixture();
    let r = execute(s, q, v, t);
    assert_eq!(r["selected_claim_ids"], json!(["c1"]), "{r}");
    assert_eq!(r["complete_answer"], false);
    assert_eq!(
        r["requests"][0]["partial"][0]["event_context"]["alpha_pattern"]["couples"],
        "couples"
    );
    for change in ["cardinality", "quantifier", "owner"] {
        let (s, mut q, v, t) = quantified_fixture();
        match change {
            "cardinality" => q["scopes"][1]["binding"]["cardinality"]["value"] = json!(7),
            "quantifier" => q["scopes"][1]["binding"]["quantifier"] = json!("forall"),
            "owner" => q["references"][0]["name"] = quote(t, "Delta"),
            _ => unreachable!(),
        }
        let r = execute(s, q, v, t);
        assert!(
            rows(&r["requests"][0], "partial")
                .iter()
                .all(|w| w["kind"] != "partial_event_context"),
            "{change}: {r}"
        );
    }
}

#[test]
fn event_pattern_does_not_override_named_event_targets() {
    for side in ["source", "question", "both"] {
        let (mut source, mut question, vocabulary, text) = fixture();
        if side != "question" {
            source["references"][2]["name"] = quote(text, "martes");
        }
        if side != "source" {
            question["references"][2]["name"] = quote(text, "viernes");
        }
        let result = execute(source, question, vocabulary, text);
        assert!(
            rows(&result["requests"][0], "partial")
                .iter()
                .all(|w| w["kind"] != "partial_event_context"),
            "{side}: {result}"
        );
    }
}

#[test]
fn event_pattern_planned_event_is_not_recovered_for_unspecified_truth_status() {
    let (source, mut question, vocabulary, text) = fixture();
    question["requests"][0]["slot"] = json!("status");
    question["requests"][0]["required_answer_extent"] = json!("truth_status");
    question["requests"][0]["target"]["status"] = json!("any");
    let result = execute(source, question, vocabulary, text);
    assert!(
        rows(&result["requests"][0], "partial")
            .iter()
            .all(|w| w["kind"] != "partial_event_context"),
        "{result}"
    );
}
#[test]
fn event_pattern_quantified_variable_links_cannot_be_hidden_by_alpha_mapping() {
    for kind in ["owned_by", "part_of"] {
        let (mut source, mut question, vocabulary, text) = quantified_fixture();
        // Retain the common association to Bruma, while adding a different
        // restrictive ownership/part relation to the bound group itself.
        for (doc, name) in [(&mut source, "martes"), (&mut question, "viernes")] {
            doc["references"].as_array_mut().unwrap().push(json!({"id":"other","scope_id":"root","mention":quote(text,name),"name":quote(text,name),"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}));
            doc["references"][4]["links"]
                .as_array_mut()
                .unwrap()
                .push(json!({"kind":kind,"target":"other","evidence":[quote(text,text)]}));
        }
        let result = execute(source, question, vocabulary, text);
        assert!(
            rows(&result["requests"][0], "partial")
                .iter()
                .all(|w| w["kind"] != "partial_event_context"),
            "{kind}: {result}"
        );
    }
}

#[test]
fn event_pattern_quantified_replacement_relation_still_requires_same_owner() {
    for kind in ["owned_by", "part_of"] {
        let (mut source, mut question, vocabulary, text) = quantified_fixture();
        source["references"][4]["links"][0]["kind"] = json!(kind);
        question["references"][4]["links"][0]["kind"] = json!(kind);
        question["references"][0]["name"] = quote(text, "Delta");
        question["references"][0]["mention"] = quote(text, "Delta");
        let result = execute(source, question, vocabulary, text);
        assert!(
            rows(&result["requests"][0], "partial")
                .iter()
                .all(|w| w["kind"] != "partial_event_context"),
            "{kind}: {result}"
        );
    }
}

#[test]
fn event_pattern_binder_domain_names_are_not_discarded() {
    let (mut source, mut question, vocabulary, text) = quantified_fixture();
    source["references"][3]["name"] = quote(text, "Bruma");
    question["references"][3]["name"] = quote(text, "Delta");
    let result = execute(source, question, vocabulary, text);
    assert!(
        rows(&result["requests"][0], "partial")
            .iter()
            .all(|w| w["kind"] != "partial_event_context"),
        "{result}"
    );
}

fn alternate_cardinality_carrier_fixture() -> (Value, Value, Value, &'static str) {
    let (source, mut question, vocabulary, text) = quantified_fixture();
    question["scopes"][1]["binding"]["domain_ref"] = Value::Null;
    question["scopes"][1]["binding"]["cardinality"]["domain"] = json!("couples");
    (source, question, vocabulary, text)
}
#[test]
fn event_pattern_normalizes_optional_plain_counting_class_as_context_only() {
    let (source, question, vocabulary, text) = alternate_cardinality_carrier_fixture();
    let original = source["claims"][0]["body"].clone();
    let result = execute(source, question, vocabulary, text);
    assert_eq!(result["selected_claim_ids"], json!(["c1"]), "{result}");
    assert_eq!(result["complete_answer"], false);
    let witness = &result["requests"][0]["partial"][0];
    assert_eq!(witness["source_event"], original);
    assert_eq!(
        witness["event_context"]["cardinality_carrier_normalized"],
        true
    );
    assert_eq!(
        witness["event_context"]["cardinality_carrier_identity_proven"],
        false
    );
    assert_eq!(witness["answer_terms"], json!([]));
    assert!(rows(witness, "missing_constraints").contains(&json!("cardinality_carrier_identity")));
}
#[test]
fn event_pattern_carrier_normalization_keeps_explicit_domain_restrictions() {
    for change in [
        "class",
        "named",
        "linked",
        "qualifier",
        "count",
        "quantifier",
    ] {
        let (mut source, mut question, vocabulary, text) = alternate_cardinality_carrier_fixture();
        match change {
            "class" => source["references"][3]["declared_class"] = quote(text, "faros"),
            "named" => source["references"][3]["name"] = quote(text, "Bruma"),
            "linked" => {
                source["references"][3]["links"] = json!([{"kind":"associated_with","target":"owner","evidence":[quote(text,text)]}])
            }
            "count" => question["scopes"][1]["binding"]["cardinality"]["value"] = json!(7),
            "quantifier" => source["scopes"][1]["binding"]["quantifier"] = json!("forall"),
            "qualifier" => {
                source["qualifiers"] = json!([{"id":"domain-setting","scope_id":"root","kind":"configuration","body":{"key":"setting","value":{"kind":"text","value":"martes","evidence":[quote(text,"martes")]}},"resolution":"known","evidence":[quote(text,"martes")]}]);
                source["dependencies"] = json!([{"consumer":{"kind":"reference","id":"couple-class"},"requires":{"kind":"qualifier","id":"domain-setting"},"role":"configuration"}]);
            }
            _ => unreachable!(),
        }
        let result = execute(source, question, vocabulary, text);
        assert!(
            rows(&result["requests"][0], "partial")
                .iter()
                .all(|w| w["kind"] != "partial_event_context"),
            "{change}: {result}"
        );
    }
}
