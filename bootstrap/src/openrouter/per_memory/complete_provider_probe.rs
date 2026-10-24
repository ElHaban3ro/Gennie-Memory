//! Test-only prospective model comparison with all evidence tasks per memory.
use super::{correction_verified, role_verified};
use crate::openrouter::OpenRouterClassifier;
use gennie_memory_retrieval::model::Candidate;
use serde_json::{Value, json};
use std::path::Path;

fn payload(query: &str, c: &Candidate, model: &str) -> Value {
    let f = role_verified::fact_payload(query, c, 1);
    let correction = correction_verified::verification_payload(query, c, 4);
    json!({"model":model,"state":f["state"],"questions":{"fact":f["questions"]["relevance"],"ordinary_correction":correction["questions"]["relevance"],"context_contrast":correction["questions"]["context_contrast"]}})
}
fn parse(body: &Value) -> Option<bool> {
    parse_mode(body, false)
}
fn probe_payload(premise_gate: bool, query: &str, c: &Candidate, model: &str) -> Value {
    let mut p = payload(query, c, model);
    if premise_gate {
        p["questions"]["necessary_premise_incompatible"] = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/premise-incompatibility-v1.json"
        ))
        .unwrap();
    }
    p
}
fn parse_mode(body: &Value, premise_gate: bool) -> Option<bool> {
    let a = body["answers"]
        .as_object()
        .filter(|a| a.len() == if premise_gate { 4 } else { 3 })?;
    let mut keys = vec!["fact", "ordinary_correction", "context_contrast"];
    if premise_gate {
        keys.push("necessary_premise_incompatible");
    }
    let mut values = std::collections::BTreeMap::new();
    for k in keys {
        let a = a.get(k)?;
        if a["type"] != "choice" {
            return None;
        }
        match a["choice"].as_str()? {
            "A" => {
                values.insert(k, true);
            }
            "none" => {
                values.insert(k, false);
            }
            _ => return None,
        }
    }
    Some(
        values["fact"]
            || values["context_contrast"]
            || values["ordinary_correction"]
                && (!premise_gate || values["necessary_premise_incompatible"]),
    )
}
fn lines(p: &Path) -> Vec<Value> {
    std::fs::read_to_string(p)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn complete_payload_preserves_all_three_tasks_without_kind_routing_or_labels() {
    let c = Candidate {
        memory_id: "m".into(),
        revision: 1,
        summary: "literal".into(),
        conditions: None,
        source: None,
        state: "active".into(),
        bm25: 0.0,
    };
    let p = payload("q", &c, "model");
    let f = role_verified::fact_payload("q", &c, 1);
    let other = correction_verified::verification_payload("q", &c, 4);
    assert_eq!(p["state"], json!({"query":"q","saved_memory":"literal"}));
    assert_eq!(p["questions"]["fact"], f["questions"]["relevance"]);
    assert_eq!(
        p["questions"]["ordinary_correction"],
        other["questions"]["relevance"]
    );
    assert_eq!(
        p["questions"]["context_contrast"],
        other["questions"]["context_contrast"]
    );
    let a = |s| json!({"type":"choice","choice":s});
    let mut b = json!({"answers":{"fact":a("none"),"ordinary_correction":a("A"),"context_contrast":a("none")}});
    assert_eq!(parse(&b), Some(true));
    b["answers"]["ordinary_correction"]["choice"] = json!("none");
    assert_eq!(parse(&b), Some(false));
    b["answers"].as_object_mut().unwrap().remove("fact");
    assert_eq!(parse(&b), None);
}

#[test]
fn premise_gate_only_limits_ordinary_correction_and_preserves_other_paths() {
    let a = |s| json!({"type":"choice","choice":s});
    let mut b = json!({"answers":{"fact":a("none"),"ordinary_correction":a("A"),"context_contrast":a("none"),"necessary_premise_incompatible":a("none")}});
    assert_eq!(parse_mode(&b, true), Some(false));
    b["answers"]["necessary_premise_incompatible"]["choice"] = json!("A");
    assert_eq!(parse_mode(&b, true), Some(true));
    b["answers"]["necessary_premise_incompatible"]["choice"] = json!("none");
    for key in ["fact", "context_contrast"] {
        b["answers"][key]["choice"] = json!("A");
        assert_eq!(parse_mode(&b, true), Some(true));
        b["answers"][key]["choice"] = json!("none");
    }
    b["answers"]
        .as_object_mut()
        .unwrap()
        .remove("necessary_premise_incompatible");
    assert_eq!(parse_mode(&b, true), None);
    let c = Candidate {
        memory_id: "m".into(),
        revision: 1,
        summary: "literal".into(),
        conditions: None,
        source: None,
        state: "active".into(),
        bm25: 0.0,
    };
    let base = payload("q", &c, "model");
    let mut revised = probe_payload(true, "q", &c, "model");
    revised["questions"]
        .as_object_mut()
        .unwrap()
        .remove("necessary_premise_incompatible");
    assert_eq!(base, revised);
}

#[test]
#[ignore = "requires explicit authorization for staged synthetic calls to three catalog decision models"]
fn live_complete_provider_probe() {
    run(false);
}

#[test]
#[ignore = "requires explicit authorization for staged synthetic four-task Perplexity checks"]
fn live_complete_premise_probe() {
    run(true);
}

fn run(premise_gate: bool) {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::Instant;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let fixture = Path::new("tests/fixtures/role-complete-provider-screen-v1.json");
    let spec: Value = serde_json::from_slice(&std::fs::read(fixture).unwrap()).unwrap();
    let base = Path::new(spec["base_directory"].as_str().unwrap());
    let originals = lines(&base.join("requests.jsonl"))
        .into_iter()
        .filter(|r| r["provider"] == "decisions")
        .collect::<Vec<_>>();
    assert_eq!(originals.len(), 43);
    let mut all_cases = Vec::new();
    for suffix in ["pilot", "v4-targeted"] {
        all_cases.extend(lines(Path::new(&format!(
            "outputs/iromem/evaluation/per-memory-role-verified-v2-{suffix}-2026-10-19/cases.jsonl"
        ))));
    }
    let dir = Path::new(if premise_gate {
        "outputs/iromem/evaluation/role-complete-premise-probe-2026-10-19"
    } else {
        "outputs/iromem/evaluation/role-complete-provider-probe-2026-10-19"
    });
    let model_count = if premise_gate { 1 } else { 3 };
    std::fs::create_dir_all(dir).unwrap();
    let mut requests = Vec::new();
    for (mi, model) in spec["models"]
        .as_array()
        .unwrap()
        .iter()
        .take(model_count)
        .enumerate()
    {
        let mut ordered = Vec::new();
        for pair in spec["screen"].as_array().unwrap() {
            ordered.push(
                originals
                    .iter()
                    .find(|r| r["original_case_id"] == pair[0] && r["memory_id"] == pair[1])
                    .unwrap(),
            );
        }
        assert_eq!(ordered.len(), 12);
        for r in &originals {
            if !ordered.iter().any(|x| x["id"] == r["id"]) {
                ordered.push(r);
            }
        }
        assert_eq!(ordered.len(), 43);
        for (i, r) in ordered.into_iter().enumerate() {
            let case = all_cases
                .iter()
                .find(|c| c["original_case_id"] == r["original_case_id"])
                .unwrap();
            let c = Candidate {
                memory_id: r["memory_id"].as_str().unwrap().into(),
                revision: r["revision"].as_i64().unwrap(),
                summary: r["payload"]["state"]["saved_memory"]
                    .as_str()
                    .unwrap()
                    .into(),
                conditions: None,
                source: None,
                state: "active".into(),
                bm25: 0.0,
            };
            requests.push(json!({"id":format!("{}_{}_model{mi}",r["original_case_id"].as_str().unwrap(),c.memory_id),"model_index":mi,"model_id":model["id"],"expected_served_model":model["expected_served_model"],"original_case_id":r["original_case_id"],"memory_id":c.memory_id,"revision":c.revision,"cohort":r["cohort"],"screen":i<12,"expected_useful":case["expected_ids"].as_array().unwrap().contains(&r["memory_id"]),"payload":probe_payload(premise_gate,r["payload"]["state"]["query"].as_str().unwrap(),&c,model["id"].as_str().unwrap())}));
        }
    }
    assert_eq!(requests.len(), model_count * 43);
    let serialized = requests
        .iter()
        .map(|r| serde_json::to_string(r).unwrap() + "\n")
        .collect::<String>();
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("requests.jsonl"))
        .expect("one-use staged run: never overwrite or retry");
    f.write_all(serialized.as_bytes()).unwrap();
    f.sync_all().unwrap();
    let mut hashes = serde_json::Map::new();
    for p in [
        fixture.to_path_buf(),
        base.join("requests.jsonl"),
        base.join("journal.jsonl"),
        base.join("report-audited.json"),
        std::path::PathBuf::from(spec["catalog_observation"].as_str().unwrap()),
        std::path::PathBuf::from(if premise_gate {
            "docs/evaluation/role-complete-premise-probe-protocol-2026-10-19.md"
        } else {
            "docs/evaluation/role-complete-provider-probe-protocol-2026-10-19.md"
        }),
    ] {
        hashes.insert(
            p.to_str().unwrap().into(),
            json!(gennie_memory_kernel::digest::sha256_hex(
                &std::fs::read(p).unwrap()
            )),
        );
    }
    if premise_gate {
        let p = "tests/fixtures/premise-incompatibility-v1.json";
        hashes.insert(
            p.into(),
            json!(gennie_memory_kernel::digest::sha256_hex(
                &std::fs::read(p).unwrap()
            )),
        );
    }
    std::fs::write(dir.join("preflight.json"),serde_json::to_vec_pretty(&json!({"version":if premise_gate{"role-complete-premise-probe-v1"}else{"role-complete-provider-probe-v1"},"prepared_operations":model_count*43,"max_http_requests":model_count*43,"initial_screen_requests":model_count*12,"requests_sha256":gennie_memory_kernel::digest::sha256_hex(serialized.as_bytes()),"inputs_sha256":hashes,"test_binary_sha256":gennie_memory_kernel::digest::sha256_hex(&std::fs::read(std::env::current_exe().unwrap()).unwrap()),"fresh_validation":false,"promoted":false})).unwrap()).unwrap();
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
    let mut statuses = Vec::new();
    for mi in 0..model_count {
        let rs = requests
            .iter()
            .filter(|r| r["model_index"] == mi)
            .collect::<Vec<_>>();
        let mut screen_pass = true;
        let mut correct = 0;
        let mut completed = 0;
        let mut technical = false;
        for (i, r) in rs.into_iter().enumerate() {
            if i == 12 && !screen_pass {
                break;
            }
            writeln!(journal, "{}", json!({"kind":"started","id":r["id"]})).unwrap();
            journal.sync_all().unwrap();
            let start = Instant::now();
            let event = match client.request(&key, &r["payload"]) {
                Ok(body) => {
                    let selected = parse_mode(&body, premise_gate);
                    let valid = selected.is_some()
                        && body["model"] == r["expected_served_model"]
                        && body["usage"]["cost"]
                            .as_f64()
                            .is_some_and(|c| c.is_finite() && c >= 0.0);
                    if i < 12 {
                        if valid && selected == r["expected_useful"].as_bool() {
                            correct += 1;
                        } else {
                            screen_pass = false;
                        }
                    }
                    technical = selected.is_none();
                    json!({"kind":"response","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"body":body})
                }
                Err(e) => {
                    screen_pass = false;
                    technical = true;
                    json!({"kind":"failed","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"code":e.code,"http_status":e.http_status,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
                }
            };
            completed += 1;
            writeln!(journal, "{event}").unwrap();
            journal.sync_all().unwrap();
            if technical {
                break;
            }
        }
        statuses.push(json!({"model_index":mi,"model_id":spec["models"][mi as usize]["id"],"screen_correct":correct,"screen_pass":screen_pass&&completed>=12,"completed":completed,"technical_stop":technical}));
    }
    std::fs::write(dir.join("runner-status.json"),serde_json::to_vec_pretty(&json!({"models":statuses,"warning":"Runner status is not independent audit or full pipeline promotion."})).unwrap()).unwrap();
}
