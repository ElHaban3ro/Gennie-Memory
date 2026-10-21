use iromem_retrieval::model::{Candidate, SearchRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    Search,
    Scope,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRequest {
    pub case_id: Option<String>,
    pub search: SearchRequest,
    pub view_mode: ViewMode,
    pub candidates: Vec<ReviewCandidate>,
    pub decisions: Vec<CandidateDecision>,
    pub outcome: ReviewOutcome,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReviewCandidate {
    pub memory_id: String,
    pub revision: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateDecision {
    pub memory_id: String,
    pub revision: i64,
    pub verdict: Verdict,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Match,
    NoMatch,
    Unsure,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewOutcome {
    Matches,
    None,
    Missing,
}

#[derive(Debug, Serialize)]
pub struct ReviewView {
    pub candidates: Vec<Candidate>,
    pub review: Option<SavedReview>,
    pub reusable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedReview {
    pub id: i64,
    pub case_id: Option<String>,
    pub decisions: Vec<CandidateDecision>,
    pub outcome: ReviewOutcome,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct CaseItem {
    pub id: String,
    pub segment: String,
    pub search: SearchRequest,
}

#[derive(Debug, Serialize)]
pub struct ReviewCount {
    pub reviewed: i64,
    pub total: usize,
}
