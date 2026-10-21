//! Fallible model judgments; context qualifies facts, never a global correction veto.
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextualEvidenceKind {
    RequestedFactOrPartial,
    PremiseCorrection,
    Unrelated,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextualRoleProposal {
    pub memory_id: String,
    pub revision: i64,
    pub context_for_fact: bool,
    pub evidence_kind: ContextualEvidenceKind,
}
impl ContextualRoleProposal {
    pub fn direct_fact(&self) -> bool {
        self.context_for_fact
            && self.evidence_kind == ContextualEvidenceKind::RequestedFactOrPartial
    }
    pub fn accepted(&self) -> bool {
        self.direct_fact() || self.evidence_kind == ContextualEvidenceKind::PremiseCorrection
    }
}

/// A real evidence-kind choice, without an invented context judgment.
#[derive(Debug, Clone, Serialize)]
pub struct EvidenceKindProposal {
    pub memory_id: String,
    pub revision: i64,
    pub evidence_kind: ContextualEvidenceKind,
}
impl EvidenceKindProposal {
    pub fn accepted(&self) -> bool {
        self.evidence_kind != ContextualEvidenceKind::Unrelated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn context_filters_only_requested_facts_and_never_globally_vetoes_corrections() {
        for context in [false, true] {
            for kind in [
                ContextualEvidenceKind::RequestedFactOrPartial,
                ContextualEvidenceKind::PremiseCorrection,
                ContextualEvidenceKind::Unrelated,
            ] {
                let p = ContextualRoleProposal {
                    memory_id: "m".into(),
                    revision: 1,
                    context_for_fact: context,
                    evidence_kind: kind,
                };
                assert_eq!(
                    p.direct_fact(),
                    context && kind == ContextualEvidenceKind::RequestedFactOrPartial
                );
                assert_eq!(
                    p.accepted(),
                    kind == ContextualEvidenceKind::PremiseCorrection || p.direct_fact()
                );
            }
        }
    }
}
