//! Provenance checks for manually supplied propositions, not a semantic prover.
use crate::{campaign::digest, isolated_provider_probe::read};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("missing_{key}"))
}
fn indexed(v: &Value) -> Result<BTreeMap<&str, &Value>, String> {
    let mut map = BTreeMap::new();
    for row in v.as_array().filter(|a| !a.is_empty()).ok_or("empty_rows")? {
        if map.insert(text(row, "id")?, row).is_some() {
            return Err("duplicate_id".into());
        }
    }
    Ok(map)
}
fn check_values(source: &Value, development: &Value, review: &Value) -> Result<Value, String> {
    let sources = indexed(source)?;
    let labels = indexed(development)?;
    let reviews = indexed(review)?;
    if sources.keys().collect::<Vec<_>>() != labels.keys().collect::<Vec<_>>()
        || sources.keys().collect::<Vec<_>>() != reviews.keys().collect::<Vec<_>>()
    {
        return Err("case_coverage_mismatch".into());
    }
    let mut count = 0;
    let mut grounded = 0;
    let mut roles = BTreeMap::<String, usize>::new();
    for (id, case) in &sources {
        let propositions = indexed(&case["propositions"])?;
        let judgments = indexed(&labels[id]["labels"])?;
        let critique = indexed(&reviews[id]["labels"])?;
        if propositions.len() != 2
            || propositions.keys().collect::<Vec<_>>() != judgments.keys().collect::<Vec<_>>()
            || propositions.keys().collect::<Vec<_>>() != critique.keys().collect::<Vec<_>>()
        {
            return Err("proposition_coverage_mismatch".into());
        }
        let question = text(case, "question")?;
        let source = text(case, "source")?;
        let mut distinct = BTreeSet::new();
        for (pid, prop) in propositions {
            let proposition = text(prop, "text")?;
            if !distinct.insert(proposition) {
                return Err("duplicate_proposition_text".into());
            }
            let label = judgments[pid];
            let judgment = critique[pid];
            let supported = label["grounded"].as_bool().ok_or("grounded_not_boolean")?;
            let role = text(label, "role")?;
            if !matches!(role, "A" | "B" | "C" | "none") {
                return Err("unknown_role".into());
            }
            for row in [label, judgment] {
                if row["source_quote"] != source
                    || row["question_quote"] != question
                    || row["proposition_quote"] != proposition
                {
                    return Err("quote_or_identity_mismatch".into());
                }
            }
            if judgment["grounded"].as_bool() != Some(supported)
                || judgment["role"] != role
                || judgment["ambiguity"] != false
                || text(judgment, "rationale").is_err()
            {
                return Err("review_not_reconciled".into());
            }
            count += 1;
            grounded += usize::from(supported);
            *roles.entry(role.to_owned()).or_default() += 1;
        }
    }
    Ok(
        json!({"cases":sources.len(),"propositions":count,"independent_judgments":count*2,"grounded_propositions":grounded,"role_counts":roles,"semantic_truth_proven":false,"extractor_evaluated":false,"fresh_validation":false}),
    )
}
pub fn validate(d: &Path) -> Result<Value, String> {
    let manifest = read(&d.join("manifest.json"))?;
    if manifest["version"] != "proposition-contribution-v1-reviewed"
        || manifest["provider_calls_before_freeze"] != 0
        || manifest["same_model_family_review"] != true
        || manifest["human_review"] != false
    {
        return Err("unreviewed_manifest".into());
    }
    let files = manifest["files_sha256"]
        .as_object()
        .ok_or("missing_file_hashes")?;
    for required in [
        "source-only.json",
        "author-initial.json",
        "blind-review-initial.json",
        "source-only-reviewed.json",
        "blind-review-final.json",
        "development.json",
        "adjudication.json",
    ] {
        if !files.contains_key(required) {
            return Err("missing_provenance".into());
        }
    }
    for (file, hash) in files {
        if Path::new(file).components().count() != 1 || digest(&d.join(file))? != *hash {
            return Err("bank_hash_mismatch".into());
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let external = manifest["external_sha256"]
        .as_object()
        .ok_or("missing_contract_hashes")?;
    for contract in [
        "docs/evaluation/proposition-contribution-v1-contract.md",
        "docs/evaluation/evidence-contribution-v3-contract.md",
    ] {
        if external.get(contract) != Some(&json!(digest(&root.join(contract))?)) {
            return Err("contract_hash_mismatch".into());
        }
    }
    let source = read(&d.join("source-only-reviewed.json"))?;
    let report = check_values(
        &source,
        &read(&d.join("development.json"))?,
        &read(&d.join("blind-review-final.json"))?,
    )?;
    if report["cases"] != 18 || report["propositions"] != 36 {
        return Err("incomplete_bank".into());
    }
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> (Value, Value, Value) {
        let source = json!([{"id":"x","question":"When?","source":"Planned on Tuesday.","propositions":[{"id":"a","text":"Planned on Tuesday."},{"id":"b","text":"Completed on Tuesday."}]}]);
        let labels = json!([{"id":"x","labels":[{"id":"a","grounded":true,"role":"A","source_quote":"Planned on Tuesday.","question_quote":"When?","proposition_quote":"Planned on Tuesday."},{"id":"b","grounded":false,"role":"none","source_quote":"Planned on Tuesday.","question_quote":"When?","proposition_quote":"Completed on Tuesday."}]}]);
        let mut review = labels.clone();
        for r in review[0]["labels"].as_array_mut().unwrap() {
            r["ambiguity"] = json!(false);
            r["rationale"] = json!("Independent review.");
        }
        (source, labels, review)
    }
    #[test]
    fn unsupported_and_irrelevant_are_independent_and_order_is_not_identity() {
        let (s, l, mut r) = sample();
        r[0]["labels"].as_array_mut().unwrap().reverse();
        let v = check_values(&s, &l, &r).unwrap();
        assert_eq!(v["grounded_propositions"], 1);
        assert_eq!(v["independent_judgments"], 4);
        assert_eq!(v["semantic_truth_proven"], false);
    }
    #[test]
    fn missing_duplicate_and_foreign_review_ids_are_rejected() {
        let (s, l, r) = sample();
        let mut missing = r.clone();
        missing[0]["labels"].as_array_mut().unwrap().pop();
        let mut duplicate = r.clone();
        duplicate[0]["labels"][1]["id"] = json!("a");
        let mut foreign = r;
        foreign[0]["id"] = json!("other");
        for bad in [missing, duplicate, foreign] {
            assert!(check_values(&s, &l, &bad).is_err());
        }
    }
    #[test]
    fn changed_quotes_and_unresolved_or_disagreeing_review_cannot_freeze() {
        let (s, l, r) = sample();
        for (key, value) in [
            ("source_quote", json!("Borrowed source.")),
            ("proposition_quote", json!("Invented.")),
            ("question_quote", json!("Another question?")),
            ("grounded", json!(false)),
            ("role", json!("none")),
            ("ambiguity", json!(true)),
            ("rationale", json!("")),
        ] {
            let mut bad = r.clone();
            bad[0]["labels"][0][key] = value;
            assert!(check_values(&s, &l, &bad).is_err(), "{key}");
        }
    }
}
