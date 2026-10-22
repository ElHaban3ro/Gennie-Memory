//! Evaluation-only automatic DTO pipeline, with manual ablations kept outside wire inputs.
use super::*;
pub mod codex;
pub mod reference_matrix;
use crate::documentary_extraction_contract::{self as contract, Profile};
use std::{
    fs::{self, OpenOptions},
    io::Write,
};
const BANK: &str = "tests/fixtures/documentary-known100-v1";
const PROTOCOL: &str = "docs/evaluation/documentary-extraction-v1-protocol-2026-10-20.md";
fn quote(v: &mut Value, field: &str, source: &str) -> Result<(), String> {
    let q = v
        .as_object_mut()
        .ok_or("DTO no es objeto")?
        .remove(field)
        .and_then(|v| v.as_str().map(str::to_owned))
        .filter(|s| !s.trim().is_empty())
        .ok_or("falta cita literal")?;
    if !source.contains(&q) {
        return Err("cita de otra fuente o inventada".into());
    }
    Ok(())
}
fn conditions(v: &Value) -> Result<Value, String> {
    let mut out = serde_json::Map::new();
    for p in v
        .as_array()
        .filter(|a| a.len() <= 12)
        .ok_or("condiciones inválidas")?
    {
        obj(p, &["key", "value"])?;
        let k = text(p, "key")?;
        if out.insert(k, json!(text(p, "value")?)).is_some() {
            return Err("clave de condición duplicada".into());
        }
    }
    Ok(Value::Object(out))
}
fn strip_fact(v: &Value, source: &str, o: &Vocabulary) -> Result<Value, String> {
    let mut f = v.clone();
    quote(&mut f, "quote", source)?;
    f["context"] = conditions(&f["context"])?;
    f["configuration"] = conditions(&f["configuration"])?;
    if f.get("measurement_method_scope")
        .is_some_and(Value::is_null)
    {
        f.as_object_mut()
            .unwrap()
            .remove("measurement_method_scope");
    }
    record_fact(&f, o)?;
    Ok(f)
}
fn facts_with_quotes(v: &Value, source: &str, o: &Vocabulary) -> Result<Vec<Value>, String> {
    v.as_array()
        .filter(|a| a.len() <= 12)
        .ok_or("lista de hechos inválida")?
        .iter()
        .map(|f| strip_fact(f, source, o))
        .collect()
}
fn decoded(kind: &str, source: &str, raw: &Value, o: &Vocabulary) -> Result<Value, String> {
    let field = match kind {
        "source" => "document",
        "question" => "frame",
        _ => return Err("etapa desconocida".into()),
    };
    obj(raw, &["status", "reason", field])?;
    if raw["status"] == "unsupported" {
        if !raw[field].is_null() {
            return Err("unsupported con DTO parcial ambiguo".into());
        }
        return Err(format!("extractor_unsupported:{}", text(raw, "reason")?));
    }
    if raw["status"] != "ok" || !raw["reason"].is_null() {
        return Err("estado de extracción inválido".into());
    }
    let v = &raw[field];
    if kind == "source" {
        obj(
            v,
            &[
                "certain_facts",
                "readings",
                "readings_exhaustive",
                "uncertainty",
            ],
        )?;
        let certain = facts_with_quotes(&v["certain_facts"], source, o)?;
        let readings = list(v, "readings")?;
        if readings.is_empty() || readings.len() > 4 {
            return Err("interpretaciones incompletas".into());
        }
        let mut interpretations = Vec::new();
        let mut alternatives = Vec::new();
        for r in readings {
            obj(r, &["note", "facts"])?;
            interpretations.push(text(r, "note")?);
            alternatives.push(facts_with_quotes(&r["facts"], source, o)?);
        }
        boolean(v, "readings_exhaustive")?;
        if !v["uncertainty"].is_null() {
            let mut cue = v["uncertainty"].clone();
            obj(&cue, &["kind", "quote"])?;
            quote(&mut cue, "quote", source)?;
        }
        Ok(
            json!({"text":source,"record_scope":"the quoted statement only","certain_facts":certain,"alternatives":alternatives,"interpretations":interpretations,"alternatives_exhaustive":v["readings_exhaustive"],"uncertainty":v["uncertainty"]}),
        )
    } else {
        obj(
            v,
            &[
                "subject",
                "subject_quote",
                "context",
                "configuration",
                "requests",
                "premises",
            ],
        )?;
        let mut frame = v.clone();
        quote(&mut frame, "subject_quote", source)?;
        frame["context"] = conditions(&frame["context"])?;
        frame["configuration"] = conditions(&frame["configuration"])?;
        let mut requests = Vec::new();
        for r in list(v, "requests")? {
            let mut r = r.clone();
            quote(&mut r, "quote", source)?;
            requests.push(r);
        }
        frame["requests"] = json!(requests);
        frame["premises"] = json!(facts_with_quotes(&v["premises"], source, o)?);
        record_question(&frame, o)?;
        Ok(json!({"text":source,"frame":frame}))
    }
}
fn requests(input: &Value) -> Result<Vec<Value>, String> {
    let mut source = input["assertions"]
        .as_object()
        .ok_or("faltan fuentes")?
        .iter();
    let mut question = input["questions"]
        .as_object()
        .ok_or("faltan preguntas")?
        .iter();
    let mut out = Vec::new();
    loop {
        let a = source.next();
        let q = question.next();
        if a.is_none() && q.is_none() {
            break;
        }
        for (kind, item) in [("source", a), ("question", q)] {
            if let Some((key, v)) = item {
                let source = text(v, "text")?;
                out.push(json!({"id":format!("op_{:03}",out.len()),"kind":kind,"target":key,"text":source,"payload":contract::payload(kind,&source,&input["ontology"])?}));
            }
        }
    }
    Ok(out)
}
fn requests_profile(input: &Value, profile: Profile) -> Result<Vec<Value>, String> {
    let mut rows = requests(input)?;
    if profile.is_openai() {
        rows = contract::PILOT_IDS
            .iter()
            .map(|id| {
                rows.iter()
                    .find(|r| r["id"] == *id)
                    .cloned()
                    .ok_or("piloto incompleto".into())
            })
            .collect::<Result<Vec<_>, String>>()?;
    }
    for r in &mut rows {
        r["payload"] = (if profile == Profile::GroundedRevisionV1 {
            Profile::GroundedReferenceV1
        } else {
            profile
        })
        .payload(
            r["kind"].as_str().unwrap(),
            r["text"].as_str().unwrap(),
            &input["ontology"],
        )?;
    }
    if profile == Profile::GroundedRevisionV1 {
        for (p, h) in contract::grounded_revision::PARENT_HASHES {
            if hash(Path::new(p))? != h {
                return Err("parent revision artifact changed".into());
            }
        }
        let prior = rows_from_revision_parent("requests.jsonl")?;
        let events = rows_from_revision_parent("journal.jsonl")?;
        rows = contract::grounded_revision::bind(rows, &prior, &events, &input["ontology"])?;
    }
    Ok(rows)
}
fn rows_from_revision_parent(name: &str) -> Result<Vec<Value>, String> {
    rows(&Path::new(contract::grounded_revision::PARENT).join(name))
}
fn write_rows(path: &Path, rows: &[Value]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    for r in rows {
        writeln!(f, "{r}").map_err(|e| e.to_string())?;
    }
    f.sync_all().map_err(|e| e.to_string())
}
fn validated_bank() -> Result<Value, String> {
    let parent = Path::new("outputs/iromem/evaluation/documentary-known100-v1-2026-10-20");
    let score = read(&parent.join("score.json"))?;
    if score
        != super::known_sufficiency::reconstruct_score(
            Path::new(BANK),
            &parent.join("report.json"),
        )?
        || score["passes_manual_representation"] != true
        || score["correct_boundary_outcomes"] != 100
    {
        return Err("pase manual no reproduce banco íntegro".into());
    }
    read(&Path::new(BANK).join("source-only-reviewed.json"))
}
pub fn prepare(d: &Path) -> Result<(), String> {
    prepare_profile(d, Profile::NativeV1)
}
pub fn prepare_json_v2(d: &Path) -> Result<(), String> {
    prepare_profile(d, Profile::JsonV2)
}
pub fn prepare_openai_pilot_v3(d: &Path) -> Result<(), String> {
    prepare_profile(d, Profile::OpenAiPilotV3)
}
pub fn prepare_openai_pilot_v4(d: &Path) -> Result<(), String> {
    prepare_profile(d, Profile::OpenAiPilotV4)
}
pub fn prepare_openai_guided_v5(d: &Path) -> Result<(), String> {
    prepare_profile(d, Profile::OpenAiGuidedPilotV5)
}
pub fn prepare_openai_guided_json(d: &Path) -> Result<(), String> {
    prepare_profile(d, Profile::OpenAiGuidedJsonPilotV1)
}
pub fn prepare_typed_clauses(d: &Path) -> Result<(), String> {
    prepare_profile(d, Profile::TypedClausePilotV1)
}
pub fn prepare_typed_clause_full(d: &Path) -> Result<(), String> {
    prepare_profile(d, Profile::TypedClauseFullV1)
}
pub fn prepare_grounded_references(d: &Path) -> Result<(), String> {
    prepare_profile(d, Profile::GroundedReferenceV1)
}
pub fn prepare_grounded_revision(d: &Path) -> Result<(), String> {
    prepare_profile(d, Profile::GroundedRevisionV1)
}
fn prepare_profile(d: &Path, profile: Profile) -> Result<(), String> {
    let input = validated_bank()?;
    let m = read(&Path::new(BANK).join("manifest.json"))?;
    for (p, h) in m["files_sha256"].as_object().ok_or("faltan hashes")? {
        if h.as_str() != Some(hash(&Path::new(BANK).join(p))?.as_str()) {
            return Err("banco alterado".into());
        }
    }
    let req = requests_profile(&input, profile)?;
    if req.len() != profile.max_requests() {
        return Err("roster conocido incompleto".into());
    }
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    let catalog = read(&d.join("catalog-observation.json"))?;
    let endpoint = catalog["response"]["data"]["endpoints"]
        .as_array()
        .ok_or("faltan endpoints")?
        .iter()
        .find(|v| v["tag"] == profile.tag() && v["model_id"] == profile.model())
        .ok_or("ruta no disponible")?;
    if endpoint["status"] != 0
        || !endpoint["supported_parameters"]
            .as_array()
            .ok_or("faltan parámetros")?
            .contains(&json!("structured_outputs"))
    {
        return Err("ruta sin salida estructurada".into());
    }
    for r in &req {
        contract::compatible_parameters(&r["payload"], endpoint)?;
    }
    let price = |k: &str| {
        endpoint["pricing"][k]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .filter(|v| v.is_finite() && *v >= 0.)
            .ok_or("precio inválido")
    };
    let prompt = price("prompt")?;
    let completion = price("completion")?;
    let bytes = req
        .iter()
        .map(|r| r["payload"].to_string().len())
        .sum::<usize>();
    write_rows(&d.join("requests.jsonl"), &req)?;
    crate::scoped_definition_probe::write_new(&d.join("ontology.json"), &input["ontology"])?;
    let mut paths = vec![
        PROTOCOL,
        "tools/evaluation/src/documentary_extraction_contract.rs",
        "tools/evaluation/src/typed_facts.rs",
        "tools/evaluation/src/typed_facts/v2.rs",
        "tools/evaluation/src/typed_facts/v2/record_projection.rs",
        "tools/evaluation/src/typed_facts/v2/record_projection/automatic_extraction.rs",
        "tools/evaluation/src/typed_facts/v2/record_projection/known_sufficiency.rs",
        "tools/evaluation/src/main.rs",
        "bootstrap/src/openrouter/per_memory/documentary_extraction_probe.rs",
        "bootstrap/src/openrouter/per_memory.rs",
    ];
    if profile == Profile::JsonV2 {
        paths.push("docs/evaluation/documentary-extraction-json-v2-protocol-2026-10-20.md");
    }
    if profile.is_openai() {
        paths.push("docs/evaluation/documentary-extraction-openai-v3-protocol-2026-10-20.md");
    }
    if profile == Profile::OpenAiPilotV4 {
        paths.push("docs/evaluation/documentary-extraction-openai-v4-protocol-2026-10-20.md");
    }
    if matches!(
        profile,
        Profile::OpenAiGuidedPilotV5 | Profile::OpenAiGuidedJsonPilotV1
    ) {
        paths
            .push("docs/evaluation/documentary-extraction-openai-guided-v5-protocol-2026-10-20.md");
    }
    if profile == Profile::OpenAiGuidedJsonPilotV1 {
        paths.push(
            "docs/evaluation/documentary-extraction-openai-guided-json-v1-protocol-2026-10-20.md",
        );
    }
    if profile.is_typed_clause() {
        paths.push("docs/evaluation/typed-clause-extraction-v1-protocol-2026-10-20.md");
        paths.push("tools/evaluation/src/documentary_extraction_contract/typed_clauses.rs");
        paths.extend([
            "Cargo.toml",
            "Cargo.lock",
            "tools/evaluation/Cargo.toml",
            "docs/decisions/0023-typed-clause-evaluation.md",
        ]);
    }
    if profile == Profile::TypedClauseFullV1 {
        paths.push("docs/evaluation/typed-clause-capacity-v1-protocol-2026-10-20.md");
    }
    if profile.is_grounded() {
        paths.push("docs/evaluation/grounded-reference-v1-protocol-2026-10-21.md");
        paths.push("tools/evaluation/src/documentary_extraction_contract/grounded_references.rs");
        paths.push("docs/decisions/0024-grounded-reference-evaluation.md");
    }
    if profile == Profile::GroundedRevisionV1 {
        paths.extend([
            "docs/evaluation/grounded-revision-v1-protocol-2026-10-21.md",
            "tools/evaluation/src/documentary_extraction_contract/grounded_revision.rs",
            "docs/decisions/0025-bounded-documentary-revision.md",
        ]);
        paths.extend(
            contract::grounded_revision::PARENT_HASHES
                .iter()
                .map(|(p, _)| *p),
        );
    }
    let sources = paths
        .iter()
        .map(|p| Ok((p.to_string(), json!(hash(Path::new(p))?))))
        .collect::<Result<serde_json::Map<_, _>, String>>()?;
    crate::scoped_definition_probe::write_new(
        &d.join("preparation.json"),
        &json!({"version":"documentary-extraction-v1-preparation","wire_profile":profile.key(),"model":profile.model(),"provider":profile.provider(),"source_operations":req.iter().filter(|r|r["kind"]=="source").count(),"question_operations":req.iter().filter(|r|r["kind"]=="question").count(),"max_http_requests":req.len(),"max_concurrent":4,"max_output_tokens_per_request":contract::MAX_TOKENS,"requests_sha256":hash(&d.join("requests.jsonl"))?,"ontology_sha256":hash(&d.join("ontology.json"))?,"catalog_sha256":hash(&d.join("catalog-observation.json"))?,"bank_manifest_sha256":hash(&Path::new(BANK).join("manifest.json"))?,"manual_score_sha256":hash(Path::new("outputs/iromem/evaluation/documentary-known100-v1-2026-10-20/score.json"))?,"manual_raw_sha256":hash(Path::new("outputs/iromem/evaluation/documentary-known100-v1-2026-10-20/report.json"))?,"source_hashes":sources,"payload_bytes":bytes,"illustrative_input_tokens_bytes_div3":bytes as f64/3.,"estimated_usd_at_900_output_tokens":bytes as f64/3.*prompt+req.len() as f64*900.*completion,"conservative_payload_bytes_plus_max_output_usd":bytes as f64*prompt+req.len() as f64*contract::MAX_TOKENS as f64*completion,"ontology_from_manual_domain_not_general_schema_discovery":true,"manual_frames_in_payload":false,"entity_catalog_in_payload":false,"fresh_validation":false,"promoted":false}),
    )
}
fn rows(path: &Path) -> Result<Vec<Value>, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())?
        .lines()
        .map(|s| serde_json::from_str(s).map_err(|e| e.to_string()))
        .collect()
}
fn p95(mut xs: Vec<f64>, fraction: f64) -> Value {
    if xs.is_empty() {
        return Value::Null;
    }
    xs.sort_by(f64::total_cmp);
    json!(xs[(xs.len() as f64 * fraction).ceil().max(1.) as usize - 1])
}
pub fn report(d: &Path, output: &Path) -> Result<(), String> {
    let input = validated_bank()?;
    let o = vocabulary(&input["ontology"])?;
    let prep = read(&d.join("preparation.json"))?;
    let profile = Profile::parse(&prep)?;
    let req = rows(&d.join("requests.jsonl"))?;
    if prep["version"] != "documentary-extraction-v1-preparation"
        || prep["requests_sha256"] != hash(&d.join("requests.jsonl"))?
        || prep["ontology_sha256"] != hash(&d.join("ontology.json"))?
        || prep["catalog_sha256"] != hash(&d.join("catalog-observation.json"))?
        || prep["bank_manifest_sha256"] != hash(&Path::new(BANK).join("manifest.json"))?
        || prep["manual_score_sha256"]
            != hash(Path::new(
                "outputs/iromem/evaluation/documentary-known100-v1-2026-10-20/score.json",
            ))?
        || prep["manual_raw_sha256"]
            != hash(Path::new(
                "outputs/iromem/evaluation/documentary-known100-v1-2026-10-20/report.json",
            ))?
        || req != requests_profile(&input, profile)?
    {
        return Err("preparación o wire alterado".into());
    }
    let pf = read(&d.join("preflight.json"))?;
    if pf["preparation_sha256"] != hash(&d.join("preparation.json"))? {
        return Err("preflight alterado".into());
    }
    for (p, h) in prep["source_hashes"].as_object().ok_or("faltan fuentes")? {
        if pf["source_hashes"][p] != *h
            || h.as_str() != Some(hash(&d.join("source-executed").join(p))?.as_str())
        {
            return Err("fuentes ejecutadas alteradas".into());
        }
    }
    let journal = rows(&d.join("journal.jsonl"))?;
    let mut starts = BTreeSet::new();
    let mut ends = BTreeMap::new();
    for e in journal {
        let id = text(&e, "id")?;
        if !req.iter().any(|r| r["id"] == id) {
            return Err("operación ajena".into());
        }
        match e["kind"].as_str() {
            Some("started") => {
                if req.get(starts.len()).is_none_or(|r| r["id"] != id) || !starts.insert(id) {
                    return Err("inicio duplicado".into());
                }
            }
            Some("response" | "failed") => {
                if !starts.contains(&id) || ends.insert(id, e).is_some() {
                    return Err("fin sin inicio o duplicado".into());
                }
            }
            _ => return Err("evento no admitido".into()),
        }
    }
    let mut sources = BTreeMap::new();
    let mut questions = BTreeMap::new();
    let mut operations = Vec::new();
    let mut fees = [0., 0.];
    let mut unknown = [0usize; 2];
    let mut times: [Vec<f64>; 2] = [vec![], vec![]];
    for r in &req {
        let id = text(r, "id")?;
        let kind = text(r, "kind")?;
        let ai = usize::from(kind == "question");
        if !starts.contains(&id) {
            operations
                .push(json!({"id":id,"kind":kind,"target":r["target"],"status":"not_attempted"}));
            continue;
        }
        let Some(e) = ends.get(&id) else {
            unknown[ai] += 1;
            operations.push(
                json!({"id":id,"kind":kind,"target":r["target"],"status":"uncertain_inflight"}),
            );
            continue;
        };
        if let Some(ms) = e["elapsed_ms"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.)
        {
            times[ai].push(ms);
        } else {
            return Err("latencia inválida".into());
        }
        let fee = e["body"]["usage"]["cost"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.);
        if let Some(f) = fee {
            fees[ai] += f
        } else {
            unknown[ai] += 1;
        }
        let parsed = if e["kind"] == "response"
            && e["http_status"]
                .as_u64()
                .is_some_and(|s| (200..300).contains(&s))
        {
            contract::checked_content(&e["body"], &r["payload"]).and_then(|raw| {
                let legacy = if profile.is_grounded() {
                    contract::grounded_references::translate(
                        &kind,
                        &text(&r, "text")?,
                        &input["ontology"],
                        &raw,
                    )?
                } else if profile.is_typed_clause() {
                    contract::typed_clauses::translate(
                        &kind,
                        r["text"].as_str().unwrap(),
                        &input["ontology"],
                        &raw,
                    )?
                } else {
                    raw.clone()
                };
                decoded(&kind, r["text"].as_str().unwrap(), &legacy, &o).map(|v| (raw, v))
            })
        } else {
            Err("transport_or_http_failure".into())
        };
        match parsed{Ok((raw,frame))=>{let target=text(r,"target")?;if kind=="source"{sources.insert(target,frame.clone());}else{questions.insert(target,frame.clone());}operations.push(json!({"id":id,"kind":kind,"target":r["target"],"text":r["text"],"status":"structurally_valid","raw_extraction":raw,"normalized":frame,"semantic_fidelity_certified":false,"known_cost_usd":fee}));},Err(error)=>operations.push(json!({"id":id,"kind":kind,"target":r["target"],"text":r["text"],"status":"invalid_or_unsupported","error":error,"known_cost_usd":fee}))}
    }
    let labels = read(&Path::new(BANK).join("labels-reviewed.json"))?;
    let gold = labels["pairs"]
        .as_array()
        .ok_or("faltan etiquetas")?
        .iter()
        .map(|p| (p["id"].as_str().unwrap(), p))
        .collect::<BTreeMap<_, _>>();
    let mut arms = Vec::new();
    for (auto_source, auto_question, name) in [
        (false, false, "manual_control"),
        (true, false, "automatic_source"),
        (false, true, "automatic_question"),
        (true, true, "automatic_both"),
    ] {
        let mut results = Vec::new();
        let mut correct = 0;
        let mut selected = 0;
        let mut missing = 0;
        let mut missing_useful = 0;
        let mut extras = 0;
        let mut losses = 0;
        let mut role_errors = 0;
        let mut count_errors = 0;
        let mut limit_errors = 0;
        for pair in list(&input, "pairs")? {
            let id = text(pair, "id")?;
            let qkey = text(pair, "question")?;
            let akey = text(pair, "assertion")?;
            let expected = gold.get(id.as_str()).ok_or("falta etiqueta")?;
            let a = if auto_source {
                sources.get(&akey)
            } else {
                input["assertions"].get(&akey)
            };
            let q = if auto_question {
                questions.get(&qkey)
            } else {
                input["questions"].get(&qkey)
            };
            let computed = match (a, q) {
                (Some(a), Some(q)) => {
                    record_question(&q["frame"], &o).and_then(|q| project(&o, &q, a))
                }
                _ => Err("missing_valid_extraction".into()),
            };
            match computed {
                Err(error) => {
                    missing += 1;
                    missing_useful += usize::from((*expected)["selected"] == true);
                    results.push(
                        json!({"id":id,"status":"degraded","error":error,"result":Value::Null}),
                    );
                }
                Ok(actual) => {
                    let role_ok = actual["relation"] == expected["relation"]
                        && actual["world_roles"] == expected["world_roles"];
                    let count_ok = actual["assertable_facts"] == expected["assertable_facts"];
                    let limit_ok = actual["review_required"] == expected["review_required"]
                        && (!boolean(expected, "qualification_required")?
                            || actual["qualification_required"] == true);
                    let select_ok = actual["selected"] == expected["selected"];
                    role_errors += usize::from(!role_ok);
                    count_errors += usize::from(!count_ok);
                    limit_errors += usize::from(!limit_ok);
                    correct += usize::from(role_ok && count_ok && limit_ok && select_ok);
                    selected += usize::from(actual["selected"] == true);
                    extras +=
                        usize::from(actual["selected"] == true && expected["selected"] == false);
                    losses +=
                        usize::from(actual["selected"] == false && expected["selected"] == true);
                    results.push(json!({"id":id,"status":"computed","matches_boundary_annotation":role_ok&&count_ok&&limit_ok&&select_ok,"expected":expected,"result":actual}));
                }
            }
        }
        if name == "manual_control" && correct != 100 {
            return Err("control manual no reproducido".into());
        }
        arms.push(json!({"name":name,"pairs":100,"correct_boundary_outcomes":correct,"selected":selected,"degraded_pairs":missing,"uncovered_expected_useful":missing_useful,"extra_selections":extras,"useful_not_selected":losses,"role_errors":role_errors,"assertion_count_errors":count_errors,"limit_errors":limit_errors,"passes_structural_role_boundary":correct==100,"semantic_fidelity_certified":false,"results":results}));
    }
    crate::scoped_definition_probe::write_new(
        output,
        &json!({"version":"documentary-extraction-v1-report","requests_sha256":hash(&d.join("requests.jsonl"))?,"journal_sha256":hash(&d.join("journal.jsonl"))?,"operations_attempted":starts.len(),"operations_ended":ends.len(),"planned_operations":req.len(),"valid_source_dtos":sources.len(),"valid_question_dtos":questions.len(),"known_source_cost_usd":fees[0],"known_question_cost_usd":fees[1],"known_provider_cost_usd":fees[0]+fees[1],"unknown_cost_operations":unknown[0]+unknown[1],"source_p50_ms":p95(times[0].clone(),0.5),"source_p95_ms":p95(times[0].clone(),0.95),"question_p50_ms":p95(times[1].clone(),0.5),"question_p95_ms":p95(times[1].clone(),0.95),"operations":operations,"arms":arms,"semantic_review_pending":true,"fresh_validation":false,"promoted":false,"passes_autonomous_quality":false}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn ontology_value() -> Value {
        json!({"mass":{"kind":"quantity","additive":true,"intermediate_types":[]},"measurement":{"kind":"measurement","additive":false,"intermediate_types":[]}})
    }
    fn q() -> Value {
        json!({"status":"ok","reason":null,"frame":{"subject":{"entity":"X","all":false,"member_of":null,"part_of":null},"subject_quote":"X","context":[],"configuration":[],"requests":[{"quote":"mass","attribute":"mass","modality":"state","value":null,"value_type":null,"path":"direct","extent":"unspecified","basis_requirement":"any","measured_attribute":null,"measurement_method":null,"measured_attribute_scope":null,"measurement_method_scope":null}],"premises":[]}})
    }
    #[test]
    fn final_revision_unsupported_never_falls_back_to_a_valid_initial_document() {
        let ontology =
            json!({"color":{"kind":"property","additive":false,"intermediate_types":[]}});
        let source = "equipo Nube rojo";
        let draft = json!({"status":"ok","document":{"certain":[{"quote":source,"subject":{"mention_quote":"equipo Nube","type_quote":"equipo","name":"Nube","owners":[],"scope":"single"},"time":"state","context":[],"configuration":[],"statement":{"type":"property","attribute":"color","value":{"type":"text","value":"rojo"},"affirmed":true,"basis":"described_fact"}}],"readings":[],"readings_complete":true}});
        let o = vocabulary(&ontology).unwrap();
        let legacy =
            contract::grounded_references::translate("source", source, &ontology, &draft).unwrap();
        assert!(decoded("source", source, &legacy, &o).is_ok());
        let mut schema = contract::grounded_references::schema("source", &ontology).unwrap();
        schema["$id"] = json!("iromem-grounded-revision-v1-source");
        let mut payload = Profile::GroundedReferenceV1
            .payload("source", source, &ontology)
            .unwrap();
        payload["messages"][1]["content"]=json!(json!({"text":source,"ontology":ontology,"output_schema":schema,"previous_output":draft,"local_validation_error":null}).to_string());
        let last = json!({"status":"unsupported","reason":"unrepresentable final revision"});
        let body = json!({"model":"openai/gpt-4.1","provider":"OpenAI","choices":[{"finish_reason":"stop","message":{"content":last.to_string()}}]});
        let raw = contract::checked_content(&body, &payload).unwrap();
        assert_eq!(raw, last);
        let legacy =
            contract::grounded_references::translate("source", source, &ontology, &raw).unwrap();
        assert!(decoded("source", source, &legacy, &o).is_err());
    }
    #[test]
    fn grounded_reference_types_cross_original_parser_and_preserve_intermediate_evidence() {
        let ontology = json!({"water_supply":{"kind":"property","additive":false,"intermediate_types":["reservoir"]}});
        let subject = json!({"mention_quote":"huerto Nube","type_quote":"huerto","name":"Nube","owners":[],"scope":"single"});
        let source = "El huerto Nube recibe agua del depósito R4.";
        let raw = json!({"status":"ok","document":{"certain":[{"quote":source,"subject":subject,"time":"state","context":[],"configuration":[],"statement":{"type":"flow","attribute":"water_supply","values":[{"type":"entity","reference":{"mention_quote":"depósito R4","type_quote":"depósito","name":"R4","owners":[],"scope":"single"},"ontology_type":{"type_id":"reservoir","quote":"depósito R4"}}],"exhaustive":false,"direction":"immediate","affirmed":true,"basis":"actual_assignment"}}],"readings":[],"readings_complete":true}});
        let o = vocabulary(&ontology).unwrap();
        let legacy =
            contract::grounded_references::translate("source", source, &ontology, &raw).unwrap();
        let normalized = decoded("source", source, &legacy, &o).unwrap();
        assert_eq!(
            normalized["certain_facts"][0]["datum"]["value_type"],
            "reservoir"
        );
        let question = "¿De qué red recibe agua el huerto Nube?";
        let raw = json!({"status":"ok","question":{"subject":subject,"context":[],"configuration":[],"assumptions":[],"requested":[{"quote":question,"time":"state","extent":"unspecified","slot":{"type":"property","attribute":"water_supply","basis":"any","path":"transitive","type_filter":{"type_id":"network","quote":"red"}}}]}});
        let legacy =
            contract::grounded_references::translate("question", question, &ontology, &raw)
                .unwrap();
        let q = decoded("question", question, &legacy, &o).unwrap();
        let q = record_question(&q["frame"], &o).unwrap();
        let result = project(&o, &q, &normalized).unwrap();
        // V1/V5 A includes qualified partial data; C is configuration contrast.
        assert_eq!(result["relation"], "A");
        assert_eq!(result["selected"], true);
        assert_eq!(result["qualification_required"], true);
        let assertions = result["record_knowledge"]["invariant_assertions"]
            .as_array()
            .unwrap();
        assert_eq!(assertions.len(), 1);
        assert_eq!(
            assertions[0]["record_asserts"]["datum"]["value_type"],
            "reservoir"
        );
        assert_eq!(assertions[0]["record_asserts"]["flow_scope"], "immediate");
        assert_eq!(
            result["record_knowledge"]["identity_resolution_claimed"],
            false
        );
    }
    #[test]
    fn query_adapter_has_no_fact_completion_and_checks_literal_subject_and_slot() {
        let o = vocabulary(&ontology_value()).unwrap();
        let r = decoded("question", "X mass", &q(), &o).unwrap();
        assert_eq!(r["frame"]["requests"][0]["attribute"], "mass");
        assert_eq!(r["frame"]["premises"], json!([]));
        assert!(decoded("question", "Y mass", &q(), &o).is_err());
        assert!(decoded("question", "X weight", &q(), &o).is_err());
    }
    #[test]
    fn conditions_reject_duplicate_instead_of_overwriting() {
        assert!(
            conditions(&json!([{"key":"route","value":"a"},{"key":"route","value":"b"}])).is_err()
        );
    }
    #[test]
    fn unsupported_is_explicit_never_empty_reliable_frame() {
        let o = vocabulary(&ontology_value()).unwrap();
        assert!(
            decoded(
                "source",
                "x",
                &json!({"status":"unsupported","reason":"outside vocabulary","document":null}),
                &o
            )
            .unwrap_err()
            .starts_with("extractor_unsupported")
        );
        assert!(
            decoded(
                "question",
                "x",
                &json!({"status":"unsupported","reason":"bad","frame":q()["frame"]}),
                &o
            )
            .is_err()
        );
    }
    #[test]
    fn payloads_are_rebuilt_only_from_kind_text_and_ontology() {
        let v = json!({"ontology":ontology_value(),"questions":{"q":{"text":"X mass","frame":{"hidden_gold":"never send"}}},"assertions":{"a":{"text":"mass of X unknown","alternatives":[{"label":"never send"}]}}});
        let r = requests(&v).unwrap();
        assert_eq!(r.len(), 2);
        assert!(!r[0]["payload"].to_string().contains("never send"));
        assert!(!r[1]["payload"].to_string().contains("hidden_gold"));
    }
    #[test]
    fn pilot_roster_uses_spaced_ordinals_only_and_never_gold() {
        let mut assertions = serde_json::Map::new();
        let mut questions = serde_json::Map::new();
        for i in 0..83 {
            assertions.insert(
                format!("a{i:03}"),
                json!({"text":format!("source{i}"),"hidden_gold":"never"}),
            );
        }
        for i in 0..33 {
            questions.insert(
                format!("q{i:03}"),
                json!({"text":format!("question{i}"),"frame":{"hidden_gold":"never"}}),
            );
        }
        let rows = requests_profile(
            &json!({"ontology":ontology_value(),"assertions":assertions,"questions":questions}),
            Profile::OpenAiPilotV3,
        )
        .unwrap();
        assert_eq!(
            rows.iter()
                .map(|r| r["id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            contract::PILOT_IDS
        );
        assert_eq!(
            rows.iter()
                .map(|r| r["text"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec![
                "source0",
                "question0",
                "source27",
                "question10",
                "source55",
                "question21",
                "source82",
                "question32"
            ]
        );
        for r in rows {
            assert!(!r["payload"].to_string().contains("hidden_gold"));
        }
    }
    #[test]
    fn typed_clause_translation_crosses_the_same_legacy_adapter_without_domain_relaxation() {
        let ontology = json!({"requires":{"kind":"property","additive":false,"intermediate_types":[]},"contact_phone":{"kind":"property","additive":false,"intermediate_types":[]},"mass":{"kind":"quantity","additive":true,"intermediate_types":[]},"measurement":{"kind":"measurement","additive":false,"intermediate_types":[]}});
        let text = "X no exige teléfono.";
        let subject = json!({"kind":"object","name":"X","owners":[],"scope":"single"});
        let raw = json!({"status":"ok","document":{"certain":[{"quote":text,"subject":subject,"context":[],"configuration":[],"time":"state","statement":{"type":"requirement","item_attribute":"contact_phone","required":false,"basis":"permission_rule"}}],"readings":[],"readings_complete":true}});
        let legacy = contract::typed_clauses::translate("source", text, &ontology, &raw).unwrap();
        let o = vocabulary(&ontology).unwrap();
        let normalized = decoded("source", text, &legacy, &o).unwrap();
        assert_eq!(normalized["certain_facts"][0]["positive"], false);
        assert_eq!(
            normalized["certain_facts"][0]["datum"]["text"],
            "contact_phone"
        );
        let question = "¿Se midió la masa de X?";
        let raw = json!({"status":"ok","question":{"subject":subject,"context":[],"configuration":[],"requested":[{"quote":question,"time":"occurred","extent":"unspecified","slot":{"type":"procedure","quantity":"mass","basis":"any"}}],"assumptions":[]}});
        let legacy =
            contract::typed_clauses::translate("question", question, &ontology, &raw).unwrap();
        let normalized = decoded("question", question, &legacy, &o).unwrap();
        assert_eq!(
            normalized["frame"]["requests"][0]["attribute"],
            "measurement"
        );
        assert_eq!(
            normalized["frame"]["requests"][0]["measurement_method_scope"],
            "any"
        );
        let mut invalid = legacy;
        invalid["frame"]["requests"][0]["measured_attribute_scope"] = json!("any");
        assert!(decoded("question", question, &invalid, &o).is_err());
    }
}
