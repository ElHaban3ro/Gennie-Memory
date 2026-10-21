//! Research only; no product policy or automatic retry.
#[path = "../../../../tools/evaluation/src/contribution_role_contract.rs"]
#[cfg_attr(not(feature = "http2-transport"), allow(dead_code))]
pub(super) mod contract;

#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic contribution16,16HTTPmax,no retries"]
fn live_contribution_role_probe() {
    run(
        "outputs/iromem/evaluation/contribution-role-v3-minimal-2026-10-20",
        contract::Profile::Initial,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic contribution16,16HTTPmax,no retries"]
fn live_contribution_complete_probe() {
    run(
        "outputs/iromem/evaluation/contribution-role-complete-v2-2026-10-20",
        contract::Profile::Complete,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic contribution16,16HTTPmax,no retries"]
fn live_contribution_strict_probe() {
    run(
        "outputs/iromem/evaluation/contribution-role-strict-v3-2026-10-20",
        contract::Profile::Strict,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic contribution16,16HTTPmax,no retries"]
fn live_contribution_shared_probe() {
    run(
        "outputs/iromem/evaluation/contribution-role-shared-v4-2026-10-20",
        contract::Profile::Shared,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic local contribution16,16HTTPmax,no retries"]
fn live_contribution_local_probe() {
    run(
        "outputs/iromem/evaluation/contribution-local-v5-minimal-2026-10-20",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic Microsoft LocalV5minimum16,16HTTPmax,no retries"]
fn live_contribution_microsoft_minimal() {
    run(
        "outputs/iromem/evaluation/contribution-local-microsoft-minimal-v1-2026-10-20",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic Perplexity LocalV5minimum16,16HTTPmax,no retries"]
fn live_contribution_perplexity_minimal() {
    run(
        "outputs/iromem/evaluation/contribution-local-perplexity-minimal-v1-2026-10-20",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic generative v2 minimum16,16HTTPmax,no retries"]
fn live_contribution_gemini_minimal() {
    run(
        "outputs/iromem/evaluation/contribution-local-gemini-minimal-v1-2026-10-20",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic generative v3 examples minimum16,16HTTPmax,no retries"]
fn live_contribution_gemini_examples_minimal() {
    run(
        "outputs/iromem/evaluation/contribution-local-gemini-examples-minimal-v1-2026-10-20",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic explicit negative alternatives16,16HTTPmax,no retries"]
fn live_contribution_negative_minimal() {
    run(
        "outputs/iromem/evaluation/contribution-local-perplexity-contrast-minimal-v1-2026-10-20",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic Microsoft negative alternatives16,16HTTPmax,no retries"]
fn live_contribution_microsoft_negative_minimal() {
    run(
        "outputs/iromem/evaluation/contribution-local-microsoft-contrast-minimal-v1-2026-10-20",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic Spanish instructions16,16HTTPmax,no retries"]
fn live_contribution_spanish_minimal() {
    run(
        "outputs/iromem/evaluation/contribution-local-perplexity-spanish-minimal-v1-2026-10-20",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic Gemma4B classifier16,16HTTPmax,USD0.005,no retries"]
fn live_contribution_gemma_minimal() {
    run(
        "outputs/iromem/evaluation/contribution-gemma-minimal-v1-2026-10-22",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic Qwen9B classifier16,16HTTPmax,USD0.01,no retries"]
fn live_contribution_qwen_minimal() {
    run(
        "outputs/iromem/evaluation/contribution-qwen9b-minimal-v1-2026-10-22",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized separate recovered Qwen classifier attempt16,USD0.01,no retries"]
fn live_contribution_qwen_recovered_attempt() {
    run(
        "outputs/iromem/evaluation/contribution-qwen9b-minimal-attempt2-2026-10-22",
        contract::Profile::Local,
    );
}
#[cfg(feature = "http2-transport")]
pub(super) fn request_generative(
    client: &crate::openrouter::OpenRouterClassifier,
    key: &str,
    payload: &serde_json::Value,
) -> Result<
    (serde_json::Value, crate::openrouter::HttpMeasurement),
    iromem_retrieval::classifier::DecisionError,
> {
    request_generative_observed(client, key, payload, None)
}
#[cfg(feature = "http2-transport")]
pub(super) fn request_generative_observed(
    client: &crate::openrouter::OpenRouterClassifier,
    key: &str,
    payload: &serde_json::Value,
    error_path: Option<&std::path::Path>,
) -> Result<
    (serde_json::Value, crate::openrouter::HttpMeasurement),
    iromem_retrieval::classifier::DecisionError,
> {
    use crate::openrouter::{HttpMeasurement, RESPONSE_LIMIT, failure, http_failure_with_body};
    use std::{io::Read, time::Instant};
    let start = Instant::now();
    let response = client
        .client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .timeout(crate::openrouter::REQUEST_TIMEOUT)
        .bearer_auth(key)
        .header(
            "X-Title",
            "iROMEM synthetic generative contribution research",
        )
        .json(payload)
        .send()
        .map_err(|_| failure("transport_error"))?;
    let status = response.status();
    let version = format!("{:?}", response.version());
    let mut bytes = Vec::new();
    let result = response.take(RESPONSE_LIMIT + 1).read_to_end(&mut bytes);
    if !status.is_success() {
        if let Some(path) = error_path {
            let text = String::from_utf8_lossy(&bytes).replace(key, "[redacted]");
            let parsed = serde_json::from_str::<serde_json::Value>(&text).ok();
            let snapshot = serde_json::json!({"http_status":status.as_u16(),"http_version":version,"read_success":result.is_ok(),"body_truncated":bytes.len()as u64>RESPONSE_LIMIT,"error":parsed.as_ref().and_then(|p|p.get("error")),"non_json_excerpt":if parsed.is_none(){Some(text.chars().take(2048).collect::<String>())}else{None}});
            if persist_http_error_snapshot(path, &snapshot).is_err() {
                eprintln!(
                    "No se pudo guardar el diagnóstico HTTP; el fallo y su recibo se conservan en el diario."
                );
            }
        }
        return Err(http_failure_with_body(
            status.as_u16(),
            result.is_ok().then_some(bytes.as_slice()),
        ));
    }
    result.map_err(|_| failure("transport_error"))?;
    if bytes.len() as u64 > RESPONSE_LIMIT {
        return Err(failure("response_too_large"));
    }
    let body = serde_json::from_slice(&bytes).map_err(|_| failure("invalid_response"))?;
    Ok((
        body,
        HttpMeasurement {
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.,
            version,
        },
    ))
}
#[cfg(feature = "http2-transport")]
fn run(directory: &str, profile: contract::Profile) {
    use super::sparse_critic_probe::log;
    use crate::openrouter::OpenRouterClassifier;
    use serde_json::{Value, json};
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        path::Path,
        time::Instant,
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let d = Path::new(directory);
    let sha = |p: &Path| iromem_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let prep: Value =
        serde_json::from_slice(&fs::read(d.join("preparation.json")).unwrap()).unwrap();
    assert_eq!(contract::Profile::from_preparation(&prep).unwrap(), profile);
    let family = contract::Family::from_preparation(&prep).unwrap();
    assert_eq!(prep["inputs_sha256"], sha(&d.join("inputs.jsonl")));
    assert_eq!(prep["requests_sha256"], sha(&d.join("requests.jsonl")));
    let read_lines = |p: &Path| {
        fs::read_to_string(p)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .collect::<Vec<_>>()
    };
    let requests = read_lines(&d.join("requests.jsonl"));
    let cases = read_lines(&d.join("inputs.jsonl"));
    assert_eq!(requests.len(), 16);
    assert_eq!(cases.len(), 16);
    let mut sources = serde_json::Map::new();
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
        let dst = d.join("source-executed").join(p);
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(p, dst).unwrap();
        sources.insert(p.clone(), h.clone());
    }
    for p in [
        "bootstrap/src/openrouter.rs",
        "bootstrap/src/openrouter/per_memory.rs",
        "bootstrap/src/openrouter/per_memory/sparse_critic_probe.rs",
        "Cargo.toml",
        "Cargo.lock",
        "bootstrap/Cargo.toml",
    ] {
        let dst = d.join("source-executed").join(p);
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(p, dst).unwrap();
        sources.insert(p.into(), json!(sha(Path::new(p))));
    }
    let mut pf = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("preflight.json"))
        .unwrap();
    pf.write_all(serde_json::to_string_pretty(&json!({"preparation_sha256":sha(&d.join("preparation.json")),"sources_sha256":sources,"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"max_http_requests":16,"http2_feature_enabled":true,"cfg_test_only":true})).unwrap().as_bytes()).unwrap();
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
    let gemma = family == contract::Family::Gemma;
    let qwen = family == contract::Family::Qwen;
    let bounded = gemma || qwen;
    let max_cost = if qwen { 0.01 } else { 0.005 };
    let estimate = |payload: &Value| {
        (payload.to_string().len() as f64 + 1024.) * (if qwen { 0.0000001 } else { 0.00000004 })
            + 128. * (if qwen { 0.00000015 } else { 0.00000005 })
    };
    if bounded {
        assert!(
            requests
                .iter()
                .map(|r| estimate(&r["payload"]))
                .sum::<f64>()
                <= max_cost
        );
    }
    let mut known_cost = 0.;
    let (mut completed, mut stopped) = (0, false);
    for (r, c) in requests.iter().zip(&cases) {
        assert_eq!(r["id"], c["id"]);
        assert_eq!(
            r["payload"],
            family.payload(
                profile,
                c["question"].as_str().unwrap(),
                c["memories"].as_array().unwrap()
            )
        );
        if bounded && known_cost + estimate(&r["payload"]) > max_cost {
            stopped = true;
            break;
        }
        let id = r["id"].as_str().unwrap();
        log(&mut f, json!({"kind":"started","id":id}));
        let start = Instant::now();
        let response = if matches!(
            family,
            contract::Family::Gemini
                | contract::Family::GeminiExamples
                | contract::Family::Gemma
                | contract::Family::Qwen
        ) {
            if qwen && directory.contains("attempt2") {
                let path = d.join(format!("{id}-http-error.json"));
                request_generative_observed(&client, &key, &r["payload"], Some(&path))
            } else {
                request_generative(&client, &key, &r["payload"])
            }
        } else {
            client.request_measured(&key, &r["payload"])
        };
        let event = match response {
            Ok((body, t)) => {
                let validation = contract::decode_family(
                    &body,
                    &r["payload"],
                    c["memories"].as_array().unwrap(),
                    family,
                );
                stopped = validation.is_err();
                json!({"kind":"response","id":id,"body":body,"http_version":t.version,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"validation_error":validation.err()})
            }
            Err(e) => {
                stopped = true;
                json!({"kind":"failed","id":id,"code":e.code,"http_status":e.http_status,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"partial_usage":e.partial_usage.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd}))})
            }
        };
        if bounded {
            match event["body"]["usage"]["cost"]
                .as_f64()
                .filter(|x| x.is_finite() && *x >= 0.)
            {
                Some(cost) => {
                    known_cost += cost;
                    if known_cost > max_cost {
                        stopped = true;
                    }
                }
                None => stopped = true,
            }
        }
        log(&mut f, event);
        completed += 1;
        if stopped {
            break;
        }
    }
    let mut status = OpenOptions::new()
        .create_new(true)
        .write(true)
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

/// Single neutral transport observation; never counted as a quality benchmark.
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized one neutral classifier transport diagnostic,USD0.0001 estimate,no retry"]
fn live_qwen_neutral_transport_diagnostic() {
    run_neutral_transport("request.json", "rust-transport.jsonl", 1);
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized one neutral structured classifier transport diagnostic,USD0.001 estimate,no retry"]
fn live_qwen_neutral_structured_transport_diagnostic() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    std::env::set_current_dir(root).unwrap();
    let memories = vec![
        serde_json::json!({"id":"one","revision":1,"text":"Lamp Orbe is green."}),
        serde_json::json!({"id":"two","revision":1,"text":"Lamp Orbe is stored in a cupboard."}),
        serde_json::json!({"id":"three","revision":1,"text":"Lamp Vega is red."}),
        serde_json::json!({"id":"four","revision":1,"text":"The color of Lamp Orbe is absent from this register."}),
    ];
    let p = contract::Family::Qwen.payload(
        contract::Profile::Local,
        "What color is Lamp Orbe?",
        &memories,
    );
    let file=std::fs::OpenOptions::new().write(true).create_new(true).open("outputs/iromem/evaluation/openrouter-classifier-429-diagnosis-2026-10-22/structured-request.json").unwrap();
    serde_json::to_writer_pretty(file, &p).unwrap();
    run_neutral_transport(
        "structured-request.json",
        "rust-structured-transport.jsonl",
        128,
    );
}
#[cfg(feature = "http2-transport")]
fn run_neutral_transport(request_file: &str, output_file: &str, max_tokens: u64) {
    use crate::openrouter::OpenRouterClassifier;
    use serde_json::{Value, json};
    use std::{
        fs,
        fs::OpenOptions,
        io::{Read, Write},
        path::Path,
        time::Instant,
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let d = Path::new("outputs/iromem/evaluation/openrouter-classifier-429-diagnosis-2026-10-22");
    let payload: Value = serde_json::from_slice(&fs::read(d.join(request_file)).unwrap()).unwrap();
    assert_eq!(payload["model"], "qwen/qwen3.5-9b");
    assert_eq!(payload["provider"]["only"], json!(["deepinfra"]));
    assert_eq!(payload["max_tokens"], max_tokens);
    assert_eq!(payload["reasoning"]["enabled"], false);
    let mut out = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join(output_file))
        .unwrap();
    super::sparse_critic_probe::log(
        &mut out,
        json!({"kind":"started","purpose":"single neutral transport observation, not benchmark retry","request_sha256":iromem_kernel::digest::sha256_hex(&fs::read(d.join(request_file)).unwrap()),"max_http_requests":1,"estimated_max_cost_usd":if max_tokens==1 {0.0001} else {0.001}}),
    );
    let key = iromem_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .unwrap();
    let client = OpenRouterClassifier::new(Some(key.clone())).unwrap();
    let start = Instant::now();
    let result = client
        .client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .timeout(crate::openrouter::REQUEST_TIMEOUT)
        .bearer_auth(&key)
        .header("X-Title", "iROMEM one classifier transport diagnostic")
        .json(&payload)
        .send();
    let event = match result {
        Ok(response) => {
            let status = response.status().as_u16();
            let version = format!("{:?}", response.version());
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned);
            let mut bytes = Vec::new();
            let read = response.take(65537).read_to_end(&mut bytes);
            let text = String::from_utf8_lossy(&bytes).replace(&key, "[redacted]");
            let parsed = serde_json::from_str::<Value>(&text).ok();
            json!({"kind":"response","http_status":status,"http_version":version,"retry_after":retry_after,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"read_success":read.is_ok(),"body_truncated":bytes.len()>65536,"body":parsed,"non_json_excerpt":if parsed.is_none(){Some(text.chars().take(2048).collect::<String>())}else{None}})
        }
        Err(_) => {
            json!({"kind":"failed","code":"transport_error","elapsed_ms":start.elapsed().as_secs_f64()*1000.})
        }
    };
    super::sparse_critic_probe::log(&mut out, event);
    out.flush().unwrap();
}

#[cfg(feature = "http2-transport")]
fn persist_http_error_snapshot(
    path: &std::path::Path,
    snapshot: &serde_json::Value,
) -> std::io::Result<()> {
    use std::io::Write;
    let bytes = serde_json::to_vec_pretty(snapshot)
        .map_err(|_| std::io::Error::other("diagnostic JSON encoding failed"))?;
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    file.write_all(&bytes)?;
    file.sync_all()
}
#[cfg(feature = "http2-transport")]
#[test]
fn failed_observation_write_does_not_replace_existing_evidence_or_panic() {
    let d = std::env::temp_dir().join(format!(
        "iromem-error-observation-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&d).unwrap();
    let p = d.join("error.json");
    persist_http_error_snapshot(&p, &serde_json::json!({"http_status":429})).unwrap();
    let original = std::fs::read(&p).unwrap();
    assert!(persist_http_error_snapshot(&p, &serde_json::json!({"http_status":500})).is_err());
    assert_eq!(std::fs::read(&p).unwrap(), original);
    assert!(
        persist_http_error_snapshot(&d.join("absent/error.json"), &serde_json::json!({})).is_err()
    );
    std::fs::remove_dir_all(d).unwrap();
}
