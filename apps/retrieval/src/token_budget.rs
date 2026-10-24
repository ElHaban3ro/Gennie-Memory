//! Application token accounting. This is not a provider-native billing estimate.
use gennie_memory_kernel::digest::sha256_hex;
use serde::Serialize;
use std::sync::OnceLock;
use tiktoken_rs::CoreBPE;

pub const PROFILE: &str = "o200k_base:tiktoken-rs-0.12.0:ordinary:utf8-json-v1";
pub const DEFAULT_INPUT: usize = 16_384;
pub const DEFAULT_SEED: usize = 2_048;

pub fn count(text: &str) -> Result<usize, String> {
    static BPE: OnceLock<Result<CoreBPE, String>> = OnceLock::new();
    let bpe = BPE.get_or_init(|| tiktoken_rs::o200k_base().map_err(|e| e.to_string()));
    bpe.as_ref()
        .map(|bpe| bpe.encode_ordinary(text).len())
        .map_err(|_| "token_counter_unavailable".into())
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestCount {
    pub stage: &'static str,
    pub request_sha256: String,
    pub json_tokens: usize,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InputBudget {
    pub remaining: usize,
    pub requests: Vec<RequestCount>,
}
impl InputBudget {
    pub fn new(limit: usize) -> Self {
        Self {
            remaining: limit,
            requests: Vec::new(),
        }
    }
    pub fn inspect(&mut self, stage: &'static str, bytes: &[u8]) -> Result<(), &'static str> {
        let text = std::str::from_utf8(bytes).map_err(|_| "token_counter_invalid_utf8")?;
        let tokens = count(text).map_err(|_| "token_counter_unavailable")?;
        let accepted = tokens <= self.remaining;
        self.requests.push(RequestCount {
            stage,
            request_sha256: sha256_hex(bytes),
            json_tokens: tokens,
            accepted,
        });
        if accepted {
            self.remaining -= tokens;
            Ok(())
        } else {
            Err("input_token_limit")
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Accounting {
    pub profile: &'static str,
    pub input_basis: &'static str,
    pub provider_tokenizer: &'static str,
    pub input_limit: usize,
    pub seed_limit: usize,
    pub accepted_input_tokens: usize,
    pub requests: Vec<RequestCount>,
    pub seed_tokens: Option<usize>,
    pub seed_status: &'static str,
}
impl Default for Accounting {
    fn default() -> Self {
        Self {
            profile: PROFILE,
            input_basis: "serialized_request_json",
            provider_tokenizer: "unverified",
            input_limit: DEFAULT_INPUT,
            seed_limit: DEFAULT_SEED,
            accepted_input_tokens: 0,
            requests: Vec::new(),
            seed_tokens: None,
            seed_status: "not_evaluated",
        }
    }
}
impl Accounting {
    pub fn remaining(&self) -> usize {
        self.input_limit.saturating_sub(self.accepted_input_tokens)
    }
    pub fn extend(&mut self, requests: Vec<RequestCount>) {
        self.accepted_input_tokens += requests
            .iter()
            .filter(|r| r.accepted)
            .map(|r| r.json_tokens)
            .sum::<usize>();
        self.requests.extend(requests);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_tokens_preserve_special_markers_unicode_and_exact_limits() {
        assert_eq!(count("hello world").unwrap(), 2);
        assert_eq!(count("").unwrap(), 0);
        assert!(count("<|endoftext|>").unwrap() > 1);
        for text in [
            "Bogotá 日本語 😀",
            "No enviar si llueve.",
            "{\"query\":\"café\"}",
        ] {
            let n = count(text).unwrap();
            let mut budget = InputBudget::new(n);
            budget.inspect("classification", text.as_bytes()).unwrap();
            assert_eq!(budget.remaining, 0);
            assert_eq!(
                budget
                    .inspect("equivalence_review", text.as_bytes())
                    .unwrap_err(),
                "input_token_limit"
            );
            assert_eq!(
                budget.requests[0].request_sha256,
                sha256_hex(text.as_bytes())
            );
            assert!(!budget.requests[1].accepted);
        }
    }
    #[test]
    fn accumulated_budget_never_counts_rejected_requests_as_accepted() {
        let n = count("hello world").unwrap();
        let mut budget = InputBudget::new(n * 2);
        budget.inspect("classification", b"hello world").unwrap();
        budget
            .inspect("equivalence_review", b"hello world")
            .unwrap();
        assert!(
            budget
                .inspect("equivalence_review", b"hello world")
                .is_err()
        );
        let mut ledger = Accounting {
            input_limit: n * 2,
            ..Accounting::default()
        };
        ledger.extend(budget.requests);
        assert_eq!(ledger.accepted_input_tokens, n * 2);
        assert_eq!(ledger.remaining(), 0);
    }
}

pub fn is_local_error(code: &str) -> bool {
    matches!(
        code,
        "input_token_limit"
            | "token_budget_not_supported"
            | "token_counter_unavailable"
            | "token_counter_invalid_utf8"
    )
}
