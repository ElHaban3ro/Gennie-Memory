//! Model-only contrast on a frozen, previously audited role payload.
use crate::{
    campaign::digest,
    evidence,
    isolated_provider_probe::{ids, lines, read},
    scoped_definition_probe::{selected_model, source_rows, write_new},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const CATALOG: &str =
    "outputs/iromem/evaluation/decision-catalog-alternative-families-2026-10-20.json";
const PROTOCOL: &str = "docs/evaluation/scoped-role-family-contrast-protocol-2026-10-20.md";
const EXECUTION: &str = "docs/evaluation/scoped-role-family-contrast-execution-v1.md";
const ROLE: &str = "outputs/iromem/evaluation/scoped-anchor-role-v5-2026-10-19";
const CONTROL: &str = "typesafe/jev-1.13";
const REVISION: &str = "typesafe/jev-1.13-20260917";
fn inputs() -> Result<(Vec<Value>, Value, Value), String> {
    let (source, mut hashes) = source_rows()?;
    let archive = read(Path::new(
        "outputs/iromem/evaluation/quality-goal-2026-10-19/snapshots/031/source-provenance.json",
    ))?;
    for f in [
        "requests.jsonl",
        "inputs.jsonl",
        "report.json",
        "preflight.json",
    ] {
        let p = Path::new(ROLE).join(f);
        let h = digest(&p)?;
        if archive["artifacts_sha256"][p.to_str().unwrap()] != h {
            return Err("fuente de roles no vinculada".into());
        }
        hashes[p.to_str().unwrap()] = json!(h);
    }
    if lines(&Path::new(ROLE).join("inputs.jsonl"))? != source {
        return Err("roster de roles distinto".into());
    }
    let role = lines(&Path::new(ROLE).join("requests.jsonl"))?;
    for p in [CATALOG, PROTOCOL] {
        let frozen =
            Path::new("outputs/iromem/evaluation/quality-goal-2026-10-19/snapshots/031/source")
                .join(p);
        if digest(Path::new(p))? != digest(&frozen)? {
            return Err("catálogo/protocolo preparado alterado".into());
        }
    }
    let catalog = read(Path::new(CATALOG))?;
    let models = catalog["models"]
        .as_array()
        .filter(|m| m.len() == 3)
        .ok_or("se requieren tres modelos")?;
    for (m, id) in models.iter().zip([
        "scaledown/classify-1",
        "microsoft/microsoft-decision-1",
        "inception/mercury-decide",
    ]) {
        if m["id"] != id || m["canonical_slug"].as_str().is_none_or(|s| s.is_empty()) {
            return Err("modelos no fijados".into());
        }
    }
    let mut requests = Vec::new();
    for (family, m) in models.iter().enumerate() {
        for (i, row) in source.iter().enumerate() {
            let original = format!("{}__definitions", row["id"].as_str().ok_or("sin ID")?);
            let matched = role
                .iter()
                .filter(|r| r["id"] == original)
                .collect::<Vec<_>>();
            if matched.len() != 1 {
                return Err("payload de roles ausente/repetido".into());
            }
            let old = &matched[0]["payload"];
            if old["model"] != CONTROL {
                return Err("modelo original distinto".into());
            }
            for variant in if i % 2 == 0 {
                ["control", "candidate"]
            } else {
                ["candidate", "control"]
            } {
                let mut payload = old.clone();
                if variant == "candidate" {
                    payload["model"] = m["id"].clone();
                }
                requests.push(json!({"id":format!("{}__family_{family:02}_{variant}",row["id"].as_str().unwrap()),"original_case_id":row["id"],"family":family,"variant":variant,"expected_served_model":if variant=="control"{json!(REVISION)}else{m["canonical_slug"].clone()},"payload":payload}));
            }
        }
    }
    for p in [CATALOG, PROTOCOL, EXECUTION] {
        hashes[p] = json!(digest(Path::new(p))?);
    }
    Ok((source, hashes, json!(requests)))
}
pub fn prepare(d: &Path) -> Result<(), String> {
    let (source, hashes, requests) = inputs()?;
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    for (file, rows) in [
        ("inputs.jsonl", source),
        ("requests.jsonl", requests.as_array().unwrap().clone()),
    ] {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(d.join(file))
            .map_err(|e| e.to_string())?;
        for row in rows {
            writeln!(f, "{row}").map_err(|e| e.to_string())?
        }
    }
    write_new(
        &d.join("preparation.json"),
        &json!({"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"requests_sha256":digest(&d.join("requests.jsonl"))?,"source_hashes":hashes,"max_http_requests":108,"fresh_validation":false,"promoted":false}),
    )
}
fn ratio(a: f64, b: f64) -> Option<f64> {
    if b > 0. { Some(a / b) } else { None }
}
fn qualifies(quality: bool, cost: Option<f64>, p50: Option<f64>, p95: Option<f64>) -> bool {
    quality
        && cost.is_some_and(|r| r <= 1.03)
        && [p50, p95].iter().all(|r| r.is_some_and(|r| r <= 1.5))
}
fn semantic_delta(
    expected: &BTreeSet<String>,
    historical: &BTreeSet<String>,
    control: &BTreeSet<String>,
    candidate: &BTreeSet<String>,
    prefix: &BTreeSet<String>,
    valid: [bool; 2],
    must_fix: bool,
) -> Value {
    json!({"candidate_unpermitted_ids":if valid[1]{Some(candidate.difference(expected).cloned().collect::<Vec<_>>())}else{None},
        "lost_control_useful_ids":if valid[0]&&valid[1]{Some(control.intersection(expected).filter(|id|!candidate.contains(*id)).cloned().collect::<Vec<_>>())}else{None},
        "lost_historical_useful_ids":if valid[1]{Some(historical.intersection(expected).filter(|id|!candidate.contains(*id)).cloned().collect::<Vec<_>>())}else{None},
        "omitted_gold_in_prefix":if valid[1]{Some(expected.intersection(prefix).filter(|id|!candidate.contains(*id)).cloned().collect::<Vec<_>>())}else{None},
        "unfixed_regression":if valid[1]{Some(must_fix&&(!candidate.difference(expected).collect::<Vec<_>>().is_empty()||!expected.is_subset(candidate)))}else{None},
        "semantic_comparison_available":valid[0]&&valid[1],"candidate_semantically_measured":valid[1]})
}
pub fn report(d: &Path, output: &Path) -> Result<(), String> {
    let (source, hashes, expected_requests) = inputs()?;
    let requests = lines(&d.join("requests.jsonl"))?;
    let prep = read(&d.join("preparation.json"))?;
    if lines(&d.join("inputs.jsonl"))? != source
        || json!(requests) != expected_requests
        || prep["source_hashes"] != hashes
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || prep["requests_sha256"] != digest(&d.join("requests.jsonl"))?
    {
        return Err("entradas/payloads alterados".into());
    }
    let pre = read(&d.join("preflight.json"))?;
    if pre["preparation_sha256"] != digest(&d.join("preparation.json"))?
        || pre["requests_sha256"] != prep["requests_sha256"]
    {
        return Err("preflight distinto".into());
    }
    for (p, h) in pre["sources_sha256"].as_object().ok_or("sin fuentes")? {
        let archived = d.join("source-executed").join(p);
        if digest(&archived)? != *h {
            return Err("fuente ejecutada no archivada".into());
        }
    }
    let events = lines(&d.join("journal.jsonl"))?;
    let mut started = BTreeSet::new();
    let mut ended = BTreeMap::new();
    for e in &events {
        let id = e["id"].as_str().ok_or("sin ID")?;
        if !requests.iter().any(|r| r["id"] == id) {
            return Err("evento ajeno".into());
        }
        if e["kind"] == "started" {
            if !started.insert(id) {
                return Err("inicio repetido".into());
            }
        } else if ["response", "failed"].contains(&e["kind"].as_str().unwrap_or("")) {
            if !started.contains(id) || ended.insert(id, e).is_some() {
                return Err("evento repetido o fuera de orden".into());
            }
        } else {
            return Err("tipo de evento inválido".into());
        }
    }
    let catalog = read(Path::new(CATALOG))?;
    let mut families = Vec::new();
    let mut total_cost = 0.;
    let mut unknown_total = 0;
    for (family, model) in catalog["models"].as_array().unwrap().iter().enumerate() {
        let mut costs = [0.; 2];
        let mut missing = [0; 2];
        let mut times = [vec![], vec![]];
        let mut wrong = 0;
        let mut lost = 0;
        let mut historic_lost = 0;
        let mut unfixed = 0;
        let mut invalid = 0;
        let mut gains = 0;
        let mut removed = 0;
        let mut attempted = 0;
        let mut finished = 0;
        let mut rows = Vec::new();
        let mut valid_counts = [0; 2];
        let mut compared = 0;
        let mut unavailable_regressions = 0;
        for input in &source {
            let mut valid = [false; 2];
            let mut sets = [BTreeSet::new(), BTreeSet::new()];
            let mut observations = Vec::new();
            let expected = ids(&input["case"]["expected_ids"])?;
            let historical = ids(&input["historical_selected_ids"])?;
            for (slot, variant) in ["control", "candidate"].iter().enumerate() {
                let key = format!(
                    "{}__family_{family:02}_{variant}",
                    input["id"].as_str().unwrap()
                );
                let r = requests
                    .iter()
                    .find(|r| r["id"] == key)
                    .ok_or("falta petición")?;
                let id = r["id"].as_str().unwrap();
                if started.contains(id) {
                    attempted += 1;
                }
                let Some(event) = ended.get(id) else {
                    invalid += 1;
                    if started.contains(id) {
                        missing[slot] += 1;
                    }
                    observations.push(json!({"variant":variant,"status":if started.contains(id){"uncertain"}else{"not_run"}}));
                    continue;
                };
                finished += 1;
                let elapsed = event["elapsed_ms"]
                    .as_f64()
                    .filter(|v| v.is_finite() && *v >= 0.)
                    .ok_or("latencia inválida")?;
                times[slot].push(elapsed);
                let cost = if event["kind"] == "response" {
                    event["body"]["usage"]["cost"].as_f64()
                } else {
                    event["partial_usage"]["reported_cost_usd"].as_f64()
                };
                if let Some(c) = cost.filter(|c| c.is_finite() && *c >= 0.) {
                    costs[slot] += c;
                } else {
                    missing[slot] += 1;
                }
                match selected_model(
                    &event["body"],
                    &r["payload"],
                    input["candidates"].as_array().unwrap(),
                    r["expected_served_model"].as_str().unwrap(),
                ) {
                    Ok((selected, trace)) => {
                        valid[slot] = true;
                        valid_counts[slot] += 1;
                        sets[slot] = selected.clone();
                        observations.push(json!({"variant":variant,"selected_ids":selected,"trace":trace,"served_model":event["body"]["model"]}));
                    }
                    Err(reason) => {
                        invalid += 1;
                        observations.push(json!({"variant":variant,"invalid":reason,"response_kind":event["kind"],"served_model":event["body"]["model"]}));
                    }
                }
            }
            let prefix = input["candidates"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c["memory_id"].as_str().unwrap().to_owned())
                .collect::<BTreeSet<_>>();
            let mut delta = semantic_delta(
                &expected,
                &historical,
                &sets[0],
                &sets[1],
                &prefix,
                valid,
                input["must_fix_regression"] == true,
            );
            compared += usize::from(valid[0] && valid[1]);
            unavailable_regressions +=
                usize::from(input["must_fix_regression"] == true && !valid[1]);
            wrong += delta["candidate_unpermitted_ids"]
                .as_array()
                .map_or(0, Vec::len);
            lost += delta["lost_control_useful_ids"]
                .as_array()
                .map_or(0, Vec::len);
            historic_lost += delta["lost_historical_useful_ids"]
                .as_array()
                .map_or(0, Vec::len);
            unfixed += usize::from(delta["unfixed_regression"] == true);
            if valid[0] && valid[1] {
                gains += sets[1]
                    .intersection(&expected)
                    .filter(|id| !sets[0].contains(*id))
                    .count();
                removed += sets[0]
                    .difference(&expected)
                    .filter(|id| !sets[1].contains(*id))
                    .count();
            }
            delta["id"] = input["id"].clone();
            delta["observations"] = json!(observations);
            rows.push(delta);
        }
        let t = [
            evidence::time_distribution(&times[0]),
            evidence::time_distribution(&times[1]),
        ];
        let cost_ratio = if missing == [0, 0] && finished == 36 {
            ratio(costs[1], costs[0])
        } else {
            None
        };
        let p50 = if finished == 36 && invalid == 0 {
            t[1]["p50"]
                .as_f64()
                .zip(t[0]["p50"].as_f64())
                .and_then(|(a, b)| ratio(a, b))
        } else {
            None
        };
        let p95 = if finished == 36 && invalid == 0 {
            t[1]["p95"]
                .as_f64()
                .zip(t[0]["p95"].as_f64())
                .and_then(|(a, b)| ratio(a, b))
        } else {
            None
        };
        let quality = attempted == 36
            && finished == 36
            && invalid == 0
            && wrong == 0
            && lost == 0
            && historic_lost == 0
            && unfixed == 0;
        let resources = qualifies(true, cost_ratio, p50, p95);
        let eligible = qualifies(quality, cost_ratio, p50, p95);
        total_cost += costs.iter().sum::<f64>();
        unknown_total += missing.iter().sum::<usize>();
        families.push(json!({"family":family,"model":model["id"],"expected_served_model":model["canonical_slug"],"attempted_operations":attempted,"ended_operations":finished,"unrun_operations":36-attempted,"uncertain_operations":attempted-finished,"invalid_or_unavailable_operations":invalid,"valid_control_operations":valid_counts[0],"valid_candidate_operations":valid_counts[1],"semantically_compared_pairs":compared,"unavailable_regression_cases":unavailable_regressions,"candidate_unpermitted_ids":if valid_counts[1]>0{Some(wrong)}else{None},"lost_control_useful_ids":if compared>0{Some(lost)}else{None},"lost_historical_useful_ids":if valid_counts[1]>0{Some(historic_lost)}else{None},"unfixed_regressions":if valid_counts[1]>0{Some(unfixed)}else{None},"gained_useful_ids":if compared>0{Some(gains)}else{None},"removed_unpermitted_ids":if compared>0{Some(removed)}else{None},"control_cost_usd":costs[0],"candidate_cost_usd":costs[1],"unknown_cost_operations":missing.iter().sum::<usize>(),"http_latency_ms":{"control":t[0],"candidate":t[1]},"ratios":{"cost":cost_ratio,"p50":p50,"p95":p95},"passes_quality_screen":quality,"passes_resource_screen":resources,"eligible_for_runtime_implementation":eligible,"cases":rows}));
    }
    let winner = families
        .iter()
        .filter(|f| f["eligible_for_runtime_implementation"] == true)
        .min_by(|a, b| {
            a["candidate_cost_usd"]
                .as_f64()
                .unwrap()
                .total_cmp(&b["candidate_cost_usd"].as_f64().unwrap())
                .then_with(|| {
                    a["http_latency_ms"]["candidate"]["p95"]
                        .as_f64()
                        .unwrap()
                        .total_cmp(&b["http_latency_ms"]["candidate"]["p95"].as_f64().unwrap())
                })
                .then_with(|| a["family"].as_u64().cmp(&b["family"].as_u64()))
        })
        .map(|f| f["model"].clone());
    write_new(
        output,
        &json!({"version":"scoped-role-family-contrast-v2","original_report_sha256":if d.join("report.json").exists(){Some(digest(&d.join("report.json"))?)}else{None},"technical_denominator_repair":"Unrun or invalid candidates are unavailable, not semantic losses or successes.","attempted_operations":started.len(),"ended_operations":ended.len(),"uncertain_operations":started.len()-ended.len(),"known_cost_usd":total_cost,"unknown_cost_operations":unknown_total,"selected_candidate":winner,"families":families,"fresh_validation":false,"promoted":false,"journal_sha256":digest(&d.join("journal.jsonl"))?,"warning":"Known frozen prefixes; model-only contrast, no integrated recall or final answer truth. Catalog prices do not replace receipts."}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_or_unrun_candidate_does_not_create_semantic_losses() {
        let expected = BTreeSet::from(["useful".into()]);
        let empty = BTreeSet::new();
        let unavailable = semantic_delta(
            &expected,
            &expected,
            &expected,
            &empty,
            &expected,
            [true, false],
            true,
        );
        assert!(unavailable["lost_historical_useful_ids"].is_null());
        assert!(unavailable["lost_control_useful_ids"].is_null());
        assert!(unavailable["unfixed_regression"].is_null());
        assert_eq!(unavailable["semantic_comparison_available"], false);
        let rejection = semantic_delta(
            &expected,
            &expected,
            &expected,
            &empty,
            &expected,
            [true, true],
            true,
        );
        assert_eq!(rejection["lost_control_useful_ids"], json!(["useful"]));
        assert_eq!(rejection["unfixed_regression"], true);
    }
    #[test]
    fn low_cost_does_not_compensate_for_quality_or_missing_receipts() {
        assert!(!qualifies(false, Some(0.5), Some(0.5), Some(0.5)));
        assert!(!qualifies(true, None, Some(0.5), Some(0.5)));
        assert!(!qualifies(true, Some(1.031), Some(0.5), Some(0.5)));
        assert!(qualifies(true, Some(1.03), Some(1.5), Some(1.5)));
    }
}
