//! Three-stage synthetic diagnostic; query and source projections never see each other.
use crate::independent_projection as projection;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
    time::{Duration, Instant},
};
fn save(path: &Path, bytes: &[u8]) {
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .expect("one-use files; no retries");
    f.write_all(bytes).unwrap();
    f.sync_all().unwrap();
}
fn json_file(path: &Path, v: &Value) {
    save(path, &serde_json::to_vec_pretty(v).unwrap());
}
fn lines(rows: &[Value]) -> String {
    rows.iter()
        .map(|r| serde_json::to_string(r).unwrap() + "\n")
        .collect()
}
pub(super) fn run(
    client: &reqwest::blocking::Client,
    key: &str,
    requests: &[Value],
    path: &Path,
) -> (Vec<Value>, bool) {
    run_for_models(
        client,
        key,
        requests,
        path,
        projection::spec()["accepted_served_models"]
            .as_array()
            .unwrap(),
        None,
    )
}
pub(super) fn run_for_models(
    client: &reqwest::blocking::Client,
    key: &str,
    requests: &[Value],
    path: &Path,
    accepted_models: &[Value],
    valid_response: Option<fn(&Value, &Value) -> bool>,
) -> (Vec<Value>, bool) {
    let mut journal = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .unwrap();
    let mut out = Vec::new();
    let mut fatal = false;
    for batch in requests.chunks(4) {
        for r in batch {
            writeln!(journal, "{}", json!({"kind":"started","id":r["id"]})).unwrap();
        }
        journal.sync_all().unwrap();
        let done = std::thread::scope(|s| {
            let jobs=batch.iter().map(|r|s.spawn(move||{let start=Instant::now();let result=(||->Result<(u16,Value),&str>{let resp=client.post("https://openrouter.ai/api/v1/chat/completions").bearer_auth(key).header("X-Title","iROMEM independent projection diagnostic").json(&r["payload"]).send().map_err(|_|"transport_error")?;let status=resp.status().as_u16();let mut bytes=Vec::new();resp.take(1_048_577).read_to_end(&mut bytes).map_err(|_|"read_error")?;if bytes.len()>1_048_576{return Err("response_too_large");}Ok((status,serde_json::from_slice(&bytes).map_err(|_|"invalid_http_json")?))})();match result{Ok((status,body))=>json!({"kind":"response","id":r["id"],"http_status":status,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"body":body}),Err(code)=>json!({"kind":"failed","id":r["id"],"code":code,"elapsed_ms":start.elapsed().as_secs_f64()*1000.})}})).collect::<Vec<_>>();
            jobs.into_iter()
                .map(|j| j.join().expect("uncertain worker: no retry"))
                .collect::<Vec<_>>()
        });
        for e in done {
            fatal |= e["kind"] != "response"
                || e["http_status"]
                    .as_u64()
                    .is_none_or(|s| !(200..300).contains(&s))
                || !accepted_models.contains(&e["body"]["model"])
                || valid_response.is_some_and(|check| {
                    !check(requests.iter().find(|r| r["id"] == e["id"]).unwrap(), &e)
                });
            writeln!(journal, "{e}").unwrap();
            out.push(e);
        }
        journal.sync_all().unwrap();
        eprintln!(
            "{} {}/{} fatal={fatal}",
            path.file_name().unwrap().to_str().unwrap(),
            out.len(),
            requests.len()
        );
        if fatal {
            break;
        }
    }
    (out, fatal)
}
#[test]
#[ignore = "authorized synthetic independent projections: max253HTTP, concurrent batches of4"]
fn live_independent_projection_probe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let source = Path::new("outputs/iromem/evaluation/full-prefix-dimension-probe-2026-10-19");
    let dir = Path::new("outputs/iromem/evaluation/independent-projection-v1-2026-10-19");
    std::fs::create_dir_all(dir).unwrap();
    let rows = std::fs::read_to_string(source.join("requests.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 128);
    let plan = projection::prepare(&rows).unwrap();
    let requests = plan["projection_requests"].as_array().unwrap();
    assert_eq!(requests.len(), 125);
    assert_eq!(requests.iter().filter(|r| r["kind"] == "query").count(), 20);
    let serialized = lines(requests);
    save(
        &dir.join("projection.requests.jsonl"),
        serialized.as_bytes(),
    );
    json_file(&dir.join("plan.json"), &plan);
    let mut hashes = serde_json::Map::new();
    for p in [
        source.join("requests.jsonl"),
        source.join("preflight.json"),
        source.join("journal.jsonl"),
        source.join("report.json"),
        Path::new("tests/fixtures/independent-projection-v1.json").to_path_buf(),
        Path::new("tools/evaluation/src/independent_projection.rs").to_path_buf(),
        Path::new("docs/evaluation/independent-projection-v1-protocol-2026-10-19.md").to_path_buf(),
    ] {
        hashes.insert(
            p.to_str().unwrap().into(),
            json!(iromem_kernel::digest::sha256_hex(
                &std::fs::read(p).unwrap()
            )),
        );
    }
    json_file(
        &dir.join("preflight.json"),
        &json!({"version":"independent-projection-diagnostic-v1","projection_operations_planned":125,"query_projections":20,"memory_projections":105,"matching_pairs":128,"max_http_requests":253,"max_concurrent":4,"plan_sha256":iromem_kernel::digest::sha256_hex(&std::fs::read(dir.join("plan.json")).unwrap()),"projection_requests_sha256":iromem_kernel::digest::sha256_hex(serialized.as_bytes()),"inputs_sha256":hashes,"test_binary_sha256":iromem_kernel::digest::sha256_hex(&std::fs::read(std::env::current_exe().unwrap()).unwrap()),"fresh_validation":false,"promoted":false}),
    );
    let key = iromem_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .expect("authorized key required");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let start = Instant::now();
    let (events, fatal) = run(
        &client,
        &key,
        requests,
        &dir.join("projection.journal.jsonl"),
    );
    if fatal {
        json_file(
            &dir.join("runner-status.json"),
            &json!({"projection_operations":events.len(),"matching_operations":0,"fatal_transport":true,"elapsed_ms":start.elapsed().as_secs_f64()*1000.}),
        );
        return;
    }
    let mut frames = BTreeMap::new();
    for (r, e) in requests.iter().zip(&events) {
        if let Ok(p) = projection::check_projection(
            r["kind"].as_str().unwrap(),
            r["source_text"].as_str().unwrap(),
            &e["body"],
        ) {
            frames.insert(r["id"].as_str().unwrap().to_string(), p["decoded"].clone());
        }
    }
    let (matches, skipped) = projection::matching_requests(&rows, &plan, &frames).unwrap();
    let serialized = lines(&matches);
    save(&dir.join("matching.requests.jsonl"), serialized.as_bytes());
    json_file(&dir.join("matching.skipped.json"), &json!(skipped));
    json_file(
        &dir.join("matching.preflight.json"),
        &json!({"version":"independent-matching-v1","matching_operations_planned":matches.len(),"skipped_pairs":skipped.len(),"matching_requests_sha256":iromem_kernel::digest::sha256_hex(serialized.as_bytes()),"projection_journal_sha256":iromem_kernel::digest::sha256_hex(&std::fs::read(dir.join("projection.journal.jsonl")).unwrap()),"phase1_preflight_sha256":iromem_kernel::digest::sha256_hex(&std::fs::read(dir.join("preflight.json")).unwrap()),"skipped_sha256":iromem_kernel::digest::sha256_hex(&std::fs::read(dir.join("matching.skipped.json")).unwrap())}),
    );
    let (matched, fatal) = run(&client, &key, &matches, &dir.join("matching.journal.jsonl"));
    json_file(
        &dir.join("runner-status.json"),
        &json!({"projection_operations":events.len(),"matching_operations":matched.len(),"fatal_transport":fatal,"skipped_pairs":skipped.len(),"elapsed_ms":start.elapsed().as_secs_f64()*1000.}),
    );
}
