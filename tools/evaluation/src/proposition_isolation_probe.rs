//! Three-arm diagnostic. Supplied assertions do not constitute automatic extraction.
use crate::{
    campaign::digest,
    contribution_role_contract,
    isolated_provider_probe::{lines, read},
    proposition_bank, proposition_contract as base,
    proposition_isolation_contract::{self as contract, Arm},
    proposition_probe,
    scoped_definition_probe::write_new,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const BANK: &str = "tests/fixtures/proposition-contribution-v1";
const PARENT: &str = "outputs/iromem/evaluation/proposition-contribution-v1-2026-10-20";
const PROTOCOL: &str = "docs/evaluation/proposition-isolation-v2-protocol-2026-10-20.md";
fn requests(cases: &[Value]) -> Vec<Value> {
    cases.iter().enumerate().flat_map(|(i,c)|contract::order(i).into_iter().map(move|a|json!({"id":format!("{}__{}",c["id"].as_str().unwrap(),a.name()),"case_index":i,"case_id":c["id"],"arm":a.name(),"payload":a.payload(base::payload(c,contribution_role_contract::RULES))}))).collect()
}
fn matches_archived_report(value: &Value, bytes: &[u8]) -> Result<bool, String> {
    Ok(serde_json::to_vec_pretty(value).map_err(|e| e.to_string())? == bytes)
}
pub fn prepare(d: &Path) -> Result<(), String> {
    proposition_bank::validate(Path::new(BANK))?;
    let p = Path::new(PARENT);
    let parent = proposition_probe::reconstruct(p)?;
    if !matches_archived_report(
        &parent,
        &fs::read(p.join("report.json")).map_err(|e| e.to_string())?,
    )? || parent["valid_operations"] != 18
        || parent["grounding_errors"] != 0
        || parent["role_errors"] != 4
        || parent["unknown_cost_operations"] != 0
        || parent["capacity_screen_passed"] != false
    {
        return Err("unbound_or_unexpected_parent_diagnostic".into());
    }
    if d.exists() {
        return Err("output_exists".into());
    }
    let cases = read(&Path::new(BANK).join("source-only-reviewed.json"))?;
    let requests = requests(cases.as_array().unwrap());
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
        "tools/evaluation/src/proposition_isolation_contract.rs",
        "tools/evaluation/src/proposition_isolation_probe.rs",
        "tools/evaluation/src/proposition_contract.rs",
        "tools/evaluation/src/contribution_role_contract.rs",
        "bootstrap/src/openrouter/per_memory/proposition_isolation_probe.rs",
    ] {
        sources.insert(p.into(), json!(digest(Path::new(p))?));
    }
    write_new(
        &d.join("preparation.json"),
        &json!({"version":"proposition-isolation-v2-preparation","bank_manifest_sha256":digest(&Path::new(BANK).join("manifest.json"))?,"parent_report":format!("{PARENT}/report.json"),"parent_report_sha256":digest(&p.join("report.json"))?,"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"requests_sha256":digest(&d.join("requests.jsonl"))?,"source_hashes":sources,"model":base::MODEL,"revision":base::REVISION,"max_http_requests":54,"fresh_validation":false,"promoted":false,"manual_propositions_not_extractor":true}),
    )
}
fn index(a: Arm) -> usize {
    match a {
        Arm::Joint => 0,
        Arm::RoleSource => 1,
        Arm::RoleOnly => 2,
    }
}
fn role_difference(actual: &str, expected: &str) -> (usize, usize, usize) {
    (
        usize::from(actual != "none" && expected == "none"),
        usize::from(actual == "none" && expected != "none"),
        usize::from(actual != "none" && expected != "none" && actual != expected),
    )
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    let prep = read(&d.join("preparation.json"))?;
    proposition_bank::validate(Path::new(BANK))?;
    if prep["version"] != "proposition-isolation-v2-preparation"
        || prep["model"] != base::MODEL
        || prep["revision"] != base::REVISION
        || prep["max_http_requests"] != 54
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || prep["requests_sha256"] != digest(&d.join("requests.jsonl"))?
        || prep["bank_manifest_sha256"] != digest(&Path::new(BANK).join("manifest.json"))?
        || prep["parent_report_sha256"] != digest(&Path::new(PARENT).join("report.json"))?
    {
        return Err("preparation_binding_mismatch".into());
    }
    let pf = read(&d.join("preflight.json"))?;
    if pf["preparation_sha256"] != digest(&d.join("preparation.json"))? {
        return Err("preflight_mismatch".into());
    }
    for (p, h) in prep["source_hashes"]
        .as_object()
        .ok_or("missing_prepared_sources")?
    {
        if pf["sources_sha256"][p] != *h {
            return Err("preflight_source_binding_mismatch".into());
        }
    }
    for (p, h) in pf["sources_sha256"].as_object().ok_or("missing_archive")? {
        if digest(&d.join("source-executed").join(p))? != *h {
            return Err("archive_changed".into());
        }
    }
    let cases = lines(&d.join("inputs.jsonl"))?;
    let req = lines(&d.join("requests.jsonl"))?;
    if json!(cases) != read(&Path::new(BANK).join("source-only-reviewed.json"))?
        || req != requests(&cases)
    {
        return Err("payload_or_order_changed".into());
    }
    let labels = read(&Path::new(BANK).join("development.json"))?;
    let events = lines(&d.join("journal.jsonl"))?;
    let (mut attempted, mut ended, mut uncertain) = (0usize, 0usize, 0usize);
    let (mut valid, mut errors, mut extras, mut omissions, mut wrong) = (
        [0usize; 3],
        [0usize; 3],
        [0usize; 3],
        [0usize; 3],
        [0usize; 3],
    );
    let mut ground_errors = 0usize;
    let mut costs = [0.; 3];
    let mut unknown = [0usize; 3];
    let mut times = [Vec::new(), Vec::new(), Vec::new()];
    let mut results = Vec::new();
    let mut role_map = BTreeMap::new();
    for pair in events.chunks(2) {
        let n = attempted;
        if n >= 54 || pair[0]["kind"] != "started" || pair[0]["id"] != req[n]["id"] {
            return Err("journal_order_invalid".into());
        }
        let request = &req[n];
        let ci = request["case_index"].as_u64().ok_or("missing_case_index")? as usize;
        let arm = Arm::parse(request["arm"].as_str().ok_or("missing_arm")?)?;
        let ai = index(arm);
        attempted += 1;
        if pair.len() == 1 {
            uncertain += 1;
            unknown[ai] += 1;
            break;
        }
        ended += 1;
        let e = &pair[1];
        if e["id"] != request["id"] || !matches!(e["kind"].as_str(), Some("response" | "failed")) {
            return Err("unexpected_journal_end".into());
        }
        let elapsed = e["elapsed_ms"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.)
            .ok_or("invalid_elapsed")?;
        times[ai].push(elapsed);
        let cost = if e["kind"] == "response" {
            e["body"]["usage"]["cost"].as_f64()
        } else {
            e["partial_usage"]["reported_cost_usd"].as_f64()
        }
        .filter(|v| v.is_finite() && *v >= 0.);
        if let Some(c) = cost {
            costs[ai] += c;
        } else {
            unknown[ai] += 1;
        }
        let decoded = if e["kind"] != "response" {
            Err("provider_failure".into())
        } else if arm == Arm::Joint {
            base::decode(&e["body"], &request["payload"], &cases[ci]).map(|values| {
                for (i, v) in values.iter().enumerate() {
                    ground_errors +=
                        usize::from(v["grounded"] != labels[ci]["labels"][i]["grounded"]);
                }
                values
                    .iter()
                    .map(|v| v["role"].as_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            })
        } else {
            contract::decode_roles(&e["body"], &request["payload"])
        };
        match decoded{Err(error)=>results.push(json!({"id":request["id"],"valid":false,"error":error,"elapsed_ms":elapsed,"http_status":e["http_status"]})),Ok(roles)=>{valid[ai]+=1;if labels[ci]["id"]!=cases[ci]["id"]{return Err("label_identity_mismatch".into());}let mut differences=Vec::new();for(i,role)in roles.iter().enumerate(){let label=&labels[ci]["labels"][i];if label["id"]!=cases[ci]["propositions"][i]["id"]{return Err("label_order_mismatch".into());}let expected=label["role"].as_str().unwrap();let(a,b,c)=role_difference(role,expected);extras[ai]+=a;omissions[ai]+=b;wrong[ai]+=c;errors[ai]+=a+b+c;if role!=expected{differences.push(json!({"id":label["id"],"actual":role,"expected":expected}));}}role_map.insert((ci,ai),roles.clone());results.push(json!({"id":request["id"],"case_id":cases[ci]["id"],"arm":arm.name(),"valid":true,"roles":roles,"role_differences":differences,"elapsed_ms":elapsed}));}}
    }
    let known_cost = costs.iter().sum::<f64>();
    let complete =
        valid == [18, 18, 18] && unknown == [0, 0, 0] && uncertain == 0 && known_cost <= 0.003;
    let regressions = |from: usize, to: usize| -> Option<usize> {
        if !complete {
            return None;
        }
        Some(
            (0..18)
                .map(|ci| {
                    (0..2)
                        .filter(|i| {
                            let gold = labels[ci]["labels"][i]["role"].as_str().unwrap();
                            role_map[&(ci, from)][*i] == gold && role_map[&(ci, to)][*i] != gold
                        })
                        .count()
                })
                .sum(),
        )
    };
    let mut arms = serde_json::Map::new();
    for a in [Arm::Joint, Arm::RoleSource, Arm::RoleOnly] {
        let i = index(a);
        times[i].sort_by(f64::total_cmp);
        let pct = |p: f64| -> Option<f64> {
            if times[i].is_empty() {
                None
            } else {
                Some(times[i][(p * times[i].len() as f64).ceil() as usize - 1])
            }
        };
        arms.insert(a.name().into(),json!({"valid_operations":valid[i],"valid_role_judgments":valid[i]*2,"correct_role_judgments":valid[i]*2-errors[i],"role_errors":errors[i],"noncontributing_propositions_accepted":extras[i],"useful_role_omissions":omissions[i],"wrong_useful_roles":wrong[i],"known_cost_usd":costs[i],"unknown_cost_operations":unknown[i],"p50_all_ended_ms":pct(0.5),"p95_all_ended_ms":pct(0.95)}));
    }
    write_new(
        out,
        &json!({"version":"proposition-isolation-v2-diagnostic","http_requests":attempted,"ended_http_requests":ended,"unrun_operations":54-attempted,"uncertain_operations":uncertain,"known_cost_usd":known_cost,"unknown_cost_operations":unknown.iter().sum::<usize>(),"all_valid_complete_within_diagnostic_budget":complete,"joint_grounding_errors":ground_errors,"task_ablation_new_role_regressions":regressions(0,1),"source_ablation_new_role_regressions":regressions(1,2),"task_ablation_signal":complete&&errors[1]<errors[0]&&regressions(0,1)==Some(0),"source_ablation_signal":complete&&errors[2]<errors[1]&&regressions(1,2)==Some(0),"role_only_capacity_passed":complete&&errors[2]==0,"arms":arms,"cases":results,"manual_propositions_not_extractor":true,"automatic_grounding_role_composition_measured":false,"runtime_recall_measured":false,"resources_against_baseline_measured":false,"fresh_validation":false,"promoted":false,"journal_sha256":digest(&d.join("journal.jsonl"))?}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archived_report_comparison_preserves_exact_serialized_fractional_cost() {
        let v = json!({"cost":0.00041286000000000003_f64,"errors":4});
        let bytes = serde_json::to_vec_pretty(&v).unwrap();
        assert!(matches_archived_report(&v, &bytes).unwrap());
        let altered = json!({"cost":0.00041286000000000003_f64,"errors":0});
        assert!(!matches_archived_report(&altered, &bytes).unwrap());
    }
    #[test]
    fn plans_extras_contrast_losses_and_wrong_useful_roles_count_separately() {
        assert_eq!(role_difference("A", "none"), (1, 0, 0));
        assert_eq!(role_difference("none", "C"), (0, 1, 0));
        assert_eq!(role_difference("B", "A"), (0, 0, 1));
        assert_eq!(role_difference("C", "C"), (0, 0, 0));
    }
    #[test]
    fn requests_use_full_bank_balanced_order_and_literal_ablation() {
        let cases:Vec<_>=(0..18).map(|i|json!({"id":format!("private_{i}"),"question":"q","source":"source","propositions":[{"id":"local_a","text":"a"},{"id":"local_b","text":"b"}]})).collect();
        let rows = requests(&cases);
        assert_eq!(rows.len(), 54);
        for (i, chunk) in rows.chunks(3).enumerate() {
            for (r, a) in chunk.iter().zip(contract::order(i)) {
                assert_eq!(r["case_index"], i);
                assert_eq!(r["arm"], a.name());
                assert_eq!(
                    r["payload"],
                    a.payload(base::payload(&cases[i], contribution_role_contract::RULES))
                );
                assert!(!r["payload"].to_string().contains("private_"));
            }
        }
    }
}
