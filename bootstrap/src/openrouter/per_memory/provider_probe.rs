//! Evaluation-only, fixed known-proposal provider comparison; no product policy.
use super::{correction_verified, role_verified};
use crate::openrouter::OpenRouterClassifier;
use gennie_memory_retrieval::model::{Candidate, ClassifierProvider};
use serde_json::{Value, json};
use std::path::Path;

fn payload(
    query: &str,
    candidate: &Candidate,
    correction: bool,
    provider: ClassifierProvider,
) -> Value {
    let mut p = if correction {
        correction_verified::verification_payload(query, candidate, 4)
    } else {
        role_verified::fact_payload(query, candidate, 1)
    };
    p["model"] = json!(provider.model_id());
    p
}

fn lines(path: &Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
#[ignore = "requires explicit authorization for 25 fixed synthetic opposite-task requests"]
fn live_role_opposite_task_probe() {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::Instant;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let base = Path::new("outputs/iromem/evaluation/role-isolated-provider-probe-2026-10-19");
    let report: Value =
        serde_json::from_slice(&std::fs::read(base.join("report-audited.json")).unwrap()).unwrap();
    let source_requests = lines(&base.join("requests.jsonl"));
    let dir = Path::new("outputs/iromem/evaluation/role-opposite-task-probe-2026-10-19");
    std::fs::create_dir_all(dir).unwrap();
    let mut requests = Vec::new();
    for group in report["providers"]["decisions"]["groups"]
        .as_array()
        .unwrap()
    {
        for case in group["cases"].as_array().unwrap() {
            for check in case["checks"].as_array().unwrap() {
                if check["selected"] != false {
                    continue;
                }
                let original = case["original_case_id"].as_str().unwrap();
                let id = format!(
                    "{}_{}_decisions",
                    original,
                    check["memory_id"].as_str().unwrap()
                );
                let r = source_requests.iter().find(|r| r["id"] == id).unwrap();
                let c = Candidate {
                    memory_id: check["memory_id"].as_str().unwrap().into(),
                    revision: check["revision"].as_i64().unwrap(),
                    summary: r["payload"]["state"]["saved_memory"]
                        .as_str()
                        .unwrap()
                        .into(),
                    conditions: None,
                    source: None,
                    state: "active".into(),
                    bm25: 0.0,
                };
                let correction = check["evidence_kind"] == "requested_fact_or_partial";
                requests.push(json!({"id":format!("{id}_opposite"),"base_request_id":id,"original_case_id":original,"cohort":group["cohort"],"memory_id":check["memory_id"],"revision":check["revision"],"original_evidence_kind":check["evidence_kind"],"correction_task":correction,"payload":payload(r["payload"]["state"]["query"].as_str().unwrap(),&c,correction,ClassifierProvider::Decisions)}));
            }
        }
    }
    assert_eq!(requests.len(), 25);
    let serialized = requests
        .iter()
        .map(|r| serde_json::to_string(r).unwrap() + "\n")
        .collect::<String>();
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("requests.jsonl"))
        .expect("one-use probe; no overwrite/retry");
    f.write_all(serialized.as_bytes()).unwrap();
    f.sync_all().unwrap();
    let mut hashes = serde_json::Map::new();
    for p in [
        base.join("requests.jsonl"),
        base.join("journal.jsonl"),
        base.join("report-audited.json"),
        std::path::PathBuf::from("docs/evaluation/opposite-task-probe-protocol-2026-10-19.md"),
    ] {
        hashes.insert(
            p.to_str().unwrap().into(),
            json!(gennie_memory_kernel::digest::sha256_hex(
                &std::fs::read(p).unwrap()
            )),
        );
    }
    std::fs::write(dir.join("preflight.json"),serde_json::to_vec_pretty(&json!({"version":"role-opposite-task-probe-v1","operations":25,"max_http_requests":25,"requests_sha256":gennie_memory_kernel::digest::sha256_hex(serialized.as_bytes()),"test_binary_sha256":gennie_memory_kernel::digest::sha256_hex(&std::fs::read(std::env::current_exe().unwrap()).unwrap()),"inputs_sha256":hashes,"eligibility":"All Decisions rejections in the frozen43positive proposal replay; no gold filter","fresh_validation":false,"promoted":false})).unwrap()).unwrap();
    let key = gennie_memory_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .expect("authorized key required");
    let client = OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut journal = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("journal.jsonl"))
        .unwrap();
    for r in requests {
        writeln!(journal, "{}", json!({"kind":"started","id":r["id"]})).unwrap();
        journal.sync_all().unwrap();
        let start = Instant::now();
        let event = match client.request(&key, &r["payload"]) {
            Ok(body) => {
                json!({"kind":"response","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"body":body})
            }
            Err(e) => {
                json!({"kind":"failed","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"code":e.code,"http_status":e.http_status,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        writeln!(journal, "{event}").unwrap();
        journal.sync_all().unwrap();
    }
}

#[test]
fn provider_probe_changes_only_the_model_for_both_complete_verification_tasks() {
    let c = Candidate {
        memory_id: "m".into(),
        revision: 1,
        summary: "A literal memory".into(),
        conditions: None,
        source: None,
        state: "active".into(),
        bm25: 0.0,
    };
    for correction in [false, true] {
        let d = payload("q", &c, correction, ClassifierProvider::Decisions);
        let mut j = payload("q", &c, correction, ClassifierProvider::Jev);
        assert_ne!(d["model"], j["model"]);
        j["model"] = d["model"].clone();
        assert_eq!(j, d);
        assert_eq!(
            d["state"],
            json!({"query":"q","saved_memory":"A literal memory"})
        );
        assert_eq!(
            d["questions"].as_object().unwrap().len(),
            if correction { 2 } else { 1 }
        );
    }
}

#[test]
#[ignore = "requires explicit authorization for 86 fixed synthetic OpenRouter requests"]
fn live_role_provider_probe() {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::Instant;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let dir = Path::new("outputs/iromem/evaluation/role-isolated-provider-probe-2026-10-19");
    std::fs::create_dir_all(dir).unwrap();
    let mut requests = Vec::new();
    let mut hashes = serde_json::Map::new();
    let mut n = 0;
    for (group, cohort) in [("pilot", "v5"), ("v4-targeted", "v4")] {
        let source = std::path::PathBuf::from(format!(
            "outputs/iromem/evaluation/per-memory-role-verified-v2-{group}-2026-10-19"
        ));
        for file in [
            source.join("journal.jsonl"),
            source.join("cases.jsonl"),
            source.join("pilot.json"),
            std::path::PathBuf::from(
                "docs/evaluation/isolated-role-provider-probe-protocol-2026-10-19.md",
            ),
            std::path::PathBuf::from(format!(
                "tests/fixtures/conversation-evidence-{cohort}/memories.json"
            )),
            std::path::PathBuf::from(format!(
                "tests/fixtures/conversation-evidence-{cohort}/development.jsonl"
            )),
        ] {
            hashes.insert(
                file.to_str().unwrap().into(),
                json!(gennie_memory_kernel::digest::sha256_hex(
                    &std::fs::read(file).unwrap()
                )),
            );
        }
        let cases = lines(&source.join("cases.jsonl"));
        let events = lines(&source.join("journal.jsonl"));
        let finished = events
            .iter()
            .filter(|e| e["kind"] == "finished")
            .collect::<Vec<_>>();
        assert_eq!(finished.len(), if cohort == "v5" { 13 } else { 5 });
        for event in finished {
            assert_eq!(event["response"]["status"], "complete");
            let case = cases.iter().find(|c| c["id"] == event["id"]).unwrap();
            let v = &event["response"]["per_memory"]["verification"];
            for kind in v["kind_proposal"].as_array().unwrap() {
                let role = kind["evidence_kind"].as_str().unwrap();
                if role == "unrelated" {
                    continue;
                }
                assert!(matches!(
                    role,
                    "requested_fact_or_partial" | "premise_correction"
                ));
                let candidate: Candidate = serde_json::from_value(
                    event["response"]["retrieval"]["candidates"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|c| {
                            c["memory_id"] == kind["memory_id"] && c["revision"] == kind["revision"]
                        })
                        .unwrap()
                        .clone(),
                )
                .unwrap();
                let query = case["request"]["search"]["query"].as_str().unwrap();
                let providers = if n % 2 == 0 {
                    [ClassifierProvider::Decisions, ClassifierProvider::Jev]
                } else {
                    [ClassifierProvider::Jev, ClassifierProvider::Decisions]
                };
                for provider in providers {
                    let name = if provider == ClassifierProvider::Jev {
                        "jev"
                    } else {
                        "decisions"
                    };
                    requests.push(json!({"id":format!("{}_{}_{}",case["original_case_id"].as_str().unwrap(),candidate.memory_id,name),"original_case_id":case["original_case_id"],"cohort":cohort,"memory_id":candidate.memory_id,"revision":candidate.revision,"evidence_kind":role,"provider":name,"payload":payload(query,&candidate,role=="premise_correction",provider)}));
                }
                n += 1;
            }
        }
    }
    assert_eq!(n, 43);
    assert_eq!(requests.len(), 86);
    let serialized = requests
        .iter()
        .map(|r| serde_json::to_string(r).unwrap() + "\n")
        .collect::<String>();
    let mut req = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("requests.jsonl"))
        .expect("one-use probe: no overwrite or uncertain retry");
    req.write_all(serialized.as_bytes()).unwrap();
    req.sync_all().unwrap();
    let executable = std::env::current_exe().unwrap();
    std::fs::write(dir.join("preflight.json"),serde_json::to_vec_pretty(&json!({"version":"role-isolated-provider-probe-v1","requests_sha256":gennie_memory_kernel::digest::sha256_hex(serialized.as_bytes()),"test_binary_sha256":gennie_memory_kernel::digest::sha256_hex(&std::fs::read(executable).unwrap()),"operations":86,"max_http_requests":86,"frozen_positive_pairs":43,"historical_inputs_sha256":hashes,"fresh_validation":false,"promoted":false})).unwrap()).unwrap();
    let key = gennie_memory_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .expect("authorized key required");
    let client = OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut journal = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("journal.jsonl"))
        .unwrap();
    for r in requests {
        writeln!(journal, "{}", json!({"kind":"started","id":r["id"]})).unwrap();
        journal.sync_all().unwrap();
        let started = Instant::now();
        let event = match client.request(&key, &r["payload"]) {
            Ok(body) => {
                json!({"kind":"response","id":r["id"],"elapsed_ms":started.elapsed().as_secs_f64()*1000.0,"body":body})
            }
            Err(e) => {
                json!({"kind":"failed","id":r["id"],"elapsed_ms":started.elapsed().as_secs_f64()*1000.0,"code":e.code,"http_status":e.http_status,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        writeln!(journal, "{event}").unwrap();
        journal.sync_all().unwrap();
    }
}
