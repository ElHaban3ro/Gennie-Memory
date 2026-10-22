//! Paired frozen-example diagnostic; never an extractor or an adoption decision.
use crate::{
    campaign::digest,
    contribution_role_contract,
    isolated_provider_probe::{lines, read},
    proposition_bank, proposition_contract as base, proposition_examples_contract as contract,
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
const PROTOCOL: &str = "docs/evaluation/proposition-examples-v3-protocol-2026-10-20.md";
fn requests(cases: &[Value]) -> Vec<Value> {
    cases.iter().enumerate().flat_map(|(i,c)|{let order=if i%2==0{[false,true]}else{[true,false]};order.into_iter().map(move|candidate|json!({"id":format!("{}__{}",c["id"].as_str().unwrap(),if candidate{"candidate"}else{"control"}),"case_index":i,"case_id":c["id"],"candidate":candidate,"payload":contract::payload(base::payload(c,contribution_role_contract::RULES),candidate)}))}).collect()
}
pub fn prepare(d: &Path) -> Result<(), String> {
    proposition_bank::validate(Path::new(BANK))?;
    let p = Path::new(PARENT);
    let parent = proposition_probe::reconstruct(p)?;
    if serde_json::to_vec_pretty(&parent).map_err(|e| e.to_string())?
        != fs::read(p.join("report.json")).map_err(|e| e.to_string())?
        || parent["valid_operations"] != 18
        || parent["grounding_errors"] != 0
        || parent["role_errors"] != 4
        || parent["unknown_cost_operations"] != 0
    {
        return Err("parent_not_reproduced".into());
    }
    if d.exists() {
        return Err("output_exists".into());
    }
    let cases = read(&Path::new(BANK).join("source-only-reviewed.json"))?;
    let req = requests(cases.as_array().unwrap());
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    for (name, rows) in [
        ("inputs.jsonl", cases.as_array().unwrap()),
        ("requests.jsonl", &req),
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
        "tools/evaluation/src/proposition_examples_contract.rs",
        "tools/evaluation/src/proposition_examples_probe.rs",
        "tools/evaluation/src/proposition_contract.rs",
        "tools/evaluation/src/contribution_role_contract.rs",
        "bootstrap/src/openrouter/per_memory/proposition_examples_probe.rs",
    ] {
        sources.insert(p.into(), json!(digest(Path::new(p))?));
    }
    write_new(
        &d.join("preparation.json"),
        &json!({"version":"proposition-examples-v3-preparation","model":base::MODEL,"revision":base::REVISION,"max_http_requests":36,"bank_manifest_sha256":digest(&Path::new(BANK).join("manifest.json"))?,"parent_report":format!("{PARENT}/report.json"),"parent_report_sha256":digest(&p.join("report.json"))?,"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"requests_sha256":digest(&d.join("requests.jsonl"))?,"source_hashes":sources,"manual_propositions_not_extractor":true,"fresh_validation":false,"promoted":false}),
    )
}
fn selection(v: &Value) -> bool {
    v["grounded"] == true && v["role"] != "none"
}
fn new_regressions(control: &[Value], candidate: &[Value], labels: &[Value]) -> usize {
    control
        .iter()
        .zip(candidate)
        .zip(labels)
        .map(|((a, b), l)| {
            ["grounded", "role"]
                .into_iter()
                .filter(|k| a[k] == l[k] && b[k] != l[k])
                .count()
        })
        .sum()
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    proposition_bank::validate(Path::new(BANK))?;
    let prep = read(&d.join("preparation.json"))?;
    if prep["version"] != "proposition-examples-v3-preparation"
        || prep["model"] != base::MODEL
        || prep["revision"] != base::REVISION
        || prep["max_http_requests"] != 36
        || prep["bank_manifest_sha256"] != digest(&Path::new(BANK).join("manifest.json"))?
        || prep["parent_report_sha256"] != digest(&Path::new(PARENT).join("report.json"))?
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || prep["requests_sha256"] != digest(&d.join("requests.jsonl"))?
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
    let (
        mut valid,
        mut ground,
        mut role,
        mut selected,
        mut extras,
        mut lost,
        mut false_accept,
        mut false_reject,
        mut c_errors,
    ) = (
        [0usize; 2],
        [0usize; 2],
        [0usize; 2],
        [0usize; 2],
        [0usize; 2],
        [0usize; 2],
        [0usize; 2],
        [0usize; 2],
        [0usize; 2],
    );
    let mut cost = [0.; 2];
    let mut unknown = [0usize; 2];
    let mut times = [Vec::new(), Vec::new()];
    let mut maps = BTreeMap::new();
    let mut results = Vec::new();
    for pair in events.chunks(2) {
        let n = attempted;
        if n >= 36 || pair[0]["kind"] != "started" || pair[0]["id"] != req[n]["id"] {
            return Err("journal_order_invalid".into());
        }
        let request = &req[n];
        let ci = request["case_index"].as_u64().ok_or("missing_case_index")? as usize;
        let ai = usize::from(request["candidate"].as_bool().ok_or("missing_candidate")?);
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
        let t = e["elapsed_ms"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.)
            .ok_or("invalid_elapsed")?;
        times[ai].push(t);
        let fee = if e["kind"] == "response" {
            e["body"]["usage"]["cost"].as_f64()
        } else {
            e["partial_usage"]["reported_cost_usd"].as_f64()
        }
        .filter(|v| v.is_finite() && *v >= 0.);
        if let Some(f) = fee {
            cost[ai] += f;
        } else {
            unknown[ai] += 1;
        }
        let decode = if e["kind"] == "response" {
            base::decode(&e["body"], &request["payload"], &cases[ci])
        } else {
            Err("provider_failure".into())
        };
        match decode{Err(error)=>results.push(json!({"id":request["id"],"valid":false,"error":error,"elapsed_ms":t,"http_status":e["http_status"]})),Ok(values)=>{valid[ai]+=1;let mut checks=Vec::new();if labels[ci]["id"]!=cases[ci]["id"]{return Err("label_identity_mismatch".into());}for(i,v)in values.iter().enumerate(){let l=&labels[ci]["labels"][i];if l["id"]!=cases[ci]["propositions"][i]["id"]{return Err("label_order_mismatch".into());}ground[ai]+=usize::from(v["grounded"]!=l["grounded"]);role[ai]+=usize::from(v["role"]!=l["role"]);c_errors[ai]+=usize::from(l["role"]=="C"&&v["role"]!="C");false_accept[ai]+=usize::from(v["grounded"]==true&&l["grounded"]==false);false_reject[ai]+=usize::from(v["grounded"]==false&&l["grounded"]==true);selected[ai]+=usize::from(selection(v)!=selection(l));extras[ai]+=usize::from(selection(v)&&!selection(l));lost[ai]+=usize::from(!selection(v)&&selection(l));checks.push(json!({"id":l["id"],"actual":v,"expected_grounded":l["grounded"],"expected_role":l["role"]}));}maps.insert((ci,ai),values);results.push(json!({"id":request["id"],"case_id":cases[ci]["id"],"candidate":ai==1,"valid":true,"checks":checks,"elapsed_ms":t}));}}
    }
    let total = cost.iter().sum::<f64>();
    let complete = valid == [18, 18] && unknown == [0, 0] && uncertain == 0 && total <= 0.003;
    let regressions = if valid == [18, 18] && uncertain == 0 {
        Some(
            (0..18)
                .map(|ci| {
                    new_regressions(
                        &maps[&(ci, 0)],
                        &maps[&(ci, 1)],
                        labels[ci]["labels"].as_array().unwrap(),
                    )
                })
                .sum::<usize>(),
        )
    } else {
        None
    };
    let mut arms = serde_json::Map::new();
    for (ai, name) in ["control", "candidate"].into_iter().enumerate() {
        times[ai].sort_by(f64::total_cmp);
        let pct = |p: f64| -> Option<f64> {
            if times[ai].is_empty() {
                None
            } else {
                Some(times[ai][(p * times[ai].len() as f64).ceil() as usize - 1])
            }
        };
        arms.insert(name.into(),json!({"valid_operations":valid[ai],"correct_independent_judgments":valid[ai]*4-ground[ai]-role[ai],"valid_independent_judgments":valid[ai]*4,"grounding_errors":ground[ai],"role_errors":role[ai],"expected_C_errors":c_errors[ai],"unsupported_accepted":false_accept[ai],"supported_rejected":false_reject[ai],"selection_errors":selected[ai],"selection_extras":extras[ai],"selection_lost_useful":lost[ai],"known_cost_usd":cost[ai],"unknown_cost_operations":unknown[ai],"p50_all_ended_ms":pct(0.5),"p95_all_ended_ms":pct(0.95)}));
    }
    write_new(
        out,
        &json!({"version":"proposition-examples-v3-diagnostic","http_requests":attempted,"ended_http_requests":ended,"unrun_operations":36-attempted,"uncertain_operations":uncertain,"known_cost_usd":total,"unknown_cost_operations":unknown.iter().sum::<usize>(),"all_valid_complete_within_diagnostic_budget":complete,"new_component_regressions":regressions,"examples_ablation_signal":complete&&c_errors[1]<c_errors[0]&&regressions==Some(0),"candidate_capacity_passed":complete&&ground[1]==0&&role[1]==0&&selected[1]==0,"arms":arms,"cases":results,"manual_propositions_not_extractor":true,"runtime_recall_measured":false,"resources_against_baseline_measured":false,"fresh_validation":false,"promoted":false,"journal_sha256":digest(&d.join("journal.jsonl"))?}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alternating_pairs_preserve_both_cases_and_payloads() {
        let cases:Vec<_>=(0..18).map(|i|json!({"id":format!("private_{i}"),"question":"q","source":"s","propositions":[{"id":"one","text":"a"},{"id":"two","text":"b"}]})).collect();
        let req = requests(&cases);
        assert_eq!(req.len(), 36);
        for (i, pair) in req.chunks(2).enumerate() {
            assert_eq!(pair[0]["candidate"], i % 2 != 0);
            assert_ne!(pair[0]["candidate"], pair[1]["candidate"]);
            for r in pair {
                assert_eq!(r["case_index"], i);
                assert!(!r["payload"].to_string().contains("private_"));
            }
        }
    }
    #[test]
    fn compensated_selection_error_still_counts_as_new_component_regression() {
        let l = vec![json!({"grounded":false,"role":"A"})];
        let a = l.clone();
        let b = vec![json!({"grounded":false,"role":"none"})];
        assert_eq!(selection(&a[0]), selection(&b[0]));
        assert_eq!(new_regressions(&a, &b, &l), 1);
    }
}
