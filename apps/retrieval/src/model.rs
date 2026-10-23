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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    pub query: String,
    pub scope_kind: String,
    pub scope_id: String,
    #[serde(default)]
    pub entity_id: Option<String>,
    pub as_of: String,
    pub limit: Option<u32>,
    #[serde(default)]
    pub query_mode: Option<QueryMode>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMode {
    Natural,
    Keywords,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClassifierProvider {
    #[default]
    Decisions,
    Jev,
    Jev114,
    #[serde(rename = "gemma3_4b", alias = "gemma3_4_b")]
    Gemma3_4B,
}

impl ClassifierProvider {
    pub fn model_id(self) -> &'static str {
        match self {
            Self::Decisions => "openai/gpt-6-luna-decisions",
            Self::Jev => "typesafe/jev-1.13",
            Self::Jev114 => "typesafe/jev-1.14",
            Self::Gemma3_4B => "google/gemma-3-4b-it",
        }
    }
}

#[cfg(test)]
mod classifier_provider_tests {
    use super::{ClassifierProvider, SelectionPolicy};

    #[test]
    fn gemma_is_explicit_and_does_not_change_the_decisions_default() {
        assert_eq!(ClassifierProvider::default(), ClassifierProvider::Decisions);
        assert_eq!(
            ClassifierProvider::Gemma3_4B.model_id(),
            "google/gemma-3-4b-it"
        );
        assert_eq!(
            serde_json::from_str::<ClassifierProvider>("\"gemma3_4b\"").unwrap(),
            ClassifierProvider::Gemma3_4B
        );
    }

    #[test]
    fn epistemic_partial_single_pass_is_the_default_and_baseline_stays_explicit() {
        assert_eq!(
            SelectionPolicy::default(),
            SelectionPolicy::EpistemicPartialSinglePassV1
        );
        assert_eq!(
            serde_json::from_str::<SelectionPolicy>("\"baseline\"").unwrap(),
            SelectionPolicy::Baseline
        );
        assert_eq!(
            serde_json::from_str::<SelectionPolicy>("\"slot_evidence_v2\"").unwrap(),
            SelectionPolicy::SlotEvidenceV2
        );
        assert_eq!(
            serde_json::from_str::<SelectionPolicy>("\"slot_evidence_v3\"").unwrap(),
            SelectionPolicy::SlotEvidenceV3
        );
        assert_eq!(
            serde_json::from_str::<SelectionPolicy>("\"epistemic_partial_single_pass_v1\"")
                .unwrap(),
            SelectionPolicy::EpistemicPartialSinglePassV1
        );
        assert_eq!(
            serde_json::from_str::<SelectionPolicy>("\"slot_evidence_v2\"").unwrap(),
            SelectionPolicy::SlotEvidenceV2
        );
        assert_eq!(
            serde_json::from_str::<SelectionPolicy>("\"slot_evidence_v3\"").unwrap(),
            SelectionPolicy::SlotEvidenceV3
        );
        assert_eq!(
            serde_json::from_str::<SelectionPolicy>("\"epistemic_partial_single_pass_v1\"")
                .unwrap(),
            SelectionPolicy::EpistemicPartialSinglePassV1
        );
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SelectionPolicy {
    Baseline,
    EvidenceV1,
    SupportGateV1,
    UtilityV1,
    UtilityV2,
    WitnessV1,
    SlotEvidenceV1,
    SlotEvidenceV2,
    SlotEvidenceV3,
    #[default]
    EpistemicPartialSinglePassV1,
    RolesV1,
    RolesFallbackV1,
    RolesV2,
    RolesConsensusV1,
    RolesConsensusBatchV1,
    RolesConsensusSharedV1,
    RolesConsensusCompactV1,
    RolesConsensusMirroredV1,
    RolesPrimaryVetoV1,
    PerMemoryV1,
    PerMemoryV2,
    PerMemoryV3,
    PerMemoryV4,
    PerMemoryBinaryV1,
    PerMemoryBinaryV2,
    PerMemoryBinaryV3,
    PerMemoryBinaryV4,
    PerMemoryBinaryV5,
    PerMemoryExamplesV1,
    PerMemoryFactorsV1,
    PerMemoryScopedV1,
    PerMemoryScopedV2,
    PerMemoryScopedV3,
    PerMemoryScopedAnchorV1,
    PerMemoryScoredV1,
    PerMemoryVerifiedV1,
    PerMemoryCorrectionVerifiedV1,
    PerMemoryCorrectionVerifiedV2,
    PerMemoryCorrectionVerifiedV3,
    PerMemoryCorrectionVerifiedV4,
    PerMemoryCorrectionVerifiedV5,
    PerMemoryRoleVerifiedV1,
    PerMemoryRoleVerifiedV2,
    PerMemoryRoleBatchV1,
    PerMemoryCompletePremiseV1,
}

impl SelectionPolicy {
    pub fn role_verification_version(self) -> Option<u8> {
        match self {
            Self::PerMemoryRoleVerifiedV1 => Some(1),
            Self::PerMemoryRoleVerifiedV2 => Some(2),
            Self::PerMemoryRoleBatchV1 => Some(3),
            _ => None,
        }
    }
    pub fn uses_kind_proposal(self) -> bool {
        self.role_verification_version().is_some() || self == Self::PerMemoryCompletePremiseV1
    }
    pub fn verifier_model(self) -> Option<&'static str> {
        if self == Self::PerMemoryCompletePremiseV1 {
            Some(crate::per_memory::verification::PREMISE_VERIFIER_MODEL)
        } else if self.uses_verification() {
            Some(crate::per_memory::verification::VERIFIER_MODEL)
        } else {
            None
        }
    }
    pub fn uses_verification(self) -> bool {
        self == Self::PerMemoryVerifiedV1
            || self == Self::PerMemoryCompletePremiseV1
            || self.role_verification_version().is_some()
            || self.correction_verification_version().is_some()
    }
    pub fn correction_verification_version(self) -> Option<u8> {
        match self {
            Self::PerMemoryCorrectionVerifiedV1 => Some(1),
            Self::PerMemoryCorrectionVerifiedV2 => Some(2),
            Self::PerMemoryCorrectionVerifiedV3 => Some(3),
            Self::PerMemoryCorrectionVerifiedV4 => Some(4),
            Self::PerMemoryCorrectionVerifiedV5 => Some(5),
            _ => None,
        }
    }

    pub fn per_memory_candidate_limit(self) -> Option<usize> {
        match self {
            Self::PerMemoryV1 => Some(16),
            Self::SlotEvidenceV1 => Some(20),
            Self::SlotEvidenceV2 => Some(20),
            Self::SlotEvidenceV3 => Some(20),
            Self::PerMemoryCorrectionVerifiedV1
            | Self::PerMemoryCorrectionVerifiedV2
            | Self::PerMemoryCorrectionVerifiedV3
            | Self::PerMemoryCorrectionVerifiedV4
            | Self::PerMemoryCorrectionVerifiedV5
            | Self::PerMemoryRoleVerifiedV1
            | Self::PerMemoryRoleVerifiedV2
            | Self::PerMemoryRoleBatchV1
            | Self::PerMemoryCompletePremiseV1
            | Self::PerMemoryScopedAnchorV1 => Some(7),
            Self::PerMemoryV2
            | Self::PerMemoryV3
            | Self::PerMemoryV4
            | Self::PerMemoryBinaryV1
            | Self::PerMemoryBinaryV2
            | Self::PerMemoryBinaryV3
            | Self::PerMemoryBinaryV4
            | Self::PerMemoryBinaryV5
            | Self::PerMemoryExamplesV1
            | Self::PerMemoryFactorsV1
            | Self::PerMemoryScopedV1
            | Self::PerMemoryScopedV2
            | Self::PerMemoryScopedV3
            | Self::PerMemoryScoredV1
            | Self::PerMemoryVerifiedV1 => Some(8),
            _ => None,
        }
    }
    pub fn consensus_candidate_limit(self) -> Option<usize> {
        match self {
            Self::RolesConsensusV1
            | Self::RolesConsensusBatchV1
            | Self::RolesConsensusMirroredV1 => Some(16),
            Self::RolesPrimaryVetoV1 => Some(16),
            Self::RolesConsensusSharedV1 => Some(20),
            Self::RolesConsensusCompactV1 => Some(4),
            _ => None,
        }
    }
    pub fn resolve_votes(self, votes: &[Option<String>; 2]) -> Option<&str> {
        self.consensus_candidate_limit()?;
        if votes[0] == votes[1] || (self == Self::RolesPrimaryVetoV1 && votes[1].is_none()) {
            votes[0].as_deref()
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecallRequest {
    pub search: SearchRequest,
    #[serde(default)]
    pub provider: ClassifierProvider,
    #[serde(default)]
    pub selection_policy: SelectionPolicy,
}

#[derive(Debug)]
pub struct DecisionResult {
    pub per_memory: Option<crate::per_memory::PerMemoryTrace>,
    pub consensus: Option<ConsensusTrace>,
    pub applicability: Option<crate::applicability::ApplicabilityTrace>,
    pub slot_evidence: Option<crate::slot_evidence::SlotEvidenceTrace>,
    pub selected_id: Option<String>,
    pub served_model: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reported_cost_usd: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RecallTimings {
    pub search_ms: f64,
    pub classifier_ms: f64,
    pub composition_ms: f64,
    pub total_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct RecallResponse {
    pub selected_ids: Vec<String>,
    pub per_memory: Option<crate::per_memory::PerMemoryTrace>,
    pub consensus: Option<ConsensusTrace>,
    pub applicability: Option<crate::applicability::ApplicabilityTrace>,
    pub slot_evidence: Option<crate::slot_evidence::SlotEvidenceTrace>,
    pub status: &'static str,
    pub retrieval: SearchResponse,
    pub classification_status: &'static str,
    pub requested_model: &'static str,
    pub served_model: Option<String>,
    pub selected_id: Option<String>,
    pub seed: SeedResponse,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reported_cost_usd: Option<f64>,
    pub error_code: Option<&'static str>,
    pub provider_http_status: Option<u16>,
    pub timings: RecallTimings,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsensusTrace {
    pub transport: Option<ConsensusTransport>,
    pub classified_candidate_count: usize,
    pub votes: [Option<String>; 2],
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsensusTransport {
    pub http_elapsed_ms: [f64; 2],
    pub http_versions: [String; 2],
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
