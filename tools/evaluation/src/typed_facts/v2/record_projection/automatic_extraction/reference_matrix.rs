//! Manual reference-catalog expressivity controls, never an extraction benchmark.
use super::*;
use crate::{distributed_reference, reference_catalog};
#[derive(Clone, Copy)]
enum Mode<'a> {
    Legacy,
    Distributed(&'a distributed_reference::Registry),
}
fn decode_side(
    kind: &str,
    side: &Value,
    ontology: &Value,
    o: &Vocabulary,
    mode: Mode<'_>,
) -> Value {
    let materialized = match reference_catalog::materialize(&side["catalog"], &side["stage2"]) {
        Ok(v) => v,
        Err(e) => {
            return json!({"materialized":null,"normalized":null,"error_stage":"catalog_materialization","error":e});
        }
    };
    let Some(text) = side["text"].as_str() else {
        return json!({"materialized":materialized,"normalized":null,"error_stage":"input","error":"control sin texto"});
    };
    let translated = match mode {
        Mode::Legacy => {
            contract::grounded_references::translate(kind, text, ontology, &materialized["raw"])
                .map(|v| (v, None))
        }
        Mode::Distributed(registry) => {
            distributed_reference::translate(kind, text, ontology, &materialized["raw"], registry)
                .map(|v| (v["legacy"].clone(), Some(v)))
        }
    };
    let (legacy, evidence) = match translated {
        Ok(v) => v,
        Err(e) => {
            return json!({"materialized":materialized,"normalized":null,"error_stage":"grounded_translation","error":e});
        }
    };
    let mut result = match decoded(kind, text, &legacy, o) {
        Ok(normalized) => {
            json!({"materialized":materialized,"translated":legacy,"normalized":normalized,"error_stage":null,"error":null})
        }
        Err(e) => {
            json!({"materialized":materialized,"translated":legacy,"normalized":null,"error_stage":"typed_normalization","error":e})
        }
    };
    if let Some(evidence) = evidence {
        result["distributed_evidence"] = evidence;
    }
    result
}
pub fn semantic_view(
    kind: &str,
    side: &Value,
    ontology: &Value,
    registry: &distributed_reference::Registry,
    registry_wire: &Value,
) -> Result<Value, String> {
    let o = vocabulary(ontology)?;
    let stages = decode_side(kind, side, ontology, &o, Mode::Distributed(registry));
    let raw = stages["materialized"]["raw"].clone();
    if !raw.is_object() {
        return Err("candidato sin materialización".into());
    }
    Ok(
        json!({"kind":kind,"text":side["text"],"raw":raw,"normalized":stages["normalized"],"normalization_error":stages["error"],"ontology":ontology,"registry":registry_wire}),
    )
}
pub fn evaluate(input: &Value) -> Result<Value, String> {
    evaluate_mode(input, Mode::Legacy)
}
fn evaluate_mode(input: &Value, mode: Mode<'_>) -> Result<Value, String> {
    evaluate_checked(
        input,
        mode,
        &[
            "LA01", "LA02", "LA03", "LA04", "LA05", "LA06", "LA07", "LA08",
        ],
    )
}
fn evaluate_checked(input: &Value, mode: Mode<'_>, expected_ids: &[&str]) -> Result<Value, String> {
    if input["version"] != "reference-catalog-controls-v1" {
        return Err("versión de controles incorrecta".into());
    }
    let mut rows = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for case in input["cases"].as_array().ok_or("faltan controles")? {
        let id = case["id"].as_str().ok_or("control sin ID")?;
        if !ids.insert(id) {
            return Err("control duplicado".into());
        }
        let ontology = &case["ontology"];
        let o = vocabulary(ontology)?;
        // Both sides attempted even when the first fails: no filtered denominator.
        let source = decode_side("source", &case["source"], ontology, &o, mode);
        let question = decode_side("question", &case["question"], ontology, &o, mode);
        let source_error = &source["error"];
        let question_error = &question["error"];
        let projection = if source["normalized"].is_object() && question["normalized"].is_object() {
            Some(project(
                &o,
                &record_question(&question["normalized"]["frame"], &o)?,
                &source["normalized"],
            )?)
        } else {
            None
        };
        let necessary = &case["necessary_condition"];
        let passes = source_error.is_null()
            && question_error.is_null()
            && projection.as_ref().is_some_and(|r| {
                (necessary.get("world_roles_contains").is_none()
                    || r["world_roles"]
                        .as_array()
                        .is_some_and(|a| a.contains(&necessary["world_roles_contains"])))
                    && r["selected"] == necessary["selected"]
                    && (necessary.get("qualification_required").is_none()
                        || r["qualification_required"] == necessary["qualification_required"])
            });
        rows.push(json!({"id":id,"risk":case["risk"],"source_error":source_error,"question_error":question_error,"source_materialized":&source["materialized"],"question_materialized":&question["materialized"],"source_stages":source,"question_stages":question,"source_normalized":&source["normalized"],"question_normalized":&question["normalized"],"projection":projection,"necessary_condition":necessary,"passes_necessary_condition":passes,"not_a_model_extraction_result":true}));
    }
    if ids != expected_ids.iter().copied().collect() {
        return Err("faltan controles requeridos o hay controles ajenos".into());
    }
    let passed = rows
        .iter()
        .filter(|r| r["passes_necessary_condition"] == true)
        .count();
    Ok(
        json!({"version":"reference-catalog-v1-matrix","controls":rows,"control_count":rows.len(),"necessary_conditions_met":passed,"passes_all_necessary_conditions":passed==expected_ids.len(),"sufficient_to_prove_semantic_fidelity":false,"source_question_inputs_isolated":true,"source_facts_and_references_manual":true,"provider_calls":0,"promoted":false}),
    )
}
pub fn audit(input: &Path, output: &Path) -> Result<(), String> {
    let bytes = std::fs::read(input).map_err(|e| e.to_string())?;
    let value =
        serde_json::from_slice(&bytes).map_err(|e| format!("control JSON inválido: {e}"))?;
    let mut report = evaluate(&value)?;
    use sha2::Digest;
    report["input_sha256"] = json!(
        sha2::Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    std::io::Write::write_all(
        &mut file,
        serde_json::to_string_pretty(&report)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
}
pub fn audit_distributed(input: &Path, registry_file: &Path, output: &Path) -> Result<(), String> {
    let bytes = std::fs::read(input).map_err(|e| e.to_string())?;
    let fixture: Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("control JSON inválido: {e}"))?;
    if fixture["version"] != "distributed-reference-controls-v1" {
        return Err("versión distribuida incorrecta".into());
    }
    let registry_bytes = std::fs::read(registry_file).map_err(|e| e.to_string())?;
    let registry = distributed_reference::Registry::parse(
        &registry_bytes,
        fixture["registry_sha256"]
            .as_str()
            .ok_or("falta hash del registro")?,
    )?;
    let mut batches = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for b in fixture["batches"].as_array().ok_or("faltan lotes")? {
        let id = b["id"].as_str().ok_or("lote sin ID")?;
        if !seen.insert(id) {
            return Err("lote duplicado".into());
        }
        batches.push(
            json!({"id":id,"result":evaluate_mode(&b["input"],Mode::Distributed(&registry))?}),
        );
    }
    if seen != ["adapted_original", "clarified"].into_iter().collect() {
        return Err("deben conservarse original y aclarado".into());
    }
    let mut guards = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for c in fixture["scope_controls"]
        .as_array()
        .ok_or("faltan controles de alcance")?
    {
        let id = c["id"].as_str().ok_or("control sin ID")?;
        if !ids.insert(id) {
            return Err("control duplicado".into());
        }
        let o = vocabulary(&c["ontology"])?;
        let kind = c["kind"].as_str().ok_or("tipo de control ausente")?;
        if !["source", "question"].contains(&kind) {
            return Err("tipo de control inválido".into());
        }
        let result = decode_side(
            kind,
            &c["side"],
            &c["ontology"],
            &o,
            Mode::Distributed(&registry),
        );
        let expected = c["expect_translated"]
            .as_bool()
            .ok_or("falta resultado estructural esperado")?;
        let passes = result["normalized"].is_object() == expected;
        guards.push(json!({"id":id,"result":result,"passes_declared_structural_control":passes,"not_semantic_validation":true}));
    }
    let expected_guards = [
        "holder_chain",
        "negated_owner",
        "corrigible_owner",
        "planned_owner",
        "normative_owner",
        "unknown_scope",
        "owner_condition_not_retained",
        "subclass_not_equivalence",
        "head_cut",
        "unknown_class",
        "bounded_owner_during_observation",
        "historical_owner_interval_dropped",
        "historical_owner_other_interval",
    ];
    if ids != expected_guards.into_iter().collect() {
        return Err("faltan controles de alcance requeridos o hay controles ajenos".into());
    }
    let counter_input = json!({"version":"reference-catalog-controls-v1","cases":fixture["semantic_counterexamples"]});
    let counter_result = evaluate_checked(
        &counter_input,
        Mode::Distributed(&registry),
        &[
            "subclasses_distinct",
            "false_exact_equivalence",
            "false_interval_value_quote",
        ],
    )?;
    let mut report = json!({"version":"distributed-reference-v1-matrix","input_sha256":distributed_reference::sha256(&bytes),"registry_sha256":distributed_reference::sha256(&registry_bytes),"batches":batches,"scope_controls":guards,"semantic_counterexamples":counter_result,"semantics_require_independent_review":true,"sufficient_to_prove_semantic_fidelity":false,"provider_calls":0,"promoted":false});
    report["all_declared_necessary_conditions_met"] = json!(
        report["semantic_counterexamples"]["passes_all_necessary_conditions"] == true
            && report["batches"]
                .as_array()
                .unwrap()
                .iter()
                .all(|b| b["result"]["passes_all_necessary_conditions"] == true)
            && report["scope_controls"]
                .as_array()
                .unwrap()
                .iter()
                .all(|g| g["passes_declared_structural_control"] == true)
    );
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    std::io::Write::write_all(
        &mut file,
        serde_json::to_string_pretty(&report)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_matrix_exposes_distributed_owner_and_class_mismatch_without_relaxation() {
        let input: Value = serde_json::from_str(include_str!(
            "../../../../../../../tests/fixtures/reference-catalog-controls-v1.json"
        ))
        .unwrap();
        let report = evaluate(&input).unwrap();
        assert_eq!(report["control_count"], 8);
        assert_eq!(report["passes_all_necessary_conditions"], false);
        let rows = report["controls"].as_array().unwrap();
        let one = rows.iter().find(|r| r["id"] == "LA01").unwrap();
        assert!(one["source_error"].is_string());
        assert!(one["source_materialized"]["raw"].is_object());
        assert_eq!(one["source_stages"]["error_stage"], "grounded_translation");
        assert_eq!(
            one["source_materialized"]["raw"]["document"]["certain"][0]["subject"],
            input["cases"][0]["source"]["catalog"][0]["reference"]
        );
        let four = rows.iter().find(|r| r["id"] == "LA04").unwrap();
        assert!(four["source_error"].is_null() && four["question_error"].is_null());
        assert_eq!(four["projection"]["selected"], false);
    }
    #[test]
    fn distributed_matrix_preserves_all_controls_and_exposes_false_exact_binding() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let input = root.join("tests/fixtures/distributed-reference-controls-v1.json");
        let registry = root.join("tests/fixtures/reference-classes-v1.json");
        let output = std::env::temp_dir().join(format!(
            "gennie-memory-distributed-matrix-{}.json",
            std::process::id()
        ));
        audit_distributed(&input, &registry, &output).unwrap();
        let report: Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
        assert_eq!(report["batches"].as_array().unwrap().len(), 2);
        for batch in report["batches"].as_array().unwrap() {
            assert_eq!(batch["result"]["control_count"], 8);
            assert_eq!(batch["result"]["necessary_conditions_met"], 8);
        }
        assert_eq!(report["scope_controls"].as_array().unwrap().len(), 13);
        assert!(
            report["scope_controls"]
                .as_array()
                .unwrap()
                .iter()
                .all(|g| g["passes_declared_structural_control"] == true)
        );
        assert_eq!(report["semantic_counterexamples"]["control_count"], 3);
        assert_eq!(
            report["semantic_counterexamples"]["necessary_conditions_met"],
            1
        );
        assert_eq!(report["all_declared_necessary_conditions_met"], false);
        let bad = report["semantic_counterexamples"]["controls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == "false_exact_equivalence")
            .unwrap();
        assert_eq!(bad["projection"]["selected"], true);
        assert_eq!(bad["passes_necessary_condition"], false);
        assert_eq!(
            bad["source_stages"]["distributed_evidence"]["semantic_assertions_verified"],
            false
        );
        assert!(audit_distributed(&input, &registry, &output).is_err());
        std::fs::remove_file(output).unwrap();
    }
}
