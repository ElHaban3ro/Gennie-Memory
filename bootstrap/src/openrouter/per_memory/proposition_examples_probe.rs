//! Ignored research test, no runtime selector and no automatic retry.
#[path = "../../../../tools/evaluation/src/proposition_examples_contract.rs"]
#[cfg_attr(not(feature = "http2-transport"), allow(dead_code))]
mod contract;
use super::contribution_role_probe::contract as contribution_role_contract;
#[cfg(feature = "http2-transport")]
use super::proposition_probe::contract as base;
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic proposition frozen examples diagnostic,36HTTPmax,no retries"]
fn live_proposition_examples() {
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
    let d = Path::new("outputs/iromem/evaluation/proposition-examples-v3-2026-10-20");
    let sha = |p: &Path| iromem_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let read = |p: &Path| serde_json::from_slice::<Value>(&fs::read(p).unwrap()).unwrap();
    let lines = |p: &Path| {
        fs::read_to_string(p)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .collect::<Vec<_>>()
    };
    let prep = read(&d.join("preparation.json"));
    assert_eq!(prep["version"], "proposition-examples-v3-preparation");
    assert_eq!(prep["max_http_requests"], 36);
    assert_eq!(prep["model"], base::MODEL);
    assert_eq!(prep["revision"], base::REVISION);
    for name in ["inputs", "requests"] {
        assert_eq!(
            prep[format!("{name}_sha256")],
            sha(&d.join(format!("{name}.jsonl")))
        );
    }
    assert_eq!(
        prep["bank_manifest_sha256"],
        sha(&Path::new("tests/fixtures/proposition-contribution-v1").join("manifest.json"))
    );
    let cases = lines(&d.join("inputs.jsonl"));
    let requests = lines(&d.join("requests.jsonl"));
    assert_eq!(cases.len(), 18);
    assert_eq!(requests.len(), 36);
    for (n, r) in requests.iter().enumerate() {
        let ci = n / 2;
        let c = &cases[ci];
        let candidate = if ci % 2 == 0 { n % 2 == 1 } else { n % 2 == 0 };
        assert_eq!(r["case_index"], ci);
        assert_eq!(r["case_id"], c["id"]);
        assert_eq!(r["candidate"], candidate);
        assert_eq!(
            r["id"],
            format!(
                "{}__{}",
                c["id"].as_str().unwrap(),
                if candidate { "candidate" } else { "control" }
            )
        );
        assert_eq!(
            r["payload"],
            contract::payload(base::payload(c, RULES), candidate)
        );
    }
    assert_eq!(
        prep["parent_report_sha256"],
        sha(Path::new(prep["parent_report"].as_str().unwrap()))
    );
    let mut sources = serde_json::Map::new();
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
        sources.insert(p.clone(), h.clone());
    }
    for p in [
        "bootstrap/src/openrouter.rs",
        "bootstrap/src/openrouter/per_memory.rs",
        "bootstrap/src/openrouter/per_memory/sparse_critic_probe.rs",
        "bootstrap/src/openrouter/per_memory/proposition_probe.rs",
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
        json!({"preparation_sha256":sha(&d.join("preparation.json")),"sources_sha256":sources,"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"max_http_requests":36,"cfg_test_only":true,"http2_feature_enabled":true}),
    );
    let key = iromem_kernel::config::load_secrets()
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
    for r in &requests {
        let c = &cases[r["case_index"].as_u64().unwrap() as usize];
        log(&mut f, json!({"kind":"started","id":r["id"]}));
        let start = Instant::now();
        let e = match client.request_measured(&key, &r["payload"]) {
            Ok((body, t)) => {
                let validation = base::decode(&body, &r["payload"], c);
                stopped = validation.is_err();
                json!({"kind":"response","id":r["id"],"body":body,"http_version":t.version,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"validation_error":validation.err()})
            }
            Err(e) => {
                stopped = true;
                json!({"kind":"failed","id":r["id"],"code":e.code,"http_status":e.http_status,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
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
