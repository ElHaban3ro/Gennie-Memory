//! Grounded lexical references for evaluation; no claim of proven coreference or entailment.
use super::typed_clauses;
use serde::Deserialize;
use serde_json::{Value, json};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Owner {
    mention_quote: String,
    type_quote: Option<String>,
    name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    mention_quote: String,
    type_quote: Option<String>,
    name: Option<String>,
    owners: Vec<Owner>,
    scope: String,
    count: Option<u32>,
    member_of: Option<Box<Reference>>,
    part_of: Option<Box<Reference>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TypeBinding {
    type_id: String,
    quote: String,
}
fn literal(s: &str, text: &str) -> Result<(), String> {
    if s.trim().is_empty() || s.len() > 4096 || !text.contains(s) {
        return Err("mención/cita de referencia ausente o no literal".into());
    }
    Ok(())
}
fn lexical_kind(s: Option<&str>) -> Result<String, String> {
    match s {
        None => Ok("lr1_unknown".into()),
        Some(s) => {
            // Injective UTF-8 encoding: no hash, case-folding, stemming, stripping or truncation.
            if s.trim().is_empty() || s.len() > 46 {
                return Err("clase léxica vacía o demasiado larga para el contrato V5".into());
            }
            Ok(format!(
                "lr1_{}",
                s.as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            ))
        }
    }
}
fn reference(r: &Reference, quote: &str, depth: usize) -> Result<Value, String> {
    if depth > 4 || r.owners.len() > 8 {
        return Err("referencia demasiado profunda o con demasiados propietarios".into());
    }
    literal(&r.mention_quote, quote)?;
    if let Some(t) = &r.type_quote {
        literal(t, &r.mention_quote)?;
    }
    if let Some(n) = &r.name {
        literal(n, &r.mention_quote)?;
    }
    if r.type_quote.is_none() && r.name.is_none() {
        return Err("referencia sin nombre ni clase declarada no identificable".into());
    }
    let owners = r
        .owners
        .iter()
        .map(|o| {
            literal(&o.mention_quote, &r.mention_quote)?;
            literal(&o.name, &o.mention_quote)?;
            if let Some(t) = &o.type_quote {
                literal(t, &o.mention_quote)?;
            }
            Ok(json!({"kind":lexical_kind(o.type_quote.as_deref())?,"name":o.name}))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(
        json!({"kind":lexical_kind(r.type_quote.as_deref())?,"name":r.name,"owners":owners,"scope":r.scope,"count":r.count,"member_of":r.member_of.as_ref().map(|x|reference(x,quote,depth+1)).transpose()?,"part_of":r.part_of.as_ref().map(|x|reference(x,quote,depth+1)).transpose()?}),
    )
}
fn required_type_quotes(v: &Value, depth: usize) -> Result<(), String> {
    if depth > 4 || v.get("type_quote").is_none() {
        return Err("falta declaración explícita de clase o referencia demasiado profunda".into());
    }
    for owner in v["owners"].as_array().ok_or("faltan propietarios")? {
        if owner.get("type_quote").is_none() {
            return Err("falta declaración explícita de clase del propietario".into());
        }
    }
    for key in ["member_of", "part_of"] {
        if !v[key].is_null() {
            required_type_quotes(&v[key], depth + 1)?;
        }
    }
    Ok(())
}
fn rewritten_reference(v: &Value, quote: &str) -> Result<Value, String> {
    required_type_quotes(v, 0)?;
    let r: Reference =
        serde_json::from_value(v.clone()).map_err(|e| format!("grounded_reference_shape:{e}"))?;
    reference(&r, quote, 0)
}
fn binding(v: &Value, quote: &str) -> Result<Option<String>, String> {
    if v.is_null() {
        return Ok(None);
    }
    let b: TypeBinding =
        serde_json::from_value(v.clone()).map_err(|e| format!("grounded_type_shape:{e}"))?;
    literal(&b.quote, quote)?;
    if b.type_id.is_empty()
        || b.type_id.len() > 96
        || !b
            .type_id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
    {
        return Err("tipo ontológico técnico inválido".into());
    }
    Ok(Some(b.type_id))
}
pub type ReferenceRewriter<'a> = dyn Fn(&Value, &str, &Value) -> Result<Value, String> + 'a;
fn term(
    v: &mut Value,
    quote: &str,
    scope: &Value,
    rewrite: &ReferenceRewriter<'_>,
) -> Result<Option<String>, String> {
    if v["type"] != "entity" {
        return Ok(None);
    }
    let r = &v["reference"];
    let mention = r["mention_quote"]
        .as_str()
        .ok_or("falta mención de entidad")?;
    let type_id = binding(&v["ontology_type"], mention)?;
    if type_id.is_some() && r["type_quote"].is_null() {
        return Err("tipo ontológico sin clase declarada".into());
    }
    if type_id.is_some() {
        literal(
            r["type_quote"].as_str().ok_or("falta clase declarada")?,
            v["ontology_type"]["quote"]
                .as_str()
                .ok_or("falta cita de tipo")?,
        )?;
    }
    let rewritten = rewrite(r, quote, scope)?;
    v.as_object_mut()
        .ok_or("término inválido")?
        .remove("ontology_type");
    v["reference"] = rewritten;
    Ok(type_id)
}
fn rewrite_clause(
    v: &mut Value,
    text: &str,
    rewrite: &ReferenceRewriter<'_>,
) -> Result<Option<Value>, String> {
    let scope = v.clone();
    let quote = v["quote"]
        .as_str()
        .ok_or("falta cita de cláusula")?
        .to_owned();
    literal(&quote, text)?;
    v["subject"] = rewrite(&v["subject"], &quote, &scope)?;
    let s = &mut v["statement"];
    match s["type"].as_str() {
        Some("property" | "assignment") => {
            Ok(Some(json!(term(&mut s["value"], &quote, &scope, rewrite)?)))
        }
        Some("flow" | "exhaustive") => {
            let closed = s["type"] == "exhaustive" || s["exhaustive"] == true;
            let values = s["values"].as_array_mut().ok_or("faltan términos")?;
            let mut types = Vec::new();
            for value in values {
                types.push(term(value, &quote, &scope, rewrite)?);
            }
            if closed && types.iter().any(Option::is_some) {
                return Err("tipos por miembro de conjunto no representables en V5".into());
            }
            Ok(if closed {
                None
            } else {
                types.first().map(|t| json!(t))
            })
        }
        _ => Ok(None),
    }
}
fn fix_fact_type(fact: &mut Value, data_type: Option<Value>) {
    if let Some(t) = data_type {
        if !fact["datum"].is_null() {
            fact["datum"]["value_type"] = t;
        }
    }
}
pub fn translate(kind: &str, text: &str, ontology: &Value, raw: &Value) -> Result<Value, String> {
    translate_with_references(kind, text, ontology, raw, &|v, quote, _| {
        rewritten_reference(v, quote)
    })
}
/// Explicit extension seam; legacy callers retain their original reference validator.
pub fn translate_with_references(
    kind: &str,
    text: &str,
    ontology: &Value,
    raw: &Value,
    rewrite: &ReferenceRewriter<'_>,
) -> Result<Value, String> {
    if raw["status"] != "ok" {
        return typed_clauses::translate(kind, text, ontology, raw);
    }
    let mut adapted = raw.clone();
    let mut certain_types = Vec::new();
    let mut reading_types = Vec::new();
    let mut premise_types = Vec::new();
    match kind {
        "source" => {
            for c in adapted["document"]["certain"]
                .as_array_mut()
                .ok_or("faltan cláusulas ciertas")?
            {
                certain_types.push(rewrite_clause(c, text, rewrite)?);
            }
            for r in adapted["document"]["readings"]
                .as_array_mut()
                .ok_or("faltan lecturas")?
            {
                let mut types = Vec::new();
                for c in r["clauses"]
                    .as_array_mut()
                    .ok_or("faltan cláusulas de lectura")?
                {
                    types.push(rewrite_clause(c, text, rewrite)?);
                }
                reading_types.push(types);
            }
        }
        "question" => {
            let q = &mut adapted["question"];
            let scope = q.clone();
            q["subject"] = rewrite(&q["subject"], text, &scope)?;
            for c in q["assumptions"].as_array_mut().ok_or("faltan premisas")? {
                premise_types.push(rewrite_clause(c, text, rewrite)?);
            }
            for request in q["requested"].as_array_mut().ok_or("faltan solicitudes")? {
                let quote = request["quote"]
                    .as_str()
                    .ok_or("falta cita de solicitud")?
                    .to_owned();
                literal(&quote, text)?;
                let slot = &mut request["slot"];
                if slot["type"] == "property" {
                    if slot.get("value_type").is_some() {
                        return Err("usar type_filter con cita, no value_type libre".into());
                    }
                    let t = binding(&slot["type_filter"], &quote)?;
                    slot.as_object_mut()
                        .ok_or("solicitud inválida")?
                        .remove("type_filter");
                    let value_type = if !slot["value_filter"].is_null() {
                        term(&mut slot["value_filter"], &quote, &scope, rewrite)?
                    } else {
                        None
                    };
                    if t.is_some() && value_type.is_some() && t != value_type {
                        return Err("tipos semánticos de solicitud y valor incompatibles".into());
                    }
                    slot["value_type"] = json!(t.or(value_type));
                }
            }
        }
        _ => return Err("tipo de tarea desconocido".into()),
    }
    let mut out = typed_clauses::translate(kind, text, ontology, &adapted)?;
    if kind == "source" {
        for (fact, t) in out["document"]["certain_facts"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .zip(certain_types)
        {
            fix_fact_type(fact, t);
        }
        for (reading, types) in out["document"]["readings"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .zip(reading_types)
        {
            for (fact, t) in reading["facts"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .zip(types)
            {
                fix_fact_type(fact, t);
            }
        }
    } else {
        for (fact, t) in out["frame"]["premises"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .zip(premise_types)
        {
            fix_fact_type(fact, t);
        }
    }
    Ok(out)
}
fn optional(v: Value) -> Value {
    json!({"anyOf":[v,{"type":"null"}]})
}
fn object(fields: Value, optional_fields: &[&str]) -> Value {
    let required = fields
        .as_object()
        .unwrap()
        .keys()
        .filter(|k| !optional_fields.contains(&k.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    json!({"type":"object","properties":fields,"required":required,"additionalProperties":false})
}
pub fn schema(kind: &str, ontology: &Value) -> Result<Value, String> {
    let mut s = typed_clauses::schema(kind, ontology)?;
    let string = json!({"type":"string"});
    let props = s["$defs"]["reference"]["properties"]
        .as_object_mut()
        .unwrap();
    props.remove("kind");
    props.insert("mention_quote".into(), string.clone());
    props.insert("type_quote".into(), optional(string.clone()));
    let required = s["$defs"]["reference"]["required"].as_array_mut().unwrap();
    required.retain(|v| v != "kind");
    required.extend([json!("mention_quote"), json!("type_quote")]);
    s["$defs"]["owner"] = object(
        json!({"mention_quote":string,"type_quote":optional(string.clone()),"name":string}),
        &[],
    );
    s["$defs"]["type_binding"] = object(json!({"type_id":string,"quote":string}), &[]);
    s["$defs"]["term"]["anyOf"][1]["properties"]["ontology_type"] =
        optional(json!({"$ref":"#/$defs/type_binding"}));
    let slot = &mut s["$defs"]["request"]["properties"]["slot"]["anyOf"][0];
    slot["properties"]
        .as_object_mut()
        .unwrap()
        .remove("value_type");
    slot["properties"]["type_filter"] = optional(json!({"$ref":"#/$defs/type_binding"}));
    s["$id"] = json!(format!("iromem-grounded-references-v1-{kind}"));
    Ok(s)
}
pub fn instructions(kind: &str) -> Result<String, String> {
    let mut guide = typed_clauses::instructions(kind)?;
    let start = guide
        .find("A reference separates")
        .ok_or("guía base incompatible")?;
    let end = guide
        .find("Context holds fixed conditions")
        .ok_or("guía base incompatible")?;
    guide.replace_range(start..end,"A reference has mention_quote copied literally from the same clause, containing the full referring noun phrase and its restrictions; name is only its explicit proper name/ID, not the whole description. type_quote is the literal noun/class phrase that classifies THIS referent inside that mention; use null if its class is not declared. Do not copy the class of another referent or invent organization/company/text/single/property as a class. Each owner has its own literal mention_quote, type_quote or null, and proper name, outermost first. A cited noun is evidence to audit, not proof of its attribution. Preserve the actual bearer of a requirement, group vs individual, owner, parent and fixed cardinality. scope single/named_group/all_members/fixed_group/class_reference is independent of class words. Anonymous references need an explicit class; absent classes never form a global anonymous class. Use member_of only for explicit membership and part_of only for additive components, not ownership or pipe connections. Never put an unknown answer or corrigible author in identity. Unused reference fields may be omitted except mention_quote/type_quote. Entity terms may add ontology_type {type_id,quote} only for a stated semantic class supported by that same mention; type_id is an English technical type (use ontology intermediate_types where applicable), not an identity encoding or a translation of a proper name. This optional semantic type is separate from lexical identity. Property requests use type_filter {type_id,quote} only if explicitly requesting a class, never the old free value_type. Omission is not a guessed class or a wildcard. Different literal class words and absent/present classes are not automatically coreferent; do not add aliases, normalize synonyms/plurals, or erase class to force a match. ");
    Ok(guide)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ontology() -> Value {
        json!({"color":{"kind":"property"},"water_supply":{"kind":"property"},"mass":{"kind":"quantity"},"requires":{"kind":"property"},"contact_phone":{"kind":"property"}})
    }
    fn r(mention: &str, noun: Option<&str>, name: Option<&str>) -> Value {
        json!({"mention_quote":mention,"type_quote":noun,"name":name,"owners":[],"scope":"single"})
    }
    fn clause(text: &str, subject: Value, statement: Value) -> Value {
        json!({"quote":text,"subject":subject,"time":"state","context":[],"configuration":[],"statement":statement})
    }
    fn source(clauses: Vec<Value>) -> Value {
        json!({"status":"ok","document":{"certain":clauses,"readings":[],"readings_complete":true}})
    }
    fn property() -> Value {
        json!({"type":"property","attribute":"color","value":{"type":"text","value":"rojo"},"affirmed":true,"basis":"described_fact"})
    }
    fn query(text: &str, subject: Value, slot: Value) -> Value {
        json!({"status":"ok","question":{"subject":subject,"context":[],"configuration":[],"assumptions":[],"requested":[{"quote":text,"time":"state","extent":"unspecified","slot":slot}]}})
    }
    fn value(text: &str, raw: &Value) -> Result<Value, String> {
        translate("source", text, &ontology(), raw)
    }
    #[test]
    fn owners_fixed_group_cardinality_and_depth_do_not_collapse_or_escape_limits() {
        let make = |owner: &str, count: u32| {
            let text = format!("{count} sensores del equipo {owner} son rojos");
            let mut subject = r(&text, Some("sensores"), None);
            subject["scope"] = json!("fixed_group");
            subject["count"] = json!(count);
            subject["owners"] = json!([{"mention_quote":format!("equipo {owner}"),"type_quote":"equipo","name":owner}]);
            let out = value(&text, &source(vec![clause(&text, subject, property())])).unwrap();
            out["document"]["certain_facts"][0]["subject"]["entity"].clone()
        };
        assert_ne!(make("L", 2), make("M", 2));
        assert_ne!(make("L", 2), make("L", 3));
        assert_eq!(make("L", 2), make("L", 2));
        let text = "2 sensores del equipo L son rojos";
        let mut subject = r(text, Some("sensores"), None);
        subject["scope"] = json!("fixed_group");
        subject["count"] = json!(0);
        assert!(
            value(
                text,
                &source(vec![clause(text, subject.clone(), property())])
            )
            .is_err()
        );
        subject["count"] = json!(2);
        subject["owners"] = json!([{"mention_quote":"equipo L","name":"L"}]);
        assert!(
            value(
                text,
                &source(vec![clause(text, subject.clone(), property())])
            )
            .is_err()
        );
        subject["owners"] = json!(vec![
            json!({"mention_quote":"equipo L","type_quote":"equipo","name":"L"});
            9
        ]);
        assert!(value(text, &source(vec![clause(text, subject, property())])).is_err());
        let mut chain = r("equipo L", Some("equipo"), Some("L"));
        for _ in 0..5 {
            let mut parent = r("equipo L", Some("equipo"), Some("L"));
            parent["part_of"] = chain;
            chain = parent;
        }
        assert!(value(text, &source(vec![clause(text, chain, property())])).is_err());
    }
    #[test]
    fn same_grounded_reference_matches_across_source_question_and_keeps_bearer() {
        let text = "El formulario del pedido R4 de Vela es rojo.";
        let mut subject = r("formulario del pedido R4 de Vela", Some("formulario"), None);
        subject["owners"] = json!([{"mention_quote":"Vela","type_quote":null,"name":"Vela"},{"mention_quote":"pedido R4 de Vela","type_quote":"pedido","name":"R4"}]);
        let out = value(
            text,
            &source(vec![clause(text, subject.clone(), property())]),
        )
        .unwrap();
        let q = translate(
            "question",
            text,
            &ontology(),
            &query(
                text,
                subject.clone(),
                json!({"type":"property","attribute":"color","basis":"any","path":"direct"}),
            ),
        )
        .unwrap();
        assert_eq!(
            out["document"]["certain_facts"][0]["subject"],
            q["frame"]["subject"]
        );
        subject["type_quote"] = json!("pedido");
        subject["name"] = json!("R4");
        let other = value(text, &source(vec![clause(text, subject, property())])).unwrap();
        assert_ne!(
            out["document"]["certain_facts"][0]["subject"]["entity"],
            other["document"]["certain_facts"][0]["subject"]["entity"]
        );
    }
    #[test]
    fn absent_class_is_explicit_and_never_an_anonymous_global_class() {
        let text = "Nube es rojo.";
        let subject = r("Nube", None, Some("Nube"));
        let out = value(
            text,
            &source(vec![clause(text, subject.clone(), property())]),
        )
        .unwrap();
        assert!(
            out["document"]["certain_facts"][0]["subject"]["entity"]
                .as_str()
                .unwrap()
                .contains("lr1_unknown")
        );
        let mut absent = subject.clone();
        absent.as_object_mut().unwrap().remove("type_quote");
        assert!(value(text, &source(vec![clause(text, absent, property())])).is_err());
        let mut anonymous = subject;
        anonymous["name"] = Value::Null;
        anonymous["scope"] = json!("class_reference");
        assert!(value(text, &source(vec![clause(text, anonymous, property())])).is_err());
    }
    #[test]
    fn class_cue_must_be_inside_its_own_mention_and_old_kind_is_not_accepted() {
        let text = "La empresa Vela emitió el expediente R4, rojo.";
        let mut subject = r("empresa Vela", Some("expediente"), Some("Vela"));
        assert!(
            value(
                text,
                &source(vec![clause(text, subject.clone(), property())])
            )
            .is_err()
        );
        subject["type_quote"] = json!("empresa");
        assert!(
            value(
                text,
                &source(vec![clause(text, subject.clone(), property())])
            )
            .is_ok()
        );
        subject["kind"] = json!("single");
        assert!(value(text, &source(vec![clause(text, subject, property())])).is_err());
    }
    #[test]
    fn lexical_encoding_is_reversible_conservative_and_does_not_truncate() {
        let text = "El depósito R4 es rojo.";
        let out = value(
            text,
            &source(vec![clause(
                text,
                r("depósito R4", Some("depósito"), Some("R4")),
                property(),
            )]),
        )
        .unwrap();
        let id: Value = serde_json::from_str(
            out["document"]["certain_facts"][0]["subject"]["entity"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let encoded = id[2].as_str().unwrap().strip_prefix("lr1_").unwrap();
        let bytes = (0..encoded.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&encoded[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(String::from_utf8(bytes).unwrap(), "depósito");
        assert_ne!(
            lexical_kind(Some("depósito")).unwrap(),
            lexical_kind(Some("deposito")).unwrap()
        );
        assert_ne!(
            lexical_kind(Some("sensor")).unwrap(),
            lexical_kind(Some("sensores")).unwrap()
        );
        assert_ne!(
            lexical_kind(Some("Sensor")).unwrap(),
            lexical_kind(Some("sensor")).unwrap()
        );
        assert_ne!(
            lexical_kind(None).unwrap(),
            lexical_kind(Some("unknown")).unwrap()
        );
        let long = "x".repeat(47);
        let text = format!("{long} R4 es rojo");
        assert!(
            value(
                &text,
                &source(vec![clause(
                    &text,
                    r(&text, Some(&long), Some("R4")),
                    property()
                )])
            )
            .is_err()
        );
    }
    #[test]
    fn parent_owner_cardinality_and_class_quantifier_keep_distinct_meanings() {
        let text = "sensor S de equipo A y equipo B; sensores de A";
        let mut s = r("sensor S de equipo A", Some("sensor"), Some("S"));
        s["part_of"] = r("equipo A", Some("equipo"), Some("A"));
        let a = value(text, &source(vec![clause(text, s.clone(), property())])).unwrap();
        s["mention_quote"] = json!("sensor S de equipo A y equipo B");
        s["part_of"] = r("equipo B", Some("equipo"), Some("B"));
        let b = value(text, &source(vec![clause(text, s, property())])).unwrap();
        assert_ne!(
            a["document"]["certain_facts"][0]["subject"]["entity"],
            b["document"]["certain_facts"][0]["subject"]["entity"]
        );
        let mut class = r("sensores de A", Some("sensores"), None);
        class["scope"] = json!("all_members");
        let a = value(text, &source(vec![clause(text, class.clone(), property())])).unwrap();
        class["scope"] = json!("class_reference");
        let b = value(text, &source(vec![clause(text, class, property())])).unwrap();
        assert_eq!(
            a["document"]["certain_facts"][0]["subject"]["entity"],
            b["document"]["certain_facts"][0]["subject"]["entity"]
        );
        assert_eq!(a["document"]["certain_facts"][0]["subject"]["all"], true);
        assert_eq!(b["document"]["certain_facts"][0]["subject"]["all"], false);
    }
    #[test]
    fn semantic_types_are_separate_from_lexical_identity_in_facts_and_requests() {
        let text = "El huerto Nube usa el depósito R4; ¿qué red abastece el huerto Nube?";
        let entity = json!({"type":"entity","reference":r("depósito R4",Some("depósito"),Some("R4")),"ontology_type":{"type_id":"reservoir","quote":"depósito R4"}});
        let stmt = json!({"type":"flow","attribute":"water_supply","values":[entity.clone()],"exhaustive":false,"direction":"immediate","affirmed":true,"basis":"actual_assignment"});
        let out = value(
            text,
            &source(vec![clause(
                text,
                r("huerto Nube", Some("huerto"), Some("Nube")),
                stmt,
            )]),
        )
        .unwrap();
        let f = &out["document"]["certain_facts"][0];
        assert_eq!(f["datum"]["value_type"], "reservoir");
        assert!(f["datum"]["text"].as_str().unwrap().contains("lr1_"));
        let slot = json!({"type":"property","attribute":"water_supply","basis":"any","path":"transitive","type_filter":{"type_id":"network","quote":"red"}});
        let q = translate(
            "question",
            text,
            &ontology(),
            &query(text, r("huerto Nube", Some("huerto"), Some("Nube")), slot),
        )
        .unwrap();
        assert_eq!(q["frame"]["requests"][0]["value_type"], "network");
        assert_eq!(q["frame"]["requests"][0]["path"], "transitive");
        let mut closed = json!({"type":"exhaustive","attribute":"water_supply","values":[entity],"basis":"described_fact"});
        assert!(
            value(
                text,
                &source(vec![clause(
                    text,
                    r("huerto Nube", Some("huerto"), Some("Nube")),
                    closed.clone()
                )])
            )
            .is_err()
        );
        closed["values"][0]
            .as_object_mut()
            .unwrap()
            .remove("ontology_type");
        assert!(
            value(
                text,
                &source(vec![clause(
                    text,
                    r("huerto Nube", Some("huerto"), Some("Nube")),
                    closed
                )])
            )
            .is_ok()
        );
    }
    #[test]
    fn ontology_binding_needs_a_class_cue_and_cannot_conflict_with_requested_filter() {
        let text = "Nube R4 rojo depósito.";
        let mut entity = json!({"type":"entity","reference":r("R4",None,Some("R4")),"ontology_type":{"type_id":"reservoir","quote":"R4"}});
        let mut stmt = property();
        stmt["value"] = entity.clone();
        assert!(
            value(
                text,
                &source(vec![clause(text, r("Nube", None, Some("Nube")), stmt)])
            )
            .is_err()
        );
        entity["reference"] = r("R4 rojo depósito", Some("depósito"), Some("R4"));
        entity["ontology_type"]["quote"] = json!("depósito");
        let slot = json!({"type":"property","attribute":"water_supply","basis":"any","path":"direct","value_filter":entity,"type_filter":{"type_id":"network","quote":"depósito"}});
        assert!(
            translate(
                "question",
                text,
                &ontology(),
                &query(text, r("Nube", None, Some("Nube")), slot)
            )
            .is_err()
        );
    }
    #[test]
    fn all_clause_collections_are_translated_and_one_invalid_reference_degrades_the_whole() {
        let text = "equipo Nube es rojo";
        let c = clause(
            text,
            r("equipo Nube", Some("equipo"), Some("Nube")),
            property(),
        );
        let mut raw = source(vec![c.clone()]);
        raw["document"]["readings"] = json!([{"note":"conditional","clauses":[c.clone()]}]);
        raw["document"]["readings_complete"] = json!(false);
        let out = value(text, &raw).unwrap();
        assert_eq!(out["document"]["readings_exhaustive"], false);
        assert_eq!(
            out["document"]["certain_facts"][0],
            out["document"]["readings"][0]["facts"][0]
        );
        raw["document"]["readings"][0]["clauses"][0]["subject"]["type_quote"] = json!("invented");
        assert!(value(text, &raw).is_err());
        let mut q = query(
            text,
            r("equipo Nube", Some("equipo"), Some("Nube")),
            json!({"type":"property","attribute":"color","basis":"any","path":"direct"}),
        );
        q["question"]["assumptions"] = json!([c]);
        let out = translate("question", text, &ontology(), &q).unwrap();
        assert_eq!(out["frame"]["premises"].as_array().unwrap().len(), 1);
        q["question"]["requested"][0]["slot"]["value_type"] = json!("free_type");
        assert!(translate("question", text, &ontology(), &q).is_err());
    }
}
