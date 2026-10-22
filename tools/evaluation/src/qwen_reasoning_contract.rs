//! Small-classifier configuration contrast. No answers or generated evidence.
use serde_json::{Value, json};
pub const MAX_TOKENS: u64 = 2048;
pub const MAX_COST: f64 = 0.03;
pub fn payload(base: &Value, enabled: bool) -> Value {
    let mut p = base.clone();
    p["max_tokens"] = json!(MAX_TOKENS);
    p["reasoning"] = json!({"enabled":enabled,"exclude":true});
    p
}
pub fn estimated_cost(p: &Value) -> f64 {
    (p.to_string().len() as f64 + 1024.) * 0.0000001 + MAX_TOKENS as f64 * 0.00000015
}
pub fn decode(
    body: &Value,
    request: &Value,
    memories: &[Value],
    enabled: bool,
) -> Result<Vec<String>, String> {
    let usage = &body["usage"];
    let completion = usage["completion_tokens"]
        .as_u64()
        .ok_or("missing_completion_tokens")?;
    let thinking = usage["completion_tokens_details"]["reasoning_tokens"]
        .as_u64()
        .ok_or("missing_reasoning_tokens")?;
    if body["id"].as_str().is_none_or(|s| s.is_empty())
        || usage["prompt_tokens"].as_u64().is_none()
        || usage["cost"]
            .as_f64()
            .is_none_or(|c| !c.is_finite() || c < 0.)
        || completion > MAX_TOKENS
        || thinking > completion
        || (enabled && thinking == 0)
        || (!enabled && thinking != 0)
        || request["max_tokens"] != MAX_TOKENS
        || request["reasoning"] != json!({"enabled":enabled,"exclude":true})
    {
        return Err("invalid_reasoning_mode_receipt_or_budget".into());
    }
    super::decode_bound(body, request, memories, super::QWEN_MODEL, "DeepInfra")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paired_payloads_only_differ_in_enabled_and_preserve_base_semantics() {
        let base = super::super::payload_qwen(
            "Query",
            &[json!({"id":"local","revision":1,"text":"Source"})],
        );
        let mut off = payload(&base, false);
        let on = payload(&base, true);
        off["reasoning"]["enabled"] = json!(true);
        assert_eq!(off, on);
        off["reasoning"] = base["reasoning"].clone();
        off["max_tokens"] = base["max_tokens"].clone();
        assert_eq!(off, base);
    }
    #[test]
    fn observed_reasoning_and_complete_receipts_are_required_without_changing_roles() {
        let m = vec![json!({"id":"local","revision":1,"text":"Source"})];
        let base = super::super::payload_qwen("Query", &m);
        let on = payload(&base, true);
        let off = payload(&base, false);
        let mut b = json!({"id":"id","model":super::super::QWEN_MODEL,"provider":"DeepInfra","usage":{"prompt_tokens":10,"completion_tokens":20,"completion_tokens_details":{"reasoning_tokens":15},"cost":0.00001},"choices":[{"finish_reason":"stop","message":{"content":"{\"roles\":[\"A\"]}"}}]});
        assert_eq!(decode(&b, &on, &m, true).unwrap(), vec!["A"]);
        assert!(decode(&b, &off, &m, false).is_err());
        b["usage"]["completion_tokens_details"]["reasoning_tokens"] = json!(0);
        assert!(decode(&b, &on, &m, true).is_err());
        assert!(decode(&b, &off, &m, false).is_ok());
        for key in [
            "prompt_tokens",
            "completion_tokens",
            "completion_tokens_details",
            "cost",
        ] {
            let mut bad = b.clone();
            bad["usage"].as_object_mut().unwrap().remove(key);
            assert!(decode(&bad, &off, &m, false).is_err());
        }
        b["choices"][0]["finish_reason"] = json!("length");
        assert!(decode(&b, &off, &m, false).is_err());
    }
}
