//! Ignored research test, no runtime selector and no automatic retry.
#[path = "../../../../tools/evaluation/src/proposition_contract.rs"]
#[cfg_attr(not(feature = "http2-transport"), allow(dead_code))]
pub(super) mod contract;
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic manual-proposition diagnostic,18HTTPmax,no retries"]
fn live_proposition_contribution() {
    run(
        "outputs/iromem/evaluation/proposition-contribution-v1-2026-10-20",
        false,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic Decisions proposition capacity18,18HTTPmax,no retries"]
fn live_proposition_decisions() {
    run(
        "outputs/iromem/evaluation/proposition-contribution-decisions-v1-2026-10-20",
        true,
    );
}
#[cfg(feature = "http2-transport")]
fn run(directory: &str, decisions: bool) {
    use super::{contribution_role_probe::contract::RULES, sparse_critic_probe::log};
    use crate::openrouter::OpenRouterClassifier;
    use serde_json::{Value, json};
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        path::Path,
        time::Instant,
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let d = Path::new(directory);
    let sha = |p: &Path| gennie_memory_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let read = |p: &Path| serde_json::from_slice::<Value>(&fs::read(p).unwrap()).unwrap();
    let lines = |p: &Path| {
        fs::read_to_string(p)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .collect::<Vec<_>>()
    };
    let prep = read(&d.join("preparation.json"));
    assert_eq!(prep["version"], "proposition-contribution-v1-preparation");
    assert_eq!(prep["max_http_requests"], 18);
    assert_eq!(
        prep["model"],
        if decisions {
            contract::DECISIONS_MODEL
        } else {
            contract::MODEL
        }
    );
    if decisions {
        assert_eq!(prep["checker_family"], "decisions");
    } else {
        assert!(prep.get("checker_family").is_none());
    }
    assert_eq!(
        prep["revision"],
        if decisions {
            contract::DECISIONS_REVISION
        } else {
            contract::REVISION
        }
    );
    for name in ["inputs", "requests"] {
        assert_eq!(
            prep[format!("{name}_sha256")],
            sha(&d.join(format!("{name}.jsonl")))
        );
    }
    assert_eq!(
        prep["bank_manifest_sha256"],
        sha(&Path::new(prep["bank"].as_str().unwrap()).join("manifest.json"))
    );
    let cases = lines(&d.join("inputs.jsonl"));
    let requests = lines(&d.join("requests.jsonl"));
    assert_eq!(cases.len(), 18);
    assert_eq!(requests.len(), 18);
    for (c, r) in cases.iter().zip(&requests) {
        assert_eq!(c["id"], r["id"]);
        assert_eq!(
            r["payload"],
            if decisions {
                contract::payload_decisions(c, RULES)
            } else {
                contract::payload(c, RULES)
            }
        );
    }
    let mut sources = serde_json::Map::new();
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
        sources.insert(p.clone(), h.clone());
    }
    for p in [
        "bootstrap/src/openrouter.rs",
        "bootstrap/src/openrouter/per_memory.rs",
        "bootstrap/src/openrouter/per_memory/sparse_critic_probe.rs",
        "Cargo.toml",
        "Cargo.lock",
        "bootstrap/Cargo.toml",
    ] {
        sources.insert(p.into(), json!(sha(Path::new(p))));
    }
    for (p, _) in &sources {
        let dst = d.join("source-executed").join(p);
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(p, dst).unwrap();
    }
    let write = |name: &str, v: Value| {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(d.join(name))
            .unwrap();
        f.write_all(serde_json::to_string_pretty(&v).unwrap().as_bytes())
            .unwrap();
        f.sync_all().unwrap();
    };
    write(
        "preflight.json",
        json!({"preparation_sha256":sha(&d.join("preparation.json")),"sources_sha256":sources,"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"max_http_requests":18,"cfg_test_only":true,"http2_feature_enabled":true}),
    );
    let key = gennie_memory_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .unwrap();
    let client = OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("journal.jsonl"))
        .unwrap();
    let (mut completed, mut stopped) = (0, false);
    for (c, r) in cases.iter().zip(&requests) {
        log(&mut f, json!({"kind":"started","id":c["id"]}));
        let start = Instant::now();
        let e = match client.request_measured(&key, &r["payload"]) {
            Ok((body, t)) => {
                let validation = if decisions {
                    contract::decode_decisions(&body, &r["payload"], c)
                } else {
                    contract::decode(&body, &r["payload"], c)
                };
                stopped = validation.is_err();
                json!({"kind":"response","id":c["id"],"body":body,"http_version":t.version,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"validation_error":validation.err()})
            }
            Err(e) => {
                stopped = true;
                json!({"kind":"failed","id":c["id"],"code":e.code,"http_status":e.http_status,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        log(&mut f, e);
        completed += 1;
        if stopped {
            break;
        }
    }
    write(
        "runner-status.json",
        json!({"completed_http_requests":completed,"stopped":stopped}),
    );
}
