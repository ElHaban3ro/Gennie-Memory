//! Manual, ignored, TRAIN-only diagnostic. No product policy or automatic retry.
#[path = "../../../../tools/evaluation/src/semantic_link_contract.rs"]
#[allow(dead_code)]
mod contract;
use contract as semantic_link_contract;
#[path = "../../../../tools/evaluation/src/semantic_link_span_contract.rs"]
#[allow(dead_code)]
mod span_contract;
use span_contract as semantic_link_span_contract;
#[path = "../../../../tools/evaluation/src/semantic_link_direct_contract.rs"]
#[allow(dead_code)]
mod direct_contract;
#[path = "../../../../tools/evaluation/src/semantic_link_local_contract.rs"]
#[allow(dead_code)]
mod local_contract;
use serde_json::{Value, json};

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum Interface {
    LiteralV1,
    SpansV2,
    LocalV3,
    DirectV4,
}
#[allow(dead_code)]
struct CheckedCard {
    artifact: Value,
    selected: bool,
    unresolved: bool,
}
#[allow(dead_code)]
impl Interface {
    fn directory(self) -> &'static str {
        match self {
            Self::LiteralV1 => "outputs/iromem/evaluation/semantic-link-flash-lite-v1-2026-10-22",
            Self::SpansV2 => "outputs/iromem/evaluation/semantic-link-span-v2-2026-10-22",
            Self::LocalV3 => "outputs/iromem/evaluation/semantic-link-span-v3-2026-10-22",
            Self::DirectV4 => "outputs/iromem/evaluation/semantic-link-direct-v4-2026-10-22",
        }
    }
    fn plan_version(self) -> &'static str {
        match self {
            Self::LiteralV1 => "semantic-link-flash-lite-v1",
            Self::SpansV2 => "semantic-link-span-v2",
            Self::LocalV3 => "semantic-link-span-v3",
            Self::DirectV4 => "semantic-link-direct-v4",
        }
    }
    fn completion_limit(self) -> u64 {
        match self {
            Self::DirectV4 => direct_contract::MAX_TOKENS,
            _ => contract::MAX_TOKENS,
        }
    }
    fn index_profile(self) -> span_contract::IndexProfile {
        match self {
            Self::DirectV4 => span_contract::IndexProfile::DocumentOnly,
            _ => span_contract::IndexProfile::BothSources,
        }
    }
    fn payload(self, query: &str, document: &str, id: &str) -> Value {
        match self {
            Self::LiteralV1 => contract::payload(query, document, id),
            Self::SpansV2 => span_contract::payload(query, document, id),
            Self::LocalV3 => local_contract::payload(query, document, id),
            Self::DirectV4 => direct_contract::payload(query, document, id),
        }
    }
    fn decode(
        self,
        body: &Value,
        query: &str,
        document: &str,
        id: &str,
    ) -> Result<CheckedCard, String> {
        match self {
            Self::LiteralV1 => {
                let (card, _) = contract::decode_body(body, query, document, id)?;
                Ok(CheckedCard {
                    selected: card.selected(),
                    unresolved: card.unresolved(),
                    artifact: serde_json::to_value(card).map_err(|e| e.to_string())?,
                })
            }
            Self::SpansV2 => {
                let (card, _) = span_contract::decode_body(body, query, document, id)?;
                Ok(CheckedCard {
                    selected: card.selected(),
                    unresolved: card.unresolved(),
                    artifact: serde_json::to_value(card).map_err(|e| e.to_string())?,
                })
            }
            Self::LocalV3 => {
                let (card, _) = local_contract::decode_body(body, query, document, id)?;
                Ok(CheckedCard {
                    selected: card.selected(),
                    unresolved: card.unresolved(),
                    artifact: serde_json::to_value(card).map_err(|e| e.to_string())?,
                })
            }
            Self::DirectV4 => {
                let (card, _) = direct_contract::decode_body(body, query, document, id)?;
                Ok(CheckedCard {
                    selected: card.selected(),
                    unresolved: card.unresolved(),
                    artifact: serde_json::to_value(card).map_err(|e| e.to_string())?,
                })
            }
        }
    }
}

fn budget_allows(spent: f64, next: f64, cap: f64) -> bool {
    spent.is_finite()
        && spent >= 0.
        && next.is_finite()
        && next > 0.
        && cap.is_finite()
        && cap > 0.
        && cap <= 0.50
        && spent + next <= cap
}
fn pilot_regression(label: bool, parent: bool, selected: bool) -> bool {
    (label && parent && !selected) || (!label && !parent && selected)
}
fn verify_binary_binding(plan: &Value, actual_sha256: &str) -> Result<(), String> {
    if plan["test_binary_sha256"].as_str() != Some(actual_sha256)
        || actual_sha256.len() != 64
        || !actual_sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("missing_or_mismatched_test_binary_sha256".into());
    }
    Ok(())
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "manual frozen TRAIN432 semantic-link pilot24 then408; max432HTTP,USD0.50,900seconds,no retries"]
fn live_semantic_link_flash_lite_v1() {
    run(Interface::LiteralV1)
        .expect("La sonda se detuvo; conservar artefactos y no reintentar el mismo intento");
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "manual frozen TRAIN432 span-link pilot24 then408; max432HTTP,USD0.50,900seconds,no retries"]
fn live_semantic_link_span_v2() {
    run(Interface::SpansV2)
        .expect("La sonda se detuvo; conservar artefactos y no reintentar el mismo intento");
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "manual frozen TRAIN432 local-policy span-link pilot24 then408; max432HTTP,USD0.50,900seconds,no retries"]
fn live_semantic_link_span_v3() {
    run(Interface::LocalV3)
        .expect("La sonda se detuvo; conservar artefactos y no reintentar el mismo intento");
}
#[cfg(feature = "http2-transport")]
#[test]
#[ignore = "manual frozen TRAIN432 direct judgment pilot24 then408; max432HTTP,USD0.50,900seconds,no retries"]
fn live_semantic_link_direct_v4() {
    run(Interface::DirectV4)
        .expect("La sonda se detuvo; conservar artefactos y no reintentar el mismo intento");
}

#[cfg(feature = "http2-transport")]
fn run(interface: Interface) -> Result<(), String> {
    use super::contribution_role_probe::request_generative_observed;
    use crate::openrouter::OpenRouterClassifier;
    use std::{
        fs::{self, File, OpenOptions},
        io::Write,
        path::Path,
        time::Instant,
    };
    const SOURCES: [&str; 7] = [
        "tools/evaluation/src/semantic_link_contract.rs",
        "bootstrap/src/openrouter/per_memory/semantic_link_probe.rs",
        "tools/evaluation/src/main.rs",
        "bootstrap/src/openrouter/per_memory.rs",
        "bootstrap/src/openrouter/per_memory/contribution_role_probe.rs",
        "bootstrap/src/openrouter.rs",
        "Cargo.lock",
    ];
    fn sha(path: &Path) -> Result<String, String> {
        fs::read(path)
            .map(|b| gennie_memory_kernel::digest::sha256_hex(&b))
            .map_err(|e| e.to_string())
    }
    fn read(path: &Path) -> Result<Value, String> {
        serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }
    fn save(path: &Path, v: &Value) -> Result<(), String> {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        f.write_all(&serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())
    }
    fn utc_now() -> String {
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let seconds = elapsed.as_secs();
        let z = (seconds / 86400) as i64 + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let mut year = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = mp + if mp < 10 { 3 } else { -9 };
        year += i64::from(month <= 2);
        format!(
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
            seconds % 86400 / 3600,
            seconds % 3600 / 60,
            seconds % 60,
            elapsed.subsec_millis()
        )
    }
    fn event(f: &mut File, seq: &mut usize, mut v: Value) -> Result<(), String> {
        v["seq"] = json!(*seq);
        *seq += 1;
        v["timestamp_utc"] = json!(utc_now());
        writeln!(f, "{v}").map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())
    }
    fn verify(plan: &Value, d: &Path, interface: Interface) -> Result<(), String> {
        let sources = plan["source_hashes"]
            .as_object()
            .ok_or("source_hashes absent")?;
        for p in SOURCES {
            if !sources.contains_key(p) {
                return Err(format!("missing source binding: {p}"));
            }
        }
        for (p, h) in sources {
            if h.as_str() != Some(sha(Path::new(p))?.as_str()) {
                return Err(format!("source changed: {p}"));
            }
        }
        let files = plan["file_hashes"]
            .as_object()
            .ok_or("file_hashes absent")?;
        if interface != Interface::LiteralV1
            && (!sources.contains_key("tools/evaluation/src/semantic_link_span_contract.rs")
                || !files.contains_key("span-index.json"))
        {
            return Err("missing_span_source_or_index_binding".into());
        }
        if interface == Interface::LocalV3
            && !sources.contains_key("tools/evaluation/src/semantic_link_local_contract.rs")
        {
            return Err("missing_local_policy_source_binding".into());
        }
        if interface == Interface::DirectV4
            && !sources.contains_key("tools/evaluation/src/semantic_link_direct_contract.rs")
        {
            return Err("missing_direct_contract_source_binding".into());
        }
        for (p, h) in files {
            if Path::new(p).is_absolute()
                || Path::new(p)
                    .components()
                    .any(|c| !matches!(c, std::path::Component::Normal(_)))
            {
                return Err("unsafe frozen file path".into());
            }
            if h.as_str() != Some(sha(&d.join(p))?.as_str()) {
                return Err(format!("asset changed: {p}"));
            }
        }
        for (name, key) in [
            ("inputs.json", "inputs_sha256"),
            ("payloads.json", "payloads_sha256"),
        ] {
            if plan[key].as_str() != Some(sha(&d.join(name))?.as_str()) {
                return Err(format!("input changed: {name}"));
            }
        }
        Ok(())
    }
    let started = Instant::now();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("workspace absent")?;
    std::env::set_current_dir(root).map_err(|e| e.to_string())?;
    let d = Path::new(interface.directory());
    for name in [
        "run-marker.json",
        "journal.jsonl",
        "runner-status.json",
        "rows.json",
        "source-executed",
        "raw-bodies",
    ] {
        if d.join(name).exists() {
            return Err(format!("attempt already exists: {name}"));
        }
    }
    let plan = read(&d.join("plan.json"))?;
    if plan["version"] != interface.plan_version()
        || plan["max_http_requests"] != 432
        || plan["pilot_count"] != 24
        || plan["deadline_seconds"] != 900
    {
        return Err("unexpected frozen protocol limits".into());
    }
    let cap = plan["max_cost_usd"].as_f64().ok_or("budget missing")?;
    if !cap.is_finite() || cap <= 0. || cap > 0.50 {
        return Err("budget exceeds hard ceiling".into());
    }
    let plan_sha = sha(&d.join("plan.json"))?;
    let preflight = read(&d.join("technical-preflight.json"))?;
    if preflight["all_passed"] != true || preflight["plan_sha256"] != plan_sha {
        return Err("technical preflight not passed for this plan".into());
    }
    verify(&plan, d, interface)?;
    let test_binary_sha256 = sha(&std::env::current_exe().map_err(|e| e.to_string())?)?;
    verify_binary_binding(&plan, &test_binary_sha256)?;
    let rows: Vec<contract::InputRow> =
        serde_json::from_slice(&fs::read(d.join("inputs.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    contract::validate_inputs(&rows)?;
    if interface != Interface::LiteralV1 {
        let expected: Vec<_> = rows
            .iter()
            .map(|r| {
                span_contract::index_row(&r.id, &r.query, &r.document, interface.index_profile())
            })
            .collect();
        if read(&d.join("span-index.json"))? != json!(expected) {
            return Err("span_index_differs_from_full_sources".into());
        }
    }
    let requests = read(&d.join("payloads.json"))?;
    let requests = requests
        .as_array()
        .filter(|a| a.len() == 432)
        .ok_or("expected 432 payloads")?;
    let bounds = plan["request_cost_upper_bounds_usd"]
        .as_array()
        .filter(|a| a.len() == 432)
        .ok_or("432 upper bounds required")?;
    for ((r, p), b) in rows.iter().zip(requests).zip(bounds) {
        if p.as_object().is_none_or(|o| o.len() != 2)
            || p["id"] != r.id
            || p["payload"] != interface.payload(&r.query, &r.document, &r.id)
            || p["payload"]["max_tokens"].as_u64() != Some(interface.completion_limit())
            || b.as_f64()
                .is_none_or(|v| !v.is_finite() || v <= 0. || v > cap)
        {
            return Err("payload or next-request upper bound mismatch".into());
        }
    }
    // Atomic exclusive marker precedes credentials and journal; a crash cannot authorize a rerun.
    save(
        &d.join("run-marker.json"),
        &json!({"plan_sha256":plan_sha,"test_binary_sha256":test_binary_sha256,"pid":std::process::id(),"timestamp_utc":utc_now(),"retry_count":0}),
    )?;
    for (p, _) in plan["source_hashes"].as_object().unwrap() {
        let src = Path::new(p);
        if src.is_absolute()
            || src
                .components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err("unsafe source snapshot path".into());
        }
        let dst = d.join("source-executed").join(src);
        fs::create_dir_all(dst.parent().unwrap()).map_err(|e| e.to_string())?;
        let bytes = fs::read(src).map_err(|e| e.to_string())?;
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dst)
            .map_err(|e| e.to_string())?;
        f.write_all(&bytes).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
    }
    fs::create_dir(d.join("raw-bodies")).map_err(|e| e.to_string())?;
    let mut journal = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("journal.jsonl"))
        .map_err(|e| e.to_string())?;
    let mut seq = 0;
    let key = gennie_memory_kernel::config::load_secrets()
        .map_err(|_| "secret configuration unavailable")?
        .openrouter_api_key
        .ok_or("credential unavailable")?;
    let client =
        OpenRouterClassifier::new(Some(key.clone())).map_err(|_| "client initialization failed")?;
    let mut checked = Vec::<Value>::new();
    let mut attempted = 0;
    let mut responses = 0;
    let mut spent = 0.;
    let mut unknown_cost = 0;
    let mut pilot_failed = false;
    let mut pilot_passed = false;
    let mut stop_reason: Option<String> = None;
    for (i, ((row, request), bound)) in rows.iter().zip(requests).zip(bounds).enumerate() {
        if started.elapsed().as_secs_f64() + 20. > 900. {
            stop_reason = Some("phase_deadline_before_next_request".into());
            break;
        }
        if !budget_allows(spent, bound.as_f64().unwrap(), cap) {
            stop_reason = Some("budget_before_next_request".into());
            break;
        }
        if let Err(error) = verify(&plan, d, interface) {
            stop_reason = Some(error);
            break;
        }
        event(
            &mut journal,
            &mut seq,
            json!({"kind":"started","index":i,"id":row.id,"spent_known_usd":spent,"next_cost_upper_bound_usd":bound,"payload_sha256":gennie_memory_kernel::digest::sha256_hex(&serde_json::to_vec(&request["payload"]).unwrap())}),
        )?;
        attempted += 1;
        let error_path = d
            .join("raw-bodies")
            .join(format!("{}-http-error.json", row.id));
        match request_generative_observed(&client, &key, &request["payload"], Some(&error_path)) {
            Err(error) => {
                let partial = error.partial_usage.as_ref();
                let cost = partial
                    .and_then(|u| u.reported_cost_usd)
                    .filter(|c| c.is_finite() && *c >= 0.);
                if let Some(c) = cost {
                    spent += c;
                } else {
                    unknown_cost += 1;
                }
                event(
                    &mut journal,
                    &mut seq,
                    json!({"kind":"failed","id":row.id,"index":i,"code":error.code,"http_status":error.http_status,"partial_usage":partial.map(|u|json!({"served_model":u.served_model,"input_tokens":u.input_tokens,"output_tokens":u.output_tokens,"reported_cost_usd":u.reported_cost_usd})),"known_cost_usd":cost,"complete_receipt":false}),
                )?;
                stop_reason = Some(format!("technical_{}", error.code));
                break;
            }
            Ok((body, http)) => {
                responses += 1;
                // Transport already redacts failures. Successful raw bodies are also scrubbed
                // before durable storage; the original object remains the decoder input.
                let safe_text = serde_json::to_string(&body)
                    .map_err(|e| e.to_string())?
                    .replace(&key, "[redacted]");
                let safe_body: Value =
                    serde_json::from_str(&safe_text).map_err(|e| e.to_string())?;
                let path = d.join("raw-bodies").join(format!("{}.json", row.id));
                save(&path, &safe_body)?; // fsync BEFORE any semantic/receipt decode.
                let cost = body["usage"]["cost"]
                    .as_f64()
                    .filter(|c| c.is_finite() && *c >= 0.);
                if let Some(c) = cost {
                    spent += c;
                } else {
                    unknown_cost += 1;
                }
                let result = interface.decode(&body, &row.query, &row.document, &row.id);
                let validation_error = result.as_ref().err().cloned();
                event(
                    &mut journal,
                    &mut seq,
                    json!({"kind":"response","id":row.id,"index":i,"http_status":200,"http_version":http.version,"elapsed_ms":http.elapsed_ms,"body_file":format!("raw-bodies/{}.json",row.id),"body_sha256":sha(&path)?,"known_cost_usd":cost,"complete_receipt":contract::receipt_cost_with_limit(&body,interface.completion_limit()).is_ok(),"validation_error":validation_error}),
                )?;
                match result {
                    Err(error) => {
                        event(
                            &mut journal,
                            &mut seq,
                            json!({"kind":"failed","id":row.id,"index":i,"code":error,"body_saved":true}),
                        )?;
                        stop_reason = Some(format!("technical_{error}"));
                        break;
                    }
                    Ok(card) => {
                        let selected = card.selected;
                        if i < 24 {
                            pilot_failed |=
                                pilot_regression(row.label, row.parent_selected, selected);
                        }
                        checked.push(json!({"id":row.id,"source_set":row.source_set,"label":row.label,"role":row.role,"parent_selected":row.parent_selected,"selected":selected,"indeterminate":card.unresolved,"card":card.artifact,"correct":selected==row.label,"new_useful_loss":row.label && row.parent_selected && !selected,"new_extra":!row.label && !row.parent_selected && selected}));
                    }
                }
                if spent > cap || cost.is_some_and(|c| c > bound.as_f64().unwrap()) {
                    stop_reason = Some("receipt_exceeds_frozen_upper_bound_or_budget".into());
                    break;
                }
                if started.elapsed().as_secs_f64() > 900. {
                    stop_reason = Some("phase_deadline_after_request".into());
                    break;
                }
                if i == 23 {
                    pilot_passed = !pilot_failed;
                    event(
                        &mut journal,
                        &mut seq,
                        json!({"kind":"pilot_gate","count":24,"passed":pilot_passed,"parent_preservation_only":true}),
                    )?;
                    if !pilot_passed {
                        stop_reason = Some("pilot_parent_regression".into());
                        break;
                    }
                }
            }
        }
    }
    let final_integrity = verify(&plan, d, interface);
    if let Err(error) = &final_integrity {
        stop_reason = Some(format!("final_integrity_{error}"));
    }
    let metric = |field: &str, value: &str| {
        let subset: Vec<_> = checked
            .iter()
            .filter(|r| field.is_empty() || r[field] == value)
            .collect();
        json!({"evaluated":subset.len(),"correct":subset.iter().filter(|r|r["correct"]==true).count(),"useful_total":subset.iter().filter(|r|r["label"]==true).count(),"useful_retained":subset.iter().filter(|r|r["label"]==true && r["selected"]==true).count(),"lost_useful":subset.iter().filter(|r|r["label"]==true && r["selected"]==false).count(),"extra":subset.iter().filter(|r|r["label"]==false && r["selected"]==true).count(),"indeterminate":subset.iter().filter(|r|r["indeterminate"]==true).count(),"new_useful_losses":subset.iter().filter(|r|r["new_useful_loss"]==true).count(),"new_extras":subset.iter().filter(|r|r["new_extra"]==true).count()})
    };
    save(&d.join("rows.json"), &json!(checked))?;
    let completed = checked.len() == 432 && stop_reason.is_none();
    let report = json!({"plan_sha256":plan_sha,"completed":completed,"all_technical_checks_passed":completed && final_integrity.is_ok(),"attempted_http_requests":attempted,"received_http_responses":responses,"valid_judgments":checked.len(),"unjudged":432-checked.len(),"missing_judgments_are_not_classified":true,"pilot_passed":pilot_passed,"stop_reason":stop_reason,"known_cost_usd":spent,"unknown_cost_operations":unknown_cost,"max_cost_usd":cap,"elapsed_seconds":started.elapsed().as_secs_f64(),"retry_count":0,"metrics":metric("",""),"by_source_set":{"background":metric("source_set","background"),"pool":metric("source_set","pool")},"by_role":{"A":metric("role","A"),"B":metric("role","B"),"C":metric("role","C"),"none":metric("role","none")},"quality_original_88":null,"six_set_gate":null,"reserve_opened":false,"product_changed":false,"promoted":false});
    save(&d.join("runner-status.json"), &report)?;
    event(
        &mut journal,
        &mut seq,
        json!({"kind":"completed","completed":completed,"report_sha256":sha(&d.join("runner-status.json"))?}),
    )?;
    if completed {
        Ok(())
    } else {
        Err("incomplete_or_stopped_attempt".into())
    }
}

#[test]
fn next_request_cannot_cross_budget_or_nonfinite_values() {
    assert!(budget_allows(0.20, 0.04, 0.25));
    assert!(!budget_allows(0.24, 0.02, 0.25));
    assert!(!budget_allows(0., 0.1, 0.51));
    assert!(!budget_allows(f64::NAN, 0.01, 0.25));
    assert!(!budget_allows(0., 0., 0.25));
}
#[test]
fn missing_or_rebuilt_binary_cannot_use_a_frozen_plan() {
    let frozen = "a".repeat(64);
    let rebuilt = "b".repeat(64);
    assert!(verify_binary_binding(&json!({}), &frozen).is_err());
    let plan = json!({"test_binary_sha256": frozen});
    assert!(verify_binary_binding(&plan, &rebuilt).is_err());
    assert!(verify_binary_binding(&plan, &frozen).is_ok());
}
#[test]
fn pilot_blocks_parent_losses_and_new_extras_not_existing_errors() {
    assert!(pilot_regression(true, true, false));
    assert!(pilot_regression(false, false, true));
    assert!(!pilot_regression(true, false, false));
    assert!(!pilot_regression(false, true, true));
    assert!(!pilot_regression(true, false, true));
}

#[test]
fn shared_harness_keeps_literal_v1_and_span_v2_decoding_isolated() {
    let literal = json!({"review_id":"opaque","requested_focus":"foco pedido","query_quotes":["foco\tpedido"],"document_quotes":["valor"],"referent_link":"context_link","focus_link":"requested_component","contribution":"partial","decision":"include","rationale":"Aporte parcial."});
    let mut body = json!({"id":"receipt","model":contract::MODEL,"provider":"Google AI Studio","usage":{"prompt_tokens":10,"completion_tokens":20,"cost":0.0001},"choices":[{"finish_reason":"stop","message":{"content":literal.to_string()}}]});
    let before = contract::decode_body(&body, "foco\tpedido", "valor fuente", "opaque")
        .unwrap()
        .0;
    let old = Interface::LiteralV1
        .decode(&body, "foco\tpedido", "valor fuente", "opaque")
        .unwrap();
    assert_eq!(old.artifact, serde_json::to_value(before).unwrap());
    assert!(old.selected);
    assert!(
        Interface::SpansV2
            .decode(&body, "foco\tpedido", "valor fuente", "opaque")
            .is_err()
    );
    let spans = json!({"review_id":"opaque","query_focus_ids":["q0","q1"],"document_evidence_ids":["d0"],"referent_link":"context_link","focus_link":"requested_component","contribution":"partial","decision":"include","rationale":"Aporte parcial."});
    body["choices"][0]["message"]["content"] = json!(spans.to_string());
    let new = Interface::SpansV2
        .decode(&body, "foco\tpedido", "valor fuente", "opaque")
        .unwrap();
    assert!(new.selected);
    assert_eq!(new.artifact["query_quotes"], old.artifact["query_quotes"]);
    assert!(
        Interface::LiteralV1
            .decode(&body, "foco\tpedido", "valor fuente", "opaque")
            .is_err()
    );
}

#[test]
fn direct_profile_limit_and_document_index_do_not_change_legacy_interfaces() {
    let literal = json!({"review_id":"opaque","requested_focus":"foco pedido","query_quotes":["foco pedido"],"document_quotes":["dato parcial"],"referent_link":"context_link","focus_link":"requested_component","contribution":"partial","decision":"include","rationale":"Aporte parcial."});
    let span = json!({"review_id":"opaque","query_focus_ids":["q0","q1"],"document_evidence_ids":["d0","d1"],"referent_link":"context_link","focus_link":"requested_component","contribution":"partial","decision":"include","rationale":"Aporte parcial."});
    let mut local = span.clone();
    local.as_object_mut().unwrap().remove("decision");
    for (interface, content) in [
        (Interface::LiteralV1, literal),
        (Interface::SpansV2, span),
        (Interface::LocalV3, local),
    ] {
        let body = json!({"id":"receipt","model":contract::MODEL,"provider":"Google AI Studio","usage":{"prompt_tokens":20,"completion_tokens":513,"cost":0.0001},"choices":[{"finish_reason":"stop","message":{"content":content.to_string()}}]});
        assert_eq!(interface.completion_limit(), 1024);
        assert_eq!(
            interface.payload("foco pedido", "dato parcial", "opaque")["max_tokens"],
            1024
        );
        assert!(
            interface
                .decode(&body, "foco pedido", "dato parcial", "opaque")
                .unwrap()
                .selected
        );
        assert_eq!(
            interface.index_profile(),
            span_contract::IndexProfile::BothSources
        );
    }
    let direct = json!({"review_id":"opaque","judgment":"include","document_evidence_ids":["d0","d1"],"rationale":"Aporte parcial."});
    let mut body = json!({"id":"receipt","model":contract::MODEL,"provider":"Google AI Studio","usage":{"prompt_tokens":20,"completion_tokens":512,"cost":0.0001},"choices":[{"finish_reason":"stop","message":{"content":direct.to_string()}}]});
    assert_eq!(Interface::DirectV4.completion_limit(), 512);
    assert_eq!(
        Interface::DirectV4.index_profile(),
        span_contract::IndexProfile::DocumentOnly
    );
    assert!(
        Interface::DirectV4
            .decode(&body, "foco pedido", "dato parcial", "opaque")
            .unwrap()
            .selected
    );
    body["usage"]["completion_tokens"] = json!(513);
    assert!(
        Interface::DirectV4
            .decode(&body, "foco pedido", "dato parcial", "opaque")
            .is_err()
    );
}
