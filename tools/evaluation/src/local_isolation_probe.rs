//! Retrospective causal diagnostic. No candidate adoption or favorable-run selection.
use crate::{
    campaign::digest,
    contribution_role_contract as contract,
    isolated_provider_probe::{lines, read},
    scoped_definition_probe::write_new,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const PARENT: &str = "outputs/iromem/evaluation/contribution-local-v5-known18-paired-2026-10-20";
const PROTOCOL: &str = "docs/evaluation/local-isolation-diagnostic-protocol-2026-10-20.md";
pub fn isolated(payload: &Value, key: &str) -> Result<Value, String> {
    let questions = payload["questions"].as_object().ok_or("sin preguntas")?;
    let question = questions.get(key).ok_or("pregunta no presentada")?;
    let mut p = payload.clone();
    p["questions"] = json!({key:question});
    Ok(p)
}
pub fn decode_single(
    body: &Value,
    payload: &Value,
    memory: &Value,
    key: &str,
) -> Result<String, String> {
    let answers = body["answers"]
        .as_object()
        .filter(|a| a.len() == 1 && a.contains_key(key))
        .ok_or("namespace aislado inválido")?;
    let questions = payload["questions"]
        .as_object()
        .filter(|a| a.len() == 1 && a.contains_key(key))
        .ok_or("pregunta aislada inválida")?;
    // Rename only an actually asked and answered key, never fabricate another answer.
    let mut b = body.clone();
    b["answers"] = json!({"memory_00_relation":answers[key]});
    let mut p = payload.clone();
    p["questions"] = json!({"memory_00_relation":questions[key]});
    Ok(contract::decode(&b, &p, &[memory.clone()])?[0].clone())
}
fn inputs() -> Result<(Vec<Value>, Value), String> {
    let d = Path::new(PARENT);
    let report = read(&d.join("report.json"))?;
    if report["candidate_canonical_role_errors"] != 6
        || report["quality_gate_passed"] != false
        || report["valid_operations"] != json!([18, 18])
        || report["journal_sha256"] != digest(&d.join("journal.jsonl"))?
    {
        return Err("origen diagnóstico no vinculado".into());
    }
    let cases = lines(&d.join("inputs.jsonl"))?;
    let requests = lines(&d.join("requests.jsonl"))?;
    let events = lines(&d.join("journal.jsonl"))?;
    let mut selected = Vec::new();
    for result in report["cases"].as_array().ok_or("sin casos origen")? {
        let candidate = result["operations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["variant"] == "candidate")
            .ok_or("sin candidata")?;
        let errors = candidate["canonical_role_differences"]
            .as_array()
            .ok_or("sin diferencias")?;
        if errors.is_empty() {
            continue;
        }
        let row = cases
            .iter()
            .find(|c| c["id"] == result["id"])
            .ok_or("sin fuente de caso")?;
        let memories = row["memories"].as_array().unwrap();
        let labels = row["labels"].as_array().unwrap();
        let key = format!("{}__candidate", row["id"].as_str().unwrap());
        let request = requests
            .iter()
            .find(|r| r["id"] == key)
            .ok_or("sin request origen")?;
        let full = contract::payload_local(row["question"].as_str().unwrap(), memories);
        if full != request["payload"] {
            return Err("payload origen no reproducible".into());
        }
        let mut indexes = errors
            .iter()
            .map(|e| {
                memories
                    .iter()
                    .position(|m| m["id"] == e["id"])
                    .ok_or("error sin fuente")
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        // One correctly classified useful control per affected case, if available.
        if let Some(i) = labels.iter().enumerate().find_map(|(i, l)| {
            if l["role"] != "none" && !indexes.contains(&i) {
                Some(i)
            } else {
                None
            }
        }) {
            indexes.insert(i);
        }
        let event = events
            .iter()
            .find(|e| e["kind"] == "response" && e["id"] == key)
            .ok_or("sin respuesta origen")?;
        let original_roles = contract::decode(&event["body"], &full, memories)?;
        selected.push(json!({"id":row["id"],"case":row,"indexes":indexes,"payload":full,"parent_roles":original_roles}));
    }
    if selected.len() != 4
        || selected
            .iter()
            .map(|r| r["indexes"].as_array().unwrap().len())
            .sum::<usize>()
            != 9
    {
        return Err("roster diagnóstico distinto de4/9".into());
    }
    let mut hashes = serde_json::Map::new();
    for file in [
        "report.json",
        "journal.jsonl",
        "inputs.jsonl",
        "requests.jsonl",
        "preparation.json",
        "preflight.json",
    ] {
        let p = d.join(file);
        hashes.insert(p.to_str().unwrap().into(), json!(digest(&p)?));
    }
    for p in [
        PROTOCOL,
        "tools/evaluation/src/local_isolation_probe.rs",
        "tools/evaluation/src/contribution_role_contract.rs",
        "bootstrap/src/openrouter/per_memory/local_isolation_probe.rs",
    ] {
        hashes.insert(p.into(), json!(digest(Path::new(p))?));
    }
    Ok((selected, Value::Object(hashes)))
}
fn requests(rows: &[Value]) -> Result<Vec<Value>, String> {
    let mut result = Vec::new();
    for (n, r) in rows.iter().enumerate() {
        let mut group = Vec::new();
        for i in r["indexes"].as_array().unwrap() {
            let i = i.as_u64().unwrap() as usize;
            let key = format!("memory_{i:02}_relation");
            group.push(json!({"id":format!("{}__single_{i:02}",r["id"].as_str().unwrap()),"case_id":r["id"],"mode":"single","index":i,"question_key":key,"payload":isolated(&r["payload"],&key)?}));
        }
        let full = json!({"id":format!("{}__full",r["id"].as_str().unwrap()),"case_id":r["id"],"mode":"full","payload":r["payload"]});
        if n % 2 == 0 {
            result.push(full);
            result.extend(group);
        } else {
            result.extend(group);
            result.push(full);
        }
    }
    Ok(result)
}
pub fn prepare(d: &Path) -> Result<(), String> {
    let (rows, hashes) = inputs()?;
    let reqs = requests(&rows)?;
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    for (name, data) in [("inputs.jsonl", rows), ("requests.jsonl", reqs)] {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(d.join(name))
            .map_err(|e| e.to_string())?;
        for v in data {
            writeln!(f, "{v}").map_err(|e| e.to_string())?;
        }
        f.sync_all().map_err(|e| e.to_string())?;
    }
    write_new(
        &d.join("preparation.json"),
        &json!({"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"requests_sha256":digest(&d.join("requests.jsonl"))?,"source_hashes":hashes,"max_http_requests":13,"fresh_validation":false,"adoption_gate":false}),
    )
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    let (rows, hashes) = inputs()?;
    let reqs = requests(&rows)?;
    let prep = read(&d.join("preparation.json"))?;
    let pre = read(&d.join("preflight.json"))?;
    if lines(&d.join("inputs.jsonl"))? != rows
        || lines(&d.join("requests.jsonl"))? != reqs
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || prep["requests_sha256"] != digest(&d.join("requests.jsonl"))?
        || pre["preparation_sha256"] != digest(&d.join("preparation.json"))?
    {
        return Err("preparación de aislamiento no vinculada".into());
    }
    // Parent artifacts remain current and immutable; executed code is checked from archive.
    for (p, h) in hashes.as_object().unwrap() {
        if p.starts_with(PARENT) && prep["source_hashes"][p] != *h {
            return Err("origen diagnóstico alterado".into());
        }
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
            return Err("fuentes preparadas/ejecutadas difieren".into());
        }
    }
    let mut starts = BTreeSet::new();
    let mut ended = BTreeMap::new();
    let mut at = 0;
    let mut fee = 0.;
    let mut unknown = 0;
    for e in lines(&d.join("journal.jsonl"))? {
        let id = e["id"].as_str().ok_or("sin id")?.to_owned();
        if e["kind"] == "started" {
            if at >= reqs.len() || reqs[at]["id"] != id || !starts.insert(id) {
                return Err("inicio duplicado/fuera de orden".into());
            }
            at += 1;
        } else if e["kind"] == "response" || e["kind"] == "failed" {
            if !starts.contains(&id) || ended.contains_key(&id) {
                return Err("fin duplicado/sin inicio".into());
            }
            let cost = if e["kind"] == "response" {
                e["body"]["usage"]["cost"].as_f64()
            } else {
                e["partial_usage"]["reported_cost_usd"].as_f64()
            };
            if let Some(c) = cost.filter(|c| c.is_finite() && *c >= 0.) {
                fee += c;
            } else {
                unknown += 1;
            }
            ended.insert(id, e);
        } else {
            return Err("evento desconocido".into());
        }
    }
    unknown += starts.len() - ended.len();
    let (mut full_errors, mut single_errors, mut comparisons, mut changed) = (0, 0, 0, 0);
    let mut full_changed_from_parent = 0;
    let mut details = Vec::new();
    let mut invalid = Vec::new();
    for row in &rows {
        let case = &row["case"];
        let memories = case["memories"].as_array().unwrap();
        let id = row["id"].as_str().unwrap();
        let full_id = format!("{id}__full");
        let full = ended
            .get(&full_id)
            .filter(|e| e["kind"] == "response")
            .map(|e| contract::decode(&e["body"], &row["payload"], memories));
        for index in row["indexes"].as_array().unwrap() {
            let i = index.as_u64().unwrap() as usize;
            let single_id = format!("{id}__single_{i:02}");
            let request = reqs.iter().find(|r| r["id"] == single_id).unwrap();
            let key = request["question_key"].as_str().unwrap();
            let single = ended
                .get(&single_id)
                .filter(|e| e["kind"] == "response")
                .map(|e| decode_single(&e["body"], &request["payload"], &memories[i], key));
            let single_valid = matches!(&single, Some(Ok(_)));
            let single_error = single.as_ref().and_then(|r| r.as_ref().err()).cloned();
            match (&full,single){(Some(Ok(roles)),Some(Ok(role)))=>{let expected=case["labels"][i]["role"].as_str().unwrap();comparisons+=1;full_changed_from_parent+=usize::from(row["parent_roles"][i]!=roles[i]);full_errors+=usize::from(roles[i]!=expected);single_errors+=usize::from(role!=expected);changed+=usize::from(role!=roles[i]);details.push(json!({"case_id":id,"memory_id":memories[i]["id"],"source":memories[i]["text"],"expected_canonical_role":expected,"parent_role":row["parent_roles"][i],"full_role":roles[i],"single_role":role,"original_question_key":key,"normalized_key":"memory_00_relation"}));},_=>invalid.push(json!({"case_id":id,"index":i,"full_error":full.as_ref().and_then(|r|r.as_ref().err()),"single_valid":single_valid,"single_error":single_error}))}
        }
    }
    write_new(
        out,
        &json!({"version":"local-source-isolation-diagnostic-v1","http_requests":starts.len(),"ended_http_requests":ended.len(),"unrun_operations":13-starts.len(),"known_cost_usd":fee,"unknown_cost_operations":unknown,"compared_judgment_pairs":comparisons,"full_context_canonical_errors":if comparisons==0{None}else{Some(full_errors)},"single_question_canonical_errors":if comparisons==0{None}else{Some(single_errors)},"changed_roles":changed,"full_roles_changed_from_parent":full_changed_from_parent,"all_expected_roles_correct_under_isolation":comparisons==9&&single_errors==0,"diagnostic_complete":comparisons==9&&unknown==0,"fresh_validation":false,"adoption_gate":false,"resource_gate_measured":false,"parent_results_replaced":false,"journal_sha256":digest(&d.join("journal.jsonl"))?,"judgments":details,"invalid":invalid}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn isolation_changes_only_question_set_and_keeps_original_key() {
        let m = vec![
            json!({"id":"a","revision":1,"text":"one"}),
            json!({"id":"b","revision":1,"text":"two"}),
        ];
        let full = contract::payload_local("q", &m);
        let p = isolated(&full, "memory_01_relation").unwrap();
        assert_eq!(p["state"], full["state"]);
        assert_eq!(p["questions"].as_object().unwrap().len(), 1);
        assert_eq!(
            p["questions"]["memory_01_relation"],
            full["questions"]["memory_01_relation"]
        );
        assert!(
            !p["questions"]
                .as_object()
                .unwrap()
                .contains_key("memory_00_relation")
        );
    }
    #[test]
    fn normalize_only_actual_answer_and_reject_same_size_wrong_namespace() {
        let m = json!({"id":"b","revision":3,"text":"two"});
        let full = contract::payload_local(
            "q",
            &[json!({"id":"a","revision":1,"text":"one"}), m.clone()],
        );
        let p = isolated(&full, "memory_01_relation").unwrap();
        let mut b = json!({"model":contract::REVISION,"answers":{"memory_01_relation":{"type":"choice","choice":"A","probabilities":{"A":1.,"B":0.,"C":0.,"none":0.}}}});
        assert_eq!(
            decode_single(&b, &p, &m, "memory_01_relation").unwrap(),
            "A"
        );
        let a = b["answers"]
            .as_object_mut()
            .unwrap()
            .remove("memory_01_relation")
            .unwrap();
        b["answers"]["memory_00_relation"] = a;
        assert!(decode_single(&b, &p, &m, "memory_01_relation").is_err());
    }
}
