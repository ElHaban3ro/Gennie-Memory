//! Diagnostic only: identical question fields, fewer sibling questions.
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic isolation diagnostic13HTTPmax,no retries,no adoption"]
fn live_local_isolation_probe() {
    use super::{contribution_role_probe::contract, sparse_critic_probe::log};
    use crate::openrouter::OpenRouterClassifier;
    use serde_json::{Value, json};
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        path::Path,
        time::Instant,
    };
    std::env::set_current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()).unwrap();
    let d = Path::new("outputs/iromem/evaluation/local-isolation-diagnostic-v1-2026-10-20");
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
    let rows = lines(&d.join("inputs.jsonl"));
    let requests = lines(&d.join("requests.jsonl"));
    assert_eq!(rows.len(), 4);
    assert_eq!(requests.len(), 13);
    assert_eq!(prep["inputs_sha256"], sha(&d.join("inputs.jsonl")));
    assert_eq!(prep["requests_sha256"], sha(&d.join("requests.jsonl")));
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
        "bootstrap/src/openrouter/per_memory/contribution_role_probe.rs",
        "bootstrap/src/openrouter/per_memory/scoped_definition_probe.rs",
        "Cargo.toml",
        "Cargo.lock",
        "bootstrap/Cargo.toml",
    ] {
        let dst = d.join("source-executed").join(p);
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(p, dst).unwrap();
        hashes.insert(p.into(), json!(sha(Path::new(p))));
    }
    let mut pf = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("preflight.json"))
        .unwrap();
    pf.write_all(serde_json::to_string_pretty(&json!({"preparation_sha256":sha(&d.join("preparation.json")),"sources_sha256":hashes,"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"max_http_requests":13,"http2_feature_enabled":true,"adoption_gate":false})).unwrap().as_bytes()).unwrap();
    pf.sync_all().unwrap();
    let key = iromem_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .unwrap();
    let client = OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("journal.jsonl"))
        .unwrap();
    let (mut completed, mut stopped) = (0, false);
    for r in requests {
        let row = rows.iter().find(|row| row["id"] == r["case_id"]).unwrap();
        let full = contract::payload_local(
            row["case"]["question"].as_str().unwrap(),
            row["case"]["memories"].as_array().unwrap(),
        );
        assert_eq!(full, row["payload"]);
        let mut expected = full;
        if r["mode"] == "single" {
            let key = r["question_key"].as_str().unwrap();
            expected["questions"]
                .as_object_mut()
                .unwrap()
                .retain(|k, _| k == key);
        }
        assert_eq!(expected, r["payload"]);
        let id = r["id"].as_str().unwrap();
        log(&mut f, json!({"kind":"started","id":id}));
        let start = Instant::now();
        let event = match client.request_measured(&key, &r["payload"]) {
            Ok((body, t)) => {
                stopped = body["model"] != contract::REVISION
                    || !super::scoped_definition_probe::role_schema_valid(&body, &r["payload"]);
                json!({"kind":"response","id":id,"body":body,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"http_version":t.version,"schema_valid":!stopped})
            }
            Err(e) => {
                stopped = true;
                json!({"kind":"failed","id":id,"code":e.code,"http_status":e.http_status,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        log(&mut f, event);
        completed += 1;
        if stopped {
            break;
        }
    }
    let mut status = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("runner-status.json"))
        .unwrap();
    status
        .write_all(
            serde_json::to_string_pretty(
                &json!({"completed_http_requests":completed,"stopped":stopped}),
            )
            .unwrap()
            .as_bytes(),
        )
        .unwrap();
    status.sync_all().unwrap();
}
