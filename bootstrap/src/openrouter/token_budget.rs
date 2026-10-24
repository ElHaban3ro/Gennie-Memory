use super::*;
use gennie_memory_retrieval::{classifier::BudgetedDecision, token_budget::InputBudget};

#[derive(Clone)]
pub(super) struct ActiveBudget {
    stage: &'static str,
    counter: Arc<Mutex<InputBudget>>,
}
impl ActiveBudget {
    pub(super) fn new(stage: &'static str, remaining: usize) -> Self {
        Self {
            stage,
            counter: Arc::new(Mutex::new(InputBudget::new(remaining))),
        }
    }
    pub(super) fn inspect(&self, bytes: &[u8]) -> Result<(), DecisionError> {
        self.counter
            .lock()
            .map_err(|_| failure("token_counter_unavailable"))?
            .inspect(self.stage, bytes)
            .map_err(failure)
    }
    pub(super) fn finish(&self, result: Result<DecisionResult, DecisionError>) -> BudgetedDecision {
        let requests = match self.counter.lock() {
            Ok(counter) => counter.requests.clone(),
            Err(_) => {
                return BudgetedDecision {
                    result: Err(failure("token_counter_unavailable")),
                    requests: Vec::new(),
                };
            }
        };
        BudgetedDecision { result, requests }
    }
}

impl OpenRouterClassifier {
    pub(super) fn classify_with_tokens(
        &self,
        query: &str,
        scope: &str,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        policy: SelectionPolicy,
        until: Instant,
        remaining: usize,
    ) -> BudgetedDecision {
        let guard = ActiveBudget::new("classification", remaining);
        let mut scoped = self.clone();
        scoped.token_budget = Some(guard.clone());
        guard.finish(scoped.classify_until(query, scope, candidates, provider, policy, until))
    }
    pub(super) fn equivalent_with_tokens(
        &self,
        scope: &str,
        anchor: &Candidate,
        candidates: &[Candidate],
        provider: ClassifierProvider,
        until: Instant,
        remaining: usize,
    ) -> BudgetedDecision {
        let guard = ActiveBudget::new("equivalence_review", remaining);
        let mut scoped = self.clone();
        scoped.token_budget = Some(guard.clone());
        guard.finish(scoped.suggest_equivalent_until(scope, anchor, candidates, provider, until))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn product_providers_and_policies_guard_full_payload_and_second_pass() {
        let classifier = OpenRouterClassifier::new(Some("synthetic-not-a-secret".into())).unwrap();
        let mut candidates = vec![Candidate {
            memory_id: "synthetic-m1".into(),
            revision: 1,
            summary: "Bogotá: no enviar si llueve.".into(),
            conditions: Some("Solo en octubre".into()),
            source: None,
            state: "active".into(),
            bm25: 0.0,
        }];
        let mut other = candidates[0].clone();
        other.memory_id = "synthetic-m2".into();
        candidates.push(other);
        for provider in [
            ClassifierProvider::Decisions,
            ClassifierProvider::Jev,
            ClassifierProvider::Jev114,
        ] {
            for policy in [
                SelectionPolicy::Baseline,
                SelectionPolicy::PerMemoryV1,
                SelectionPolicy::PerMemoryV2,
            ] {
                let result = classifier.classify_with_tokens(
                    "¿Enviar?",
                    "synthetic",
                    &candidates,
                    provider,
                    policy,
                    Instant::now() + Duration::from_secs(25),
                    1,
                );
                assert_eq!(result.result.unwrap_err().code, "input_token_limit");
                assert_eq!(result.requests.len(), 1);
                assert!(!result.requests[0].accepted);
                assert!(result.requests[0].json_tokens > 1);
            }
            let result = classifier.equivalent_with_tokens(
                "synthetic",
                &candidates[0],
                &candidates,
                provider,
                Instant::now() + Duration::from_secs(25),
                1,
            );
            assert_eq!(result.result.unwrap_err().code, "input_token_limit");
            assert_eq!(result.requests.len(), 1);
            assert_eq!(result.requests[0].stage, "equivalence_review");
            assert!(!result.requests[0].accepted);
        }
    }
}
