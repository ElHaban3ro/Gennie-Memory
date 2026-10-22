//! Quoted generative roles: six frozen questions, all seven candidates each.
use crate::{
    campaign::digest,
    isolated_provider_probe::{ids, lines, read},
    quote_role_contract,
    scoped_definition_probe::write_new,
    sparse_critic_probe::replacement,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const PROTOCOL: &str = "docs/evaluation/quoted-role-generative-v1-protocol-2026-10-20.md";
const CATALOG: &str = "outputs/iromem/evaluation/generative-role-catalog-2026-10-20.json";
fn inputs() -> Result<(Vec<Value>, Value), String> {
    let (rows, mut hashes) = replacement::screen_inputs()?;
    let catalog = read(Path::new(CATALOG))?;
    let model = &catalog["data"]["data"];
    if model["id"] != quote_role_contract::MODEL
        || !model["endpoints"]
            .as_array()
            .ok_or("sin endpoints")?
            .iter()
            .any(|e| {
                e["tag"] == "google-ai-studio"
                    && e["supported_parameters"]
                        .as_array()
                        .is_some_and(|p| p.iter().any(|s| s == "structured_outputs"))
            })
    {
        return Err("modelo/proveedor no vinculado al catálogo".into());
    }
    for p in [
        PROTOCOL,
        CATALOG,
        "tools/evaluation/src/quote_role_contract.rs",
    ] {
        hashes[p] = json!(digest(Path::new(p))?);
    }
    Ok((rows, hashes))
}
fn requests(rows: &[Value]) -> Vec<Value> {
    rows.iter().map(|r|json!({"id":r["id"],"payload":quote_role_contract::payload(r["role_payload"]["state"]["user_prompt"].as_str().unwrap(),r["candidates"].as_array().unwrap())})).collect()
}
pub fn prepare(d: &Path) -> Result<(), String> {
    let (rows, hashes) = inputs()?;
    let reqs = requests(&rows);
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    for (file, data) in [("inputs.jsonl", rows), ("requests.jsonl", reqs)] {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(d.join(file))
            .map_err(|e| e.to_string())?;
        for v in data {
            writeln!(f, "{v}").map_err(|e| e.to_string())?;
        }
        f.sync_all().map_err(|e| e.to_string())?;
    }
    write_new(
        &d.join("preparation.json"),
        &json!({"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"requests_sha256":digest(&d.join("requests.jsonl"))?,"source_hashes":hashes,"max_http_requests":6,"model":quote_role_contract::MODEL,"weight_revision_verified":false,"fresh_validation":false,"promoted":false}),
    )
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    let (rows, hashes) = inputs()?;
    let reqs = requests(&rows);
    let prep = read(&d.join("preparation.json"))?;
    let pre = read(&d.join("preflight.json"))?;
    if lines(&d.join("inputs.jsonl"))? != rows
        || lines(&d.join("requests.jsonl"))? != reqs
        || prep["source_hashes"] != hashes
        || prep["inputs_sha256"] != digest(&d.join("inputs.jsonl"))?
        || prep["requests_sha256"] != digest(&d.join("requests.jsonl"))?
        || pre["preparation_sha256"] != digest(&d.join("preparation.json"))?
    {
        return Err("filtro generativo no vinculado".into());
    }
    for (p, h) in pre["sources_sha256"].as_object().ok_or("sin fuentes")? {
        if digest(&d.join("source-executed").join(p))? != *h {
            return Err("fuente ejecutada alterada".into());
        }
    }
    let events = lines(&d.join("journal.jsonl"))?;
    let mut begun = BTreeSet::new();
    let mut ended = BTreeMap::new();
    let mut at = 0;
    let mut cost = 0.;
    let mut unknown = 0;
    let mut tokens = [0u64; 2];
    for e in &events {
        let id = e["id"].as_str().ok_or("sin ID")?;
        if e["kind"] == "started" {
            if at >= reqs.len() || reqs[at]["id"] != id || !begun.insert(id) {
                return Err("inicio fuera de orden/repetido".into());
            }
            at += 1;
        } else if e["kind"] == "response" || e["kind"] == "failed" {
            if !begun.contains(id) || ended.insert(id, e).is_some() {
                return Err("respuesta sin inicio o repetida".into());
            }
            if let Some(c) = e["body"]["usage"]["cost"]
                .as_f64()
                .filter(|c| c.is_finite() && *c >= 0.)
            {
                cost += c;
            } else {
                unknown += 1;
            }
            tokens[0] += e["body"]["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
            tokens[1] += e["body"]["usage"]["completion_tokens"]
                .as_u64()
                .unwrap_or(0);
        } else {
            return Err("evento desconocido".into());
        }
    }
    unknown += begun.len() - ended.len();
    let mut cases = Vec::new();
    let mut valid = 0;
    let mut wrong = 0;
    let mut lost = 0;
    let mut retained = 0;
    let mut exact = 0;
    let mut bretained = 0;
    let mut brejected = 0;
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        let query = row["role_payload"]["state"]["user_prompt"]
            .as_str()
            .unwrap();
        let result = ended
            .get(id)
            .filter(|e| e["kind"] == "response" && e["http_status"] == 200)
            .map(|e| {
                quote_role_contract::decode_body(
                    &e["body"],
                    query,
                    row["candidates"].as_array().unwrap(),
                )
            });
        if let Some(Ok((selected, trace))) = result {
            valid += 1;
            let gold = ids(&row["case"]["expected_ids"])?;
            let b = ids(&row["proposed_b_ids"])?;
            let bad = selected.difference(&gold).cloned().collect::<Vec<_>>();
            let missing = gold.difference(&selected).cloned().collect::<Vec<_>>();
            wrong += bad.len();
            lost += missing.len();
            retained += selected.intersection(&gold).count();
            exact += usize::from(selected == gold);
            bretained += b
                .intersection(&gold)
                .filter(|id| selected.contains(*id))
                .count();
            brejected += b
                .difference(&gold)
                .filter(|id| !selected.contains(*id))
                .count();
            cases.push(json!({"id":id,"valid":true,"selected_ids":selected,"unpermitted_ids":bad,"omitted_gold_ids":missing,"roles_and_literal_quotes":trace}));
        } else {
            cases.push(json!({"id":id,"valid":false,"error":result.map(|r|r.err())}));
        }
    }
    let passed =
        valid == 6 && unknown == 0 && wrong == 0 && lost == 0 && bretained == 5 && brejected == 6;
    write_new(
        out,
        &json!({"version":"quoted-role-generative-v1","http_requests":begun.len(),"ended_http_requests":ended.len(),"uncertain_http_requests":begun.len()-ended.len(),"valid_questions":valid,"valid_candidate_judgments":valid*7,"known_cost_usd":cost,"unknown_cost_operations":unknown,"reported_input_tokens":tokens[0],"reported_output_tokens":tokens[1],"unpermitted_ids":if valid==0{None}else{Some(wrong)},"omitted_gold_ids":if valid==0{None}else{Some(lost)},"retained_gold_ids":retained,"exact_sets":exact,"archived_b_permitted_retained":bretained,"archived_b_unpermitted_rejected":brejected,"screen_passed":passed,"resource_gate_measured":false,"fresh_validation":false,"promoted":false,"literal_grounding_is_not_semantic_proof":true,"weight_revision_verified":false,"journal_sha256":digest(&d.join("journal.jsonl"))?,"cases":cases}),
    )
}
