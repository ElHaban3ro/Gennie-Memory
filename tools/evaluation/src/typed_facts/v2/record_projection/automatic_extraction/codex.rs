//! Local Codex-task adapter. No provider journal or claim of production integration.
use super::*;
pub mod bank;
use std::collections::BTreeSet;

pub const ENTRY_VIA_GUIDANCE: &str = " Domain vocabulary extension: entry_via is the directed relation of the subject entering or passing through an explicitly stated conduit, access or route. Preserve the literal subject, owners and the value reference; never replace a network by its owner. It is a property, not an immediate supplier or upstream origin. It neither implies water_supply/network_provenance nor uniqueness, transitivity, membership or composition. Use it only for expressed entry/passage, never proximity or mere mention. Keep negation, normative permission, planning and occurrence distinct. Conditions and uncertain alternatives must restrict the operative relation/values: do not publish alternatives as independent certain facts or drop a condition. Do not assume a disjunction is exclusive/exhaustive; if this scope cannot be represented, declare the limitation rather than asserting it. Entry-route questions request entry_via; supply/origin questions retain their original attributes. No aliases or relevance decisions are added.";
fn closed(v: &Value, keys: &[&str]) -> Result<(), String> {
    let o = v.as_object().ok_or("objeto local ausente")?;
    if o.len() != keys.len() || keys.iter().any(|k| !o.contains_key(*k)) {
        return Err("campos locales ausentes o ajenos".into());
    }
    Ok(())
}
fn checked_outputs(packet: &Value, output: &Value, packet_hash: &str) -> Result<(), String> {
    closed(output, &["version", "packet_sha256", "exposure", "items"])?;
    let expected_version = match packet["version"].as_str() {
        Some("codex-source-extraction-v1-packet") => "codex-source-extraction-v1-output",
        Some("codex-source-extraction-v2-packet") => "codex-source-extraction-v2-output",
        _ => return Err("versión del paquete local desconocida".into()),
    };
    if output["version"] != expected_version
        || output["packet_sha256"] != packet_hash
        || output["exposure"]
            != json!({"only_assigned_packet":true,"manual_frames_seen":false,"gold_seen":false,"prior_outputs_seen":false,"normalizer_feedback_seen":false})
    {
        return Err("salida sin vínculo o exposición compatible".into());
    }
    let expected = list(packet, "items")?;
    let actual = list(output, "items")?;
    if actual.len() != expected.len() {
        return Err("cobertura local incompleta".into());
    }
    let mut ids = BTreeSet::new();
    for (p, a) in expected.iter().zip(actual) {
        closed(a, &["id", "raw"])?;
        if a["id"] != p["id"] || !ids.insert(text(a, "id")?) {
            return Err("ID local duplicado, ajeno o desordenado".into());
        }
    }
    Ok(())
}
// Only the published grounded-schema vocabulary is supported; this does not replace
// the historical native-DTO shape checker, semantic translator or domain parser.
fn grounded_shape(v: &Value, s: &Value, root: &Value, depth: usize) -> Result<(), String> {
    if depth > 128 {
        return Err("schema local demasiado profundo".into());
    }
    let allowed = [
        "$id",
        "$defs",
        "$ref",
        "description",
        "type",
        "anyOf",
        "enum",
        "properties",
        "required",
        "additionalProperties",
        "items",
        "maxItems",
        "minimum",
    ];
    if s.as_object()
        .is_none_or(|o| o.keys().any(|k| !allowed.contains(&k.as_str())))
    {
        return Err("keyword de schema local no soportada".into());
    }
    if let Some(r) = s.get("$ref") {
        let path = r
            .as_str()
            .and_then(|r| r.strip_prefix('#'))
            .ok_or("referencia local inválida")?;
        return grounded_shape(
            v,
            root.pointer(path).ok_or("referencia local inexistente")?,
            root,
            depth + 1,
        );
    }
    if let Some(a) = s.get("anyOf") {
        return if a
            .as_array()
            .ok_or("ramas inválidas")?
            .iter()
            .any(|b| grounded_shape(v, b, root, depth + 1).is_ok())
        {
            Ok(())
        } else {
            Err("ninguna rama del schema grounded coincide".into())
        };
    }
    let valid_type = match s["type"].as_str() {
        Some("null") => v.is_null(),
        Some("string") => v.is_string(),
        Some("boolean") => v.is_boolean(),
        Some("object") => v.is_object(),
        Some("array") => v.is_array(),
        Some("integer") => v.is_i64() || v.is_u64(),
        Some("number") => v.is_number(),
        _ => false,
    };
    if !valid_type
        || s.get("enum")
            .is_some_and(|a| a.as_array().is_none_or(|a| !a.contains(v)))
    {
        return Err("tipo/enum grounded incompatible".into());
    }
    if let Some(min) = s.get("minimum") {
        if v.as_f64().zip(min.as_f64()).is_none_or(|(v, min)| v < min) {
            return Err("número grounded por debajo del mínimo".into());
        }
    }
    if let Some(o) = v.as_object() {
        let props = s["properties"]
            .as_object()
            .ok_or("propiedades de schema ausentes")?;
        if s.get("required").is_some_and(|a| {
            a.as_array().is_none_or(|a| {
                a.iter()
                    .any(|k| k.as_str().is_none_or(|k| !o.contains_key(k)))
            })
        }) {
            return Err("campo grounded requerido ausente".into());
        }
        if s["additionalProperties"] != false || o.keys().any(|k| !props.contains_key(k)) {
            return Err("campo grounded ajeno o schema no cerrado".into());
        }
        for (k, v) in o {
            grounded_shape(v, &props[k], root, depth + 1)?;
        }
    }
    if let Some(a) = v.as_array() {
        if s["maxItems"]
            .as_u64()
            .is_some_and(|max| a.len() as u64 > max)
        {
            return Err("array grounded excedido".into());
        }
        for v in a {
            grounded_shape(v, &s["items"], root, depth + 1)?;
        }
    }
    Ok(())
}
fn entry_property_only(v: &Value) -> Result<(), String> {
    match v {
        Value::Object(o) => {
            if o.get("item_attribute").is_some_and(|v| v == "entry_via")
                || (o.get("attribute").is_some_and(|v| v == "entry_via")
                    && o.get("type").is_none_or(|v| v != "property"))
            {
                return Err("entry_via requiere property, no flujo/origen u otro statement".into());
            }
            for v in o.values() {
                entry_property_only(v)?;
            }
        }
        Value::Array(a) => {
            for v in a {
                entry_property_only(v)?;
            }
        }
        _ => (),
    }
    Ok(())
}
fn entry_packet(mut packet: Value, kind: &str) -> Result<Value, String> {
    closed(
        &packet,
        &["version", "kind", "ontology", "instructions", "items"],
    )?;
    if packet["version"] != "codex-source-extraction-v1-packet"
        || packet["kind"] != kind
        || packet["instructions"] != contract::grounded_references::instructions(kind)?
        || packet["ontology"].get("entry_via").is_some()
    {
        return Err("paquete padre incompatible".into());
    }
    packet["version"] = json!("codex-source-extraction-v2-packet");
    packet["ontology"]["entry_via"] =
        json!({"kind":"property","additive":false,"intermediate_types":[]});
    packet["instructions"] = json!(format!(
        "{}{}",
        contract::grounded_references::instructions(kind)?,
        ENTRY_VIA_GUIDANCE
    ));
    let ontology = packet["ontology"].clone();
    for item in packet["items"].as_array_mut().ok_or("faltan items padre")? {
        closed(item, &["id", "text", "output_schema"])?;
        item["output_schema"] = contract::grounded_references::schema(kind, &ontology)?;
    }
    Ok(packet)
}
fn check_entry_lineage(dir: &Path, protocol: &Value) -> Result<(), String> {
    let old = Path::new("outputs/iromem/evaluation/codex-source-extraction-v1-2026-10-21");
    if protocol["parent_protocol_sha256"] != hash(&old.join("protocol.json"))? {
        return Err("protocolo padre alterado".into());
    }
    let parent = read(&old.join("protocol.json"))?;
    for job in list(&parent, "jobs")? {
        let kind = text(job, "kind")?;
        let name = format!("{kind}-packet.json");
        if job["packet_sha256"] != hash(&old.join(&name))? {
            return Err("paquete padre alterado".into());
        }
        if read(&dir.join(&name))? != entry_packet(read(&old.join(&name))?, &kind)? {
            return Err("V2 no deriva exactamente del paquete padre".into());
        }
    }
    let preflight = read(&dir.join("preflight.json"))?;
    if preflight["protocol_sha256"] != hash(&dir.join("protocol.json"))? {
        return Err("preflight V2 alterado".into());
    }
    let adapter =
        "tools/evaluation/src/typed_facts/v2/record_projection/automatic_extraction/codex.rs";
    if preflight["source_sha256"][adapter] != protocol["adapter_sha256_before_inference"] {
        return Err("vínculo previo del adaptador alterado".into());
    }
    for name in [
        adapter,
        "tools/evaluation/src/typed_facts/v2/record_projection/automatic_extraction.rs",
        "tools/evaluation/src/main.rs",
        "docs/evaluation/codex-source-extraction-v2-protocol-2026-10-21.md",
        "docs/evaluation/entry-via-v1-contract-2026-10-21.md",
    ] {
        if preflight["source_sha256"][name]
            != hash(&dir.join("source-before-inference").join(name))?
        {
            return Err("archivo previo a inferencia alterado".into());
        }
    }
    Ok(())
}
pub fn prepare_entry(dir: &Path) -> Result<(), String> {
    let old = Path::new("outputs/iromem/evaluation/codex-source-extraction-v1-2026-10-21");
    let previous = read(&old.join("protocol.json"))?;
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    let mut jobs = Vec::new();
    for job in list(&previous, "jobs")? {
        let kind = text(job, "kind")?;
        let name = format!("{kind}-packet.json");
        if hash(&old.join(&name))? != text(job, "packet_sha256")? {
            return Err("paquete padre alterado".into());
        }
        let packet = entry_packet(read(&old.join(&name))?, &kind)?;
        crate::scoped_definition_probe::write_new(&dir.join(&name), &packet)?;
        jobs.push(json!({"kind":kind,"packet":name,"packet_sha256":hash(&dir.join(&name))?,"output":format!("{kind}-output.json"),"expected_ids":job["expected_ids"]}));
    }
    let spec = Path::new("docs/evaluation/codex-source-extraction-v2-protocol-2026-10-21.md");
    let source = Path::new(
        "tools/evaluation/src/typed_facts/v2/record_projection/automatic_extraction/codex.rs",
    );
    crate::scoped_definition_probe::write_new(
        &dir.join("protocol.json"),
        &json!({"version":"codex-source-extraction-v2-protocol","specification":spec,"specification_sha256":hash(spec)?,"entry_via_contract_sha256":hash(Path::new("docs/evaluation/entry-via-v1-contract-2026-10-21.md"))?,"adapter_sha256_before_inference":hash(source)?,"jobs":jobs,"expected_operations":8,"raw_submissions_per_job":1,"normalizer_feedback_before_submission":false,"pass_requires_every_dimension_faithful_for_each_reviewer":true,"arbitration_diagnostic_only":true,"openrouter_calls_planned":0,"codex_cost_tokens_unknown":true,"within_stage_shared_context":true,"parent_protocol_sha256":hash(&old.join("protocol.json"))?}),
    )
}
fn normalize(kind: &str, text: &str, ontology: &Value, raw: &Value) -> Value {
    let schema = match contract::grounded_references::schema(kind, ontology)
        .and_then(|s| grounded_shape(raw, &s, &s, 0))
    {
        Ok(()) => None,
        Err(e) => Some(e),
    };
    if let Some(e) = schema {
        return json!({"normalized":null,"translated":null,"normalization_error":e,"error_stage":"schema"});
    }
    let legacy = match contract::grounded_references::translate(kind, text, ontology, raw) {
        Ok(v) => v,
        Err(e) => {
            return json!({"normalized":null,"translated":null,"normalization_error":e,"error_stage":"grounded_translation"});
        }
    };
    match vocabulary(ontology).and_then(|o| decoded(kind, text, &legacy, &o)) {
        Ok(v) => {
            json!({"normalized":v,"translated":legacy,"normalization_error":null,"error_stage":null})
        }
        Err(e) => {
            json!({"normalized":null,"translated":legacy,"normalization_error":e,"error_stage":"typed_normalization"})
        }
    }
}
pub fn report(dir: &Path, output: &Path) -> Result<(), String> {
    let protocol = read(&dir.join("protocol.json"))?;
    let v2 = protocol["version"] == "codex-source-extraction-v2-protocol";
    if (protocol["version"] != "codex-source-extraction-v1-protocol" && !v2)
        || protocol["expected_operations"] != 8
        || protocol["raw_submissions_per_job"] != 1
        || protocol["normalizer_feedback_before_submission"] != false
        || protocol["pass_requires_every_dimension_faithful_for_each_reviewer"] != true
        || protocol["arbitration_diagnostic_only"] != true
        || protocol["specification_sha256"] != hash(Path::new(&text(&protocol, "specification")?))?
    {
        return Err("protocolo local incompatible".into());
    }
    if v2
        && protocol["entry_via_contract_sha256"]
            != hash(Path::new(
                "docs/evaluation/entry-via-v1-contract-2026-10-21.md",
            ))?
    {
        return Err("contrato entry_via alterado".into());
    }
    if v2 {
        check_entry_lineage(dir, &protocol)?;
    }
    let jobs = list(&protocol, "jobs")?;
    if jobs.len() != 2 {
        return Err("faltan etapas fuente/pregunta".into());
    }
    let mut rows = Vec::new();
    let mut bindings = serde_json::Map::new();
    for (job, kind) in jobs.iter().zip(["source", "question"]) {
        closed(
            job,
            &["kind", "packet", "packet_sha256", "output", "expected_ids"],
        )?;
        let name = format!("{kind}-packet.json");
        let outname = format!("{kind}-output.json");
        if job["kind"] != kind || job["packet"] != name || job["output"] != outname {
            return Err("archivo/etapa local incompatible".into());
        }
        let packet = read(&dir.join(&name))?;
        let ph = hash(&dir.join(&name))?;
        if job["packet_sha256"] != ph {
            return Err("paquete local alterado".into());
        }
        closed(
            &packet,
            &["version", "kind", "ontology", "instructions", "items"],
        )?;
        let expected_packet_version = if v2 {
            "codex-source-extraction-v2-packet"
        } else {
            "codex-source-extraction-v1-packet"
        };
        let mut instructions = contract::grounded_references::instructions(kind)?;
        if v2 {
            instructions.push_str(ENTRY_VIA_GUIDANCE);
            if packet["ontology"]["entry_via"]
                != json!({"kind":"property","additive":false,"intermediate_types":[]})
            {
                return Err("vocabulario entry_via alterado".into());
            }
        }
        if packet["version"] != expected_packet_version
            || packet["kind"] != kind
            || packet["instructions"] != instructions
        {
            return Err("contrato local alterado".into());
        }
        let items = list(&packet, "items")?;
        let expected: Vec<_> = items.iter().map(|v| v["id"].clone()).collect();
        if items.len() != 4 || job["expected_ids"] != json!(expected) {
            return Err("roster local alterado".into());
        }
        let response = read(&dir.join(&outname))?;
        checked_outputs(&packet, &response, &ph)?;
        for (item, answer) in items.iter().zip(list(&response, "items")?) {
            closed(item, &["id", "text", "output_schema"])?;
            if item["output_schema"]
                != contract::grounded_references::schema(kind, &packet["ontology"])?
            {
                return Err("esquema local alterado".into());
            }
            let entry_error = if v2 {
                entry_property_only(&answer["raw"]).err()
            } else {
                None
            };
            let mut row = match entry_error {
                Some(error) => {
                    json!({"normalized":null,"translated":null,"normalization_error":error,"error_stage":"entry_via_contract"})
                }
                None => normalize(
                    kind,
                    &text(item, "text")?,
                    &packet["ontology"],
                    &answer["raw"],
                ),
            };
            row["id"] = item["id"].clone();
            row["kind"] = json!(kind);
            row["text"] = item["text"].clone();
            row["raw"] = answer["raw"].clone();
            row["ontology"] = packet["ontology"].clone();
            rows.push(row);
        }
        bindings.insert(name, json!(ph));
        bindings.insert(outname, json!(hash(&dir.join(&text(job, "output")?))?));
    }
    let valid = rows.iter().filter(|r| r["normalized"].is_object()).count();
    let report = json!({"version":if v2 { "codex-source-extraction-v2-normalization" } else { "codex-source-extraction-v1-normalization" },"protocol_sha256":hash(&dir.join("protocol.json"))?,"bindings_sha256":bindings,"planned_operations":8,"observed_raw_operations":rows.len(),"valid_dtos":valid,"operations":rows,"semantic_fidelity_measured":false,"passes_pilot_quality":false,"passes_full_quality_goal":false,"openrouter_calls":0,"codex_cost_usd":null,"codex_tokens":null,"production_latency_measured":false,"promoted":false});
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_incomplete_duplicate_foreign_and_exposed_outputs() {
        let p =
            json!({"version":"codex-source-extraction-v1-packet","items":[{"id":"a"},{"id":"b"}]});
        let good = json!({"version":"codex-source-extraction-v1-output","packet_sha256":"h","exposure":{"only_assigned_packet":true,"manual_frames_seen":false,"gold_seen":false,"prior_outputs_seen":false,"normalizer_feedback_seen":false},"items":[{"id":"a","raw":null},{"id":"b","raw":null}]});
        assert!(checked_outputs(&p, &good, "h").is_ok());
        assert!(checked_outputs(&p, &good, "wrong").is_err());
        for items in [
            json!([]),
            json!([{"id":"a","raw":null},{"id":"a","raw":null}]),
            json!([{"id":"a","raw":null},{"id":"x","raw":null}]),
        ] {
            let mut v = good.clone();
            v["items"] = items;
            assert!(checked_outputs(&p, &v, "h").is_err());
        }
        let mut v = good;
        v["exposure"]["gold_seen"] = json!(true);
        assert!(checked_outputs(&p, &v, "h").is_err());
    }
    #[test]
    fn grounded_schema_resolves_refs_optional_fields_and_numeric_constraints() {
        let schema = json!({"type":"object","properties":{"ref":{"$ref":"#/$defs/ref"},"optional":{"type":"string"}},"required":["ref"],"additionalProperties":false,"$defs":{"ref":{"anyOf":[{"type":"null"},{"type":"object","properties":{"count":{"type":"integer","minimum":1},"items":{"type":"array","maxItems":1,"items":{"type":"string"}}},"required":["count"],"additionalProperties":false}]}}});
        assert!(grounded_shape(&json!({"ref":{"count":1}}), &schema, &schema, 0).is_ok());
        assert!(grounded_shape(&json!({"ref":null}), &schema, &schema, 0).is_ok());
        for v in [
            json!({}),
            json!({"ref":{"count":0}}),
            json!({"ref":{"count":1.5}}),
            json!({"ref":{"count":1,"extra":null}}),
            json!({"ref":{"count":1,"items":["a","b"]}}),
        ] {
            assert!(grounded_shape(&v, &schema, &schema, 0).is_err(), "{v}");
        }
        assert!(grounded_shape(&json!(1), &json!({"$ref":"#/$defs/missing"}), &schema, 0).is_err());
    }
    #[test]
    fn normalizes_grounded_source_and_question_without_inventing_positive_or_answer() {
        let o = json!({"color":{"kind":"property","additive":false,"intermediate_types":[]}});
        let subject = json!({"mention_quote":"caja R","type_quote":"caja","name":"R","owners":[],"scope":"single"});
        let source = json!({"status":"ok","document":{"certain":[{"quote":"La caja R no es roja.","subject":subject,"time":"state","context":[],"configuration":[],"statement":{"type":"property","attribute":"color","value":{"type":"text","value":"roja"},"affirmed":false,"basis":"described_fact"}}],"readings":[],"readings_complete":true}});
        let s = normalize("source", "La caja R no es roja.", &o, &source);
        assert!(s["normalization_error"].is_null(), "{s}");
        assert_eq!(s["normalized"]["certain_facts"][0]["positive"], false);
        let query = json!({"status":"ok","question":{"subject":subject,"context":[],"configuration":[],"assumptions":[],"requested":[{"quote":"¿De qué color es la caja R?","time":"state","extent":"unspecified","slot":{"type":"property","attribute":"color","basis":"any","path":"direct"}}]}});
        let q = normalize("question", "¿De qué color es la caja R?", &o, &query);
        assert!(q["normalization_error"].is_null(), "{q}");
        assert_eq!(
            q["normalized"]["frame"]["requests"][0]["attribute"],
            "color"
        );
        let mut invented = source;
        invented["document"]["certain"][0]["subject"]["name"] = json!("inventado");
        assert!(
            normalize("source", "La caja R no es roja.", &o, &invented)["normalized"].is_null()
        );
    }
    #[test]
    fn explicit_entry_relation_keeps_route_owner_and_cannot_publish_a_water_origin() {
        let ontology = json!({"entry_via":{"kind":"property","additive":false,"intermediate_types":[]},"water_supply":{"kind":"property","additive":false,"intermediate_types":[]}});
        let text = "La red del huerto Alba entra por el conducto T7.";
        let subject = json!({"mention_quote":"red del huerto Alba","type_quote":"red","name":null,"owners":[{"mention_quote":"huerto Alba","type_quote":"huerto","name":"Alba"}],"scope":"single"});
        let raw = json!({"status":"ok","document":{"certain":[{"quote":text,"subject":subject,"time":"state","context":[],"configuration":[],"statement":{"type":"property","attribute":"entry_via","value":{"type":"entity","reference":{"mention_quote":"conducto T7","type_quote":"conducto","name":"T7","owners":[],"scope":"single"}},"affirmed":true,"basis":"described_fact"}}],"readings":[],"readings_complete":true}});
        let normalized = normalize("source", text, &ontology, &raw);
        assert!(normalized["normalization_error"].is_null(), "{normalized}");
        let fact = &normalized["normalized"]["certain_facts"][0];
        assert_eq!(fact["attribute"], "entry_via");
        assert_eq!(fact["flow_scope"], "property");
        assert!(fact["only_values"].is_null());
        let identity: Value =
            serde_json::from_str(fact["subject"]["entity"].as_str().unwrap()).unwrap();
        assert_eq!(identity[2], "lr1_726564");
        assert_eq!(identity[1][0][1], "Alba");
        assert!(identity[3].is_null()); // The network is not replaced by its named owner.
        let o = vocabulary(&ontology).unwrap();
        let query = |attribute: &str, text: &str| {
            normalize(
                "question",
                text,
                &ontology,
                &json!({"status":"ok","question":{"subject":subject,"context":[],"configuration":[],"assumptions":[],"requested":[{"quote":text,"time":"state","extent":"unspecified","slot":{"type":"property","attribute":attribute,"basis":"any","path":"direct"}}]}}),
            )
        };
        let q = query("entry_via", "¿Por dónde entra la red del huerto Alba?");
        assert!(q["normalization_error"].is_null(), "{q}");
        let result = project(
            &o,
            &record_question(&q["normalized"]["frame"], &o).unwrap(),
            &normalized["normalized"],
        )
        .unwrap();
        assert_eq!(result["selected"], true);
        assert_eq!(result["assertable_facts"], 1);
        let origin = query(
            "water_supply",
            "¿Cuál es el origen del agua de la red del huerto Alba?",
        );
        let result = project(
            &o,
            &record_question(&origin["normalized"]["frame"], &o).unwrap(),
            &normalized["normalized"],
        )
        .unwrap();
        assert_eq!(result["assertable_facts"], 0); // Partial relevance remains a separate gate.
        let mut planned = raw.clone();
        let planned_text = "Se prevé que la red del huerto Alba entre por el conducto T7.";
        planned["document"]["certain"][0]["quote"] = json!(planned_text);
        planned["document"]["certain"][0]["time"] = json!("planned");
        let n = normalize("source", planned_text, &ontology, &planned);
        assert_eq!(n["normalized"]["certain_facts"][0]["modality"], "planned");
        let mut negative = raw.clone();
        let negative_text = "La red del huerto Alba no entra por el conducto T7.";
        negative["document"]["certain"][0]["quote"] = json!(negative_text);
        negative["document"]["certain"][0]["statement"]["affirmed"] = json!(false);
        let n = normalize("source", negative_text, &ontology, &negative);
        assert_eq!(n["normalized"]["certain_facts"][0]["positive"], false);
        let conditional_text = "La red del huerto Alba entra por el conducto T7 solo en invierno.";
        let mut conditional = raw.clone();
        conditional["document"]["certain"][0]["quote"] = json!(conditional_text);
        conditional["document"]["certain"][0]["context"] =
            json!([{"key":"season","value":"invierno","quote":"en invierno"}]);
        let n = normalize("source", conditional_text, &ontology, &conditional);
        assert!(n["normalization_error"].is_null(), "{n}");
        assert_eq!(
            n["normalized"]["certain_facts"][0]["context"]["season"],
            "invierno"
        );
        let uncertainty_text =
            "El registro no aclara si la red del huerto Alba entra por el conducto T7.";
        let mut possible = raw["document"]["certain"][0].clone();
        possible["quote"] = json!(uncertainty_text);
        let uncertain = json!({"status":"ok","document":{"certain":[],"readings":[{"note":"possible route, not a certain assertion","clauses":[possible]},{"note":"route is not established by this reading","clauses":[]}],"readings_complete":false,"uncertainty":{"kind":"value","quote":"no aclara"}}});
        let n = normalize("source", uncertainty_text, &ontology, &uncertain);
        assert!(n["normalization_error"].is_null(), "{n}");
        assert_eq!(n["normalized"]["certain_facts"], json!([]));
        assert_eq!(n["normalized"]["alternatives_exhaustive"], false);
        let result = project(
            &o,
            &record_question(&q["normalized"]["frame"], &o).unwrap(),
            &n["normalized"],
        )
        .unwrap();
        assert_eq!(result["assertable_facts"], 0);
    }
    #[test]
    fn entry_property_guard_checks_every_reading_and_requested_slot() {
        assert!(entry_property_only(&json!({"document":{"certain":[{"statement":{"type":"property","attribute":"entry_via"}}]}})).is_ok());
        for raw in [
            json!({"document":{"certain":[{"statement":{"type":"requirement","item_attribute":"entry_via"}}]}}),
            json!({"document":{"readings":[{"clauses":[{"statement":{"type":"flow","attribute":"entry_via"}}]}]}}),
            json!({"question":{"requested":[{"slot":{"type":"quantity","attribute":"entry_via"}}]}}),
        ] {
            assert!(entry_property_only(&raw).is_err());
        }
    }
    #[test]
    fn unsupported_and_invalid_raw_remain_failed_without_repair() {
        let o = json!({"x":{"kind":"quantity","additive":false,"intermediate_types":[]}});
        for raw in [
            json!({}),
            json!({"status":"unsupported","reason":"fuera de contrato","document":null}),
        ] {
            let r = normalize("source", "Texto.", &o, &raw);
            assert!(r["normalized"].is_null());
            assert!(r["normalization_error"].is_string());
        }
    }
}
