//! Text-only full known-bank extraction; integrity is not semantic certification.
use super::*;
use std::collections::BTreeMap;

const SPEC: &str = "docs/evaluation/codex-entry-bank-v1-protocol-2026-10-21.md";
const ENTRY: &str = "docs/evaluation/entry-via-v1-contract-2026-10-21.md";
const PILOT: &str = "outputs/iromem/evaluation/codex-source-extraction-v2-2026-10-21";
const SOURCES: &[&str] = &[
    "tools/evaluation/src/typed_facts/v2/record_projection/automatic_extraction/codex/bank.rs",
    "tools/evaluation/src/typed_facts/v2/record_projection/automatic_extraction/codex.rs",
    "tools/evaluation/src/typed_facts/v2/record_projection/automatic_extraction.rs",
    "tools/evaluation/src/documentary_extraction_contract.rs",
    "tools/evaluation/src/documentary_extraction_contract/grounded_references.rs",
    "tools/evaluation/src/documentary_extraction_contract/typed_clauses.rs",
    "tools/evaluation/src/typed_facts.rs",
    "tools/evaluation/src/typed_facts/v2.rs",
    "tools/evaluation/src/typed_facts/v2/record_projection.rs",
    "tools/evaluation/src/main.rs",
];

fn digest_value(value: &Value) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    Ok(
        Sha256::digest(serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
    )
}
fn write(path: &Path, value: &Value) -> Result<(), String> {
    crate::scoped_definition_probe::write_new(path, value)
}

// Only the text leaves and ontology are projected. Frames, pairs and labels never
// enter a packet, even when extra fields occur in a bank record.
fn packets(input: &Value) -> Result<(Vec<Value>, Value), String> {
    let mut ontology = input["ontology"].clone();
    vocabulary(&ontology)?;
    if ontology.get("entry_via").is_some() {
        return Err("banco ya contiene entry_via".into());
    }
    ontology["entry_via"] = json!({"kind":"property","additive":false,"intermediate_types":[]});
    vocabulary(&ontology)?;
    let mut batches = Vec::new();
    let mut mapping = serde_json::Map::new();
    for (kind, field, size) in [("source", "assertions", 12), ("question", "questions", 11)] {
        let records: BTreeMap<_, _> = input[field]
            .as_object()
            .ok_or("faltan textos del banco")?
            .iter()
            .collect();
        let schema = contract::grounded_references::schema(kind, &ontology)?;
        let mut items = Vec::new();
        for (index, (target, record)) in records.into_iter().enumerate() {
            let id = format!("{kind}_{index:03}");
            mapping.insert(id.clone(), json!({"kind":kind,"target":target}));
            items.push(json!({"id":id,"text":text(record,"text")?,"output_schema":schema}));
        }
        for (index, chunk) in items.chunks(size).enumerate() {
            batches.push(json!({"name":format!("{kind}-{index:02}"),"packet":{"version":"codex-source-extraction-v2-packet","kind":kind,"ontology":ontology,"instructions":format!("{}{}",contract::grounded_references::instructions(kind)?,ENTRY_VIA_GUIDANCE),"items":chunk}}));
        }
    }
    Ok((batches, Value::Object(mapping)))
}

const PILOT_ARTIFACTS: &[&str] = &[
    "protocol.json",
    "preflight.json",
    "normalization.json",
    "review-packet.json",
    "review-launch.json",
    "blind-review.json",
    "adversarial-review.json",
    "review-reconciliation.json",
    "adapter-code-review.json",
    "adapter-code-review-fixed.json",
    "negative-cli-checks.json",
    "execution-provenance.json",
    "verification.json",
];
const PILOT_DELIVERIES: &[&str] = &[
    "source-packet.json",
    "source-output.json",
    "question-packet.json",
    "question-output.json",
];
const DIMENSIONS: &[&str] = &[
    "fidelity",
    "completeness",
    "normalized_preservation",
    "evidence_attribution",
];

fn review_projection(rows: &[Value]) -> Vec<Value> {
    rows.iter().map(|row| json!({"id":row["id"],"kind":row["kind"],"text":row["text"],"raw":row["raw"],"normalized":row["normalized"],"normalization_error":row["normalization_error"],"ontology":row["ontology"]})).collect()
}

fn faithful_review(review: &Value, items: &[Value], packet_hash: &str) -> Result<(), String> {
    closed(review, &["version", "packet_sha256", "exposure", "items"])?;
    if review["version"] != "codex-source-extraction-v2-review"
        || review["packet_sha256"] != packet_hash
        || review["exposure"]
            != json!({"only_assigned_packet":true,"gold_seen":false,"manual_frames_seen":false,"other_reviews_seen":false})
        || list(review, "items")?.len() != items.len()
    {
        return Err("crítica piloto sin vínculo, exposición o cobertura fiables".into());
    }
    let mut ids = BTreeSet::new();
    for (expected, actual) in items.iter().zip(list(review, "items")?) {
        closed(actual, &["id", "judgments"])?;
        if actual["id"] != expected["id"] || !ids.insert(text(actual, "id")?) {
            return Err("IDs de crítica piloto incompatibles".into());
        }
        closed(&actual["judgments"], DIMENSIONS)?;
        for dimension in DIMENSIONS {
            let judgment = &actual["judgments"][*dimension];
            closed(judgment, &["label", "reason", "findings"])?;
            if judgment["label"] != "faithful"
                || text(judgment, "reason")?.trim().is_empty()
                || judgment["findings"] != json!([])
            {
                return Err("crítica piloto no demuestra pase completo declarado".into());
            }
        }
    }
    Ok(())
}

fn reconstructed_pilot(pilot: &Path, normalized: &Value) -> Result<(), String> {
    let mut rows = Vec::new();
    let mut contracts = serde_json::Map::new();
    for kind in ["source", "question"] {
        let packet_path = pilot.join(format!("{kind}-packet.json"));
        let packet = read(&packet_path)?;
        let response = read(&pilot.join(format!("{kind}-output.json")))?;
        checked_outputs(&packet, &response, &hash(&packet_path)?)?;
        let items = list(&packet, "items")?;
        if items.len() != 4 || packet["kind"] != kind {
            return Err("etapa piloto incompleta".into());
        }
        let schema = contract::grounded_references::schema(kind, &packet["ontology"])?;
        contracts.insert(
            kind.into(),
            json!({"instructions":packet["instructions"],"output_schema":schema}),
        );
        for (item, answer) in items.iter().zip(list(&response, "items")?) {
            entry_property_only(&answer["raw"])?;
            let mut row = normalize(
                kind,
                &text(item, "text")?,
                &packet["ontology"],
                &answer["raw"],
            );
            if !row["normalized"].is_object() || !row["normalization_error"].is_null() {
                return Err("DTO piloto no reproduce pase válido".into());
            }
            for key in ["id", "text"] {
                row[key] = item[key].clone();
            }
            row["kind"] = json!(kind);
            row["ontology"] = packet["ontology"].clone();
            row["raw"] = answer["raw"].clone();
            rows.push(row);
        }
    }
    if rows.len() != 8 || normalized["operations"] != json!(rows) {
        return Err("normalización piloto no reproduce entregas".into());
    }
    let packet = read(&pilot.join("review-packet.json"))?;
    closed(
        &packet,
        &[
            "version",
            "scope",
            "contracts",
            "normalized_contract",
            "dimensions",
            "labels",
            "items",
        ],
    )?;
    let projected = review_projection(&rows);
    if packet["version"] != "codex-source-extraction-v2-review-packet"
        || packet["items"] != json!(projected)
        || packet["contracts"] != Value::Object(contracts)
        || packet["dimensions"] != json!(DIMENSIONS)
        || packet["labels"] != json!(["faithful", "defective", "uncertain", "unavailable"])
    {
        return Err("paquete de crítica piloto no deriva de normalización y contratos".into());
    }
    let packet_hash = hash(&pilot.join("review-packet.json"))?;
    for name in ["blind-review.json", "adversarial-review.json"] {
        faithful_review(&read(&pilot.join(name))?, &projected, &packet_hash)?;
    }
    Ok(())
}

fn frozen_inputs() -> Result<(Value, Value), String> {
    let input = validated_bank()?;
    if input["assertions"].as_object().map(|o| o.len()) != Some(83)
        || input["questions"].as_object().map(|o| o.len()) != Some(33)
        || input["pairs"].as_array().map(|a| a.len()) != Some(100)
    {
        return Err("cobertura del banco conocido incompatible".into());
    }
    let bank = Path::new(BANK);
    let manifest = read(&bank.join("manifest.json"))?;
    let mut hashes = serde_json::Map::new();
    for field in ["files_sha256", "external_sha256"] {
        for (name, expected) in manifest[field]
            .as_object()
            .ok_or("faltan hashes del banco")?
        {
            let path = if field == "files_sha256" {
                bank.join(name)
            } else {
                Path::new(name).to_path_buf()
            };
            let actual = hash(&path)?;
            if expected != &json!(actual) {
                return Err("banco congelado alterado".into());
            }
            hashes.insert(path.to_string_lossy().into_owned(), json!(actual));
        }
    }
    let pilot = Path::new(PILOT);
    let report = read(&pilot.join("report.json"))?;
    if report["version"] != "codex-source-extraction-v2-report"
        || report["planned_raws"] != 8
        || report["received_raws"] != 8
        || report["valid_dtos"] != 8
        || report["both_reviewers_all_four_dimensions_faithful"] != 8
        || report["review_counts"] != json!([{"faithful":32},{"faithful":32}])
        || report["review_dimensions_per_reviewer"] != 32
        || report["review_disagreements"] != json!([])
        || report["pilot_passed"] != true
        || report["passes_full_quality_goal"] != false
    {
        return Err("piloto V2 no permite ampliar al banco".into());
    }
    closed(&report["artifact_sha256"], PILOT_ARTIFACTS)?;
    for (name, expected) in report["artifact_sha256"]
        .as_object()
        .ok_or("piloto sin vínculos")?
    {
        let path = pilot.join(name);
        let actual = hash(&path)?;
        if expected != &json!(actual) {
            return Err("artefacto del piloto alterado".into());
        }
        hashes.insert(path.to_string_lossy().into_owned(), json!(actual));
    }
    let parent = read(&pilot.join("protocol.json"))?;
    check_entry_lineage(pilot, &parent)?;
    // Bind actual deliveries too: normalization's bindings are checked, not trusted.
    let normalized = read(&pilot.join("normalization.json"))?;
    if normalized["valid_dtos"] != 8
        || normalized["planned_operations"] != 8
        || normalized["protocol_sha256"] != hash(&pilot.join("protocol.json"))?
    {
        return Err("normalización piloto incompatible".into());
    }
    closed(&normalized["bindings_sha256"], PILOT_DELIVERIES)?;
    for (name, expected) in normalized["bindings_sha256"]
        .as_object()
        .ok_or("faltan vínculos piloto")?
    {
        let path = pilot.join(name);
        let actual = hash(&path)?;
        if expected != &json!(actual) {
            return Err("entrega piloto alterada".into());
        }
        hashes.insert(path.to_string_lossy().into_owned(), json!(actual));
    }
    reconstructed_pilot(pilot, &normalized)?;
    for path in [
        bank.join("manifest.json"),
        pilot.join("report.json"),
        Path::new(SPEC).to_path_buf(),
        Path::new(ENTRY).to_path_buf(),
    ] {
        hashes.insert(path.to_string_lossy().into_owned(), json!(hash(&path)?));
    }
    Ok((input, Value::Object(hashes)))
}

fn protocol(
    batches: &[Value],
    mapping: &Value,
    inputs: Value,
    sources: Value,
) -> Result<Value, String> {
    let jobs: Result<Vec<_>,String> = batches.iter().map(|batch| {
        let name = text(batch,"name")?;
        let packet = &batch["packet"];
        Ok(json!({"kind":packet["kind"],"packet":format!("{name}-packet.json"),"output":format!("{name}-output.json"),"packet_sha256":digest_value(packet)?,"expected_ids":list(packet,"items")?.iter().map(|v|v["id"].clone()).collect::<Vec<_>>()}))
    }).collect();
    Ok(
        json!({"version":"codex-entry-bank-v1-protocol","specification":SPEC,"entry_via_contract":ENTRY,"frozen_inputs_sha256":inputs,"source_sha256_before_inference":sources,"mapping_sha256":digest_value(mapping)?,"jobs":jobs?,"expected_operations":116,"expected_source_operations":83,"expected_question_operations":33,"known_pairs":100,"raw_submissions_per_job":1,"normalizer_feedback_before_submission":false,"within_batch_shared_context":true,"cross_stage_context_shared":false,"openrouter_calls_planned":0,"codex_cost_tokens_unknown":true,"manual_integrity_is_not_semantic_fidelity":true}),
    )
}

pub fn prepare(dir: &Path) -> Result<(), String> {
    let (input, inputs) = frozen_inputs()?;
    let (batches, mapping) = packets(&input)?;
    let mut sources = serde_json::Map::new();
    for name in SOURCES {
        sources.insert((*name).into(), json!(hash(Path::new(name))?));
    }
    let protocol = protocol(&batches, &mapping, inputs, Value::Object(sources))?;
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    for name in SOURCES {
        let target = dir.join("source-before-inference").join(name);
        fs::create_dir_all(target.parent().ok_or("ruta fuente inválida")?)
            .map_err(|e| e.to_string())?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)
            .map_err(|e| e.to_string())?;
        file.write_all(&fs::read(name).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
    }
    for batch in &batches {
        write(
            &dir.join(format!("{}-packet.json", text(batch, "name")?)),
            &batch["packet"],
        )?;
    }
    write(&dir.join("mapping.json"), &mapping)?;
    write(&dir.join("protocol.json"), &protocol)
}

fn normalized_rows(
    packet: &Value,
    mapping: &Value,
    response: Option<&Value>,
    failure: Option<&str>,
) -> Result<Vec<Value>, String> {
    let kind = text(packet, "kind")?;
    let mut rows = Vec::new();
    for (index, item) in list(packet, "items")?.iter().enumerate() {
        let raw = response
            .and_then(|r| r["items"].as_array())
            .and_then(|a| a.get(index))
            .map(|v| v["raw"].clone())
            .unwrap_or(Value::Null);
        let mut row = if let Some(error) = failure {
            json!({"normalized":null,"translated":null,"normalization_error":error,"error_stage":"batch_delivery"})
        } else if let Err(error) = entry_property_only(&raw) {
            json!({"normalized":null,"translated":null,"normalization_error":error,"error_stage":"entry_via_contract"})
        } else {
            normalize(&kind, &text(item, "text")?, &packet["ontology"], &raw)
        };
        let id = text(item, "id")?;
        row["id"] = json!(id);
        row["kind"] = json!(kind);
        row["target"] = mapping[&id]["target"].clone();
        row["text"] = item["text"].clone();
        row["ontology"] = packet["ontology"].clone();
        // Unreliably bound rows never borrow any content from a malformed envelope.
        row["raw"] = if failure.is_some() { Value::Null } else { raw };
        rows.push(row);
    }
    Ok(rows)
}

pub fn report(dir: &Path, output: &Path) -> Result<(), String> {
    let (input, inputs) = frozen_inputs()?;
    let (batches, mapping) = packets(&input)?;
    let prepared = read(&dir.join("protocol.json"))?;
    let sources = prepared["source_sha256_before_inference"]
        .as_object()
        .ok_or("sin fuentes congeladas")?;
    if sources.len() != SOURCES.len() {
        return Err("roster fuente alterado".into());
    }
    for name in SOURCES {
        if sources.get(*name)
            != Some(&json!(hash(
                &dir.join("source-before-inference").join(name)
            )?))
        {
            return Err("copia de fuente previa alterada".into());
        }
    }
    if prepared != protocol(&batches, &mapping, inputs, Value::Object(sources.clone()))? {
        return Err("protocolo del banco alterado".into());
    }
    if read(&dir.join("mapping.json"))? != mapping
        || hash(&dir.join("mapping.json"))? != text(&prepared, "mapping_sha256")?
    {
        return Err("mapa privado alterado".into());
    }
    let mut rows = Vec::new();
    let mut statuses = Vec::new();
    let mut bindings = serde_json::Map::new();
    let mut observed = 0;
    for (batch, job) in batches.iter().zip(list(&prepared, "jobs")?) {
        let name = text(job, "packet")?;
        let packet = &batch["packet"];
        let ph = hash(&dir.join(&name))?;
        if read(&dir.join(&name))? != *packet || job["packet_sha256"] != ph {
            return Err("paquete del banco alterado".into());
        }
        bindings.insert(name, json!(ph));
        let outname = text(job, "output")?;
        let path = dir.join(&outname);
        let delivery = match fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.to_string()),
        };
        let parsed = delivery
            .as_ref()
            .map(|bytes| serde_json::from_slice::<Value>(bytes));
        let failure = match &parsed {
            None => Some("entrega del lote ausente".to_string()),
            Some(Err(e)) => Some(format!("entrega del lote sin JSON fiable: {e}")),
            Some(Ok(response)) => checked_outputs(packet, response, &ph).err(),
        };
        let response = parsed.as_ref().and_then(|r| r.as_ref().ok());
        if let Some(response) = response {
            observed += response["items"].as_array().map_or(0, |a| a.len());
        }
        let batch_rows = normalized_rows(packet, &mapping, response, failure.as_deref())?;
        let valid = batch_rows
            .iter()
            .filter(|r| r["normalized"].is_object())
            .count();
        if delivery.is_some() {
            bindings.insert(outname.clone(), json!(hash(&path)?));
        }
        // The original bytes survive also for non-UTF8 or non-JSON deliveries.
        statuses.push(json!({"packet":job["packet"],"output":outname,"status":if failure.is_some(){"degraded"}else{"bound"},"delivery_error":failure,"planned_operations":batch_rows.len(),"valid_dtos":valid,"raw_delivery_utf8":delivery.as_ref().and_then(|b|std::str::from_utf8(b).ok()),"raw_delivery_bytes":delivery}));
        rows.extend(batch_rows);
    }
    let valid = rows.iter().filter(|r| r["normalized"].is_object()).count();
    bindings.insert(
        "mapping.json".into(),
        json!(hash(&dir.join("mapping.json"))?),
    );
    let report = json!({"version":"codex-entry-bank-v1-normalization","protocol_sha256":hash(&dir.join("protocol.json"))?,"bindings_sha256":bindings,"planned_operations":116,"observed_raw_operations":observed,"valid_dtos":valid,"operations":rows,"batch_statuses":statuses,"semantic_review_pending":true,"semantic_fidelity_measured":false,"passes_full_quality_goal":false,"new_openrouter_calls":0,"codex_cost_usd":null,"codex_tokens":null,"production_latency_measured":false,"promoted":false});
    write(output, &report)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bank() -> Value {
        let mut b = json!({"ontology":{"color":{"kind":"property","additive":false,"intermediate_types":[]}},"assertions":{},"questions":{},"pairs":[{"gold":"must not escape"}],"gold":"hidden"});
        for (field, n) in [("assertions", 83), ("questions", 33)] {
            for i in (0..n).rev() {
                b[field][format!("original_{i:03}")] =
                    json!({"text":format!("Texto {i}"),"frame":{"secret":true},"gold":"hidden"});
            }
        }
        b
    }
    #[test]
    fn pilot_requires_exact_hash_rosters_and_complete_faithful_reviews() {
        for keys in [PILOT_ARTIFACTS, PILOT_DELIVERIES] {
            assert!(closed(&json!({}), keys).is_err());
            let mut hashes = serde_json::Map::new();
            for key in keys {
                hashes.insert((*key).into(), json!("hash"));
            }
            assert!(closed(&Value::Object(hashes.clone()), keys).is_ok());
            hashes.insert("forged.json".into(), json!("hash"));
            assert!(closed(&Value::Object(hashes), keys).is_err());
        }
        let items = vec![json!({"id":"op_000"}), json!({"id":"op_001"})];
        let mut judgments = serde_json::Map::new();
        for dimension in DIMENSIONS {
            judgments.insert(
                (*dimension).into(),
                json!({"label":"faithful","reason":"Literal preservation checked.","findings":[]}),
            );
        }
        let review = json!({"version":"codex-source-extraction-v2-review","packet_sha256":"h","exposure":{"only_assigned_packet":true,"gold_seen":false,"manual_frames_seen":false,"other_reviews_seen":false},"items":[{"id":"op_000","judgments":judgments},{"id":"op_001","judgments":judgments}]});
        faithful_review(&review, &items, "h").unwrap();
        for (path, value) in [
            ("/items/1/judgments/fidelity/label", json!("uncertain")),
            ("/items/1/judgments/fidelity/reason", json!(" ")),
            ("/items/1/judgments/fidelity/findings", json!([{}])),
            ("/items/1/id", json!("op_000")),
            ("/exposure/other_reviews_seen", json!(true)),
            ("/packet_sha256", json!("forged")),
        ] {
            let mut forged = review.clone();
            *forged.pointer_mut(path).unwrap() = value;
            assert!(faithful_review(&forged, &items, "h").is_err(), "{path}");
        }
        let mut missing = review;
        missing["items"] = json!([]);
        assert!(faithful_review(&missing, &items, "h").is_err());
    }
    #[test]
    fn opaque_sorted_complete_chunks_and_only_text_projection() {
        let (batches, mapping) = packets(&bank()).unwrap();
        assert_eq!(batches.len(), 10);
        assert_eq!(mapping.as_object().unwrap().len(), 116);
        let sizes: Vec<_> = batches
            .iter()
            .map(|b| b["packet"]["items"].as_array().unwrap().len())
            .collect();
        assert_eq!(sizes, vec![12, 12, 12, 12, 12, 12, 11, 11, 11, 11]);
        for batch in &batches {
            closed(
                &batch["packet"],
                &["version", "kind", "ontology", "instructions", "items"],
            )
            .unwrap();
            for item in list(&batch["packet"], "items").unwrap() {
                closed(item, &["id", "text", "output_schema"]).unwrap();
            }
            let serialized = batch.to_string();
            assert!(!serialized.contains("hidden"));
            assert!(!serialized.contains("original_"));
            assert!(!serialized.contains("secret"));
        }
        assert_eq!(batches[0]["packet"]["items"][0]["id"], "source_000");
        assert_eq!(batches[6]["packet"]["items"][10]["id"], "source_082");
        assert_eq!(batches[9]["packet"]["items"][10]["id"], "question_032");
        assert_eq!(mapping["source_000"]["target"], "original_000");
        assert_eq!(mapping["question_032"]["target"], "original_032");
    }
    #[test]
    fn bound_content_failure_only_degrades_its_own_row() {
        let ontology =
            json!({"color":{"kind":"property","additive":false,"intermediate_types":[]}});
        let source = "La caja R no es roja.";
        let subject = json!({"mention_quote":"caja R","type_quote":"caja","name":"R","owners":[],"scope":"single"});
        let raw = json!({"status":"ok","document":{"certain":[{"quote":source,"subject":subject,"time":"state","context":[],"configuration":[],"statement":{"type":"property","attribute":"color","value":{"type":"text","value":"roja"},"affirmed":false,"basis":"described_fact"}}],"readings":[],"readings_complete":true}});
        let packet = json!({"version":"codex-source-extraction-v2-packet","kind":"source","ontology":ontology,"items":[{"id":"source_000","text":source},{"id":"source_001","text":source}]});
        let mapping = json!({"source_000":{"target":"a"},"source_001":{"target":"b"}});
        let response = json!({"version":"codex-source-extraction-v2-output","packet_sha256":"h","exposure":{"only_assigned_packet":true,"manual_frames_seen":false,"gold_seen":false,"prior_outputs_seen":false,"normalizer_feedback_seen":false},"items":[{"id":"source_000","raw":raw},{"id":"source_001","raw":{}}]});
        checked_outputs(&packet, &response, "h").unwrap();
        let rows = normalized_rows(&packet, &mapping, Some(&response), None).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["normalized"]["certain_facts"][0]["positive"], false);
        assert!(rows[0]["normalization_error"].is_null());
        assert!(rows[1]["normalized"].is_null());
        assert_eq!(rows[1]["error_stage"], "schema");
        assert_eq!(rows[1]["raw"], json!({}));
        assert_eq!(rows[1]["target"], "b");
    }
    #[test]
    fn malformed_envelope_never_recovers_a_valid_subset() {
        let (batches, mapping) = packets(&bank()).unwrap();
        let packet = &batches[0]["packet"];
        let response = json!({"version":"codex-source-extraction-v2-output","packet_sha256":"hash","exposure":{"only_assigned_packet":true,"manual_frames_seen":false,"gold_seen":false,"prior_outputs_seen":false,"normalizer_feedback_seen":false},"items":[{"id":"source_000","raw":{"secret":"unreliably linked"}}]});
        let error = checked_outputs(packet, &response, "hash").unwrap_err();
        let rows = normalized_rows(packet, &mapping, Some(&response), Some(&error)).unwrap();
        assert_eq!(rows.len(), 12);
        assert!(rows.iter().all(|r| r["raw"].is_null()
            && r["normalized"].is_null()
            && r["error_stage"] == "batch_delivery"));
        let missing = normalized_rows(packet, &mapping, None, Some("ausente")).unwrap();
        assert_eq!(missing.len(), 12);
    }
}
