//! Fixed, source-bound contrastive-example pilot. No relabeling or promotion.
use crate::{campaign::digest, evidence};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const CONTROL: &str = "per_memory_binary_v5";
const CANDIDATE: &str = "per_memory_examples_v1";
const IDS: [&str; 10] = [
    "cev5q_066_01",
    "cev5q_067_06",
    "cev5q_071_04",
    "cev5q_073_03",
    "cev5q_068_05",
    "cev5q_069_06",
    "cev5q_070_05",
    "cev5q_066_07",
    "cev5q_069_07",
    "cev5q_073_08",
];
fn read(p: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(p).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn lines(p: &Path) -> Result<Vec<Value>, String> {
    fs::read_to_string(p)
        .map_err(|e| e.to_string())?
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| serde_json::from_str(s).map_err(|e| e.to_string()))
        .collect()
}
fn array<'a>(v: &'a Value, k: &str) -> Result<&'a Vec<Value>, String> {
    v[k].as_array().ok_or_else(|| format!("falta {k}"))
}
fn text<'a>(v: &'a Value, k: &str) -> Result<&'a str, String> {
    v[k].as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("falta {k}"))
}
fn set(v: &Value, key: &str) -> Result<BTreeSet<String>, String> {
    let a = array(v, key)?;
    let s = a
        .iter()
        .map(|x| x.as_str().map(str::to_string).ok_or("ID inválido".into()))
        .collect::<Result<BTreeSet<_>, String>>()?;
    if s.len() != a.len() {
        return Err("IDs repetidos".into());
    }
    Ok(s)
}
fn gain_loss(
    expected: &BTreeSet<String>,
    control: &BTreeSet<String>,
    candidate: &BTreeSet<String>,
) -> Value {
    json!({"removed_unpermitted_ids":control.difference(expected).filter(|x|!candidate.contains(*x)).collect::<Vec<_>>(),"gained_useful_ids":candidate.intersection(expected).filter(|x|!control.contains(*x)).collect::<Vec<_>>(),"lost_useful_ids":control.intersection(expected).filter(|x|!candidate.contains(*x)).collect::<Vec<_>>(),"candidate_unpermitted_ids":candidate.difference(expected).collect::<Vec<_>>()})
}

fn bound_candidate_ids(
    root: &Path,
    directory: &Path,
    cases: &[Value],
    profile: &str,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let prep = read(&directory.join("preparation.json"))?;
    let report = read(&directory.join("report.json"))?;
    let audit = read(&directory.join("audit.json"))?;
    let audited = array(&audit, "campaigns")?;
    if prep["candidate_policy"] != profile
        || report["candidate_policy"] != profile
        || report["pairs"] != cases.len()
        || audited.len() != 1
        || audited[0]["integrity"]["valid"] != true
        || prep["cases_sha256"] != digest(&root.join("development.jsonl"))?
        || prep["corpus_sha256"] != digest(&root.join("memories.json"))?
        || prep["paired_cases_sha256"] != digest(&directory.join("paired-cases.jsonl"))?
        || lines(&directory.join("paired-cases.jsonl"))? != evidence::utility_pairs(cases, profile)?
        || report["audit_sha256"] != digest(&directory.join("audit.json"))?
        || report["journal_sha256"] != digest(&directory.join("journal.jsonl"))?
    {
        return Err("campaña no vinculada al mismo banco/protocolo auditado".into());
    }
    for (key, path) in [
        ("cases", directory.join("paired-cases.jsonl")),
        ("journal", directory.join("journal.jsonl")),
        ("corpus", root.join("memories.json")),
    ] {
        if audited[0]["sha256"][key] != digest(&path)? {
            return Err("auditoría de preservación obsoleta".into());
        }
    }
    let rows = lines(&directory.join("journal.jsonl"))?;
    let mut result = BTreeMap::new();
    for case in cases {
        let id = text(case, "id")?;
        let key = format!("{id}__{profile}");
        let events = rows
            .iter()
            .filter(|e| e["kind"] == "finished" && e["id"] == key)
            .collect::<Vec<_>>();
        if events.len() != 1 {
            return Err("candidata ausente o repetida".into());
        }
        let response = &events[0]["response"];
        if response["status"] != "complete"
            || response["requested_model"] != "typesafe/jev-1.13"
            || events[0]["expected_ids"] != case["expected_ids"]
        {
            return Err("candidata degradada, modelo o etiquetas incompatibles".into());
        }
        let correction_verified = profile.starts_with("jev_per_memory_correction_verified_v")
            || (profile == "jev_per_memory_role_batch_v1"
                || profile == "jev_per_memory_complete_premise_v1")
            || profile.starts_with("jev_per_memory_role_verified_v");
        let anchor = profile == "jev_per_memory_scoped_anchor_v1";
        let mut policy_case = case.clone();
        if anchor {
            policy_case["request"]["selection_policy"] = json!("per_memory_scoped_anchor_v1");
        }
        evidence::per_memory_case(
            &policy_case,
            response,
            if correction_verified || anchor { 7 } else { 8 },
            true,
        )?;
        if response["classification_status"] != "skipped_empty" {
            if correction_verified {
                if profile == "jev_per_memory_complete_premise_v1" {
                    crate::verified::inspect_complete_premise(response, "typesafe/jev-1.13")?;
                } else if profile == "jev_per_memory_role_batch_v1" {
                    crate::verified::inspect_role_batch(response, "typesafe/jev-1.13")?;
                } else if profile == "jev_per_memory_role_verified_v2" {
                    crate::verified::inspect_roles_v2(response, "typesafe/jev-1.13")?;
                } else if profile == "jev_per_memory_role_verified_v1" {
                    crate::verified::inspect_roles(response, "typesafe/jev-1.13")?;
                } else if profile == "jev_per_memory_correction_verified_v5" {
                    crate::verified::inspect_corrections_v5(response, "typesafe/jev-1.13")?;
                } else if profile == "jev_per_memory_correction_verified_v4" {
                    crate::verified::inspect_corrections_v4(response, "typesafe/jev-1.13")?;
                } else if profile == "jev_per_memory_correction_verified_v3" {
                    crate::verified::inspect_corrections_v3(response, "typesafe/jev-1.13")?;
                } else if profile == "jev_per_memory_correction_verified_v2" {
                    crate::verified::inspect_corrections_v2(response, "typesafe/jev-1.13")?;
                } else {
                    crate::verified::inspect_corrections(response, "typesafe/jev-1.13")?;
                }
            } else if anchor {
                crate::factor_audit::inspect_scoped_anchor(
                    text(&case["request"]["search"], "query")?,
                    response,
                )?;
            } else {
                crate::factor_audit::inspect_scoped(response)?;
            }
        }
        result.insert(
            id.to_string(),
            evidence::selected_set(response)?
                .into_iter()
                .map(str::to_string)
                .collect(),
        );
    }
    Ok(result)
}

fn passes_all_known_screens(report: &Value, lost: usize, wrong: usize) -> bool {
    report["passes_quality_and_resource_screen"] == true && lost == 0 && wrong == 0
}

pub fn report_preservation(
    root: &Path,
    earlier: &Path,
    current: &Path,
    output: &Path,
) -> Result<(), String> {
    let (_, cases, _) = evidence::load(root)?;
    if read(&earlier.join("report.json"))?["passes_development_screen"] != true {
        return Err("la versión anterior no pasó calidad".into());
    }
    let before = bound_candidate_ids(root, earlier, &cases, "jev_per_memory_scoped_v1")?;
    let current_preparation = read(&current.join("preparation.json"))?;
    let current_profile = text(&current_preparation, "candidate_policy")?;
    if ![
        "jev_per_memory_scoped_anchor_v1",
        "jev_per_memory_scoped_v2",
        "jev_per_memory_scoped_v3",
        "jev_per_memory_correction_verified_v1",
        "jev_per_memory_correction_verified_v2",
        "jev_per_memory_correction_verified_v3",
        "jev_per_memory_correction_verified_v4",
        "jev_per_memory_correction_verified_v5",
        "jev_per_memory_role_verified_v1",
        "jev_per_memory_role_verified_v2",
        "jev_per_memory_role_batch_v1",
        "jev_per_memory_complete_premise_v1",
    ]
    .contains(&current_profile)
    {
        return Err("perfil de preservación no admitido".into());
    }
    let after = bound_candidate_ids(root, current, &cases, current_profile)?;
    let mut details = Vec::new();
    let mut lost = 0;
    let mut wrong = 0;
    for case in &cases {
        let id = text(case, "id")?;
        let expected = set(case, "expected_ids")?;
        let delta = gain_loss(&expected, &before[id], &after[id]);
        lost += array(&delta, "lost_useful_ids")?.len();
        wrong += array(&delta, "candidate_unpermitted_ids")?.len();
        details.push(json!({"id":id,"earlier_selected_ids":before[id],"current_selected_ids":after[id],"changes":delta}));
    }
    let result = json!({"version":"scoped-version-preservation-v1","cases":cases.len(),"lost_earlier_useful_ids":lost,"current_unpermitted_ids":wrong,"passes_preservation":lost==0&&wrong==0,"passes_global_and_preservation":passes_all_known_screens(&read(&current.join("report.json"))?,lost,wrong),"earlier_report_sha256":digest(&earlier.join("report.json"))?,"current_report_sha256":digest(&current.join("report.json"))?,"source_cases_sha256":digest(&root.join("development.jsonl"))?,"source_corpus_sha256":digest(&root.join("memories.json"))?,"fresh_validation":false,"promoted":false,"temporal_resource_comparison":false,"details":details});
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    f.write_all(
        serde_json::to_string_pretty(&result)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
}
pub fn report(directory: &Path, output: &Path) -> Result<(), String> {
    report_for(
        directory,
        output,
        CONTROL,
        CANDIDATE,
        "docs/evaluation/per-memory-examples-v1-protocol-2026-10-19.md",
        false,
    )
}
pub fn report_factors(directory: &Path, output: &Path) -> Result<(), String> {
    report_for(
        directory,
        output,
        CONTROL,
        "per_memory_factors_v1",
        "docs/evaluation/per-memory-factors-v1-protocol-2026-10-19.md",
        false,
    )
}
pub fn report_scoped(directory: &Path, output: &Path) -> Result<(), String> {
    report_for(
        directory,
        output,
        CONTROL,
        "per_memory_scoped_v1",
        "docs/evaluation/per-memory-scoped-v1-protocol-2026-10-19.md",
        false,
    )
}
pub fn report_scoped_compact(directory: &Path, output: &Path) -> Result<(), String> {
    report_for(
        directory,
        output,
        "per_memory_scoped_v1",
        "per_memory_scoped_v2",
        "docs/evaluation/per-memory-scoped-v2-protocol-2026-10-19.md",
        true,
    )
}
pub fn report_scoped_flat(directory: &Path, output: &Path) -> Result<(), String> {
    report_for(
        directory,
        output,
        "per_memory_scoped_v1",
        "per_memory_scoped_v3",
        "docs/evaluation/per-memory-scoped-v3-protocol-2026-10-19.md",
        true,
    )
}
pub fn report_scoped_anchor(directory: &Path, output: &Path) -> Result<(), String> {
    report_for(
        directory,
        output,
        "per_memory_scoped_v1",
        "per_memory_scoped_anchor_v1",
        "docs/evaluation/scoped-anchor-v1-protocol-2026-10-19.md",
        true,
    )
}
fn report_for(
    directory: &Path,
    output: &Path,
    control_policy: &str,
    candidate_policy: &str,
    protocol: &str,
    reduce_cost: bool,
) -> Result<(), String> {
    let anchor = candidate_policy == "per_memory_scoped_anchor_v1";
    let ids: &[&str] = if anchor {
        &[
            "cev5q_066_01",
            "cev5q_066_07",
            "cev5q_067_06",
            "cev5q_068_03",
            "cev5q_069_05",
            "cev5q_070_05",
            "cev5q_071_04",
            "cev5q_073_03",
        ]
    } else {
        &IDS
    };
    let root = Path::new("tests/fixtures/conversation-evidence-v5");
    let (_, originals, _) = evidence::load(root)?;
    let pre = read(&directory.join("preflight.json"))?;
    let required = [
        root.join("development.jsonl"),
        root.join("memories.json"),
        root.join("manifest.json"),
        directory.join("cases.jsonl"),
        Path::new(protocol).to_path_buf(),
    ];
    let hashes = pre["hashes"].as_object().ok_or("faltan huellas previas")?;
    for path in &required {
        if hashes.get(path.to_str().ok_or("ruta inválida")?) != Some(&json!(digest(path)?)) {
            return Err("entrada sin vínculo previo o alterada".into());
        }
    }
    let cases = lines(&directory.join("cases.jsonl"))?;
    let mut exact = Vec::new();
    for (n, id) in ids.iter().enumerate() {
        let original = originals
            .iter()
            .find(|c| c["id"] == *id)
            .ok_or("falta caso original")?;
        for policy in if n % 2 == 0 {
            [control_policy, candidate_policy]
        } else {
            [candidate_policy, control_policy]
        } {
            let mut c = original.clone();
            c["id"] = json!(format!("{id}__{policy}"));
            c["original_case_id"] = json!(id);
            c["request"]["provider"] = json!("jev");
            c["request"]["selection_policy"] = json!(policy);
            exact.push(c);
        }
    }
    if cases != exact {
        return Err("piloto no coincide con consultas/gold originales y orden fijado".into());
    }
    let audit = read(&directory.join("audit.json"))?;
    let audited = array(&audit, "campaigns")?;
    if audited.len() != 1
        || audited[0]["integrity"]["valid"] != true
        || audited[0]["auxiliary_metadata_projection"]["enabled"] != false
    {
        return Err("auditoría íntegra ordinaria requerida".into());
    }
    for (key, path) in [
        ("cases", directory.join("cases.jsonl")),
        ("journal", directory.join("journal.jsonl")),
        ("corpus", root.join("memories.json")),
    ] {
        if audited[0]["sha256"][key] != digest(&path)? {
            return Err("auditoría obsoleta".into());
        }
    }
    let journal = lines(&directory.join("journal.jsonl"))?;
    let manifest = journal.first().ok_or("diario vacío")?;
    let binary_hash = hashes
        .get("target/debug/iromem")
        .and_then(Value::as_str)
        .ok_or("falta binario congelado")?;
    let database_hash = text(&pre, "database_sha256")?;
    if manifest["kind"] != "manifest"
        || manifest["binary_sha256"] != binary_hash
        || manifest["database_sha256"] != database_hash
        || manifest["case_count"] != json!(cases.len())
        || manifest["cases_sha256"] != digest(&directory.join("cases.jsonl"))?
        || manifest["provider"] != "per_case"
        || manifest["selection_policy"] != "per_case"
    {
        return Err("ejecución no vinculada al binario/base previos".into());
    }
    let mut finished = BTreeMap::new();
    for event in journal.iter().filter(|e| e["kind"] == "finished") {
        let id = text(event, "id")?;
        if !cases.iter().any(|c| c["id"] == id) || finished.insert(id, event).is_some() {
            return Err("evento duplicado o ajeno".into());
        }
    }
    if finished.len() != cases.len() {
        return Err("se requieren las20operaciones terminadas".into());
    }
    let mut costs = [0.0; 2];
    let mut unknown = [0; 2];
    let mut degraded = [0; 2];
    let mut times = [Vec::new(), Vec::new()];
    let mut rows = Vec::new();
    let mut gained = 0;
    let mut removed = 0;
    let mut lost = 0;
    let mut wrong = 0;
    let mut new_degraded = 0;
    let historic = if anchor {
        Some(bound_candidate_ids(
            root,
            Path::new(
                "outputs/iromem/evaluation/per-memory-scoped-v1-development-2026-10-19/paired-v5",
            ),
            &originals,
            "jev_per_memory_scoped_v1",
        )?)
    } else {
        None
    };
    let mut historical_lost = 0;
    for &id in ids {
        let mut selections = [BTreeSet::new(), BTreeSet::new()];
        let mut responses = Vec::new();
        let mut statuses = [false; 2];
        let original = originals
            .iter()
            .find(|c| c["id"] == id)
            .ok_or("caso ausente")?;
        let expected = set(original, "expected_ids")?;
        for (i, policy) in [control_policy, candidate_policy].iter().enumerate() {
            let key = format!("{id}__{policy}");
            let event = finished.get(key.as_str()).ok_or("par incompleto")?;
            let case = cases.iter().find(|c| c["id"] == key).ok_or("caso ajeno")?;
            let response = &event["response"];
            if event["expected_ids"] != case["expected_ids"]
                || event["forbidden_ids"] != case["forbidden_ids"]
                || response["requested_model"] != "typesafe/jev-1.13"
            {
                return Err("etiquetas/modelo alterados".into());
            }
            let elapsed = event["elapsed_ms"]
                .as_f64()
                .filter(|v| v.is_finite() && *v >= 0.0)
                .ok_or("latencia inválida")?;
            times[i].push(elapsed);
            let (cost, missing, _) = crate::score_diagnostic::operation_usage(response)?;
            costs[i] += cost.unwrap_or(0.0);
            unknown[i] += usize::from(missing);
            statuses[i] = response["status"] == "complete";
            if statuses[i] {
                if response["seed"]["classifier"] != "typesafe/jev-1.13" {
                    return Err("clasificador distinto del modelo del par".into());
                }
                evidence::per_memory_case(
                    case,
                    response,
                    if *policy == "per_memory_scoped_anchor_v1" {
                        7
                    } else {
                        8
                    },
                    true,
                )?;
                if (!response["per_memory"]["scoped_checks"].is_null())
                    != (matches!(
                        *policy,
                        "per_memory_scoped_v1"
                            | "per_memory_scoped_v2"
                            | "per_memory_scoped_v3"
                            | "per_memory_scoped_anchor_v1"
                    ) && response["classification_status"] != "skipped_empty")
                {
                    return Err("vías de otra política o ausentes".into());
                }
                if (!response["per_memory"]["factor_checks"].is_null())
                    != (*policy == "per_memory_factors_v1"
                        && response["classification_status"] != "skipped_empty")
                {
                    return Err("factores de otra política o ausentes".into());
                }
                if !response["per_memory"]["choice_scores"].is_null()
                    || !response["per_memory"]["request_sha256"].is_null()
                    || !response["per_memory"]["verification"].is_null()
                {
                    return Err("traza de otra política".into());
                }
                selections[i] = evidence::selected_set(response)?
                    .into_iter()
                    .map(str::to_string)
                    .collect();
            } else {
                degraded[i] += 1;
            }
            responses.push(response);
        }
        if responses[0]["retrieval"]["candidates"] != responses[1]["retrieval"]["candidates"] {
            return Err("candidatos distintos por par".into());
        }
        let historical_lost_ids = historic
            .as_ref()
            .and_then(|h| h.get(id))
            .map(|h| {
                h.intersection(&expected)
                    .filter(|m| !selections[1].contains(*m))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        historical_lost += historical_lost_ids.len();
        let delta = gain_loss(&expected, &selections[0], &selections[1]);
        gained += array(&delta, "gained_useful_ids")?.len();
        removed += array(&delta, "removed_unpermitted_ids")?.len();
        lost += array(&delta, "lost_useful_ids")?.len();
        wrong += array(&delta, "candidate_unpermitted_ids")?.len();
        new_degraded += usize::from(statuses[0] && !statuses[1]);
        rows.push(json!({"id":id,"control_selected_ids":selections[0],"candidate_selected_ids":selections[1],"expected_ids":expected,"candidate_factor_checks":responses[1]["per_memory"]["factor_checks"],"candidate_scoped_checks":responses[1]["per_memory"]["scoped_checks"],"changes":delta,"original_gold_unchanged":true,"historical_lost_useful_ids":historical_lost_ids}));
    }
    let time = [
        evidence::time_distribution(&times[0]),
        evidence::time_distribution(&times[1]),
    ];
    let ratio = |a: f64, b: f64| if b > 0.0 { Some(a / b) } else { None };
    let cost_ratio = if unknown == [0, 0] {
        ratio(costs[1], costs[0])
    } else {
        None
    };
    let p50 = ratio(
        time[1]["p50"].as_f64().ok_or("sinp50")?,
        time[0]["p50"].as_f64().ok_or("sinp50")?,
    );
    let p95 = ratio(
        time[1]["p95"].as_f64().ok_or("sinp95")?,
        time[0]["p95"].as_f64().ok_or("sinp95")?,
    );
    let quality = (reduce_cost || gained > 0 || removed > 0)
        && lost == 0
        && historical_lost == 0
        && wrong == 0
        && new_degraded == 0
        && degraded == [0, 0];
    let resources = unknown == [0, 0]
        && degraded == [0, 0]
        && [cost_ratio, p50, p95]
            .iter()
            .all(|r| r.is_some_and(|v| v <= 1.5));
    let cost_reduced = unknown == [0, 0] && costs[1] < costs[0];
    let objective_passes = quality && resources && (!reduce_cost || cost_reduced);
    let result = json!({"probe_objective":if reduce_cost{"cost_reduction_preserving_quality"}else{"quality_gain"},"cost_reduced":cost_reduced,"version":if anchor {"scoped-anchor-policy-probe-v1"}else if candidate_policy == "per_memory_scoped_v3" {"scoped-flat-policy-probe-v1"}else if candidate_policy == "per_memory_scoped_v2" {"scoped-compact-policy-probe-v1"}else if candidate_policy == "per_memory_scoped_v1" {"scoped-policy-probe-v1"}else if candidate_policy == "per_memory_factors_v1" {"factor-policy-probe-v1"}else{"contrastive-policy-probe-v1"},"operations":cases.len(),"paired_questions":ids.len(),"historical_lost_useful_ids":historical_lost,"source_gold_unchanged":true,"fresh_validation":false,"promoted":false,"known_cost_usd":costs.iter().sum::<f64>(),"unknown_cost_operations":unknown.iter().sum::<usize>(),"control":{"policy":control_policy,"known_cost_usd":costs[0],"unknown_cost_operations":unknown[0],"degraded":degraded[0],"latency_ms":time[0]},"candidate":{"policy":candidate_policy,"known_cost_usd":costs[1],"unknown_cost_operations":unknown[1],"degraded":degraded[1],"latency_ms":time[1]},"ratios":{"cost":cost_ratio,"p50":p50,"p95":p95},"removed_unpermitted_ids":removed,"gained_useful_ids":gained,"lost_useful_ids":lost,"candidate_unpermitted_ids":wrong,"new_degradations":new_degraded,"passes_quality_screen":quality,"passes_resource_screen":resources,"eligible_for_full_development_campaign":objective_passes,"audit_sha256":digest(&directory.join("audit.json"))?,"journal_sha256":digest(&directory.join("journal.jsonl"))?,"cases":rows});
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    f.write_all(
        serde_json::to_string_pretty(&result)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn green_baseline_screen_cannot_qualify_a_candidate_that_loses_earlier_evidence() {
        let green = json!({"passes_quality_and_resource_screen":true});
        assert!(!passes_all_known_screens(&green, 1, 0));
        assert!(!passes_all_known_screens(&green, 0, 1));
        assert!(passes_all_known_screens(&green, 0, 0));
        assert!(!passes_all_known_screens(&json!({}), 0, 0));
    }
    #[test]
    fn gains_do_not_compensate_for_lost_parts_or_unpermitted_context() {
        let ids = |v: &[&str]| v.iter().map(|s| s.to_string()).collect();
        let d = gain_loss(
            &ids(&["support", "refute", "partial"]),
            &ids(&["support", "refute"]),
            &ids(&["support", "partial", "context"]),
        );
        assert_eq!(d["gained_useful_ids"], json!(["partial"]));
        assert_eq!(d["lost_useful_ids"], json!(["refute"]));
        assert_eq!(d["candidate_unpermitted_ids"], json!(["context"]));
    }
}
