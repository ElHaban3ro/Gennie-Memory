//! Known-case diagnostic. Product code never imports this module.
use crate::{
    campaign::digest,
    evidence,
    isolated_provider_probe::{ids, lines, read},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const DIR: &str = "outputs/iromem/evaluation/scoped-anchor-v1-2026-10-19";
const SPEC: &str = "tests/fixtures/scoped-anchor-definitions-v2.json";
const PROTOCOL: &str = "docs/evaluation/scoped-anchor-definitions-v2-protocol-2026-10-19.md";
const ROSTER: [&str; 18] = [
    "cev4q_060_06",
    "cev4q_062_01",
    "cev4q_061_06",
    "cev4q_063_05",
    "cev4q_059_08",
    "cev4q_059_01",
    "cev4q_060_03",
    "cev4q_061_01",
    "cev4q_063_01",
    "cev5q_066_01",
    "cev5q_066_07",
    "cev5q_067_06",
    "cev5q_068_03",
    "cev5q_069_05",
    "cev5q_070_05",
    "cev5q_071_04",
    "cev5q_073_03",
    "cev5q_073_08",
];
pub(crate) fn write_new(p: &Path, v: &Value) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(p)
        .map_err(|e| e.to_string())?;
    f.write_all(
        serde_json::to_string_pretty(v)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
}
pub(crate) fn source_rows() -> Result<(Vec<Value>, Value), String> {
    let archived = read(Path::new(
        "outputs/iromem/evaluation/quality-goal-2026-10-19/snapshots/029/source-provenance.json",
    ))?;
    let mut hashes = serde_json::Map::new();
    let mut out = Vec::new();
    for cohort in ["v4", "v5"] {
        let root =
            std::path::PathBuf::from(format!("tests/fixtures/conversation-evidence-{cohort}"));
        let (_, cases, _) = evidence::load(&root)?;
        for f in ["manifest.json", "development.jsonl", "memories.json"] {
            let p = root.join(f);
            hashes.insert(p.to_str().unwrap().into(), json!(digest(&p)?));
        }
        let d = Path::new(DIR).join(format!("paired-{cohort}"));
        for f in [
            "journal.jsonl",
            "paired-cases.jsonl",
            "report.json",
            "audit.json",
        ] {
            let p = d.join(f);
            let h = digest(&p)?;
            if archived["artifacts_sha256"][p.to_str().unwrap()] != h {
                return Err("fuente histórica alterada".into());
            }
            hashes.insert(p.to_str().unwrap().into(), json!(h));
        }
        let events = lines(&d.join("journal.jsonl"))?;
        for id in ROSTER
            .iter()
            .filter(|id| id.starts_with(&format!("ce{cohort}q_")))
        {
            let case = cases
                .iter()
                .find(|c| c["id"] == *id)
                .ok_or("caso original ausente")?;
            let key = format!("{id}__jev_per_memory_scoped_anchor_v1");
            let matched = events
                .iter()
                .filter(|e| e["kind"] == "finished" && e["id"] == key)
                .collect::<Vec<_>>();
            if matched.len() != 1 {
                return Err("evento ausente/repetido".into());
            }
            let e = matched[0];
            if e["expected_ids"] != case["expected_ids"] || e["response"]["status"] != "complete" {
                return Err("gold o respuesta histórica inválida".into());
            }
            let mut policy_case = case.clone();
            policy_case["request"]["selection_policy"] = json!("per_memory_scoped_anchor_v1");
            evidence::per_memory_case(&policy_case, &e["response"], 7, true)?;
            let candidates = e["response"]["retrieval"]["candidates"]
                .as_array()
                .ok_or("candidatos ausentes")?
                .iter()
                .take(7)
                .cloned()
                .collect::<Vec<_>>();
            out.push(json!({"id":id,"case":case,"candidates":candidates,"historical_selected_ids":e["response"]["selected_ids"],"must_fix_regression":ROSTER[..5].contains(id)}));
        }
    }
    out.sort_by_key(|r| ROSTER.iter().position(|id| r["id"] == *id).unwrap());
    if out.len() != 18 {
        return Err("roster incompleto".into());
    }
    Ok((out, json!(hashes)))
}
pub fn prepare(d: &Path) -> Result<(), String> {
    prepare_for(d, SPEC, PROTOCOL)
}
pub fn prepare_inline(d: &Path) -> Result<(), String> {
    prepare_for(
        d,
        "tests/fixtures/scoped-anchor-inline-v3.json",
        "docs/evaluation/scoped-anchor-inline-v3-protocol-2026-10-19.md",
    )
}
pub fn prepare_presupposition(d: &Path) -> Result<(), String> {
    prepare_for(
        d,
        "tests/fixtures/scoped-anchor-presupposition-v4.json",
        "docs/evaluation/scoped-anchor-presupposition-v4-protocol-2026-10-19.md",
    )
}
pub fn prepare_role(d: &Path) -> Result<(), String> {
    prepare_for(
        d,
        "tests/fixtures/scoped-anchor-role-v5.json",
        "docs/evaluation/scoped-anchor-role-v5-protocol-2026-10-19.md",
    )
}
pub fn prepare_role_criteria(d: &Path) -> Result<(), String> {
    prepare_for(
        d,
        "tests/fixtures/scoped-anchor-role-criteria-v2.json",
        "docs/evaluation/scoped-anchor-role-criteria-v2-protocol-2026-10-19.md",
    )
}
pub fn prepare_path(d: &Path) -> Result<(), String> {
    prepare_for(
        d,
        "tests/fixtures/scoped-anchor-path-v1.json",
        "docs/evaluation/scoped-anchor-path-v1-protocol-2026-10-19.md",
    )
}
fn prepare_for(d: &Path, spec_path: &str, protocol_path: &str) -> Result<(), String> {
    let (rows, hashes) = source_rows()?;
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    let p = d.join("inputs.jsonl");
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&p)
        .map_err(|e| e.to_string())?;
    for r in rows {
        writeln!(f, "{r}").map_err(|e| e.to_string())?
    }
    write_new(
        &d.join("preparation.json"),
        &json!({"inputs_sha256":digest(&p)?,"source_hashes":hashes,"spec_sha256":digest(Path::new(spec_path))?,"protocol_sha256":digest(Path::new(protocol_path))?,"operations":36,"fresh_validation":false,"promoted":false}),
    )
}
fn selected(
    body: &Value,
    payload: &Value,
    candidates: &[Value],
) -> Result<(BTreeSet<String>, Value), String> {
    selected_model(body, payload, candidates, "typesafe/jev-1.13-20260917")
}
pub(crate) fn selected_model(
    body: &Value,
    payload: &Value,
    candidates: &[Value],
    expected_revision: &str,
) -> Result<(BTreeSet<String>, Value), String> {
    if body["model"] != expected_revision {
        return Err("revisión servida inesperada".into());
    }
    let qs = payload["questions"].as_object().ok_or("sin preguntas")?;
    let answers = body["answers"]
        .as_object()
        .filter(|a| a.len() == qs.len())
        .ok_or("respuestas ausentes/extra")?;
    let mut choices = BTreeMap::new();
    for (k, q) in qs {
        let a = answers.get(k).ok_or("falta respuesta")?;
        let criteria = q["criteria"].as_object().ok_or("sin opciones")?;
        let choice = a["choice"].as_str().ok_or("choice inválido")?;
        if a["type"] != "choice" || !criteria.contains_key(choice) {
            return Err("opción inválida".into());
        }
        let probs = a["probabilities"]
            .as_object()
            .filter(|p| p.len() == criteria.len())
            .ok_or("probabilidades incompletas")?;
        let mut sum = 0.;
        for key in criteria.keys() {
            let p = probs
                .get(key)
                .and_then(Value::as_f64)
                .filter(|p| p.is_finite() && (0.0..=1.0).contains(p))
                .ok_or("probabilidad inválida")?;
            sum += p;
        }
        if (sum - 1.).abs() > 0.02 {
            return Err("probabilidades no normalizadas".into());
        }
        choices.insert(k.as_str(), choice);
    }
    let anchor = choices
        .get("case_anchor")
        .filter(|s| **s != "none")
        .map(|s| {
            qs["case_anchor"]["criteria"][*s]
                .as_str()
                .ok_or("nombre inválido")
        })
        .transpose()?;
    let words = |s: &str| {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase)
            .collect::<BTreeSet<_>>()
    };
    let mut selected = BTreeSet::new();
    let mut checks = Vec::new();
    let mut items = Vec::new();
    for (n, c) in candidates.iter().enumerate() {
        let relation_key = format!("memory_{n:02}_relation");
        let (fact, correction) = if let Some(role) = choices.get(relation_key.as_str()) {
            (*role == "A", *role == "B")
        } else {
            (
                *choices
                    .get(format!("memory_{n:02}_fact").as_str())
                    .ok_or("dato ausente")?
                    == "A",
                *choices
                    .get(format!("memory_{n:02}_correction").as_str())
                    .ok_or("corrección ausente")?
                    == "A",
            )
        };
        let present = anchor.map(|name| {
            let need = words(name);
            !need.is_empty() && need.is_subset(&words(c["summary"].as_str().unwrap_or("")))
        });
        let accepted = (fact || correction) && present != Some(false);
        let id = c["memory_id"].as_str().ok_or("sin ID")?;
        if accepted {
            selected.insert(id.into());
        }
        checks.push(json!({"memory_id":id,"revision":c["revision"],"requested_fact_same_context":fact,"premise_correction_same_context":correction,"literal_anchor":{"query_token":anchor,"present_in_source":present}}));
        items.push(json!({"memory_id":id,"revision":c["revision"],"relation":if accepted{"useful_evidence"}else{"unrelated"}}));
    }
    let trace = json!({"retrieval":{"candidates":candidates},"per_memory":{"scoped_checks":checks,"items":items}});
    crate::factor_audit::inspect_scoped_anchor(
        payload["state"]["user_prompt"]
            .as_str()
            .ok_or("sin consulta")?,
        &trace,
    )?;
    let mut trace = trace;
    if qs.contains_key("memory_00_relation") {
        let checks = trace["per_memory"]
            .as_object_mut()
            .unwrap()
            .remove("scoped_checks")
            .unwrap();
        let roles=checks.as_array().unwrap().iter().enumerate().map(|(n,c)|json!({"memory_id":c["memory_id"],"revision":c["revision"],"role":match choices[format!("memory_{n:02}_relation").as_str()]{"A"=>"requested_contribution","B"=>"premise_refutation",_=>"none"},"literal_anchor":c["literal_anchor"]})).collect::<Vec<_>>();
        trace["per_memory"]["role_checks"] = json!(roles);
        trace["composition"] = json!(
            "exclusive role A/B AND literal presence when a name was chosen; no separate F/C judgments"
        );
    }
    Ok((selected, trace))
}
pub fn report(d: &Path, output: &Path) -> Result<(), String> {
    report_for(d, output, SPEC, PROTOCOL)
}
pub fn report_inline(d: &Path, output: &Path) -> Result<(), String> {
    report_for(
        d,
        output,
        "tests/fixtures/scoped-anchor-inline-v3.json",
        "docs/evaluation/scoped-anchor-inline-v3-protocol-2026-10-19.md",
    )
}
pub fn report_presupposition(d: &Path, output: &Path) -> Result<(), String> {
    report_for(
        d,
        output,
        "tests/fixtures/scoped-anchor-presupposition-v4.json",
        "docs/evaluation/scoped-anchor-presupposition-v4-protocol-2026-10-19.md",
    )
}
pub fn report_role(d: &Path, output: &Path) -> Result<(), String> {
    report_for(
        d,
        output,
        "tests/fixtures/scoped-anchor-role-v5.json",
        "docs/evaluation/scoped-anchor-role-v5-protocol-2026-10-19.md",
    )
}
pub fn report_role_criteria(d: &Path, output: &Path) -> Result<(), String> {
    report_for(
        d,
        output,
        "tests/fixtures/scoped-anchor-role-criteria-v2.json",
        "docs/evaluation/scoped-anchor-role-criteria-v2-protocol-2026-10-19.md",
    )
}
pub fn report_path(d: &Path, output: &Path) -> Result<(), String> {
    report_for(
        d,
        output,
        "tests/fixtures/scoped-anchor-path-v1.json",
        "docs/evaluation/scoped-anchor-path-v1-protocol-2026-10-19.md",
    )
}
fn report_for(d: &Path, output: &Path, spec_path: &str, protocol_path: &str) -> Result<(), String> {
    let (source, hashes) = source_rows()?;
    let prep = read(&d.join("preparation.json"))?;
    if lines(&d.join("inputs.jsonl"))? != source
        || prep["source_hashes"] != hashes
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || prep["spec_sha256"] != digest(Path::new(spec_path))?
        || prep["protocol_sha256"] != digest(Path::new(protocol_path))?
    {
        return Err("entradas no vinculadas".into());
    }
    let pre = read(&d.join("preflight.json"))?;
    if pre["requests_sha256"] != digest(&d.join("requests.jsonl"))?
        || pre["preparation_sha256"] != digest(&d.join("preparation.json"))?
    {
        return Err("preflight inválido".into());
    }
    for (p, h) in pre["sources_sha256"].as_object().ok_or("sin fuentes")? {
        let archive = d.join("source-executed").join(p);
        if digest(if archive.exists() {
            &archive
        } else {
            Path::new(p)
        })? != *h
        {
            return Err("fuente de ejecución cambió".into());
        }
    }
    let requests = lines(&d.join("requests.jsonl"))?;
    if requests.len() != 36 {
        return Err("requiere36peticiones".into());
    }
    let spec = read(Path::new(spec_path))?;
    let journal = lines(&d.join("journal.jsonl"))?;
    let mut ended = BTreeMap::new();
    let mut started = BTreeSet::new();
    for e in &journal {
        let id = e["id"].as_str().ok_or("sin ID")?;
        if !requests.iter().any(|r| r["id"] == id) {
            return Err("evento fuera del roster congelado".into());
        }
        if e["kind"] == "started" {
            if !started.insert(id) {
                return Err("inicio repetido".into());
            }
        } else if e["kind"] == "response" || e["kind"] == "failed" {
            if !started.contains(id) || ended.insert(id, e).is_some() {
                return Err("evento repetido/fuera de orden".into());
            }
        } else {
            return Err("evento desconocido".into());
        }
    }
    let mut costs = [0.; 2];
    let mut missing = [0; 2];
    let mut times = [vec![], vec![]];
    let mut invalid = 0;
    let mut wrong = 0;
    let mut lost = 0;
    let mut historical_lost = 0;
    let mut unresolved = 0;
    let mut gains = 0;
    let mut rows = Vec::new();
    for (i, input) in source.iter().enumerate() {
        let pair = &requests[2 * i..2 * i + 2];
        let order = if i % 2 == 0 {
            ["control", "definitions"]
        } else {
            ["definitions", "control"]
        };
        let control = pair
            .iter()
            .find(|r| r["variant"] == "control")
            .ok_or("control ausente")?;
        let candidates = input["candidates"].as_array().unwrap();
        let expected = ids(&input["case"]["expected_ids"])?;
        let historical = ids(&input["historical_selected_ids"])?;
        let mut selections = [BTreeSet::new(), BTreeSet::new()];
        let mut result = Vec::new();
        for (r, variant) in pair.iter().zip(order) {
            if r["id"] != format!("{}__{variant}", input["id"].as_str().unwrap())
                || r["variant"] != variant
            {
                return Err("orden/roster alterado".into());
            }
            let slot = usize::from(variant == "definitions");
            let mut rebuilt = control["payload"].clone();
            if slot == 1 {
                if !spec["evidence_tests"].is_null() {
                    rebuilt["state"]["evidence_tests"] = spec["evidence_tests"].clone();
                }
                if spec["path_syntax"] == true {
                    for q in rebuilt["questions"]
                        .as_object_mut()
                        .ok_or("sin preguntas")?
                        .values_mut()
                    {
                        let mut text = q["instructions"]
                            .as_str()
                            .ok_or("sin instrucción")?
                            .replace(
                                spec["query_path"]["before"].as_str().ok_or("sin ruta")?,
                                spec["query_path"]["after"].as_str().ok_or("sin ruta")?,
                            );
                        for n in 0..candidates.len() {
                            let key = format!("memory_{n:02}");
                            text = text.replace(
                                &spec["memory_path"]["before"]
                                    .as_str()
                                    .ok_or("sin ruta")?
                                    .replace("{key}", &key),
                                &spec["memory_path"]["after"]
                                    .as_str()
                                    .ok_or("sin ruta")?
                                    .replace("{key}", &key),
                            );
                        }
                        q["instructions"] = json!(text);
                    }
                } else if !spec["relation_instruction"].is_null() {
                    let mut qs = serde_json::Map::new();
                    if let Some(a) = rebuilt["questions"].get("case_anchor") {
                        qs.insert("case_anchor".into(), a.clone());
                    }
                    for n in 0..candidates.len() {
                        let key = format!("memory_{n:02}");
                        qs.insert(format!("{key}_relation"),json!({"type":"choice","instructions":spec["relation_instruction"].as_str().ok_or("falta instrucción")?.replace("{key}",&key),"criteria":spec["criteria"]}));
                    }
                    rebuilt["questions"] = json!(qs);
                } else {
                    for n in 0..candidates.len() {
                        for factor in ["fact", "correction"] {
                            rebuilt["questions"][format!("memory_{n:02}_{factor}")]["instructions"] = json!(
                                spec[format!("{factor}_instruction")]
                                    .as_str()
                                    .ok_or("sin template")?
                                    .replace("{key}", &format!("memory_{n:02}"))
                            );
                        }
                    }
                }
            }
            if r["payload"] != rebuilt
                || r["payload"]["model"] != "typesafe/jev-1.13"
                || r["payload"]["state"]["user_prompt"]
                    != input["case"]["request"]["search"]["query"]
            {
                return Err("payload distinto del template".into());
            }
            for (n, c) in candidates.iter().enumerate() {
                if r["payload"]["state"]["memories"][format!("memory_{n:02}")] != c["summary"] {
                    return Err("recuerdo modificado".into());
                }
            }
            let id = r["id"].as_str().unwrap();
            let Some(e) = ended.get(id) else {
                invalid += 1;
                if started.contains(id) {
                    missing[slot] += 1;
                }
                result.push(json!({"variant":variant,"status":if started.contains(id){"uncertain"}else{"not_run"}}));
                continue;
            };
            times[slot].push(e["elapsed_ms"].as_f64().ok_or("latencia ausente")?);
            let cost = if e["kind"] == "response" {
                e["body"]["usage"]["cost"].as_f64()
            } else {
                e["partial_usage"]["reported_cost_usd"].as_f64()
            };
            if let Some(c) = cost.filter(|c| c.is_finite() && *c >= 0.) {
                costs[slot] += c
            } else {
                missing[slot] += 1
            }
            match selected(&e["body"], &r["payload"], candidates) {
                Ok((s, t)) => {
                    selections[slot] = s.clone();
                    result.push(json!({"variant":variant,"selected_ids":s,"trace":t}));
                }
                Err(reason) => {
                    invalid += 1;
                    result.push(json!({"variant":variant,"invalid":reason}));
                }
            }
        }
        let bad = selections[1]
            .difference(&expected)
            .cloned()
            .collect::<Vec<_>>();
        let lost_ids = selections[0]
            .intersection(&expected)
            .filter(|id| !selections[1].contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        let old_lost = historical
            .intersection(&expected)
            .filter(|id| !selections[1].contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        let gained = selections[1]
            .intersection(&expected)
            .filter(|id| !selections[0].contains(*id))
            .count();
        let prefix = candidates
            .iter()
            .map(|c| c["memory_id"].as_str().unwrap().to_string())
            .collect::<BTreeSet<_>>();
        let omitted = expected
            .intersection(&prefix)
            .filter(|id| !selections[1].contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        let unfixed = input["must_fix_regression"] == true
            && (!bad.is_empty() || !expected.is_subset(&selections[1]));
        wrong += bad.len();
        lost += lost_ids.len();
        historical_lost += old_lost.len();
        gains += gained;
        unresolved += usize::from(unfixed);
        rows.push(json!({"id":input["id"],"candidate_unpermitted_ids":bad,"lost_control_useful_ids":lost_ids,"lost_historical_useful_ids":old_lost,"omitted_gold_in_prefix":omitted,"unfixed_regression":unfixed,"results":result}));
    }
    let t = [
        evidence::time_distribution(&times[0]),
        evidence::time_distribution(&times[1]),
    ];
    let ratio = |a: f64, b: f64| if b > 0. { Some(a / b) } else { None };
    let cr = if missing == [0, 0] {
        ratio(costs[1], costs[0])
    } else {
        None
    };
    let p50 = t[1]["p50"]
        .as_f64()
        .zip(t[0]["p50"].as_f64())
        .and_then(|(a, b)| ratio(a, b));
    let p95 = t[1]["p95"]
        .as_f64()
        .zip(t[0]["p95"].as_f64())
        .and_then(|(a, b)| ratio(a, b));
    let quality =
        invalid == 0 && wrong == 0 && lost == 0 && historical_lost == 0 && unresolved == 0;
    let resources =
        cr.is_some_and(|r| r <= 1.03) && [p50, p95].iter().all(|r| r.is_some_and(|r| r <= 1.5));
    write_new(
        output,
        &json!({"version":"scoped-definition-probe","candidate_template":spec_path,"operations":ended.len(),"attempted_operations":started.len(),"uncertain_operations":started.len()-ended.len(),"questions":18,"known_cost_usd":costs.iter().sum::<f64>(),"unknown_cost_operations":missing.iter().sum::<usize>(),"control_cost_usd":costs[0],"candidate_cost_usd":costs[1],"http_latency_ms":{"control":t[0],"candidate":t[1]},"ratios":{"cost":cr,"p50":p50,"p95":p95},"invalid_operations":invalid,"candidate_unpermitted_ids":wrong,"lost_control_useful_ids":lost,"lost_historical_useful_ids":historical_lost,"unfixed_regressions":unresolved,"useful_ids_gained_vs_control":gains,"passes_quality_screen":quality,"passes_resource_screen":resources,"eligible_for_runtime_implementation":quality&&resources,"fresh_validation":false,"promoted":false,"journal_sha256":digest(&d.join("journal.jsonl"))?,"cases":rows}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exclusive_role_is_preserved_without_fabricating_separate_judgments() {
        let p = json!({"state":{"user_prompt":"¿Quién hizo Orbe?"},"questions":{"case_anchor":{"criteria":{"A":"Orbe","none":"unknown"}},"memory_00_relation":{"criteria":{"A":"contribution","B":"refutation","none":"absent"}},"memory_01_relation":{"criteria":{"A":"contribution","B":"refutation","none":"absent"}}}});
        let answer = |choice, probs| json!({"type":"choice","choice":choice,"probabilities":probs});
        let mut b = json!({"model":"typesafe/jev-1.13-20260917","answers":{"case_anchor":answer("A",json!({"A":1.0,"none":0.0})),"memory_00_relation":answer("A",json!({"A":1.0,"B":0.0,"none":0.0})),"memory_01_relation":answer("B",json!({"A":0.0,"B":1.0,"none":0.0}))}});
        let c = vec![
            json!({"memory_id":"m0","revision":1,"summary":"Orbe sigue pendiente."}),
            json!({"memory_id":"m1","revision":1,"summary":"Vega sigue pendiente."}),
        ];
        let (s, t) = selected(&b, &p, &c).unwrap();
        assert_eq!(s, BTreeSet::from(["m0".into()]));
        assert!(t["per_memory"]["scoped_checks"].is_null());
        assert_eq!(
            t["per_memory"]["role_checks"][1]["role"],
            "premise_refutation"
        );
        assert_eq!(
            t["per_memory"]["role_checks"][1]["literal_anchor"]["present_in_source"],
            false
        );
        b["model"] = json!("inception/mercury-decide-20260930");
        assert!(selected(&b, &p, &c).is_err());
        let (other, _) = selected_model(&b, &p, &c, "inception/mercury-decide-20260930").unwrap();
        assert_eq!(other, s);
        assert_eq!(b["model"], "inception/mercury-decide-20260930");
        b["model"] = json!("typesafe/jev-1.13-20260917");
        b["answers"]["memory_01_relation"]["probabilities"]["B"] = json!(0.4);
        assert!(selected(&b, &p, &c).is_err());
    }
    #[test]
    fn malformed_probabilities_are_not_a_negative_judgment() {
        let p = json!({"questions":{"memory_00_fact":{"criteria":{"A":"yes","none":"no"}}},"state":{"user_prompt":"¿qué?"}});
        let b = json!({"model":"typesafe/jev-1.13-20260917","answers":{"memory_00_fact":{"type":"choice","choice":"none","probabilities":{"none":1.0}}}});
        assert!(selected(&b, &p, &[]).is_err());
    }
}
