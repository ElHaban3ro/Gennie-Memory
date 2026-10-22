//! Versioned evaluation adapter; literal evidence and a class registry are not semantic proofs.
use crate::documentary_extraction_contract::grounded_references;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Class {
    id: String,
    definition: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryWire {
    version: String,
    classes: Vec<Class>,
}
pub struct Registry {
    definitions: BTreeMap<String, String>,
    hash: String,
}
pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn key(s: &str) -> bool {
    s.len() <= 64 && condition_key(s)
}
fn condition_key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 96
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
impl Registry {
    pub fn parse(bytes: &[u8], expected: &str) -> Result<Self, String> {
        let hash = sha256(bytes);
        if hash != expected {
            return Err("registro de clases modificado".into());
        }
        let wire: RegistryWire =
            serde_json::from_slice(bytes).map_err(|e| format!("registro inválido: {e}"))?;
        if wire.version != "reference-classes-v1"
            || wire.classes.is_empty()
            || wire.classes.len() > 64
        {
            return Err("versión o tamaño del registro incorrecto".into());
        }
        let mut definitions = BTreeMap::new();
        for c in wire.classes {
            if !key(&c.id)
                || c.definition.trim().is_empty()
                || c.definition.len() > 4096
                || definitions.insert(c.id, c.definition).is_some()
            {
                return Err("clase duplicada o definición inválida".into());
            }
        }
        Ok(Self { definitions, hash })
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassBinding {
    class_id: String,
    quote: String,
    equivalence: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Condition {
    key: String,
    value: String,
    quote: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Support {
    holder: String,
    quote: String,
    bearer_quote: String,
    owner_quote: String,
    positive: bool,
    role: String,
    basis: String,
    relation_modality: String,
    applicability: String,
    context: Vec<Condition>,
    configuration: Vec<Condition>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Owner {
    mention_quote: String,
    type_quote: Option<String>,
    name: String,
    class_binding: Option<ClassBinding>,
    ownership_support: Support,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    mention_quote: String,
    type_quote: Option<String>,
    name: Option<String>,
    class_binding: Option<ClassBinding>,
    owners: Vec<Owner>,
    scope: String,
    count: Option<u32>,
    member_of: Option<Box<Value>>,
    part_of: Option<Box<Value>>,
}
fn literal(s: &str, text: &str) -> Result<(), String> {
    if s.trim().is_empty() || s.len() > 4096 || !text.contains(s) {
        Err("cita de referencia ausente o no literal".into())
    } else {
        Ok(())
    }
}
fn class_literal(s: &str, text: &str) -> Result<(), String> {
    literal(s, text)?;
    let continuation =
        |c: char| c.is_alphanumeric() || c == '_' || ('\u{300}'..='\u{36f}').contains(&c);
    let full = text.match_indices(s).any(|(i, _)| {
        text[..i]
            .chars()
            .next_back()
            .is_none_or(|c| !continuation(c))
            && text[i + s.len()..]
                .chars()
                .next()
                .is_none_or(|c| !continuation(c))
    });
    if full {
        Ok(())
    } else {
        Err("clase citada mediante recorte de una palabra".into())
    }
}
fn class_key(
    noun: Option<&str>,
    binding: Option<&ClassBinding>,
    mention: &str,
    registry: &Registry,
) -> Result<String, String> {
    if let Some(n) = noun {
        class_literal(n, mention)?;
    }
    match (noun, binding) {
        (Some(n), Some(b)) => {
            if b.equivalence != "exact"
                || b.quote != n
                || !registry.definitions.contains_key(&b.class_id)
            {
                return Err("vínculo de clase desconocido, parcial o no exacto".into());
            }
            Ok(format!("dc1_{}", b.class_id))
        }
        (Some(n), None) => {
            if n.len() > 46 {
                return Err("clase léxica demasiado larga para el DTO actual".into());
            }
            Ok(format!(
                "dl1_{}",
                n.as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            ))
        }
        (None, None) => Ok("du1_unknown".into()),
        (None, Some(_)) => Err("clase semántica sin clase declarada".into()),
    }
}
fn scope_map(v: &Value) -> Result<BTreeMap<String, String>, String> {
    let list = v.as_array().ok_or("faltan condiciones del uso")?;
    let mut out = BTreeMap::new();
    for c in list {
        let k = c["key"].as_str().ok_or("condición sin clave")?;
        let val = c["value"].as_str().ok_or("condición sin valor")?;
        if !condition_key(k)
            || val.trim().is_empty()
            || out.insert(k.to_owned(), val.to_owned()).is_some()
        {
            return Err("condición duplicada o inválida".into());
        }
    }
    Ok(out)
}
fn applicable(conditions: &[Condition], site: &Value, quote: &str) -> Result<(), String> {
    if conditions.len() > 12 {
        return Err("demasiadas condiciones del vínculo".into());
    }
    let scope = scope_map(site)?;
    let mut seen = BTreeSet::new();
    for c in conditions {
        literal(&c.quote, quote)?;
        if !condition_key(&c.key) || !seen.insert(&c.key) || scope.get(&c.key) != Some(&c.value) {
            return Err("aplicabilidad del vínculo no conservada en el uso".into());
        }
    }
    Ok(())
}
fn declared(v: &Value) -> Result<(), String> {
    if v.get("type_quote").is_none() || v.get("class_binding").is_none() {
        return Err("falta declaración explícita de clase o binding".into());
    }
    Ok(())
}
fn reference(
    v: &Value,
    quote: &str,
    site: &Value,
    registry: &Registry,
    depth: usize,
) -> Result<Value, String> {
    if depth > 4 {
        return Err("referencia demasiado profunda".into());
    }
    declared(v)?;
    let r: Reference = serde_json::from_value(v.clone())
        .map_err(|e| format!("distributed_reference_shape:{e}"))?;
    if r.owners.len() > 8 {
        return Err("demasiados propietarios".into());
    }
    literal(&r.mention_quote, quote)?;
    if let Some(name) = &r.name {
        literal(name, &r.mention_quote)?;
    }
    if r.type_quote.is_none() && r.name.is_none() {
        return Err("referencia anónima no identificable".into());
    }
    let kind = class_key(
        r.type_quote.as_deref(),
        r.class_binding.as_ref(),
        &r.mention_quote,
        registry,
    )?;
    let mut owners = Vec::new();
    for (i, o) in r.owners.iter().enumerate() {
        declared(&v["owners"][i])?;
        literal(&o.mention_quote, quote)?;
        literal(&o.name, &o.mention_quote)?;
        let support = &o.ownership_support;
        let (holder, bearer) = match r.owners.get(i + 1) {
            Some(inner) => (format!("owner_{}", i + 1), inner.mention_quote.as_str()),
            None => ("subject".into(), r.mention_quote.as_str()),
        };
        if support.holder != holder
            || support.bearer_quote != bearer
            || support.owner_quote != o.mention_quote
        {
            return Err("arista de propietario no corresponde a su eslabón".into());
        }
        literal(&support.quote, quote)?;
        literal(bearer, &support.quote)?;
        literal(&o.mention_quote, &support.quote)?;
        if !support.positive
            || support.role != "fixed_identity"
            || support.relation_modality != "state"
            || !["described_fact", "observed_fact", "actual_assignment"]
                .contains(&support.basis.as_str())
        {
            return Err("vínculo no afirmado como identidad fija de estado".into());
        }
        match support.applicability.as_str() {
            "identifier_only" => {}
            "bounded" if support.context.iter().any(|c| c.key == "identity_interval") => {}
            _ => {
                return Err(
                    "aplicabilidad de propiedad ausente, desconocida o sin intervalo".into(),
                );
            }
        }
        applicable(&support.context, &site["context"], &support.quote)?;
        applicable(
            &support.configuration,
            &site["configuration"],
            &support.quote,
        )?;
        let owner_kind = class_key(
            o.type_quote.as_deref(),
            o.class_binding.as_ref(),
            &o.mention_quote,
            registry,
        )?;
        owners.push(json!({"kind":owner_kind,"name":o.name}));
    }
    Ok(
        json!({"kind":kind,"name":r.name,"owners":owners,"scope":r.scope,"count":r.count,"member_of":r.member_of.as_ref().map(|x|reference(x,quote,site,registry,depth+1)).transpose()?,"part_of":r.part_of.as_ref().map(|x|reference(x,quote,site,registry,depth+1)).transpose()?}),
    )
}
pub fn translate(
    kind: &str,
    text: &str,
    ontology: &Value,
    raw: &Value,
    registry: &Registry,
) -> Result<Value, String> {
    let legacy = grounded_references::translate_with_references(
        kind,
        text,
        ontology,
        raw,
        &|v, quote, site| reference(v, quote, site, registry, 0),
    )?;
    Ok(
        json!({"legacy":legacy,"reference_evidence":raw,"registry_sha256":registry.hash,"semantic_assertions_verified":false,"unspecified_applicability_is_not_unlimited_validity":true,"identifier_scope_is_not_unbounded_ownership":true}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn registry() -> Registry {
        let bytes = include_bytes!("../../../tests/fixtures/reference-classes-v1.json");
        Registry::parse(bytes, &sha256(bytes)).unwrap()
    }
    fn r(mention: &str, noun: Option<&str>, name: Option<&str>) -> Value {
        json!({"mention_quote":mention,"type_quote":noun,"name":name,"class_binding":null,"owners":[],"scope":"single"})
    }
    fn site() -> Value {
        json!({"time":"state","context":[],"configuration":[]})
    }
    fn owned() -> (String, Value) {
        let text = "El formulario es verde. Pertenece al pedido E8 de Nácar.".to_owned();
        let mut v = r("El formulario", Some("formulario"), None);
        v["owners"] = json!([
            {"mention_quote":"Nácar","type_quote":null,"name":"Nácar","class_binding":null,"ownership_support":{"holder":"owner_1","quote":text,"bearer_quote":"pedido E8 de Nácar","owner_quote":"Nácar","positive":true,"role":"fixed_identity","basis":"described_fact","relation_modality":"state","applicability":"identifier_only","context":[],"configuration":[]}},
            {"mention_quote":"pedido E8 de Nácar","type_quote":"pedido","name":"E8","class_binding":null,"ownership_support":{"holder":"subject","quote":text,"bearer_quote":"El formulario","owner_quote":"pedido E8 de Nácar","positive":true,"role":"fixed_identity","basis":"described_fact","relation_modality":"state","applicability":"identifier_only","context":[],"configuration":[]}}
        ]);
        (text, v)
    }
    #[test]
    fn distributed_chain_keeps_phrase_order_and_old_validator_rejects_same_external_owner() {
        let (text, v) = owned();
        let before = v.clone();
        let out = reference(&v, &text, &site(), &registry(), 0).unwrap();
        assert_eq!(out["owners"][0]["name"], "Nácar");
        assert_eq!(out["owners"][1]["name"], "E8");
        assert_eq!(v, before);
        let mut old = v.clone();
        old.as_object_mut().unwrap().remove("class_binding");
        for owner in old["owners"].as_array_mut().unwrap() {
            owner.as_object_mut().unwrap().remove("class_binding");
            owner.as_object_mut().unwrap().remove("ownership_support");
        }
        let old_raw = json!({"status":"ok","document":{"certain":[{"quote":text,"subject":old,"time":"state","context":[],"configuration":[],"statement":{"type":"property","attribute":"color","value":{"type":"text","value":"verde"},"affirmed":true,"basis":"described_fact"}}],"readings":[],"readings_complete":true}});
        assert!(
            grounded_references::translate(
                "source",
                &text,
                &json!({"color":{"kind":"property"}}),
                &old_raw
            )
            .is_err()
        );

        let mut wrong = v.clone();
        wrong["owners"][0]["ownership_support"]["holder"] = json!("subject");
        assert!(reference(&wrong, &text, &site(), &registry(), 0).is_err());
        wrong = v.clone();
        wrong["owners"][1]["ownership_support"]["bearer_quote"] = json!("Nácar");
        assert!(reference(&wrong, &text, &site(), &registry(), 0).is_err());
    }
    #[test]
    fn exact_class_binding_is_explicit_and_lexical_unknown_namespaces_do_not_collide() {
        let reg = registry();
        let b = |noun: &str| ClassBinding {
            class_id: "sensor".into(),
            quote: noun.into(),
            equivalence: "exact".into(),
        };
        assert_eq!(
            class_key(Some("sensor"), Some(&b("sensor")), "sensor", &reg).unwrap(),
            class_key(Some("sensores"), Some(&b("sensores")), "sensores", &reg).unwrap()
        );
        assert_ne!(
            class_key(Some("sensor"), None, "sensor", &reg).unwrap(),
            class_key(Some("sensor"), Some(&b("sensor")), "sensor", &reg).unwrap()
        );
        assert_ne!(
            class_key(None, None, "X", &reg).unwrap(),
            class_key(Some("du1_unknown"), None, "du1_unknown", &reg).unwrap()
        );
        assert!(class_key(Some("sensor"), Some(&b("sensor")), "sensores", &reg).is_err());
        assert!(class_key(None, Some(&b("sensor")), "sensor", &reg).is_err());
        let mut narrow = b("sensor");
        narrow.equivalence = "subclass".into();
        assert!(class_key(Some("sensor"), Some(&narrow), "sensor", &reg).is_err());
    }
    #[test]
    fn false_exact_equivalence_can_pass_shape_and_is_not_semantic_certificate() {
        let b = ClassBinding {
            class_id: "sensor".into(),
            quote: "sensores de presión".into(),
            equivalence: "exact".into(),
        };
        // A deliberately false semantic assertion: the adapter checks evidence, not NLI.
        assert_eq!(
            class_key(
                Some("sensores de presión"),
                Some(&b),
                "sensores de presión",
                &registry()
            )
            .unwrap(),
            "dc1_sensor"
        );
    }
    #[test]
    fn registry_tamper_unknown_ids_duplicates_and_missing_scope_are_rejected() {
        let bytes = include_bytes!("../../../tests/fixtures/reference-classes-v1.json");
        assert!(Registry::parse(bytes, "wrong").is_err());
        let d=json!({"version":"reference-classes-v1","classes":[{"id":"x","definition":"a"},{"id":"x","definition":"b"}]}).to_string();
        assert!(Registry::parse(d.as_bytes(), &sha256(d.as_bytes())).is_err());
        let mut x = r("sensor S", Some("sensor"), Some("S"));
        x["class_binding"] = json!({"class_id":"injected","quote":"sensor","equivalence":"exact"});
        assert!(reference(&x, "sensor S", &site(), &registry(), 0).is_err());
        let (text, mut x) = owned();
        x["owners"][0]["ownership_support"]
            .as_object_mut()
            .unwrap()
            .remove("applicability");
        assert!(reference(&x, &text, &site(), &registry(), 0).is_err());
    }
    #[test]
    fn relation_state_does_not_have_to_equal_event_modality_but_interval_must_survive() {
        let (mut text, mut v) = owned();
        text.push_str(" Durante ayer se realizó el pesaje.");
        for o in v["owners"].as_array_mut().unwrap() {
            o["ownership_support"]["quote"] = json!(text);
            o["ownership_support"]["applicability"] = json!("bounded");
            o["ownership_support"]["context"] =
                json!([{"key":"identity_interval","value":"ayer","quote":"ayer"}]);
        }
        let good = json!({"time":"occurred","context":[{"key":"identity_interval","value":"ayer","quote":"ayer"}],"configuration":[]});
        assert!(reference(&v, &text, &good, &registry(), 0).is_ok());
        assert!(reference(&v, &text, &site(), &registry(), 0).is_err());
        let mut other = good;
        other["context"][0]["value"] = json!("hoy");
        assert!(reference(&v, &text, &other, &registry(), 0).is_err());
    }
    #[test]
    fn owner_support_preserves_legacy_condition_keys_up_to_96_bytes() {
        let (text, mut v) = owned();
        for length in [65, 96] {
            let k = "a".repeat(length);
            let use_site = json!({"time":"state","context":[{"key":k,"value":"verde","quote":"verde"}],"configuration":[]});
            assert!(reference(&v, &text, &use_site, &registry(), 0).is_ok());
            v["owners"][1]["ownership_support"]["context"] =
                json!([{"key":k,"value":"verde","quote":"verde"}]);
            assert!(reference(&v, &text, &use_site, &registry(), 0).is_ok());
            v["owners"][1]["ownership_support"]["context"] = json!([]);
        }
        let too_long = json!({"time":"state","context":[{"key":"a".repeat(97),"value":"verde","quote":"verde"}],"configuration":[]});
        assert!(reference(&v, &text, &too_long, &registry(), 0).is_err());
    }
    #[test]
    fn negated_planned_normative_corrigible_and_unknown_links_do_not_fix_identity() {
        let (text, v) = owned();
        for (k, bad) in [
            ("positive", json!(false)),
            ("role", json!("corrigible_assumption")),
            ("relation_modality", json!("planned")),
            ("basis", json!("permission_rule")),
            ("applicability", json!("unknown")),
        ] {
            let mut x = v.clone();
            x["owners"][1]["ownership_support"][k] = bad;
            assert!(reference(&x, &text, &site(), &registry(), 0).is_err());
        }
    }
}
