//! Independent query/source projections and source-bound evidence edges; no semantic certification.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
pub fn spec() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/independent-projection-v1.json"
    ))
    .unwrap()
}
fn object(fields: Value) -> Value {
    let required = fields
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    json!({"type":"object","additionalProperties":false,"required":required,"properties":fields})
}
fn refs(prefix: &str, text: &str) -> Value {
    json!({"type":"array","items":{"type":"string","enum":text.split_whitespace().enumerate().map(|(i,_)|format!("{prefix}{i}")).collect::<Vec<_>>()}})
}
fn chat(instructions: &str, input: Value, schema: Value, name: &str) -> Value {
    let s = spec();
    json!({"model":s["model"],"temperature":0,"max_tokens":s["max_tokens"],"provider":{"require_parameters":true,"allow_fallbacks":false},"messages":[{"role":"system","content":instructions},{"role":"user","content":input.to_string()}],"response_format":{"type":"json_schema","json_schema":{"name":name,"strict":true,"schema":schema}}})
}
fn inventory(prefix: &str, text: &str) -> Vec<Value> {
    text.split_whitespace()
        .enumerate()
        .map(|(i, w)| json!([format!("{prefix}{i}"), w]))
        .collect()
}
pub fn query_payload(query: &str) -> Value {
    let a = refs("q", query);
    let schema = object(
        json!({"identity_tokens":a,"requested_groups":{"type":"array","items":a},"premise_groups":{"type":"array","items":a}}),
    );
    chat(
        spec()["query_instructions"].as_str().unwrap(),
        json!({"query":query,"query_tokens":inventory("q",query)}),
        schema,
        "independent_query",
    )
}
const FIELDS: [&str; 7] = [
    "subjects",
    "predicates",
    "objects",
    "values",
    "context",
    "negators",
    "conditions",
];
pub fn memory_payload(memory: &str) -> Value {
    let a = refs("m", memory);
    let fields = FIELDS
        .iter()
        .map(|k| (k.to_string(), a.clone()))
        .collect::<serde_json::Map<_, _>>();
    let schema = object(json!({"assertions":{"type":"array","items":object(json!(fields))}}));
    chat(
        spec()["memory_instructions"].as_str().unwrap(),
        json!({"saved_memory":memory,"memory_tokens":inventory("m",memory)}),
        schema,
        "independent_memory",
    )
}
fn keys(v: &Value, expected: &[&str]) -> Result<(), String> {
    let a = v.as_object().ok_or("objeto requerido")?;
    if a.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected.iter().copied().collect() {
        return Err("campos faltantes o ajenos".into());
    }
    Ok(())
}
fn decode(v: &Value, prefix: &str, text: &str) -> Result<Vec<String>, String> {
    let words = text.split_whitespace().collect::<Vec<_>>();
    let a = v
        .as_array()
        .filter(|a| a.len() <= 256)
        .ok_or("conjunto inválido")?;
    let mut ii = BTreeSet::new();
    for id in a {
        let s = id.as_str().ok_or("ID no textual")?;
        let n = s
            .strip_prefix(prefix)
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|n| *n < words.len())
            .ok_or("ID de otra fuente o inexistente")?;
        if s != format!("{prefix}{n}") || !ii.insert(n) {
            return Err("ID no canónico o duplicado".into());
        }
    }
    Ok(ii.into_iter().map(|i| words[i].to_string()).collect())
}
pub fn check_query(text: &str, w: &Value) -> Result<Value, String> {
    keys(
        w,
        &["identity_tokens", "requested_groups", "premise_groups"],
    )?;
    let identity = decode(&w["identity_tokens"], "q", text)?;
    let mut out = json!({"identity_tokens":identity});
    for field in ["requested_groups", "premise_groups"] {
        let a = w[field]
            .as_array()
            .filter(|a| a.len() <= 16)
            .ok_or("grupos inválidos")?;
        let mut seen = BTreeSet::new();
        let mut groups = Vec::new();
        for g in a {
            let tokens = decode(g, "q", text)?;
            if tokens.is_empty() || !seen.insert(tokens.clone()) {
                return Err("grupo vacío o duplicado".into());
            }
            groups.push(json!(tokens));
        }
        out[field] = json!(groups);
    }
    Ok(out)
}
pub fn check_memory(text: &str, w: &Value) -> Result<Value, String> {
    keys(w, &["assertions"])?;
    let claims = w["assertions"]
        .as_array()
        .filter(|a| a.len() <= 8)
        .ok_or("afirmaciones inválidas")?;
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for claim in claims {
        keys(claim, &FIELDS)?;
        let mut c = serde_json::Map::new();
        for field in FIELDS {
            c.insert(field.into(), json!(decode(&claim[field], "m", text)?));
        }
        if c["predicates"].as_array().unwrap().is_empty() {
            return Err("predicado vacío".into());
        }
        let c = json!(c);
        if !seen.insert(c.to_string()) {
            return Err("afirmación duplicada".into());
        }
        out.push(c);
    }
    Ok(json!({"assertions":out}))
}
pub fn response_content(body: &Value) -> Result<Value, String> {
    if !spec()["accepted_served_models"]
        .as_array()
        .unwrap()
        .contains(&body["model"])
    {
        return Err("modelo servido inválido".into());
    }
    if body["choices"].as_array().is_none_or(|a| a.len() != 1)
        || body["choices"][0]["finish_reason"] != "stop"
    {
        return Err("respuesta incompleta".into());
    }
    serde_json::from_str(
        body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("contenido ausente")?,
    )
    .map_err(|_| "JSON inválido".into())
}
pub fn check_projection(kind: &str, text: &str, body: &Value) -> Result<Value, String> {
    let raw = response_content(body)?;
    let decoded = match kind {
        "query" => check_query(text, &raw)?,
        "memory" => check_memory(text, &raw)?,
        _ => return Err("etapa desconocida".into()),
    };
    Ok(json!({"raw":raw,"decoded":decoded,"semantic_alignment_certified":false}))
}
pub fn edges(q: &Value, m: &Value) -> Value {
    let mut out = serde_json::Map::new();
    for (kind, field) in [("r", "requested_groups"), ("p", "premise_groups")] {
        for (i, _) in q[field].as_array().unwrap().iter().enumerate() {
            for (j, _) in m["assertions"].as_array().unwrap().iter().enumerate() {
                out.insert(format!("{kind}{i}a{j}"),json!({"kind":if kind=="r"{"requested_contribution"}else{"premise_refutation"},"query_slot":i,"memory_assertion":j}));
            }
        }
    }
    out.insert("context".into(), json!({"kind":"related_context"}));
    out.insert("none".into(), json!({"kind":"unrelated_or_unestablished"}));
    json!(out)
}
pub fn match_payload(query: &str, memory: &str, q: &Value, m: &Value) -> Value {
    let allowed = edges(q, m);
    let choices = allowed
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    let schema = object(json!({"choice":{"type":"string","enum":choices}}));
    chat(
        spec()["match_instructions"].as_str().unwrap(),
        json!({"query":query,"saved_memory":memory,"query_projection":q,"memory_projection":m,"allowed_edges":allowed}),
        schema,
        "independent_match",
    )
}
pub fn check_match(body: &Value, q: &Value, m: &Value) -> Result<Value, String> {
    let c = response_content(body)?;
    keys(&c, &["choice"])?;
    let key = c["choice"].as_str().ok_or("elección ausente")?;
    let e = edges(q, m);
    let edge = e.get(key).ok_or("enlace no permitido")?;
    Ok(
        json!({"selected":key!="context"&&key!="none","choice":key,"edge":edge,"semantic_alignment_certified":false}),
    )
}
/// All original rows retained; projections keyed by question and memory revision, never labels.
pub fn prepare(rows: &[Value]) -> Result<Value, String> {
    let mut qq = BTreeMap::new();
    let mut mm = BTreeMap::new();
    let mut links = Vec::new();
    for r in rows {
        let qid = r["original_case_id"]
            .as_str()
            .ok_or("ID consulta ausente")?;
        let mid = r["memory_id"].as_str().ok_or("ID memoria ausente")?;
        let revision = r["revision"].as_u64().ok_or("revisión ausente")?;
        let query = r["payload"]["state"]["query"]
            .as_str()
            .ok_or("consulta ausente")?;
        let memory = r["payload"]["state"]["saved_memory"]
            .as_str()
            .ok_or("memoria ausente")?;
        let q = json!({"id":format!("query_{qid}"),"kind":"query","source_text":query,"payload":query_payload(query)});
        let mkey = format!("memory_{mid}_r{revision}");
        let m = json!({"id":mkey,"kind":"memory","source_text":memory,"memory_id":mid,"revision":revision,"payload":memory_payload(memory)});
        if qq.get(qid).is_some_and(|old| old != &q) || mm.get(&mkey).is_some_and(|old| old != &m) {
            return Err("misma identidad con contenido distinto".into());
        }
        qq.insert(qid.to_string(), q);
        mm.insert(mkey.clone(), m);
        links.push(json!({"id":r["id"],"query_projection_id":format!("query_{qid}"),"memory_projection_id":mkey}));
    }
    let requests = qq.into_values().chain(mm.into_values()).collect::<Vec<_>>();
    Ok(json!({"projection_requests":requests,"links":links}))
}
pub fn matching_requests(
    rows: &[Value],
    plan: &Value,
    projections: &BTreeMap<String, Value>,
) -> Result<(Vec<Value>, Vec<Value>), String> {
    let mut requests = Vec::new();
    let mut skipped = Vec::new();
    let links = plan["links"].as_array().ok_or("enlaces ausentes")?;
    if links.len() != rows.len() {
        return Err("roster de enlaces incompleto".into());
    }
    for (r, link) in rows.iter().zip(links) {
        if r["id"] != link["id"] {
            return Err("roster alterado".into());
        }
        let q = projections.get(link["query_projection_id"].as_str().unwrap());
        let m = projections.get(link["memory_projection_id"].as_str().unwrap());
        if let (Some(q), Some(m)) = (q, m) {
            let mut rr = r.clone();
            rr["query_projection_id"] = link["query_projection_id"].clone();
            rr["memory_projection_id"] = link["memory_projection_id"].clone();
            rr["payload"] = match_payload(
                r["payload"]["state"]["query"].as_str().unwrap(),
                r["payload"]["state"]["saved_memory"].as_str().unwrap(),
                q,
                m,
            );
            requests.push(rr);
        } else {
            skipped.push(json!({"id":r["id"],"query_projection_id":link["query_projection_id"],"memory_projection_id":link["memory_projection_id"],"reason":"invalid_or_missing_projection"}));
        }
    }
    Ok((requests, skipped))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn separate_payloads_do_not_include_other_side_or_gold() {
        let q = query_payload("QUERY_ONLY");
        let m = memory_payload("MEMORY_ONLY");
        assert!(!q.to_string().contains("MEMORY_ONLY"));
        assert!(!m.to_string().contains("QUERY_ONLY"));
        assert!(!q.to_string().contains("expected_ids"));
    }
    #[test]
    fn negation_condition_and_antecedent_can_coexist_without_clause_range_loss() {
        let m = json!({"assertions":[{"subjects":["m0"],"predicates":["m3"],"objects":[],"values":[],"context":[],"negators":["m1"],"conditions":["m4","m5"]}]});
        let got = check_memory("Orbe no se publica hasta revisar.", &m).unwrap();
        assert_eq!(got["assertions"][0]["negators"], json!(["no"]));
        assert_eq!(
            got["assertions"][0]["conditions"],
            json!(["hasta", "revisar."])
        );
    }
    #[test]
    fn wrong_source_invented_or_duplicate_ids_are_invalid() {
        for a in [
            json!(["m0"]),
            json!(["q9"]),
            json!(["q0", "q0"]),
            json!(["ñ0"]),
        ] {
            assert!(decode(&a, "q", "Orbe").is_err());
        }
    }
    #[test]
    fn requested_part_and_refutation_edges_have_distinct_real_targets() {
        let q = json!({"identity_tokens":["Orbe"],"requested_groups":[["peso"],["longitud"]],"premise_groups":[["compró"]]});
        let m = json!({"assertions":[{}]});
        let e = edges(&q, &m);
        assert_eq!(e["p0a0"]["kind"], "premise_refutation");
        assert_eq!(e["r1a0"]["query_slot"], 1);
        assert!(e.get("r2a0").is_none());
        let response = json!({"model":"openai/gpt-4.1-mini","choices":[{"finish_reason":"stop","message":{"content":"{\"choice\":\"r9a9\"}"}}]});
        assert!(check_match(&response, &q, &m).is_err());
    }
    #[test]
    fn one_question_projection_is_reused_for_multiple_memories() {
        let r = |id: &str, memory: &str| json!({"id":id,"original_case_id":"q","memory_id":id,"revision":1,"payload":{"state":{"query":"¿Peso Orbe?","saved_memory":memory}}});
        let p = prepare(&[r("a", "Orbe pesa ocho."), r("b", "Otro pesa diez.")]).unwrap();
        assert_eq!(p["projection_requests"].as_array().unwrap().len(), 3);
        assert_eq!(
            p["links"][0]["query_projection_id"],
            p["links"][1]["query_projection_id"]
        );
    }
}
