//! Token-index grounding prevents invented quotations; semantic alignment is not certified.
use serde_json::{Value, json};
use std::collections::BTreeSet;
pub fn contract() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/assertion-projection-tokens-v2.json"
    ))
    .unwrap()
}
fn tokens(s: &str) -> Vec<(usize, usize, &str)> {
    let mut cursor = 0;
    s.split_whitespace()
        .map(|word| {
            let start = cursor + s[cursor..].find(word).unwrap();
            cursor = start + word.len();
            (start, cursor, word)
        })
        .collect()
}
pub fn payload(query: &str, memory: &str) -> Value {
    let c = contract();
    let inventory = |s: &str| {
        tokens(s)
            .into_iter()
            .enumerate()
            .map(|(i, (_, _, w))| json!([i, w]))
            .collect::<Vec<_>>()
    };
    json!({"model":c["model"],"temperature":0,"max_tokens":c["max_tokens"],"provider":{"require_parameters":true,"allow_fallbacks":false},"messages":[{"role":"system","content":c["instructions"]},{"role":"user","content":json!({"query":query,"saved_memory":memory,"query_tokens":inventory(query),"memory_tokens":inventory(memory)}).to_string()}],"response_format":{"type":"json_schema","json_schema":{"name":"assertion_projection_tokens","strict":true,"schema":c["schema"]}}})
}
fn keys(v: &Value, expected: &[&str]) -> Result<(), String> {
    let a = v.as_object().ok_or("objeto requerido")?;
    if a.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected.iter().copied().collect() {
        return Err("campos faltantes o ajenos".into());
    }
    Ok(())
}
fn indices(v: &Value, n: usize) -> Result<Vec<usize>, String> {
    let a = v
        .as_array()
        .filter(|a| a.len() <= 256)
        .ok_or("índices inválidos")?;
    let mut out = Vec::new();
    for x in a {
        let i = x
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .filter(|i| *i < n)
            .ok_or("índice fuera de fuente")?;
        if out.last().is_some_and(|p| *p >= i) {
            return Err("índices repetidos o desordenados".into());
        }
        out.push(i);
    }
    Ok(out)
}
fn span(v: &Value, n: usize) -> Result<(usize, usize), String> {
    keys(v, &["first", "last"])?;
    let f = v["first"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or("inicio inválido")?;
    let l = v["last"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .filter(|l| f <= *l && *l < n)
        .ok_or("rango fuera de fuente")?;
    Ok((f, l))
}
pub fn inspect(query: &str, memory: &str, w: &Value) -> Result<Value, String> {
    keys(w, &["version", "query", "memory_claims"])?;
    if w["version"] != "assertion-projection-tokens-v2" {
        return Err("versión inválida".into());
    }
    let qt = tokens(query);
    let mt = tokens(memory);
    let q = &w["query"];
    keys(
        q,
        &[
            "identity_token_indices",
            "requested_token_groups",
            "premise_token_groups",
        ],
    )?;
    let identity = indices(&q["identity_token_indices"], qt.len())?;
    let mut groups = Vec::new();
    let mut query_view = serde_json::Map::new();
    for field in ["requested_token_groups", "premise_token_groups"] {
        let rows = q[field]
            .as_array()
            .filter(|a| a.len() <= 16)
            .ok_or("grupos inválidos")?;
        let mut seen = BTreeSet::new();
        let mut decoded = Vec::new();
        for row in rows {
            let ii = indices(row, qt.len())?;
            if ii.is_empty() || !seen.insert(ii.clone()) {
                return Err("grupo vacío o duplicado".into());
            }
            decoded.push(ii.iter().map(|i| qt[*i].2).collect::<Vec<_>>());
        }
        query_view.insert(field.into(), json!(decoded));
        groups.push(rows.len());
    }
    let claims = w["memory_claims"]
        .as_array()
        .filter(|a| a.len() <= 8)
        .ok_or("afirmaciones inválidas")?;
    let mut selected = Vec::new();
    let mut unestablished = Vec::new();
    let mut decoded = Vec::new();
    let mut seen = BTreeSet::new();
    for (i, c) in claims.iter().enumerate() {
        keys(
            c,
            &[
                "evidence_token_range",
                "subject_token_indices",
                "property_token_indices",
                "polarity",
                "negation_token_range",
                "relation",
                "target_kind",
                "target_index",
                "same_case",
            ],
        )?;
        let (f, l) = span(&c["evidence_token_range"], mt.len())?;
        let subject = indices(&c["subject_token_indices"], mt.len())?;
        let property = indices(&c["property_token_indices"], mt.len())?;
        if property.is_empty() || subject.iter().chain(&property).any(|j| *j < f || *j > l) {
            return Err("sujeto/propiedad fuera de afirmación".into());
        }
        let polarity = c["polarity"]
            .as_str()
            .filter(|s| matches!(*s, "affirmed" | "negated" | "conditional" | "unknown"))
            .ok_or("polaridad inválida")?;
        let negation = if polarity == "negated" {
            let (a, b) = span(&c["negation_token_range"], mt.len())?;
            if a < f || b > l || property.iter().any(|j| *j < a || *j > b) {
                return Err("negación sin predicado vinculado".into());
            }
            Some(&memory[mt[a].0..mt[b].1])
        } else {
            if !c["negation_token_range"].is_null() {
                return Err("negación incoherente".into());
            }
            None
        };
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
            let (kind, count) = if relation == "premise_refutation" {
                ("premise", groups[1])
            } else {
                ("requested", groups[0])
            };
            if c["target_kind"] != kind
                || c["target_index"]
                    .as_u64()
                    .is_none_or(|n| n as usize >= count)
            {
                return Err("destino inválido".into());
            }
        } else if c["target_kind"] != "none" || !c["target_index"].is_null() {
            return Err("destino de contexto inválido".into());
        }
        if !seen.insert(serde_json::to_string(c).map_err(|e| e.to_string())?) {
            return Err("afirmación duplicada".into());
        }
        if useful {
            if same && !identity.is_empty() && !subject.is_empty() && polarity != "unknown" {
                selected.push(i);
            } else {
                unestablished.push(i);
            }
        }
        decoded.push(json!({"evidence_quote":&memory[mt[f].0..mt[l].1],"subject_tokens":subject.iter().map(|j|mt[*j].2).collect::<Vec<_>>(),"property_tokens":property.iter().map(|j|mt[*j].2).collect::<Vec<_>>(),"negation_scope_quote":negation}));
    }
    Ok(
        json!({"selected":!selected.is_empty(),"selected_claim_indices":selected,"unestablished_useful_claim_indices":unestablished,"literal_and_target_links_valid":true,"semantic_alignment_certified":false,"grounded_query":query_view,"grounded_claims":decoded}),
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
    let w: Value = serde_json::from_str(text).map_err(|_| "JSON inválido")?;
    let check = inspect(query, memory, &w)?;
    Ok(json!({"witness":w,"check":check}))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn w() -> Value {
        json!({"version":"assertion-projection-tokens-v2","query":{"identity_token_indices":[5],"requested_token_groups":[[0,1,5],[3,4]],"premise_token_groups":[]},"memory_claims":[{"evidence_token_range":{"first":0,"last":3},"subject_token_indices":[0],"property_token_indices":[1,2,3],"polarity":"affirmed","negation_token_range":null,"relation":"requested_partial","target_kind":"requested","target_index":0,"same_case":true}]})
    }
    #[test]
    fn discontiguous_question_groups_are_grounded_without_invented_quotes() {
        let x = w();
        let out = inspect(
            "¿Qué peso tiene y longitud Orbe?",
            "Orbe pesa ocho kilos.",
            &x,
        )
        .unwrap();
        assert_eq!(out["selected"], true);
        assert_eq!(
            out["grounded_claims"][0]["evidence_quote"],
            "Orbe pesa ocho kilos."
        );
    }
    #[test]
    fn out_of_range_duplicate_and_outside_assertion_references_are_errors() {
        for kind in 0..3 {
            let mut x = w();
            match kind {
                0 => x["query"]["identity_token_indices"] = json!([99]),
                1 => x["memory_claims"][0]["property_token_indices"] = json!([1, 1]),
                _ => x["memory_claims"][0]["evidence_token_range"]["last"] = json!(1),
            };
            assert!(
                inspect(
                    "¿Qué peso tiene y longitud Orbe?",
                    "Orbe pesa ocho kilos.",
                    &x
                )
                .is_err()
            );
        }
    }
    #[test]
    fn literal_tokens_do_not_rescue_another_case() {
        let mut x = w();
        x["memory_claims"][0]["same_case"] = json!(false);
        assert_eq!(
            inspect(
                "¿Qué peso tiene y longitud Orbe?",
                "Orbe pesa ocho kilos.",
                &x
            )
            .unwrap()["selected"],
            false
        );
    }
}
