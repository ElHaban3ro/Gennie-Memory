//! Paired known-case comparison with versioned semantic labels and historical discordance.
use crate::{
    campaign::digest,
    contribution_role_contract as contract,
    isolated_provider_probe::{ids, lines, read},
    scoped_definition_probe::write_new,
    sparse_critic_probe,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const BANK: &str = "tests/fixtures/evidence-contribution-known18-v1";
const PROTOCOL: &str = "docs/evaluation/contribution-shared-known18-protocol-2026-10-20.md";
fn bank() -> Result<Vec<Value>, String> {
    let d = Path::new(BANK);
    let manifest = read(&d.join("manifest.json"))?;
    if manifest["version"] != "evidence-contribution-known18-v1-reviewed"
        || manifest["provider_calls_before_freeze"] != 0
    {
        return Err("banco conocido no congelado".into());
    }
    for (field, local) in [("files_sha256", true), ("external_sha256", false)] {
        let map = manifest[field]
            .as_object()
            .filter(|m| !m.is_empty())
            .ok_or("faltan hashes")?;
        for (p, h) in map {
            let path = if local {
                d.join(p)
            } else {
                Path::new(p).to_owned()
            };
            if digest(&path)? != *h {
                return Err(format!("fuente alterada:{p}"));
            }
        }
    }
    for f in [
        "development.json",
        "development-proposed.json",
        "provenance.json",
        "adjudication-review.json",
        "annotation-changes.json",
    ] {
        if manifest["files_sha256"][f].is_null() {
            return Err("hash obligatorio ausente".into());
        }
    }
    let b = read(&d.join("development.json"))?;
    let review = read(&d.join("adjudication-review.json"))?;
    let prov = read(&d.join("provenance.json"))?;
    if b["version"] != "evidence-contribution-known18-development-v1"
        || b["synthetic"] != true
        || b["fresh_validation"] != false
        || review["blocking_disagreement"] != false
    {
        return Err("conciliación no válida".into());
    }
    let source = Path::new(prov["source_inputs"].as_str().ok_or("sin origen")?);
    if digest(source)? != prov["source_inputs_sha256"] {
        return Err("origen no vinculado".into());
    }
    let old = lines(source)?;
    let cases = b["cases"]
        .as_array()
        .filter(|c| c.len() == 18)
        .ok_or("sin18casos")?;
    if old.len() != cases.len() {
        return Err("origen incompleto".into());
    }
    let mut seen = BTreeSet::new();
    let mut useful = 0;
    for (c, r) in cases.iter().zip(&old) {
        let id = c["id"].as_str().ok_or("sin id")?;
        if !seen.insert(id)
            || c["id"] != r["id"]
            || c["question"] != r["case"]["request"]["search"]["query"]
            || c["scope_id"] != r["role_payload"]["state"]["scope"]
            || c["full_candidates"] != r["full_candidates"]
            || c["original_gold_ids"] != r["case"]["expected_ids"]
            || c["historical_selected_ids"] != r["historical_selected_ids"]
        {
            return Err("origen/orden/historia distinto".into());
        }
        let memories = c["memories"]
            .as_array()
            .filter(|m| m.len() == 7)
            .ok_or("sin7memorias")?;
        let labels = c["labels"]
            .as_array()
            .filter(|m| m.len() == 7)
            .ok_or("sin7roles")?;
        let candidates = r["candidates"]
            .as_array()
            .ok_or("sin candidatos originales")?;
        let q = c["question"].as_str().ok_or("sin pregunta")?;
        let mut mids = BTreeSet::new();
        for ((m, l), original) in memories.iter().zip(labels).zip(candidates) {
            let mid = m["id"].as_str().ok_or("sin id de memoria")?;
            let text = m["text"].as_str().ok_or("sin texto")?;
            if !mids.insert(mid)
                || m["id"] != original["memory_id"]
                || m["revision"] != original["revision"]
                || m["text"] != original["summary"]
                || l["id"] != m["id"]
                || !matches!(l["role"].as_str(), Some("A" | "B" | "C" | "none"))
                || !l["residual_ambiguity"].is_boolean()
            {
                return Err("identidad/fuente/rol inválido".into());
            }
            for (quote, source) in [(&l["source_quote"], text), (&l["question_quote"], q)] {
                if !quote
                    .as_str()
                    .is_some_and(|s| !s.is_empty() && source.contains(s))
                {
                    return Err("cita de conciliación inventada".into());
                }
            }
            useful += usize::from(l["role"] != "none");
        }
    }
    if useful != 17 {
        return Err("contribuciones del banco distintas de17".into());
    }
    Ok(cases.clone())
}
fn requests(cases: &[Value], local: bool, family: contract::Family) -> Vec<Value> {
    let mut rows = Vec::new();
    for (n, c) in cases.iter().enumerate() {
        let baseline_row = json!({"full_candidates":c["full_candidates"],"role_payload":{"state":{"user_prompt":c["question"],"scope":c["scope_id"]}}});
        let order: Vec<&str> = if family.reference().is_some() {
            let orders = [
                ["control", "candidate", "reference"],
                ["reference", "candidate", "control"],
                ["candidate", "control", "reference"],
                ["reference", "control", "candidate"],
                ["control", "reference", "candidate"],
                ["candidate", "reference", "control"],
            ];
            orders[n % 6].to_vec()
        } else if n % 2 == 0 {
            vec!["control", "candidate"]
        } else {
            vec!["candidate", "control"]
        };
        for variant in order {
            let p = if variant != "control" {
                (if variant == "reference" {
                    family.reference().expect("reference family")
                } else {
                    family
                })
                .payload(
                    if local {
                        contract::Profile::Local
                    } else {
                        contract::Profile::Shared
                    },
                    c["question"].as_str().unwrap(),
                    c["memories"].as_array().unwrap(),
                )
            } else {
                sparse_critic_probe::baseline(&baseline_row)
            };
            rows.push(json!({"id":format!("{}__{variant}",c["id"].as_str().unwrap()),"case_id":c["id"],"variant":variant,"payload":p}));
        }
    }
    rows
}
pub fn prepare(d: &Path) -> Result<(), String> {
    prepare_profile(d, false, contract::Family::Jev)
}
pub fn prepare_local(d: &Path) -> Result<(), String> {
    prepare_profile(d, true, contract::Family::Jev)
}
pub fn prepare_family(d: &Path, family: contract::Family) -> Result<(), String> {
    prepare_profile(d, true, family)
}
fn prepare_profile(d: &Path, local: bool, family: contract::Family) -> Result<(), String> {
    if matches!(
        family,
        contract::Family::PerplexityContrast
            | contract::Family::MicrosoftContrast
            | contract::Family::PerplexitySpanish
    ) {
        return Err("ablación de criterios requiere control contemporáneo del mismo modelo; preparador de dos brazos no válido".into());
    }
    if family != contract::Family::Jev && !local {
        return Err("familia requiereLocalV5".into());
    }
    prepare_profile_inner(d, local, family)
}
pub fn prepare_negative_contrast(d: &Path) -> Result<(), String> {
    prepare_profile_inner(d, true, contract::Family::PerplexityContrast)
}
pub fn prepare_spanish_contrast(d: &Path) -> Result<(), String> {
    prepare_profile_inner(d, true, contract::Family::PerplexitySpanish)
}
fn prepare_profile_inner(d: &Path, local: bool, family: contract::Family) -> Result<(), String> {
    if family == contract::Family::PerplexitySpanish {
        crate::contribution_role_probe::validate_spanish_glossary()?;
    }
    let cases = bank()?;
    let reqs = requests(&cases, local, family);
    let mut hashes = serde_json::Map::new();
    for p in [
        PROTOCOL,
        "tools/evaluation/src/contribution_role_contract.rs",
        "tools/evaluation/src/contribution_known_probe.rs",
        "tools/evaluation/src/sparse_critic_probe.rs",
        "bootstrap/src/openrouter/per_memory/contribution_known_probe.rs",
        "docs/evaluation/contribution-known18-adjudication-2026-10-20.md",
    ] {
        hashes.insert(p.into(), json!(digest(Path::new(p))?));
    }
    if local {
        let path = "docs/evaluation/contribution-local-v5-protocol-2026-10-20.md";
        hashes.insert(path.into(), json!(digest(Path::new(path))?));
    }
    if family != contract::Family::Jev {
        let path = if family == contract::Family::PerplexitySpanish {
            "docs/evaluation/instruction-language-es-v1-protocol-2026-10-20.md"
        } else if family == contract::Family::PerplexityContrast {
            "docs/evaluation/negative-choice-contrast-v1-protocol-2026-10-20.md"
        } else if family == contract::Family::GeminiExamples {
            "docs/evaluation/generative-contribution-v3-examples-protocol-2026-10-20.md"
        } else if family == contract::Family::Gemini {
            "docs/evaluation/generative-contribution-v2-protocol-2026-10-20.md"
        } else {
            "docs/evaluation/contribution-local-family-protocol-2026-10-20.md"
        };
        hashes.insert(path.into(), json!(digest(Path::new(path))?));
    }
    if matches!(
        family,
        contract::Family::Gemini | contract::Family::GeminiExamples
    ) {
        let path = "tools/evaluation/src/generative_contribution_contract.rs";
        hashes.insert(path.into(), json!(digest(Path::new(path))?));
        let catalog = "outputs/iromem/evaluation/generative-role-catalog-2026-10-20.json";
        hashes.insert(catalog.into(), json!(digest(Path::new(catalog))?));
    }
    if family == contract::Family::PerplexityContrast {
        let path = "docs/evaluation/negative-choice-contrast-known18-v1-protocol-2026-10-20.md";
        hashes.insert(path.into(), json!(digest(Path::new(path))?));
    }
    if family == contract::Family::PerplexitySpanish {
        for path in [
            "docs/evaluation/instruction-language-es-known18-v1-protocol-2026-10-20.md",
            "tools/evaluation/src/spanish_contribution_contract.rs",
            "tools/evaluation/src/contribution_role_probe.rs",
            "tests/fixtures/local-role-instructions-es-v1.json",
            "outputs/iromem/evaluation/instruction-language-es-v1-preparation-2026-10-20/review-final.json",
            "outputs/iromem/evaluation/instruction-language-es-v1-preparation-2026-10-20/preparation.json",
        ] {
            hashes.insert(path.into(), json!(digest(Path::new(path))?));
        }
    }
    let minimal_name = if family != contract::Family::Jev {
        format!(
            "outputs/iromem/evaluation/contribution-local-{}-minimal-v1-2026-10-20",
            family.name()
        )
    } else if local {
        "outputs/iromem/evaluation/contribution-local-v5-minimal-2026-10-20".to_owned()
    } else {
        "outputs/iromem/evaluation/contribution-role-shared-v4-2026-10-20".to_owned()
    };
    let minimal = Path::new(&minimal_name);
    let passed = crate::contribution_role_probe::reconstruct(minimal)?;
    if passed != read(&minimal.join("report.json"))?
        || contract::Family::from_preparation(&read(&minimal.join("preparation.json"))?)? != family
    {
        return Err("mínimo no reproducible o familia incorrecta".into());
    }
    if passed["screen_passed"] != true
        || passed["known_cost_usd"]
            .as_f64()
            .is_none_or(|c| c >= if local { 0.001451520 } else { 0.001569792 })
    {
        return Err("filtro mínimo o ahorro no aprobado".into());
    }
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    for (file, data) in [("inputs.jsonl", &cases), ("requests.jsonl", &reqs)] {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(d.join(file))
            .map_err(|e| e.to_string())?;
        for v in data {
            writeln!(f, "{v}").map_err(|e| e.to_string())?;
        }
        f.sync_all().map_err(|e| e.to_string())?;
    }
    hashes.insert(
        minimal.join("report.json").to_str().unwrap().into(),
        json!(digest(&minimal.join("report.json"))?),
    );
    let mut preparation = json!({"bank_manifest_sha256":digest(&Path::new(BANK).join("manifest.json"))?,"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"requests_sha256":digest(&d.join("requests.jsonl"))?,"source_hashes":hashes,"max_http_requests":reqs.len(),"operations":reqs.len(),"fresh_validation":false,"promoted":false});
    if local {
        preparation["source_local_profile"] = json!(true);
    }
    family.set_preparation(&mut preparation);
    write_new(&d.join("preparation.json"), &preparation)
}
fn percentile(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        let mut v = values.to_vec();
        v.sort_by(f64::total_cmp);
        Some(v[((v.len() as f64 * p).ceil() as usize).saturating_sub(1)])
    }
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    let prep = read(&d.join("preparation.json"))?;
    let local = prep["source_local_profile"] == true;
    let family = contract::Family::from_preparation(&prep)?;
    if family != contract::Family::Jev && !local {
        return Err("familia requiereLocalV5".into());
    }
    let cases = bank()?;
    let reqs = requests(&cases, local, family);
    let pre = read(&d.join("preflight.json"))?;
    if prep["bank_manifest_sha256"] != digest(&Path::new(BANK).join("manifest.json"))?
        || lines(&d.join("inputs.jsonl"))? != cases
        || lines(&d.join("requests.jsonl"))? != reqs
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || prep["requests_sha256"] != digest(&d.join("requests.jsonl"))?
        || pre["preparation_sha256"] != digest(&d.join("preparation.json"))?
    {
        return Err("preparación no reproducible".into());
    }
    for (p, h) in pre["sources_sha256"]
        .as_object()
        .ok_or("sin fuentes ejecutadas")?
    {
        if digest(&d.join("source-executed").join(p))? != *h {
            return Err("fuente ejecutada alterada".into());
        }
    }
    for (p, h) in prep["source_hashes"]
        .as_object()
        .ok_or("sin fuentes preparadas")?
    {
        if pre["sources_sha256"][p] != *h {
            return Err("fuentes ejecutadas/preparadas diferentes".into());
        }
    }
    let mut start = BTreeSet::new();
    let mut end = BTreeMap::new();
    let mut at = 0;
    let slots = if family.reference().is_some() { 3 } else { 2 };
    let mut costs = vec![0.; slots];
    let mut unknown = 0;
    let mut times = vec![Vec::new(); slots];
    let mut versions = BTreeMap::<String, usize>::new();
    for e in lines(&d.join("journal.jsonl"))? {
        let id = e["id"].as_str().ok_or("sin id de evento")?.to_owned();
        if e["kind"] == "started" {
            if at >= reqs.len() || reqs[at]["id"] != id || !start.insert(id.clone()) {
                return Err("inicio repetido/fuera de orden".into());
            }
            at += 1;
        } else if e["kind"] == "response" || e["kind"] == "failed" {
            if !start.contains(&id) || end.contains_key(&id) {
                return Err("fin sin inicio/repetido".into());
            }
            let r = reqs.iter().find(|r| r["id"] == id).ok_or("sin request")?;
            let slot = match r["variant"].as_str() {
                Some("control") => 0,
                Some("candidate") => 1,
                Some("reference") if slots == 3 => 2,
                _ => return Err("variante desconocida".into()),
            };
            let c = if e["kind"] == "response" {
                e["body"]["usage"]["cost"].as_f64()
            } else {
                e["partial_usage"]["reported_cost_usd"].as_f64()
            };
            if let Some(c) = c.filter(|c| c.is_finite() && *c >= 0.) {
                costs[slot] += c;
            } else {
                unknown += 1;
            }
            if let Some(v) = e["http_version"].as_str() {
                *versions.entry(v.to_owned()).or_default() += 1;
            }
            end.insert(id, e);
        } else {
            return Err("evento desconocido".into());
        }
    }
    unknown += start.len() - end.len();
    let mut valid = vec![0; slots];
    let mut tp = vec![0; slots];
    let mut wrong = vec![0; slots];
    let mut role_errors = 0;
    let mut reference_role_errors = 0;
    let mut lost = 0;
    let mut gained = 0;
    let mut unreviewed = 0;
    let mut results = Vec::new();
    let mut historical_discordance = 0;
    for c in &cases {
        let id = c["id"].as_str().unwrap();
        let m = c["memories"].as_array().unwrap();
        let mut gold = BTreeSet::new();
        let reviewed = m
            .iter()
            .map(|v| v["id"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        for l in c["labels"].as_array().unwrap() {
            if l["role"] != "none" {
                gold.insert(l["id"].as_str().unwrap().to_owned());
            }
        }
        let old = ids(&c["original_gold_ids"])?;
        let mut selections = vec![None; slots];
        let mut details = Vec::new();
        for (slot, name) in ["control", "candidate", "reference"]
            .into_iter()
            .take(slots)
            .enumerate()
        {
            let key = format!("{id}__{name}");
            let request = reqs.iter().find(|r| r["id"] == key).unwrap();
            if let Some(e) = end.get(&key).filter(|e| e["kind"] == "response") {
                let decoded = if slot == 0 {
                    sparse_critic_probe::baseline_selected(
                        &e["body"],
                        &request["payload"],
                        c["full_candidates"].as_array().unwrap(),
                    )
                    .map(|s| (s, Vec::new()))
                } else {
                    contract::decode_family(
                        &e["body"],
                        &request["payload"],
                        m,
                        if slot == 2 {
                            family.reference().expect("reference family")
                        } else {
                            family
                        },
                    )
                    .and_then(|r| {
                        contract::selected(&r, m).map(|s| (s.keys().cloned().collect(), r))
                    })
                };
                match decoded {
                    Ok((selected, roles)) => {
                        valid[slot] += 1;
                        let elapsed = e["elapsed_ms"]
                            .as_f64()
                            .filter(|t| t.is_finite() && *t >= 0.)
                            .ok_or("tiempo inválido")?;
                        times[slot].push(elapsed);
                        tp[slot] += selected.intersection(&gold).count();
                        let extra = selected.difference(&gold).cloned().collect::<Vec<_>>();
                        let unknown_ids =
                            selected.difference(&reviewed).cloned().collect::<Vec<_>>();
                        unreviewed += unknown_ids.len();
                        wrong[slot] += extra.iter().filter(|s| reviewed.contains(*s)).count();
                        let mut role_diffs = Vec::new();
                        if slot > 0 {
                            for ((r, l), memory) in
                                roles.iter().zip(c["labels"].as_array().unwrap()).zip(m)
                            {
                                if *r != l["role"] {
                                    if slot == 1 {
                                        role_errors += 1;
                                    } else {
                                        reference_role_errors += 1;
                                    }
                                    role_diffs.push(json!({"id":memory["id"],"expected":l["role"],"actual":r,"residual_ambiguity":l["residual_ambiguity"]}));
                                }
                            }
                        }
                        let hist_extra = selected.difference(&old).cloned().collect::<Vec<_>>();
                        let hist_omitted = old.difference(&selected).cloned().collect::<Vec<_>>();
                        if slot == 1 {
                            historical_discordance += hist_extra.len();
                        }
                        details.push(json!({"variant":name,"valid":true,"selected_ids":selected,"new_development_extra_ids":extra,"unreviewed_selected_ids":unknown_ids,"canonical_role_differences":role_diffs,"original_gold_extra_ids":hist_extra,"original_gold_omitted_ids":hist_omitted,"elapsed_ms":elapsed}));
                        selections[slot] = Some(selected);
                    }
                    Err(e) => details.push(json!({"variant":name,"valid":false,"error":e})),
                }
            } else {
                details.push(json!({"variant":name,"valid":false,"error":"no complete response"}));
            }
        }
        if let (Some(control), Some(candidate)) = (&selections[0], &selections[1]) {
            lost += control
                .intersection(&gold)
                .filter(|s| !candidate.contains(*s))
                .count();
            gained += candidate
                .intersection(&gold)
                .filter(|s| !control.contains(*s))
                .count();
        }
        results.push(json!({"id":id,"operations":details}));
    }
    let ratio = |a: f64, b: f64| if b > 0. { Some(a / b) } else { None };
    let p50 = [percentile(&times[0], 0.5), percentile(&times[1], 0.5)];
    let p95 = [percentile(&times[0], 0.95), percentile(&times[1], 0.95)];
    let ratios = [
        ratio(costs[1], costs[0]),
        p50[0].zip(p50[1]).and_then(|(b, a)| ratio(a, b)),
        p95[0].zip(p95[1]).and_then(|(b, a)| ratio(a, b)),
    ];
    let quality = valid == vec![18; slots]
        && wrong[1] == 0
        && lost == 0
        && gained > 0
        && role_errors == 0
        && tp[1] == 17
        && unreviewed == 0;
    let resource = valid == vec![18; slots]
        && unknown == 0
        && ratios.iter().all(|r| r.is_some_and(|r| r <= 1.5));
    let version = if family.reference().is_some() {
        format!("contribution-local-{}-known18-triple-v1", family.name())
    } else if family != contract::Family::Jev {
        format!("contribution-local-{}-known18-paired-v1", family.name())
    } else {
        (if local {
            "contribution-local-v5-known18-paired"
        } else {
            "contribution-shared-known18-paired-v1"
        })
        .to_owned()
    };
    let mut output = json!({"version":version,"valid_operations":&valid[..2],"http_requests":start.len(),"ended_http_requests":end.len(),"unrun_operations":reqs.len()-start.len(),"unknown_cost_operations":unknown,"known_cost_usd":costs.iter().sum::<f64>(),"control_candidate_cost_usd":&costs[..2],"control_candidate_p50_ms":p50,"control_candidate_p95_ms":p95,"cost_p50_p95_ratios":ratios,"http_versions":versions,"control_candidate_retained_development_ids":&tp[..2],"control_candidate_unpermitted_reviewed_ids":&wrong[..2],"unreviewed_selected_ids":unreviewed,"candidate_canonical_role_errors":role_errors,"lost_contemporary_useful_ids":lost,"gained_useful_ids":gained,"candidate_original_gold_extra_ids":historical_discordance,"quality_gate_passed":quality,"resources_gate_passed":resource,"pilot_passed":quality&&resource,"fresh_validation":false,"labels_are_retrospective_development_not_historical_correction":true,"runtime_recall_measured":false,"promoted":false,"journal_sha256":digest(&d.join("journal.jsonl"))?,"cases":results});
    if slots == 3 {
        output["reference"] = json!({"family":family.reference().unwrap().name(),"valid_operations":valid[2],"known_cost_usd":costs[2],"retained_development_ids":tp[2],"unpermitted_reviewed_ids":wrong[2],"canonical_role_errors":reference_role_errors,"p50_ms":percentile(&times[2],0.5),"p95_ms":percentile(&times[2],0.95)});
        output["all_valid_operations"] = json!(&valid);
        output[if family == contract::Family::PerplexitySpanish {
            "instruction_language_ablation_quality_signal"
        } else {
            "criteria_ablation_quality_signal"
        }] = json!(
            valid == vec![18; 3]
                && role_errors < reference_role_errors
                && tp[1] >= tp[2]
                && wrong[1] <= wrong[2]
        );
        output["negative_subcategories_semantically_scored"] = json!(false);
    }
    write_new(out, &output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn triple_contrast_balances_all_orders_and_has_literal_reference_payloads() {
        let cases=(0..18).map(|i|json!({"id":format!("q{i}"),"question":"query","scope_id":"scope","full_candidates":[],"memories":[{"id":"m","revision":1,"text":"source"}]})).collect::<Vec<_>>();
        for family in [
            contract::Family::PerplexityContrast,
            contract::Family::PerplexitySpanish,
        ] {
            let rows = requests(&cases, true, family);
            assert_eq!(rows.len(), 54);
            let mut orders = BTreeMap::<Vec<String>, usize>::new();
            for (c, batch) in cases.iter().zip(rows.chunks(3)) {
                let order = batch
                    .iter()
                    .map(|r| r["variant"].as_str().unwrap().to_owned())
                    .collect::<Vec<_>>();
                *orders.entry(order).or_default() += 1;
                let reference = batch.iter().find(|r| r["variant"] == "reference").unwrap();
                assert_eq!(
                    reference["payload"]["questions"]["memory_00_relation"]["criteria"]
                        .as_object()
                        .unwrap()
                        .len(),
                    if family == contract::Family::PerplexitySpanish {
                        9
                    } else {
                        4
                    }
                );
                assert_eq!(
                    reference["payload"],
                    family.reference().unwrap().payload(
                        contract::Profile::Local,
                        c["question"].as_str().unwrap(),
                        c["memories"].as_array().unwrap()
                    )
                );
            }
            assert_eq!(orders.len(), 6);
            assert!(orders.values().all(|n| *n == 3));
        }
    }
    #[test]
    fn adjudicated_bank_preserves_sources_and_visible_historical_discordance() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        // Commands run from workspace root; avoid changing process cwd in parallel tests.
        let b = read(&root.join(BANK).join("development.json")).unwrap();
        let cases = b["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 18);
        let changes = cases
            .iter()
            .map(|c| {
                let old = ids(&c["original_gold_ids"]).unwrap();
                let new = c["labels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|l| l["role"] != "none")
                    .map(|l| l["id"].as_str().unwrap().to_owned())
                    .collect::<BTreeSet<_>>();
                (new.difference(&old).count(), old.difference(&new).count())
            })
            .fold((0, 0), |(a, b), (c, d)| (a + c, b + d));
        assert_eq!(changes, (1, 3));
        let req = requests(cases, false, contract::Family::Jev);
        let local = requests(cases, true, contract::Family::Jev);
        for (a, b) in req.iter().zip(&local) {
            if a["variant"] == "control" {
                assert_eq!(a, b);
            }
        }
        assert_eq!(req.len(), 36);
        assert_eq!(req[0]["variant"], "control");
        assert_eq!(req[2]["variant"], "candidate");
        assert!(!req[1]["payload"].to_string().contains("original_gold"));
    }
    #[test]
    fn p95_uses_nearest_rank_and_keeps_slowest_of18() {
        let v = (1..=18).map(|n| n as f64).collect::<Vec<_>>();
        assert_eq!(percentile(&v, 0.95), Some(18.));
    }
}
