use super::*;
fn named(id: &str, name: &str) -> Value {
    json!({"id":id,"name":{"quote":name},"mention":{"quote":name},"denotation":"entity","declared_class":null,"semantic_type":null,"anaphora":null,"binding_scope_id":null,"links":[]})
}
fn fixture() -> (Value, Value, Value, Value) {
    let c = json!({"id":"use","scope_id":"document","qualifier_ids":["resolution"],"evidence":[{"quote":"Cima usa Fino a 600 ppp"}],"body":{"kind":"proposition","value":{"atom":{"predicate":"uses","polarity":"positive","arguments":[{"role":"subject","value":{"kind":"ref","id":"printer"}},{"role":"object","value":{"kind":"ref","id":"setting"}}]},"basis":"assignment","status":"state","judgment":"fact","normative_scope":null,"qualifier_ids":["resolution"]}}});
    let source = json!({"domain_namespace":"test","references":[named("printer","Cima"),named("setting","Fino")],"claims":[c],"qualifiers":[{"id":"resolution","kind":"configuration","resolution":"known","body":{"key":"resolution","value":{"kind":"quantity","decimal":"600","unit":"ppp","bound":"exact"}}}],"dependencies":[],"identity":[],"scopes":[]});
    let r = json!({"id":"request","scope_id":"document","mode":"answer","slot":"value","basis_requirement":{"kind":"exact","value":"assignment"},"normative_scope_filter":null,"unit":null,"class_filter":null,"premise_ids":[],"qualifier_ids":["mode"],"target":{"atom":{"predicate":"property","polarity":"positive","arguments":[{"role":"subject","value":{"kind":"ref","id":"printer"}},{"role":"attribute","value":{"kind":"symbol","id":"resolution"}}]},"status":"state","judgment":"fact","qualifier_ids":["mode"]}});
    let question = json!({"domain_namespace":"test","references":[named("printer","Cima"),named("setting","Fino")],"claims":[],"requests":[r],"qualifiers":[{"id":"mode","kind":"configuration","resolution":"known","body":{"key":"mode","value":{"kind":"ref","id":"setting"}}}],"dependencies":[],"identity":[],"scopes":[]});
    (
        source,
        question,
        json!({"assignment_projection":{"version":"uses_qualifier_assignment_v1","context_keys":["mode","profile","program"]}}),
        json!({"namespace":"test","configuration":[],"time_selector":null,"closed_world":false}),
    )
}
fn execute(source: &Value, question: &Value, vocabulary: &Value, scope: &Value) -> Evidence {
    let report = json!({"candidates":[]});
    let engine = Engine {
        possessive_source: &BTreeSet::new(),
        lexical_source: &BTreeSet::new(),
        possessive_question: &BTreeSet::new(),
        source,
        question,
        vocabulary,
        scope,
        source_report: &report,
        question_report: &report,
        pending_claims: &[],
        context_v2: true,
    };
    let mut result = Evidence::default();
    engine.partial_assignment_context(&source["claims"][0], &question["requests"][0], &mut result);
    result
}
#[test]
fn assignment_property_context_preserves_source_and_unproved_premise_without_numeric_answer() {
    let (source, mut question, vocabulary, scope) = fixture();
    question["requests"][0]["premise_ids"] = json!(["operation"]);
    let result = execute(&source, &question, &vocabulary, &scope);
    assert_eq!(result.partial.len(), 1);
    let w = &result.partial[0];
    assert_eq!(
        w["source_proposition"],
        source["claims"][0]["body"]["value"]
    );
    assert_eq!(w["source_qualifiers"], source["qualifiers"]);
    assert_eq!(w["answer_terms"], json!([]));
    assert_eq!(w["answer_propositions"], json!([]));
    assert!(rows(w, "missing_constraints").contains(&json!("premise")));
    assert!(result.support.is_empty() && result.refutation.is_empty() && !result.closed);
}
#[test]
fn assignment_property_context_rejects_other_name_setting_attribute_unknown_value_and_conditions() {
    for variant in [
        "name",
        "setting",
        "attribute",
        "absent_quantity",
        "unknown_value",
        "conditional",
        "binder",
        "dependency",
    ] {
        let (mut source, mut question, vocabulary, scope) = fixture();
        match variant {
            "name" => question["references"][0]["name"]["quote"] = json!("Loma"),
            "setting" => question["references"][1]["name"]["quote"] = json!("Borrador"),
            "attribute" => {
                question["requests"][0]["target"]["atom"]["arguments"][1]["value"]["id"] =
                    json!("frequency")
            }
            "absent_quantity" => {
                source["qualifiers"][0]["body"]["value"] = json!({"kind":"text","value":"600 ppp"})
            }
            "unknown_value" => source["qualifiers"][0]["resolution"] = json!("unknown"),
            "conditional" => {
                source["scopes"] =
                    json!([{"id":"document","branch_id":"alternative","parent_id":null}])
            }
            "binder" => {
                source["scopes"] = json!([{"id":"document","binding":{"kind":"universal","variable_id":"x"},"parent_id":null}])
            }
            "dependency" => {
                source["dependencies"] = json!([{"consumer":{"kind":"claim","id":"use"},"requires":{"kind":"qualifier","id":"missing"},"role":"configuration"}])
            }
            _ => unreachable!(),
        };
        assert!(
            execute(&source, &question, &vocabulary, &scope)
                .partial
                .is_empty(),
            "{variant}"
        );
    }
}
