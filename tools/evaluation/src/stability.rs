//! Fixed A/A diagnosis: all repetitions retained, never best-of selection.
use crate::{campaign::digest, evidence};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const IDS: [&str; 8] = [
    "cev5q_066_01",
    "cev5q_066_07",
    "cev5q_067_06",
    "cev5q_068_03",
    "cev5q_069_05",
    "cev5q_070_05",
    "cev5q_071_04",
    "cev5q_073_03",
];
fn read(p: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(p).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn lines(p: &Path) -> Result<Vec<Value>, String> {
    fs::read_to_string(p)
        .map_err(|e| e.to_string())?
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| serde_json::from_str(s).map_err(|e| e.to_string()))
        .collect()
}
fn sets_vary(sets: &[BTreeSet<String>]) -> bool {
    sets.first()
        .is_some_and(|first| sets.iter().any(|s| s != first))
}
pub fn report(directory: &Path, output: &Path) -> Result<(), String> {
    let root = Path::new("tests/fixtures/conversation-evidence-v5");
    let (_, source, _) = evidence::load(root)?;
    let pre = read(&directory.join("preflight.json"))?;
    for path in [
        root.join("development.jsonl"),
        root.join("memories.json"),
        root.join("manifest.json"),
        directory.join("cases.jsonl"),
        Path::new("docs/evaluation/scoped-stability-protocol-2026-10-19.md").to_path_buf(),
    ] {
        if pre["hashes"][path.to_str().ok_or("ruta inválida")?] != digest(&path)? {
            return Err("entrada de estabilidad alterada/sin vínculo previo".into());
        }
    }
    let cases = lines(&directory.join("cases.jsonl"))?;
    let mut expected_cases = Vec::new();
    for repeat in 0..3 {
        for id in IDS {
            let original = source
                .iter()
                .find(|c| c["id"] == id)
                .ok_or("caso original ausente")?;
            let mut c = original.clone();
            c["id"] = json!(format!("{id}__stability_{repeat}"));
            c["original_case_id"] = json!(id);
            c["request"]["provider"] = json!("jev");
            c["request"]["selection_policy"] = json!("per_memory_scoped_v1");
            expected_cases.push(c);
        }
    }
    if cases != expected_cases {
        return Err(
            "estabilidad requiere las24repeticiones fijadas, sin alterar consultas/gold".into(),
        );
    }
    let audit = read(&directory.join("audit.json"))?;
    let campaigns = audit["campaigns"].as_array().ok_or("falta auditoría")?;
    if campaigns.len() != 1
        || campaigns[0]["integrity"]["valid"] != true
        || campaigns[0]["auxiliary_metadata_projection"]["enabled"] != false
    {
        return Err("auditoría ordinaria obligatoria".into());
    }
    for (key, path) in [
        ("cases", directory.join("cases.jsonl")),
        ("journal", directory.join("journal.jsonl")),
        ("corpus", root.join("memories.json")),
    ] {
        if campaigns[0]["sha256"][key] != digest(&path)? {
            return Err("auditoría de estabilidad obsoleta".into());
        }
    }
    let journal = lines(&directory.join("journal.jsonl"))?;
    let manifest = journal.first().ok_or("diario vacío")?;
    if manifest["binary_sha256"] != pre["hashes"]["target/debug/iromem"]
        || manifest["database_sha256"] != pre["database_sha256"]
        || manifest["case_count"] != 24
        || manifest["cases_sha256"] != digest(&directory.join("cases.jsonl"))?
    {
        return Err("binario/base/casos no vinculados a preflight".into());
    }
    let mut events = BTreeMap::new();
    for event in journal.iter().filter(|e| e["kind"] == "finished") {
        let id = event["id"].as_str().ok_or("ID ausente")?;
        if !cases.iter().any(|c| c["id"] == id) || events.insert(id, event).is_some() {
            return Err("evento ajeno/repetido".into());
        }
    }
    if events.len() != 24 {
        return Err("se requieren24operaciones terminadas".into());
    }
    let mut rows = Vec::new();
    let mut known = 0.0;
    let mut missing = 0;
    let mut degraded = 0;
    let mut wrong_operations = 0;
    let mut variations = 0;
    let mut complete_variations = 0;
    let mut empty_with_evidence = 0;
    let mut models = BTreeSet::new();
    for id in IDS {
        let original = source
            .iter()
            .find(|c| c["id"] == id)
            .ok_or("sin original")?;
        let gold = original["expected_ids"]
            .as_array()
            .ok_or("sin gold")?
            .iter()
            .map(|s| s.as_str().map(str::to_string).ok_or("gold inválido"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        let mut candidates = None;
        let mut selections = Vec::new();
        let mut completed_selections = Vec::new();
        let mut details = Vec::new();
        for repeat in 0..3 {
            let key = format!("{id}__stability_{repeat}");
            let event = events.get(key.as_str()).ok_or("repetición omitida")?;
            let response = &event["response"];
            if event["expected_ids"] != original["expected_ids"]
                || response["requested_model"] != "typesafe/jev-1.13"
            {
                return Err("etiquetas/modelo alterados".into());
            }
            if let Some(ref previous) = candidates {
                if *previous != response["retrieval"]["candidates"] {
                    return Err("candidatos cambiaron entre repeticiones".into());
                }
            } else {
                candidates = Some(response["retrieval"]["candidates"].clone());
            }
            let (cost, unknown, _) = crate::score_diagnostic::operation_usage(response)?;
            known += cost.unwrap_or(0.0);
            missing += usize::from(unknown);
            if let Some(model) = response["served_model"].as_str() {
                models.insert(model.to_string());
            }
            if response["status"] == "complete" {
                evidence::per_memory_case(original, response, 8, true)?;
                if response["classification_status"] != "skipped_empty" {
                    crate::factor_audit::inspect_scoped(response)?;
                }
            } else {
                degraded += 1;
            }
            let selected = evidence::selected_set(response)?
                .into_iter()
                .map(str::to_string)
                .collect::<BTreeSet<_>>();
            let wrong = selected.difference(&gold).cloned().collect::<Vec<_>>();
            if response["status"] == "complete" {
                completed_selections.push(selected.clone());
                if !gold.is_empty() && selected.is_empty() {
                    empty_with_evidence += 1;
                }
            }
            wrong_operations += usize::from(!wrong.is_empty());
            details.push(json!({"repeat":repeat,"selected_ids":selected,"unpermitted_ids":wrong,"status":response["status"],"input_tokens":response["input_tokens"],"reported_cost_usd":cost}));
            selections.push(selected);
        }
        let varies = sets_vary(&selections);
        variations += usize::from(varies);
        let complete_varies = sets_vary(&completed_selections);
        complete_variations += usize::from(complete_varies);
        rows.push(json!({"id":id,"query":original["request"]["search"]["query"],"expected_ids":gold,"selected_id_sets_vary":varies,"complete_selected_id_sets_vary":complete_varies,"complete_repetitions":completed_selections.len(),"repetitions":details}));
    }
    let result = json!({"version":"scoped-aa-stability-v1","operations":24,"original_queries":8,"repeats_per_query":3,"known_cost_usd":known,"unknown_cost_operations":missing,"degraded_operations":degraded,"unpermitted_operations":wrong_operations,"queries_with_selected_id_variation":variations,"queries_with_complete_selection_variation":complete_variations,"complete_empty_operations_despite_annotated_evidence":empty_with_evidence,"served_models":models,"cases":rows,"original_labels_changed":false,"fresh_validation":false,"promoted":false,"best_of_selection":false,"wire_payload_captured":false,"payload_identity_basis":"Same canonical query/candidates/provider/policy and deterministic frozen payload builder; repetition IDs remain outside the request.","interpretation":"Observed variation does not identify its provider/model cause; absence of variation in three repetitions does not establish determinism or population reliability. Permitted alternatives can differ without an error. No best run replaces any previous metric.","audit_sha256":digest(&directory.join("audit.json"))?,"journal_sha256":digest(&directory.join("journal.jsonl"))?});
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    file.write_all(
        serde_json::to_string_pretty(&result)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
}
