//! Semantic role ablation and exact-reference duplicate normalization; prior results stay unchanged.
use crate::independent_projection as base;
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub fn spec() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/independent-role-matcher-v2.json"
    ))
    .unwrap()
}
pub fn check_projection(kind: &str, text: &str, body: &Value) -> Result<Value, String> {
    if kind == "query" {
        return base::check_projection(kind, text, body);
    }
    if kind != "memory" {
        return Err("etapa desconocida".into());
    }
    let original = base::response_content(body)?;
    let claims = original["assertions"]
        .as_array()
        .ok_or("afirmaciones ausentes")?;
    if original.as_object().is_none_or(|o| o.len() != 1) || claims.len() > 8 {
        return Err("esquema inválido".into());
    }
    let mut unique = Vec::new();
    let mut signatures = BTreeMap::new();
    let mut mapping = Vec::new();
    for claim in claims {
        // Validate every assertion independently; only repetitions of exactly the same source IDs merge.
        base::check_memory(text, &json!({"assertions":[claim]}))?;
        let mut canonical = claim.clone();
        for ids in canonical.as_object_mut().unwrap().values_mut() {
            ids.as_array_mut()
                .unwrap()
                .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        }
        let signature = canonical.to_string();
        let index = if let Some(i) = signatures.get(&signature) {
            *i
        } else {
            let n = unique.len();
            unique.push(claim.clone());
            signatures.insert(signature, n);
            n
        };
        mapping.push(index);
    }
    let normalized = json!({"assertions":unique});
    let decoded = base::check_memory(text, &normalized)?;
    Ok(
        json!({"source_raw":original,"raw":normalized,"decoded":decoded,"raw_to_unique_assertion":mapping,"removed_exact_duplicates":claims.len()-unique.len(),"semantic_alignment_certified":false}),
    )
}
pub fn match_payload(query: &str, memory: &str, q: &Value, m: &Value) -> Value {
    let mut p = base::match_payload(query, memory, q, m);
    let mut input: Value =
        serde_json::from_str(p["messages"][1]["content"].as_str().unwrap()).unwrap();
    input.as_object_mut().unwrap().remove("allowed_edges");
    p["messages"][0]["content"] = spec()["instructions"].clone();
    p["messages"][1]["content"] = json!(input.to_string());
    p["response_format"]["json_schema"]["name"] = json!("independent_semantic_role");
    p["response_format"]["json_schema"]["schema"] = json!({"type":"object","additionalProperties":false,"required":["relation"],"properties":{"relation":{"type":"string","enum":["supports_requested_part","refutes_premise","related_context","unrelated"]}}});
    p
}
pub fn check_match(body: &Value, _q: &Value, _m: &Value) -> Result<Value, String> {
    let c = base::response_content(body)?;
    if c.as_object().is_none_or(|a| a.len() != 1) {
        return Err("campos inválidos".into());
    }
    let relation = c["relation"]
        .as_str()
        .filter(|r| {
            matches!(
                *r,
                "supports_requested_part" | "refutes_premise" | "related_context" | "unrelated"
            )
        })
        .ok_or("relación inválida")?;
    Ok(
        json!({"selected":matches!(relation,"supports_requested_part"|"refutes_premise"),"relation":relation,"semantic_alignment_certified":false}),
    )
}
pub fn matching_requests(
    rows: &[Value],
    plan: &Value,
    frames: &BTreeMap<String, Value>,
) -> Result<(Vec<Value>, Vec<Value>), String> {
    let (requests, skipped) = base::matching_requests(rows, plan, frames)?;
    let mut out = Vec::new();
    for (mut r, source) in requests.into_iter().map(|r| {
        let s = rows.iter().find(|s| s["id"] == r["id"]).unwrap();
        (r, s)
    }) {
        let q = &frames[r["query_projection_id"].as_str().unwrap()];
        let m = &frames[r["memory_projection_id"].as_str().unwrap()];
        r["payload"] = match_payload(
            source["payload"]["state"]["query"].as_str().unwrap(),
            source["payload"]["state"]["saved_memory"].as_str().unwrap(),
            q,
            m,
        );
        out.push(r);
    }
    Ok((out, skipped))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn body(w: Value) -> Value {
        json!({"model":"openai/gpt-4.1-mini","choices":[{"finish_reason":"stop","message":{"content":w.to_string()}}]})
    }
    #[test]
    fn only_identical_source_reference_assertions_merge() {
        let a = json!({"subjects":["m0"],"predicates":["m1"],"objects":[],"values":["m2"],"context":[],"negators":[],"conditions":[]});
        let b = body(json!({"assertions":[a,a]}));
        let x = check_projection("memory", "Orbe pesa ocho.", &b).unwrap();
        assert_eq!(x["removed_exact_duplicates"], 1);
        assert_eq!(x["raw_to_unique_assertion"], json!([0, 0]));
        assert_eq!(x["decoded"]["assertions"].as_array().unwrap().len(), 1);
    }
    #[test]
    fn duplicate_normalization_does_not_accept_bad_source_ids() {
        let a = json!({"subjects":["m9"],"predicates":["m1"],"objects":[],"values":[],"context":[],"negators":[],"conditions":[]});
        assert!(
            check_projection(
                "memory",
                "Orbe pesa ocho.",
                &body(json!({"assertions":[a,a]}))
            )
            .is_err()
        );
    }
    #[test]
    fn role_payload_keeps_originals_and_projections_without_suggested_edges() {
        let q =
            json!({"identity_tokens":["Orbe"],"requested_groups":[["peso"]],"premise_groups":[]});
        let m = json!({"assertions":[{}]});
        let p = match_payload("¿Peso Orbe?", "Orbe pesa ocho.", &q, &m);
        let input: Value =
            serde_json::from_str(p["messages"][1]["content"].as_str().unwrap()).unwrap();
        assert_eq!(input["query_projection"], q);
        assert_eq!(input["memory_projection"], m);
        assert!(input.get("allowed_edges").is_none());
    }
}
