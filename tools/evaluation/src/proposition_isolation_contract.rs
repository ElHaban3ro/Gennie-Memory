//! Research ablation; role-only output never fabricates grounding judgments.
use serde_json::Value;
#[cfg(test)]
use serde_json::json;
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    Joint,
    RoleSource,
    RoleOnly,
}
impl Arm {
    pub fn name(self) -> &'static str {
        match self {
            Self::Joint => "joint",
            Self::RoleSource => "role_source",
            Self::RoleOnly => "role_only",
        }
    }
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "joint" => Ok(Self::Joint),
            "role_source" => Ok(Self::RoleSource),
            "role_only" => Ok(Self::RoleOnly),
            _ => Err("unknown_isolation_arm".into()),
        }
    }
    pub fn payload(self, mut full: Value) -> Value {
        if self != Self::Joint {
            full["questions"]
                .as_object_mut()
                .unwrap()
                .retain(|k, _| k.ends_with("_role"));
        }
        if self == Self::RoleOnly {
            full["state"].as_object_mut().unwrap().remove("source");
        }
        full
    }
}
pub fn order(index: usize) -> [Arm; 3] {
    use Arm::*;
    [
        [Joint, RoleSource, RoleOnly],
        [RoleOnly, RoleSource, Joint],
        [RoleSource, Joint, RoleOnly],
        [RoleOnly, Joint, RoleSource],
        [Joint, RoleOnly, RoleSource],
        [RoleSource, RoleOnly, Joint],
    ][index % 6]
}
pub fn decode_roles(body: &Value, request: &Value) -> Result<Vec<String>, String> {
    if request["model"] != "perplexity/pplx-decider-v1.2"
        || body["model"] != "perplexity/pplx-decider-v1.2-20261016"
    {
        return Err("unexpected_model_revision".into());
    }
    let questions = request["questions"]
        .as_object()
        .ok_or("missing_questions")?;
    let answers = body["answers"].as_object().ok_or("missing_answers")?;
    let names: BTreeSet<_> = ["claim_00_role", "claim_01_role"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    if questions.keys().cloned().collect::<BTreeSet<_>>() != names
        || answers.keys().cloned().collect::<BTreeSet<_>>() != names
    {
        return Err("unexpected_role_namespace".into());
    }
    let mut roles = Vec::new();
    for key in ["claim_00_role", "claim_01_role"] {
        let a = &answers[key];
        let choice = a["choice"]
            .as_str()
            .filter(|s| matches!(*s, "A" | "B" | "C" | "none"))
            .ok_or("unknown_role")?;
        if a["type"] != "choice" {
            return Err("invalid_type".into());
        }
        let probabilities = a["probabilities"]
            .as_object()
            .ok_or("missing_probabilities")?;
        let expected: BTreeSet<_> = ["A", "B", "C", "none"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        if probabilities.keys().cloned().collect::<BTreeSet<_>>() != expected
            || questions[key]["criteria"]
                .as_object()
                .ok_or("missing_criteria")?
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>()
                != expected
        {
            return Err("invalid_probability_or_criteria_namespace".into());
        }
        let values: Vec<_> = probabilities
            .values()
            .map(|v| {
                v.as_f64()
                    .filter(|v| v.is_finite() && (0.0..=1.).contains(v))
                    .ok_or("invalid_probability")
            })
            .collect::<Result<_, _>>()?;
        if (values.iter().sum::<f64>() - 1.).abs() > 0.02 {
            return Err("unnormalized_probabilities".into());
        }
        roles.push(choice.to_owned());
    }
    Ok(roles)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn full() -> Value {
        json!({"model":"perplexity/pplx-decider-v1.2","state":{"source":"PRIVATE source","question":"Exact query","propositions":{"claim_00":"assertion","claim_01":"other"},"role_rules":"unchanged","grounding_rules":"unchanged"},"questions":{"claim_00_grounded":{"instructions":"support"},"claim_01_grounded":{"instructions":"support2"},"claim_00_role":{"type":"choice","instructions":"assume true","criteria":{"A":"part","B":"refute","C":"contrast","none":"unrelated"}},"claim_01_role":{"type":"choice","instructions":"assume true","criteria":{"A":"part","B":"refute","C":"contrast","none":"unrelated"}}}})
    }
    #[test]
    fn ablations_change_only_grounding_questions_then_source_field() {
        let f = full();
        assert_eq!(Arm::Joint.payload(f.clone()), f);
        let mut with = Arm::RoleSource.payload(f.clone());
        assert_eq!(with["state"], f["state"]);
        assert_eq!(with["questions"].as_object().unwrap().len(), 2);
        for k in ["claim_00_grounded", "claim_01_grounded"] {
            with["questions"][k] = f["questions"][k].clone();
        }
        assert_eq!(with, f);
        let with = Arm::RoleSource.payload(f.clone());
        let mut isolated = Arm::RoleOnly.payload(f);
        assert!(isolated["state"].get("source").is_none());
        assert!(!isolated.to_string().contains("PRIVATE source"));
        isolated["state"]["source"] = with["state"]["source"].clone();
        assert_eq!(isolated, with);
    }
    #[test]
    fn role_only_namespace_does_not_require_or_fabricate_grounding() {
        let p = Arm::RoleOnly.payload(full());
        let mut b = json!({"model":"perplexity/pplx-decider-v1.2-20261016","answers":{}});
        for k in ["claim_00_role", "claim_01_role"] {
            b["answers"][k] = json!({"type":"choice","choice":"C","probabilities":{"A":0.,"B":0.,"C":1.,"none":0.}});
        }
        assert_eq!(decode_roles(&b, &p).unwrap(), vec!["C", "C"]);
        b["answers"]["claim_00_grounded"] = json!({"choice":"yes"});
        assert!(decode_roles(&b, &p).is_err());
        b["answers"]
            .as_object_mut()
            .unwrap()
            .remove("claim_00_grounded");
        b["answers"]["claim_01_role"]["probabilities"]["none"] = json!(1.);
        assert!(decode_roles(&b, &p).is_err());
    }
    #[test]
    fn order_is_balanced_over_eighteen_cases() {
        let mut counts = std::collections::BTreeMap::new();
        for i in 0..18 {
            let o = order(i);
            let names: Vec<_> = o.iter().map(|a| a.name()).collect();
            assert_eq!(names.iter().collect::<BTreeSet<_>>().len(), 3);
            *counts.entry(names).or_insert(0) += 1;
        }
        assert_eq!(counts.len(), 6);
        assert!(counts.values().all(|v| *v == 3));
    }
}
