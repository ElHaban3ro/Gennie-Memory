//! File transport for the local, non-answering qualified evidence selector.
use crate::documentary_v3::lexical_clause_context::select;
use serde_json::json;
use std::io::Read;
use std::path::Path;

const LIMIT: usize = 16 * 1024 * 1024;
fn bounded(path: &Path, allowance: usize) -> Result<Option<Vec<u8>>, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("lectura de selección léxica: {e}"))?;
    let mut bytes = Vec::new();
    file.take(allowance as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("lectura de selección léxica: {e}"))?;
    Ok((bytes.len() <= allowance).then_some(bytes))
}
pub fn run(input: &Path, trust: &Path, output: &Path) -> Result<(), String> {
    let limit = || json!({"kind":"error","category":"operation","code":"request_context_limit"});
    let Some(input_bytes) = bounded(input, LIMIT)? else {
        return super::write_report(output, &limit());
    };
    let Some(trust_bytes) = bounded(trust, LIMIT - input_bytes.len())? else {
        return super::write_report(output, &limit());
    };
    // Duplicate keys are checked before conversion into a potentially lossy Value.
    let result = match super::context_json::parse_strict(&input_bytes) {
        Err(_) => json!({"kind":"error","category":"structural","code":"invalid_document"}),
        Ok(input) => match super::context_json::parse_strict(&trust_bytes) {
            Err(_) => json!({"kind":"error","category":"structural","code":"invalid_provenance"}),
            Ok(trust) => select(&input, &trust),
        },
    };
    super::write_report(output, &result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_is_bounded_before_json_and_existing_reports_are_preserved() {
        let dir = std::env::temp_dir().join(format!(
            "lexical-transport-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let input = dir.join("input.json");
        let trust = dir.join("trust.json");
        let output = dir.join("result.json");
        let file = std::fs::File::create(&input).unwrap();
        file.set_len((LIMIT + 1) as u64).unwrap();
        std::fs::write(&trust, b"{}").unwrap();
        run(&input, &trust, &output).unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
        assert_eq!(report["code"], "request_context_limit");
        assert_eq!(report["category"], "operation");
        let original = std::fs::read(&output).unwrap();
        assert!(run(&input, &trust, &output).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), original);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
