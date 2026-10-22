use super::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
const ST: &str =
    include_str!("../../../../tests/fixtures/documentary-until-composition-v1/source.txt");
const QT: &str =
    include_str!("../../../../tests/fixtures/documentary-until-composition-v1/question.txt");
const PT: &str = include_str!(
    "../../../../tests/fixtures/documentary-until-composition-v1/possessive-source.txt"
);
fn fixture(name: &str) -> Value {
    serde_json::from_str(match name {
        "source" => {
            include_str!("../../../../tests/fixtures/documentary-until-composition-v1/source.json")
        }
        "question" => include_str!(
            "../../../../tests/fixtures/documentary-until-composition-v1/question.json"
        ),
        "possessive" => include_str!(
            "../../../../tests/fixtures/documentary-until-composition-v1/possessive-source.json"
        ),
        _ => include_str!(
            "../../../../tests/fixtures/documentary-until-composition-v1/vocabulary.json"
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
fn quote(text: &str, part: &str) -> Value {
    let start = text.find(part).unwrap();
    json!({"start_byte":start,"end_byte":start+part.len(),"quote":part})
}
fn rehash(d: &mut Value, v: &Value, text: &str) {
    d["vocabulary_sha256"] = json!(hash(v));
    d["text_sha256"] = json!(
        Sha256::digest(text.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
}
fn complete(d: &Value, t: &str, v: &Value) -> DocumentResult {
    let r = validate(d, t, v);
    assert_eq!(r.report["status"], "complete", "{}", r.report);
    r
}
fn pair(s: &Value, st: &str, q: &Value, v: &Value, scope: &Value) -> Value {
    classify_pair_context(&complete(s, st, v), &complete(q, QT, v), v, scope)
}
fn scope() -> Value {
    json!({"namespace":"until-controls","configuration":[],"time_selector":null,"closed_world":false})
}
fn partial(result: &Value) -> &[Value] {
    result["requests"][0]["partial"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
}
#[test]
fn linguistic_composition_positive_preserves_tree_and_all_open_debts() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    let q = fixture("question");
    let r = pair(&s, ST, &q, &v, &scope());
    assert_eq!(partial(&r).len(), 1, "{r}");
    let w = &partial(&r)[0];
    assert_eq!(w["kind"], "partial_relevance");
    assert_eq!(w["source_constraint"], s["claims"][0]["body"]);
    assert_eq!(w["source_owner"], s["claims"][0]);
    assert_eq!(w["source_document"], s);
    assert_eq!(w["question_document"], q);
    for key in ["answer_terms", "answer_propositions", "bindings"] {
        assert_eq!(w[key], json!([]));
    }
    for debt in [
        "until_attachment",
        "until_negation_scope",
        "boundary_subject",
        "measurement_episode",
        "answer_slot",
    ] {
        assert!(
            w["missing_constraints"]
                .as_array()
                .unwrap()
                .contains(&json!(debt)),
            "{w}"
        );
    }
    for key in [
        "core_independently_asserted",
        "measurement_occurrence_inferred",
        "boundary_occurrence_inferred",
        "internal_attachment_resolved",
        "boundary_subject_resolved",
    ] {
        assert_eq!(w["context_transport"][key], false);
    }
    assert_eq!(r["complete_answer"], false);
    assert_eq!(v["predicate_rows"].as_array().unwrap().len(), 1);
    assert_eq!(v["fixed_action_rows"], json!([]));
}
#[test]
fn linguistic_composition_possessive_keeps_no_physical_placeholder_and_never_selects_by_proximity()
{
    let v = fixture("vocabulary");
    let s = fixture("possessive");
    let q = fixture("question");
    let r = pair(&s, PT, &q, &v, &scope());
    assert!(partial(&r).is_empty(), "{r}");
    assert_eq!(r["selected_claim_ids"], json!([]));
    assert_eq!(s["references"], json!([]));
    assert!(
        r["unresolved_candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["source_document"] == s)
    );
}
#[test]
fn linguistic_composition_closed_ast_rejects_wrong_regions_resolution_incidence_and_utf8() {
    let v = fixture("vocabulary");
    for mutation in 0..10 {
        let mut s = fixture("source");
        let b = &mut s["claims"][0]["body"];
        match mutation {
            0 => b["operators"][0]["attachment"]["region_root_id"] = json!("player"),
            1 => b["operators"][0]["attachment"]["resolution"] = json!("duration"),
            2 => {
                b["operators"][0]["boundary"]["subject"]["resolution"] =
                    json!({"kind":"ref","id":"player"})
            }
            3 => b["core"]["operand"]["object"]["id"] = b["core"]["id"].clone(),
            4 => {
                let op = b["operators"][0].clone();
                b["operators"].as_array_mut().unwrap().push(op);
            }
            5 => b["core"]["operand"]["voice"]["kind"] = json!("impersonal_se"),
            6 => {
                b["operators"][0]["boundary"]["subject"]["realization_evidence"] =
                    quote(ST, "midió")
            }
            7 => b["core"]["operand"]["object"]["head_evidence"]["end_byte"] = json!(23),
            8 => b["core"]["operand"]["object"]["owner"]["mention_evidence"] = quote(ST, "Rumo"),
            _ => b["core"]["basis"] = json!("observation"),
        }
        assert_eq!(
            validate(&s, ST, &v).report["status"],
            "invalid",
            "mutation={mutation}"
        );
    }
}
#[test]
fn linguistic_composition_feature_requires_duration_and_absence_adds_no_walk() {
    let mut v = fixture("vocabulary");
    v["fixed_attribute_rows"] = json!(["weight"]);
    assert!(!request_context_profile::valid(&v));
    let mut v = fixture("vocabulary");
    v["request_context_profile"]["features"] = json!(["nominal_during"]);
    let mut s = fixture("source");
    rehash(&mut s, &v, ST);
    assert_eq!(validate(&s, ST, &v).report["status"], "invalid");
    context_budget::with_operation(&v, &json!({}), || {
        assert!(context_budget::charge(999_998, 0));
        assert!(linguistic_composition::blocked_nodes(&s).is_empty());
        assert!(context_budget::charge(1, 0));
    })
    .unwrap();
}
#[test]
fn linguistic_composition_canonical_ids_preserve_semantics_without_resolving_any_hole() {
    let s = fixture("source");
    let original = &s["claims"][0]["body"];
    let mut renamed = original.clone();
    for p in [
        "/core/id",
        "/core/operand/id",
        "/core/operand/object/id",
        "/operators/0/id",
        "/operators/0/boundary/id",
        "/operators/0/boundary/subject/id",
    ] {
        *renamed.pointer_mut(p).unwrap() = json!(format!("renamed-{p}"));
    }
    renamed["operators"][0]["attachment"]["region_root_id"] = renamed["core"]["id"].clone();
    assert!(linguistic_composition::valid(&renamed, ST, Some(&s)));
    assert_eq!(
        linguistic_composition::canonical_ast(original),
        linguistic_composition::canonical_ast(&renamed)
    );
    renamed["operators"][0]["boundary"]["subject"]["resolution"] = json!("player");
    assert_ne!(
        linguistic_composition::canonical_ast(original),
        linguistic_composition::canonical_ast(&renamed)
    );
}
#[test]
fn linguistic_composition_wrong_identity_attribute_status_and_namespace_are_not_selected() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    for mutation in 0..5 {
        let mut q = fixture("question");
        let mut sc = scope();
        match mutation {
            0 => q["references"][0]["name"] = Value::Null,
            1 => {
                q["requests"][0]["target"]["atom"]["arguments"][1]["value"]["id"] = json!("weight")
            }
            2 => q["requests"][0]["target"]["status"] = json!("planned"),
            3 => q["requests"][0]["mode"] = json!("constraint"),
            _ => sc["namespace"] = json!("other-namespace"),
        }
        let r = pair(&s, ST, &q, &v, &sc);
        assert!(partial(&r).is_empty(), "mutation={mutation}: {r}");
        assert_eq!(r["complete_answer"], false);
    }
}
fn configuration(text: &str, known: bool) -> Value {
    json!({"id":"condition","scope_id":"root","kind":"configuration","body":{"key":"fixture_context","value":{"kind":"text","value":"local","evidence":[quote(text,"duración")]}},"resolution":if known{"known"}else{"unknown"},"evidence":[quote(text,"duración")]})
}
fn condition(doc: &mut Value, text: &str, claim: bool, known: bool) {
    doc["qualifiers"] = json!([configuration(text, known)]);
    let table = if claim { "claims" } else { "requests" };
    let kind = if claim { "claim" } else { "request" };
    let id = doc[table][0]["id"].clone();
    doc[table][0]["qualifier_ids"] = json!(["condition"]);
    doc["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":kind,"id":id},"requires":{"kind":"qualifier","id":"condition"},"role":"configuration"}));
}
#[test]
fn linguistic_composition_configuration_is_complete_not_an_absent_wildcard() {
    let v = fixture("vocabulary");
    let mut s = fixture("source");
    let mut q = fixture("question");
    condition(&mut s, ST, true, true);
    condition(&mut q, QT, false, true);
    let r = pair(&s, ST, &q, &v, &scope());
    assert_eq!(partial(&r).len(), 1, "{r}");
    assert_eq!(partial(&r)[0]["source_qualifiers"], s["qualifiers"]);
    let s = fixture("source");
    let r = pair(&s, ST, &q, &v, &scope());
    assert!(partial(&r).is_empty(), "{r}");
    let mut sc = scope();
    sc["configuration"] = json!([{"key":"fixture_context","value":{"kind":"text","value":"local","evidence":[quote(QT,"duración")]}}]);
    let r = pair(&s, ST, &fixture("question"), &v, &sc);
    assert!(partial(&r).is_empty(), "{r}");
    let mut s = fixture("source");
    condition(&mut s, ST, true, true);
    let r = pair(&s, ST, &fixture("question"), &v, &sc);
    assert_eq!(partial(&r).len(), 1, "{r}");
}
#[test]
fn linguistic_composition_basis_remains_debt_and_internal_refs_do_not_assert_events() {
    let v = fixture("vocabulary");
    let s = fixture("source");
    let mut q = fixture("question");
    q["requests"][0]["basis_requirement"] = json!({"kind":"exact","value":"observation"});
    let r = pair(&s, ST, &q, &v, &scope());
    assert_eq!(partial(&r).len(), 1, "{r}");
    assert!(
        partial(&r)[0]["missing_constraints"]
            .as_array()
            .unwrap()
            .contains(&json!("basis"))
    );
    assert!(r["requests"][0]["support"].as_array().unwrap().is_empty());
    assert!(
        r["requests"][0]["refutation"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
#[test]
fn linguistic_composition_scalar_evidence_and_nested_work_share_one_global_budget() {
    let v = fixture("vocabulary");
    let many = json!(
        (0..2050)
            .map(|_| json!({"start_byte":0,"end_byte":1,"quote":"x"}))
            .collect::<Vec<_>>()
    );
    assert!(
        context_budget::with_operation(&v, &many, || {
            assert!(!context_budget::preflight(&many));
            context_budget::charge(usize::MAX, 0);
        })
        .is_err()
    );
    let s = fixture("source");
    assert!(
        context_budget::with_operation(&v, &json!({}), || {
            assert!(context_budget::charge(999_998, 0));
            let _ = linguistic_composition::blocked_nodes(&s);
        })
        .is_err()
    );
}

#[test]
fn linguistic_composition_effective_coverage_debt_is_not_an_independent_veto() {
    let v = fixture("vocabulary");
    for side in ["source", "question", "independent-reference"] {
        let mut s = fixture("source");
        let mut q = fixture("question");
        if side == "source" {
            s["coverage"][0]["state"] = json!("unresolved");
        } else if side == "question" {
            q["coverage"][0]["state"] = json!("unsupported");
        } else {
            let mut r = s["references"][0].clone();
            r["id"] = json!("independent");
            s["references"].as_array_mut().unwrap().push(r);
            s["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":"independent"},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
            s["coverage"].as_array_mut().unwrap().push(json!({"span":quote(ST,"Rumo"),"state":"unresolved","owner":{"kind":"reference","id":"independent"},"issue_id":null}));
        }
        let sr = validate(&s, ST, &v);
        let qr = validate(&q, QT, &v);
        assert_ne!(sr.report["status"], "invalid", "{}", sr.report);
        assert_ne!(qr.report["status"], "invalid", "{}", qr.report);
        let result = classify_pair_context(&sr, &qr, &v, &scope());
        assert_eq!(
            partial(&result).len(),
            if side == "independent-reference" {
                1
            } else {
                0
            },
            "{side}: {result}"
        );
    }
}
#[test]
fn linguistic_composition_same_alias_keeps_every_restrictive_link() {
    let v = fixture("vocabulary");
    let q = fixture("question");
    let mut s = fixture("source");
    let mut alias = s["references"][0].clone();
    alias["id"] = json!("alias");
    s["references"].as_array_mut().unwrap().push(alias);
    s["identity"] =
        json!([{"kind":"same","left":"player","right":"alias","evidence":[quote(ST,"Rumo")]}]);
    s["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":"alias"},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
    let r = pair(&s, ST, &q, &v, &scope());
    assert_eq!(partial(&r).len(), 1, "{r}");
    let mut restriction = s["references"][0].clone();
    restriction["id"] = json!("restriction");
    s["references"].as_array_mut().unwrap().push(restriction);
    s["references"][1]["links"] = json!([{"kind":"owned_by","target":"restriction","evidence":[quote(ST,"reproductor Rumo")]}]);
    s["dependencies"].as_array_mut().unwrap().extend([json!({"consumer":{"kind":"reference","id":"restriction"},"requires":{"kind":"scope","id":"root"},"role":"binding"}),json!({"consumer":{"kind":"reference","id":"alias"},"requires":{"kind":"reference","id":"restriction"},"role":"identity"})]);
    let r = pair(&s, ST, &q, &v, &scope());
    assert!(partial(&r).is_empty(), "{r}");
}
fn duration_fact(source: &mut Value, text: &str, local: &str, negative: bool) -> Value {
    let phrase = if negative {
        "No duró doce minutos"
    } else {
        "El reproductor Rumo duró doce minutos"
    };
    let p = json!({"atom":{"predicate":"quantity","arguments":[{"role":"subject","value":{"kind":"ref","id":"player"}},{"role":"attribute","value":{"kind":"symbol","id":"duration","evidence":[quote(text,"duró")] }},{"role":"amount","value":{"kind":"quantity","decimal":"12","unit":"min","bound":"exact","evidence":[quote(text,"doce minutos")]}}],"polarity":if negative{"negative"}else{"positive"},"evidence":[quote(text,phrase)]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]});
    let claim = json!({"id":local,"scope_id":"root","qualifier_ids":[],"evidence":[quote(text,phrase)],"body":{"kind":"proposition","value":p}});
    source["claims"].as_array_mut().unwrap().push(claim);
    source["dependencies"].as_array_mut().unwrap().extend([json!({"consumer":{"kind":"claim","id":local},"requires":{"kind":"reference","id":"player"},"role":"identity"}),json!({"consumer":{"kind":"claim","id":local},"requires":{"kind":"scope","id":"root"},"role":"binding"})]);
    source["coverage"].as_array_mut().unwrap().push(json!({"span":quote(text,phrase),"state":"represented","owner":{"kind":"claim","id":local},"issue_id":null}));
    p
}
#[test]
fn linguistic_composition_cuts_consumers_before_legacy_but_keeps_independent_answer() {
    let v = fixture("vocabulary");
    let q = fixture("question");
    let mut s = fixture("source");
    let text = format!("{ST} El reproductor Rumo duró doce minutos.");
    duration_fact(&mut s, &text, "independent", false);
    duration_fact(&mut s, &text, "dependent", false);
    s["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":"dependent"},"requires":{"kind":"claim","id":"composition"},"role":"premise"}));
    rehash(&mut s, &v, &text);
    let r = pair(&s, &text, &q, &v, &scope());
    assert!(
        r["selected_claim_ids"]
            .as_array()
            .unwrap()
            .contains(&json!("independent")),
        "{r}"
    );
    assert!(
        !r["selected_claim_ids"]
            .as_array()
            .unwrap()
            .contains(&json!("dependent")),
        "{r}"
    );
    assert!(
        r["requests"][0]["support"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["claim_ids"]
                .as_array()
                .unwrap()
                .contains(&json!("independent")))
    );
    assert_eq!(
        r["complete_answer"], true,
        "independent full proof must remain complete: {r}"
    );
    assert_eq!(r["requests"][0]["coverage"], "complete");
    assert!(
        !r["requests"][0]["missing_constraints"]
            .as_array()
            .unwrap()
            .contains(&json!("until_attachment"))
    );
    assert!(partial(&r).iter().any(|w| {
        w["missing_constraints"]
            .as_array()
            .unwrap()
            .contains(&json!("until_attachment"))
    }));
    // The dependent copy alone cannot be promoted after the independent proof leaves.
    s["claims"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["id"] != "independent");
    s["dependencies"]
        .as_array_mut()
        .unwrap()
        .retain(|d| d["consumer"]["id"] != "independent");
    s["coverage"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["owner"]["id"] != "independent");
    let only_dependent = pair(&s, &text, &q, &v, &scope());
    assert_eq!(only_dependent["complete_answer"], false, "{only_dependent}");
    assert!(
        !only_dependent["selected_claim_ids"]
            .as_array()
            .unwrap()
            .contains(&json!("dependent")),
        "{only_dependent}"
    );
}
#[test]
fn linguistic_composition_guard_needs_an_executable_unconflicted_fact() {
    let v = fixture("vocabulary");
    let q = fixture("question");
    let text = format!("{ST} El reproductor Rumo duró doce minutos. No duró doce minutos.");
    let mut s = fixture("source");
    let guard = duration_fact(&mut s, &text, "proof", false);
    s["claims"][0]["qualifier_ids"] = json!(["guard"]);
    s["qualifiers"] = json!([{"id":"guard","scope_id":"root","kind":"guard","body":guard,"resolution":"known","evidence":[quote(&text,"El reproductor Rumo duró doce minutos")]}]);
    s["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":"composition"},"requires":{"kind":"qualifier","id":"guard"},"role":"guard"}));
    rehash(&mut s, &v, &text);
    let r = pair(&s, &text, &q, &v, &scope());
    assert_eq!(
        partial(&r)
            .iter()
            .filter(|w| w["context_transport"]["construction"]
                == "negative_se_measurement_with_until_v1")
            .count(),
        1,
        "{r}"
    );
    duration_fact(&mut s, &text, "counterproof", true);
    rehash(&mut s, &v, &text);
    let r = pair(&s, &text, &q, &v, &scope());
    assert!(
        !partial(&r)
            .iter()
            .any(|w| w["context_transport"]["construction"]
                == "negative_se_measurement_with_until_v1"),
        "{r}"
    );
}
