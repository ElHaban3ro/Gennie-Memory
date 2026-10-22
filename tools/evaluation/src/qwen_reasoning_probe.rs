//! Paired configuration screen on known data, never a production quality claim.
use crate::{
    campaign::digest,
    contribution_role_contract::{Family, Profile, reasoning},
    isolated_provider_probe::{lines, read},
    scoped_definition_probe::write_new,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const BANK: &str = "tests/fixtures/evidence-contribution-v3";
const PROTOCOL: &str = "docs/evaluation/qwen9b-reasoning-pair-protocol-2026-10-22.md";
fn requests(cases: &[Value]) -> Vec<Value> {
    let mut rows = Vec::new();
    for (i, c) in cases.iter().enumerate() {
        for enabled in if i % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let base = Family::Qwen.payload(
                Profile::Local,
                c["question"].as_str().unwrap(),
                c["memories"].as_array().unwrap(),
            );
            rows.push(json!({"id":format!("{}__{}",c["id"].as_str().unwrap(),if enabled{"on"}else{"off"}),"case_id":c["id"],"enabled":enabled,"payload":reasoning::payload(&base,enabled)}));
        }
    }
    rows
}
fn write_lines(path: &Path, rows: &[Value]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    for row in rows {
        writeln!(f, "{row}").map_err(|e| e.to_string())?;
    }
    f.sync_all().map_err(|e| e.to_string())
}
pub fn prepare(d: &Path) -> Result<(), String> {
    prepare_inner(d, false)
}
pub fn prepare_total_deadline(d: &Path) -> Result<(), String> {
    prepare_inner(d, true)
}
fn prepare_inner(d: &Path, total_deadline: bool) -> Result<(), String> {
    let cases = crate::contribution_role_probe::bank(Path::new(BANK))?;
    let reqs = requests(&cases);
    let estimate = reqs
        .iter()
        .map(|r| reasoning::estimated_cost(&r["payload"]))
        .sum::<f64>();
    if estimate > reasoning::MAX_COST {
        return Err("estimación excede presupuesto; no ejecutar".into());
    }
    fs::create_dir(d).map_err(|e| e.to_string())?;
    write_lines(&d.join("inputs.jsonl"), &cases)?;
    write_lines(&d.join("requests.jsonl"), &reqs)?;
    let mut hashes = serde_json::Map::new();
    for p in [
        PROTOCOL,
        "docs/evaluation/qwen9b-reasoning-pair-preflight-review-2026-10-22.md",
        "tools/evaluation/src/qwen_reasoning_probe.rs",
        "tools/evaluation/src/qwen_reasoning_contract.rs",
        "tools/evaluation/src/generative_contribution_contract.rs",
        "tools/evaluation/src/contribution_role_contract.rs",
        "tools/evaluation/src/contribution_role_probe.rs",
        "tools/evaluation/src/main.rs",
        "bootstrap/src/openrouter/per_memory/qwen_reasoning_probe.rs",
        "bootstrap/src/openrouter/per_memory/contribution_role_probe.rs",
        "outputs/iromem/evaluation/qwen9b-endpoints-2026-10-22.json",
        "outputs/iromem/evaluation/qwen9b-reasoning-catalog-2026-10-22.json",
    ] {
        hashes.insert(p.into(), json!(digest(Path::new(p))?));
    }
    if total_deadline {
        let protocol = "docs/evaluation/qwen9b-total-deadline-v2-protocol-2026-10-22.md";
        hashes.insert(protocol.into(), json!(digest(Path::new(protocol))?));
        let catalog = "outputs/iromem/evaluation/qwen9b-deadline-v2-endpoints-2026-10-22.json";
        hashes.insert(catalog.into(), json!(digest(Path::new(catalog))?));
    }
    write_new(
        &d.join("preparation.json"),
        &json!({"version":"qwen_reasoning_pair_v1","source_hashes":hashes,"bank":BANK,"bank_manifest_sha256":digest(&Path::new(BANK).join("manifest.json"))?,"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"requests_sha256":digest(&d.join("requests.jsonl"))?,"operations":32,"questions":16,"max_http_requests":32,"max_known_cost_usd":reasoning::MAX_COST,"estimated_cost_usd":estimate,"fresh_validation":false,"promoted":false}),
    )
}
fn empty_arm() -> Value {
    json!({"valid_questions":0,"role_errors":0,"correct_roles":0,"useful_preserved":0,"lost_useful_ids":0,"unpermitted_ids":0,"known_cost_usd":0.0,"unknown_cost_operations":0})
}
fn add(a: &mut Value, key: &str, n: u64) {
    a[key] = json!(a[key].as_u64().unwrap() + n);
}
fn verify_executed_sources(d: &Path, hashes: &Value) -> Result<(), String> {
    let sources = hashes
        .as_object()
        .filter(|m| !m.is_empty())
        .ok_or("sin fuentes ejecutadas")?;
    for (p, h) in sources {
        if Path::new(p)
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
            || digest(&d.join("source-executed").join(p))? != *h
        {
            return Err("fuente ejecutada alterada".into());
        }
    }
    Ok(())
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    let prep = read(&d.join("preparation.json"))?;
    let pre = read(&d.join("preflight.json"))?;
    let cases = crate::contribution_role_probe::bank(Path::new(BANK))?;
    let reqs = requests(&cases);
    verify_executed_sources(d, &pre["source_hashes"])?;
    if pre["max_http_requests"] != 32
        || pre["max_known_cost_usd"] != reasoning::MAX_COST
        || pre["network_window_seconds"] != 300
        || pre["reserved_timeout_seconds"] != 20
        || pre["cfg_test_only"] != true
    {
        return Err("límites ejecutados no vinculados".into());
    }

    if prep["version"] != "qwen_reasoning_pair_v1"
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || prep["requests_sha256"] != digest(&d.join("requests.jsonl"))?
        || prep["bank_manifest_sha256"] != digest(&Path::new(BANK).join("manifest.json"))?
        || lines(&d.join("inputs.jsonl"))? != cases
        || lines(&d.join("requests.jsonl"))? != reqs
        || pre["preparation_sha256"] != digest(&d.join("preparation.json"))?
    {
        return Err("preparación/entrada no vinculada".into());
    }
    for (p, h) in prep["source_hashes"].as_object().ok_or("sin fuentes")? {
        if pre["source_hashes"][p] != *h || digest(&d.join("source-executed").join(p))? != *h {
            return Err("fuente ejecutada alterada".into());
        }
    }
    let events = lines(&d.join("journal.jsonl"))?;
    if events.len() > 64 {
        return Err("diario incompleto/repetido".into());
    }
    let mut arms = json!({"off":empty_arm(),"on":empty_arm()});
    let mut results = Vec::new();
    let mut ids = BTreeSet::new();
    let (mut known, mut unknown) = (0., 0);
    for (i, pair) in events.chunks_exact(2).enumerate() {
        let req = &reqs[i];
        let c = &cases[i / 2];
        let enabled = req["enabled"].as_bool().unwrap();
        let name = if enabled { "on" } else { "off" };
        let a = &mut arms[name];
        let end = &pair[1];
        if pair[0]["kind"] != "started"
            || pair[0]["id"] != req["id"]
            || end["id"] != req["id"]
            || !matches!(end["kind"].as_str(), Some("response" | "failed"))
        {
            return Err("transición no vinculada".into());
        }
        if end["kind"] == "response" {
            if let Some(receipt) = end["body"]["id"].as_str().filter(|s| !s.is_empty()) {
                if !ids.insert(receipt) {
                    return Err("recibo duplicado".into());
                }
            }
        }
        let cost = if end["kind"] == "response" {
            end["body"]["usage"]["cost"].as_f64()
        } else {
            end["partial_usage"]["reported_cost_usd"].as_f64()
        }
        .filter(|x| x.is_finite() && *x >= 0.);
        if let Some(cost) = cost {
            known += cost;
            a["known_cost_usd"] = json!(a["known_cost_usd"].as_f64().unwrap() + cost);
        } else {
            unknown += 1;
            add(a, "unknown_cost_operations", 1);
        }
        let decoded = if end["kind"] == "response" {
            reasoning::decode(
                &end["body"],
                &req["payload"],
                c["memories"].as_array().unwrap(),
                enabled,
            )
        } else {
            Err("http_or_transport_failure".into())
        };
        match decoded {
            Ok(roles) => {
                add(a, "valid_questions", 1);
                let mut js = Vec::new();
                for ((role, l), m) in roles
                    .iter()
                    .zip(c["labels"].as_array().unwrap())
                    .zip(c["memories"].as_array().unwrap())
                {
                    let expected = l["role"].as_str().unwrap();
                    let correct = role == expected;
                    add(
                        a,
                        if correct {
                            "correct_roles"
                        } else {
                            "role_errors"
                        },
                        1,
                    );
                    if expected != "none" {
                        add(
                            a,
                            if role == "none" {
                                "lost_useful_ids"
                            } else {
                                "useful_preserved"
                            },
                            1,
                        );
                    } else if role != "none" {
                        add(a, "unpermitted_ids", 1);
                    }
                    js.push(json!({"memory_id":m["id"],"expected_role":expected,"actual_role":role,"correct":correct}));
                }
                results.push(json!({"id":req["id"],"case_id":c["id"],"arm":name,"valid":true,"judgments":js}));
            }
            Err(error) => results.push(
                json!({"id":req["id"],"case_id":c["id"],"arm":name,"valid":false,"error":error}),
            ),
        }
    }
    let ended = events.len() / 2;
    let unresolved = events.len() % 2;
    if unresolved == 1 {
        let start = events.last().unwrap();
        let req = &reqs[ended];
        if start["kind"] != "started" || start["id"] != req["id"] {
            return Err("inicio pendiente no vinculado".into());
        }
        let name = if req["enabled"] == true { "on" } else { "off" };
        unknown += 1;
        add(&mut arms[name], "unknown_cost_operations", 1);
        results.push(json!({"id":req["id"],"case_id":req["case_id"],"arm":name,"valid":false,"unresolved":true,"error":"started_without_terminal_event"}));
    }
    for name in ["off", "on"] {
        let evaluable = arms[name]["valid_questions"].as_u64().unwrap() > 0;
        arms[name]["selection_metrics_evaluable"] = json!(evaluable);
        if !evaluable {
            for key in [
                "correct_roles",
                "role_errors",
                "useful_preserved",
                "lost_useful_ids",
                "unpermitted_ids",
            ] {
                arms[name][key] = Value::Null;
            }
        }
    }
    let interruption = d.join("coordinator-interruption.json");
    let interruption_sha256 = if interruption.exists() {
        Some(digest(&interruption)?)
    } else {
        None
    };
    let on = &arms["on"];
    let passed = events.len() == 64
        && on["valid_questions"] == 16
        && on["role_errors"] == 0
        && on["lost_useful_ids"] == 0
        && on["unpermitted_ids"] == 0
        && unknown == 0
        && known <= reasoning::MAX_COST
        && arms["off"]["valid_questions"] == 16
        && arms["off"]["role_errors"].as_u64().is_some_and(|n| n > 0);
    write_new(
        out,
        &json!({"version":"qwen_reasoning_pair_v1","http_requests":ended+unresolved,"ended_http_requests":ended,"unresolved_http_requests":unresolved,"unrun_operations":32-ended-unresolved,"interruption_record_sha256":interruption_sha256,"arms":arms,"known_cost_usd":known,"unknown_cost_operations":unknown,"screen_passed":passed,"fresh_validation":false,"resource_gate_measured":false,"runtime_recall_measured":false,"promoted":false,"journal_sha256":digest(&d.join("journal.jsonl"))?,"results":results}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshots_added_by_transport_are_verified_as_well_as_prepared_sources() {
        let d = std::env::temp_dir().join(format!(
            "iromem-reasoning-source-audit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let p = d.join("source-executed/bootstrap/client.rs");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, b"timeout20").unwrap();
        let hashes = json!({"bootstrap/client.rs":digest(&p).unwrap()});
        assert!(verify_executed_sources(&d, &hashes).is_ok());
        fs::write(&p, b"timeout200").unwrap();
        assert!(verify_executed_sources(&d, &hashes).is_err());
        fs::remove_file(&p).unwrap();
        assert!(verify_executed_sources(&d, &hashes).is_err());
        fs::remove_dir_all(d).unwrap();
    }
}
