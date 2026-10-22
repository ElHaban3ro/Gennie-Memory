//! Pure relation calculation over manually supplied facts; no text extraction.
pub mod v2;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
#[derive(Clone, Debug, PartialEq, Eq)]
struct Subject {
    entity: String,
    all: bool,
    member_of: Option<String>,
    part_of: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Datum {
    kind: String,
    text: Option<String>,
    value_type: Option<String>,
}
#[derive(Clone, Debug)]
struct Fact {
    subject: Subject,
    attribute: String,
    modality: String,
    context: BTreeMap<String, String>,
    configuration: BTreeMap<String, String>,
    positive: bool,
    datum: Option<Datum>,
    only_values: Option<BTreeSet<String>>,
    basis: String,
    availability_scope: Option<String>,
    flow_scope: String,
    quantity_scope: String,
}
#[derive(Clone, Debug)]
struct Request {
    attribute: String,
    modality: String,
    value: Option<String>,
    value_type: Option<String>,
    path: String,
    extent: String,
}
#[derive(Clone, Debug)]
struct Question {
    subject: Subject,
    context: BTreeMap<String, String>,
    configuration: BTreeMap<String, String>,
    requests: Vec<Request>,
    premises: Vec<Fact>,
}
#[derive(Clone, Debug)]
struct Attribute {
    additive: bool,
    intermediate_types: BTreeSet<String>,
}
type Ontology = BTreeMap<String, Attribute>;
fn obj(v: &Value, allowed: &[&str]) -> Result<(), String> {
    let m = v.as_object().ok_or("se esperaba objeto")?;
    if m.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("campo no permitido en hechos tipados".into());
    }
    Ok(())
}
fn text(v: &Value, k: &str) -> Result<String, String> {
    v.get(k)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("falta texto {k}"))
}
fn opt(v: &Value, k: &str) -> Result<Option<String>, String> {
    match v.get(k) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) if !s.trim().is_empty() => Ok(Some(s.clone())),
        _ => Err(format!("texto opcional inválido {k}")),
    }
}
fn list<'a>(v: &'a Value, k: &str) -> Result<&'a Vec<Value>, String> {
    v.get(k)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("falta lista {k}"))
}
fn boolean(v: &Value, k: &str) -> Result<bool, String> {
    v.get(k)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("falta booleano {k}"))
}
fn map(v: &Value, k: &str) -> Result<BTreeMap<String, String>, String> {
    v.get(k)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("falta mapa {k}"))?
        .iter()
        .map(|(a, b)| {
            let s = b
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or("valor de condición inválido")?;
            if a.is_empty() {
                return Err("clave vacía".into());
            }
            Ok((a.clone(), s.into()))
        })
        .collect()
}
fn modality(v: &Value) -> Result<String, String> {
    let s = text(v, "modality")?;
    if !["state", "planned", "occurred"].contains(&s.as_str()) {
        return Err("modalidad no admitida".into());
    }
    Ok(s)
}
fn subject(v: &Value) -> Result<Subject, String> {
    obj(v, &["entity", "all", "member_of", "part_of"])?;
    let s = Subject {
        entity: text(v, "entity")?,
        all: boolean(v, "all")?,
        member_of: opt(v, "member_of")?,
        part_of: opt(v, "part_of")?,
    };
    if s.member_of.as_ref() == Some(&s.entity)
        || s.part_of.as_ref() == Some(&s.entity)
        || s.member_of.is_some() && s.part_of.is_some()
        || s.all && (s.member_of.is_some() || s.part_of.is_some())
    {
        return Err("ámbito de sujeto inválido".into());
    }
    Ok(s)
}
fn datum(v: &Value) -> Result<Datum, String> {
    obj(v, &["kind", "text", "value_type"])?;
    let d = Datum {
        kind: text(v, "kind")?,
        text: opt(v, "text")?,
        value_type: opt(v, "value_type")?,
    };
    if ![
        "known",
        "lower_bound",
        "upper_bound",
        "unknown",
        "pending",
        "illegible",
    ]
    .contains(&d.kind.as_str())
        || ["known", "lower_bound", "upper_bound"].contains(&d.kind.as_str()) != d.text.is_some()
    {
        return Err("dato/estado inválido".into());
    }
    Ok(d)
}
fn fact(v: &Value, o: &Ontology) -> Result<Fact, String> {
    obj(
        v,
        &[
            "subject",
            "attribute",
            "modality",
            "context",
            "configuration",
            "positive",
            "datum",
            "only_values",
            "basis",
            "availability_scope",
            "flow_scope",
            "quantity_scope",
        ],
    )?;
    let a = text(v, "attribute")?;
    if !o.contains_key(&a) {
        return Err("atributo no declarado".into());
    }
    let d = match v.get("datum") {
        None | Some(Value::Null) => None,
        Some(x) => Some(datum(x)?),
    };
    let only = match v.get("only_values") {
        None | Some(Value::Null) => None,
        Some(Value::Array(a)) => Some(
            a.iter()
                .map(|x| {
                    x.as_str()
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .ok_or("valor exhaustivo inválido".into())
                })
                .collect::<Result<BTreeSet<_>, String>>()?,
        ),
        _ => return Err("only_values inválido".into()),
    };
    let f = Fact {
        subject: subject(&v["subject"])?,
        attribute: a,
        modality: modality(v)?,
        context: map(v, "context")?,
        configuration: map(v, "configuration")?,
        positive: boolean(v, "positive")?,
        datum: d,
        only_values: only,
        basis: opt(v, "basis")?.unwrap_or_else(|| "description".into()),
        availability_scope: opt(v, "availability_scope")?,
        flow_scope: opt(v, "flow_scope")?.unwrap_or_else(|| "property".into()),
        quantity_scope: opt(v, "quantity_scope")?.unwrap_or_else(|| "unspecified".into()),
    };
    if ![
        "description",
        "assignment",
        "admission",
        "observation",
        "specification",
        "normative",
    ]
    .contains(&f.basis.as_str())
        || !["property", "immediate", "upstream"].contains(&f.flow_scope.as_str())
        || !["unspecified", "trial", "total", "component", "maximum"]
            .contains(&f.quantity_scope.as_str())
        || f.basis == "admission" && f.modality == "occurred"
        || f.availability_scope.is_some()
            && f.datum
                .as_ref()
                .is_none_or(|d| !["unknown", "pending", "illegible"].contains(&d.kind.as_str()))
    {
        return Err("calificador de hecho inválido".into());
    }
    if f.datum.is_some() == f.only_values.is_some() || !f.positive && f.only_values.is_some() {
        return Err("afirmación necesita dato o conjunto exhaustivo positivo".into());
    }
    Ok(f)
}
fn ontology(v: &Value) -> Result<Ontology, String> {
    v.as_object()
        .ok_or("falta ontología")?
        .iter()
        .map(|(k, v)| {
            obj(v, &["additive", "intermediate_types"])?;
            let types = list(v, "intermediate_types")?
                .iter()
                .map(|x| {
                    x.as_str()
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .ok_or("tipo intermedio inválido".to_string())
                })
                .collect::<Result<_, _>>()?;
            if k.is_empty() {
                return Err("atributo vacío".into());
            }
            Ok((
                k.clone(),
                Attribute {
                    additive: boolean(v, "additive")?,
                    intermediate_types: types,
                },
            ))
        })
        .collect()
}
fn question(v: &Value, o: &Ontology) -> Result<Question, String> {
    obj(
        v,
        &[
            "subject",
            "context",
            "configuration",
            "requests",
            "premises",
        ],
    )?;
    let requests = list(v, "requests")?
        .iter()
        .map(|x| {
            obj(
                x,
                &[
                    "attribute",
                    "modality",
                    "value",
                    "value_type",
                    "path",
                    "extent",
                ],
            )?;
            let a = text(x, "attribute")?;
            if !o.contains_key(&a) {
                return Err("atributo solicitado no declarado".into());
            }
            let path = text(x, "path")?;
            if !["direct", "transitive"].contains(&path.as_str()) {
                return Err("tipo de recorrido inválido".into());
            }
            Ok(Request {
                attribute: a,
                modality: modality(x)?,
                value: opt(x, "value")?,
                value_type: opt(x, "value_type")?,
                path,
                extent: opt(x, "extent")?.unwrap_or_else(|| "unspecified".into()),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    if requests
        .iter()
        .any(|r| !["unspecified", "maximum", "total", "common"].contains(&r.extent.as_str()))
    {
        return Err("extensión de pregunta inválida".into());
    }
    if requests.is_empty() {
        return Err("pregunta sin atributo solicitado".into());
    }
    Ok(Question {
        subject: subject(&v["subject"])?,
        context: map(v, "context")?,
        configuration: map(v, "configuration")?,
        requests,
        premises: list(v, "premises")?
            .iter()
            .map(|x| fact(x, o))
            .collect::<Result<_, _>>()?,
    })
}
fn covers(f: &Subject, q: &Subject, additive: bool) -> bool {
    f.entity == q.entity && f.all == q.all
        || f.all && !q.all && q.member_of.as_ref() == Some(&f.entity)
        || additive && !q.all && f.part_of.as_ref() == Some(&q.entity)
}
fn data_matches(d: &Datum, r: &Request, a: &Attribute) -> bool {
    if let Some(value) = &r.value {
        if d.kind != "known" || d.text.as_ref() != Some(value) {
            return false;
        }
    }
    match (&r.value_type, &d.value_type) {
        (Some(q), Some(f)) if q != f => r.path == "transitive" && a.intermediate_types.contains(f),
        (Some(_), None) => ["unknown", "pending", "illegible"].contains(&d.kind.as_str()),
        _ => true,
    }
}
fn resolve(o: &Ontology, q: &Question, facts: &[Fact]) -> &'static str {
    for p in &q.premises {
        for f in facts {
            if !(covers(&f.subject, &p.subject, false)
                || p.subject.all
                    && !f.subject.all
                    && f.subject.member_of.as_ref() == Some(&p.subject.entity))
                || f.attribute != p.attribute
                || f.modality != p.modality
                || f.context != p.context
                || f.configuration != p.configuration
                || f.flow_scope != p.flow_scope
                || f.availability_scope != p.availability_scope
            {
                continue;
            }
            if let (Some(fd), Some(pd)) = (&f.datum, &p.datum) {
                if fd == pd && f.positive != p.positive {
                    return "B";
                }
            }
            if p.positive {
                if let (Some(allowed), Some(d)) = (&f.only_values, &p.datum) {
                    if f.flow_scope == p.flow_scope
                        && d.kind == "known"
                        && d.text.as_ref().is_some_and(|s| !allowed.contains(s))
                    {
                        return "B";
                    }
                }
            }
        }
    }
    for r in &q.requests {
        let a = &o[&r.attribute];
        for f in facts {
            if f.attribute == r.attribute
                && f.modality == r.modality
                && f.context == q.context
                && f.configuration == q.configuration
                && covers(&f.subject, &q.subject, a.additive)
                && f.datum.as_ref().is_some_and(|d| {
                    data_matches(d, r, a) && (f.positive || r.value.is_some() && d.kind == "known")
                })
            // A preserves qualified partial evidence, not a complete answer.
            {
                return "A";
            }
        }
    }
    if !q.configuration.is_empty() {
        for r in &q.requests {
            for f in facts {
                if f.attribute != r.attribute
                    || f.modality != r.modality
                    || f.context != q.context
                    || !covers(&f.subject, &q.subject, false)
                    || !f.positive
                    || f.datum.as_ref().is_none_or(|d| d.kind != "known")
                    || !q
                        .configuration
                        .iter()
                        .any(|(key, value)| f.configuration.get(key).is_some_and(|v| v != value))
                    || q.configuration
                        .keys()
                        .any(|key| !f.configuration.contains_key(key))
                {
                    continue;
                }
                for other in facts {
                    if other.attribute == f.attribute
                        && other.modality == f.modality
                        && other.context == f.context
                        && other.subject.entity != q.subject.entity
                        && other.configuration == q.configuration
                        && other.positive
                        && other.datum.as_ref().is_some_and(|d| d.kind == "known")
                    {
                        return "C";
                    }
                }
            }
        }
    }
    "none"
}
fn calculate(o: &Ontology, qv: &Value, alternatives: &Value) -> Result<Value, String> {
    let q = question(qv, o)?;
    let variants = alternatives
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or("faltan lecturas de hechos")?;
    let mut roles = BTreeSet::new();
    for v in variants {
        let facts = v
            .as_array()
            .ok_or("lectura debe ser lista de hechos")?
            .iter()
            .map(|x| fact(x, o))
            .collect::<Result<Vec<_>, _>>()?;
        roles.insert(resolve(o, &q, &facts));
    }
    if roles.len() == 1 {
        Ok(json!({"status":"resolved","role":roles.first().unwrap()}))
    } else {
        Ok(json!({"status":"unsupported_ambiguous","possible_roles":roles}))
    }
}
pub fn audit(input: &Path, output: &Path) -> Result<(), String> {
    let bytes = std::fs::read(input).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    obj(
        &v,
        &[
            "version",
            "annotation_notice",
            "ontology",
            "questions",
            "assertions",
            "pairs",
        ],
    )?;
    if v["version"] != "typed-facts-v1" {
        return Err("versión no admitida".into());
    }
    text(&v, "annotation_notice")?;
    let o = ontology(&v["ontology"])?;
    let questions = v["questions"].as_object().ok_or("faltan preguntas")?;
    let assertions = v["assertions"].as_object().ok_or("faltan afirmaciones")?;
    let mut ids = BTreeSet::new();
    let mut text_questions = BTreeMap::new();
    let mut text_assertions = BTreeMap::new();
    let mut results = Vec::new();
    for p in list(&v, "pairs")? {
        obj(p, &["id", "question", "assertion"])?;
        let id = text(p, "id")?;
        if !ids.insert(id.clone()) {
            return Err("par duplicado".into());
        }
        let q = questions
            .get(&text(p, "question")?)
            .ok_or("pregunta desconocida")?;
        let a = assertions
            .get(&text(p, "assertion")?)
            .ok_or("afirmación desconocida")?;
        obj(q, &["text", "frame"])?;
        obj(a, &["text", "interpretations", "alternatives"])?;
        let qt = text(q, "text")?;
        let at = text(a, "text")?;
        if text_questions
            .insert(qt, q["frame"].clone())
            .is_some_and(|old| old != q["frame"])
            || text_assertions
                .insert(at, a["alternatives"].clone())
                .is_some_and(|old| old != a["alternatives"])
        {
            return Err("textos idénticos con hechos distintos".into());
        }
        let interpretations = list(a, "interpretations")?;
        if interpretations.len() != a["alternatives"].as_array().map_or(0, Vec::len)
            || interpretations
                .iter()
                .any(|x| x.as_str().is_none_or(|s| s.trim().is_empty()))
        {
            return Err("cada lectura necesita interpretación explícita".into());
        }
        results.push(json!({"id":id,"result":calculate(&o,&q["frame"],&a["alternatives"])?}));
    }
    use sha2::{Digest, Sha256};
    let sha = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    crate::scoped_definition_probe::write_new(
        output,
        &json!({"version":"typed-facts-report-v1","input_sha256":sha,"manual_facts":true,"automatic_extraction_evaluated":false,"provider_calls":0,"known_provider_cost_usd":0,"pairs":results}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn s(id: &str) -> Subject {
        Subject {
            entity: id.into(),
            all: false,
            member_of: None,
            part_of: None,
        }
    }
    fn f(id: &str, attr: &str, value: &str) -> Fact {
        Fact {
            subject: s(id),
            attribute: attr.into(),
            modality: "state".into(),
            context: BTreeMap::new(),
            configuration: BTreeMap::new(),
            positive: true,
            datum: Some(Datum {
                kind: "known".into(),
                text: Some(value.into()),
                value_type: None,
            }),
            only_values: None,
            basis: "description".into(),
            availability_scope: None,
            flow_scope: "property".into(),
            quantity_scope: "unspecified".into(),
        }
    }
    fn q(id: &str, attr: &str) -> Question {
        Question {
            subject: s(id),
            context: BTreeMap::new(),
            configuration: BTreeMap::new(),
            requests: vec![Request {
                attribute: attr.into(),
                modality: "state".into(),
                value: None,
                value_type: None,
                path: "direct".into(),
                extent: "unspecified".into(),
            }],
            premises: vec![],
        }
    }
    fn o() -> Ontology {
        BTreeMap::from([
            (
                "p".into(),
                Attribute {
                    additive: false,
                    intermediate_types: BTreeSet::new(),
                },
            ),
            (
                "count".into(),
                Attribute {
                    additive: true,
                    intermediate_types: BTreeSet::new(),
                },
            ),
        ])
    }
    #[test]
    fn wrong_entity_and_attribute_rejected() {
        assert_eq!(
            resolve(
                &o(),
                &q("X", "p"),
                &[f("Y", "p", "v"), f("X", "count", "3")]
            ),
            "none"
        );
    }
    #[test]
    fn plans_do_not_establish_execution() {
        let mut fact = f("X", "p", "v");
        fact.modality = "planned".into();
        assert_eq!(resolve(&o(), &q("X", "p"), &[fact]), "none");
    }
    #[test]
    fn universal_applies_to_member_not_reverse() {
        let mut query = q("item", "p");
        query.subject.member_of = Some("class".into());
        let mut fact = f("class", "p", "v");
        fact.subject.all = true;
        assert_eq!(resolve(&o(), &query, &[fact.clone()]), "A");
        query.subject = fact.subject;
        assert_eq!(resolve(&o(), &query, &[f("item", "p", "v")]), "none");
    }
    #[test]
    fn additive_part_preserved_but_not_common_policy() {
        let mut fact = f("part", "count", "3");
        fact.subject.part_of = Some("X".into());
        assert_eq!(resolve(&o(), &q("X", "count"), &[fact.clone()]), "A");
        fact.attribute = "p".into();
        assert_eq!(resolve(&o(), &q("X", "p"), &[fact]), "none");
    }
    #[test]
    fn positive_alternative_does_not_refute() {
        let mut query = q("X", "p");
        query.premises.push(f("X", "p", "R"));
        assert_eq!(resolve(&o(), &query, &[f("X", "p", "S")]), "A");
        let mut neg = f("X", "p", "R");
        neg.positive = false;
        assert_eq!(resolve(&o(), &query, &[neg]), "B");
    }
    #[test]
    fn exhaustive_set_can_refute_but_not_other_subject() {
        let mut query = q("X", "count");
        query.premises.push(f("X", "p", "R"));
        let mut only = f("X", "p", "S");
        only.datum = None;
        only.only_values = Some(BTreeSet::from(["S".into()]));
        assert_eq!(resolve(&o(), &query, &[only.clone()]), "B");
        only.subject = s("Y");
        assert_eq!(resolve(&o(), &query, &[only]), "none");
    }
    #[test]
    fn explicit_contrast_needs_both_assignments() {
        let mut query = q("X", "p");
        query.configuration.insert("mode".into(), "R".into());
        let mut own = f("X", "p", "2");
        own.configuration.insert("mode".into(), "S".into());
        let mut other = f("Y", "p", "4");
        other.configuration = query.configuration.clone();
        assert_eq!(resolve(&o(), &query, &[own.clone(), other.clone()]), "C");
        assert_eq!(resolve(&o(), &query, &[other]), "none");
        assert_eq!(resolve(&o(), &query, &[own]), "none");
    }
    #[test]
    fn fixed_context_is_not_configuration() {
        let mut query = q("X", "p");
        query.context.insert("edition".into(), "2026".into());
        let mut fact = f("X", "p", "v");
        fact.context.insert("edition".into(), "2025".into());
        assert_eq!(resolve(&o(), &query, &[fact]), "none");
    }
    #[test]
    fn unknown_and_bounds_are_useful_without_claiming_exactness() {
        for kind in [
            "unknown",
            "pending",
            "illegible",
            "lower_bound",
            "upper_bound",
        ] {
            let mut fact = f("X", "p", "75");
            fact.datum.as_mut().unwrap().kind = kind.into();
            assert_eq!(resolve(&o(), &q("X", "p"), &[fact]), "A");
        }
    }
    #[test]
    fn negated_unknown_is_not_positive_availability_evidence() {
        let mut fact = f("X", "p", "v");
        fact.positive = false;
        fact.datum.as_mut().unwrap().kind = "unknown".into();
        fact.datum.as_mut().unwrap().text = None;
        assert_eq!(resolve(&o(), &q("X", "p"), &[fact]), "none");
    }
    #[test]
    fn individual_counterexample_refutes_factual_universal_premise() {
        let mut query = q("class", "count");
        query.subject.all = true;
        let mut premise = f("class", "p", "R");
        premise.subject.all = true;
        query.premises.push(premise);
        let mut counter = f("item", "p", "R");
        counter.subject.member_of = Some("class".into());
        counter.positive = false;
        assert_eq!(resolve(&o(), &query, &[counter]), "B");
    }
    #[test]
    fn extra_configuration_key_does_not_create_contrast() {
        let mut query = q("X", "p");
        query.configuration.insert("mode".into(), "R".into());
        let mut own = f("X", "p", "2");
        own.configuration = query.configuration.clone();
        own.configuration.insert("firmware".into(), "v2".into());
        let mut other = f("Y", "p", "4");
        other.configuration = query.configuration.clone();
        assert_eq!(resolve(&o(), &query, &[own, other]), "none");
    }
    #[test]
    fn explicit_unavailable_status_preserved_without_inventing_value_type() {
        let mut query = q("X", "p");
        query.requests[0].value_type = Some("phone".into());
        let mut unknown = f("X", "p", "v");
        unknown.datum = Some(Datum {
            kind: "unknown".into(),
            text: None,
            value_type: None,
        });
        unknown.availability_scope = Some("local record".into());
        assert_eq!(resolve(&o(), &query, &[unknown]), "A");
    }
    #[test]
    fn direct_exclusion_does_not_refute_upstream_premise() {
        let mut query = q("X", "count");
        let mut premise = f("X", "p", "R");
        premise.flow_scope = "upstream".into();
        query.premises.push(premise);
        let mut exclusion = f("X", "p", "R");
        exclusion.positive = false;
        exclusion.flow_scope = "immediate".into();
        assert_eq!(resolve(&o(), &query, &[exclusion]), "none");
    }
    #[test]
    fn availability_in_distinct_records_is_not_a_contradiction() {
        let mut query = q("X", "count");
        let mut premise = f("X", "p", "v");
        premise.datum = Some(Datum {
            kind: "unknown".into(),
            text: None,
            value_type: None,
        });
        premise.availability_scope = Some("record_A".into());
        query.premises.push(premise.clone());
        premise.positive = false;
        premise.availability_scope = Some("record_B".into());
        assert_eq!(resolve(&o(), &query, &[premise]), "none");
    }
    #[test]
    fn hidden_role_fields_rejected() {
        let v = json!({"entity":"X","all":false,"same_case":true});
        assert!(subject(&v).is_err());
    }
}

/// Separate scoring adapter. Labels never enter the pure relation calculation.
pub fn score(input: &Path, raw: &Path, labels: &Path, output: &Path) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let read = |p: &Path| -> Result<Value, String> {
        serde_json::from_slice(&std::fs::read(p).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    };
    let hash = |p: &Path| -> Result<String, String> {
        Ok(Sha256::digest(std::fs::read(p).map_err(|e| e.to_string())?)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect())
    };
    let v = read(input)?;
    let r = read(raw)?;
    let gold = read(labels)?;
    if r["version"] != "typed-facts-report-v1"
        || gold["version"] != "typed-facts-labels-v1"
        || r["input_sha256"] != hash(input)?
    {
        return Err("reporte/etiquetas/input no coinciden".into());
    }
    let sources = gold["source_files_sha256"]
        .as_object()
        .ok_or("faltan fuentes de etiquetas")?;
    for (p, h) in sources {
        if h.as_str() != Some(hash(Path::new(p))?.as_str()) {
            return Err("fuente de etiquetas alterada".into());
        }
    }
    let mut actual = BTreeMap::new();
    for pair in list(&r, "pairs")? {
        let id = text(pair, "id")?;
        if actual.insert(id, pair["result"].clone()).is_some() {
            return Err("resultado duplicado".into());
        }
    }
    let mut expected = BTreeMap::new();
    for pair in list(&gold, "pairs")? {
        let id = text(pair, "id")?;
        let role = text(pair, "role")?;
        if !["A", "B", "C", "none"].contains(&role.as_str()) || expected.insert(id, role).is_some()
        {
            return Err("etiqueta inválida/duplicada".into());
        }
    }
    let mut source_gold = BTreeMap::new();
    for path in sources.keys() {
        let source = read(Path::new(path))?;
        let cases = if let Some(cases) = source.as_array() {
            cases
        } else {
            list(&source, "cases")?
        };
        for case in cases {
            let case_id = text(case, "id")?;
            for label in list(case, "labels")? {
                let id = format!("{case_id}/{}", text(label, "id")?);
                if source_gold.insert(id, text(label, "role")?).is_some() {
                    return Err("etiqueta fuente duplicada".into());
                }
            }
        }
    }
    if source_gold != expected {
        return Err("etiquetas no reproducen gold fuente".into());
    }
    let o = ontology(&v["ontology"])?;
    let mut correct = 0;
    let mut errors = Vec::new();
    let mut uncovered = Vec::new();
    let mut seen = BTreeSet::new();
    let mut signatures: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut by_bank: BTreeMap<String, Value> = BTreeMap::new();
    fn canonical(v: &Value) -> Value {
        match v {
            Value::Array(a) => {
                let mut a = a.iter().map(canonical).collect::<Vec<_>>();
                a.sort_by_key(Value::to_string);
                Value::Array(a)
            }
            Value::Object(m) => {
                Value::Object(m.iter().map(|(k, v)| (k.clone(), canonical(v))).collect())
            }
            _ => v.clone(),
        }
    }
    for pair in list(&v, "pairs")? {
        let id = text(pair, "id")?;
        if !seen.insert(id.clone()) {
            return Err("par duplicado".into());
        }
        let expect = expected.get(&id).ok_or("falta etiqueta")?;
        let result = actual.get(&id).ok_or("falta resultado")?;
        let q = &v["questions"][text(pair, "question")?]["frame"];
        let a = &v["assertions"][text(pair, "assertion")?]["alternatives"];
        if result != &calculate(&o, q, a)? {
            return Err("resultado no reproduce código vigente".into());
        }
        let signature = canonical(&json!({"question":q,"alternatives":a})).to_string();
        signatures
            .entry(signature)
            .or_default()
            .insert(expect.clone());
        let bank = id.split('/').next().unwrap_or("").starts_with("cv3q_");
        let counts = by_bank
            .entry(if bank { "minimal" } else { "propositions" }.into())
            .or_insert(json!({"total":0,"correct":0,"errors":0,"unsupported":0}));
        counts["total"] = json!(counts["total"].as_u64().unwrap() + 1);
        match result["status"].as_str() {
            Some("resolved") => {
                if result["role"] == *expect {
                    correct += 1;
                    counts["correct"] = json!(counts["correct"].as_u64().unwrap() + 1);
                } else {
                    errors.push(json!({"id":id,"expected":expect,"actual":result["role"]}));
                    counts["errors"] = json!(counts["errors"].as_u64().unwrap() + 1);
                }
            }
            Some("unsupported_ambiguous") => {
                uncovered.push(json!({"id":id,"historical_expected":expect,"possible_roles":result["possible_roles"]}));
                counts["unsupported"] = json!(counts["unsupported"].as_u64().unwrap() + 1);
            }
            _ => return Err("estado inválido".into()),
        }
    }
    if seen.len() != expected.len() || seen.len() != actual.len() || seen.is_empty() {
        return Err("universo de pares desigual/vacío".into());
    }
    let collisions = signatures.values().filter(|s| s.len() > 1).count();
    let report = json!({"version":"typed-facts-score-v1","input_sha256":hash(input)?,"raw_report_sha256":hash(raw)?,"labels_sha256":hash(labels)?,"total_pairs":seen.len(),"correct_roles":correct,"role_errors":errors,"unsupported":uncovered,"by_bank":by_bank,"representation_role_collisions":collisions,"passes_manual_sufficiency":errors.is_empty()&&uncovered.is_empty()&&collisions==0,"manual_frames_not_extraction":true,"historical_gold_modified":false,"provider_calls":0,"known_provider_cost_usd":0});
    crate::scoped_definition_probe::write_new(output, &report)
}
