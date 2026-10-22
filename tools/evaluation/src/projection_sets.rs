//! Unordered grounded token sets; no dropping references or changing linked group order.
use crate::projection_ids;
use serde_json::{Value, json};
pub fn contract() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/assertion-projection-sets-v4.json"
    ))
    .unwrap()
}
pub fn payload(query: &str, memory: &str) -> Value {
    let c = contract();
    let mut p = projection_ids::payload(query, memory);
    let mut template = c["schema"].clone();
    template["properties"]["version"] =
        projection_ids::contract()["schema"]["properties"]["version"].clone();
    assert_eq!(
        template,
        projection_ids::contract()["schema"],
        "v4 changes only group order semantics and version"
    );
    p["messages"][0]["content"] = c["instructions"].clone();
    p["response_format"]["json_schema"]["name"] = json!("assertion_projection_sets");
    p["response_format"]["json_schema"]["schema"]["properties"]["version"] =
        c["schema"]["properties"]["version"].clone();
    p
}
fn order(v: &mut Value) -> Result<(), String> {
    let a = v.as_array_mut().ok_or("conjunto inválido")?;
    for x in a.iter() {
        let s = x.as_str().ok_or("ID no textual")?;
        let suffix = s
            .strip_prefix('q')
            .or_else(|| s.strip_prefix('m'))
            .ok_or("prefijo de ID inválido")?;
        if suffix.parse::<usize>().is_err() {
            return Err("ID inválido".into());
        }
    }
    a.sort_by_key(|x| x.as_str().unwrap()[1..].parse::<usize>().unwrap());
    Ok(())
}
pub fn inspect(query: &str, memory: &str, w: &Value) -> Result<Value, String> {
    if w["version"] != "assertion-projection-sets-v4" {
        return Err("versión inválida".into());
    }
    let mut normalized = w.clone();
    normalized["version"] = json!("assertion-projection-ids-v3");
    order(&mut normalized["query"]["identity_token_indices"])?;
    for field in ["requested_token_groups", "premise_token_groups"] {
        for group in normalized["query"][field]
            .as_array_mut()
            .ok_or("grupos inválidos")?
        {
            order(group)?;
        }
    }
    for claim in normalized["memory_claims"]
        .as_array_mut()
        .ok_or("afirmaciones inválidas")?
    {
        for field in ["subject_token_indices", "property_token_indices"] {
            order(&mut claim[field])?;
        }
    }
    projection_ids::inspect(query, memory, &normalized)
}
pub fn inspect_response(query: &str, memory: &str, body: &Value) -> Result<Value, String> {
    let c = contract();
    if !c["accepted_served_models"]
        .as_array()
        .unwrap()
        .contains(&body["model"])
    {
        return Err("modelo servido no admitido".into());
    }
    if body["choices"].as_array().is_none_or(|a| a.len() != 1)
        || body["choices"][0]["finish_reason"] != "stop"
    {
        return Err("respuesta incompleta".into());
    }
    let text = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("falta extracción")?;
    let w: Value = serde_json::from_str(text).map_err(|_| "JSON inválido")?;
    let check = inspect(query, memory, &w)?;
    Ok(json!({"witness":w,"check":check}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn set_order_does_not_change_membership_or_group_index() {
        assert!(order(&mut json!(["ñ0"])).is_err());
        let mut x = json!(["q8", "q7"]);
        order(&mut x).unwrap();
        assert_eq!(x, json!(["q7", "q8"]));
        let mut y = json!(["q7", "q7"]);
        order(&mut y).unwrap();
        assert_eq!(y, json!(["q7", "q7"]));
    }
    #[test]
    fn base_schema_and_source_specific_enums_are_preserved() {
        let a = projection_ids::payload("¿Peso Orbe?", "Orbe pesa ocho kilos.");
        let b = payload("¿Peso Orbe?", "Orbe pesa ocho kilos.");
        assert_eq!(a["messages"][1], b["messages"][1]);
        assert_eq!(
            a["response_format"]["json_schema"]["schema"]["properties"]["query"],
            b["response_format"]["json_schema"]["schema"]["properties"]["query"]
        );
    }
}
