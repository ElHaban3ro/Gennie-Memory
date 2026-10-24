//! Filesystem adapter for the isolated semantic-check capacity diagnostic; no product policy.
use crate::typed_facts::v2::record_projection::automatic_extraction::reference_matrix;
use crate::{
    distributed_reference::{self, Registry},
    semantic_check_contract as contract,
};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
pub const ORIGINAL: &str = "tests/fixtures/distributed-reference-controls-v1.json";
pub const EXTRA: &str = "tests/fixtures/semantic-representation-controls-v1.json";
pub const REGISTRY: &str = "tests/fixtures/reference-classes-v1.json";
pub const SOURCES: [&str; 14] = [
    "tools/evaluation/src/semantic_presentation.rs",
    "tools/evaluation/src/typed_facts.rs",
    "tools/evaluation/src/typed_facts/v2.rs",
    "tools/evaluation/src/typed_facts/v2/record_projection.rs",
    "bootstrap/src/openrouter/per_memory/independent_projection_probe.rs",
    "bootstrap/src/openrouter/per_memory.rs",
    "tools/evaluation/src/main.rs",
    "tools/evaluation/src/semantic_check_contract.rs",
    "tools/evaluation/src/semantic_representation.rs",
    "tools/evaluation/src/distributed_reference.rs",
    "tools/evaluation/src/reference_catalog.rs",
    "tools/evaluation/src/documentary_extraction_contract/grounded_references.rs",
    "tools/evaluation/src/typed_facts/v2/record_projection/automatic_extraction.rs",
    "tools/evaluation/src/typed_facts/v2/record_projection/automatic_extraction/reference_matrix.rs",
];
fn read(p: impl AsRef<Path>) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(p).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn save(p: &Path, v: &Value) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(p)
        .map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec_pretty(v).unwrap())
        .map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())
}
pub fn rows() -> Result<Vec<Value>, String> {
    rows_at(Path::new("."))
}
fn rows_at(root: &Path) -> Result<Vec<Value>, String> {
    let f = read(root.join(ORIGINAL))?;
    let extra = read(root.join(EXTRA))?;
    let bytes = fs::read(root.join(REGISTRY)).map_err(|e| e.to_string())?;
    let wire: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let reg = Registry::parse(
        &bytes,
        f["registry_sha256"].as_str().ok_or("falta registro")?,
    )?;
    if f["version"] != "distributed-reference-controls-v1"
        || extra["version"] != "semantic-representation-controls-v1"
        || extra["registry_sha256"] != f["registry_sha256"]
    {
        return Err("inputs semánticos sin versión/registro".into());
    }
    let mut rows = Vec::new();
    let mut origins = std::collections::BTreeSet::new();
    let mut push = |origin: String, kind: &str, side: &Value, o: &Value| -> Result<(), String> {
        if !origins.insert(origin.clone()) {
            return Err("origen semántico duplicado".into());
        }
        let input = reference_matrix::semantic_view(kind, side, o, &reg, &wire)?;
        let payload = contract::payload(&input)?;
        rows.push(json!({"origin":origin,"input":input,"payload":payload}));
        Ok(())
    };
    for batch in f["batches"].as_array().ok_or("faltan lotes")? {
        let cases = batch["input"]["cases"]
            .as_array()
            .ok_or("faltan controles")?;
        if cases.len() != 8 {
            return Err("faltan ocho controles por lote".into());
        }
        for c in cases {
            for kind in ["source", "question"] {
                push(
                    format!(
                        "{}/{}/{kind}",
                        batch["id"].as_str().ok_or("falta lote")?,
                        c["id"].as_str().ok_or("falta control")?
                    ),
                    kind,
                    &c[kind],
                    &c["ontology"],
                )?;
            }
        }
    }
    if f["batches"].as_array().unwrap().len() != 2 {
        return Err("faltan ambos lotes".into());
    }
    let counters = f["semantic_counterexamples"]
        .as_array()
        .ok_or("faltan contraejemplos")?;
    if counters.len() != 3 {
        return Err("faltan contraejemplos".into());
    }
    for c in counters {
        for kind in ["source", "question"] {
            push(
                format!(
                    "counter/{}/{kind}",
                    c["id"].as_str().ok_or("falta control")?
                ),
                kind,
                &c[kind],
                &c["ontology"],
            )?;
        }
    }
    let controls = extra["controls"].as_array().ok_or("faltan extras")?;
    if controls.len() != 8 {
        return Err("faltan ocho controles nuevos".into());
    }
    for c in controls {
        push(
            format!("extra/{}", c["id"].as_str().ok_or("falta control")?),
            c["kind"].as_str().ok_or("falta kind")?,
            &c["side"],
            &c["ontology"],
        )?;
    }
    if rows.len() != 46 {
        return Err("roster incompleto".into());
    }
    rows.sort_by_key(|r| distributed_reference::sha256(r.to_string().as_bytes()));
    for (i, row) in rows.iter_mut().enumerate() {
        row["id"] = json!(format!("s{i:03}"));
    }
    Ok(rows)
}
pub fn prepare(d: &Path) -> Result<(), String> {
    prepare_with(d, 1)
}
pub fn prepare_presentation(d: &Path) -> Result<(), String> {
    prepare_with(d, 2)
}
pub fn prepare_coordinates(d: &Path) -> Result<(), String> {
    prepare_with(d, 3)
}
pub fn prepare_constrained(d: &Path) -> Result<(), String> {
    prepare_with(d, 4)
}
pub fn prepare_capacity(d: &Path) -> Result<(), String> {
    prepare_with(d, 5)
}
pub fn prepare_basis_scope(d: &Path) -> Result<(), String> {
    prepare_with(d, 6)
}
fn prepare_with(d: &Path, version: u8) -> Result<(), String> {
    let mut rows = rows()?;
    if version >= 2 {
        let parent =
            Path::new("outputs/iromem/evaluation/semantic-representation-check-v1-2026-10-21");
        let previous = fs::read_to_string(parent.join("requests.jsonl"))
            .map_err(|e| e.to_string())?
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        if previous.len() != 46 {
            return Err("padre incompleto".into());
        }
        for r in &mut rows {
            let old = previous
                .iter()
                .find(|v| v["id"] == r["id"])
                .ok_or("ID distinto del padre")?;
            if old["input"] != r["input"] || old["payload"] != r["payload"] {
                return Err("entrada o ruta V1 modificadas".into());
            }
            r["payload"] = if version == 6 {
                crate::semantic_presentation::basis_scope_payload(&r["input"], &r["payload"])?
            } else if version == 5 {
                crate::semantic_presentation::capacity_payload(&r["input"], &r["payload"])?
            } else if version == 4 {
                crate::semantic_presentation::constrained_payload(&r["input"], &r["payload"])?
            } else if version == 3 {
                crate::semantic_presentation::coordinate_payload(&r["input"], &r["payload"])?
            } else {
                crate::semantic_presentation::payload(&r["input"], &r["payload"])?
            };
        }
    }
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    let requests: Vec<Value> = rows
        .iter()
        .map(|r| json!({"id":r["id"],"input":r["input"],"payload":r["payload"]}))
        .collect();
    let serialized = requests
        .iter()
        .map(|r| r.to_string() + "\n")
        .collect::<String>();
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(d.join("requests.jsonl"))
        .map_err(|e| e.to_string())?;
    f.write_all(serialized.as_bytes())
        .map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())?;
    save(
        &d.join("custodian-map.json"),
        &json!(
            rows.iter()
                .map(|r| json!({"id":r["id"],"origin":r["origin"],"input_sha256":distributed_reference::sha256(r["input"].to_string().as_bytes())}))
                .collect::<Vec<_>>()
        ),
    )?;
    save(
        &d.join("blind-packet.json"),
        &json!({"contract":contract::CONTRACT,"rubric":contract::INSTRUCTIONS,"items":requests.iter().map(|r|json!({"id":r["id"],"input":r["input"]})).collect::<Vec<_>>()}),
    )?;
    let mut hashes = serde_json::Map::new();
    for path in SOURCES.into_iter().chain([
        ORIGINAL,
        EXTRA,
        REGISTRY,
        "docs/evaluation/semantic-representation-check-v1-protocol-2026-10-21.md",
        "bootstrap/src/openrouter/per_memory/semantic_check_probe.rs",
    ]) {
        hashes.insert(
            path.into(),
            json!(distributed_reference::sha256(
                &fs::read(path).map_err(|e| e.to_string())?
            )),
        );
    }
    if version >= 2 {
        hashes.insert(
            if version == 6 {
                "docs/evaluation/semantic-basis-scope-v1-protocol-2026-10-21.md"
            } else if version == 5 {
                "docs/evaluation/semantic-capacity-v5-protocol-2026-10-21.md"
            } else if version == 4 {
                "docs/evaluation/semantic-schema-v4-protocol-2026-10-21.md"
            } else if version == 3 {
                "docs/evaluation/semantic-coordinates-v3-protocol-2026-10-21.md"
            } else {
                "docs/evaluation/semantic-presentation-v2-protocol-2026-10-21.md"
            }
            .into(),
            json!(distributed_reference::sha256(
                &fs::read(if version == 6 {
                    "docs/evaluation/semantic-basis-scope-v1-protocol-2026-10-21.md"
                } else if version == 5 {
                    "docs/evaluation/semantic-capacity-v5-protocol-2026-10-21.md"
                } else if version == 4 {
                    "docs/evaluation/semantic-schema-v4-protocol-2026-10-21.md"
                } else if version == 3 {
                    "docs/evaluation/semantic-coordinates-v3-protocol-2026-10-21.md"
                } else {
                    "docs/evaluation/semantic-presentation-v2-protocol-2026-10-21.md"
                })
                .map_err(|e| e.to_string())?
            )),
        );
        let parent =
            Path::new("outputs/iromem/evaluation/semantic-representation-check-v1-2026-10-21");
        for name in ["blind-packet.json", "custodian-map.json"] {
            if fs::read(d.join(name)).map_err(|e| e.to_string())?
                != fs::read(parent.join(name)).map_err(|e| e.to_string())?
            {
                return Err("paquete canónico o mapa alterado".into());
            }
        }
        let mut bindings = serde_json::Map::new();
        for name in [
            "blind-packet.json",
            "custodian-map.json",
            "requests.jsonl",
            "frozen-labels.json",
            "blind-labels-a.json",
            "blind-labels-b.json",
            "label-reconciliation.json",
        ] {
            let bytes = fs::read(parent.join(name)).map_err(|e| e.to_string())?;
            bindings.insert(name.into(), json!(distributed_reference::sha256(&bytes)));
            if !["blind-packet.json", "custodian-map.json", "requests.jsonl"].contains(&name) {
                let mut copied = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(d.join(name))
                    .map_err(|e| e.to_string())?;
                copied.write_all(&bytes).map_err(|e| e.to_string())?;
                copied.sync_all().map_err(|e| e.to_string())?;
                if fs::read(d.join(name)).unwrap() != bytes {
                    return Err("anotaciones copiadas no idénticas".into());
                }
            }
        }
        save(
            &d.join("parent-bindings.json"),
            &json!({"version":if version == 6 { "semantic-basis-scope-v1-parent" } else if version == 5 { "semantic-capacity-v5-parent" } else if version == 4 { "semantic-schema-v4-parent" } else if version == 3 { "semantic-coordinates-v3-parent" } else { "semantic-presentation-v2-parent" },"parent_directory":parent,"files_sha256":bindings,"known_development_not_fresh_validation":true}),
        )?;
    }
    let mut preparation = json!({"version":if version >= 2{if version == 6 { "semantic-basis-scope-v1-preparation" } else if version == 5 { "semantic-capacity-v5-preparation" } else if version == 4 { "semantic-schema-v4-preparation" } else if version == 3 { "semantic-coordinates-v3-preparation" } else { "semantic-presentation-v2-preparation" }}else{"semantic-representation-check-v1-preparation"},"model":if version>=5 { crate::semantic_presentation::CAPACITY_MODEL } else { contract::MODEL },"provider":if version>=5 { crate::semantic_presentation::CAPACITY_PROVIDER } else { "OpenAI" },"max_http_requests":46,"max_concurrent":4,"max_output_tokens":if version>=5 { crate::semantic_presentation::CAPACITY_MAX_TOKENS } else { contract::MAX_TOKENS },"requests_sha256":distributed_reference::sha256(serialized.as_bytes()),"eval_binary_sha256":distributed_reference::sha256(&fs::read(std::env::current_exe().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?),"custodian_map_sha256":distributed_reference::sha256(&fs::read(d.join("custodian-map.json")).unwrap()),"source_hashes":hashes,"blind_packet_sha256":distributed_reference::sha256(&fs::read(d.join("blind-packet.json")).unwrap()),"manual_constructions_not_extraction":true,"fresh_validation":false,"promoted":false,"all_38_historical_sides_and_8_new_controls":true});
    if version >= 2 {
        preparation["parent_bindings_sha256"] = json!(distributed_reference::sha256(
            &fs::read(d.join("parent-bindings.json")).map_err(|e| e.to_string())?
        ));
    }
    save(&d.join("preparation.json"), &preparation)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn agent_adapter_rejects_missing_duplicate_foreign_and_exposed_outputs() {
        let input = json!({"kind":"source","text":"verde","raw":{},"normalized":{},"normalization_error":null,"ontology":{},"registry":{}});
        let packet = json!({"items":[{"id":"one","input":input}]});
        let mut judgments = json!({});
        for dim in contract::DIMENSIONS {
            judgments[dim] = json!({"label":"faithful","reason":"test","findings":[]});
        }
        let good = json!({"metadata":{"reference_labels_seen":false,"prior_predictions_seen":false,"external_calls":0},"items":[{"id":"one","judgments":judgments}]});
        assert_eq!(checked_agent_items(&packet, &good).unwrap().len(), 1);
        for variant in [
            json!({"items":[]}),
            json!({"items":[good["items"][0].clone(),good["items"][0].clone()]}),
            json!({"items":[{"id":"other","judgments":judgments}]}),
        ] {
            let mut bad = good.clone();
            bad["items"] = variant["items"].clone();
            assert!(checked_agent_items(&packet, &bad).is_err());
        }
        let mut exposed = good.clone();
        exposed["metadata"]["reference_labels_seen"] = json!(true);
        assert!(checked_agent_items(&packet, &exposed).is_err());
        let mut forged = good.clone();
        forged["items"][0]["judgments"]["fidelity"] = json!({"label":"defective","reason":"test","findings":[{"quote":"invented","raw_paths":[],"normalized_paths":[],"explanation":"bad"}]});
        assert!(checked_agent_items(&packet, &forged).is_err());
    }
    #[test]
    fn basis_scope_profile_changes_only_system_audit_for_all46_inputs() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for row in rows_at(&root).unwrap() {
            let base =
                crate::semantic_presentation::capacity_payload(&row["input"], &row["payload"])
                    .unwrap();
            let mut focused =
                crate::semantic_presentation::basis_scope_payload(&row["input"], &row["payload"])
                    .unwrap();
            let original = base["messages"][0]["content"].as_str().unwrap();
            assert_eq!(
                focused["messages"][0]["content"],
                json!(format!(
                    "{original}\n{}",
                    crate::semantic_presentation::BASIS_SCOPE_AUDIT
                ))
            );
            focused["messages"][0]["content"] = base["messages"][0]["content"].clone();
            assert_eq!(focused, base);
        }
    }
    #[test]
    fn capacity_profile_preserves_all_messages_schema_and_known_inputs() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for row in rows_at(&root).unwrap() {
            let v4 =
                crate::semantic_presentation::constrained_payload(&row["input"], &row["payload"])
                    .unwrap();
            let mut v5 =
                crate::semantic_presentation::capacity_payload(&row["input"], &row["payload"])
                    .unwrap();
            assert_eq!(v5["model"], "anthropic/claude-opus-5.5");
            assert_eq!(v5["provider"]["only"], json!(["anthropic"]));
            assert_eq!(v5["provider"]["allow_fallbacks"], false);
            assert!(v5.get("temperature").is_none());
            assert_eq!(v5["max_tokens"], 8192);
            for key in ["model", "provider", "temperature", "max_tokens"] {
                v5[key] = v4[key].clone();
            }
            assert_eq!(v5, v4);
        }
    }
    #[test]
    fn all46_schema_variants_change_only_response_format_and_allow_exact_displayed_paths() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for row in rows_at(&root).unwrap() {
            let v3 =
                crate::semantic_presentation::coordinate_payload(&row["input"], &row["payload"])
                    .unwrap();
            let mut v4 =
                crate::semantic_presentation::constrained_payload(&row["input"], &row["payload"])
                    .unwrap();
            let body: Value =
                serde_json::from_str(v3["messages"][1]["content"].as_str().unwrap()).unwrap();
            assert_eq!(v4["response_format"]["type"], "json_schema");
            assert_eq!(v4["response_format"]["json_schema"]["strict"], true);
            let schema = &v4["response_format"]["json_schema"]["schema"];
            assert_eq!(schema["additionalProperties"], false);
            assert_eq!(schema["required"], json!(contract::DIMENSIONS));
            for (side, field) in [("raw", "raw_paths"), ("normalized", "normalized_paths")] {
                let expected = body["finding_coordinates"][side]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v["displayed_path"].clone())
                    .collect::<Vec<_>>();
                let actual = &schema["$defs"]["finding"]["properties"][field]["items"]["enum"];
                assert_eq!(actual, &json!(expected));
                assert!(
                    !actual
                        .as_array()
                        .unwrap()
                        .contains(&json!("/evidence_fields/0/path"))
                );
            }
            for def in ["finding", "judgment"] {
                let o = &schema["$defs"][def];
                assert_eq!(o["additionalProperties"], false);
                assert_eq!(
                    o["properties"].as_object().unwrap().len(),
                    o["required"].as_array().unwrap().len()
                );
            }
            v4["response_format"] = v3["response_format"].clone();
            assert_eq!(v4, v3);
        }
    }
    #[test]
    fn all46_coordinate_tables_are_exhaustive_and_validate_only_declared_namespaces() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for row in rows_at(&root).unwrap() {
            let mut p =
                crate::semantic_presentation::coordinate_payload(&row["input"], &row["payload"])
                    .unwrap();
            let body: Value =
                serde_json::from_str(p["messages"][1]["content"].as_str().unwrap()).unwrap();
            for (key, channel, original) in [
                ("raw", "proposed_representation", "raw"),
                ("normalized", "operative_result", "normalized"),
            ] {
                let table = body["finding_coordinates"][key].as_array().unwrap();
                assert!(!table.is_empty());
                let mut canonical = std::collections::BTreeSet::new();
                for item in table {
                    let path = item["canonical_path"].as_str().unwrap();
                    assert!(canonical.insert(path));
                    assert!(row["input"][original].pointer(path).is_some());
                    assert!(
                        body[channel]
                            .pointer(item["displayed_path"].as_str().unwrap())
                            .is_some()
                    );
                    // Every declared coordinate roundtrips through the response protocol.
                    let mut j = json!({});
                    let quote = row["input"]["text"]
                        .as_str()
                        .unwrap()
                        .chars()
                        .next()
                        .unwrap()
                        .to_string();
                    for d in contract::DIMENSIONS {
                        j[d] = json!({"label":"faithful","reason":"structural test only","findings":[]});
                    }
                    j["fidelity"]["findings"] = json!([{"quote":quote,"raw_paths":[],"normalized_paths":[],"explanation":"coordinate test, no truth claim"}]);
                    let finding_key = if key == "raw" {
                        "raw_paths"
                    } else {
                        "normalized_paths"
                    };
                    j["fidelity"]["findings"][0][finding_key] = json!([item["displayed_path"]]);
                    let mapped = crate::semantic_presentation::canonical_judgments(
                        &row["input"],
                        &j.to_string(),
                    )
                    .unwrap();
                    assert_eq!(
                        mapped["fidelity"]["findings"][0][finding_key],
                        json!([path])
                    );
                    assert!(contract::checked(&row["input"], &mapped.to_string()).is_ok());
                    // Never accept a metadata pointer, unlisted wrapper or guessed correction.
                    for bad in [
                        "/evidence_fields/0/path",
                        "/proposed_representation/fields",
                        "/fields/missing",
                    ] {
                        j["fidelity"]["findings"][0][finding_key] = json!([bad]);
                        assert!(
                            crate::semantic_presentation::canonical_judgments(
                                &row["input"],
                                &j.to_string()
                            )
                            .is_err()
                        );
                    }
                }
            }
            p["messages"] = row["payload"]["messages"].clone();
            assert_eq!(p, row["payload"]);
        }
    }
    #[test]
    fn all46_presentations_reconstruct_originals_without_changing_route_or_semantic_fields() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for row in rows_at(&root).unwrap() {
            let changed =
                crate::semantic_presentation::payload(&row["input"], &row["payload"]).unwrap();
            let view: Value =
                serde_json::from_str(changed["messages"][1]["content"].as_str().unwrap()).unwrap();
            assert_eq!(
                crate::semantic_presentation::restore(&view["proposed_representation"]).unwrap(),
                row["input"]["raw"]
            );
            assert_eq!(
                crate::semantic_presentation::restore(&view["operative_result"]).unwrap(),
                row["input"]["normalized"]
            );
            assert_eq!(view["original_text_only_evidence"], row["input"]["text"]);
            let mut reversed = changed;
            reversed["messages"] = row["payload"]["messages"].clone();
            assert_eq!(reversed, row["payload"]);
        }
    }

    #[test]
    fn roster_preserves_all_sides_and_labels_never_enter_payload() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rows = rows_at(&root).unwrap();
        assert_eq!(rows.len(), 46);
        assert_eq!(
            rows.iter()
                .filter(|r| r["origin"].as_str().unwrap().starts_with("extra/"))
                .count(),
            8
        );
        for r in rows {
            assert_eq!(r["payload"], contract::payload(&r["input"]).unwrap());
            assert!(r["input"].get("origin").is_none());
            assert!(
                r["input"]["normalized"].is_object(),
                "{}: {}",
                r["origin"],
                r["input"]["normalization_error"]
            );
        }
    }
}

pub fn validate_annotations(d: &Path) -> Result<(), String> {
    let packet = read(d.join("blind-packet.json"))?;
    let a = read(d.join("blind-labels-a.json"))?;
    let b = read(d.join("blind-labels-b.json"))?;
    let rec = read(d.join("label-reconciliation.json"))?;
    let frozen = read(d.join("frozen-labels.json"))?;
    let hash = |n: &str| {
        fs::read(d.join(n))
            .map(|v| distributed_reference::sha256(&v))
            .map_err(|e| e.to_string())
    };
    if rec["blind_packet_sha256"] != hash("blind-packet.json")?
        || frozen["blind_packet_sha256"] != hash("blind-packet.json")?
        || rec["reviewer_a_sha256"] != hash("blind-labels-a.json")?
        || rec["reviewer_b_sha256"] != hash("blind-labels-b.json")?
        || rec["pending_adjudications"] != 0
    {
        return Err("conciliación sin vínculos íntegros".into());
    }
    contract::check_labels(&packet, &a, &b, &rec, &frozen)
}
fn checked_agent_items(
    packet: &Value,
    document: &Value,
) -> Result<std::collections::BTreeMap<String, Value>, String> {
    let mut out = std::collections::BTreeMap::new();
    if document["metadata"]["reference_labels_seen"] != false
        || document["metadata"]["prior_predictions_seen"] != false
        || document["metadata"]["external_calls"] != 0
    {
        return Err("exposición de agente no declarada correctamente".into());
    }
    let expected = packet["items"].as_array().ok_or("paquete sin entradas")?;
    for item in document["items"].as_array().ok_or("agente sin entradas")? {
        let id = item["id"].as_str().ok_or("agente sin ID")?;
        if item.as_object().is_none_or(|v| v.len() != 2) {
            return Err("campos de entrada de agente inválidos".into());
        }
        let input = expected
            .iter()
            .find(|r| r["id"] == id)
            .ok_or("ID de agente ajeno")?;
        let checked = contract::checked(&input["input"], &item["judgments"].to_string())?;
        if out.insert(id.to_owned(), checked).is_some() {
            return Err("ID de agente duplicado".into());
        }
    }
    if out.len() != expected.len() {
        return Err("cobertura de agente incompleta".into());
    }
    Ok(out)
}
/// Local Codex-agent evaluation; never performs network calls or changes reference labels.
pub fn report_agents(d: &Path, output: &Path) -> Result<(), String> {
    let parent = Path::new("outputs/iromem/evaluation/semantic-representation-check-v1-2026-10-21");
    validate_annotations(parent)?;
    let packet = read(d.join("blind-packet.json"))?;
    let protocol = read(d.join("protocol.json"))?;
    let canonical = read(parent.join("blind-packet.json"))?;
    let hash = |p: &Path| {
        fs::read(p)
            .map(|v| distributed_reference::sha256(&v))
            .map_err(|e| e.to_string())
    };
    if protocol["version"] != "semantic-codex-agents-v1"
        || packet.as_object().is_none_or(|o| o.len() != 6)
        || packet["coordinate_namespace"]
            != "Use canonical RFC6901 paths relative to each input.raw/input.normalized, as in rubric. These are agent protocol outputs, not replayed HTTP responses; displayed-envelope paths are not used."
        || protocol["packet_sha256"] != hash(&d.join("blind-packet.json"))?
        || packet["version"] != "semantic-codex-agents-v1-packet"
        || packet["contract"] != contract::CONTRACT
        || packet["rubric"] != contract::INSTRUCTIONS
        || packet["audit_instructions"] != crate::semantic_presentation::BASIS_SCOPE_AUDIT
        || packet["items"] != canonical["items"]
        || packet["items"].as_array().is_none_or(|v| v.len() != 46)
    {
        return Err("paquete de agentes distinto de contrato/inputs congelados".into());
    }
    for (stem, packet_name) in [
        ("agent-author", "blind-packet.json"),
        ("agent-adversary", "blind-packet.json"),
        ("agent-arbiter", "adjudication-packet.json"),
    ] {
        let attestation = read(d.join(format!("{stem}-attestation.json")))?;
        if attestation["packet_sha256"] != hash(&d.join(packet_name))?
            || attestation["output_sha256"] != hash(&d.join(format!("{stem}.json")))?
            || attestation["packet_used_to_generate_output"] != true
            || attestation["reference_labels_seen"] != false
            || attestation["external_calls"] != 0
            || attestation["recorded_after_initial_output"] != true
        {
            return Err("vínculo retrospectivo de agente inválido".into());
        }
    }
    let labels = read(parent.join("frozen-labels.json"))?;
    let author = checked_agent_items(&packet, &read(d.join("agent-author.json"))?)?;
    let critic = checked_agent_items(&packet, &read(d.join("agent-adversary.json"))?)?;
    let adjudication = read(d.join("adjudication-packet.json"))?;
    let adjudication_protocol = read(d.join("adjudication-protocol.json"))?;
    if adjudication_protocol["packet_sha256"] != hash(&d.join("adjudication-packet.json"))? {
        return Err("paquete arbitral alterado".into());
    }
    let arbiter = read(d.join("agent-arbiter.json"))?;
    if arbiter["metadata"]["reference_labels_seen"] != false
        || arbiter["metadata"]["reference_scores_seen"] != false
        || arbiter["metadata"]["proposals_visible"] != true
        || arbiter["metadata"]["external_calls"] != 0
    {
        return Err("exposición arbitral inválida".into());
    }
    let mut disputes = Vec::new();
    for row in packet["items"].as_array().unwrap() {
        let id = row["id"].as_str().ok_or("falta ID")?;
        for dim in contract::DIMENSIONS {
            let a = &author[id]["judgments"][dim];
            let b = &critic[id]["judgments"][dim];
            if a["label"] != b["label"] {
                let (first, second) = if disputes.len() % 2 == 0 {
                    (a, b)
                } else {
                    (b, a)
                };
                disputes.push(json!({"id":id,"dimension":dim,"input":row["input"],"proposal_a":first,"proposal_b":second}));
            }
        }
    }
    if adjudication.as_object().is_none_or(|o| o.len() != 6)
        || adjudication["version"] != "semantic-codex-adjudication-packet-v1"
        || adjudication["selection_rule"]
            != "Every dimension where two independent agent labels differ; selected without reference labels."
        || adjudication_protocol["version"] != "semantic-codex-adjudication-protocol-v1"
        || adjudication_protocol["selected_disagreements"] != json!(disputes.len())
        || adjudication["items"] != json!(disputes)
        || adjudication["contract"] != packet["contract"]
        || adjudication["rubric"] != packet["rubric"]
        || adjudication["audit_instructions"] != packet["audit_instructions"]
    {
        return Err("selección arbitral no reproduce desacuerdos".into());
    }
    let mut resolutions = std::collections::BTreeMap::new();
    for item in arbiter["items"].as_array().ok_or("árbitro sin entradas")? {
        let id = item["id"].as_str().ok_or("árbitro sin ID")?;
        let dim = item["dimension"].as_str().ok_or("árbitro sin dimensión")?;
        if item.as_object().is_none_or(|o| o.len() != 3)
            || !disputes
                .iter()
                .any(|v| v["id"] == id && v["dimension"] == dim)
            || resolutions
                .insert((id.to_owned(), dim.to_owned()), item["judgment"].clone())
                .is_some()
        {
            return Err("arbitraje ajeno/duplicado".into());
        }
    }
    if resolutions.len() != disputes.len() {
        return Err("arbitraje incompleto".into());
    }
    let mut metrics = [(0usize, 0usize, 0usize); 3];
    let mut operations = Vec::new();
    let mut aggregate_false_acceptances = 0;
    let mut aggregate_false_rejections = 0;
    for row in packet["items"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let expected = labels["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == id)
            .ok_or("falta referencia")?;
        let expected_accept = contract::DIMENSIONS
            .iter()
            .all(|d| expected[*d] == "faithful")
            && row["input"]["normalized"].is_object();
        let mut merged = author[id]["judgments"].clone();
        for dim in contract::DIMENSIONS {
            if let Some(j) = resolutions.get(&(id.to_owned(), dim.to_owned())) {
                merged[dim] = j.clone();
            }
        }
        let resolved = contract::checked(&row["input"], &merged.to_string())?;
        let aggregate = author[id]["accepts_representation"] == true
            && critic[id]["accepts_representation"] == true;
        aggregate_false_acceptances += usize::from(aggregate && !expected_accept);
        aggregate_false_rejections += usize::from(!aggregate && expected_accept);
        let variants = [&author[id], &critic[id], &resolved];
        for (i, v) in variants.iter().enumerate() {
            let accepted = v["accepts_representation"] == true;
            metrics[i].0 += contract::DIMENSIONS
                .iter()
                .filter(|d| v["judgments"][**d]["label"] != expected[**d])
                .count();
            metrics[i].1 += usize::from(accepted && !expected_accept);
            metrics[i].2 += usize::from(!accepted && expected_accept);
        }
        operations.push(json!({"id":id,"expected":expected,"expected_acceptance":expected_accept,"author":author[id],"adversary":critic[id],"original_both_accept":aggregate,"later_adjudicated":resolved}));
    }
    let scores=metrics.iter().map(|(m,a,r)|json!({"valid_inputs":46,"dimension_comparisons":184,"dimension_mismatches":m,"false_acceptances":a,"false_rejections":r})).collect::<Vec<_>>();
    let mut bindings = serde_json::Map::new();
    for name in [
        "blind-packet.json",
        "protocol.json",
        "agent-author.json",
        "agent-adversary.json",
        "adjudication-packet.json",
        "adjudication-protocol.json",
        "agent-arbiter.json",
        "agent-author-attestation.json",
        "agent-adversary-attestation.json",
        "agent-arbiter-attestation.json",
    ] {
        bindings.insert(name.into(), json!(hash(&d.join(name))?));
    }
    save(
        output,
        &json!({"version":"semantic-codex-agents-v1-report","planned_inputs":46,"planned_dimensions":184,"author":scores[0],"adversary":scores[1],"original_aggregate":{"false_acceptances":aggregate_false_acceptances,"false_rejections":aggregate_false_rejections,"acceptance_requires_both":true},"disputed_dimensions":disputes.len(),"later_adjudicated":scores[2],"original_individual_capacity_gate_passed":metrics[0]==(0,0,0)&&metrics[1]==(0,0,0),"later_adjudicated_known_label_agreement_passed":metrics[2]==(0,0,0),"individual_errors_not_repaired":true,"packet_output_attestations_recorded_after_initial_outputs":true,"pre_inference_output_binding_proven":false,"full_rationale_review_pending":true,"openrouter_http_requests_this_workflow":0,"codex_cost_usd":null,"codex_tokens":null,"codex_work_not_claimed_free":true,"production_latency_measured":false,"runtime_policy_changed":false,"fresh_validation":false,"passes_full_quality_goal":false,"reference_labels_sha256":hash(&parent.join("frozen-labels.json"))?,"bindings_sha256":bindings,"operations":operations}),
    )
}
pub fn report(d: &Path, output: &Path) -> Result<(), String> {
    report_with(d, output, 1)
}
pub fn report_presentation(d: &Path, output: &Path) -> Result<(), String> {
    report_with(d, output, 2)
}
pub fn report_coordinates(d: &Path, output: &Path) -> Result<(), String> {
    report_with(d, output, 3)
}
pub fn report_constrained(d: &Path, output: &Path) -> Result<(), String> {
    report_with(d, output, 4)
}
pub fn report_capacity(d: &Path, output: &Path) -> Result<(), String> {
    report_with(d, output, 5)
}
pub fn report_basis_scope(d: &Path, output: &Path) -> Result<(), String> {
    report_with(d, output, 6)
}
fn report_with(d: &Path, output: &Path, version: u8) -> Result<(), String> {
    use std::collections::{BTreeMap, BTreeSet};
    validate_annotations(d)?;
    let prep = read(d.join("preparation.json"))?;
    let preflight = read(d.join("preflight.json"))?;
    let hash = |p: &Path| {
        fs::read(p)
            .map(|v| distributed_reference::sha256(&v))
            .map_err(|e| e.to_string())
    };
    if prep["version"]
        != if version >= 2 {
            if version == 6 {
                "semantic-basis-scope-v1-preparation"
            } else if version == 5 {
                "semantic-capacity-v5-preparation"
            } else if version == 4 {
                "semantic-schema-v4-preparation"
            } else if version == 3 {
                "semantic-coordinates-v3-preparation"
            } else {
                "semantic-presentation-v2-preparation"
            }
        } else {
            "semantic-representation-check-v1-preparation"
        }
        || prep["requests_sha256"] != hash(&d.join("requests.jsonl"))?
        || prep["blind_packet_sha256"] != hash(&d.join("blind-packet.json"))?
    {
        return Err("preparación alterada".into());
    }
    for (name, h) in preflight["inputs_sha256"]
        .as_object()
        .ok_or("preflight sin vínculos")?
    {
        if *h != hash(&d.join(name))? {
            return Err(format!("input alterado: {name}"));
        }
    }
    if preflight["source_hashes"] != prep["source_hashes"] {
        return Err("fuentes preflight no coinciden".into());
    }
    for (name, h) in prep["source_hashes"].as_object().ok_or("faltan fuentes")? {
        if *h != hash(&d.join("source-executed").join(name))? {
            return Err(format!("fuente ejecutada alterada: {name}"));
        }
    }
    let lines = |name: &str| -> Result<Vec<Value>, String> {
        fs::read_to_string(d.join(name))
            .map_err(|e| e.to_string())?
            .lines()
            .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
            .collect()
    };
    let requests = lines("requests.jsonl")?;
    let events = lines("journal.jsonl")?;
    let labels = read(d.join("frozen-labels.json"))?;
    let mut request_ids = BTreeSet::new();
    for r in &requests {
        let id = r["id"].as_str().ok_or("request sin ID")?;
        let base = contract::payload(&r["input"])?;
        let expected = if version == 6 {
            crate::semantic_presentation::basis_scope_payload(&r["input"], &base)?
        } else if version == 5 {
            crate::semantic_presentation::capacity_payload(&r["input"], &base)?
        } else if version == 4 {
            crate::semantic_presentation::constrained_payload(&r["input"], &base)?
        } else if version >= 2 {
            if version == 3 {
                crate::semantic_presentation::coordinate_payload(&r["input"], &base)?
            } else {
                crate::semantic_presentation::payload(&r["input"], &base)?
            }
        } else {
            base
        };
        if !request_ids.insert(id) || r["payload"] != expected {
            return Err("request duplicado o payload alterado".into());
        }
    }
    if requests.len() != 46 {
        return Err("roster esperado incompleto".into());
    }
    let mut starts = BTreeSet::new();
    let mut terminal = BTreeMap::new();
    for e in &events {
        let id = e["id"].as_str().ok_or("evento sin ID")?;
        if !request_ids.contains(id) {
            return Err("evento ajeno".into());
        }
        match e["kind"].as_str() {
            Some("started") => {
                if !starts.insert(id) {
                    return Err("operación repetida".into());
                }
            }
            Some("response" | "failed") => {
                if !starts.contains(id) || terminal.insert(id, e).is_some() {
                    return Err("terminal repetido o no iniciado".into());
                }
            }
            _ => return Err("evento desconocido".into()),
        }
    }
    let mut rows = Vec::new();
    let mut known_cost = 0.;
    let mut unknown = 0;
    let mut completed = 0;
    let mut mismatches = 0;
    let mut false_acceptances = 0;
    let mut false_rejections = 0;
    let mut times = Vec::new();
    let mut tokens_in = 0u64;
    let mut tokens_out = 0u64;
    let mut confusion = BTreeMap::<String, usize>::new();
    for r in &requests {
        let id = r["id"].as_str().unwrap();
        let expected = labels["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == id)
            .ok_or("falta label")?;
        let expected_accept = contract::DIMENSIONS
            .iter()
            .all(|dim| expected[*dim] == "faithful")
            && r["input"]["normalized"].is_object();
        let mut row = json!({"id":id,"expected":expected,"expected_acceptance":expected_accept,"technical_status":"not_sent","semantic_result":null});
        if starts.contains(id) {
            row["technical_status"] = json!("terminal_missing");
            let mut fee = None;
            if let Some(e) = terminal.get(id) {
                if let Some(ms) = e["elapsed_ms"]
                    .as_f64()
                    .filter(|n| n.is_finite() && *n >= 0.)
                {
                    times.push(ms);
                }
                let body = &e["body"];
                fee = body["usage"]["cost"]
                    .as_f64()
                    .filter(|n| n.is_finite() && *n >= 0.);
                let tin = body["usage"]["prompt_tokens"].as_u64();
                let tout = body["usage"]["completion_tokens"].as_u64();
                tokens_in += tin.unwrap_or(0);
                tokens_out += tout.unwrap_or(0);
                row["event"] = (*e).clone();
                let valid = e["kind"] == "response"
                    && e["http_status"]
                        .as_u64()
                        .is_some_and(|s| (200..300).contains(&s))
                    && (if version >= 5 {
                        vec![json!(crate::semantic_presentation::CAPACITY_MODEL)]
                    } else {
                        vec![json!(contract::MODEL), json!("openai/gpt-4.1-2025-04-14")]
                    })
                    .contains(&body["model"])
                    && body["provider"]
                        == if version >= 5 {
                            crate::semantic_presentation::CAPACITY_PROVIDER
                        } else {
                            "OpenAI"
                        }
                    && body["choices"].as_array().is_some_and(|a| a.len() == 1)
                    && body["choices"][0]["finish_reason"] == "stop"
                    && body["choices"][0]["message"]
                        .get("refusal")
                        .is_none_or(|v| v.is_null());
                if valid {
                    let parsed = body["choices"][0]["message"]["content"]
                        .as_str()
                        .ok_or_else(|| "contenido no textual".to_owned())
                        .and_then(|s| {
                            if version >= 3 {
                                let canonical = crate::semantic_presentation::canonical_judgments(
                                    &r["input"],
                                    s,
                                )?;
                                contract::checked(&r["input"], &canonical.to_string())
                            } else {
                                contract::checked(&r["input"], s)
                            }
                        });
                    match parsed {
                        Ok(result) => {
                            let receipt = fee.is_some() && tin.is_some() && tout.is_some();
                            row["technical_status"] = json!(if receipt {
                                "completed"
                            } else {
                                "receipt_mismatch"
                            });
                            if receipt {
                                completed += 1;
                            }
                            let actual_accept = result["accepts_representation"] == true;
                            false_acceptances += usize::from(actual_accept && !expected_accept);
                            false_rejections += usize::from(!actual_accept && expected_accept);
                            let mut errors = Vec::new();
                            for dim in contract::DIMENSIONS {
                                let actual = result["judgments"][dim]["label"].as_str().unwrap();
                                let want = expected[dim].as_str().unwrap();
                                *confusion
                                    .entry(format!("{dim}/{want}/{actual}"))
                                    .or_default() += 1;
                                if actual != want {
                                    errors.push(
                                        json!({"dimension":dim,"expected":want,"actual":actual}),
                                    );
                                    mismatches += 1;
                                }
                            }
                            row["dimension_mismatches"] = json!(errors);
                            row["semantic_acceptance"] = json!(actual_accept);
                            row["operative_acceptance"] = json!(receipt && actual_accept);
                            row["semantic_result"] = result;
                        }
                        Err(e) => {
                            row["technical_status"] = json!("invalid_response");
                            row["validation_error"] = json!(e);
                        }
                    }
                } else {
                    row["technical_status"] = json!("provider_error");
                }
            }
            if let Some(fee) = fee {
                known_cost += fee;
                row["known_cost_usd"] = json!(fee);
            } else {
                unknown += 1;
                row["known_cost_usd"] = Value::Null;
            }
        }
        rows.push(row);
    }
    times.sort_by(f64::total_cmp);
    let quantile = |p: f64| {
        if times.is_empty() {
            None
        } else {
            Some(times[((times.len() as f64 * p).ceil() as usize).saturating_sub(1)])
        }
    };
    save(
        output,
        &json!({"version":if version >= 2{if version == 6 { "semantic-basis-scope-v1-report" } else if version == 5 { "semantic-capacity-v5-report" } else if version == 4 { "semantic-schema-v4-report" } else if version == 3 { "semantic-coordinates-v3-report" } else { "semantic-presentation-v2-report" }}else{"semantic-representation-check-v1-report"},"planned_operations":46,"attempted_operations":starts.len(),"terminal_events":terminal.len(),"completed_valid_operations":completed,"unsent_operations":46-starts.len(),"known_provider_cost_usd":known_cost,"unknown_cost_operations":unknown,"reported_input_tokens_subtotal":tokens_in,"reported_output_tokens_subtotal":tokens_out,"dimension_comparisons_planned":184,"dimension_comparisons_observed":confusion.values().sum::<usize>(),"dimension_mismatches":mismatches,"false_acceptances":false_acceptances,"false_rejections":false_rejections,"dimension_confusion":confusion,"attempt_terminal_p50_ms":quantile(0.5),"attempt_terminal_p95_ms":quantile(0.95),"timed_terminal_attempts":times.len(),"quantitative_capacity_gate_passed":completed==46&&mismatches==0&&false_acceptances==0&&false_rejections==0&&unknown==0,"critical_finding_semantic_review_pending":true,"passes_capacity_gate":false,"model_extraction_measured":false,"fresh_validation":false,"promoted":false,"operations":rows,"requests_sha256":hash(&d.join("requests.jsonl"))?,"journal_sha256":hash(&d.join("journal.jsonl"))?,"preflight_sha256":hash(&d.join("preflight.json"))?,"frozen_labels_sha256":hash(&d.join("frozen-labels.json"))?}),
    )
}

#[cfg(test)]
mod report_tests {
    use super::*;
    #[test]
    fn report_keeps_charged_http_errors_unknown_inflight_and_unsent_in_denominator() {
        let d = std::env::temp_dir().join(format!(
            "gennie-memory-semantic-report-{}",
            std::process::id()
        ));
        assert!(!d.exists());
        fs::create_dir(&d).unwrap();
        let input = json!({"kind":"source","text":"verde","raw":{},"normalized":{},"normalization_error":null,"ontology":{},"registry":{}});
        let mut j = json!({});
        for dim in contract::DIMENSIONS {
            j[dim] = json!({"label":"faithful","reason":"conservado","findings":[]});
        }
        let mut req = Vec::new();
        let mut items = Vec::new();
        let mut labels = Vec::new();
        let mut annotations = Vec::new();
        for i in 0..46 {
            let id = format!("s{i:03}");
            req.push(json!({"id":id,"input":input,"payload":contract::payload(&input).unwrap()}));
            items.push(json!({"id":id,"input":input}));
            let mut label = json!({"id":id});
            let mut review = json!({"id":id});
            for dim in contract::DIMENSIONS {
                label[dim] = json!("faithful");
                review[dim] = j[dim].clone();
            }
            labels.push(label);
            annotations.push(review);
        }
        let serialized = req.iter().map(|v| v.to_string() + "\n").collect::<String>();
        fs::write(d.join("requests.jsonl"), &serialized).unwrap();
        save(&d.join("blind-packet.json"), &json!({"items":items})).unwrap();
        save(
            &d.join("blind-labels-a.json"),
            &json!({"items":annotations}),
        )
        .unwrap();
        save(
            &d.join("blind-labels-b.json"),
            &json!({"items":annotations}),
        )
        .unwrap();
        let hash = |n: &str| distributed_reference::sha256(&fs::read(d.join(n)).unwrap());
        save(&d.join("frozen-labels.json"),&json!({"pending_adjudications":0,"blind_packet_sha256":hash("blind-packet.json"),"items":labels})).unwrap();
        save(&d.join("label-reconciliation.json"),&json!({"pending_adjudications":0,"blind_packet_sha256":hash("blind-packet.json"),"reviewer_a_sha256":hash("blind-labels-a.json"),"reviewer_b_sha256":hash("blind-labels-b.json"),"decisions":[]})).unwrap();
        save(&d.join("preparation.json"),&json!({"version":"semantic-representation-check-v1-preparation","requests_sha256":hash("requests.jsonl"),"blind_packet_sha256":hash("blind-packet.json"),"source_hashes":{}})).unwrap();
        save(&d.join("preflight.json"),&json!({"inputs_sha256":{"preparation.json":hash("preparation.json"),"requests.jsonl":hash("requests.jsonl"),"blind-packet.json":hash("blind-packet.json"),"frozen-labels.json":hash("frozen-labels.json"),"blind-labels-a.json":hash("blind-labels-a.json"),"blind-labels-b.json":hash("blind-labels-b.json"),"label-reconciliation.json":hash("label-reconciliation.json")},"source_hashes":{}})).unwrap();
        let events = vec![
            json!({"id":"s000","kind":"started"}),
            json!({"id":"s000","kind":"response","http_status":200,"elapsed_ms":10.,"body":{"model":contract::MODEL,"provider":"OpenAI","choices":[{"finish_reason":"stop","message":{"content":j.to_string()}}],"usage":{"cost":0.1,"prompt_tokens":2,"completion_tokens":3}}}),
            json!({"id":"s001","kind":"started"}),
            json!({"id":"s001","kind":"response","http_status":429,"elapsed_ms":20.,"body":{"usage":{"cost":0.2,"prompt_tokens":1,"completion_tokens":0}}}),
            json!({"id":"s002","kind":"started"}),
            json!({"id":"s003","kind":"started"}),
            json!({"id":"s003","kind":"response","http_status":200,"elapsed_ms":30.,"body":{"model":contract::MODEL,"provider":"OpenAI","choices":[{"finish_reason":"stop","message":{"content":j.to_string()}}],"usage":{"prompt_tokens":2,"completion_tokens":3}}}),
        ];
        let journal = events
            .iter()
            .map(|v| v.to_string() + "\n")
            .collect::<String>();
        fs::write(d.join("journal.jsonl"), &journal).unwrap();
        report(&d, &d.join("report.json")).unwrap();
        let r = read(d.join("report.json")).unwrap();
        assert_eq!(r["attempted_operations"], 4);
        assert_eq!(r["unsent_operations"], 42);
        assert_eq!(r["completed_valid_operations"], 1);
        assert_eq!(r["unknown_cost_operations"], 2);
        assert!((r["known_provider_cost_usd"].as_f64().unwrap() - 0.3).abs() < 1e-12);
        assert_eq!(r["quantitative_capacity_gate_passed"], false);
        assert_eq!(r["operations"][2]["technical_status"], "terminal_missing");
        assert_eq!(r["operations"][3]["technical_status"], "receipt_mismatch");
        assert_eq!(r["operations"][3]["semantic_acceptance"], true);
        assert_eq!(r["operations"][3]["operative_acceptance"], false);
        assert_eq!(r["false_rejections"], 0);
        assert!(report(&d, &d.join("report.json")).is_err());
        let mut repeated = journal;
        repeated.push_str("{\"id\":\"s000\",\"kind\":\"started\"}\n");
        fs::write(d.join("journal.jsonl"), repeated).unwrap();
        assert!(report(&d, &d.join("bad.json")).is_err());
        fs::remove_dir_all(&d).unwrap();
    }
}
