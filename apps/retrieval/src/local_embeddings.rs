//! Explicit offline CPU encoder. Never downloads artifacts or truncates input.
pub mod index;
#[cfg(test)]
mod queue_tests;
#[cfg(test)]
mod tests;

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use gennie_memory_kernel::digest::sha256_hex;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};
use tokenizers::Tokenizer;

pub const RUNTIME: &str = "candle-0.11.0-cpu-f32-tokenizers-0.23.2-mean-l2-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema_version: u32,
    pub model_id: String,
    pub model_revision: String,
    pub model_sha256: String,
    pub config_sha256: String,
    pub tokenizer_sha256: String,
    pub license: String,
    pub runtime: String,
    pub dimensions: usize,
    pub max_tokens: usize,
    pub query_prefix: String,
    pub document_prefix: String,
}
impl Profile {
    pub fn validate(&self) -> Result<(), String> {
        let expected = match self.model_id.as_str() {
            "intfloat/multilingual-e5-small" => (512, "query: ", "passage: ", "mit"),
            "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2" => {
                (128, "", "", "apache-2.0")
            }
            _ => return Err("embedding_profile_unsupported: modelo no implementado".into()),
        };
        if self.schema_version != 1
            || self.runtime != RUNTIME
            || self.dimensions != 384
            || (
                self.max_tokens,
                self.query_prefix.as_str(),
                self.document_prefix.as_str(),
                self.license.as_str(),
            ) != expected
            || self.model_revision.len() != 40
            || !self.model_revision.bytes().all(|b| b.is_ascii_hexdigit())
            || [
                &self.model_sha256,
                &self.config_sha256,
                &self.tokenizer_sha256,
            ]
            .iter()
            .any(|h| h.len() != 64 || !h.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(
                "embedding_profile_invalid: identidad, límites o procedimiento no admitidos".into(),
            );
        }
        Ok(())
    }
    pub fn space_id(&self) -> String {
        format!(
            "local-v1-{}",
            sha256_hex(&serde_json::to_vec(self).expect("profile serializable"))
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub profile: Profile,
    pub artifact_dir: String,
}

pub struct Encoder {
    pub profile: Profile,
    tokenizer: Tokenizer,
    model: BertModel,
}

pub trait TextEncoder {
    fn profile(&self) -> &Profile;
    fn fragments(&self, text: &str) -> Result<Vec<Fragment>, String>;
    fn embed(&self, text: &str, query: bool, until: Instant) -> Result<Vec<f32>, String>;
}
impl TextEncoder for Encoder {
    fn profile(&self) -> &Profile {
        &self.profile
    }
    fn fragments(&self, text: &str) -> Result<Vec<Fragment>, String> {
        Encoder::fragments(self, text)
    }
    fn embed(&self, text: &str, query: bool, until: Instant) -> Result<Vec<f32>, String> {
        Encoder::embed(self, text, query, until)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fragment {
    pub start_byte: usize,
    pub end_byte: usize,
    pub text_sha256: String,
    pub token_count: usize,
}

pub(crate) fn deadline(ms: u64) -> Result<Instant, String> {
    if !(1..=300_000).contains(&ms) {
        return Err("embedding_budget_invalid: deadline_ms entre 1 y 300000".into());
    }
    Ok(Instant::now() + Duration::from_millis(ms))
}
pub(crate) fn check_deadline(until: Instant) -> Result<(), String> {
    if Instant::now() >= until {
        Err("embedding_deadline_exceeded: operación descartada sin resultado parcial".into())
    } else {
        Ok(())
    }
}
fn verified_file(dir: &Path, name: &str, expected: &str) -> Result<Vec<u8>, String> {
    let file = dir.join(name);
    let len = fs::metadata(&file)
        .map_err(|_| format!("embedding_artifact_unavailable: {name}"))?
        .len();
    let max = if name == "model.safetensors" {
        600_000_000
    } else {
        25_000_000
    };
    if len > max {
        return Err(format!("embedding_artifact_too_large: {name}"));
    }
    let bytes = fs::read(file).map_err(|_| format!("embedding_artifact_unavailable: {name}"))?;
    if sha256_hex(&bytes) != expected {
        return Err(format!("embedding_artifact_hash_mismatch: {name}"));
    }
    Ok(bytes)
}
impl Encoder {
    pub fn load(settings: &Settings) -> Result<Self, String> {
        settings.profile.validate()?;
        let dir = Path::new(&settings.artifact_dir);
        let config = verified_file(dir, "config.json", &settings.profile.config_sha256)?;
        let tokenizer = verified_file(dir, "tokenizer.json", &settings.profile.tokenizer_sha256)?;
        let config: Config = serde_json::from_slice(&config).map_err(|e| e.to_string())?;
        if config.hidden_size != settings.profile.dimensions
            || config.max_position_embeddings < settings.profile.max_tokens
        {
            return Err("embedding_config_incompatible".into());
        }
        let mut tokenizer = Tokenizer::from_bytes(tokenizer).map_err(|e| e.to_string())?;
        tokenizer.with_truncation(None).map_err(|e| e.to_string())?;
        tokenizer.with_padding(None);
        let weights = verified_file(dir, "model.safetensors", &settings.profile.model_sha256)?;
        let vb = VarBuilder::from_buffered_safetensors(weights, DType::F32, &Device::Cpu)
            .map_err(|e| e.to_string())?;
        let model = BertModel::load(vb, &config).map_err(|e| e.to_string())?;
        Ok(Self {
            profile: settings.profile.clone(),
            tokenizer,
            model,
        })
    }
    fn encode_ids(&self, text: &str, query: bool) -> Result<Vec<u32>, String> {
        let prefix = if query {
            &self.profile.query_prefix
        } else {
            &self.profile.document_prefix
        };
        let encoding = self
            .tokenizer
            .encode(format!("{prefix}{text}"), true)
            .map_err(|e| e.to_string())?;
        let ids = encoding.get_ids().to_vec();
        if ids.len() > self.profile.max_tokens {
            return Err("embedding_input_too_long: no se permite truncamiento".into());
        }
        Ok(ids)
    }
    pub fn fragments(&self, text: &str) -> Result<Vec<Fragment>, String> {
        fragments(&self.tokenizer, &self.profile, text)
    }
    pub fn embed(&self, text: &str, query: bool, until: Instant) -> Result<Vec<f32>, String> {
        check_deadline(until)?;
        let ids = self.encode_ids(text, query)?;
        let input = Tensor::new(ids.as_slice(), &Device::Cpu)
            .and_then(|t| t.unsqueeze(0))
            .map_err(|e| e.to_string())?;
        let types = input.zeros_like().map_err(|e| e.to_string())?;
        // One sequence, no padding. All real tokens (including specials) enter mean pooling.
        let hidden = self
            .model
            .forward(&input, &types, None)
            .map_err(|e| e.to_string())?;
        let pooled = hidden
            .mean(1)
            .and_then(|t| t.squeeze(0))
            .and_then(|t| t.to_vec1::<f32>())
            .map_err(|e| e.to_string())?;
        let norm = vector_norm(&pooled, self.profile.dimensions)?;
        let result = pooled
            .iter()
            .map(|x| (f64::from(*x) / norm) as f32)
            .collect();
        check_deadline(until)?;
        Ok(result)
    }
}

pub(crate) fn vector_norm(v: &[f32], dimensions: usize) -> Result<f64, String> {
    if v.len() != dimensions || v.iter().any(|x| !x.is_finite()) {
        return Err("embedding_vector_invalid".into());
    }
    let norm = v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
    if !norm.is_finite() || norm <= 0.0 {
        return Err("embedding_vector_zero_norm".into());
    }
    Ok(norm)
}

fn fragments(
    tokenizer: &Tokenizer,
    profile: &Profile,
    text: &str,
) -> Result<Vec<Fragment>, String> {
    if text.trim().is_empty() || text.chars().count() > 16_000 {
        return Err("embedding_text_invalid".into());
    }
    let encoded = tokenizer.encode(text, false).map_err(|e| e.to_string())?;
    let offsets = encoded.get_offsets();
    if offsets.is_empty() {
        return Err("embedding_text_unencodable".into());
    }
    let reserve = tokenizer
        .encode(profile.document_prefix.as_str(), true)
        .map_err(|e| e.to_string())?
        .len()
        + 4;
    let budget = profile
        .max_tokens
        .checked_sub(reserve)
        .filter(|n| *n > 0)
        .ok_or("embedding_token_budget_invalid")?;
    let mut result = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let first = offsets.partition_point(|(_, end)| *end <= start);
        let next = (first + budget).min(offsets.len());
        let mut end = if next == offsets.len() {
            text.len()
        } else {
            offsets[next].0
        };
        while end > start && !text.is_char_boundary(end) {
            end -= 1;
        }
        loop {
            if end <= start {
                return Err("embedding_chunk_no_progress".into());
            }
            let count = tokenizer
                .encode(
                    format!("{}{}", profile.document_prefix, &text[start..end]),
                    true,
                )
                .map_err(|e| e.to_string())?
                .len();
            if count <= profile.max_tokens {
                result.push(Fragment {
                    start_byte: start,
                    end_byte: end,
                    text_sha256: sha256_hex(text[start..end].as_bytes()),
                    token_count: count,
                });
                break;
            }
            end -= 1;
            while end > start && !text.is_char_boundary(end) {
                end -= 1;
            }
        }
        start = end;
        if result.len() > 4096 {
            return Err("embedding_chunk_limit".into());
        }
    }
    Ok(result)
}
