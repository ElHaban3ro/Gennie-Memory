use super::*;
use serde_json::json;
use sha2::{Digest, Sha256};
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn q(t: &str, part: &str) -> Value {
    let start = t.find(part).unwrap();
    json!({"start_byte":start,"end_byte":start+part.len(),"quote":part})
}
fn vocab() -> Value {
    json!({"version":"nominal-synthetic-v1","predicate_rows":[{"id":"quantity","roles":{"subject":"ref","attribute":"attribute_id","amount":"quantity"},"distributive":true},{"id":"property","roles":{"subject":"ref","attribute":"attribute_id","value":"term"},"distributive":true}],"fixed_attribute_rows":["mass","nominal_qualitative_descriptor"],"fixed_action_rows":[],"semantic_type_rows":[],"relevance_links":[],"request_context_profile":{"version":"documentary_request_context_v1","features":["nominal_complement_de"]}})
}
fn rf(t: &str, id: &str, mention: &str, name: Option<&str>, class: Option<&str>) -> Value {
    json!({"id":id,"scope_id":"root","mention":q(t,mention),"name":name.map(|n|q(t,n)),"declared_class":class.map(|c|q(t,c)),"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null})
}
fn atom(t: &str, rid: &str, metadata: bool, filled: bool) -> Value {
    let mut args = vec![
        json!({"role":"subject","value":{"kind":"ref","id":rid}}),
        json!({"role":"attribute","value":{"kind":"symbol","id":if metadata{"nominal_qualitative_descriptor"}else{"mass"},"evidence":[q(t,t)]}}),
    ];
    if filled {
        args.push(if metadata {json!({"role":"value","value":{"kind":"text","value":"grande","evidence":[q(t,"grande")]}})} else {json!({"role":"amount","value":{"kind":"quantity","decimal":"8","unit":null,"bound":"exact","evidence":[q(t,t)]}})});
    }
    json!({"predicate":if metadata{"property"}else{"quantity"},"arguments":args,"polarity":"positive","evidence":[q(t,t)]})
}
fn deps(d: &mut Value) {
    let mut out = vec![];
    for r in d["references"].as_array().unwrap() {
        out.push(json!({"consumer":{"kind":"reference","id":r["id"]},"requires":{"kind":"scope","id":r["scope_id"]},"role":"binding"}));
        for l in r["links"].as_array().unwrap() {
            out.push(json!({"consumer":{"kind":"reference","id":r["id"]},"requires":{"kind":"reference","id":l["target"]},"role":"identity"}));
        }
    }
    for (table, kind) in [("claims", "claim"), ("requests", "request")] {
        for c in d[table].as_array().unwrap() {
            out.push(json!({"consumer":{"kind":kind,"id":c["id"]},"requires":{"kind":"scope","id":c["scope_id"]},"role":"binding"}));
            let p = if kind == "claim" {
                &c["body"]["value"]
            } else {
                &c["target"]
            };
            for a in p["atom"]["arguments"].as_array().unwrap() {
                if a["value"]["kind"] == "ref" {
                    out.push(json!({"consumer":{"kind":kind,"id":c["id"]},"requires":{"kind":"reference","id":a["value"]["id"]},"role":"identity"}));
                }
            }
        }
    }
    d["dependencies"] = json!(out);
}
fn doc(t: &str, kind: &str, tail: &str, metadata: bool) -> Value {
    let phrase = format!("Berta de {tail}");
    let mut head = rf(t, "head", &phrase, Some("Berta"), Some("bandeja"));
    head["links"] =
        json!([{"kind":"nominal_complement_de","target":"tail","evidence":[q(t,&phrase)]}]);
    let mut d = json!({"version":"documentary_evidence_v3","kind":kind,"text_sha256":hash(t.as_bytes()),"vocabulary_sha256":hash(&serde_json::to_vec(&vocab()).unwrap()),"domain_namespace":"nominal-controls","scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":[head,rf(t,"tail",tail,None,Some(tail)),rf(t,"independent","Lino",Some("Lino"),None)],"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":[],"coverage":[{"span":q(t,t),"state":"represented","owner":null,"issue_id":null}]});
    let p = json!({"atom":atom(t,if metadata{"tail"}else{"head"},metadata,kind=="source"),"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]});
    if kind == "source" {
        d["claims"] = json!([{"id":"c","scope_id":"root","qualifier_ids":[],"evidence":[q(t,t)],"body":{"kind":"proposition","value":p}}]);
    } else {
        let mut target = p;
        target.as_object_mut().unwrap().remove("basis");
        target.as_object_mut().unwrap().remove("normative_scope");
        d["requests"] = json!([{"id":"ask","scope_id":"root","mode":"answer","target":target,"slot":if metadata{"value"}else{"amount"},"class_filter":null,"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[q(t,t)]}]);
    }
    deps(&mut d);
    d
}
fn validated(d: &Value, t: &str) -> DocumentResult {
    let v = validate(d, t, &vocab());
    assert_eq!(v.report["status"], "complete", "{}", v.report);
    v
}
fn pair(a: &Value, at: &str, b: &Value, bt: &str) -> Value {
    classify_pair_context(
        &validated(a, at),
        &validated(b, bt),
        &vocab(),
        &json!({"namespace":"nominal-controls","configuration":[],"time_selector":null,"closed_world":false}),
    )
}
const PAPER: &str = "bandeja Berta de papel grande. Lino tiene masa ocho.";
const METAL: &str = "bandeja Berta de metal grande. Lino tiene masa ocho.";
#[test]
fn nominal_complement_complete_validation_and_typed_identity_guard() {
    let a = doc(PAPER, "source", "papel", false);
    let b = doc(PAPER, "question", "papel", false);
    assert_eq!(pair(&a, PAPER, &b, PAPER)["complete_answer"], true);
    let metal = doc(METAL, "source", "metal", false);
    {
        let mut bad = a.clone();
        bad["references"][0]["links"] = json!([]);
        deps(&mut bad);
        let r = pair(&bad, PAPER, &b, PAPER);
        assert_eq!(r["complete_answer"], false, "{r}");
        assert_eq!(r["selected_claim_ids"], json!([]), "{r}");
    }
    assert_eq!(
        pair(&metal, METAL, &b, PAPER)["selected_claim_ids"],
        json!([])
    );
    let mut generic = a.clone();
    generic["references"][0]["links"][0]["kind"] = json!("associated_with");
    deps(&mut generic);
    assert_eq!(
        pair(&generic, PAPER, &b, PAPER)["selected_claim_ids"],
        json!([])
    );
    let mut alias = a.clone();
    let mut r = alias["references"][0].clone();
    r["id"] = json!("alias");
    r["links"] = json!([]);
    alias["references"].as_array_mut().unwrap().push(r);
    alias["identity"] =
        json!([{"kind":"same","left":"head","right":"alias","evidence":[q(PAPER,"Berta")]}]);
    alias["claims"][0]["body"]["value"]["atom"]["arguments"][0]["value"]["id"] = json!("alias");
    deps(&mut alias);
    assert_eq!(pair(&alias, PAPER, &b, PAPER)["complete_answer"], true);
    let mut anon = a.clone();
    anon["references"][0]["name"] = Value::Null;
    anon["references"][0]["mention"] = q(PAPER, "bandeja Berta de papel");
    anon["references"][0]["declared_class"] = q(PAPER, "bandeja Berta");
    anon["references"][0]["links"][0]["evidence"][0] = q(PAPER, "bandeja Berta de papel");
    let mut aq = anon.clone();
    aq["kind"] = json!("question");
    aq["claims"] = json!([]);
    aq["requests"] = b["requests"].clone();
    deps(&mut aq);
    assert_eq!(
        pair(&anon, PAPER, &aq, PAPER)["selected_claim_ids"],
        json!([])
    );
}
#[test]
fn nominal_complement_rejects_invalid_evidence_permission_and_scope() {
    let base = doc(PAPER, "source", "papel", false);
    for mode in [
        "quote", "marker", "target", "self", "scope", "extra", "empty",
    ] {
        let mut d = base.clone();
        match mode {
            "quote" => d["references"][0]["links"][0]["evidence"][0]["quote"] = json!("invented"),
            "marker" => d["references"][0]["links"][0]["evidence"][0] = q(PAPER, "papel"),
            "target" => d["references"][0]["links"][0]["target"] = json!("independent"),
            "self" => d["references"][0]["links"][0]["target"] = json!("head"),
            "scope" => d["references"][1]["scope_id"] = json!("missing"),
            "extra" => d["references"][0]["links"][0]["extra"] = json!(true),
            _ => d["references"][0]["links"][0]["evidence"] = json!([]),
        }
        assert_eq!(
            validate(&d, PAPER, &vocab()).report["status"],
            "invalid",
            "{mode}"
        );
    }
    let mut v = vocab();
    v.as_object_mut().unwrap().remove("request_context_profile");
    let mut d = base;
    d["vocabulary_sha256"] = json!(hash(&serde_json::to_vec(&v).unwrap()));
    assert_eq!(validate(&d, PAPER, &v).report["status"], "invalid");
}
#[test]
fn nominal_complement_metadata_is_local_partial_and_cannot_be_global_fact() {
    let a = doc(PAPER, "source", "papel", true);
    let b = doc(PAPER, "question", "papel", true);
    let r = pair(&a, PAPER, &b, PAPER);
    assert_eq!(r["complete_answer"], false);
    assert_eq!(r["selected_claim_ids"], json!(["c"]), "{r}");
    let w = &r["requests"][0]["partial"][0];
    assert_eq!(w["kind"], "local_nominal_metadata_context");
    for field in ["bindings", "answer_terms", "answer_propositions"] {
        assert_eq!(w[field], json!([]));
    }
    assert_eq!(w["source_constraint"], a["claims"][0]);
    let mut global = b.clone();
    global["references"][0]["links"] = json!([]);
    deps(&mut global);
    assert_eq!(
        pair(&a, PAPER, &global, PAPER)["selected_claim_ids"],
        json!([])
    );
    let metal = doc(METAL, "question", "metal", true);
    assert_eq!(
        pair(&a, PAPER, &metal, METAL)["selected_claim_ids"],
        json!([])
    );
}
fn replace_text(d: &mut Value, old: &str, new: &str, text: &str) {
    assert_eq!(old.len(), new.len());
    fn walk(v: &mut Value, old: &str, new: &str) {
        match v {
            Value::String(s) => *s = s.replace(old, new),
            Value::Array(a) => {
                for v in a {
                    walk(v, old, new)
                }
            }
            Value::Object(o) => {
                for v in o.values_mut() {
                    walk(v, old, new)
                }
            }
            _ => {}
        }
    }
    walk(d, old, new);
    d["text_sha256"] = json!(hash(text.as_bytes()));
}
#[test]
fn nominal_complement_literal_marker_utf8_and_distinct_owner_controls() {
    let utf8 = "Á bandeja Berta de papel grande. Lino tiene masa ocho.";
    validated(&doc(utf8, "source", "papel", false), utf8);
    let mut bad = doc(PAPER, "source", "papel", false);
    let en = PAPER.replace(" de ", " en ");
    replace_text(&mut bad, " de ", " en ", &en);
    assert_eq!(validate(&bad, &en, &vocab()).report["status"], "invalid");
    let source = doc(PAPER, "source", "papel", true);
    let mut other = doc(PAPER, "question", "papel", true);
    let ct = PAPER.replace("Berta", "Celia");
    replace_text(&mut other, "Berta", "Celia", &ct);
    assert_eq!(
        pair(&source, PAPER, &other, &ct)["selected_claim_ids"],
        json!([])
    );
    let mut reused = doc(PAPER, "source", "papel", false);
    let mut head = reused["references"][0].clone();
    head["id"] = json!("other-head");
    reused["references"].as_array_mut().unwrap().push(head);
    deps(&mut reused);
    let result = validate(&reused, PAPER, &vocab());
    assert_eq!(result.report["status"], "partial", "{}", result.report);
    assert!(nominal_de::descriptor_key(&reused, "tail").is_none());
}
#[test]
fn nominal_complement_metadata_consumers_are_blocked_independent_fact_remains() {
    let mut source = doc(PAPER, "source", "papel", true);
    let quantity = doc(PAPER, "source", "papel", false)["claims"][0].clone();
    let mut dependent = quantity.clone();
    dependent["id"] = json!("dependent");
    let mut independent = quantity;
    independent["id"] = json!("independent-claim");
    independent["body"]["value"]["atom"]["arguments"][0]["value"]["id"] = json!("independent");
    source["claims"]
        .as_array_mut()
        .unwrap()
        .extend([dependent, independent]);
    deps(&mut source);
    source["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":"dependent"},"requires":{"kind":"claim","id":"c"},"role":"premise"}));
    let head_question = doc(PAPER, "question", "papel", false);
    let r = pair(&source, PAPER, &head_question, PAPER);
    assert!(
        !r["selected_claim_ids"]
            .as_array()
            .unwrap()
            .contains(&json!("dependent")),
        "{r}"
    );
    let mut independent_question = head_question;
    independent_question["requests"][0]["target"]["atom"]["arguments"][0]["value"]["id"] =
        json!("independent");
    deps(&mut independent_question);
    let r = pair(&source, PAPER, &independent_question, PAPER);
    assert_eq!(r["selected_claim_ids"], json!(["independent-claim"]), "{r}");
    assert_eq!(r["complete_answer"], true, "{r}");
    let blocked =
        context_budget::with_operation(&vocab(), &json!({}), || nominal_de::blocked_nodes(&source))
            .unwrap();
    assert!(blocked.contains(&("claim".into(), "c".into())));
    assert!(blocked.contains(&("claim".into(), "dependent".into())));
    assert!(!blocked.contains(&("claim".into(), "independent-claim".into())));
}
#[test]
fn nominal_complement_tail_alias_restrictions_and_legacy_adapter_are_not_bypassed() {
    let a = doc(PAPER, "source", "papel", false);
    let b = doc(PAPER, "question", "papel", false);
    let mut alias = a.clone();
    let mut tail = alias["references"][1].clone();
    tail["id"] = json!("tail-alias");
    alias["references"].as_array_mut().unwrap().push(tail);
    alias["identity"] =
        json!([{"kind":"same","left":"tail","right":"tail-alias","evidence":[q(PAPER,"papel")]}]);
    deps(&mut alias);
    assert_eq!(pair(&alias, PAPER, &b, PAPER)["complete_answer"], true);
    alias["references"][3]["name"] = q(PAPER, "papel");
    assert_eq!(
        validate(&alias, PAPER, &vocab()).report["status"],
        "partial"
    );
    let av = validated(&a, PAPER);
    let bv = validated(&b, PAPER);
    let scope = json!({"namespace":"nominal-controls","configuration":[],"time_selector":null,"closed_world":false});
    assert_eq!(
        classify_pair(&av, &bv, &vocab(), &scope)["status"],
        "invalid"
    );
    let mut v = vocab();
    v.as_object_mut().unwrap().remove("request_context_profile");
    let mut legacy = a;
    legacy["references"][0]["links"] = json!([]);
    deps(&mut legacy);
    legacy["vocabulary_sha256"] = json!(hash(&serde_json::to_vec(&v).unwrap()));
    let before = validate(&legacy, PAPER, &v);
    let after = validate(&legacy, PAPER, &v);
    assert_eq!(
        serde_json::to_vec(&before).unwrap(),
        serde_json::to_vec(&after).unwrap()
    );
}

#[test]
fn nominal_complement_absent_permission_has_no_traversal_budget_cost() {
    let input = doc(PAPER, "source", "papel", true);
    let other = json!({"request_context_profile":{"version":"documentary_request_context_v1","features":["nominal_during"]}});
    context_budget::with_operation(&other, &json!({}), || {
        assert!(!context_budget::nominal_enabled());
        // Exhaust the actual remainder after preflight: disabled helpers spend nothing.
        let (remaining_steps, remaining_bytes) = context_budget::remaining_for_test().unwrap();
        assert!(context_budget::charge(remaining_steps, remaining_bytes));
        assert_eq!(nominal_de::targets(&input, "head"), Some(vec![]));
        assert!(nominal_de::component(&input, "head").is_none());
        assert!(nominal_de::descriptor_key(&input, "tail").is_none());
        assert!(!nominal_de::is_tail(&input, "tail"));
        assert!(!nominal_de::lexical_metadata(&input, &input["claims"][0]));
        assert!(!nominal_de::local_metadata_request(
            &input,
            &input["claims"][0]
        ));
        assert!(nominal_de::blocked_nodes(&input).is_empty());
        assert!(!context_budget::exhausted());
    })
    .unwrap();
}
#[test]
fn nominal_complement_metadata_materialization_limit_preserves_validated_input() {
    let source = doc(PAPER, "source", "papel", true);
    let question = doc(PAPER, "question", "papel", true);
    let a = validated(&source, PAPER);
    let b = validated(&question, PAPER);
    let before = serde_json::to_vec(&a).unwrap();
    let result = context_budget::with_operation(&vocab(), &json!({}), || {
        let (_, remaining_bytes) = context_budget::remaining_for_test().unwrap();
        assert!(context_budget::charge(
            0,
            remaining_bytes.checked_sub(2048).unwrap()
        ));
        classify_pair_context(
            &a,
            &b,
            &vocab(),
            &json!({"namespace":"nominal-controls","configuration":[],"time_selector":null,"closed_world":false}),
        )
    });
    assert_eq!(result.unwrap_err(), "request_context_limit");
    assert_eq!(before, serde_json::to_vec(&a).unwrap());
    assert_eq!(a.report["document"], source);
}

#[test]
fn nominal_complement_long_alias_ids_spend_shared_copy_budget() {
    let long = "x".repeat(1024 * 1024);
    let d = json!({"identity":[{"kind":"same","left":long,"right":long}],"references":[]});
    let result = context_budget::with_operation(&vocab(), &json!({}), || {
        for _ in 0..32 {
            if nominal_de::component(&d, &long).is_none() {
                break;
            }
        }
        assert!(context_budget::exhausted());
    });
    assert_eq!(result.unwrap_err(), "request_context_limit");
    // A duplicate self-alias must not allocate a second copy of its ID.
    context_budget::with_operation(&vocab(), &json!({}), || {
        let (_, remaining_bytes) = context_budget::remaining_for_test().unwrap();
        assert!(context_budget::charge(
            0,
            remaining_bytes.checked_sub(long.len() + 64).unwrap()
        ));
        assert_eq!(nominal_de::component(&d, &long).unwrap().len(), 1);
        assert!(!context_budget::exhausted());
    })
    .unwrap();
}
