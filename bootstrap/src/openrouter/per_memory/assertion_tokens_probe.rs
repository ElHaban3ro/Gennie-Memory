//! Manual synthetic assertion extraction, isolated from product compilation.
use crate::projection_tokens as projection;
use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
    time::{Duration, Instant},
};
#[test]
#[ignore = "authorized synthetic 128 HTTP assertion projection, concurrent batches of four"]
fn live_assertion_projection_tokens_v2_probe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let source = Path::new("outputs/iromem/evaluation/full-prefix-dimension-probe-2026-10-19");
    let dir =
        Path::new("outputs/iromem/evaluation/assertion-projection-tokens-v2-probe-2026-10-19");
    std::fs::create_dir_all(dir).unwrap();
    let rows = std::fs::read_to_string(source.join("requests.jsonl"))
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 128);
    let requests = rows
        .iter()
        .map(|s| {
            let mut r = s.clone();
            r["payload"] = projection::payload(
                s["payload"]["state"]["query"].as_str().unwrap(),
                s["payload"]["state"]["saved_memory"].as_str().unwrap(),
            );
            r
        })
        .collect::<Vec<_>>();
    let serialized = requests
        .iter()
        .map(|r| serde_json::to_string(r).unwrap() + "\n")
        .collect::<String>();
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("requests.jsonl"))
        .expect("one-use directory, no retry");
    f.write_all(serialized.as_bytes()).unwrap();
    f.sync_all().unwrap();
    let mut hashes = serde_json::Map::new();
    for p in [
        source.join("requests.jsonl"),
        source.join("preflight.json"),
        source.join("journal.jsonl"),
        source.join("report.json"),
        Path::new("tests/fixtures/assertion-projection-tokens-v2.json").to_path_buf(),
        Path::new("docs/evaluation/assertion-projection-tokens-v2-protocol-2026-10-19.md")
            .to_path_buf(),
        Path::new("tools/evaluation/src/projection_tokens.rs").to_path_buf(),
    ] {
        hashes.insert(
            p.to_str().unwrap().into(),
            json!(iromem_kernel::digest::sha256_hex(
                &std::fs::read(p).unwrap()
            )),
        );
    }
    let pre = json!({"version":"assertion-projection-tokens-probe-v2","prepared_operations":128,"max_http_requests":128,"max_concurrent":4,"inputs_sha256":hashes,"requests_sha256":iromem_kernel::digest::sha256_hex(serialized.as_bytes()),"test_binary_sha256":iromem_kernel::digest::sha256_hex(&std::fs::read(std::env::current_exe().unwrap()).unwrap()),"fresh_validation":false,"promoted":false});
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("preflight.json"))
        .unwrap();
    f.write_all(serde_json::to_string_pretty(&pre).unwrap().as_bytes())
        .unwrap();
    f.sync_all().unwrap();
    let key = iromem_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .expect("authorized key required");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let mut journal = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("journal.jsonl"))
        .unwrap();
    let mut done = 0;
    let mut stop = false;
    let start_all = Instant::now();
    for batch in requests.chunks(4) {
        for r in batch {
            writeln!(journal, "{}", json!({"kind":"started","id":r["id"]})).unwrap();
        }
        journal.sync_all().unwrap();
        let events = std::thread::scope(|scope| {
            let handles=batch.iter().map(|r|{let key=&key;let client=&client;scope.spawn(move||{let start=Instant::now();let body=(||->Result<(u16,Value),&str>{let resp=client.post("https://openrouter.ai/api/v1/chat/completions").bearer_auth(key).header("X-Title","iROMEM assertion projection diagnostic").json(&r["payload"]).send().map_err(|_|"transport_error")?;let status=resp.status().as_u16();let mut bytes=Vec::new();resp.take(1_048_577).read_to_end(&mut bytes).map_err(|_|"read_error")?;if bytes.len()>1_048_576{return Err("response_too_large");}let body=serde_json::from_slice(&bytes).map_err(|_|"invalid_http_json")?;Ok((status,body))})();match body{Ok((status,body))=>json!({"kind":"response","id":r["id"],"http_status":status,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"body":body}),Err(code)=>json!({"kind":"failed","id":r["id"],"code":code,"elapsed_ms":start.elapsed().as_secs_f64()*1000.})}})}).collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|h| {
                    h.join()
                        .expect("worker panic: uncertain calls, do not retry")
                })
                .collect::<Vec<_>>()
        });
        for (r, e) in batch.iter().zip(events) {
            let input: Value =
                serde_json::from_str(r["payload"]["messages"][1]["content"].as_str().unwrap())
                    .unwrap();
            stop |= e["kind"] != "response"
                || e["http_status"]
                    .as_u64()
                    .is_none_or(|s| !(200..300).contains(&s))
                || projection::inspect_response(
                    input["query"].as_str().unwrap(),
                    input["saved_memory"].as_str().unwrap(),
                    &e["body"],
                )
                .is_err();
            writeln!(journal, "{e}").unwrap();
            done += 1;
        }
        journal.sync_all().unwrap();
        eprintln!("assertion projection {done}/128, technical_stop={stop}");
        if stop {
            break;
        }
    }
    std::fs::write(dir.join("runner-status.json"),serde_json::to_vec_pretty(&json!({"operations":done,"technical_stop":stop,"elapsed_ms":start_all.elapsed().as_secs_f64()*1000.,"not_independent_audit":true})).unwrap()).unwrap();
}
