//! Independent JSON composition audit, not a semantic verifier.
use serde_json::Value;
pub(crate) fn inspect(response: &Value) -> Result<(), String> {
    let candidates = response["retrieval"]["candidates"]
        .as_array()
        .ok_or("faltan candidatos")?;
    let items = response["per_memory"]["items"]
        .as_array()
        .ok_or("faltan juicios")?;
    let factors = response["per_memory"]["factor_checks"]
        .as_array()
        .ok_or("faltan factores")?;
    if factors.len() != candidates.len().min(8) || items.len() != factors.len() {
        return Err("factores incompletos".into());
    }
    for ((f, i), c) in factors.iter().zip(items).zip(candidates) {
        if f["memory_id"] != i["memory_id"]
            || f["revision"] != i["revision"]
            || f["memory_id"] != c["memory_id"]
            || f["revision"] != c["revision"]
        {
            return Err("factor ajeno al recuerdo/revisión".into());
        }
        let context = f["same_context"].as_bool().ok_or("contexto inválido")?;
        let fact = f["requested_fact"]
            .as_bool()
            .ok_or("dato solicitado inválido")?;
        let correction = f["premise_correction"]
            .as_bool()
            .ok_or("corrección inválida")?;
        let expected = if context && (fact || correction) {
            "useful_evidence"
        } else {
            "unrelated"
        };
        if i["relation"] != expected {
            return Err("composición no corresponde a factores".into());
        }
    }
    Ok(())
}

pub(crate) fn inspect_scoped(response: &Value) -> Result<(), String> {
    inspect_scoped_mode(None, response)
}
pub(crate) fn inspect_scoped_anchor(query: &str, response: &Value) -> Result<(), String> {
    inspect_scoped_mode(Some(query), response)
}
fn inspect_scoped_mode(query: Option<&str>, response: &Value) -> Result<(), String> {
    let candidates = response["retrieval"]["candidates"]
        .as_array()
        .ok_or("faltan candidatos")?;
    let items = response["per_memory"]["items"]
        .as_array()
        .ok_or("faltan juicios")?;
    let checks = response["per_memory"]["scoped_checks"]
        .as_array()
        .ok_or("faltan vías")?;
    if checks.len() != candidates.len().min(if query.is_some() { 7 } else { 8 })
        || items.len() != checks.len()
    {
        return Err("vías incompletas".into());
    }
    let mut names = Vec::new();
    if let Some(q) = query {
        for (i, t) in q.split_whitespace().enumerate() {
            if i == 0 {
                continue;
            }
            let t = t.trim_matches(|c: char| !c.is_alphanumeric());
            if t.chars().next().is_some_and(char::is_uppercase)
                && !names.iter().any(|n: &String| n == t)
            {
                names.push(t.to_string());
                if names.len() == 16 {
                    break;
                }
            }
        }
    }
    let words = |s: &str| {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase)
            .collect::<std::collections::BTreeSet<_>>()
    };
    let mut global: Option<Value> = None;
    for ((check, item), candidate) in checks.iter().zip(items).zip(candidates) {
        if check["memory_id"] != item["memory_id"]
            || check["revision"] != item["revision"]
            || check["memory_id"] != candidate["memory_id"]
            || check["revision"] != candidate["revision"]
        {
            return Err("vía de otro recuerdo".into());
        }
        let fact = check["requested_fact_same_context"]
            .as_bool()
            .ok_or("dato inválido")?;
        let correction = check["premise_correction_same_context"]
            .as_bool()
            .ok_or("corrección inválida")?;
        let present = if query.is_some() {
            let a = check["literal_anchor"]
                .as_object()
                .filter(|a| {
                    a.len() == 2
                        && a.contains_key("query_token")
                        && a.contains_key("present_in_source")
                })
                .ok_or("falta testigo literal")?;
            let token = &a["query_token"];
            if global.as_ref().is_some_and(|g| g != token) {
                return Err("nombre global inconsistente".into());
            }
            global = Some(token.clone());
            let expected = if token.is_null() {
                None
            } else {
                let t = token.as_str().ok_or("nombre inválido")?;
                if !names.iter().any(|n| n == t) {
                    return Err("nombre fuera de pregunta".into());
                }
                let target = words(t);
                Some(
                    !target.is_empty()
                        && target.is_subset(&words(
                            candidate["summary"].as_str().ok_or("falta resumen")?,
                        )),
                )
            };
            if a["present_in_source"] != serde_json::json!(expected) {
                return Err("presencia de nombre fabricada".into());
            }
            expected != Some(false)
        } else {
            if !check["literal_anchor"].is_null() {
                return Err("testigo en política anterior".into());
            }
            true
        };
        if item["relation"]
            != if (fact || correction) && present {
                "useful_evidence"
            } else {
                "unrelated"
            }
        {
            return Err("composición de vías inválida".into());
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn literal_anchor_rejects_invented_presence_and_requires_null_for_unknown_identity() {
        let mut r = json!({"retrieval":{"candidates":[{"memory_id":"m","revision":1,"summary":"Sara escribió Luciérnaga."}]},"per_memory":{"items":[{"memory_id":"m","revision":1,"relation":"useful_evidence"}],"scoped_checks":[{"memory_id":"m","revision":1,"requested_fact_same_context":false,"premise_correction_same_context":true,"literal_anchor":{"query_token":"Luciérnaga","present_in_source":true}}]}});
        let q = "¿Cuándo escribió Teo Luciérnaga?";
        assert!(inspect_scoped_anchor(q, &r).is_ok());
        assert!(inspect_scoped(&r).is_err());
        r["per_memory"]["scoped_checks"][0]["literal_anchor"]["query_token"] = json!("Teo");
        assert!(inspect_scoped_anchor(q, &r).is_err());
        r["per_memory"]["scoped_checks"][0]["literal_anchor"] =
            json!({"query_token":null,"present_in_source":true});
        assert!(inspect_scoped_anchor(q, &r).is_err());
        r["per_memory"]["scoped_checks"][0]["literal_anchor"]["present_in_source"] = Value::Null;
        assert!(inspect_scoped_anchor(q, &r).is_ok());
    }
    #[test]
    fn scoped_json_checks_do_not_allow_an_unestablished_path_or_changed_revision() {
        let mut r = json!({"retrieval":{"candidates":[{"memory_id":"m","revision":1}]},"per_memory":{"items":[{"memory_id":"m","revision":1,"relation":"useful_evidence"}],"scoped_checks":[{"memory_id":"m","revision":1,"requested_fact_same_context":false,"premise_correction_same_context":false}]}});
        assert!(inspect_scoped(&r).is_err());
        r["per_memory"]["scoped_checks"][0]["premise_correction_same_context"] = json!(true);
        assert!(inspect_scoped(&r).is_ok());
        r["per_memory"]["scoped_checks"][0]["revision"] = json!(2);
        assert!(inspect_scoped(&r).is_err());
    }
    #[test]
    fn factor_json_cannot_forge_a_positive_or_rebind_a_revision() {
        let mut r = json!({"retrieval":{"candidates":[{"memory_id":"m","revision":1}]},"per_memory":{"items":[{"memory_id":"m","revision":1,"relation":"unrelated"}],"factor_checks":[{"memory_id":"m","revision":1,"same_context":true,"requested_fact":false,"premise_correction":false}]}});
        assert!(inspect(&r).is_ok());
        r["per_memory"]["items"][0]["relation"] = json!("useful_evidence");
        assert!(inspect(&r).is_err());
        r["per_memory"]["factor_checks"][0]["requested_fact"] = json!(true);
        assert!(inspect(&r).is_ok());
        r["per_memory"]["factor_checks"][0]["revision"] = json!(2);
        assert!(inspect(&r).is_err());
    }
}
