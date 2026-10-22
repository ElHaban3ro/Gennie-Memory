//! Lossless, packet-bound receipt of frozen extraction deliveries; no semantic gate.
use super::packet::{LIMIT, checked_packet, digest, read_limited};
use crate::documentary_v3;
use serde_json::{Value, json};
use std::path::Path;

fn audit(
    packet_bytes: &[u8],
    expected_hash: &str,
    delivery_bytes: Option<&[u8]>,
) -> Result<Value, String> {
    audit_format(packet_bytes, expected_hash, delivery_bytes, false)
}
fn audit_format(
    packet_bytes: &[u8],
    expected_hash: &str,
    delivery_bytes: Option<&[u8]>,
    assembled_only: bool,
) -> Result<Value, String> {
    if packet_bytes.len() > LIMIT || delivery_bytes.is_some_and(|b| b.len() > LIMIT) {
        return Err("entrada de entrega V3 supera 16 MiB".into());
    }
    let packet = checked_packet(packet_bytes, expected_hash)?;
    let items = packet["items"].as_array().unwrap();
    let delivery: Option<Value> = delivery_bytes.and_then(|b| serde_json::from_slice(b).ok());
    let assembled = delivery
        .as_ref()
        .is_some_and(|d| d["version"] == "documentary_v3_assembled_delivery_v1");
    let assembly_check = (assembled || assembled_only).then(|| {
        delivery_bytes
            .ok_or_else(|| "falta entrega ensamblada V3".to_owned())
            .and_then(|bytes| super::assembly::verify(packet_bytes, expected_hash, bytes))
    });
    let mut batch_issues = Vec::new();
    match &delivery {
        None => batch_issues.push(if delivery_bytes.is_none() {
            "missing_delivery"
        } else {
            "malformed_delivery_json"
        }),
        Some(d) => {
            if super::closed(
                d,
                if assembled {
                    &["version", "packet_sha256", "items", "assembly"]
                } else {
                    &["version", "packet_sha256", "items"]
                },
            )
            .is_err()
                || (!assembled && d["version"] != "documentary_v3_strict_extraction_delivery_v1")
            {
                batch_issues.push("delivery_shape");
            }
            if d["packet_sha256"] != expected_hash {
                batch_issues.push("packet_binding");
            }
            if assembly_check.as_ref().is_some_and(|check| check.is_err()) {
                batch_issues.push("assembly_provenance");
            }
            match d["items"].as_array() {
                None => batch_issues.push("delivery_items"),
                Some(rows) => {
                    if rows.len() != items.len() {
                        batch_issues.push("item_count");
                    }
                    if rows
                        .iter()
                        .any(|r| super::closed(r, &["id", "raw"]).is_err())
                    {
                        batch_issues.push("item_shape");
                    }
                    if rows
                        .iter()
                        .map(|r| &r["id"])
                        .ne(items.iter().map(|r| &r["id"]))
                    {
                        batch_issues.push("item_order_or_identity");
                    }
                }
            }
        }
    }
    let bound = batch_issues.is_empty();
    let mut rows = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let mut binding_issues = Vec::new();
        let raw = bound.then(|| delivery.as_ref().unwrap()["items"][index]["raw"].clone());
        let validated = raw.as_ref().map(|raw| {
            if raw["kind"] != packet["kind"] {
                binding_issues.push("document_kind");
            }
            if raw["domain_namespace"] != packet["domain_namespace"] {
                binding_issues.push("domain_namespace");
            }
            if raw["version"] == "documentary_reading_set_v1" {
                super::readings::validate(
                    raw,
                    item["text"].as_str().unwrap(),
                    &packet["vocabulary"],
                )
            } else {
                serde_json::to_value(documentary_v3::validate(
                    raw,
                    item["text"].as_str().unwrap(),
                    &packet["vocabulary"],
                ))
                .expect("DocumentResult serializes JSON values")
            }
        });
        let status = if !bound {
            "unavailable"
        } else if !binding_issues.is_empty() {
            "invalid_binding"
        } else {
            validated.as_ref().unwrap()["status"]
                .as_str()
                .unwrap_or("invalid")
        };
        rows.push(json!({"id":item["id"],"text":item["text"],"text_sha256":item["text_sha256"],"status":status,"binding_issues":binding_issues,"raw":raw,"validated":validated}));
    }
    let complete = rows.iter().filter(|r| r["status"] == "complete").count();
    let mut report = json!({"version":"documentary-v3-delivery-audit-v1","packet_sha256":expected_hash,"delivery_sha256":delivery_bytes.map(digest),"delivery_utf8":delivery_bytes.and_then(|b|std::str::from_utf8(b).ok()),"delivery_non_utf8_bytes":delivery_bytes.filter(|b|std::str::from_utf8(b).is_err()),"batch_integrity_passed":bound,"batch_issues":batch_issues,"expected_items":items.len(),"structurally_complete":complete,"rows":rows,"semantic_fidelity_proven":false,"automatic_utility_evaluated":false,"quality_gate_passed":false,"new_provider_calls":0});
    if let Some(check) = assembly_check {
        report["assembly_provenance"]=check.unwrap_or_else(|_|json!({"kind":"coordinator_packet_binding","original_values_match":false,"model_authorship_verified":false,"freshness_verified":false,"source_only_behavior_verified":false}));
    }
    if assembled_only {
        report["expected_delivery_format"] = json!("assembled_only");
    }
    Ok(report)
}

/// A missing delivery path is an unavailable batch, never an empty passing sample.
pub fn run(
    packet: &Path,
    expected_hash: &str,
    delivery: &Path,
    output: &Path,
    assembled_only: bool,
) -> Result<(), String> {
    let packet = read_limited(packet)?;
    let delivery = match std::fs::metadata(delivery) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(format!("acceso a entrega V3: {e}")),
        Ok(_) => Some(read_limited(delivery)?),
    };
    let report = if assembled_only {
        audit_format(&packet, expected_hash, delivery.as_deref(), true)?
    } else {
        audit(&packet, expected_hash, delivery.as_deref())?
    };
    super::write_report(output, &report)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scenario() -> (Vec<u8>, Value) {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-v3-closure-domain-controls-v1.json"
        ))
        .unwrap();
        let input = &fixture["cases"][0]["input"];
        let packet = json!({"version":"documentary_v3_strict_extraction_packet_v1","kind":"source","vocabulary":input["vocabulary"],"vocabulary_sha256":input["source"]["raw"]["vocabulary_sha256"],"domain_namespace":input["source"]["raw"]["domain_namespace"],"instructions":"test only","notation":{},"semantic_selection_guide":{},"validator_contract_source_without_tests":"","items":[{"id":"a","text":input["source"]["text"],"text_sha256":input["source"]["raw"]["text_sha256"]},{"id":"b","text":input["source"]["text"],"text_sha256":input["source"]["raw"]["text_sha256"]}]});
        let packet = serde_json::to_vec(&packet).unwrap();
        let d = json!({"version":"documentary_v3_strict_extraction_delivery_v1","packet_sha256":digest(&packet),"items":[{"id":"a","raw":input["source"]["raw"]},{"id":"b","raw":input["source"]["raw"]}]});
        (packet, d)
    }
    fn check(packet: &[u8], d: &Value) -> Value {
        audit(
            packet,
            &digest(packet),
            Some(&serde_json::to_vec(d).unwrap()),
        )
        .unwrap()
    }
    #[test]
    fn author_guide_packet_preserves_closed_shape_and_frozen_binding() {
        let (bytes, mut delivery) = scenario();
        let mut packet: Value = serde_json::from_slice(&bytes).unwrap();
        packet
            .as_object_mut()
            .unwrap()
            .remove("validator_contract_source_without_tests");
        packet["author_guide"] = json!({"instructions":"own text only"});
        let bytes = serde_json::to_vec(&packet).unwrap();
        let frozen_hash = digest(&bytes);
        delivery["packet_sha256"] = json!(frozen_hash);
        let report = check(&bytes, &delivery);
        assert_eq!(report["structurally_complete"], 2);
        assert_eq!(report["rows"][0]["raw"], delivery["items"][0]["raw"]);
        assert_eq!(report["semantic_fidelity_proven"], false);
        for key in ["gold", "validator_contract_source_without_tests"] {
            let mut altered = packet.clone();
            altered[key] = json!("unexpected");
            let altered = serde_json::to_vec(&altered).unwrap();
            assert!(checked_packet(&altered, &digest(&altered)).is_err());
        }
        packet["author_guide"]["instructions"] = json!("changed");
        assert!(checked_packet(&serde_json::to_vec(&packet).unwrap(), &frozen_hash).is_err());
    }
    #[test]
    fn bound_complete_delivery_preserves_raw_and_does_not_claim_fidelity() {
        let (packet, d) = scenario();
        let bytes = serde_json::to_vec_pretty(&d).unwrap();
        let r = audit(&packet, &digest(&packet), Some(&bytes)).unwrap();
        assert_eq!(r["batch_integrity_passed"], true);
        assert_eq!(r["structurally_complete"], 2);
        assert_eq!(r["delivery_utf8"], std::str::from_utf8(&bytes).unwrap());
        assert_eq!(r["rows"][0]["raw"], d["items"][0]["raw"]);
        assert_eq!(r["semantic_fidelity_proven"], false);
        assert_eq!(r["quality_gate_passed"], false);
    }
    #[test]
    fn reading_envelope_is_retained_and_cannot_hide_invalid_or_open_leaves() {
        let (packet, mut delivery) = scenario();
        let raw = delivery["items"][0]["raw"].clone();
        let envelope = json!({
            "version":"documentary_reading_set_v1",
            "kind":raw["kind"],"text_sha256":raw["text_sha256"],
            "vocabulary_sha256":raw["vocabulary_sha256"],
            "domain_namespace":raw["domain_namespace"],"closure":"closed",
            "readings":[{"id":"only","document":raw}],
            "ambiguities":[],"unresolved_dimensions":[]
        });
        delivery["items"][0]["raw"] = envelope.clone();
        let report = check(&packet, &delivery);
        assert_eq!(report["rows"][0]["status"], "complete", "{report}");
        assert_eq!(report["rows"][0]["validated"]["document"], envelope);
        assert_eq!(report["rows"][1]["status"], "complete");
        assert_eq!(report["semantic_fidelity_proven"], false);
        delivery["items"][0]["raw"]["closure"] = json!("open");
        let open = check(&packet, &delivery);
        assert_eq!(open["rows"][0]["status"], "partial");
        assert_eq!(open["rows"][0]["raw"], delivery["items"][0]["raw"]);
        delivery["items"][0]["raw"] = envelope;
        delivery["items"][0]["raw"]["readings"][0]["document"]["kind"] = json!("question");
        let invalid = check(&packet, &delivery);
        assert_eq!(invalid["rows"][0]["status"], "invalid");
        assert_eq!(invalid["rows"][1]["status"], "complete");
        assert_eq!(invalid["quality_gate_passed"], false);
    }
    #[test]
    fn missing_malformed_and_binary_delivery_keep_every_expected_row_unavailable() {
        let (packet, _) = scenario();
        for bytes in [
            None,
            Some(b"not json".as_slice()),
            Some([255u8, 0].as_slice()),
        ] {
            let r = audit(&packet, &digest(&packet), bytes).unwrap();
            assert_eq!(r["batch_integrity_passed"], false);
            assert_eq!(r["structurally_complete"], 0);
            assert_eq!(r["rows"].as_array().unwrap().len(), 2);
            assert!(
                r["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|r| r["status"] == "unavailable" && r["raw"].is_null())
            );
            if let Some(bytes) = bytes.filter(|b| std::str::from_utf8(b).is_err()) {
                assert_eq!(r["delivery_non_utf8_bytes"], json!(bytes));
            }
        }
    }
    #[test]
    fn wrapper_mismatch_degrades_whole_batch_without_salvaging_rows() {
        let (packet, d) = scenario();
        for mutation in [
            "hash",
            "missing",
            "duplicate",
            "order",
            "extra",
            "extra_item_field",
            "version",
        ] {
            let mut d = d.clone();
            match mutation {
                "hash" => d["packet_sha256"] = json!("forged"),
                "missing" => {
                    d["items"].as_array_mut().unwrap().pop();
                }
                "duplicate" => d["items"][1]["id"] = json!("a"),
                "order" => d["items"].as_array_mut().unwrap().swap(0, 1),
                "extra" => d["gold"] = json!("support"),
                "extra_item_field" => d["items"][0]["label"] = json!("support"),
                "version" => d["version"] = json!("older"),
                _ => unreachable!(),
            }
            let r = check(&packet, &d);
            assert_eq!(r["batch_integrity_passed"], false, "{mutation} {r}");
            assert_eq!(r["structurally_complete"], 0, "{mutation} {r}");
            assert!(
                r["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|r| r["raw"].is_null())
            );
        }
    }
    #[test]
    fn invalid_raw_is_retained_without_discarding_an_independent_valid_row() {
        let (packet, mut d) = scenario();
        d["items"][0]["raw"]["invented"] = json!(true);
        let r = check(&packet, &d);
        assert_eq!(r["batch_integrity_passed"], true);
        assert_eq!(r["structurally_complete"], 1);
        assert_eq!(r["rows"][0]["status"], "invalid");
        assert_eq!(r["rows"][0]["raw"], d["items"][0]["raw"]);
        assert_eq!(r["rows"][1]["status"], "complete");
    }
    #[test]
    fn wrong_kind_or_namespace_is_not_a_valid_campaign_entry() {
        let (packet, d) = scenario();
        for (key, value) in [("kind", "question"), ("domain_namespace", "other")] {
            let mut d = d.clone();
            d["items"][0]["raw"][key] = json!(value);
            let r = check(&packet, &d);
            assert_eq!(r["rows"][0]["status"], "invalid_binding");
            assert_eq!(r["structurally_complete"], 1);
            assert_eq!(r["rows"][0]["raw"], d["items"][0]["raw"]);
        }
    }
    #[test]
    fn changed_packet_cannot_bind_to_original_frozen_hash() {
        let (mut packet, d) = scenario();
        let hash = digest(&packet);
        packet.push(b' ');
        assert!(audit(&packet, &hash, Some(&serde_json::to_vec(&d).unwrap())).is_err());
    }
    #[test]
    fn required_assembly_rejects_legacy_downgrade_without_changing_legacy_mode() {
        let (packet, legacy) = scenario();
        let bytes = serde_json::to_vec(&legacy).unwrap();
        let duplicate = format!(
            "{{\"version\":\"documentary_v3_assembled_delivery_v1\",{}",
            std::str::from_utf8(&bytes)
                .unwrap()
                .strip_prefix('{')
                .unwrap()
        );
        for raw in [Some(bytes.as_slice()), Some(duplicate.as_bytes()), None] {
            let r = audit_format(&packet, &digest(&packet), raw, true).unwrap();
            assert_eq!(r["batch_integrity_passed"], false);
            assert_eq!(r["structurally_complete"], 0);
            assert_eq!(r["expected_items"], 2);
            assert_eq!(r["expected_delivery_format"], "assembled_only");
            assert!(rows_unavailable(&r));
        }
        assert_eq!(
            audit(&packet, &digest(&packet), Some(duplicate.as_bytes())).unwrap()["structurally_complete"],
            2
        );
        fn rows_unavailable(r: &Value) -> bool {
            r["rows"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| row["status"] == "unavailable" && row["raw"].is_null())
        }
    }
}
