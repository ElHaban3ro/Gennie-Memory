//! Retrospective source-only review. Structural checks do not certify semantics.
use crate::isolated_provider_probe::{lines, read};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::OpenOptions,
    io::Write,
    path::Path,
};
const DIMENSIONS: [&str; 5] = [
    "subject",
    "attribute",
    "scope_conditions",
    "polarity",
    "status",
];
fn id(v: &Value, key: &str) -> Result<String, String> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("falta {key}"))
}
fn digest(p: &Path) -> Result<String, String> {
    crate::campaign::digest(p)
}
fn reviews(inputs: &[Value], rows: &[Value]) -> Result<BTreeMap<String, Value>, String> {
    let sources = inputs
        .iter()
        .map(|r| Ok((id(r, "review_id")?, r)))
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    if sources.len() != inputs.len() || rows.len() != inputs.len() {
        return Err("roster incompleto o repetido".into());
    }
    let mut checked = BTreeMap::new();
    for row in rows {
        let key = id(row, "review_id")?;
        let source = sources.get(&key).ok_or("revisión fuera de roster")?;
        let fields = row.as_object().ok_or("revisión no es objeto")?;
        if fields.len() != 4
            || !["review_id", "verdict", "dimensions", "issues"]
                .iter()
                .all(|k| fields.contains_key(*k))
        {
            return Err("campos inesperados de revisión".into());
        }
        let dimensions = row["dimensions"]
            .as_object()
            .ok_or("dimensiones ausentes")?;
        if dimensions
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != DIMENSIONS.into_iter().collect()
            || dimensions
                .values()
                .any(|v| !matches!(v.as_str(), Some("preserved" | "defective" | "unclear")))
        {
            return Err("dimensiones inválidas".into());
        }
        let verdict = row["verdict"].as_str().ok_or("veredicto ausente")?;
        match verdict {
            "faithful" if dimensions.values().all(|v| v == "preserved") => {}
            "defective" if dimensions.values().any(|v| v == "defective") => {}
            "unclear"
                if dimensions.values().any(|v| v == "unclear")
                    && !dimensions.values().any(|v| v == "defective") => {}
            _ => return Err("veredicto incoherente".into()),
        }
        let issues = row["issues"].as_array().ok_or("incidencias ausentes")?;
        if (verdict == "faithful") != issues.is_empty() {
            return Err("incidencias incoherentes".into());
        }
        let mut covered = BTreeSet::new();
        for issue in issues {
            let dimension = id(issue, "dimension")?;
            if !DIMENSIONS.contains(&dimension.as_str()) || dimensions[&dimension] == "preserved" {
                return Err("incidencia de dimensión no marcada".into());
            }
            let quote = id(issue, "source_quote")?;
            if !source["source_text"]
                .as_str()
                .ok_or("fuente ausente")?
                .contains(&quote)
            {
                return Err("cita de fuente no literal".into());
            }
            id(issue, "frame_quote")?;
            id(issue, "explanation")?;
            covered.insert(dimension);
        }
        if dimensions
            .iter()
            .any(|(k, v)| v != "preserved" && !covered.contains(k))
        {
            return Err("dimensión sin explicación".into());
        }
        if checked.insert(key, row.clone()).is_some() {
            return Err("revisión duplicada".into());
        }
    }
    Ok(checked)
}
fn pair_verdict(query: &Value, memory: &Value) -> &'static str {
    if query["verdict"] == "defective" || memory["verdict"] == "defective" {
        "defective"
    } else if query["verdict"] == "unclear" || memory["verdict"] == "unclear" {
        "unclear"
    } else {
        "faithful"
    }
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    let prep = read(&d.join("preparation.json"))?;
    let complete = read(&d.join("review-completion.json"))?;
    for (file, h) in [
        ("blind-inputs.jsonl", &prep["inputs_sha256"]),
        ("private-map.jsonl", &prep["private_map_sha256"]),
        ("review.jsonl", &complete["review_sha256"]),
        (
            "private-all-pair-join.jsonl",
            &complete["private_all_pair_join_sha256"],
        ),
    ] {
        if digest(&d.join(file))? != *h {
            return Err("artefacto de revisión alterado".into());
        }
    }
    if digest(Path::new(
        "docs/evaluation/source-frame-review-v1-protocol-2026-10-20.md",
    ))? != complete["protocol_sha256"]
    {
        return Err("protocolo de revisión alterado".into());
    }
    if complete["review_complete_before_outcome_join"] != true
        || complete["source_only_reviewer"] != true
        || complete["human_review"] != false
        || complete["same_model_family"] != true
    {
        return Err("procedencia de revisión inválida".into());
    }
    let parent = Path::new("outputs/iromem/evaluation/independent-role-matcher-v2-2026-10-19");
    if digest(&parent.join("report.json"))? != prep["parent_report_sha256"]
        || digest(&parent.join("projections.json"))? != prep["projections_sha256"]
    {
        return Err("padre alterado".into());
    }
    let validation = d.join(format!("parent-revalidation-{}.json", std::process::id()));
    crate::independent_role_audit::report(parent, &validation)?;
    let reconstructed = read(&validation)?;
    std::fs::remove_file(validation).map_err(|e| e.to_string())?;
    let parent_report = read(&parent.join("report.json"))?;
    if reconstructed != parent_report {
        return Err("padre no reproducible".into());
    }
    let input = lines(&d.join("blind-inputs.jsonl"))?;
    if input.len() != 125 {
        return Err("se requieren125frames".into());
    }
    let checked = reviews(&input, &lines(&d.join("review.jsonl"))?)?;
    let sources = input
        .iter()
        .map(|r| Ok((id(r, "review_id")?, r)))
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let projections = read(&parent.join("projections.json"))?;
    let projections = projections.as_array().ok_or("proyecciones ausentes")?;
    let original = lines(Path::new(
        "outputs/iromem/evaluation/independent-projection-v1-2026-10-19/projection.requests.jsonl",
    ))?;
    let map = lines(&d.join("private-map.jsonl"))?;
    if map.len() != 125 {
        return Err("mapa incompleto".into());
    }
    let mut projected = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for m in map {
        let rid = id(&m, "review_id")?;
        let pid = id(&m, "projection_id")?;
        let source = sources.get(&rid).ok_or("mapa ajeno")?;
        let p = projections
            .iter()
            .find(|r| r["id"] == pid)
            .ok_or("proyección inexistente")?;
        let o = original
            .iter()
            .find(|r| r["id"] == pid)
            .ok_or("fuente inexistente")?;
        if !seen.insert(rid.clone())
            || source["kind"] != o["kind"]
            || source["source_text"] != o["source_text"]
            || source["decoded_frame"] != p["projection"]["decoded"]
            || !p["error"].is_null()
            || projected.insert(pid, checked[&rid].clone()).is_some()
        {
            return Err("fuente/frame/mapa incompatibles".into());
        }
    }
    let requests = lines(&parent.join("requests.jsonl"))?;
    let observations = parent_report["observations"]
        .as_array()
        .ok_or("observaciones ausentes")?;
    let mut joined = Vec::new();
    let mut counts = BTreeMap::<String, usize>::new();
    for r in &requests {
        let o = observations
            .iter()
            .find(|o| o["id"] == r["id"])
            .ok_or("observación ausente")?;
        let q = projected
            .get(&id(r, "query_projection_id")?)
            .ok_or("frame de pregunta ausente")?;
        let m = projected
            .get(&id(r, "memory_projection_id")?)
            .ok_or("frame de memoria ausente")?;
        let verdict = pair_verdict(q, m);
        let classification = id(o, "classification")?;
        let prefix = if o["is_control"] == true {
            "control"
        } else {
            "candidate"
        };
        *counts
            .entry(format!("{prefix}/{classification}/{verdict}"))
            .or_default() += 1;
        let content: Value = serde_json::from_str(
            r["payload"]["messages"][1]["content"]
                .as_str()
                .ok_or("contenido ausente")?,
        )
        .map_err(|e| e.to_string())?;
        joined.push(json!({"pair_id":r["id"],"query_review_id":q["review_id"],"memory_review_id":m["review_id"],"frame_pair_verdict":verdict,"historical_classification":classification,"historical_selected":o["selected"],"historical_expected_useful":o["expected_useful"],"is_control":o["is_control"],"question":content["query"],"memory":content["saved_memory"],"query_frame":content["query_projection"],"memory_frame":content["memory_projection"],"historical_relation":o["match"]["check"]["relation"]}));
    }
    if joined.len() != 128 {
        return Err("unión incompleta".into());
    }
    let mut verdict_counts = BTreeMap::<String, usize>::new();
    for r in checked.values() {
        *verdict_counts.entry(id(r, "verdict")?).or_default() += 1;
    }
    let errors_with_faithful = joined
        .iter()
        .filter(|r| {
            r["frame_pair_verdict"] == "faithful"
                && matches!(
                    r["historical_classification"].as_str(),
                    Some("false_positive" | "false_negative")
                )
        })
        .count();
    let mut defective_dimensions = BTreeMap::<String, usize>::new();
    for r in checked.values() {
        for dim in DIMENSIONS {
            if r["dimensions"][dim] == "defective" {
                *defective_dimensions.entry(dim.into()).or_default() += 1;
            }
        }
    }
    let result = json!({"version":"source-frame-retrospective-audit-v1","review_frames":125,"query_frames":20,"memory_frames":105,"frame_verdict_counts":verdict_counts,"defective_dimension_counts":defective_dimensions,"pair_counts":counts,"pairs_joined":128,"historical_errors_with_both_frames_reviewed_faithful":errors_with_faithful,"fresh_validation":false,"human_review":false,"same_model_family_review":true,"semantic_fidelity_is_reviewer_judgment":true,"causal_intervention_performed":false,"new_network_calls":0,"new_provider_cost_usd":0,"parent_results_replaced":false,"review_sha256":digest(&d.join("review.jsonl"))?,"rows":joined});
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(out)
        .map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> Vec<Value> {
        vec![json!({"review_id":"x","source_text":"The result is pending."})]
    }
    fn good() -> Value {
        json!({"review_id":"x","verdict":"faithful","dimensions":{"subject":"preserved","attribute":"preserved","scope_conditions":"preserved","polarity":"preserved","status":"preserved"},"issues":[]})
    }
    #[test]
    fn missing_duplicate_and_unknown_reviews_fail() {
        assert!(reviews(&input(), &[]).is_err());
        assert!(reviews(&input(), &[good(), good()]).is_err());
        let mut r = good();
        r["review_id"] = json!("other");
        assert!(reviews(&input(), &[r]).is_err());
    }
    #[test]
    fn defective_review_requires_literal_source_and_explanation() {
        let mut r = good();
        r["verdict"] = json!("defective");
        r["dimensions"]["status"] = json!("defective");
        r["issues"] = json!([{"dimension":"status","source_quote":"pending","frame_quote":"status omitted","explanation":"Missing pending state"}]);
        assert!(reviews(&input(), &[r.clone()]).is_ok());
        let mut blank = r.clone();
        blank["issues"][0]["source_quote"] = json!(" ");
        assert!(reviews(&input(), &[blank]).is_err());
        r["issues"][0]["source_quote"] = json!("completed");
        assert!(reviews(&input(), &[r]).is_err());
    }
    #[test]
    fn uncertainty_cannot_be_counted_as_faithful_or_bad_without_review() {
        let a = good();
        let mut b = good();
        b["verdict"] = json!("unclear");
        b["dimensions"]["attribute"] = json!("unclear");
        assert_eq!(pair_verdict(&a, &b), "unclear");
        b["verdict"] = json!("defective");
        assert_eq!(pair_verdict(&a, &b), "defective");
        assert_eq!(pair_verdict(&a, &a), "faithful");
    }
}
