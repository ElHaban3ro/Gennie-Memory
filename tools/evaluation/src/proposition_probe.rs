//! Manually supplied claim diagnostic; never an extractor or adoption gate.
use crate::{
    campaign::digest,
    contribution_role_contract,
    isolated_provider_probe::{lines, read},
    proposition_bank, proposition_contract as contract,
    scoped_definition_probe::write_new,
};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const BANK: &str = "tests/fixtures/proposition-contribution-v1";
const PROTOCOL: &str = "docs/evaluation/proposition-contribution-v1-protocol-2026-10-20.md";
fn decisions_family(prep: &Value) -> Result<bool, String> {
    match prep.get("checker_family") {
        None => Ok(false),
        Some(v) if v == "decisions" => Ok(true),
        _ => Err("unknown_checker_family".into()),
    }
}
pub fn prepare_decisions(d: &Path) -> Result<(), String> {
    let parent = Path::new("outputs/iromem/evaluation/proposition-contribution-v1-2026-10-20");
    let r = reconstruct(parent)?;
    if serde_json::to_vec_pretty(&r).map_err(|e| e.to_string())?
        != fs::read(parent.join("report.json")).map_err(|e| e.to_string())?
        || r["grounding_errors"] != 0
        || r["role_errors"] != 4
        || r["valid_operations"] != 18
    {
        return Err("parent_diagnostic_not_reproduced".into());
    }
    prepare_family(d, true)
}
pub fn prepare(d: &Path) -> Result<(), String> {
    prepare_family(d, false)
}
fn prepare_family(d: &Path, decisions: bool) -> Result<(), String> {
    let validation = proposition_bank::validate(Path::new(BANK))?;
    if d.exists() {
        return Err("output_exists".into());
    }
    let cases = read(&Path::new(BANK).join("source-only-reviewed.json"))?;
    let requests:Vec<_>=cases.as_array().unwrap().iter().map(|c|json!({"id":c["id"],"payload":if decisions {contract::payload_decisions(c,contribution_role_contract::RULES)} else {contract::payload(c,contribution_role_contract::RULES)}})).collect();
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    for (name, rows) in [
        ("inputs.jsonl", cases.as_array().unwrap()),
        ("requests.jsonl", &requests),
    ] {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(d.join(name))
            .map_err(|e| e.to_string())?;
        for row in rows {
            writeln!(f, "{row}").map_err(|e| e.to_string())?;
        }
        f.sync_all().map_err(|e| e.to_string())?;
    }
    let mut sources = serde_json::Map::new();
    for p in [
        PROTOCOL,
        "docs/evaluation/proposition-contribution-v1-contract.md",
        "docs/evaluation/evidence-contribution-v3-contract.md",
        "tools/evaluation/src/proposition_contract.rs",
        "tools/evaluation/src/proposition_probe.rs",
        "tools/evaluation/src/proposition_bank.rs",
        "tools/evaluation/src/contribution_role_contract.rs",
        "bootstrap/src/openrouter/per_memory/proposition_probe.rs",
    ] {
        sources.insert(p.into(), json!(digest(Path::new(p))?));
    }
    if decisions {
        let p = "docs/evaluation/proposition-decisions-v1-protocol-2026-10-20.md";
        sources.insert(p.into(), json!(digest(Path::new(p))?));
    }
    let mut preparation = json!({"version":"proposition-contribution-v1-preparation","bank":BANK,"bank_manifest_sha256":digest(&Path::new(BANK).join("manifest.json"))?,"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"requests_sha256":digest(&d.join("requests.jsonl"))?,"source_hashes":sources,"model":if decisions{contract::DECISIONS_MODEL}else{contract::MODEL},"revision":if decisions{contract::DECISIONS_REVISION}else{contract::REVISION},"max_http_requests":18,"manual_propositions_not_extractor":true,"fresh_validation":false,"promoted":false,"bank_validation":validation});
    if decisions {
        preparation["checker_family"] = json!("decisions");
    }
    write_new(&d.join("preparation.json"), &preparation)
}
fn selection(v: &Value) -> bool {
    v["grounded"] == true && v["role"] != "none"
}
fn differences(actual: &Value, expected: &Value) -> (usize, usize, usize, usize) {
    let g = usize::from(actual["grounded"] != expected["grounded"]);
    let r = usize::from(actual["role"] != expected["role"]);
    let s = usize::from(selection(actual) != selection(expected));
    (g, r, s, usize::from(g > 0 && r > 0 && s == 0))
}
fn bind_preflight_sources(prep: &Value, pf: &Value) -> Result<(), String> {
    for (p, h) in prep["source_hashes"]
        .as_object()
        .ok_or("missing_prepared_sources")?
    {
        if pf["sources_sha256"][p] != *h {
            return Err("preflight_source_binding_mismatch".into());
        }
    }
    Ok(())
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    write_new(out, &reconstruct(d)?)
}
pub fn reconstruct(d: &Path) -> Result<Value, String> {
    let prep = read(&d.join("preparation.json"))?;
    let decisions = decisions_family(&prep)?;
    if prep["version"] != "proposition-contribution-v1-preparation"
        || prep["model"]
            != if decisions {
                contract::DECISIONS_MODEL
            } else {
                contract::MODEL
            }
        || prep["revision"]
            != if decisions {
                contract::DECISIONS_REVISION
            } else {
                contract::REVISION
            }
        || prep["max_http_requests"] != 18
        || prep["bank_manifest_sha256"] != digest(&Path::new(BANK).join("manifest.json"))?
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || prep["requests_sha256"] != digest(&d.join("requests.jsonl"))?
    {
        return Err("preparation_binding_mismatch".into());
    }
    proposition_bank::validate(Path::new(BANK))?;
    let pf = read(&d.join("preflight.json"))?;
    if pf["preparation_sha256"] != digest(&d.join("preparation.json"))? {
        return Err("preflight_mismatch".into());
    }
    bind_preflight_sources(&prep, &pf)?;
    for (p, h) in pf["sources_sha256"]
        .as_object()
        .ok_or("missing_source_archive")?
    {
        if digest(&d.join("source-executed").join(p))? != *h {
            return Err("executed_archive_changed".into());
        }
    }
    let cases = lines(&d.join("inputs.jsonl"))?;
    let requests = lines(&d.join("requests.jsonl"))?;
    let source = read(&Path::new(BANK).join("source-only-reviewed.json"))?;
    let labels = read(&Path::new(BANK).join("development.json"))?;
    if cases.len() != 18 || requests.len() != 18 || json!(cases) != source {
        return Err("source_bank_mismatch".into());
    }
    for (c, r) in cases.iter().zip(&requests) {
        if c["id"] != r["id"]
            || r["payload"]
                != if decisions {
                    contract::payload_decisions(c, contribution_role_contract::RULES)
                } else {
                    contract::payload(c, contribution_role_contract::RULES)
                }
        {
            return Err("payload_mismatch".into());
        }
    }
    let events = lines(&d.join("journal.jsonl"))?;
    let (mut started, mut ended, mut valid, mut unknown, mut uncertain) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    let (mut gerr, mut rerr, mut serr, mut compensated) = (0usize, 0usize, 0usize, 0usize);
    let (mut unsupported_accepted, mut supported_rejected) = (0usize, 0usize);
    let mut fee = 0.;
    let mut results = Vec::new();
    let mut latencies = Vec::new();
    for pair in events.chunks(2) {
        let index = started;
        if index >= 18 || pair[0]["kind"] != "started" || pair[0]["id"] != cases[index]["id"] {
            return Err("journal_order_invalid".into());
        }
        started += 1;
        if pair.len() == 1 {
            unknown += 1;
            uncertain += 1;
            break;
        }
        let e = &pair[1];
        ended += 1;
        if e["id"] != cases[index]["id"]
            || !matches!(e["kind"].as_str(), Some("response" | "failed"))
        {
            return Err("unexpected_journal_end".into());
        }
        let elapsed = e["elapsed_ms"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.)
            .ok_or("invalid_elapsed")?;
        latencies.push(elapsed);
        let cost = if e["kind"] == "response" {
            e["body"]["usage"]["cost"].as_f64()
        } else {
            e["partial_usage"]["reported_cost_usd"].as_f64()
        }
        .filter(|v| v.is_finite() && *v >= 0.);
        if let Some(c) = cost {
            fee += c;
        } else {
            unknown += 1;
        }
        let decoded = if e["kind"] == "response" {
            if decisions {
                contract::decode_decisions(&e["body"], &requests[index]["payload"], &cases[index])
            } else {
                contract::decode(&e["body"], &requests[index]["payload"], &cases[index])
            }
        } else {
            Err("provider_failure".into())
        };
        match decoded{
            Err(error)=>results.push(json!({"id":cases[index]["id"],"valid":false,"error":error,"elapsed_ms":elapsed,"http_status":e["http_status"]})),
            Ok(values)=>{valid+=1;let expected=&labels[index];if expected["id"]!=cases[index]["id"]{return Err("label_identity_mismatch".into());}let mut checks=Vec::new();
                for(i,actual)in values.iter().enumerate(){let label=&expected["labels"][i];if label["id"]!=cases[index]["propositions"][i]["id"]{return Err("label_order_mismatch".into());}let(g,r,s,c)=differences(actual,label);gerr+=g;rerr+=r;serr+=s;compensated+=c;unsupported_accepted+=usize::from(actual["grounded"]==true && label["grounded"]==false);supported_rejected+=usize::from(actual["grounded"]==false && label["grounded"]==true);checks.push(json!({"id":label["id"],"actual":actual,"expected_grounded":label["grounded"],"expected_role":label["role"],"selection_error":s>0}));}
                results.push(json!({"id":cases[index]["id"],"valid":true,"checks":checks,"elapsed_ms":elapsed}));}
        }
    }
    latencies.sort_by(f64::total_cmp);
    let percentile = |p: f64| -> Option<f64> {
        if latencies.is_empty() {
            None
        } else {
            Some(latencies[(p * latencies.len() as f64).ceil() as usize - 1])
        }
    };
    Ok(
        json!({"version":if decisions{"proposition-contribution-decisions-v1-diagnostic"}else{"proposition-contribution-v1-diagnostic"},"http_requests":started,"ended_http_requests":ended,"valid_operations":valid,"unrun_operations":18-started,"uncertain_operations":uncertain,"unknown_cost_operations":unknown,"known_cost_usd":fee,"grounding_errors":gerr,"unsupported_propositions_accepted":unsupported_accepted,"supported_propositions_rejected":supported_rejected,"role_errors":rerr,"selection_errors":serr,"compensated_errors":compensated,"valid_independent_judgments":valid*4,"correct_independent_judgments":valid*4-gerr-rerr,"p50_all_ended_ms":percentile(0.5),"p95_all_ended_ms":percentile(0.95),"capacity_screen_passed":valid==18&&started==18&&gerr==0&&rerr==0&&serr==0&&unknown==0&&uncertain==0&&fee<=0.003,"manual_propositions_not_extractor":true,"runtime_recall_measured":false,"resources_against_baseline_measured":false,"fresh_validation":false,"promoted":false,"journal_sha256":digest(&d.join("journal.jsonl"))?,"cases":results}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_or_changed_preflight_source_bindings_are_rejected() {
        let p = json!({"source_hashes":{"contract":"abc","runner":"def"}});
        for bad in [
            json!({"sources_sha256":{}}),
            json!({"sources_sha256":{"contract":"abc"}}),
            json!({"sources_sha256":{"contract":"changed","runner":"def"}}),
        ] {
            assert!(bind_preflight_sources(&p, &bad).is_err());
        }
        assert!(
            bind_preflight_sources(
                &p,
                &json!({"sources_sha256":{"contract":"abc","runner":"def","transport":"ghi"}})
            )
            .is_ok()
        );
    }
    #[test]
    fn compensated_grounding_and_role_errors_are_not_a_checking_success() {
        let a = json!({"grounded":false,"role":"A"});
        let e = json!({"grounded":true,"role":"none"});
        assert_eq!(differences(&a, &e), (1, 1, 0, 1));
        assert_eq!(
            differences(
                &json!({"grounded":false,"role":"A"}),
                &json!({"grounded":false,"role":"A"})
            ),
            (0, 0, 0, 0)
        );
    }
}
