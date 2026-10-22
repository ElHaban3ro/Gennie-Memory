//! Research-only role output. Schema validity does not establish semantic truth.
use serde_json::{Value, json};
use std::collections::BTreeSet;
#[path = "qwen_reasoning_contract.rs"]
pub mod reasoning;
pub const MODEL: &str = "google/gemini-2.5-flash-lite";
pub const QWEN_MODEL: &str = "qwen/qwen3.5-9b";
pub const GEMMA_MODEL: &str = "google/gemma-3-4b-it";
const PROCEDURE: &str = "First identify the requested subject, attribute and conditions in the question. For each source independently identify what it actually asserts about that attribute and subject. Distinguish a measured result or explicit measurement status from a generic check and from a future appointment. Keep literal intermediate feeding relations as partial contributions without claiming exclusivity. Apply B/A/C/none precedence; do not reject a useful clause merely because accessory details are irrelevant. For ambiguous total-result wording that supplies a real requested part, preserve A without claiming the total. Return only the roles array in source index order. Do not generate quotations, answers, explanations or invented values.";
pub fn payload(query: &str, memories: &[Value]) -> Value {
    let n = memories.len();
    let sources: Vec<_> = memories
        .iter()
        .enumerate()
        .map(|(i, m)| json!({"index":i,"text":m["text"]}))
        .collect();
    json!({"model":MODEL,"temperature":0,"max_tokens":128,"stream":false,"reasoning":{"enabled":false},"provider":{"order":["google-ai-studio"],"only":["google-ai-studio"],"ignore":["google-ai-studio/flex","google-ai-studio/priority"],"allow_fallbacks":false,"require_parameters":true},"messages":[{"role":"system","content":format!("{} {PROCEDURE}",super::RULES)},{"role":"user","content":json!({"question":query,"memories":sources}).to_string()}],"response_format":{"type":"json_schema","json_schema":{"name":"contribution_roles_v2","strict":true,"schema":{"type":"object","additionalProperties":false,"required":["roles"],"properties":{"roles":{"type":"array","minItems":n,"maxItems":n,"items":{"type":"string","enum":["A","B","C","none"]}}}}}}})
}
pub fn payload_gemma(query: &str, memories: &[Value]) -> Value {
    let mut p = payload(query, memories);
    p["model"] = json!(GEMMA_MODEL);
    p["provider"] = json!({"only":["parasail"],"allow_fallbacks":false,"require_parameters":true});
    p.as_object_mut().unwrap().remove("reasoning");
    p
}
pub fn decode_gemma(
    body: &Value,
    request: &Value,
    memories: &[Value],
) -> Result<Vec<String>, String> {
    if body["id"].as_str().is_none_or(|s| s.is_empty())
        || body["usage"]["prompt_tokens"].as_u64().is_none()
        || body["usage"]["completion_tokens"].as_u64().is_none()
        || body["usage"]["cost"]
            .as_f64()
            .is_none_or(|c| !c.is_finite() || c < 0.)
    {
        return Err("incomplete_gemma_receipt".into());
    }
    decode_bound(body, request, memories, GEMMA_MODEL, "Parasail")
}
pub fn payload_qwen(query: &str, memories: &[Value]) -> Value {
    let mut p = payload(query, memories);
    p["model"] = json!(QWEN_MODEL);
    p["provider"] = json!({"only":["deepinfra"],"allow_fallbacks":false,"require_parameters":true});
    p
}
pub fn decode_qwen(
    body: &Value,
    request: &Value,
    memories: &[Value],
) -> Result<Vec<String>, String> {
    if body["id"].as_str().is_none_or(|s| s.is_empty())
        || body["usage"]["prompt_tokens"].as_u64().is_none()
        || body["usage"]["completion_tokens"].as_u64().is_none()
        || body["usage"]["cost"]
            .as_f64()
            .is_none_or(|c| !c.is_finite() || c < 0.)
        || body["usage"]["completion_tokens_details"]["reasoning_tokens"].as_u64() != Some(0)
    {
        return Err("incomplete_qwen_receipt_or_reasoning_not_disabled".into());
    }
    decode_bound(body, request, memories, QWEN_MODEL, "DeepInfra")
}
pub fn payload_examples(query: &str, memories: &[Value]) -> Value {
    let mut p = payload(query, memories);
    let examples = super::payload_local(query, memories)["state"]["contribution_examples"].clone();
    let system = p["messages"][0]["content"].as_str().unwrap();
    p["messages"][0]["content"] = json!(format!(
        "{system} Example triplets [question,source,role] are analogies, never evidence for the current query: {examples}"
    ));
    p
}
pub fn decode(body: &Value, request: &Value, memories: &[Value]) -> Result<Vec<String>, String> {
    decode_bound(body, request, memories, MODEL, "Google AI Studio")
}
fn decode_bound(
    body: &Value,
    request: &Value,
    memories: &[Value],
    model: &str,
    provider: &str,
) -> Result<Vec<String>, String> {
    if memories.is_empty()
        || memories.len() > 20
        || request["model"] != model
        || body["model"] != model
        || body["provider"] != provider
    {
        return Err("unexpected_generative_model_provider_or_count".into());
    }
    let mut seen = BTreeSet::new();
    for m in memories {
        let id = m["id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or("missing_id")?;
        if !seen.insert(id)
            || m["revision"].as_u64().is_none_or(|r| r == 0)
            || m["text"].as_str().is_none_or(|s| s.trim().is_empty())
        {
            return Err("invalid_memory_identity_or_source".into());
        }
    }
    let choices = body["choices"]
        .as_array()
        .filter(|c| c.len() == 1)
        .ok_or("invalid_choice_count")?;
    let choice = &choices[0];
    if choice["finish_reason"] != "stop"
        || choice["message"]
            .get("refusal")
            .is_some_and(|r| !r.is_null())
    {
        return Err("generation_incomplete_or_refused".into());
    }
    let value: Value = serde_json::from_str(
        choice["message"]["content"]
            .as_str()
            .ok_or("content_absent")?,
    )
    .map_err(|_| "invalid_json_content")?;
    let root = value
        .as_object()
        .filter(|v| v.len() == 1 && v.contains_key("roles"))
        .ok_or("unexpected_output_fields")?;
    root["roles"]
        .as_array()
        .filter(|r| r.len() == memories.len())
        .ok_or("incomplete_or_extra_roles")?
        .iter()
        .map(|r| {
            r.as_str()
                .filter(|r| matches!(*r, "A" | "B" | "C" | "none"))
                .map(str::to_owned)
                .ok_or_else(|| "unknown_role".into())
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    fn memories() -> Vec<Value> {
        vec![
            json!({"id":"private","revision":3,"text":"A source"}),
            json!({"id":"second","revision":1,"text":"Another source"}),
        ]
    }
    fn body(value: Value) -> Value {
        json!({"model":MODEL,"provider":"Google AI Studio","choices":[{"finish_reason":"stop","message":{"content":value.to_string()}}]})
    }
    #[test]
    fn strict_roles_remap_original_ids_without_fake_probabilities() {
        let m = memories();
        let p = payload("question", &m);
        let roles = decode(&body(json!({"roles":["C","none"]})), &p, &m).unwrap();
        assert_eq!(
            super::super::selected(&roles, &m).unwrap().get("private"),
            Some(&3)
        );
        assert!(!p.to_string().contains("private"));
        assert!(!p.to_string().contains("probabilities"));
    }
    #[test]
    fn examples_change_only_system_message_and_keep_frozen_triplets() {
        let m = memories();
        let base = payload("question", &m);
        let mut p = payload_examples("question", &m);
        let examples =
            super::super::payload_local("question", &m)["state"]["contribution_examples"].clone();
        assert_eq!(examples.as_array().unwrap().len(), 8);
        assert!(
            p["messages"][0]["content"]
                .as_str()
                .unwrap()
                .ends_with(&examples.to_string())
        );
        p["messages"][0] = base["messages"][0].clone();
        assert_eq!(p, base);
    }
    #[test]
    fn malformed_partial_extra_roles_and_generation_fail_atomically() {
        let m = memories();
        let p = payload("question", &m);
        for v in [
            json!({"roles":["A"]}),
            json!({"roles":["A","none","B"]}),
            json!({"roles":["A","maybe"]}),
            json!({"roles":["A","none"],"quote":"invented"}),
        ] {
            assert!(decode(&body(v), &p, &m).is_err());
        }
        let good = body(json!({"roles":["A","none"]}));
        for (path, v) in [("provider", json!("another")), ("model", json!("another"))] {
            let mut b = good.clone();
            b[path] = v;
            assert!(decode(&b, &p, &m).is_err());
        }
        let mut b = good.clone();
        b["choices"][0]["finish_reason"] = json!("length");
        assert!(decode(&b, &p, &m).is_err());
        b = good;
        b["choices"][0]["message"]["refusal"] = json!("refused");
        assert!(decode(&b, &p, &m).is_err());
    }
}

#[cfg(test)]
mod gemma_tests {
    use super::*;
    #[test]
    fn gemma_changes_only_declared_transport_fields() {
        let m = vec![json!({"id":"local","revision":1,"text":"Original source"})];
        let base = payload("Query", &m);
        let mut candidate = payload_gemma("Query", &m);
        assert!(candidate.get("reasoning").is_none());
        assert_eq!(candidate["provider"]["only"], json!(["parasail"]));
        assert_eq!(candidate["provider"]["allow_fallbacks"], false);
        candidate["model"] = base["model"].clone();
        candidate["provider"] = base["provider"].clone();
        candidate["reasoning"] = base["reasoning"].clone();
        assert_eq!(candidate, base);
    }
    #[test]
    fn gemma_requires_exact_model_provider_and_complete_roles() {
        let m = vec![json!({"id":"local","revision":1,"text":"Original source"})];
        let p = payload_gemma("Query", &m);
        let good = json!({"id":"test-id","usage":{"prompt_tokens":10,"completion_tokens":4,"cost":0.000001},"model":GEMMA_MODEL,"provider":"Parasail","choices":[{"finish_reason":"stop","message":{"content":"{\"roles\":[\"A\"]}"}}]});
        assert_eq!(decode_gemma(&good, &p, &m).unwrap(), vec!["A"]);
        assert!(decode(&good, &p, &m).is_err());
        for key in ["id", "usage"] {
            let mut bad = good.clone();
            bad.as_object_mut().unwrap().remove(key);
            assert!(decode_gemma(&bad, &p, &m).is_err());
        }
        for key in ["prompt_tokens", "completion_tokens", "cost"] {
            let mut bad = good.clone();
            bad["usage"].as_object_mut().unwrap().remove(key);
            assert!(decode_gemma(&bad, &p, &m).is_err());
        }

        for (key, value) in [("model", json!(MODEL)), ("provider", json!("DeepInfra"))] {
            let mut bad = good.clone();
            bad[key] = value;
            assert!(decode_gemma(&bad, &p, &m).is_err());
        }
        for content in [
            r#"{"roles":[]}"#,
            r#"{"roles":["A","none"]}"#,
            r#"{"roles":["A"],"answer":"invented"}"#,
        ] {
            let mut bad = good.clone();
            bad["choices"][0]["message"]["content"] = json!(content);
            assert!(decode_gemma(&bad, &p, &m).is_err());
        }
    }
}

#[cfg(test)]
mod qwen_tests {
    use super::*;
    #[test]
    fn qwen_preserves_semantics_and_disables_reasoning_without_fallback() {
        let m = vec![json!({"id":"local","revision":1,"text":"Source"})];
        let base = payload("Query", &m);
        let mut p = payload_qwen("Query", &m);
        assert_eq!(p["reasoning"], json!({"enabled":false}));
        assert_eq!(
            p["provider"],
            json!({"only":["deepinfra"],"allow_fallbacks":false,"require_parameters":true})
        );
        p["model"] = base["model"].clone();
        p["provider"] = base["provider"].clone();
        assert_eq!(p, base);
    }
    #[test]
    fn qwen_requires_complete_receipt_exact_identity_and_zero_reasoning() {
        let m = vec![json!({"id":"local","revision":1,"text":"Source"})];
        let p = payload_qwen("Query", &m);
        let good = json!({"id":"test","model":QWEN_MODEL,"provider":"DeepInfra","usage":{"prompt_tokens":100,"completion_tokens":4,"cost":0.00001,"completion_tokens_details":{"reasoning_tokens":0}},"choices":[{"finish_reason":"stop","message":{"content":"{\"roles\":[\"A\"]}"}}]});
        assert_eq!(decode_qwen(&good, &p, &m).unwrap(), vec!["A"]);
        for k in ["id", "usage", "provider", "model"] {
            let mut b = good.clone();
            b.as_object_mut().unwrap().remove(k);
            assert!(decode_qwen(&b, &p, &m).is_err());
        }
        for k in [
            "prompt_tokens",
            "completion_tokens",
            "cost",
            "completion_tokens_details",
        ] {
            let mut b = good.clone();
            b["usage"].as_object_mut().unwrap().remove(k);
            assert!(decode_qwen(&b, &p, &m).is_err());
        }
        let mut b = good.clone();
        b["usage"]["completion_tokens_details"]["reasoning_tokens"] = json!(1);
        assert!(decode_qwen(&b, &p, &m).is_err());
        for content in [
            r#"{"roles":[]}"#,
            r#"{"roles":["invented"]}"#,
            r#"{"roles":["A"],"answer":"invented"}"#,
        ] {
            let mut b = good.clone();
            b["choices"][0]["message"]["content"] = json!(content);
            assert!(decode_qwen(&b, &p, &m).is_err());
        }
    }
}
