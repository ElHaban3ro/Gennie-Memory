//! Offline audit of frozen M5 campaigns. Exact-ID scoring is kept separate from truth.

use crate::campaign::digest;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("falta texto {key}"))
}

fn list<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("falta lista {key}"))
}

fn strings(value: &Value, key: &str) -> Result<Vec<String>, String> {
    let result = list(value, key)?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{key} contiene un ID inválido"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if result.iter().collect::<HashSet<_>>().len() != result.len() {
        return Err(format!("{key} contiene IDs duplicados"));
    }
    Ok(result)
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?)
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn read_jsonl(path: &Path) -> Result<Vec<Value>, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line)
                .map_err(|error| format!("{} línea {}: {error}", path.display(), index + 1))
        })
        .collect()
}

fn quantile(values: &[f64], fraction: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    let position = (ordered.len() - 1) as f64 * fraction;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    Some(ordered[lower] + (ordered[upper] - ordered[lower]) * position.fract())
}

fn distribution(values: &[f64]) -> Value {
    json!({"count":values.len(), "p50":quantile(values,0.5), "p95":quantile(values,0.95),
        "p99":quantile(values,0.99), "sum":values.iter().sum::<f64>()})
}

fn wilson(correct: usize, total: usize) -> Value {
    if total == 0 {
        return Value::Null;
    }
    let z = 1.959_963_984_540_054_f64;
    let n = total as f64;
    let p = correct as f64 / n;
    let denominator = 1.0 + z * z / n;
    let center = (p + z * z / (2.0 * n)) / denominator;
    let half = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / denominator;
    json!({"lower":(center-half).max(0.0), "upper":(center+half).min(1.0),
        "confidence":0.95, "unit":"case", "independence_assumed_not_established":true})
}

#[derive(Default)]
struct Counts {
    cases: usize,
    correct: usize,
    positive: usize,
    positive_correct: usize,
    negative: usize,
    negative_empty: usize,
    negative_selected: usize,
    positive_abstentions: usize,
    positive_wrong_selection: usize,
    degraded: usize,
    covered: usize,
    top1: usize,
    top5: usize,
    negative_candidate_empty: usize,
    forbidden_nonempty: usize,
    forbidden_hits: usize,
}

impl Counts {
    fn add(&mut self, outcome: &Outcome, forbidden: &[String]) {
        self.cases += 1;
        self.correct += usize::from(outcome.correct);
        self.degraded += usize::from(!outcome.complete);
        self.forbidden_nonempty += usize::from(!forbidden.is_empty());
        self.forbidden_hits += usize::from(
            forbidden
                .iter()
                .any(|id| outcome.selected.contains(id) || outcome.candidate_ids.contains(id)),
        );
        if outcome.expected.is_empty() {
            self.negative += 1;
            self.negative_empty += usize::from(outcome.correct);
            self.negative_selected += usize::from(!outcome.selected.is_empty());
            self.negative_candidate_empty += usize::from(outcome.candidate_ids.is_empty());
        } else {
            self.positive += 1;
            self.positive_correct += usize::from(outcome.correct);
            self.positive_abstentions +=
                usize::from(outcome.complete && outcome.selected.is_empty());
            self.positive_wrong_selection +=
                usize::from(!outcome.selected.is_empty() && outcome.selected != outcome.expected);
            if let Some(rank) = outcome
                .candidate_ids
                .iter()
                .position(|id| outcome.expected.contains(id))
            {
                self.covered += 1;
                self.top1 += usize::from(rank == 0);
                self.top5 += usize::from(rank < 5);
            }
        }
    }

    fn report(&self) -> Value {
        json!({"cases":self.cases,"strict_id_correct":self.correct,
            "positive":self.positive,"positive_strict_id_correct":self.positive_correct,
            "negative":self.negative,"negative_complete_empty":self.negative_empty,
            "negative_nonempty_selection":self.negative_selected,
            "positive_complete_abstention":self.positive_abstentions,
            "positive_nonempty_id_disagreement":self.positive_wrong_selection,
            "degraded":self.degraded,"positive_candidate_coverage":self.covered,
            "positive_expected_top1":self.top1,"positive_expected_top5":self.top5,
            "negative_candidate_empty":self.negative_candidate_empty,
            "strict_id_wilson_95":wilson(self.correct,self.cases),
            "positive_strict_id_wilson_95":wilson(self.positive_correct,self.positive),
            "negative_complete_empty_wilson_95":wilson(self.negative_empty,self.negative),
            "selection_given_coverage": {"correct":self.positive_correct,"covered":self.covered},
            "forbidden_label_nonempty_cases":self.forbidden_nonempty,
            "forbidden_labels_all_empty":self.forbidden_nonempty==0,
            "forbidden_label_hit_cases":self.forbidden_hits})
    }
}

struct Outcome {
    expected: Vec<String>,
    selected: Vec<String>,
    candidate_ids: Vec<String>,
    candidates: Vec<Value>,
    complete: bool,
    correct: bool,
    scope: String,
}

struct AuditedCampaign {
    report: Value,
    outcomes: BTreeMap<String, Outcome>,
}

fn normalize_source(source: Option<&Value>) -> Value {
    let mut source = source.cloned().unwrap_or(Value::Null);
    if let Some(object) = source.as_object_mut() {
        object.entry("locator").or_insert(Value::Null);
    }
    source
}

fn render_seed(memories: &[Value]) -> Result<Option<String>, String> {
    if memories.is_empty() {
        return Ok(None);
    }
    memories
        .iter()
        .map(|memory| {
            let revision = memory
                .get("revision")
                .and_then(Value::as_u64)
                .ok_or("revisión de semilla inválida")?;
            let mut result = format!(
                "[{}@{}] {}",
                text(memory, "memory_id")?,
                revision,
                text(memory, "summary")?
            );
            if let Some(conditions) = memory.get("conditions").filter(|v| !v.is_null()) {
                result.push_str(&format!(
                    "\n  Condiciones: {}",
                    conditions.as_str().ok_or("condiciones inválidas")?
                ));
            }
            if let Some(source) = memory.get("source").filter(|v| !v.is_null()) {
                result.push_str(&format!(
                    "\n  Fuente declarada: {} / {}",
                    text(source, "kind")?,
                    text(source, "ref")?
                ));
                if let Some(locator) = source.get("locator").filter(|v| !v.is_null()) {
                    result.push_str(&format!(
                        " / {}",
                        locator.as_str().ok_or("locator inválido")?
                    ));
                }
            }
            Ok(result)
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|items| Some(items.join("\n")))
}

fn check_memory(
    item: &Value,
    search: &Value,
    corpus: &HashMap<String, Value>,
    seed: bool,
) -> Result<(), String> {
    let id = text(item, "memory_id")?;
    let original = corpus
        .get(id)
        .ok_or_else(|| format!("ID {id} no existe en corpus"))?;
    for key in ["scope_kind", "scope_id"] {
        if original.get(key) != search.get(key) {
            return Err(format!("{id}: ámbito solicitado distinto ({key})"));
        }
        if seed && item.get(key) != original.get(key) {
            return Err(format!("{id}: ámbito de semilla distinto ({key})"));
        }
    }
    for key in ["revision", "state", "summary", "conditions"] {
        if item.get(key).unwrap_or(&Value::Null) != original.get(key).unwrap_or(&Value::Null) {
            return Err(format!("{id}: {key} no coincide con corpus"));
        }
    }
    if text(original, "state")? != "active" {
        return Err(format!("{id}: memoria inactiva"));
    }
    if normalize_source(item.get("source")) != normalize_source(original.get("source")) {
        return Err(format!("{id}: fuente declarada no coincide con corpus"));
    }
    let as_of = text(search, "as_of")?;
    if original
        .get("valid_from")
        .and_then(Value::as_str)
        .is_some_and(|start| as_of < start)
        || original
            .get("valid_until")
            .and_then(Value::as_str)
            .is_some_and(|end| as_of >= end)
    {
        return Err(format!("{id}: memoria fuera de vigencia"));
    }
    if let Some(entity) = search.get("entity_id").filter(|v| !v.is_null()) {
        if original.get("entity_id") != Some(entity) {
            return Err(format!("{id}: entidad solicitada distinta"));
        }
    }
    Ok(())
}

fn add_number(
    values: &mut Vec<f64>,
    value: Option<&Value>,
    field: &str,
    required: bool,
) -> Result<(), String> {
    match value.filter(|v| !v.is_null()) {
        Some(value) => {
            let number = value
                .as_f64()
                .filter(|v| v.is_finite() && *v >= 0.0)
                .ok_or_else(|| format!("{field} no es número finito no negativo"))?;
            values.push(number);
        }
        None if required => return Err(format!("falta {field}")),
        None => (),
    }
    Ok(())
}

fn audit_campaign(base: &Path, config: &Value) -> Result<AuditedCampaign, String> {
    audit_campaign_with_auxiliary_projection(base, config, false)
}

/// For diagnostic projections whose caller has separately verified source gold.
/// Never repairs requests, IDs, events, text or non-null label metadata.
pub(crate) fn audit_projected_source_metadata(
    base: &Path,
    config: &Value,
) -> Result<Value, String> {
    audit_campaign_with_auxiliary_projection(base, config, true).map(|c| c.report)
}

fn audit_campaign_with_auxiliary_projection(
    base: &Path,
    config: &Value,
    project_missing_auxiliary: bool,
) -> Result<AuditedCampaign, String> {
    let campaign_id = text(config, "id")?;
    let corpus_path = base.join(text(config, "corpus")?);
    let cases_path = base.join(text(config, "cases")?);
    let journal_path = base.join(text(config, "journal")?);
    let allow_partial = config
        .get("allow_partial")
        .and_then(Value::as_bool)
        .ok_or("falta allow_partial")?;
    let provenance = text(config, "provenance")?;
    let corpus_json = read_json(&corpus_path)?;
    let mut corpus = HashMap::<String, Value>::new();
    let mut revisions = HashSet::new();
    for memory in corpus_json.as_array().ok_or("corpus no es lista")? {
        let id = text(memory, "id")?;
        let revision = memory
            .get("revision")
            .and_then(Value::as_u64)
            .ok_or("revisión inválida en corpus")?;
        if !revisions.insert((id.to_owned(), revision)) {
            return Err(format!("revisión duplicada en corpus: {id}@{revision}"));
        }
        if corpus
            .get(id)
            .and_then(|v| v.get("revision"))
            .and_then(Value::as_u64)
            .is_none_or(|old| revision > old)
        {
            corpus.insert(id.to_owned(), memory.clone());
        }
    }
    if corpus.is_empty() {
        return Err("corpus vacío".into());
    }
    let mut cases = BTreeMap::new();
    let mut projected_metadata = Vec::new();
    for mut case in read_jsonl(&cases_path)? {
        let id = text(&case, "id")?.to_owned();
        let expected = strings(&case, "expected_ids")?;
        strings(&case, "forbidden_ids")?;
        if project_missing_auxiliary
            && expected.len() == 1
            && ["expected_fact", "expected_condition", "expected_source"]
                .iter()
                .all(|key| case[key].is_null())
        {
            let memory = corpus
                .get(&expected[0])
                .ok_or("ID esperado inexistente en proyección")?;
            case["expected_fact"] = memory["summary"].clone();
            case["expected_condition"] = memory["conditions"].clone();
            case["expected_source"] = memory["source"].clone();
            projected_metadata.push(json!({"id":id,"memory_id":expected[0],"expected_fact":case["expected_fact"],"expected_condition":case["expected_condition"],"expected_source":case["expected_source"]}));
        }
        if cases.insert(id.clone(), case).is_some() {
            return Err(format!("caso duplicado: {id}"));
        }
    }
    let records = read_jsonl(&journal_path)?;
    let manifest = records
        .first()
        .filter(|row| row.get("kind").and_then(Value::as_str) == Some("manifest"))
        .ok_or("el diario debe comenzar por manifest")?;
    let cases_hash = digest(&cases_path)?;
    if text(manifest, "cases_sha256")? != cases_hash {
        return Err(format!("{campaign_id}: hash de casos distinto"));
    }
    if manifest.get("case_count").and_then(Value::as_u64) != Some(cases.len() as u64) {
        return Err(format!("{campaign_id}: case_count no coincide con casos"));
    }
    let mut starts = HashSet::new();
    let mut finished = BTreeMap::new();
    for record in records.iter().skip(1) {
        let id = text(record, "id")?;
        if !cases.contains_key(id) {
            return Err(format!("evento con caso desconocido: {id}"));
        }
        match text(record, "kind")? {
            "started" => {
                if finished.contains_key(id) || !starts.insert(id.to_owned()) {
                    return Err(format!("started duplicado o fuera de orden: {id}"));
                }
            }
            "finished" => {
                if !starts.contains(id) {
                    return Err(format!("finished sin started: {id}"));
                }
                if finished.insert(id.to_owned(), record).is_some() {
                    return Err(format!("finished duplicado: {id}"));
                }
            }
            other => return Err(format!("evento no admitido: {other}")),
        }
    }
    if starts.len() != finished.len() {
        return Err(format!(
            "{campaign_id}: hay llamadas iniciadas sin resultado; no se infiere abstención"
        ));
    }
    if finished.is_empty() || (!allow_partial && finished.len() != cases.len()) {
        return Err(format!("{campaign_id}: campaña incompleta"));
    }
    let mut counts = Counts::default();
    let mut scopes = BTreeMap::<String, Counts>::new();
    let mut segments = BTreeMap::<String, Counts>::new();
    let mut outcomes = BTreeMap::new();
    let mut error_codes = BTreeMap::<String, usize>::new();
    let mut models = BTreeMap::<String, usize>::new();
    let mut cost = Vec::new();
    let mut input = Vec::new();
    let mut output = Vec::new();
    let mut seed_bytes = Vec::new();
    let mut elapsed = Vec::new();
    let mut service = Vec::new();
    let mut search_times = Vec::new();
    let mut classifier = Vec::new();
    let mut candidate_count = 0;
    let mut seeded_count = 0;
    let mut disagreements = Vec::new();
    for (id, record) in &finished {
        let case = &cases[id];
        for field in ["expected_ids", "forbidden_ids", "segment"] {
            if record.get(field) != case.get(field) {
                return Err(format!(
                    "{id}: etiqueta {field} difiere del archivo de casos"
                ));
            }
        }
        if case.get("split") != manifest.get("split") {
            return Err(format!("{id}: split difiere del manifiesto"));
        }
        if manifest.get("label_source").is_some()
            && case.get("label_source") != manifest.get("label_source")
        {
            return Err(format!("{id}: label_source difiere del manifiesto"));
        }
        let expected = strings(case, "expected_ids")?;
        let forbidden = strings(case, "forbidden_ids")?;
        let search = case
            .pointer("/request/search")
            .ok_or_else(|| format!("{id}: falta request.search"))?;
        for expected_id in &expected {
            let memory = corpus
                .get(expected_id)
                .ok_or_else(|| format!("{id}: esperado no existe en corpus"))?;
            for field in ["scope_kind", "scope_id"] {
                if memory.get(field) != search.get(field) {
                    return Err(format!("{id}: esperado fuera de ámbito"));
                }
            }
        }
        if expected.len() == 1 {
            let memory = &corpus[&expected[0]];
            if case.get("expected_fact") != memory.get("summary")
                || case.get("expected_condition").unwrap_or(&Value::Null)
                    != memory.get("conditions").unwrap_or(&Value::Null)
                || normalize_source(case.get("expected_source"))
                    != normalize_source(memory.get("source"))
            {
                return Err(format!(
                    "{id}: metadatos de etiqueta no coinciden con corpus"
                ));
            }
        }
        let response = record.get("response").ok_or("falta response")?;
        let status = text(response, "status")?;
        if !["complete", "degraded"].contains(&status) {
            return Err(format!("{id}: status no admitido"));
        }
        let candidates = response
            .pointer("/retrieval/candidates")
            .and_then(Value::as_array)
            .ok_or("faltan candidatos")?;
        let memories = response
            .pointer("/seed/memories")
            .and_then(Value::as_array)
            .ok_or("falta seed.memories")?;
        let mut candidate_ids = Vec::new();
        for candidate in candidates {
            check_memory(candidate, search, &corpus, false)
                .map_err(|error| format!("{id}: candidato: {error}"))?;
            let memory_id = text(candidate, "memory_id")?.to_owned();
            if candidate_ids.contains(&memory_id) {
                return Err(format!("{id}: candidato duplicado"));
            }
            candidate_ids.push(memory_id);
        }
        let mut selected = Vec::new();
        for memory in memories {
            check_memory(memory, search, &corpus, true)
                .map_err(|error| format!("{id}: semilla: {error}"))?;
            let memory_id = text(memory, "memory_id")?.to_owned();
            if !candidate_ids.contains(&memory_id) || selected.contains(&memory_id) {
                return Err(format!("{id}: selección no presentada o duplicada"));
            }
            selected.push(memory_id);
        }
        if status == "degraded" && !selected.is_empty() {
            return Err(format!("{id}: resultado degradado con semilla no vacía"));
        }
        let rendered = render_seed(memories)?;
        if response.pointer("/seed/seed_text").unwrap_or(&Value::Null) != &json!(rendered) {
            return Err(format!("{id}: seed_text no coincide con sus memorias"));
        }
        if response.get("selected_ids").is_some() && response["selected_ids"] != json!(selected) {
            return Err(format!(
                "{id}: selected_ids difiere del conjunto de semilla"
            ));
        }
        if response.get("selected_id").unwrap_or(&Value::Null) != &json!(selected.first()) {
            return Err(format!("{id}: selected_id difiere de semilla"));
        }
        candidate_count += candidates.len();
        seeded_count += memories.len();
        seed_bytes.push(rendered.as_ref().map_or(0, |s| s.len()) as f64);
        let scope = format!(
            "{}/{}",
            text(search, "scope_kind")?,
            text(search, "scope_id")?
        );
        let outcome = Outcome {
            correct: status == "complete" && selected == expected,
            complete: status == "complete",
            expected,
            selected,
            candidate_ids,
            candidates: candidates.clone(),
            scope: scope.clone(),
        };
        counts.add(&outcome, &forbidden);
        scopes.entry(scope).or_default().add(&outcome, &forbidden);
        segments
            .entry(text(case, "segment")?.to_owned())
            .or_default()
            .add(&outcome, &forbidden);
        if !outcome.correct {
            disagreements.push(json!({"id":id,"query":search.get("query"),"expected_ids":outcome.expected,
                "selected_ids":outcome.selected,"status":status,"error_code":response.get("error_code"),
                "selected_summaries":memories.iter().map(|m|m.get("summary")).collect::<Vec<_>>(),
                "meaning":"strict_id_disagreement_not_a_measure_of_answer_truth"}));
        }
        *error_codes
            .entry(
                response
                    .get("error_code")
                    .and_then(Value::as_str)
                    .unwrap_or("none")
                    .to_owned(),
            )
            .or_default() += 1;
        *models
            .entry(
                response
                    .get("served_model")
                    .and_then(Value::as_str)
                    .unwrap_or("not_reported")
                    .to_owned(),
            )
            .or_default() += 1;
        add_number(
            &mut cost,
            response.get("reported_cost_usd"),
            "reported_cost_usd",
            false,
        )?;
        add_number(
            &mut input,
            response.get("input_tokens"),
            "input_tokens",
            false,
        )?;
        add_number(
            &mut output,
            response.get("output_tokens"),
            "output_tokens",
            false,
        )?;
        add_number(&mut elapsed, record.get("elapsed_ms"), "elapsed_ms", true)?;
        add_number(
            &mut service,
            response.pointer("/timings/total_ms"),
            "timings.total_ms",
            true,
        )?;
        add_number(
            &mut search_times,
            response.pointer("/timings/search_ms"),
            "timings.search_ms",
            true,
        )?;
        add_number(
            &mut classifier,
            response.pointer("/timings/classifier_ms"),
            "timings.classifier_ms",
            true,
        )?;
        outcomes.insert(id.clone(), outcome);
    }
    Ok(AuditedCampaign {
        report: json!({"id":campaign_id,"provenance":provenance,"manifest":manifest,
        "paths":{"corpus":config["corpus"],"cases":config["cases"],"journal":config["journal"]},
        "sha256":{"corpus":digest(&corpus_path)?,"cases":cases_hash,"journal":digest(&journal_path)?},
        "planned_cases":cases.len(),"finished_cases":finished.len(),"not_executed_cases":cases.len()-finished.len(),
        "allow_partial":allow_partial,"metrics":counts.report(),
        "auxiliary_metadata_projection":{"enabled":project_missing_auxiliary,"input_auxiliary_metadata_was_valid":projected_metadata.is_empty(),"projected_cases":projected_metadata,"files_modified":false,"requests_permitted_ids_and_results_modified":false,"note":"If enabled, only all-null auxiliary fields for one frozen expected ID are derived from corpus in memory. Source-gold binding must be checked by diagnostic caller. Ordinary audit-m5 remains strict."},
        "by_scope":scopes.iter().map(|(key,value)|(key.clone(),value.report())).collect::<BTreeMap<_,_>>(),
        "by_segment":segments.iter().map(|(key,value)|(key.clone(),value.report())).collect::<BTreeMap<_,_>>(),
        "models":models,"error_codes":error_codes,"strict_disagreements":disagreements,
        "usage":{"reported_cost_usd":if cost.is_empty(){None}else{Some(cost.iter().sum::<f64>())},"cost_reported_cases":cost.len(),
            "cost_unreported_cases":finished.len()-cost.len(),"input_tokens":distribution(&input),
            "output_tokens":distribution(&output),"seed_utf8_bytes":distribution(&seed_bytes),
            "seed_tokens":Value::Null,"seed_tokens_measured":false},
        "latency_ms":{"runner_elapsed":distribution(&elapsed),"service_total":distribution(&service),
            "search":distribution(&search_times),"classifier":distribution(&classifier)},
        "integrity":{"valid":true,"candidates_checked":candidate_count,"seed_memories_checked":seeded_count,
            "scope_revision_state_vigency_and_entity_checked":true,"summary_conditions_declared_source_checked":true,
            "seed_text_checked":true,"external_source_verified":false,"answer_truth_measured":false}}),
        outcomes,
    })
}

fn compare(config: &Value, campaigns: &BTreeMap<String, AuditedCampaign>) -> Result<Value, String> {
    let left = text(config, "left")?;
    let right = text(config, "right")?;
    let a = campaigns
        .get(left)
        .ok_or_else(|| format!("comparación sin campaña {left}"))?;
    let b = campaigns
        .get(right)
        .ok_or_else(|| format!("comparación sin campaña {right}"))?;
    let mut matrix = [0_usize; 4];
    let mut consensus = Counts::default();
    let mut identical = 0;
    let mut paired_complete = 0;
    let mut left_complete_correct = 0;
    let mut right_complete_correct = 0;
    for (id, x) in &a.outcomes {
        let Some(y) = b.outcomes.get(id) else {
            continue;
        };
        if x.expected != y.expected || x.scope != y.scope {
            return Err(format!("{id}: etiquetas o ámbitos no comparables"));
        }
        matrix[usize::from(x.correct) * 2 + usize::from(y.correct)] += 1;
        identical += usize::from(x.candidates == y.candidates);
        if x.complete && y.complete {
            paired_complete += 1;
            left_complete_correct += usize::from(x.correct);
            right_complete_correct += usize::from(y.correct);
        }
        let selected = if x.selected == y.selected {
            x.selected.clone()
        } else {
            Vec::new()
        };
        let complete = x.complete && y.complete;
        let outcome = Outcome {
            correct: complete && selected == x.expected,
            expected: x.expected.clone(),
            selected,
            candidate_ids: x.candidate_ids.clone(),
            candidates: Vec::new(),
            complete,
            scope: x.scope.clone(),
        };
        consensus.add(&outcome, &[]);
    }
    let pairs = matrix.iter().sum::<usize>();
    if pairs == 0 {
        return Err("comparación sin casos compartidos".into());
    }
    Ok(
        json!({"id":text(config,"id")?,"left":left,"right":right,"provenance":text(config,"provenance")?,
        "shared_cases":pairs,"identical_candidate_lists":identical,
        "strict_id_matrix":{"both_disagree":matrix[0],"right_only_correct":matrix[1],"left_only_correct":matrix[2],"both_correct":matrix[3]},
        "both_complete":{"cases":paired_complete,"left_strict_id_correct":left_complete_correct,"right_strict_id_correct":right_complete_correct},
        "retrospective_consensus":{"policy":"keep ID only when both completed and selected the same ID; otherwise empty or degraded",
            "diagnostic_only":true,"promoted":false,"metrics":consensus.report(),
            "warning":"Computed from existing outputs, not a new campaign; aggregate accuracy does not establish answer truth or safe deployment."}}),
    )
}

/// Audit configured frozen campaigns without database access or provider calls.
pub fn run(suite: &Path, output: &Path) -> Result<(), String> {
    let config = read_json(suite)?;
    if text(&config, "version")? != "m5-closeout-v1" {
        return Err("versión de suite no admitida".into());
    }
    let base: PathBuf = suite
        .parent()
        .unwrap_or(Path::new("."))
        .join(text(&config, "base_dir")?);
    let mut campaigns = BTreeMap::new();
    for campaign in list(&config, "campaigns")? {
        let id = text(campaign, "id")?.to_owned();
        if campaigns.contains_key(&id) {
            return Err(format!("campaña duplicada: {id}"));
        }
        campaigns.insert(id, audit_campaign(&base, campaign)?);
    }
    let comparisons = list(&config, "comparisons")?
        .iter()
        .map(|item| compare(item, &campaigns))
        .collect::<Result<Vec<_>, _>>()?;
    let report = json!({"version":"m5-closeout-audit-v1","suite_sha256":digest(suite)?,"status":"complete",
        "method":{"provider_calls":0,"database_access":false,"labels_rewritten":false,
            "strict_id_scoring":"complete response and ordered seed IDs exactly equal expected_ids",
            "negative_selection_is_not_automatically_falsehood":true,"final_answer_truth_measured":false,
            "uncertainty":"Wilson intervals describe cases under an independence assumption not established; campaigns, repeated prompts and subjects are not pooled as independent observations.",
            "cost":"Sum of provider-reported amounts per campaign; missing cost is not imputed as zero. No combined quality score across campaigns.",
            "source":"Declared fixture source fidelity only; no external provenance verification.",
            "timestamps":"Fixture UTC timestamps are compared lexicographically in their fixed RFC 3339 format."},
        "campaigns":campaigns.values().map(|campaign|&campaign.report).collect::<Vec<_>>(),"comparisons":comparisons});
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        output,
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())? + "\n",
    )
    .map_err(|error| format!("{}: {error}", output.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        config: Value,
        records: Vec<Value>,
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "iromem-closeout-test-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&root).unwrap();
            let source = json!({"kind":"user","ref":"synthetic://test"});
            let memory = json!({"id":"m1","revision":1,"scope_kind":"conversation","scope_id":"s1","state":"active","summary":"Aún no se firmó la licencia.","conditions":null,"source":source,"valid_from":null,"valid_until":null,"entity_id":null});
            fs::write(root.join("corpus.json"), json!([memory]).to_string()).unwrap();
            let case = json!({"id":"q1","expected_ids":[],"forbidden_ids":[],"segment":"conversation/pending","split":"development","label_source":"agent_synthetic_unreviewed","request":{"search":{"query":"¿Qué licencia se firmó?","scope_kind":"conversation","scope_id":"s1","entity_id":null,"as_of":"2026-10-17T12:00:00.000Z"}}});
            fs::write(root.join("cases.jsonl"), case.to_string() + "\n").unwrap();
            let candidate = json!({"memory_id":"m1","revision":1,"state":"active","summary":memory["summary"],"conditions":null,"source":source,"bm25":-1});
            let mut seed = candidate.clone();
            seed.as_object_mut().unwrap().remove("bm25");
            seed["scope_kind"] = json!("conversation");
            seed["scope_id"] = json!("s1");
            let seed_text = render_seed(&[seed.clone()]).unwrap();
            let finished = json!({"kind":"finished","id":"q1","expected_ids":[],"forbidden_ids":[],"segment":"conversation/pending","elapsed_ms":2.0,"response":{"status":"complete","selected_id":"m1","served_model":"test","reported_cost_usd":0.001,"input_tokens":10,"output_tokens":0,"error_code":null,"retrieval":{"candidates":[candidate]},"seed":{"memories":[seed],"seed_text":seed_text},"timings":{"search_ms":0.1,"classifier_ms":1.0,"total_ms":1.2}}});
            let records = vec![
                json!({"kind":"manifest","cases_sha256":digest(&root.join("cases.jsonl")).unwrap(),"case_count":1,"split":"development","label_source":"agent_synthetic_unreviewed"}),
                json!({"kind":"started","id":"q1"}),
                finished,
            ];
            Self {
                root,
                config: json!({"id":"test","corpus":"corpus.json","cases":"cases.jsonl","journal":"journal.jsonl","allow_partial":false,"provenance":"unit test"}),
                records,
            }
        }
        fn audit(&self) -> Result<AuditedCampaign, String> {
            fs::write(
                self.root.join("journal.jsonl"),
                self.records
                    .iter()
                    .map(Value::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\n",
            )
            .unwrap();
            audit_campaign(&self.root, &self.config)
        }
    }

    #[test]
    fn explicit_diagnostic_projection_fills_only_null_auxiliary_metadata_without_rewriting_files() {
        let mut f = Fixture::new();
        let mut case = read_jsonl(&f.root.join("cases.jsonl")).unwrap().remove(0);
        case["expected_ids"] = json!(["m1"]);
        fs::write(f.root.join("cases.jsonl"), case.to_string() + "\n").unwrap();
        f.records[0]["cases_sha256"] = json!(digest(&f.root.join("cases.jsonl")).unwrap());
        f.records[2]["expected_ids"] = json!(["m1"]);
        assert!(f.audit().err().unwrap().contains("metadatos de etiqueta"));
        let before = digest(&f.root.join("cases.jsonl")).unwrap();
        let projected = audit_projected_source_metadata(&f.root, &f.config).unwrap();
        assert_eq!(
            projected["auxiliary_metadata_projection"]["input_auxiliary_metadata_was_valid"],
            false
        );
        assert_eq!(
            projected["auxiliary_metadata_projection"]["projected_cases"][0]["id"],
            "q1"
        );
        assert_eq!(before, digest(&f.root.join("cases.jsonl")).unwrap());
        assert!(f.audit().is_err());
        case["expected_fact"] = json!("An incorrect supplied fact");
        fs::write(f.root.join("cases.jsonl"), case.to_string() + "\n").unwrap();
        f.records[0]["cases_sha256"] = json!(digest(&f.root.join("cases.jsonl")).unwrap());
        assert!(f.audit().is_err());
        assert!(audit_projected_source_metadata(&f.root, &f.config).is_err());
    }

    #[test]
    fn negative_evidence_remains_id_disagreement_without_claiming_falsehood() {
        let f = Fixture::new();
        let report = f.audit().unwrap().report;
        assert_eq!(report["metrics"]["negative_nonempty_selection"], 1);
        assert_eq!(report["metrics"]["strict_id_correct"], 0);
        assert_eq!(report["integrity"]["answer_truth_measured"], false);
        assert_eq!(report["metrics"]["forbidden_labels_all_empty"], true);
        assert!(report["usage"]["seed_utf8_bytes"]["p50"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn missing_cost_is_unknown_rather_than_zero() {
        let mut f = Fixture::new();
        f.records[2]["response"]["reported_cost_usd"] = Value::Null;
        let report = f.audit().unwrap().report;
        assert!(report["usage"]["reported_cost_usd"].is_null());
        assert_eq!(report["usage"]["cost_unreported_cases"], 1);
    }

    #[test]
    fn rejects_duplicate_finished_event() {
        let mut f = Fixture::new();
        f.records.push(f.records[2].clone());
        assert!(f.audit().err().unwrap().contains("finished duplicado"));
    }

    #[test]
    fn rejects_corrupted_candidate_and_seed_text() {
        let mut f = Fixture::new();
        f.records[2]["response"]["retrieval"]["candidates"][0]["summary"] =
            json!("Se firmó la licencia.");
        assert!(f.audit().err().unwrap().contains("summary no coincide"));
        let mut f = Fixture::new();
        f.records[2]["response"]["seed"]["seed_text"] = json!("Se firmó la licencia.");
        assert!(f.audit().err().unwrap().contains("seed_text no coincide"));
    }

    #[test]
    fn rejects_case_hash_change_and_cross_scope_seed() {
        let mut f = Fixture::new();
        f.records[0]["cases_sha256"] = json!("different");
        assert!(f.audit().err().unwrap().contains("hash de casos distinto"));
        let mut f = Fixture::new();
        f.records[2]["response"]["seed"]["memories"][0]["scope_id"] = json!("s2");
        assert!(
            f.audit()
                .err()
                .unwrap()
                .contains("ámbito de semilla distinto")
        );
    }

    #[test]
    fn incomplete_started_call_is_not_an_empty_answer_even_when_partial_allowed() {
        let mut f = Fixture::new();
        f.config["allow_partial"] = json!(true);
        f.records.pop();
        assert!(f.audit().err().unwrap().contains("iniciadas sin resultado"));
    }

    #[test]
    fn wilson_interval_does_not_turn_perfect_small_sample_into_certainty() {
        let interval = wilson(12, 12);
        assert!((interval["lower"].as_f64().unwrap() - 0.7575).abs() < 0.0001);
        assert!(wilson(0, 0).is_null());
    }
}
