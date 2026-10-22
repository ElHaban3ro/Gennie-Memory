//! Shared frozen packet validation and bounded file reads.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;
pub(super) const LIMIT: usize = 16 * 1024 * 1024;
pub(super) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub(super) fn read_limited(path: &Path) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("entrada de entrega V3: {e}"))?;
    let mut bytes = Vec::new();
    file.take((LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("lectura de entrega V3: {e}"))?;
    if bytes.len() > LIMIT {
        return Err("entrada de entrega V3 supera 16 MiB".into());
    }
    Ok(bytes)
}

pub(super) fn checked_packet(packet_bytes: &[u8], expected_hash: &str) -> Result<Value, String> {
    if packet_bytes.len() > LIMIT {
        return Err("paquete V3 supera 16 MiB".into());
    }
    if digest(packet_bytes) != expected_hash {
        return Err("el paquete V3 no coincide con su hash congelado".into());
    }
    let packet: Value =
        serde_json::from_slice(packet_bytes).map_err(|e| format!("paquete V3: {e}"))?;
    super::closed(
        &packet,
        &[
            "version",
            "kind",
            "vocabulary",
            "vocabulary_sha256",
            "domain_namespace",
            "instructions",
            "notation",
            "semantic_selection_guide",
            if packet.get("author_guide").is_some() {
                "author_guide"
            } else {
                "validator_contract_source_without_tests"
            },
            "items",
        ],
    )?;
    if packet["version"] != "documentary_v3_strict_extraction_packet_v1"
        || !matches!(packet["kind"].as_str(), Some("source" | "question"))
        || packet["vocabulary_sha256"]
            != digest(&serde_json::to_vec(&packet["vocabulary"]).map_err(|e| e.to_string())?)
    {
        return Err("contrato o vocabulario del paquete V3 incompatible".into());
    }
    let items = packet["items"]
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or("paquete V3 sin filas")?;
    let mut ids = BTreeSet::new();
    for item in items {
        super::closed(item, &["id", "text", "text_sha256"])?;
        let id = item["id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or("ID de paquete V3 ausente")?;
        let text = item["text"].as_str().ok_or("texto de paquete V3 ausente")?;
        if !ids.insert(id) || item["text_sha256"] != digest(text.as_bytes()) {
            return Err("IDs o textos del paquete V3 alterados".into());
        }
    }
    Ok(packet)
}
