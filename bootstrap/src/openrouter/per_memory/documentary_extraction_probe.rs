//! Manual synthetic extraction experiment; no application policy or automatic retries.
#[path = "../../../../tools/evaluation/src/documentary_extraction_contract.rs"]
#[cfg_attr(not(feature = "http2-transport"), allow(dead_code))]
mod contract;
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic source-only/question-only extraction,116HTTPmax,4concurrent,no retries"]
fn live_documentary_extraction_v1() {
    run(
        "outputs/iromem/evaluation/documentary-extraction-v1-2026-10-20",
        contract::Profile::NativeV1,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic JSONwireV2extraction,116HTTPmax,4concurrent,no retries"]
fn live_documentary_extraction_v2() {
    run(
        "outputs/iromem/evaluation/documentary-extraction-json-v2-2026-10-20",
        contract::Profile::JsonV2,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic native OpenAIpilotV3,8HTTPmax,no retries"]
fn live_documentary_extraction_openai_v3() {
    run(
        "outputs/iromem/evaluation/documentary-extraction-openai-v3-2026-10-20",
        contract::Profile::OpenAiPilotV3,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic native OpenAIpilotV4,8HTTPmax,no retries"]
fn live_documentary_extraction_openai_v4() {
    run(
        "outputs/iromem/evaluation/documentary-extraction-openai-v4-2026-10-20",
        contract::Profile::OpenAiPilotV4,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic guided OpenAIpilotV5,8HTTPmax,no retries"]
fn live_documentary_extraction_openai_guided_v5() {
    run(
        "outputs/iromem/evaluation/documentary-extraction-openai-guided-v5-2026-10-20",
        contract::Profile::OpenAiGuidedPilotV5,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic guided OpenAI JSONpilot,8HTTPmax,no retries"]
fn live_documentary_extraction_openai_guided_json() {
    run(
        "outputs/iromem/evaluation/documentary-extraction-openai-guided-json-v1-2026-10-20",
        contract::Profile::OpenAiGuidedJsonPilotV1,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic typedclause pilot,8HTTPmax,no retries"]
fn live_typed_clause_extraction_v1() {
    run(
        "outputs/iromem/evaluation/typed-clause-extraction-v1-2026-10-20",
        contract::Profile::TypedClausePilotV1,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic samecontract capacity contrast,8HTTPmax,no retries"]
fn live_typed_clause_capacity_v1() {
    run(
        "outputs/iromem/evaluation/typed-clause-capacity-v1-2026-10-20",
        contract::Profile::TypedClauseFullV1,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic grounded references,8HTTPmax,no retries"]
fn live_grounded_reference_v1() {
    run(
        "outputs/iromem/evaluation/grounded-reference-v1-2026-10-21",
        contract::Profile::GroundedReferenceV1,
    );
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "authorized synthetic mandatory revision of8completed drafts,no uncertain retries"]
fn live_grounded_revision_v1() {
    run(
        "outputs/iromem/evaluation/grounded-revision-v1-2026-10-21",
        contract::Profile::GroundedRevisionV1,
    );
}
#[cfg(feature = "http2-transport")]
fn run(directory: &str, profile: contract::Profile) {
    use serde_json::{Value, json};
    use std::{
        fs::{self, OpenOptions},
        io::{Read, Write},
        path::Path,
        time::{Duration, Instant},
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root).unwrap();
    let d = Path::new(directory);
    let read = |p: &Path| serde_json::from_slice::<Value>(&fs::read(p).unwrap()).unwrap();
    let sha = |p: &Path| iromem_kernel::digest::sha256_hex(&fs::read(p).unwrap());
    let save = |p: &Path, v: &Value| {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(p)
            .unwrap();
        f.write_all(&serde_json::to_vec_pretty(v).unwrap()).unwrap();
        f.sync_all().unwrap();
    };
    let prep = read(&d.join("preparation.json"));
    assert_eq!(prep["version"], "documentary-extraction-v1-preparation");
    assert_eq!(prep["max_http_requests"], profile.max_requests());
    assert_eq!(prep["max_concurrent"], 4);
    assert_eq!(prep["model"], profile.model());
    assert_eq!(prep["provider"], profile.provider());
    assert_eq!(
        contract::Profile::parse(&prep).expect("unknown wire profile; no call"),
        profile
    );
    for (n, key) in [
        ("requests.jsonl", "requests_sha256"),
        ("ontology.json", "ontology_sha256"),
        ("catalog-observation.json", "catalog_sha256"),
    ] {
        assert_eq!(prep[key], sha(&d.join(n)));
    }
    let catalog = read(&d.join("catalog-observation.json"));
    let endpoint = catalog["response"]["data"]["endpoints"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["tag"] == profile.tag() && e["model_id"] == profile.model())
        .expect("declared route absent");
    assert_eq!(endpoint["status"], 0);
    assert!(
        endpoint["supported_parameters"]
            .as_array()
            .unwrap()
            .contains(&json!("structured_outputs"))
    );
    assert_eq!(
        prep["bank_manifest_sha256"],
        sha(Path::new(
            "tests/fixtures/documentary-known100-v1/manifest.json"
        ))
    );
    let bank = Path::new("tests/fixtures/documentary-known100-v1");
    let manifest = read(&bank.join("manifest.json"));
    for (p, h) in manifest["files_sha256"].as_object().unwrap() {
        assert!(!p.contains('/') && !p.contains(".."));
        assert_eq!(*h, sha(&bank.join(p)));
    }
    for (p, h) in manifest["external_sha256"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
    }
    for (n, key) in [
        ("score.json", "manual_score_sha256"),
        ("report.json", "manual_raw_sha256"),
    ] {
        assert_eq!(
            prep[key],
            sha(&Path::new("outputs/iromem/evaluation/documentary-known100-v1-2026-10-20").join(n))
        );
    }
    let req = fs::read_to_string(d.join("requests.jsonl"))
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .collect::<Vec<_>>();
    let ontology = read(&d.join("ontology.json"));
    let input = read(&bank.join("source-only-reviewed.json"));
    assert_eq!(ontology, input["ontology"]);
    let request_profile = if profile == contract::Profile::GroundedRevisionV1 {
        contract::Profile::GroundedReferenceV1
    } else {
        profile
    };
    let mut sources = input["assertions"].as_object().unwrap().iter();
    let mut questions = input["questions"].as_object().unwrap().iter();
    let mut expected = Vec::new();
    loop {
        let a = sources.next();
        let q = questions.next();
        if a.is_none() && q.is_none() {
            break;
        }
        for (kind, item) in [("source", a), ("question", q)] {
            if let Some((target, v)) = item {
                let text = v["text"].as_str().unwrap();
                expected.push(json!({"id":format!("op_{:03}",expected.len()),"target":target,"kind":kind,"text":text,"payload":request_profile.payload(kind,text,&ontology).unwrap()}));
            }
        }
    }
    if profile.is_openai() {
        expected = contract::PILOT_IDS
            .iter()
            .map(|id| expected.iter().find(|r| r["id"] == *id).unwrap().clone())
            .collect();
    }
    if profile == contract::Profile::GroundedRevisionV1 {
        for (p, h) in contract::grounded_revision::PARENT_HASHES {
            assert_eq!(h, sha(Path::new(p)));
        }
        let rows = |name: &str| {
            fs::read_to_string(Path::new(contract::grounded_revision::PARENT).join(name))
                .unwrap()
                .lines()
                .map(|s| serde_json::from_str::<Value>(s).unwrap())
                .collect::<Vec<_>>()
        };
        expected = contract::grounded_revision::bind(
            expected,
            &rows("requests.jsonl"),
            &rows("journal.jsonl"),
            &ontology,
        )
        .unwrap();
    }
    for r in &req {
        contract::compatible_parameters(&r["payload"], endpoint)
            .expect("unsupported parameters: no call");
    }
    assert_eq!(req.len(), profile.max_requests());
    assert_eq!(
        req, expected,
        "roster/text/ontology/payload must reproduce bank without manual frames"
    );
    for (p, h) in prep["source_hashes"].as_object().unwrap() {
        assert_eq!(*h, sha(Path::new(p)));
        let target = d.join("source-executed").join(p);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        assert!(!target.exists());
        fs::copy(p, target).unwrap();
    }
    let key = iromem_kernel::config::load_secrets()
        .unwrap()
        .openrouter_api_key
        .expect("authorized existing key required");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    save(
        &d.join("preflight.json"),
        &json!({"version":"documentary-extraction-v1-preflight","preparation_sha256":sha(&d.join("preparation.json")),"source_hashes":prep["source_hashes"],"test_binary_sha256":sha(&std::env::current_exe().unwrap()),"max_http_requests":profile.max_requests(),"source_only_and_question_only_payloads_verified":true,"retries":0}),
    );
    let mut journal = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("journal.jsonl"))
        .expect("never retry a journal or uncertain operation");
    let start = Instant::now();
    let mut attempted = 0;
    let mut completed = 0;
    let mut fatal = false;
    let mut deadline = false;
    for batch in req.chunks(4) {
        if start.elapsed() > Duration::from_secs(300) {
            deadline = true;
            break;
        }
        for r in batch {
            writeln!(journal, "{}", json!({"kind":"started","id":r["id"]})).unwrap();
            attempted += 1;
        }
        journal.sync_all().unwrap();
        let events = std::thread::scope(|s| {
            let jobs=batch.iter().map(|r|{let client=&client;let key=&key;s.spawn(move||{
                let t=Instant::now();let received=(||->Result<(u16,String,Value),&str>{let resp=client.post("https://openrouter.ai/api/v1/chat/completions").bearer_auth(key).header("X-Title","iROMEM synthetic documentary extraction V1").json(&r["payload"]).send().map_err(|_|"transport_error")?;let status=resp.status().as_u16();let version=format!("{:?}",resp.version());let mut bytes=Vec::new();resp.take(1_048_577).read_to_end(&mut bytes).map_err(|_|"read_error")?;if bytes.len()>1_048_576{return Err("response_too_large");}let body=serde_json::from_slice(&bytes).map_err(|_|"invalid_http_json")?;Ok((status,version,body))})();
                match received{Ok((status,version,body))=>json!({"kind":"response","id":r["id"],"http_status":status,"http_version":version,"elapsed_ms":t.elapsed().as_secs_f64()*1000.,"body":body}),Err(error)=>json!({"kind":"failed","id":r["id"],"elapsed_ms":t.elapsed().as_secs_f64()*1000.,"error":error})}
            })}).collect::<Vec<_>>();
            jobs.into_iter()
                .map(|j| j.join().expect("uncertain worker: no retry"))
                .collect::<Vec<_>>()
        });
        for mut e in events {
            fatal |= e["kind"] != "response"
                || e["http_status"]
                    .as_u64()
                    .is_none_or(|n| !(200..300).contains(&n))
                || !profile.accepts_model(&e["body"]["model"])
                || e["body"]["provider"] != profile.provider()
                || e["body"]["choices"].as_array().is_none_or(|a| a.len() != 1)
                || e["body"]["choices"][0]["finish_reason"] != "stop"
                || e["body"]["choices"][0]["message"]
                    .get("refusal")
                    .is_some_and(|v| !v.is_null());
            if e["kind"] == "response" {
                let r = req.iter().find(|r| r["id"] == e["id"]).unwrap();
                let checked = contract::checked_content(&e["body"], &r["payload"]);
                e["wire_dto_shape_valid"] = json!(checked.is_ok());
                if let Err(error) = checked {
                    e["wire_dto_error"] = json!(error);
                }
            }
            writeln!(journal, "{e}").unwrap();
            completed += 1;
        }
        journal.sync_all().unwrap();
        eprintln!(
            "documentary extraction {completed}/{} fatal={fatal}",
            profile.max_requests()
        );
        if fatal {
            break;
        }
    }
    save(
        &d.join("runner-status.json"),
        &json!({"attempted_http_requests":attempted,"completed_http_requests":completed,"stopped_on_fatal":fatal,"deadline_reached":deadline,"elapsed_ms":start.elapsed().as_secs_f64()*1000.,"max_concurrent":4,"retry_count":0}),
    );
}
