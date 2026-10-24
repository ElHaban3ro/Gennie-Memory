//! Known-case paired research only; no retries, services or product policy.
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic contribution paired18:36HTTPmax,no retries"]
fn live_contribution_known_probe() {
    run(
        "outputs/iromem/evaluation/contribution-shared-known18-paired-v1-2026-10-20",
        false,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic local contribution paired18:36HTTPmax,no retries"]
fn live_contribution_local_known_probe() {
    run(
        "outputs/iromem/evaluation/contribution-local-v5-known18-paired-2026-10-20",
        true,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic Perplexity LocalV5paired18:36HTTPmax,no retries"]
fn live_contribution_perplexity_known() {
    run(
        "outputs/iromem/evaluation/contribution-local-perplexity-known18-paired-v1-2026-10-20",
        true,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic generative v2 paired18:36HTTPmax,no retries"]
fn live_contribution_gemini_known() {
    run(
        "outputs/iromem/evaluation/contribution-local-gemini-known18-paired-v1-2026-10-20",
        true,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic generative v3 examples paired18:36HTTPmax,no retries"]
fn live_contribution_gemini_examples_known() {
    run(
        "outputs/iromem/evaluation/contribution-local-gemini-examples-known18-paired-v1-2026-10-20",
        true,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic negative alternative triple18:54HTTPmax,no retries"]
fn live_contribution_negative_known() {
    run(
        "outputs/iromem/evaluation/contribution-local-perplexity-contrast-known18-triple-v1-2026-10-20",
        true,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic Spanish language triple18:54HTTPmax,no retries"]
fn live_contribution_spanish_known() {
    run(
        "outputs/iromem/evaluation/contribution-local-perplexity-spanish-known18-triple-v1-2026-10-20",
        true,
    );
}
#[cfg(feature = "http2-transport")]
fn run(directory: &str, local: bool) {
    use super::contribution_role_probe::contract;
    use super::sparse_critic_probe::log;
    use crate::openrouter;
    use gennie_memory_retrieval::model::{Candidate, ClassifierProvider};
    use serde_json::{Value, json};
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        path::Path,
        time::Instant,
    };
    std::env::set_current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()).unwrap();
    let d = Path::new(directory);
    let sha = |p: &Path| gennie_memory_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let read = |p: &Path| serde_json::from_slice::<Value>(&fs::read(p).unwrap()).unwrap();
    let prep = read(&d.join("preparation.json"));
    assert_eq!(prep["source_local_profile"] == true, local);
    let family = contract::Family::from_preparation(&prep).unwrap();
    assert_eq!(prep["inputs_sha256"], sha(&d.join("inputs.jsonl")));
    assert_eq!(prep["requests_sha256"], sha(&d.join("requests.jsonl")));
    assert_eq!(
        prep["bank_manifest_sha256"],
        sha(Path::new(
            "tests/fixtures/evidence-contribution-known18-v1/manifest.json"
        ))
    );
    let lines = |p: &Path| {
        fs::read_to_string(p)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .collect::<Vec<_>>()
    };
    let inputs = lines(&d.join("inputs.jsonl"));
    let requests = lines(&d.join("requests.jsonl"));
    assert_eq!(inputs.len(), 18);
    let planned = if family.reference().is_some() { 54 } else { 36 };
    assert_eq!(requests.len(), planned);
    let mut hashes = serde_json::Map::new();
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
        let dst = d.join("source-executed").join(p);
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(p, dst).unwrap();
        hashes.insert(p.clone(), h.clone());
    }
    for p in [
        "bootstrap/src/openrouter.rs",
        "bootstrap/src/openrouter/per_memory.rs",
        "bootstrap/src/openrouter/per_memory/contribution_role_probe.rs",
        "bootstrap/src/openrouter/per_memory/sparse_critic_probe.rs",
        "Cargo.toml",
        "Cargo.lock",
        "bootstrap/Cargo.toml",
    ] {
        let dst = d.join("source-executed").join(p);
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(p, dst).unwrap();
        hashes.insert(p.into(), json!(sha(Path::new(p))));
    }
    let mut pf = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("preflight.json"))
        .unwrap();
    pf.write_all(serde_json::to_string_pretty(&json!({"sources_sha256":hashes,"preparation_sha256":sha(&d.join("preparation.json")),"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"max_http_requests":planned,"http2_feature_enabled":true,"cfg_test_only":true})).unwrap().as_bytes()).unwrap();
    pf.sync_all().unwrap();
    let key = gennie_memory_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .unwrap();
    let client = openrouter::OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("journal.jsonl"))
        .unwrap();
    let (mut completed, mut stopped) = (0, false);
    for r in requests {
        let case = inputs.iter().find(|c| c["id"] == r["case_id"]).unwrap();
        let id = r["id"].as_str().unwrap();
        let candidate = r["variant"] != "control";
        let request_family = if r["variant"] == "reference" {
            family.reference().expect("reference family")
        } else {
            family
        };
        let originals = case["full_candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| serde_json::from_value::<Candidate>(c.clone()).unwrap())
            .collect::<Vec<_>>();
        let (options, ids) = openrouter::criteria(&originals);
        let p = if candidate {
            request_family.payload(
                if local {
                    contract::Profile::Local
                } else {
                    contract::Profile::Shared
                },
                case["question"].as_str().unwrap(),
                case["memories"].as_array().unwrap(),
            )
        } else {
            openrouter::choice_payload(
                case["question"].as_str().unwrap(),
                case["scope_id"].as_str().unwrap(),
                options,
                openrouter::INSTRUCTIONS,
                ClassifierProvider::Decisions,
            )
        };
        assert_eq!(p, r["payload"]);
        log(&mut f, json!({"kind":"started","id":id}));
        let start = Instant::now();
        let response = if matches!(
            family,
            contract::Family::Gemini | contract::Family::GeminiExamples
        ) && candidate
        {
            super::contribution_role_probe::request_generative(&client, &key, &p)
        } else {
            client.request_measured(&key, &p)
        };
        let event = match response {
            Ok((body, t)) => {
                let check = if candidate {
                    contract::decode_family(
                        &body,
                        &p,
                        case["memories"].as_array().unwrap(),
                        request_family,
                    )
                    .map(|_| ())
                } else if body["model"] != "openai/gpt-6-luna-decisions-20261006" {
                    Err("unexpected baseline revision".into())
                } else {
                    openrouter::parse(body.clone(), &ids)
                        .map(|_| ())
                        .map_err(|e| e.code.to_owned())
                };
                stopped = check.is_err();
                json!({"kind":"response","id":id,"body":body,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"http_version":t.version,"validation_error":check.err()})
            }
            Err(e) => {
                stopped = true;
                json!({"kind":"failed","id":id,"code":e.code,"http_status":e.http_status,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        log(&mut f, event);
        completed += 1;
        if stopped {
            break;
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
                &json!({"completed_http_requests":completed,"stopped":stopped}),
            )
            .unwrap()
            .as_bytes(),
        )
        .unwrap();
    status.sync_all().unwrap();
}
