//! Conditional record knowledge, not extraction or externally verified truth.
use super::*;
pub mod automatic_extraction;
pub mod known_sufficiency;

fn canonical(f: &MeasurementFact) -> Value {
    let b = &f.base;
    let d = b
        .datum
        .as_ref()
        .map(|d| json!({"kind":d.kind,"text":d.text,"value_type":d.value_type}));
    let mut result = json!({"subject":{"entity":b.subject.entity,"all":b.subject.all,"member_of":b.subject.member_of,"part_of":b.subject.part_of},"attribute":b.attribute,"modality":b.modality,"context":b.context,"configuration":b.configuration,"positive":b.positive,"datum":d,"only_values":b.only_values,"basis":b.basis,"availability_scope":b.availability_scope,"flow_scope":b.flow_scope,"quantity_scope":b.quantity_scope,"measured_attribute":f.measured_attribute,"measurement_method":f.method});
    if f.measured_attribute.is_some() && f.method.is_none() {
        result["measurement_method_scope"] = json!("unspecified");
    }
    result
}
struct RecordQuestion {
    typed: TypedQuestion,
    bases: Vec<String>,
}
fn record_question(v: &Value, o: &Vocabulary) -> Result<RecordQuestion, String> {
    let mut plain = v.clone();
    let mut bases = Vec::new();
    for r in plain["requests"]
        .as_array_mut()
        .ok_or("faltan solicitudes")?
    {
        let basis = r
            .as_object_mut()
            .ok_or("solicitud inválida")?
            .remove("basis_requirement")
            .unwrap_or(json!("any"));
        let basis = basis
            .as_str()
            .filter(|b| ["any", "actual_assignment", "admission"].contains(b))
            .ok_or("fundamento solicitado inválido")?;
        bases.push(basis.into());
    }
    Ok(RecordQuestion {
        typed: typed_question(&plain, o)?,
        bases,
    })
}
fn permits(basis: &str, f: &MeasurementFact) -> bool {
    match basis {
        "actual_assignment" => {
            !["admission", "normative", "specification"].contains(&f.base.basis.as_str())
        }
        "admission" => f.base.basis == "admission",
        _ => true,
    }
}
fn one_request(q: &RecordQuestion, i: usize) -> TypedQuestion {
    let mut base = q.typed.base.clone();
    base.requests = vec![base.requests[i].clone()];
    base.premises.clear();
    TypedQuestion {
        base,
        request_identities: vec![q.typed.request_identities[i].clone()],
        premise_identities: vec![],
    }
}
fn direct(o: &Vocabulary, q: &RecordQuestion, f: &MeasurementFact) -> bool {
    q.typed.base.requests.iter().enumerate().any(|(i, r)| {
        r.attribute == f.base.attribute
            && permits(&q.bases[i], f)
            && ordinary_role(o, &one_request(q, i), std::slice::from_ref(f)) == "A"
    })
}
fn refuting_basis(f: &Fact, p: &Fact) -> bool {
    if f.basis == p.basis {
        return true;
    }
    if ["normative", "specification"].contains(&f.basis.as_str())
        || ["normative", "specification"].contains(&p.basis.as_str())
    {
        return false;
    }
    if p.basis == "admission" {
        return !p.positive
            && f.positive
            && ["description", "assignment", "observation"].contains(&f.basis.as_str());
    }
    if f.basis == "admission" {
        return p.positive && (!f.positive || f.only_values.is_some());
    }
    true
}
fn positive_basis_entails(f: &Fact, p: &Fact) -> bool {
    if f.basis == p.basis {
        return true;
    }
    let actual = |b: &str| ["description", "assignment", "observation"].contains(&b);
    actual(&f.basis) && (actual(&p.basis) || p.basis == "admission")
}
fn record_refutes(o: &Vocabulary, q: &RecordQuestion, fs: &[MeasurementFact]) -> bool {
    q.typed.base.premises.iter().enumerate().any(|(i, p)| {
        let mut base = q.typed.base.clone();
        base.requests.clear();
        base.premises = vec![p.clone()];
        let one = TypedQuestion {
            base,
            request_identities: vec![],
            premise_identities: vec![q.typed.premise_identities[i].clone()],
        };
        let compatible = fs
            .iter()
            .filter(|f| refuting_basis(&f.base, p))
            .cloned()
            .collect::<Vec<_>>();
        if ordinary_role(o, &one, &compatible) == "B" {
            return true;
        }
        if p.only_values.is_none() || !p.positive {
            return false;
        }
        fs.iter().any(|f| {
            let identity = &q.typed.premise_identities[i];
            if !f.base.positive
                || f.base.datum.as_ref().is_none_or(|d| d.kind != "known")
                || !positive_basis_entails(&f.base, p)
                || f.base.attribute != p.attribute
                || identity.0.is_some() && f.measured_attribute != identity.0
                || identity.1.is_some() && f.method != identity.1
            {
                return false;
            }
            let mut base = q.typed.base.clone();
            base.requests.clear();
            base.premises = vec![f.base.clone()];
            let inverse = TypedQuestion {
                base,
                request_identities: vec![],
                premise_identities: vec![(f.measured_attribute.clone(), f.method.clone())],
            };
            let closed = MeasurementFact {
                base: p.clone(),
                measured_attribute: identity.0.clone(),
                method: identity.1.clone(),
            };
            ordinary_role(o, &inverse, &[closed]) == "B"
        })
    })
}
fn record_classify(o: &Vocabulary, q: &RecordQuestion, variants: &[Vec<MeasurementFact>]) -> Value {
    let mut roles = BTreeSet::new();
    let mut status = false;
    for fs in variants {
        if record_refutes(o, q, fs) {
            roles.insert("B".to_owned());
            continue;
        }
        let mut role = "none";
        for i in 0..q.typed.base.requests.len() {
            let attr = &q.typed.base.requests[i].attribute;
            let filtered = fs
                .iter()
                .filter(|f| f.base.attribute != *attr || permits(&q.bases[i], f))
                .cloned()
                .collect::<Vec<_>>();
            let single = one_request(q, i);
            let mut r = classify(o, &single, &[filtered.clone()]);
            if r["roles"] == json!(["C"]) {
                let assignments = filtered
                    .into_iter()
                    .filter(|f| {
                        !["admission", "normative", "specification"]
                            .contains(&f.base.basis.as_str())
                    })
                    .collect::<Vec<_>>();
                r = classify(o, &single, &[assignments]);
            }
            if r["roles"] == json!(["A"]) {
                role = "A";
                status |= r["measurement_status"] == true;
            } else if r["roles"] == json!(["C"]) && role != "A" {
                role = "C";
            }
        }
        roles.insert(role.to_owned());
    }
    let selected = !roles.is_empty() && !roles.contains("none");
    let review = roles.len() > 1 && roles.contains("none");
    json!({"roles":roles,"selected":selected,"requires_review":review,"measurement_status":status,"qualification_required":selected&&(status||roles.len()>1||roles.contains("C"))})
}
fn record_fact(v: &Value, o: &Vocabulary) -> Result<MeasurementFact, String> {
    if v.get("measurement_method_scope").is_none() {
        return measurement_fact(v, o);
    }
    let mut plain = v.clone();
    let fields = plain.as_object_mut().ok_or("hecho de medición inválido")?;
    let scope = fields
        .remove("measurement_method_scope")
        .ok_or("falta ámbito de método")?;
    let target = fields
        .remove("measured_attribute")
        .ok_or("falta magnitud medida")?;
    let method = fields.remove("measurement_method").unwrap_or(Value::Null);
    if scope != "unspecified" || !method.is_null() {
        return Err("ámbito de método no especificado inconsistente".into());
    }
    let b = fact(&plain, &o.base)?;
    let target = target
        .as_str()
        .filter(|t| o.kinds.get(*t).map(String::as_str) == Some("quantity"))
        .ok_or("magnitud de medición no especificada inválida")?;
    if o.kinds.get(&b.attribute).map(String::as_str) != Some("measurement")
        || b.datum.as_ref().is_none_or(|d| {
            !(["pending", "unknown", "illegible"].contains(&d.kind.as_str())
                || d.kind == "known" && d.text.as_deref() == Some("measurement"))
        })
    {
        return Err("método no especificado incompatible con hecho o dato".into());
    }
    Ok(MeasurementFact {
        base: b,
        measured_attribute: Some(target.into()),
        method: None,
    })
}
fn facts(v: &[Value], o: &Vocabulary) -> Result<Vec<MeasurementFact>, String> {
    v.iter().map(|f| record_fact(f, o)).collect()
}
fn direct_set(
    o: &Vocabulary,
    q: &RecordQuestion,
    fs: &[MeasurementFact],
) -> BTreeMap<String, Value> {
    fs.iter()
        .filter(|f| direct(o, q, f))
        .map(|f| {
            let v = canonical(f);
            (v.to_string(), v)
        })
        .collect()
}
fn project(o: &Vocabulary, q: &RecordQuestion, a: &Value) -> Result<Value, String> {
    obj(
        a,
        &[
            "text",
            "record_scope",
            "certain_facts",
            "alternatives",
            "alternatives_exhaustive",
            "uncertainty",
            "interpretations",
        ],
    )?;
    let source = text(a, "text")?;
    let scope = text(a, "record_scope")?;
    let closed = boolean(a, "alternatives_exhaustive")?;
    let certain = facts(list(a, "certain_facts")?, o)?;
    let alts = list(a, "alternatives")?;
    if alts.is_empty()
        || list(a, "interpretations")?.len() != alts.len()
        || list(a, "interpretations")?
            .iter()
            .any(|v| v.as_str().is_none_or(|s| s.trim().is_empty()))
    {
        return Err("lecturas documentales incompletas".into());
    }
    let cue = match a.get("uncertainty") {
        Some(Value::Null) => false,
        Some(v) => {
            obj(v, &["kind", "quote"])?;
            if !["identity", "value", "condition", "scope"].contains(&text(v, "kind")?.as_str())
                || !source.contains(&text(v, "quote")?)
            {
                return Err("incertidumbre sin cita literal o tipo válido".into());
            }
            true
        }
        None => return Err("falta estado documental de incertidumbre".into()),
    };
    let mut variants = Vec::new();
    let mut projections = Vec::new();
    for alt in alts {
        let mut fs = certain.clone();
        fs.extend(facts(alt.as_array().ok_or("lectura inválida")?, o)?);
        projections.push(direct_set(o, q, &fs));
        variants.push(fs);
    }
    let certain_set = direct_set(o, q, &certain);
    let union: BTreeMap<String, Value> = projections.iter().flat_map(|p| p.clone()).collect();
    let invariant: BTreeMap<String, Value> = if closed {
        union
            .iter()
            .filter(|(k, _)| projections.iter().all(|p| p.contains_key(*k)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    } else {
        certain_set.clone()
    };
    let variable = union.keys().any(|k| !invariant.contains_key(k));
    let epistemic = cue && variable;
    let v4 = record_classify(o, q, &variants);
    let certain_role = record_classify(o, q, &[certain]);
    let selected =
        certain_role["selected"] == true || closed && v4["selected"] == true || epistemic;
    let relevant_open_ambiguity = !closed
        && (certain_role["selected"] != true || variable || v4["roles"] != certain_role["roles"]);
    let review = relevant_open_ambiguity || !selected && v4["requires_review"] == true;
    let roles = list(&v4, "roles")?;
    let relation = if !selected {
        if review { "unresolved" } else { "none" }
    } else if !closed {
        let guaranteed = list(&certain_role, "roles")?;
        if certain_role["selected"] == true && v4["roles"] != certain_role["roles"] {
            "ambiguous"
        } else if certain_role["selected"] == true && guaranteed.len() == 1 {
            guaranteed[0].as_str().ok_or("rol cierto inválido")?
        } else if epistemic {
            "A"
        } else {
            "ambiguous"
        }
    } else if certain_role["roles"] == json!(["B"]) {
        "B"
    } else if epistemic && roles.contains(&json!("none")) {
        "A"
    } else if roles.len() == 1 {
        roles[0].as_str().ok_or("rol inválido")?
    } else {
        "ambiguous"
    };
    let basis = if !selected {
        if review {
            "requires_review"
        } else {
            "unrelated"
        }
    } else if epistemic {
        if invariant.is_empty() {
            "record_uncertainty"
        } else {
            "assertable_and_record_uncertainty"
        }
    } else if !invariant.is_empty() {
        "assertable"
    } else {
        "v4_useful"
    };
    let scoped_availability = invariant.values().any(|f| {
        !f["availability_scope"].is_null()
            || f["measurement_method_scope"] == "unspecified"
            || f["modality"] == "planned"
            || f["datum"]["kind"] != "known"
            || !["description", "assignment"]
                .iter()
                .any(|b| f["basis"] == *b)
            || f["subject"]["entity"] != q.typed.base.subject.entity
            || f["subject"]["all"] != q.typed.base.subject.all
            || q.typed.base.requests.iter().any(|r| {
                r.attribute == f["attribute"]
                    && r.value_type.is_some()
                    && f["datum"]["value_type"] != json!(r.value_type)
            })
    });
    let assertions = invariant
        .iter()
        .map(|(k, f)| {
            let witnesses = if certain_set.contains_key(k) {
                json!({"certain_facts":true})
            } else {
                json!({"readings":(0..projections.len()).collect::<Vec<_>>()})
            };
            json!({"record_asserts":f,"witnesses":witnesses})
        })
        .collect::<Vec<_>>();
    Ok(
        json!({"relation":relation,"selected":selected,"basis":basis,"assertable_facts":assertions.len(),"epistemic_contribution":epistemic,"world_roles":v4["roles"],"review_required":review,"coverage_incomplete":!closed,"qualification_required":selected&&(variable||v4["qualification_required"]==true||review||scoped_availability||!closed),"record_knowledge":{"text":source,"scope":scope,"external_truth_verified":false,"invariant_assertions":assertions,"variable_requested_fact_count":union.len()-invariant.len(),"requested_projection_invariant":!variable,"identity_resolution_claimed":false,"uncertainty":a["uncertainty"],"alternatives":a["alternatives"],"interpretations":a["interpretations"]}}),
    )
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
    if v["version"] != "epistemic-binding-v5" {
        return Err("versión de proyección no admitida".into());
    }
    text(v, "annotation_notice")?;
    let o = vocabulary(&v["ontology"])?;
    let questions = v["questions"].as_object().ok_or("faltan preguntas")?;
    let assertions = v["assertions"].as_object().ok_or("faltan afirmaciones")?;
    let mut same_q = BTreeMap::new();
    let mut same_a = BTreeMap::new();
    for q in questions.values() {
        obj(q, &["text", "frame"])?;
        record_question(&q["frame"], &o)?;
        if same_q
            .insert(text(q, "text")?, q["frame"].clone())
            .is_some_and(|old| old != q["frame"])
        {
            return Err("pregunta idéntica con frames distintos".into());
        }
    }
    for a in assertions.values() {
        let s = text(a, "text")?;
        if same_a.insert(s, a.clone()).is_some_and(|old| old != *a) {
            return Err("fuente idéntica con conocimiento distinto".into());
        }
    }
    let mut ids = BTreeSet::new();
    let mut out = Vec::new();
    for p in list(v, "pairs")? {
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
        out.push(json!({"id":id,"result":project(&o,&record_question(&q["frame"],&o)?,a)?}));
    }
    if out.is_empty() {
        return Err("banco vacío".into());
    }
    Ok(json!({"pairs":out}))
}
pub fn audit(input: &Path, output: &Path) -> Result<(), String> {
    let mut r = reconstruct(&read(input)?)?;
    r["version"] = json!("epistemic-binding-v5-report");
    r["input_sha256"] = json!(hash(input)?);
    r["manual_facts_not_extraction"] = json!(true);
    r["external_truth_verified"] = json!(false);
    r["provider_calls"] = json!(0);
    r["known_provider_cost_usd"] = json!(0);
    crate::scoped_definition_probe::write_new(output, &r)
}
const FIELDS: [&str; 9] = [
    "relation",
    "selected",
    "basis",
    "assertable_facts",
    "epistemic_contribution",
    "world_roles",
    "review_required",
    "coverage_incomplete",
    "qualification_required",
];
fn label(v: &Value) -> Result<Value, String> {
    let mut out = json!({});
    for k in FIELDS {
        if v.get(k).is_none() {
            return Err(format!("falta campo de etiqueta {k}"));
        }
        out[k] = v[k].clone();
    }
    for k in [
        "selected",
        "epistemic_contribution",
        "review_required",
        "coverage_incomplete",
        "qualification_required",
    ] {
        boolean(v, k)?;
    }
    for k in ["relation", "basis"] {
        text(v, k)?;
    }
    if v["assertable_facts"].as_u64().is_none() {
        return Err("cuenta de hechos inválida".into());
    }
    let roles = list(v, "world_roles")?;
    let mut seen = BTreeSet::new();
    for r in roles {
        let r = r.as_str().ok_or("rol inválido")?;
        if !["A", "B", "C", "none"].contains(&r) || !seen.insert(r) {
            return Err("roles inválidos o duplicados".into());
        }
    }
    if seen.is_empty() {
        return Err("roles vacíos".into());
    }
    out["world_roles"] = json!(seen);
    Ok(out)
}

fn audit_assertion_witnesses(source: &Value, report: &Value) -> Result<usize, String> {
    let o = vocabulary(&source["ontology"])?;
    let mut checked = 0;
    for p in list(report, "pairs")? {
        let id = text(p, "id")?;
        let pair = list(source, "pairs")?
            .iter()
            .find(|x| x["id"] == id)
            .ok_or("testigo sin par fuente")?;
        let a = &source["assertions"][text(pair, "assertion")?];
        let q = record_question(&source["questions"][text(pair, "question")?]["frame"], &o)?;
        let r = &p["result"];
        let certain = list(a, "certain_facts")?;
        let asserted = list(&r["record_knowledge"], "invariant_assertions")?;
        if r["assertable_facts"].as_u64() != Some(asserted.len() as u64) {
            return Err("cuenta de testigos desigual".into());
        }
        for claim in asserted {
            let f = record_fact(&claim["record_asserts"], &o)?;
            let c = canonical(&f);
            if !direct(&o, &q, &f) {
                return Err("testigo no aporta campo pedido".into());
            }
            let contains = |fs: &[Value]| -> Result<bool, String> {
                for x in fs {
                    if canonical(&record_fact(x, &o)?) == c {
                        return Ok(true);
                    }
                }
                Ok(false)
            };
            if !contains(certain)? {
                if !boolean(a, "alternatives_exhaustive")? {
                    return Err("hecho variable afirmado con lista abierta".into());
                }
                for alt in list(a, "alternatives")? {
                    if !contains(alt.as_array().ok_or("lectura inválida")?)? {
                        return Err("hecho afirmado sin testigo en cada lectura".into());
                    }
                }
            }
            checked += 1;
        }
    }
    Ok(checked)
}

pub fn score(bank: &Path, raw: &Path, output: &Path) -> Result<(), String> {
    let m = read(&bank.join("manifest.json"))?;
    if m["version"] != "epistemic-binding-v5-reviewed" {
        return Err("banco sin revisión".into());
    }
    let files = m["files_sha256"].as_object().ok_or("faltan hashes")?;
    for name in [
        "source-only-reviewed.json",
        "labels-reviewed.json",
        "blind-review.json",
        "blind-review-final.json",
        "blind-review-qualified.json",
        "adjudication.json",
    ] {
        if !files.contains_key(name) {
            return Err("archivo requerido no ligado".into());
        }
    }
    for (p, h) in files {
        if p.contains('/') || p.contains("..") || h.as_str() != Some(hash(&bank.join(p))?.as_str())
        {
            return Err("hash de banco inválido".into());
        }
    }
    let external = m["external_sha256"]
        .as_object()
        .ok_or("faltan contratos ligados")?;
    for name in [
        "docs/evaluation/evidence-contribution-v3-contract.md",
        "docs/evaluation/evidence-contribution-v4-contract.md",
        "docs/evaluation/epistemic-binding-v5-contract.md",
    ] {
        if !external.contains_key(name) {
            return Err("contrato requerido no ligado".into());
        }
    }
    for (p, h) in external {
        if h.as_str() != Some(hash(Path::new(p))?.as_str()) {
            return Err("contrato alterado".into());
        }
    }
    let source = bank.join("source-only-reviewed.json");
    let r = read(raw)?;
    if r["version"] != "epistemic-binding-v5-report"
        || r["input_sha256"] != hash(&source)?
        || r["pairs"] != reconstruct(&read(&source)?)?["pairs"]
    {
        return Err("resultado no reproducido".into());
    }
    let witnessed_facts = audit_assertion_witnesses(&read(&source)?, &r)?;
    let review = read(&bank.join("blind-review-qualified.json"))?;
    let adj = read(&bank.join("adjudication.json"))?;
    if review["ready_for_manual_boundary_execution"] != true
        || adj["all_objections_resolved_before_execution"] != true
        || adj["reviewed_source_sha256"] != hash(&source)?
        || adj["blind_final_sha256"] != hash(&bank.join("blind-review-qualified.json"))?
        || adj["labels_reviewed_sha256"] != hash(&bank.join("labels-reviewed.json"))?
    {
        return Err("revisión no conciliada".into());
    }
    let labels = read(&bank.join("labels-reviewed.json"))?;
    if labels["version"] != "epistemic-binding-v5-reviewed-labels" {
        return Err("etiquetas no revisadas".into());
    }
    let mut gold = BTreeMap::new();
    let mut reviewed = BTreeMap::new();
    for p in list(&labels, "pairs")? {
        if gold.insert(text(p, "id")?, label(p)?).is_some() {
            return Err("etiqueta duplicada".into());
        }
    }
    for p in list(&review, "pairs")? {
        if reviewed.insert(text(p, "id")?, label(p)?).is_some() {
            return Err("revisión duplicada".into());
        }
    }
    if gold != reviewed {
        return Err("etiquetas y revisión no conciliadas".into());
    }
    let mut errors = Vec::new();
    let mut correct = 0;
    let mut selected = 0;
    let mut epistemic_only = 0;
    let mut factual_pairs = 0;
    let mut review_count = 0;
    let mut unresolved_binding = 0;
    for p in list(&r, "pairs")? {
        let id = text(p, "id")?;
        let expected = gold.remove(&id).ok_or("falta etiqueta")?;
        let actual = label(&p["result"])?;
        if actual == expected {
            correct += 1
        } else {
            errors.push(json!({"id":id,"expected":expected,"actual":actual}));
        }
        selected += usize::from(actual["selected"] == true);
        epistemic_only += usize::from(
            actual["selected"] == true
                && actual["epistemic_contribution"] == true
                && actual["assertable_facts"] == 0,
        );
        factual_pairs += usize::from(actual["assertable_facts"].as_u64().unwrap() > 0);
        review_count += usize::from(actual["review_required"] == true);
        unresolved_binding +=
            usize::from(p["result"]["record_knowledge"]["requested_projection_invariant"] == false);
    }
    if !gold.is_empty() || m["pairs"].as_u64() != Some((correct + errors.len()) as u64) {
        return Err("cobertura desigual".into());
    }
    crate::scoped_definition_probe::write_new(
        output,
        &json!({"version":"epistemic-binding-v5-score","bank_manifest_sha256":hash(&bank.join("manifest.json"))?,"raw_report_sha256":hash(raw)?,"invariant_record_assertions_independently_witnessed":witnessed_facts,"pairs":correct+errors.len(),"correct_boundary_outcomes":correct,"errors":errors,"passes_boundary_behavior":errors.is_empty(),"selected":selected,"epistemic_only_selected":epistemic_only,"pairs_with_invariant_record_assertions":factual_pairs,"review_required":review_count,"pairs_with_variable_requested_projection":unresolved_binding,"automatic_extraction_evaluated":false,"fresh_validation":false,"passes_autonomous_quality":false,"historical_metrics_modified":false,"provider_calls":0,"known_provider_cost_usd":0}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> Value {
        json!({"ontology":{"phone":{"kind":"property","additive":false,"intermediate_types":[]},"mass":{"kind":"quantity","additive":true,"intermediate_types":[]}},"q":{"subject":{"entity":"X","all":false},"context":{},"configuration":{},"requests":[{"attribute":"phone","modality":"state","path":"direct"}],"premises":[]},"a":{"text":"The label may be X or Y, exclusively; phone 123.","record_scope":"quoted record only","certain_facts":[],"alternatives":[[{"subject":{"entity":"X","all":false},"attribute":"phone","modality":"state","context":{},"configuration":{},"positive":true,"datum":{"kind":"known","text":"123"}}],[{"subject":{"entity":"Y","all":false},"attribute":"phone","modality":"state","context":{},"configuration":{},"positive":true,"datum":{"kind":"known","text":"123"}}]],"alternatives_exhaustive":true,"uncertainty":{"kind":"identity","quote":"may be X or Y"},"interpretations":["X","Y"]}})
    }
    fn run(v: &Value) -> Result<Value, String> {
        let o = vocabulary(&v["ontology"])?;
        project(&o, &record_question(&v["q"], &o)?, &v["a"])
    }
    #[test]
    fn identity_uncertainty_does_not_assign_phone() {
        let r = run(&setup()).unwrap();
        assert_eq!(r["selected"], true);
        assert_eq!(r["assertable_facts"], 0);
        assert_eq!(
            r["record_knowledge"]["requested_projection_invariant"],
            false
        );
        assert_eq!(r["record_knowledge"]["external_truth_verified"], false);
    }
    #[test]
    fn certain_distinct_value_survives_without_transferring_conditional_one() {
        let mut v = setup();
        let mut certain = v["a"]["alternatives"][0][0].clone();
        certain["datum"]["text"] = json!("789");
        v["a"]["certain_facts"] = json!([certain]);
        let r = run(&v).unwrap();
        assert_eq!(r["assertable_facts"], 1);
        assert_eq!(r["epistemic_contribution"], true);
        assert_eq!(
            r["record_knowledge"]["invariant_assertions"][0]["record_asserts"]["datum"]["text"],
            "789"
        );
    }
    #[test]
    fn duplicate_conditional_fact_cannot_erase_certain_value() {
        let mut v = setup();
        v["a"]["certain_facts"] = json!([v["a"]["alternatives"][0][0]]);
        let r = run(&v).unwrap();
        assert_eq!(r["assertable_facts"], 1);
        assert_eq!(r["epistemic_contribution"], false);
        assert_eq!(
            r["record_knowledge"]["invariant_assertions"][0]["witnesses"]["certain_facts"],
            true
        );
    }
    #[test]
    fn absent_cue_keeps_analytic_utility_ambiguity_under_review() {
        let mut v = setup();
        v["a"]["uncertainty"] = Value::Null;
        let r = run(&v).unwrap();
        assert_eq!(r["selected"], false);
        assert_eq!(r["review_required"], true);
    }
    #[test]
    fn open_single_reading_cannot_prove_concrete_fact() {
        let mut v = setup();
        v["a"]["alternatives"].as_array_mut().unwrap().pop();
        v["a"]["interpretations"].as_array_mut().unwrap().pop();
        v["a"]["alternatives_exhaustive"] = json!(false);
        let r = run(&v).unwrap();
        assert_eq!(r["assertable_facts"], 0);
        assert_eq!(r["selected"], true);
        assert_eq!(r["review_required"], true);
        assert_eq!(r["coverage_incomplete"], true);
    }
    #[test]
    fn open_list_without_named_subject_is_not_irrelevant_or_selected() {
        let mut v = setup();
        v["q"]["subject"]["entity"] = json!("Z");
        v["a"]["alternatives_exhaustive"] = json!(false);
        let r = run(&v).unwrap();
        assert_eq!(r["relation"], "unresolved");
        assert_eq!(r["selected"], false);
    }
    #[test]
    fn nonexistent_quote_and_target_encoded_fields_are_rejected() {
        let mut v = setup();
        v["a"]["uncertainty"]["quote"] = json!("invented");
        assert!(run(&v).is_err());
        v = setup();
        v["a"]["role"] = json!("A");
        assert!(run(&v).is_err());
    }
    #[test]
    fn condition_mismatch_does_not_manufacture_epistemic_contribution() {
        let mut v = setup();
        v["q"]["context"] = json!({"site":"other"});
        let r = run(&v).unwrap();
        assert_eq!(r["selected"], false);
        assert_eq!(r["epistemic_contribution"], false);
    }
    #[test]
    fn optional_field_spelling_does_not_hide_invariance() {
        let mut v = setup();
        v["a"]["alternatives"][1][0] = v["a"]["alternatives"][0][0].clone();
        v["a"]["alternatives"][1][0]["basis"] = json!("description");
        v["a"]["alternatives"][1][0]["subject"]["member_of"] = Value::Null;
        let r = run(&v).unwrap();
        assert_eq!(r["assertable_facts"], 1);
        assert_eq!(r["epistemic_contribution"], false);
        assert_eq!(
            r["record_knowledge"]["invariant_assertions"][0]["witnesses"]["readings"],
            json!([0, 1])
        );
    }
    #[test]
    fn component_quantity_stays_component_not_total() {
        let mut v = setup();
        v["q"]["requests"][0]["attribute"] = json!("mass");
        for i in 0..2 {
            v["a"]["alternatives"][i][0]["attribute"] = json!("mass");
            v["a"]["alternatives"][i][0]["subject"] =
                json!({"entity":"battery","all":false,"part_of":"X"});
        }
        let r = run(&v).unwrap();
        assert_eq!(r["assertable_facts"], 1);
        assert_eq!(
            r["record_knowledge"]["invariant_assertions"][0]["record_asserts"]["subject"]["entity"],
            "battery"
        );
    }
    #[test]
    fn uncertain_refutation_does_not_turn_into_requested_field() {
        let mut v = setup();
        v["q"]["requests"][0]["attribute"] = json!("mass");
        v["q"]["premises"] = json!([v["a"]["alternatives"][0][0]]);
        for i in 0..2 {
            v["a"]["alternatives"][i][0]["positive"] = json!(false);
        }
        let r = run(&v).unwrap();
        assert_eq!(r["world_roles"], json!(["B", "none"]));
        assert_eq!(r["selected"], false);
        assert_eq!(r["epistemic_contribution"], false);
    }
    #[test]
    fn invariant_refutation_retains_priority_over_requested_fact() {
        let mut v = setup();
        let premise = v["a"]["alternatives"][0][0].clone();
        v["q"]["premises"] = json!([premise.clone()]);
        let mut negative = premise;
        negative["positive"] = json!(false);
        v["a"]["certain_facts"] = json!([negative]);
        let r = run(&v).unwrap();
        assert_eq!(r["relation"], "B");
        assert_eq!(r["selected"], true);
    }
    #[test]
    fn open_conditional_refutation_does_not_override_certain_answer() {
        let mut v = setup();
        let certain = v["a"]["alternatives"][0][0].clone();
        v["a"]["certain_facts"] = json!([certain]);
        let premise = json!({"subject":{"entity":"X","all":false},"attribute":"mass","modality":"state","context":{},"configuration":{},"positive":true,"datum":{"kind":"known","text":"10"}});
        v["q"]["premises"] = json!([premise.clone()]);
        let mut negative = premise;
        negative["positive"] = json!(false);
        v["a"]["alternatives"] = json!([[negative]]);
        v["a"]["interpretations"] = json!(["one represented possibility"]);
        v["a"]["alternatives_exhaustive"] = json!(false);
        let r = run(&v).unwrap();
        assert_eq!(r["world_roles"], json!(["B"]));
        assert_eq!(r["relation"], "ambiguous");
        assert_eq!(r["review_required"], true);
        assert_eq!(r["assertable_facts"], 1);
    }
    #[test]
    fn witness_auditor_rejects_conditional_fact_injected_as_invariant() {
        let v = setup();
        let source = json!({"ontology":v["ontology"],"questions":{"q":{"frame":v["q"]}},"assertions":{"a":v["a"]},"pairs":[{"id":"pair","question":"q","assertion":"a"}]});
        let mut r = run(&v).unwrap();
        r["assertable_facts"] = json!(1);
        r["record_knowledge"]["invariant_assertions"] =
            json!([{"record_asserts":v["a"]["alternatives"][0][0]}]);
        assert!(
            audit_assertion_witnesses(&source, &json!({"pairs":[{"id":"pair","result":r}]}))
                .is_err()
        );
    }
    #[test]
    fn all_useful_analytic_values_still_require_qualification() {
        let mut v = setup();
        v["a"]["uncertainty"] = Value::Null;
        v["a"]["alternatives"][1][0]["subject"]["entity"] = json!("X");
        v["a"]["alternatives"][1][0]["datum"]["text"] = json!("456");
        let r = run(&v).unwrap();
        assert_eq!(r["selected"], true);
        assert_eq!(r["epistemic_contribution"], false);
        assert_eq!(r["assertable_facts"], 0);
        assert_eq!(r["qualification_required"], true);
    }
    fn unspecified() -> Value {
        let mut v = setup();
        v["ontology"]["measurement"] =
            json!({"kind":"measurement","additive":false,"intermediate_types":[]});
        v["q"]["requests"] = json!([{"attribute":"measurement","modality":"occurred","path":"direct","measured_attribute":"mass","measurement_method_scope":"any"}]);
        let mut f = v["a"]["alternatives"][0][0].clone();
        f["attribute"] = json!("measurement");
        f["modality"] = json!("occurred");
        f["datum"] = json!({"kind":"known","text":"measurement"});
        f["measured_attribute"] = json!("mass");
        f["measurement_method_scope"] = json!("unspecified");
        v["a"]["alternatives"] = json!([[f]]);
        v["a"]["interpretations"] = json!(["literal measurement, method unspecified"]);
        v["a"]["uncertainty"] = Value::Null;
        v
    }
    #[test]
    fn unspecified_method_answers_generic_measurement_without_inventing_method() {
        let v = unspecified();
        let r = run(&v).unwrap();
        assert_eq!(r["relation"], "A");
        let f = &r["record_knowledge"]["invariant_assertions"][0]["record_asserts"];
        assert_eq!(f["measurement_method_scope"], "unspecified");
        assert!(f["measurement_method"].is_null());
        assert_eq!(f["measured_attribute"], "mass");
    }
    #[test]
    fn unspecified_method_never_answers_specific_method_or_other_quantity() {
        let mut v = unspecified();
        v["q"]["requests"][0]
            .as_object_mut()
            .unwrap()
            .remove("measurement_method_scope");
        v["q"]["requests"][0]["measurement_method"] = json!("scale");
        assert_eq!(run(&v).unwrap()["selected"], false);
        v = unspecified();
        v["ontology"]["duration"] =
            json!({"kind":"quantity","additive":false,"intermediate_types":[]});
        v["q"]["requests"][0]["measured_attribute"] = json!("duration");
        assert_eq!(run(&v).unwrap()["selected"], false);
    }
    #[test]
    fn unknown_method_does_not_open_execution_to_quantity_bridge() {
        let mut v = unspecified();
        v["q"]["requests"] = json!([{"attribute":"mass","modality":"state","path":"direct"}]);
        assert_eq!(run(&v).unwrap()["selected"], false);
    }
    #[test]
    fn unknown_method_requires_explicit_scope_and_neutral_datum() {
        let mut v = unspecified();
        v["a"]["alternatives"][0][0]
            .as_object_mut()
            .unwrap()
            .remove("measurement_method_scope");
        assert!(run(&v).is_err());
        v = unspecified();
        v["a"]["alternatives"][0][0]["datum"] = json!({"kind":"lower_bound","text":"10"});
        assert!(run(&v).is_err());
        v = unspecified();
        v["a"]["alternatives"][0][0]["measurement_method"] = json!("scale");
        assert!(run(&v).is_err());
    }
    #[test]
    fn generic_measurement_is_not_execution_when_only_planned() {
        let mut v = unspecified();
        v["a"]["alternatives"][0][0]["modality"] = json!("planned");
        assert_eq!(run(&v).unwrap()["selected"], false);
    }
    #[test]
    fn actual_assignment_rejects_admission_without_losing_its_qualifier() {
        let mut v = setup();
        v["a"]["alternatives"] = json!([[v["a"]["alternatives"][0][0]]]);
        v["a"]["interpretations"] = json!(["literal admission"]);
        v["a"]["alternatives"][0][0]["basis"] = json!("admission");
        v["a"]["uncertainty"] = Value::Null;
        v["q"]["requests"][0]["basis_requirement"] = json!("actual_assignment");
        assert_eq!(run(&v).unwrap()["selected"], false);
        v["q"]["requests"][0]["basis_requirement"] = json!("admission");
        let r = run(&v).unwrap();
        assert_eq!(r["relation"], "A");
        assert_eq!(r["qualification_required"], true);
        assert_eq!(
            r["record_knowledge"]["invariant_assertions"][0]["record_asserts"]["basis"],
            "admission"
        );
    }
    #[test]
    fn capacity_exclusion_can_refute_actual_use_without_establishing_use() {
        let mut v = setup();
        v["q"]["premises"] = json!([v["a"]["alternatives"][0][0]]);
        let mut exclusion = v["a"]["alternatives"][0][0].clone();
        exclusion["datum"] = Value::Null;
        exclusion["only_values"] = json!(["other"]);
        exclusion["basis"] = json!("admission");
        v["a"]["alternatives"] = json!([[exclusion]]);
        v["a"]["interpretations"] = json!(["admitted values exclude premise"]);
        v["q"]["requests"][0]["basis_requirement"] = json!("actual_assignment");
        let r = run(&v).unwrap();
        assert_eq!(r["relation"], "B");
        assert_eq!(r["assertable_facts"], 0);
    }
    #[test]
    fn contrast_does_not_use_admission_as_assignment() {
        let mut v = setup();
        v["q"]["configuration"] = json!({"mode":"fine"});
        let mut own = v["a"]["alternatives"][0][0].clone();
        own["configuration"] = json!({"mode":"draft"});
        own["basis"] = json!("admission");
        let mut other = own.clone();
        other["subject"]["entity"] = json!("Y");
        other["configuration"] = json!({"mode":"fine"});
        other["basis"] = json!("assignment");
        v["a"]["alternatives"] = json!([[own, other]]);
        v["a"]["interpretations"] = json!(["explicit capability and assignment"]);
        assert_eq!(run(&v).unwrap()["selected"], false);
        v["a"]["alternatives"][0][0]["basis"] = json!("assignment");
        let r = run(&v).unwrap();
        assert_eq!(r["relation"], "C");
        assert_eq!(r["qualification_required"], true);
    }
    #[test]
    fn irrelevant_open_reading_does_not_poison_certain_requested_fact() {
        let mut v = setup();
        v["a"]["certain_facts"] = json!([v["a"]["alternatives"][0][0]]);
        v["a"]["alternatives"] = json!([[], []]);
        v["a"]["alternatives_exhaustive"] = json!(false);
        let r = run(&v).unwrap();
        assert_eq!(r["selected"], true);
        assert_eq!(r["assertable_facts"], 1);
        assert_eq!(r["review_required"], false);
        assert_eq!(r["coverage_incomplete"], true);
        assert_eq!(r["qualification_required"], true);
    }
    #[test]
    fn actual_nonuse_does_not_refute_capability_but_incapability_can_refute_use() {
        let mut v = setup();
        let mut p = v["a"]["alternatives"][0][0].clone();
        p["basis"] = json!("admission");
        v["q"]["premises"] = json!([p]);
        v["q"]["requests"][0]["attribute"] = json!("mass");
        v["a"]["alternatives"] = json!([[v["a"]["alternatives"][0][0]]]);
        v["a"]["interpretations"] = json!(["non-use"]);
        v["a"]["alternatives"][0][0]["positive"] = json!(false);
        v["a"]["alternatives"][0][0]["basis"] = json!("assignment");
        assert_eq!(run(&v).unwrap()["relation"], "none");
        v["q"]["premises"][0]["basis"] = json!("assignment");
        v["a"]["alternatives"][0][0]["basis"] = json!("admission");
        assert_eq!(run(&v).unwrap()["relation"], "B");
    }
    #[test]
    fn normative_restriction_does_not_prove_actual_impossibility() {
        let mut v = setup();
        let mut p = v["a"]["alternatives"][0][0].clone();
        p["basis"] = json!("assignment");
        v["q"]["premises"] = json!([p]);
        let mut f = v["a"]["alternatives"][0][0].clone();
        f["datum"] = Value::Null;
        f["only_values"] = json!(["permitted"]);
        f["basis"] = json!("normative");
        v["a"]["alternatives"] = json!([[f]]);
        v["a"]["interpretations"] = json!(["rule"]);
        v["q"]["requests"][0]["attribute"] = json!("mass");
        assert_eq!(run(&v).unwrap()["relation"], "none");
        v["q"]["premises"][0]["basis"] = json!("normative");
        assert_eq!(run(&v).unwrap()["relation"], "B");
    }
    #[test]
    fn observed_use_refutes_incapacity_in_both_scalar_and_closed_set_premises() {
        let mut v = setup();
        let mut p = v["a"]["alternatives"][0][0].clone();
        p["basis"] = json!("admission");
        p["positive"] = json!(false);
        v["q"]["premises"] = json!([p]);
        v["q"]["requests"][0]["attribute"] = json!("mass");
        v["a"]["alternatives"] = json!([[v["a"]["alternatives"][0][0]]]);
        v["a"]["interpretations"] = json!(["actual use"]);
        v["a"]["alternatives"][0][0]["basis"] = json!("observation");
        assert_eq!(run(&v).unwrap()["relation"], "B");
        v["q"]["premises"][0]["positive"] = json!(true);
        v["q"]["premises"][0]["datum"] = Value::Null;
        v["q"]["premises"][0]["only_values"] = json!(["other"]);
        assert_eq!(run(&v).unwrap()["relation"], "B");
        v["a"]["alternatives"][0][0]["context"] = json!({"time":"different"});
        assert_eq!(run(&v).unwrap()["relation"], "none");
    }
    #[test]
    fn closed_premise_does_not_turn_permission_into_observed_use() {
        let mut v = setup();
        let mut p = v["a"]["alternatives"][0][0].clone();
        p["basis"] = json!("assignment");
        p["datum"] = Value::Null;
        p["only_values"] = json!(["other"]);
        v["q"]["premises"] = json!([p]);
        v["q"]["requests"][0]["attribute"] = json!("mass");
        v["a"]["alternatives"] = json!([[v["a"]["alternatives"][0][0]]]);
        v["a"]["interpretations"] = json!(["capability without use"]);
        v["a"]["alternatives"][0][0]["basis"] = json!("admission");
        assert_eq!(run(&v).unwrap()["relation"], "none");
        v["a"]["alternatives"][0][0]["basis"] = json!("normative");
        assert_eq!(run(&v).unwrap()["relation"], "none");
    }
}
