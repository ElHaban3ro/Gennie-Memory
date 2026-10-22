use super::*;
const BASE: &str = "documentary-nominal-possessive-v1";
fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(BASE)
        .join(format!("{name}.json"));
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn text(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(BASE)
            .join(format!("{name}.txt")),
    )
    .unwrap()
}
fn canonical_hash(v: &Value) -> String {
    hex_digest(&Sha256::digest(serde_json::to_vec(v).unwrap()))
}
fn delivery() -> Value {
    json!({"version":DELIVERY,"origin_id":"occurrence-a","delivery_id":"logical-delivery-a",
        "selection":{"request_id":"ask","claim_id":"fact"},
        "pair":{"vocabulary":fixture("vocabulary"),"source":{"text":text("source"),"raw":fixture("source")},
        "question":{"text":text("question"),"raw":fixture("question")},
        "pair_scope":{"namespace":"possessive-controls","configuration":[],"time_selector":null,"closed_world":false}}})
}
fn manifest(deliveries: &[Value]) -> Value {
    let mut origins = vec![];
    let mut rows = vec![];
    for d in deliveries {
        let p = &d["pair"];
        let s = json!({"vocabulary":p["vocabulary"],"text":p["source"]["text"],"raw":p["source"]["raw"]});
        if !origins
            .iter()
            .any(|o: &Value| o["origin_id"] == d["origin_id"])
        {
            origins
                .push(json!({"origin_id":d["origin_id"],"source_input_sha256":canonical_hash(&s)}));
        }
        rows.push(json!({"delivery_id":d["delivery_id"],"origin_id":d["origin_id"],"delivery_input_sha256":canonical_hash(d)}));
    }
    json!({"version":MANIFEST,"origins":origins,"deliveries":rows})
}
fn consume(d: &Value, r: &Value) -> Value {
    json!({"version":CONSUME,"delivery":d,"reference":r})
}
fn complete(d: &Value) {
    for side in ["source", "question"] {
        let p = &d["pair"];
        let report =
            serde_json::to_value(super::super::side(&p[side], &p["vocabulary"]).unwrap()).unwrap();
        assert_eq!(report["status"], "complete", "{report}");
    }
}
#[test]
fn bind_and_consume_preserve_full_occurrence_and_logical_replay() {
    let d = delivery();
    complete(&d);
    let m = manifest(&[d.clone()]);
    let r = execute("bind", &d, &m, 0).unwrap();
    assert_eq!(r["witness"]["source_document"], d["pair"]["source"]["raw"]);
    assert_eq!(
        r["witness"]["source_constraint"],
        d["pair"]["source"]["raw"]["claims"][0]
    );
    assert_eq!(
        r["witness"]["possessive_context"]["referent_resolution"],
        "source_indeterminate"
    );
    assert!(
        r["witness"]["missing_constraints"]
            .as_array()
            .unwrap()
            .contains(&json!("nominal_possessive_referent"))
    );
    assert_eq!(r, execute("bind", &d, &m, 0).unwrap());
    assert_eq!(r, execute("consume", &consume(&d, &r), &m, 0).unwrap());
    for key in ["answer_terms", "answer_propositions", "bindings"] {
        assert_eq!(r[key], json!([]));
    }
}
#[test]
fn multiple_questions_and_identical_text_origins_do_not_collapse_occurrences() {
    let a = delivery();
    let mut b = a.clone();
    b["delivery_id"] = json!("logical-delivery-b");
    let q = serde_json::to_string(&b["pair"]["question"]["raw"])
        .unwrap()
        .replace("\"ask\"", "\"ask-b\"");
    b["pair"]["question"]["raw"] = serde_json::from_str(&q).unwrap();
    b["selection"]["request_id"] = json!("ask-b");
    let mut c = a.clone();
    c["origin_id"] = json!("occurrence-b");
    c["delivery_id"] = json!("logical-delivery-c");
    complete(&b);
    complete(&c);
    let m = manifest(&[a.clone(), b.clone(), c.clone()]);
    let ra = execute("bind", &a, &m, 0).unwrap();
    let rb = execute("bind", &b, &m, 0).unwrap();
    let rc = execute("bind", &c, &m, 0).unwrap();
    assert_eq!(ra["source_input"], rb["source_input"]);
    assert_eq!(
        ra["witness"]["source_constraint"],
        rb["witness"]["source_constraint"]
    );
    assert_eq!(ra["origin_id"], rb["origin_id"]);
    assert_ne!(ra["delivery_id"], rb["delivery_id"]);
    assert_eq!(ra["source_input_sha256"], rc["source_input_sha256"]);
    assert_ne!(ra["origin_id"], rc["origin_id"]);
    assert!(execute("consume", &consume(&c, &ra), &m, 0).is_err());
    assert_eq!(rc, execute("consume", &consume(&c, &rc), &m, 0).unwrap());
}
#[test]
fn changed_requested_owner_is_complete_but_cannot_resolve_source_possessive() {
    let mut d = delivery();
    let qt = text("question").replace("Nora", "Mira");
    let q = serde_json::to_string(&d["pair"]["question"]["raw"])
        .unwrap()
        .replace("Nora", "Mira");
    d["pair"]["question"]["raw"] = serde_json::from_str(&q).unwrap();
    d["pair"]["question"]["text"] = json!(qt);
    d["pair"]["question"]["raw"]["text_sha256"] = json!(hex_digest(&Sha256::digest(qt.as_bytes())));
    complete(&d);
    let before = d["pair"]["source"].clone();
    assert!(
        execute("bind", &d, &manifest(&[d.clone()]), 0)
            .unwrap_err()
            .contains("testigo")
    );
    assert_eq!(d["pair"]["source"], before);
}
#[test]
fn reclassification_rejects_each_tampered_reference_component() {
    let d = delivery();
    let m = manifest(&[d.clone()]);
    let r = execute("bind", &d, &m, 0).unwrap();
    for pointer in [
        "/origin_id",
        "/delivery_id",
        "/manifest_sha256",
        "/source_input_sha256",
        "/delivery_input_sha256",
        "/selection/claim_id",
        "/selection/request_id",
        "/source_input/text",
        "/source_input/raw",
        "/source_input/vocabulary",
        "/witness/source_document",
        "/witness/source_constraint",
        "/witness/request_constraint",
        "/witness/question_document",
        "/witness/source_references",
        "/witness/source_scopes",
        "/witness/source_qualifiers",
        "/witness/source_dependencies",
        "/witness/question_references",
        "/witness/question_scopes",
        "/witness/question_qualifiers",
        "/witness/question_dependencies",
        "/witness/missing_constraints",
        "/witness/possessive_context/source_owners",
        "/witness/possessive_context/source_uses",
        "/witness/possessive_context/referent_resolution",
        "/answer_terms",
        "/answer_propositions",
        "/bindings",
        "/witness/answer_terms",
        "/witness/answer_propositions",
        "/witness/bindings",
        "/interpretation",
    ] {
        let mut altered = r.clone();
        *altered.pointer_mut(pointer).unwrap() = json!(["tampered"]);
        assert!(
            execute("consume", &consume(&d, &altered), &m, 0).is_err(),
            "{pointer}"
        );
    }
    let mut altered = r.clone();
    altered["unregistered_field"] = json!(true);
    assert!(execute("consume", &consume(&d, &altered), &m, 0).is_err());
    assert!(execute("consume", &r, &m, 0).is_err());
}
#[test]
fn closed_manifest_delivery_and_invalid_documents_reject_without_semantic_credit() {
    let d = delivery();
    let m = manifest(&[d.clone()]);
    for table in ["origins", "deliveries"] {
        let mut bad = m.clone();
        let row = bad[table][0].clone();
        bad[table].as_array_mut().unwrap().push(row);
        assert!(execute("bind", &d, &bad, 0).is_err());
        let mut bad = m.clone();
        bad[table][0]["extra"] = json!(true);
        assert!(execute("bind", &d, &bad, 0).is_err());
    }
    for pointer in [
        "/origins/0/origin_id",
        "/origins/0/source_input_sha256",
        "/deliveries/0/delivery_id",
        "/deliveries/0/origin_id",
        "/deliveries/0/delivery_input_sha256",
    ] {
        let mut bad = m.clone();
        *bad.pointer_mut(pointer).unwrap() = json!("unregistered");
        assert!(execute("bind", &d, &bad, 0).is_err());
    }
    let r = execute("bind", &d, &m, 0).unwrap();
    let mut changed = d.clone();
    changed["delivery_id"] = json!("other-delivery");
    let changed_manifest = manifest(&[d.clone(), changed]);
    assert!(execute("consume", &consume(&d, &r), &changed_manifest, 0).is_err());
    for pointer in [
        "/pair/source/raw",
        "/pair/question/raw",
        "/pair/source/raw/coverage",
        "/pair/question/raw/requests",
    ] {
        let mut bad = d.clone();
        *bad.pointer_mut(pointer).unwrap() = Value::Null;
        assert!(
            execute("bind", &bad, &manifest(&[bad.clone()]), 0).is_err(),
            "{pointer}"
        );
    }
    let mut bad = d.clone();
    bad["selection"]["claim_id"] = json!("other-claim");
    complete(&bad);
    assert!(execute("bind", &bad, &manifest(&[bad.clone()]), 0).is_err());
    let mut bad = d.clone();
    bad["extra"] = json!(true);
    assert!(execute("bind", &bad, &manifest(&[bad.clone()]), 0).is_err());
}
#[test]
fn aggregate_preflight_and_nested_budget_are_atomic() {
    let d = delivery();
    let m = manifest(&[d.clone()]);
    assert!(execute("bind", &d, &m, LIMIT + 1).is_err());
    let v = json!({"request_context_profile":{}});
    assert!(
        budget::with_operation(&v, &Value::Null, || {
            assert!(budget::charge(999_990, 0));
            execute("bind", &d, &m, 0)
        })
        .is_err()
    );
    for (a, b) in [
        (
            json!({"claims":vec![Value::Null;257]}),
            json!({"requests":vec![Value::Null;256]}),
        ),
        (
            json!({"nodes":vec![Value::Null;50_000]}),
            json!({"nodes":vec![Value::Null;50_000]}),
        ),
        (json!({"evidence":vec![Value::Null;33]}), Value::Null),
    ] {
        assert!(
            budget::with_operation(&v, &Value::Null, || {
                let mut counts = Counts::default();
                preflight(&a, 0, &mut counts)?;
                preflight(&b, 0, &mut counts)
            })
            .unwrap()
            .is_err()
        );
    }
    let mut deep = Value::Null;
    for _ in 0..65 {
        deep = json!([deep]);
    }
    assert!(
        budget::with_operation(&v, &Value::Null, || preflight(
            &deep,
            0,
            &mut Counts::default()
        ))
        .unwrap()
        .is_err()
    );
    let mut no_feature = d.clone();
    no_feature["pair"]["vocabulary"]
        .as_object_mut()
        .unwrap()
        .remove("request_context_profile");
    assert!(
        execute(
            "bind",
            &no_feature,
            &manifest(&[no_feature.clone()]),
            LIMIT + 1
        )
        .is_err()
    );
    assert!(execute("bind", &d, &m, 0).is_ok());
}
#[test]
fn file_transport_rejects_duplicates_limits_and_existing_output() {
    let dir = std::env::temp_dir().join(format!(
        "source-context-reference-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap();
    let input = dir.join("input.json");
    let trust = dir.join("manifest.json");
    let output = dir.join("output.json");
    let d = delivery();
    let m = manifest(&[d.clone()]);
    std::fs::write(&input, serde_json::to_vec(&d).unwrap()).unwrap();
    std::fs::write(&trust, serde_json::to_vec(&m).unwrap()).unwrap();
    run("bind", &input, &trust, &output).unwrap();
    let r: Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    std::fs::write(&input, serde_json::to_vec(&consume(&d, &r)).unwrap()).unwrap();
    let verified = dir.join("verified.json");
    run("consume", &input, &trust, &verified).unwrap();
    assert_eq!(
        std::fs::read(&output).unwrap(),
        std::fs::read(&verified).unwrap()
    );
    let before = std::fs::read(&output).unwrap();
    assert!(run("consume", &input, &trust, &output).is_err());
    assert_eq!(before, std::fs::read(&output).unwrap());
    let rejected = dir.join("rejected.json");
    for duplicate in [
        br#"{"version":1,"version":2}"#.as_slice(),
        br#"{"nested":{"x":1,"x":2}}"#,
    ] {
        std::fs::write(&input, duplicate).unwrap();
        assert!(run("bind", &input, &trust, &rejected).is_err());
        assert!(!rejected.exists());
    }
    std::fs::write(&input, serde_json::to_vec(&d).unwrap()).unwrap();
    std::fs::write(&trust, br#"{"origins":[{"x":1,"x":2}]}"#).unwrap();
    assert!(run("bind", &input, &trust, &rejected).is_err());
    assert!(!rejected.exists());
    let file = std::fs::File::create(&trust).unwrap();
    file.set_len(LIMIT as u64).unwrap();
    assert!(run("bind", &input, &trust, &rejected).is_err());
    assert!(!rejected.exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn v2_preserves_legacy_context_and_enforces_version_separation() {
    let old = delivery();
    let old_manifest = manifest(&[old.clone()]);
    let old_reference = execute("bind", &old, &old_manifest, 0).unwrap();
    let mut d = old.clone();
    d["version"] = json!(DELIVERY_V2);
    let m = manifest(&[d.clone()]);
    let r = execute_protocol(Protocol::V2, "bind", &d, &m, 0).unwrap();
    assert_eq!(r["version"], REFERENCE_V2);
    assert_eq!(r["witness"], old_reference["witness"]);
    assert_eq!(r["source_input"], old_reference["source_input"]);
    let c = json!({"version":CONSUME_V2,"delivery":d,"reference":r});
    assert_eq!(
        execute_protocol(Protocol::V2, "consume", &c, &m, 0).unwrap(),
        r
    );
    assert!(execute("bind", &d, &m, 0).is_err());
    assert!(execute_protocol(Protocol::V2, "bind", &old, &old_manifest, 0).is_err());
    assert!(execute("consume", &c, &m, 0).is_err());
    let mut mixed = c.clone();
    mixed["reference"] = old_reference;
    assert!(execute_protocol(Protocol::V2, "consume", &mixed, &m, 0).is_err());
    let mut changed = c;
    changed["reference"]["witness"]["possessive_context"]["possessive_referent_proven"] =
        json!(true);
    assert!(execute_protocol(Protocol::V2, "consume", &changed, &m, 0).is_err());
    assert_eq!(
        execute("bind", &old, &old_manifest, 0).unwrap()["version"],
        REFERENCE
    );
}

// Isolate the closed transport schema; end-to-end tests additionally require the
// actual selector to generate this witness from fully validated feature-12 input.
fn nominal_schema_fixture() -> (Value, Value) {
    let d = delivery();
    let mut pair = d["pair"].clone();
    pair["vocabulary"]["request_context_profile"]["features"]
        .as_array_mut()
        .unwrap()
        .push(json!("nonagentive_nominal_event_with_contexts_v1"));
    pair["vocabulary"]["fixed_action_rows"] = json!(["remove"]);
    let body = json!({"kind":"linguistic_composition", "constructor":"nonagentive_nominal_event_with_contexts_v1",
        "composition_id":"event", "core":{"id":"core"}, "region":{"id":"region"},
        "contexts":[{"id":"pp-a"},{"id":"pp-b"}]});
    pair["source"]["raw"]["claims"][0]["body"] = body.clone();
    let mut witness = json!({});
    for key in [
        "id",
        "kind",
        "claim_ids",
        "request_id",
        "bindings",
        "evidence_refs",
        "qualifier_ids",
        "branch_ids",
        "missing_constraints",
        "answer_terms",
        "answer_propositions",
        "requirement_states",
        "conclusion_judgment",
        "normative_scope",
        "source_constraint",
        "source_body",
        "request_constraint",
        "source_document",
        "question_document",
        "vocabulary",
        "source_references",
        "question_references",
        "source_scopes",
        "question_scopes",
        "source_qualifiers",
        "question_qualifiers",
        "source_dependencies",
        "question_dependencies",
        "effective_source_closure",
        "effective_request_closure",
        "nominal_event_context",
    ] {
        witness[key] = Value::Null;
    }
    witness["kind"] = json!("source_bound_linguistic_context");
    witness["vocabulary"] = pair["vocabulary"].clone();
    witness["source_document"] = pair["source"]["raw"].clone();
    witness["question_document"] = pair["question"]["raw"].clone();
    witness["source_constraint"] = pair["source"]["raw"]["claims"][0].clone();
    witness["request_constraint"] = pair["question"]["raw"]["requests"][0].clone();
    witness["source_body"] = body;
    for key in [
        "bindings",
        "answer_terms",
        "answer_propositions",
        "requirement_states",
    ] {
        witness[key] = json!([]);
    }
    witness["missing_constraints"] = json!([
        "answer_slot",
        "patient_identity",
        "context_attachment",
        "requested_role",
        "additional_debt"
    ]);
    witness["nominal_event_context"] = json!({"version":"documentary_nominal_event_context_v1",
        "constructor":"nonagentive_nominal_event_with_contexts_v1", "composition_id":"event", "core_id":"core",
        "region_id":"region", "context_ids":["pp-a","pp-b"], "attachment_resolution":"not_published",
        "patient_identity_proven":false, "requested_role_proven":false, "core_independently_asserted":false,
        "full_answer_supported":false});
    for (side, kind, table, field) in [
        ("source", "claim", "claims", "effective_source_closure"),
        (
            "question",
            "request",
            "requests",
            "effective_request_closure",
        ),
    ] {
        let doc = &pair[side]["raw"];
        let owner = doc[table][0]["id"].as_str().unwrap();
        witness[field] = json!(
            documentary_v3::lexical_clause_context::closure(doc, kind, owner)
                .into_iter()
                .map(|(kind, id)| json!({"kind":kind,"id":id}))
                .collect::<Vec<_>>()
        );
    }
    (witness, pair)
}

#[test]
fn nominal_transport_schema_preserves_debts_flags_body_and_real_closures() {
    let (w, pair) = nominal_schema_fixture();
    assert!(nominal_event_witness(&w, &pair).is_ok());
    for pointer in [
        "/source_body",
        "/vocabulary",
        "/effective_source_closure",
        "/effective_request_closure",
        "/nominal_event_context/context_ids",
        "/nominal_event_context/core_id",
        "/nominal_event_context/region_id",
        "/nominal_event_context/composition_id",
        "/nominal_event_context/constructor",
        "/nominal_event_context/version",
        "/nominal_event_context/attachment_resolution",
        "/nominal_event_context/patient_identity_proven",
        "/nominal_event_context/requested_role_proven",
        "/nominal_event_context/core_independently_asserted",
        "/nominal_event_context/full_answer_supported",
        "/requirement_states",
    ] {
        let mut bad = w.clone();
        *bad.pointer_mut(pointer).unwrap() = json!(["altered"]);
        assert!(nominal_event_witness(&bad, &pair).is_err(), "{pointer}");
    }
    for required in [
        "answer_slot",
        "patient_identity",
        "context_attachment",
        "requested_role",
    ] {
        let mut bad = w.clone();
        bad["missing_constraints"]
            .as_array_mut()
            .unwrap()
            .retain(|v| v != required);
        assert!(nominal_event_witness(&bad, &pair).is_err(), "{required}");
    }
    let mut bad = w.clone();
    bad["nominal_event_context"]["resolved_attachment"] = json!("beneficiary");
    assert!(nominal_event_witness(&bad, &pair).is_err());
    let mut bad = w.clone();
    bad["extra"] = json!(true);
    assert!(nominal_event_witness(&bad, &pair).is_err());
    let mut forbidden = pair.clone();
    forbidden["vocabulary"]["request_context_profile"]["features"] =
        json!(["nominal_complement_de"]);
    assert!(nominal_event_witness(&w, &forbidden).is_err());
}

#[test]
fn v2_file_transport_rejects_mutations_atomically_and_preserves_existing_output() {
    let dir = std::env::temp_dir().join(format!(
        "source-context-v2-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap();
    let input = dir.join("input.json");
    let trust = dir.join("trust.json");
    let output = dir.join("output.json");
    let mut d = delivery();
    d["version"] = json!(DELIVERY_V2);
    std::fs::write(&input, serde_json::to_vec(&d).unwrap()).unwrap();
    std::fs::write(&trust, serde_json::to_vec(&manifest(&[d.clone()])).unwrap()).unwrap();
    run_v2("bind", &input, &trust, &output).unwrap();
    let original = std::fs::read(&output).unwrap();
    let r: Value = serde_json::from_slice(&original).unwrap();
    let mut c = json!({"version":CONSUME_V2,"delivery":d,"reference":r});
    c["reference"]["witness"]["answer_terms"] = json!(["invented"]);
    std::fs::write(&input, serde_json::to_vec(&c).unwrap()).unwrap();
    let rejected = dir.join("rejected.json");
    assert!(run_v2("consume", &input, &trust, &rejected).is_err());
    assert!(!rejected.exists());
    c["reference"] = r;
    std::fs::write(&input, serde_json::to_vec(&c).unwrap()).unwrap();
    assert!(run_v2("consume", &input, &trust, &output).is_err());
    assert_eq!(std::fs::read(&output).unwrap(), original);
    std::fs::write(&input, br#"{"version":1,"version":2}"#).unwrap();
    assert!(run_v2("bind", &input, &trust, &rejected).is_err());
    assert!(!rejected.exists());
    std::fs::remove_dir_all(dir).unwrap();
}
