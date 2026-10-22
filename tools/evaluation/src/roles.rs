//! Offline specification checker for annotated roles, never an automatic extractor.
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

fn field<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("missing_{key}"))
}
fn rows<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    v[key]
        .as_array()
        .filter(|a| a.len() <= 16)
        .ok_or_else(|| format!("invalid_{key}"))
}
fn grounded(v: &Value, key: &str, source: &str) -> Result<(), String> {
    let quote = field(v, key)?;
    if quote.len() > 512 || !source.contains(quote) {
        return Err(format!("ungrounded_{key}"));
    }
    Ok(())
}
fn normal(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Roles and completeness are annotations supplied by the caller, not proven here.
pub fn check(query: &str, memory: &str, w: &Value) -> Result<&'static str, String> {
    if w["version"] != "evidence-roles-v1" {
        return Err("invalid_version".into());
    }
    let identity = rows(w, "identity")?;
    let premises = rows(w, "premises")?;
    let requested = rows(w, "requested")?;
    let facts = rows(w, "facts")?;
    let mut dimensions = BTreeSet::new();
    let mut mismatch = false;
    let mut missing = false;
    for binding in identity {
        let dimension = field(binding, "dimension")?;
        if ![
            "subject",
            "item",
            "edition",
            "site",
            "team",
            "environment",
            "time",
            "condition",
            "person",
        ]
        .contains(&dimension)
            || !dimensions.insert(dimension)
        {
            return Err("invalid_identity_dimension".into());
        }
        grounded(binding, "query_quote", query)?;
        match binding["memory_quote"].as_str() {
            Some(_) => {
                grounded(binding, "memory_quote", memory)?;
                mismatch |= normal(field(binding, "query_quote")?)
                    != normal(field(binding, "memory_quote")?);
            }
            None if binding["memory_quote"].is_null() => missing = true,
            _ => return Err("invalid_memory_quote".into()),
        }
    }
    let mut premise_ids = BTreeSet::new();
    for p in premises {
        if !premise_ids.insert(field(p, "id")?) {
            return Err("duplicate_premise".into());
        }
        grounded(p, "query_quote", query)?;
    }
    let mut slots = BTreeSet::new();
    for r in requested {
        if !slots.insert(field(r, "slot")?) {
            return Err("duplicate_requested_slot".into());
        }
        grounded(r, "query_quote", query)?;
    }
    let mut useful = false;
    for f in facts {
        grounded(f, "memory_quote", memory)?;
        let target = field(f, "target")?;
        match field(f, "relation")? {
            "answers" | "partial" if slots.contains(target) => useful = true,
            "refutes" if premise_ids.contains(target) => useful = true,
            "unrelated" => (),
            _ => return Err("invalid_fact_target".into()),
        }
    }
    if !["complete", "uncertain"].contains(&field(w, "identity_coverage")?) {
        return Err("invalid_coverage".into());
    }
    if mismatch {
        return Ok("incompatible");
    }
    if missing || !dimensions.contains("subject") || w["identity_coverage"] != "complete" {
        return Ok("not_established");
    }
    Ok(if useful {
        "accepted"
    } else {
        "not_established"
    })
}

pub fn run(input: &Path, output: &Path) -> Result<(), String> {
    let bytes = std::fs::read(input).map_err(|e| e.to_string())?;
    let bank: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if bank["version"] != "role-contract-development-v1" || bank["synthetic"] != true {
        return Err("invalid_role_bank".into());
    }
    let cases = bank["cases"]
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or("empty_role_bank")?;
    let mut ids = BTreeSet::new();
    let mut results = Vec::new();
    for c in cases {
        let id = field(c, "id")?;
        if !ids.insert(id) {
            return Err("duplicate_case".into());
        }
        let outcome = check(field(c, "query")?, field(c, "memory")?, &c["witness"]);
        let actual = match outcome {
            Ok(o) => o.to_string(),
            Err(e) => format!("invalid:{e}"),
        };
        let expected = field(c, "expected")?;
        results
            .push(json!({"id":id,"expected":expected,"actual":actual,"passed":expected==actual}));
    }
    let passed = results.iter().filter(|r| r["passed"] == true).count();
    let report = json!({"version":"role-contract-report-v1", "input_sha256":crate::campaign::digest(input)?,
        "cases":cases.len(),"passed":passed,"failed":cases.len()-passed,"results":results,
        "annotated_roles_only":true,"automatic_extraction_tested":false,"retrieval_quality_measured":false,
        "remote_calls":0,"reported_remote_cost_usd":0.0,"passes_local_contract":passed==cases.len()});
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    file.write_all(
        (serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n").as_bytes(),
    )
    .map_err(|e| e.to_string())?;
    if passed != cases.len() {
        return Err("role_contract_failures".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn annotated_development_contract_and_mutations() {
        let bank: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/evidence-roles-v1/development.json"
        ))
        .unwrap();
        let cases = bank["cases"].as_array().unwrap();
        assert!(cases.len() >= 16);
        for c in cases {
            let outcome = check(
                c["query"].as_str().unwrap(),
                c["memory"].as_str().unwrap(),
                &c["witness"],
            );
            let actual = outcome
                .map(str::to_string)
                .unwrap_or_else(|e| format!("invalid:{e}"));
            assert_eq!(actual, c["expected"].as_str().unwrap(), "{}", c["id"]);
        }
        let c = cases.iter().find(|c| c["expected"] == "accepted").unwrap();
        let mut w = c["witness"].clone();
        w["facts"][0]["memory_quote"] = json!("invented evidence not in source");
        assert!(
            check(
                c["query"].as_str().unwrap(),
                c["memory"].as_str().unwrap(),
                &w
            )
            .is_err()
        );
        w = c["witness"].clone();
        w["identity"] = json!([]);
        assert_eq!(
            check(
                c["query"].as_str().unwrap(),
                c["memory"].as_str().unwrap(),
                &w
            )
            .unwrap(),
            "not_established"
        );
    }
}
