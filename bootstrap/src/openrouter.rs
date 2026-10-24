mod baseline;
#[cfg(test)]
mod deadline_tests;
mod per_memory;
mod slot_evidence;
mod token_budget;
mod write_relations;
use gennie_memory_retrieval::classifier::{Classifier, DecisionError, PartialDecisionUsage};
use gennie_memory_retrieval::model::{
    Candidate, ClassifierProvider, DecisionResult, SelectionPolicy,
};
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const ENDPOINT: &str = "https://openrouter.ai/api/alpha/decisions";
const INSTRUCTIONS: &str = "Select the one saved memory whose stated fact answers the user's specific question, including clear paraphrases. A matching person or project name alone is not enough. Choose none only when no listed fact supplies the requested answer. Do not guess facts absent from the memories.";
const EVIDENCE_V1_INSTRUCTIONS: &str = "Select exactly one saved memory only if its stated fact directly answers the specific question, including clear paraphrases. Compare the requested claim, time, and completion or authorization status with each memory. A proposal, request, plan, test, preliminary estimate, or comparison does not establish a final, approved, completed, or selected outcome. Shared topic, entity, or words are insufficient. If no memory states the requested fact at the requested status, choose none. Do not infer missing outcomes.";
const SUPPORT_GATE_INSTRUCTIONS: &str = "Decide whether saved memory A, by itself, explicitly supplies a truthful answer to the user's specific question. Preserve negation, time, approval, and completion status. If A says the requested outcome did not occur, is pending, or was only proposed or tested, choose none. A related topic alone is insufficient. Choose A only when its stated fact supports the requested answer.";
const UTILITY_V1_INSTRUCTIONS: &str = "Select the one saved memory that provides the most useful explicit evidence for responding to the user's specific question. Useful evidence can directly answer the question, explicitly correct or refute a premise of the question, or supply a concrete part of the requested information even when other requested parts are unknown. Preserve negation, time, conditions, and completion or authorization status: a plan or preliminary result is not a completed outcome. Select corrective or partial evidence for what it actually states, never as proof of a missing outcome. Shared topic, person, project, or words alone are insufficient. Choose none when no listed fact provides an answer, a correction, or a concrete requested part. Do not infer or invent missing facts.";
const UTILITY_V2_INSTRUCTIONS: &str = "Select the one saved memory that provides the most useful explicit evidence for the user's specific question. First check applicability: the stated fact must concern the same subject and context requested, including person, item, edition, site, team, environment, time, and conditions when specified. A value about a different subject or context supplies no requested information, even when its attribute or wording matches. Do not transfer facts or infer equivalence between contexts. An explicit correction or denial of the requested premise is useful evidence about that same subject, not a context mismatch. Once applicability is satisfied, select a direct answer, an explicit premise correction, or a concrete part of the requested information if other requested parts are unknown. Missing parts do not permit borrowing facts from another subject or context. Preserve negation, time, conditions, and completion or authorization status; a plan or preliminary result is not a completed outcome. Shared topic or words alone are insufficient. Choose none if no listed fact passes applicability and supplies an answer, correction, or concrete requested part. Do not infer or invent missing facts.";
const EPISTEMIC_PARTIAL_SINGLE_PASS_V1_INSTRUCTIONS: &str = "Select the one saved memory that is useful for the user's exact requested fact. First require the same specific referent (including any record, order, sample, person, place, or event ID) and the same requested attribute. A different ID or a different attribute is unrelated, even in the same conversation. Then accept an explicit direct answer, an explicit correction of a premise about that same referent, or a concrete provisional/estimated/pending value or status for that same requested attribute as partial evidence. Preserve its limitation: an estimate is not a confirmed value, a forecast is not an observed result, and a diagnosis is not a completed repair. A memory about a different attribute is not evidence that the requested fact is absent. Choose none when no memory provides one of those forms of evidence. Never infer or invent missing facts.";
const UTILITY_V1_NONE: &str = "No listed memory provides an answer, a correction of the question's premise, or a concrete part of the requested information.";
const ROLES_V1_INSTRUCTIONS: &str = "Select one memory using three distinct roles in the question. Identity: identify the particular subject, object, team, place, version, environment or period whose facts are requested. A memory about a different such referent cannot supply those facts. Premise: an assumed property or event can be explicitly denied or corrected by a memory about that same referent; do not require the assumption to be true. Requested information: a concrete answer to any requested part is useful even if other parts remain unknown. Compare identity first, then usefulness. Do not mistake an answer value (a count, date, status or modality) for an identity constraint. Do not treat different named objects or environments as corrections unless the memory explicitly connects them as a correction of this same case. Choose the memory that explicitly answers a requested part or corrects a premise about the same case. Preserve negation, conditions, uncertainty, time and authorization or completion status: proposed, tested, approved, executed and revoked are different facts. A plan is not evidence of execution; an explicit pending or cancelled status can correct a claim of completion. Attribute each fact to its own subject even when a memory mentions several subjects. If the referent remains ambiguous, choose none. Never infer the missing parts. Shared project, topic or vocabulary alone is insufficient. Choose none when all memories are about another referent or none supplies any requested fact or explicit premise correction.";
const RESPONSE_LIMIT: u64 = 1_048_576;
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const ROLES_V2_INSTRUCTIONS: &str = "Choose one memory that explicitly supplies AT LEAST ONE requested fact about the specified referent, or explicitly corrects a premise about that referent. When several facts are requested, one answered part is sufficient: do not choose none merely because the remaining parts are missing. First require explicit association of the fact with the requested subject and context. Never transfer a value from a different named item, person, team, site, environment, edition or period. Matching topic or attribute is not an answer. Distinguish a referent constraint from a premise: an explicit denial of the assumed event or status is useful; an unrelated context is not a correction. A pending result can provide useful status, but a plan, test or prior result cannot establish a completed or final outcome. Keep negation, conditions and unknowns. Prefer the most complete applicable evidence. Choose none only if no option supplies a requested fact or an explicit premise correction.";

struct HttpMeasurement {
    elapsed_ms: f64,
    version: String,
}

#[derive(Clone)]
pub struct OpenRouterClassifier {
    client: Client,
    api_key: Option<String>,
    evaluation_capture: Option<Arc<Mutex<File>>>,
    evaluation_response_content: Option<Arc<Mutex<File>>>,
    operation_deadline: Option<Instant>,
    token_budget: Option<token_budget::ActiveBudget>,
}

impl OpenRouterClassifier {
    pub fn new(api_key: Option<String>) -> Result<Self, String> {
        Self::new_with_evaluation_capture(api_key, None)
    }

    pub fn new_with_evaluation_capture(
        api_key: Option<String>,
        capture_path: Option<&Path>,
    ) -> Result<Self, String> {
        Self::new_with_evaluation_capture_and_content(api_key, capture_path, None)
    }

    pub fn new_with_evaluation_capture_and_content(
        api_key: Option<String>,
        capture_path: Option<&Path>,
        response_content_path: Option<&Path>,
    ) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "no se pudo preparar el cliente de OpenRouter".to_string())?;
        let evaluation_capture = capture_path
            .map(|path| {
                let mut options = OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                options.mode(0o600);
                options
                    .open(path)
                    .map(|file| Arc::new(Mutex::new(file)))
                    .map_err(|_| "no se pudo crear el archivo de captura de evaluación".to_string())
            })
            .transpose()?;
        let evaluation_response_content = response_content_path
            .map(|path| {
                let mut options = OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                options.mode(0o600);
                options
                    .open(path)
                    .map(|file| Arc::new(Mutex::new(file)))
                    .map_err(|_| {
                        "no se pudo crear el archivo de respuestas de evaluación".to_string()
                    })
            })
            .transpose()?;
        Ok(Self {
            client,
            api_key,
            evaluation_capture,
            evaluation_response_content,
            operation_deadline: None,
            token_budget: None,
        })
    }

    fn capture_exchange(
        &self,
        payload: &Value,
        request_bytes: &[u8],
        http_status: Option<u16>,
        http_version: Option<&str>,
        elapsed_ms: f64,
        response_bytes: &[u8],
        response_complete: bool,
        exchange_error: Option<&str>,
    ) -> Result<(), String> {
        if self.evaluation_capture.is_none() && self.evaluation_response_content.is_none() {
            return Ok(());
        }
        let prompt = payload.get("messages").cloned().unwrap_or_else(
            || json!({"state":payload.get("state"),"questions":payload.get("questions")}),
        );
        let schema = payload
            .pointer("/response_format/json_schema/schema")
            .cloned()
            .unwrap_or(Value::Null);
        let response = response_complete
            .then(|| serde_json::from_slice::<Value>(response_bytes).ok())
            .flatten()
            .unwrap_or(Value::Null);
        let choice = response.pointer("/choices/0").unwrap_or(&Value::Null);
        let content_bytes = choice
            .pointer("/message/content")
            .and_then(Value::as_str)
            .map(str::len);
        let scope_id = payload["state"]["scope"]
            .as_str()
            .map(str::to_owned)
            .or_else(|| payload["state"]["scope_id"].as_str().map(str::to_owned))
            .or_else(|| {
                payload["messages"]
                    .as_array()?
                    .iter()
                    .find(|message| message["role"] == "user")?
                    .get("content")?
                    .as_str()
                    .and_then(|content| serde_json::from_str::<Value>(content).ok())
                    .and_then(|content| content["scope_id"].as_str().map(str::to_owned))
            });
        let record = json!({
            "scope_id":scope_id,
            "request_payload_sha256":gennie_memory_kernel::digest::sha256_hex(request_bytes),
            "prompt_sha256":gennie_memory_kernel::digest::sha256_hex(&serde_json::to_vec(&prompt).map_err(|_| "prompt hash serialization failed")?),
            "schema_sha256":gennie_memory_kernel::digest::sha256_hex(&serde_json::to_vec(&schema).map_err(|_| "schema hash serialization failed")?),
            "response_sha256":if response_complete {Some(gennie_memory_kernel::digest::sha256_hex(response_bytes))} else {None},
            "response_body_complete":response_complete,
            "requested_model":payload["model"],
            "served_model":response["model"],
            "served_provider":response["provider"],
            "http_status":http_status,
            "http_version":http_version,
            "elapsed_ms":elapsed_ms,
            "input_tokens":usage_token_count(&response["usage"], "input_tokens", "prompt_tokens"),
            "output_tokens":usage_token_count(&response["usage"], "output_tokens", "completion_tokens"),
            "reported_cost_usd":response["usage"]["cost"],
            "finish_reason":choice.get("finish_reason"),
            "response_content_bytes":content_bytes,
            "exchange_error":exchange_error
        });
        if let Some(capture) = &self.evaluation_capture {
            let mut file = capture
                .lock()
                .map_err(|_| "evaluation capture lock poisoned".to_string())?;
            serde_json::to_writer(&mut *file, &record)
                .map_err(|_| "evaluation capture write failed".to_string())?;
            file.write_all(b"\n")
                .and_then(|_| file.sync_data())
                .map_err(|_| "evaluation capture write failed".to_string())?;
        }
        if let Some(capture) = &self.evaluation_response_content {
            let content = choice
                .pointer("/message/content")
                .and_then(Value::as_str)
                .filter(|content| content.len() <= slot_evidence::MAX_TRACE_BYTES);
            let content_record = json!({
                "request_payload_sha256":gennie_memory_kernel::digest::sha256_hex(request_bytes),
                "response_body_complete":response_complete,
                "response_content":content
            });
            let mut file = capture
                .lock()
                .map_err(|_| "evaluation response capture lock poisoned".to_string())?;
            serde_json::to_writer(&mut *file, &content_record)
                .map_err(|_| "evaluation response capture write failed".to_string())?;
            file.write_all(b"\n")
                .and_then(|_| file.sync_data())
                .map_err(|_| "evaluation response capture write failed".to_string())?;
        }
        Ok(())
    }

    fn choose(
        &self,
        key: &str,
        query: &str,
        scope_id: &str,
        options: serde_json::Map<String, Value>,
        ids: &[String],
        instructions: &str,
        provider: ClassifierProvider,
    ) -> Result<DecisionResult, DecisionError> {
        self.choose_measured(key, query, scope_id, options, ids, instructions, provider)
            .map(|(result, _)| result)
    }

    fn choose_measured(
        &self,
        key: &str,
        query: &str,
        scope_id: &str,
        options: serde_json::Map<String, Value>,
        ids: &[String],
        instructions: &str,
        provider: ClassifierProvider,
    ) -> Result<(DecisionResult, HttpMeasurement), DecisionError> {
        let payload = choice_payload(query, scope_id, options, instructions, provider);
        let (body, transport) = self.request_measured(key, &payload)?;
        Ok((parse(body, ids)?, transport))
    }

    fn request(&self, key: &str, payload: &Value) -> Result<Value, DecisionError> {
        self.request_measured(key, payload).map(|(body, _)| body)
    }

    fn request_measured(
        &self,
        key: &str,
        payload: &Value,
    ) -> Result<(Value, HttpMeasurement), DecisionError> {
        self.request_measured_at(key, payload, ENDPOINT, REQUEST_TIMEOUT)
    }

    fn request_measured_at(
        &self,
        key: &str,
        payload: &Value,
        endpoint: &str,
        timeout: Duration,
    ) -> Result<(Value, HttpMeasurement), DecisionError> {
        let started = Instant::now();
        let timeout = match self.operation_deadline {
            Some(deadline) => deadline
                .checked_duration_since(started)
                .filter(|remaining| !remaining.is_zero())
                .ok_or_else(|| failure("operation_deadline"))?
                .min(timeout),
            None => timeout,
        };
        let request_bytes =
            serde_json::to_vec(payload).map_err(|_| failure("payload_serialization_failed"))?;
        if self.operation_deadline.is_some() && request_bytes.len() > 131_072 {
            return Err(failure("provider_input_limit"));
        }
        if let Some(guard) = &self.token_budget {
            guard.inspect(&request_bytes)?;
        }
        // Tokenization is part of the operation deadline too, including cold initialization.
        let timeout = if let Some(deadline) = self.operation_deadline {
            deadline
                .checked_duration_since(Instant::now())
                .filter(|remaining| !remaining.is_zero())
                .ok_or_else(|| failure("operation_deadline"))?
                .min(timeout)
        } else {
            timeout
        };
        let response = self
            .client
            .post(endpoint)
            // The blocking-client timeout is per read; this also arms the total body deadline.
            .timeout(timeout)
            .bearer_auth(key)
            .header("X-Title", "Gennie Memory local retrieval")
            .header("Content-Type", "application/json")
            .body(request_bytes.clone())
            .send();
        let response = match response {
            Ok(response) => response,
            Err(error) => {
                let code = if self.operation_deadline.is_some() && error.is_timeout() {
                    "request_timeout"
                } else {
                    "transport_error"
                };
                if self
                    .capture_exchange(
                        payload,
                        &request_bytes,
                        None,
                        None,
                        started.elapsed().as_secs_f64() * 1000.0,
                        &[],
                        false,
                        Some(code),
                    )
                    .is_err()
                {
                    return Err(failure("evaluation_capture_write_failed"));
                }
                return Err(failure(code));
            }
        };
        let status = response.status();
        let retry_after_ms = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|seconds| *seconds <= 86_400)
            .and_then(|seconds| seconds.checked_mul(1000));
        let version = format!("{:?}", response.version());
        let mut bytes = Vec::new();
        let read_result = response.take(RESPONSE_LIMIT + 1).read_to_end(&mut bytes);
        let read_error_code = if self.operation_deadline.is_some()
            && read_result.as_ref().err().is_some_and(is_timeout_io)
        {
            "request_timeout"
        } else {
            "transport_error"
        };
        let response_complete = read_result.is_ok() && bytes.len() as u64 <= RESPONSE_LIMIT;
        let exchange_error = if read_result.is_err() {
            Some(read_error_code)
        } else if !status.is_success() {
            Some("http_error")
        } else if !response_complete {
            Some("response_too_large")
        } else {
            None
        };
        if self
            .capture_exchange(
                payload,
                &request_bytes,
                Some(status.as_u16()),
                Some(&version),
                started.elapsed().as_secs_f64() * 1000.0,
                &bytes,
                response_complete,
                exchange_error,
            )
            .is_err()
        {
            return Err(DecisionError {
                diagnostic: None,
                code: "evaluation_capture_write_failed",
                http_status: Some(status.as_u16()),
                partial_usage: response_complete
                    .then(|| serde_json::from_slice::<Value>(&bytes).ok())
                    .flatten()
                    .and_then(|body| known_receipt(&body)),
            });
        }
        if !status.is_success() {
            let mut error = http_failure_with_body(
                status.as_u16(),
                response_complete.then_some(bytes.as_slice()),
            );
            if let Some(diagnostic) = error.diagnostic.as_mut() {
                diagnostic.retry_after_ms = retry_after_ms;
            }
            return Err(error);
        }
        read_result.map_err(|_| failure(read_error_code))?;
        if bytes.len() as u64 > RESPONSE_LIMIT {
            return Err(failure("response_too_large"));
        }
        let body: Value =
            serde_json::from_slice(&bytes).map_err(|_| failure("invalid_response"))?;
        if body.get("error").is_some_and(|error| !error.is_null()) {
            return Err(http_failure_with_body(status.as_u16(), Some(&bytes)));
        }
        Ok((
            body,
            HttpMeasurement {
                elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
                version,
            },
        ))
    }
}

// Preserve only validated receipt fields from a complete, bounded error body.
// HTTP failures remain failures even when answers or a cost are present.
fn http_failure_with_body(status: u16, body: Option<&[u8]>) -> DecisionError {
    let body = body
        .filter(|bytes| bytes.len() as u64 <= RESPONSE_LIMIT)
        .and_then(|bytes| serde_json::from_slice::<Value>(bytes).ok());
    let receipt = body.as_ref().and_then(known_receipt);
    let metadata = body.as_ref().and_then(|b| b.pointer("/error/metadata"));
    let error_type = metadata
        .and_then(|m| m.get("error_type"))
        .and_then(Value::as_str);
    let limit_source = metadata
        .and_then(|m| m.get("limit_source"))
        .and_then(Value::as_str);
    let effective_status = if status == 200 {
        body.as_ref()
            .and_then(|b| b.pointer("/error/code"))
            .and_then(Value::as_u64)
            .and_then(|v| u16::try_from(v).ok())
            .unwrap_or(status)
    } else {
        status
    };
    let diagnostic_code = match (limit_source, error_type, effective_status) {
        (Some("openrouter_in_flight_budget"), _, _) => "provider_in_flight_budget",
        (
            Some(
                "openrouter_guardrail_budget"
                | "openrouter_workspace_budget"
                | "openrouter_key_limit",
            ),
            _,
            _,
        ) => "provider_budget_exceeded",
        (_, Some("context_length_exceeded"), _) => "provider_context_limit",
        (_, Some("rate_limit_exceeded"), _) => "provider_rate_limited",
        (_, Some("content_filter"), _) => "provider_content_rejected",
        (_, _, 402) => "provider_payment_required",
        (_, _, 429) => "provider_rate_limited",
        (_, _, 408 | 504) => "provider_timeout",
        (_, _, 401) => "provider_authentication_rejected",
        (_, _, 403) => "provider_access_rejected",
        (_, _, 500..=599) => "provider_unavailable",
        _ => "provider_failure_unclassified",
    };
    DecisionError {
        diagnostic: Some(gennie_memory_retrieval::classifier::ProviderDiagnostic {
            code: diagnostic_code,
            retry_after_ms: None,
        }),
        code: "http_error",
        http_status: Some(status),
        partial_usage: receipt,
    }
}

fn is_timeout_io(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::TimedOut {
        return true;
    }
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(error) = source {
        if error
            .downcast_ref::<reqwest::Error>()
            .is_some_and(|e| e.is_timeout())
        {
            return true;
        }
        source = error.source();
    }
    false
}

fn failure(code: &'static str) -> DecisionError {
    DecisionError {
        diagnostic: None,
        code,
        http_status: None,
        partial_usage: None,
    }
}

fn consensus_payload(
    query: &str,
    scope_id: &str,
    candidates: &[Candidate],
    provider: ClassifierProvider,
    shared: bool,
) -> (Value, Vec<String>) {
    let limit = if shared { 20 } else { 16 };
    let (mut options, ids) = criteria(&candidates[..candidates.len().min(limit)]);
    options.insert("none".into(), Value::String(UTILITY_V1_NONE.into()));
    let mut state = json!({"user_prompt":query,"scope":scope_id});
    if shared {
        let mut memories = options.clone();
        memories.remove("none");
        for label in memories.keys() {
            options.insert(
                label.clone(),
                json!(format!("Saved memory {label} in state.memories.")),
            );
        }
        state["memories"] = json!(memories);
    }
    let payload = json!({"model":provider.model_id(),"state":state,
                "questions":{
                    "roles_a":{"type":"choice","instructions":ROLES_V1_INSTRUCTIONS,"criteria":options},
                    "roles_b":{"type":"choice","instructions":ROLES_V2_INSTRUCTIONS,"criteria":options}}});
    (payload, ids)
}

fn mirrored_consensus_payload(
    query: &str,
    scope_id: &str,
    candidates: &[Candidate],
    provider: ClassifierProvider,
) -> (Value, Vec<String>) {
    let (mut payload, ids) = consensus_payload(query, scope_id, candidates, provider, false);
    let reversed: Vec<Candidate> = candidates[..candidates.len().min(16)]
        .iter()
        .rev()
        .cloned()
        .collect();
    let (mut options, _) = criteria(&reversed);
    options.insert("none".into(), Value::String(UTILITY_V1_NONE.into()));
    payload["questions"]["roles_b"]["criteria"] = json!(options);
    (payload, ids)
}

fn parse_consensus_batch(body: Value, ids: &[String]) -> Result<DecisionResult, DecisionError> {
    parse_consensus_batch_with_order(body, ids, false)
}

fn parse_consensus_batch_with_order(
    body: Value,
    ids: &[String],
    reverse_second: bool,
) -> Result<DecisionResult, DecisionError> {
    if !body.is_object() {
        return Err(failure("roles_consensus_batch_invalid_response"));
    }
    let known = || PartialDecisionUsage {
        served_model: body["model"]
            .as_str()
            .filter(|s| !s.trim().is_empty() && s.len() <= 128)
            .map(str::to_owned),
        input_tokens: body["usage"]["input_tokens"].as_u64(),
        output_tokens: body["usage"]["output_tokens"].as_u64(),
        reported_cost_usd: body["usage"]["cost"]
            .as_f64()
            .filter(|n| n.is_finite() && *n >= 0.0),
    };
    let second_ids: Vec<String> = if reverse_second {
        ids.iter().rev().cloned().collect()
    } else {
        ids.to_vec()
    };
    let parsed = |name: &str, labels: &[String]| {
        let mut one = body.clone();
        one["answers"] = json!({"relevance":body["answers"][name]});
        parse(one, labels).map_err(|_| DecisionError {
            diagnostic: None,
            code: "roles_consensus_batch_invalid_response",
            http_status: None,
            partial_usage: Some(known()),
        })
    };
    let mut first = parsed("roles_a", ids)?;
    let second = parsed("roles_b", &second_ids)?;
    let trace = gennie_memory_retrieval::model::ConsensusTrace {
        transport: None,
        classified_candidate_count: ids.len(),
        votes: [first.selected_id.clone(), second.selected_id.clone()],
    };
    if first.selected_id != second.selected_id {
        first.selected_id = None;
    }
    // One HTTP response has one global receipt, even though it contains two votes.
    first.consensus = Some(trace);
    Ok(first)
}

fn parse_primary_veto(body: Value, ids: &[String]) -> Result<DecisionResult, DecisionError> {
    let mut result = parse_consensus_batch_with_order(body, ids, true)?;
    result.selected_id = result
        .consensus
        .as_ref()
        .and_then(|trace| SelectionPolicy::RolesPrimaryVetoV1.resolve_votes(&trace.votes))
        .map(str::to_owned);
    Ok(result)
}

fn choice_payload(
    query: &str,
    scope_id: &str,
    options: serde_json::Map<String, Value>,
    instructions: &str,
    provider: ClassifierProvider,
) -> Value {
    json!({"model":provider.model_id(),"state":{"user_prompt":query,"scope":scope_id},"questions":{"relevance":{"type":"choice","instructions":instructions,"criteria":options}}})
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

fn known_receipt(body: &Value) -> Option<PartialDecisionUsage> {
    let receipt = PartialDecisionUsage {
        served_model: body
            .get("model")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty() && s.len() <= 128)
            .map(str::to_owned),
        input_tokens: usage_token_count(&body["usage"], "input_tokens", "prompt_tokens"),
        output_tokens: usage_token_count(&body["usage"], "output_tokens", "completion_tokens"),
        reported_cost_usd: body
            .pointer("/usage/cost")
            .and_then(Value::as_f64)
            .filter(|n| n.is_finite() && *n >= 0.0),
    };
    (receipt.served_model.is_some()
        || receipt.input_tokens.is_some()
        || receipt.output_tokens.is_some()
        || receipt.reported_cost_usd.is_some())
    .then_some(receipt)
}

pub(super) fn usage_token_count(usage: &Value, primary: &str, alternate: &str) -> Option<u64> {
    usage[primary]
        .as_u64()
        .or_else(|| usage[alternate].as_u64())
}

fn merge_known_receipts(
    left: Option<PartialDecisionUsage>,
    right: Option<PartialDecisionUsage>,
) -> Option<PartialDecisionUsage> {
    match (left, right) {
        (None, other) | (other, None) => other,
        (Some(a), Some(b)) => {
            let tokens = |a: Option<u64>, b: Option<u64>| match (a, b) {
                (Some(a), Some(b)) => a.checked_add(b),
                (None, other) | (other, None) => other,
            };
            let cost = match (a.reported_cost_usd, b.reported_cost_usd) {
                (Some(a), Some(b)) => Some(a + b).filter(|n| n.is_finite()),
                (None, other) | (other, None) => other,
            };
            Some(PartialDecisionUsage {
                served_model: b.served_model.or(a.served_model),
                input_tokens: tokens(a.input_tokens, b.input_tokens),
                output_tokens: tokens(a.output_tokens, b.output_tokens),
                reported_cost_usd: cost,
            })
        }
    }
}

fn parse(body: Value, labels: &[String]) -> Result<DecisionResult, DecisionError> {
    parse_choice(&body, labels).map_err(|mut error| {
        error.partial_usage = known_receipt(&body);
        error
    })
}

fn parse_choice(body: &Value, labels: &[String]) -> Result<DecisionResult, DecisionError> {
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
        per_memory: None,
        consensus: None,
        applicability: None,
        slot_evidence: None,
        selected_id,
        served_model,
        input_tokens: tokens("input_tokens")?,
        output_tokens: tokens("output_tokens")?,
        reported_cost_usd,
    })
}

impl Classifier for OpenRouterClassifier {
    fn factor_profile_sha256(&self) -> Option<String> {
        Some(gennie_memory_kernel::digest::sha256_hex(
            include_str!("openrouter/per_memory/factors.rs").as_bytes(),
        ))
    }
    fn classify_relations(
        &self,
        memory: &gennie_memory_retrieval::write::MemoryInput,
        candidates: &[gennie_memory_retrieval::reconciliation::RelationCandidate],
        provider: ClassifierProvider,
    ) -> Result<gennie_memory_retrieval::reconciliation::RelationReport, DecisionError> {
        write_relations::classify(self, memory, candidates, provider)
    }

    fn classify_budgeted_until(
        &self,
        query: &str,
        scope: &str,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        policy: SelectionPolicy,
        until: Instant,
        remaining: usize,
    ) -> gennie_memory_retrieval::classifier::BudgetedDecision {
        self.classify_with_tokens(query, scope, candidates, provider, policy, until, remaining)
    }
    fn suggest_equivalent_budgeted_until(
        &self,
        scope: &str,
        anchor: &Candidate,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        until: Instant,
        remaining: usize,
    ) -> gennie_memory_retrieval::classifier::BudgetedDecision {
        self.equivalent_with_tokens(scope, anchor, candidates, provider, until, remaining)
    }

    fn classify_until(
        &self,
        query: &str,
        scope_id: &str,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        policy: SelectionPolicy,
        deadline: Instant,
    ) -> Result<DecisionResult, DecisionError> {
        let mut scoped = self.clone();
        scoped.operation_deadline = Some(deadline);
        scoped.classify(query, scope_id, candidates, provider, policy)
    }
    fn suggest_equivalent_until(
        &self,
        scope_id: &str,
        anchor: &Candidate,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        deadline: Instant,
    ) -> Result<DecisionResult, DecisionError> {
        let mut scoped = self.clone();
        scoped.operation_deadline = Some(deadline);
        let key = scoped
            .api_key
            .as_ref()
            .filter(|value| !value.trim().is_empty() && !value.chars().any(char::is_whitespace))
            .ok_or_else(|| failure("missing_api_key"))?;
        let alternatives: Vec<Candidate> = candidates
            .iter()
            .filter(|candidate| candidate.memory_id != anchor.memory_id)
            .take(19)
            .cloned()
            .collect();
        if alternatives.is_empty() {
            return Err(failure("equivalence_review_no_candidates"));
        }
        let (mut options, ids) = criteria(&alternatives);
        options.insert(
            "none".into(),
            Value::String("No alternative is clearly the same underlying memory.".into()),
        );
        for (index, candidate) in alternatives.iter().enumerate() {
            let label = char::from(b'A' + index as u8).to_string();
            let text = candidate.conditions.as_ref().map_or_else(
                || candidate.summary.clone(),
                |conditions| format!("{}\nConditions: {}", candidate.summary, conditions),
            );
            options.insert(label, Value::String(text));
        }
        scoped.choose(
            key,
            &json!({"task":"possible_equivalent_memory","anchor":{"id":anchor.memory_id,"revision":anchor.revision,"summary":anchor.summary,"conditions":anchor.conditions},"instruction":"Compare the anchor with each alternative. Choose an alternative only if it expresses the same underlying personal fact or preference with compatible subject, polarity, time, conditions, and status. Similar topic or wording is insufficient. If they may be complementary, contradictory, or distinct, choose none."}).to_string(),
            scope_id,
            options,
            &ids,
            "Choose the one alternative that is most likely the same underlying memory as the anchor. This is a review suggestion, not permission to merge or discard either memory. Choose none unless equivalence is clear.",
            provider,
        )
    }
    fn classify(
        &self,
        query: &str,
        scope_id: &str,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        selection_policy: SelectionPolicy,
    ) -> Result<DecisionResult, DecisionError> {
        let key = self
            .api_key
            .as_ref()
            .filter(|value| !value.trim().is_empty() && !value.chars().any(char::is_whitespace))
            .ok_or_else(|| failure("missing_api_key"))?;
        if selection_policy == SelectionPolicy::SlotEvidenceV1 {
            if provider != ClassifierProvider::Gemma3_4B {
                return Err(failure("slot_evidence_requires_gemma_3_4b"));
            }
            return slot_evidence::classify(self, key, query, scope_id, candidates, provider);
        }
        if selection_policy == SelectionPolicy::SlotEvidenceV2 {
            return Err(failure("slot_evidence_v2_retired"));
        }
        if selection_policy == SelectionPolicy::SlotEvidenceV3 {
            if provider != ClassifierProvider::Gemma3_4B {
                return Err(failure("slot_evidence_requires_gemma_3_4b"));
            }
            return slot_evidence::classify_v3(self, key, query, scope_id, candidates, provider);
        }
        if selection_policy == SelectionPolicy::EpistemicPartialSinglePassV1 {
            let (options, ids) = criteria(candidates);
            return self.choose(
                key,
                query,
                scope_id,
                options,
                &ids,
                EPISTEMIC_PARTIAL_SINGLE_PASS_V1_INSTRUCTIONS,
                provider,
            );
        }
        if provider == ClassifierProvider::Gemma3_4B {
            if selection_policy == SelectionPolicy::Baseline {
                return slot_evidence::baseline(self, key, query, scope_id, candidates, provider);
            }
            return Err(failure("gemma_3_4b_policy_unsupported"));
        }
        if selection_policy == SelectionPolicy::PerMemoryCompletePremiseV1 {
            return per_memory::complete_premise::classify(
                self, key, query, scope_id, candidates, provider,
            );
        }
        if selection_policy == SelectionPolicy::PerMemoryRoleBatchV1 {
            return per_memory::role_batch::classify(
                self, key, query, scope_id, candidates, provider,
            );
        }
        if let Some(version) = selection_policy.role_verification_version() {
            return per_memory::role_verified::classify(
                self, key, query, scope_id, candidates, provider, version,
            );
        }
        if selection_policy.correction_verification_version().is_some() {
            return per_memory::correction_verified::classify(
                self,
                key,
                query,
                scope_id,
                candidates,
                provider,
                selection_policy.correction_verification_version().unwrap(),
            );
        }
        if selection_policy == SelectionPolicy::PerMemoryVerifiedV1 {
            return per_memory::classify_verified(self, key, query, scope_id, candidates, provider);
        }
        if let Some(limit) = selection_policy.per_memory_candidate_limit() {
            let candidates = &candidates[..candidates.len().min(limit)];
            let binary = matches!(
                selection_policy,
                SelectionPolicy::PerMemoryBinaryV1
                    | SelectionPolicy::PerMemoryBinaryV2
                    | SelectionPolicy::PerMemoryBinaryV3
                    | SelectionPolicy::PerMemoryBinaryV4
                    | SelectionPolicy::PerMemoryBinaryV5
                    | SelectionPolicy::PerMemoryExamplesV1
                    | SelectionPolicy::PerMemoryFactorsV1
                    | SelectionPolicy::PerMemoryScopedV1
                    | SelectionPolicy::PerMemoryScopedV2
                    | SelectionPolicy::PerMemoryScopedV3
                    | SelectionPolicy::PerMemoryScopedAnchorV1
                    | SelectionPolicy::PerMemoryScoredV1
            );
            let payload = if selection_policy == SelectionPolicy::PerMemoryScopedAnchorV1 {
                per_memory::scoped_anchor::payload(query, scope_id, candidates, provider)
            } else if selection_policy == SelectionPolicy::PerMemoryScopedV3 {
                per_memory::scoped::flat_payload(query, scope_id, candidates, provider)
            } else if selection_policy == SelectionPolicy::PerMemoryScopedV2 {
                per_memory::scoped::compact_payload(query, scope_id, candidates, provider)
            } else if selection_policy == SelectionPolicy::PerMemoryScopedV1 {
                per_memory::scoped::payload(query, scope_id, candidates, provider)
            } else if selection_policy == SelectionPolicy::PerMemoryFactorsV1 {
                if self.token_budget.is_some() {
                    per_memory::factors::product_payload(query, scope_id, candidates, provider)
                } else {
                    per_memory::factors::payload(query, scope_id, candidates, provider)
                }
            } else if selection_policy == SelectionPolicy::PerMemoryExamplesV1 {
                per_memory::examples_binary_payload(query, scope_id, candidates, provider)
            } else if matches!(
                selection_policy,
                SelectionPolicy::PerMemoryBinaryV5 | SelectionPolicy::PerMemoryScoredV1
            ) {
                per_memory::counterfactual_or_binary_payload(query, scope_id, candidates, provider)
            } else if selection_policy == SelectionPolicy::PerMemoryBinaryV4 {
                per_memory::counterfactual_binary_payload(query, scope_id, candidates, provider)
            } else if selection_policy == SelectionPolicy::PerMemoryBinaryV3 {
                per_memory::explicit_binary_payload(query, scope_id, candidates, provider)
            } else if selection_policy == SelectionPolicy::PerMemoryBinaryV2 {
                per_memory::strict_binary_payload(query, scope_id, candidates, provider)
            } else if binary {
                per_memory::binary_payload(query, scope_id, candidates, provider)
            } else if selection_policy == SelectionPolicy::PerMemoryV4 {
                per_memory::compact_payload(query, scope_id, candidates, provider)
            } else if selection_policy == SelectionPolicy::PerMemoryV3 {
                per_memory::self_contained_payload(query, scope_id, candidates, provider)
            } else {
                per_memory::payload(query, scope_id, candidates, provider)
            };
            let body = self.request(key, &payload).map_err(|mut error| {
                if self.operation_deadline.is_none() {
                    error.code = "per_memory_request_failed";
                }
                error
            })?;
            return if selection_policy == SelectionPolicy::PerMemoryScopedAnchorV1 {
                per_memory::scoped_anchor::parse(body, query, candidates)
            } else if selection_policy == SelectionPolicy::PerMemoryScopedV3 {
                per_memory::scoped::parse_flat(body, candidates)
            } else if matches!(
                selection_policy,
                SelectionPolicy::PerMemoryScopedV1
                    | SelectionPolicy::PerMemoryScopedV2
                    | SelectionPolicy::PerMemoryScopedV3
                    | SelectionPolicy::PerMemoryScopedAnchorV1
            ) {
                per_memory::scoped::parse(body, candidates)
            } else if selection_policy == SelectionPolicy::PerMemoryFactorsV1 {
                per_memory::factors::parse(body, candidates)
            } else if selection_policy == SelectionPolicy::PerMemoryScoredV1 {
                let request_sha256 = gennie_memory_kernel::digest::sha256_hex(
                    &serde_json::to_vec(&payload)
                        .map_err(|_| failure("payload_serialization_failed"))?,
                );
                let mut result = per_memory::parse_scored_binary_response(body, candidates)?;
                result
                    .per_memory
                    .as_mut()
                    .ok_or_else(|| failure("per_memory_invalid_scores"))?
                    .request_sha256 = Some(request_sha256);
                Ok(result)
            } else if binary {
                per_memory::parse_binary_response(body, candidates)
            } else {
                per_memory::parse_response(body, candidates)
            };
        }
        if matches!(
            selection_policy,
            SelectionPolicy::RolesConsensusMirroredV1 | SelectionPolicy::RolesPrimaryVetoV1
        ) {
            let (payload, ids) = mirrored_consensus_payload(query, scope_id, candidates, provider);
            let body = self.request(key, &payload).map_err(|mut error| {
                error.code = "roles_consensus_batch_failed";
                error
            })?;
            return if selection_policy == SelectionPolicy::RolesPrimaryVetoV1 {
                parse_primary_veto(body, &ids)
            } else {
                parse_consensus_batch_with_order(body, &ids, true)
            };
        }
        if matches!(
            selection_policy,
            SelectionPolicy::RolesConsensusBatchV1 | SelectionPolicy::RolesConsensusSharedV1
        ) {
            let (payload, ids) = consensus_payload(
                query,
                scope_id,
                candidates,
                provider,
                selection_policy == SelectionPolicy::RolesConsensusSharedV1,
            );
            let body = self.request(key, &payload).map_err(|mut e| {
                e.code = "roles_consensus_batch_failed";
                e
            })?;
            return parse_consensus_batch(body, &ids);
        }
        if matches!(
            selection_policy,
            SelectionPolicy::RolesConsensusV1 | SelectionPolicy::RolesConsensusCompactV1
        ) {
            let limit = selection_policy
                .consensus_candidate_limit()
                .expect("parallel consensus has a limit");
            let candidates = &candidates[..candidates.len().min(limit)];
            let vote = |instructions: &str| {
                let (mut options, ids) = criteria(candidates);
                options.insert("none".into(), Value::String(UTILITY_V1_NONE.into()));
                self.choose_measured(key, query, scope_id, options, &ids, instructions, provider)
            };
            return std::thread::scope(|scope| {
                let first = scope.spawn(|| vote(ROLES_V1_INSTRUCTIONS));
                let second = scope.spawn(|| vote(ROLES_V2_INSTRUCTIONS));
                let left = first
                    .join()
                    .unwrap_or_else(|_| Err(failure("roles_consensus_thread_failed")));
                let right = second
                    .join()
                    .unwrap_or_else(|_| Err(failure("roles_consensus_thread_failed")));
                let transport = match (&left, &right) {
                    (Ok((_, a)), Ok((_, b))) => {
                        Some(gennie_memory_retrieval::model::ConsensusTransport {
                            http_elapsed_ms: [a.elapsed_ms, b.elapsed_ms],
                            http_versions: [a.version.clone(), b.version.clone()],
                        })
                    }
                    _ => None,
                };
                let mut result = roles_consensus(
                    left.map(|(r, _)| r),
                    right.map(|(r, _)| r),
                    candidates.len(),
                )?;
                if let Some(trace) = result.consensus.as_mut() {
                    trace.transport = transport;
                }
                Ok(result)
            });
        }
        let (mut options, ids) = criteria(candidates);
        if matches!(
            selection_policy,
            SelectionPolicy::UtilityV1
                | SelectionPolicy::UtilityV2
                | SelectionPolicy::WitnessV1
                | SelectionPolicy::RolesV1
                | SelectionPolicy::RolesV2
        ) {
            options.insert("none".into(), Value::String(UTILITY_V1_NONE.into()));
        }
        let instructions = match selection_policy {
            SelectionPolicy::PerMemoryV1
            | SelectionPolicy::PerMemoryV2
            | SelectionPolicy::PerMemoryV3
            | SelectionPolicy::PerMemoryV4
            | SelectionPolicy::PerMemoryBinaryV1
            | SelectionPolicy::PerMemoryBinaryV2
            | SelectionPolicy::PerMemoryBinaryV3
            | SelectionPolicy::PerMemoryBinaryV4
            | SelectionPolicy::PerMemoryBinaryV5
            | SelectionPolicy::PerMemoryExamplesV1
            | SelectionPolicy::PerMemoryFactorsV1
            | SelectionPolicy::PerMemoryScopedV1
            | SelectionPolicy::PerMemoryScopedV2
            | SelectionPolicy::PerMemoryScopedV3
            | SelectionPolicy::PerMemoryScopedAnchorV1
            | SelectionPolicy::PerMemoryScoredV1
            | SelectionPolicy::PerMemoryVerifiedV1
            | SelectionPolicy::PerMemoryCorrectionVerifiedV1
            | SelectionPolicy::PerMemoryCorrectionVerifiedV2
            | SelectionPolicy::PerMemoryCorrectionVerifiedV3
            | SelectionPolicy::PerMemoryCorrectionVerifiedV4
            | SelectionPolicy::PerMemoryCorrectionVerifiedV5
            | SelectionPolicy::PerMemoryRoleVerifiedV1
            | SelectionPolicy::PerMemoryRoleVerifiedV2
            | SelectionPolicy::PerMemoryRoleBatchV1
            | SelectionPolicy::PerMemoryCompletePremiseV1
            | SelectionPolicy::SlotEvidenceV1
            | SelectionPolicy::SlotEvidenceV2
            | SelectionPolicy::SlotEvidenceV3
            | SelectionPolicy::EpistemicPartialSinglePassV1 => {
                unreachable!("handled before single-choice policies")
            }
            SelectionPolicy::Baseline => INSTRUCTIONS,
            SelectionPolicy::RolesFallbackV1 => INSTRUCTIONS,
            SelectionPolicy::EvidenceV1 => EVIDENCE_V1_INSTRUCTIONS,
            SelectionPolicy::SupportGateV1 => INSTRUCTIONS,
            SelectionPolicy::UtilityV1 => UTILITY_V1_INSTRUCTIONS,
            SelectionPolicy::UtilityV2 | SelectionPolicy::WitnessV1 => UTILITY_V2_INSTRUCTIONS,
            SelectionPolicy::RolesV1 => ROLES_V1_INSTRUCTIONS,
            SelectionPolicy::RolesV2 => ROLES_V2_INSTRUCTIONS,
            SelectionPolicy::RolesConsensusV1 | SelectionPolicy::RolesConsensusCompactV1 => {
                unreachable!("handled before one-call policies")
            }
            SelectionPolicy::RolesConsensusBatchV1 => {
                unreachable!("handled before one-call policies")
            }
            SelectionPolicy::RolesConsensusSharedV1
            | SelectionPolicy::RolesConsensusMirroredV1
            | SelectionPolicy::RolesPrimaryVetoV1 => {
                unreachable!("handled before one-call policies")
            }
        };
        let initial = if selection_policy == SelectionPolicy::Baseline {
            let body = self.request(key, &baseline::payload(query, scope_id, options, provider))?;
            parse(body, &ids)?
        } else {
            self.choose(key, query, scope_id, options, &ids, instructions, provider)?
        };
        if selection_policy == SelectionPolicy::RolesFallbackV1 {
            return roles_fallback(initial, || {
                let (mut options, ids) = criteria(candidates);
                options.insert("none".into(), Value::String(UTILITY_V1_NONE.into()));
                self.choose(
                    key,
                    query,
                    scope_id,
                    options,
                    &ids,
                    ROLES_V1_INSTRUCTIONS,
                    provider,
                )
            });
        }
        if selection_policy == SelectionPolicy::WitnessV1 {
            let Some(id) = initial.selected_id.as_ref() else {
                return Ok(initial);
            };
            let candidate = candidates
                .iter()
                .find(|c| &c.memory_id == id)
                .ok_or_else(|| failure("invalid_choice"))?;
            return crate::witness::verify(&self.client, key, query, candidate, initial);
        }
        if selection_policy != SelectionPolicy::SupportGateV1 {
            return Ok(initial);
        }
        let Some(selected_id) = initial.selected_id.as_ref() else {
            return Ok(initial);
        };
        let candidate = candidates
            .iter()
            .find(|candidate| &candidate.memory_id == selected_id)
            .ok_or_else(|| failure("invalid_choice"))?;
        let (single_option, single_id) = criteria(std::slice::from_ref(candidate));
        let checked = self
            .choose(
                key,
                query,
                scope_id,
                single_option,
                &single_id,
                SUPPORT_GATE_INSTRUCTIONS,
                provider,
            )
            .map_err(|error| support_gate_failure(error, &initial))?;
        Ok(combine_verified(initial, checked))
    }
}

fn roles_consensus(
    left: Result<DecisionResult, DecisionError>,
    right: Result<DecisionResult, DecisionError>,
    classified_candidate_count: usize,
) -> Result<DecisionResult, DecisionError> {
    match (left, right) {
        (Ok(first), Ok(mut second)) => {
            let trace = gennie_memory_retrieval::model::ConsensusTrace {
                transport: None,
                classified_candidate_count,
                votes: [first.selected_id.clone(), second.selected_id.clone()],
            };
            if first.selected_id != second.selected_id {
                second.selected_id = None;
            }
            let mut result = combine_verified(first, second);
            result.consensus = Some(trace);
            Ok(result)
        }
        (Ok(known), Err(error)) | (Err(error), Ok(known)) => {
            let mut error = support_gate_failure(error, &known);
            error.code = "roles_consensus_failed";
            Err(error)
        }
        (Err(mut first), Err(second)) => {
            first.partial_usage = merge_known_receipts(first.partial_usage, second.partial_usage);
            first.code = "roles_consensus_failed";
            Err(first)
        }
    }
}

fn roles_fallback(
    initial: DecisionResult,
    fallback: impl FnOnce() -> Result<DecisionResult, DecisionError>,
) -> Result<DecisionResult, DecisionError> {
    if initial.selected_id.is_some() {
        return Ok(initial);
    }
    let checked = fallback().map_err(|error| {
        let mut error = support_gate_failure(error, &initial);
        error.code = "roles_fallback_failed";
        error
    })?;
    Ok(combine_verified(initial, checked))
}

fn support_gate_failure(mut error: DecisionError, initial: &DecisionResult) -> DecisionError {
    error.code = match error.code {
        "transport_error" => "support_gate_transport_error",
        "http_error" => "support_gate_http_error",
        "response_too_large" => "support_gate_response_too_large",
        "invalid_response" => "support_gate_invalid_response",
        "invalid_choice" => "support_gate_invalid_choice",
        _ => "support_gate_failed",
    };
    error.partial_usage = merge_known_receipts(
        Some(PartialDecisionUsage {
            served_model: initial.served_model.clone(),
            input_tokens: initial.input_tokens,
            output_tokens: initial.output_tokens,
            reported_cost_usd: initial.reported_cost_usd,
        }),
        error.partial_usage.take(),
    );
    error
}

fn combine_verified(initial: DecisionResult, checked: DecisionResult) -> DecisionResult {
    DecisionResult {
        per_memory: None,
        consensus: None,
        applicability: None,
        slot_evidence: None,
        selected_id: checked.selected_id,
        served_model: checked.served_model.or(initial.served_model),
        input_tokens: initial
            .input_tokens
            .zip(checked.input_tokens)
            .and_then(|(a, b)| a.checked_add(b)),
        output_tokens: initial
            .output_tokens
            .zip(checked.output_tokens)
            .and_then(|(a, b)| a.checked_add(b)),
        reported_cost_usd: initial
            .reported_cost_usd
            .zip(checked.reported_cost_usd)
            .map(|(a, b)| a + b)
            .filter(|sum| sum.is_finite()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_http_replies_keep_valid_usage_without_inventing_receipts_for_html_or_oversized_bodies()
     {
        let body = json!({"error":{"code":504},"model":"test-model","usage":{"input_tokens":18,"output_tokens":0,"cost":0.003},"answers":{"relevance":{"choice":"A"}}});
        let bytes = serde_json::to_vec(&body).unwrap();
        let error = http_failure_with_body(504, Some(&bytes));
        assert_eq!(error.http_status, Some(504));
        assert_eq!(error.code, "http_error");
        let receipt = error.partial_usage.unwrap();
        assert_eq!(receipt.reported_cost_usd, Some(0.003));
        assert_eq!(receipt.input_tokens, Some(18));
        let zero = serde_json::to_vec(&json!({"usage":{"cost":0.0}})).unwrap();
        assert_eq!(
            http_failure_with_body(503, Some(&zero))
                .partial_usage
                .unwrap()
                .reported_cost_usd,
            Some(0.0)
        );
        assert!(
            http_failure_with_body(504, Some(b"<html>gateway timeout</html>"))
                .partial_usage
                .is_none()
        );
        assert!(http_failure_with_body(504, None).partial_usage.is_none());
        let huge = vec![b' '; RESPONSE_LIMIT as usize + 1];
        assert!(
            http_failure_with_body(504, Some(&huge))
                .partial_usage
                .is_none()
        );
    }

    #[test]
    fn invalid_choice_preserves_valid_receipt_fields() {
        let error = parse(json!({
            "answers":{"relevance":{"type":"choice","choice":"E","probabilities":{"A":0.25,"B":0.25,"C":0.25,"D":0.25,"none":0.0}}},
            "model":"test-model","usage":{"input_tokens":12,"output_tokens":0,"cost":0.002}
        }), &["a".into(),"b".into(),"c".into(),"d".into()]).unwrap_err();
        assert_eq!(error.code, "invalid_choice");
        let usage = error
            .partial_usage
            .expect("invalid choice still has a known receipt");
        assert_eq!(usage.input_tokens, Some(12));
        assert_eq!(usage.reported_cost_usd, Some(0.002));
        let malformed = parse(json!({"usage":{"cost":-1,"input_tokens":"12"}}), &[]).unwrap_err();
        assert!(malformed.partial_usage.is_none());
    }

    #[test]
    fn failed_votes_aggregate_known_receipts_without_inventing_unknowns() {
        let err = |cost: Option<f64>| DecisionError {
            diagnostic: None,
            code: "invalid_choice",
            http_status: None,
            partial_usage: cost.map(|reported_cost_usd| PartialDecisionUsage {
                served_model: Some("test-model".into()),
                input_tokens: Some(10),
                output_tokens: None,
                reported_cost_usd: Some(reported_cost_usd),
            }),
        };
        let both = roles_consensus(Err(err(Some(0.002))), Err(err(Some(0.003))), 4).unwrap_err();
        let usage = both.partial_usage.unwrap();
        assert_eq!(usage.input_tokens, Some(20));
        assert_eq!(usage.reported_cost_usd, Some(0.005));
        assert_eq!(usage.output_tokens, None);
        let right_only = roles_consensus(Err(err(None)), Err(err(Some(0.003))), 4).unwrap_err();
        assert_eq!(
            right_only.partial_usage.unwrap().reported_cost_usd,
            Some(0.003)
        );
        let unknown = roles_consensus(Err(err(None)), Err(err(None)), 4).unwrap_err();
        assert!(unknown.partial_usage.is_none());
        let successful = DecisionResult {
            per_memory: None,
            consensus: None,
            applicability: None,
            slot_evidence: None,
            selected_id: None,
            served_model: Some("test-model".into()),
            input_tokens: Some(20),
            output_tokens: Some(0),
            reported_cost_usd: Some(0.004),
        };
        let partial = support_gate_failure(err(Some(0.002)), &successful)
            .partial_usage
            .unwrap();
        assert_eq!(partial.input_tokens, Some(30));
        assert_eq!(partial.reported_cost_usd, Some(0.006));
    }

    #[test]
    fn primary_veto_is_asymmetric_and_never_turns_invalid_vote_into_abstention() {
        let ids = vec!["m0".into(), "m1".into()];
        let answer = |choice: &str| json!({"type":"choice","choice":choice,"probabilities":{"A":0.3,"B":0.3,"none":0.4}});
        let body = |first: &str, second: &str| json!({"answers":{"roles_a":answer(first),"roles_b":answer(second)},"model":"test","usage":{"input_tokens":20,"output_tokens":0,"cost":0.001}});
        for (a, b, expected) in [
            ("A", "B", Some("m0")),
            ("A", "none", Some("m0")),
            ("A", "A", None),
            ("none", "B", None),
            ("none", "none", None),
        ] {
            let result = parse_primary_veto(body(a, b), &ids).unwrap();
            assert_eq!(result.selected_id.as_deref(), expected);
            assert_eq!(result.reported_cost_usd, Some(0.001));
            assert_eq!(result.input_tokens, Some(20));
        }
        assert!(
            parse_consensus_batch_with_order(body("A", "none"), &ids, true)
                .unwrap()
                .selected_id
                .is_none()
        );
        let invalid = parse_primary_veto(body("A", "C"), &ids).unwrap_err();
        assert_eq!(
            invalid.partial_usage.unwrap().reported_cost_usd,
            Some(0.001)
        );
        let mut missing = body("A", "none");
        missing["answers"]
            .as_object_mut()
            .unwrap()
            .remove("roles_b");
        assert!(parse_primary_veto(missing, &ids).is_err());
    }

    #[test]
    fn mirrored_payload_reverses_only_the_same_presented_prefix() {
        for count in [1, 7, 20] {
            let candidates: Vec<Candidate> = (0..count)
                .map(|i| Candidate {
                    memory_id: format!("m-{i}"),
                    revision: 1,
                    summary: format!("Fact {i}."),
                    conditions: None,
                    source: None,
                    state: "active".into(),
                    bm25: 0.0,
                })
                .collect();
            let (normal, normal_ids) =
                consensus_payload("q", "scope", &candidates, ClassifierProvider::Jev, false);
            let (mirrored, ids) =
                mirrored_consensus_payload("q", "scope", &candidates, ClassifierProvider::Jev);
            assert_eq!(ids, normal_ids);
            let n = count.min(16);
            assert_eq!(ids.len(), n);
            assert_eq!(
                mirrored["questions"]["roles_a"],
                normal["questions"]["roles_a"]
            );
            assert_eq!(
                mirrored["questions"]["roles_b"]["instructions"],
                normal["questions"]["roles_b"]["instructions"]
            );
            assert_eq!(mirrored["state"], normal["state"]);
            for i in 0..n {
                let label = char::from(b'A' + i as u8).to_string();
                assert_eq!(
                    mirrored["questions"]["roles_b"]["criteria"][&label],
                    json!(candidates[n - 1 - i].summary)
                );
            }
            assert_eq!(
                mirrored["questions"]["roles_b"]["criteria"]["none"],
                json!(UTILITY_V1_NONE)
            );
            assert_eq!(
                mirrored["questions"]["roles_b"]["criteria"]
                    .as_object()
                    .unwrap()
                    .len(),
                n + 1
            );
        }
    }

    #[test]
    fn mirrored_consensus_compares_memory_ids_instead_of_labels() {
        let ids: Vec<String> = (0..16).map(|i| format!("m-{i}")).collect();
        let answer = |choice: &str| {
            let mut probabilities = serde_json::Map::new();
            for i in 0..16 {
                probabilities.insert(char::from(b'A' + i as u8).to_string(), json!(1.0 / 17.0));
            }
            probabilities.insert("none".into(), json!(1.0 / 17.0));
            json!({"type":"choice","choice":choice,"probabilities":probabilities})
        };
        let mut body = json!({"answers":{"roles_a":answer("A"),"roles_b":answer("P")},"model":"test","usage":{"input_tokens":100,"output_tokens":0,"cost":0.001}});
        let result = parse_consensus_batch_with_order(body.clone(), &ids, true).unwrap();
        assert_eq!(result.selected_id.as_deref(), Some("m-0"));
        assert_eq!(
            result.consensus.unwrap().votes,
            [Some("m-0".into()), Some("m-0".into())]
        );
        assert_eq!(result.input_tokens, Some(100));
        assert_eq!(result.reported_cost_usd, Some(0.001));
        body["answers"]["roles_b"] = answer("A");
        let mismatch = parse_consensus_batch_with_order(body.clone(), &ids, true).unwrap();
        assert!(mismatch.selected_id.is_none());
        assert_eq!(
            mismatch.consensus.unwrap().votes,
            [Some("m-0".into()), Some("m-15".into())]
        );
        body["answers"]["roles_a"] = answer("none");
        body["answers"]["roles_b"] = answer("none");
        assert!(
            parse_consensus_batch_with_order(body.clone(), &ids, true)
                .unwrap()
                .selected_id
                .is_none()
        );
        body["answers"]["roles_b"] = answer("Q");
        let invalid = parse_consensus_batch_with_order(body.clone(), &ids, true).unwrap_err();
        assert_eq!(
            invalid.partial_usage.unwrap().reported_cost_usd,
            Some(0.001)
        );
        body["answers"].as_object_mut().unwrap().remove("roles_b");
        assert_eq!(
            parse_consensus_batch_with_order(body.clone(), &ids, true)
                .unwrap_err()
                .partial_usage
                .unwrap()
                .input_tokens,
            Some(100)
        );
        for malformed in [json!([]), json!(true), Value::Null] {
            assert!(parse_consensus_batch_with_order(malformed, &ids, true).is_err());
        }
    }

    #[test]
    fn shared_consensus_payload_keeps_all_twenty_memories_once() {
        let candidates: Vec<Candidate> = (0..20)
            .map(|i| Candidate {
                memory_id: format!("m-{i}"),
                revision: 1,
                summary: format!("Unique synthetic fact {i}."),
                conditions: None,
                source: None,
                state: "active".into(),
                bm25: 0.0,
            })
            .collect();
        let (body, ids) = consensus_payload(
            "question",
            "scope",
            &candidates,
            ClassifierProvider::Jev,
            true,
        );
        assert_eq!(ids.len(), 20);
        assert_eq!(body["state"]["memories"].as_object().unwrap().len(), 20);
        assert_eq!(
            body["questions"]["roles_a"]["criteria"],
            body["questions"]["roles_b"]["criteria"]
        );
        let serialized = body.to_string();
        for c in &candidates {
            assert_eq!(serialized.matches(&c.summary).count(), 1);
        }
        let (legacy, old_ids) = consensus_payload(
            "question",
            "scope",
            &candidates,
            ClassifierProvider::Jev,
            false,
        );
        assert_eq!(old_ids.len(), 16);
        assert!(legacy["state"]["memories"].is_null());
    }
    #[test]
    fn batched_consensus_counts_one_receipt_and_rejects_missing_vote() {
        for invalid in [json!([]), json!("bad"), json!(3), json!(false), Value::Null] {
            assert_eq!(
                parse_consensus_batch(invalid, &[]).unwrap_err().code,
                "roles_consensus_batch_invalid_response"
            );
        }
        let answer = json!({"type":"choice","choice":"A","probabilities":{"A":0.8,"none":0.2}});
        let mut body = json!({"answers":{"roles_a":answer,"roles_b":answer},"usage":{"input_tokens":10,"output_tokens":0,"cost":0.001},"model":"test"});
        let ids = vec!["m1".into()];
        let agreed = parse_consensus_batch(body.clone(), &ids).unwrap();
        assert_eq!(agreed.selected_id.as_deref(), Some("m1"));
        assert_eq!(agreed.input_tokens, Some(10));
        assert_eq!(agreed.reported_cost_usd, Some(0.001));
        body["answers"]["roles_b"] =
            json!({"type":"choice","choice":"none","probabilities":{"A":0.2,"none":0.8}});
        assert!(
            parse_consensus_batch(body.clone(), &ids)
                .unwrap()
                .selected_id
                .is_none()
        );
        body["answers"].as_object_mut().unwrap().remove("roles_b");
        let failure = parse_consensus_batch(body, &ids).unwrap_err();
        assert_eq!(failure.code, "roles_consensus_batch_invalid_response");
        assert_eq!(
            failure.partial_usage.unwrap().reported_cost_usd,
            Some(0.001)
        );
    }

    #[test]
    fn consensus_requires_exact_agreement_and_retains_known_receipts_on_failure() {
        let result = |id: Option<&str>| DecisionResult {
            per_memory: None,
            consensus: None,
            applicability: None,
            slot_evidence: None,
            selected_id: id.map(str::to_string),
            served_model: Some("test".into()),
            input_tokens: Some(10),
            output_tokens: Some(0),
            reported_cost_usd: Some(0.001),
        };
        let accepted =
            roles_consensus(Ok(result(Some("same"))), Ok(result(Some("same"))), 16).unwrap();
        assert_eq!(accepted.selected_id.as_deref(), Some("same"));
        assert_eq!(accepted.input_tokens, Some(20));
        assert!(
            roles_consensus(Ok(result(Some("a"))), Ok(result(Some("b"))), 16)
                .unwrap()
                .selected_id
                .is_none()
        );
        assert!(
            roles_consensus(Ok(result(None)), Ok(result(Some("a"))), 16)
                .unwrap()
                .selected_id
                .is_none()
        );
        let failed = roles_consensus(Err(failure("transport_error")), Ok(result(Some("a"))), 16)
            .unwrap_err();
        assert_eq!(failed.code, "roles_consensus_failed");
        assert_eq!(failed.partial_usage.unwrap().reported_cost_usd, Some(0.001));
        assert!(
            roles_consensus(
                Err(failure("transport_error")),
                Err(failure("transport_error")),
                16
            )
            .is_err()
        );
    }
    #[test]
    fn roles_fallback_preserves_first_selection_and_accounts_for_rescue_or_failure() {
        let result = |id: Option<&str>| DecisionResult {
            per_memory: None,
            consensus: None,
            applicability: None,
            slot_evidence: None,
            selected_id: id.map(str::to_string),
            served_model: Some("test-model".into()),
            input_tokens: Some(10),
            output_tokens: Some(0),
            reported_cost_usd: Some(0.001),
        };
        let kept = roles_fallback(result(Some("first")), || {
            panic!("must not call fallback after selection")
        })
        .unwrap();
        assert_eq!(kept.selected_id.as_deref(), Some("first"));
        assert_eq!(kept.input_tokens, Some(10));
        let rescued = roles_fallback(result(None), || Ok(result(Some("second")))).unwrap();
        assert_eq!(rescued.selected_id.as_deref(), Some("second"));
        assert_eq!(rescued.input_tokens, Some(20));
        let error = roles_fallback(result(None), || Err(failure("transport_error"))).unwrap_err();
        assert_eq!(error.code, "roles_fallback_failed");
        assert_eq!(error.partial_usage.unwrap().input_tokens, Some(10));
    }

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

    #[test]
    fn support_gate_vetoes_selection_and_accounts_for_both_calls() {
        let initial = DecisionResult {
            per_memory: None,
            consensus: None,
            applicability: None,
            slot_evidence: None,
            selected_id: Some("memory-1".into()),
            served_model: Some("model-a".into()),
            input_tokens: Some(100),
            output_tokens: Some(2),
            reported_cost_usd: Some(0.001),
        };
        let checked = DecisionResult {
            per_memory: None,
            consensus: None,
            applicability: None,
            slot_evidence: None,
            selected_id: None,
            served_model: Some("model-a".into()),
            input_tokens: Some(80),
            output_tokens: Some(1),
            reported_cost_usd: Some(0.0008),
        };
        let result = combine_verified(initial, checked);
        assert_eq!(result.selected_id, None);
        assert_eq!(result.input_tokens, Some(180));
        assert_eq!(result.output_tokens, Some(3));
        assert!((result.reported_cost_usd.unwrap() - 0.0018).abs() < 1e-10);
    }

    #[test]
    fn failed_support_gate_preserves_only_the_known_first_call_usage() {
        let initial = DecisionResult {
            per_memory: None,
            consensus: None,
            applicability: None,
            slot_evidence: None,
            selected_id: Some("memory-1".into()),
            served_model: Some("model-a".into()),
            input_tokens: Some(100),
            output_tokens: Some(2),
            reported_cost_usd: Some(0.001),
        };
        let error = support_gate_failure(failure("transport_error"), &initial);
        assert_eq!(error.code, "support_gate_transport_error");
        let known = error.partial_usage.unwrap();
        assert_eq!(known.served_model.as_deref(), Some("model-a"));
        assert_eq!(known.input_tokens, Some(100));
        assert_eq!(known.output_tokens, Some(2));
        assert_eq!(known.reported_cost_usd, Some(0.001));

        let http_error = DecisionError {
            diagnostic: None,
            code: "http_error",
            http_status: Some(503),
            partial_usage: None,
        };
        let error = support_gate_failure(http_error, &initial);
        assert_eq!(error.code, "support_gate_http_error");
        assert_eq!(error.http_status, Some(503));
    }
}

#[cfg(test)]
mod product_diagnostic_tests {
    use super::*;
    #[test]
    fn structured_metadata_distinguishes_budget_from_access_without_echoing_raw_text() {
        for (status, metadata, expected) in [
            (
                403,
                json!({"limit_source":"openrouter_key_limit"}),
                "provider_budget_exceeded",
            ),
            (
                402,
                json!({"limit_source":"openrouter_in_flight_budget"}),
                "provider_in_flight_budget",
            ),
            (
                400,
                json!({"error_type":"context_length_exceeded"}),
                "provider_context_limit",
            ),
            (429, json!({}), "provider_rate_limited"),
        ] {
            let body = json!({"error":{"code":status,"message":"sensitive input must not escape","metadata":metadata},"usage":{"input_tokens":12,"cost":0.001}});
            let error = http_failure_with_body(status, Some(body.to_string().as_bytes()));
            let issue = gennie_memory_retrieval::product::Issue::provider(&error);
            let serialized = serde_json::to_string(&issue).unwrap();
            assert_eq!(issue.code, expected);
            assert!(!serialized.contains("sensitive input"));
            assert_eq!(error.partial_usage.unwrap().reported_cost_usd, Some(0.001));
        }
        let error = http_failure_with_body(200, Some(br#"{"error":{"code":429}}"#));
        assert_eq!(error.http_status, Some(200));
        assert_eq!(
            gennie_memory_retrieval::product::Issue::provider(&error).code,
            "provider_rate_limited"
        );
    }
}
