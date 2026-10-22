//! Prospectively versioned evidence-state relation; V1 stays available unchanged.
use super::*;
pub mod record_projection;
struct Vocabulary {
    base: Ontology,
    kinds: BTreeMap<String, String>,
}
#[derive(Clone)]
struct MeasurementFact {
    base: Fact,
    measured_attribute: Option<String>,
    method: Option<String>,
}
fn vocabulary(v: &Value) -> Result<Vocabulary, String> {
    let mut plain = v.clone();
    let mut kinds = BTreeMap::new();
    for (key, value) in plain.as_object_mut().ok_or("falta ontología")? {
        let kind = value
            .as_object_mut()
            .ok_or("atributo inválido")?
            .remove("kind")
            .and_then(|v| v.as_str().map(str::to_owned))
            .ok_or("falta tipo de atributo")?;
        if !["property", "quantity", "measurement"].contains(&kind.as_str()) {
            return Err("tipo de atributo inválido".into());
        }
        kinds.insert(key.clone(), kind);
    }
    Ok(Vocabulary {
        base: ontology(&plain)?,
        kinds,
    })
}
fn measurement_fact(v: &Value, o: &Vocabulary) -> Result<MeasurementFact, String> {
    let mut plain = v.clone();
    let object = plain.as_object_mut().ok_or("hecho inválido")?;
    let measured = object.remove("measured_attribute").unwrap_or(Value::Null);
    let method = object.remove("measurement_method").unwrap_or(Value::Null);
    let ext = json!({"measured_attribute":measured,"measurement_method":method});
    let f = MeasurementFact {
        base: fact(&plain, &o.base)?,
        measured_attribute: opt(&ext, "measured_attribute")?,
        method: opt(&ext, "measurement_method")?,
    };
    if f.measured_attribute.is_some() != f.method.is_some() {
        return Err("medición necesita magnitud y procedimiento independientes".into());
    }
    if let Some(target) = &f.measured_attribute {
        if o.kinds.get(&f.base.attribute).map(String::as_str) != Some("measurement")
            || o.kinds.get(target).map(String::as_str) != Some("quantity")
        {
            return Err("relación de medición con tipos incompatibles".into());
        }
        if let Some(d) = &f.base.datum {
            if d.kind == "known" && d.text != f.method {
                return Err("identidad del procedimiento inconsistente".into());
            }
        }
    }
    Ok(f)
}
struct TypedQuestion {
    base: Question,
    request_identities: Vec<(Option<String>, Option<String>)>,
    premise_identities: Vec<(Option<String>, Option<String>)>,
}
fn typed_question(v: &Value, o: &Vocabulary) -> Result<TypedQuestion, String> {
    let mut plain = v.clone();
    let mut request_identities = Vec::new();
    for r in plain["requests"]
        .as_array_mut()
        .ok_or("faltan atributos pedidos")?
    {
        let mut ext = serde_json::Map::new();
        for key in [
            "measured_attribute",
            "measurement_method",
            "measured_attribute_scope",
            "measurement_method_scope",
        ] {
            ext.insert(
                key.into(),
                r.as_object_mut()
                    .ok_or("atributo solicitado inválido")?
                    .remove(key)
                    .unwrap_or(Value::Null),
            );
        }
        let ext = Value::Object(ext);
        let target = opt(&ext, "measured_attribute")?;
        let named_method = opt(&ext, "measurement_method")?;
        let legacy_method = opt(r, "value")?;
        if named_method
            .as_ref()
            .zip(legacy_method.as_ref())
            .is_some_and(|(a, b)| a != b)
        {
            return Err("identificadores de método contradictorios".into());
        }
        let method = named_method.or(legacy_method);
        let target_scope = opt(&ext, "measured_attribute_scope")?;
        let method_scope = opt(&ext, "measurement_method_scope")?;
        let is_measurement =
            o.kinds.get(&text(r, "attribute")?).map(String::as_str) == Some("measurement");
        if is_measurement {
            if target
                .as_ref()
                .is_some_and(|t| o.kinds.get(t).map(String::as_str) != Some("quantity"))
                || target.is_none() && target_scope.as_deref() != Some("any")
                || method.is_none() && method_scope.as_deref() != Some("any")
                || target_scope
                    .as_ref()
                    .is_some_and(|s| s != "any" || target.is_some())
                || method_scope
                    .as_ref()
                    .is_some_and(|s| s != "any" || method.is_some())
            {
                return Err("identidad de medición solicitada incompleta/inconsistente".into());
            }
            // In the legacy scalar form `value` names the procedure, not its status.
            // Canonicalize the alias only after checking that both identities agree.
            r["value"] = json!(method);
            request_identities.push((target, method));
        } else {
            if target.is_some()
                || opt(&ext, "measurement_method")?.is_some()
                || target_scope.is_some()
                || method_scope.is_some()
            {
                return Err("identidad de medición en otro atributo".into());
            }
            request_identities.push((None, None));
        }
    }
    let mut premise_identities = Vec::new();
    for p in plain["premises"].as_array_mut().ok_or("faltan premisas")? {
        let f = measurement_fact(p, o)?;
        if o.kinds.get(&f.base.attribute).map(String::as_str) == Some("measurement")
            && (f.measured_attribute.is_none() || f.method.is_none())
        {
            return Err("premisa de medición sin identidad completa".into());
        }
        premise_identities.push((f.measured_attribute, f.method));
        p.as_object_mut().unwrap().remove("measured_attribute");
        p.as_object_mut().unwrap().remove("measurement_method");
    }
    Ok(TypedQuestion {
        base: question(&plain, &o.base)?,
        request_identities,
        premise_identities,
    })
}
fn ordinary_role(o: &Vocabulary, q: &TypedQuestion, facts: &[MeasurementFact]) -> &'static str {
    let matching = |attribute: &str, identity: &(Option<String>, Option<String>)| {
        facts
            .iter()
            .filter(|f| {
                f.base.attribute != attribute
                    || (identity.0.is_none() || f.measured_attribute == identity.0)
                        && (identity.1.is_none() || f.method == identity.1)
            })
            .map(|f| f.base.clone())
            .collect::<Vec<_>>()
    };
    for (p, identity) in q.base.premises.iter().zip(&q.premise_identities) {
        let mut one = q.base.clone();
        one.requests.clear();
        one.premises = vec![p.clone()];
        if resolve(&o.base, &one, &matching(&p.attribute, identity)) == "B" {
            return "B";
        }
    }
    let mut answer = "none";
    for (r, identity) in q.base.requests.iter().zip(&q.request_identities) {
        if o.kinds.get(&r.attribute).map(String::as_str) == Some("measurement") {
            let direct = facts.iter().any(|f| {
                f.base.attribute == r.attribute
                    && (identity.0.is_none() || f.measured_attribute == identity.0)
                    && (identity.1.is_none() || f.method == identity.1)
                    && covers(&f.base.subject, &q.base.subject, false)
                    && f.base.modality == r.modality
                    && f.base.context == q.base.context
                    && f.base.configuration == q.base.configuration
                    && f.base
                        .datum
                        .as_ref()
                        .is_some_and(|d| f.base.positive || d.kind == "known")
            });
            if direct {
                return "A";
            }
        }
        let mut one = q.base.clone();
        one.requests = vec![r.clone()];
        one.premises.clear();
        match resolve(&o.base, &one, &matching(&r.attribute, identity)) {
            "A" => return "A",
            "C" => answer = "C",
            _ => {}
        }
    }
    answer
}
fn execution_status(f: &MeasurementFact) -> bool {
    f.base.datum.as_ref().is_some_and(|d| {
        f.base.modality == "occurred" && d.kind == "known"
            || f.base.modality == "state" && f.base.positive && d.kind == "pending"
    })
}
fn classify(o: &Vocabulary, q: &TypedQuestion, variants: &[Vec<MeasurementFact>]) -> Value {
    let mut roles = BTreeSet::new();
    let mut any_status = false;
    for facts in variants {
        let base_role = ordinary_role(o, q, facts);
        let status = q.base.requests.iter().any(|r| {
            o.kinds.get(&r.attribute).map(String::as_str) == Some("quantity")
                && r.modality == "state"
                && facts.iter().any(|f| {
                    f.measured_attribute.as_ref() == Some(&r.attribute)
                        && f.method.is_some()
                        && execution_status(f)
                        && covers(&f.base.subject, &q.base.subject, false)
                        && f.base.context == q.base.context
                        && f.base.configuration == q.base.configuration
                })
        });
        let effective = status && base_role != "B";
        any_status |= effective;
        roles.insert(if effective { "A" } else { base_role });
    }
    let selected = !roles.is_empty() && !roles.contains("none");
    let requires_review = roles.len() > 1 && roles.contains("none");
    let state = if requires_review {
        "utility_unresolved"
    } else if roles.len() > 1 {
        "relation_ambiguous"
    } else {
        "resolved"
    };
    json!({"status":state,"roles":roles,"selected":selected,"measurement_status":any_status,"requires_review":requires_review,"qualification_required":selected&&(any_status||roles.len()>1)})
}
fn calculate(o: &Vocabulary, qv: &Value, alternatives: &Value) -> Result<Value, String> {
    let q = typed_question(qv, o)?;
    let variants = alternatives
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or("faltan interpretaciones")?
        .iter()
        .map(|a| {
            a.as_array()
                .ok_or("lectura inválida")?
                .iter()
                .map(|f| measurement_fact(f, o))
                .collect::<Result<Vec<_>, String>>()
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(classify(o, &q, &variants))
}
fn hash(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    Ok(
        Sha256::digest(std::fs::read(path).map_err(|e| e.to_string())?)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )
}
fn read(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn reconstruct(v: &Value) -> Result<Value, String> {
    obj(
        v,
        &[
            "version",
            "annotation_notice",
            "ontology",
            "questions",
            "assertions",
            "pairs",
        ],
    )?;
    if v["version"] != "typed-facts-v2" {
        return Err("versión de estado de evidencia no admitida".into());
    }
    text(v, "annotation_notice")?;
    let o = vocabulary(&v["ontology"])?;
    let questions = v["questions"].as_object().ok_or("faltan preguntas")?;
    let assertions = v["assertions"].as_object().ok_or("faltan afirmaciones")?;
    let mut ids = BTreeSet::new();
    let mut text_questions = BTreeMap::new();
    let mut text_assertions = BTreeMap::new();
    let mut results = Vec::new();
    for pair in list(v, "pairs")? {
        obj(pair, &["id", "question", "assertion"])?;
        let id = text(pair, "id")?;
        if !ids.insert(id.clone()) {
            return Err("par duplicado".into());
        }
        let q = questions
            .get(&text(pair, "question")?)
            .ok_or("pregunta desconocida")?;
        let a = assertions
            .get(&text(pair, "assertion")?)
            .ok_or("afirmación desconocida")?;
        obj(q, &["text", "frame"])?;
        obj(a, &["text", "interpretations", "alternatives"])?;
        if text_questions
            .insert(text(q, "text")?, q["frame"].clone())
            .is_some_and(|old| old != q["frame"])
            || text_assertions
                .insert(text(a, "text")?, a["alternatives"].clone())
                .is_some_and(|old| old != a["alternatives"])
        {
            return Err("texto idéntico con hechos diferentes".into());
        }
        let interpretations = list(a, "interpretations")?;
        if interpretations.len() != a["alternatives"].as_array().map_or(0, Vec::len)
            || interpretations
                .iter()
                .any(|x| x.as_str().is_none_or(|s| s.trim().is_empty()))
        {
            return Err("lecturas sin interpretación explícita".into());
        }
        results.push(json!({"id":id,"result":calculate(&o,&q["frame"],&a["alternatives"])?}));
    }
    if results.is_empty() {
        return Err("banco vacío".into());
    }
    Ok(json!({"pairs":results}))
}
pub fn audit(input: &Path, output: &Path) -> Result<(), String> {
    let mut report = reconstruct(&read(input)?)?;
    report["version"] = json!("typed-facts-v2-report");
    report["input_sha256"] = json!(hash(input)?);
    report["manual_facts"] = json!(true);
    report["automatic_extraction_evaluated"] = json!(false);
    report["provider_calls"] = json!(0);
    report["known_provider_cost_usd"] = json!(0);
    crate::scoped_definition_probe::write_new(output, &report)
}
pub fn score(bank: &Path, raw: &Path, output: &Path) -> Result<(), String> {
    let manifest = read(&bank.join("manifest.json"))?;
    if manifest["version"] != "evidence-state-v4-reviewed" {
        return Err("banco no revisado".into());
    }
    let files = manifest["files_sha256"]
        .as_object()
        .ok_or("faltan hashes de banco")?;
    for (file, h) in files {
        if file.contains('/')
            || file.contains("..")
            || h.as_str() != Some(hash(&bank.join(file))?.as_str())
        {
            return Err("hash de banco inválido".into());
        }
    }
    for required in [
        "source-only-reviewed.json",
        "labels-reviewed.json",
        "blind-review.json",
        "blind-review-final.json",
        "adjudication.json",
    ] {
        if !files.contains_key(required) {
            return Err("archivo obligatorio no ligado".into());
        }
    }
    let external = manifest["external_sha256"]
        .as_object()
        .ok_or("faltan contratos ligados")?;
    for (path, h) in external {
        if h.as_str() != Some(hash(Path::new(path))?.as_str()) {
            return Err("contrato de banco alterado".into());
        }
    }
    for required in [
        "docs/evaluation/evidence-contribution-v3-contract.md",
        "docs/evaluation/evidence-contribution-v4-contract.md",
    ] {
        if !external.contains_key(required) {
            return Err("contrato obligatorio no ligado".into());
        }
    }
    let input = bank.join("source-only-reviewed.json");
    let v = read(&input)?;
    let r = read(raw)?;
    if r["version"] != "typed-facts-v2-report"
        || r["input_sha256"] != hash(&input)?
        || r["pairs"] != reconstruct(&v)?["pairs"]
    {
        return Err("resultado no reproduce entrada/código".into());
    }
    let labels = read(&bank.join("labels-reviewed.json"))?;
    if labels["version"] != "evidence-state-v4-reviewed-labels" {
        return Err("versión de etiquetas no admitida".into());
    }
    let mut expected = BTreeMap::new();
    for label in list(&labels, "pairs")? {
        let id = text(label, "id")?;
        if expected.insert(id, label).is_some() {
            return Err("etiqueta duplicada".into());
        }
    }
    let review = read(&bank.join("blind-review-final.json"))?;
    let adjudication = read(&bank.join("adjudication.json"))?;
    if review["summary"]["all_O1_through_O6_closed"] != true
        || adjudication["all_objections_resolved_before_execution"] != true
        || adjudication["blind_final_sha256"] != hash(&bank.join("blind-review-final.json"))?
        || adjudication["reviewed_source_sha256"] != hash(&input)?
        || adjudication["labels_reviewed_sha256"] != hash(&bank.join("labels-reviewed.json"))?
    {
        return Err("revisión/adjudicación no conciliada".into());
    }
    let mut reviewed = BTreeMap::new();
    for p in list(&review, "pairs")? {
        if reviewed.insert(text(p, "id")?, p).is_some() {
            return Err("revisión duplicada".into());
        }
    }
    if reviewed.len() != expected.len() {
        return Err("cobertura de revisión desigual".into());
    }
    for (id, label) in &expected {
        let reviewed = reviewed.get(id).ok_or("falta revisión de par")?;
        let role_set = |v: &Value| -> Result<BTreeSet<String>, String> {
            list(v, "roles")?
                .iter()
                .map(|r| {
                    r.as_str()
                        .map(str::to_owned)
                        .ok_or("rol de revisión inválido".into())
                })
                .collect()
        };
        if role_set(label)? != role_set(reviewed)? {
            return Err("roles sin conciliar con revisión".into());
        }
        for key in [
            "selected",
            "measurement_status",
            "requires_review",
            "qualification_required",
        ] {
            if boolean(label, key)? != boolean(reviewed, key)? {
                return Err("límites sin conciliar con revisión".into());
            }
        }
    }
    let mut errors = Vec::new();
    let mut correct = 0;
    let mut unresolved = 0;
    let mut ambiguous = 0;
    let mut meta = 0;
    let mut useful_in_any = 0;
    let mut unresolved_useful = 0;
    let mut selected = 0;
    for pair in list(&r, "pairs")? {
        let id = text(pair, "id")?;
        let gold = expected.remove(&id).ok_or("falta etiqueta")?;
        let actual = &pair["result"];
        let roles = list(gold, "roles")?
            .iter()
            .map(|v| v.as_str().ok_or("rol inválido"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if roles.is_empty()
            || roles.iter().any(|s| !["A", "B", "C", "none"].contains(s))
            || roles.len() != list(gold, "roles")?.len()
        {
            return Err("conjunto de roles inválido".into());
        }
        let is_selected = boolean(gold, "selected")?;
        let need_review = roles.len() > 1 && roles.contains("none");
        if is_selected == roles.contains("none")
            || boolean(gold, "requires_review")? != need_review
            || boolean(gold, "qualification_required")?
                != (is_selected && (boolean(gold, "measurement_status")? || roles.len() > 1))
            || boolean(gold, "measurement_status")? && !roles.contains("A")
        {
            return Err("etiquetas de relación/selección/calificación inconsistentes".into());
        }
        let status = if boolean(gold, "requires_review")? {
            "utility_unresolved"
        } else if roles.len() > 1 {
            "relation_ambiguous"
        } else {
            "resolved"
        };
        let target = json!({"status":status,"roles":roles,"selected":is_selected,"measurement_status":boolean(gold,"measurement_status")?,"requires_review":boolean(gold,"requires_review")?,"qualification_required":boolean(gold,"qualification_required")?});
        if actual == &target {
            correct += 1;
        } else {
            errors.push(json!({"id":id,"expected":target,"actual":actual}));
        }
        if roles.iter().any(|r| *r != "none") {
            useful_in_any += 1;
            if !actual["selected"].as_bool().ok_or("selección inválida")? {
                unresolved_useful += 1;
            }
        }
        unresolved += usize::from(actual["requires_review"] == true);
        ambiguous += usize::from(actual["status"] == "relation_ambiguous");
        meta += usize::from(actual["measurement_status"] == true);
        selected += usize::from(actual["selected"] == true);
    }
    if !expected.is_empty() || manifest["pairs"].as_u64() != Some((correct + errors.len()) as u64) {
        return Err("sobran etiquetas".into());
    }
    crate::scoped_definition_probe::write_new(
        output,
        &json!({"version":"evidence-state-v4-score","bank_manifest_sha256":hash(&bank.join("manifest.json"))?,"raw_report_sha256":hash(raw)?,"pairs":correct+errors.len(),"correct_outcomes":correct,"errors":errors,"passes_boundary_behavior":errors.is_empty(),"passes_autonomous_quality":errors.is_empty()&&unresolved_useful==0,"selected":selected,"relation_ambiguous_selected":ambiguous,"utility_unresolved":unresolved,"useful_in_any_reading":useful_in_any,"useful_not_automatically_selected":unresolved_useful,"measurement_status_outcomes":meta,"manual_facts_not_extraction":true,"historical_metrics_modified":false,"provider_calls":0,"known_provider_cost_usd":0}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Value {
        json!({"ontology":{"autonomy":{"kind":"quantity","additive":false,"intermediate_types":[]},"mass":{"kind":"quantity","additive":false,"intermediate_types":[]},"measurement":{"kind":"measurement","additive":false,"intermediate_types":[]}},"q":{"subject":{"entity":"X","all":false},"context":{},"configuration":{},"requests":[{"attribute":"autonomy","modality":"state","path":"direct"}],"premises":[]},"facts":[[{"subject":{"entity":"X","all":false},"attribute":"measurement","modality":"occurred","context":{},"configuration":{},"positive":false,"datum":{"kind":"known","text":"cycle"},"measured_attribute":"autonomy","measurement_method":"cycle"}]]})
    }
    fn run(v: &Value) -> Value {
        calculate(&vocabulary(&v["ontology"]).unwrap(), &v["q"], &v["facts"]).unwrap()
    }
    #[test]
    fn no_measurement_is_evidence_not_unknown() {
        let v = fixture();
        let before = v.clone();
        let r = run(&v);
        assert_eq!(r["roles"], json!(["A"]));
        assert_eq!(r["measurement_status"], true);
        assert_eq!(r["qualification_required"], true);
        assert_eq!(v, before);
    }
    #[test]
    fn future_intent_does_not_establish_execution_status() {
        let mut v = fixture();
        v["facts"][0][0]["modality"] = json!("planned");
        assert_eq!(run(&v)["selected"], false);
    }
    #[test]
    fn pending_execution_differs_from_a_plan() {
        let mut v = fixture();
        v["facts"][0][0]["modality"] = json!("state");
        v["facts"][0][0]["positive"] = json!(true);
        v["facts"][0][0]["datum"] = json!({"kind":"pending"});
        assert_eq!(run(&v)["measurement_status"], true);
    }
    #[test]
    fn other_quantity_is_not_a_contribution() {
        let mut v = fixture();
        v["facts"][0][0]["measured_attribute"] = json!("mass");
        assert_eq!(run(&v)["selected"], false);
    }
    #[test]
    fn component_measurement_does_not_establish_whole_autonomy() {
        let mut v = fixture();
        v["facts"][0][0]["subject"] = json!({"entity":"part","all":false,"part_of":"X"});
        assert_eq!(run(&v)["selected"], false);
    }
    #[test]
    fn fixed_conditions_are_preserved() {
        let mut v = fixture();
        v["q"]["context"] = json!({"route":"R"});
        v["facts"][0][0]["context"] = json!({"route":"S"});
        assert_eq!(run(&v)["selected"], false);
    }
    #[test]
    fn missing_quantity_or_method_cannot_be_filled_from_question() {
        let mut v = fixture();
        v["facts"][0][0]
            .as_object_mut()
            .unwrap()
            .remove("measurement_method");
        assert!(calculate(&vocabulary(&v["ontology"]).unwrap(), &v["q"], &v["facts"]).is_err());
    }
    #[test]
    fn direct_procedure_question_binds_its_measured_quantity() {
        let mut v = fixture();
        v["q"]["requests"] = json!([{"attribute":"measurement","modality":"occurred","value":"cycle","path":"direct","measured_attribute":"autonomy"}]);
        v["facts"][0][0]["measured_attribute"] = json!("mass");
        assert_eq!(run(&v)["selected"], false);
        v["facts"][0][0]["measured_attribute"] = json!("autonomy");
        assert_eq!(run(&v)["roles"], json!(["A"]));
        assert_eq!(run(&v)["measurement_status"], false);
    }
    #[test]
    fn pending_methods_are_not_conflated_for_refutation() {
        let mut v = fixture();
        let mut premise = v["facts"][0][0].clone();
        premise["modality"] = json!("state");
        premise["positive"] = json!(true);
        premise["datum"] = json!({"kind":"pending"});
        v["q"]["premises"] = json!([premise]);
        v["facts"][0][0]["modality"] = json!("state");
        v["facts"][0][0]["datum"] = json!({"kind":"pending"});
        v["facts"][0][0]["measurement_method"] = json!("estimation");
        assert_eq!(run(&v)["roles"], json!(["none"]));
        v["facts"][0][0]["measurement_method"] = json!("cycle");
        assert_eq!(run(&v)["roles"], json!(["B"]));
    }
    #[test]
    fn missing_measurement_quantity_is_not_an_implicit_wildcard() {
        let mut v = fixture();
        v["q"]["requests"] = json!([{"attribute":"measurement","modality":"occurred","value":"cycle","path":"direct"}]);
        assert!(calculate(&vocabulary(&v["ontology"]).unwrap(), &v["q"], &v["facts"]).is_err());
        v["q"]["requests"][0]["measured_attribute_scope"] = json!("any");
        assert_eq!(run(&v)["roles"], json!(["A"]));
    }
    #[test]
    fn equivalent_method_encodings_preserve_negative_execution_answer() {
        let mut v = fixture();
        v["q"]["requests"] = json!([{"attribute":"measurement","modality":"occurred","measurement_method":"cycle","measured_attribute":"autonomy","path":"direct"}]);
        let named = run(&v);
        assert_eq!(named["roles"], json!(["A"]));
        v["q"]["requests"][0]
            .as_object_mut()
            .unwrap()
            .remove("measurement_method");
        v["q"]["requests"][0]["value"] = json!("cycle");
        assert_eq!(run(&v), named);
    }
    #[test]
    fn contradictory_method_aliases_are_rejected() {
        let mut v = fixture();
        v["q"]["requests"] = json!([{"attribute":"measurement","modality":"occurred","measurement_method":"cycle","value":"estimation","measured_attribute":"autonomy","path":"direct"}]);
        assert!(calculate(&vocabulary(&v["ontology"]).unwrap(), &v["q"], &v["facts"]).is_err());
    }
    #[test]
    fn pending_status_can_answer_direct_procedure_state_question() {
        let mut v = fixture();
        v["q"]["requests"] = json!([{"attribute":"measurement","modality":"state","measurement_method":"cycle","measured_attribute":"autonomy","path":"direct"}]);
        v["facts"][0][0]["modality"] = json!("state");
        v["facts"][0][0]["positive"] = json!(true);
        v["facts"][0][0]["datum"] = json!({"kind":"pending"});
        let r = run(&v);
        assert_eq!(r["roles"], json!(["A"]));
        assert_eq!(r["measurement_status"], false);
    }
    #[test]
    fn utility_disagreement_requires_review_and_counts_nonselection() {
        let mut v = fixture();
        let mut other = v["facts"][0].clone();
        other[0]["subject"]["entity"] = json!("Y");
        v["facts"].as_array_mut().unwrap().push(other);
        let r = run(&v);
        assert_eq!(r["roles"], json!(["A", "none"]));
        assert_eq!(r["requires_review"], true);
        assert_eq!(r["selected"], false);
    }
}
