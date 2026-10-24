//! Coordinator-owned serialization; original semantic decisions remain unchanged.
use super::packet::{LIMIT, checked_packet, digest, read_limited};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Value, json};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

// Documentary decimals are strings. Restrict JSON numbers to exact integers;
// serde_json::Value alone silently collapses duplicate keys and can round floats.
struct StrictValue(Value);
impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("JSON sin claves duplicadas y con números enteros exactos")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(json!(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(json!(v)))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(json!(v)))
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
                Err(E::custom(
                    "literal numérico no entero o fuera de rango; cantidades decimales requieren strings",
                ))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(json!(v)))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(StrictValue(json!(v)))
            }
            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(v) = a.next_element::<StrictValue>()? {
                    values.push(v.0);
                }
                Ok(StrictValue(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = a.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom("clave JSON duplicada"));
                    }
                    values.insert(key, a.next_value::<StrictValue>()?.0);
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        d.deserialize_any(StrictVisitor)
    }
}
fn strict_json(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() > LIMIT {
        return Err("entrada de ensamblado V3 supera 16 MiB".into());
    }
    std::str::from_utf8(bytes).map_err(|_| "entrada de ensamblado V3 no es UTF8")?;
    serde_json::from_slice::<StrictValue>(bytes)
        .map(|v| v.0)
        .map_err(|e| format!("JSON estricto V3: {e}"))
}
fn inventory(packet: &Value, expected_hash: &str, items: &Value) -> Result<(), String> {
    super::closed(items, &["version", "input_packet_sha256", "items"])?;
    if items["version"] != "documentary_v3_semantic_items_v1"
        || items["input_packet_sha256"] != expected_hash
    {
        return Err("items V3 no corresponden al paquete declarado".into());
    }
    let rows = items["items"]
        .as_array()
        .ok_or("faltan filas semánticas V3")?;
    let expected = packet["items"].as_array().unwrap();
    if rows.len() != expected.len()
        || rows
            .iter()
            .map(|r| &r["id"])
            .ne(expected.iter().map(|r| &r["id"]))
    {
        return Err("inventario u orden semántico V3 alterado".into());
    }
    for row in rows {
        super::closed(row, &["id", "raw"])?;
    }
    Ok(())
}
fn assemble(packet_bytes: &[u8], expected_hash: &str, item_bytes: &[u8]) -> Result<Value, String> {
    strict_json(packet_bytes)?;
    let packet = checked_packet(packet_bytes, expected_hash)?;
    let items = strict_json(item_bytes)?;
    inventory(&packet, expected_hash, &items)?;
    Ok(
        json!({"version":"documentary_v3_assembled_delivery_v1","packet_sha256":expected_hash,"items":items["items"],"assembly":{"kind":"coordinator_packet_binding","semantic_items_sha256":digest(item_bytes),"semantic_items_utf8":std::str::from_utf8(item_bytes).map_err(|e|e.to_string())?}}),
    )
}
/// Recompute original-byte hash and compare every id/raw value, not a producer flag.
pub(super) fn verify(
    packet_bytes: &[u8],
    expected_hash: &str,
    delivery_bytes: &[u8],
) -> Result<Value, String> {
    strict_json(packet_bytes)?;
    let packet = checked_packet(packet_bytes, expected_hash)?;
    let d = strict_json(delivery_bytes)?;
    super::closed(&d, &["version", "packet_sha256", "items", "assembly"])?;
    super::closed(
        &d["assembly"],
        &["kind", "semantic_items_sha256", "semantic_items_utf8"],
    )?;
    if d["version"] != "documentary_v3_assembled_delivery_v1"
        || d["packet_sha256"] != expected_hash
        || d["assembly"]["kind"] != "coordinator_packet_binding"
    {
        return Err("procedencia de ensamblado V3 incompatible".into());
    }
    let original = d["assembly"]["semantic_items_utf8"]
        .as_str()
        .ok_or("faltan bytes originales semánticos V3")?;
    if d["assembly"]["semantic_items_sha256"] != digest(original.as_bytes()) {
        return Err("hash original semántico V3 alterado".into());
    }
    let items = strict_json(original.as_bytes())?;
    inventory(&packet, expected_hash, &items)?;
    if d["items"] != items["items"] {
        return Err("el derivado modifica decisiones semánticas V3".into());
    }
    Ok(
        json!({"kind":"coordinator_packet_binding","semantic_items_sha256":d["assembly"]["semantic_items_sha256"],"original_bytes_retained":true,"original_values_match":true,"model_packet_echo_matches":true,"model_authorship_verified":false,"freshness_verified":false,"source_only_behavior_verified":false}),
    )
}
fn encoded(delivery: &Value) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(delivery).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    if bytes.len() > LIMIT {
        return Err("derivado de ensamblado V3 supera 16 MiB".into());
    }
    Ok(bytes)
}
fn publish_with(
    output: &Path,
    writer: impl FnOnce(&mut File) -> std::io::Result<()>,
) -> Result<(), String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let temp = parent.join(format!(
        ".gennie-memory-delivery-{}-{}-{stamp}.partial",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| format!("temporal V3: {e}"))?;
    let result = writer(&mut file)
        .and_then(|_| file.sync_all())
        .and_then(|_| std::fs::hard_link(&temp, output));
    drop(file);
    let cleanup = std::fs::remove_file(&temp);
    result.map_err(|e| format!("publicación V3 sin reemplazo: {e}"))?;
    cleanup.map_err(|e| format!("derivado V3 publicado, limpieza del temporal fallida: {e}"))?;
    Ok(())
}
pub fn run(packet: &Path, expected_hash: &str, items: &Path, output: &Path) -> Result<(), String> {
    let packet = read_limited(packet)?;
    let items = read_limited(items)?;
    let derived = assemble(&packet, expected_hash, &items)?;
    let bytes = encoded(&derived)?;
    verify(&packet, expected_hash, &bytes)?;
    publish_with(output, |f| f.write_all(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scenario() -> (Vec<u8>, Value) {
        let bank: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-closure-domain-controls-v1.json"
        ))
        .unwrap();
        let input = &bank["cases"][0]["input"];
        let p = json!({"version":"documentary_v3_strict_extraction_packet_v1","kind":"source","vocabulary":input["vocabulary"],"vocabulary_sha256":input["source"]["raw"]["vocabulary_sha256"],"domain_namespace":input["source"]["raw"]["domain_namespace"],"instructions":"generic test","notation":{},"semantic_selection_guide":{},"validator_contract_source_without_tests":"","items":[{"id":"a","text":input["source"]["text"],"text_sha256":input["source"]["raw"]["text_sha256"]},{"id":"b","text":input["source"]["text"],"text_sha256":input["source"]["raw"]["text_sha256"]}]});
        let packet = serde_json::to_vec(&p).unwrap();
        let items = json!({"version":"documentary_v3_semantic_items_v1","input_packet_sha256":digest(&packet),"items":[{"id":"a","raw":input["source"]["raw"]},{"id":"b","raw":input["source"]["raw"]}]});
        (packet, items)
    }
    #[test]
    fn preserves_original_utf8_and_values_without_claiming_authorship_or_freshness() {
        let (packet, items) = scenario();
        let mut original = serde_json::to_vec_pretty(&items).unwrap();
        original.push(b'\n');
        let d = assemble(&packet, &digest(&packet), &original).unwrap();
        assert_eq!(d["items"], items["items"]);
        assert_eq!(
            d["assembly"]["semantic_items_utf8"],
            std::str::from_utf8(&original).unwrap()
        );
        assert_eq!(d["assembly"]["semantic_items_sha256"], digest(&original));
        let verified = verify(&packet, &digest(&packet), &encoded(&d).unwrap()).unwrap();
        assert_eq!(verified["original_values_match"], true);
        for key in [
            "model_authorship_verified",
            "freshness_verified",
            "source_only_behavior_verified",
        ] {
            assert_eq!(verified[key], false);
        }
    }
    #[test]
    fn no_rebinding_reordering_filling_or_partial_salvage() {
        let (packet, items) = scenario();
        let hash = digest(&packet);
        for mutation in [
            "hash",
            "order",
            "duplicate",
            "missing",
            "extra",
            "extra_key",
            "extra_row_key",
            "missing_raw",
        ] {
            let mut v = items.clone();
            match mutation {
                "hash" => v["input_packet_sha256"] = json!("other"),
                "order" => v["items"].as_array_mut().unwrap().swap(0, 1),
                "duplicate" => v["items"][1]["id"] = json!("a"),
                "missing" => {
                    v["items"].as_array_mut().unwrap().pop();
                }
                "extra" => {
                    let extra = v["items"][0].clone();
                    v["items"].as_array_mut().unwrap().push(extra);
                }
                "extra_key" => v["gold"] = json!(true),
                "extra_row_key" => v["items"][0]["label"] = json!("useful"),
                "missing_raw" => {
                    v["items"][0].as_object_mut().unwrap().remove("raw");
                }
                _ => unreachable!(),
            }
            assert!(
                assemble(&packet, &hash, &serde_json::to_vec(&v).unwrap()).is_err(),
                "{mutation}"
            );
        }
        let mut other: Value = serde_json::from_slice(&packet).unwrap();
        other["instructions"] = json!("different instructions, same texts and IDs");
        let other = serde_json::to_vec(&other).unwrap();
        assert!(
            assemble(
                &other,
                &digest(&other),
                &serde_json::to_vec(&items).unwrap()
            )
            .is_err()
        );
        assert!(assemble(&other, &hash, &serde_json::to_vec(&items).unwrap()).is_err());
    }
    #[test]
    fn strict_parse_rejects_duplicates_and_lossy_numbers_at_any_depth() {
        for input in [
            r#"{"a":1,"a":2}"#,
            r#"{"nested":[{"kind":"ref","kind":"text"}]}"#,
            r#"{"a":1,"\u0061":2}"#,
            "1.0",
            "1e0",
            "0.123456789012345678901234",
            "18446744073709551616",
        ] {
            assert!(strict_json(input.as_bytes()).is_err(), "{input}");
        }
        assert_eq!(
            strict_json(b"9007199254740993").unwrap().as_u64(),
            Some(9007199254740993)
        );
        assert_eq!(
            strict_json(b"18446744073709551615").unwrap().as_u64(),
            Some(u64::MAX)
        );
        assert!(strict_json(&[255]).is_err());
    }
    #[test]
    fn receipt_verifies_original_against_derived_instead_of_trusting_flags() {
        let (packet, items) = scenario();
        let d = assemble(
            &packet,
            &digest(&packet),
            &serde_json::to_vec(&items).unwrap(),
        )
        .unwrap();
        for mutation in [
            "raw",
            "original_hash",
            "original_bytes",
            "extra_provenance",
            "version",
        ] {
            let mut v = d.clone();
            match mutation {
                "raw" => v["items"][0]["raw"]["invented"] = json!(true),
                "original_hash" => v["assembly"]["semantic_items_sha256"] = json!("wrong"),
                "original_bytes" => v["assembly"]["semantic_items_utf8"] = json!("{}"),
                "extra_provenance" => v["assembly"]["freshness_verified"] = json!(true),
                "version" => v["version"] = json!("documentary_v3_strict_extraction_delivery_v1"),
                _ => unreachable!(),
            }
            assert!(
                verify(&packet, &digest(&packet), &serde_json::to_vec(&v).unwrap()).is_err(),
                "{mutation}"
            );
        }
    }
    #[test]
    fn incorrect_raw_metadata_or_null_raw_is_preserved_not_repaired() {
        let (packet, mut items) = scenario();
        items["items"][0]["raw"]["kind"] = json!("question");
        items["items"][0]["raw"]["text_sha256"] = json!("wrong");
        items["items"][1]["raw"] = Value::Null;
        let d = assemble(
            &packet,
            &digest(&packet),
            &serde_json::to_vec(&items).unwrap(),
        )
        .unwrap();
        assert_eq!(d["items"], items["items"]);
        assert!(verify(&packet, &digest(&packet), &encoded(&d).unwrap()).is_ok());
    }
    #[test]
    fn atomic_publication_never_replaces_or_exposes_incomplete_output() {
        let dir = std::env::temp_dir().join(format!(
            "gennie-memory-assembly-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let output = dir.join("output.json");
        assert!(
            publish_with(&output, |f| {
                f.write_all(b"incomplete")?;
                Err(std::io::Error::other("injected writer failure"))
            })
            .is_err()
        );
        assert!(!output.exists());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        publish_with(&output, |f| f.write_all(b"complete")).unwrap();
        assert_eq!(std::fs::read(&output).unwrap(), b"complete");
        assert!(publish_with(&output, |f| f.write_all(b"replacement")).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), b"complete");
        #[cfg(unix)]
        {
            let link = dir.join("link.json");
            std::os::unix::fs::symlink(&output, &link).unwrap();
            assert!(publish_with(&link, |f| f.write_all(b"replacement")).is_err());
            assert_eq!(std::fs::read(&output).unwrap(), b"complete");
        }
        assert!(std::fs::read_dir(&dir).unwrap().all(|p| {
            !p.unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".partial")
        }));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn oversized_derived_output_is_not_transportable() {
        assert!(encoded(&json!({"value":"x".repeat(LIMIT)})).is_err());
    }
}
