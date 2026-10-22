//! Evaluation-only grounded assertion contract; literal links are not semantic proof.
use serde_json::{Value, json};
use std::collections::BTreeSet;
pub fn contract() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/assertion-projection-v1.json"
    ))
    .unwrap()
}
pub fn payload(query: &str, memory: &str) -> Value {
    let c = contract();
    json!({"model":c["model"],"temperature":0,"max_tokens":c["max_tokens"],"provider":{"require_parameters":true,"allow_fallbacks":false},"messages":[{"role":"system","content":c["instructions"]},{"role":"user","content":json!({"query":query,"saved_memory":memory}).to_string()}],"response_format":{"type":"json_schema","json_schema":{"name":"assertion_projection","strict":true,"schema":c["schema"]}}})
}
fn keys(v: &Value, expected: &[&str]) -> Result<(), String> {
    let a = v.as_object().ok_or("objeto requerido")?;
    if a.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected.iter().copied().collect() {
        return Err("campos faltantes o ajenos".into());
    }
    Ok(())
}
fn quote<'a>(v: &'a Value, source: &str) -> Result<&'a str, String> {
    v.as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 2048 && source.contains(*s))
        .ok_or("cita no literal, vacía o excedida".into())
}
pub fn inspect(query: &str, memory: &str, w: &Value) -> Result<Value, String> {
    keys(w, &["version", "query", "memory_claims"])?;
    if w["version"] != "assertion-projection-v1" {
        return Err("versión inválida".into());
    }
    let q = &w["query"];
    keys(
        q,
        &["identity_quotes", "requested_quotes", "premise_quotes"],
    )?;
    for k in ["identity_quotes", "requested_quotes", "premise_quotes"] {
        let a = q[k]
            .as_array()
            .filter(|a| a.len() <= 16)
            .ok_or("lista de citas inválida")?;
        let mut seen = BTreeSet::new();
        for x in a {
            if !seen.insert(quote(x, query)?) {
                return Err("cita duplicada".into());
            }
        }
    }
    let claims = w["memory_claims"]
        .as_array()
        .filter(|a| a.len() <= 8)
        .ok_or("afirmaciones excedidas o inválidas")?;
    let mut selected = Vec::new();
    let mut unestablished = Vec::new();
    let mut seen = BTreeSet::new();
    for (i, c) in claims.iter().enumerate() {
        keys(
            c,
            &[
                "evidence_quote",
                "subject_quote",
                "property_quote",
                "polarity",
                "negation_scope_quote",
                "relation",
                "target_kind",
                "target_index",
                "same_case",
            ],
        )?;
        let evidence = quote(&c["evidence_quote"], memory)?;
        let property = quote(&c["property_quote"], evidence)?;
        let subject = if c["subject_quote"].is_null() {
            None
        } else {
            Some(quote(&c["subject_quote"], evidence)?)
        };
        let polarity = c["polarity"]
            .as_str()
            .filter(|s| matches!(*s, "affirmed" | "negated" | "conditional" | "unknown"))
            .ok_or("polaridad inválida")?;
        if polarity == "negated" {
            let n = quote(&c["negation_scope_quote"], evidence)?;
            if !n.contains(property) {
                return Err("negación sin predicado vinculado".into());
            }
        } else if !c["negation_scope_quote"].is_null() {
            return Err("negación incoherente".into());
        }
        let relation = c["relation"]
            .as_str()
            .filter(|s| {
                matches!(
                    *s,
                    "requested_fact"
                        | "requested_partial"
                        | "premise_refutation"
                        | "related_context"
                        | "unrelated"
                        | "unknown"
                )
            })
            .ok_or("relación inválida")?;
        let same = c["same_case"].as_bool().ok_or("falta juicio de caso")?;
        let useful = matches!(
            relation,
            "requested_fact" | "requested_partial" | "premise_refutation"
        );
        if useful {
            let (kind, field) = if relation == "premise_refutation" {
                ("premise", "premise_quotes")
            } else {
                ("requested", "requested_quotes")
            };
            if c["target_kind"] != kind
                || c["target_index"]
                    .as_u64()
                    .is_none_or(|n| n as usize >= q[field].as_array().unwrap().len())
            {
                return Err("destino de afirmación ausente o fuera de rango".into());
            }
        } else if c["target_kind"] != "none" || !c["target_index"].is_null() {
            return Err("contexto sin destino vacío".into());
        }
        let signature = serde_json::to_string(c).map_err(|e| e.to_string())?;
        if !seen.insert(signature) {
            return Err("afirmación duplicada".into());
        }
        if useful {
            if same
                && subject.is_some()
                && polarity != "unknown"
                && !q["identity_quotes"].as_array().unwrap().is_empty()
            {
                selected.push(i);
            } else {
                unestablished.push(i);
            }
        }
    }
    Ok(
        json!({"selected":!selected.is_empty(),"selected_claim_indices":selected,"unestablished_useful_claim_indices":unestablished,"literal_and_target_links_valid":true,"semantic_alignment_certified":false}),
    )
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
    let w: Value = serde_json::from_str(text).map_err(|_| "JSON de extracción inválido")?;
    let result = inspect(query, memory, &w)?;
    Ok(json!({"witness":w,"check":result}))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn w() -> Value {
        json!({"version":"assertion-projection-v1","query":{"identity_quotes":["Orbe"],"requested_quotes":["peso"],"premise_quotes":["compró"]},"memory_claims":[{"evidence_quote":"Orbe no se compró.","subject_quote":"Orbe","property_quote":"compró","polarity":"negated","negation_scope_quote":"no se compró","relation":"premise_refutation","target_kind":"premise","target_index":0,"same_case":true}]})
    }
    #[test]
    fn literal_refutation_links_to_premise_without_demanding_requested_value() {
        assert_eq!(
            inspect("¿Qué peso compró Orbe?", "Orbe no se compró.", &w()).unwrap()["selected"],
            true
        );
    }
    #[test]
    fn wrong_case_is_not_rescued_by_literal_quote_and_parts_do_not_require_missing_values() {
        let mut x = w();
        x["memory_claims"][0]["same_case"] = json!(false);
        assert_eq!(
            inspect("¿Qué peso compró Orbe?", "Orbe no se compró.", &x).unwrap()["selected"],
            false
        );
        let y = json!({"version":"assertion-projection-v1","query":{"identity_quotes":["Orbe"],"requested_quotes":["peso","longitud"],"premise_quotes":[]},"memory_claims":[{"evidence_quote":"Orbe pesa ocho kilos.","subject_quote":"Orbe","property_quote":"pesa","polarity":"affirmed","negation_scope_quote":null,"relation":"requested_partial","target_kind":"requested","target_index":0,"same_case":true}]});
        assert_eq!(
            inspect(
                "¿Qué peso y longitud tiene Orbe?",
                "Orbe pesa ocho kilos.",
                &y
            )
            .unwrap()["selected"],
            true
        );
    }
    #[test]
    fn invented_quotes_dangling_targets_extra_fields_and_unscoped_negation_are_errors() {
        for kind in 0..4 {
            let mut x = w();
            match kind {
                0 => x["memory_claims"][0]["subject_quote"] = json!("Otro"),
                1 => x["memory_claims"][0]["target_index"] = json!(8),
                2 => x["query"]["extra"] = json!(true),
                _ => x["memory_claims"][0]["negation_scope_quote"] = json!("Orbe"),
            };
            assert!(inspect("¿Qué peso compró Orbe?", "Orbe no se compró.", &x).is_err());
        }
    }
}
