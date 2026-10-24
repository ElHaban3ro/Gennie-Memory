//! Structural validation of individual relevance assessments, not semantic proof.
use crate::model::Candidate;
use serde::Serialize;
pub mod anchor;
pub mod contextual;
pub mod factors;
pub mod verification;

pub const CANDIDATE_LIMIT: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelation {
    SupportsAnswer,
    RefutesPremise,
    PendingOrPartial,
    /// Binary utility judgment; deliberately makes no relation-category claim.
    UsefulEvidence,
    Unrelated,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryAssessment {
    pub memory_id: String,
    pub revision: i64,
    pub relation: EvidenceRelation,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryChoiceScore {
    pub memory_id: String,
    pub revision: i64,
    pub a: f64,
    pub none: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerMemoryTrace {
    pub items: Vec<MemoryAssessment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factor_checks: Option<Vec<factors::MemoryFactorChecks>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoped_checks: Option<Vec<factors::ScopedEvidenceChecks>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choice_scores: Option<Vec<MemoryChoiceScore>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<verification::VerificationTrace>,
}

impl PerMemoryTrace {
    pub fn selected_ids(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter(|item| item.relation != EvidenceRelation::Unrelated)
            .map(|item| item.memory_id.as_str())
            .collect()
    }

    pub fn valid(&self, candidates: &[Candidate], first_selected: Option<&str>) -> bool {
        let count = candidates.len().min(CANDIDATE_LIMIT);
        self.scoped_checks.as_ref().is_none_or(|checks| {
            checks.len() == count
                && checks.iter().zip(&self.items).all(|(check, item)| {
                    check.memory_id == item.memory_id
                        && check.revision == item.revision
                        && item.relation
                            == if check.accepted() {
                                EvidenceRelation::UsefulEvidence
                            } else {
                                EvidenceRelation::Unrelated
                            }
                })
        }) && self.factor_checks.as_ref().is_none_or(|checks| {
            checks
                .iter()
                .all(|c| c.scores.as_ref().is_none_or(|s| s.valid()))
                && checks.len() == count
                && checks.iter().zip(&self.items).all(|(check, item)| {
                    check.memory_id == item.memory_id
                        && check.revision == item.revision
                        && item.relation
                            == if check.accepted() {
                                EvidenceRelation::UsefulEvidence
                            } else {
                                EvidenceRelation::Unrelated
                            }
                })
        }) && self.request_sha256.as_ref().is_none_or(|h| {
            h.len() == 64
                && h.bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        }) && self.choice_scores.as_ref().is_none_or(|scores| {
            scores.len() == count
                && scores.iter().zip(&self.items).all(|(s, item)| {
                    s.memory_id == item.memory_id
                        && s.revision == item.revision
                        && s.a.is_finite()
                        && s.none.is_finite()
                        && (0.0..=1.0).contains(&s.a)
                        && (0.0..=1.0).contains(&s.none)
                        && (s.a + s.none - 1.0).abs() <= 0.02
                })
        }) && self.items.len() == count
            && self
                .items
                .iter()
                .zip(&candidates[..count])
                .all(|(item, candidate)| {
                    item.memory_id == candidate.memory_id && item.revision == candidate.revision
                })
            && self.selected_ids().first().copied() == first_selected
    }

    pub fn anchors_valid(&self, query: &str, candidates: &[Candidate], required: bool) -> bool {
        let Some(checks) = &self.scoped_checks else {
            return !required;
        };
        if !required {
            return checks.iter().all(|c| c.literal_anchor.is_none());
        }
        let Some(first) = checks.first().and_then(|c| c.literal_anchor.as_ref()) else {
            return false;
        };
        checks.len() == candidates.len()
            && checks.iter().zip(candidates).all(|(c, m)| {
                c.literal_anchor.as_ref().is_some_and(|a| {
                    a.query_token == first.query_token && a.valid(query, &m.summary)
                })
            })
    }

    pub fn relations_match_task(&self, binary: bool) -> bool {
        self.items.iter().all(|item| match item.relation {
            EvidenceRelation::Unrelated => true,
            EvidenceRelation::UsefulEvidence => binary,
            _ => !binary,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatible_positives_survive_and_trace_cannot_reorder_omit_or_change_revision() {
        let candidates: Vec<Candidate> = (0..20)
            .map(|i| Candidate {
                memory_id: format!("m-{i}"),
                revision: 2,
                summary: "saved fact".into(),
                conditions: None,
                source: None,
                state: "active".into(),
                bm25: 0.0,
            })
            .collect();
        let mut trace = PerMemoryTrace {
            choice_scores: None,
            scoped_checks: None,
            factor_checks: None,
            request_sha256: None,
            verification: None,
            items: candidates[..16]
                .iter()
                .enumerate()
                .map(|(i, c)| MemoryAssessment {
                    memory_id: c.memory_id.clone(),
                    revision: c.revision,
                    relation: if i < 2 {
                        EvidenceRelation::PendingOrPartial
                    } else {
                        EvidenceRelation::Unrelated
                    },
                })
                .collect(),
        };
        assert_eq!(trace.selected_ids(), ["m-0", "m-1"]);
        assert!(trace.valid(&candidates, Some("m-0")));
        trace.choice_scores = Some(
            trace
                .items
                .iter()
                .map(|item| MemoryChoiceScore {
                    memory_id: item.memory_id.clone(),
                    revision: item.revision,
                    a: 0.8,
                    none: 0.2,
                })
                .collect(),
        );
        assert!(trace.valid(&candidates, Some("m-0")));
        trace.choice_scores.as_mut().unwrap()[0].revision = 1;
        assert!(!trace.valid(&candidates, Some("m-0")));
        trace.choice_scores.as_mut().unwrap()[0].revision = 2;
        trace.choice_scores.as_mut().unwrap()[0].a = f64::NAN;
        assert!(!trace.valid(&candidates, Some("m-0")));
        trace.choice_scores.as_mut().unwrap()[0].a = 0.8;
        trace.choice_scores.as_mut().unwrap().pop();
        assert!(!trace.valid(&candidates, Some("m-0")));
        trace.choice_scores = None;
        assert!(trace.relations_match_task(false));
        assert!(!trace.relations_match_task(true));
        trace.items[0].relation = EvidenceRelation::UsefulEvidence;
        assert!(!trace.relations_match_task(false));
        assert!(!trace.relations_match_task(true));
        trace.items[1].relation = EvidenceRelation::UsefulEvidence;
        assert!(trace.relations_match_task(true));
        assert!(trace.valid(&candidates, Some("m-0")));
        assert!(!trace.valid(&candidates, Some("m-1")));
        trace.items[1].revision = 1;
        assert!(!trace.valid(&candidates, Some("m-0")));
        trace.items[1].revision = 2;
        trace.items.swap(0, 1);
        assert!(!trace.valid(&candidates, Some("m-1")));
        trace.items.swap(0, 1);
        trace.items.pop();
        assert!(!trace.valid(&candidates, Some("m-0")));
        assert!(
            PerMemoryTrace {
                choice_scores: None,
                scoped_checks: None,
                factor_checks: None,
                request_sha256: None,
                items: vec![],
                verification: None
            }
            .valid(&[], None)
        );
    }
}
