//! Manual, bounded synthetic configuration contrast; no production policy.
use super::contribution_role_probe::contract;
use serde_json::{Value, json};
use std::{fs, fs::OpenOptions, path::Path, time::Instant};
#[test]
#[ignore = "authorized paired Qwen9B classification32HTTP,maxUSD0.03,300s,no retries"]
fn live_qwen_reasoning_pair() {
    run_pair("outputs/iromem/evaluation/qwen9b-reasoning-pair-v1-2026-10-22");
}
#[test]
#[ignore = "authorized total-deadline Qwen9B pair32HTTP,maxUSD0.03,300s,no retries"]
fn live_qwen_reasoning_total_deadline_pair() {
    run_pair("outputs/iromem/evaluation/qwen9b-reasoning-total-deadline-v2-2026-10-22");
}
fn run_pair(directory: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let d = Path::new(directory);
    // Never overwrite the source snapshot of an already-started attempt.
    for artifact in ["preflight.json", "journal.jsonl", "source-executed"] {
        assert!(
            !d.join(artifact).exists(),
            "attempt already started: {artifact}"
        );
    }
    let read = |p: &Path| serde_json::from_slice::<Value>(&fs::read(p).unwrap()).unwrap();
    let sha = |p: &Path| gennie_memory_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let lines = |p: &Path| {
        fs::read_to_string(p)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .collect::<Vec<_>>()
    };
    let prep = read(&d.join("preparation.json"));
    let requests = lines(&d.join("requests.jsonl"));
    let cases = lines(&d.join("inputs.jsonl"));
    assert_eq!(prep["version"], "qwen_reasoning_pair_v1");
    assert_eq!(requests.len(), 32);
    assert_eq!(cases.len(), 16);
    assert_eq!(prep["requests_sha256"], sha(&d.join("requests.jsonl")));
    assert_eq!(prep["inputs_sha256"], sha(&d.join("inputs.jsonl")));
    let estimated = requests
        .iter()
        .map(|r| contract::reasoning::estimated_cost(&r["payload"]))
        .sum::<f64>();
    assert!(estimated <= contract::reasoning::MAX_COST);
    let mut hashes = serde_json::Map::new();
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
        let dst = d.join("source-executed").join(p);
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(p, dst).unwrap();
        hashes.insert(p.clone(), h.clone());
    }
    for p in [
        "bootstrap/src/openrouter.rs",
        "bootstrap/src/openrouter/per_memory.rs",
        "bootstrap/src/openrouter/per_memory/sparse_critic_probe.rs",
        "Cargo.toml",
        "Cargo.lock",
        "bootstrap/Cargo.toml",
    ] {
        let dst = d.join("source-executed").join(p);
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(p, dst).unwrap();
        hashes.insert(p.into(), json!(sha(Path::new(p))));
    }
    let pre = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("preflight.json"))
        .unwrap();
    serde_json::to_writer_pretty(&pre,&json!({"preparation_sha256":sha(&d.join("preparation.json")),"source_hashes":hashes,"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"estimated_cost_usd":estimated,"max_known_cost_usd":contract::reasoning::MAX_COST,"max_http_requests":32,"network_window_seconds":300,"reserved_timeout_seconds":20,"cfg_test_only":true})).unwrap();
    pre.sync_all().unwrap();
    let key = gennie_memory_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .unwrap();
    let client = crate::openrouter::OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut journal = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("journal.jsonl"))
        .unwrap();
    let campaign = Instant::now();
    let (mut cost, mut completed, mut stopped) = (0., 0, false);
    let mut reason = "completed";
    for (i, r) in requests.iter().enumerate() {
        let c = &cases[i / 2];
        let enabled = r["enabled"].as_bool().unwrap();
        let arm = if enabled { "on" } else { "off" };
        assert_eq!(r["case_id"], c["id"]);
        assert_eq!(r["id"], format!("{}__{arm}", c["id"].as_str().unwrap()));
        let base = contract::Family::Qwen.payload(
            contract::Profile::Local,
            c["question"].as_str().unwrap(),
            c["memories"].as_array().unwrap(),
        );
        assert_eq!(r["payload"], contract::reasoning::payload(&base, enabled));
        assert_eq!(
            enabled,
            if (i / 2) % 2 == 0 {
                i % 2 == 1
            } else {
                i % 2 == 0
            }
        );
        if campaign.elapsed().as_secs_f64() + 20. > 300. {
            stopped = true;
            reason = "window";
            break;
        }
        if cost + contract::reasoning::estimated_cost(&r["payload"]) > contract::reasoning::MAX_COST
        {
            stopped = true;
            reason = "budget";
            break;
        }
        let id = r["id"].as_str().unwrap();
        super::sparse_critic_probe::log(&mut journal, json!({"kind":"started","id":id}));
        let start = Instant::now();
        let error_path = d.join(format!("{id}-http-error.json"));
        let response = super::contribution_role_probe::request_generative_observed(
            &client,
            &key,
            &r["payload"],
            Some(&error_path),
        );
        let event = match response {
            Ok((body, t)) => {
                let checked = contract::reasoning::decode(
                    &body,
                    &r["payload"],
                    c["memories"].as_array().unwrap(),
                    enabled,
                );
                stopped = checked.is_err();
                if stopped {
                    reason = "invalid_response_or_receipt";
                }
                json!({"kind":"response","id":id,"body":body,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"http_version":t.version,"validation_error":checked.err()})
            }
            Err(e) => {
                stopped = true;
                reason = "http_or_transport_failure";
                json!({"kind":"failed","id":id,"code":e.code,"http_status":e.http_status,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        if let Some(c) = event["body"]["usage"]["cost"]
            .as_f64()
            .filter(|c| c.is_finite() && *c >= 0.)
        {
            cost += c;
            if cost > contract::reasoning::MAX_COST {
                stopped = true;
                reason = "budget_exceeded";
            }
        } else if !stopped {
            stopped = true;
            reason = "missing_cost";
        }
        super::sparse_critic_probe::log(&mut journal, event);
        completed += 1;
        if stopped {
            break;
        }
    }
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("runner-status.json"))
        .unwrap();
    serde_json::to_writer_pretty(&file,&json!({"completed_http_requests":completed,"stopped":stopped,"reason":reason,"known_response_cost_usd":cost,"elapsed_seconds":campaign.elapsed().as_secs_f64()})).unwrap();
    file.sync_all().unwrap();
}
