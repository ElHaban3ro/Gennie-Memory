//! Independent reconstruction of the selective critic experiment, outside product.
pub(crate) mod replacement;
use crate::{
    campaign::digest,
    isolated_provider_probe::{ids, lines, read},
    scoped_definition_probe::{selected_model, source_rows, write_new},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const PROTOCOL: &str = "docs/evaluation/sparse-role-critic-pilot-v1-protocol-2026-10-20.md";
const ROLE: &str = "outputs/iromem/evaluation/scoped-anchor-role-v5-2026-10-19/requests.jsonl";
const MODELS: [&str; 3] = [
    "inception/mercury-decide",
    "typesafe/jev-1.13",
    "microsoft/microsoft-decision-1",
];
const REVISIONS: [&str; 3] = [
    "inception/mercury-decide-20260930",
    "typesafe/jev-1.13-20260917",
    "microsoft/microsoft-decision-1-20261009",
];
fn inputs() -> Result<(Vec<Value>, Value), String> {
    let (mut source, mut hashes) = source_rows()?;
    let archive = read(Path::new(
        "outputs/iromem/evaluation/quality-goal-2026-10-19/snapshots/031/source-provenance.json",
    ))?;
    if archive["artifacts_sha256"][ROLE] != digest(Path::new(ROLE))? {
        return Err("payload de roles alterado".into());
    }
    let protocol_archive =
        Path::new("outputs/iromem/evaluation/quality-goal-2026-10-19/snapshots/032/source")
            .join(PROTOCOL);
    if digest(&protocol_archive)? != digest(Path::new(PROTOCOL))? {
        return Err("protocolo alterado".into());
    }
    hashes[ROLE] = json!(digest(Path::new(ROLE))?);
    hashes[PROTOCOL] = json!(digest(Path::new(PROTOCOL))?);
    for p in [
        "tests/fixtures/scoped-anchor-role-v5.json",
        "docs/evaluation/sparse-role-critic-pilot-v1-execution.md",
    ] {
        hashes[p] = json!(digest(Path::new(p))?);
    }
    let role = lines(Path::new(ROLE))?;
    for row in &mut source {
        let id = row["id"].as_str().unwrap();
        let cohort = if id.starts_with("cev4") { "v4" } else { "v5" };
        let path = format!(
            "outputs/iromem/evaluation/scoped-anchor-v1-2026-10-19/paired-{cohort}/journal.jsonl"
        );
        let e = lines(Path::new(&path))?
            .into_iter()
            .find(|e| {
                e["kind"] == "finished"
                    && e["id"] == format!("{id}__jev_per_memory_scoped_anchor_v1")
            })
            .ok_or("sin fuente completa")?;
        let payload = role
            .iter()
            .find(|r| r["id"] == format!("{id}__definitions"))
            .ok_or("sin payload roles")?["payload"]
            .clone();
        row["full_candidates"] = e["response"]["retrieval"]["candidates"].clone();
        if row["full_candidates"]
            .as_array()
            .filter(|c| !c.is_empty() && c.len() <= 20)
            .is_none()
        {
            return Err("se requieren candidatos recuperados, máximo veinte".into());
        }
        for (n, c) in row["candidates"].as_array().unwrap().iter().enumerate() {
            if payload["state"]["memories"][format!("memory_{n:02}")] != c["summary"] {
                return Err("memoria no vinculada".into());
            }
        }
        row["role_payload"] = payload;
    }
    Ok((source, hashes))
}
pub fn prepare(d: &Path) -> Result<(), String> {
    prepare_for(d, 1)
}
pub fn prepare_context(d: &Path) -> Result<(), String> {
    prepare_for(d, 2)
}
pub fn prepare_http2(d: &Path) -> Result<(), String> {
    prepare_for(d, 4)
}
pub fn prepare_targeted(d: &Path) -> Result<(), String> {
    prepare_for(d, 3)
}
pub fn prepare_direct_perplexity(d: &Path) -> Result<(), String> {
    prepare_for(d, 6)
}
fn prepare_for(d: &Path, version: u8) -> Result<(), String> {
    let parallel = version >= 2;
    let targeted = version >= 3;
    let direct = version == 6;
    let http2 = version >= 4;
    let (rows, mut hashes) = inputs()?;
    if parallel {
        let p = "docs/evaluation/context-role-critics-v2-protocol-2026-10-20.md";
        hashes[p] = json!(digest(Path::new(p))?);
    }
    if targeted {
        let p = "docs/evaluation/context-targeted-critics-v3-protocol-2026-10-20.md";
        hashes[p] = json!(digest(Path::new(p))?);
    }
    if http2 {
        for p in [
            "docs/evaluation/context-critics-http2-v4-protocol-2026-10-20.md",
            "docs/decisions/0022-opt-in-http2-evaluation.md",
            "Cargo.toml",
            "Cargo.lock",
            "bootstrap/Cargo.toml",
        ] {
            hashes[p] = json!(digest(Path::new(p))?);
        }
    }
    if direct {
        let p = "docs/evaluation/direct-role-perplexity-v1-protocol-2026-10-20.md";
        hashes[p] = json!(digest(Path::new(p))?);
        let (_, guards) = replacement::screen_inputs()?;
        hashes
            .as_object_mut()
            .unwrap()
            .extend(guards.as_object().unwrap().clone());
    }
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(d.join("inputs.jsonl"))
        .map_err(|e| e.to_string())?;
    for r in rows {
        writeln!(f, "{r}").map_err(|e| e.to_string())?;
    }
    f.sync_all().map_err(|e| e.to_string())?;
    write_new(
        &d.join("preparation.json"),
        &json!({"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"source_hashes":hashes,"direct_perplexity":direct,"http2_transport":http2,"targeted_critics":targeted,"parallel_critics":parallel,"max_http_requests":if direct{36}else{72},"known_queries":18,"fresh_validation":false,"promoted":false}),
    )
}
fn payload(template: &Value, candidates: &[Value], model: &str) -> Value {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/scoped-anchor-role-v5.json"
    ))
    .unwrap();
    let mut p = template.clone();
    p["model"] = json!(model);
    p["state"]["memories"] = json!({});
    let mut qs = serde_json::Map::new();
    if let Some(a) = template["questions"].get("case_anchor") {
        qs.insert("case_anchor".into(), a.clone());
    }
    for (n, c) in candidates.iter().enumerate() {
        let k = format!("memory_{n:02}");
        p["state"]["memories"][&k] = c["summary"].clone();
        qs.insert(format!("{k}_relation"),json!({"type":"choice","instructions":spec["relation_instruction"].as_str().unwrap().replace("{key}",&k),"criteria":spec["criteria"]}));
    }
    p["questions"] = json!(qs);
    p
}
pub(crate) fn baseline(row: &Value) -> Value {
    let mut criteria = serde_json::Map::new();
    for (n, c) in row["full_candidates"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        criteria.insert(char::from(b'A' + n as u8).to_string(), c["summary"].clone());
    }
    criteria.insert(
        "none".into(),
        json!("No listed memory contains the requested fact."),
    );
    json!({"model":"openai/gpt-6-luna-decisions","state":{"user_prompt":row["role_payload"]["state"]["user_prompt"],"scope":row["role_payload"]["state"]["scope"]},"questions":{"relevance":{"type":"choice","instructions":"Select the one saved memory whose stated fact answers the user's specific question, including clear paraphrases. A matching person or project name alone is not enough. Choose none only when no listed fact supplies the requested answer. Do not guess facts absent from the memories.","criteria":criteria}}})
}
pub(crate) fn baseline_selected(
    body: &Value,
    p: &Value,
    c: &[Value],
) -> Result<BTreeSet<String>, String> {
    // Reuse schema/probability checks via one dummy scoped fact for each baseline option.
    let a = body["answers"]
        .as_object()
        .filter(|m| m.len() == 1 && m.contains_key("relevance"))
        .ok_or("baseline namespace inválido")?;
    let mut surrogate = body.clone();
    surrogate["answers"] = json!({"memory_00_fact":a["relevance"],"memory_00_correction":{"type":"choice","choice":"none","probabilities":{"A":0.,"none":1.}}});
    let mut sp = json!({"state":{"user_prompt":p["state"]["user_prompt"]},"questions":{"memory_00_fact":p["questions"]["relevance"],"memory_00_correction":{"criteria":{"A":"yes","none":"no"}}}});
    // selected_model maps a non-A baseline to false, but validates its complete choice schema first.
    sp["questions"]["memory_00_fact"]["type"] = json!("choice");
    selected_model(
        &surrogate,
        &sp,
        &c[..1],
        "openai/gpt-6-luna-decisions-20261006",
    )?;
    match a["relevance"]["choice"].as_str().ok_or("sin choice")? {
        "none" => Ok(BTreeSet::new()),
        s => {
            let n = s
                .as_bytes()
                .first()
                .and_then(|b| b.checked_sub(b'A'))
                .ok_or("etiqueta inválida")? as usize;
            if s.len() != 1 || n >= c.len() {
                return Err("etiqueta fuera de rango".into());
            }
            Ok(BTreeSet::from([c[n]["memory_id"]
                .as_str()
                .unwrap()
                .to_owned()]))
        }
    }
}
fn pct(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    Some(v[((v.len() as f64 * p).ceil() as usize).saturating_sub(1)])
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    let (rows, mut hashes) = inputs()?;
    let prep = read(&d.join("preparation.json"))?;
    let parallel = prep["parallel_critics"] == true;
    let targeted = prep["targeted_critics"] == true;
    let http2 = prep["http2_transport"] == true;
    let direct = prep["direct_perplexity"] == true;
    if direct {
        let p = "docs/evaluation/direct-role-perplexity-v1-protocol-2026-10-20.md";
        hashes[p] = json!(digest(Path::new(p))?);
        let (_, guards) = replacement::screen_inputs()?;
        hashes
            .as_object_mut()
            .unwrap()
            .extend(guards.as_object().unwrap().clone());
    }
    if http2 {
        for p in [
            "docs/evaluation/context-critics-http2-v4-protocol-2026-10-20.md",
            "docs/decisions/0022-opt-in-http2-evaluation.md",
            "Cargo.toml",
            "Cargo.lock",
            "bootstrap/Cargo.toml",
        ] {
            let archived = d.join("source-executed").join(p);
            hashes[p] = json!(digest(if archived.exists() {
                &archived
            } else {
                Path::new(p)
            })?);
        }
    }
    if targeted {
        let p = "docs/evaluation/context-targeted-critics-v3-protocol-2026-10-20.md";
        hashes[p] = json!(digest(Path::new(p))?);
    }
    if parallel {
        let p = "docs/evaluation/context-role-critics-v2-protocol-2026-10-20.md";
        hashes[p] = json!(digest(Path::new(p))?);
    }
    let pre = read(&d.join("preflight.json"))?;
    if lines(&d.join("inputs.jsonl"))? != rows
        || prep["source_hashes"] != hashes
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || pre["preparation_sha256"] != digest(&d.join("preparation.json"))?
    {
        return Err("preparación no vinculada".into());
    }
    for (p, h) in pre["sources_sha256"].as_object().ok_or("sin fuentes")? {
        if digest(&d.join("source-executed").join(p))? != *h {
            return Err("fuente ejecutada alterada".into());
        }
    }
    let raw_events = lines(&d.join("journal.jsonl"))?;
    let mut versions = std::collections::BTreeMap::<String, usize>::new();
    for e in raw_events.iter().filter(|e| e["kind"] == "response") {
        let v = e["http_version"].as_str().unwrap_or("unrecorded");
        *versions.entry(v.into()).or_default() += 1;
    }
    let events = if parallel {
        sequential_audit_order(&raw_events)?
    } else {
        raw_events
    };
    let mut at = 0;
    let mut calls = 0;
    let mut unknown = 0;
    let mut cost = [0.; 2];
    let mut times = [vec![], vec![]];
    let mut cases = Vec::new();
    let mut valid = [0; 2];
    let mut wrong = 0;
    let mut lost = 0;
    let mut histloss = 0;
    let mut unresolved = 0;
    let mut gains = 0;
    let mut removed = 0;
    for (n, row) in rows.iter().enumerate() {
        let mut selections: [Option<BTreeSet<String>>; 2] = [None, None];
        let expected = ids(&row["case"]["expected_ids"])?;
        let history = ids(&row["historical_selected_ids"])?;
        let mut observations = Vec::new();
        for variant in if n % 2 == 0 {
            ["control", "candidate"]
        } else {
            ["candidate", "control"]
        } {
            if at == events.len() {
                break;
            }
            let slot = usize::from(variant == "candidate");
            let id = format!("{}__{variant}", row["id"].as_str().unwrap());
            if events[at] != json!({"kind":"operation_started","id":id}) {
                return Err("inicio u orden inválido".into());
            }
            at += 1;
            let original = if slot == 0 {
                row["full_candidates"].as_array().unwrap()
            } else {
                row["candidates"].as_array().unwrap()
            };
            let mut current = original.clone();
            let mut keep = BTreeSet::new();
            let mut error = false;
            let mut stage_times = 0.;
            let mut op_calls = 0;
            let mut had_b = false;
            let mut proposed_b = BTreeSet::new();
            let mut first_critic_ms: f64 = 0.;
            for stage in 0..if slot == 0 || direct { 1 } else { 3 } {
                if current.is_empty() && !(parallel && stage > 0 && had_b) {
                    break;
                }
                let stage_name = if slot == 0 {
                    "baseline"
                } else {
                    ["proposal", "jev_critic", "microsoft_critic"][stage]
                };
                let presented = if parallel && stage > 0 {
                    original.clone()
                } else {
                    current.clone()
                };
                let mut p = if slot == 0 {
                    baseline(row)
                } else {
                    payload(
                        &row["role_payload"],
                        &presented,
                        if direct {
                            "perplexity/pplx-decider-v1.2"
                        } else {
                            MODELS[stage]
                        },
                    )
                };
                if targeted && stage > 0 {
                    p["questions"].as_object_mut().unwrap().retain(|k, _| {
                        k == "case_anchor"
                            || original.iter().enumerate().any(|(n, c)| {
                                k == &format!("memory_{n:02}_relation")
                                    && proposed_b.contains(c["memory_id"].as_str().unwrap())
                            })
                    });
                }
                let started = events.get(at).ok_or("falta inicioHTTP")?;
                if started["kind"] != "started"
                    || started["id"] != id
                    || started["stage"] != stage_name
                    || started["payload"] != p
                    || started["candidates"] != json!(presented)
                    || started["payload_sha256"] != digest_json(&p)
                {
                    return Err("etapa o payload no reconstruible".into());
                }
                at += 1;
                calls += 1;
                op_calls += 1;
                let Some(e) = events.get(at) else {
                    unknown += 1;
                    error = true;
                    break;
                };
                if e["id"] != id
                    || e["stage"] != stage_name
                    || !["response", "failed"].iter().any(|k| e["kind"] == *k)
                {
                    return Err("respuesta HTTP no vinculada".into());
                }
                at += 1;
                let stage_elapsed = e["elapsed_ms"]
                    .as_f64()
                    .filter(|v| v.is_finite() && *v >= 0.)
                    .ok_or("tiempo inválido")?;
                if parallel && stage == 2 {
                    stage_times += stage_elapsed.max(first_critic_ms) - first_critic_ms;
                } else {
                    stage_times += stage_elapsed;
                }
                if stage == 1 {
                    first_critic_ms = stage_elapsed;
                }
                let amount = if e["kind"] == "response" {
                    e["body"]["usage"]["cost"].as_f64()
                } else {
                    e["partial_usage"]["reported_cost_usd"].as_f64()
                };
                if let Some(v) = amount.filter(|v| v.is_finite() && *v >= 0.) {
                    cost[slot] += v;
                } else {
                    unknown += 1;
                }
                if e["kind"] == "failed" {
                    error = true;
                    if parallel && stage == 1 {
                        continue;
                    }
                    break;
                }
                if slot == 0 {
                    match baseline_selected(&e["body"], &p, &current) {
                        Ok(s) => keep = s,
                        Err(_) => {
                            error = true;
                        }
                    }
                    break;
                }
                let (s, _) = match validated_subset(
                    &e["body"],
                    &p,
                    &presented,
                    if direct {
                        "perplexity/pplx-decider-v1.2-20261016"
                    } else {
                        REVISIONS[stage]
                    },
                ) {
                    Ok(x) => x,
                    Err(_) => {
                        error = true;
                        if parallel && stage == 1 {
                            continue;
                        }
                        break;
                    }
                };
                if direct {
                    keep = s;
                    current.clear();
                    break;
                }
                if stage == 0 {
                    keep = current
                        .iter()
                        .enumerate()
                        .filter(|(i, c)| {
                            s.contains(c["memory_id"].as_str().unwrap())
                                && e["body"]["answers"][format!("memory_{i:02}_relation")]["choice"]
                                    == "A"
                        })
                        .map(|(_, c)| c["memory_id"].as_str().unwrap().to_owned())
                        .collect();
                    current = current
                        .into_iter()
                        .enumerate()
                        .filter(|(i, c)| {
                            s.contains(c["memory_id"].as_str().unwrap())
                                && e["body"]["answers"][format!("memory_{i:02}_relation")]["choice"]
                                    == "B"
                        })
                        .map(|(_, c)| c)
                        .collect();
                    had_b = !current.is_empty();
                    proposed_b = current
                        .iter()
                        .map(|c| c["memory_id"].as_str().unwrap().to_owned())
                        .collect();
                } else {
                    current.retain(|c| s.contains(c["memory_id"].as_str().unwrap()));
                }
            }
            if slot == 1 {
                keep.extend(
                    current
                        .iter()
                        .map(|c| c["memory_id"].as_str().unwrap().to_owned()),
                );
            }
            if at == events.len() {
                observations
                    .push(json!({"variant":variant,"status":"uncertain","http_requests":op_calls}));
                break;
            }
            let end = &events[at];
            at += 1;
            let ordered = original
                .iter()
                .filter_map(|c| {
                    let id = c["memory_id"].as_str().unwrap();
                    keep.contains(id).then(|| id.to_owned())
                })
                .collect::<Vec<_>>();
            if end["kind"] != "operation_finished"
                || end["id"] != id
                || end["status"] != if error { "failed" } else { "complete" }
                || (!error && end["selected_ids"] != json!(ordered))
                || (error && !end["selected_ids"].is_null())
            {
                return Err("resultado no coincide con etapas".into());
            }
            let elapsed = end["elapsed_ms"]
                .as_f64()
                .filter(|v| v.is_finite() && *v >= stage_times)
                .ok_or("tiempo cadena inválido")?;
            times[slot].push(elapsed);
            if !error {
                valid[slot] += 1;
                selections[slot] = Some(keep.clone());
            }
            observations.push(json!({"variant":variant,"status":end["status"],"selected_ids":end["selected_ids"],"elapsed_ms":elapsed,"http_requests":op_calls}));
            if error && at != events.len() {
                return Err("continuación tras fallo técnico".into());
            }
        }
        let mut delta = Value::Null;
        if let Some(s) = &selections[1] {
            let bad = s.difference(&expected).cloned().collect::<Vec<_>>();
            let hl = history
                .intersection(&expected)
                .filter(|id| !s.contains(*id))
                .cloned()
                .collect::<Vec<_>>();
            wrong += bad.len();
            histloss += hl.len();
            let unresolved_case = row["must_fix_regression"] == true
                && (!bad.is_empty()
                    || (!expected.is_empty() && s.intersection(&expected).next().is_none()));
            unresolved += usize::from(unresolved_case);
            let omitted = expected.difference(s).cloned().collect::<Vec<_>>();
            delta = json!({"unpermitted_ids":bad,"lost_historical_useful_ids":hl,"omitted_gold_ids":omitted,"unresolved_regression":unresolved_case});
            if let Some(c) = &selections[0] {
                let l = c
                    .intersection(&expected)
                    .filter(|id| !s.contains(*id))
                    .cloned()
                    .collect::<Vec<_>>();
                lost += l.len();
                let g = s
                    .intersection(&expected)
                    .filter(|id| !c.contains(*id))
                    .count();
                gains += g;
                removed += c
                    .difference(&expected)
                    .filter(|id| !s.contains(*id))
                    .count();
                delta["lost_contemporary_useful_ids"] = json!(l);
                delta["gained_useful_ids"] = json!(g);
            }
        }
        cases.push(json!({"id":row["id"],"must_fix_regression":row["must_fix_regression"],"operations":observations,"semantic_delta":delta}));
    }
    if at != events.len() || calls > if direct { 36 } else { 72 } {
        return Err("eventos sobrantes o límite excedido".into());
    }
    let ratio =
        |a: Option<f64>, b: Option<f64>| a.zip(b).filter(|(_, b)| *b > 0.).map(|(a, b)| a / b);
    let ratios = [
        ratio(Some(cost[1]), Some(cost[0])),
        ratio(pct(&times[1], 0.5), pct(&times[0], 0.5)),
        ratio(pct(&times[1], 0.95), pct(&times[0], 0.95)),
    ];
    let complete = valid == [18, 18];
    let quality = complete
        && wrong == 0
        && lost == 0
        && histloss == 0
        && unresolved == 0
        && (gains > 0 || removed > 0);
    let resources = complete
        && unknown == 0
        && (!http2 || transport_valid(pre["http2_feature_enabled"] == true, &versions, calls))
        && ratios.iter().all(|r| r.is_some_and(|v| v <= 1.5));
    let mut result = json!({"http_versions":versions,"version":if direct{"direct-role-perplexity-v1"}else if http2{"context-critics-http2-v4"}else if targeted{"context-targeted-critics-v3"}else if parallel{"context-role-critics-v2"}else{"sparse-role-critic-pilot-v1"},"known_queries":18,"fresh_validation":false,"promoted":false,"journal_sha256":digest(&d.join("journal.jsonl"))?,"http_requests":calls,"known_cost_usd":cost.iter().sum::<f64>(),"unknown_cost_operations":unknown,"valid_operations":valid,"control_cost_usd":cost[0],"candidate_cost_usd":cost[1],"control_p50_ms":pct(&times[0],0.5),"candidate_p50_ms":pct(&times[1],0.5),"control_p95_ms":pct(&times[0],0.95),"candidate_p95_ms":pct(&times[1],0.95),"cost_p50_p95_ratios":ratios,"candidate_unpermitted_ids":wrong,"lost_contemporary_useful_ids":lost,"lost_historical_useful_ids":histloss,"unresolved_regression_cases":unresolved,"gained_useful_ids":gains,"removed_unpermitted_ids":removed,"quality_gate_passed":quality,"resources_gate_passed":resources,"pilot_passed":quality&&resources,"runtime_recall_measured":false,"cases":cases});
    if direct {
        let (rows, _) = replacement::screen_inputs()?;
        let guard = replacement::archived_guard(&result, &rows)?;
        let ok = guard["passed"] == true;
        result["quality_gate_passed"] = json!(quality && ok);
        result["pilot_passed"] = json!(quality && resources && ok);
        result["archived_proposal_guard"] = guard;
    }
    if !http2 {
        result.as_object_mut().unwrap().remove("http_versions");
    }
    write_new(out, &result)
}
fn transport_valid(
    feature: bool,
    versions: &std::collections::BTreeMap<String, usize>,
    calls: usize,
) -> bool {
    feature && calls > 0 && versions.len() == 1 && versions.get("HTTP/2.0") == Some(&calls)
}
fn validated_subset(
    body: &Value,
    p: &Value,
    candidates: &[Value],
    revision: &str,
) -> Result<(BTreeSet<String>, Value), String> {
    let qs = p["questions"].as_object().ok_or("sin preguntas")?;
    let answers = body["answers"].as_object().ok_or("sin respuestas")?;
    if qs.len() != answers.len() || qs.keys().any(|k| !answers.contains_key(k)) {
        return Err("respuestas no ligadas al subconjunto".into());
    }
    let mut normalized = p.clone();
    normalized["questions"] = json!({});
    let mut result = body.clone();
    result["answers"] = json!({});
    let mut chosen = Vec::new();
    let mut mapping = Vec::new();
    if let Some(a) = qs.get("case_anchor") {
        normalized["questions"]["case_anchor"] = a.clone();
        result["answers"]["case_anchor"] = answers["case_anchor"].clone();
    }
    for (n, c) in candidates.iter().enumerate() {
        let key = format!("memory_{n:02}_relation");
        if let Some(q) = qs.get(&key) {
            let canonical = format!("memory_{:02}_relation", chosen.len());
            normalized["questions"][&canonical] = q.clone();
            result["answers"][&canonical] = answers[&key].clone();
            chosen.push(c.clone());
            mapping.push(json!({"source_key":key,"normalized_key":canonical,"memory_id":c["memory_id"],"revision":c["revision"]}));
        }
    }
    if normalized["questions"].as_object().unwrap().len() != qs.len() || chosen.is_empty() {
        return Err("pregunta fuera de candidatos o lote vacío".into());
    }
    let (selected, mut trace) = selected_model(&result, &normalized, &chosen, revision)?;
    trace["answer_index_map"] = json!(mapping);
    Ok((selected, trace))
}
fn sequential_audit_order(events: &[Value]) -> Result<Vec<Value>, String> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < events.len() {
        let e = &events[at];
        if e["kind"] == "started" && e["stage"] == "jev_critic" {
            let chunk = events
                .get(at..at + 4)
                .ok_or("lote paralelo incompleto; conservar para auditoría de incidentes")?;
            if chunk[1]["kind"] != "started"
                || chunk[1]["stage"] != "microsoft_critic"
                || chunk.iter().any(|x| x["id"] != e["id"])
                || chunk[2]["stage"] != "jev_critic"
                || chunk[3]["stage"] != "microsoft_critic"
                || chunk[2..]
                    .iter()
                    .any(|x| x["kind"] != "response" && x["kind"] != "failed")
            {
                return Err("orden paralelo inválido".into());
            }
            out.extend([
                chunk[0].clone(),
                chunk[2].clone(),
                chunk[1].clone(),
                chunk[3].clone(),
            ]);
            at += 4;
        } else {
            out.push(e.clone());
            at += 1;
        }
    }
    Ok(out)
}
fn digest_json(v: &Value) -> String {
    {
        use sha2::{Digest, Sha256};
        Sha256::digest(serde_json::to_vec(v).unwrap())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transport_gate_requires_feature_and_every_observed_response_http2() {
        let h2 = std::collections::BTreeMap::from([("HTTP/2.0".into(), 4)]);
        assert!(transport_valid(true, &h2, 4));
        assert!(!transport_valid(false, &h2, 4));
        assert!(!transport_valid(true, &h2, 5));
        let h1 = std::collections::BTreeMap::from([("HTTP/1.1".into(), 4)]);
        assert!(!transport_valid(true, &h1, 4));
    }
    #[test]
    fn targeted_answers_use_only_requested_sources_and_preserve_original_index_map() {
        let c = vec![
            json!({"memory_id":"m0","revision":1,"summary":"other"}),
            json!({"memory_id":"m1","revision":2,"summary":"part"}),
        ];
        let p = json!({"state":{"user_prompt":"q"},"questions":{"memory_01_relation":{"type":"choice","criteria":{"A":"part","B":"correction","none":"no"}}}});
        let mut body = json!({"model":"rev","answers":{"memory_01_relation":{"type":"choice","choice":"A","probabilities":{"A":1.,"B":0.,"none":0.}}}});
        let (set, trace) = validated_subset(&body, &p, &c, "rev").unwrap();
        assert_eq!(set, BTreeSet::from(["m1".into()]));
        assert_eq!(
            trace["answer_index_map"][0]["source_key"],
            "memory_01_relation"
        );
        assert_eq!(
            trace["per_memory"]["role_checks"].as_array().unwrap().len(),
            1
        );
        body["answers"]["memory_00_relation"] = body["answers"]["memory_01_relation"].clone();
        assert!(validated_subset(&body, &p, &c, "rev").is_err());
    }
    #[test]
    fn parallel_audit_requires_both_starts_and_retains_failed_receipt() {
        let e = vec![
            json!({"kind":"started","id":"op","stage":"jev_critic"}),
            json!({"kind":"started","id":"op","stage":"microsoft_critic"}),
            json!({"kind":"failed","id":"op","stage":"jev_critic","partial_usage":{"reported_cost_usd":0.001}}),
            json!({"kind":"response","id":"op","stage":"microsoft_critic","body":{"usage":{"cost":0.002}}}),
        ];
        let ordered = sequential_audit_order(&e).unwrap();
        assert_eq!(ordered[1]["partial_usage"]["reported_cost_usd"], 0.001);
        assert_eq!(ordered[3]["body"]["usage"]["cost"], 0.002);
        let mut bad = e.clone();
        bad.swap(1, 2);
        assert!(sequential_audit_order(&bad).is_err());
        assert!(sequential_audit_order(&e[..3]).is_err());
    }
    #[test]
    fn baseline_validation_rejects_unexpected_answers_and_decodes_last_option() {
        let c = (0..20)
            .map(|n| json!({"memory_id":format!("m{n}"),"revision":1,"summary":"x"}))
            .collect::<Vec<_>>();
        let row =
            json!({"full_candidates":c,"role_payload":{"state":{"user_prompt":"q","scope":"s"}}});
        let p = baseline(&row);
        let probabilities = p["questions"]["relevance"]["criteria"]
            .as_object()
            .unwrap()
            .keys()
            .map(|k| (k.clone(), json!(if k == "T" { 1. } else { 0. })))
            .collect::<serde_json::Map<_, _>>();
        let mut body = json!({"model":"openai/gpt-6-luna-decisions-20261006","answers":{"relevance":{"type":"choice","choice":"T","probabilities":probabilities}}});
        assert_eq!(
            baseline_selected(&body, &p, &c).unwrap(),
            BTreeSet::from(["m19".into()])
        );
        body["answers"]["extra"] = json!({});
        assert!(baseline_selected(&body, &p, &c).is_err());
    }
}
