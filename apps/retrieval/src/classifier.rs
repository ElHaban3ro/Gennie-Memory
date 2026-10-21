use crate::model::{Candidate, ClassifierProvider, DecisionResult, SelectionPolicy};

#[derive(Debug, Clone)]
pub struct PartialDecisionUsage {
    pub served_model: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reported_cost_usd: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct DecisionError {
    pub code: &'static str,
    pub http_status: Option<u16>,
    pub partial_usage: Option<PartialDecisionUsage>,
}

pub trait Classifier {
    fn classify(
        &self,
        query: &str,
        scope_id: &str,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        selection_policy: SelectionPolicy,
    ) -> Result<DecisionResult, DecisionError>;
}
