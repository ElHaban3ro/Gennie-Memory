//! Evaluation-only paired probe; production classifier and policies are unchanged.
use super::{Candidate, ClassifierProvider, scoped_anchor};
use crate::openrouter::OpenRouterClassifier;
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};
const DIR: &str = "outputs/iromem/evaluation/scoped-anchor-definitions-v2-2026-10-19";
fn definitions(mut p: Value, version: u8) -> Value {
    let spec: Value = serde_json::from_str(match version {
        7 => include_str!("../../../../tests/fixtures/scoped-anchor-path-v1.json"),
        6 => include_str!("../../../../tests/fixtures/scoped-anchor-role-criteria-v2.json"),
        5 => include_str!("../../../../tests/fixtures/scoped-anchor-role-v5.json"),
        4 => include_str!("../../../../tests/fixtures/scoped-anchor-presupposition-v4.json"),
        3 => include_str!("../../../../tests/fixtures/scoped-anchor-inline-v3.json"),
        _ => include_str!("../../../../tests/fixtures/scoped-anchor-definitions-v2.json"),
    })
    .unwrap();
    if spec["path_syntax"] == true {
        let keys = p["state"]["memories"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for q in p["questions"].as_object_mut().unwrap().values_mut() {
            let mut text = q["instructions"].as_str().unwrap().replace(
                spec["query_path"]["before"].as_str().unwrap(),
                spec["query_path"]["after"].as_str().unwrap(),
            );
            for key in &keys {
                text = text.replace(
                    &spec["memory_path"]["before"]
                        .as_str()
                        .unwrap()
                        .replace("{key}", key),
                    &spec["memory_path"]["after"]
                        .as_str()
                        .unwrap()
                        .replace("{key}", key),
                );
            }
            q["instructions"] = json!(text);
        }
        return p;
    }
    if !spec["relation_instruction"].is_null() {
        let count = p["state"]["memories"].as_object().unwrap().len();
        let mut questions = serde_json::Map::new();
        if let Some(anchor) = p["questions"].get("case_anchor") {
            questions.insert("case_anchor".into(), anchor.clone());
        }
        for n in 0..count {
            let key = format!("memory_{n:02}");
            questions.insert(format!("{key}_relation"),json!({"type":"choice","instructions":spec["relation_instruction"].as_str().unwrap().replace("{key}",&key),"criteria":spec["criteria"]}));
        }
        p["questions"] = json!(questions);
        return p;
    }
    if !spec["evidence_tests"].is_null() {
        p["state"]["evidence_tests"] = spec["evidence_tests"].clone();
    }
    for (key, q) in p["questions"].as_object_mut().unwrap() {
        if let Some((memory, factor)) = key.rsplit_once('_') {
            if ["fact", "correction"].contains(&factor) {
                q["instructions"] = json!(
                    spec[format!("{factor}_instruction")]
                        .as_str()
                        .unwrap()
                        .replace("{key}", memory)
                );
            }
        }
    }
    p
}
#[test]
fn definitions_change_only_evidence_instructions_and_keep_original_parser_contract() {
    let c = Candidate {
        memory_id: "m".into(),
        revision: 1,
        summary: "A partial measurement.".into(),
        conditions: None,
        source: None,
        state: "active".into(),
        bm25: 0.,
    };
    let a = scoped_anchor::payload("¿Qué parte de Orbe?", "s", &[c], ClassifierProvider::Jev);
    let mut b = definitions(a.clone(), 2);
    assert_ne!(a, b);
    b["state"].as_object_mut().unwrap().remove("evidence_tests");
    for (k, v) in b["questions"].as_object_mut().unwrap() {
        v["instructions"] = a["questions"][k]["instructions"].clone();
    }
    assert_eq!(a, b);
}
fn lines(p: &Path) -> Vec<Value> {
    fs::read_to_string(p)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}
#[test]
#[ignore = "authorized synthetic diagnostic: 18 known questions, two variants, 36 HTTP, no retries"]
fn live_scoped_definition_probe() {
    run(2);
}
#[test]
#[ignore = "authorized synthetic inline diagnostic: same 18 known questions, 36 HTTP, no retries"]
fn live_scoped_inline_probe() {
    run(3);
}
#[test]
fn inline_definitions_change_no_state_or_choice_namespace() {
    let c = Candidate {
        memory_id: "m".into(),
        revision: 1,
        summary: "partial".into(),
        conditions: None,
        source: None,
        state: "active".into(),
        bm25: 0.,
    };
    let a = scoped_anchor::payload("¿Qué parte de Orbe?", "s", &[c], ClassifierProvider::Jev);
    let mut b = definitions(a.clone(), 3);
    assert_ne!(a, b);
    assert_eq!(a["state"], b["state"]);
    for (k, q) in b["questions"].as_object_mut().unwrap() {
        q["instructions"] = a["questions"][k]["instructions"].clone();
    }
    assert_eq!(a, b);
}
#[test]
#[ignore = "authorized synthetic presupposition diagnostic: same18questions,36HTTP,no retries"]
fn live_scoped_presupposition_probe() {
    run(4);
}
#[test]
fn presupposition_revision_changes_only_correction_instructions() {
    let c = Candidate {
        memory_id: "m".into(),
        revision: 1,
        summary: "partial".into(),
        conditions: None,
        source: None,
        state: "active".into(),
        bm25: 0.,
    };
    let base = scoped_anchor::payload("¿Qué parte de Orbe?", "s", &[c], ClassifierProvider::Jev);
    let a = definitions(base.clone(), 3);
    let mut b = definitions(base, 4);
    assert_ne!(a, b);
    for (k, q) in b["questions"].as_object_mut().unwrap() {
        if k.ends_with("_correction") {
            q["instructions"] = a["questions"][k]["instructions"].clone();
        }
    }
    assert_eq!(a, b);
}
#[test]
#[ignore = "authorized synthetic role diagnostic: same18questions,36HTTP,no retries"]
fn live_scoped_role_probe() {
    run(5);
}
pub(super) fn role_schema_valid(body: &Value, payload: &Value) -> bool {
    let Some(answers) = body["answers"].as_object() else {
        return false;
    };
    let qs = payload["questions"].as_object().unwrap();
    if answers.len() != qs.len() {
        return false;
    }
    qs.iter().all(|(key, q)| {
        let Some(answer) = answers.get(key) else {
            return false;
        };
        let mut one = body.clone();
        one["answers"] = json!({"relevance":answer});
        let labels = q["criteria"]
            .as_object()
            .unwrap()
            .iter()
            .filter(|(k, _)| *k != "none")
            .map(|(_, v)| v.as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        super::super::parse(one, &labels).is_ok()
    })
}
#[test]
fn exclusive_roles_preserve_state_anchor_and_reject_missing_or_unbound_answers() {
    let c = Candidate {
        memory_id: "m".into(),
        revision: 1,
        summary: "partial".into(),
        conditions: None,
        source: None,
        state: "active".into(),
        bm25: 0.,
    };
    let a = scoped_anchor::payload("¿Qué parte de Orbe?", "s", &[c], ClassifierProvider::Jev);
    let b = definitions(a.clone(), 5);
    assert_eq!(a["state"], b["state"]);
    assert_eq!(a["questions"]["case_anchor"], b["questions"]["case_anchor"]);
    assert_eq!(b["questions"].as_object().unwrap().len(), 2);
    let mut body = json!({"model":"test","usage":{"input_tokens":20,"output_tokens":0,"cost":0.001},"answers":{"case_anchor":{"type":"choice","choice":"A","probabilities":{"A":1.0,"none":0.0}},"memory_00_relation":{"type":"choice","choice":"B","probabilities":{"A":0.0,"B":1.0,"none":0.0}}}});
    assert!(role_schema_valid(&body, &b));
    let mut swapped = body.clone();
    let answer = swapped["answers"]
        .as_object_mut()
        .unwrap()
        .remove("memory_00_relation")
        .unwrap();
    swapped["answers"]["unknown"] = answer;
    assert!(!role_schema_valid(&swapped, &b));
    body["answers"]["memory_00_relation"]["choice"] = json!("C");
    assert!(!role_schema_valid(&body, &b));
    body["answers"]
        .as_object_mut()
        .unwrap()
        .remove("memory_00_relation");
    assert!(!role_schema_valid(&body, &b));
}
#[test]
#[ignore = "authorized synthetic role criteria diagnostic:18questions36HTTP,no retries"]
fn live_scoped_role_criteria_probe() {
    run(6);
}
#[test]
fn role_criteria_revision_only_changes_contribution_and_refutation_descriptions() {
    let c = Candidate {
        memory_id: "m".into(),
        revision: 1,
        summary: "partial".into(),
        conditions: None,
        source: None,
        state: "active".into(),
        bm25: 0.,
    };
    let base = scoped_anchor::payload("¿Qué parte de Orbe?", "s", &[c], ClassifierProvider::Jev);
    let a = definitions(base.clone(), 5);
    let mut b = definitions(base, 6);
    assert_ne!(a, b);
    for (k, q) in b["questions"].as_object_mut().unwrap() {
        if k.ends_with("_relation") {
            q["criteria"]["A"] = a["questions"][k]["criteria"]["A"].clone();
            q["criteria"]["B"] = a["questions"][k]["criteria"]["B"].clone();
        }
    }
    assert_eq!(a, b);
}
#[test]
#[ignore = "authorized synthetic documented-path diagnostic:18questions36HTTP,no retries"]
fn live_scoped_path_probe() {
    run(7);
}
fn paths_resolve(p: &Value) -> bool {
    p["questions"].as_object().unwrap().values().all(|q| {
        q["instructions"]
            .as_str()
            .unwrap()
            .split('`')
            .enumerate()
            .filter(|(i, _)| i % 2 == 1)
            .all(|(_, path)| {
                let mut value = &p["state"];
                for part in path.split('.') {
                    let Some(v) = value.get(part) else {
                        return false;
                    };
                    value = v;
                }
                !value.is_null()
            })
    })
}
#[test]
fn documented_paths_resolve_and_reversing_only_paths_restores_the_entire_payload() {
    let c = Candidate {
        memory_id: "m".into(),
        revision: 1,
        summary: "partial".into(),
        conditions: None,
        source: None,
        state: "active".into(),
        bm25: 0.,
    };
    let a = scoped_anchor::payload("¿Qué parte de Orbe?", "s", &[c], ClassifierProvider::Jev);
    let mut b = definitions(a.clone(), 7);
    assert_ne!(a, b);
    assert!(paths_resolve(&b));
    for q in b["questions"].as_object_mut().unwrap().values_mut() {
        q["instructions"] = json!(
            q["instructions"]
                .as_str()
                .unwrap()
                .replace("`user_prompt`", "state.user_prompt")
                .replace("`memories.memory_00`", "state.memories.memory_00")
        );
    }
    assert_eq!(a, b);
    b["questions"]["case_anchor"]["instructions"] = json!("Read `state.user_prompt`.");
    assert!(!paths_resolve(&b));
}
fn run(version: u8) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let (directory, spec_path, protocol_path) = if version == 7 {
        (
            "outputs/iromem/evaluation/scoped-anchor-path-v1-2026-10-19",
            "tests/fixtures/scoped-anchor-path-v1.json",
            "docs/evaluation/scoped-anchor-path-v1-protocol-2026-10-19.md",
        )
    } else if version == 6 {
        (
            "outputs/iromem/evaluation/scoped-anchor-role-criteria-v2-2026-10-19",
            "tests/fixtures/scoped-anchor-role-criteria-v2.json",
            "docs/evaluation/scoped-anchor-role-criteria-v2-protocol-2026-10-19.md",
        )
    } else if version >= 5 {
        (
            "outputs/iromem/evaluation/scoped-anchor-role-v5-2026-10-19",
            "tests/fixtures/scoped-anchor-role-v5.json",
            "docs/evaluation/scoped-anchor-role-v5-protocol-2026-10-19.md",
        )
    } else if version == 4 {
        (
            "outputs/iromem/evaluation/scoped-anchor-presupposition-v4-2026-10-19",
            "tests/fixtures/scoped-anchor-presupposition-v4.json",
            "docs/evaluation/scoped-anchor-presupposition-v4-protocol-2026-10-19.md",
        )
    } else if version == 3 {
        (
            "outputs/iromem/evaluation/scoped-anchor-inline-v3-2026-10-19",
            "tests/fixtures/scoped-anchor-inline-v3.json",
            "docs/evaluation/scoped-anchor-inline-v3-protocol-2026-10-19.md",
        )
    } else {
        (
            DIR,
            "tests/fixtures/scoped-anchor-definitions-v2.json",
            "docs/evaluation/scoped-anchor-definitions-v2-protocol-2026-10-19.md",
        )
    };
    let dir = Path::new(directory);
    let input = lines(&dir.join("inputs.jsonl"));
    assert_eq!(input.len(), 18);
    let prep: Value =
        serde_json::from_slice(&fs::read(dir.join("preparation.json")).unwrap()).unwrap();
    let sha = |p: &Path| iromem_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    assert_eq!(prep["inputs_sha256"], sha(&dir.join("inputs.jsonl")));
    assert_eq!(prep["spec_sha256"], sha(Path::new(spec_path)));
    assert_eq!(prep["protocol_sha256"], sha(Path::new(protocol_path)));
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
    }
    let mut requests = Vec::new();
    for (i, row) in input.iter().enumerate() {
        let candidates: Vec<Candidate> = serde_json::from_value(row["candidates"].clone()).unwrap();
        let query = row["case"]["request"]["search"]["query"].as_str().unwrap();
        let scope = row["case"]["request"]["search"]["scope_id"]
            .as_str()
            .unwrap();
        let old = scoped_anchor::payload(query, scope, &candidates, ClassifierProvider::Jev);
        let new = definitions(old.clone(), version);
        if version == 7 {
            assert!(paths_resolve(&new));
        }
        for variant in if i % 2 == 0 {
            ["control", "definitions"]
        } else {
            ["definitions", "control"]
        } {
            requests.push(json!({"id":format!("{}__{variant}",row["id"].as_str().unwrap()),"variant":variant,"payload":if variant=="control"{old.clone()}else{new.clone()}}));
        }
    }
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("requests.jsonl"))
        .unwrap();
    for r in &requests {
        writeln!(f, "{r}").unwrap();
    }
    f.sync_all().unwrap();
    let mut hashes = serde_json::Map::new();
    for p in [
        "bootstrap/src/openrouter/per_memory/scoped.rs",
        "bootstrap/src/openrouter/per_memory/scoped_anchor.rs",
        "bootstrap/src/openrouter/per_memory/scoped_definition_probe.rs",
        "tools/evaluation/src/scoped_definition_probe.rs",
        spec_path,
        protocol_path,
    ] {
        hashes.insert(p.into(), json!(sha(Path::new(p))));
        let archive = dir.join("source-executed").join(p);
        fs::create_dir_all(archive.parent().unwrap()).unwrap();
        fs::copy(p, archive).unwrap();
    }
    let pre = json!({"requests_sha256":sha(&dir.join("requests.jsonl")),"preparation_sha256":sha(&dir.join("preparation.json")),"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"sources_sha256":hashes,"operations":36,"max_http_requests":36,"fresh_validation":false,"promoted":false});
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
        let result = match client.request(&key, &r["payload"]) {
            Ok(body) => {
                let index = input
                    .iter()
                    .position(|x| {
                        r["id"]
                            .as_str()
                            .unwrap()
                            .starts_with(&format!("{}__", x["id"].as_str().unwrap()))
                    })
                    .unwrap();
                let row = &input[index];
                let candidates: Vec<Candidate> =
                    serde_json::from_value(row["candidates"].clone()).unwrap();
                technical_stop = (if matches!(version, 5 | 6) {
                    !role_schema_valid(&body, &r["payload"])
                } else {
                    scoped_anchor::parse(
                        body.clone(),
                        row["case"]["request"]["search"]["query"].as_str().unwrap(),
                        &candidates,
                    )
                    .is_err()
                }) || body["model"] != "typesafe/jev-1.13-20260917";
                json!({"kind":"response","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"body":body})
            }
            Err(e) => {
                technical_stop = true;
                json!({"kind":"failed","id":r["id"],"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"code":e.code,"http_status":e.http_status,"partial_usage":e.partial_usage.map(|u|json!({"reported_cost_usd":u.reported_cost_usd,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"served_model":u.served_model}))})
            }
        };
        writeln!(journal, "{result}").unwrap();
        journal.sync_all().unwrap();
        done += 1;
        if technical_stop {
            break;
        }
    }
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("runner-status.json"))
        .unwrap();
    f.write_all(
        serde_json::to_string_pretty(&json!({"operations":done,"technical_stop":technical_stop}))
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    f.sync_all().unwrap();
}
