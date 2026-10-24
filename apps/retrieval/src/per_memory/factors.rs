//! Semantic judgments remain fallible; only their composition is deterministic.
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryFactorScores {
    pub same_context: f64,
    pub requested_fact: f64,
    pub premise_correction: f64,
}
impl MemoryFactorScores {
    pub fn valid(&self) -> bool {
        [
            self.same_context,
            self.requested_fact,
            self.premise_correction,
        ]
        .iter()
        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryFactorChecks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scores: Option<MemoryFactorScores>,
    pub memory_id: String,
    pub revision: i64,
    pub same_context: bool,
    pub requested_fact: bool,
    pub premise_correction: bool,
}
impl MemoryFactorChecks {
    pub fn accepted(&self) -> bool {
        self.same_context && (self.requested_fact || self.premise_correction)
    }
}

/// Each judgment includes referent/context compatibility; there is no third veto.
#[derive(Debug, Clone, Serialize)]
pub struct ScopedEvidenceChecks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal_anchor: Option<super::anchor::LiteralAnchor>,
    pub memory_id: String,
    pub revision: i64,
    pub requested_fact_same_context: bool,
    pub premise_correction_same_context: bool,
}
impl ScopedEvidenceChecks {
    pub fn accepted(&self) -> bool {
        (self.requested_fact_same_context || self.premise_correction_same_context)
            && self
                .literal_anchor
                .as_ref()
                .is_none_or(|a| a.present_in_source != Some(false))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_never_substitutes_for_a_requested_fact_or_correction() {
        for context in [false, true] {
            for fact in [false, true] {
                for correction in [false, true] {
                    let c = MemoryFactorChecks {
                        scores: None,
                        memory_id: "m".into(),
                        revision: 1,
                        same_context: context,
                        requested_fact: fact,
                        premise_correction: correction,
                    };
                    let expected = matches!(
                        (context, fact, correction),
                        (true, true, false) | (true, false, true) | (true, true, true)
                    );
                    assert_eq!(c.accepted(), expected);
                }
            }
        }
    }
}
