use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureMemory {
    pub id: String,
    pub scope_kind: String,
    pub scope_id: String,
    #[serde(default)]
    pub entity_id: Option<String>,
    pub revision: i64,
    pub memory_type: String,
    pub original_text: String,
    pub summary: String,
    #[serde(default)]
    pub conditions: Option<String>,
    #[serde(default)]
    pub source: Option<SourceRef>,
    pub state: String,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceRef {
    pub kind: String,
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(default)]
    pub locator: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    pub query: String,
    pub scope_kind: String,
    pub scope_id: String,
    #[serde(default)]
    pub entity_id: Option<String>,
    pub as_of: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Candidate {
    pub memory_id: String,
    pub revision: i64,
    pub summary: String,
    pub conditions: Option<String>,
    pub source: Option<SourceRef>,
    pub state: String,
    pub bm25: f64,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub status: &'static str,
    pub method: &'static str,
    pub candidates: Vec<Candidate>,
    pub semantic_coverage: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedRequest {
    pub search: Option<SearchRequest>,
    pub hybrid: Option<HybridRequest>,
    pub presented: Vec<SelectedCandidate>,
    pub selected: Vec<SelectedCandidate>,
    pub classifier: String,
    pub classification_status: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedCandidate {
    pub memory_id: String,
    pub revision: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeedMemory {
    pub memory_id: String,
    pub revision: i64,
    pub scope_kind: String,
    pub scope_id: String,
    pub summary: String,
    pub conditions: Option<String>,
    pub source: Option<SourceRef>,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct SeedResponse {
    pub status: &'static str,
    pub method: &'static str,
    pub classifier: String,
    pub semantic_coverage: &'static str,
    pub memories: Vec<SeedMemory>,
    pub seed_text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingBatch {
    pub space_id: String,
    pub model_id: String,
    pub model_sha256: String,
    pub tokenizer_sha256: String,
    pub procedure: String,
    pub dimensions: usize,
    pub vectors: Vec<FixtureVector>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureVector {
    pub memory_id: String,
    pub revision: i64,
    pub values: Vec<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HybridRequest {
    pub search: SearchRequest,
    pub space_id: String,
    pub query_vector: Vec<f32>,
}

#[derive(Debug, Serialize)]
pub struct HybridCandidate {
    pub memory_id: String,
    pub revision: i64,
    pub summary: String,
    pub conditions: Option<String>,
    pub source: Option<SourceRef>,
    pub state: String,
    pub text_rank: Option<usize>,
    pub semantic_rank: Option<usize>,
    pub cosine: Option<f64>,
    pub rrf: f64,
}

#[derive(Debug, Serialize)]
pub struct HybridResponse {
    pub status: &'static str,
    pub method: &'static str,
    pub semantic_coverage: &'static str,
    pub eligible_memories: usize,
    pub indexed_memories: usize,
    pub candidates: Vec<HybridCandidate>,
}
