use crate::{
    model::{ClassifierProvider, SourceRef},
    write::{MemoryInput, Scope},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WriteCommand {
    pub schema_version: u32,
    pub operation_id: String,
    pub action: String,
    #[serde(default)]
    pub memory_id: Option<String>,
    #[serde(default)]
    pub expected_revision: Option<i64>,
    #[serde(default)]
    pub scope: Option<Scope>,
    #[serde(default)]
    pub entity_id: Option<String>,
    #[serde(default)]
    pub memory: Option<MemoryInput>,
    #[serde(default)]
    pub topic_key: Option<String>,
    #[serde(default)]
    pub evidence: Option<Evidence>,
    #[serde(default)]
    pub source_policy: crate::source_verification::SourcePolicy,
    #[serde(default)]
    pub relation_provider: Option<ClassifierProvider>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "review_patch"
    )]
    pub review_after: Option<Option<String>>,
}
fn review_patch<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(d).map(Some)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub original_text: String,
    pub source: SourceRef,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub memory_id: String,
    pub revision: i64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    Duplicate,
    Complement,
    Correction,
    Exception,
    Contradiction,
    Unrelated,
    Undetermined,
}
impl Relation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Duplicate => "duplicate",
            Self::Complement => "complement",
            Self::Correction => "correction",
            Self::Exception => "exception",
            Self::Contradiction => "contradiction",
            Self::Unrelated => "unrelated",
            Self::Undetermined => "undetermined",
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct ExistingMemory {
    pub original_text: String,
    pub summary: String,
    pub memory_type: String,
    pub source: Option<SourceRef>,
    pub conditions: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
}
impl From<MemoryInput> for ExistingMemory {
    fn from(m: MemoryInput) -> Self {
        Self {
            original_text: m.original_text,
            summary: m.summary,
            memory_type: m.memory_type,
            source: Some(m.source),
            conditions: m.conditions,
            valid_from: m.valid_from,
            valid_until: m.valid_until,
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct RelationCandidate {
    pub target: Target,
    pub memory: ExistingMemory,
    pub topic_key: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RelationJudgment {
    pub target: Target,
    pub relation: Relation,
    pub reason: String,
    pub incoming_quote: String,
    pub existing_quote: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RelationReport {
    pub judgments: Vec<RelationJudgment>,
    pub usage: Option<serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolveCommand {
    pub schema_version: u32,
    pub operation_id: String,
    pub proposal_id: String,
    pub expected_targets: Vec<Target>,
    pub decision: String,
    pub evidence: Evidence,
    #[serde(default)]
    pub target_memory_id: Option<String>,
    #[serde(default)]
    pub scope: Option<Scope>,
    #[serde(default)]
    pub memory: Option<MemoryInput>,
    #[serde(default)]
    pub topic_key: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectRequest {
    pub schema_version: u32,
    pub scope: Scope,
    #[serde(default)]
    pub memory_id: Option<String>,
    #[serde(default)]
    pub proposal_id: Option<String>,
    #[serde(default)]
    pub operation_id: Option<String>,
}
