//! Closed local transport for a source occurrence; never resolves its open possessive.
use crate::documentary_v3::{self, context_budget as budget};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;

const LIMIT: usize = 16 * 1024 * 1024;
const DELIVERY: &str = "documentary_source_context_delivery_v1";
const MANIFEST: &str = "documentary_source_context_manifest_v1";
const REFERENCE: &str = "documentary_source_context_reference_v1";
const CONSUME: &str = "documentary_source_context_consume_v1";
const DELIVERY_V2: &str = "documentary_source_context_delivery_v2";
const REFERENCE_V2: &str = "documentary_source_context_reference_v2";
const CONSUME_V2: &str = "documentary_source_context_consume_v2";
#[derive(Clone, Copy)]
enum Protocol {
    V1,
    V2,
}
impl Protocol {
    fn delivery(self) -> &'static str {
        match self {
            Self::V1 => DELIVERY,
            Self::V2 => DELIVERY_V2,
        }
    }
    fn reference(self) -> &'static str {
        match self {
            Self::V1 => REFERENCE,
            Self::V2 => REFERENCE_V2,
        }
    }
    fn consume(self) -> &'static str {
        match self {
            Self::V1 => CONSUME,
            Self::V2 => CONSUME_V2,
        }
    }
    fn permits(self, witness: &Value) -> bool {
        witness["kind"] == "nominal_possessive_context"
            || matches!(self, Self::V2) && witness["kind"] == "source_bound_linguistic_context"
    }
}
const LIMIT_ERROR: &str = "request_context_limit";

fn charge(steps: usize, bytes: usize) -> Result<(), String> {
    budget::charge(steps, bytes)
        .then_some(())
        .ok_or_else(|| LIMIT_ERROR.into())
}
// A recursive preflight avoids allocating a stack proportional to caller arrays.
#[derive(Default)]
struct Counts {
    nodes: usize,
    owners: usize,
    evidence: usize,
}
fn preflight(v: &Value, depth: usize, counts: &mut Counts) -> Result<(), String> {
    charge(1, 0)?;
    counts.nodes += 1;
    if depth > 64 || counts.nodes > 100_000 {
        return Err(LIMIT_ERROR.into());
    }
    match v {
        Value::Array(xs) => {
            for x in xs {
                preflight(x, depth + 1, counts)?;
            }
        }
        Value::Object(m) => {
            if m.contains_key("start_byte") && m.contains_key("end_byte") && m.contains_key("quote")
            {
                counts.evidence += 1;
                if counts.evidence > 4096 {
                    return Err(LIMIT_ERROR.into());
                }
            }
            for (key, x) in m {
                charge(1, key.len())?;
                if matches!(key.as_str(), "claims" | "requests") {
                    counts.owners += x.as_array().map_or(0, Vec::len);
                    if counts.owners > 512 {
                        return Err(LIMIT_ERROR.into());
                    }
                }
                if matches!(key.as_str(), "evidence" | "boundary_evidence")
                    && x.as_array().is_some_and(|a| a.len() > 32)
                {
                    return Err(LIMIT_ERROR.into());
                }
                preflight(x, depth + 1, counts)?;
            }
        }
        Value::String(s) => charge(1, s.len())?,
        _ => (),
    }
    Ok(())
}
// Serialization writes are charged before counting, hashing, or allocating their bytes.
struct Meter {
    bytes: usize,
    hash: Option<Sha256>,
}
impl Write for Meter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        charge(1, bytes.len()).map_err(std::io::Error::other)?;
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other(LIMIT_ERROR))?;
        if let Some(hash) = &mut self.hash {
            hash.update(bytes);
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn measure<T: Serialize>(v: &T, pretty: bool) -> Result<usize, String> {
    let mut meter = Meter {
        bytes: 0,
        hash: None,
    };
    let result = if pretty {
        serde_json::to_writer_pretty(&mut meter, v)
    } else {
        serde_json::to_writer(&mut meter, v)
    };
    result.map_err(|e| format!("serialización de referencia: {e}"))?;
    Ok(meter.bytes)
}
fn reserve<T: Serialize>(v: &T, copies: usize) -> Result<(), String> {
    let size = measure(v, false)?;
    charge(1, size.checked_mul(copies).ok_or(LIMIT_ERROR)?)
}
fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 15) as usize] as char);
    }
    result
}
fn digest(v: &Value) -> Result<String, String> {
    let mut meter = Meter {
        bytes: 0,
        hash: Some(Sha256::new()),
    };
    serde_json::to_writer(&mut meter, v).map_err(|e| format!("hash de referencia: {e}"))?;
    charge(1, 64)?;
    Ok(hex_digest(&meter.hash.unwrap().finalize()))
}
fn same(a: &Value, b: &Value) -> Result<bool, String> {
    charge(
        1,
        a.as_str()
            .map_or(0, str::len)
            .saturating_add(b.as_str().map_or(0, str::len)),
    )?;
    Ok(match (a, b) {
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Ok(false);
            }
            for (a, b) in a.iter().zip(b) {
                if !same(a, b)? {
                    return Ok(false);
                }
            }
            true
        }
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return Ok(false);
            }
            for ((ka, a), (kb, b)) in a.iter().zip(b) {
                charge(1, ka.len().saturating_add(kb.len()))?;
                if ka != kb || !same(a, b)? {
                    return Ok(false);
                }
            }
            true
        }
        (Value::Array(_) | Value::Object(_), _) | (_, Value::Array(_) | Value::Object(_)) => false,
        _ => a == b,
    })
}
fn require_same(a: &Value, b: &Value) -> Result<(), String> {
    if same(a, b)? {
        Ok(())
    } else {
        Err("referencia o procedencia alterada".into())
    }
}
fn id(v: &Value) -> Result<&str, String> {
    let s = v.as_str().ok_or("ID de referencia inválido")?;
    charge(1, s.len())?;
    if s.is_empty() || s.len() > 128 {
        return Err("ID de referencia inválido".into());
    }
    Ok(s)
}
fn hash_shape(v: &Value) -> Result<(), String> {
    let s = v.as_str().ok_or("digest inválido")?;
    charge(1, s.len())?;
    if s.len() != 64
        || !s
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err("digest inválido".into());
    }
    Ok(())
}
fn rows<'a>(v: &'a Value, key: &str) -> Result<&'a [Value], String> {
    v[key]
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("falta array {key}"))
}
fn version(v: &Value, expected: &str, keys: &[&str]) -> Result<(), String> {
    super::closed(v, keys)?;
    if v["version"] != expected {
        return Err("versión de referencia inválida".into());
    }
    Ok(())
}
fn manifest_valid(m: &Value) -> Result<(), String> {
    version(m, MANIFEST, &["version", "origins", "deliveries"])?;
    let origins = rows(m, "origins")?;
    let deliveries = rows(m, "deliveries")?;
    for (table, key, fields) in [
        (
            origins,
            "origin_id",
            &["origin_id", "source_input_sha256"][..],
        ),
        (
            deliveries,
            "delivery_id",
            &["delivery_id", "origin_id", "delivery_input_sha256"][..],
        ),
    ] {
        if table.is_empty() || table.len() > 512 {
            return Err("tabla de manifiesto fuera de límite".into());
        }
        // Bounded scans avoid an unmetered caller-controlled index allocation.
        for (i, row) in table.iter().enumerate() {
            charge(1, 0)?;
            super::closed(row, fields)?;
            id(&row[key])?;
            for prior in &table[..i] {
                if same(&prior[key], &row[key])? {
                    return Err("ID duplicado en manifiesto".into());
                }
            }
            hash_shape(
                &row[if key == "origin_id" {
                    "source_input_sha256"
                } else {
                    "delivery_input_sha256"
                }],
            )?;
        }
    }
    for delivery in deliveries {
        id(&delivery["origin_id"])?;
        let mut found = false;
        for origin in origins {
            found |= same(&origin["origin_id"], &delivery["origin_id"])?;
        }
        if !found {
            return Err("origen de entrega no registrado".into());
        }
    }
    Ok(())
}
fn source_input(pair: &Value) -> Result<Value, String> {
    for v in [
        &pair["vocabulary"],
        &pair["source"]["text"],
        &pair["source"]["raw"],
    ] {
        reserve(v, 1)?;
    }
    Ok(
        json!({"vocabulary":pair["vocabulary"],"text":pair["source"]["text"],"raw":pair["source"]["raw"]}),
    )
}
fn nominal_event_witness(w: &Value, pair: &Value) -> Result<(), String> {
    super::closed(
        w,
        &[
            "id",
            "kind",
            "claim_ids",
            "request_id",
            "bindings",
            "evidence_refs",
            "qualifier_ids",
            "branch_ids",
            "missing_constraints",
            "answer_terms",
            "answer_propositions",
            "requirement_states",
            "conclusion_judgment",
            "normative_scope",
            "source_constraint",
            "source_body",
            "request_constraint",
            "source_document",
            "question_document",
            "vocabulary",
            "source_references",
            "question_references",
            "source_scopes",
            "question_scopes",
            "source_qualifiers",
            "question_qualifiers",
            "source_dependencies",
            "question_dependencies",
            "effective_source_closure",
            "effective_request_closure",
            "nominal_event_context",
        ],
    )?;
    if !documentary_v3::request_context_profile::permits(
        &pair["vocabulary"],
        "nonagentive_nominal_event_with_contexts_v1",
    ) {
        return Err("falta permiso para contexto de evento nominal".into());
    }
    require_same(&w["vocabulary"], &pair["vocabulary"])?;
    require_same(&w["source_body"], &w["source_constraint"]["body"])?;
    let body = &w["source_body"];
    let context = &w["nominal_event_context"];
    super::closed(
        context,
        &[
            "version",
            "constructor",
            "composition_id",
            "core_id",
            "region_id",
            "context_ids",
            "attachment_resolution",
            "patient_identity_proven",
            "requested_role_proven",
            "core_independently_asserted",
            "full_answer_supported",
        ],
    )?;
    if body["kind"] != "linguistic_composition"
        || body["constructor"] != "nonagentive_nominal_event_with_contexts_v1"
        || context["version"] != "documentary_nominal_event_context_v1"
        || context["constructor"] != body["constructor"]
        || context["attachment_resolution"] != "not_published"
    {
        return Err("interpretación de evento nominal inválida".into());
    }
    for flag in [
        "patient_identity_proven",
        "requested_role_proven",
        "core_independently_asserted",
        "full_answer_supported",
    ] {
        if context[flag] != false {
            return Err("el contexto nominal no resuelve identidad, papel ni respuesta".into());
        }
    }
    for (key, expected) in [
        ("composition_id", &body["composition_id"]),
        ("core_id", &body["core"]["id"]),
        ("region_id", &body["region"]["id"]),
    ] {
        id(expected)?;
        require_same(&context[key], expected)?;
    }
    let contexts = rows(body, "contexts")?;
    if !(1..=2).contains(&contexts.len()) {
        return Err("cantidad de contextos nominales inválida".into());
    }
    let context_ids = rows(context, "context_ids")?;
    if context_ids.len() != contexts.len() {
        return Err("contextos nominales incompletos".into());
    }
    for (pp, local) in contexts.iter().zip(context_ids) {
        id(&pp["id"])?;
        require_same(local, &pp["id"])?;
    }
    for key in [
        "answer_terms",
        "answer_propositions",
        "bindings",
        "requirement_states",
    ] {
        if !rows(w, key)?.is_empty() {
            return Err("el evento contextual no puede publicar respuestas ni estados".into());
        }
    }
    let missing = rows(w, "missing_constraints")?;
    for (i, debt) in missing.iter().enumerate() {
        id(debt)?;
        for previous in &missing[..i] {
            if same(previous, debt)? {
                return Err("deuda contextual duplicada".into());
            }
        }
    }
    for required in [
        "answer_slot",
        "patient_identity",
        "context_attachment",
        "requested_role",
    ] {
        let mut found = false;
        for debt in missing {
            charge(1, debt.as_str().map_or(0, str::len))?;
            found |= debt.as_str() == Some(required);
        }
        if !found {
            return Err("el testigo debe conservar todas las deudas nominales".into());
        }
    }
    for (document, kind, owner, field) in [
        (
            &w["source_document"],
            "claim",
            &w["source_constraint"]["id"],
            "effective_source_closure",
        ),
        (
            &w["question_document"],
            "request",
            &w["request_constraint"]["id"],
            "effective_request_closure",
        ),
    ] {
        let owner = id(owner)?;
        let closure = documentary_v3::lexical_clause_context::closure(document, kind, owner);
        let mut expected = Vec::new();
        for (kind, local) in closure {
            charge(1, kind.len().saturating_add(local.len()).saturating_add(64))?;
            expected.push(json!({"kind":kind,"id":local}));
        }
        let expected = Value::Array(expected);
        require_same(&w[field], &expected)?;
    }
    Ok(())
}
fn bind(delivery: &Value, manifest: &Value, protocol: Protocol) -> Result<Value, String> {
    version(
        delivery,
        protocol.delivery(),
        &["version", "origin_id", "delivery_id", "selection", "pair"],
    )?;
    id(&delivery["origin_id"])?;
    id(&delivery["delivery_id"])?;
    super::closed(&delivery["selection"], &["request_id", "claim_id"])?;
    id(&delivery["selection"]["request_id"])?;
    id(&delivery["selection"]["claim_id"])?;
    let pair = &delivery["pair"];
    super::closed(pair, &["vocabulary", "source", "question", "pair_scope"])?;
    for side in ["source", "question"] {
        super::closed(&pair[side], &["text", "raw"])?;
        if !pair[side]["text"].is_string()
            || pair[side]["raw"]["version"] != "documentary_evidence_v3"
            || pair[side]["raw"]["kind"] != side
        {
            return Err("se exige documento V3 plano íntegro".into());
        }
    }
    let source_input = source_input(pair)?;
    let source_hash = digest(&source_input)?;
    let delivery_hash = digest(delivery)?;
    let mut origin_found = false;
    for origin in rows(manifest, "origins")? {
        if same(&origin["origin_id"], &delivery["origin_id"])? {
            require_same(&origin["source_input_sha256"], &json!(source_hash))?;
            origin_found = true;
        }
    }
    let mut delivery_found = false;
    for row in rows(manifest, "deliveries")? {
        if same(&row["delivery_id"], &delivery["delivery_id"])? {
            require_same(&row["origin_id"], &delivery["origin_id"])?;
            require_same(&row["delivery_input_sha256"], &json!(delivery_hash))?;
            delivery_found = true;
        }
    }
    if !origin_found || !delivery_found {
        return Err("ocurrencia o entrega no registrada".into());
    }
    // Reserve raw/report copies before the existing validator materializes them.
    reserve(pair, 2)?;
    let source = super::side(&pair["source"], &pair["vocabulary"])?;
    let question = super::side(&pair["question"], &pair["vocabulary"])?;
    // Serialize reports only after reserving their complete representation.
    reserve(&source, 1)?;
    reserve(&question, 1)?;
    let sr = serde_json::to_value(&source).map_err(|e| e.to_string())?;
    let qr = serde_json::to_value(&question).map_err(|e| e.to_string())?;
    if sr["status"] != "complete" || qr["status"] != "complete" {
        return Err(
            "fuente y pregunta deben validar completas antes de seleccionar contexto".into(),
        );
    }
    let result = documentary_v3::classify_pair_context(
        &source,
        &question,
        &pair["vocabulary"],
        &pair["pair_scope"],
    );
    let mut selected = None;
    for request in rows(&result, "requests")? {
        if !same(&request["request_id"], &delivery["selection"]["request_id"])? {
            continue;
        }
        for witness in rows(request, "partial")? {
            charge(1, 0)?;
            if protocol.permits(witness)
                && same(
                    &witness["source_constraint"]["id"],
                    &delivery["selection"]["claim_id"],
                )?
            {
                if selected.is_some() {
                    return Err("testigo nominal posesivo múltiple".into());
                }
                selected = Some(witness);
            }
        }
    }
    let witness = selected.ok_or("no existe testigo nominal posesivo único para la selección")?;
    let sd = &pair["source"]["raw"];
    let qd = &pair["question"]["raw"];
    require_same(&witness["source_document"], sd)?;
    require_same(&witness["question_document"], qd)?;
    for (document, table, key) in [
        (sd, "claims", "source_constraint"),
        (qd, "requests", "request_constraint"),
    ] {
        let selection_key = if table == "claims" {
            "claim_id"
        } else {
            "request_id"
        };
        let mut expected = None;
        for owner in rows(document, table)? {
            if same(&owner["id"], &delivery["selection"][selection_key])? {
                expected = Some(owner);
            }
        }
        require_same(
            &witness[key],
            expected.ok_or("propietario seleccionado ausente")?,
        )?;
    }
    for (prefix, document) in [("source", sd), ("question", qd)] {
        for table in ["references", "scopes", "qualifiers", "dependencies"] {
            require_same(&witness[format!("{prefix}_{table}")], &document[table])?;
        }
    }
    for key in ["answer_terms", "answer_propositions", "bindings"] {
        if !rows(witness, key)?.is_empty() {
            return Err("el contexto no puede publicar respuestas ni bindings".into());
        }
    }
    if witness["kind"] == "source_bound_linguistic_context" {
        nominal_event_witness(witness, pair)?;
    } else {
        let missing = rows(witness, "missing_constraints")?;
        for debt in ["nominal_possessive_referent", "answer_slot"] {
            let mut found = false;
            for value in missing {
                charge(1, value.as_str().map_or(0, str::len))?;
                found |= value.as_str() == Some(debt);
            }
            if !found {
                return Err(
                    "el testigo debe conservar la deuda posesiva y el slot de respuesta".into(),
                );
            }
        }
        if witness["possessive_context"]["referent_resolution"] != "source_indeterminate"
            || witness["possessive_context"]["possessive_referent_proven"] != false
        {
            return Err("resolución posesiva no admitida".into());
        }
    }
    let manifest_hash = digest(manifest)?;
    charge(1, 1024)?; // Fixed envelope strings, digests, and repeated origin ID.
    for v in [
        &delivery["origin_id"],
        &delivery["delivery_id"],
        &delivery["selection"],
        &source_input,
        witness,
    ] {
        reserve(v, 1)?;
    }
    Ok(
        json!({"version":protocol.reference(),"origin_id":delivery["origin_id"],"delivery_id":delivery["delivery_id"],
        "manifest_sha256":manifest_hash,"source_input_sha256":source_hash,"delivery_input_sha256":delivery_hash,
        "selection":delivery["selection"],"source_input":source_input,"witness":witness,
        "interpretation":{"kind":"source_context_reference","origin_id":delivery["origin_id"]},
        "answer_terms":[],"answer_propositions":[],"bindings":[]}),
    )
}
fn execute_protocol(
    protocol: Protocol,
    mode: &str,
    input: &Value,
    manifest: &Value,
    read_bytes: usize,
) -> Result<Value, String> {
    let delivery = if mode == "consume" {
        &input["delivery"]
    } else {
        input
    };
    // A transport operation is bounded even when the caller omitted all features.
    let forced = json!({"request_context_profile":{}});
    let vocabulary = &delivery["pair"]["vocabulary"];
    let operation_vocabulary = if vocabulary.get("request_context_profile").is_some() {
        vocabulary
    } else {
        &forced
    };
    budget::with_operation(operation_vocabulary, &Value::Null, || {
        charge(1, read_bytes)?;
        let mut counts = Counts::default();
        preflight(input, 0, &mut counts)?;
        preflight(manifest, 0, &mut counts)?;
        manifest_valid(manifest)?;
        match mode {
            "bind" => bind(input, manifest, protocol),
            "consume" => {
                version(
                    input,
                    protocol.consume(),
                    &["version", "delivery", "reference"],
                )?;
                let expected = bind(&input["delivery"], manifest, protocol)?;
                require_same(&input["reference"], &expected)?;
                Ok(expected)
            }
            _ => Err("operación de referencia desconocida: use bind o consume".into()),
        }
    })?
}
#[cfg(test)]
fn execute(
    mode: &str,
    input: &Value,
    manifest: &Value,
    read_bytes: usize,
) -> Result<Value, String> {
    execute_protocol(Protocol::V1, mode, input, manifest, read_bytes)
}
fn bounded(path: &Path, allowance: usize) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("entrada de referencia: {e}"))?;
    let mut bytes = Vec::new();
    file.take(allowance as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("lectura de referencia: {e}"))?;
    if bytes.len() > allowance {
        return Err(LIMIT_ERROR.into());
    }
    Ok(bytes)
}
pub fn run(mode: &str, input: &Path, manifest: &Path, output: &Path) -> Result<(), String> {
    run_protocol(Protocol::V1, mode, input, manifest, output)
}
pub fn run_v2(mode: &str, input: &Path, manifest: &Path, output: &Path) -> Result<(), String> {
    run_protocol(Protocol::V2, mode, input, manifest, output)
}
fn run_protocol(
    protocol: Protocol,
    mode: &str,
    input: &Path,
    manifest: &Path,
    output: &Path,
) -> Result<(), String> {
    let input_bytes = bounded(input, LIMIT)?;
    let manifest_bytes = bounded(manifest, LIMIT - input_bytes.len())?;
    let input = super::context_json::parse_strict(&input_bytes)?;
    let manifest = super::context_json::parse_strict(&manifest_bytes)?;
    // Keep output reservation in the same operation as validation and reclassification.
    let delivery = if mode == "consume" {
        &input["delivery"]
    } else {
        &input
    };
    let forced = json!({"request_context_profile":{}});
    let vocabulary = &delivery["pair"]["vocabulary"];
    let vocabulary = if vocabulary.get("request_context_profile").is_some() {
        vocabulary
    } else {
        &forced
    };
    let report = budget::with_operation(vocabulary, &Value::Null, || {
        let report = execute_protocol(
            protocol,
            mode,
            &input,
            &manifest,
            input_bytes.len() + manifest_bytes.len(),
        )?;
        let bytes = measure(&report, true)?;
        charge(1, bytes.checked_add(1).ok_or(LIMIT_ERROR)?)?;
        Ok::<_, String>(report)
    })??;
    super::write_report(output, &report)
}

#[cfg(test)]
mod tests;
