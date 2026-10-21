//! Proposed proposition-level evidence from an experimental classifier.
//! Literal validation checks structure and grounding, not semantic truth.
use crate::model::Candidate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SlotEvidenceTrace {
    #[serde(default)]
    pub version: String,
    pub query_anchors: Vec<QueryAnchor>,
    pub slots: Vec<QueryTarget>,
    pub premises: Vec<QueryTarget>,
    pub candidates: Vec<CandidateEvidence>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QueryAnchor {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub subject: Option<String>,
    pub stable_id: String,
    pub quote: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QueryTarget {
    pub id: String,
    #[serde(default)]
    pub dimension: String,
    pub quote: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateEvidence {
    pub memory_id: String,
    pub revision: i64,
    pub identity: IdentityAssessment,
    pub propositions: Vec<Proposition>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IdentityAssessment {
    pub status: IdentityStatus,
    pub anchor_id: Option<String>,
    pub stable_id_quote: Option<String>,
    pub subject_quote: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdentityStatus {
    Match,
    Incompatible,
    Unresolved,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Proposition {
    #[serde(default)]
    pub occurrence_id: String,
    pub quote: String,
    #[serde(default)]
    pub subject_quote: String,
    pub identity_anchor_id: Option<String>,
    #[serde(default)]
    pub predicate_quote: String,
    pub value: Option<String>,
    pub polarity_quote: Option<String>,
    pub modality_quote: Option<String>,
    pub event_quote: Option<String>,
    pub time_quote: Option<String>,
    pub conditions_quote: Option<String>,
    pub relation: EvidenceRelation,
    pub target_id: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelation {
    Answers,
    Partial,
    ExplicitUnmeasured,
    Refutes,
    Supports,
    ContextOnly,
    Unrelated,
    Unknown,
    /// The source quote is grounded, but its relation to the bound identity is ambiguous.
    /// This value is produced locally by the V2 adapter, never by the remote schema.
    UnresolvedAttribution,
}

impl SlotEvidenceTrace {
    /// Preserve ambiguous V2 source quotes while preventing them from becoming actionable.
    /// This is a conservative abstention signal, not a semantic parser or truth judgment.
    pub fn demote_ambiguous_attributions(&mut self, query: &str) -> usize {
        if self.version != "slot-evidence-v2" && self.version != "slot-evidence-v3" {
            return 0;
        }
        let mut demoted = 0;
        for entry in &mut self.candidates {
            for proposition in &mut entry.propositions {
                let actionable = matches!(
                    proposition.relation,
                    EvidenceRelation::Answers
                        | EvidenceRelation::Partial
                        | EvidenceRelation::ExplicitUnmeasured
                        | EvidenceRelation::Refutes
                        | EvidenceRelation::Supports
                );
                if !actionable {
                    continue;
                }
                // A memory may contain useful propositions about more than its
                // candidate-level identity. Attribute guards must inspect the
                // proposition's own validated query anchor, not the memory's
                // primary subject.
                let query_anchor = proposition
                    .identity_anchor_id
                    .as_deref()
                    .and_then(|anchor_id| self.query_anchors.iter().find(|a| a.id == anchor_id));
                let Some(query_anchor) = query_anchor else {
                    continue;
                };
                let id = query_anchor.stable_id.as_str();
                let anchor_kind = query_anchor.kind.as_str();
                let anchor_quote = query_anchor.quote.as_str();
                let anchor_subject = query_anchor.subject.as_deref();
                // Ignore IDs that occur before this proposition's subject: a
                // relative clause after a second entity usually binds to that
                // nearer subject, not to the candidate-level identity.
                let proposition_scope =
                    proposition_subject_scope(&proposition.quote, &proposition.subject_quote);
                let assessment = if self.version == "slot-evidence-v3" {
                    has_foreign_subject_before_predicate(
                        &proposition.quote,
                        id,
                        &proposition.predicate_quote,
                    )
                    .then_some(true)
                } else {
                    foreign_relative_assessment(proposition_scope, id)
                };
                let ambiguous_but_requested = assessment == Some(false)
                    && query_targets_speech_complement(
                        query,
                        &proposition.quote,
                        id,
                        &proposition.subject_quote,
                        anchor_kind,
                    );
                // A quoted/reporting subject is not automatically the subject of the
                // embedded fact. For example, "Ana reported that Bob lives in Quito"
                // does not answer where Ana lives. This narrow guard catches an
                // explicitly named, different subject after common reporting markers;
                // it is not a general coreference parser.
                let report_scope = proposition_subject_clause_scope(proposition_scope);
                let embedded_foreign_report_subject = anchor_subject
                    .is_some_and(|subject| has_foreign_named_report_subject(report_scope, subject))
                    && !query_requests_reported_content(
                        query,
                        &proposition.subject_quote,
                        anchor_kind,
                    );
                let explicitly_excluded = explicitly_excluded_speech_recipient_for_query(
                    &proposition.quote,
                    id,
                    query,
                    anchor_quote,
                    anchor_kind,
                );
                let exclusion_refutes_recipient_premise = explicitly_excluded
                    && proposition.relation == EvidenceRelation::Refutes
                    && self.premises.iter().any(|premise| {
                        premise.id == proposition.target_id
                            && premise.dimension == "correction_premise"
                            && has_speech_verb(&premise.quote)
                            && premise
                                .quote
                                .split(|character: char| {
                                    !character.is_alphanumeric()
                                        && !matches!(character, '-' | '_' | '.' | '/')
                                })
                                .any(|token| {
                                    token != id
                                        && token.chars().any(char::is_alphabetic)
                                        && token.chars().any(char::is_numeric)
                                        && token.contains(['-', '_', '.', '/'])
                                        && query_requests_speech_recipient(
                                            query,
                                            id,
                                            token,
                                            anchor_quote,
                                            anchor_kind,
                                        )
                                })
                    });
                // An explicit exclusion invalidates a claim that answers the requested
                // recipient slot, but can itself be useful when the classifier grounds it
                // as a refutation of that query premise. Do not erase corrective evidence.
                let exclusion_conflicts_with_relation = explicitly_excluded
                    && matches!(
                        proposition.relation,
                        EvidenceRelation::Answers
                            | EvidenceRelation::Partial
                            | EvidenceRelation::ExplicitUnmeasured
                            | EvidenceRelation::Supports
                    );
                if actionable
                    && !exclusion_refutes_recipient_premise
                    && (exclusion_conflicts_with_relation
                        || assessment.is_some() && !ambiguous_but_requested
                        || embedded_foreign_report_subject)
                {
                    proposition.relation = EvidenceRelation::UnresolvedAttribution;
                    proposition.target_id.clear();
                    demoted += 1;
                }
            }
        }
        demoted
    }

    /// Preserve propositions whose requested target explicitly names a different
    /// query entity than the proposition's own source subject.
    pub fn demote_mismatched_target_attributions(&mut self) -> usize {
        if self.version != "slot-evidence-v2" && self.version != "slot-evidence-v3" {
            return 0;
        }
        let (anchors, slots, premises) = (&self.query_anchors, &self.slots, &self.premises);
        let mut demoted = 0;
        for entry in &mut self.candidates {
            for proposition in &mut entry.propositions {
                if !matches!(
                    proposition.relation,
                    EvidenceRelation::Answers
                        | EvidenceRelation::Partial
                        | EvidenceRelation::ExplicitUnmeasured
                        | EvidenceRelation::Refutes
                        | EvidenceRelation::Supports
                ) {
                    continue;
                }
                if proposition_target_quote(proposition, slots, premises).is_some()
                    && !proposition_target_anchor_matches(
                        proposition,
                        entry.identity.anchor_id.as_deref(),
                        slots,
                        premises,
                        anchors,
                    )
                {
                    proposition.relation = EvidenceRelation::UnresolvedAttribution;
                    proposition.target_id.clear();
                    demoted += 1;
                }
            }
        }
        demoted
    }

    /// Preserve version-history memories while refusing to treat an older draft
    /// citation as a complete answer to a final/current-version query.
    pub fn qualify_temporal_evidence(&mut self, query: &str, candidates: &[Candidate]) -> usize {
        if self.version != "slot-evidence-v2" && self.version != "slot-evidence-v3" {
            return 0;
        }
        let query = query.to_lowercase();
        let asks_earlier_version = contains_any_phrase(
            &query,
            &[
                "earlier draft",
                "previous draft",
                "borrador anterior",
                "versión anterior",
                "version anterior",
            ],
        );
        let asks_final =
            contains_any_phrase(&query, &["final", "última versión", "ultima version"]);
        let asks_current = contains_any_phrase(
            &query,
            &[
                "current",
                "currently",
                "latest",
                "most recent",
                "actual",
                "actualmente",
                "vigente",
            ],
        );
        if !asks_final && !asks_current {
            return 0;
        }
        let has_final_scoped_slot = self.slots.iter().any(|slot| {
            contains_any_phrase(
                &slot.quote.to_lowercase(),
                &["final", "última versión", "ultima version"],
            )
        });
        let has_current_scoped_slot = self.slots.iter().any(|slot| {
            contains_any_phrase(
                &slot.quote.to_lowercase(),
                &[
                    "current",
                    "currently",
                    "latest",
                    "most recent",
                    "actual",
                    "actualmente",
                    "vigente",
                ],
            )
        });
        let all_temporal_qualifiers_scoped =
            (!asks_final || has_final_scoped_slot) && (!asks_current || has_current_scoped_slot);

        let mut qualified = 0;
        for evidence in &mut self.candidates {
            if !candidates.iter().any(|candidate| {
                candidate.memory_id == evidence.memory_id && candidate.revision == evidence.revision
            }) {
                continue;
            }
            for proposition in &mut evidence.propositions {
                if proposition.relation != EvidenceRelation::Answers {
                    continue;
                }
                let quote = proposition.quote.to_lowercase();
                let quote_is_final = has_final_version_marker(&quote);
                let quote_is_current = has_current_version_marker(&quote);
                let target_quote = self
                    .slots
                    .iter()
                    .find(|slot| slot.id == proposition.target_id)
                    .map(|slot| slot.quote.as_str())
                    .unwrap_or_default();
                let target_quote_lower = target_quote.to_lowercase();
                // If the extracted slot retains a temporal modifier, scope the
                // guard to it. If no slot retains one, keep the global guard as
                // a conservative fallback rather than allowing stale evidence
                // to pass because extraction omitted the modifier.
                let requires_final = asks_final
                    && (self.slots.len() <= 1
                        || !all_temporal_qualifiers_scoped
                        || contains_any_phrase(
                            &target_quote_lower,
                            &["final", "última versión", "ultima version"],
                        ));
                let requires_current = asks_current
                    && (self.slots.len() <= 1
                        || !all_temporal_qualifiers_scoped
                        || contains_any_phrase(
                            &target_quote_lower,
                            &[
                                "current",
                                "currently",
                                "latest",
                                "most recent",
                                "actual",
                                "actualmente",
                                "vigente",
                            ],
                        ));
                let target_asks_earlier = contains_any_phrase(
                    target_quote,
                    &[
                        "earlier draft",
                        "previous draft",
                        "borrador anterior",
                        "versión anterior",
                        "version anterior",
                    ],
                );
                let quote_has_final_target = quote_has_qualified_target(
                    &quote,
                    target_quote,
                    &[
                        "final version",
                        "final note",
                        "final draft",
                        "versión final",
                        "version final",
                        "nota final",
                        "versión definitiva",
                        "version definitiva",
                    ],
                );
                let quote_has_current_target = quote_has_qualified_target(
                    &quote,
                    target_quote,
                    &[
                        "current",
                        "currently",
                        "current version",
                        "current note",
                        "latest version",
                        "most recent version",
                        "actual",
                        "actualmente",
                        "vigente",
                        "versión actual",
                        "version actual",
                        "versión vigente",
                        "version vigente",
                        "nota vigente",
                    ],
                );
                let must_be_partial = !(asks_earlier_version && target_asks_earlier)
                    && ((requires_final && (!quote_is_final || !quote_has_final_target))
                        || (requires_current && (!quote_is_current || !quote_has_current_target)));
                if must_be_partial {
                    proposition.relation = EvidenceRelation::Partial;
                    qualified += 1;
                }
            }
        }
        qualified
    }

    pub fn selected_ids(&self, candidates: &[Candidate], query: &str) -> Vec<String> {
        // Selection is an execution boundary: never let callers select from a trace
        // that fails the same complete, query-bound validation used by the adapter.
        if self.validation_error(query, candidates).is_some() {
            return Vec::new();
        }
        self.candidates
            .iter()
            .filter(|entry| {
                entry.identity.status == IdentityStatus::Match
                    && entry.propositions.iter().any(|proposition| {
                        if !matches!(
                            proposition.relation,
                            EvidenceRelation::Answers
                                | EvidenceRelation::Partial
                                | EvidenceRelation::ExplicitUnmeasured
                                | EvidenceRelation::Refutes
                                | EvidenceRelation::Supports
                        ) {
                            return false;
                        }
                        if self.version == "slot-evidence-v2" || self.version == "slot-evidence-v3"
                        {
                            let Some(anchor) =
                                proposition
                                    .identity_anchor_id
                                    .as_ref()
                                    .and_then(|anchor_id| {
                                        self.query_anchors
                                            .iter()
                                            .find(|anchor| &anchor.id == anchor_id)
                                    })
                            else {
                                return false;
                            };
                            contains_identifier(&anchor.stable_id, &proposition.quote)
                                && proposition.quote.contains(&proposition.subject_quote)
                                && contains_identifier(
                                    &anchor.stable_id,
                                    &proposition.subject_quote,
                                )
                                && subject_quote_matches_anchor(
                                    &proposition.subject_quote,
                                    anchor,
                                    &self.query_anchors,
                                )
                                && proposition_target_anchor_matches(
                                    proposition,
                                    entry.identity.anchor_id.as_deref(),
                                    &self.slots,
                                    &self.premises,
                                    &self.query_anchors,
                                )
                                && if self.version == "slot-evidence-v3" {
                                    !has_foreign_subject_before_predicate(
                                        &proposition.quote,
                                        &anchor.stable_id,
                                        &proposition.predicate_quote,
                                    )
                                } else {
                                    !has_foreign_relative_clause(
                                        proposition_subject_scope(
                                            &proposition.quote,
                                            &proposition.subject_quote,
                                        ),
                                        &anchor.stable_id,
                                    )
                                }
                        } else {
                            entry.identity.stable_id_quote.as_ref().is_some_and(|id| {
                                contains_identifier(id, &proposition.quote)
                                    && proposition.quote.contains(&proposition.subject_quote)
                                    && contains_identifier(id, &proposition.subject_quote)
                                    && entry.identity.subject_quote.as_ref()
                                        == Some(&proposition.subject_quote)
                            })
                        }
                    })
            })
            .filter(|entry| {
                candidates.iter().any(|candidate| {
                    candidate.memory_id == entry.memory_id && candidate.revision == entry.revision
                })
            })
            .map(|entry| entry.memory_id.clone())
            .collect()
    }

    /// Validate the bounded roster and literal citations. This is not semantic proof.
    pub fn validate(&self, query: &str, candidates: &[Candidate]) -> bool {
        self.validation_error(query, candidates).is_none()
    }

    /// Return a stable, non-content-bearing reason when a trace fails local checks.
    pub fn validation_error(&self, query: &str, candidates: &[Candidate]) -> Option<&'static str> {
        if self.version != "slot-evidence-v1"
            && self.version != "slot-evidence-v2"
            && self.version != "slot-evidence-v3"
        {
            return Some("trace_version_mismatch");
        }
        if candidates.len() > 20
            || self.slots.len() > 16
            || self.premises.len() > 16
            || self.query_anchors.len() > 16
        {
            return Some("trace_bounds_exceeded");
        }
        if self.candidates.len() != candidates.len() {
            return Some(if self.candidates.len() < candidates.len() {
                "trace_candidate_roster_missing"
            } else {
                "trace_candidate_roster_extra"
            });
        }
        if candidates
            .iter()
            .map(|candidate| &candidate.memory_id)
            .collect::<std::collections::HashSet<_>>()
            .len()
            != candidates.len()
        {
            return Some("duplicate_input_candidate_id");
        }
        let unique = |values: &[String]| {
            values.iter().all(|value| !value.is_empty())
                && values
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    == values.len()
        };
        if !unique(
            &self
                .query_anchors
                .iter()
                .map(|x| x.id.clone())
                .collect::<Vec<_>>(),
        ) {
            return Some("query_anchor_ids_invalid");
        }
        if !unique(&self.slots.iter().map(|x| x.id.clone()).collect::<Vec<_>>()) {
            return Some("query_slot_ids_invalid");
        }
        if !unique(
            &self
                .premises
                .iter()
                .map(|x| x.id.clone())
                .collect::<Vec<_>>(),
        ) {
            return Some("query_premise_ids_invalid");
        }
        for anchor in &self.query_anchors {
            if anchor.stable_id.is_empty() || anchor.kind.is_empty() {
                return Some("query_anchor_identity_fields_missing");
            }
            if !query.contains(&anchor.quote) {
                return Some("query_anchor_quote_not_literal");
            }
            if !contains_identifier(&anchor.stable_id, &anchor.quote) {
                return Some("query_anchor_quote_omits_stable_id");
            }
            if !anchor.quote.contains(&anchor.kind) {
                return Some("query_anchor_quote_omits_kind");
            }
        }
        if self
            .slots
            .iter()
            .chain(&self.premises)
            .any(|x| x.quote.is_empty() || !query.contains(&x.quote))
        {
            return Some("query_target_quote_not_literal");
        }
        for (entry, candidate) in self.candidates.iter().zip(candidates) {
            if entry.memory_id != candidate.memory_id
                || entry.revision != candidate.revision
                || entry.propositions.len() > 4
                || entry
                    .propositions
                    .iter()
                    .map(|p| &p.occurrence_id)
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    != entry.propositions.len()
            {
                return Some("candidate_identity_revision_or_proposition_bounds");
            }
            let summary = &candidate.summary;
            let identity_valid = match entry.identity.status {
                IdentityStatus::Match => {
                    let (Some(anchor_id), Some(id_quote), Some(subject_quote)) = (
                        entry.identity.anchor_id.as_ref(),
                        entry.identity.stable_id_quote.as_ref(),
                        entry.identity.subject_quote.as_ref(),
                    ) else {
                        return Some("matched_identity_fields_missing");
                    };
                    let Some(anchor) = self.query_anchors.iter().find(|x| &x.id == anchor_id)
                    else {
                        return Some("matched_identity_anchor_missing");
                    };
                    if anchor.stable_id != *id_quote {
                        return Some("matched_identity_id_differs_from_query_anchor");
                    }
                    if !contains_identifier(id_quote, summary) {
                        return Some("matched_identity_id_absent_from_memory");
                    }
                    if !contains_identifier(id_quote, subject_quote) {
                        return Some("matched_identity_subject_omits_id");
                    }
                    if !subject_quote_matches_anchor(subject_quote, anchor, &self.query_anchors) {
                        return Some("matched_identity_subject_not_bound_to_query");
                    }
                    if !contains_domain_type(&anchor.kind, summary) {
                        return Some("matched_identity_domain_type_absent_from_memory");
                    }
                    if !summary.contains(subject_quote) {
                        return Some("matched_identity_subject_absent_from_memory");
                    }
                    true
                }
                IdentityStatus::Incompatible | IdentityStatus::Unresolved => {
                    if entry.identity.anchor_id.is_some()
                        || entry.identity.stable_id_quote.is_some()
                        || entry.identity.subject_quote.is_some()
                    {
                        return Some("unresolved_identity_contains_match_only_fields");
                    }
                    true
                }
            };
            if !identity_valid {
                return Some("identity_binding_or_stable_id_citation");
            }
            for proposition in &entry.propositions {
                let is_v2 =
                    self.version == "slot-evidence-v2" || self.version == "slot-evidence-v3";
                let is_v3 = self.version == "slot-evidence-v3";
                let actionable = matches!(
                    proposition.relation,
                    EvidenceRelation::Answers
                        | EvidenceRelation::Partial
                        | EvidenceRelation::ExplicitUnmeasured
                        | EvidenceRelation::Refutes
                        | EvidenceRelation::Supports
                );
                let source_quote_invalid = proposition.quote.is_empty()
                    || proposition.quote.len() > 512
                    || !summary.contains(&proposition.quote);
                let proposition_anchor_invalid = proposition
                    .identity_anchor_id
                    .as_ref()
                    .is_some_and(|id| !self.query_anchors.iter().any(|anchor| &anchor.id == id));
                let legacy_quote_invalid = !is_v2
                    && (proposition.occurrence_id.is_empty()
                        || proposition.subject_quote.is_empty()
                        || proposition.predicate_quote.is_empty()
                        || !proposition.quote.contains(&proposition.subject_quote)
                        || !proposition.quote.contains(&proposition.predicate_quote)
                        || proposition.subject_quote.len() > 512
                        || proposition.predicate_quote.len() > 512
                        || proposition.value.as_ref().is_some_and(|v| v.len() > 512)
                        || !optional_quote_in(
                            &proposition.quote,
                            proposition.polarity_quote.as_ref(),
                        )
                        || !optional_quote_in(
                            &proposition.quote,
                            proposition.modality_quote.as_ref(),
                        )
                        || !optional_quote_in(
                            &proposition.quote,
                            proposition.event_quote.as_ref(),
                        )
                        || !optional_quote_in(&proposition.quote, proposition.time_quote.as_ref())
                        || !optional_quote_in(
                            &proposition.quote,
                            proposition.conditions_quote.as_ref(),
                        ));
                let v3_predicate_invalid = is_v3
                    && actionable
                    && (proposition.predicate_quote.is_empty()
                        || proposition.predicate_quote.len() > 512
                        || proposition
                            .quote
                            .match_indices(&proposition.predicate_quote)
                            .count()
                            != 1
                        || proposition
                            .quote
                            .find(&proposition.predicate_quote)
                            .zip(proposition.quote.find(&proposition.subject_quote))
                            .is_none_or(|(predicate, subject)| predicate <= subject));
                let v2_subject_invalid = is_v2
                    && actionable
                    && proposition
                        .identity_anchor_id
                        .as_ref()
                        .and_then(|anchor_id| {
                            self.query_anchors
                                .iter()
                                .find(|anchor| &anchor.id == anchor_id)
                        })
                        .is_none_or(|anchor| {
                            proposition.subject_quote.is_empty()
                                || !proposition.quote.contains(&proposition.subject_quote)
                                || !subject_quote_matches_anchor(
                                    &proposition.subject_quote,
                                    anchor,
                                    &self.query_anchors,
                                )
                        });
                if source_quote_invalid {
                    return Some("proposition_source_quote_invalid");
                }
                if proposition_anchor_invalid {
                    return Some("proposition_anchor_id_invalid");
                }
                if v2_subject_invalid {
                    return Some("proposition_subject_mismatch");
                }
                if legacy_quote_invalid {
                    return Some("proposition_citation_or_quote_bounds");
                }
                if v3_predicate_invalid {
                    return Some("proposition_predicate_quote_invalid");
                }
                if self.version == "slot-evidence-v2"
                    && actionable
                    && proposition
                        .identity_anchor_id
                        .as_ref()
                        .and_then(|anchor_id| {
                            self.query_anchors
                                .iter()
                                .find(|anchor| &anchor.id == anchor_id)
                        })
                        .is_some_and(|anchor| {
                            has_foreign_relative_clause(
                                proposition_subject_scope(
                                    &proposition.quote,
                                    &proposition.subject_quote,
                                ),
                                &anchor.stable_id,
                            )
                        })
                {
                    return Some("proposition_foreign_relative_clause");
                }
                if is_v3
                    && actionable
                    && proposition
                        .identity_anchor_id
                        .as_ref()
                        .and_then(|anchor_id| {
                            self.query_anchors
                                .iter()
                                .find(|anchor| &anchor.id == anchor_id)
                        })
                        .is_some_and(|anchor| {
                            has_foreign_subject_before_predicate(
                                &proposition.quote,
                                &anchor.stable_id,
                                &proposition.predicate_quote,
                            )
                        })
                {
                    return Some("proposition_foreign_relative_clause");
                }
                let v2_binding_invalid = is_v2
                    && actionable
                    && (entry.identity.status != IdentityStatus::Match
                        || (proposition_target_quote(proposition, &self.slots, &self.premises)
                            .is_some()
                            && !proposition_target_anchor_matches(
                                proposition,
                                entry.identity.anchor_id.as_deref(),
                                &self.slots,
                                &self.premises,
                                &self.query_anchors,
                            )));
                if v2_binding_invalid {
                    return Some("v2_proposition_binding_invalid");
                }
                let target_reference_error = match proposition.relation {
                    EvidenceRelation::Answers
                    | EvidenceRelation::Partial
                    | EvidenceRelation::ExplicitUnmeasured => {
                        if self
                            .slots
                            .iter()
                            .any(|target| target.id == proposition.target_id)
                        {
                            None
                        } else if self
                            .premises
                            .iter()
                            .any(|target| target.id == proposition.target_id)
                        {
                            Some("proposition_target_relation_mismatch")
                        } else {
                            Some("proposition_target_unknown")
                        }
                    }
                    EvidenceRelation::Refutes | EvidenceRelation::Supports => {
                        if self
                            .premises
                            .iter()
                            .any(|target| target.id == proposition.target_id)
                        {
                            None
                        } else if self
                            .slots
                            .iter()
                            .any(|target| target.id == proposition.target_id)
                        {
                            Some("proposition_target_relation_mismatch")
                        } else {
                            Some("proposition_target_unknown")
                        }
                    }
                    EvidenceRelation::ContextOnly
                    | EvidenceRelation::Unrelated
                    | EvidenceRelation::Unknown
                    | EvidenceRelation::UnresolvedAttribution => {
                        if proposition.target_id.is_empty() {
                            None
                        } else {
                            Some("proposition_non_actionable_target_not_empty")
                        }
                    }
                };
                if let Some(error) = target_reference_error {
                    return Some(error);
                }
            }
        }
        None
    }
}

/// Detect a different explicitly named subject in a clause introduced by a
/// reporting verb. This intentionally fails closed only for clear proper-name
/// starts; an embedded subject that cannot be confidently matched is demoted.
fn has_foreign_named_report_subject(quote: &str, bound_subject: &str) -> bool {
    const MARKERS: &[&str] = &[
        "reported that",
        "said that",
        "stated that",
        "claimed that",
        "reportó que",
        "informó que",
        "dijo que",
    ];
    let bound = normalized_identity_words(bound_subject);
    if bound.is_empty() {
        return false;
    }
    let lowercase_quote = quote.to_lowercase();
    MARKERS.iter().any(|marker| {
        lowercase_quote.match_indices(marker).any(|(start, found)| {
            let lowercase_end = start + found.len();
            let Some(original_end) =
                original_byte_offset_after_lowercase_byte_offset(quote, lowercase_end)
            else {
                return false;
            };
            let Some(rest) = quote.get(original_end..) else {
                return false;
            };
            let tokens = rest
                .split(|character: char| !character.is_alphanumeric())
                .filter(|token| !token.is_empty());
            let tokens: Vec<&str> = tokens.collect();
            let mut name = Vec::new();
            let mut predicate_start = None;
            for (index, token) in tokens.iter().enumerate() {
                let normalized = normalized_identity_words(token);
                if normalized.len() != 1 {
                    continue;
                }
                let word = &normalized[0];
                if name.is_empty()
                    && ["someone", "somebody", "anyone", "anybody", "some"].contains(&word.as_str())
                {
                    return true;
                }
                if name.is_empty()
                    && [
                        "another",
                        "other",
                        "different",
                        "second",
                        "otro",
                        "otra",
                        "distinto",
                        "distinta",
                    ]
                    .contains(&word.as_str())
                {
                    return true;
                }
                if name.is_empty()
                    && [
                        "the",
                        "a",
                        "an",
                        "employee",
                        "engineer",
                        "contractor",
                        "well",
                        "known",
                        "recently",
                        "hired",
                        "newly",
                        "appointed",
                        "researcher",
                        "manager",
                        "colleague",
                        "person",
                        "individual",
                        "named",
                        "called",
                        "la",
                        "el",
                        "los",
                        "las",
                        "un",
                        "una",
                        "unos",
                        "unas",
                        "same",
                        "sr",
                        "sra",
                        "mr",
                        "mrs",
                        "ms",
                        "miss",
                        "dr",
                        "dra",
                        "prof",
                        "professor",
                    ]
                    .contains(&word.as_str())
                {
                    continue;
                }
                let Some(initial) = token.chars().next() else {
                    continue;
                };
                if initial.is_uppercase() {
                    name.push(word.clone());
                } else {
                    if word == "s" && name == bound {
                        // Apostrophe splitting turns a possessive such as
                        // "Ana Ruiz's colleague Bob" into the tokens "Ana
                        // Ruiz s colleague Bob". The embedded fact belongs to
                        // the possessed noun phrase, not necessarily Ana.
                        return true;
                    }
                    // Keep common lowercase particles inside surnames (for
                    // example, "Ana de la Cruz") so a shortened prefix cannot
                    // accidentally bind a different full name.
                    let is_name_particle = [
                        "al", "abu", "bin", "binti", "da", "das", "de", "del", "der", "di", "dos",
                        "du", "e", "ibn", "la", "las", "le", "van", "von", "y",
                    ]
                    .contains(&word.as_str());
                    let next_is_capitalized = tokens
                        .get(index + 1)
                        .and_then(|next| next.chars().next())
                        .is_some_and(char::is_uppercase);
                    let next_two_is_connector_then_capitalized = tokens
                        .get(index + 1)
                        .map(|next| normalized_identity_words(next))
                        .and_then(|next| next.first().cloned())
                        .is_some_and(|next| {
                            ["la", "las", "los", "del", "der", "van"].contains(&next.as_str())
                                && tokens
                                    .get(index + 2)
                                    .and_then(|last| last.chars().next())
                                    .is_some_and(char::is_uppercase)
                        });
                    if !name.is_empty()
                        && is_name_particle
                        && (next_is_capitalized || next_two_is_connector_then_capitalized)
                    {
                        name.push(word.clone());
                        continue;
                    }
                    if name.is_empty() {
                        // A reporting clause whose subject is a pronoun, a
                        // generic noun, or an unrecognized construction cannot
                        // be safely bound to the queried person from this trace.
                        return true;
                    }
                    // A lowercase token after a collected name begins the
                    // predicate; compare the full name collected so far.
                    predicate_start = Some(word.clone());
                    break;
                }
            }
            let includes_bound_in_spanish_coordination = name.len() > bound.len()
                && name.starts_with(&bound)
                && ["y", "e"].contains(&name.get(bound.len()).map(String::as_str).unwrap_or(""))
                && predicate_start.is_some_and(|predicate| {
                    [
                        "viven",
                        "trabajan",
                        "estudian",
                        "residen",
                        "son",
                        "están",
                        "estan",
                        "tienen",
                        "fueron",
                        "estuvieron",
                    ]
                    .contains(&predicate.as_str())
                });
            if includes_bound_in_spanish_coordination {
                return false;
            }
            !name.is_empty() && name != bound
        })
    })
}

fn original_byte_offset_after_lowercase_byte_offset(
    text: &str,
    lowercase_offset: usize,
) -> Option<usize> {
    let mut folded_offset = 0;
    for (original_offset, character) in text.char_indices() {
        if folded_offset == lowercase_offset {
            return Some(original_offset);
        }
        folded_offset += character.to_lowercase().map(char::len_utf8).sum::<usize>();
        if folded_offset > lowercase_offset {
            return None;
        }
    }
    (folded_offset == lowercase_offset).then_some(text.len())
}

fn query_requests_reported_content(query: &str, bound_subject: &str, anchor_kind: &str) -> bool {
    let lowercase_query = query.to_lowercase();
    let prefixes = [
        "what did",
        "which city did",
        "who did",
        "which person did",
        "what has",
        "what does",
        "who does",
        "who has",
        "qué dijo",
        "quién dijo",
        "qué ha dicho",
        "qué reportó",
        "qué informó",
    ];
    prefixes.iter().any(|prefix| {
        let Some(start) = lowercase_query.find(prefix) else {
            return false;
        };
        let lowercase_context_start = start + prefix.len();
        let Some(context_start) =
            original_byte_offset_after_lowercase_byte_offset(query, lowercase_context_start)
        else {
            return false;
        };
        let Some(context) = query.get(context_start..) else {
            return false;
        };
        if !subject_prefix_is_direct_question_subject(context, bound_subject, anchor_kind) {
            return false;
        }
        [
            "report", "say", "says", "said", "tell", "told", "dijo", "dicho", "reportó", "informó",
        ]
        .iter()
        .any(|verb| lowercase_query[start..].contains(verb))
    })
}

fn has_speech_verb(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    ["tell", "told", "ask", "asked", "dijo", "dicho"]
        .iter()
        .any(|verb| contains_identifier(verb, &lower))
}

/// Detect explicit recipient exclusions independently of relative-clause
/// classification. Otherwise an intervening temporal aside can hide the
/// complement marker from the ambiguity detector and bypass the exclusion.
#[cfg(test)]
fn explicitly_excluded_speech_recipient(text: &str, bound_id: &str) -> bool {
    stable_ids_in(text)
        .into_iter()
        .any(|recipient_id| explicitly_excluded_recipient_id(text, bound_id, recipient_id))
}

/// Scope a detected exclusion to the exact ID the question asks about as a
/// recipient. An exclusion of a third person must not erase an answer about
/// the queried recipient.
fn explicitly_excluded_speech_recipient_for_query(
    text: &str,
    bound_id: &str,
    query: &str,
    subject_quote: &str,
    anchor_kind: &str,
) -> bool {
    stable_ids_in(text).into_iter().any(|recipient_id| {
        query_requests_speech_recipient(query, bound_id, recipient_id, subject_quote, anchor_kind)
            && explicitly_excluded_recipient_id(text, bound_id, recipient_id)
    })
}

fn stable_ids_in(text: &str) -> Vec<&str> {
    text.split(|character: char| {
        !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
    })
    .filter(|token| {
        token.chars().any(char::is_alphabetic)
            && token.chars().any(char::is_numeric)
            && token.contains(['-', '_', '.', '/'])
    })
    .collect()
}

fn explicitly_excluded_recipient_id(text: &str, bound_id: &str, recipient_id: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let (Some(bound_start), Some(recipient_start)) = (
        lower.find(&bound_id.to_ascii_lowercase()),
        lower.find(&recipient_id.to_ascii_lowercase()),
    ) else {
        return false;
    };
    let bound_end = bound_start + bound_id.len();
    if recipient_start <= bound_end {
        return false;
    }
    let speech_verbs = ["asked", "told", "dijo", "dicho"];
    let Some((_, verb_end)) = speech_verbs.iter().find_map(|verb| {
        lower[bound_end..recipient_start]
            .match_indices(verb)
            .find_map(|(offset, found)| {
                let start = bound_end + offset;
                let before_boundary = lower[..start]
                    .chars()
                    .next_back()
                    .is_none_or(|character| !character.is_alphanumeric());
                let end = start + found.len();
                let after_boundary = lower[end..]
                    .chars()
                    .next()
                    .is_none_or(|character| !character.is_alphanumeric());
                (before_boundary && after_boundary).then_some((start, end))
            })
    }) else {
        return false;
    };
    let between = &lower[verb_end..recipient_start];
    let markers = [
        "other than",
        "but not",
        "not",
        "no",
        "sino",
        "instead",
        "except",
        "excepto",
        "salvo",
        "menos",
        "excluding",
        "excepting",
    ];
    markers.iter().any(|marker| {
        let Some(marker_start) = find_word_marker(between, marker) else {
            return false;
        };
        let prefix = between[..marker_start].trim_end();
        if !prefix
            .chars()
            .next_back()
            .is_some_and(|character| matches!(character, ',' | '(' | '—' | '–' | '-'))
        {
            return false;
        }
        let name_tokens = between[marker_start + marker.len()..]
            .split(|character: char| !character.is_alphanumeric())
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>();
        !(*marker == "no"
            && (name_tokens.starts_with(&["later", "than"])
                || name_tokens.starts_with(&["more", "than"])))
            && !(*marker == "instead" && name_tokens.first() == Some(&"of"))
            && name_tokens.len() <= 5
            && !name_tokens.iter().any(|token| {
                [
                    "to",
                    "because",
                    "that",
                    "which",
                    "only",
                    "just",
                    "but",
                    "merely",
                    "simply",
                    "solely",
                    "tell",
                    "invite",
                    "ask",
                    "decir",
                    "invitar",
                    "porque",
                    "para",
                    "solo",
                    "solamente",
                    "when",
                    "if",
                    "unless",
                    "cuando",
                    "que",
                ]
                .contains(token)
            })
    })
}

fn query_requests_speech_recipient(
    query: &str,
    bound_id: &str,
    recipient_id: &str,
    subject_quote: &str,
    anchor_kind: &str,
) -> bool {
    let query_lower = query.to_lowercase();
    let spanish_query = query_lower.contains("qué") && query_lower.contains("dijo");
    let query_subject_context = if spanish_query {
        let Some(start) = query_lower.find("dijo").map(|index| index + "dijo".len()) else {
            return false;
        };
        &query[start..]
    } else {
        let Some(start) = query_lower
            .find("what did")
            .map(|index| index + "what did".len())
        else {
            return false;
        };
        &query[start..]
    };
    let speech_verbs = if spanish_query {
        &["dijo", "dicho"][..]
    } else {
        &[
            "ask",
            "asked",
            "tell",
            "told",
            "inform",
            "informed",
            "notify",
            "notified",
            "warn",
            "warned",
            "explain",
            "explained",
            "report",
            "reported",
            "advise",
            "advised",
        ][..]
    };
    subject_prefix_is_direct_question_subject(query_subject_context, subject_quote, anchor_kind)
        && contains_identifier(bound_id, query)
        && contains_identifier(recipient_id, query)
        && has_ordered_recipient_speech_act(query, bound_id, recipient_id, speech_verbs)
}

/// Confirm the bound person is the direct question's subject. Unlike the
/// clause-local check used for source quotes, punctuation cannot restart this
/// prefix: a parenthetical after another subject must not turn a nested
/// request into a direct question about the bound person.
fn subject_prefix_is_direct_question_subject(text: &str, subject_quote: &str, kind: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    let subject_quote = subject_quote.to_ascii_lowercase();
    let Some(subject_start) = normalized.find(&subject_quote) else {
        return false;
    };
    let prefix = &normalized[..subject_start];
    if !prefix
        .chars()
        .all(|character| character.is_alphanumeric() || character.is_whitespace())
    {
        return false;
    }
    let allowed_kind = kind.to_ascii_lowercase();
    prefix
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .all(|token| {
            token == allowed_kind
                || [
                    "the", "a", "an", "la", "el", "los", "las", "un", "una", "unos", "unas", "my",
                    "your", "his", "her", "our", "their", "mi", "mis", "tu", "tus", "su", "sus",
                    "nuestro", "nuestra", "nuestros", "nuestras",
                ]
                .contains(&token)
        })
}

fn find_word_marker(text: &str, marker: &str) -> Option<usize> {
    text.match_indices(marker).find_map(|(start, found)| {
        let end = start + found.len();
        let before_is_boundary = text[..start]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric());
        let after_is_boundary = text[end..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric());
        (before_is_boundary && after_is_boundary).then_some(start)
    })
}

fn optional_quote_in(text: &str, quote: Option<&String>) -> bool {
    quote.is_none_or(|value| value.is_empty() || text.contains(value))
}

/// Match a complete identifier token, so `E-17` cannot bind `E-170` or `XE-17`.
fn contains_identifier(identifier: &str, text: &str) -> bool {
    if identifier.is_empty() {
        return false;
    }
    text.match_indices(identifier).any(|(start, _)| {
        let before_is_identifier = text[..start]
            .chars()
            .next_back()
            .is_some_and(is_identifier_char);
        let end = start + identifier.len();
        let after_is_identifier = text[end..].chars().next().is_some_and(is_identifier_char);
        !before_is_identifier && !after_is_identifier
    })
}

fn subject_quote_matches_anchor(
    quote: &str,
    anchor: &QueryAnchor,
    anchors: &[QueryAnchor],
) -> bool {
    if anchor
        .subject
        .as_deref()
        .is_some_and(|subject| literal_subject_occurrences(subject, quote) != 1)
        || anchor.subject.as_deref() == Some("")
    {
        return false;
    }
    // Anchor quotes are emitted as a compact identity phrase. Remove one declared kind
    // phrase and the stable ID, preserving the remaining literal name tokens. Removing
    // only one kind occurrence also supports names such as "Engineer" when kind is
    // "engineer". Kinds may contain multiple words.
    let anchor_tokens = normalized_identity_words(&anchor.quote);
    let kind_tokens = normalized_identity_words(&anchor.kind);
    let stable_id_tokens = normalized_identity_words(&anchor.stable_id);
    if kind_tokens.is_empty() || stable_id_tokens.len() != 1 {
        return false;
    }
    let kind_positions: Vec<_> = anchor_tokens
        .windows(kind_tokens.len())
        .enumerate()
        .filter_map(|(index, window)| (window == kind_tokens.as_slice()).then_some(index))
        .collect();
    let id_positions_in_anchor: Vec<_> = anchor_tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| (token == &stable_id_tokens[0]).then_some(index))
        .collect();
    if kind_positions.is_empty() || id_positions_in_anchor.len() != 1 {
        return false;
    }
    let id_position_in_anchor = id_positions_in_anchor[0];
    let quote_tokens = normalized_identity_words(quote);
    let id_positions: Vec<_> = quote_tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| (token == &stable_id_tokens[0]).then_some(index))
        .collect();
    if id_positions.len() != 1 {
        return false;
    }
    // A name may itself contain the declared kind. Try each contiguous kind phrase
    // in the anchor, but fail closed if those removals produce distinct subject
    // readings: the compact trace does not retain a separate query-subject field
    // that could disambiguate them.
    let mut subject_candidates: Vec<Vec<String>> = kind_positions
        .iter()
        .filter_map(|kind_start| {
            let subject_tokens: Vec<_> = anchor_tokens
                .iter()
                .enumerate()
                .filter_map(|(index, token)| {
                    (!(*kind_start..*kind_start + kind_tokens.len()).contains(&index)
                        && index != id_position_in_anchor)
                        .then_some(token.clone())
                })
                .collect();
            (!subject_tokens.is_empty()).then_some(subject_tokens)
        })
        .collect();
    subject_candidates.sort();
    subject_candidates.dedup();
    let [subject_tokens] = subject_candidates.as_slice() else {
        return false;
    };
    let subject_positions: Vec<_> = quote_tokens
        .windows(subject_tokens.len())
        .enumerate()
        .filter_map(|(index, window)| (window == subject_tokens.as_slice()).then_some(index))
        .collect();
    if subject_positions.len() != 1 {
        return false;
    }
    let subject_end = subject_positions[0] + subject_tokens.len();
    // Permit the complete declared kind phrase between name and ID for legacy
    // compact anchors such as "Ana Ruiz project manager E-17".
    let directly_bound = id_positions[0] == subject_end
        || (id_positions[0] == subject_end + kind_tokens.len()
            && quote_tokens[subject_end..id_positions[0]] == kind_tokens);
    directly_bound
        // Bind the identifier directly to this name. Merely mentioning the name and
        // identifier somewhere in a sentence can attach another person's ID to it.
        && anchors.iter().all(|other| {
            other.id == anchor.id || identifier_occurrences(&other.stable_id, quote) == 0
        })
}

fn proposition_subject_scope<'a>(quote: &'a str, subject_quote: &str) -> &'a str {
    let Some(start) = quote.find(subject_quote) else {
        return quote;
    };
    let scope = &quote[start..];
    for (offset, character) in scope.char_indices() {
        if !matches!(character, '.' | '?' | '!') {
            continue;
        }
        let punctuation_end = offset + character.len_utf8();
        let suffix = &scope[punctuation_end..];
        let next = suffix.trim_start().chars().next();
        let abbreviation = scope[..offset]
            .split_whitespace()
            .next_back()
            .is_some_and(|word| {
                [
                    "dr", "mr", "mrs", "ms", "prof", "sr", "sra", "srta", "st", "vs",
                ]
                .iter()
                .any(|known| {
                    word.trim_matches(|c: char| !c.is_alphabetic())
                        .eq_ignore_ascii_case(known)
                })
            });
        let decimal_or_dotted_id = character == '.'
            && suffix
                .chars()
                .next()
                .is_some_and(|next_character| next_character.is_ascii_digit());
        let sentence_end = character != '.'
            || next.is_none()
            || next.is_some_and(char::is_uppercase)
            || next.is_some_and(|value| matches!(value, '¿' | '¡'));
        if !abbreviation && !decimal_or_dotted_id && sentence_end {
            return &scope[..punctuation_end];
        }
    }
    scope
}

fn proposition_subject_clause_scope(scope: &str) -> &str {
    const CLAUSE_BOUNDARIES: &[&str] = &[
        ", while ",
        ", whereas ",
        ", but ",
        ", although ",
        ", however ",
        ", mientras ",
        ", pero ",
        ", aunque ",
        " and ",
        " y ",
        ";",
    ];
    let lowercase = scope.to_lowercase();
    let first_bound_identifier_end = scope
        .split(|character: char| {
            !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
        })
        .filter(|token| {
            token.chars().any(char::is_alphabetic)
                && token.chars().any(char::is_numeric)
                && token.contains(['-', '_', '.', '/'])
        })
        .find_map(|token| scope.find(token).map(|start| start + token.len()));
    let boundary = CLAUSE_BOUNDARIES
        .iter()
        .filter_map(|marker| {
            lowercase.find(marker).filter(|start| {
                if *marker == " and " {
                    first_bound_identifier_end.is_none_or(|id_end| *start > id_end)
                } else if *marker == " y " {
                    let continuation = lowercase[*start + marker.len()..].trim_start();
                    first_bound_identifier_end.is_none_or(|id_end| *start > id_end)
                        && [
                            "employee ",
                            "contractor ",
                            "empleado ",
                            "empleada ",
                            "contratista ",
                            "el empleado ",
                            "la empleada ",
                            "el contratista ",
                        ]
                        .iter()
                        .any(|prefix| continuation.starts_with(prefix))
                } else {
                    true
                }
            })
        })
        .min();
    boundary.map_or(scope, |end| &scope[..end])
}

/// Resolve the unique query anchor expressed by a literal proposition subject.
/// Returning `None` means the quote is ambiguous or does not match an anchor.
pub fn query_anchor_id_for_subject_quote(
    subject_quote: &str,
    anchors: &[QueryAnchor],
) -> Option<String> {
    let mut matching = anchors
        .iter()
        .filter(|anchor| subject_quote_matches_anchor(subject_quote, anchor, anchors));
    let id = matching.next()?.id.clone();
    matching.next().is_none().then_some(id)
}

fn literal_subject_occurrences(subject: &str, text: &str) -> usize {
    text.match_indices(subject)
        .filter(|(start, matched)| {
            let before_is_identifier = text[..*start]
                .chars()
                .next_back()
                .is_some_and(is_identifier_char);
            let end = *start + matched.len();
            let after_is_identifier = text[end..].chars().next().is_some_and(is_identifier_char);
            !before_is_identifier && !after_is_identifier
        })
        .count()
}

fn quote_mentions_anchor(quote: &str, anchor: &QueryAnchor) -> bool {
    anchor
        .subject
        .as_deref()
        .is_some_and(|subject| literal_subject_occurrences(subject, quote) > 0)
        || contains_identifier(&anchor.stable_id, quote)
}

fn proposition_target_anchor_matches(
    proposition: &Proposition,
    candidate_anchor_id: Option<&str>,
    slots: &[QueryTarget],
    premises: &[QueryTarget],
    anchors: &[QueryAnchor],
) -> bool {
    let Some(proposition_anchor_id) = proposition.identity_anchor_id.as_deref() else {
        return false;
    };
    let Some(target_quote) = proposition_target_quote(proposition, slots, premises) else {
        return false;
    };
    let target_anchors: Vec<_> = anchors
        .iter()
        .filter(|anchor| quote_mentions_anchor(target_quote, anchor))
        .collect();
    if matches!(
        proposition.relation,
        EvidenceRelation::Refutes | EvidenceRelation::Supports
    ) && candidate_anchor_id == Some(proposition_anchor_id)
    {
        // A premise can be a relation between the candidate's subject and a
        // separately named participant, even when the exact premise excerpt
        // names only that other participant.
        return true;
    }
    if target_anchors.is_empty() {
        return candidate_anchor_id == Some(proposition_anchor_id);
    }
    // An excerpt naming multiple query anchors does not identify which one
    // owns the requested fact. Do not let an incidental entity in a temporal
    // or relational clause authorize a proposition about that entity. A
    // premise can still bind to the candidate's source subject above because
    // premises may describe relations.
    target_anchors.len() == 1 && target_anchors[0].id == proposition_anchor_id
}

fn proposition_target_quote<'a>(
    proposition: &Proposition,
    slots: &'a [QueryTarget],
    premises: &'a [QueryTarget],
) -> Option<&'a str> {
    match proposition.relation {
        EvidenceRelation::Answers
        | EvidenceRelation::Partial
        | EvidenceRelation::ExplicitUnmeasured => slots
            .iter()
            .find(|target| target.id == proposition.target_id)
            .map(|target| target.quote.as_str()),
        EvidenceRelation::Refutes | EvidenceRelation::Supports => premises
            .iter()
            .find(|target| target.id == proposition.target_id)
            .map(|target| target.quote.as_str()),
        _ => None,
    }
}

fn normalized_identity_words(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|character: char| {
                !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
            })
            .chars()
            .flat_map(char::to_lowercase)
            .collect()
        })
        .filter(|word: &String| !word.is_empty())
        .collect()
}

fn contains_any_phrase(text: &str, phrases: &[&str]) -> bool {
    let words = normalized_identity_words(text);
    phrases.iter().any(|phrase| {
        let phrase_words = normalized_identity_words(phrase);
        !phrase_words.is_empty()
            && words
                .windows(phrase_words.len())
                .any(|window| window == phrase_words)
    })
}

fn has_final_version_marker(text: &str) -> bool {
    contains_any_unnegated_phrase(
        text,
        &[
            "final version",
            "final note",
            "final draft",
            "versión final",
            "version final",
            "nota final",
            "versión definitiva",
            "version definitiva",
        ],
        &[
            "not",
            "not the",
            "not a",
            "was not",
            "was not the",
            "was never",
            "is not",
            "is not the",
            "no era la",
            "no fue la",
            "no es la",
            "no era una",
        ],
    )
}

fn has_older_version_marker(text: &str) -> bool {
    contains_any_phrase(
        text,
        &[
            "earlier draft",
            "previous draft",
            "prior draft",
            "superseded draft",
            "borrador anterior",
            "borrador previo",
            "versión anterior",
            "version anterior",
        ],
    )
}

fn contains_any_unnegated_phrase(text: &str, phrases: &[&str], negations: &[&str]) -> bool {
    let words = normalized_identity_words(text);
    let negation_words = negations
        .iter()
        .map(|phrase| normalized_identity_words(phrase))
        .collect::<Vec<_>>();
    phrases.iter().any(|phrase| {
        let phrase_words = normalized_identity_words(phrase);
        !phrase_words.is_empty()
            && words
                .windows(phrase_words.len())
                .enumerate()
                .any(|(start, window)| {
                    window == phrase_words
                        && !negation_words.iter().any(|negation| {
                            negation.len() <= start
                                && words[start - negation.len()..start] == *negation
                        })
                })
    })
}

fn quote_has_qualified_target(quote: &str, target: &str, qualifiers: &[&str]) -> bool {
    let target_terms = normalized_identity_words(target)
        .into_iter()
        .filter(|term| {
            !matches!(
                term.as_str(),
                "a" | "an"
                    | "and"
                    | "as"
                    | "at"
                    | "current"
                    | "currently"
                    | "de"
                    | "del"
                    | "el"
                    | "en"
                    | "es"
                    | "final"
                    | "for"
                    | "from"
                    | "in"
                    | "is"
                    | "la"
                    | "las"
                    | "latest"
                    | "los"
                    | "most"
                    | "of"
                    | "on"
                    | "or"
                    | "the"
                    | "to"
                    | "una"
                    | "un"
                    | "version"
                    | "versión"
                    | "vigente"
                    | "actual"
                    | "actualmente"
                    | "what"
                    | "which"
            )
        })
        .collect::<Vec<_>>();
    if target_terms.is_empty() {
        return false;
    }

    let clauses = quote
        .replace(" but ", " § ")
        .replace(" while ", " § ")
        .replace(" whereas ", " § ")
        .replace(" pero ", " § ")
        .replace(" mientras ", " § ")
        .replace(" aunque ", " § ");
    clauses
        .split(|character: char| matches!(character, '.' | ';' | ',' | '!' | '?' | '§'))
        .any(|clause| {
            let clause_words = normalized_identity_words(clause);
            qualifiers
                .iter()
                .any(|qualifier| contains_any_phrase(clause, &[qualifier]))
                && !has_older_version_marker(clause)
                && target_terms.iter().all(|term| clause_words.contains(term))
        })
}

fn has_current_version_marker(text: &str) -> bool {
    let has_positive_marker = contains_any_phrase(
        text,
        &[
            "current",
            "currently",
            "current version",
            "current note",
            "latest version",
            "most recent version",
            "actual",
            "actualmente",
            "vigente",
            "versión actual",
            "version actual",
            "versión vigente",
            "version vigente",
            "nota vigente",
        ],
    );
    has_positive_marker
        && !contains_any_phrase(
            text,
            &[
                "not current",
                "not currently",
                "not the current",
                "not identified as current",
                "not marked as current",
                "no longer current",
                "neither is current",
                "neither identified as current",
                "neither is identified as current",
                "no es vigente",
                "no está vigente",
                "no esta vigente",
                "no se identifica como vigente",
                "no es la versión vigente",
                "no es la version vigente",
                "no corresponde a la versión vigente",
                "no corresponde a la version vigente",
                "ninguna está identificada como vigente",
                "ninguna esta identificada como vigente",
                "ninguna versión final está identificada como vigente",
                "ninguna version final esta identificada como vigente",
            ],
        )
}

fn identifier_occurrences(identifier: &str, text: &str) -> usize {
    if identifier.is_empty() {
        return 0;
    }
    text.match_indices(identifier)
        .filter(|(start, _)| {
            let before_is_identifier = text[..*start]
                .chars()
                .next_back()
                .is_some_and(is_identifier_char);
            let end = start + identifier.len();
            let after_is_identifier = text[end..].chars().next().is_some_and(is_identifier_char);
            !before_is_identifier && !after_is_identifier
        })
        .count()
}

/// Reject an actionable quote that attaches a relative clause to a different
/// ID-like entity than the candidate's bound identity. This is deliberately a
/// narrow syntactic guard, not a general coreference or truth check.
fn has_foreign_relative_clause(quote: &str, bound_id: &str) -> bool {
    foreign_relative_assessment(quote, bound_id) == Some(true)
}

/// Attribute relative clauses only up to the exact predicate excerpt proposed for
/// the target. This lets a quote retain later facts about other entities while
/// rejecting a predicate that follows a relative clause attached to another ID.
fn has_foreign_relative_before_predicate(
    quote: &str,
    bound_id: &str,
    predicate_quote: &str,
) -> bool {
    let mut predicate_matches = quote.match_indices(predicate_quote);
    let Some((predicate_start, _)) = predicate_matches.next() else {
        return false;
    };
    if predicate_matches.next().is_some() {
        return false;
    }

    let mut events = Vec::new();
    for token in quote.split(|character: char| {
        !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
    }) {
        if !token.chars().any(char::is_alphabetic)
            || !token.chars().any(char::is_numeric)
            || !token.contains(['-', '_', '.', '/'])
        {
            continue;
        }
        for (start, _) in quote.match_indices(token) {
            if start >= predicate_start {
                continue;
            }
            let end = start + token.len();
            let suffix = &quote[end..predicate_start];
            let mut marker_start = 0;
            while let Some(character) = suffix[marker_start..].chars().next() {
                if character.is_whitespace()
                    || character.is_ascii_punctuation()
                    || matches!(
                        character,
                        '—' | '–' | '‒' | '‑' | '“' | '”' | '‘' | '’' | '…'
                    )
                {
                    marker_start += character.len_utf8();
                } else {
                    break;
                }
            }
            let rest = &suffix[marker_start..];
            let comma_before_marker = suffix[..marker_start].contains(',');
            if let Some((relative_marker_offset, is_relative)) =
                relative_marker_assessment(rest, comma_before_marker)
            {
                if is_relative {
                    let marker_position = end + marker_start + relative_marker_offset;
                    events.push((marker_position, token.to_ascii_lowercase()));
                }
            }
        }
    }
    events
        .into_iter()
        .max_by_key(|(position, _)| *position)
        .is_some_and(|(_, id)| !id.eq_ignore_ascii_case(bound_id))
}

/// Whether the target predicate follows a named subject directly or after a
/// delimited appositive such as "Carlos, his colleague, lives".
fn predicate_follows_named_subject(suffix: &str) -> bool {
    const PREDICATE_MODIFIERS: &[&str] = &[
        "currently",
        "also",
        "probably",
        "perhaps",
        "possibly",
        "still",
        "now",
        "certainly",
        "likely",
        "allegedly",
        "reportedly",
        "apparently",
        "actually",
        "usually",
        "always",
        "actualmente",
        "también",
        "probablemente",
        "quizá",
        "quizás",
        "todavía",
        "supuestamente",
        "incluso",
        "ya",
    ];
    let punctuation_only = |text: &str| {
        text.chars().all(|character| {
            character.is_whitespace()
                || character.is_ascii_punctuation()
                || matches!(character, '—' | '–' | '‒' | '‑' | '”' | '’' | '…')
        })
    };
    if punctuation_only(suffix) {
        return true;
    }
    let punctuation_or_adverbs = |text: &str| {
        punctuation_only(text)
            || text
                .split(|character: char| !character.is_alphabetic())
                .filter(|word| !word.is_empty())
                .all(|word| PREDICATE_MODIFIERS.contains(&word.to_lowercase().as_str()))
    };
    if punctuation_or_adverbs(suffix) {
        return true;
    }
    let suffix_start = suffix.len() - suffix.trim_start().len();
    let content = &suffix[suffix_start..];
    let (appositive, trailing) = if content.starts_with('(') {
        let Some(closing) = content.find(')') else {
            return false;
        };
        if content[closing + 1..].contains(['(', ')']) {
            return false;
        }
        (&content[1..closing], &content[closing + 1..])
    } else {
        let commas = suffix
            .match_indices(',')
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if commas.len() != 2 || !punctuation_only(&suffix[..commas[0]]) {
            return false;
        }
        (&suffix[commas[0] + 1..commas[1]], &suffix[commas[1] + 1..])
    };
    let appositive = appositive.trim();
    let words = appositive.split_whitespace().collect::<Vec<_>>();
    !appositive.is_empty()
        && punctuation_or_adverbs(trailing)
        && !appositive
            .split(|character: char| !character.is_alphabetic())
            .any(|word| {
                matches!(
                    word.to_lowercase().as_str(),
                    "who" | "which" | "that" | "que" | "quien" | "quienes" | "and" | "y" | "e"
                )
            })
        && !words.iter().any(|word| {
            word.chars().any(char::is_numeric)
                && word
                    .chars()
                    .any(|character| matches!(character, '-' | '_' | '.' | '/'))
        })
}

/// Catch a named subject inside an explicit reported complement even when the
/// complement uses no relative marker (for example, "Bob said Carlos lives in Lima").
fn has_foreign_reported_subject_before_predicate(
    quote: &str,
    bound_id: &str,
    predicate_quote: &str,
) -> bool {
    const REPORT_MARKERS: &[&str] = &[
        " said ",
        " thought ",
        " think ",
        " thinks ",
        " believed ",
        " believe ",
        " believes ",
        " assume ",
        " assumed ",
        " assumes ",
        " reckoned ",
        " reckon ",
        " reckons ",
        " suspected ",
        " suspect ",
        " suspects ",
        " knew ",
        " knows ",
        " know ",
        " understood ",
        " understand ",
        " understands ",
        " realized ",
        " realize ",
        " realizes ",
        " noticed ",
        " notice ",
        " notices ",
        " figured ",
        " figures ",
        " learned ",
        " learns ",
        " told ",
        " reported ",
        " stated ",
        " claimed ",
        " dijo ",
        " pensó ",
        " pensaba ",
        " creyó ",
        " creía ",
        " supuso ",
        " suponía ",
        " asumió ",
        " consideró ",
        " sospechó ",
        " sospechaba ",
        " sabía ",
        " supo ",
        " entendió ",
        " entendía ",
        " entiende ",
        " notó ",
        " notaba ",
        " descubrió ",
        " descubría ",
        " aprendió ",
        " cree ",
        " creo ",
        " crees ",
        " creen ",
        " piensan ",
        " piensa ",
        " pienso ",
        " pensás ",
        " consideró que ",
        " consideraba que ",
        " reportó ",
        " informó ",
    ];
    let mut predicate_matches = quote.match_indices(predicate_quote);
    let Some((predicate_start, _)) = predicate_matches.next() else {
        return false;
    };
    if predicate_matches.next().is_some() {
        return false;
    }
    let Some(bound_id_end) = quote.find(bound_id).map(|start| start + bound_id.len()) else {
        return false;
    };
    let prefix = &quote[..predicate_start];
    let last_stable_id = prefix
        .split(|character: char| {
            !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
        })
        .filter(|token| {
            token.chars().any(char::is_alphabetic)
                && token.chars().any(char::is_numeric)
                && token.contains(['-', '_', '.', '/'])
        })
        .next_back();
    if let Some(foreign_id) = last_stable_id.filter(|id| !id.eq_ignore_ascii_case(bound_id)) {
        let Some(foreign_start) = prefix.rfind(foreign_id) else {
            return false;
        };
        if foreign_start < bound_id_end {
            return false;
        }
        let foreign_end = foreign_start + foreign_id.len();
        let before_foreign = &quote[bound_id_end..foreign_start];
        let after_foreign = &quote[foreign_end..predicate_start];
        let has_complementizer = before_foreign
            .split(|character: char| !character.is_alphabetic())
            .any(|word| {
                matches!(
                    word.to_lowercase().as_str(),
                    "that" | "whether" | "if" | "que" | "si"
                )
            });
        if has_complementizer && predicate_follows_named_subject(after_foreign) {
            return true;
        }
    }
    let lowercase = quote.to_lowercase();
    REPORT_MARKERS.iter().any(|marker| {
        lowercase
            .match_indices(marker)
            .any(|(marker_start, found)| {
                let marker_end = marker_start + found.len();
                if marker_start < bound_id_end || marker_end > predicate_start {
                    return false;
                }
                let segment = &quote[marker_end..predicate_start];
                let Some(foreign_id) = segment
                    .split(|character: char| {
                        !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
                    })
                    .filter(|token| {
                        token.chars().any(char::is_alphabetic)
                            && token.chars().any(char::is_numeric)
                            && token.contains(['-', '_', '.', '/'])
                    })
                    .next_back()
                else {
                    return false;
                };
                if foreign_id.eq_ignore_ascii_case(bound_id) {
                    return false;
                }
                let Some(foreign_id_start) = quote[..predicate_start].rfind(foreign_id) else {
                    return false;
                };
                let foreign_id_end = foreign_id_start + foreign_id.len();
                foreign_id_end <= predicate_start
                    && predicate_follows_named_subject(&quote[foreign_id_end..predicate_start])
            })
    })
}

/// Relative clauses inside an explicitly coordinated subject can modify the
/// second participant while the shared plural predicate still applies to both.
fn has_coordinated_plural_subject(quote: &str, bound_id: &str, predicate_quote: &str) -> bool {
    const PLURAL_PREDICATES: &[&str] = &[
        "live", "work", "are", "have", "were", "viven", "trabajan", "están", "son", "tienen",
    ];
    let Some((predicate_start, _)) = quote.match_indices(predicate_quote).next() else {
        return false;
    };
    let first_word = predicate_quote
        .split(|character: char| !character.is_alphabetic())
        .find(|word| !word.is_empty())
        .unwrap_or_default()
        .to_lowercase();
    if !PLURAL_PREDICATES.contains(&first_word.as_str()) {
        return false;
    }
    let Some(bound_start) = quote.find(bound_id) else {
        return false;
    };
    let bound_end = bound_start + bound_id.len();
    let last_id = quote[..predicate_start]
        .split(|character: char| {
            !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
        })
        .filter(|token| {
            token.chars().any(char::is_alphabetic)
                && token.chars().any(char::is_numeric)
                && token.contains(['-', '_', '.', '/'])
        })
        .next_back();
    let Some(foreign_id) = last_id.filter(|id| !id.eq_ignore_ascii_case(bound_id)) else {
        return false;
    };
    let Some(foreign_start) = quote[..predicate_start].rfind(foreign_id) else {
        return false;
    };
    if foreign_start <= bound_end {
        return false;
    }
    let subject_join = quote[bound_end..foreign_start].to_lowercase();
    let subject_join = subject_join.trim_start_matches(|character: char| {
        character.is_whitespace() || matches!(character, ',' | ';')
    });
    subject_join.starts_with("and ")
        || subject_join.starts_with("y ")
        || subject_join.starts_with("e ")
}

fn has_foreign_subject_before_predicate(
    quote: &str,
    bound_id: &str,
    predicate_quote: &str,
) -> bool {
    if has_foreign_reported_subject_before_predicate(quote, bound_id, predicate_quote) {
        return true;
    }
    (has_foreign_relative_before_predicate(quote, bound_id, predicate_quote)
        && !has_coordinated_plural_subject(quote, bound_id, predicate_quote))
        || has_foreign_sentence_subject_before_predicate(quote, bound_id, predicate_quote)
}

/// Reject a predicate in a later sentence or explicitly contrastive clause
/// when it names a different stable-ID subject and does not repeat the
/// query-bound ID. This is a narrow attribution guard, not a parser.
fn has_foreign_sentence_subject_before_predicate(
    quote: &str,
    bound_id: &str,
    predicate_quote: &str,
) -> bool {
    let mut predicate_matches = quote.match_indices(predicate_quote);
    let Some((predicate_start, _)) = predicate_matches.next() else {
        return false;
    };
    if predicate_matches.next().is_some() {
        return false;
    }
    let Some(bound_start) = quote.find(bound_id) else {
        return false;
    };
    let bound_end = bound_start + bound_id.len();
    if predicate_start <= bound_end {
        return false;
    }

    let prefix = &quote[..predicate_start];
    let mut boundaries = prefix
        .char_indices()
        .filter_map(|(offset, character)| match character {
            ';' => Some(offset + character.len_utf8()),
            '.' | '?' | '!' => {
                let end = offset + character.len_utf8();
                let suffix = &quote[end..predicate_start];
                let trimmed = suffix.trim_start();
                let next = trimmed.chars().next()?;
                if !next.is_uppercase() && !matches!(next, '¿' | '¡') {
                    return None;
                }
                let abbreviation =
                    prefix[..offset]
                        .split_whitespace()
                        .next_back()
                        .is_some_and(|word| {
                            [
                                "dr", "mr", "mrs", "ms", "prof", "sr", "sra", "srta", "st", "vs",
                            ]
                            .iter()
                            .any(|known| {
                                word.trim_matches(|c: char| !c.is_alphabetic())
                                    .eq_ignore_ascii_case(known)
                            })
                        });
                (!abbreviation).then_some(end + (suffix.len() - trimmed.len()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    for marker in [", while ", ", mientras "] {
        boundaries.extend(
            prefix
                .match_indices(marker)
                .map(|(offset, matched)| offset + matched.len()),
        );
    }
    let Some(sentence_start) = boundaries.into_iter().max() else {
        return false;
    };
    if sentence_start <= bound_end {
        return false;
    }

    let sentence_prefix = &quote[sentence_start..predicate_start];
    if contains_identifier(bound_id, sentence_prefix) {
        return false;
    }
    let foreign_id = sentence_prefix
        .split(|character: char| {
            !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
        })
        .filter(|token| {
            token.chars().any(char::is_alphabetic)
                && token.chars().any(char::is_numeric)
                && token.contains(['-', '_', '.', '/'])
        })
        .next_back();
    let Some(foreign_id) = foreign_id else {
        return false;
    };
    if foreign_id.eq_ignore_ascii_case(bound_id) {
        return false;
    }
    let foreign_end = sentence_prefix.rfind(foreign_id).unwrap_or(0) + foreign_id.len();
    predicate_follows_named_subject(&sentence_prefix[foreign_end..])
}

/// `Some(false)` means a marker could be a relative or a complement; retain it as
/// unresolved rather than allowing it to assert a fact or failing the whole trace.
fn foreign_relative_assessment(quote: &str, bound_id: &str) -> Option<bool> {
    let mut relative_events = Vec::new();
    for token in quote.split(|character: char| {
        !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
    }) {
        if !token.chars().any(char::is_alphabetic)
            || !token.chars().any(char::is_numeric)
            || !token.contains(['-', '_', '.', '/'])
        {
            continue;
        }
        for (start, _) in quote.match_indices(token) {
            let end = start + token.len();
            let suffix = &quote[end..];
            let mut marker_start = 0;
            while let Some(character) = suffix[marker_start..].chars().next() {
                if character.is_whitespace()
                    || character.is_ascii_punctuation()
                    || matches!(
                        character,
                        '—' | '–' | '‒' | '‑' | '“' | '”' | '‘' | '’' | '…'
                    )
                {
                    marker_start += character.len_utf8();
                } else {
                    break;
                }
            }
            let rest = &suffix[marker_start..];
            let comma_before_marker = suffix[..marker_start].contains(',');
            if let Some((relative_marker_offset, is_relative)) =
                relative_marker_assessment(rest, comma_before_marker)
            {
                let marker_position = end + marker_start + relative_marker_offset;
                relative_events.push((marker_position, is_relative));
            }
        }
    }
    relative_events.sort_by_key(|(position, _)| *position);
    let mut ambiguous_foreign = false;
    let mut index = 0;
    while index < relative_events.len() {
        let marker_position = relative_events[index].0;
        let mut is_relative = false;
        while index < relative_events.len() && relative_events[index].0 == marker_position {
            is_relative |= relative_events[index].1;
            index += 1;
        }
        let nearest_id = quote[..marker_position]
            .split(|character: char| {
                !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
            })
            .filter(|token| {
                token.chars().any(char::is_alphabetic)
                    && token.chars().any(char::is_numeric)
                    && token.contains(['-', '_', '.', '/'])
            })
            .next_back();
        let is_foreign = nearest_id.is_none_or(|id| !id.eq_ignore_ascii_case(bound_id));
        if !is_foreign {
            if is_relative {
                return None;
            }
            continue;
        }
        if is_relative {
            return Some(true);
        }
        ambiguous_foreign = true;
    }
    ambiguous_foreign.then_some(false)
}

/// Rescue an ambiguous `that`/`which`/`que` complement only when the question
/// explicitly asks what the bound person asked/told the same ID-linked recipient.
fn query_targets_speech_complement(
    query: &str,
    quote: &str,
    bound_id: &str,
    subject_quote: &str,
    anchor_kind: &str,
) -> bool {
    let query_lower = query.to_lowercase();
    let english_query = query_lower.contains("what did");
    let spanish_query = query_lower.contains("qué") && query_lower.contains("dijo");
    if !(english_query || spanish_query) || !contains_identifier(bound_id, query) {
        return false;
    }
    let query_subject_context = if spanish_query {
        let Some(start) = query_lower.find("dijo").map(|index| index + "dijo".len()) else {
            return false;
        };
        &query[start..]
    } else {
        let Some(start) = query_lower
            .find("what did")
            .map(|index| index + "what did".len())
        else {
            return false;
        };
        &query[start..]
    };
    if !subject_prefix_is_direct_question_subject(query_subject_context, subject_quote, anchor_kind)
    {
        return false;
    }
    quote
        .split(|character: char| {
            !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
        })
        .filter(|token| {
            *token != bound_id
                && token.chars().any(char::is_alphabetic)
                && token.chars().any(char::is_numeric)
                && token.contains(['-', '_', '.', '/'])
        })
        .any(|recipient_id| {
            let query_verbs: &[&str] = if spanish_query {
                &["dijo"]
            } else {
                &[
                    "ask", "tell", "inform", "notify", "warn", "explain", "report", "advise",
                ]
            };
            let quote_verbs: &[&str] = if spanish_query {
                &["dijo", "dicho"]
            } else {
                &[
                    "asked",
                    "told",
                    "informed",
                    "notified",
                    "warned",
                    "explained",
                    "reported",
                    "advised",
                ]
            };
            contains_identifier(recipient_id, query)
                && contains_identifier(recipient_id, quote)
                && has_ordered_recipient_speech_act(query, bound_id, recipient_id, query_verbs)
                && (spanish_query
                    || subject_is_clause_initial_or_kind_prefixed(
                        query_subject_context,
                        subject_quote,
                        anchor_kind,
                        query_verbs,
                    ))
                && has_ordered_recipient_speech_act(quote, bound_id, recipient_id, quote_verbs)
                && subject_is_clause_initial_or_kind_prefixed(
                    quote,
                    subject_quote,
                    anchor_kind,
                    quote_verbs,
                )
                && has_immediate_clause_marker_after_id(quote, recipient_id)
        })
}

/// Require the quoted identity to begin its local clause, optionally preceded
/// only by determiners and the query-declared anchor kind. This is a narrow
/// fail-closed surface rule, not a syntactic parser.
fn subject_is_clause_initial_or_kind_prefixed(
    text: &str,
    subject_quote: &str,
    kind: &str,
    speech_verbs: &[&str],
) -> bool {
    let normalized = text.to_ascii_lowercase();
    let subject_quote = subject_quote.to_ascii_lowercase();
    let Some(subject_start) = normalized.find(&subject_quote) else {
        return false;
    };
    let subject_end = subject_start + subject_quote.len();
    let Some((verb_start, _)) = speech_verbs.iter().find_map(|verb| {
        normalized[subject_end..]
            .match_indices(verb)
            .find_map(|(relative_start, found)| {
                let start = subject_end + relative_start;
                let before_is_boundary = normalized[..start]
                    .chars()
                    .next_back()
                    .is_none_or(|character| !character.is_alphanumeric());
                let end = start + found.len();
                let after_is_boundary = normalized[end..]
                    .chars()
                    .next()
                    .is_none_or(|character| !character.is_alphanumeric());
                (before_is_boundary && after_is_boundary).then_some((start, end))
            })
    }) else {
        return false;
    };
    let identity_to_verb = &normalized[subject_end..verb_start];
    if identity_to_verb.contains(':') {
        return false;
    }
    if !identity_to_verb
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .all(|token| {
            [
                "le", "les", "lo", "la", "los", "las", "me", "te", "nos", "os", "se", "had",
                "había",
            ]
            .contains(&token)
                || [
                    "quickly",
                    "slowly",
                    "already",
                    "clearly",
                    "directly",
                    "firmly",
                    "after",
                    "lunch",
                    "formally",
                    "privately",
                    "publicly",
                    "briefly",
                    "immediately",
                    "rápidamente",
                    "directamente",
                    "claramente",
                    "formalmente",
                    "brevemente",
                    "inmediatamente",
                ]
                .contains(&token)
        })
    {
        return false;
    }
    subject_prefix_is_clause_initial_or_kind_prefixed(&normalized, &subject_quote, kind)
}

fn subject_prefix_is_clause_initial_or_kind_prefixed(
    text: &str,
    subject_quote: &str,
    kind: &str,
) -> bool {
    let normalized = text.to_ascii_lowercase();
    let subject_quote = subject_quote.to_ascii_lowercase();
    let Some(subject_start) = normalized.find(&subject_quote) else {
        return false;
    };
    let clause_prefix = normalized[..subject_start]
        .rsplit(['.', '!', '?', ';', ':', ',', '—', '–'])
        .next()
        .unwrap_or_default();
    let allowed_kind = kind.to_ascii_lowercase();
    clause_prefix
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .all(|token| {
            token == allowed_kind
                || [
                    "the", "a", "an", "la", "el", "los", "las", "un", "una", "unos", "unas", "my",
                    "your", "his", "her", "our", "their", "mi", "mis", "tu", "tus", "su", "sus",
                    "nuestro", "nuestra", "nuestros", "nuestras",
                ]
                .contains(&token)
        })
}

/// Require both the bound identity and a matching speech verb before the exact
/// recipient ID, with no subordinate clause introducing that recipient.
fn has_ordered_recipient_speech_act(
    text: &str,
    bound_id: &str,
    recipient_id: &str,
    speech_verbs: &[&str],
) -> bool {
    let normalized = text.to_ascii_lowercase();
    let (Some(bound_start), Some(recipient_start)) = (text.find(bound_id), text.find(recipient_id))
    else {
        return false;
    };
    let bound_end = bound_start + bound_id.len();
    if bound_end > recipient_start {
        return false;
    }
    let Some((verb_start, verb_end)) = speech_verbs.iter().find_map(|verb| {
        normalized.match_indices(verb).find_map(|(start, found)| {
            let before_is_boundary = normalized[..start]
                .chars()
                .next_back()
                .is_none_or(|character| !character.is_alphanumeric());
            let end = start + found.len();
            let after_is_boundary = normalized[end..]
                .chars()
                .next()
                .is_none_or(|character| !character.is_alphanumeric());
            (before_is_boundary && after_is_boundary && end <= recipient_start)
                .then_some((start, end))
        })
    }) else {
        return false;
    };
    let intervening_start = bound_end.max(verb_end);
    if verb_start >= recipient_start || intervening_start > recipient_start {
        return false;
    }
    let intervening = &normalized[intervening_start..recipient_start];
    !intervening
        .split(|character: char| !character.is_alphanumeric())
        .any(|token| {
            [
                "that",
                "which",
                "who",
                "whom",
                "whose",
                "where",
                "que",
                "quien",
                "si",
                "by",
                "por",
                "about",
                "of",
                "for",
                "with",
                "from",
                "not",
                "no",
                "sino",
                "instead",
                "except",
                "rather",
                "excluding",
                "salvo",
                "excepto",
                "menos",
                "regarding",
                "concerning",
                "sobre",
                "acerca",
                "respecto",
                "de",
                "para",
                "con",
            ]
            .contains(&token)
        })
}

fn has_immediate_clause_marker_after_id(text: &str, id: &str) -> bool {
    let Some(start) = text.find(id).map(|index| index + id.len()) else {
        return false;
    };
    let suffix = text[start..].trim_start_matches(|character: char| {
        character.is_whitespace()
            || character.is_ascii_punctuation()
            || matches!(
                character,
                '—' | '–' | '‒' | '‑' | '“' | '”' | '‘' | '’' | '…'
            )
    });
    ["that", "which", "que"].iter().any(|marker| {
        let Some(marker_start) = suffix.to_ascii_lowercase().find(marker) else {
            return false;
        };
        let marker_end = marker_start + marker.len();
        if suffix[marker_end..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric)
        {
            return false;
        }
        let aside = suffix[..marker_start]
            .split(|character: char| !character.is_alphanumeric())
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>();
        aside.is_empty()
            || is_role_contrast_aside(&aside)
            || is_third_party_exclusion_aside(&suffix[..marker_start], id)
            || aside == ["at", "lunch"]
            || aside == ["after", "lunch"]
            || aside == ["on", "monday"]
            || aside == ["el", "lunes"]
            || aside == ["yesterday"]
            || aside == ["last", "night"]
    })
}

/// A contrastive exclusion after the queried recipient can be an aside rather
/// than evidence that the recipient slot is wrong: "told Bob, not Alice, that…".
/// Require a literal third-party stable ID so vague markers do not rescue an
/// ambiguous attribution.
fn is_third_party_exclusion_aside(text: &str, queried_recipient_id: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let Some((marker, marker_start)) = ["but not", "not", "sino", "no"]
        .iter()
        .find_map(|marker| find_word_marker(&lower, marker).map(|start| (*marker, start)))
    else {
        return false;
    };
    let prefix = lower[..marker_start].trim_end();
    if !prefix.is_empty()
        && !prefix
            .chars()
            .next_back()
            .is_some_and(|character| matches!(character, ',' | '(' | '—' | '–' | '-'))
    {
        return false;
    }
    let name_span = &text[marker_start + marker.len()..];
    let has_other_stable_id = name_span
        .split(|character: char| {
            !character.is_alphanumeric() && !matches!(character, '-' | '_' | '.' | '/')
        })
        .filter(|token| {
            token != &queried_recipient_id
                && token.chars().any(char::is_alphabetic)
                && token.chars().any(char::is_numeric)
                && token.contains(['-', '_', '.', '/'])
        })
        .next()
        .is_some();
    let name_tokens = name_span
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    has_other_stable_id
        && name_tokens.len() <= 6
        && !name_tokens.iter().any(|token| {
            [
                "that", "which", "who", "whom", "whose", "where", "when", "if", "unless", "porque",
                "quien", "quienes", "cuyo", "cuya", "cuyos", "cuyas", "donde", "cuando", "que",
            ]
            .contains(token)
        })
}

/// Keep a recipient when the aside contrasts the capacity in which the person
/// was addressed, rather than excluding the recipient from the speech act.
fn is_role_contrast_aside(tokens: &[&str]) -> bool {
    let (prefix, contrast) = if tokens.starts_with(&["not", "as"]) {
        (2, "but")
    } else if tokens.starts_with(&["no", "como"]) {
        (2, "sino")
    } else {
        return false;
    };
    let skip_article = |index: usize| {
        if tokens
            .get(index)
            .is_some_and(|token| ["a", "an", "un", "una"].contains(token))
        {
            index + 1
        } else {
            index
        }
    };
    let first_role_start = skip_article(prefix);
    let Some(contrast_index) = tokens[first_role_start..]
        .iter()
        .position(|token| *token == contrast)
        .map(|offset| first_role_start + offset)
    else {
        return false;
    };
    let first_role = &tokens[first_role_start..contrast_index];
    let second_start = contrast_index + 1;
    let expected_as = if contrast == "but" { "as" } else { "como" };
    if tokens.get(second_start) != Some(&expected_as) {
        return false;
    }
    let second_role_start = skip_article(second_start + 1);
    let second_role = &tokens[second_role_start..];
    let role_words = |words: &[&str]| {
        (1..=2).contains(&words.len())
            && words
                .iter()
                .all(|word| word.chars().all(char::is_alphabetic))
    };
    role_words(first_role) && role_words(second_role)
}

/// Detect a direct relative marker or one after a short article-led appositive.
/// Avoid scanning arbitrary later clauses, where `which`, `that`, or `que`
/// may introduce a question or complement rather than describe the foreign ID.
fn relative_marker_assessment(text: &str, comma_before_marker: bool) -> Option<(usize, bool)> {
    const MAX_LOOKAHEAD_BYTES: usize = 120;
    let lower = text.to_lowercase();
    let bounded_end = lower
        .char_indices()
        .take_while(|(index, character)| {
            *index < MAX_LOOKAHEAD_BYTES && !matches!(character, '.' | '?' | '!' | ';' | '\n')
        })
        .last()
        .map_or(0, |(index, character)| index + character.len_utf8());
    let bounded = &lower[..bounded_end];
    let markers = [
        "who",
        "whom",
        "whose",
        "which",
        "that",
        "where",
        "quienes",
        "quiénes",
        "quien",
        "quién",
        "el cual",
        "la cual",
        "los cuales",
        "las cuales",
        "el que",
        "la que",
        "los que",
        "las que",
        "que",
    ];
    markers
        .iter()
        .filter_map(|marker| {
            bounded
                .match_indices(marker)
                .filter_map(|(start, found)| {
                    let before_is_boundary = bounded[..start]
                        .chars()
                        .next_back()
                        .is_none_or(|character| !character.is_alphanumeric());
                    let end = start + found.len();
                    let after_is_boundary = bounded[end..]
                        .chars()
                        .next()
                        .is_none_or(|character| !character.is_alphanumeric());
                    if !before_is_boundary || !after_is_boundary {
                        return None;
                    }
                    let intervening = bounded[..start].trim();
                    let is_direct = intervening.is_empty();
                    let is_article_led_appositive = !is_direct
                        && intervening.split_whitespace().count() <= 30
                        && [
                            "a", "an", "the", "el", "la", "los", "las", "un", "una", "unos", "unas",
                        ]
                        .iter()
                        .any(|article| {
                            intervening
                                .split_whitespace()
                                .next()
                                .is_some_and(|first| first == *article)
                        });
                    let marker_is_strong_relative =
                        ["who", "whom", "whose", "quien", "quienes", "que"]
                            .iter()
                            .any(|strong| marker == strong);
                    let is_prepositional_relative = *marker == "which"
                        && ["at ", "in ", "on ", "for ", "con ", "de "]
                            .iter()
                            .any(|preposition| intervening.starts_with(preposition));
                    if !is_direct
                        && !is_article_led_appositive
                        && !(marker_is_strong_relative
                            && intervening.split_whitespace().count() <= 30)
                        && !is_prepositional_relative
                    {
                        return None;
                    }

                    let following = bounded[end..].trim_start();
                    let following_words = following.split_whitespace().take(3).collect::<Vec<_>>();
                    let original_next_is_uppercase =
                        original_char_after_lowercase_byte_offset(text, end)
                            .is_some_and(char::is_uppercase);
                    let ambiguous_complement = match *marker {
                        "which" => {
                            ["city", "request", "option", "way", "place", "time"]
                                .iter()
                                .any(|noun| following.starts_with(noun))
                                || [
                                    "to", "i", "we", "you", "he", "she", "they", "it", "our",
                                    "their", "his", "her",
                                ]
                                .contains(following_words.get(1).unwrap_or(&&""))
                        }
                        "that" => {
                            [
                                "the ", "a ", "an ", "this ", "these ", "those ", "he ", "she ",
                                "they ", "we ", "i ", "you ", "it ", "alice ", "rain ",
                            ]
                            .iter()
                            .any(|prefix| following.starts_with(prefix))
                                || original_next_is_uppercase
                        }
                        "que" => {
                            [
                                "el ",
                                "la ",
                                "los ",
                                "las ",
                                "un ",
                                "una ",
                                "su ",
                                "sus ",
                                "mi ",
                                "mis ",
                                "tu ",
                                "tus ",
                                "mañana ",
                                "ayer ",
                                "hoy ",
                                "entonces ",
                                "estaría ",
                                "estaria ",
                            ]
                            .iter()
                            .any(|prefix| following.starts_with(prefix))
                                || original_next_is_uppercase
                        }
                        "where" | "who" | "whom" if following.starts_with("to ") => true,
                        _ => false,
                    };
                    // In the reviewed pattern, a comma immediately before "que"
                    // introduces a relative ("Bob, que vive…"); a comma elsewhere
                    // may be an unrelated adjunct before a clause-taking verb's
                    // complement ("Bob, el viernes, que estaría…"). Preserve the
                    // lexical heuristic for unpunctuated cases, whose role still
                    // needs broader validation.
                    let comma_introduced_que_relative =
                        comma_before_marker && *marker == "que" && is_direct;
                    if comma_introduced_que_relative {
                        Some((start, true))
                    } else {
                        Some((start, !ambiguous_complement))
                    }
                })
                .min_by_key(|(start, _)| *start)
        })
        .min_by_key(|(start, _)| *start)
}

fn original_char_after_lowercase_byte_offset(text: &str, offset: usize) -> Option<char> {
    let mut lower_offset = 0;
    for (original_offset, character) in text.char_indices() {
        if lower_offset == offset {
            return text[original_offset..].trim_start().chars().next();
        }
        lower_offset += character.to_lowercase().map(char::len_utf8).sum::<usize>();
        if lower_offset > offset {
            return None;
        }
    }
    None
}

fn contains_domain_type(kind: &str, text: &str) -> bool {
    if kind.is_ascii() {
        contains_identifier(&kind.to_ascii_lowercase(), &text.to_ascii_lowercase())
    } else {
        text.contains(kind)
    }
}

fn is_identifier_char(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '-' | '_' | '.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SourceRef;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct LocalBank {
        synthetic: bool,
        cases: Vec<LocalBankCase>,
    }

    #[derive(Deserialize)]
    struct LocalBankCase {
        id: String,
        query: String,
        candidate: LocalBankCandidate,
        trace: SlotEvidenceTrace,
        expected_valid: bool,
        expected_selected: Vec<String>,
        #[serde(default)]
        expected_relation: Option<EvidenceRelation>,
        expected_error: Option<String>,
        #[serde(default)]
        expected_demoted: usize,
    }

    #[derive(Deserialize)]
    struct LocalBankCandidate {
        memory_id: String,
        revision: i64,
        summary: String,
    }

    #[derive(Deserialize)]
    struct LocalRetrospectiveOverlay {
        version: String,
        source_bank: String,
        source_bank_sha256: String,
        annotations: Vec<LocalRetrospectiveAnnotation>,
    }

    #[derive(Deserialize)]
    struct LocalRetrospectiveAnnotation {
        case_id: String,
        relation: String,
        premise_dimension: String,
        premise_quote: String,
    }

    #[test]
    fn frozen_v2_local_bank_preserves_evidence_and_rejects_wrong_referents() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank.json"
        ))
        .expect("valid synthetic slot-evidence-v2 local bank");
        assert!(bank.synthetic);
        assert_eq!(bank.cases.len(), 19);

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} validity error={:?}",
                case.id,
                case.trace.validation_error(&case.query, &[memory.clone()])
            );
            if let Some(expected_error) = case.expected_error {
                assert_eq!(
                    case.trace.validation_error(&case.query, &[memory.clone()]),
                    Some(expected_error.as_str()),
                    "case {} validation error",
                    case.id
                );
            }
            assert_eq!(
                case.trace.selected_ids(&[memory], &case.query),
                case.expected_selected,
                "case {} selected IDs",
                case.id
            );
        }
    }

    #[test]
    fn v2_new_holdout_bank_binds_useful_and_non_actionable_evidence() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r22.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r22 holdout bank");
        assert!(bank.synthetic);
        assert_eq!(bank.cases.len(), 11);

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.expected_relation,
                case.trace
                    .candidates
                    .first()
                    .and_then(|candidate| candidate.propositions.first())
                    .map(|proposition| proposition.relation),
                "case {} relation annotation differs from its trace",
                case.id
            );
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} validity error={:?}",
                case.id,
                case.trace.validation_error(&case.query, &[memory.clone()])
            );
            assert_eq!(
                case.trace.selected_ids(&[memory], &case.query),
                case.expected_selected,
                "case {} selected IDs",
                case.id
            );
        }
    }

    #[test]
    fn v2_reported_subject_matrix_blocks_foreign_fact_without_losing_report_content() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/reported-subject-matrix-v1.json"
        ))
        .expect("valid synthetic reported-subject matrix");
        assert!(bank.synthetic);
        assert_eq!(bank.cases.len(), 3);

        for mut case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace
                    .validate(&case.query, std::slice::from_ref(&memory)),
                case.expected_valid,
                "case {} validation error={:?}",
                case.id,
                case.trace
                    .validation_error(&case.query, std::slice::from_ref(&memory))
            );
            let before = case
                .trace
                .selected_ids(std::slice::from_ref(&memory), &case.query);
            if case.expected_demoted > 0 {
                assert_eq!(
                    before,
                    vec![case.candidate.memory_id.clone()],
                    "{}",
                    case.id
                );
            }
            assert_eq!(
                case.trace.demote_ambiguous_attributions(&case.query),
                case.expected_demoted,
                "case {} demoted propositions",
                case.id
            );
            assert_eq!(
                case.trace.selected_ids(&[memory], &case.query),
                case.expected_selected,
                "case {} selected IDs after local attribution check",
                case.id
            );
        }
    }

    #[test]
    fn v2_reported_subject_matrix_v2_blocks_explicitly_distinct_homonym() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/reported-subject-matrix-v2.json"
        ))
        .expect("valid synthetic reported-subject matrix v2");
        assert!(bank.synthetic);
        assert_eq!(bank.cases.len(), 1);
        let mut case = bank.cases.into_iter().next().unwrap();
        let mut memory = candidate(&case.candidate.summary);
        memory.memory_id = case.candidate.memory_id.clone();
        memory.revision = case.candidate.revision;
        assert!(
            case.trace
                .validate(&case.query, std::slice::from_ref(&memory))
        );
        assert_eq!(
            case.trace
                .selected_ids(std::slice::from_ref(&memory), &case.query),
            vec![case.candidate.memory_id.clone()]
        );
        assert_eq!(
            case.trace.demote_ambiguous_attributions(&case.query),
            case.expected_demoted
        );
        assert_eq!(
            case.trace.selected_ids(&[memory], &case.query),
            case.expected_selected
        );
    }

    #[test]
    fn v2_reported_subject_matrix_v3_blocks_descriptive_foreign_subjects() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/reported-subject-matrix-v3.json"
        ))
        .expect("valid synthetic reported-subject matrix v3");
        assert!(bank.synthetic);
        assert_eq!(bank.cases.len(), 2);
        for mut case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert!(
                case.trace
                    .validate(&case.query, std::slice::from_ref(&memory)),
                "{}",
                case.id
            );
            assert_eq!(
                case.trace
                    .selected_ids(std::slice::from_ref(&memory), &case.query),
                vec![case.candidate.memory_id.clone()],
                "{} before normalization",
                case.id
            );
            assert_eq!(
                case.trace.demote_ambiguous_attributions(&case.query),
                case.expected_demoted,
                "{} demoted propositions",
                case.id
            );
            assert_eq!(
                case.trace.selected_ids(&[memory], &case.query),
                case.expected_selected,
                "{} after normalization",
                case.id
            );
        }
    }

    #[test]
    fn v2_reported_subject_matrix_v4_preserves_compound_names_without_prefix_binding() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/reported-subject-matrix-v4.json"
        ))
        .expect("valid synthetic reported-subject matrix v4");
        assert!(bank.synthetic);
        assert_eq!(bank.cases.len(), 8);
        for mut case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert!(
                case.trace
                    .validate(&case.query, std::slice::from_ref(&memory)),
                "{}: {:?}",
                case.id,
                case.trace
                    .validation_error(&case.query, std::slice::from_ref(&memory))
            );
            assert_eq!(
                case.trace
                    .selected_ids(std::slice::from_ref(&memory), &case.query),
                vec![case.candidate.memory_id.clone()],
                "{} before normalization",
                case.id
            );
            assert_eq!(
                case.trace.demote_ambiguous_attributions(&case.query),
                case.expected_demoted,
                "{} demoted propositions",
                case.id
            );
            assert_eq!(
                case.trace.selected_ids(&[memory], &case.query),
                case.expected_selected,
                "{} after normalization",
                case.id
            );
        }
    }

    #[test]
    fn v2_revision_matrix_preserves_finality_conflict_and_correction_roles() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/revision-matrix-v1.json"
        ))
        .expect("valid synthetic slot-evidence-v2 revision matrix");
        assert!(bank.synthetic);
        assert_eq!(bank.cases.len(), 8);

        let mut relations = std::collections::HashMap::new();
        for case in &bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            let proposition = case.trace.candidates[0].propositions.first().unwrap();
            relations.insert(case.id.as_str(), proposition.relation);
            assert_eq!(
                Some(proposition.relation),
                case.expected_relation,
                "case {}",
                case.id
            );
            assert!(
                case.trace
                    .validate(&case.query, std::slice::from_ref(&memory)),
                "case {} validation error={:?}",
                case.id,
                case.trace
                    .validation_error(&case.query, std::slice::from_ref(&memory))
            );
            assert_eq!(
                case.trace.selected_ids(&[memory], &case.query),
                case.expected_selected,
                "case {}",
                case.id
            );
        }
        assert_eq!(relations["revision-03"], EvidenceRelation::Partial);
        assert_eq!(relations["revision-06"], EvidenceRelation::Partial);
        assert_eq!(relations["revision-08"], EvidenceRelation::Refutes);
    }

    #[test]
    fn v2_revision_matrix_exposes_known_stale_draft_false_acceptance() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/revision-matrix-v1.json"
        ))
        .expect("valid synthetic slot-evidence-v2 revision matrix");
        let case = bank
            .cases
            .into_iter()
            .find(|case| case.id == "revision-01")
            .expect("final-date contrast exists");
        let mut memory = candidate(&case.candidate.summary);
        memory.memory_id = case.candidate.memory_id.clone();
        memory.revision = case.candidate.revision;

        let mut trace = case.trace;
        trace.candidates[0].propositions[0].quote =
            "For project Atlas (ID AT-17), an earlier draft lists the launch date as 12 May."
                .into();

        assert!(trace.validate(&case.query, std::slice::from_ref(&memory)));
        assert_eq!(
            trace.selected_ids(&[memory], &case.query),
            vec!["revision-01"],
            "literal evidence binding does not establish revision authority"
        );
    }

    #[test]
    fn v2_temporal_query_downgrades_old_draft_without_losing_version_context() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/revision-matrix-v1.json"
        ))
        .expect("valid synthetic slot-evidence-v2 revision matrix");
        let cases = bank.cases;

        let final_case = cases
            .iter()
            .find(|case| case.id == "revision-01")
            .expect("final-date contrast exists");
        let mut final_memory = candidate(&final_case.candidate.summary);
        final_memory.memory_id = final_case.candidate.memory_id.clone();
        final_memory.revision = final_case.candidate.revision;
        let mut stale_final_trace = final_case.trace.clone();
        stale_final_trace.candidates[0].propositions[0].quote =
            "For project Atlas (ID AT-17), an earlier draft lists the launch date as 12 May."
                .into();
        assert_eq!(
            stale_final_trace
                .qualify_temporal_evidence(&final_case.query, std::slice::from_ref(&final_memory)),
            1
        );
        assert_eq!(
            stale_final_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Partial
        );
        assert_eq!(
            stale_final_trace.selected_ids(std::slice::from_ref(&final_memory), &final_case.query),
            vec![final_case.candidate.memory_id.clone()],
            "version history stays available as partial evidence"
        );

        let mut rejected_final_trace = final_case.trace.clone();
        rejected_final_trace.candidates[0].propositions[0].quote = "For project Atlas (ID AT-17), an earlier draft listed 12 May; it was not the final version.".into();
        assert_eq!(
            rejected_final_trace
                .qualify_temporal_evidence(&final_case.query, std::slice::from_ref(&final_memory)),
            1,
            "a negated final-version marker cannot qualify an earlier draft"
        );
        assert_eq!(
            rejected_final_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Partial
        );

        let mut copied_draft_trace = final_case.trace.clone();
        copied_draft_trace.candidates[0].propositions[0].quote = "For project Atlas (ID AT-17), a final version was prepared containing the launch date copied from an earlier draft: 12 May.".into();
        assert_eq!(
            copied_draft_trace
                .qualify_temporal_evidence(&final_case.query, std::slice::from_ref(&final_memory)),
            1,
            "a final marker in the same clause does not qualify a value attributed to an older draft"
        );
        assert_eq!(
            copied_draft_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Partial
        );

        let mut both_versions_trace = final_case.trace.clone();
        both_versions_trace.candidates[0].propositions[0].quote = "For project Atlas (ID AT-17), an earlier draft listed 12 May and was not the final version; the final version lists the launch date as 19 May.".into();
        assert_eq!(
            both_versions_trace
                .qualify_temporal_evidence(&final_case.query, std::slice::from_ref(&final_memory)),
            0,
            "a separate affirmative final-version clause remains usable"
        );
        assert_eq!(
            both_versions_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Answers
        );

        let earlier_case = cases
            .iter()
            .find(|case| case.id == "revision-02")
            .expect("earlier-draft query exists");
        let mut earlier_memory = candidate(&earlier_case.candidate.summary);
        earlier_memory.memory_id = earlier_case.candidate.memory_id.clone();
        earlier_memory.revision = earlier_case.candidate.revision;
        let mut earlier_trace = earlier_case.trace.clone();
        earlier_trace.candidates[0].propositions[0].quote =
            "For project Beacon (ID BE-24), an earlier draft lists the launch date as 12 May."
                .into();
        assert_eq!(
            earlier_trace.qualify_temporal_evidence(
                &earlier_case.query,
                std::slice::from_ref(&earlier_memory)
            ),
            0
        );
        assert_eq!(
            earlier_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Answers,
            "a question explicitly about the earlier draft remains answerable"
        );

        let current_case = cases
            .iter()
            .find(|case| case.id == "revision-03")
            .expect("current-date contrast exists");
        let mut current_memory = candidate(&current_case.candidate.summary);
        current_memory.memory_id = current_case.candidate.memory_id.clone();
        current_memory.revision = current_case.candidate.revision;
        let mut current_trace = current_case.trace.clone();
        current_trace.candidates[0].propositions[0].relation = EvidenceRelation::Answers;
        assert_eq!(
            current_trace.qualify_temporal_evidence(
                &current_case.query,
                std::slice::from_ref(&current_memory)
            ),
            1
        );
        assert_eq!(
            current_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Partial,
            "a final version is not proof of current authority"
        );

        let mut positive_current_trace = current_case.trace.clone();
        positive_current_trace.candidates[0].propositions[0].relation = EvidenceRelation::Answers;
        positive_current_trace.candidates[0].propositions[0].quote =
            "For project Cedar (ID CE-31), the current version lists the launch date as 19 May."
                .into();
        assert_eq!(
            positive_current_trace.qualify_temporal_evidence(
                &current_case.query,
                std::slice::from_ref(&current_memory)
            ),
            0,
            "an explicit positive current-version marker remains answerable"
        );
        assert_eq!(
            positive_current_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Answers
        );

        let mut mixed_history_current_trace = current_case.trace.clone();
        mixed_history_current_trace.candidates[0].propositions[0].relation =
            EvidenceRelation::Answers;
        mixed_history_current_trace.candidates[0].propositions[0].quote = "For project Cedar (ID CE-31), an earlier draft listed 12 May, but the current version lists the launch date as 19 May.".into();
        assert_eq!(
            mixed_history_current_trace.qualify_temporal_evidence(
                &current_case.query,
                std::slice::from_ref(&current_memory)
            ),
            0,
            "an explicit current proposition is not demoted because its quote also gives history"
        );
        assert_eq!(
            mixed_history_current_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Answers
        );

        let combined_query =
            "What is the current launch date in the final version for project Atlas?";
        let mut combined_trace = current_case.trace.clone();
        combined_trace.candidates[0].propositions[0].relation = EvidenceRelation::Answers;
        combined_trace.candidates[0].propositions[0].quote =
            "For project Atlas (ID AT-17), the final version lists the launch date as 19 May."
                .into();
        assert_eq!(
            combined_trace
                .qualify_temporal_evidence(combined_query, std::slice::from_ref(&current_memory)),
            1,
            "combined current-and-final intent still requires current evidence"
        );
        assert_eq!(
            combined_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Partial
        );

        let spanish_query = "¿Cuál es la fecha vigente en la versión final del proyecto Atlas?";
        let mut spanish_trace = combined_trace.clone();
        spanish_trace.candidates[0].propositions[0].relation = EvidenceRelation::Answers;
        assert_eq!(
            spanish_trace
                .qualify_temporal_evidence(spanish_query, std::slice::from_ref(&current_memory)),
            1,
            "Spanish combined current-and-final intent receives the same guard"
        );
        assert_eq!(
            spanish_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Partial
        );

        let conflict_case = cases
            .iter()
            .find(|case| case.id == "revision-06")
            .expect("conflicting final versions case exists");
        let mut conflict_memory = candidate(&conflict_case.candidate.summary);
        conflict_memory.memory_id = conflict_case.candidate.memory_id.clone();
        conflict_memory.revision = conflict_case.candidate.revision;
        let mut conflict_trace = conflict_case.trace.clone();
        conflict_trace.candidates[0].propositions[0].relation = EvidenceRelation::Answers;
        assert_eq!(
            conflict_trace.qualify_temporal_evidence(
                &conflict_case.query,
                std::slice::from_ref(&conflict_memory)
            ),
            1,
            "a negated current marker must not count as evidence of current authority"
        );
        assert_eq!(
            conflict_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Partial
        );

        let mut negated_current_trace = current_case.trace.clone();
        negated_current_trace.candidates[0].propositions[0].relation = EvidenceRelation::Answers;
        negated_current_trace.candidates[0].propositions[0].quote = "For project Cedar (ID CE-31), this is not the current version; the launch date is 12 May.".into();
        assert_eq!(
            negated_current_trace.qualify_temporal_evidence(
                &current_case.query,
                std::slice::from_ref(&current_memory)
            ),
            1,
            "explicitly negated current-version wording must not satisfy current intent"
        );
        assert_eq!(
            negated_current_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Partial
        );

        let spanish_case = cases
            .iter()
            .find(|case| case.id == "revision-07")
            .expect("Spanish revision case exists");
        let mut spanish_memory = candidate(&spanish_case.candidate.summary);
        spanish_memory.memory_id = spanish_case.candidate.memory_id.clone();
        spanish_memory.revision = spanish_case.candidate.revision;
        let mut spanish_negation_trace = spanish_case.trace.clone();
        spanish_negation_trace.candidates[0].propositions[0].relation = EvidenceRelation::Answers;
        spanish_negation_trace.candidates[0].propositions[0].quote =
            "Para el proyecto Girasol (ID GI-78), ninguna versión final está identificada como vigente."
                .into();
        assert_eq!(
            spanish_negation_trace.qualify_temporal_evidence(
                "¿Cuál es la fecha vigente del proyecto Girasol?",
                std::slice::from_ref(&spanish_memory)
            ),
            1,
            "a Spanish negated marker cannot satisfy a current-date query"
        );
        assert_eq!(
            spanish_negation_trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Partial
        );
    }

    #[test]
    fn v2_temporal_guard_is_scoped_to_the_requested_slot_in_multi_slot_queries() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/revision-matrix-v1.json"
        ))
        .expect("valid synthetic slot-evidence-v2 revision matrix");
        let case = bank
            .cases
            .into_iter()
            .find(|case| case.id == "revision-03")
            .expect("current-date contrast exists");
        let mut memory = candidate(&case.candidate.summary);
        memory.memory_id = case.candidate.memory_id.clone();
        memory.revision = case.candidate.revision;
        memory.summary = "For project Atlas (ID AT-17), the final version lists the launch date as 19 May. For project Atlas (ID AT-17), the organization was founded in 1998.".into();
        let query = "For project Atlas (ID AT-17), what is the current final launch date and in what year was the organization founded?";
        let mut trace = case.trace;
        trace.query_anchors[0].subject = Some("Atlas".into());
        trace.query_anchors[0].stable_id = "AT-17".into();
        trace.query_anchors[0].quote = "project Atlas (ID AT-17)".into();
        trace.slots[0].quote = "what is the current final launch date".into();
        trace.slots.push(QueryTarget {
            id: "founded_year".into(),
            dimension: "requested_slot".into(),
            quote: "in what year was the organization founded".into(),
        });

        trace.candidates[0].identity.stable_id_quote = Some("AT-17".into());
        trace.candidates[0].identity.subject_quote = Some("Atlas (ID AT-17)".into());
        let launch_date = &mut trace.candidates[0].propositions[0];
        launch_date.quote =
            "For project Atlas (ID AT-17), the final version lists the launch date as 19 May."
                .into();
        launch_date.subject_quote = "Atlas (ID AT-17)".into();
        launch_date.target_id = "date".into();
        launch_date.relation = EvidenceRelation::Answers;
        let mut founding_year = launch_date.clone();
        founding_year.occurrence_id = "p2".into();
        founding_year.quote =
            "For project Atlas (ID AT-17), the organization was founded in 1998.".into();
        founding_year.target_id = "founded_year".into();
        founding_year.relation = EvidenceRelation::Answers;
        trace.candidates[0].propositions.push(founding_year);

        assert_eq!(
            trace.qualify_temporal_evidence(query, std::slice::from_ref(&memory)),
            1,
            "the current/final qualifier should affect the date slot, not the independent founding year"
        );
        assert_eq!(
            trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Partial
        );
        assert_eq!(
            trace.candidates[0].propositions[1].relation,
            EvidenceRelation::Answers
        );

        let mut omitted_qualifier = trace.clone();
        omitted_qualifier.slots[0].quote = "launch date".into();
        omitted_qualifier.slots[1].quote = "organization founded".into();
        for proposition in &mut omitted_qualifier.candidates[0].propositions {
            proposition.relation = EvidenceRelation::Answers;
        }
        assert_eq!(
            omitted_qualifier.qualify_temporal_evidence(query, std::slice::from_ref(&memory)),
            2,
            "if extraction drops all temporal wording, retain the conservative global guard"
        );
        assert!(
            omitted_qualifier.candidates[0]
                .propositions
                .iter()
                .all(|proposition| proposition.relation == EvidenceRelation::Partial)
        );

        let mut partially_omitted = trace.clone();
        partially_omitted.slots[0].quote = "final launch date".into();
        partially_omitted.slots[1].quote = "organization founded".into();
        for proposition in &mut partially_omitted.candidates[0].propositions {
            proposition.relation = EvidenceRelation::Answers;
        }
        assert_eq!(
            partially_omitted.qualify_temporal_evidence(query, std::slice::from_ref(&memory)),
            2,
            "if only some query-level temporal qualifiers survive extraction, keep the global fallback"
        );
        assert!(
            partially_omitted.candidates[0]
                .propositions
                .iter()
                .all(|proposition| proposition.relation == EvidenceRelation::Partial)
        );

        assert!(
            trace.validate(query, std::slice::from_ref(&memory)),
            "validation error: {:?}",
            trace.validation_error(query, std::slice::from_ref(&memory))
        );
        assert_eq!(
            trace.selected_ids(std::slice::from_ref(&memory), query),
            vec![case.candidate.memory_id],
            "the record with one temporally qualified and one independent useful slot stays selectable"
        );
    }

    #[test]
    fn v2_slot_cannot_be_answered_by_a_different_explicit_query_entity() {
        let summary =
            "Project Atlas AT-17 is mentioned. Project Boreal BR-20 is owned by Ana Ruiz.";
        let memory = candidate(summary);
        let query = "For project Atlas AT-17 and project Boreal BR-20, who owns Boreal?";
        let trace = SlotEvidenceTrace {
            version: "slot-evidence-v2".into(),
            query_anchors: vec![
                QueryAnchor {
                    id: "atlas".into(),
                    kind: "project".into(),
                    subject: Some("Atlas".into()),
                    stable_id: "AT-17".into(),
                    quote: "project Atlas AT-17".into(),
                },
                QueryAnchor {
                    id: "boreal".into(),
                    kind: "project".into(),
                    subject: Some("Boreal".into()),
                    stable_id: "BR-20".into(),
                    quote: "project Boreal BR-20".into(),
                },
            ],
            slots: vec![QueryTarget {
                id: "owner".into(),
                dimension: "requested_slot".into(),
                quote: "who owns Boreal".into(),
            }],
            premises: vec![],
            candidates: vec![CandidateEvidence {
                memory_id: "m1".into(),
                revision: 1,
                identity: IdentityAssessment {
                    status: IdentityStatus::Match,
                    anchor_id: Some("atlas".into()),
                    stable_id_quote: Some("AT-17".into()),
                    subject_quote: Some("Atlas AT-17".into()),
                },
                propositions: vec![Proposition {
                    occurrence_id: "p1".into(),
                    quote: summary.into(),
                    subject_quote: "Atlas AT-17".into(),
                    identity_anchor_id: Some("atlas".into()),
                    predicate_quote: String::new(),
                    value: None,
                    polarity_quote: None,
                    modality_quote: None,
                    event_quote: None,
                    time_quote: None,
                    conditions_quote: None,
                    relation: EvidenceRelation::Answers,
                    target_id: "owner".into(),
                }],
            }],
        };

        assert!(!trace.validate(query, std::slice::from_ref(&memory)));
        assert!(
            trace
                .selected_ids(std::slice::from_ref(&memory), query)
                .is_empty(),
            "Atlas identity in a separate sentence must not authorize a Boreal slot answer"
        );

        let mut corrected = trace.clone();
        let proposition = &mut corrected.candidates[0].propositions[0];
        proposition.quote = "Project Boreal BR-20 is owned by Ana Ruiz.".into();
        proposition.subject_quote = "Boreal BR-20".into();
        proposition.identity_anchor_id = Some("boreal".into());
        assert!(corrected.validate(query, std::slice::from_ref(&memory)));
        assert_eq!(
            corrected.selected_ids(std::slice::from_ref(&memory), query),
            vec![memory.memory_id.clone()],
            "a proposition bound to the Boreal slot owner remains selectable even when candidate identity is Atlas"
        );

        let mut normalized = trace;
        assert_eq!(normalized.demote_mismatched_target_attributions(), 1);
        assert!(normalized.validate(query, std::slice::from_ref(&memory)));
        assert_eq!(
            normalized.candidates[0].propositions[0].relation,
            EvidenceRelation::UnresolvedAttribution
        );
        assert_eq!(
            normalized.candidates[0].propositions[0].quote, summary,
            "the source quote remains available as unresolved evidence"
        );
        assert!(
            normalized
                .selected_ids(std::slice::from_ref(&memory), query)
                .is_empty()
        );
    }

    #[test]
    fn v2_multi_anchor_slot_abstains_when_ambiguous_and_keeps_isolated_subject() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/multi-anchor-target-v1.json"
        ))
        .expect("valid synthetic multi-anchor target matrix");
        assert!(bank.synthetic);
        assert_eq!(bank.cases.len(), 2);

        for mut case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace
                    .validate(&case.query, std::slice::from_ref(&memory)),
                case.expected_valid,
                "{} raw validation error={:?}",
                case.id,
                case.trace
                    .validation_error(&case.query, std::slice::from_ref(&memory))
            );
            assert_eq!(
                case.trace.demote_mismatched_target_attributions(),
                case.expected_demoted,
                "{} demoted propositions",
                case.id
            );
            assert!(
                case.trace
                    .validate(&case.query, std::slice::from_ref(&memory)),
                "{} normalized trace must remain structurally valid",
                case.id
            );
            assert_eq!(
                case.trace
                    .selected_ids(std::slice::from_ref(&memory), &case.query),
                case.expected_selected,
                "{} selected IDs after normalization",
                case.id
            );
            if case.expected_demoted > 0 {
                let proposition = &case.trace.candidates[0].propositions[0];
                assert_eq!(
                    proposition.relation,
                    EvidenceRelation::UnresolvedAttribution
                );
                assert!(
                    !proposition.quote.is_empty(),
                    "source quote must be preserved"
                );
            }
        }
    }

    #[test]
    fn v2_literal_binding_known_false_acceptance_for_superseded_draft() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r22.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r22 holdout bank");
        let case = bank
            .cases
            .into_iter()
            .find(|case| case.id == "holdout-08")
            .expect("observatory revision control exists");
        let mut memory = candidate(&case.candidate.summary);
        memory.memory_id = case.candidate.memory_id.clone();
        memory.revision = case.candidate.revision;

        let stale_quote = "Author Sol (S-118)'s earlier draft proposed Valparaíso";
        let mut trace = case.trace;
        let evidence = &mut trace.candidates[0];
        evidence.identity.subject_quote = Some("Author Sol (S-118)".into());
        evidence.propositions[0].quote = stale_quote.into();
        evidence.propositions[0].subject_quote = "Author Sol (S-118)".into();

        assert_eq!(
            trace.validate(&case.query, std::slice::from_ref(&memory)),
            true
        );
        assert_eq!(
            trace.selected_ids(&[memory], &case.query),
            vec![case.candidate.memory_id],
            "literal identity binding does not establish that a cited draft is current"
        );
    }

    #[test]
    fn r19_retrospective_overlay_preserves_premise_corrections_without_relabeling_bank() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r19.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r19 bank");
        let overlay: LocalRetrospectiveOverlay = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r19-retrospective-overlay.json"
        ))
        .expect("valid retrospective annotation overlay");
        assert_eq!(
            overlay.version,
            "slot-evidence-v2-r19-retrospective-overlay-v1"
        );
        assert_eq!(overlay.source_bank, "local-bank-r19.json");
        assert_eq!(overlay.source_bank_sha256.len(), 64);
        assert_eq!(bank.cases.len(), 50);
        assert_eq!(overlay.annotations.len(), 6);

        let mut frozen_selected = 0;
        let mut frozen_abstained = 0;
        for case in &bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            let mut trace = case.trace.clone();
            trace.demote_ambiguous_attributions(&case.query);
            let selected = trace.selected_ids(&[memory], &case.query);
            if case.expected_selected.is_empty() {
                frozen_abstained += 1;
                assert!(selected.is_empty(), "case {}", case.id);
            } else {
                frozen_selected += 1;
                assert_eq!(selected, case.expected_selected, "case {}", case.id);
            }
        }
        assert_eq!(frozen_selected, 22);
        assert_eq!(frozen_abstained, 28);

        for annotation in overlay.annotations {
            let case_id = annotation.case_id;
            assert_eq!(annotation.relation, "refutes_premise", "case {case_id}");
            assert_eq!(
                annotation.premise_dimension, "recipient",
                "case {case_id}: retrospective ontology label"
            );
            let case = bank
                .cases
                .iter()
                .find(|case| case.id == case_id)
                .unwrap_or_else(|| panic!("missing retrospective case {case_id}"));
            assert!(
                case.expected_selected.is_empty(),
                "case {case_id} stays frozen"
            );
            assert!(
                case.query.contains(&annotation.premise_quote),
                "case {case_id}: overlay quote must be literal in the query"
            );
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;

            let mut correction = case.trace.clone();
            correction.premises.push(QueryTarget {
                id: "recipient-premise".into(),
                dimension: "correction_premise".into(),
                quote: annotation.premise_quote,
            });
            let proposition = &mut correction.candidates[0].propositions[0];
            proposition.relation = EvidenceRelation::Refutes;
            proposition.target_id = "recipient-premise".into();
            assert!(
                correction.validate(&case.query, std::slice::from_ref(&memory)),
                "case {case_id} correction trace invalid: {:?}",
                correction.validation_error(&case.query, &[memory.clone()])
            );
            assert_eq!(
                correction.demote_ambiguous_attributions(&case.query),
                0,
                "case {case_id}: a grounded refutation must survive the answer-slot exclusion guard"
            );
            assert_eq!(
                correction.selected_ids(std::slice::from_ref(&memory), &case.query),
                vec![case.candidate.memory_id.clone()],
                "case {case_id}"
            );

            let mut false_answer = correction;
            let proposition = &mut false_answer.candidates[0].propositions[0];
            proposition.relation = EvidenceRelation::Answers;
            proposition.target_id = "answer".into();
            assert_eq!(
                false_answer.demote_ambiguous_attributions(&case.query),
                1,
                "case {case_id}: the same sentence must not answer the excluded-recipient slot"
            );
            assert_eq!(
                false_answer.candidates[0].propositions[0].relation,
                EvidenceRelation::UnresolvedAttribution,
                "case {case_id}"
            );
            assert!(
                false_answer.selected_ids(&[memory], &case.query).is_empty(),
                "case {case_id}"
            );
        }
    }

    #[test]
    fn conservative_abstention_bank_r4_rejects_foreign_facts_without_losing_useful_cases() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r4.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r4 local bank");
        assert_eq!(bank.cases.len(), 20);
        let mut demoted_relative = 0;
        let mut lost_useful = Vec::new();
        let mut useful_count = 0;
        let mut selected_useful_count = 0;
        let mut false_selections = 0;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            let mut trace = case.trace.clone();
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            if case.expected_error.as_deref() == Some("proposition_foreign_relative_clause") {
                assert_eq!(demoted, 1, "case {}", case.id);
                demoted_relative += 1;
            }
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {}",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);
            if !case.expected_selected.is_empty() {
                useful_count += 1;
                if selected == case.expected_selected {
                    selected_useful_count += 1;
                } else {
                    lost_useful.push(case.id.clone());
                }
            } else if !selected.is_empty() {
                false_selections += 1;
            }
            if demoted > 0 {
                assert!(trace.candidates[0].propositions.iter().any(|proposition| {
                    proposition.relation == EvidenceRelation::UnresolvedAttribution
                        && proposition.target_id.is_empty()
                }));
            }
        }

        assert_eq!(demoted_relative, 9);
        assert!(lost_useful.is_empty(), "lost useful cases: {lost_useful:?}");
        assert_eq!(useful_count, 7);
        assert_eq!(selected_useful_count, 7);
        assert_eq!(false_selections, 0);
    }

    #[test]
    fn conservative_abstention_bank_r5_closes_possessive_subject_escape() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r5.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r5 local bank");
        assert_eq!(bank.cases.len(), 21);
        let mut useful_count = 0;
        let mut selected_useful_count = 0;
        let mut false_selections = 0;
        let mut demoted = 0;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            let raw_valid = case.trace.validate(&case.query, &[memory.clone()]);
            assert_eq!(
                raw_valid, case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted_count = trace.demote_ambiguous_attributions(&case.query);
            demoted += demoted_count;
            if case.id == "manager-possessive-does-not-rescue-speech-complement" {
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted_count, 1);
                let proposition = &trace.candidates[0].propositions[0];
                assert_eq!(
                    proposition.relation,
                    EvidenceRelation::UnresolvedAttribution
                );
                assert!(proposition.target_id.is_empty());
                assert!(proposition.quote.contains("'s manager told"));
            }
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted_count > 0,
                "case {}",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);
            if !case.expected_selected.is_empty() {
                useful_count += 1;
                if selected == case.expected_selected {
                    selected_useful_count += 1;
                }
            } else {
                if !raw_selected.is_empty() {
                    assert!(
                        demoted_count > 0,
                        "case {} baseline extra must be explained",
                        case.id
                    );
                }
                if !selected.is_empty() {
                    false_selections += 1;
                }
            }
        }

        assert_eq!(useful_count, 7);
        assert_eq!(selected_useful_count, 7);
        assert_eq!(false_selections, 0);
        assert!(demoted >= 11);
    }

    #[test]
    fn r6_rejects_prepositional_speech_subject_and_preserves_useful_complements() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r6.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r6 local bank");
        assert_eq!(bank.cases.len(), 22);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra = 0;
        let mut extra_ids = Vec::new();
        let mut attacked_case_seen = false;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw trace validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed trace validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id == "assistant-prepositional-identity-does-not-rescue-complement" {
                attacked_case_seen = true;
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted, 1);
                assert!(selected.is_empty());
                assert_eq!(
                    trace.candidates[0].propositions[0].relation,
                    EvidenceRelation::UnresolvedAttribution
                );
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra += 1;
                extra_ids.push(case.id);
            }
        }

        assert!(attacked_case_seen);
        assert_eq!(useful, 7);
        assert_eq!(retained_useful, 7);
        assert_eq!(extra, 0, "false selections: {extra_ids:?}");
    }

    #[test]
    fn r7_clause_local_subject_gate_handles_long_roles_and_contextual_prefixes() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r7.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r7 local bank");
        assert_eq!(bank.cases.len(), 24);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra = 0;
        let mut extra_ids = Vec::new();
        let mut long_role_seen = false;
        let mut contextual_subject_seen = false;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id == "long-prepositional-role-does-not-rescue-complement" {
                long_role_seen = true;
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted, 1);
                assert!(selected.is_empty());
            }
            if case.id == "clause-initial-subject-after-contextual-comma-remains-useful" {
                contextual_subject_seen = true;
                assert_eq!(selected, case.expected_selected);
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra += 1;
                extra_ids.push(case.id);
            }
        }

        assert!(long_role_seen && contextual_subject_seen);
        assert_eq!(useful, 8);
        assert_eq!(retained_useful, 8);
        assert_eq!(extra, 0, "false selections: {extra_ids:?}");
    }

    #[test]
    fn r8_rejects_passive_speech_and_keeps_bounded_subject_modifiers() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r8.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r8 local bank");
        assert_eq!(bank.cases.len(), 27);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut passive_case_seen = false;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id == "passive-told-by-foreign-recipient-does-not-rescue" {
                passive_case_seen = true;
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted, 1);
                assert!(selected.is_empty());
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert!(passive_case_seen);
        assert_eq!(useful, 10);
        assert_eq!(retained_useful, 10);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r9_rejects_colon_imperative_and_preserves_past_perfect_complements() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r9.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r9 local bank");
        assert_eq!(bank.cases.len(), 30);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut imperative_seen = false;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id == "colon-imperative-does-not-rescue-speech-complement" {
                imperative_seen = true;
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted, 1);
                assert!(selected.is_empty());
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert!(imperative_seen);
        assert_eq!(useful, 12);
        assert_eq!(retained_useful, 12);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r10_requires_the_query_to_ask_about_the_bound_identity_as_speaker() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r10.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r10 local bank");
        assert_eq!(bank.cases.len(), 32);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut manager_query_cases = 0;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case
                .id
                .contains("query-targets-manager-not-identity-speech")
            {
                manager_query_cases += 1;
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted, 1);
                assert!(selected.is_empty());
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert_eq!(manager_query_cases, 2);
        assert_eq!(useful, 12);
        assert_eq!(retained_useful, 12);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r11_rejects_comma_vocative_and_preserves_firmly_modified_speech() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r11.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r11 local bank");
        assert_eq!(bank.cases.len(), 34);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut imperative_seen = false;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id == "comma-vocative-imperative-does-not-rescue" {
                imperative_seen = true;
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted, 1);
                assert!(selected.is_empty());
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert!(imperative_seen);
        assert_eq!(useful, 13);
        assert_eq!(retained_useful, 13);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r12_distinguishes_recipient_from_topic_and_keeps_time_asides() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r12.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r12 local bank");
        assert_eq!(bank.cases.len(), 36);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut topic_case_seen = false;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id == "foreign-person-as-topic-not-speech-recipient" {
                topic_case_seen = true;
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted, 1);
                assert!(selected.is_empty());
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert!(topic_case_seen);
        assert_eq!(useful, 14);
        assert_eq!(retained_useful, 14);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r13_blocks_explicit_recipient_exclusion_and_keeps_bounded_time_asides() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r13.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r13 local bank");
        assert_eq!(bank.cases.len(), 38);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut exclusion_case_seen = false;
        let mut aside_case_seen = false;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id == "explicitly-excluded-recipient-does-not-rescue" {
                exclusion_case_seen = true;
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted, 1);
                assert!(selected.is_empty());
            }
            if case.id == "recipient-with-bounded-time-aside-remains-useful" {
                aside_case_seen = true;
                assert_eq!(selected, case.expected_selected);
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert!(exclusion_case_seen && aside_case_seen);
        assert_eq!(useful, 15);
        assert_eq!(retained_useful, 15);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r14_blocks_spanish_recipient_exclusion_and_keeps_monday_asides() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r14.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r14 local bank");
        assert_eq!(bank.cases.len(), 41);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut exclusion_case_seen = false;
        let mut aside_cases_seen = 0;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id == "explicit-spanish-recipient-exclusion-does-not-rescue" {
                exclusion_case_seen = true;
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted, 1);
                assert!(selected.is_empty());
            }
            if case.id.contains("aside-remains-useful")
                && (case.id.contains("monday") || case.id.contains("lunes"))
            {
                aside_cases_seen += 1;
                assert_eq!(selected, case.expected_selected);
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert!(exclusion_case_seen);
        assert_eq!(aside_cases_seen, 2);
        assert_eq!(useful, 17);
        assert_eq!(retained_useful, 17);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r16_demotes_parenthetical_and_dashed_exclusions_when_marker_assessment_is_inconclusive() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r16.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r16 local bank");
        assert_eq!(bank.cases.len(), 45);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut explicit_exclusions_seen = 0;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let raw_selected = case.trace.selected_ids(&[memory.clone()], &case.query);
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id.contains("excluded-recipient")
                || case.id.contains("other-than-recipient")
                || case.id.contains("exclusion-does-not-rescue")
            {
                explicit_exclusions_seen += 1;
                assert_eq!(raw_selected, vec![case.candidate.memory_id.clone()]);
                assert_eq!(demoted, 1, "case {} exclusion demotion", case.id);
                assert!(selected.is_empty(), "case {} remained selected", case.id);
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert_eq!(explicit_exclusions_seen, 6);
        assert_eq!(useful, 17);
        assert_eq!(retained_useful, 17);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn recipient_exclusion_guard_does_not_capture_a_nested_infinitive() {
        assert!(explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, not contractor Bob B-29, on Monday, that rain was expected.",
            "E-17"
        ));
        assert!(explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, but not contractor Bob B-29, on Monday, that rain was expected.",
            "E-17"
        ));
        assert!(explicitly_excluded_speech_recipient(
            "Employee Ana E-17 le dijo al equipo, salvo el contratista Bob B-29, ayer que estaría ocupada.",
            "E-17"
        ));
        assert!(explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, other than contractor Bob B-29, that rain was expected.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told Bob B-29 not to tell contractor Carl C-30 that rain was expected.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, no later than the deadline from contractor Bob B-29, that the report was ready.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, no more than 2 contractors including contractor Bob B-29, on Monday, that rain was expected.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, not merely contractor Bob B-29, but also contractor Carl C-30, on Monday, that rain was expected.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, no one but contractor Bob B-29, on Monday, that rain was expected.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, notably contractor Bob B-29, on Monday, that rain was expected.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, exceptional contractor Bob B-29, on Monday, that rain was expected.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, instead of contractor Bob B-29, that the group changed plans.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, except when contractor Bob B-29 was on leave, that the schedule changed.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 told the team, except if contractor Bob B-29 was away, that the schedule changed.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 le dijo al equipo, salvo cuando el contratista Bob B-29 estaba de licencia, que el horario cambió.",
            "E-17"
        ));
        assert!(!explicitly_excluded_speech_recipient(
            "Employee Ana E-17 le dijo al equipo, salvo que el contratista Bob B-29 estuviera de licencia, que el horario cambió.",
            "E-17"
        ));
    }

    #[test]
    fn excluding_a_third_party_does_not_veto_the_queried_recipient_answer() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r21.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r21 local bank");
        let mut case = bank
            .cases
            .into_iter()
            .find(|case| case.id == "useful-complement-about-foreign-person")
            .expect("the accepted positive is frozen in the r21 bank");
        let query = case.query.clone();
        let quote = "Employee Ana Ruiz E-17 told contractor Bob Torres B-29, not contractor Alice A-30, that rain was expected tomorrow.";
        case.candidate.summary = quote.into();
        case.trace.candidates[0].propositions[0].quote = quote.into();

        assert!(explicitly_excluded_speech_recipient(quote, "E-17"));
        assert!(!explicitly_excluded_speech_recipient_for_query(
            quote,
            "E-17",
            &query,
            "employee Ana Ruiz E-17",
            "employee"
        ));
        assert!(!is_third_party_exclusion_aside(
            "not contractor Alice A-30, who was on the team, ",
            "B-29"
        ));
        assert_eq!(case.trace.demote_ambiguous_attributions(&query), 0);
        assert_eq!(
            case.trace.candidates[0].propositions[0].relation,
            EvidenceRelation::Answers
        );
        let memory = candidate(&case.candidate.summary);
        let mut memory = memory;
        memory.memory_id = case.candidate.memory_id;
        memory.revision = case.candidate.revision;
        assert!(case.trace.validate(&query, std::slice::from_ref(&memory)));
        assert_eq!(
            case.trace.selected_ids(&[memory], &query),
            case.expected_selected
        );
    }

    #[test]
    fn excluding_the_queried_recipient_still_blocks_the_answer() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r21.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r21 local bank");
        let mut case = bank
            .cases
            .into_iter()
            .find(|case| case.id == "useful-complement-about-foreign-person")
            .expect("the accepted positive is frozen in the r21 bank");
        let quote = "Employee Ana Ruiz E-17 told contractor Alice A-30, not contractor Bob Torres B-29, that rain was expected tomorrow.";
        case.candidate.summary = quote.into();
        case.trace.candidates[0].propositions[0].quote = quote.into();
        let memory = candidate(&case.candidate.summary);
        let mut memory = memory;
        memory.memory_id = case.candidate.memory_id;
        memory.revision = case.candidate.revision;

        assert_eq!(case.trace.demote_ambiguous_attributions(&case.query), 1);
        assert_eq!(
            case.trace.candidates[0].propositions[0].relation,
            EvidenceRelation::UnresolvedAttribution
        );
        assert!(
            case.trace
                .validate(&case.query, std::slice::from_ref(&memory))
        );
        assert!(case.trace.selected_ids(&[memory], &case.query).is_empty());
    }

    #[test]
    fn queried_recipient_exclusion_does_not_preserve_a_refutation_about_someone_else() {
        let query = "What did employee Ana Ruiz E-17 tell contractor Bob Torres B-29?";
        let quote = "Employee Ana Ruiz E-17 told the team, not contractor Alice A-30, that rain was expected tomorrow.";
        let memory = candidate(quote);
        let mut evidence = trace(IdentityStatus::Match, quote, "Ana Ruiz E-17");
        evidence.version = "slot-evidence-v2".into();
        evidence.slots[0].quote = "What did".into();
        evidence.query_anchors[0].quote = "employee Ana Ruiz E-17".into();
        evidence.premises = vec![QueryTarget {
            id: "recipient".into(),
            dimension: "correction_premise".into(),
            quote: "Ana told Alice A-30".into(),
        }];
        evidence.candidates[0].propositions[0].relation = EvidenceRelation::Refutes;
        evidence.candidates[0].propositions[0].target_id = "recipient".into();

        assert!(!query_requests_speech_recipient(
            query,
            "E-17",
            "A-30",
            "employee Ana Ruiz E-17",
            "employee"
        ));
        for nested_query in [
            "Who asked employee Ana Ruiz E-17 to tell contractor Bob Torres B-29?",
            "What did the manager ask, employee Ana Ruiz E-17, to tell contractor Bob Torres B-29?",
            "What did the manager ask—employee Ana Ruiz E-17—to tell contractor Bob Torres B-29?",
        ] {
            assert!(!query_requests_speech_recipient(
                nested_query,
                "E-17",
                "B-29",
                "employee Ana Ruiz E-17",
                "employee"
            ));
            assert!(!query_targets_speech_complement(
                nested_query,
                "employee Ana Ruiz E-17 told contractor Bob Torres B-29 that rain was expected.",
                "E-17",
                "Ana Ruiz E-17",
                "employee"
            ));
        }
        assert_eq!(evidence.demote_ambiguous_attributions(query), 1);
        assert!(evidence.selected_ids(&[memory], query).is_empty());
    }

    #[test]
    fn nested_request_does_not_turn_a_third_party_exclusion_into_a_refutation() {
        let query =
            "What did the manager ask, employee Ana Ruiz E-17, to tell contractor Bob Torres B-29?";
        let quote = "Employee Ana Ruiz E-17 told the team, not contractor Bob Torres B-29, that rain was expected tomorrow.";
        let memory = candidate(quote);
        let mut evidence = trace(IdentityStatus::Match, quote, "Ana Ruiz E-17");
        evidence.version = "slot-evidence-v2".into();
        evidence.query_anchors[0].quote = "employee Ana Ruiz E-17".into();
        evidence.slots[0].quote = "What did".into();
        evidence.premises = vec![QueryTarget {
            id: "recipient".into(),
            dimension: "correction_premise".into(),
            quote: "employee Ana Ruiz E-17, to tell contractor Bob Torres B-29".into(),
        }];
        evidence.candidates[0].propositions[0].relation = EvidenceRelation::Refutes;
        evidence.candidates[0].propositions[0].target_id = "recipient".into();

        assert!(!query_requests_speech_recipient(
            query,
            "E-17",
            "B-29",
            "employee Ana Ruiz E-17",
            "employee"
        ));
        assert_eq!(evidence.demote_ambiguous_attributions(query), 1);
        assert_eq!(
            evidence.candidates[0].propositions[0].relation,
            EvidenceRelation::UnresolvedAttribution
        );
        assert!(evidence.validate(query, std::slice::from_ref(&memory)));
        assert!(evidence.selected_ids(&[memory], query).is_empty());
    }

    #[test]
    fn r17_preserves_an_informed_that_complement_without_changing_r16() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r17.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r17 local bank");
        assert_eq!(bank.cases.len(), 46);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut informed_case_seen = false;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id == "useful-informed-complement-about-foreign-person" {
                informed_case_seen = true;
                assert_eq!(demoted, 0);
                assert_eq!(selected, case.expected_selected);
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert!(informed_case_seen);
        assert_eq!(useful, 18);
        assert_eq!(retained_useful, 18);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r18_preserves_advised_complements_and_role_contrast_asides() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r18.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r18 local bank");
        assert_eq!(bank.cases.len(), 49);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut new_cases_seen = 0;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id.starts_with("useful-advised-") || case.id.starts_with("useful-role-") {
                new_cases_seen += 1;
                assert_eq!(demoted, 0, "case {} was unnecessarily demoted", case.id);
                assert_eq!(
                    selected, case.expected_selected,
                    "case {} selection",
                    case.id
                );
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert_eq!(new_cases_seen, 3);
        assert_eq!(useful, 21);
        assert_eq!(retained_useful, 21);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r19_preserves_an_advised_complement_after_lunch_aside() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r19.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r19 local bank");
        assert_eq!(bank.cases.len(), 50);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut after_lunch_case_seen = false;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity",
                case.id
            );
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);

            if case.id == "useful-advised-complement-after-lunch-aside" {
                after_lunch_case_seen = true;
                assert_eq!(demoted, 0);
                assert_eq!(selected, case.expected_selected);
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert!(after_lunch_case_seen);
        assert_eq!(useful, 22);
        assert_eq!(retained_useful, 22);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r20_binds_independent_subject_quotes_to_the_query_without_losing_bank_cases() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r20.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r20 local bank");
        assert_eq!(bank.cases.len(), 52);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut context_variation_seen = false;
        let mut wrong_subject_seen = false;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity error={:?}",
                case.id,
                case.trace.validation_error(&case.query, &[memory.clone()])
            );
            if let Some(expected_error) = case.expected_error.as_deref() {
                assert_eq!(
                    case.trace.validation_error(&case.query, &[memory.clone()]),
                    Some(expected_error),
                    "case {} validation error",
                    case.id
                );
            }
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);
            assert_eq!(
                selected, case.expected_selected,
                "case {} selection",
                case.id
            );

            match case.id.as_str() {
                "context-variation-same-query-subject" => context_variation_seen = true,
                "same-id-wrong-query-subject" => wrong_subject_seen = true,
                _ => {}
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert!(context_variation_seen);
        assert!(wrong_subject_seen);
        assert_eq!(useful, 23);
        assert_eq!(retained_useful, 23);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    #[test]
    fn r21_rejects_cooccurring_foreign_ids_and_preserves_literal_name_controls() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r21.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r21 local bank");
        assert_eq!(bank.cases.len(), 58);
        let mut useful = 0;
        let mut retained_useful = 0;
        let mut extra_ids = Vec::new();
        let mut adversarial_controls = 0;

        for case in bank.cases {
            let mut memory = candidate(&case.candidate.summary);
            memory.memory_id = case.candidate.memory_id.clone();
            memory.revision = case.candidate.revision;
            assert_eq!(
                case.trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid,
                "case {} raw validity error={:?}",
                case.id,
                case.trace.validation_error(&case.query, &[memory.clone()])
            );
            if let Some(expected_error) = case.expected_error.as_deref() {
                assert_eq!(
                    case.trace.validation_error(&case.query, &[memory.clone()]),
                    Some(expected_error),
                    "case {} validation error",
                    case.id
                );
            }
            let mut trace = case.trace;
            let demoted = trace.demote_ambiguous_attributions(&case.query);
            assert_eq!(
                trace.validate(&case.query, &[memory.clone()]),
                case.expected_valid || demoted > 0,
                "case {} transformed validity",
                case.id
            );
            let selected = trace.selected_ids(&[memory], &case.query);
            assert_eq!(
                selected, case.expected_selected,
                "case {} selection",
                case.id
            );
            match case.id.as_str() {
                "same-query-name-but-id-belongs-to-cooccurring-person"
                | "same-query-name-but-id-in-bob-factual-clause"
                | "hyphenated-query-name-keeps-literal-boundary"
                | "entity-name-may-contain-the-declared-kind"
                | "multiword-domain-kind-retains-useful-memory"
                | "ambiguous-name-kind-order-abstains" => adversarial_controls += 1,
                _ => {}
            }
            if !case.expected_selected.is_empty() {
                useful += 1;
                if selected == case.expected_selected {
                    retained_useful += 1;
                }
            } else if !selected.is_empty() {
                extra_ids.push(case.id);
            }
        }

        assert_eq!(adversarial_controls, 6);
        assert_eq!(useful, 26);
        assert_eq!(retained_useful, 26);
        assert!(extra_ids.is_empty(), "false selections: {extra_ids:?}");
    }

    fn candidate(summary: &str) -> Candidate {
        Candidate {
            memory_id: "m1".into(),
            revision: 1,
            summary: summary.into(),
            conditions: None,
            source: Some(SourceRef {
                kind: "test".into(),
                reference: "fixture".into(),
                locator: None,
            }),
            state: "active".into(),
            bm25: 0.0,
        }
    }

    fn trace(identity: IdentityStatus, quote: &str, subject: &str) -> SlotEvidenceTrace {
        SlotEvidenceTrace {
            version: "slot-evidence-v1".into(),
            query_anchors: vec![QueryAnchor {
                id: "person".into(),
                kind: "employee".into(),
                subject: Some("Ana Ruiz".into()),
                stable_id: "E-17".into(),
                quote: "Ana Ruiz employee E-17".into(),
            }],
            slots: vec![QueryTarget {
                id: "city".into(),
                dimension: "city".into(),
                quote: "which city".into(),
            }],
            premises: vec![],
            candidates: vec![CandidateEvidence {
                memory_id: "m1".into(),
                revision: 1,
                identity: IdentityAssessment {
                    status: identity,
                    anchor_id: (identity == IdentityStatus::Match).then(|| "person".into()),
                    stable_id_quote: (identity == IdentityStatus::Match).then(|| "E-17".into()),
                    subject_quote: (identity == IdentityStatus::Match)
                        .then(|| "Ana Ruiz E-17".into()),
                },
                propositions: vec![Proposition {
                    occurrence_id: "p1".into(),
                    quote: quote.into(),
                    subject_quote: subject.into(),
                    identity_anchor_id: (identity == IdentityStatus::Match)
                        .then(|| "person".into()),
                    predicate_quote: "lives in".into(),
                    value: Some("Quito".into()),
                    polarity_quote: None,
                    modality_quote: None,
                    event_quote: None,
                    time_quote: None,
                    conditions_quote: None,
                    relation: EvidenceRelation::Answers,
                    target_id: "city".into(),
                }],
            }],
        }
    }

    #[test]
    fn matched_stable_id_and_literal_fact_can_be_selected() {
        let memory = candidate("employee Ana Ruiz E-17 lives in Quito");
        let evidence = trace(
            IdentityStatus::Match,
            "employee Ana Ruiz E-17 lives in Quito",
            "Ana Ruiz E-17",
        );
        assert!(evidence.validate("which city for Ana Ruiz employee E-17", &[memory.clone()]));
        assert_eq!(
            evidence.selected_ids(&[memory], "which city for Ana Ruiz employee E-17"),
            vec!["m1"]
        );
    }

    #[test]
    fn query_anchor_quote_must_bind_subject_kind_and_stable_id_together() {
        let memory = candidate("employee Ana Ruiz E-17 lives in Quito");
        let mut evidence = trace(
            IdentityStatus::Match,
            "employee Ana Ruiz E-17 lives in Quito",
            "Ana Ruiz E-17",
        );
        evidence.query_anchors[0].quote = "Ana Ruiz".into();
        assert!(!evidence.validate("which city for Ana Ruiz employee E-17", &[memory]));
    }

    #[test]
    fn v2_subject_binding_requires_direct_name_id_link_and_preserves_literal_names() {
        let anchors = vec![QueryAnchor {
            id: "person".into(),
            kind: "engineer".into(),
            subject: Some("Ana Ruiz".into()),
            stable_id: "E-17".into(),
            quote: "engineer Ana Ruiz E-17".into(),
        }];
        assert!(subject_quote_matches_anchor(
            "Ana Ruiz E-17",
            &anchors[0],
            &anchors
        ));
        let punctuated_label = QueryAnchor {
            id: "platform".into(),
            kind: "platform".into(),
            subject: Some("Yahoo!".into()),
            stable_id: "Y-1".into(),
            quote: "platform Yahoo! Y-1".into(),
        };
        assert!(subject_quote_matches_anchor(
            "Yahoo! Y-1",
            &punctuated_label,
            std::slice::from_ref(&punctuated_label)
        ));
        assert!(!subject_quote_matches_anchor(
            "Yahoo Y-1",
            &punctuated_label,
            std::slice::from_ref(&punctuated_label)
        ));
        let punctuated_name = QueryAnchor {
            id: "person".into(),
            kind: "employee".into(),
            subject: Some("Ana!".into()),
            stable_id: "E-17".into(),
            quote: "employee Ana! E-17".into(),
        };
        assert!(!subject_quote_matches_anchor(
            "Ana E-17",
            &punctuated_name,
            std::slice::from_ref(&punctuated_name)
        ));
        assert!(!subject_quote_matches_anchor(
            "Ana Ruiz and Bob Ruiz E-17",
            &anchors[0],
            &anchors
        ));
        assert!(!subject_quote_matches_anchor(
            "Ana Ruiz reported that Bob Ruiz E-17 lives in Quito",
            &anchors[0],
            &anchors
        ));

        let hyphenated = QueryAnchor {
            id: "person".into(),
            kind: "employee".into(),
            subject: Some("Anne-Marie".into()),
            stable_id: "E-17".into(),
            quote: "employee Anne-Marie E-17".into(),
        };
        assert!(subject_quote_matches_anchor(
            "Anne-Marie E-17",
            &hyphenated,
            std::slice::from_ref(&hyphenated)
        ));
        assert!(!subject_quote_matches_anchor(
            "Anne Marie E-17",
            &hyphenated,
            std::slice::from_ref(&hyphenated)
        ));

        let kind_in_name = QueryAnchor {
            id: "person".into(),
            kind: "engineer".into(),
            subject: Some("Engineer".into()),
            stable_id: "E-17".into(),
            quote: "engineer Engineer E-17".into(),
        };
        assert!(subject_quote_matches_anchor(
            "Engineer E-17",
            &kind_in_name,
            std::slice::from_ref(&kind_in_name)
        ));
        let kind_in_name_after_subject = QueryAnchor {
            id: "person".into(),
            kind: "engineer".into(),
            subject: Some("Engineer Ruiz".into()),
            stable_id: "E-17".into(),
            quote: "Engineer Ruiz engineer E-17".into(),
        };
        assert!(!subject_quote_matches_anchor(
            "Engineer Ruiz E-17",
            &kind_in_name_after_subject,
            std::slice::from_ref(&kind_in_name_after_subject)
        ));

        let multiword_kind = QueryAnchor {
            id: "person".into(),
            kind: "project manager".into(),
            subject: Some("Ana Ruiz".into()),
            stable_id: "E-17".into(),
            quote: "project manager Ana Ruiz E-17".into(),
        };
        assert!(subject_quote_matches_anchor(
            "Ana Ruiz E-17",
            &multiword_kind,
            std::slice::from_ref(&multiword_kind)
        ));
        assert!(subject_quote_matches_anchor(
            "Ana Ruiz project manager E-17",
            &multiword_kind,
            std::slice::from_ref(&multiword_kind)
        ));
    }

    #[test]
    fn invalid_trace_cannot_yield_selected_ids() {
        let query = "which city for Ana Ruiz employee E-17";
        let memory = candidate("employee Ana Ruiz E-17 lives in Quito");
        let mut evidence = trace(
            IdentityStatus::Match,
            "employee Ana Ruiz E-17 lives in Quito",
            "Ana Ruiz E-17",
        );
        assert!(evidence.validate(query, std::slice::from_ref(&memory)));
        evidence.candidates[0].propositions[0].quote = "Ana Ruiz E-17 lives in Lima".into();
        assert!(!evidence.validate(query, std::slice::from_ref(&memory)));
        assert!(evidence.selected_ids(&[memory], query).is_empty());
    }

    #[test]
    fn stable_id_matching_rejects_prefix_and_suffix_collisions() {
        for similar_id in ["E-170", "XE-17", "E-17A", "E-17.2"] {
            assert!(!contains_identifier("E-17", similar_id));
        }
        assert!(contains_identifier(
            "E-17",
            "employee Ana Ruiz E-17 lives in Quito"
        ));

        let similar_quote = "Ana Ruiz employee E-170";
        let memory = candidate("employee Ana Ruiz E-170 lives in Quito");
        let mut evidence = trace(
            IdentityStatus::Match,
            "employee Ana Ruiz E-170 lives in Quito",
            "Ana Ruiz E-170",
        );
        evidence.version = "slot-evidence-v2".into();
        evidence.query_anchors[0].quote = similar_quote.into();
        assert!(!evidence.validate("which city for Ana Ruiz employee E-170", &[memory]));

        let memory = candidate("employee Ana Ruiz E-170 lives in Quito");
        let mut evidence = trace(
            IdentityStatus::Match,
            "employee Ana Ruiz E-170 lives in Quito",
            "Ana Ruiz E-170",
        );
        evidence.version = "slot-evidence-v2".into();
        evidence.candidates[0].identity.subject_quote = Some("Ana Ruiz E-170".into());
        evidence.candidates[0].propositions[0].subject_quote = "Ana Ruiz E-170".into();
        assert!(!evidence.validate("which city for Ana Ruiz employee E-17", &[memory]));
    }

    #[test]
    fn foreign_relative_guard_recognizes_common_id_separators_but_not_dates() {
        for foreign_id in ["B-29", "B_29", "B.29", "B/29"] {
            let quote = format!(
                "Employee Ana Ruiz E-17 discussed Bob Torres {foreign_id}, who lives in Cusco."
            );
            assert!(
                has_foreign_relative_clause(&quote, "E-17"),
                "foreign ID {foreign_id}"
            );
        }
        assert!(!has_foreign_relative_clause(
            "Employee Ana Ruiz E-17 visited Lima on 2024-05-01, which was a Tuesday.",
            "E-17"
        ));
        for quote in [
            "Employee Ana E-17 discussed Bob (B-29), a contractor who lives in Cusco.",
            "La empleada Ana E-17 habló con Bob (B-29), el contratista que vive en Cusco.",
        ] {
            assert!(has_foreign_relative_clause(quote, "E-17"), "{quote}");
        }
        assert!(!has_foreign_relative_clause(
            "Employee Ana E-17 approved request R-5 for contractor Bob (B-29).",
            "E-17"
        ));
        assert!(!has_foreign_relative_clause(
            "Employee Ana E-17 asked contractor Bob B-29 which city to visit.",
            "E-17"
        ));
        assert!(!has_foreign_relative_clause(
            "La empleada Ana E-17 dijo a Bob B-29 que Quito era bonito.",
            "E-17"
        ));
        assert!(!has_foreign_relative_clause(
            "Employee Ana E-17 asked contractor Bob B-29 which supplier to call.",
            "E-17"
        ));
        assert!(!has_foreign_relative_clause(
            "Ana E-17 told Bob B-29 that Alice had resigned.",
            "E-17"
        ));
        assert!(!has_foreign_relative_clause(
            "Ana E-17 dijo a Bob B-29 que su pedido estaba listo.",
            "E-17"
        ));
        for quote in [
            "Employee Ana E-17 asked Bob B-29 which supplier I should call.",
            "Ana E-17 asked Bob B-29 which supplier our team should contact.",
            "Ana E-17 told Bob B-29 that we can meet tomorrow.",
            "Ana E-17 told Bob B-29 that rain was expected tomorrow.",
            "Ana E-17 dijo a Bob B-29 que mañana estaría en Quito.",
            "Ana E-17 dijo a Bob B-29 que estaría ocupada.",
        ] {
            assert!(!has_foreign_relative_clause(quote, "E-17"), "{quote}");
        }
        assert_eq!(
            foreign_relative_assessment(
                "Ana E-17 habló con Bob (B-29), que estaría en Cusco.",
                "E-17"
            ),
            Some(true)
        );
        assert!(!has_foreign_relative_clause(
            "Ana E-17 habló con Bob (B-29), y le dijo que estaría ocupada.",
            "E-17"
        ));
        assert!(!has_foreign_relative_clause(
            "Ana E-17 dijo a Bob (B-29), el viernes, que estaría ocupada.",
            "E-17"
        ));
        assert!(has_foreign_relative_clause(
            "Ana E-17 habló con Bob (B-29), un contratista local de la ciudad que vive en Cusco.",
            "E-17"
        ));
        assert!(has_foreign_relative_clause(
            "Ana E-17 habló con Bob (B-29), antiguo contratista quien vive en Cusco.",
            "E-17"
        ));
        assert!(has_foreign_relative_clause(
            "Ana E-17 habló con Bob (B-29), antiguo contratista que vive en Cusco.",
            "E-17"
        ));
        assert!(
            original_char_after_lowercase_byte_offset("İthat Alice", 7)
                .is_some_and(char::is_uppercase)
        );
    }

    #[test]
    fn ambiguous_foreign_attribution_is_preserved_but_demoted_from_actionable_selection() {
        let quote = "employee Ana Ruiz E-17 habló con Bob (B-29) que estaría en Cusco.";
        let memory = candidate(quote);
        let mut evidence = trace(IdentityStatus::Match, quote, "Ana Ruiz E-17");
        evidence.version = "slot-evidence-v2".into();
        evidence.query_anchors[0].quote = "employee Ana Ruiz E-17".into();

        assert_eq!(foreign_relative_assessment(quote, "E-17"), Some(false));
        assert_eq!(
            evidence.demote_ambiguous_attributions("Which city for employee Ana Ruiz E-17"),
            1
        );
        assert_eq!(evidence.candidates[0].propositions[0].quote, quote);
        assert_eq!(
            evidence.candidates[0].propositions[0].relation,
            EvidenceRelation::UnresolvedAttribution
        );
        assert!(evidence.candidates[0].propositions[0].target_id.is_empty());
        assert!(
            evidence.validate("which city for employee Ana Ruiz E-17", &[memory.clone()]),
            "validation error: {:?}",
            evidence.validation_error("which city for employee Ana Ruiz E-17", &[memory.clone()])
        );
        assert!(
            evidence
                .selected_ids(&[memory], "which city for Ana Ruiz employee E-17")
                .is_empty()
        );
    }

    #[test]
    fn named_subject_inside_reported_clause_cannot_answer_anchor_fact() {
        let query = "Which city does employee Ana Ruiz E-17 live in?";
        let foreign_quote = "Employee Ana Ruiz E-17 reported that Bob Ruiz lives in Quito.";
        let foreign_memory = candidate(foreign_quote);
        let mut foreign = trace(IdentityStatus::Match, foreign_quote, "Ana Ruiz E-17");
        foreign.version = "slot-evidence-v2".into();
        foreign.query_anchors[0].quote = "employee Ana Ruiz E-17".into();
        foreign.slots[0].quote = "Which city".into();
        foreign.candidates[0].identity.subject_quote = Some("Ana Ruiz E-17".into());
        foreign.candidates[0].propositions[0].subject_quote = "Ana Ruiz E-17".into();
        foreign.candidates[0].propositions[0].quote = foreign_quote.into();

        assert_eq!(
            foreign.selected_ids(std::slice::from_ref(&foreign_memory), query),
            vec!["m1"]
        );
        assert_eq!(foreign.demote_ambiguous_attributions(query), 1);
        assert!(foreign.selected_ids(&[foreign_memory], query).is_empty());

        let same_subject_quote = "Employee Ana Ruiz E-17 reported that Ana Ruiz lives in Quito.";
        let same_subject_memory = candidate(same_subject_quote);
        let mut same_subject = trace(IdentityStatus::Match, same_subject_quote, "Ana Ruiz E-17");
        same_subject.version = "slot-evidence-v2".into();
        same_subject.query_anchors[0].quote = "employee Ana Ruiz E-17".into();
        same_subject.slots[0].quote = "Which city".into();
        same_subject.candidates[0].identity.subject_quote = Some("Ana Ruiz E-17".into());
        same_subject.candidates[0].propositions[0].subject_quote = "Ana Ruiz E-17".into();
        same_subject.candidates[0].propositions[0].quote = same_subject_quote.into();
        assert_eq!(same_subject.demote_ambiguous_attributions(query), 0);
        assert_eq!(
            same_subject.selected_ids(&[same_subject_memory], query),
            vec!["m1"]
        );

        let report_query = "What did employee Ana Ruiz E-17 report?";
        let reported_memory = candidate(foreign_quote);
        let mut reported = trace(IdentityStatus::Match, foreign_quote, "Ana Ruiz E-17");
        reported.version = "slot-evidence-v2".into();
        reported.query_anchors[0].quote = "employee Ana Ruiz E-17".into();
        reported.slots[0].quote = "What did".into();
        reported.candidates[0].identity.subject_quote = Some("Ana Ruiz E-17".into());
        reported.candidates[0].propositions[0].subject_quote = "Ana Ruiz E-17".into();
        reported.candidates[0].propositions[0].quote = foreign_quote.into();
        assert_eq!(reported.demote_ambiguous_attributions(report_query), 0);
        assert_eq!(
            reported.selected_ids(&[reported_memory], report_query),
            vec!["m1"]
        );

        assert!(has_foreign_named_report_subject(
            "Employee Ana Ruiz E-17 reported that Ana Pérez lives in Quito.",
            "Ana Ruiz"
        ));
        assert!(has_foreign_named_report_subject(
            "Employee Ana Ruiz E-17 INFORMÓ que Bob vive en Quito.",
            "Ana Ruiz"
        ));
        assert!(has_foreign_named_report_subject(
            "Employee Ana Ruiz E-17 reported that employee Bob Ruiz lives in Quito.",
            "Ana Ruiz"
        ));
        assert!(has_foreign_named_report_subject(
            "Employee Ana Ruiz E-17 reported that another Ana Ruiz lives in Quito.",
            "Ana Ruiz"
        ));
        assert!(has_foreign_named_report_subject(
            "Employee Ana Ruiz E-17 reported that the other Ana Ruiz lives in Quito.",
            "Ana Ruiz"
        ));
        assert!(has_foreign_named_report_subject(
            "Employee Ana Ruiz E-17 reported that someone named Bob Ruiz lives in Quito.",
            "Ana Ruiz"
        ));
        assert!(has_foreign_named_report_subject(
            "Employee Ana Ruiz E-17 reported that the employee named Bob Ruiz lives in Quito.",
            "Ana Ruiz"
        ));
        assert!(!has_foreign_named_report_subject(
            "Employee Ana Ruiz E-17 reported that Dr. Ana Ruiz lives in Quito.",
            "Ana Ruiz"
        ));
        assert!(query_requests_reported_content(
            "Who did employee Ana Ruiz E-17 say lives in Quito?",
            "Ana Ruiz E-17",
            "employee"
        ));
        assert!(!query_requests_reported_content(
            "Who did employee Bob Torres B-29 say lives in Quito about employee Ana Ruiz E-17?",
            "Ana Ruiz E-17",
            "employee"
        ));
        assert!(query_requests_reported_content(
            "¿QUÉ DIJO employee Ana Ruiz E-17 sobre el informe?",
            "Ana Ruiz E-17",
            "employee"
        ));
        assert!(query_requests_reported_content(
            "Who has employee Ana Ruiz E-17 said lives in Quito?",
            "Ana Ruiz E-17",
            "employee"
        ));
        assert!(query_requests_reported_content(
            "¿Qué ha dicho employee Ana Ruiz E-17 sobre Bob?",
            "Ana Ruiz E-17",
            "employee"
        ));
    }

    #[test]
    fn foreign_relative_guard_uses_proposition_anchor_not_candidate_identity() {
        let query = "Which city does contractor Bob Torres B-29 live in after meeting employee Ana Ruiz E-17?";
        let quote = "Contractor Bob Torres B-29, who lives in Quito, told employee Ana Ruiz E-17, who works in Bogotá, about the report.";
        let memory = candidate(quote);
        let mut evidence = trace(IdentityStatus::Match, quote, "Ana Ruiz E-17");
        evidence.version = "slot-evidence-v2".into();
        evidence.query_anchors[0].quote = "employee Ana Ruiz E-17".into();
        evidence.query_anchors.push(QueryAnchor {
            id: "bob".into(),
            kind: "contractor".into(),
            subject: Some("Bob Torres".into()),
            stable_id: "B-29".into(),
            quote: "contractor Bob Torres B-29".into(),
        });
        evidence.slots[0].quote = "Which city does contractor Bob Torres B-29 live in".into();
        evidence.candidates[0].identity.subject_quote = Some("Ana Ruiz E-17".into());
        evidence.candidates[0].propositions[0].identity_anchor_id = Some("bob".into());
        evidence.candidates[0].propositions[0].subject_quote = "Bob Torres B-29".into();

        let proposition_scope = &quote[quote.find("Bob Torres B-29").unwrap()..];
        assert_eq!(foreign_relative_assessment(proposition_scope, "B-29"), None);
        assert!(!has_foreign_relative_clause(proposition_scope, "B-29"));
        assert!(!has_foreign_named_report_subject(quote, "Bob Torres"));
        assert!(!explicitly_excluded_speech_recipient_for_query(
            quote,
            "B-29",
            query,
            "contractor Bob Torres B-29",
            "contractor",
        ));
        assert_eq!(evidence.demote_ambiguous_attributions(query), 0);
        assert!(
            evidence.validate(query, std::slice::from_ref(&memory)),
            "validation error: {:?}",
            evidence.validation_error(query, std::slice::from_ref(&memory))
        );
        assert_eq!(evidence.selected_ids(&[memory], query), vec!["m1"]);
    }

    #[test]
    fn report_guard_ignores_an_unrelated_later_report_clause() {
        let query = "Which city does contractor Bob Torres B-29 live in?";
        let quotes = [
            "Contractor Bob Torres B-29 lives in Quito. Employee Ana Ruiz E-17 reported that Carlos C-3 moved to Lima.",
            "Contractor Bob Torres B-29 lives in Quito, while employee Ana Ruiz E-17 reported that Carlos C-3 moved to Lima.",
        ];
        for quote in quotes {
            let memory = candidate(quote);
            let mut evidence = trace(IdentityStatus::Match, quote, "Bob Torres B-29");
            evidence.version = "slot-evidence-v2".into();
            evidence.query_anchors[0] = QueryAnchor {
                id: "bob".into(),
                kind: "contractor".into(),
                subject: Some("Bob Torres".into()),
                stable_id: "B-29".into(),
                quote: "contractor Bob Torres B-29".into(),
            };
            evidence.candidates[0].identity.anchor_id = Some("bob".into());
            evidence.candidates[0].identity.stable_id_quote = Some("B-29".into());
            evidence.candidates[0].identity.subject_quote = Some("Bob Torres B-29".into());
            evidence.candidates[0].propositions[0].identity_anchor_id = Some("bob".into());
            evidence.candidates[0].propositions[0].subject_quote = "Bob Torres B-29".into();
            evidence.slots[0].quote = query.into();

            assert_eq!(evidence.demote_ambiguous_attributions(query), 0, "{quote}");
            assert!(
                evidence.validate(query, std::slice::from_ref(&memory)),
                "validation error for {quote:?}: {:?}",
                evidence.validation_error(query, std::slice::from_ref(&memory))
            );
            assert_eq!(
                evidence.selected_ids(&[memory], query),
                vec!["m1"],
                "{quote}"
            );
        }
    }

    #[test]
    #[ignore = "Known V2 gap: no clause-local predicate evidence to distinguish sequential relatives"]
    fn foreign_relative_after_bound_relative_is_not_misattributed() {
        let quote = "Contractor Bob Torres B-29, who joined the team in 2020, met employee Carlos C-3, who lives in Lima.";
        let proposition_scope = proposition_subject_scope(quote, "Bob Torres B-29");
        assert_eq!(
            foreign_relative_assessment(proposition_scope, "B-29"),
            Some(true)
        );
    }

    #[test]
    fn v3_predicate_scope_preserves_bound_fact_and_rejects_later_foreign_relative() {
        let query = "Which city does contractor Bob Torres B-29 live in?";
        let bound_quote = "Contractor Bob Torres B-29, who lives in Quito, told employee Ana Ruiz E-17, who works in Bogotá, about the report.";
        let mut bound = trace(IdentityStatus::Match, bound_quote, "Bob Torres B-29");
        bound.version = "slot-evidence-v3".into();
        bound.query_anchors[0] = QueryAnchor {
            id: "bob".into(),
            kind: "contractor".into(),
            subject: Some("Bob Torres".into()),
            stable_id: "B-29".into(),
            quote: "contractor Bob Torres B-29".into(),
        };
        bound.candidates[0].identity.anchor_id = Some("bob".into());
        bound.candidates[0].identity.stable_id_quote = Some("B-29".into());
        bound.candidates[0].identity.subject_quote = Some("Bob Torres B-29".into());
        bound.candidates[0].propositions[0].identity_anchor_id = Some("bob".into());
        bound.candidates[0].propositions[0].predicate_quote = "lives in Quito".into();
        bound.slots[0].quote = query.into();
        let bound_memory = candidate(bound_quote);

        assert!(bound.validate(query, std::slice::from_ref(&bound_memory)));
        assert_eq!(bound.selected_ids(&[bound_memory], query), ["m1"]);
        let mut malformed = bound.clone();
        malformed.candidates[0].propositions[0].predicate_quote = "lives in Mars".into();
        assert_eq!(
            malformed.validation_error(query, &[candidate(bound_quote)]),
            Some("proposition_predicate_quote_invalid")
        );
        assert!(!has_foreign_relative_before_predicate(
            "Employee Carlos C-3, who lives in Lima, met contractor Bob Torres B-29, who lives in Quito.",
            "B-29",
            "lives in Quito",
        ));

        let coordinated_quote = "Contractor Bob Torres B-29 and employee Carlos Ruiz C-3, who joined in 2020, live in Quito.";
        let mut coordinated = bound.clone();
        coordinated.candidates[0].propositions[0].quote = coordinated_quote.into();
        coordinated.candidates[0].propositions[0].predicate_quote = "live in Quito".into();
        let coordinated_memory = candidate(coordinated_quote);
        assert!(coordinated.validate(query, std::slice::from_ref(&coordinated_memory)));
        assert_eq!(
            coordinated.selected_ids(&[coordinated_memory], query),
            ["m1"]
        );

        let foreign_quote = "Contractor Bob Torres B-29, who joined the team in 2020, met employee Carlos C-3, who lives in Lima.";
        let mut foreign = trace(IdentityStatus::Match, foreign_quote, "Bob Torres B-29");
        foreign.version = "slot-evidence-v3".into();
        foreign.query_anchors[0] = QueryAnchor {
            id: "bob".into(),
            kind: "contractor".into(),
            subject: Some("Bob Torres".into()),
            stable_id: "B-29".into(),
            quote: "contractor Bob Torres B-29".into(),
        };
        foreign.candidates[0].identity.anchor_id = Some("bob".into());
        foreign.candidates[0].identity.stable_id_quote = Some("B-29".into());
        foreign.candidates[0].identity.subject_quote = Some("Bob Torres B-29".into());
        foreign.candidates[0].propositions[0].identity_anchor_id = Some("bob".into());
        foreign.candidates[0].propositions[0].predicate_quote = "lives in Lima".into();
        foreign.slots[0].quote = query.into();
        let foreign_memory = candidate(foreign_quote);

        assert_eq!(
            foreign.demote_ambiguous_attributions(query),
            1,
            "a foreign relative before the target predicate must be demoted"
        );
        assert!(foreign.validate(query, std::slice::from_ref(&foreign_memory)));
        assert!(foreign.selected_ids(&[foreign_memory], query).is_empty());

        let reported_quote =
            "Contractor Bob Torres B-29 said employee Carlos Ruiz C-3 lives in Lima.";
        let mut reported = trace(IdentityStatus::Match, reported_quote, "Bob Torres B-29");
        reported.version = "slot-evidence-v3".into();
        reported.query_anchors[0] = QueryAnchor {
            id: "bob".into(),
            kind: "contractor".into(),
            subject: Some("Bob Torres".into()),
            stable_id: "B-29".into(),
            quote: "contractor Bob Torres B-29".into(),
        };
        reported.candidates[0].identity.anchor_id = Some("bob".into());
        reported.candidates[0].identity.stable_id_quote = Some("B-29".into());
        reported.candidates[0].identity.subject_quote = Some("Bob Torres B-29".into());
        reported.candidates[0].propositions[0].identity_anchor_id = Some("bob".into());
        reported.candidates[0].propositions[0].predicate_quote = "lives in Lima".into();
        reported.slots[0].quote = query.into();
        let reported_memory = candidate(reported_quote);
        assert_eq!(reported.demote_ambiguous_attributions(query), 1);
        assert!(reported.selected_ids(&[reported_memory], query).is_empty());
    }

    #[test]
    fn v3_demotes_foreign_later_sentence_and_keeps_useful_proposition() {
        let query = "Where does engineer Marta Solis M-58 work now?";
        let first_sentence = "Engineer Marta Solis M-58 currently works from Valencia.";
        let source = format!(
            "{first_sentence} Archivist Teo Rivas T-26 lives in Girona and catalogued the flood maps."
        );
        let mut evidence = trace(IdentityStatus::Match, &source, "Engineer Marta Solis M-58");
        evidence.version = "slot-evidence-v3".into();
        evidence.query_anchors[0] = QueryAnchor {
            id: "marta".into(),
            kind: "engineer".into(),
            subject: Some("Marta Solis".into()),
            stable_id: "M-58".into(),
            quote: "engineer Marta Solis M-58".into(),
        };
        evidence.slots[0].quote = "Where does".into();
        evidence.premises = vec![QueryTarget {
            id: "current-work".into(),
            dimension: "correction_premise".into(),
            quote: "work now".into(),
        }];
        evidence.candidates[0].identity.anchor_id = Some("marta".into());
        evidence.candidates[0].identity.stable_id_quote = Some("M-58".into());
        evidence.candidates[0].identity.subject_quote = Some("Engineer Marta Solis M-58".into());

        let useful = &mut evidence.candidates[0].propositions[0];
        useful.quote = first_sentence.into();
        useful.subject_quote = "Engineer Marta Solis M-58".into();
        useful.identity_anchor_id = Some("marta".into());
        useful.predicate_quote = "currently works from Valencia".into();
        useful.value = Some("Valencia".into());

        let mut foreign = useful.clone();
        foreign.occurrence_id = "p2".into();
        foreign.quote = source.clone();
        foreign.predicate_quote = "lives in Girona".into();
        foreign.value = Some("Girona".into());
        evidence.candidates[0].propositions.push(foreign);

        assert_eq!(
            evidence.demote_ambiguous_attributions(query),
            1,
            "the proposition whose predicate belongs to Teo must be demoted"
        );
        assert_eq!(
            evidence.candidates[0].propositions[0].relation,
            EvidenceRelation::Answers
        );
        assert_eq!(
            evidence.candidates[0].propositions[1].relation,
            EvidenceRelation::UnresolvedAttribution
        );
        let memory = candidate(&source);
        assert!(
            evidence.validate(query, std::slice::from_ref(&memory)),
            "the invalid foreign proposition must not invalidate the useful one"
        );
        assert_eq!(evidence.selected_ids(&[memory], query), ["m1"]);
    }

    #[test]
    fn v3_critic_regressions_distinguish_reporting_and_subject_coordination() {
        assert!(!has_foreign_subject_before_predicate(
            "Contractor Bob Torres B-29 said employee Carlos Ruiz C-3 lives in Lima, and Bob Torres B-29 lives in Quito.",
            "B-29",
            "lives in Quito",
        ));
        assert!(has_foreign_subject_before_predicate(
            "Contractor Bob Torres B-29 met employee Carlos C-3 and employee Dana D-4, who live in Lima.",
            "B-29",
            "live in Lima",
        ));
        assert!(!has_foreign_subject_before_predicate(
            "Contractor Bob Torres B-29 and employee Carlos Ruiz C-3, who joined in 2020, live in Quito.",
            "B-29",
            "live in Quito",
        ));
        for quote in [
            "Contractor Bob Torres B-29 thought that employee Carlos Ruiz C-3 lives in Lima.",
            "Contractor Bob Torres B-29 believed employee Carlos Ruiz C-3 lives in Lima.",
            "Contractor Bob Torres B-29 understands that employee Carlos Ruiz C-3 lives in Lima.",
            "Contractor Bob Torres B-29 understands that employee Carlos Ruiz C-3 currently lives in Lima.",
            "Contractor Bob Torres B-29 realized employee Carlos Ruiz C-3 lived in Lima.",
            "Contractor Bob Torres B-29 understood employee Carlos Ruiz C-3 lived in Lima.",
            "Contractor Bob Torres B-29 understands that employee Carlos Ruiz C-3, his colleague, lives in Lima.",
            "Contractor Bob Torres B-29 realized that employee Carlos Ruiz C-3 (a close colleague) lived in Lima.",
            "Contractor Bob Torres B-29 said that employee Carlos C-3, the chief compliance officer at the firm, lives in Lima.",
            "Contractor Bob Torres B-29 reckoned that employee Carlos Ruiz C-3 lives in Lima.",
            "Contractor Bob Torres B-29 pensó que employee Carlos Ruiz C-3 vive en Lima.",
            "Contractor Bob Torres B-29 creía que employee Carlos Ruiz C-3 vive en Lima.",
            "Contractor Bob Torres B-29 cree que employee Carlos Ruiz C-3 vive en Lima.",
            "Contractor Bob Torres B-29 piensa que employee Carlos Ruiz C-3 vive en Lima.",
            "Contractor Bob Torres B-29 entiende que employee Carlos Ruiz C-3 vive en Lima.",
            "Contractor Bob Torres B-29 dijo que employee Carlos Ruiz C-3, su colega, vive en Lima.",
        ] {
            assert!(
                has_foreign_subject_before_predicate(
                    quote,
                    "B-29",
                    if quote.contains("vive en Lima") {
                        "vive en Lima"
                    } else if quote.contains("lived in Lima") {
                        "lived in Lima"
                    } else {
                        "lives in Lima"
                    },
                ),
                "a named person in a cognitive complement must not inherit the predicate: {quote}"
            );
        }
        assert!(!has_foreign_subject_before_predicate(
            "Contractor Bob Torres B-29 thought that Quito was beautiful.",
            "B-29",
            "Quito was beautiful",
        ));
        assert!(!has_foreign_subject_before_predicate(
            "Contractor Bob Torres B-29 considered employee Carlos C-3 a close friend and lives in Quito.",
            "B-29",
            "lives in Quito",
        ));
        assert!(!has_foreign_subject_before_predicate(
            "Contractor Bob Torres B-29 told employee Carlos C-3 a joke and lives in Quito.",
            "B-29",
            "lives in Quito",
        ));
        assert!(has_foreign_subject_before_predicate(
            "Contractor Bob Torres B-29 lives in Quito. Employee Carlos C-3 lives in Lima.",
            "B-29",
            "lives in Lima",
        ));
        assert!(!has_foreign_subject_before_predicate(
            "Employee Carlos C-3 lives in Lima. Contractor Bob Torres B-29 lives in Quito.",
            "B-29",
            "lives in Quito",
        ));
        assert!(!has_foreign_subject_before_predicate(
            "Contractor Bob Torres B-29, who thought that employee Carlos C-3 lives in Lima, lives in Quito.",
            "B-29",
            "lives in Quito",
        ));
    }

    #[test]
    fn v3_new_blind_bank_matches_independent_attribution_adjudication() {
        let bank: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v3-attribution-blind-v1.json"
        ))
        .expect("the frozen blind attribution bank is valid JSON");
        let cases = bank["cases"].as_array().expect("cases array");
        let expected = [
            ("lives in Quito", true),
            ("lives in Lima", false),
            ("lives in Lima", false),
            ("lived in Lima", false),
            ("lives in Lima", false),
            ("lives in Lima", false),
            ("lives in Lima", false),
            ("lives in Quito", true),
            ("lives in Quito", true),
            ("lives in Quito", true),
            ("live in Lima", false),
            ("live in Quito", true),
            ("lives in Quito", true),
            ("lives in Quito", true),
        ];
        assert_eq!(cases.len(), expected.len());

        for (case, (predicate_quote, should_select)) in cases.iter().zip(expected) {
            let query = case["query"].as_str().expect("query string");
            let summary = case["summary"].as_str().expect("summary string");
            let case_id = case["id"].as_str().expect("case id");
            let subject_quote = if summary.contains("Contractor Bob Torres") {
                "Contractor Bob Torres B-29"
            } else {
                "contractor Bob Torres B-29"
            };
            let mut evidence = trace(IdentityStatus::Match, summary, subject_quote);
            evidence.version = "slot-evidence-v3".into();
            evidence.query_anchors[0] = QueryAnchor {
                id: "person".into(),
                kind: "contractor".into(),
                subject: Some("Bob Torres".into()),
                stable_id: "B-29".into(),
                quote: "contractor Bob Torres (B-29)".into(),
            };
            evidence.slots[0].quote = "city".into();
            evidence.candidates[0].identity.anchor_id = Some("person".into());
            evidence.candidates[0].identity.stable_id_quote = Some("B-29".into());
            evidence.candidates[0].identity.subject_quote = Some(subject_quote.into());
            evidence.candidates[0].propositions[0].identity_anchor_id = Some("person".into());
            evidence.candidates[0].propositions[0].subject_quote = subject_quote.into();
            evidence.candidates[0].propositions[0].predicate_quote = predicate_quote.into();

            let memory = candidate(summary);
            evidence.demote_ambiguous_attributions(query);
            assert!(
                evidence.validate(query, std::slice::from_ref(&memory)),
                "{case_id}: {:?}",
                evidence.validation_error(query, std::slice::from_ref(&memory))
            );
            let selected = evidence.selected_ids(std::slice::from_ref(&memory), query);
            assert_eq!(
                selected == ["m1"],
                should_select,
                "blind-adjudicated expected selection for {case_id}"
            );
        }
    }

    #[test]
    fn report_guard_stops_at_coordinated_foreign_report_clause() {
        let quote = "Contractor Bob Torres B-29 lives in Quito and employee Ana Ruiz E-17 reported that Carlos C-3 moved to Lima.";
        let scope =
            proposition_subject_clause_scope(proposition_subject_scope(quote, "Bob Torres B-29"));
        assert!(!has_foreign_named_report_subject(scope, "Bob Torres"));
    }

    #[test]
    fn query_requested_speech_complement_survives_local_abstention() {
        let quote = "employee Ana Ruiz E-17 dijo a Bob (B-29) que estaría ocupada.";
        let memory = candidate(quote);
        let mut evidence = trace(IdentityStatus::Match, quote, "Ana Ruiz E-17");
        evidence.version = "slot-evidence-v2".into();
        evidence.query_anchors[0].quote = "employee Ana Ruiz E-17".into();

        assert_eq!(foreign_relative_assessment(quote, "E-17"), Some(false));
        evidence.slots[0].quote = "¿Qué le dijo".into();
        assert!(has_ordered_recipient_speech_act(
            "¿Qué le dijo employee Ana Ruiz E-17 al contractor Bob Torres B-29?",
            "E-17",
            "B-29",
            &["dijo"]
        ));
        assert!(has_ordered_recipient_speech_act(
            quote,
            "E-17",
            "B-29",
            &["dijo"]
        ));
        assert!(has_immediate_clause_marker_after_id(quote, "B-29"));
        assert_eq!(
            evidence.demote_ambiguous_attributions(
                "¿Qué le dijo employee Ana Ruiz E-17 al contractor Bob Torres B-29?"
            ),
            0
        );
        assert!(evidence.validate(
            "¿Qué le dijo employee Ana Ruiz E-17 al contractor Bob Torres B-29?",
            &[memory.clone()]
        ));
        assert_eq!(
            evidence.selected_ids(
                &[memory],
                "¿Qué le dijo employee Ana Ruiz E-17 al contractor Bob Torres B-29?"
            ),
            vec!["m1"]
        );
        assert!(
            evidence.candidates[0].propositions[0]
                .quote
                .contains("que estaría ocupada")
        );

        let mixed_quote = "employee Ana Ruiz E-17 asked contractor Bob Torres B-29, who lives in Cusco, which supplier our team should contact.";
        let mixed_memory = candidate(mixed_quote);
        let mut mixed = trace(IdentityStatus::Match, mixed_quote, "Ana Ruiz E-17");
        mixed.version = "slot-evidence-v2".into();
        mixed.query_anchors[0].quote = "Ana Ruiz employee E-17".into();
        mixed.slots[0].quote = "What did".into();
        let query = "What did Ana Ruiz employee E-17 ask contractor Bob Torres B-29?";
        assert_eq!(foreign_relative_assessment(mixed_quote, "E-17"), Some(true));
        assert_eq!(mixed.demote_ambiguous_attributions(query), 1);
        assert!(mixed.validate(query, &[mixed_memory.clone()]));
        assert!(
            mixed
                .selected_ids(&[mixed_memory], "which city for Ana Ruiz employee E-17")
                .is_empty()
        );

        let unrelated_recipient_quote = "employee Ana Ruiz E-17 dijo al equipo que Bob Torres B-29 que estaría en Cusco sería el contratista elegido.";
        let unrelated_recipient_memory = candidate(unrelated_recipient_quote);
        let mut unrelated_recipient = trace(
            IdentityStatus::Match,
            unrelated_recipient_quote,
            "Ana Ruiz E-17",
        );
        unrelated_recipient.version = "slot-evidence-v2".into();
        unrelated_recipient.query_anchors[0].quote = "employee Ana Ruiz E-17".into();
        unrelated_recipient.slots[0].quote = "¿Qué le dijo".into();
        let query = "¿Qué le dijo employee Ana Ruiz E-17 al contractor Bob Torres B-29?";
        assert_eq!(unrelated_recipient.demote_ambiguous_attributions(query), 1);
        assert!(unrelated_recipient.validate(query, &[unrelated_recipient_memory.clone()]));
        assert!(
            unrelated_recipient
                .selected_ids(
                    &[unrelated_recipient_memory],
                    "which city for Ana Ruiz employee E-17"
                )
                .is_empty()
        );
    }

    #[test]
    fn r4_counterexample_requires_the_same_recipient_without_an_intervening_clause() {
        let bank: LocalBank = serde_json::from_str(include_str!(
            "../../../tests/fixtures/slot-evidence-v2/local-bank-r4.json"
        ))
        .expect("valid synthetic slot-evidence-v2 r4 bank");
        assert_eq!(bank.cases.len(), 20);
        let case = bank
            .cases
            .iter()
            .find(|case| case.id == "speech-query-does-not-rescue-unrelated-recipient-relative")
            .expect("independent critical counterexample is retained");
        let mut memory = candidate(&case.candidate.summary);
        memory.memory_id = case.candidate.memory_id.clone();
        memory.revision = case.candidate.revision;
        assert!(case.trace.validate(&case.query, &[memory.clone()]));
        assert_eq!(
            case.trace.selected_ids(&[memory.clone()], &case.query),
            vec!["unrelated-recipient-relative-m1"],
            "the new case must reproduce the r3 false positive before the r4 gate"
        );

        let mut trace = case.trace.clone();
        assert_eq!(trace.demote_ambiguous_attributions(&case.query), 1);
        assert!(trace.validate(&case.query, &[memory.clone()]));
        assert!(trace.selected_ids(&[memory], &case.query).is_empty());
        assert_eq!(
            trace.candidates[0].propositions[0].relation,
            EvidenceRelation::UnresolvedAttribution
        );
    }

    #[test]
    fn prospective_sentence_scope_v1_matches_blind_subject_adjudication() {
        let cases = [
            (
                "sentence-01 direct target remains usable",
                "Engineer Marta Solis M-58 currently works from Valencia. Archivist Teo Rivas T-26 lives in Girona and catalogued the flood maps.",
                "M-58",
                "currently works from Valencia",
                false,
            ),
            (
                "sentence-02 target after unrelated sentence remains usable",
                "Consultant Lee Park L-81 lives in Lisbon. Employee Dana Ruiz D-44 lives in Bogotá.",
                "D-44",
                "lives in Bogotá",
                false,
            ),
            (
                "sentence-04 later foreign subject is rejected",
                "Project manager Ana Ruiz A-12 is a colleague of analyst Bruno Vale B-8. Analyst Bruno Vale B-8 lives in Lima.",
                "A-12",
                "lives in Lima",
                true,
            ),
            (
                "sentence-08 later foreign subject is rejected",
                "Architect Noor Vale N-13 left Porto last year. Consultant Sam Ivo S-21 currently works from Dakar.",
                "N-13",
                "currently works from Dakar",
                true,
            ),
            (
                "sentence-09 Spanish direct target remains usable",
                "La ingeniera Marta Solis M-58 trabaja actualmente desde Valencia. El archivista Teo Rivas T-26 vive en Girona.",
                "M-58",
                "trabaja actualmente desde Valencia",
                false,
            ),
        ];

        for (name, quote, bound_id, predicate, expected_foreign_subject) in cases {
            assert_eq!(
                has_foreign_subject_before_predicate(quote, bound_id, predicate),
                expected_foreign_subject,
                "{name}"
            );
        }

        // Same-sentence cases exercise the existing clause/relative guard rather
        // than the new sentence-boundary helper.
        let while_quote = "Engineer Marta Solis M-58 works from Valencia, while archivist Teo Rivas T-26 lives in Girona.";
        assert!(has_foreign_subject_before_predicate(
            while_quote,
            "M-58",
            "lives in Girona"
        ));
        let semicolon_quote = "Engineer Marta Solis M-58 works from Valencia; archivist Teo Rivas T-26 lives in Girona.";
        assert!(has_foreign_subject_before_predicate(
            semicolon_quote,
            "M-58",
            "lives in Girona"
        ));
    }

    #[test]
    fn domain_type_matching_ignores_ascii_case_but_requires_token_boundaries() {
        assert!(contains_domain_type("employee", "Employee Ana Ruiz E-17"));
        assert!(!contains_domain_type(
            "employee",
            "unemployee Ana Ruiz E-17"
        ));
    }

    #[test]
    fn compact_v2_trace_preserves_binding_and_selects_exact_roster_entry() {
        let memory = candidate("employee Ana Ruiz E-17 lives in Quito");
        let compact = serde_json::json!({
            "query_anchors":[{"id":"person","kind":"employee","stable_id":"E-17","quote":"employee Ana Ruiz E-17"}],
            "slots":[{"id":"city","quote":"which city"}],
            "premises":[],
            "candidates":[{"memory_id":"m1","revision":1,
                "identity":{"status":"match","anchor_id":"person","subject_quote":"employee Ana Ruiz E-17"},
                "propositions":[{"identity_anchor_id":"person","quote":"employee Ana Ruiz E-17 lives in Quito","relation":"answers","target_id":"city"}]
            }]
        });
        let mut trace: SlotEvidenceTrace = serde_json::from_value(compact).unwrap();
        trace.version = "slot-evidence-v2".into();
        let stable_id = trace.query_anchors[0].stable_id.clone();
        let entry = &mut trace.candidates[0];
        entry.identity.stable_id_quote = Some(stable_id);
        entry.propositions[0].occurrence_id = "p1".into();
        entry.propositions[0].subject_quote = entry.identity.subject_quote.clone().unwrap();
        assert!(trace.validate("which city for employee Ana Ruiz E-17", &[memory.clone()]));
        assert_eq!(
            trace.selected_ids(&[memory], "which city for employee Ana Ruiz E-17"),
            vec!["m1"]
        );

        let mut invalid_source_quote = trace.clone();
        invalid_source_quote.candidates[0].propositions[0].quote = "not in memory".into();
        assert_eq!(
            invalid_source_quote.validation_error(
                "which city for employee Ana Ruiz E-17",
                &[candidate("employee Ana Ruiz E-17 lives in Quito")]
            ),
            Some("proposition_source_quote_invalid")
        );

        let mut invalid_anchor_reference = trace.clone();
        invalid_anchor_reference.candidates[0].propositions[0].identity_anchor_id =
            Some("different_person".into());
        assert_eq!(
            invalid_anchor_reference.validation_error(
                "which city for employee Ana Ruiz E-17",
                &[candidate("employee Ana Ruiz E-17 lives in Quito")]
            ),
            Some("proposition_anchor_id_invalid")
        );

        let mut unknown_target = trace.clone();
        unknown_target.candidates[0].propositions[0].target_id = "missing_slot".into();
        assert_eq!(
            unknown_target.validation_error(
                "which city for employee Ana Ruiz E-17",
                &[candidate("employee Ana Ruiz E-17 lives in Quito")]
            ),
            Some("proposition_target_unknown")
        );

        let mut wrong_target_kind = trace.clone();
        wrong_target_kind.premises.push(QueryTarget {
            id: "premise_1".into(),
            dimension: "correction_premise".into(),
            quote: "employee Ana Ruiz E-17".into(),
        });
        wrong_target_kind.candidates[0].propositions[0].target_id = "premise_1".into();
        assert_eq!(
            wrong_target_kind.validation_error(
                "which city for employee Ana Ruiz E-17",
                &[candidate("employee Ana Ruiz E-17 lives in Quito")]
            ),
            Some("proposition_target_relation_mismatch")
        );

        let mut non_actionable_target = trace;
        non_actionable_target.candidates[0].propositions[0].relation = EvidenceRelation::Unknown;
        assert_eq!(
            non_actionable_target.validation_error(
                "which city for employee Ana Ruiz E-17",
                &[candidate("employee Ana Ruiz E-17 lives in Quito")]
            ),
            Some("proposition_non_actionable_target_not_empty")
        );
    }

    #[test]
    fn compact_v2_accepts_independent_subject_quotes_bound_to_the_query_anchor() {
        let memory = candidate("Senior engineer Ana Ruiz E-17 lives in Quito");
        let compact = serde_json::json!({
            "query_anchors":[{"id":"person","kind":"engineer","stable_id":"E-17","quote":"engineer Ana Ruiz E-17"}],
            "slots":[{"id":"city","quote":"which city"}],
            "premises":[],
            "candidates":[{"memory_id":"m1","revision":1,
                "identity":{"status":"match","anchor_id":"person","subject_quote":"Senior engineer Ana Ruiz E-17"},
                "propositions":[{"identity_anchor_id":"person","quote":"Senior engineer Ana Ruiz E-17 lives in Quito","subject_quote":"Ana Ruiz E-17","relation":"answers","target_id":"city"}]
            }]
        });
        let mut trace: SlotEvidenceTrace = serde_json::from_value(compact).unwrap();
        trace.version = "slot-evidence-v2".into();
        trace.candidates[0].identity.stable_id_quote = Some("E-17".into());
        trace.candidates[0].propositions[0].occurrence_id = "p1".into();
        assert!(trace.validate(
            "which city for engineer Ana Ruiz E-17",
            std::slice::from_ref(&memory)
        ));
        assert_eq!(
            trace.selected_ids(&[memory], "which city for engineer Ana Ruiz E-17"),
            vec!["m1"]
        );
    }

    #[test]
    fn compact_v2_rejects_subject_quotes_for_a_different_query_person() {
        let memory = candidate("Senior engineer Bob Ruiz E-17 lives in Quito");
        let compact = serde_json::json!({
            "query_anchors":[{"id":"person","kind":"engineer","stable_id":"E-17","quote":"engineer Ana Ruiz E-17"}],
            "slots":[{"id":"city","quote":"which city"}],
            "premises":[],
            "candidates":[{"memory_id":"m1","revision":1,
                "identity":{"status":"match","anchor_id":"person","subject_quote":"Senior engineer Bob Ruiz E-17"},
                "propositions":[{"identity_anchor_id":"person","quote":"Senior engineer Bob Ruiz E-17 lives in Quito","subject_quote":"Bob Ruiz E-17","relation":"answers","target_id":"city"}]
            }]
        });
        let mut trace: SlotEvidenceTrace = serde_json::from_value(compact).unwrap();
        trace.version = "slot-evidence-v2".into();
        trace.candidates[0].identity.stable_id_quote = Some("E-17".into());
        trace.candidates[0].propositions[0].occurrence_id = "p1".into();

        assert_eq!(
            trace.validation_error("which city for engineer Ana Ruiz E-17", &[memory]),
            Some("matched_identity_subject_not_bound_to_query")
        );
    }

    #[test]
    fn supporting_a_correction_premise_preserves_relevant_evidence() {
        let memory = candidate("employee Ana Ruiz E-17 supports assumption city was Quito");
        let mut evidence = trace(
            IdentityStatus::Match,
            "employee Ana Ruiz E-17 supports assumption city was Quito",
            "Ana Ruiz E-17",
        );
        evidence.version = "slot-evidence-v2".into();
        evidence.slots.clear();
        evidence.premises = vec![QueryTarget {
            id: "premise".into(),
            dimension: "correction_premise".into(),
            quote: "assumption city was Quito".into(),
        }];
        evidence.candidates[0].propositions[0].relation = EvidenceRelation::Supports;
        evidence.candidates[0].propositions[0].target_id = "premise".into();
        assert!(evidence.validate(
            "which city for Ana Ruiz employee E-17; assumption city was Quito",
            &[memory.clone()]
        ));
        assert_eq!(
            evidence.selected_ids(
                &[memory],
                "which city for Ana Ruiz employee E-17; assumption city was Quito"
            ),
            vec!["m1"]
        );
    }

    #[test]
    fn explicit_unmeasured_slot_evidence_is_selected_without_inferencing_a_value() {
        let query = "What was the PIN for locker BlueBox L-417?";
        let summary = "Locker BlueBox L-417 has no PIN recorded.";
        let memory = candidate(summary);
        let mut evidence = trace(IdentityStatus::Match, summary, "BlueBox L-417");
        evidence.version = "slot-evidence-v2".into();
        evidence.query_anchors = vec![QueryAnchor {
            id: "locker".into(),
            kind: "locker".into(),
            subject: Some("BlueBox".into()),
            stable_id: "L-417".into(),
            quote: "locker BlueBox L-417".into(),
        }];
        evidence.slots = vec![QueryTarget {
            id: "pin".into(),
            dimension: "requested_slot".into(),
            quote: "the PIN".into(),
        }];
        evidence.premises.clear();
        let candidate_evidence = &mut evidence.candidates[0];
        candidate_evidence.identity.anchor_id = Some("locker".into());
        candidate_evidence.identity.stable_id_quote = Some("L-417".into());
        candidate_evidence.identity.subject_quote = Some("BlueBox L-417".into());
        let proposition = &mut candidate_evidence.propositions[0];
        proposition.subject_quote = "BlueBox L-417".into();
        proposition.identity_anchor_id = Some("locker".into());
        proposition.relation = EvidenceRelation::ExplicitUnmeasured;
        proposition.target_id = "pin".into();

        assert!(evidence.validate(query, std::slice::from_ref(&memory)));
        assert_eq!(
            evidence.selected_ids(&[memory], query),
            vec!["m1"],
            "the explicit absence is useful evidence for the requested slot, not a value to infer"
        );
    }

    #[test]
    fn unresolved_identity_is_retained_but_never_selected() {
        let memory = candidate("employee Ana Ruiz E-17 lives in Quito");
        let evidence = trace(
            IdentityStatus::Unresolved,
            "employee Ana Ruiz E-17 lives in Quito",
            "employee Ana Ruiz E-17",
        );
        assert!(evidence.validate("which city for Ana Ruiz employee E-17", &[memory.clone()]));
        assert!(
            evidence
                .selected_ids(&[memory], "which city for Ana Ruiz employee E-17")
                .is_empty()
        );
    }

    #[test]
    fn fact_about_another_subject_is_not_selected_even_with_matching_memory_identity() {
        let memory = candidate("employee Ana Ruiz E-17 discussed colleague Bob E-29 lives in Lima");
        let evidence = trace(IdentityStatus::Match, "Bob E-29 lives in Lima", "Bob E-29");
        assert!(evidence.validate("which city for Ana Ruiz employee E-17", &[memory.clone()]));
        assert!(
            evidence
                .selected_ids(&[memory], "which city for Ana Ruiz employee E-17")
                .is_empty()
        );
    }

    #[test]
    fn duplicate_roster_identity_fails_closed() {
        let one = candidate("employee Ana Ruiz E-17 lives in Quito");
        let mut duplicate = one.clone();
        duplicate.revision = 2;
        let evidence = trace(
            IdentityStatus::Match,
            "employee Ana Ruiz E-17 lives in Quito",
            "employee Ana Ruiz E-17",
        );
        let mut evidence = evidence;
        let mut second = evidence.candidates[0].clone();
        second.revision = 2;
        evidence.candidates.push(second);
        assert!(!evidence.validate("which city for Ana Ruiz employee E-17", &[one, duplicate]));
    }

    #[test]
    fn roster_gate_rejects_extra_missing_and_reordered_candidates() {
        let one = candidate("employee Ana Ruiz E-17 lives in Quito");
        let mut two = candidate("employee Ana Ruiz E-17 lives in Quito");
        two.memory_id = "m2".into();
        let query = "which city for Ana Ruiz employee E-17";

        let mut extra = trace(
            IdentityStatus::Match,
            "employee Ana Ruiz E-17 lives in Quito",
            "employee Ana Ruiz E-17",
        );
        let mut extra_entry = extra.candidates[0].clone();
        extra_entry.memory_id = "unexpected".into();
        extra.candidates.push(extra_entry);
        assert_eq!(
            extra.validation_error(query, std::slice::from_ref(&one)),
            Some("trace_candidate_roster_extra")
        );

        let missing = trace(
            IdentityStatus::Match,
            "employee Ana Ruiz E-17 lives in Quito",
            "employee Ana Ruiz E-17",
        );
        assert_eq!(
            missing.validation_error(query, &[one.clone(), two.clone()]),
            Some("trace_candidate_roster_missing")
        );

        let mut reordered = missing;
        let mut first = reordered.candidates[0].clone();
        first.memory_id = "m2".into();
        let mut second = first.clone();
        second.memory_id = "m1".into();
        reordered.candidates = vec![first, second];
        assert_eq!(
            reordered.validation_error(query, &[one, two]),
            Some("candidate_identity_revision_or_proposition_bounds")
        );
    }
}
