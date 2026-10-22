//! Source-bound known-development score collection. Never a promotion gate.
use crate::{campaign::digest, evidence};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const POLICY: &str = "per_memory_scored_v1";
fn read(p: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(p).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn lines(p: &Path) -> Result<Vec<Value>, String> {
    fs::read_to_string(p)
        .map_err(|e| e.to_string())?
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
        .collect()
}
fn text<'a>(v: &'a Value, k: &str) -> Result<&'a str, String> {
    v[k].as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("falta {k}"))
}
fn array<'a>(v: &'a Value, k: &str) -> Result<&'a Vec<Value>, String> {
    v[k].as_array().ok_or_else(|| format!("falta {k}"))
}
fn save(p: &Path, v: &Value) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)
        .map_err(|e| e.to_string())?;
    f.write_all(
        serde_json::to_string_pretty(v)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
}

pub fn prepare(root: &Path, directory: &Path) -> Result<(), String> {
    let (_, originals, manifest) = evidence::load(root)?;
    if manifest["version"] != "conversation-evidence-v5" || originals.len() != 64 {
        return Err("diagnóstico limitado a64conocidas v5".into());
    }
    if directory.exists() {
        return Err("elige directorio nuevo".into());
    }
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let cases: Vec<_> = originals
        .iter()
        .map(|c| {
            let mut r = c.clone();
            r["original_case_id"] = c["id"].clone();
            r["id"] = json!(format!("{}__{POLICY}", c["id"].as_str().unwrap()));
            r["request"]["provider"] = json!("jev");
            r["request"]["selection_policy"] = json!(POLICY);
            r
        })
        .collect();
    let case_path = directory.join("cases.jsonl");
    fs::write(
        &case_path,
        cases
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .map_err(|e| e.to_string())?;
    let source = root.join("development.jsonl");
    let corpus = root.join("memories.json");
    let protocol = Path::new("docs/evaluation/per-memory-score-diagnostic-protocol-2026-10-19.md");
    let inputs = [
        source.clone(),
        corpus.clone(),
        root.join("manifest.json"),
        case_path.clone(),
        protocol.to_path_buf(),
    ]
    .iter()
    .map(|p| Ok((p.to_string_lossy().into_owned(), json!(digest(p)?))))
    .collect::<Result<serde_json::Map<String, Value>, String>>()?;
    save(
        &directory.join("diagnostic.json"),
        &json!({"version":"per-memory-score-diagnostic-v1","operations":64,"max_http_requests":64,"source_cases":source,"corpus":corpus,"protocol":protocol,"inputs":inputs,"minimum_total_gap":0.05,"threshold_method":"midpoint","accept":"a>=threshold","promoted":false}),
    )
}

pub fn validate(directory: &Path) -> Result<(), String> {
    let spec = read(&directory.join("diagnostic.json"))?;
    if spec["operations"] != 64
        || spec["max_http_requests"] != 64
        || spec["minimum_total_gap"] != 0.05
        || spec["threshold_method"] != "midpoint"
        || spec["accept"] != "a>=threshold"
        || spec["promoted"] != false
    {
        return Err("método/límites alterados".into());
    }
    let inputs = spec["inputs"].as_object().ok_or("faltan huellas")?;
    let source = Path::new(text(&spec, "source_cases")?);
    let source_root = source.parent().ok_or("sin raíz fuente")?;
    for p in [
        source.to_path_buf(),
        source_root.join("manifest.json"),
        std::path::PathBuf::from(text(&spec, "corpus")?),
        std::path::PathBuf::from(text(&spec, "protocol")?),
        directory.join("cases.jsonl"),
    ] {
        if !inputs.contains_key(p.to_str().ok_or("ruta inválida")?) {
            return Err("fuente requerida sin huella".into());
        }
    }
    for (p, h) in inputs {
        if digest(Path::new(p))? != h.as_str().ok_or("hash inválido")? {
            return Err(format!("fuente alterada: {p}"));
        }
    }
    let (_, originals, manifest) = evidence::load(source_root)?;
    if manifest["version"] != "conversation-evidence-v5"
        || source != source_root.join("development.jsonl")
        || Path::new(text(&spec, "corpus")?) != source_root.join("memories.json")
    {
        return Err("fuente ajena al protocolo v5".into());
    }
    let cases = lines(&directory.join("cases.jsonl"))?;
    let mut seen = BTreeSet::new();
    if originals.len() != 64 || cases.len() != 64 {
        return Err("se requieren64casos".into());
    }
    for c in &cases {
        let id = text(c, "original_case_id")?;
        if !seen.insert(id) {
            return Err("caso duplicado".into());
        }
        let mut expected = originals
            .iter()
            .find(|o| o["id"] == id)
            .ok_or("caso ajeno")?
            .clone();
        expected["id"] = json!(format!("{id}__{POLICY}"));
        expected["original_case_id"] = json!(id);
        expected["request"]["provider"] = json!("jev");
        expected["request"]["selection_policy"] = json!(POLICY);
        if expected != *c {
            return Err("consulta/gold alterado".into());
        }
    }
    Ok(())
}

/// Checks scores independently of retrieval's types; does not force choice=argmax.
pub(crate) fn inspect(response: &Value) -> Result<Vec<Value>, String> {
    let candidates = array(&response["retrieval"], "candidates")?;
    if candidates.is_empty() {
        if response["classification_status"] != "skipped_empty" || !response["per_memory"].is_null()
        {
            return Err("búsqueda vacía incoherente".into());
        }
        return Ok(vec![]);
    }
    let payload_hash = response["per_memory"]["request_sha256"]
        .as_str()
        .ok_or("falta hash de payload")?;
    if payload_hash.len() != 64
        || !payload_hash
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("hash de payload inválido".into());
    }
    let items = array(&response["per_memory"], "items")?;
    let scores = array(&response["per_memory"], "choice_scores")?;
    if items.len() != candidates.len().min(8) || scores.len() != items.len() {
        return Err("scores omitidos o adicionales".into());
    }
    let mut out = Vec::new();
    for (i, ((item, score), candidate)) in items.iter().zip(scores).zip(candidates).enumerate() {
        if score["memory_id"] != item["memory_id"]
            || score["revision"] != item["revision"]
            || score["memory_id"] != candidate["memory_id"]
            || score["revision"] != candidate["revision"]
        {
            return Err("score ajeno o revisión cambiada".into());
        }
        let a = score["a"]
            .as_f64()
            .filter(|n| n.is_finite() && (0.0..=1.0).contains(n))
            .ok_or("score A inválido")?;
        let none = score["none"]
            .as_f64()
            .filter(|n| n.is_finite() && (0.0..=1.0).contains(n))
            .ok_or("score none inválido")?;
        if (a + none - 1.0).abs() > 0.02 {
            return Err("scores no conservados/válidos".into());
        }
        let choice = match text(item, "relation")? {
            "useful_evidence" => "A",
            "unrelated" => "none",
            _ => return Err("juicio no binario".into()),
        };
        out.push(json!({"memory_id":item["memory_id"],"revision":item["revision"],"position":i+1,"summary":candidate["summary"],"choice":choice,"a":a,"none":none,"choice_score_disagreement":(choice=="A" && a<none)||(choice=="none" && a>none)}));
    }
    Ok(out)
}

pub(crate) fn fit(scores: &[(f64, bool)], complete: bool) -> Value {
    let min_positive = scores
        .iter()
        .filter(|s| s.1)
        .map(|s| s.0)
        .min_by(f64::total_cmp);
    let max_negative = scores
        .iter()
        .filter(|s| !s.1)
        .map(|s| s.0)
        .max_by(f64::total_cmp);
    let gap = min_positive.zip(max_negative).map(|(p, n)| p - n);
    let feasible = complete && gap.is_some_and(|g| g >= 0.05);
    json!({"min_positive":min_positive,"max_negative":max_negative,"total_gap":gap,"required_total_gap":0.05,"all_score_operations_valid":complete,"separation_exists":feasible,"threshold":if feasible{min_positive.zip(max_negative).map(|(p,n)|(p+n)/2.0)}else{None},"method":"global midpoint; accept a>=threshold","discard_global_threshold_hypothesis":complete && !feasible})
}

pub(crate) fn operation_usage(response: &Value) -> Result<(Option<f64>, bool, bool), String> {
    let cost = match response.get("reported_cost_usd") {
        Some(Value::Null) => None,
        Some(v) => Some(
            v.as_f64()
                .filter(|c| c.is_finite() && *c >= 0.0)
                .ok_or("coste inválido")?,
        ),
        None => return Err("falta campo de coste".into()),
    };
    let skipped = response["status"] == "complete"
        && response["classification_status"] == "skipped_empty"
        && response["retrieval"]["candidates"] == json!([]);
    if skipped && cost.is_some() {
        return Err("coste declarado sin petición".into());
    }
    Ok((
        cost,
        response["status"] != "complete" || (cost.is_none() && !skipped),
        skipped,
    ))
}

pub fn report(directory: &Path, output: &Path) -> Result<(), String> {
    validate(directory)?;
    let spec = read(&directory.join("diagnostic.json"))?;
    let audit = read(&directory.join("audit.json"))?;
    let campaign = array(&audit, "campaigns")?.first().ok_or("sin campaña")?;
    if campaign["integrity"]["valid"] != true {
        return Err("auditoría no válida".into());
    }
    for (key, p) in [
        ("cases", directory.join("cases.jsonl")),
        ("journal", directory.join("journal.jsonl")),
        ("corpus", std::path::PathBuf::from(text(&spec, "corpus")?)),
    ] {
        if campaign["sha256"][key] != digest(&p)? {
            return Err("auditoría obsoleta".into());
        }
    }
    let cases = lines(&directory.join("cases.jsonl"))?;
    let journal = lines(&directory.join("journal.jsonl"))?;
    let mut observations = Vec::new();
    let mut rows = Vec::new();
    let mut failures = Vec::new();
    let mut scores = Vec::new();
    let mut known_cost = 0.0;
    let mut unknown = 0;
    let mut skipped_operations = 0;
    let mut unpresented = Vec::new();
    for c in &cases {
        let found: Vec<_> = journal
            .iter()
            .filter(|r| r["kind"] == "finished" && r["id"] == c["id"])
            .collect();
        if found.len() != 1 {
            return Err("operación ausente o repetida".into());
        }
        let row = found[0];
        let response = &row["response"];
        if row["expected_ids"] != c["expected_ids"]
            || response["requested_model"] != "typesafe/jev-1.13"
        {
            return Err("gold/modelo cambiado".into());
        }
        let (cost, missing_receipt, skipped) = operation_usage(response)?;
        known_cost += cost.unwrap_or(0.0);
        if !known_cost.is_finite() {
            return Err("desbordamiento coste conocido".into());
        }
        unknown += usize::from(missing_receipt);
        skipped_operations += usize::from(skipped);
        if response["status"] != "complete" {
            failures.push(json!({"id":c["id"],"error_code":response["error_code"]}));
            continue;
        }
        evidence::per_memory_case(c, response, 8, true)?;
        if !response["per_memory"]["verification"].is_null() {
            return Err("traza ajena".into());
        }
        let presented = inspect(response)?;
        let expected = array(c, "expected_ids")?;
        let omitted: Vec<_> = expected
            .iter()
            .filter(|id| !presented.iter().any(|p| p["memory_id"] == **id))
            .cloned()
            .collect();
        if !omitted.is_empty() {
            unpresented.push(json!({"id":c["id"],"useful_ids_outside_prefix":omitted}));
        }
        let mut wrong = Vec::new();
        for mut observation in presented {
            let useful = expected.contains(&observation["memory_id"]);
            scores.push((observation["a"].as_f64().unwrap(), useful));
            if observation["choice"] == "A" && !useful {
                wrong.push(observation["memory_id"].clone());
            }
            let hash_text = |value: &str| {
                <sha2::Sha256 as sha2::Digest>::digest(value.as_bytes())
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            observation["summary_sha256"] = json!(hash_text(
                observation["summary"].as_str().ok_or("resumen ausente")?
            ));
            observation["query_sha256"] = json!(hash_text(
                c["request"]["search"]["query"]
                    .as_str()
                    .ok_or("consulta ausente")?
            ));
            observation["request_sha256"] = response["per_memory"]["request_sha256"].clone();
            observation["expected_useful"] = json!(useful);
            observation["case_id"] = c["id"].clone();
            observation["source_case_id"] = c["original_case_id"].clone();
            observation["relation"] = c["relation"].clone();
            observation["requested_model"] = response["requested_model"].clone();
            observation["served_model"] = response["served_model"].clone();
            observations.push(observation);
        }
        rows.push(json!({"id":c["id"],"selected_ids":response["selected_ids"],"wrong_ids":wrong,"elapsed_ms":row["elapsed_ms"],"reported_cost_usd":response["reported_cost_usd"]}));
    }
    let fitted = fit(&scores, failures.is_empty());
    let threshold_sets:Vec<_>=fitted["threshold"].as_f64().map(|threshold|cases.iter().map(|case| {
        let selected:Vec<_>=observations.iter().filter(|o|o["case_id"]==case["id"] && o["a"].as_f64().is_some_and(|a|a>=threshold)).map(|o|o["memory_id"].clone()).collect();
        json!({"id":case["id"],"hypothetical_selected_ids":selected,"declared_useful_ids":case["expected_ids"],"all_declared_useful_ids_present":case["expected_ids"].as_array().unwrap().iter().all(|id|selected.contains(id))})
    }).collect()).unwrap_or_default();
    let disagreements = observations
        .iter()
        .filter(|o| o["choice_score_disagreement"] == true)
        .count();
    save(
        output,
        &json!({"version":"per-memory-score-report-v1","operations":cases.len(),"completed_score_operations":rows.len(),"judgments":scores.len(),"choice_score_disagreements":disagreements,"hypothetical_threshold_sets":threshold_sets,"observations":observations,"cases":rows,"failures":failures,"useful_evidence_outside_prefix":unpresented,"fit":fitted,"eligible_for_future_gate_experiment":fitted["separation_exists"]==true && unpresented.is_empty(),"known_cost_usd":known_cost,"unknown_cost_operations":unknown,"skipped_empty_operations":skipped_operations,"all_receipts_complete":unknown==0,"promoted":false,"fresh_validation":false,"journal_sha256":digest(&directory.join("journal.jsonl"))?,"audit_sha256":digest(&directory.join("audit.json"))?,"warning":"Known correlated synthetic development. Score is not factual confidence. No threshold adoption or changed selection in this collection."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_http_is_not_an_unknown_receipt_and_partial_failed_cost_is_not_total() {
        let mut r = json!({"status":"complete","classification_status":"skipped_empty","retrieval":{"candidates":[]},"reported_cost_usd":null});
        assert_eq!(operation_usage(&r).unwrap(), (None, false, true));
        r["status"] = json!("degraded");
        r["reported_cost_usd"] = json!(0.001);
        assert_eq!(operation_usage(&r).unwrap(), (Some(0.001), true, false));
        r["status"] = json!("complete");
        r["classification_status"] = json!("complete");
        r["reported_cost_usd"] = json!(0.0);
        assert_eq!(operation_usage(&r).unwrap(), (Some(0.0), false, false));
        r["reported_cost_usd"] = Value::Null;
        assert_eq!(operation_usage(&r).unwrap(), (None, true, false));
    }
    #[test]
    fn midpoint_requires_complete_cohort_and_predeclared_total_margin() {
        let good = [(0.9, true), (0.8, true), (0.2, false)];
        assert_eq!(fit(&good, true)["threshold"], 0.5);
        assert_eq!(fit(&good, false)["threshold"], Value::Null);
        assert_eq!(
            fit(&[(0.4, true), (0.7, false)], true)["discard_global_threshold_hypothesis"],
            true
        );
        assert_eq!(fit(&[(0.9, true)], true)["separation_exists"], false);
    }
    #[test]
    fn score_audit_preserves_disagreement_and_rejects_wrong_revision_or_missing_score() {
        let mut response = json!({"retrieval":{"candidates":[{"memory_id":"a","revision":2,"summary":"fact"}]},"per_memory":{"items":[{"memory_id":"a","revision":2,"relation":"useful_evidence"}],"choice_scores":[{"memory_id":"a","revision":2,"a":0.3,"none":0.7}]}});
        response["per_memory"]["request_sha256"] = json!("a".repeat(64));
        assert_eq!(
            inspect(&response).unwrap()[0]["choice_score_disagreement"],
            true
        );
        response["per_memory"]["choice_scores"][0]["revision"] = json!(1);
        assert!(inspect(&response).is_err());
        response["per_memory"]["choice_scores"] = json!([]);
        assert!(inspect(&response).is_err());
    }
}
