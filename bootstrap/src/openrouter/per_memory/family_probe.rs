//! Model-only synthetic research, using the frozen role requests from evaluation.
use crate::openrouter::OpenRouterClassifier;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};
const DIR: &str = "outputs/iromem/evaluation/scoped-role-family-contrast-2026-10-20";
#[test]
#[ignore = "authorized synthetic family contrast:108HTTP maximum,18knownquestions,3models,no retries"]
fn live_scoped_family_probe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let d = Path::new(DIR);
    let sha = |p: &Path| gennie_memory_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let prep: Value =
        serde_json::from_slice(&fs::read(d.join("preparation.json")).unwrap()).unwrap();
    assert_eq!(prep["requests_sha256"], sha(&d.join("requests.jsonl")));
    assert_eq!(prep["inputs_sha256"], sha(&d.join("inputs.jsonl")));
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
    }
    let requests = fs::read_to_string(d.join("requests.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(requests.len(), 108);
    let mut hashes = serde_json::Map::new();
    for p in [
        "bootstrap/src/openrouter/per_memory/family_probe.rs",
        "bootstrap/src/openrouter/per_memory/scoped_definition_probe.rs",
        "tools/evaluation/src/scoped_family_probe.rs",
        "tools/evaluation/src/scoped_definition_probe.rs",
        "docs/evaluation/scoped-role-family-contrast-execution-v1.md",
    ] {
        hashes.insert(p.into(), json!(sha(Path::new(p))));
        let archive = d.join("source-executed").join(p);
        fs::create_dir_all(archive.parent().unwrap()).unwrap();
        fs::copy(p, archive).unwrap();
    }
    let pre = json!({"preparation_sha256":sha(&d.join("preparation.json")),"requests_sha256":prep["requests_sha256"],"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"sources_sha256":hashes,"max_http_requests":108,"fresh_validation":false,"promoted":false});
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("preflight.json"))
        .unwrap();
    f.write_all(serde_json::to_string_pretty(&pre).unwrap().as_bytes())
        .unwrap();
    f.sync_all().unwrap();
    let key = gennie_memory_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .expect("authorized key required");
    let client = OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut log = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("journal.jsonl"))
        .unwrap();
    let mut stopped = BTreeSet::new();
    let mut done = 0;
    for r in requests {
        let family = r["family"].as_u64().unwrap();
        if stopped.contains(&family) {
            continue;
        }
        writeln!(log, "{}", json!({"kind":"started","id":r["id"]})).unwrap();
        log.sync_all().unwrap();
        let start = Instant::now();
        let event = match client.request(&key, &r["payload"]) {
            Ok(body) => {
                if !super::scoped_definition_probe::role_schema_valid(&body, &r["payload"])
                    || body["model"] != r["expected_served_model"]
                {
                    stopped.insert(family);
                }
                json!({"kind":"response","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"body":body})
            }
            Err(e) => {
                stopped.insert(family);
                json!({"kind":"failed","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"code":e.code,"http_status":e.http_status,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        writeln!(log, "{event}").unwrap();
        log.sync_all().unwrap();
        done += 1;
    }
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("runner-status.json"))
        .unwrap();
    f.write_all(
        serde_json::to_string_pretty(
            &json!({"operations":done,"technically_stopped_families":stopped}),
        )
        .unwrap()
        .as_bytes(),
    )
    .unwrap();
    f.sync_all().unwrap();
}
