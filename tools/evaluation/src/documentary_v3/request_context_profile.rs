//! Closed, opt-in documentary context carrier permission; no predicate equivalence.
use serde_json::Value;
use std::collections::BTreeSet;

pub(crate) const VERSION: &str = "documentary_request_context_v1";
pub(crate) fn enabled(vocabulary: &Value) -> bool {
    vocabulary.get("request_context_profile").is_some()
}
pub(crate) fn valid(vocabulary: &Value) -> bool {
    let Some(profile) = vocabulary.get("request_context_profile") else {
        return true;
    };
    let Some(object) = profile.as_object() else {
        return false;
    };
    if object.len() != 2 || profile["version"] != VERSION {
        return false;
    }
    let Some(features) = profile["features"].as_array() else {
        return false;
    };
    if features.is_empty() || features.len() > 12 {
        return false;
    }
    let mut seen = BTreeSet::new();
    if !features.iter().all(|feature| {
        feature.as_str().is_some_and(|f| {
            matches!(
                f,
                "nominal_during"
                    | "lexical_clause_context"
                    | "linguistic_composition_until_measurement_v1"
                    | "nonagentive_nominal_event_with_contexts_v1"
                    | "nominal_possessive"
                    | "existential_conclusion_projection"
                    | "nominal_complement_de"
                    | "normative_action_relation"
                    | "normative_question_indeterminacy"
                    | "normative_open_pattern"
                    | "pending_task_scope"
                    | "relative_nonoccurrence_bound"
            ) && seen.insert(f)
        })
    }) {
        return false;
    }
    (!seen.contains("relative_nonoccurrence_bound")
        || [
            ("fixed_action_rows", "switch_off"),
            ("fixed_attribute_rows", "duration"),
        ]
        .iter()
        .all(|(key, required)| {
            vocabulary[*key]
                .as_array()
                .is_some_and(|rows| rows.iter().any(|row| row.as_str() == Some(*required)))
        }))
        && (!seen.contains("linguistic_composition_until_measurement_v1")
            || vocabulary["fixed_attribute_rows"]
                .as_array()
                .is_some_and(|rows| rows.iter().any(|r| r == "duration")))
        && (!seen.contains("nonagentive_nominal_event_with_contexts_v1")
            || (seen.contains("nominal_complement_de")
                && vocabulary["fixed_action_rows"]
                    .as_array()
                    .is_some_and(|rows| rows.iter().any(|r| r == "remove"))))
        && (!seen.contains("normative_question_indeterminacy")
            || seen.contains("normative_action_relation"))
        && (!seen.contains("normative_open_pattern") || seen.contains("normative_action_relation"))
        && (!seen.contains("pending_task_scope")
            || vocabulary["fixed_attribute_rows"]
                .as_array()
                .is_some_and(|rows| rows.iter().any(|row| row == "pending_pages")))
}
pub(crate) fn permits(vocabulary: &Value, feature: &str) -> bool {
    enabled(vocabulary)
        && valid(vocabulary)
        && vocabulary["request_context_profile"]["features"]
            .as_array()
            .is_some_and(|features| features.iter().any(|f| f.as_str() == Some(feature)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn absence_never_grants_carrier_permissions() {
        let v = json!({});
        assert!(valid(&v));
        assert!(!enabled(&v));
        assert!(!permits(&v, "nominal_during"));
    }
    #[test]
    fn profile_is_closed_and_checks_feature_dependency() {
        for profile in [
            Value::Null,
            json!({"version":VERSION,"features":[]}),
            json!({"version":VERSION,"features":["normative_open_pattern"]}),
            json!({"version":VERSION,"features":["nominal_during","nominal_during"]}),
            json!({"version":VERSION,"features":["route"]}),
            json!({"version":VERSION,"features":["nominal_during"],"extra":false}),
            json!({"version":"old","features":["nominal_during"]}),
        ] {
            let v = json!({"request_context_profile":profile});
            assert!(enabled(&v));
            assert!(!valid(&v), "{v}");
            assert!(!permits(&v, "nominal_during"));
        }
        assert!(permits(
            &json!({"request_context_profile":{"version":VERSION,"features":["normative_action_relation","normative_open_pattern"]}}),
            "normative_open_pattern"
        ));
    }
    #[test]
    fn pending_scope_requires_registered_attribute_and_keeps_closed_features() {
        let mut v = json!({"request_context_profile":{"version":VERSION,"features":["pending_task_scope"]}});
        assert!(!valid(&v));
        v["fixed_attribute_rows"] = json!(["pending_pages"]);
        assert!(permits(&v, "pending_task_scope"));
        v["request_context_profile"]["features"] = json!([
            "nominal_during",
            "normative_action_relation",
            "normative_open_pattern",
            "pending_task_scope"
        ]);
        assert!(valid(&v));
        v["request_context_profile"]["features"]
            .as_array_mut()
            .unwrap()
            .push(json!("pending_task_scope"));
        assert!(!valid(&v));
    }
}
