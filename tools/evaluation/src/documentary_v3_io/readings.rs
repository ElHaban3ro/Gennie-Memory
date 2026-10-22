//! Opt-in finite interpretative readings. Documentary alternatives remain V3 claims.
use crate::documentary_v3;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

#[path = "existential_projection.rs"]
mod existential_projection;

const VERSION: &str = "documentary_reading_set_v1";
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn fields(v: &Value, keys: &[&str]) -> bool {
    v.as_object()
        .is_some_and(|m| m.len() == keys.len() && keys.iter().all(|k| m.contains_key(*k)))
}
fn quote(v: &Value, text: &str) -> bool {
    if !fields(v, &["start_byte", "end_byte", "quote"]) {
        return false;
    }
    match (
        v["start_byte"].as_u64(),
        v["end_byte"].as_u64(),
        v["quote"].as_str(),
    ) {
        (Some(a), Some(b), Some(q)) if a < b && b <= text.len() as u64 => {
            text.get(a as usize..b as usize) == Some(q)
        }
        _ => false,
    }
}
fn envelope(raw: &Value) -> bool {
    raw["version"] == VERSION
}

/// Legacy documents retain their exact validator report. Envelopes retain every raw leaf.
pub fn validate(raw: &Value, text: &str, vocabulary: &Value) -> Value {
    documentary_v3::context_budget::with_operation(vocabulary,raw,||validate_inner(raw,text,vocabulary)).unwrap_or_else(|reason|json!({"version":"documentary_reading_validation_v1","status":"invalid","document":raw,"readings":[],"issues":[{"code":reason,"path":""}],"semantic_fidelity_proven":false}))
}
fn validate_inner(raw: &Value, text: &str, vocabulary: &Value) -> Value {
    if !envelope(raw) {
        return serde_json::to_value(documentary_v3::validate(raw, text, vocabulary)).unwrap();
    }
    let mut issues = vec![];
    let mut invalid = false;
    let mut issue = |code: &str, path: &str| {
        invalid = true;
        issues.push(json!({"code":code,"path":path}));
    };
    if !fields(
        raw,
        &[
            "version",
            "kind",
            "text_sha256",
            "vocabulary_sha256",
            "domain_namespace",
            "closure",
            "readings",
            "ambiguities",
            "unresolved_dimensions",
        ],
    ) {
        issue("invalid_shape", "");
    }
    if !matches!(raw["kind"].as_str(), Some("source" | "question"))
        || !raw["domain_namespace"].is_string()
        || raw["domain_namespace"] == ""
    {
        issue("invalid_shape", "/kind_or_namespace");
    }
    if raw["text_sha256"] != hash(text.as_bytes()) {
        issue("invalid_quote", "/text_sha256");
    }
    if raw["vocabulary_sha256"] != hash(&serde_json::to_vec(vocabulary).unwrap_or_default()) {
        issue("unknown_vocabulary", "/vocabulary_sha256");
    }
    if !matches!(raw["closure"].as_str(), Some("closed" | "open")) {
        issue("invalid_shape", "/closure");
    }
    if !raw["readings"].is_array() || !(1..=8).contains(&arr(raw, "readings").len()) {
        issue("reading_limit", "/readings");
    }
    if !raw["ambiguities"].is_array() || !raw["unresolved_dimensions"].is_array() {
        issue("invalid_shape", "/ambiguities");
    }
    let mut ids = BTreeSet::new();
    let mut reports = vec![];
    // A limit failure never truncates to a favorable subset.
    if arr(raw, "readings").len() <= 8 {
        for (i, reading) in arr(raw, "readings").iter().enumerate() {
            let id = reading["id"].as_str().unwrap_or("");
            if !fields(reading, &["id", "document"]) || id.is_empty() || !ids.insert(id.to_owned())
            {
                issue("invalid_reading", &format!("/readings/{i}"));
            }
            let d = &reading["document"];
            if d["version"] != "documentary_evidence_v3"
                || [
                    "kind",
                    "text_sha256",
                    "vocabulary_sha256",
                    "domain_namespace",
                ]
                .iter()
                .any(|k| d[*k] != raw[*k])
            {
                issue(
                    "reading_identity_mismatch",
                    &format!("/readings/{i}/document"),
                );
            }
            reports.push(json!({"id":id,"report":documentary_v3::validate(d,text,vocabulary)}));
        }
    }
    let mut ambiguity_ids = BTreeSet::new();
    for (i, a) in arr(raw, "ambiguities").iter().enumerate() {
        let candidates = arr(a, "candidate_reading_ids");
        let unique: BTreeSet<_> = candidates.iter().filter_map(Value::as_str).collect();
        if !fields(
            a,
            &[
                "id",
                "span",
                "dimension",
                "candidate_reading_ids",
                "closure",
                "evidence",
            ],
        ) || a["id"]
            .as_str()
            .is_none_or(|s| s.is_empty() || !ambiguity_ids.insert(s.to_owned()))
            || !quote(&a["span"], text)
            || !matches!(
                a["dimension"].as_str(),
                Some("modal_basis" | "relation" | "reference" | "temporal_aspect")
            )
            || !matches!(a["closure"].as_str(), Some("closed" | "open"))
            || candidates.is_empty()
            || unique.len() != candidates.len()
            || unique.iter().any(|id| !ids.contains(*id))
            || arr(a, "evidence").is_empty()
            || arr(a, "evidence").iter().any(|e| !quote(e, text))
        {
            issue("invalid_ambiguity", &format!("/ambiguities/{i}"));
        }
    }
    if arr(raw, "readings").len() > 1 && arr(raw, "ambiguities").is_empty() {
        issue("missing_ambiguity", "/ambiguities");
    }
    let mut unresolved_ids = BTreeSet::new();
    for (i, u) in arr(raw, "unresolved_dimensions").iter().enumerate() {
        if !fields(
            u,
            &["id", "span", "dimension", "reason", "affected_targets"],
        ) || !quote(&u["span"], text)
            || u["id"]
                .as_str()
                .is_none_or(|s| s.is_empty() || !unresolved_ids.insert(s.to_owned()))
            || !u["dimension"].is_string()
            || !u["reason"].is_string()
            || !u["affected_targets"].is_array()
            || arr(u, "affected_targets").iter().any(|x| !x.is_string())
        {
            issue(
                "invalid_unresolved_dimension",
                &format!("/unresolved_dimensions/{i}"),
            );
        }
    }
    if reports.iter().any(|r| r["report"]["status"] == "invalid") {
        invalid = true;
    }
    let partial = raw["closure"] != "closed"
        || !arr(raw, "unresolved_dimensions").is_empty()
        || arr(raw, "ambiguities")
            .iter()
            .any(|a| a["closure"] != "closed")
        || reports.iter().any(|r| r["report"]["status"] != "complete");
    json!({"version":"documentary_reading_validation_v1","status":if invalid {"invalid"} else if partial {"partial"} else {"complete"},"document":raw,"readings":reports,"issues":issues,"semantic_fidelity_proven":false})
}
fn leaves<'a>(raw: &'a Value) -> Vec<(&'a str, &'a Value)> {
    if envelope(raw) {
        arr(raw, "readings")
            .iter()
            .filter_map(|r| Some((r["id"].as_str()?, &r["document"])))
            .collect()
    } else {
        vec![("literal", raw)]
    }
}

// Identity is conservative: explicit names with a unique, unbound root reference.
// Anonymous, scope-bound or unresolved identities stay conditional, never matched by local ID.
fn reference_key(doc: &Value, id: &str, seen: &mut BTreeSet<String>) -> Option<Value> {
    if documentary_v3::nominal_possessive::reference_blocked(doc, id) {
        return None;
    }
    if documentary_v3::context_budget::active() {
        return contextual_reference_key(doc, id, seen);
    }
    reference_key_inner(doc, id, seen, &mut 4096, &mut (4 * 1024 * 1024))
}
fn contextual_reference_key(doc: &Value, id: &str, seen: &mut BTreeSet<String>) -> Option<Value> {
    if seen.len() >= 64
        || !seen.insert(id.to_owned())
        || !documentary_v3::context_budget::charge(1, 0)
    {
        return None;
    }
    let mut component = BTreeSet::from([id.to_owned()]);
    loop {
        let before = component.len();
        for edge in arr(doc, "identity") {
            if !documentary_v3::context_budget::charge(1, 0) {
                return None;
            }
            if edge["kind"] == "same" {
                let (a, b) = (edge["left"].as_str()?, edge["right"].as_str()?);
                if component.contains(a) {
                    component.insert(b.to_owned());
                }
                if component.contains(b) {
                    component.insert(a.to_owned());
                }
            }
        }
        if component.len() == before {
            break;
        }
        if component.len() > 512 {
            return None;
        }
    }
    if arr(doc, "identity").iter().any(|edge| {
        edge["kind"] == "different"
            && component.contains(edge["left"].as_str().unwrap_or(""))
            && component.contains(edge["right"].as_str().unwrap_or(""))
    }) {
        return None;
    }
    let mut keys = Vec::new();
    for member in &component {
        let r = arr(doc, "references").iter().find(|r| r["id"] == *member)?;
        let name = r["name"]["quote"].as_str()?;
        if name.is_empty()
            || !r["binding_scope_id"].is_null()
            || !r["anaphora"].is_null()
            || arr(doc, "references").iter().any(|x| {
                x["name"]["quote"] == name && !component.contains(x["id"].as_str().unwrap_or(""))
            })
            || scope_key(doc, &r["scope_id"], &mut BTreeSet::new()).is_none()
        {
            return None;
        }
        let mut links = Vec::new();
        for link in arr(r, "links") {
            let target = link["target"].as_str()?;
            if component.contains(target) {
                return None;
            }
            let target_key = if documentary_v3::context_budget::nominal_enabled()
                && link["kind"] == documentary_v3::nominal_de::KIND
            {
                // This descriptor is nested under its named owner incidence.
                // It never creates a top-level anonymous reference identity.
                documentary_v3::nominal_de::descriptor_key(doc, target)?
            } else {
                contextual_reference_key(doc, target, seen)?
            };
            links.push(json!({"kind":link["kind"],"target":target_key}));
        }
        links.sort_by_key(Value::to_string);
        keys.push(json!({"namespace":doc["domain_namespace"],"text_sha256":doc["text_sha256"],"name_anchor":r["name"],"mention_anchor":r["mention"],"name":name,"class":r["declared_class"]["quote"],"type":r["semantic_type"]["type_id"],"denotation":r["denotation"],"links":links}));
    }
    keys.sort_by_key(Value::to_string);
    let bytes = serde_json::to_vec(&keys).ok()?;
    if !documentary_v3::context_budget::charge(0, bytes.len()) {
        return None;
    }
    seen.remove(id);
    Some(json!({"identity_sha256":hash(&bytes)}))
}
fn reference_key_inner(
    doc: &Value,
    id: &str,
    seen: &mut BTreeSet<String>,
    nodes: &mut usize,
    bytes: &mut usize,
) -> Option<Value> {
    if !documentary_v3::context_budget::charge(1, 0) || *nodes == 0 {
        return None;
    }
    *nodes -= 1;
    if seen.len() >= 64 || !seen.insert(id.to_owned()) {
        return None;
    }
    let r = arr(doc, "references").iter().find(|r| r["id"] == id)?;
    let name = r["name"]["quote"].as_str()?;
    let cost = [
        &r["name"]["quote"],
        &r["mention"]["quote"],
        &r["declared_class"]["quote"],
        &r["semantic_type"]["type_id"],
    ]
    .iter()
    .fold(256usize, |n, v| {
        n.saturating_add(v.as_str().map(str::len).unwrap_or(0))
    })
    .saturating_add(arr(r, "links").len().saturating_mul(128));
    if cost > *bytes {
        return None;
    }
    *bytes -= cost;

    if name.is_empty()
        || !r["binding_scope_id"].is_null()
        || !r["anaphora"].is_null()
        || arr(doc, "references")
            .iter()
            .filter(|x| x["name"]["quote"] == name)
            .count()
            != 1
        || !arr(doc, "scopes").iter().any(|s| {
            s["id"] == r["scope_id"] && s["kind"] == "document" && s["parent_id"].is_null()
        })
    {
        return None;
    }
    let mut links = vec![];
    for link in arr(r, "links") {
        links.push(
            json!({"kind":link["kind"],"target":reference_key_inner(doc,link["target"].as_str()?,seen,nodes,bytes)?}),
        );
    }
    seen.remove(id);
    let key = json!({"namespace":doc["domain_namespace"],"text_sha256":doc["text_sha256"],"name_anchor":r["name"],"mention_anchor":r["mention"],"name":name,"class":r["declared_class"]["quote"],"type":r["semantic_type"]["type_id"],"denotation":r["denotation"],"links":links});
    Some(json!({"identity_sha256":hash(&serde_json::to_vec(&key).ok()?)}))
}
fn canonical(v: &Value, doc: &Value) -> Option<Value> {
    canonical_inner(v, doc, 0, &mut BTreeSet::new(), &mut 4096)
}
fn canonical_inner(
    v: &Value,
    doc: &Value,
    depth: usize,
    active: &mut BTreeSet<String>,
    remaining: &mut usize,
) -> Option<Value> {
    if !documentary_v3::context_budget::charge(1, 0) || *remaining == 0 {
        return None;
    }
    *remaining -= 1;
    if depth > 64 {
        return None;
    }
    if let Some(xs) = v.as_array() {
        return Some(Value::Array(
            xs.iter()
                .map(|x| canonical_inner(x, doc, depth + 1, active, remaining))
                .collect::<Option<Vec<_>>>()?,
        ));
    }
    if !v.is_object() {
        return Some(v.clone());
    }
    let prepared;
    let v = if v["kind"] == documentary_v3::linguistic_composition::KIND {
        prepared = documentary_v3::linguistic_composition::canonical_ast(v)?;
        &prepared
    } else {
        v
    };
    let m = v.as_object()?;
    let kind = v["kind"].as_str().unwrap_or("");
    if kind == "ref" || kind == "described_ref" {
        let identity = reference_key(doc, v["id"].as_str()?, &mut BTreeSet::new())?;
        if kind == "ref" {
            return Some(json!({"kind":"ref","identity":identity}));
        }
        return Some(
            json!({"kind":"described_ref","identity":identity,"constraints":canonical_inner(&v["constraints"],doc,depth+1,active,remaining)?,"resolution":v["resolution"]}),
        );
    }
    let mut out = serde_json::Map::new();
    for (k, x) in m {
        if matches!(
            k.as_str(),
            "evidence" | "evidence_refs" | "start_byte" | "end_byte"
        ) {
            continue;
        }
        if k == "qualifier_ids" {
            let mut qs = vec![];
            for id in x.as_array()? {
                let qid = id.as_str()?;
                if !active.insert(qid.to_owned()) {
                    return None;
                }
                let q = arr(doc, "qualifiers").iter().find(|q| q["id"] == *id)?;
                let mut q = q.clone();
                q.as_object_mut()?.remove("id");
                q.as_object_mut()?.remove("scope_id");
                // Nested qualifier references need an explicit graph canonicalizer.
                if !arr(&q, "qualifier_ids").is_empty() {
                    return None;
                }
                qs.push(canonical_inner(&q, doc, depth + 1, active, remaining)?);
                active.remove(qid);
            }
            if documentary_v3::context_budget::active() {
                qs.sort_by_key(Value::to_string);
            }
            out.insert(k.clone(), Value::Array(qs));
            continue;
        }
        if matches!(
            k.as_str(),
            "normative_scope"
                | "subject"
                | "boundary_subject"
                | "event_ref"
                | "rule_ref"
                | "beneficiary"
        ) && x.is_string()
        {
            out.insert(
                k.clone(),
                reference_key(doc, x.as_str()?, &mut BTreeSet::new())?,
            );
            continue;
        }
        out.insert(
            k.clone(),
            canonical_inner(x, doc, depth + 1, active, remaining)?,
        );
    }
    let result = Value::Object(out);
    if documentary_v3::context_budget::active()
        && !documentary_v3::context_budget::serialized_bytes(&result)
            .is_some_and(|n| documentary_v3::context_budget::charge(0, n))
    {
        return None;
    }
    Some(result)
}
// Context-local IDs are resolved before erasure; unresolved graph keys never compare equal.
fn context_bindings_key(owner: &Value, doc: &Value) -> Option<Value> {
    let Some(bindings) = owner.get("context_bindings") else {
        return Some(Value::Null);
    };
    let mut result = Vec::new();
    for binding in bindings.as_array()? {
        let mut value = binding.clone();
        value.as_object_mut()?.remove("id");
        result.push(canonical(&value, doc)?);
    }
    result.sort_by_key(Value::to_string);
    Some(json!(result))
}
fn scope_key(doc: &Value, scope: &Value, seen: &mut BTreeSet<String>) -> Option<Value> {
    if !documentary_v3::context_budget::charge(1, 0) {
        return None;
    }
    let id = scope.as_str()?;
    if seen.len() >= 64 || !seen.insert(id.to_owned()) {
        return None;
    }
    let node = arr(doc, "scopes").iter().find(|n| n["id"] == id)?;
    // Quantified/bound branch canonicalization needs its own proof. Never erase it.
    if node["kind"] != "document"
        || !node["parent_id"].is_null()
        || !node["binding"].is_null()
        || !node["branch_id"].is_null()
    {
        return None;
    }
    Some(json!({"kind":"document","namespace":doc["domain_namespace"]}))
}
fn context_owner_key(owner: &Value, doc: &Value) -> Option<Value> {
    let mut node = owner.clone();
    let object = node.as_object_mut()?;
    object.remove("id");
    object.remove("evidence");
    object.remove("context_bindings");
    object.remove("scope_id");
    let mut premises = Vec::new();
    if let Some(ids) = object.remove("premise_ids") {
        for id in ids.as_array()? {
            let premise = arr(doc, "claims").iter().find(|c| c["id"] == *id)?;
            // Cyclic/compound premise ownership is retained as ineligible.
            if premise.get("context_bindings").is_some() {
                return None;
            }
            premises.push(json!({"body":canonical(&premise["body"],doc)?,"scope":scope_key(doc,&premise["scope_id"],&mut BTreeSet::new())?,"qualifiers":canonical(&json!({"qualifier_ids":premise["qualifier_ids"]}),doc)?}));
        }
    }
    premises.sort_by_key(Value::to_string);
    Some(
        json!({"owner":canonical(&node,doc)?,"scope":scope_key(doc,&owner["scope_id"],&mut BTreeSet::new())?,"contexts":context_bindings_key(owner,doc)?,"premises":premises}),
    )
}
fn context_witness_key(w: &Value, source: &Value, question: &Value) -> Option<Value> {
    // Full owner copies and their scopes, context and qualifier operands are mandatory.
    let request = arr(question, "requests")
        .iter()
        .find(|r| r["id"] == w["request_id"])?;
    let mut claims = Vec::new();
    for id in arr(w, "claim_ids") {
        let claim = arr(source, "claims").iter().find(|c| c["id"] == *id)?;
        claims.push(context_owner_key(claim, source)?);
    }
    if claims.is_empty() {
        return None;
    }
    claims.sort_by_key(Value::to_string);
    let proof = w.get("request_context_witness")?;
    let mut mapping = Vec::new();
    for pair in arr(proof, "reference_map") {
        mapping.push(json!({"source":reference_key(source,pair["source"].as_str()?,&mut BTreeSet::new())?,"question":reference_key(question,pair["question"].as_str()?,&mut BTreeSet::new())?}));
    }
    mapping.sort_by_key(Value::to_string);
    let control = &proof["control"];
    let control = if control.is_null() {
        Value::Null
    } else {
        json!({"question_participant":canonical(&control["question_participant"],question)?,"source_participant":canonical(&control["source_participant"],source)?,"source_actor":canonical(&control["source_actor"],source)?,"substitution":control["substitution"]})
    };
    Some(
        json!({"kind":w["kind"],"request":context_owner_key(request,question)?,"claims":claims,"reference_map":mapping,"control":control,"missing_constraints":canonical(&w["missing_constraints"],source)?}),
    )
}
fn request_signature(
    request: &Value,
    row: &Value,
    source: &Value,
    question: &Value,
) -> Option<Value> {
    if row["coverage"] != "complete" || !arr(row, "conflicts").is_empty() {
        return None;
    }
    let contextual = request.get("context_bindings").is_some()
        || ["support", "refutation"]
            .iter()
            .flat_map(|kind| arr(row, kind))
            .flat_map(|w| arr(w, "claim_ids"))
            .any(|id| {
                arr(source, "claims")
                    .iter()
                    .any(|c| c["id"] == *id && c.get("context_bindings").is_some())
            });
    let request_context = if contextual {
        Some(context_owner_key(request, question)?)
    } else {
        None
    };
    let generic_constraint =
        request["mode"] == "constraint" && request["basis_requirement"]["kind"] == "any";
    let target = canonical(&request["target"], question)?;
    let mut signatures = vec![];
    for relation in ["support", "refutation"] {
        for w in arr(row, relation) {
            let mut constraint = w["source_constraint"].clone();
            // Basis is not erased from transport: only the generic textual constraint
            // key abstracts it, while every full witness remains in the matrix.
            if generic_constraint {
                if let Some(m) = constraint.as_object_mut() {
                    m.remove("basis");
                }
            }
            let mut claim_contents = vec![];
            if !generic_constraint || contextual {
                for cid in arr(w, "claim_ids") {
                    let claim = arr(source, "claims").iter().find(|c| c["id"] == *cid)?;
                    // Claim ID and quotes are provenance, not semantic equality.
                    let mut content = json!({"body":canonical(&claim["body"],source)?,"qualifiers":canonical(&json!({"qualifier_ids":claim["qualifier_ids"]}),source)?});
                    if contextual {
                        content["context_owner"] = context_owner_key(claim, source)?;
                    }
                    claim_contents.push(content);
                }
            }
            let mut signature = json!({"relation":relation,"target":target,"slot":request["slot"],"extent":request["required_answer_extent"],"basis_requirement":request["basis_requirement"],"mode":request["mode"],"unit":request["unit"],"class_filter":canonical(&request["class_filter"],question)?,"normative_scope_filter":canonical(&json!({"normative_scope":request["normative_scope_filter"]}),question)?,"answer_terms":canonical(&w["answer_terms"],source)?,"answer_propositions":canonical(&w["answer_propositions"],source)?,"source_constraint":canonical(&constraint,source)?,"claims":claim_contents,"qualifiers":canonical(&json!({"qualifier_ids":w["qualifier_ids"]}),source)?,"normative_scope":canonical(&json!({"normative_scope":w["normative_scope"]}),source)?});
            if let Some(key) = &request_context {
                signature["request_context_owner"] = key.clone();
                signature["context_witness"] = context_witness_key(w, source, question)?;
            }
            signatures.push(signature);
        }
    }
    if signatures.is_empty() {
        return None;
    }
    signatures.sort_by_key(Value::to_string);
    signatures.dedup();
    Some(json!(signatures))
}
fn classify(input: &Value) -> Value {
    classify_with_context(input, false)
}
fn classify_with_context(input: &Value, context_v2: bool) -> Value {
    documentary_v3::context_budget::with_operation(&input["vocabulary"], input, || {
        classify_with_context_inner(input, context_v2)
    })
    .unwrap_or_else(|reason| context_limit_report(input, &reason))
}
fn context_limit_report(input: &Value, reason: &str) -> Value {
    json!({"version":"documentary_request_context_limit_v1","source":input["source"],"question":input["question"],"result":{"status":"ineligible","requests":[],"complete_answer":false,"requires_review":true,"selected_claim_ids":[],"issues":[{"code":reason}]},"automatic_extraction_measured":false,"semantic_fidelity_proven":false,"production_latency_measured":false,"new_provider_calls":0})
}
fn classify_with_context_inner(input: &Value, context_v2: bool) -> Value {
    let vocab = &input["vocabulary"];
    let sr = &input["source"]["raw"];
    let qr = &input["question"]["raw"];
    let st = input["source"]["text"].as_str().unwrap_or("");
    let qt = input["question"]["text"].as_str().unwrap_or("");
    let sv = validate(sr, st, vocab);
    let qv = validate(qr, qt, vocab);
    let sl = leaves(sr);
    let ql = leaves(qr);
    let mut eligible = sv["status"] == "complete"
        && qv["status"] == "complete"
        && sl.len() <= 8
        && ql.len() <= 8
        && sr["kind"] == "source"
        && qr["kind"] == "question";
    let ids: Vec<Value> = ql
        .first()
        .map(|(_, d)| arr(d, "requests").iter().map(|r| r["id"].clone()).collect())
        .unwrap_or_default();
    if ids.is_empty()
        || ql.iter().any(|(_, d)| {
            arr(d, "requests")
                .iter()
                .map(|r| r["id"].clone())
                .collect::<Vec<_>>()
                != ids
        })
    {
        eligible = false;
    }
    if vocab.get("request_context_profile").is_some() {
        let inventories: Option<Vec<_>> = ql
            .iter()
            .map(|(_, d)| {
                arr(d, "requests")
                    .iter()
                    .map(|r| context_owner_key(r, d))
                    .collect::<Option<Vec<_>>>()
            })
            .collect();
        if inventories
            .as_ref()
            .is_none_or(|xs| xs.iter().any(|x| Some(x) != xs.first()))
        {
            eligible = false;
        }
    }
    let mut matrix = vec![];
    if sl.len() <= 8 && ql.len() <= 8 {
        for (sid, sd) in &sl {
            for (qid, qd) in &ql {
                if !documentary_v3::context_budget::charge(1, 0) {
                    return context_limit_report(input, "request_context_limit");
                }
                let source = documentary_v3::validate(sd, st, vocab);
                let question = documentary_v3::validate(qd, qt, vocab);
                let result = if context_v2 {
                    documentary_v3::classify_pair_context(
                        &source,
                        &question,
                        vocab,
                        &input["pair_scope"],
                    )
                } else {
                    documentary_v3::classify_pair(&source, &question, vocab, &input["pair_scope"])
                };
                matrix.push(
                    json!({"source_reading_id":sid,"question_reading_id":qid,"result":result}),
                );
            }
        }
    }
    let mut requests = vec![];
    for (index, id) in ids.iter().enumerate() {
        let mut keys = vec![];
        let mut all = eligible;
        let mut any = false;
        for cell in &matrix {
            let sd = sl
                .iter()
                .find(|(rid, _)| Some(*rid) == cell["source_reading_id"].as_str())
                .map(|(_, d)| *d)
                .unwrap();
            let qd = ql
                .iter()
                .find(|(rid, _)| Some(*rid) == cell["question_reading_id"].as_str())
                .map(|(_, d)| *d)
                .unwrap();
            let row = arr(&cell["result"], "requests").get(index);
            if let Some(row) = row.filter(|r| r["request_id"] == *id) {
                any |= !arr(row, "support").is_empty()
                    || !arr(row, "refutation").is_empty()
                    || !arr(row, "partial").is_empty();
                if let Some(key) = arr(qd, "requests")
                    .get(index)
                    .and_then(|r| request_signature(r, row, sd, qd))
                {
                    keys.push(key);
                } else {
                    all = false;
                }
            } else {
                all = false;
            }
        }
        all &= !matrix.is_empty()
            && keys.len() == matrix.len()
            && keys.iter().all(|k| Some(k) == keys.first());
        requests.push(json!({"request_id":id,"coverage":if all {"complete"} else if any {"conditional_reading"} else {"none"},"complete_answer":all,"evidence_by_reading":matrix.iter().map(|c|json!({"source_reading_id":c["source_reading_id"],"question_reading_id":c["question_reading_id"],"request":arr(&c["result"],"requests").get(index).cloned().unwrap_or(Value::Null)})).collect::<Vec<_>>() }));
    }
    let complete =
        eligible && !requests.is_empty() && requests.iter().all(|r| r["complete_answer"] == true);
    let mut report = json!({"version":"documentary_reading_pair_report_v1","source":sv,"question":qv,"result":{"status":if eligible {"valid"} else {"ineligible"},"requests":requests,"reading_matrix":matrix,"complete_answer":complete,"requires_review":!complete},"automatic_extraction_measured":false,"semantic_fidelity_proven":false,"production_latency_measured":false,"new_provider_calls":0});
    existential_projection::upgrade(input, &mut report, eligible);
    report
}

pub fn run(input: &Path, output: &Path, paired: bool) -> Result<(), String> {
    run_with_context(input, output, paired, false)
}
pub fn run_context(input: &Path, output: &Path) -> Result<(), String> {
    run_with_context(input, output, true, true)
}
fn run_with_context(
    input: &Path,
    output: &Path,
    paired: bool,
    context_v2: bool,
) -> Result<(), String> {
    let bytes = super::packet::read_limited(input)?;
    let input = super::context_json::parse(&bytes)?;
    let report = documentary_v3::context_budget::with_operation(&input["vocabulary"], &input, || -> Result<Value,String> {
    Ok(if paired {
        super::closed(&input, &["vocabulary", "source", "question", "pair_scope"])?;
        if !context_v2 && input["vocabulary"].get("request_context_profile").is_some() { return Err("el contrato request_context V1 exige la entrada contextual explícita".into()); }
        for side in ["source", "question"] {
            super::closed(&input[side], &["text", "raw"])?;
            if !input[side]["text"].is_string() {
                return Err("falta texto original".into());
            }
        }
        if !envelope(&input["source"]["raw"]) && !envelope(&input["question"]["raw"]) && !existential_projection::enabled(&input["vocabulary"]) {
            let source = super::side(&input["source"], &input["vocabulary"])?;
            let question = super::side(&input["question"], &input["vocabulary"])?;
            let result = if context_v2 {
                documentary_v3::classify_pair_context(
                    &source,
                    &question,
                    &input["vocabulary"],
                    &input["pair_scope"],
                )
            } else {
                documentary_v3::classify_pair(
                    &source,
                    &question,
                    &input["vocabulary"],
                    &input["pair_scope"],
                )
            };
            json!({"version":"documentary-v3-local-pair-report-v1","result":result,"source":source,"question":question,"automatic_extraction_measured":false,"semantic_fidelity_proven":false,"production_latency_measured":false,"new_provider_calls":0})
        } else {
            if context_v2 {
                classify_with_context(&input, true)
            } else {
                classify(&input)
            }
        }
    } else {
        super::closed(&input, &["vocabulary", "text", "raw"])?;
        validate(
            &input["raw"],
            input["text"].as_str().ok_or("falta texto original")?,
            &input["vocabulary"],
        )
    })
    }).unwrap_or_else(|reason| Ok(context_limit_report(&input,&reason)))?;
    super::write_report(output, &report)
}

#[cfg(test)]
include!("readings_tests.rs");

#[cfg(test)]
mod relative_bound_signature_tests {
    use super::*;
    #[test]
    fn relative_bound_signature_uses_boundary_identity_and_preserves_operator() {
        let mut d = json!({"domain_namespace":"relative-signature-control","text_sha256":"same-source","scopes":[{"id":"root","kind":"document","parent_id":null}],"references":[{"id":"device","scope_id":"root","name":{"quote":"Vela"},"mention":{"quote":"Vela"},"binding_scope_id":null,"anaphora":null,"denotation":"entity","links":[]}],"qualifiers":[{"id":"bound","scope_id":"root","kind":"relative_nonoccurrence_bound","body":{"relation":"until","boundary_subject":"device","boundary_action":"switch_off"},"resolution":"known","evidence":[]}]});
        let a = canonical(&json!({"qualifier_ids":["bound"]}), &d).unwrap();
        d["references"][0]["id"] = json!("renamed-device");
        d["qualifiers"][0]["body"]["boundary_subject"] = json!("renamed-device");
        d["qualifiers"][0]["id"] = json!("renamed-bound");
        assert_eq!(
            a,
            canonical(&json!({"qualifier_ids":["renamed-bound"]}), &d).unwrap()
        );
        d["references"][0]["name"]["quote"] = json!("Senda");
        assert_ne!(
            a,
            canonical(&json!({"qualifier_ids":["renamed-bound"]}), &d).unwrap()
        );
        d["qualifiers"][0]["body"]["boundary_subject"] = json!("dangling");
        assert!(canonical(&json!({"qualifier_ids":["renamed-bound"]}), &d).is_none());
    }
    #[test]
    fn nominal_complement_signature_preserves_kind_owner_and_local_lexical_use() {
        let mut d = json!({"domain_namespace":"nominal-signature-control","text_sha256":"same-source","scopes":[{"id":"root","kind":"document","parent_id":null,"binding":null,"branch_id":null}],"references":[{"id":"head","scope_id":"root","name":{"quote":"Berta"},"mention":{"quote":"Berta de papel"},"declared_class":{"quote":"bandeja"},"semantic_type":null,"binding_scope_id":null,"anaphora":null,"denotation":"entity","links":[{"kind":"nominal_complement_de","target":"tail","evidence":[]}]},{"id":"tail","scope_id":"root","name":null,"mention":{"quote":"papel"},"declared_class":{"quote":"papel"},"semantic_type":null,"binding_scope_id":null,"anaphora":null,"denotation":"entity","links":[]}],"identity":[],"dependencies":[]});
        documentary_v3::context_budget::with_operation(
            &json!({"request_context_profile":{"version":"documentary_request_context_v1","features":["nominal_complement_de"]}}),
            &json!({}),
            || {
                let key = reference_key(&d, "head", &mut BTreeSet::new()).unwrap();
                assert!(reference_key(&d, "tail", &mut BTreeSet::new()).is_none());
                d["references"][0]["id"] = json!("renamed-head");
                d["references"][1]["id"] = json!("renamed-tail");
                d["references"][0]["links"][0]["target"] = json!("renamed-tail");
                assert_eq!(
                    key,
                    reference_key(&d, "renamed-head", &mut BTreeSet::new()).unwrap()
                );
                d["references"][1]["declared_class"]["quote"] = json!("metal");
                assert_ne!(
                    key,
                    reference_key(&d, "renamed-head", &mut BTreeSet::new()).unwrap()
                );
                d["references"][0]["links"][0]["kind"] = json!("associated_with");
                assert!(reference_key(&d, "renamed-head", &mut BTreeSet::new()).is_none());
            },
        )
        .unwrap();
    }
}

#[cfg(test)]
mod until_composition_canonical_tests {
    use super::*;
    #[test]
    fn linguistic_composition_io_signature_uses_normalized_internal_incidence_ids() {
        let source: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/documentary-until-composition-v1/source.json"
        ))
        .unwrap();
        let original = &source["claims"][0]["body"];
        let mut renamed = original.clone();
        for path in [
            "/core/id",
            "/core/operand/id",
            "/core/operand/object/id",
            "/operators/0/id",
            "/operators/0/boundary/id",
            "/operators/0/boundary/subject/id",
        ] {
            *renamed.pointer_mut(path).unwrap() = json!(format!("local-renamed-{path}"));
        }
        renamed["operators"][0]["attachment"]["region_root_id"] = renamed["core"]["id"].clone();
        let left = canonical(original, &source)
            .expect("complete source owner has a canonical reference anchor");
        let right = canonical(&renamed, &source).unwrap();
        assert_eq!(left, right);
        assert_eq!(
            left["operators"][0]["attachment"]["resolution"],
            Value::Null
        );
        assert_eq!(
            left["operators"][0]["boundary"]["subject"]["resolution"],
            Value::Null
        );
    }
}
