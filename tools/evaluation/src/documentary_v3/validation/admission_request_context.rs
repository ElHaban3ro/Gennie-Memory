//! Optional admission context applicability profile, never a predicate equivalence.
use super::*;
pub(super) fn validate(c: &mut Check<'_>, vocabulary: &Value) {
    let Some(profile) = vocabulary.get("admission_request_context") else {
        return;
    };
    let path = "/vocabulary/admission_request_context";
    if !c.shape(profile, path, &["version", "rows"])
        || profile["version"] != "admission_request_context_v1"
        || !profile["rows"].as_array().is_some_and(|r| !r.is_empty())
    {
        c.issue("invalid_shape", path);
        return;
    }
    let mut ids = BTreeSet::new();
    for row in array(&profile["rows"]) {
        if !c.shape(
            row,
            path,
            &[
                "id",
                "link_id",
                "target_attribute",
                "request_slot",
                "setting_key",
                "premise_predicate",
                "premise_subject_role",
                "premise_setting_role",
            ],
        ) || !row["id"].as_str().is_some_and(|s| !s.trim().is_empty())
            || !ids.insert(row["id"].to_string())
            || row["request_slot"] != "value"
            || row["setting_key"] != "mode"
            || row["premise_predicate"] != "operates_mode"
            || row["premise_subject_role"] != "subject"
            || row["premise_setting_role"] != "mode"
            || !array(&vocabulary["fixed_attribute_rows"]).contains(&row["target_attribute"])
        {
            c.issue("invalid_shape", path);
            continue;
        }
        let links: Vec<_> = array(&vocabulary["relevance_links"])
            .iter()
            .filter(|l| l["id"] == row["link_id"])
            .collect();
        if links.len() != 1 {
            c.issue("unknown_vocabulary", path);
            continue;
        }
        let l = links[0];
        if l["source_predicate"] != "admits"
            || l["target_predicate"] != "property"
            || l["bindings"] != json!([["source.subject", "target.subject"]])
            || l["carry_roles"] != json!(["source.object"])
            || l["conclusion"] != "partial_intermediate_only"
            || l["max_hops"] != 1
            || l["allow_reverse"] != false
            || l["allow_transitive_composition"] != false
        {
            c.issue("invalid_shape", path);
        }
        for (predicate, roles) in [
            ("admits", vec!["subject", "object"]),
            ("property", vec!["subject"]),
            ("operates_mode", vec!["subject", "mode"]),
        ] {
            let rows: Vec<_> = array(&vocabulary["predicate_rows"])
                .iter()
                .filter(|r| r["id"] == predicate)
                .collect();
            if rows.len() != 1 || roles.iter().any(|role| rows[0]["roles"][*role] != "ref") {
                c.issue("unknown_vocabulary", path);
            }
        }
    }
}
