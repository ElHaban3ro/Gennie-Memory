use super::*;
fn frozen(index: usize) -> Value {
    let bytes = match index {
        16 => include_bytes!(
            "../../../../tests/fixtures/documentary-v3-water-context-v1/receiver-class.json"
        )
        .as_slice(),
        24 => include_bytes!(
            "../../../../tests/fixtures/documentary-v3-water-context-v1/receiver-local-origin.json"
        )
        .as_slice(),
        93 => include_bytes!(
            "../../../../tests/fixtures/documentary-v3-water-context-v1/supply-local-operand.json"
        )
        .as_slice(),
        _ => panic!("unknown frozen fixture"),
    };
    serde_json::from_slice(bytes).unwrap()
}
fn classified(d: &Value, active: bool) -> Value {
    let a = super::super::super::validate(
        &d["source"]["raw"],
        s(&d["source"], "text"),
        &d["vocabulary"],
    );
    let b = super::super::super::validate(
        &d["question"]["raw"],
        s(&d["question"], "text"),
        &d["vocabulary"],
    );
    for result in [&a, &b] {
        let report = serde_json::to_value(result).unwrap();
        assert_eq!(report["status"], "complete", "{report}");
    }
    if active {
        super::super::super::classify_pair_context(&a, &b, &d["vocabulary"], &d["pair_scope"])
    } else {
        super::super::super::classify_pair(&a, &b, &d["vocabulary"], &d["pair_scope"])
    }
}
#[test]
fn water_operand_context_recovers_three_frozen_omissions_without_origin_answer() {
    for index in [16, 24, 93] {
        let d = frozen(index);
        let old = classified(&d, false);
        assert_eq!(old["selected_claim_ids"], json!([]), "{index}: {old}");
        let new = classified(&d, true);
        let witnesses: Vec<_> = rows(&new, "requests")
            .iter()
            .flat_map(|r| rows(r, "partial"))
            .filter(|w| s(w, "kind") == "partial_water_operand_context_v2")
            .collect();
        assert!(!witnesses.is_empty(), "{index}: {new}");
        assert_eq!(new["complete_answer"], false, "{index}: {new}");
        for w in witnesses {
            for key in ["answer_terms", "answer_propositions", "bindings"] {
                assert_eq!(w[key], json!([]), "{index}/{key}: {w}");
            }
            assert_eq!(w["context_transport"]["identity_proven"], false);
            assert_eq!(w["context_transport"]["requested_origin_inferred"], false);
            assert_eq!(w["source_scopes"], d["source"]["raw"]["scopes"]);
            assert!(!rows(w, "source_propositions").is_empty());
            for p in rows(w, "source_propositions") {
                assert!(
                    rows(&d["source"]["raw"], "claims")
                        .iter()
                        .any(|c| c["body"]["value"] == *p)
                );
            }
        }
    }
}
