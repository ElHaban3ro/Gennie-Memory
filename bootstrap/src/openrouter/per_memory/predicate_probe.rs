//! Manual evaluation of a single property-bound relation; no runtime policy.
use crate::openrouter::OpenRouterClassifier;
use serde_json::{Value, json};
use std::{fs::OpenOptions, io::Write, path::Path, time::Instant};

fn relation(body: &Value) -> Option<&str> {
    let a = body["answers"].as_object().filter(|a| a.len() == 1)?;
    let r = a.get("relation")?;
    if r["type"] != "choice" {
        return None;
    }
    r["choice"]
        .as_str()
        .filter(|s| matches!(*s, "A" | "B" | "C" | "none"))
}
#[test]
fn context_is_distinct_from_requested_evidence_and_invalid_choice_is_not_none() {
    for s in ["A", "B", "C", "none"] {
        let b = json!({"answers":{"relation":{"type":"choice","choice":s}}});
        assert_eq!(relation(&b), Some(s));
    }
    for b in [
        json!({"answers":{}}),
        json!({"answers":{"relation":{"type":"choice","choice":"other"}}}),
        json!({"answers":{"relation":{"type":"choice","choice":"A"},"extra":{}}}),
    ] {
        assert!(relation(&b).is_none());
    }
}
#[test]
#[ignore = "authorized synthetic 384 HTTP property-relation diagnostic; one-use directory"]
fn live_predicate_relation_probe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let source = Path::new("outputs/iromem/evaluation/full-prefix-dimension-probe-2026-10-19");
    let dir = Path::new("outputs/iromem/evaluation/predicate-relation-probe-2026-10-19");
    std::fs::create_dir_all(dir).unwrap();
    let source_rows = std::fs::read_to_string(source.join("requests.jsonl"))
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(source_rows.len(), 128);
    let catalog_path =
        Path::new("outputs/iromem/evaluation/decision-catalog-observation-2026-10-19.json");
    let catalog: Value = serde_json::from_slice(&std::fs::read(catalog_path).unwrap()).unwrap();
    let models = catalog["models"].as_array().unwrap();
    assert_eq!(models.len(), 3);
    let question: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/predicate-relation-v1.json"
    ))
    .unwrap();
    let mut requests = Vec::new();
    for (i, m) in models.iter().enumerate() {
        for s in &source_rows {
            let mut r = s.clone();
            r["id"] = json!(format!("model{i}_{}", s["id"].as_str().unwrap()));
            r["source_request_id"] = s["id"].clone();
            r["model_id"] = m["id"].clone();
            r["expected_served_model"] = m["canonical_slug"].clone();
            r["payload"] = json!({"model":m["id"],"state":s["payload"]["state"],"questions":{"relation":question}});
            requests.push(r);
        }
    }
    let serialized = requests
        .iter()
        .map(|r| serde_json::to_string(r).unwrap() + "\n")
        .collect::<String>();
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("requests.jsonl"))
        .expect("no overwrite or uncertain retries");
    f.write_all(serialized.as_bytes()).unwrap();
    f.sync_all().unwrap();
    let mut hashes = serde_json::Map::new();
    for p in [
        source.join("requests.jsonl"),
        source.join("preflight.json"),
        source.join("journal.jsonl"),
        source.join("report.json"),
        catalog_path.to_path_buf(),
        Path::new("tests/fixtures/predicate-relation-v1.json").to_path_buf(),
        Path::new("docs/evaluation/predicate-relation-probe-protocol-2026-10-19.md").to_path_buf(),
    ] {
        hashes.insert(
            p.to_str().unwrap().into(),
            json!(iromem_kernel::digest::sha256_hex(
                &std::fs::read(&p).unwrap()
            )),
        );
    }
    let pre = json!({"version":"predicate-relation-probe-v1","prepared_operations":384,"max_http_requests":384,"models":models,"requests_sha256":iromem_kernel::digest::sha256_hex(serialized.as_bytes()),"inputs_sha256":hashes,"test_binary_sha256":iromem_kernel::digest::sha256_hex(&std::fs::read(std::env::current_exe().unwrap()).unwrap()),"fresh_validation":false,"promoted":false});
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
    let client = OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut journal = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("journal.jsonl"))
        .unwrap();
    let mut done = 0;
    let mut technical_stop = false;
    for r in requests {
        writeln!(journal, "{}", json!({"kind":"started","id":r["id"]})).unwrap();
        journal.sync_all().unwrap();
        let start = Instant::now();
        let e = match client.request(&key, &r["payload"]) {
            Ok(body) => {
                technical_stop =
                    relation(&body).is_none() || body["model"] != r["expected_served_model"];
                json!({"kind":"response","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"body":body})
            }
            Err(e) => {
                technical_stop = true;
                json!({"kind":"failed","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"code":e.code,"http_status":e.http_status,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        writeln!(journal, "{e}").unwrap();
        journal.sync_all().unwrap();
        done += 1;
        if done % 32 == 0 {
            eprintln!("predicate probe {done}/384");
        }
        if technical_stop {
            break;
        }
    }
    std::fs::write(dir.join("runner-status.json"),serde_json::to_vec_pretty(&json!({"operations":done,"technical_stop":technical_stop,"not_independent_audit":true})).unwrap()).unwrap();
}
