//! Matcher-only ablation on reused source projections; no preparation calls.
use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};
#[test]
#[ignore = "authorized synthetic role matcher: max128HTTP, existing projection receipts reused"]
fn live_independent_role_matcher() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let dir = Path::new("outputs/iromem/evaluation/independent-role-matcher-v2-2026-10-19");
    let pre: Value =
        serde_json::from_slice(&std::fs::read(dir.join("preflight.json")).unwrap()).unwrap();
    let bytes = std::fs::read(dir.join("requests.jsonl")).unwrap();
    let requests = std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect::<Vec<_>>();
    assert!(requests.len() <= 128);
    assert_eq!(pre["prepared_matching_operations"], requests.len());
    assert_eq!(
        pre["requests_sha256"],
        gennie_memory_kernel::digest::sha256_hex(&bytes)
    );
    let execution = json!({"preflight_sha256":gennie_memory_kernel::digest::sha256_hex(&std::fs::read(dir.join("preflight.json")).unwrap()),"requests_sha256":pre["requests_sha256"],"test_binary_sha256":gennie_memory_kernel::digest::sha256_hex(&std::fs::read(std::env::current_exe().unwrap()).unwrap())});
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("execution-preflight.json"))
        .unwrap();
    f.write_all(serde_json::to_string_pretty(&execution).unwrap().as_bytes())
        .unwrap();
    f.sync_all().unwrap();
    let key = gennie_memory_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .expect("authorized key required");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let start = Instant::now();
    let (events, fatal) = super::independent_projection_probe::run(
        &client,
        &key,
        &requests,
        &dir.join("journal.jsonl"),
    );
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("runner-status.json"))
        .unwrap();
    f.write_all(serde_json::to_string_pretty(&json!({"operations":events.len(),"fatal_transport":fatal,"elapsed_ms":start.elapsed().as_secs_f64()*1000.})).unwrap().as_bytes()).unwrap();
    f.sync_all().unwrap();
}
