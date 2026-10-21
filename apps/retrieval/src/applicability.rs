//! Grounded referent contract. Exact agreement cannot prove extraction completeness.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub dimension: String,
    pub quote: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Witness {
    pub version: String,
    pub query_bindings: Vec<Binding>,
    pub memory_bindings: Vec<Binding>,
    pub relation: String,
    pub coverage: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct WitnessUsage {
    pub model: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reported_cost_usd: Option<f64>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ApplicabilityTrace {
    pub proposal_id: String,
    pub proposal_usage: WitnessUsage,
    pub extraction_usage: WitnessUsage,
    pub witness: Witness,
    pub outcome: String,
    pub reasons: Vec<String>,
}
fn normal(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
const DIMENSIONS: [&str; 9] = [
    "subject",
    "item",
    "edition",
    "site",
    "team",
    "environment",
    "time",
    "condition",
    "person",
];
fn bindings<'a>(
    rows: &'a [Binding],
    source: &str,
) -> Result<BTreeMap<&'a str, BTreeSet<String>>, String> {
    if rows.len() > 16 {
        return Err("too_many_bindings".into());
    }
    let mut map: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for row in rows {
        if !DIMENSIONS.contains(&row.dimension.as_str())
            || row.quote.trim().is_empty()
            || row.quote.len() > 512
            || !source.contains(&row.quote)
        {
            return Err("ungrounded_binding".into());
        }
        if !map
            .entry(row.dimension.as_str())
            .or_default()
            .insert(normal(&row.quote))
        {
            return Err("duplicate_binding".into());
        }
    }
    Ok(map)
}
/// Err means malformed evidence, not a reliable semantic abstention.
pub fn check(
    query: &str,
    memory: &str,
    witness: &Witness,
) -> Result<(String, Vec<String>), String> {
    if witness.version != "applicability-witness-v1"
        || !["complete", "uncertain"].contains(&witness.coverage.as_str())
        || ![
            "supports_answer",
            "refutes_premise",
            "pending_or_partial",
            "unrelated",
            "unknown",
        ]
        .contains(&witness.relation.as_str())
    {
        return Err("invalid_witness_contract".into());
    }
    let q = bindings(&witness.query_bindings, query)?;
    let m = bindings(&witness.memory_bindings, memory)?;
    if witness.coverage != "complete" || !q.contains_key("subject") {
        return Ok((
            "not_established".into(),
            vec!["incomplete_extraction".into()],
        ));
    }
    let mut missing = Vec::new();
    let mut differences = Vec::new();
    for (dimension, value) in &q {
        match m.get(dimension) {
            None => missing.push(format!("missing_memory_{dimension}")),
            Some(actual) if !value.is_subset(actual) => {
                differences.push(format!("different_{dimension}"))
            }
            _ => (),
        }
    }
    if !differences.is_empty() {
        return Ok(("incompatible".into(), differences));
    }
    if !missing.is_empty() {
        return Ok(("not_established".into(), missing));
    }
    if ["unrelated", "unknown"].contains(&witness.relation.as_str()) {
        return Ok(("not_established".into(), vec!["no_useful_relation".into()]));
    }
    Ok(("accepted".into(), Vec::new()))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn witness() -> Witness {
        Witness {
            version: "applicability-witness-v1".into(),
            query_bindings: vec![
                Binding {
                    dimension: "subject".into(),
                    quote: "Atlas".into(),
                },
                Binding {
                    dimension: "edition".into(),
                    quote: "azul".into(),
                },
            ],
            memory_bindings: vec![
                Binding {
                    dimension: "subject".into(),
                    quote: "Atlas".into(),
                },
                Binding {
                    dimension: "edition".into(),
                    quote: "roja".into(),
                },
            ],
            relation: "pending_or_partial".into(),
            coverage: "complete".into(),
        }
    }
    #[test]
    fn mismatched_and_unknown_context_do_not_pass() {
        let mut w = witness();
        assert_eq!(
            check("Atlas azul", "Atlas roja", &w).unwrap().0,
            "incompatible"
        );
        w.memory_bindings.pop();
        assert_eq!(
            check("Atlas azul", "Atlas roja", &w).unwrap().0,
            "not_established"
        );
    }
    #[test]
    fn fabricated_quote_and_duplicate_dimension_are_invalid() {
        let mut w = witness();
        w.memory_bindings[1].quote = "azul".into();
        assert!(check("Atlas azul", "Atlas roja", &w).is_err());
        let mut w = witness();
        w.query_bindings.push(w.query_bindings[0].clone());
        assert!(check("Atlas azul", "Atlas roja", &w).is_err());
    }
    #[test]
    fn multiple_conditions_are_grounded_and_memory_may_have_extra_values() {
        let mut w = witness();
        w.query_bindings[1] = Binding {
            dimension: "condition".into(),
            quote: "diurna".into(),
        };
        w.memory_bindings[1] = Binding {
            dimension: "condition".into(),
            quote: "diurna".into(),
        };
        w.query_bindings.push(Binding {
            dimension: "condition".into(),
            quote: "presencial".into(),
        });
        w.memory_bindings.push(Binding {
            dimension: "condition".into(),
            quote: "presencial".into(),
        });
        w.memory_bindings.push(Binding {
            dimension: "condition".into(),
            quote: "gratuita".into(),
        });
        assert_eq!(
            check(
                "Atlas diurna presencial",
                "Atlas diurna presencial gratuita",
                &w
            )
            .unwrap()
            .0,
            "accepted"
        );
    }
    #[test]
    fn compatible_correction_is_allowed_but_uncertain_extraction_is_not() {
        let mut w = witness();
        w.memory_bindings[1].quote = "azul".into();
        w.relation = "refutes_premise".into();
        assert_eq!(
            check("Atlas azul", "Atlas azul fue cancelado", &w)
                .unwrap()
                .0,
            "accepted"
        );
        w.coverage = "uncertain".into();
        assert_eq!(
            check("Atlas azul", "Atlas azul fue cancelado", &w)
                .unwrap()
                .0,
            "not_established"
        );
    }
}
