use super::*;
use serde_json::json;
use sha2::{Digest, Sha256};
const ST: &str =
    include_str!("../../../../tests/fixtures/documentary-normative-indeterminacy-v1/source.txt");
const QT: &str =
    include_str!("../../../../tests/fixtures/documentary-normative-indeterminacy-v1/question.txt");
fn fixture(name: &str) -> Value {
    serde_json::from_str(match name {
        "source" => include_str!(
            "../../../../tests/fixtures/documentary-normative-indeterminacy-v1/source.json"
        ),
        "question" => include_str!(
            "../../../../tests/fixtures/documentary-normative-indeterminacy-v1/question.json"
        ),
        _ => include_str!(
            "../../../../tests/fixtures/documentary-normative-indeterminacy-v1/vocabulary.json"
        ),
    })
    .unwrap()
}
fn hash(v: &Value) -> String {
    Sha256::digest(serde_json::to_vec(v).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn rehash(d: &mut Value, v: &Value) {
    d["vocabulary_sha256"] = json!(hash(v));
}
fn scope() -> Value {
    json!({"namespace":"normative-indeterminacy-controls","configuration":[],"time_selector":null,"closed_world":false})
}
fn complete(raw: &Value, text: &str, v: &Value) -> DocumentResult {
    let r = validate(raw, text, v);
    assert_eq!(r.report["status"], "complete", "{}", r.report);
    r
}
fn blocked(result: &Value, q: &Value) {
    assert_eq!(result["complete_answer"], false, "{result}");
    assert_eq!(result["selected_claim_ids"], json!([]), "{result}");
    let r = &result["requests"][0];
    for k in ["support", "refutation", "partial", "conflicts"] {
        assert_eq!(r[k], json!([]), "{result}");
    }
    let pending = &r["unresolved"][0];
    assert_eq!(pending["question_document"], *q);
    assert_eq!(pending["request_constraint"], q["requests"][0]);
    assert_eq!(pending["resolution_eligible"], false);
    for k in ["answer_terms", "answer_propositions", "bindings"] {
        assert_eq!(pending[k], json!([]));
    }
}
#[test]
fn normative_indeterminacy_representation_complete_resolution_blocked_full_payload() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    let q = fixture("question");
    let sr = complete(&s, ST, &v);
    let qr = complete(&q, QT, &v);
    assert_eq!(qr.report["document"], q);
    assert_eq!(qr.report["resolution_disposition"]["state"], "blocked");
    assert_eq!(
        qr.report["resolution_disposition"]["debts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(qr.report["issues"].as_array().unwrap().len(), 2);
    assert_eq!(qr.report["candidates"][0]["state"], "unresolved");
    blocked(&classify_pair_context(&sr, &qr, &v, &scope()), &q);
}
#[test]
fn normative_indeterminacy_original_issues_and_paths_preserved_feature_absent() {
    let v = fixture("vocabulary");
    let q = fixture("question");
    let current = complete(&q, QT, &v);
    let mut oldv = v.clone();
    oldv["request_context_profile"]["features"]
        .as_array_mut()
        .unwrap()
        .retain(|f| f != normative_indeterminacy::FEATURE);
    let mut oldq = q.clone();
    rehash(&mut oldq, &oldv);
    let old = validate(&oldq, QT, &oldv);
    assert_eq!(old.report["status"], "partial");
    assert_eq!(current.report["issues"], old.report["issues"]);
    assert_eq!(current.report["candidates"], old.report["candidates"]);
    assert!(old.report.get("representation_disposition").is_none());
    assert!(old.report.get("resolution_disposition").is_none());
    context_budget::with_operation(&oldv, &json!({}), || {
        assert!(context_budget::charge(999_999, 0));
        assert!(
            normative_indeterminacy::pending(&oldq, &oldq["requests"][0], &old.report, &oldv)
                .is_none()
        );
    })
    .unwrap();
}
#[test]
fn normative_indeterminacy_extraction_relation_and_coverage_debts_never_promote() {
    let v = fixture("vocabulary");
    for mutation in 0..5 {
        let mut q = fixture("question");
        match mutation {
            0 => {
                q["requests"][0]["context_bindings"][0]["action"]["actor"]["reason"] =
                    json!("extraction_unresolved")
            }
            1 => {
                q["requests"][0]["context_bindings"][0]["relation"] = json!({"kind":"unresolved","alternatives":["purpose","necessary_condition"],"evidence":q["requests"][0]["context_bindings"][0]["relation"]["evidence"]})
            }
            2 => q["coverage"] = json!([]),
            3 => q["coverage"][0]["state"] = json!("unresolved"),
            _ => q["requests"][0]["target"]["judgment"] = json!("fact"),
        };
        let r = validate(&q, QT, &v);
        assert_ne!(
            r.report["status"], "complete",
            "mutation {mutation}: {}",
            r.report
        );
        assert_eq!(r.report["document"], q);
    }
}
#[test]
fn normative_indeterminacy_wrong_basis_mode_and_nested_shape_do_not_share_exemption() {
    let v = fixture("vocabulary");
    for mutation in 0..6 {
        let mut q = fixture("question");
        match mutation {
            0 => q["requests"][0]["basis_requirement"] = json!({"kind":"any"}),
            1 => q["requests"][0]["target"]["status"] = json!("planned"),
            2 => q["requests"][0]["target"]["atom"]["polarity"] = json!("negative"),
            3 => q["requests"][0]["required_answer_extent"] = json!("all_values"),
            4 => q["requests"][0]["context_bindings"][0]["action"]["evidence"] = json!([]),
            _ => {
                q["requests"][0]["context_bindings"][0]["action"]["destination"]["id"] =
                    json!("missing")
            }
        };
        let r = validate(&q, QT, &v);
        assert_ne!(
            r.report["status"], "complete",
            "mutation {mutation}: {}",
            r.report
        );
    }
}
#[test]
fn normative_indeterminacy_explicit_actor_does_not_fill_null_scope_or_wildcard() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    let sr = complete(&s, ST, &v);
    for actor in [
        json!({"kind":"reference","value":{"kind":"ref","id":"person"},"evidence":[fixture("question")["references"][0]["mention"]]}),
        json!({"kind":"owner_slot","role":"actor","evidence":fixture("question")["requests"][0]["context_bindings"][0]["action"]["actor"]["evidence"]}),
    ] {
        let mut q = fixture("question");
        q["requests"][0]["context_bindings"][0]["action"]["actor"] = actor;
        let qr = complete(&q, QT, &v);
        assert_eq!(
            qr.report["resolution_disposition"]["debts"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        blocked(&classify_pair_context(&sr, &qr, &v, &scope()), &q);
    }
}
#[test]
fn normative_indeterminacy_scope_same_or_different_cannot_resolve_ambiguous_actor() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    let sr = complete(&s, ST, &v);
    for sid in ["course", "person"] {
        let mut q = fixture("question");
        q["requests"][0]["normative_scope_filter"] = json!(sid);
        q["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"request","id":"requested_tutor"},"requires":{"kind":"reference","id":sid},"role":"normative_target"}));
        let qr = complete(&q, QT, &v);
        assert_eq!(
            qr.report["resolution_disposition"]["debts"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        blocked(&classify_pair_context(&sr, &qr, &v, &scope()), &q);
    }
}
#[test]
fn normative_indeterminacy_cut_derived_from_content_when_internal_report_issues_missing() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    let q = fixture("question");
    let sr = complete(&s, ST, &v);
    let mut qr = complete(&q, QT, &v);
    qr.report["issues"] = json!([]);
    qr.report["candidates"] = json!([]);
    qr.report["status"] = json!("partial");
    blocked(&classify_pair_context(&sr, &qr, &v, &scope()), &q);
}
#[test]
fn normative_indeterminacy_shared_budget_fails_entire_operation_before_materialization() {
    let v = fixture("vocabulary");
    let q = fixture("question");
    let qr = complete(&q, QT, &v);
    let result = context_budget::with_operation(&v, &json!({}), || {
        assert!(context_budget::charge(999_990, 0));
        normative_indeterminacy::pending(&q, &q["requests"][0], &qr.report, &v)
    });
    assert!(result.is_err());
}
#[test]
fn normative_indeterminacy_resolved_request_has_identical_matching_without_feature() {
    let v = fixture("vocabulary");
    let mut s = fixture("source");
    let mut q = fixture("question");
    q["requests"][0]["normative_scope_filter"] = json!("course");
    q["requests"][0]["context_bindings"][0]["action"]["actor"] = json!({"kind":"reference","value":{"kind":"ref","id":"person"},"evidence":[q["references"][0]["mention"]]});
    let new = classify_pair_context(&complete(&s, ST, &v), &complete(&q, QT, &v), &v, &scope());
    let mut oldv = v;
    oldv["request_context_profile"]["features"]
        .as_array_mut()
        .unwrap()
        .retain(|f| f != normative_indeterminacy::FEATURE);
    rehash(&mut s, &oldv);
    rehash(&mut q, &oldv);
    let old = classify_pair_context(
        &complete(&s, ST, &oldv),
        &complete(&q, QT, &oldv),
        &oldv,
        &scope(),
    );
    assert_eq!(new, old);
    assert_eq!(new["complete_answer"], true, "{new}");
}
fn quote(text: &str, value: &str) -> Value {
    let start = text.find(value).unwrap();
    json!({"start_byte":start,"end_byte":start+value.len(),"quote":value})
}
fn independent(v: &Value) -> (Value, String, Value, String) {
    let mut s = fixture("source");
    let mut q = fixture("question");
    let st = format!("{ST} Vera posee la tarjeta Azul.");
    let qt = format!("{QT} ¿Qué posee Vera?");
    for (doc, text) in [(&mut s, &st), (&mut q, &qt)] {
        doc["text_sha256"] = json!(
            Sha256::digest(text.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        rehash(doc, v);
    }
    s["references"].as_array_mut().unwrap().push(json!({"id":"card","scope_id":"root","mention":quote(&st,"Azul"),"name":quote(&st,"Azul"),"declared_class":quote(&st,"tarjeta"),"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}));
    let atom = json!({"predicate":"owns","arguments":[{"role":"owner","value":{"kind":"ref","id":"person"}},{"role":"object","value":{"kind":"ref","id":"card"}}],"polarity":"positive","evidence":[quote(&st,"Vera posee la tarjeta Azul.")]});
    s["claims"].as_array_mut().unwrap().push(json!({"id":"independent","scope_id":"root","qualifier_ids":[],"evidence":[quote(&st,"Vera posee la tarjeta Azul.")],"body":{"kind":"proposition","value":{"atom":atom,"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]}}}));
    q["requests"].as_array_mut().unwrap().push(json!({"id":"independent_request","scope_id":"root","mode":"answer","target":{"atom":{"predicate":"owns","arguments":[{"role":"owner","value":{"kind":"ref","id":"person"}}],"polarity":"positive","evidence":[quote(&qt,"¿Qué posee Vera?")]},"judgment":"fact","status":"state","qualifier_ids":[]},"slot":"object","class_filter":null,"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[quote(&qt,"¿Qué posee Vera?")]}));
    for (kind, id, target_kind, target_id) in [
        ("reference", "card", "scope", "root"),
        ("claim", "independent", "scope", "root"),
        ("claim", "independent", "reference", "person"),
        ("claim", "independent", "reference", "card"),
    ] {
        s["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":kind,"id":id},"requires":{"kind":target_kind,"id":target_id},"role":if target_kind=="scope"{"binding"}else{"identity"}}));
    }
    for (kind, id) in [("scope", "root"), ("reference", "person")] {
        q["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"request","id":"independent_request"},"requires":{"kind":kind,"id":id},"role":if kind=="scope"{"binding"}else{"identity"}}));
    }
    (s, st, q, qt)
}
#[test]
fn normative_indeterminacy_independent_request_retains_complete_answer() {
    let v = fixture("vocabulary");
    let (s, st, q, qt) = independent(&v);
    let sr = complete(&s, &st, &v);
    let qr = complete(&q, &qt, &v);
    let result = classify_pair_context(&sr, &qr, &v, &scope());
    assert_eq!(result["requests"][0]["support"], json!([]));
    assert_eq!(result["requests"][1]["coverage"], "complete", "{result}");
    assert_eq!(
        result["selected_claim_ids"],
        json!(["independent"]),
        "{result}"
    );
}
#[test]
fn normative_indeterminacy_request_dependency_endpoint_is_technical_invalid() {
    let v = fixture("vocabulary");
    let (s, st, mut q, qt) = independent(&v);
    q["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"request","id":"independent_request"},"requires":{"kind":"request","id":"requested_tutor"},"role":"premise"}));
    let sr = complete(&s, &st, &v);
    let qr = validate(&q, &qt, &v);
    // Invalid graph endpoint proves technical rejection, never a semantic control.
    assert_eq!(qr.report["status"], "invalid", "{}", qr.report);
    assert!(
        qr.report["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["code"] == "dangling_id")
    );
    let result = classify_pair_context(&sr, &qr, &v, &scope());
    assert_eq!(result["status"], "invalid");
    assert_eq!(result["selected_claim_ids"], json!([]));
}
#[test]
fn normative_indeterminacy_other_validation_debts_are_never_not_blocked() {
    let v = fixture("vocabulary");
    let mut q = fixture("question");
    q["requests"][0]["normative_scope_filter"] = json!("course");
    q["requests"][0]["context_bindings"][0]["action"]["actor"]["reason"] =
        json!("extraction_unresolved");
    let qr = validate(&q, QT, &v);
    assert_eq!(qr.report["status"], "partial");
    assert_eq!(qr.report["resolution_disposition"]["state"], "blocked");
    assert_eq!(
        qr.report["resolution_disposition"]["debts"][0]["kind"],
        "existing_validation_issue"
    );
    assert_eq!(
        qr.report["resolution_disposition"]["debts"][0]["issue"],
        qr.report["issues"][0]
    );
}
#[test]
fn normative_indeterminacy_internal_deep_invalid_input_exhausts_globally() {
    let v = fixture("vocabulary");
    let mut deep = Value::Null;
    for _ in 0..80 {
        deep = json!([deep]);
    }
    let mut q = fixture("question");
    q["extra"] = deep;
    let result = context_budget::with_operation(&v, &json!({}), || {
        let mut report = json!({"status":"invalid","issues":[],"document":null});
        normative_indeterminacy::decorate(&mut report, &q, &v);
        report
    });
    assert!(result.is_err());
}

#[test]
fn normative_indeterminacy_missing_constructor_evidence_never_promotes() {
    let v = fixture("vocabulary");
    for mutation in 0..4 {
        let mut q = fixture("question");
        match mutation {
            0 => q["requests"][0]["evidence"] = json!([]),
            1 => q["requests"][0]["target"]["atom"]["evidence"] = json!([]),
            2 => {
                q["requests"][0]["target"]["atom"]["arguments"][0]["value"]["evidence"] = json!([])
            }
            _ => {
                q["requests"][0]["context_bindings"][0]["action"]["action_kind"]["evidence"] =
                    json!([])
            }
        }
        let qr = validate(&q, QT, &v);
        assert_ne!(
            qr.report["status"], "complete",
            "mutation {mutation}: {}",
            qr.report
        );
    }
}

#[test]
fn normative_indeterminacy_partial_context_debt_keeps_actual_index() {
    let v = fixture("vocabulary");
    let mut q = fixture("question");
    let mut second = q["requests"][0]["context_bindings"][0].clone();
    second["id"] = json!("second_purpose");
    q["requests"][0]["context_bindings"][0]["action"]["actor"]["reason"] = json!("unspecified");
    q["requests"][0]["context_bindings"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let qr = validate(&q, QT, &v);
    assert_eq!(qr.report["status"], "partial", "{}", qr.report);
    let debt = qr.report["resolution_disposition"]["debts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["kind"] == "ambiguous_subordinate_actor")
        .unwrap();
    assert_eq!(debt["path"], "/requests/0/context_bindings/1/action/actor");
    assert_eq!(debt["issue"]["path"], debt["path"]);
    assert_eq!(debt["issue"]["code"], "request_context_ambiguous");
    assert_eq!(qr.report["document"], q);
    let s = fixture("source");
    blocked(
        &classify_pair_context(&complete(&s, ST, &v), &qr, &v, &scope()),
        &q,
    );
}
