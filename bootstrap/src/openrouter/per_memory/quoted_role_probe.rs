//! Research only: no product policy, all raw receipts retained.
#[path = "../../../../tools/evaluation/src/quote_role_contract.rs"]
// The default build runs parser tests; only the opt-in runner uses HTTP helpers.
#[cfg_attr(not(feature = "http2-transport"), allow(dead_code))]
mod quote_contract;
#[cfg(feature = "http2-transport")]
use super::sparse_critic_probe::log;
#[cfg(feature = "http2-transport")]
use crate::openrouter::OpenRouterClassifier;
#[cfg(feature = "http2-transport")]
use serde_json::{Value, json};
#[cfg(feature = "http2-transport")]
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
    time::Instant,
};
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic generative quoted roles:6knownqueries/42judgments,6HTTP maximum,no retries"]
fn live_quoted_role_probe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let d = Path::new("outputs/iromem/evaluation/quoted-role-generative-v1-2026-10-20");
    let sha = |p: &Path| gennie_memory_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let prep: Value =
        serde_json::from_slice(&fs::read(d.join("preparation.json")).unwrap()).unwrap();
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
    }
    assert_eq!(prep["inputs_sha256"], sha(&d.join("inputs.jsonl")));
    assert_eq!(prep["requests_sha256"], sha(&d.join("requests.jsonl")));
    let reqs = fs::read_to_string(d.join("requests.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect::<Vec<_>>();
    let inputs = fs::read_to_string(d.join("inputs.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(reqs.len(), 6);
    let mut hashes = serde_json::Map::new();
    for p in [
        "bootstrap/src/openrouter.rs",
        "bootstrap/src/openrouter/per_memory/quoted_role_probe.rs",
        "tools/evaluation/src/quote_role_contract.rs",
        "tools/evaluation/src/quoted_role_probe.rs",
        "tools/evaluation/src/sparse_critic_probe.rs",
        "tools/evaluation/src/sparse_critic_probe/replacement.rs",
        "tools/evaluation/src/scoped_definition_probe.rs",
        "Cargo.toml",
        "Cargo.lock",
        "bootstrap/Cargo.toml",
    ] {
        hashes.insert(p.into(), json!(sha(Path::new(p))));
        let dst = d.join("source-executed").join(p);
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(p, dst).unwrap();
    }
    let pre = json!({"preparation_sha256":sha(&d.join("preparation.json")),"sources_sha256":hashes,"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"max_http_requests":6,"cfg_test_only":true});
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("preflight.json"))
        .unwrap();
    f.write_all(serde_json::to_string_pretty(&pre).unwrap().as_bytes())
        .unwrap();
    f.sync_all().unwrap();
    let key = gennie_memory_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .unwrap();
    let client = OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut logf = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("journal.jsonl"))
        .unwrap();
    let mut completed = 0;
    let mut stopped = false;
    for (r, row) in reqs.iter().zip(&inputs) {
        assert_eq!(r["id"], row["id"]);
        let q = row["role_payload"]["state"]["user_prompt"]
            .as_str()
            .unwrap();
        let c = row["candidates"].as_array().unwrap();
        assert_eq!(r["payload"], quote_contract::payload(q, c));
        let id = r["id"].as_str().unwrap();
        log(&mut logf, json!({"kind":"started","id":id}));
        let start = Instant::now();
        let result = (|| -> Result<(u16, String, Value), &str> {
            let response = client
                .client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .bearer_auth(&key)
                .header("X-Title", "Gennie Memory synthetic quoted role research")
                .json(&r["payload"])
                .send()
                .map_err(|_| "transport_error")?;
            let status = response.status().as_u16();
            let version = format!("{:?}", response.version());
            let mut bytes = Vec::new();
            response
                .take(1_048_577)
                .read_to_end(&mut bytes)
                .map_err(|_| "read_error")?;
            if bytes.len() > 1_048_576 {
                return Err("response_too_large");
            }
            let body = serde_json::from_slice(&bytes).map_err(|_| "invalid_json_body")?;
            Ok((status, version, body))
        })();
        let event = match result {
            Ok((status, version, body)) => {
                let validation = if status == 200 {
                    quote_contract::decode_body(&body, q, c).map(|_| ()).err()
                } else {
                    Some("http_error".into())
                };
                stopped = validation.is_some();
                let sanitized = serde_json::to_string(&body)
                    .unwrap()
                    .replace(&key, "[redacted]");
                let body: Value = serde_json::from_str(&sanitized).unwrap();
                json!({"kind":"response","id":id,"http_status":status,"http_version":version,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"validation_error":validation,"body":body})
            }
            Err(code) => {
                stopped = true;
                json!({"kind":"failed","id":id,"code":code,"elapsed_ms":start.elapsed().as_secs_f64()*1000.})
            }
        };
        log(&mut logf, event);
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
