//! Frozen-proposal reviewer screen. No new proposal, no integrated latency claim.
#[cfg(feature = "http2-transport")]
use super::*;
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic reviewer replacement:6knownqueries,12HTTP maximum,no retries"]
fn live_replacement_critic_screen() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let d = Path::new("outputs/iromem/evaluation/fast-critic-replacement-screen-v1-2026-10-20");
    let sha = |p: &Path| gennie_memory_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let prep: Value =
        serde_json::from_slice(&fs::read(d.join("preparation.json")).unwrap()).unwrap();
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
    }
    assert_eq!(prep["inputs_sha256"], sha(&d.join("inputs.jsonl")));
    assert_eq!(prep["requests_sha256"], sha(&d.join("requests.jsonl")));
    let requests = fs::read_to_string(d.join("requests.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(requests.len(), 12);
    let mut hashes = serde_json::Map::new();
    for p in [
        "bootstrap/src/openrouter.rs",
        "bootstrap/src/openrouter/per_memory/sparse_critic_probe.rs",
        "bootstrap/src/openrouter/per_memory/sparse_critic_probe/replacement.rs",
        "bootstrap/src/openrouter/per_memory/scoped_definition_probe.rs",
        "tools/evaluation/src/sparse_critic_probe.rs",
        "tools/evaluation/src/sparse_critic_probe/replacement.rs",
        "tools/evaluation/src/scoped_definition_probe.rs",
        "Cargo.toml",
        "Cargo.lock",
        "bootstrap/Cargo.toml",
    ] {
        hashes.insert(p.into(), json!(sha(Path::new(p))));
        let archive = d.join("source-executed").join(p);
        fs::create_dir_all(archive.parent().unwrap()).unwrap();
        fs::copy(p, archive).unwrap();
    }
    let pre = json!({"preparation_sha256":sha(&d.join("preparation.json")),"requests_sha256":prep["requests_sha256"],"sources_sha256":hashes,"http2_feature_enabled":true,"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"max_http_requests":12,"runtime_latency_measured":false});
    let mut pf = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("preflight.json"))
        .unwrap();
    pf.write_all(serde_json::to_string_pretty(&pre).unwrap().as_bytes())
        .unwrap();
    pf.sync_all().unwrap();
    let key = gennie_memory_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .unwrap();
    let client = OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("journal.jsonl"))
        .unwrap();
    let mut completed = 0;
    let mut stopped = false;
    for r in requests {
        let id = r["id"].as_str().unwrap();
        log(&mut f, json!({"kind":"started","id":id}));
        let start = Instant::now();
        let e = match client.request_measured(&key, &r["payload"]) {
            Ok((body, t)) => {
                if body["model"] != r["expected_served_model"]
                    || !super::super::scoped_definition_probe::role_schema_valid(
                        &body,
                        &r["payload"],
                    )
                {
                    stopped = true;
                }
                json!({"kind":"response","id":id,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"http_version":t.version,"body":body})
            }
            Err(e) => {
                stopped = true;
                json!({"kind":"failed","id":id,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"code":e.code,"http_status":e.http_status,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        log(&mut f, e);
        completed += 1;
        if stopped {
            break;
        }
    }
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("runner-status.json"))
        .unwrap();
    f.write_all(
        serde_json::to_string_pretty(
            &json!({"completed_http_requests":completed,"stopped":stopped}),
        )
        .unwrap()
        .as_bytes(),
    )
    .unwrap();
    f.sync_all().unwrap();
}
