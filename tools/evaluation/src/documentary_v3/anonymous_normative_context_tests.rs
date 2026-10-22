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
fn rf(text: &str, rid: &str, name: Option<&str>, class: Option<&str>) -> Value {
    json!({"id":rid,"scope_id":"root","mention":quote(text,name.or(class).unwrap()),"name":name.map(|n|quote(text,n)),"declared_class":class.map(|c|quote(text,c)),"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null})
}
fn fixture() -> (Value, Value, Value, &'static str) {
    let text = "policy class Acacia file class File9 Project7 27 days other class Other8";
    let v = json!({"version":"anonymous-context-test-v1","predicate_rows":[{"id":"governs","roles":{"authority":"ref","item":"ref","period":"quantity"},"distributive":true}],"fixed_attribute_rows":[],"fixed_action_rows":[],"semantic_type_rows":[{"id":"policy_type","class_quotes":["policy class"]},{"id":"file_type","class_quotes":["file class"]},{"id":"other_type","class_quotes":["other class"]}],"relevance_links":[]});
    let doc = |kind| json!({"version":"documentary_evidence_v3","text_sha256":digest(text.as_bytes()),"kind":kind,"vocabulary_sha256":digest(&serde_json::to_vec(&v).unwrap()),"domain_namespace":"anonymous-controls","scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":[rf(text,"policy",None,Some("policy class")),rf(text,"org",Some("Acacia"),None),rf(text,"item",Some("File9"),Some("file class")),rf(text,"project",Some("Project7"),None)],"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":[],"coverage":[{"span":quote(text,text),"state":"represented","owner":null,"issue_id":null}]});
    let mut a = doc("source");
    let mut b = doc("question");
    for d in [&mut a, &mut b] {
        d["references"][0]["links"] =
            json!([{"kind":"associated_with","target":"org","evidence":[quote(text,text)]}]);
        d["references"][2]["links"] =
            json!([{"kind":"associated_with","target":"project","evidence":[quote(text,text)]}]);
    }
    let atom = json!({"predicate":"governs","arguments":[{"role":"authority","value":{"kind":"ref","id":"policy"}},{"role":"item","value":{"kind":"ref","id":"item"}},{"role":"period","value":{"kind":"quantity","decimal":"27","unit":"days","bound":"exact","evidence":[quote(text,"27 days")]}}],"polarity":"positive","evidence":[quote(text,text)]});
    let p = json!({"atom":atom,"judgment":"fact","status":"state","basis":"normative","normative_scope":"policy","qualifier_ids":[]});
    a["claims"] = json!([{"id":"claim","scope_id":"root","qualifier_ids":[],"evidence":[quote(text,text)],"body":{"kind":"proposition","value":p}}]);
    let mut target = p;
    target.as_object_mut().unwrap().remove("basis");
    target.as_object_mut().unwrap().remove("normative_scope");
    target["atom"]["arguments"].as_array_mut().unwrap().pop();
    b["requests"] = json!([{"id":"ask","scope_id":"root","mode":"answer","target":target,"slot":"period","class_filter":null,"unit":null,"basis_requirement":{"kind":"exact","value":"normative"},"required_answer_extent":"one_value","normative_scope_filter":"policy","qualifier_ids":[],"premise_ids":[],"evidence":[quote(text,text)]}]);
    (a, b, v, text)
}
fn validated(mut d: Value, text: &str, v: &Value) -> DocumentResult {
    d["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(v).unwrap()));
    let mut deps = vec![];
    for rf in rows(&d, "references") {
        deps.push(json!({"consumer":{"kind":"reference","id":rf["id"]},"requires":{"kind":"scope","id":rf["scope_id"]},"role":"binding"}));
        for link in rows(rf, "links") {
            deps.push(json!({"consumer":{"kind":"reference","id":rf["id"]},"requires":{"kind":"reference","id":link["target"]},"role":"identity"}));
        }
    }
    for (collection, kind) in [("claims", "claim"), ("requests", "request")] {
        for item in rows(&d, collection) {
            deps.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"scope","id":item["scope_id"]},"role":"binding"}));
            for rf in rows(&d, "references") {
                deps.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"reference","id":rf["id"]},"role":"identity"}));
            }
            deps.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"reference","id":"policy"},"role":"normative_target"}));
        }
    }
    d["dependencies"] = json!(deps);
    let result = super::super::super::validate(&d, text, v);
    let report = serde_json::to_value(&result).unwrap();
    assert_eq!(report["status"], "complete", "{report}");
    result
}
fn classify(a: Value, b: Value, v: &Value, text: &str, active: bool) -> Value {
    let a = validated(a, text, v);
    let b = validated(b, text, v);
    let scope = json!({"namespace":"anonymous-controls","configuration":[],"time_selector":null,"closed_world":false});
    if active {
        super::super::super::classify_pair_context(&a, &b, v, &scope)
    } else {
        super::super::super::classify_pair(&a, &b, v, &scope)
    }
}
#[test]
fn anonymous_normative_context_preserves_source_without_scope_identity_or_answer() {
    let (a, b, v, t) = fixture();
    let original = a["claims"][0]["body"]["value"].clone();
    let old = classify(a.clone(), b.clone(), &v, t, false);
    assert_eq!(old["selected_claim_ids"], json!([]), "{old}");
    let new = classify(a, b, &v, t, true);
    assert_eq!(new["selected_claim_ids"], json!(["claim"]), "{new}");
    assert_eq!(new["complete_answer"], false);
    let w = &new["requests"][0]["partial"][0];
    assert_eq!(w["answer_terms"], json!([]));
    assert_eq!(w["answer_propositions"], json!([]));
    assert_eq!(w["bindings"], json!([]));
    assert_eq!(w["source_proposition"], original);
    assert_eq!(w["context_transport"]["identity_proven"], false);
    assert_eq!(
        w["context_transport"]["normative_scope_equal_proven"],
        false
    );
    assert_eq!(w["context_transport"]["requested_value_inferred"], false);
}
#[test]
fn anonymous_normative_context_rejects_unanchored_other_context_and_unsupported_class() {
    for change in [
        "unanchored",
        "other_context",
        "other_item",
        "unsupported_class",
        "wrong_basis",
        "extra_link",
    ] {
        let (mut a, mut b, v, t) = fixture();
        match change {
            "unanchored" => {
                a["references"][0]["links"] = json!([]);
                b["references"][0]["links"] = json!([]);
            }
            "other_context" => b["references"][1]["name"] = quote(t, "Other8"),
            "other_item" => b["references"][2]["name"] = quote(t, "Other8"),
            "unsupported_class" => {
                a["references"][0]["declared_class"] = quote(t, "other class");
            }
            "wrong_basis" => b["requests"][0]["basis_requirement"]["value"] = json!("observation"),
            "extra_link" => a["references"][0]["links"]
                .as_array_mut()
                .unwrap()
                .push(json!({"kind":"part_of","target":"project","evidence":[quote(t,t)]})),
            _ => unreachable!(),
        };
        let old = classify(a.clone(), b.clone(), &v, t, false);
        let r = classify(a, b, &v, t, true);
        assert_eq!(r, old, "{change}: legacy behavior changed: {r}");
        assert!(
            !rows(&r["requests"][0], "partial")
                .iter()
                .any(|w| s(w, "kind") == "partial_anonymous_normative_context_v2"),
            "{change}: {r}"
        );
    }
}
