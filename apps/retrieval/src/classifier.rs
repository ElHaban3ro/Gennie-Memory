use crate::model::{Candidate, ClassifierProvider, DecisionResult};

#[derive(Debug, Clone, Copy)]
pub struct DecisionError {
    pub code: &'static str,
    pub http_status: Option<u16>,
}

pub trait Classifier {
    fn classify(
        &self,
        query: &str,
        scope_id: &str,
        candidates: &[Candidate],
        provider: ClassifierProvider,
    ) -> Result<DecisionResult, DecisionError>;
}
