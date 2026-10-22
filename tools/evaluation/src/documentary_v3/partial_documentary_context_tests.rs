use super::*;

fn reference(name: &str) -> Value {
    json!({"id":"item","name":{"quote":name},"mention":{"quote":name},"denotation":"entity","declared_class":null,"semantic_type":null,"anaphora":null,"binding_scope_id":null,"links":[]})
}
fn fixture() -> (Value, Value, Value, Value) {
    let p = json!({"atom":{"predicate":"documentary_availability","polarity":"positive","arguments":[{"role":"subject","value":{"kind":"ref","id":"item"}},{"role":"attribute","value":{"kind":"symbol","id":"salt_quantity"}},{"role":"state","value":{"kind":"text","value":"ilegible"}},{"role":"document_scope","value":{"kind":"ref","id":"copy"}}]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":["preserved"]});
    let c = json!({"id":"claim","scope_id":"document","qualifier_ids":["preserved"],"evidence":[{"quote":"ilegible en la copia conservada"}],"body":{"kind":"proposition","value":p}});
    let q = json!({"id":"preserved","kind":"configuration","resolution":"known","body":{"key":"document_preservation_state","value":{"kind":"text","value":"conservada"}}});
    let source = json!({"domain_namespace":"test","references":[reference("Brisa")],"claims":[c],"qualifiers":[q],"scopes":[],"identity":[],"dependencies":[]});
    let r = json!({"id":"request","scope_id":"document","mode":"answer","slot":"amount","normative_scope_filter":null,"basis_requirement":{"kind":"exact","value":"specification"},"premise_ids":[],"qualifier_ids":[],"target":{"atom":{"predicate":"quantity","polarity":"positive","arguments":[{"role":"subject","value":{"kind":"ref","id":"item"}},{"role":"attribute","value":{"kind":"symbol","id":"salt_quantity"}}]},"judgment":"fact","status":"state","qualifier_ids":[]}});
    let question = json!({"domain_namespace":"test","references":[reference("Brisa")],"requests":[r],"qualifiers":[],"scopes":[],"identity":[],"dependencies":[]});
    (
        source,
        question,
        json!({}),
        json!({"namespace":"test","configuration":[],"time_selector":null,"closed_world":false}),
    )
}
fn execute(
    source: &Value,
    question: &Value,
    vocabulary: &Value,
    scope: &Value,
    measurement: bool,
) -> Evidence {
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
    let r = &question["requests"][0];
    let mut out = Evidence::default();
    if measurement {
        engine.partial_negative_measurement_context(r, &mut out);
    } else {
        engine.partial_documentary_context(&source["claims"][0], r, &mut out);
    }
    out
}
#[test]
fn documentary_context_preserves_whole_state_and_support_without_value() {
    let (source, question, vocabulary, scope) = fixture();
    let out = execute(&source, &question, &vocabulary, &scope, false);
    assert_eq!(out.partial.len(), 1);
    let w = &out.partial[0];
    assert_eq!(
        w["answer_propositions"],
        json!([source["claims"][0]["body"]["value"]])
    );
    assert_eq!(w["source_qualifiers"], source["qualifiers"]);
    assert_eq!(w["answer_terms"], json!([]));
    assert!(rows(w, "missing_constraints").contains(&json!("basis")));
    assert!(out.support.is_empty() && out.refutation.is_empty() && !out.closed);
}
#[test]
fn documentary_context_rejects_other_name_attribute_positive_value_and_applicability_context() {
    for change in ["name", "attribute", "value", "context", "premise", "branch"] {
        let (mut source, mut question, vocabulary, scope) = fixture();
        match change {
            "name" => question["references"][0]["name"]["quote"] = json!("Otra"),
            "attribute" => {
                question["requests"][0]["target"]["atom"]["arguments"][1]["value"]["id"] =
                    json!("mass")
            }
            "value" => {
                source["claims"][0]["body"]["value"]["atom"]["predicate"] = json!("quantity")
            }
            "context" => source["qualifiers"][0]["body"]["key"] = json!("operating_mode"),
            "premise" => question["requests"][0]["premise_ids"] = json!(["unproved"]),
            "branch" => {
                source["scopes"] = json!([{"id":"document","branch_id":"a","parent_id":null}]);
            }
            _ => unreachable!(),
        }
        assert!(
            execute(&source, &question, &vocabulary, &scope, false)
                .partial
                .is_empty(),
            "{change}"
        );
    }
}
#[test]
fn documentary_context_rejects_explicitly_different_preservation_context() {
    let (source, mut question, vocabulary, scope) = fixture();
    let mut q = source["qualifiers"][0].clone();
    q["body"]["value"]["value"] = json!("destruida");
    question["qualifiers"] = json!([q]);
    question["requests"][0]["qualifier_ids"] = json!(["preserved"]);
    assert!(
        execute(&source, &question, &vocabulary, &scope, false)
            .partial
            .is_empty()
    );
}
#[test]
fn negative_measurement_context_keeps_basis_debt_and_rejects_positive_or_wrong_aspect() {
    let (mut source, mut question, mut vocabulary, scope) = fixture();
    source["claims"][0] = json!({"id":"measurement","scope_id":"document","qualifier_ids":[],"evidence":[{"quote":"No se midió la duración"}],"body":{"kind":"event","event_kind":"measurement","event_ref":"event","aspect":"duration","subject":"item","status":"negated_occurrence","basis":"observation","result":null,"times":[],"method":{"state":"unspecified"}}});
    source["qualifiers"] = json!([]);
    question["requests"][0]["basis_requirement"]["value"] = json!("capability");
    question["requests"][0]["target"]["atom"]["arguments"][1]["value"]["id"] = json!("autonomy");
    vocabulary["measurement_context"] = json!({"version":"negative_measurement_context_v1","rows":[{"id":"gap","event_aspect":"duration","target_attribute":"autonomy","source_configuration":[],"target_configuration":[],"context_bindings":[]}]});
    let out = execute(&source, &question, &vocabulary, &scope, true);
    assert_eq!(out.partial.len(), 1);
    assert_eq!(out.partial[0]["source_event"], source["claims"][0]["body"]);
    assert_eq!(out.partial[0]["answer_terms"], json!([]));
    assert!(rows(&out.partial[0], "missing_constraints").contains(&json!("basis")));
    for status in ["occurred", "planned"] {
        let mut wrong = source.clone();
        wrong["claims"][0]["body"]["status"] = json!(status);
        assert!(
            execute(&wrong, &question, &vocabulary, &scope, true)
                .partial
                .is_empty()
        );
    }
    source["claims"][0]["body"]["aspect"] = json!("mass");
    assert!(
        execute(&source, &question, &vocabulary, &scope, true)
            .partial
            .is_empty()
    );
}
