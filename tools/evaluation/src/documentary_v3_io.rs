//! File adapter for the prospective V3 evaluator; no provider or product state.
pub mod assembly;
mod context_json;
pub mod delivery;
pub mod evidence_locator;
pub mod lexical_clause_context;
mod packet;
pub mod readings;
pub mod source_context_reference;
use crate::documentary_v3;
use serde_json::{Value, json};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

fn closed(v: &Value, keys: &[&str]) -> Result<(), String> {
    let fields = v.as_object().ok_or("falta objeto de entrada V3")?;
    if fields.len() != keys.len() || keys.iter().any(|k| !fields.contains_key(*k)) {
        return Err("campos de entrada V3 ausentes o ajenos".into());
    }
    Ok(())
}

fn side(v: &Value, vocabulary: &Value) -> Result<documentary_v3::DocumentResult, String> {
    closed(v, &["text", "raw"])?;
    let text = v["text"].as_str().ok_or("falta texto original V3")?;
    Ok(documentary_v3::validate(&v["raw"], text, vocabulary))
}

pub fn run(input: &Path, output: &Path, paired: bool) -> Result<(), String> {
    let bytes = std::fs::read(input).map_err(|e| format!("entrada V3: {e}"))?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("entrada V3 supera el límite local de 16 MiB".into());
    }
    let input = context_json::parse(&bytes)?;
    let report = documentary_v3::context_budget::with_operation(&input["vocabulary"], &input, || -> Result<Value,String> {
    Ok(if paired {
        closed(&input, &["vocabulary", "source", "question", "pair_scope"])?;
        if input["vocabulary"].get("request_context_profile").is_some() { return Err("el contrato request_context V1 exige la entrada contextual explícita".into()); }
        let source = side(&input["source"], &input["vocabulary"])?;
        let question = side(&input["question"], &input["vocabulary"])?;
        let result = documentary_v3::classify_pair(
            &source,
            &question,
            &input["vocabulary"],
            &input["pair_scope"],
        );
        json!({
            "version":"documentary-v3-local-pair-report-v1",
            "source":source,"question":question,"result":result,
            "automatic_extraction_measured":false,
            "semantic_fidelity_proven":false,
            "production_latency_measured":false,
            "new_provider_calls":0
        })
    } else {
        closed(&input, &["vocabulary", "text", "raw"])?;
        let text = input["text"].as_str().ok_or("falta texto original V3")?;
        serde_json::to_value(documentary_v3::validate(
            &input["raw"],
            text,
            &input["vocabulary"],
        ))
        .map_err(|e| format!("serialización V3: {e}"))?
    })
    }).unwrap_or_else(|reason| Ok(json!({"version":"documentary_request_context_limit_v1","source":input["source"],"question":input["question"],"raw":input["raw"],"result":{"status":"ineligible","requests":[],"selected_claim_ids":[],"complete_answer":false,"requires_review":true,"issues":[{"code":reason}]},"new_provider_calls":0})))?;
    write_report(output, &report)
}

fn write_report(output: &Path, report: &Value) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(report)
        .map_err(|e| format!("serialización del informe V3: {e}"))?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| format!("salida V3 debe ser nueva: {e}"))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("escritura V3: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_adapter_never_overwrites_existing_report() {
        let dir = std::env::temp_dir().join(format!(
            "gennie-memory-v3-existing-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let input = dir.join("input.json");
        let output = dir.join("output.json");
        std::fs::write(&input, br#"{"vocabulary":{},"text":"x","raw":{}}"#).unwrap();
        std::fs::write(&output, b"preserved report\n").unwrap();
        assert!(run(&input, &output, false).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), b"preserved report\n");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn file_adapter_rejects_extra_fields_without_creating_output() {
        let dir = std::env::temp_dir().join(format!(
            "gennie-memory-v3-input-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let input = dir.join("input.json");
        let output = dir.join("output.json");
        std::fs::write(
            &input,
            br#"{"vocabulary":{},"text":"x","raw":{},"gold":true}"#,
        )
        .unwrap();
        assert!(run(&input, &output, false).is_err());
        assert!(!output.exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
