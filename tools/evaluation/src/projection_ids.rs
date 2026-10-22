//! Source-specific constrained IDs; numeric decode preserves the original witness exactly.
use crate::projection_tokens;
use serde_json::{Value, json};
pub fn contract() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/assertion-projection-ids-v3.json"
    ))
    .unwrap()
}
pub fn payload(query: &str, memory: &str) -> Value {
    let mut p = projection_tokens::payload(query, memory);
    let c = contract();
    let mut input: Value =
        serde_json::from_str(p["messages"][1]["content"].as_str().unwrap()).unwrap();
    let mut schema = c["schema"].clone();
    let mut refs = Vec::new();
    for (field, prefix) in [("query_tokens", "q"), ("memory_tokens", "m")] {
        let rows = input[field].as_array_mut().unwrap();
        let mut ids = Vec::new();
        for (i, row) in rows.iter_mut().enumerate() {
            let id = json!(format!("{prefix}{i}"));
            row[0] = id.clone();
            ids.push(id);
        }
        refs.push(json!({"type":"string","enum":ids}));
    }
    let q = &mut schema["properties"]["query"]["properties"];
    q["identity_token_indices"]["items"] = refs[0].clone();
    for k in ["requested_token_groups", "premise_token_groups"] {
        q[k]["items"]["items"] = refs[0].clone();
    }
    let m = &mut schema["properties"]["memory_claims"]["items"]["properties"];
    for k in ["subject_token_indices", "property_token_indices"] {
        m[k]["items"] = refs[1].clone();
    }
    for k in ["evidence_token_range", "negation_token_range"] {
        for f in ["first", "last"] {
            m[k]["properties"][f] = refs[1].clone();
        }
    }
    p["messages"][0]["content"] = c["instructions"].clone();
    p["messages"][1]["content"] = json!(input.to_string());
    p["response_format"]["json_schema"]["name"] = json!("assertion_projection_ids");
    p["response_format"]["json_schema"]["schema"] = schema;
    p
}
fn decode(v: &mut Value, prefix: &str, n: usize) -> Result<(), String> {
    let s = v.as_str().ok_or("ID no textual")?;
    let index = s
        .strip_prefix(prefix)
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|i| *i < n)
        .ok_or("ID de otra fuente o inexistente")?;
    if s != format!("{prefix}{index}") {
        return Err("ID no canónico".into());
    }
    *v = json!(index);
    Ok(())
}
fn group(v: &mut Value, prefix: &str, n: usize) -> Result<(), String> {
    for x in v.as_array_mut().ok_or("grupo inválido")? {
        decode(x, prefix, n)?;
    }
    Ok(())
}
pub fn inspect(query: &str, memory: &str, w: &Value) -> Result<Value, String> {
    if w["version"] != "assertion-projection-ids-v3" {
        return Err("versión inválida".into());
    }
    let mut decoded = w.clone();
    decoded["version"] = json!("assertion-projection-tokens-v2");
    let qn = query.split_whitespace().count();
    let mn = memory.split_whitespace().count();
    group(&mut decoded["query"]["identity_token_indices"], "q", qn)?;
    for field in ["requested_token_groups", "premise_token_groups"] {
        for x in decoded["query"][field]
            .as_array_mut()
            .ok_or("grupos inválidos")?
        {
            group(x, "q", qn)?;
        }
    }
    for claim in decoded["memory_claims"]
        .as_array_mut()
        .ok_or("afirmaciones inválidas")?
    {
        for field in ["subject_token_indices", "property_token_indices"] {
            group(&mut claim[field], "m", mn)?;
        }
        for field in ["evidence_token_range", "negation_token_range"] {
            if !claim[field].is_null() {
                for end in ["first", "last"] {
                    decode(&mut claim[field][end], "m", mn)?;
                }
            }
        }
    }
    projection_tokens::inspect(query, memory, &decoded)
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
    fn dynamic_schema_separates_namespaces_and_only_enumerates_real_tokens() {
        let p = payload("¿Peso Orbe?", "Orbe pesa ocho kilos.");
        let s = &p["response_format"]["json_schema"]["schema"];
        assert_eq!(
            s["properties"]["query"]["properties"]["identity_token_indices"]["items"]["enum"],
            json!(["q0", "q1"])
        );
        assert_eq!(
            s["properties"]["memory_claims"]["items"]["properties"]["property_token_indices"]["items"]
                ["enum"],
            json!(["m0", "m1", "m2", "m3"])
        );
    }
    #[test]
    fn invented_ids_wrong_source_and_ambiguous_spelling_are_rejected() {
        for s in ["q8", "m0", "q01"] {
            let mut v = json!(s);
            assert!(decode(&mut v, "q", 2).is_err());
        }
        let mut v = json!("q1");
        decode(&mut v, "q", 2).unwrap();
        assert_eq!(v, 1);
    }
}
