use super::*;
use sha2::{Digest, Sha256};
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn evidence(text: &str, part: &str) -> Value {
    let start = text.find(part).unwrap();
    json!({"start_byte":start,"end_byte":start+part.len(),"quote":part})
}
fn fixture() -> (Value, Value, Value, &'static str) {
    let text = "Melsa operó durante 17 minutos bajo ajuste A. Melsa operó durante 23 minutos bajo ajuste A. Nerva operó durante 23 minutos bajo ajuste B. Nerva revisó durante 23 segundos bajo ajuste B.";
    let vocabulary = json!({"version":"configuration-slot-controls-v1","predicate_rows":[{"id":"action_actor","roles":{"actor":"ref","action_kind":"action_id"},"distributive":true},{"id":"action_undergoer","roles":{"subject":"ref","action_kind":"action_id"},"distributive":true}],"fixed_attribute_rows":[],"fixed_action_rows":["operate","review"],"semantic_type_rows":[],"relevance_links":[],"configuration_request_profile":{"version":"documentary_configuration_slots_v1","keys":["operating_duration","other_key"]}});
    let mut source = json!({"version":"documentary_evidence_v3","text_sha256":digest(text.as_bytes()),"kind":"source","vocabulary_sha256":digest(&serde_json::to_vec(&vocabulary).unwrap()),"domain_namespace":"configuration-controls","scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":[{"id":"actor","scope_id":"root","mention":evidence(text,"Melsa"),"name":evidence(text,"Melsa"),"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}],"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":[],"coverage":[{"span":evidence(text,text),"state":"represented","owner":null,"issue_id":null}]});
    let setting = json!({"id":"setting","scope_id":"root","kind":"configuration","body":{"key":"setting","value":{"kind":"text","value":"A","evidence":[evidence(text,"A")]}},"resolution":"known","evidence":[evidence(text,"A")]});
    let duration = json!({"id":"duration","scope_id":"root","kind":"configuration","body":{"key":"operating_duration","value":{"kind":"quantity","decimal":"17","unit":"minutos","bound":"exact","evidence":[evidence(text,"17 minutos")]}},"resolution":"known","evidence":[evidence(text,"17 minutos")]});
    let atom = json!({"predicate":"action_actor","arguments":[{"role":"actor","value":{"kind":"ref","id":"actor"}},{"role":"action_kind","value":{"kind":"symbol","id":"operate","evidence":[evidence(text,"operó")]}}],"polarity":"positive","evidence":[evidence(text,text)]});
    source["qualifiers"] = json!([setting.clone(), duration]);
    source["claims"] = json!([{"id":"c1","scope_id":"root","qualifier_ids":["duration","setting"],"evidence":[evidence(text,text)],"body":{"kind":"proposition","value":{"atom":atom.clone(),"judgment":"fact","status":"occurred","basis":"assignment","normative_scope":null,"qualifier_ids":[]}}}]);
    let mut question = source.clone();
    question["kind"] = json!("question");
    question["claims"] = json!([]);
    question["qualifiers"] = json!([setting]);
    question["requests"] = json!([{"id":"ask","scope_id":"root","mode":"configuration_slot","target":{"atom":atom,"judgment":"fact","status":"occurred","qualifier_ids":[]},"slot":"operating_duration","class_filter":null,"unit":{"symbol":"minutos","evidence":[evidence(text,"minutos")]},"basis_requirement":{"kind":"exact","value":"assignment"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":["setting"],"premise_ids":[],"evidence":[evidence(text,text)]}]);
    (source, question, vocabulary, text)
}
fn run(
    mut source: Value,
    mut question: Value,
    vocabulary: Value,
    text: &str,
) -> (Value, Value, Value) {
    for doc in [&mut source, &mut question] {
        doc["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&vocabulary).unwrap()));
        let mut dependencies = vec![];
        for reference in rows(doc, "references") {
            dependencies.push(json!({"consumer":{"kind":"reference","id":reference["id"]},"requires":{"kind":"scope","id":reference["scope_id"]},"role":"binding"}));
        }
        for (collection, kind) in [("claims", "claim"), ("requests", "request")] {
            for item in rows(doc, collection) {
                dependencies.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"scope","id":item["scope_id"]},"role":"binding"}));
                for reference in rows(doc, "references") {
                    dependencies.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"reference","id":reference["id"]},"role":"identity"}));
                }
                let mut ids = rows(item, "qualifier_ids").to_vec();
                ids.extend(
                    rows(&item["body"]["value"], "qualifier_ids")
                        .iter()
                        .cloned(),
                );
                ids.extend(rows(&item["target"], "qualifier_ids").iter().cloned());
                ids.sort_by_key(Value::to_string);
                ids.dedup();
                for qid in ids {
                    dependencies.push(json!({"consumer":{"kind":kind,"id":item["id"]},"requires":{"kind":"qualifier","id":qid},"role":"configuration"}));
                }
            }
        }
        doc["dependencies"] = json!(dependencies);
    }
    let source = super::super::super::validate(&source, text, &vocabulary);
    let question = super::super::super::validate(&question, text, &vocabulary);
    let sr = serde_json::to_value(&source).unwrap();
    let qr = serde_json::to_value(&question).unwrap();
    let report = super::super::super::classify_pair(
        &source,
        &question,
        &vocabulary,
        &json!({"namespace":"configuration-controls","configuration":[],"time_selector":null,"closed_world":false}),
    );
    (sr, qr, report)
}
fn complete_case(source: Value, question: Value, vocabulary: Value, text: &str) -> Value {
    let (sr, qr, r) = run(source, question, vocabulary, text);
    assert_eq!(sr["status"], "complete", "{sr}");
    assert_eq!(qr["status"], "complete", "{qr}");
    r
}
#[test]
fn configuration_slot_recovers_exact_same_claim_parameter_without_intrinsic_property() {
    let (s, q, v, t) = fixture();
    let original = s["claims"][0]["body"]["value"].clone();
    let r = complete_case(s, q, v, t);
    assert_eq!(r["complete_answer"], true, "{r}");
    let w = &r["requests"][0]["support"][0];
    assert_eq!(w["kind"], "configuration_slot");
    assert_eq!(w["source_proposition"], original);
    assert_eq!(w["answer_terms"][0]["decimal"], "17");
    assert_eq!(w["answer_terms"][0]["unit"], "minutos");
    assert_eq!(w["configuration_key"], "operating_duration");
}
#[test]
fn configuration_slot_keeps_actor_predicate_key_setting_basis_time_and_guards_restrictive() {
    for change in [
        "actor",
        "predicate",
        "key",
        "setting",
        "basis",
        "time",
        "guard",
    ] {
        let (mut s, mut q, v, t) = fixture();
        match change {
            "actor" => {
                q["references"][0]["name"] = evidence(t, "Nerva");
                q["references"][0]["mention"] = evidence(t, "Nerva");
            }
            "predicate" => {
                q["requests"][0]["target"]["atom"]["predicate"] = json!("action_undergoer");
                q["requests"][0]["target"]["atom"]["arguments"][0]["role"] = json!("subject");
            }
            "key" => q["requests"][0]["slot"] = json!("other_key"),
            "setting" => {
                q["qualifiers"][0]["body"]["value"]["value"] = json!("B");
                q["qualifiers"][0]["body"]["value"]["evidence"] = json!([evidence(t, "B")]);
            }
            "basis" => s["claims"][0]["body"]["value"]["basis"] = json!("description"),
            "time" => {
                s["qualifiers"].as_array_mut().unwrap().push(json!({"id":"when","scope_id":"root","kind":"time","body":{"role":"occurred_at","value":{"kind":"time","value":"A","format":"literal","evidence":[evidence(t,"A")]}},"resolution":"known","evidence":[evidence(t,"A")]}));
                s["claims"][0]["qualifier_ids"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("when"));
            }
            "guard" => {
                s["qualifiers"][0]["kind"] = json!("guard");
                s["qualifiers"][0]["body"] = s["claims"][0]["body"]["value"].clone();
                s["qualifiers"][0]["body"]["status"] = json!("state");
            }
            _ => unreachable!(),
        }
        let (sr, qr, r) = run(s, q, v, t);
        assert_ne!(sr["status"], "invalid", "{change}: {sr}");
        assert_ne!(qr["status"], "invalid", "{change}: {qr}");
        assert_eq!(r["complete_answer"], false, "{change}: {r}");
        if change == "actor" {
            assert_eq!(r["selected_claim_ids"], json!([]), "{r}");
        }
    }
}
#[test]
fn configuration_slot_rejects_absent_unknown_inexact_or_wrong_unit_values() {
    for change in ["absent", "unknown", "bound", "unit"] {
        let (mut s, mut q, v, t) = fixture();
        match change {
            "absent" => s["claims"][0]["qualifier_ids"] = json!(["setting"]),
            "unknown" => s["qualifiers"][1]["resolution"] = json!("unknown"),
            "bound" => s["qualifiers"][1]["body"]["value"]["bound"] = json!("at_least"),
            "unit" => {
                q["requests"][0]["unit"]["symbol"] = json!("segundos");
                q["requests"][0]["unit"]["evidence"] = json!([evidence(t, "segundos")]);
            }
            _ => unreachable!(),
        }
        let (sr, qr, r) = run(s, q, v, t);
        assert_ne!(sr["status"], "invalid", "{sr}");
        assert_ne!(qr["status"], "invalid", "{qr}");
        assert_eq!(r["complete_answer"], false, "{change}: {r}");
    }
}
#[test]
fn configuration_slot_never_donates_parameter_from_another_claim_or_selects_first_conflict() {
    for change in ["other_actor", "conflicting", "equal"] {
        let (mut s, q, v, t) = fixture();
        let mut second = s["claims"][0].clone();
        second["id"] = json!("c2");
        let mut duration = s["qualifiers"][1].clone();
        duration["id"] = json!("duration2");
        if change != "equal" {
            duration["body"]["value"]["decimal"] = json!("23");
            duration["body"]["value"]["evidence"] = json!([evidence(t, "23")]);
        }
        second["qualifier_ids"] = json!(["duration2", "setting"]);
        if change == "other_actor" {
            let mut actor = s["references"][0].clone();
            actor["id"] = json!("other");
            actor["name"] = evidence(t, "Nerva");
            actor["mention"] = evidence(t, "Nerva");
            s["references"].as_array_mut().unwrap().push(actor);
            second["body"]["value"]["atom"]["arguments"][0]["value"]["id"] = json!("other");
            s["claims"][0]["qualifier_ids"] = json!(["setting"]);
        }
        s["qualifiers"].as_array_mut().unwrap().push(duration);
        s["claims"].as_array_mut().unwrap().push(second);
        let r = complete_case(s, q, v, t);
        assert_eq!(r["complete_answer"], change == "equal", "{change}: {r}");
    }
}
#[test]
fn configuration_slot_requires_closed_opt_in_profile_and_complete_target() {
    for change in [
        "absent",
        "duplicate",
        "extra_field",
        "wrong_version",
        "missing_role",
        "wrong_extent",
    ] {
        let (s, mut q, mut v, t) = fixture();
        match change {
            "absent" => {
                v.as_object_mut()
                    .unwrap()
                    .remove("configuration_request_profile");
            }
            "duplicate" => {
                v["configuration_request_profile"]["keys"] =
                    json!(["operating_duration", "operating_duration"])
            }
            "extra_field" => v["configuration_request_profile"]["extra"] = json!(true),
            "wrong_version" => v["configuration_request_profile"]["version"] = json!("future"),
            "missing_role" => {
                q["requests"][0]["target"]["atom"]["arguments"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            "wrong_extent" => q["requests"][0]["required_answer_extent"] = json!("all_values"),
            _ => unreachable!(),
        }
        let (_, qr, r) = run(s, q, v, t);
        assert_eq!(qr["status"], "invalid", "{change}: {qr}");
        assert_eq!(r["complete_answer"], false);
    }
}

#[test]
fn configuration_slot_rejects_requested_key_as_direct_question_restriction() {
    for in_target in [false, true] {
        let (source, mut question, vocabulary, text) = fixture();
        question["qualifiers"]
            .as_array_mut()
            .unwrap()
            .push(source["qualifiers"][1].clone());
        if in_target {
            question["requests"][0]["target"]["qualifier_ids"] = json!(["duration"]);
        } else {
            question["requests"][0]["qualifier_ids"]
                .as_array_mut()
                .unwrap()
                .push(json!("duration"));
        }
        let (_, qr, result) = run(source, question, vocabulary, text);
        assert_eq!(qr["status"], "invalid", "{qr}");
        assert_eq!(result["complete_answer"], false);
    }
}
#[test]
fn configuration_slot_unknown_or_incompatible_second_value_cannot_choose_favorable_first() {
    for change in ["unknown", "unit", "nested_guard"] {
        let (mut source, question, vocabulary, text) = fixture();
        let mut second = source["claims"][0].clone();
        second["id"] = json!("c2");
        let mut q = source["qualifiers"][1].clone();
        q["id"] = json!("duration2");
        if change == "unknown" {
            q["resolution"] = json!("unknown");
        }
        if change == "unit" {
            q["body"]["value"]["unit"] = json!("segundos");
            q["body"]["value"]["evidence"] = json!([evidence(text, "23 segundos")]);
            q["body"]["value"]["decimal"] = json!("23");
        }
        if change == "nested_guard" {
            // The guard remains a guard even when its proposition carries a
            // qualifier with the requested configuration key.
            let mut guard = source["qualifiers"][0].clone();
            guard["id"] = json!("guard");
            guard["kind"] = json!("guard");
            guard["body"] = source["claims"][0]["body"]["value"].clone();
            guard["body"]["qualifier_ids"] = json!(["duration2"]);
            guard["body"]["atom"]["arguments"][1]["value"]["id"] = json!("review");
            guard["body"]["atom"]["arguments"][1]["value"]["evidence"] =
                json!([evidence(text, "revisó")]);
            source["qualifiers"].as_array_mut().unwrap().push(guard);
            source["claims"][0]["qualifier_ids"]
                .as_array_mut()
                .unwrap()
                .push(json!("guard"));
        } else {
            second["qualifier_ids"] = json!(["duration2", "setting"]);
            source["claims"].as_array_mut().unwrap().push(second);
        }
        source["qualifiers"].as_array_mut().unwrap().push(q);
        let (sr, qr, result) = run(source, question, vocabulary, text);
        assert_ne!(sr["status"], "invalid", "{change}: {sr}");
        assert_ne!(qr["status"], "invalid", "{qr}");
        assert_eq!(result["complete_answer"], false, "{change}: {result}");
        if change == "unit" {
            assert!(
                !rows(&result["requests"][0], "conflicts").is_empty(),
                "{result}"
            );
        }
    }
}

#[test]
fn configuration_profile_does_not_change_legacy_truth_mode() {
    let (source, mut question, vocabulary, text) = fixture();
    question["requests"][0]["mode"] = json!("truth");
    question["requests"][0]["slot"] = json!("truth");
    question["requests"][0]["required_answer_extent"] = json!("truth_status");
    question["requests"][0]["unit"] = Value::Null;
    let with_profile = complete_case(source.clone(), question.clone(), vocabulary.clone(), text);
    let mut absent = vocabulary;
    absent
        .as_object_mut()
        .unwrap()
        .remove("configuration_request_profile");
    let without_profile = complete_case(source, question, absent, text);
    assert_eq!(with_profile, without_profile);
}

#[test]
fn configuration_slot_explicit_negation_blocks_only_compatible_positive_operation() {
    for change in ["same", "other_actor", "other_action", "other_setting"] {
        let (mut source, question, vocabulary, text) = fixture();
        let mut negative = source["claims"][0].clone();
        negative["id"] = json!("negative");
        negative["body"]["value"]["atom"]["polarity"] = json!("negative");
        match change {
            "other_actor" => {
                let mut actor = source["references"][0].clone();
                actor["id"] = json!("other");
                actor["name"] = evidence(text, "Nerva");
                actor["mention"] = evidence(text, "Nerva");
                source["references"].as_array_mut().unwrap().push(actor);
                negative["body"]["value"]["atom"]["arguments"][0]["value"]["id"] = json!("other");
            }
            "other_action" => {
                negative["body"]["value"]["atom"]["arguments"][1]["value"]["id"] = json!("review");
                negative["body"]["value"]["atom"]["arguments"][1]["value"]["evidence"] =
                    json!([evidence(text, "revisó")]);
            }
            "other_setting" => {
                let mut setting = source["qualifiers"][0].clone();
                setting["id"] = json!("setting_b");
                setting["body"]["value"]["value"] = json!("B");
                setting["body"]["value"]["evidence"] = json!([evidence(text, "B")]);
                source["qualifiers"].as_array_mut().unwrap().push(setting);
                negative["qualifier_ids"] = json!(["duration", "setting_b"]);
            }
            _ => (),
        }
        source["claims"].as_array_mut().unwrap().push(negative);
        let result = complete_case(source, question, vocabulary, text);
        assert_eq!(
            result["complete_answer"],
            change != "same",
            "{change}: {result}"
        );
        if change == "same" {
            assert_eq!(result["selected_claim_ids"], json!([]), "{result}");
        }
    }
}

#[test]
fn configuration_slot_exact_basis_excludes_other_basis_before_uncertainty_collection() {
    let (mut source, question, vocabulary, text) = fixture();
    let mut second = source["claims"][0].clone();
    second["id"] = json!("different_basis");
    second["body"]["value"]["basis"] = json!("description");
    second["qualifier_ids"] = json!(["other_duration", "setting"]);
    let mut duration = source["qualifiers"][1].clone();
    duration["id"] = json!("other_duration");
    duration["body"]["value"]["decimal"] = json!("23");
    duration["body"]["value"]["evidence"] = json!([evidence(text, "23 minutos")]);
    source["claims"].as_array_mut().unwrap().push(second);
    source["qualifiers"].as_array_mut().unwrap().push(duration);
    let result = complete_case(source, question, vocabulary, text);
    assert_eq!(result["complete_answer"], true, "{result}");
    assert_eq!(result["selected_claim_ids"], json!(["c1"]));
    assert_eq!(
        result["requests"][0]["support"][0]["answer_terms"][0]["decimal"],
        "17"
    );
}

#[test]
fn configuration_slot_pending_negative_context_cannot_hide_compatible_negation() {
    let (mut source, question, vocabulary, text) = fixture();
    let mut setting = source["qualifiers"][0].clone();
    setting["id"] = json!("pending_setting");
    setting["resolution"] = json!("unknown");
    source["qualifiers"].as_array_mut().unwrap().push(setting);
    let mut negative = source["claims"][0].clone();
    negative["id"] = json!("pending_negative");
    negative["body"]["value"]["atom"]["polarity"] = json!("negative");
    negative["qualifier_ids"] = json!(["duration", "pending_setting"]);
    source["claims"].as_array_mut().unwrap().push(negative);
    let (sr, qr, result) = run(source, question, vocabulary, text);
    assert_eq!(sr["status"], "partial", "{sr}");
    assert_eq!(qr["status"], "complete", "{qr}");
    assert_eq!(result["complete_answer"], false, "{result}");
    assert_eq!(result["selected_claim_ids"], json!([]), "{result}");
    assert!(
        rows(&result["requests"][0], "unresolved").iter().any(|c| {
            rows(c, "reason_codes").contains(&json!("configuration_negative_pending"))
                && c["source_proposition"]["atom"]["polarity"] == "negative"
        }),
        "{result}"
    );
}

#[test]
fn configuration_slot_unrelated_pending_negative_does_not_veto_known_actor() {
    let (mut source, question, vocabulary, text) = fixture();
    let mut other = source["references"][0].clone();
    other["id"] = json!("other_actor");
    other["name"] = evidence(text, "Nerva");
    other["mention"] = evidence(text, "Nerva");
    source["references"].as_array_mut().unwrap().push(other);
    let mut setting = source["qualifiers"][0].clone();
    setting["id"] = json!("pending_setting");
    setting["resolution"] = json!("unknown");
    source["qualifiers"].as_array_mut().unwrap().push(setting);
    let mut negative = source["claims"][0].clone();
    negative["id"] = json!("unrelated_negative");
    negative["body"]["value"]["atom"]["polarity"] = json!("negative");
    negative["body"]["value"]["atom"]["arguments"][0]["value"]["id"] = json!("other_actor");
    negative["qualifier_ids"] = json!(["duration", "pending_setting"]);
    source["claims"].as_array_mut().unwrap().push(negative);
    let (sr, qr, result) = run(source, question, vocabulary, text);
    assert_eq!(sr["status"], "partial", "{sr}");
    assert_eq!(qr["status"], "complete", "{qr}");
    assert_eq!(result["complete_answer"], true, "{result}");
    assert_eq!(result["selected_claim_ids"], json!(["c1"]), "{result}");
    assert_eq!(
        result["requests"][0]["support"][0]["answer_terms"][0]["decimal"],
        "17"
    );
}
