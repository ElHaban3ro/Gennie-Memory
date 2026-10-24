use crate::model::{Candidate, ClassifierProvider, DecisionResult, SelectionPolicy};

#[derive(Debug, Clone)]
pub struct PartialDecisionUsage {
    pub served_model: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reported_cost_usd: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ProviderDiagnostic {
    pub code: &'static str,
    pub retry_after_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DecisionError {
    pub diagnostic: Option<ProviderDiagnostic>,
    pub code: &'static str,
    pub http_status: Option<u16>,
    pub partial_usage: Option<PartialDecisionUsage>,
}

pub struct BudgetedDecision {
    pub result: Result<DecisionResult, DecisionError>,
    pub requests: Vec<crate::token_budget::RequestCount>,
}
impl BudgetedDecision {
    pub fn unsupported() -> Self {
        Self {
            result: Err(DecisionError {
                diagnostic: None,
                code: "token_budget_not_supported",
                http_status: None,
                partial_usage: None,
            }),
            requests: Vec::new(),
        }
    }
}

pub trait Classifier {
    /// Write judgments never authorize overwriting. The default stays uncertain.
    fn classify_relations(
        &self,
        _memory: &crate::write::MemoryInput,
        _candidates: &[crate::reconciliation::RelationCandidate],
        _provider: ClassifierProvider,
    ) -> Result<crate::reconciliation::RelationReport, DecisionError> {
        Err(DecisionError {
            diagnostic: None,
            code: "relation_classifier_unavailable",
            http_status: None,
            partial_usage: None,
        })
    }
    /// Product v1 requires accounting on the exact payload before transport.
    /// Adapters must not delegate to an unbounded implementation by default.
    fn classify_budgeted_until(
        &self,
        _query: &str,
        _scope_id: &str,
        _candidates: &[Candidate],
        _provider: ClassifierProvider,
        _policy: SelectionPolicy,
        _deadline: std::time::Instant,
        _remaining_tokens: usize,
    ) -> BudgetedDecision {
        BudgetedDecision::unsupported()
    }

    fn suggest_equivalent_budgeted_until(
        &self,
        _scope_id: &str,
        _anchor: &Candidate,
        _candidates: &[Candidate],
        _provider: ClassifierProvider,
        _deadline: std::time::Instant,
        _remaining_tokens: usize,
    ) -> BudgetedDecision {
        BudgetedDecision::unsupported()
    }

    /// Adapters must honor the deadline at transport level. No fallback to an
    /// unbounded implementation is allowed by the versioned product boundary.
    fn classify_until(
        &self,
        _query: &str,
        _scope_id: &str,
        _candidates: &[Candidate],
        _provider: ClassifierProvider,
        _selection_policy: SelectionPolicy,
        _deadline: std::time::Instant,
    ) -> Result<DecisionResult, DecisionError> {
        Err(DecisionError {
            diagnostic: None,
            code: "deadline_not_supported",
            http_status: None,
            partial_usage: None,
        })
    }

    fn classify(
        &self,
        query: &str,
        scope_id: &str,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        selection_policy: SelectionPolicy,
    ) -> Result<DecisionResult, DecisionError>;

    /// Optional bounded second pass that proposes at most one likely equivalent
    /// memory for the selected anchor. It never mutates stored memories.
    fn suggest_equivalent_until(
        &self,
        _scope_id: &str,
        _anchor: &Candidate,
        _candidates: &[Candidate],
        _provider: ClassifierProvider,
        _deadline: std::time::Instant,
    ) -> Result<DecisionResult, DecisionError> {
        Err(DecisionError {
            diagnostic: None,
            code: "equivalence_review_not_supported",
            http_status: None,
            partial_usage: None,
        })
    }
}
