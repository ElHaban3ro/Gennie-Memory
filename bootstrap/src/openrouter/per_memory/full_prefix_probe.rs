//! Evaluation-only complete-prefix semantic diagnostic, including negative proposals.
use super::complete_premise;
use crate::openrouter::OpenRouterClassifier;
use gennie_memory_retrieval::model::Candidate;
use gennie_memory_retrieval::per_memory::verification::{
    PREMISE_VERIFIER_MODEL, PREMISE_VERIFIER_REVISION,
};
use serde_json::{Value, json};
use std::path::Path;

fn payload(query: &str, c: &Candidate) -> Value {
    let mut p = complete_premise::verification_payload(query, c);
    p["questions"]["asked_dimension"] = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/asked-dimension-v1.json"
    ))
    .unwrap();
    p
}
fn parse(body: &Value) -> Option<bool> {
    let a = body["answers"].as_object().filter(|a| a.len() == 5)?;
    let mut b = std::collections::BTreeMap::new();
    for k in [
        "fact",
        "ordinary_correction",
        "context_contrast",
        "necessary_premise_incompatible",
        "asked_dimension",
    ] {
        let r = a.get(k)?;
        if r["type"] != "choice" {
            return None;
        }
        b.insert(
            k,
            match r["choice"].as_str()? {
                "A" => true,
                "none" => false,
                _ => return None,
            },
        );
    }
    Some(
        b["fact"] && b["asked_dimension"]
            || b["ordinary_correction"] && b["necessary_premise_incompatible"]
            || b["context_contrast"],
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
fn asked_dimension_limits_only_data_and_never_globally_vetoes_corrections() {
    let a = |s| json!({"type":"choice","choice":s});
    let mut b = json!({"answers":{"fact":a("A"),"ordinary_correction":a("none"),"context_contrast":a("none"),"necessary_premise_incompatible":a("none"),"asked_dimension":a("none")}});
    assert_eq!(parse(&b), Some(false));
    b["answers"]["asked_dimension"]["choice"] = json!("A");
    assert_eq!(parse(&b), Some(true));
    b["answers"]["asked_dimension"]["choice"] = json!("none");
    b["answers"]["ordinary_correction"]["choice"] = json!("A");
    b["answers"]["necessary_premise_incompatible"]["choice"] = json!("A");
    assert_eq!(parse(&b), Some(true));
    b["answers"]["ordinary_correction"]["choice"] = json!("none");
    b["answers"]["context_contrast"]["choice"] = json!("A");
    assert_eq!(parse(&b), Some(true));
    b["answers"]
        .as_object_mut()
        .unwrap()
        .remove("asked_dimension");
    assert_eq!(parse(&b), None);
    let c = Candidate {
        memory_id: "m".into(),
        revision: 1,
        summary: "literal".into(),
        conditions: None,
        source: None,
        state: "active".into(),
        bm25: 0.0,
    };
    let mut five = payload("q", &c);
    five["questions"]
        .as_object_mut()
        .unwrap()
        .remove("asked_dimension");
    assert_eq!(five, complete_premise::verification_payload("q", &c));
}

#[test]
#[ignore = "requires explicit authorization for 128 complete-prefix synthetic Perplexity judgments"]
fn live_full_prefix_dimension_probe() {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::Instant;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let dir = Path::new("outputs/iromem/evaluation/full-prefix-dimension-probe-2026-10-19");
    std::fs::create_dir_all(dir).unwrap();
    let mut requests = Vec::new();
    let mut hashes = serde_json::Map::new();
    let mut source_rows = Vec::new();
    for (suffix, cohort, count) in [("pilot", "v5", 13), ("v4-targeted", "v4", 5)] {
        let source = std::path::PathBuf::from(format!(
            "outputs/iromem/evaluation/per-memory-complete-premise-v1-{suffix}-2026-10-19"
        ));
        for p in [
            source.join("cases.jsonl"),
            source.join("journal.jsonl"),
            source.join("pilot.json"),
            source.join("audit.json"),
        ] {
            hashes.insert(
                p.to_str().unwrap().into(),
                json!(gennie_memory_kernel::digest::sha256_hex(
                    &std::fs::read(p).unwrap()
                )),
            );
        }
        let cases = lines(&source.join("cases.jsonl"));
        let events = lines(&source.join("journal.jsonl"));
        let finished = events
            .iter()
            .filter(|e| e["kind"] == "finished")
            .collect::<Vec<_>>();
        assert_eq!(finished.len(), count);
        for e in finished {
            assert_eq!(e["response"]["status"], "complete");
            let case = cases.iter().find(|c| c["id"] == e["id"]).unwrap();
            let candidates = e["response"]["retrieval"]["candidates"].as_array().unwrap();
            assert!(candidates.len() >= 7);
            for (rank, c) in candidates.iter().take(7).enumerate() {
                let candidate: Candidate = serde_json::from_value(c.clone()).unwrap();
                let original = case["original_case_id"].as_str().unwrap();
                source_rows.push((case.clone(), candidate.clone()));
                requests.push(json!({"id":format!("{original}_{}",candidate.memory_id),"original_case_id":original,"source_original_case_id":original,"memory_id":candidate.memory_id,"revision":candidate.revision,"cohort":cohort,"presented_rank":rank,"is_control":false,"expected_useful":case["expected_ids"].as_array().unwrap().contains(&c["memory_id"]),"payload":payload(case["request"]["search"]["query"].as_str().unwrap(),&candidate)}));
            }
        }
    }
    assert_eq!(requests.len(), 126);
    let controls: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/asked-dimension-controls-v1.json"
    ))
    .unwrap();
    for control in controls["cases"].as_array().unwrap() {
        let (_, c) = source_rows
            .iter()
            .find(|(case, c)| {
                case["original_case_id"] == control["source_original_case_id"]
                    && c.memory_id == control["source_memory_id"]
            })
            .unwrap();
        requests.push(json!({"id":control["id"],"original_case_id":control["id"],"source_original_case_id":control["source_original_case_id"],"memory_id":c.memory_id,"revision":c.revision,"cohort":"development_controls","presented_rank":null,"is_control":true,"expected_useful":control["expected_useful"],"payload":payload(control["query"].as_str().unwrap(),c)}));
    }
    assert_eq!(requests.len(), 128);
    let serialized = requests
        .iter()
        .map(|r| serde_json::to_string(r).unwrap() + "\n")
        .collect::<String>();
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("requests.jsonl"))
        .expect("one-use diagnostic: no overwrite or retry");
    f.write_all(serialized.as_bytes()).unwrap();
    f.sync_all().unwrap();
    for p in [
        "tests/fixtures/asked-dimension-v1.json",
        "tests/fixtures/asked-dimension-controls-v1.json",
        "tests/fixtures/premise-incompatibility-v1.json",
        "docs/evaluation/full-prefix-dimension-probe-protocol-2026-10-19.md",
        "outputs/iromem/evaluation/role-complete-premise-probe-2026-10-19/requests.jsonl",
        "outputs/iromem/evaluation/role-complete-premise-probe-2026-10-19/preflight.json",
    ] {
        hashes.insert(
            p.into(),
            json!(gennie_memory_kernel::digest::sha256_hex(
                &std::fs::read(p).unwrap()
            )),
        );
    }
    std::fs::write(dir.join("preflight.json"),serde_json::to_vec_pretty(&json!({"version":"full-prefix-dimension-probe-v1","prepared_operations":128,"max_http_requests":128,"candidate_judgments":126,"development_controls":2,"model_id":PREMISE_VERIFIER_MODEL,"expected_served_model":PREMISE_VERIFIER_REVISION,"requests_sha256":gennie_memory_kernel::digest::sha256_hex(serialized.as_bytes()),"inputs_sha256":hashes,"test_binary_sha256":gennie_memory_kernel::digest::sha256_hex(&std::fs::read(std::env::current_exe().unwrap()).unwrap()),"fresh_validation":false,"promoted":false})).unwrap()).unwrap();
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
    let mut done = 0;
    let mut technical_stop = false;
    for r in requests {
        writeln!(journal, "{}", json!({"kind":"started","id":r["id"]})).unwrap();
        journal.sync_all().unwrap();
        let start = Instant::now();
        let e = match client.request(&key, &r["payload"]) {
            Ok(body) => {
                technical_stop =
                    parse(&body).is_none() || body["model"] != PREMISE_VERIFIER_REVISION;
                json!({"kind":"response","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"body":body})
            }
            Err(e) => {
                technical_stop = true;
                json!({"kind":"failed","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"code":e.code,"http_status":e.http_status,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        done += 1;
        writeln!(journal, "{e}").unwrap();
        journal.sync_all().unwrap();
        if technical_stop {
            break;
        }
    }
    std::fs::write(dir.join("runner-status.json"),serde_json::to_vec_pretty(&json!({"operations":done,"technical_stop":technical_stop,"warning":"Not independent quality audit; no product promotion."})).unwrap()).unwrap();
}
