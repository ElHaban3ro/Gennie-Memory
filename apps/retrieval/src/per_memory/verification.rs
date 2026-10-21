//! Receipt and intersection validation; never a semantic applicability proof.
use super::{EvidenceRelation, MemoryAssessment};
use crate::model::DecisionResult;
use serde::Serialize;

pub const VERIFIER_MODEL: &str = "openai/gpt-6-luna-decisions";
pub const PREMISE_VERIFIER_MODEL: &str = "perplexity/pplx-decider-v1.2";
pub const PREMISE_VERIFIER_REVISION: &str = "perplexity/pplx-decider-v1.2-20261016";

#[derive(Debug, Clone, Serialize)]
pub struct CompleteEvidencePaths {
    pub requested_fact_or_partial: bool,
    pub ordinary_correction: bool,
    pub necessary_premise_incompatible: bool,
    pub explicit_context_contrast: bool,
}
impl CompleteEvidencePaths {
    pub fn accepted(&self) -> bool {
        self.requested_fact_or_partial
            || self.ordinary_correction && self.necessary_premise_incompatible
            || self.explicit_context_contrast
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StageUsage {
    pub requested_model: String,
    pub served_model: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reported_cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorrectionEvidencePaths {
    pub ordinary_correction: bool,
    pub explicit_context_contrast: bool,
}
impl CorrectionEvidencePaths {
    pub fn accepted(&self) -> bool {
        self.ordinary_correction || self.explicit_context_contrast
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckedMemory {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete_paths: Option<CompleteEvidencePaths>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<CorrectionEvidencePaths>,
    pub memory_id: String,
    pub revision: i64,
    pub useful: bool,
    pub usage: StageUsage,
}

/// One verdict per proposed positive; the batch receipt belongs to the request.
#[derive(Debug, Clone, Serialize)]
pub struct BatchVerdict {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<CorrectionEvidencePaths>,
    pub memory_id: String,
    pub revision: i64,
    pub useful: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchVerification {
    pub usage: StageUsage,
    pub verdicts: Vec<BatchVerdict>,
}

struct VerdictRef<'a> {
    complete_paths: Option<&'a CompleteEvidencePaths>,
    memory_id: &'a str,
    revision: i64,
    useful: bool,
    paths: Option<&'a CorrectionEvidencePaths>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationTrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch: Option<BatchVerification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind_proposal: Option<Vec<super::contextual::EvidenceKindProposal>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_proposal: Option<Vec<super::contextual::ContextualRoleProposal>>,
    pub proposal: Vec<MemoryAssessment>,
    /// Present only for correction-only verification. Direct facts are not checked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoped_proposal: Option<Vec<super::factors::ScopedEvidenceChecks>>,
    pub proposal_usage: StageUsage,
    pub checks: Vec<CheckedMemory>,
}

#[derive(Debug, PartialEq)]
pub struct UsageTotals {
    pub served_model: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reported_cost_usd: Option<f64>,
    pub known_cost_usd: Option<f64>,
    pub missing_cost_receipts: usize,
}

impl VerificationTrace {
    fn verdicts(&self) -> Vec<VerdictRef<'_>> {
        match &self.batch {
            Some(batch) => batch
                .verdicts
                .iter()
                .map(|c| VerdictRef {
                    complete_paths: None,
                    memory_id: &c.memory_id,
                    revision: c.revision,
                    useful: c.useful,
                    paths: c.paths.as_ref(),
                })
                .collect(),
            None => self
                .checks
                .iter()
                .map(|c| VerdictRef {
                    complete_paths: c.complete_paths.as_ref(),
                    memory_id: &c.memory_id,
                    revision: c.revision,
                    useful: c.useful,
                    paths: c.paths.as_ref(),
                })
                .collect(),
        }
    }
    pub fn matches_policy(&self, policy: crate::model::SelectionPolicy) -> bool {
        let complete = policy == crate::model::SelectionPolicy::PerMemoryCompletePremiseV1;
        if self
            .checks
            .iter()
            .any(|c| c.complete_paths.is_some() != complete || complete && c.paths.is_some())
        {
            return false;
        }
        let batched = policy == crate::model::SelectionPolicy::PerMemoryRoleBatchV1;
        if self.batch.is_some() && !batched || batched && !self.checks.is_empty() {
            return false;
        }
        if policy.uses_kind_proposal() {
            self.kind_proposal.is_some()
                && self.scoped_proposal.is_none()
                && self.role_proposal.is_none()
        } else {
            policy.uses_verification()
                && self.kind_proposal.is_none()
                && (self.scoped_proposal.is_some()
                    == matches!(policy.correction_verification_version(), Some(1..=4)))
                && (self.role_proposal.is_some()
                    == (policy.correction_verification_version() == Some(5)))
                && self.checks.iter().all(|c| {
                    c.paths.is_some()
                        == matches!(policy.correction_verification_version(), Some(4..=5))
                })
        }
    }

    pub fn totals(&self) -> Option<UsageTotals> {
        if self.batch.as_ref().is_some_and(|b| {
            !self.checks.is_empty()
                || b.verdicts.is_empty()
                || b.usage.requested_model != VERIFIER_MODEL
        }) || self.checks.iter().any(|c| {
            c.usage.requested_model != VERIFIER_MODEL
                && c.usage.requested_model != PREMISE_VERIFIER_MODEL
        }) {
            return None;
        }
        let receipts: Vec<_> = std::iter::once(&self.proposal_usage)
            .chain(self.checks.iter().map(|c| &c.usage))
            .chain(self.batch.iter().map(|b| &b.usage))
            .collect();
        if receipts.iter().any(|r| {
            r.requested_model.trim().is_empty()
                || r.served_model
                    .as_ref()
                    .is_some_and(|m| m.trim().is_empty() || m.len() > 128)
                || r.reported_cost_usd
                    .is_some_and(|c| !c.is_finite() || c < 0.0)
        }) {
            return None;
        }
        let mut known_cost = None;
        let mut missing = 0;
        for receipt in &receipts {
            if let Some(cost) = receipt.reported_cost_usd {
                let sum = known_cost.unwrap_or(0.0) + cost;
                if !sum.is_finite() {
                    return None;
                }
                known_cost = Some(sum);
            } else {
                missing += 1;
            }
        }
        let tokens = |input: bool| -> Option<Option<u64>> {
            let mut sum = 0_u64;
            let mut complete = true;
            for receipt in &receipts {
                match if input {
                    receipt.input_tokens
                } else {
                    receipt.output_tokens
                } {
                    Some(n) => sum = sum.checked_add(n)?,
                    None => complete = false,
                }
            }
            Some(complete.then_some(sum))
        };
        let first = receipts[0].served_model.clone();
        let served = if receipts.iter().all(|r| r.served_model == first) {
            first
        } else {
            None
        };
        Some(UsageTotals {
            served_model: served,
            input_tokens: tokens(true)?,
            output_tokens: tokens(false)?,
            reported_cost_usd: if missing == 0 { known_cost } else { None },
            known_cost_usd: known_cost,
            missing_cost_receipts: missing,
        })
    }

    pub fn valid(&self, final_items: &[MemoryAssessment], proposer_model: &str) -> bool {
        self.checks.iter().all(|c| c.complete_paths.is_none())
            && self.valid_with_verifier(final_items, proposer_model, VERIFIER_MODEL)
    }
    pub fn valid_for_policy(
        &self,
        final_items: &[MemoryAssessment],
        proposer_model: &str,
        policy: crate::model::SelectionPolicy,
    ) -> bool {
        self.matches_policy(policy)
            && policy.verifier_model().is_some_and(|verifier| {
                self.valid_with_verifier(final_items, proposer_model, verifier)
                    && (policy != crate::model::SelectionPolicy::PerMemoryCompletePremiseV1
                        || self.checks.iter().all(|c| {
                            c.usage.served_model.as_deref() == Some(PREMISE_VERIFIER_REVISION)
                        }))
            })
    }
    fn valid_with_verifier(
        &self,
        final_items: &[MemoryAssessment],
        proposer_model: &str,
        verifier: &str,
    ) -> bool {
        if self.proposal.len() != final_items.len()
            || self.proposal_usage.requested_model != proposer_model
            || self.totals().is_none()
            || self
                .checks
                .iter()
                .any(|c| c.usage.requested_model != verifier)
            || [
                self.scoped_proposal.is_some(),
                self.role_proposal.is_some(),
                self.kind_proposal.is_some(),
            ]
            .into_iter()
            .filter(|x| *x)
            .count()
                > 1
        {
            return false;
        }
        if self.scoped_proposal.as_ref().is_some_and(|rows| {
            rows.len() != self.proposal.len()
                || rows.iter().zip(&self.proposal).any(|(s, p)| {
                    s.memory_id != p.memory_id
                        || s.revision != p.revision
                        || (p.relation == EvidenceRelation::UsefulEvidence) != s.accepted()
                })
        }) {
            return false;
        }
        if self.role_proposal.as_ref().is_some_and(|rows| {
            rows.len() != self.proposal.len()
                || rows.iter().zip(&self.proposal).any(|(s, p)| {
                    s.memory_id != p.memory_id
                        || s.revision != p.revision
                        || (p.relation == EvidenceRelation::UsefulEvidence) != s.accepted()
                })
        }) {
            return false;
        }
        if self.kind_proposal.as_ref().is_some_and(|rows| {
            rows.len() != self.proposal.len()
                || rows.iter().zip(&self.proposal).any(|(s, p)| {
                    s.memory_id != p.memory_id
                        || s.revision != p.revision
                        || (p.relation == EvidenceRelation::UsefulEvidence) != s.accepted()
                })
        }) {
            return false;
        }
        let direct = |i: usize| {
            self.role_proposal.as_ref().map_or_else(
                || {
                    self.scoped_proposal
                        .as_ref()
                        .is_some_and(|s| s[i].requested_fact_same_context)
                },
                |r| r[i].direct_fact(),
            )
        };
        let positive: Vec<_> = self
            .proposal
            .iter()
            .enumerate()
            .filter(|(i, p)| p.relation == EvidenceRelation::UsefulEvidence && !direct(*i))
            .collect();
        let verdicts = self.verdicts();
        if self.batch.is_some() && self.kind_proposal.is_none()
            || positive.len() != verdicts.len()
            || positive.iter().zip(&verdicts).any(|((i, p), c)| {
                p.memory_id != c.memory_id
                    || p.revision != c.revision
                    || c.paths.as_ref().is_some_and(|r| r.accepted() != c.useful)
                    || c.complete_paths.is_some_and(|p| p.accepted() != c.useful)
                    || self.kind_proposal.as_ref().is_some_and(|rows| {
                        c.complete_paths.is_none()
                            && c.paths.is_some()
                                != (rows[*i].evidence_kind
                                    == super::contextual::ContextualEvidenceKind::PremiseCorrection)
                    })
            })
        {
            return false;
        }
        self.proposal
            .iter()
            .zip(final_items)
            .enumerate()
            .all(|(i, (p, f))| {
                if p.memory_id != f.memory_id || p.revision != f.revision {
                    return false;
                }
                let useful = match p.relation {
                    EvidenceRelation::Unrelated => false,
                    EvidenceRelation::UsefulEvidence if direct(i) => true,
                    EvidenceRelation::UsefulEvidence => verdicts
                        .iter()
                        .find(|c| c.memory_id == p.memory_id)
                        .is_some_and(|c| c.useful),
                    _ => return false,
                };
                f.relation
                    == if useful {
                        EvidenceRelation::UsefulEvidence
                    } else {
                        EvidenceRelation::Unrelated
                    }
            })
    }

    pub fn usage_matches(&self, decision: &DecisionResult) -> bool {
        self.totals().is_some_and(|t| {
            t.served_model == decision.served_model
                && t.input_tokens == decision.input_tokens
                && t.output_tokens == decision.output_tokens
                && t.reported_cost_usd == decision.reported_cost_usd
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_policy_binds_all_paths_the_verifier_and_its_revision() {
        use super::super::contextual::{ContextualEvidenceKind as K, EvidenceKindProposal as P};
        use crate::model::SelectionPolicy as S;
        let proposal = vec![MemoryAssessment {
            memory_id: "a".into(),
            revision: 1,
            relation: EvidenceRelation::UsefulEvidence,
        }];
        let final_items = vec![MemoryAssessment {
            memory_id: "a".into(),
            revision: 1,
            relation: EvidenceRelation::Unrelated,
        }];
        let mut u = receipt(PREMISE_VERIFIER_MODEL, Some(0.02));
        u.served_model = Some(PREMISE_VERIFIER_REVISION.into());
        let mut v = VerificationTrace {
            proposal: proposal.clone(),
            proposal_usage: receipt("proposal", Some(0.01)),
            kind_proposal: Some(vec![P {
                memory_id: "a".into(),
                revision: 1,
                evidence_kind: K::RequestedFactOrPartial,
            }]),
            scoped_proposal: None,
            role_proposal: None,
            batch: None,
            checks: vec![CheckedMemory {
                memory_id: "a".into(),
                revision: 1,
                useful: false,
                paths: None,
                complete_paths: Some(CompleteEvidencePaths {
                    requested_fact_or_partial: false,
                    ordinary_correction: true,
                    necessary_premise_incompatible: false,
                    explicit_context_contrast: false,
                }),
                usage: u,
            }],
        };
        assert!(v.valid_for_policy(&final_items, "proposal", S::PerMemoryCompletePremiseV1));
        assert!(!v.valid_for_policy(&final_items, "proposal", S::PerMemoryRoleVerifiedV2));
        let mut bad = v.clone();
        bad.checks[0]
            .complete_paths
            .as_mut()
            .unwrap()
            .necessary_premise_incompatible = true;
        assert!(!bad.valid_for_policy(&final_items, "proposal", S::PerMemoryCompletePremiseV1));
        let mut bad = v.clone();
        bad.checks[0].usage.served_model = Some("unversioned".into());
        assert!(!bad.valid_for_policy(&final_items, "proposal", S::PerMemoryCompletePremiseV1));
        let mut bad = v.clone();
        bad.checks[0].complete_paths = None;
        assert!(!bad.valid_for_policy(&final_items, "proposal", S::PerMemoryCompletePremiseV1));
        v.checks.clear();
        assert!(!v.valid_for_policy(&final_items, "proposal", S::PerMemoryCompletePremiseV1));
    }
    #[test]
    fn kind_only_proposal_requires_every_positive_check_and_role_specific_paths() {
        use super::super::contextual::{ContextualEvidenceKind as K, EvidenceKindProposal as P};
        use crate::model::SelectionPolicy;
        let mut final_items = items();
        final_items[0].relation = EvidenceRelation::Unrelated;
        let mut t = VerificationTrace {
            batch: None,
            kind_proposal: Some(vec![
                P {
                    memory_id: "m0".into(),
                    revision: 1,
                    evidence_kind: K::RequestedFactOrPartial,
                },
                P {
                    memory_id: "m1".into(),
                    revision: 1,
                    evidence_kind: K::PremiseCorrection,
                },
                P {
                    memory_id: "m2".into(),
                    revision: 1,
                    evidence_kind: K::Unrelated,
                },
            ]),
            role_proposal: None,
            scoped_proposal: None,
            proposal: items(),
            proposal_usage: receipt("proposal", Some(0.01)),
            checks: vec![
                CheckedMemory {
                    complete_paths: None,
                    memory_id: "m0".into(),
                    revision: 1,
                    useful: false,
                    paths: None,
                    usage: receipt(VERIFIER_MODEL, Some(0.02)),
                },
                CheckedMemory {
                    complete_paths: None,
                    memory_id: "m1".into(),
                    revision: 1,
                    useful: true,
                    paths: Some(CorrectionEvidencePaths {
                        ordinary_correction: false,
                        explicit_context_contrast: true,
                    }),
                    usage: receipt(VERIFIER_MODEL, Some(0.02)),
                },
            ],
        };
        assert!(t.matches_policy(SelectionPolicy::PerMemoryRoleVerifiedV1));
        assert!(!t.matches_policy(SelectionPolicy::PerMemoryVerifiedV1));
        assert!(t.valid(&final_items, "proposal"));
        let mut batch = t.clone();
        batch.batch = Some(BatchVerification {
            usage: receipt(VERIFIER_MODEL, Some(0.02)),
            verdicts: batch
                .checks
                .iter()
                .map(|c| BatchVerdict {
                    memory_id: c.memory_id.clone(),
                    revision: c.revision,
                    useful: c.useful,
                    paths: c.paths.clone(),
                })
                .collect(),
        });
        assert!(batch.totals().is_none()); // Reject duplicated individual receipts.
        batch.checks.clear();
        assert!(batch.matches_policy(SelectionPolicy::PerMemoryRoleBatchV1));
        assert!(!batch.matches_policy(SelectionPolicy::PerMemoryRoleVerifiedV1));
        assert!(batch.valid(&final_items, "proposal"));
        assert_eq!(batch.totals().unwrap().input_tokens, Some(20));
        assert!((batch.totals().unwrap().reported_cost_usd.unwrap() - 0.03).abs() < 1e-12);
        let mut missing = batch.clone();
        missing.batch.as_mut().unwrap().verdicts.remove(0);
        assert!(!missing.valid(&final_items, "proposal"));
        batch.batch.as_mut().unwrap().usage.reported_cost_usd = None;
        assert_eq!(batch.totals().unwrap().missing_cost_receipts, 1);
        assert_eq!(batch.totals().unwrap().reported_cost_usd, None);
        let mut bad = t.clone();
        bad.checks.remove(0);
        assert!(!bad.valid(&final_items, "proposal"));
        t.checks[0].paths = Some(CorrectionEvidencePaths {
            ordinary_correction: false,
            explicit_context_contrast: false,
        });
        assert!(!t.valid(&final_items, "proposal"));
        t.checks[0].paths = None;
        t.checks[1].paths = None;
        assert!(!t.valid(&final_items, "proposal"));
    }
    #[test]
    fn contextual_proposal_binds_raw_roles_and_checks_corrections_despite_false_fact_context() {
        use super::super::contextual::{ContextualEvidenceKind as K, ContextualRoleProposal as R};
        let mut proposal = items();
        proposal[2].relation = EvidenceRelation::Unrelated;
        let mut final_items = proposal.clone();
        final_items[1].relation = EvidenceRelation::Unrelated;
        let mut trace = VerificationTrace {
            batch: None,
            kind_proposal: None,
            role_proposal: Some(vec![
                R {
                    memory_id: "m0".into(),
                    revision: 1,
                    context_for_fact: true,
                    evidence_kind: K::RequestedFactOrPartial,
                },
                R {
                    memory_id: "m1".into(),
                    revision: 1,
                    context_for_fact: false,
                    evidence_kind: K::PremiseCorrection,
                },
                R {
                    memory_id: "m2".into(),
                    revision: 1,
                    context_for_fact: false,
                    evidence_kind: K::RequestedFactOrPartial,
                },
            ]),
            scoped_proposal: None,
            proposal,
            proposal_usage: receipt("proposal", Some(0.01)),
            checks: vec![CheckedMemory {
                complete_paths: None,
                memory_id: "m1".into(),
                revision: 1,
                useful: false,
                paths: Some(CorrectionEvidencePaths {
                    ordinary_correction: false,
                    explicit_context_contrast: false,
                }),
                usage: receipt(VERIFIER_MODEL, Some(0.02)),
            }],
        };
        assert!(trace.valid(&final_items, "proposal"));
        trace.checks[0].useful = true;
        trace.checks[0]
            .paths
            .as_mut()
            .unwrap()
            .explicit_context_contrast = true;
        final_items[1].relation = EvidenceRelation::UsefulEvidence;
        assert!(trace.valid(&final_items, "proposal"));
        let mut bad = trace.clone();
        bad.proposal[2].relation = EvidenceRelation::UsefulEvidence;
        assert!(!bad.valid(&final_items, "proposal"));
        let mut bad = trace.clone();
        bad.role_proposal.as_mut().unwrap()[1].revision = 2;
        assert!(!bad.valid(&final_items, "proposal"));
        let mut bad = trace.clone();
        bad.checks.clear();
        assert!(!bad.valid(&final_items, "proposal"));
        assert!((trace.totals().unwrap().reported_cost_usd.unwrap() - 0.03).abs() < 1e-12);
    }
    #[test]
    fn typed_contrast_cannot_disagree_with_the_accepted_check() {
        let mut final_items = items();
        let mut trace = VerificationTrace {
            batch: None,
            kind_proposal: None,
            role_proposal: None,
            proposal: items(),
            scoped_proposal: None,
            proposal_usage: receipt("proposal", Some(0.01)),
            checks: (0..2)
                .map(|i| CheckedMemory {
                    complete_paths: None,
                    memory_id: format!("m{i}"),
                    revision: 1,
                    useful: true,
                    paths: Some(CorrectionEvidencePaths {
                        ordinary_correction: false,
                        explicit_context_contrast: true,
                    }),
                    usage: receipt(VERIFIER_MODEL, Some(0.02)),
                })
                .collect(),
        };
        assert!(trace.valid(&final_items, "proposal"));
        trace.checks[0].useful = false;
        final_items[0].relation = EvidenceRelation::Unrelated;
        assert!(!trace.valid(&final_items, "proposal"));
        trace.checks[0]
            .paths
            .as_mut()
            .unwrap()
            .explicit_context_contrast = false;
        assert!(trace.valid(&final_items, "proposal"));
    }
    #[test]
    fn selective_checks_preserve_direct_facts_and_require_every_correction_only_check() {
        let mut final_items = items();
        final_items[1].relation = EvidenceRelation::Unrelated;
        let mut trace = VerificationTrace {
            batch: None,
            kind_proposal: None,
            role_proposal: None,
            proposal: items(),
            scoped_proposal: Some(
                (0..3)
                    .map(|i| super::super::factors::ScopedEvidenceChecks {
                        literal_anchor: None,
                        memory_id: format!("m{i}"),
                        revision: 1,
                        requested_fact_same_context: i == 0,
                        premise_correction_same_context: i == 1,
                    })
                    .collect(),
            ),
            proposal_usage: receipt("proposal", Some(0.01)),
            checks: vec![CheckedMemory {
                complete_paths: None,
                paths: None,
                memory_id: "m1".into(),
                revision: 1,
                useful: false,
                usage: receipt(VERIFIER_MODEL, Some(0.02)),
            }],
        };
        assert!(trace.valid(&final_items, "proposal"));
        assert!((trace.totals().unwrap().reported_cost_usd.unwrap() - 0.03).abs() < 1e-12);
        let mut bad = trace.clone();
        bad.checks.clear();
        assert!(!bad.valid(&final_items, "proposal"));
        let mut bad = trace.clone();
        bad.checks[0].memory_id = "m0".into();
        assert!(!bad.valid(&final_items, "proposal"));
        let mut bad = trace.clone();
        bad.scoped_proposal = None;
        assert!(!bad.valid(&final_items, "proposal"));
        trace.checks[0].useful = true;
        final_items[1].relation = EvidenceRelation::UsefulEvidence;
        assert!(trace.valid(&final_items, "proposal"));
    }
    fn receipt(model: &str, cost: Option<f64>) -> StageUsage {
        StageUsage {
            requested_model: model.into(),
            served_model: Some(model.into()),
            input_tokens: Some(10),
            output_tokens: Some(0),
            reported_cost_usd: cost,
        }
    }
    fn items() -> Vec<MemoryAssessment> {
        (0..3)
            .map(|i| MemoryAssessment {
                memory_id: format!("m{i}"),
                revision: 1,
                relation: if i < 2 {
                    EvidenceRelation::UsefulEvidence
                } else {
                    EvidenceRelation::Unrelated
                },
            })
            .collect()
    }
    #[test]
    fn intersection_checks_every_proposal_and_keeps_unknown_cost_distinct_from_zero() {
        let mut final_items = items();
        final_items[1].relation = EvidenceRelation::Unrelated;
        let mut trace = VerificationTrace {
            batch: None,
            kind_proposal: None,
            role_proposal: None,
            scoped_proposal: None,
            proposal: items(),
            proposal_usage: receipt("proposal", Some(0.01)),
            checks: (0..2)
                .map(|i| CheckedMemory {
                    complete_paths: None,
                    paths: None,
                    memory_id: format!("m{i}"),
                    revision: 1,
                    useful: i == 0,
                    usage: receipt(VERIFIER_MODEL, Some(0.02)),
                })
                .collect(),
        };
        assert!(trace.valid(&final_items, "proposal"));
        let total = trace.totals().unwrap();
        assert_eq!(total.input_tokens, Some(30));
        assert!((total.reported_cost_usd.unwrap() - 0.05).abs() < 1e-12);
        assert!(total.served_model.is_none());
        trace.checks[1].usage.reported_cost_usd = None;
        let total = trace.totals().unwrap();
        assert!(total.reported_cost_usd.is_none());
        assert!((total.known_cost_usd.unwrap() - 0.03).abs() < 1e-12);
        assert_eq!(total.missing_cost_receipts, 1);
        assert!(trace.valid(&final_items, "proposal"));
        for mutation in 0..5 {
            let mut bad = trace.clone();
            match mutation {
                0 => {
                    bad.checks.pop();
                }
                1 => bad.checks.push(bad.checks[0].clone()),
                2 => bad.checks[1].revision = 2,
                3 => bad.checks[1].usage.requested_model = "different".into(),
                _ => bad.checks[0].useful = false,
            }
            assert!(!bad.valid(&final_items, "proposal"));
        }
        trace.checks[1].usage.reported_cost_usd = Some(0.0);
        assert!(trace.totals().unwrap().reported_cost_usd.is_some());
        trace.checks[1].usage.reported_cost_usd = Some(-0.01);
        assert!(trace.totals().is_none());
    }
}
