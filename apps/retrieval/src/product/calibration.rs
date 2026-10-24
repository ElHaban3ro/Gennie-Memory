//! Empirical, profile-bound factor reliability. Never factual confidence.
use crate::model::{ClassifierProvider, SelectionPolicy};
use crate::per_memory::factors::MemoryFactorScores;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bin {
    pub upper: f64,
    pub positives: u32,
    pub n: u32,
}
impl Bin {
    fn value(&self) -> f64 {
        (f64::from(self.positives) + 1.0) / (f64::from(self.n) + 2.0)
    }
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Factors {
    pub same_context: Vec<Bin>,
    pub requested_fact: Vec<Bin>,
    pub premise_correction: Vec<Bin>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Calibration {
    pub schema_version: u32,
    pub profile: String,
    pub profile_sha256: String,
    pub provider: ClassifierProvider,
    pub selection_policy: SelectionPolicy,
    pub requested_model: String,
    pub served_model: String,
    pub training_sha256: String,
    pub factors: Factors,
}
fn hash(h: &str) -> bool {
    h.len() == 64
        && h.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn valid_bins(bins: &[Bin]) -> bool {
    !bins.is_empty()
        && bins.len() <= 20
        && bins.last().is_some_and(|b| b.upper == 1.0)
        && bins.iter().enumerate().all(|(i, b)| {
            b.upper.is_finite()
                && b.upper > 0.0
                && b.upper <= 1.0
                && b.n > 0
                && b.positives <= b.n
                && (i == 0 || b.upper > bins[i - 1].upper)
        })
}
fn compatible_served_model(requested: &str, served: &str) -> bool {
    served == requested
        || served
            .strip_prefix(requested)
            .and_then(|tail| tail.strip_prefix('-'))
            .is_some_and(|date| date.len() == 8 && date.bytes().all(|b| b.is_ascii_digit()))
}

impl Calibration {
    pub fn compatible(
        &self,
        provider: ClassifierProvider,
        policy: SelectionPolicy,
        fingerprint: Option<&str>,
    ) -> bool {
        self.schema_version == 1
            && self.profile == "factor_empirical_bins_v1"
            && self.provider == provider
            && policy == SelectionPolicy::PerMemoryFactorsV1
            && self.selection_policy == policy
            && self.requested_model == provider.model_id()
            && compatible_served_model(&self.requested_model, &self.served_model)
            && hash(&self.training_sha256)
            && hash(&self.profile_sha256)
            && fingerprint == Some(self.profile_sha256.as_str())
            && [
                &self.factors.same_context,
                &self.factors.requested_fact,
                &self.factors.premise_correction,
            ]
            .iter()
            .all(|bins| valid_bins(bins))
    }
    pub fn apply(&self, raw: &MemoryFactorScores) -> Option<MemoryFactorScores> {
        if !raw.valid() {
            return None;
        }
        let map = |bins: &[Bin], v: f64| bins.iter().find(|b| v <= b.upper).map(Bin::value);
        Some(MemoryFactorScores {
            same_context: map(&self.factors.same_context, raw.same_context)?,
            requested_fact: map(&self.factors.requested_fact, raw.requested_fact)?,
            premise_correction: map(&self.factors.premise_correction, raw.premise_correction)?,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empirical_rates_are_smoothed_and_never_select_memories() {
        let bins = vec![
            Bin {
                upper: 0.5,
                positives: 0,
                n: 8,
            },
            Bin {
                upper: 1.0,
                positives: 8,
                n: 8,
            },
        ];
        assert!(valid_bins(&bins));
        assert_eq!(bins[0].value(), 0.1);
        assert_eq!(bins[1].value(), 0.9);
        assert!(!valid_bins(&[Bin {
            upper: 1.0,
            positives: 2,
            n: 1
        }]));
        assert!(!valid_bins(&[Bin {
            upper: 0.5,
            positives: 0,
            n: 1
        }]));
        assert!(!valid_bins(&[Bin {
            upper: 1.0,
            positives: 0,
            n: 0
        }]));
    }
}

#[cfg(test)]
mod model_identity_tests {
    use super::*;
    #[test]
    fn an_observed_snapshot_is_distinct_from_the_requested_alias() {
        let alias = "openai/gpt-6-luna-decisions";
        assert!(compatible_served_model(alias, alias));
        assert!(compatible_served_model(
            alias,
            "openai/gpt-6-luna-decisions-20261006"
        ));
        for wrong in [
            "openai/gpt-6-luna-decisions-evil",
            "openai/gpt-6-luna-decisions-20261006-other",
            "openai/gpt-6-luna-decisions-other-20261006",
            "other/gpt-6-luna-decisions-20261006",
        ] {
            assert!(!compatible_served_model(alias, wrong));
        }
    }
}
