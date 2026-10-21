//! Manual synthetic classifier-chain probe. No policy or service activation.
mod replacement;
use crate::openrouter::{self, OpenRouterClassifier};
use iromem_retrieval::{
    model::{Candidate, ClassifierProvider},
    per_memory::anchor::present,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};
const DIR: &str = "outputs/iromem/evaluation/sparse-role-critic-pilot-v1-2026-10-20";
const MODELS: [&str; 3] = [
    "inception/mercury-decide",
    "typesafe/jev-1.13",
    "microsoft/microsoft-decision-1",
];
const REVISIONS: [&str; 3] = [
    "inception/mercury-decide-20260930",
    "typesafe/jev-1.13-20260917",
    "microsoft/microsoft-decision-1-20261009",
];
pub(super) fn log(f: &mut File, event: Value) {
    writeln!(f, "{event}").unwrap();
    f.sync_all().unwrap();
}
fn subset(template: &Value, candidates: &[Value], model: &str) -> Value {
    let mut p = template.clone();
    p["model"] = json!(model);
    p["state"]["memories"] = json!({});
    let mut qs = serde_json::Map::new();
    if let Some(a) = template["questions"].get("case_anchor") {
        qs.insert("case_anchor".into(), a.clone());
    }
    for (n, c) in candidates.iter().enumerate() {
        let k = format!("memory_{n:02}");
        p["state"]["memories"][&k] = c["summary"].clone();
        let mut q = template["questions"]["memory_00_relation"].clone();
        q["instructions"] = json!(q["instructions"].as_str().unwrap().replace("memory_00", &k));
        qs.insert(format!("{k}_relation"), q);
    }
    p["questions"] = json!(qs);
    p
}
fn roles(
    body: &Value,
    payload: &Value,
    candidates: &[Value],
    revision: &str,
) -> Result<(BTreeSet<String>, BTreeSet<String>), String> {
    if body["model"] != revision
        || !super::scoped_definition_probe::role_schema_valid(body, payload)
    {
        return Err("invalid role response".into());
    }
    let anchor = body["answers"]
        .get("case_anchor")
        .and_then(|a| a["choice"].as_str())
        .filter(|s| *s != "none")
        .map(|s| {
            payload["questions"]["case_anchor"]["criteria"][s]
                .as_str()
                .unwrap()
        });
    let mut a = BTreeSet::new();
    let mut b = BTreeSet::new();
    for (n, c) in candidates.iter().enumerate() {
        if anchor.is_some_and(|name| !present(name, c["summary"].as_str().unwrap())) {
            continue;
        }
        let id = c["memory_id"].as_str().unwrap().to_owned();
        match body["answers"][format!("memory_{n:02}_relation")]["choice"].as_str() {
            Some("A") => {
                a.insert(id);
            }
            Some("B") => {
                b.insert(id);
            }
            _ => {}
        }
    }
    Ok((a, b))
}
fn request(
    client: &OpenRouterClassifier,
    key: &str,
    logfile: &mut File,
    id: &str,
    stage: &str,
    payload: &Value,
    candidates: &[Value],
) -> Result<Value, String> {
    log(
        logfile,
        json!({"kind":"started","id":id,"stage":stage,"payload":payload,"payload_sha256":iromem_kernel::digest::sha256_hex(serde_json::to_string(payload).unwrap().as_bytes()),"candidates":candidates}),
    );
    let start = Instant::now();
    match client.request_measured(key, payload) {
        Ok((body, transport)) => {
            log(
                logfile,
                json!({"kind":"response","id":id,"stage":stage,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"http_version":transport.version,"http_elapsed_ms":transport.elapsed_ms,"body":body}),
            );
            Ok(body)
        }
        Err(e) => {
            log(
                logfile,
                json!({"kind":"failed","id":id,"stage":stage,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"code":e.code,"http_status":e.http_status,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))}),
            );
            Err(e.code.into())
        }
    }
}
fn candidate(
    client: &OpenRouterClassifier,
    key: &str,
    f: &mut File,
    id: &str,
    row: &Value,
) -> Result<Vec<String>, String> {
    let original = row["candidates"].as_array().unwrap();
    let mut current = original.clone();
    let mut kept = BTreeSet::new();
    for n in 0..3 {
        if current.is_empty() {
            break;
        }
        let p = subset(&row["role_payload"], &current, MODELS[n]);
        let body = request(
            client,
            key,
            f,
            id,
            ["proposal", "jev_critic", "microsoft_critic"][n],
            &p,
            &current,
        )?;
        let (a, b) = roles(&body, &p, &current, REVISIONS[n])?;
        if n == 0 {
            kept = a;
            current.retain(|c| b.contains(c["memory_id"].as_str().unwrap()));
        } else {
            current.retain(|c| {
                a.contains(c["memory_id"].as_str().unwrap())
                    || b.contains(c["memory_id"].as_str().unwrap())
            });
        }
    }
    kept.extend(
        current
            .iter()
            .map(|c| c["memory_id"].as_str().unwrap().to_owned()),
    );
    Ok(original
        .iter()
        .filter_map(|c| {
            let id = c["memory_id"].as_str().unwrap();
            kept.contains(id).then(|| id.to_owned())
        })
        .collect())
}
fn parallel_candidate(
    client: &OpenRouterClassifier,
    key: &str,
    f: &mut File,
    id: &str,
    row: &Value,
    targeted: bool,
) -> Result<Vec<String>, String> {
    let original = row["candidates"].as_array().unwrap();
    let p = subset(&row["role_payload"], original, MODELS[0]);
    let body = request(client, key, f, id, "proposal", &p, original)?;
    let (mut a, b) = roles(&body, &p, original, REVISIONS[0])?;
    if !b.is_empty() {
        let mut payloads = [
            subset(&row["role_payload"], original, MODELS[1]),
            subset(&row["role_payload"], original, MODELS[2]),
        ];
        if targeted {
            for p in &mut payloads {
                p["questions"].as_object_mut().unwrap().retain(|k, _| {
                    k == "case_anchor"
                        || original.iter().enumerate().any(|(n, c)| {
                            k == &format!("memory_{n:02}_relation")
                                && b.contains(c["memory_id"].as_str().unwrap())
                        })
                });
            }
        }
        for (stage, p) in ["jev_critic", "microsoft_critic"]
            .into_iter()
            .zip(&payloads)
        {
            log(
                f,
                json!({"kind":"started","id":id,"stage":stage,"payload":p,"payload_sha256":iromem_kernel::digest::sha256_hex(serde_json::to_string(p).unwrap().as_bytes()),"candidates":original}),
            );
        }
        let results = std::thread::scope(|scope| {
            let handles = payloads
                .iter()
                .map(|p| {
                    scope.spawn(move || {
                        let start = Instant::now();
                        let result = client.request_measured(key, p);
                        (result, start.elapsed().as_secs_f64() * 1000.)
                    })
                })
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });
        // Drain and persist both outcomes before any validation or early return.
        let mut bodies = Vec::new();
        let mut failed = false;
        for ((result, elapsed), stage) in
            results.into_iter().zip(["jev_critic", "microsoft_critic"])
        {
            match result {
                Ok((body, transport)) => {
                    log(
                        f,
                        json!({"kind":"response","id":id,"stage":stage,"elapsed_ms":elapsed,"http_version":transport.version,"http_elapsed_ms":transport.elapsed_ms,"body":body}),
                    );
                    bodies.push(Some(body));
                }
                Err(e) => {
                    failed = true;
                    log(
                        f,
                        json!({"kind":"failed","id":id,"stage":stage,"elapsed_ms":elapsed,"code":e.code,"http_status":e.http_status,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))}),
                    );
                    bodies.push(None);
                }
            }
        }
        if failed {
            return Err("parallel critic failed".into());
        }
        let mut survivors = b;
        for n in 0..2 {
            let (yes_a, yes_b) = roles(
                bodies[n].as_ref().unwrap(),
                &payloads[n],
                original,
                REVISIONS[n + 1],
            )?;
            survivors.retain(|id| yes_a.contains(id) || yes_b.contains(id));
        }
        a.extend(survivors);
    }
    Ok(original
        .iter()
        .filter_map(|c| {
            let id = c["memory_id"].as_str().unwrap();
            a.contains(id).then(|| id.to_owned())
        })
        .collect())
}
fn direct_perplexity(
    client: &OpenRouterClassifier,
    key: &str,
    f: &mut File,
    id: &str,
    row: &Value,
) -> Result<Vec<String>, String> {
    let c = row["candidates"].as_array().unwrap();
    let p = subset(&row["role_payload"], c, "perplexity/pplx-decider-v1.2");
    let body = request(client, key, f, id, "proposal", &p, c)?;
    let (a, b) = roles(&body, &p, c, "perplexity/pplx-decider-v1.2-20261016")?;
    Ok(c.iter()
        .filter_map(|c| {
            let id = c["memory_id"].as_str().unwrap();
            (a.contains(id) || b.contains(id)).then(|| id.to_owned())
        })
        .collect())
}
#[test]
#[ignore = "authorized synthetic full-context parallel critics:18pairs,max72HTTP,no retries"]
fn live_context_critic_probe() {
    run(2);
}
#[test]
#[ignore = "authorized synthetic sparse chain:18pairedknownqueries,max72HTTP,no retries,no ports"]
fn live_sparse_critic_probe() {
    run(1);
}
fn run(version: u8) {
    let parallel = version >= 2;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let d = Path::new(if version == 6 {
        "outputs/iromem/evaluation/direct-role-perplexity-v1-2026-10-20"
    } else if version == 4 {
        "outputs/iromem/evaluation/context-critics-http2-v4-2026-10-20"
    } else if version == 3 {
        "outputs/iromem/evaluation/context-targeted-critics-v3-2026-10-20"
    } else if parallel {
        "outputs/iromem/evaluation/context-role-critics-v2-2026-10-20"
    } else {
        DIR
    });
    let sha = |p: &Path| iromem_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let prep: Value =
        serde_json::from_slice(&fs::read(d.join("preparation.json")).unwrap()).unwrap();
    assert_eq!(prep["inputs_sha256"], sha(&d.join("inputs.jsonl")));
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
    }
    let mut hashes = serde_json::Map::new();
    for p in [
        "Cargo.toml",
        "Cargo.lock",
        "bootstrap/Cargo.toml",
        "bootstrap/src/openrouter.rs",
        "bootstrap/src/openrouter/per_memory/sparse_critic_probe.rs",
        "bootstrap/src/openrouter/per_memory/scoped_definition_probe.rs",
        "tools/evaluation/src/sparse_critic_probe.rs",
        "tools/evaluation/src/sparse_critic_probe/replacement.rs",
        "tools/evaluation/src/scoped_definition_probe.rs",
    ] {
        hashes.insert(p.into(), json!(sha(Path::new(p))));
        let archive = d.join("source-executed").join(p);
        fs::create_dir_all(archive.parent().unwrap()).unwrap();
        fs::copy(p, archive).unwrap();
    }
    let pre = json!({"preparation_sha256":sha(&d.join("preparation.json")),"inputs_sha256":prep["inputs_sha256"],"sources_sha256":hashes,"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"http2_feature_enabled":cfg!(feature="http2-transport"),"max_http_requests":if version==6{36}else{72},"fresh_validation":false,"promoted":false});
    let mut pf = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("preflight.json"))
        .unwrap();
    pf.write_all(serde_json::to_string_pretty(&pre).unwrap().as_bytes())
        .unwrap();
    pf.sync_all().unwrap();
    let key = iromem_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .unwrap();
    let client = OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("journal.jsonl"))
        .unwrap();
    let rows = fs::read_to_string(d.join("inputs.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 18);
    let mut completed = 0;
    'outer: for (n, row) in rows.iter().enumerate() {
        for variant in if n % 2 == 0 {
            ["control", "candidate"]
        } else {
            ["candidate", "control"]
        } {
            let id = format!("{}__{variant}", row["id"].as_str().unwrap());
            log(&mut f, json!({"kind":"operation_started","id":id}));
            let start = Instant::now();
            let result = if variant == "candidate" {
                if version == 6 {
                    direct_perplexity(&client, &key, &mut f, &id, row)
                } else if parallel {
                    parallel_candidate(&client, &key, &mut f, &id, row, version >= 3)
                } else {
                    candidate(&client, &key, &mut f, &id, row)
                }
            } else {
                let values = row["full_candidates"].as_array().unwrap();
                let candidates = values
                    .iter()
                    .map(|c| serde_json::from_value::<Candidate>(c.clone()).unwrap())
                    .collect::<Vec<_>>();
                let (opts, ids) = openrouter::criteria(&candidates);
                let p = openrouter::choice_payload(
                    row["role_payload"]["state"]["user_prompt"]
                        .as_str()
                        .unwrap(),
                    row["role_payload"]["state"]["scope"].as_str().unwrap(),
                    opts,
                    openrouter::INSTRUCTIONS,
                    ClassifierProvider::Decisions,
                );
                request(&client, &key, &mut f, &id, "baseline", &p, values).and_then(|body| {
                    if body["model"] != "openai/gpt-6-luna-decisions-20261006" {
                        return Err("unexpected baseline revision".into());
                    }
                    openrouter::parse(body, &ids)
                        .map(|d| d.selected_id.into_iter().collect())
                        .map_err(|e| e.code.into())
                })
            };
            let ok = result.is_ok();
            log(
                &mut f,
                json!({"kind":"operation_finished","id":id,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"status":if ok{"complete"}else{"failed"},"selected_ids":result.as_ref().ok(),"error":result.as_ref().err()}),
            );
            completed += 1;
            if !ok {
                break 'outer;
            }
        }
    }
    let mut status = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("runner-status.json"))
        .unwrap();
    status
        .write_all(
            serde_json::to_string_pretty(
                &json!({"completed_operations":completed,"stopped_early":completed!=36}),
            )
            .unwrap()
            .as_bytes(),
        )
        .unwrap();
    status.sync_all().unwrap();
}
#[test]
fn sparse_payload_exposes_only_remapped_sources_without_prior_judgments() {
    let template = json!({"model":"old","state":{"user_prompt":"q","scope":"s","rules":"r","memories":{"memory_00":"old","memory_01":"hidden"}},"questions":{"case_anchor":{"type":"choice"},"memory_00_relation":{"type":"choice","instructions":"Check state.memories.memory_00","criteria":{"A":"part","B":"refutation","none":"no"}},"memory_01_relation":{"type":"choice"}}});
    let c = vec![json!({"memory_id":"chosen","revision":3,"summary":"visible"})];
    let p = subset(&template, &c, "new");
    assert_eq!(p["state"]["memories"], json!({"memory_00":"visible"}));
    assert_eq!(p["questions"].as_object().unwrap().len(), 2);
    assert!(!p.to_string().contains("chosen"));
    assert!(!p.to_string().contains("hidden"));
    assert_eq!(p["state"]["rules"], template["state"]["rules"]);
}

#[test]
fn role_parser_is_atomic_and_keeps_literal_guard_with_exclusive_categories() {
    let candidates = vec![
        json!({"memory_id":"wrong","revision":1,"summary":"Neblina"}),
        json!({"memory_id":"right","revision":2,"summary":"Luciérnaga"}),
    ];
    let p = json!({"questions":{"case_anchor":{"criteria":{"A":"Luciérnaga","none":"unknown"}},"memory_00_relation":{"criteria":{"A":"part","B":"refutation","none":"no"}},"memory_01_relation":{"criteria":{"A":"part","B":"refutation","none":"no"}}}});
    let answer =
        |c: &str| json!({"type":"choice","choice":c,"probabilities":{"A":0.0,"B":1.0,"none":0.0}});
    let mut body = json!({"model":"test","usage":{"input_tokens":1,"output_tokens":0,"cost":0.001},"answers":{"case_anchor":{"type":"choice","choice":"A","probabilities":{"A":1.0,"none":0.0}},"memory_00_relation":answer("B"),"memory_01_relation":answer("B")}});
    let (a, b) = roles(&body, &p, &candidates, "test").unwrap();
    assert!(a.is_empty());
    assert_eq!(b, BTreeSet::from(["right".into()]));
    body["answers"]
        .as_object_mut()
        .unwrap()
        .remove("memory_01_relation");
    assert!(roles(&body, &p, &candidates, "test").is_err());
    assert!(roles(&body, &p, &candidates, "different").is_err());
}

#[test]
#[ignore = "authorized synthetic targeted critics with full context:18pairs,max72HTTP,no retries"]
fn live_targeted_critic_probe() {
    run(3);
}

#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic HTTP2 transport contrast:18pairs,max72HTTP,no retries"]
fn live_http2_critic_probe() {
    run(4);
}

#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic directRoleV5Perplexity:18pairs,36HTTP,no retries"]
fn live_direct_perplexity_probe() {
    run(6);
}
