//! Manual synthetic capacity test; pure judgments are separate from transport and product.
#[path = "../../../../tools/evaluation/src/semantic_check_contract.rs"]
#[allow(dead_code)]
mod contract;
#[path = "../../../../tools/evaluation/src/semantic_presentation.rs"]
#[allow(dead_code)]
mod presentation;
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized isolated semantic representation pilot,46HTTPmaximum,no retries"]
fn live_semantic_representation_check_v1() {
    run(1);
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized known semantic presentation diagnostic,46HTTPmax,no retries"]
fn live_semantic_presentation_v2() {
    run(2);
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized V3 coordinate diagnostic,46HTTPmax,no retries"]
fn live_semantic_coordinates_v3() {
    run(3);
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized V4 schema diagnostic,46HTTPmax,no retries"]
fn live_semantic_schema_v4() {
    run(4);
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized V5 capacity diagnostic,46HTTPmax,no retries"]
fn live_semantic_capacity_v5() {
    run(5);
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized basis/scope diagnostic,46HTTPmax,no retries"]
fn live_semantic_basis_scope_v1() {
    run(6);
}
#[cfg(feature = "http2-transport")]
fn require_historical_frontier_probe_permission() -> Result<(), &'static str> {
    Err(
        "Usa subagentes Codex para estos LLM. Decisions/Jev y otras comparativas de clasificación siguen en sus runners existentes.",
    )
}
#[cfg(feature = "http2-transport")]
fn run(version: u8) {
    // User policy: frontier-model work now runs in Codex agents, not this paid HTTP probe.
    require_historical_frontier_probe_permission()
        .expect("piloto HTTP de LLM deshabilitado por política del usuario");
    use serde_json::{Value, json};
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        path::Path,
        time::{Duration, Instant},
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let d = Path::new(if version >= 2 {
        if version == 6 {
            "outputs/iromem/evaluation/semantic-basis-scope-v1-2026-10-21"
        } else if version == 5 {
            "outputs/iromem/evaluation/semantic-capacity-v5-2026-10-21"
        } else if version == 4 {
            "outputs/iromem/evaluation/semantic-schema-v4-2026-10-21"
        } else if version == 3 {
            "outputs/iromem/evaluation/semantic-coordinates-v3-2026-10-21"
        } else {
            "outputs/iromem/evaluation/semantic-presentation-v2-2026-10-21"
        }
    } else {
        "outputs/iromem/evaluation/semantic-representation-check-v1-2026-10-21"
    });
    let read = |p: &Path| serde_json::from_slice::<Value>(&fs::read(p).unwrap()).unwrap();
    let sha = |p: &Path| gennie_memory_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let save = |p: &Path, v: &Value| {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(p)
            .unwrap();
        f.write_all(&serde_json::to_vec_pretty(v).unwrap()).unwrap();
        f.sync_all().unwrap();
    };
    let prep = read(&d.join("preparation.json"));
    assert_eq!(
        prep["version"],
        if version >= 2 {
            if version == 6 {
                "semantic-basis-scope-v1-preparation"
            } else if version == 5 {
                "semantic-capacity-v5-preparation"
            } else if version == 4 {
                "semantic-schema-v4-preparation"
            } else if version == 3 {
                "semantic-coordinates-v3-preparation"
            } else {
                "semantic-presentation-v2-preparation"
            }
        } else {
            "semantic-representation-check-v1-preparation"
        }
    );
    let expected_model = if version >= 5 {
        presentation::CAPACITY_MODEL
    } else {
        contract::MODEL
    };
    assert_eq!(prep["model"], expected_model);
    assert_eq!(prep["max_http_requests"], 46);
    assert_eq!(prep["max_concurrent"], 4);
    assert_eq!(prep["requests_sha256"], sha(&d.join("requests.jsonl")));
    assert_eq!(
        prep["blind_packet_sha256"],
        sha(&d.join("blind-packet.json"))
    );
    let evaluator = root.join("target/debug/gennie-memory-eval");
    assert_eq!(prep["eval_binary_sha256"], sha(&evaluator));
    assert_eq!(
        prep["custodian_map_sha256"],
        sha(&d.join("custodian-map.json"))
    );
    let temp = std::env::temp_dir().join(format!(
        "gennie-memory-semantic-preflight-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(!temp.exists());
    let replay = std::process::Command::new(&evaluator)
        .arg(if version >= 2 {
            if version == 6 {
                "prepare-semantic-basis-scope"
            } else if version == 5 {
                "prepare-semantic-capacity"
            } else if version == 4 {
                "prepare-semantic-schema"
            } else if version == 3 {
                "prepare-semantic-coordinates"
            } else {
                "prepare-semantic-presentation"
            }
        } else {
            "prepare-semantic-check"
        })
        .arg(&temp)
        .output()
        .unwrap();
    assert!(
        replay.status.success(),
        "normalizer replay failed before HTTP"
    );
    let mut replay_files = vec![
        "preparation.json",
        "requests.jsonl",
        "blind-packet.json",
        "custodian-map.json",
    ];
    if version >= 2 {
        replay_files.extend([
            "frozen-labels.json",
            "blind-labels-a.json",
            "blind-labels-b.json",
            "label-reconciliation.json",
            "parent-bindings.json",
        ]);
    }
    for name in replay_files {
        assert_eq!(
            fs::read(temp.join(name)).unwrap(),
            fs::read(d.join(name)).unwrap(),
            "source/roster replay differs: {name}"
        );
    }
    fs::remove_dir_all(&temp).unwrap();
    let req = fs::read_to_string(d.join("requests.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(req.len(), 46);
    let packet = read(&d.join("blind-packet.json"));
    let items = packet["items"].as_array().unwrap();
    assert_eq!(items.len(), 46);
    let mut seen = std::collections::BTreeSet::new();
    for r in &req {
        assert!(seen.insert(r["id"].as_str().unwrap()));
        let base = contract::payload(&r["input"]).unwrap();
        let expected = if version == 6 {
            presentation::basis_scope_payload(&r["input"], &base).unwrap()
        } else if version == 5 {
            presentation::capacity_payload(&r["input"], &base).unwrap()
        } else if version == 4 {
            presentation::constrained_payload(&r["input"], &base).unwrap()
        } else if version >= 2 {
            if version == 3 {
                presentation::coordinate_payload(&r["input"], &base).unwrap()
            } else {
                presentation::payload(&r["input"], &base).unwrap()
            }
        } else {
            base
        };
        assert_eq!(r["payload"], expected);
        assert!(
            items
                .iter()
                .any(|v| v["id"] == r["id"] && v["input"] == r["input"])
        );
    }
    let labels = read(&d.join("frozen-labels.json"));
    assert_eq!(
        labels["blind_packet_sha256"],
        sha(&d.join("blind-packet.json"))
    );
    assert_eq!(labels["pending_adjudications"], 0);
    let expected = labels["items"].as_array().unwrap();
    assert_eq!(expected.len(), 46);
    let mut label_ids = std::collections::BTreeSet::new();
    for v in expected {
        let id = v["id"].as_str().unwrap();
        assert!(seen.contains(id) && label_ids.insert(id));
        for dim in contract::DIMENSIONS {
            assert!(["faithful", "defective", "uncertain"].contains(&v[dim].as_str().unwrap()));
        }
    }
    let catalog = read(&d.join("catalog-observation.json"));
    let route = catalog["response"]["data"]["endpoints"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| {
            e["tag"] == if version >= 5 { "anthropic" } else { "openai" }
                && e["model_id"] == expected_model
        })
        .unwrap();
    assert_eq!(route["status"], 0);
    let mut parameters = vec!["response_format", "max_tokens"];
    if version < 5 {
        parameters.push("temperature");
    }
    if version >= 4 {
        parameters.push("structured_outputs");
    }
    for p in parameters {
        assert!(
            route["supported_parameters"]
                .as_array()
                .unwrap()
                .contains(&json!(p))
        );
    }
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
        let target = d.join("source-executed").join(p);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        assert!(!target.exists());
        fs::copy(p, target).unwrap();
    }
    let reconciliation = read(&d.join("label-reconciliation.json"));
    assert_eq!(
        reconciliation["blind_packet_sha256"],
        sha(&d.join("blind-packet.json"))
    );
    assert_eq!(
        reconciliation["reviewer_a_sha256"],
        sha(&d.join("blind-labels-a.json"))
    );
    assert_eq!(
        reconciliation["reviewer_b_sha256"],
        sha(&d.join("blind-labels-b.json"))
    );
    assert_eq!(reconciliation["pending_adjudications"], 0);
    contract::check_labels(
        &packet,
        &read(&d.join("blind-labels-a.json")),
        &read(&d.join("blind-labels-b.json")),
        &read(&d.join("label-reconciliation.json")),
        &labels,
    )
    .expect("labels require full checked derivation before HTTP");
    let mut bindings = serde_json::Map::new();
    if version >= 2 {
        let parent = read(&d.join("parent-bindings.json"));
        assert_eq!(
            prep["parent_bindings_sha256"],
            sha(&d.join("parent-bindings.json"))
        );
        let original = Path::new(parent["parent_directory"].as_str().unwrap());
        for (name, h) in parent["files_sha256"].as_object().unwrap() {
            assert_eq!(*h, sha(&original.join(name)));
        }
        bindings.insert(
            "parent-bindings.json".into(),
            json!(sha(&d.join("parent-bindings.json"))),
        );
    }
    for name in [
        "preparation.json",
        "requests.jsonl",
        "blind-packet.json",
        "frozen-labels.json",
        "catalog-observation.json",
        "blind-labels-a.json",
        "blind-labels-b.json",
        "label-reconciliation.json",
        "custodian-map.json",
    ] {
        bindings.insert(name.into(), json!(sha(&d.join(name))));
    }
    save(
        &d.join("preflight.json"),
        &json!({"version":if version >= 2{if version == 6 { "semantic-basis-scope-v1-preflight" } else if version == 5 { "semantic-capacity-v5-preflight" } else if version == 4 { "semantic-schema-v4-preflight" } else if version == 3 { "semantic-coordinates-v3-preflight" } else { "semantic-presentation-v2-preflight" }}else{"semantic-representation-check-v1-preflight"},"inputs_sha256":bindings,"source_hashes":prep["source_hashes"],"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"max_http_requests":46,"all_sides_isolated":true,"labels_not_in_payload":true,"retry_count":0}),
    );
    let key = gennie_memory_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .expect("authorized existing key required");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let start = Instant::now();
    let (events, fatal) = super::independent_projection_probe::run_for_models(
        &client,
        &key,
        &req,
        &d.join("journal.jsonl"),
        &if version >= 5 {
            vec![json!(presentation::CAPACITY_MODEL)]
        } else {
            vec![json!(contract::MODEL), json!("openai/gpt-4.1-2025-04-14")]
        },
        Some(if version >= 5 {
            valid_capacity_response
        } else if version >= 3 {
            valid_coordinate_response
        } else {
            valid_response
        }),
    );
    save(
        &d.join("runner-status.json"),
        &json!({"attempted_http_requests":events.len(),"completed_http_requests":events.iter().filter(|e|e["kind"]=="response").count(),"terminal_events":events.len(),"transport_failures":events.iter().filter(|e|e["kind"]=="failed").count(),"stopped_on_fatal":fatal,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"max_concurrent":4,"retry_count":0}),
    );
}

fn valid_capacity_response(r: &serde_json::Value, e: &serde_json::Value) -> bool {
    let Some(content) = e["body"]["choices"][0]["message"]["content"].as_str() else {
        return false;
    };
    let Ok(canonical) = presentation::canonical_judgments(&r["input"], content) else {
        return false;
    };
    let mut derived = e.clone();
    derived["body"]["choices"][0]["message"]["content"] = serde_json::json!(canonical.to_string());
    valid_response_for_provider(r, &derived, presentation::CAPACITY_PROVIDER)
}
fn valid_coordinate_response(r: &serde_json::Value, e: &serde_json::Value) -> bool {
    let Some(content) = e["body"]["choices"][0]["message"]["content"].as_str() else {
        return false;
    };
    let Ok(canonical) = presentation::canonical_judgments(&r["input"], content) else {
        return false;
    };
    let mut derived = e.clone();
    derived["body"]["choices"][0]["message"]["content"] = serde_json::json!(canonical.to_string());
    valid_response(r, &derived)
}
fn valid_response(r: &serde_json::Value, e: &serde_json::Value) -> bool {
    valid_response_for_provider(r, e, "OpenAI")
}
fn valid_response_for_provider(
    r: &serde_json::Value,
    e: &serde_json::Value,
    provider: &str,
) -> bool {
    let b = &e["body"];
    b["provider"] == provider
        && b["choices"].as_array().is_some_and(|a| a.len() == 1)
        && b["choices"][0]["finish_reason"] == "stop"
        && b["choices"][0]["message"]
            .get("refusal")
            .is_none_or(|v| v.is_null())
        && b["usage"]["cost"]
            .as_f64()
            .is_some_and(|n| n.is_finite() && n >= 0.)
        && b["usage"]["prompt_tokens"].as_u64().is_some()
        && b["usage"]["completion_tokens"].as_u64().is_some()
        && b["choices"][0]["message"]["content"]
            .as_str()
            .is_some_and(|s| contract::checked(&r["input"], s).is_ok())
}
#[test]
fn terminal_response_requires_valid_judgments_receipt_provider_and_complete_output() {
    use serde_json::json;
    let input = json!({"kind":"source","text":"verde","raw":{},"normalized":{},"normalization_error":null,"ontology":{},"registry":{}});
    let mut judgments = json!({});
    for d in contract::DIMENSIONS {
        judgments[d] = json!({"label":"faithful","reason":"preserved","findings":[]});
    }
    let r = json!({"input":input});
    let e = json!({"body":{"provider":"OpenAI","choices":[{"finish_reason":"stop","message":{"content":judgments.to_string()}}],"usage":{"cost":0.1,"prompt_tokens":1,"completion_tokens":1}}});
    assert!(valid_response(&r, &e));
    for (path, bad) in [
        ("/body/provider", json!("other")),
        ("/body/usage/cost", json!(null)),
        ("/body/choices/0/finish_reason", json!("length")),
        ("/body/choices/0/message/content", json!("{}")),
    ] {
        let mut x = e.clone();
        *x.pointer_mut(path).unwrap() = bad;
        assert!(!valid_response(&r, &x));
    }
}
#[test]
fn capacity_terminal_does_not_accept_crossed_provider_or_repair_paths() {
    use serde_json::json;
    let r = json!({"input":{"kind":"source","text":"verde","raw":{"value":true},"normalized":{"value":true},"normalization_error":null,"ontology":{},"registry":{}}});
    let mut j = json!({});
    for d in contract::DIMENSIONS {
        j[d] = json!({"label":"faithful","reason":"test","findings":[]});
    }
    let mut e = json!({"body":{"provider":"Anthropic","choices":[{"finish_reason":"stop","message":{"content":j.to_string()}}],"usage":{"cost":0.1,"prompt_tokens":1,"completion_tokens":1}}});
    assert!(valid_capacity_response(&r, &e));
    assert!(!valid_coordinate_response(&r, &e));
    e["body"]["provider"] = json!("OpenAI");
    assert!(!valid_capacity_response(&r, &e));
    e["body"]["provider"] = json!("Anthropic");
    j["fidelity"]["findings"] = json!([{"quote":"verde","raw_paths":["/value"],"normalized_paths":[],"explanation":"wrong namespace"}]);
    e["body"]["choices"][0]["message"]["content"] = json!(j.to_string());
    assert!(!valid_capacity_response(&r, &e));
}
