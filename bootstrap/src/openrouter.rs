use iromem_retrieval::classifier::{Classifier, DecisionError};
use iromem_retrieval::model::{Candidate, ClassifierProvider, DecisionResult};
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::io::Read;
use std::time::Duration;

const ENDPOINT: &str = "https://openrouter.ai/api/alpha/decisions";
const INSTRUCTIONS: &str = "Select the one saved memory whose stated fact answers the user's specific question, including clear paraphrases. A matching person or project name alone is not enough. Choose none only when no listed fact supplies the requested answer. Do not guess facts absent from the memories.";
const RESPONSE_LIMIT: u64 = 1_048_576;

pub struct OpenRouterClassifier {
    client: Client,
    api_key: Option<String>,
}

impl OpenRouterClassifier {
    pub fn new(api_key: Option<String>) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "no se pudo preparar el cliente de OpenRouter".to_string())?;
        Ok(Self { client, api_key })
    }
}

fn failure(code: &'static str) -> DecisionError {
    DecisionError {
        code,
        http_status: None,
    }
}

fn criteria(candidates: &[Candidate]) -> (serde_json::Map<String, Value>, Vec<String>) {
    let mut options = serde_json::Map::new();
    let mut ids = Vec::with_capacity(candidates.len());
    for (index, candidate) in candidates.iter().enumerate() {
        let label = char::from(b'A' + index as u8).to_string();
        options.insert(label, Value::String(candidate.summary.clone()));
        ids.push(candidate.memory_id.clone());
    }
    options.insert(
        "none".into(),
        Value::String("No listed memory contains the requested fact.".into()),
    );
    (options, ids)
}

fn parse(body: Value, labels: &[String]) -> Result<DecisionResult, DecisionError> {
    let answer = body
        .get("answers")
        .and_then(|item| item.get("relevance"))
        .ok_or_else(|| failure("invalid_response"))?;
    if answer.get("type").and_then(Value::as_str) != Some("choice") {
        return Err(failure("invalid_response"));
    }
    let choice = answer
        .get("choice")
        .and_then(Value::as_str)
        .ok_or_else(|| failure("invalid_response"))?;
    let selected_id = if choice == "none" {
        None
    } else {
        let index = choice
            .as_bytes()
            .first()
            .filter(|_| choice.len() == 1)
            .and_then(|letter| letter.checked_sub(b'A'))
            .map(usize::from)
            .ok_or_else(|| failure("invalid_choice"))?;
        Some(
            labels
                .get(index)
                .cloned()
                .ok_or_else(|| failure("invalid_choice"))?,
        )
    };
    let probabilities = answer
        .get("probabilities")
        .and_then(Value::as_object)
        .ok_or_else(|| failure("invalid_response"))?;
    if probabilities.len() != labels.len() + 1 || !probabilities.contains_key("none") {
        return Err(failure("invalid_response"));
    }
    let mut sum = 0.0;
    for index in 0..labels.len() {
        let label = char::from(b'A' + index as u8).to_string();
        let value = probabilities
            .get(&label)
            .and_then(Value::as_f64)
            .ok_or_else(|| failure("invalid_response"))?;
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(failure("invalid_response"));
        }
        sum += value;
    }
    let none = probabilities
        .get("none")
        .and_then(Value::as_f64)
        .ok_or_else(|| failure("invalid_response"))?;
    sum += none;
    if !none.is_finite() || !(0.0..=1.0).contains(&none) || (sum - 1.0).abs() > 0.02 {
        return Err(failure("invalid_response"));
    }
    let usage = body
        .get("usage")
        .and_then(Value::as_object)
        .ok_or_else(|| failure("invalid_response"))?;
    let tokens = |name: &str| -> Result<Option<u64>, DecisionError> {
        match usage.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(value) => value
                .as_u64()
                .map(Some)
                .ok_or_else(|| failure("invalid_response")),
        }
    };
    let reported_cost_usd = match usage.get("cost") {
        None | Some(Value::Null) => None,
        Some(value) => {
            let cost = value.as_f64().ok_or_else(|| failure("invalid_response"))?;
            if !cost.is_finite() || cost < 0.0 {
                return Err(failure("invalid_response"));
            }
            Some(cost)
        }
    };
    let served_model = match body.get("model") {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) if !value.trim().is_empty() && value.len() <= 128 => {
            Some(value.clone())
        }
        _ => return Err(failure("invalid_response")),
    };
    Ok(DecisionResult {
        selected_id,
        served_model,
        input_tokens: tokens("input_tokens")?,
        output_tokens: tokens("output_tokens")?,
        reported_cost_usd,
    })
}

impl Classifier for OpenRouterClassifier {
    fn classify(
        &self,
        query: &str,
        scope_id: &str,
        candidates: &[Candidate],
        provider: ClassifierProvider,
    ) -> Result<DecisionResult, DecisionError> {
        let key = self
            .api_key
            .as_ref()
            .filter(|value| !value.trim().is_empty() && !value.chars().any(char::is_whitespace))
            .ok_or_else(|| failure("missing_api_key"))?;
        let (options, ids) = criteria(candidates);
        let payload = json!({
            "model": provider.model_id(),
            "state": {"user_prompt": query, "scope": scope_id},
            "questions": {"relevance": {"type": "choice", "instructions": INSTRUCTIONS,
                                        "criteria": options}}
        });
        let response = self
            .client
            .post(ENDPOINT)
            .bearer_auth(key)
            .header("X-Title", "iROMEM local retrieval")
            .json(&payload)
            .send()
            .map_err(|_| failure("transport_error"))?;
        if !response.status().is_success() {
            return Err(DecisionError {
                code: "http_error",
                http_status: Some(response.status().as_u16()),
            });
        }
        let mut bytes = Vec::new();
        response
            .take(RESPONSE_LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failure("transport_error"))?;
        if bytes.len() as u64 > RESPONSE_LIMIT {
            return Err(failure("response_too_large"));
        }
        let body = serde_json::from_slice(&bytes).map_err(|_| failure("invalid_response"))?;
        parse(body, &ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_choice_maps_only_known_labels_and_validates_usage() {
        let valid = json!({"answers": {"relevance": {"type": "choice", "choice": "A",
            "probabilities": {"A": 0.8, "none": 0.2}}},
            "usage": {"input_tokens": 10, "output_tokens": 0, "cost": 0.001},
            "model": "openai/gpt-6-luna-decisions"});
        let ids = vec!["memory-1".into()];
        assert_eq!(
            parse(valid.clone(), &ids).unwrap().selected_id.as_deref(),
            Some("memory-1")
        );
        let mut invalid = valid;
        invalid["answers"]["relevance"]["choice"] = json!("B");
        assert_eq!(parse(invalid, &ids).unwrap_err().code, "invalid_choice");
    }
}
