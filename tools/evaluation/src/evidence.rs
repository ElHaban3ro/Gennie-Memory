//! Source-grounded development rubric. Structural checks never imply semantic review.
use crate::campaign::digest;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const RELATIONS: [&str; 4] = [
    "supports_answer",
    "refutes_premise",
    "pending_or_partial",
    "unrelated",
];
fn read(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn lines(path: &Path) -> Result<Vec<Value>, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())?
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| serde_json::from_str(s).map_err(|e| e.to_string()))
        .collect()
}
fn text<'a>(row: &'a Value, key: &str) -> Result<&'a str, String> {
    row.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("falta {key}"))
}
fn array<'a>(row: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    row.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("falta {key}"))
}
fn check_data(source: &Value, memories: &[Value], cases: &[Value]) -> Result<Value, String> {
    let version = source["version"]
        .as_str()
        .unwrap_or("conversation-evidence-v1");
    let (prefix, expected_memories, as_of) = match version {
        "conversation-evidence-v1" => ("cev1_", 64, "2026-10-17T12:00:00.000Z"),
        "conversation-evidence-v2" => ("cev2_", 224, "2026-10-18T12:00:00.000Z"),
        "conversation-evidence-v3" => ("cev3_", 224, "2026-10-18T12:00:00.000Z"),
        "conversation-evidence-v4" => ("cev4_", 224, "2026-10-19T12:00:00.000Z"),
        "conversation-evidence-v5" => ("cev5_", 224, "2026-10-19T12:00:00.000Z"),
        "conversation-evidence-v6" => ("cev6_", 224, "2026-10-19T12:00:00.000Z"),
        _ => return Err("versión de fuente no admitida".into()),
    };
    if source["synthetic"] != true
        || source["human_review"] != false
        || source["independent_review"] != false
    {
        return Err("procedencia sintética o revisión incorrectas".into());
    }
    let mut statements = BTreeMap::new();
    let mut scopes = BTreeSet::new();
    for conversation in array(source, "conversations")? {
        let scope = text(conversation, "scope_id")?;
        if !scopes.insert(scope) {
            return Err("ámbito duplicado".into());
        }
        for statement in array(conversation, "statements")? {
            let id = text(statement, "id")?;
            if statements
                .insert(id, (scope, text(statement, "text")?))
                .is_some()
            {
                return Err("frase duplicada".into());
            }
        }
    }
    let mut corpus = BTreeMap::new();
    for memory in memories {
        let id = text(memory, "id")?;
        let sid = id.strip_prefix(prefix).ok_or("ID de recuerdo inválido")?;
        let (scope, quote) = statements.get(sid).ok_or("recuerdo sin frase fuente")?;
        if memory["scope_kind"] != "conversation"
            || memory["scope_id"] != *scope
            || memory["summary"] != *quote
            || memory["original_text"] != *quote
            || memory["revision"] != 1
            || memory["state"] != "active"
            || !memory["conditions"].is_null()
            || !memory["entity_id"].is_null()
            || !memory["valid_from"].is_null()
            || !memory["valid_until"].is_null()
            || memory["source"]["kind"] != "user"
        {
            return Err(format!("recuerdo distinto de la fuente: {id}"));
        }
        let (number, line) = sid.split_once('_').ok_or("ID de frase inválido")?;
        if memory["source"]["ref"] != format!("synthetic://{version}/{number}/{line}") {
            return Err("referencia sintética distinta".into());
        }
        if corpus.insert(id, memory).is_some() {
            return Err("recuerdo duplicado".into());
        }
    }
    if corpus.len() != statements.len() {
        return Err("frases sin recuerdo".into());
    }
    let mut ids = BTreeSet::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_scope: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for case in cases {
        let id = text(case, "id")?;
        let relation = text(case, "relation")?;
        let search = &case["request"]["search"];
        let scope = text(search, "scope_id")?;
        if !ids.insert(id)
            || !RELATIONS.contains(&relation)
            || !scopes.contains(scope)
            || case["split"] != "development"
            || case["label_source"] != "agent_synthetic_unreviewed"
            || case["segment"] != format!("conversation/{relation}")
            || search["scope_kind"] != "conversation"
            || search["limit"] != 20
            || search["query_mode"] != "keywords"
            || !search["entity_id"].is_null()
            || search["as_of"] != as_of
        {
            return Err(format!("caso o ámbito inválido: {id}"));
        }
        text(search, "query")?;
        text(case, "rationale")?;
        text(case, "answer_boundary")?;
        let expected = array(case, "expected_ids")?;
        let evidence = array(case, "evidence")?;
        if !array(case, "forbidden_ids")?.is_empty() || !case["expected_condition"].is_null() {
            return Err("etiquetas auxiliares incompatibles".into());
        }
        if relation == "unrelated" {
            if !expected.is_empty()
                || !evidence.is_empty()
                || !case["expected_fact"].is_null()
                || !case["expected_source"].is_null()
            {
                return Err("ausencia con evidencia etiquetada".into());
            }
        } else {
            let multiple_allowed = [
                "conversation-evidence-v4",
                "conversation-evidence-v5",
                "conversation-evidence-v6",
            ]
            .contains(&version);
            if expected.is_empty()
                || evidence.len() != expected.len()
                || (!multiple_allowed && expected.len() != 1)
            {
                return Err("evidencias y conjunto permitido incompatibles".into());
            }
            let mut seen = BTreeSet::new();
            for (mid, annotation) in expected.iter().zip(evidence) {
                let mid = mid.as_str().ok_or("ID esperado inválido")?;
                if !seen.insert(mid) {
                    return Err("evidencia duplicada".into());
                }
                let memory = corpus.get(mid).ok_or("evidencia inexistente")?;
                let sid = text(annotation, "statement_id")?;
                let (source_scope, quote) = statements.get(sid).ok_or("cita inexistente")?;
                if memory["scope_id"] != scope
                    || *source_scope != scope
                    || annotation["memory_id"] != mid
                    || format!("{prefix}{sid}") != mid
                    || annotation["quote"] != *quote
                    || memory["summary"] != *quote
                {
                    return Err(format!("cita o ámbito de evidencia distinto: {id}"));
                }
                if multiple_allowed && !RELATIONS[..3].contains(&text(annotation, "relation")?) {
                    return Err("relación por evidencia no útil".into());
                }
                if expected.len() == 1
                    && (case["expected_fact"] != memory["summary"]
                        || case["expected_source"] != memory["source"])
                {
                    return Err("metadatos de evidencia distintos".into());
                }
            }
            if expected.len() > 1
                && (!case["expected_fact"].is_null() || !case["expected_source"].is_null())
            {
                return Err("varias evidencias requieren metadatos por cita".into());
            }
        }
        if version == "conversation-evidence-v6" {
            coverage(case, &[])?;
        }
        *counts.entry(relation.into()).or_default() += 1;
        *by_scope
            .entry(scope.into())
            .or_default()
            .entry(relation.into())
            .or_default() += 1;
    }
    if scopes.len() != 8
        || memories.len() != expected_memories
        || cases.len() != 64
        || (![
            "conversation-evidence-v4",
            "conversation-evidence-v5",
            "conversation-evidence-v6",
        ]
        .contains(&version)
            && RELATIONS.iter().any(|r| counts.get(*r) != Some(&16)))
        || ([
            "conversation-evidence-v4",
            "conversation-evidence-v5",
            "conversation-evidence-v6",
        ]
        .contains(&version)
            && (RELATIONS
                .iter()
                .any(|r| counts.get(*r).copied().unwrap_or(0) == 0)
                || by_scope.values().any(|c| c.values().sum::<usize>() != 8)))
        || (![
            "conversation-evidence-v4",
            "conversation-evidence-v5",
            "conversation-evidence-v6",
        ]
        .contains(&version)
            && by_scope
                .values()
                .any(|c| RELATIONS.iter().any(|r| c.get(*r) != Some(&2))))
    {
        return Err("tamaño o equilibrio distinto del banco de evidencia".into());
    }
    Ok(
        json!({"valid":true,"version":version,"scopes":scopes.len(),"memories":memories.len(),"cases":cases.len(),
        "by_relation":counts,"by_scope":by_scope,"human_review":false,"independent_review":false,
        "semantic_correctness_verified":false,"split":"development"}),
    )
}
pub(crate) fn load(root: &Path) -> Result<(Vec<Value>, Vec<Value>, Value), String> {
    crate::benchmark::verify_hashes(root)?;
    let manifest = read(&root.join("manifest.json"))?;
    if ![
        "conversation-evidence-v1",
        "conversation-evidence-v2",
        "conversation-evidence-v3",
        "conversation-evidence-v4",
        "conversation-evidence-v5",
        "conversation-evidence-v6",
    ]
    .iter()
    .any(|v| manifest["version"] == *v)
        || manifest["human_review"] != false
        || manifest["independent_review"] != false
    {
        return Err("manifiesto de rúbrica inválido".into());
    }
    if manifest["rubric_path"] != "docs/evaluation/evidence-rubric-v1.md"
        || manifest["rubric_sha256"]
            != digest(
                &Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../docs/evaluation/evidence-rubric-v1.md"),
            )?
    {
        return Err("rúbrica modificada sin versionar el banco".into());
    }
    for name in [
        "source.json",
        "memories.json",
        "development.jsonl",
        "README.md",
        "agent-review.md",
    ] {
        if manifest["files_sha256"][name].as_str().is_none() {
            return Err(format!("falta hash obligatorio: {name}"));
        }
    }
    if [
        "conversation-evidence-v2",
        "conversation-evidence-v3",
        "conversation-evidence-v4",
        "conversation-evidence-v5",
        "conversation-evidence-v6",
    ]
    .iter()
    .any(|v| manifest["version"] == *v)
    {
        for name in [
            "blind-questions.json",
            "blind-review.json",
            "adjudication.json",
        ] {
            if manifest["files_sha256"][name].as_str().is_none() {
                return Err(format!("falta revisión v2: {name}"));
            }
        }
        let adjudication = read(&root.join("adjudication.json"))?;
        if manifest["review_status"] != "separate_agent_blind_reconciled"
            || adjudication["unresolved_case_ids"] != json!([])
            || adjudication["blind_review_sha256"] != digest(&root.join("blind-review.json"))?
            || adjudication["source_sha256"] != digest(&root.join("source.json"))?
            || adjudication["questions_sha256"] != digest(&root.join("blind-questions.json"))?
            || adjudication["final_labels_sha256"] != digest(&root.join("development.jsonl"))?
        {
            return Err("revisión separada pendiente o no ligada a los datos v2".into());
        }
        let review = read(&root.join("blind-review.json"))?;
        if review["human_review"] != false
            || review["reviewer_kind"] != "separate_agent_same_model_family"
        {
            return Err("procedencia de revisión no admitida".into());
        }
        let blind = read(&root.join("blind-questions.json"))?;
        let blind = blind.as_array().ok_or("preguntas ciegas inválidas")?;
        let cases = lines(&root.join("development.jsonl"))?;
        let reviewed = array(&review, "cases")?;
        if blind.len() != cases.len() || reviewed.len() != cases.len() {
            return Err("revisión incompleta".into());
        }
        let mut seen = BTreeSet::new();
        for row in reviewed {
            let id = text(row, "id")?;
            if !seen.insert(id) {
                return Err("caso revisado duplicado".into());
            }
            let case = cases
                .iter()
                .find(|c| c["id"] == id)
                .ok_or("revisión de caso desconocido")?;
            let question = blind
                .iter()
                .find(|q| q["id"] == id)
                .ok_or("falta pregunta ciega")?;
            let evidence: Vec<Value> = array(case, "evidence")?
                .iter()
                .map(|e| e["statement_id"].clone())
                .collect();
            if manifest["version"] == "conversation-evidence-v3" {
                let gold = array(case, "required_query_bindings")?;
                let blind = array(row, "required_query_bindings")?;
                let query = text(&case["request"]["search"], "query")?;
                let to_set = |rows: &Vec<Value>| -> Result<BTreeSet<(String, String)>, String> {
                    rows.iter()
                        .map(|b| Ok((text(b, "dimension")?.into(), text(b, "quote")?.into())))
                        .collect()
                };
                if gold.is_empty()
                    || to_set(gold)? != to_set(blind)?
                    || gold
                        .iter()
                        .any(|b| !query.contains(b["quote"].as_str().unwrap_or("")))
                {
                    return Err("referentes gold no conciliados o no citados".into());
                }
            }
            if [
                "conversation-evidence-v4",
                "conversation-evidence-v5",
                "conversation-evidence-v6",
            ]
            .iter()
            .any(|v| manifest["version"] == *v)
            {
                let to_set = |rows: &Vec<Value>| -> Result<BTreeSet<(String, String)>, String> {
                    rows.iter()
                        .map(|r| Ok((text(r, "statement_id")?.into(), text(r, "relation")?.into())))
                        .collect()
                };
                if to_set(array(row, "evidence_relations")?)? != to_set(array(case, "evidence")?)? {
                    return Err("relaciones por evidencia no conciliadas".into());
                }
            }
            if manifest["version"] == "conversation-evidence-v6" {
                let coverage_set =
                    |row: &Value| -> Result<BTreeSet<(String, Vec<String>)>, String> {
                        array(row, "evidence")?
                            .iter()
                            .map(|e| {
                                let mut parts = array(e, "covers")?
                                    .iter()
                                    .map(|v| {
                                        v.as_str()
                                            .map(str::to_owned)
                                            .ok_or("cobertura inválida".to_owned())
                                    })
                                    .collect::<Result<Vec<_>, _>>()?;
                                parts.sort();
                                Ok((text(e, "memory_id")?.into(), parts))
                            })
                            .collect()
                    };
                if row["requested_parts"] != case["requested_parts"]
                    || coverage_set(row)? != coverage_set(case)?
                {
                    return Err("cobertura de partes no conciliada".into());
                }
                for name in [
                    "author-cases-initial.jsonl",
                    "blind-review-initial.json",
                    "reconciliation-proposal.json",
                ] {
                    if manifest["files_sha256"][name].as_str().is_none() {
                        return Err(format!("falta historial inicial v6: {name}"));
                    }
                }
            }
            if row["relation"] != case["relation"]
                || row["evidence_statement_ids"] != json!(evidence)
                || question["query"] != case["request"]["search"]["query"]
                || question["scope_id"] != case["request"]["search"]["scope_id"]
            {
                return Err("anotación separada no reconciliada".into());
            }
        }
    }
    let source = read(&root.join("source.json"))?;
    let memories = read(&root.join("memories.json"))?
        .as_array()
        .ok_or("corpus inválido")?
        .clone();
    let cases = lines(&root.join("development.jsonl"))?;
    let summary = check_data(&source, &memories, &cases)?;
    if manifest["version"] != summary["version"] {
        return Err("versión de manifiesto y fuente distinta".into());
    }
    for key in ["memories", "cases", "by_relation"] {
        if manifest[key] != summary[key] {
            return Err(format!("recuento declarado distinto: {key}"));
        }
    }
    if manifest["scope_count"] != summary["scopes"]
        || manifest["split"] != "development"
        || manifest["label_source"] != "agent_synthetic_unreviewed"
    {
        return Err("procedencia o recuento de ámbitos distinto".into());
    }

    Ok((memories, cases, summary))
}
pub fn validate(root: &Path) -> Result<(), String> {
    let (_, _, summary) = load(root)?;
    println!("{summary}");
    Ok(())
}
pub fn report_retrieval(root: &Path, results: &Path, output: &Path) -> Result<(), String> {
    let (memories, cases, validation) = load(root)?;
    let manifest = read(&results.with_extension("manifest.json"))?;
    if manifest["cases_sha256"] != digest(&root.join("development.jsonl"))?
        || manifest["kind"] != "search_benchmark"
        || manifest["split"] != "development"
        || manifest["mode"] != "keywords"
    {
        return Err("manifiesto de búsqueda distinto".into());
    }
    let corpus: BTreeMap<&str, &Value> = memories
        .iter()
        .map(|m| Ok((text(m, "id")?, m)))
        .collect::<Result<_, String>>()?;
    let rows = lines(results)?;
    let mut indexed = BTreeMap::new();
    for row in &rows {
        if indexed.insert(text(row, "id")?, row).is_some() {
            return Err("resultado duplicado".into());
        }
    }
    if indexed.len() != cases.len() {
        return Err("resultados incompletos".into());
    }
    let mut counts: BTreeMap<String, Value> = BTreeMap::new();
    let mut misses = Vec::new();
    let mut case_details = Vec::new();
    for case in &cases {
        let id = text(case, "id")?;
        let row = indexed.get(id).ok_or("falta caso en resultados")?;
        if row["expected_ids"] != case["expected_ids"] || row["response"]["status"] != "complete" {
            return Err("etiquetas o búsqueda no completa".into());
        }
        let candidates = array(&row["response"], "candidates")?;
        if candidates.len() > 20 {
            return Err("límite excedido".into());
        }
        let mut candidate_ids = BTreeSet::new();
        for candidate in candidates {
            let mid = text(candidate, "memory_id")?;
            let memory = corpus.get(mid).ok_or("candidato desconocido")?;
            if !candidate_ids.insert(mid)
                || memory["scope_id"] != case["request"]["search"]["scope_id"]
                || ["summary", "revision", "state", "conditions"]
                    .iter()
                    .any(|k| candidate[k] != memory[k])
                || ["kind", "ref", "locator"]
                    .iter()
                    .any(|k| candidate["source"][k] != memory["source"][k])
            {
                return Err("candidato distinto del corpus o ámbito".into());
            }
        }
        let relation = text(case, "relation")?;
        let first_rank = candidates
            .iter()
            .position(|c| {
                case["expected_ids"]
                    .as_array()
                    .unwrap()
                    .contains(&c["memory_id"])
            })
            .map(|r| r + 1);
        case_details.push(json!({"id":id,"scope_id":case["request"]["search"]["scope_id"],
            "relation":relation,"first_evidence_rank":first_rank,"candidate_count":candidates.len()}));
        let count=counts.entry(relation.into()).or_insert(json!({"cases":0,"with_expected_evidence":0,"covered_at_20":0,"covered_at_1":0,"empty_candidate_lists":0}));
        count["cases"] = json!(count["cases"].as_u64().unwrap() + 1);
        if candidates.is_empty() {
            count["empty_candidate_lists"] =
                json!(count["empty_candidate_lists"].as_u64().unwrap() + 1);
        }
        if let Some(expected) = case["expected_ids"].as_array().and_then(|v| v.first()) {
            count["with_expected_evidence"] =
                json!(count["with_expected_evidence"].as_u64().unwrap() + 1);
            if let Some(rank) = candidates.iter().position(|c| c["memory_id"] == *expected) {
                count["covered_at_20"] = json!(count["covered_at_20"].as_u64().unwrap() + 1);
                if rank == 0 {
                    count["covered_at_1"] = json!(count["covered_at_1"].as_u64().unwrap() + 1);
                }
            } else {
                misses.push(
                    json!({"id":id,"relation":relation,"query":case["request"]["search"]["query"]}),
                );
            }
        }
    }
    let report = json!({"version":"evidence-retrieval-report-v1","validation":validation,"by_relation":counts,"cases":case_details,"misses":misses,
        "cases_sha256":digest(&root.join("development.jsonl"))?,"corpus_sha256":digest(&root.join("memories.json"))?,
        "results_sha256":digest(results)?,"search_manifest":manifest,"provider_calls":0,
        "selection_measured":false,"answer_truth_measured":false,"promoted":false});
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    file.write_all(
        (serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n").as_bytes(),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn v6_binds_reviewed_coverage_and_keeps_joint_answer_distinct_from_single_parts() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/conversation-evidence-v6");
        let (memories, cases, summary) = load(&root).unwrap();
        assert_eq!(summary["by_relation"]["supports_answer"], 17);
        let case = cases.iter().find(|c| c["id"] == "cev6q_075_05").unwrap();
        assert_eq!(case["relation"], "supports_answer");
        assert!(
            case["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .all(|e| e["relation"] == "pending_or_partial")
        );
        assert_eq!(
            coverage(case, &["cev6_075_08"]).unwrap()["all_available_parts_retained"],
            false
        );
        assert_eq!(
            coverage(case, &["cev6_075_08", "cev6_075_26"]).unwrap()["all_requested_parts_documented"],
            true
        );
        let mut bad = cases;
        bad[0]["evidence"][0]["covers"] = json!(["invented"]);
        assert!(check_data(&read(&root.join("source.json")).unwrap(), &memories, &bad).is_err());
        let pairs = utility_pairs(&bad[1..3], "per_memory_v4").unwrap();
        assert_eq!(pairs[1]["request"]["selection_policy"], "per_memory_v4");
        assert_eq!(pairs[1]["request"]["provider"], Value::Null);
    }
    #[test]
    fn coverage_distinguishes_complementary_parts_and_undocumented_parts() {
        let case = json!({"request":{"search":{"query":"¿humedad, sellado y aplastamiento?"}},
            "requested_parts":[{"id":"humidity","kind":"requested","query_quote":"humedad"},{"id":"seal","kind":"requested","query_quote":"sellado"},{"id":"crush","kind":"requested","query_quote":"aplastamiento"}],
            "evidence":[{"memory_id":"a","covers":["humidity"]},{"memory_id":"b","covers":["seal"]}]});
        let one = coverage(&case, &["a"]).unwrap();
        assert_eq!(one["missing_available_parts"], json!(["seal"]));
        assert_eq!(one["unavailable_parts"], json!(["crush"]));
        let both = coverage(&case, &["a", "b"]).unwrap();
        assert_eq!(both["all_available_parts_retained"], true);
        assert_eq!(both["all_requested_parts_documented"], false);
        let mut bad = case.clone();
        bad["evidence"][0]["covers"] = json!(["invented"]);
        assert!(coverage(&bad, &[]).is_err());
        let mut bad = case;
        bad["requested_parts"][0]["query_quote"] = json!("invented");
        assert!(coverage(&bad, &[]).is_err());
    }
    #[test]
    fn all_seed_records_and_all_judgments_are_audited() {
        let case = json!({"id":"q","expected_ids":["a","b"]});
        let response = json!({
            "status":"complete","classification_status":"complete",
            "selected_id":"a","selected_ids":["a","b"],
            "seed":{"memories":[{"memory_id":"a"},{"memory_id":"b"}]},
            "retrieval":{"candidates":[{"memory_id":"a","revision":1},{"memory_id":"b","revision":2}]},
            "per_memory":{"items":[{"memory_id":"a","revision":1,"relation":"pending_or_partial"},{"memory_id":"b","revision":2,"relation":"pending_or_partial"}]}
        });
        assert_eq!(
            selection_kind(&case, &response).unwrap(),
            "permitted_evidence"
        );
        assert_eq!(
            per_memory_case(&case, &response, 16, false).unwrap()["utility_classification_counts"]
                ["true_positive"],
            2
        );
        let mut bad = response.clone();
        bad["seed"]["memories"][1]["memory_id"] = json!("wrong");
        bad["selected_ids"][1] = json!("wrong");
        assert_eq!(
            selection_kind(&case, &bad).unwrap(),
            "unpermitted_selection"
        );
        assert!(per_memory_case(&case, &bad, 16, false).is_err());
        for (field, value) in [
            ("revision", json!(3)),
            ("relation", json!("invented")),
            ("memory_id", json!("wrong")),
        ] {
            let mut bad = response.clone();
            bad["per_memory"]["items"][1][field] = value;
            assert!(per_memory_case(&case, &bad, 16, false).is_err());
        }
        let mut bad = response.clone();
        bad["per_memory"]["items"].as_array_mut().unwrap().pop();
        assert!(per_memory_case(&case, &bad, 16, false).is_err());
        let mut bad = response.clone();
        bad["per_memory"]["items"][1]["relation"] = json!("unrelated");
        assert!(per_memory_case(&case, &bad, 16, false).is_err());
        let mut bad = response.clone();
        bad["seed"]["memories"].as_array_mut().unwrap().pop();
        bad["selected_ids"] = json!(["a"]);
        assert!(per_memory_case(&case, &bad, 16, false).is_err());
        let mut bad = response.clone();
        bad["selected_id"] = json!("b");
        assert!(selected_set(&bad).is_err());
        let mut bad = response;
        bad["selected_ids"] = json!(["a"]);
        assert!(selected_set(&bad).is_err());
    }
    fn fixture() -> (Value, Vec<Value>, Vec<Value>) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/conversation-evidence-v1");
        (
            read(&root.join("source.json")).unwrap(),
            read(&root.join("memories.json"))
                .unwrap()
                .as_array()
                .unwrap()
                .clone(),
            lines(&root.join("development.jsonl")).unwrap(),
        )
    }
    #[test]
    fn resource_gate_rejects_missing_receipts_and_slow_tail() {
        let u = json!({"known_cost_usd":1.0,"cost_receipts":2,"cost_missing_cases":0,"skipped_empty_operations":0});
        let t = json!({"p50":100.0,"p95":200.0});
        assert_eq!(resource_screen(&u, &u, &t, &t, 2, true)["passes"], true);
        let mut unknown = u.clone();
        unknown["cost_missing_cases"] = json!(1);
        assert_eq!(
            resource_screen(&u, &unknown, &t, &t, 2, true)["passes"],
            false
        );
        assert!(resource_screen(&u, &unknown, &t, &t, 2, true)["cost_ratio"].is_null());
        let mut slow = t.clone();
        slow["p95"] = json!(301.0);
        assert_eq!(resource_screen(&u, &u, &t, &slow, 2, true)["passes"], false);
        assert_eq!(resource_screen(&u, &u, &t, &t, 2, false)["passes"], false);
    }
    #[test]
    fn primary_veto_report_distinguishes_acceptance_without_second_vote() {
        let case = json!({"id":"q","expected_ids":["a"]});
        let mut response = json!({"classification_status":"complete","retrieval":{"candidates":[{"memory_id":"a"},{"memory_id":"b"}]},"selected_id":"a","consensus":{"classified_candidate_count":2,"votes":["a",null]}});
        let result = consensus_case(&case, &response, 16, true).unwrap();
        assert_eq!(result["resolution"], "primary_without_second");
        assert_eq!(result["agreement"], false);
        assert!(consensus_case(&case, &response, 16, false).is_err());
        response["consensus"]["votes"] = json!([null, "a"]);
        assert!(consensus_case(&case, &response, 16, true).is_err());
        response["selected_id"] = Value::Null;
        assert_eq!(
            consensus_case(&case, &response, 16, true).unwrap()["resolution"],
            "second_without_primary"
        );
        response["consensus"]["votes"] = json!(["a", "b"]);
        assert_eq!(
            consensus_case(&case, &response, 16, true).unwrap()["resolution"],
            "conflict"
        );
        response["selected_id"] = json!("a");
        assert!(consensus_case(&case, &response, 16, true).is_err());
    }

    #[test]
    fn consensus_report_distinguishes_empty_search_from_missing_votes() {
        let c = json!({"id":"q","expected_ids":[]});
        let mut r = json!({"classification_status":"skipped_empty","retrieval":{"candidates":[]},"selected_id":null,"consensus":null});
        assert_eq!(
            consensus_case(&c, &r, 16, false).unwrap()["skipped_empty"],
            true
        );
        r["retrieval"]["candidates"] = json!([{"memory_id":"m"}]);
        assert!(consensus_case(&c, &r, 16, false).is_err());
        r["retrieval"]["candidates"] = json!([]);
        r["selected_id"] = json!("m");
        assert!(consensus_case(&c, &r, 16, false).is_err());
        let full = json!({"classification_status":"complete","retrieval":{"candidates":(0..20).map(|i|json!({"memory_id":format!("m-{i}")})).collect::<Vec<_>>()},"selected_id":"m-19","consensus":{"classified_candidate_count":20,"votes":["m-19","m-19"]}});
        assert!(consensus_case(&c, &full, 20, false).is_ok());
        assert!(consensus_case(&c, &full, 16, false).is_err());
    }
    #[test]
    fn v4_accepts_reviewed_alternatives_without_claiming_complete_answers() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/conversation-evidence-v4");
        let source = read(&root.join("source.json")).unwrap();
        let memories = read(&root.join("memories.json"))
            .unwrap()
            .as_array()
            .unwrap()
            .clone();
        let mut cases = lines(&root.join("development.jsonl")).unwrap();
        assert!(check_data(&source, &memories, &cases).is_ok());
        let c = cases
            .iter_mut()
            .find(|c| c["id"] == "cev4q_059_05")
            .unwrap();
        let response = json!({"selected_id":"cev4_059_18"});
        assert_eq!(selected_relation(c, &response), Some("pending_or_partial"));
        assert!(
            evidence_tier(selected_relation(c, &response)) < evidence_tier(Some("supports_answer"))
        );
        c["evidence"][1]["quote"] = json!("invented");
        assert!(check_data(&source, &memories, &cases).is_err());
    }
    #[test]
    fn v2_requires_larger_corpus_and_matching_source_identity() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/conversation-evidence-v2");
        let mut source = read(&root.join("source.json")).unwrap();
        let memories = read(&root.join("memories.json"))
            .unwrap()
            .as_array()
            .unwrap()
            .clone();
        let cases = lines(&root.join("development.jsonl")).unwrap();
        assert!(check_data(&source, &memories, &cases).is_ok());
        assert!(check_data(&source, &memories[..223], &cases).is_err());
        source["version"] = json!("conversation-evidence-v1");
        assert!(check_data(&source, &memories, &cases).is_err());
    }
    #[test]
    fn utility_pairs_preserve_labels_and_gate_rejects_regressions() {
        let (_, _, cases) = fixture();
        let paired = utility_pairs(&cases[..2], "utility_v1").unwrap();
        assert_eq!(paired[0]["request"]["selection_policy"], "baseline");
        assert_eq!(paired[1]["request"]["selection_policy"], "utility_v1");
        assert_eq!(paired[2]["request"]["selection_policy"], "utility_v1");
        assert_eq!(paired[0]["expected_ids"], cases[0]["expected_ids"]);
        assert_eq!(
            paired[0]["request"]["search"],
            cases[0]["request"]["search"]
        );
        assert_eq!(
            utility_block("permitted_evidence", "empty"),
            Some("lost_useful_evidence")
        );
        assert_eq!(
            utility_block("empty", "unpermitted_selection"),
            Some("unpermitted_selection")
        );
        assert_eq!(utility_block("empty", "degraded"), Some("new_degradation"));
        assert_eq!(utility_block("empty", "permitted_evidence"), None);
        let v2 = utility_pairs(&cases[..2], "utility_v2").unwrap();
        let jev = utility_pairs(&cases[..2], "jev_roles_v1").unwrap();
        assert_eq!(jev[0]["request"]["provider"], "decisions");
        assert_eq!(jev[1]["request"]["provider"], "jev");
        assert_eq!(jev[1]["request"]["selection_policy"], "roles_v1");
        let jev2 = utility_pairs(&cases[..2], "jev_roles_v2").unwrap();
        assert_eq!(jev2[1]["request"]["selection_policy"], "roles_v2");
        for profile in [
            "jev_roles_consensus_v1",
            "jev_roles_consensus_batch_v1",
            "jev_roles_consensus_shared_v1",
            "jev_roles_consensus_compact_v1",
            "jev_roles_consensus_mirrored_v1",
            "jev_roles_primary_veto_v1",
            "jev_per_memory_v1",
            "jev_per_memory_v2",
            "jev_per_memory_v3",
            "jev_per_memory_binary_v1",
            "jev_per_memory_binary_v2",
            "jev_per_memory_binary_v3",
            "jev_per_memory_binary_v4",
            "jev_per_memory_binary_v5",
            "jev_per_memory_examples_v1",
            "jev_per_memory_factors_v1",
            "jev_per_memory_scoped_v1",
            "jev_per_memory_scoped_v2",
            "jev_per_memory_scoped_v3",
            "jev_per_memory_scoped_anchor_v1",
            "jev_per_memory_scored_v1",
            "jev_per_memory_verified_v1",
            "jev_per_memory_correction_verified_v1",
            "jev_per_memory_correction_verified_v2",
            "jev_per_memory_correction_verified_v3",
            "jev_per_memory_correction_verified_v4",
            "jev_per_memory_correction_verified_v5",
            "jev_per_memory_role_verified_v1",
            "jev_per_memory_role_verified_v2",
            "jev_per_memory_role_batch_v1",
            "jev_per_memory_complete_premise_v1",
        ] {
            let pairs = utility_pairs(&cases[..2], profile).unwrap();
            assert_eq!(
                pairs[1]["request"]["selection_policy"],
                profile.strip_prefix("jev_").unwrap()
            );
            assert_eq!(pairs[1]["request"]["provider"], "jev");
            if profile.starts_with("jev_per_memory_correction_verified_v")
                || (profile == "jev_per_memory_role_batch_v1"
                    || profile == "jev_per_memory_complete_premise_v1")
                || profile.starts_with("jev_per_memory_role_verified_v")
            {
                assert_eq!(per_memory_limit(profile), Some(7));
            }
            assert_eq!(pairs[0]["request"]["provider"], "decisions");
        }
        assert_eq!(v2[1]["request"]["selection_policy"], "utility_v2");
        assert_eq!(v2[1]["request"]["search"], cases[0]["request"]["search"]);
        assert!(utility_pairs(&cases[..2], "baseline").is_err());
    }
    #[test]
    fn rejects_cross_scope_evidence_and_changed_quotes() {
        let (source, memories, mut cases) = fixture();
        assert!(check_data(&source, &memories, &cases).is_ok());
        let index = cases
            .iter()
            .position(|c| c["relation"] == "refutes_premise")
            .unwrap();
        cases[index]["evidence"][0]["quote"] = json!("Invented statement");
        assert!(check_data(&source, &memories, &cases).is_err());
        let (_, _, mut cases) = fixture();
        let index = cases
            .iter()
            .position(|c| c["relation"] == "pending_or_partial")
            .unwrap();
        cases[index]["request"]["search"]["scope_id"] =
            json!("synthetic_conversation_evidence_034");
        if cases[index]["expected_ids"][0]
            .as_str()
            .unwrap()
            .starts_with("cev1_034")
        {
            cases[index]["request"]["search"]["scope_id"] =
                json!("synthetic_conversation_evidence_035");
        }
        assert!(check_data(&source, &memories, &cases).is_err());
    }
    #[test]
    fn absence_cannot_hide_evidence_or_be_called_independently_reviewed() {
        let (mut source, memories, mut cases) = fixture();
        let index = cases
            .iter()
            .position(|c| c["relation"] == "supports_answer")
            .unwrap();
        cases[index]["relation"] = json!("unrelated");
        cases[index]["segment"] = json!("conversation/unrelated");
        assert!(check_data(&source, &memories, &cases).is_err());
        source["independent_review"] = json!(true);
        let (_, _, cases) = fixture();
        assert!(check_data(&source, &memories, &cases).is_err());
    }
}

pub(crate) fn utility_pairs(cases: &[Value], candidate_policy: &str) -> Result<Vec<Value>, String> {
    if ![
        "utility_v1",
        "utility_v2",
        "per_memory_v4",
        "per_memory_binary_v1",
        "per_memory_binary_v2",
        "per_memory_binary_v3",
        "per_memory_binary_v4",
        "per_memory_binary_v5",
        "per_memory_examples_v1",
        "per_memory_factors_v1",
        "per_memory_scoped_v1",
        "per_memory_scoped_v2",
        "per_memory_scoped_v3",
        "per_memory_scoped_anchor_v1",
        "per_memory_scored_v1",
        "witness_v1",
        "roles_v1",
        "roles_fallback_v1",
        "roles_v2",
        "jev_roles_v1",
        "jev_roles_v2",
        "jev_roles_consensus_v1",
        "jev_roles_consensus_batch_v1",
        "jev_roles_consensus_shared_v1",
        "jev_roles_consensus_compact_v1",
        "jev_roles_consensus_mirrored_v1",
        "jev_roles_primary_veto_v1",
        "jev_per_memory_v1",
        "jev_per_memory_v2",
        "jev_per_memory_v3",
        "jev_per_memory_binary_v1",
        "jev_per_memory_binary_v2",
        "jev_per_memory_binary_v3",
        "jev_per_memory_binary_v4",
        "jev_per_memory_binary_v5",
        "jev_per_memory_examples_v1",
        "jev_per_memory_factors_v1",
        "jev_per_memory_scoped_v1",
        "jev_per_memory_scoped_v2",
        "jev_per_memory_scoped_v3",
        "jev_per_memory_scoped_anchor_v1",
        "jev_per_memory_scored_v1",
        "jev_per_memory_verified_v1",
        "jev_per_memory_correction_verified_v1",
        "jev_per_memory_correction_verified_v2",
        "jev_per_memory_correction_verified_v3",
        "jev_per_memory_correction_verified_v4",
        "jev_per_memory_correction_verified_v5",
        "jev_per_memory_role_verified_v1",
        "jev_per_memory_role_verified_v2",
        "jev_per_memory_role_batch_v1",
        "jev_per_memory_complete_premise_v1",
    ]
    .contains(&candidate_policy)
    {
        return Err("candidata de utilidad inválida".into());
    }

    let mut paired = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let id = text(case, "id")?;
        let policies = if index % 2 == 0 {
            ["baseline", candidate_policy]
        } else {
            [candidate_policy, "baseline"]
        };
        for policy in policies {
            let mut row = case.clone();
            row["id"] = json!(format!("{id}__{policy}"));
            row["original_case_id"] = json!(id);
            if [
                "jev_roles_v1",
                "jev_roles_v2",
                "jev_roles_consensus_v1",
                "jev_roles_consensus_batch_v1",
                "jev_roles_consensus_shared_v1",
                "jev_roles_consensus_compact_v1",
                "jev_roles_consensus_mirrored_v1",
                "jev_roles_primary_veto_v1",
                "jev_per_memory_v1",
                "jev_per_memory_v2",
                "jev_per_memory_v3",
                "jev_per_memory_binary_v1",
                "jev_per_memory_binary_v2",
                "jev_per_memory_binary_v3",
                "jev_per_memory_binary_v4",
                "jev_per_memory_binary_v5",
                "jev_per_memory_examples_v1",
                "jev_per_memory_factors_v1",
                "jev_per_memory_scoped_v1",
                "jev_per_memory_scoped_v2",
                "jev_per_memory_scoped_v3",
                "jev_per_memory_scoped_anchor_v1",
                "jev_per_memory_scored_v1",
                "jev_per_memory_verified_v1",
                "jev_per_memory_correction_verified_v1",
                "jev_per_memory_correction_verified_v2",
                "jev_per_memory_correction_verified_v3",
                "jev_per_memory_correction_verified_v4",
                "jev_per_memory_correction_verified_v5",
                "jev_per_memory_role_verified_v1",
                "jev_per_memory_role_verified_v2",
                "jev_per_memory_role_batch_v1",
                "jev_per_memory_complete_premise_v1",
            ]
            .contains(&candidate_policy)
            {
                row["request"]["selection_policy"] = json!(if policy == "baseline" {
                    "baseline"
                } else {
                    candidate_policy
                        .strip_prefix("jev_")
                        .ok_or("perfil Jev inválido")?
                });
                row["request"]["provider"] = json!(if policy == "baseline" {
                    "decisions"
                } else {
                    "jev"
                });
            } else {
                row["request"]["selection_policy"] = json!(policy);
            }
            paired.push(row);
        }
    }
    Ok(paired)
}
pub fn prepare_utility(root: &Path, output: &Path, candidate_policy: &str) -> Result<(), String> {
    let (_, cases, _) = load(root)?;
    if output.exists() {
        return Err("elige un directorio nuevo".into());
    }
    let pairs = utility_pairs(&cases, candidate_policy)?;
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let path = output.join("paired-cases.jsonl");
    fs::write(
        &path,
        pairs
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .map_err(|e| e.to_string())?;
    fs::write(output.join("preparation.json"),serde_json::to_string_pretty(&json!({
        "version":"utility-paired-v1","candidate_policy":candidate_policy,"cases_sha256":digest(&root.join("development.jsonl"))?,
        "corpus_sha256":digest(&root.join("memories.json"))?,"rubric_sha256":read(&root.join("manifest.json"))?["rubric_sha256"],
        "paired_cases_sha256":digest(&path)?,"original_cases":cases.len(),"paired_operations":pairs.len(),
        "order":"Adjacent policies; baseline first on even source positions, utility first on odd positions; labels not used for order.",
        "human_review":false,"independent_review":false,"promoted":false})).map_err(|e|e.to_string())?+"\n").map_err(|e|e.to_string())
}
// Coverage annotations are source-reviewed gold, never inferred by the classifier.
fn coverage(case: &Value, selected: &[&str]) -> Result<Value, String> {
    let query = text(&case["request"]["search"], "query")?;
    let parts = array(case, "requested_parts")?;
    let mut declared = BTreeSet::new();
    for part in parts {
        let id = text(part, "id")?;
        if !declared.insert(id)
            || !["requested", "premise"].contains(&text(part, "kind")?)
            || !query.contains(text(part, "query_quote")?)
        {
            return Err("parte duplicada o sin cita literal de pregunta".into());
        }
    }
    if declared.is_empty() {
        return Err("faltan partes solicitadas".into());
    }
    let mut available = BTreeSet::new();
    let mut covered = BTreeSet::new();
    for evidence in array(case, "evidence")? {
        let covers = array(evidence, "covers")?;
        let mut seen = BTreeSet::new();
        if covers.is_empty() {
            return Err("evidencia sin cobertura declarada".into());
        }
        for part in covers {
            let id = part.as_str().ok_or("ID de parte inválido")?;
            if !declared.contains(id) || !seen.insert(id) {
                return Err("cobertura desconocida o duplicada".into());
            }
            available.insert(id);
            if selected.contains(&text(evidence, "memory_id")?) {
                covered.insert(id);
            }
        }
    }
    let missing: Vec<_> = available.difference(&covered).copied().collect();
    let unavailable: Vec<_> = declared.difference(&available).copied().collect();
    Ok(
        json!({"requested_parts":declared,"available_parts":available,"covered_parts":covered,"missing_available_parts":missing,"unavailable_parts":unavailable,
        "all_available_parts_retained":missing.is_empty(),"all_requested_parts_documented":unavailable.is_empty()}),
    )
}
fn per_memory_limit(profile: &str) -> Option<usize> {
    match profile {
        "jev_per_memory_v1" => Some(16),
        "jev_per_memory_v2"
        | "jev_per_memory_v3"
        | "per_memory_v4"
        | "per_memory_binary_v1"
        | "per_memory_binary_v2"
        | "per_memory_binary_v3"
        | "per_memory_binary_v4"
        | "per_memory_binary_v5"
        | "per_memory_examples_v1"
        | "per_memory_factors_v1"
        | "per_memory_scoped_v1"
        | "per_memory_scoped_v2"
        | "per_memory_scoped_v3"
        | "per_memory_scored_v1"
        | "jev_per_memory_binary_v1"
        | "jev_per_memory_binary_v2"
        | "jev_per_memory_binary_v3"
        | "jev_per_memory_binary_v4"
        | "jev_per_memory_binary_v5"
        | "jev_per_memory_examples_v1"
        | "jev_per_memory_factors_v1"
        | "jev_per_memory_scoped_v1"
        | "jev_per_memory_scoped_v2"
        | "jev_per_memory_scoped_v3"
        | "jev_per_memory_scored_v1"
        | "jev_per_memory_verified_v1" => Some(8),
        "jev_per_memory_correction_verified_v1"
        | "jev_per_memory_correction_verified_v2"
        | "jev_per_memory_correction_verified_v3"
        | "jev_per_memory_correction_verified_v4"
        | "jev_per_memory_correction_verified_v5"
        | "jev_per_memory_role_verified_v1"
        | "jev_per_memory_role_verified_v2"
        | "jev_per_memory_role_batch_v1"
        | "jev_per_memory_complete_premise_v1"
        | "per_memory_scoped_anchor_v1"
        | "jev_per_memory_scoped_anchor_v1" => Some(7),
        _ => None,
    }
}
pub(crate) fn selected_set(response: &Value) -> Result<Vec<&str>, String> {
    let memories = array(&response["seed"], "memories")?;
    let ids: Vec<_> = memories
        .iter()
        .map(|m| text(m, "memory_id"))
        .collect::<Result<_, _>>()?;
    if ids.iter().copied().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err("selección duplicada".into());
    }
    if response.get("selected_ids").is_some() && response["selected_ids"] != json!(ids) {
        return Err("selected_ids difiere de semilla".into());
    }
    let legacy_multi_review = response["classification_status"] == "review_reused"
        && ids.len() > 1
        && response["selected_id"].is_null();
    if !legacy_multi_review && response["selected_id"] != json!(ids.first()) {
        return Err("alias singular difiere de semilla".into());
    }
    Ok(ids)
}
fn selection_kind(case: &Value, response: &Value) -> Result<&'static str, String> {
    if response["status"] != "complete" {
        return Ok("degraded");
    }
    let ids = selected_set(response)?;
    if ids.is_empty() {
        return Ok("empty");
    }
    let expected = array(case, "expected_ids")?;
    Ok(
        if ids
            .iter()
            .all(|id| expected.iter().any(|e| e.as_str() == Some(id)))
        {
            "permitted_evidence"
        } else {
            "unpermitted_selection"
        },
    )
}

pub(crate) fn per_memory_case(
    case: &Value,
    response: &Value,
    limit: usize,
    binary: bool,
) -> Result<Value, String> {
    let candidates = array(&response["retrieval"], "candidates")?;
    let selected = selected_set(response)?;
    if candidates.is_empty() {
        if response["classification_status"] != "skipped_empty"
            || !response["per_memory"].is_null()
            || !selected.is_empty()
        {
            return Err("clasificación por recuerdo vacía inconsistente".into());
        }
        return Ok(json!({"id":case["id"],"skipped_empty":true,"items":[]}));
    }
    if !response["consensus"].is_null() || !response["applicability"].is_null() {
        return Err("trazas ajenas a per_memory".into());
    }
    let items = array(&response["per_memory"], "items")?;
    let count = candidates.len().min(limit);
    if items.len() != count {
        return Err("juicios incompletos o adicionales".into());
    }
    if case["request"]["selection_policy"] == "per_memory_scoped_anchor_v1"
        && response["per_memory"]["scoped_checks"].is_null()
    {
        return Err("falta testigo literal de caso".into());
    }
    if !response["per_memory"]["scoped_checks"].is_null() {
        if case["request"]["selection_policy"] == "per_memory_scoped_anchor_v1" {
            crate::factor_audit::inspect_scoped_anchor(
                text(&case["request"]["search"], "query")?,
                response,
            )?;
        } else {
            crate::factor_audit::inspect_scoped(response)?;
        }
    }
    if !response["per_memory"]["factor_checks"].is_null() {
        crate::factor_audit::inspect(response)?;
    }
    let mut positive = Vec::new();
    let mut counts = BTreeMap::<String, usize>::new();
    for (item, candidate) in items.iter().zip(&candidates[..count]) {
        if item["memory_id"] != candidate["memory_id"] || item["revision"] != candidate["revision"]
        {
            return Err("juicio difiere del ID/revisión presentado".into());
        }
        let relation = text(item, "relation")?;
        if !(if binary {
            ["useful_evidence", "unrelated"].contains(&relation)
        } else {
            RELATIONS.contains(&relation)
        }) {
            return Err("relación por recuerdo incompatible con tarea".into());
        }
        let id = text(item, "memory_id")?;
        let predicted = relation != "unrelated";
        let expected = array(case, "expected_ids")?
            .iter()
            .any(|e| e.as_str() == Some(id));
        let kind = match (predicted, expected) {
            (true, true) => "true_positive",
            (true, false) => "false_positive",
            (false, true) => "false_negative",
            (false, false) => "true_negative",
        };
        *counts.entry(kind.into()).or_default() += 1;
        if predicted {
            positive.push(id);
        }
    }
    if positive != selected || response["selected_ids"] != json!(positive) {
        return Err("semilla no coincide con todos los juicios positivos".into());
    }
    Ok(
        json!({"id":case["id"],"items":items,"classified_candidates":count,"selected_ids":positive,"utility_classification_counts":counts}),
    )
}
fn utility_block(baseline: &str, candidate: &str) -> Option<&'static str> {
    if candidate == "degraded" && baseline != "degraded" {
        Some("new_degradation")
    } else if candidate == "unpermitted_selection" {
        Some("unpermitted_selection")
    } else if baseline == "permitted_evidence" && candidate != "permitted_evidence" {
        Some("lost_useful_evidence")
    } else {
        None
    }
}
pub(crate) fn time_distribution(values: &[f64]) -> Value {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let q = |fraction: f64| -> Option<f64> {
        if sorted.is_empty() {
            None
        } else {
            let p = (sorted.len() - 1) as f64 * fraction;
            Some(
                sorted[p.floor() as usize]
                    + (sorted[p.ceil() as usize] - sorted[p.floor() as usize]) * p.fract(),
            )
        }
    };
    json!({"count":values.len(),"p50":q(0.5),"p95":q(0.95),"p99":q(0.99)})
}
fn selected_relation<'a>(case: &'a Value, response: &Value) -> Option<&'a str> {
    let id = response["selected_id"].as_str()?;
    case["evidence"]
        .as_array()?
        .iter()
        .find(|e| e["memory_id"] == id)
        .and_then(|e| e["relation"].as_str().or(case["relation"].as_str()))
}
fn evidence_tier(relation: Option<&str>) -> u8 {
    match relation {
        Some("supports_answer" | "refutes_premise") => 2,
        Some("pending_or_partial") => 1,
        _ => 0,
    }
}
fn resource_screen(
    base: &Value,
    other: &Value,
    base_time: &Value,
    other_time: &Value,
    cases: usize,
    operations_complete: bool,
) -> Value {
    let cost_ratio = base["known_cost_usd"]
        .as_f64()
        .filter(|n| *n > 0.0)
        .zip(other["known_cost_usd"].as_f64())
        .map(|(a, b)| b / a);
    let ratio = |q: &str| {
        base_time[q]
            .as_f64()
            .filter(|n| *n > 0.0)
            .zip(other_time[q].as_f64())
            .map(|(a, b)| b / a)
    };
    let p50 = ratio("p50");
    let p95 = ratio("p95");
    let receipts = [base, other].iter().all(|u| {
        u["cost_missing_cases"] == 0
            && u["cost_receipts"].as_u64().unwrap_or(0)
                + u["skipped_empty_operations"].as_u64().unwrap_or(0)
                == cases as u64
    });
    // A ratio of known subtotals is not the full cost ratio when a receipt is missing.
    let cost_ratio = if receipts { cost_ratio } else { None };
    let passes = operations_complete
        && receipts
        && [cost_ratio, p50, p95]
            .iter()
            .all(|n| n.is_some_and(|n| n.is_finite() && n <= 1.5));
    json!({"predeclared_ratio_limit":1.5,"cost_ratio":cost_ratio,"p50_ratio":p50,"p95_ratio":p95,"complete_cost_receipts_for_attempted_operations":receipts,"all_operations_complete":operations_complete,"passes":passes})
}

fn consensus_limit(profile: &str) -> Option<usize> {
    match profile {
        "jev_roles_consensus_v1"
        | "jev_roles_consensus_batch_v1"
        | "jev_roles_consensus_mirrored_v1"
        | "jev_roles_primary_veto_v1" => Some(16),
        "jev_roles_consensus_shared_v1" => Some(20),
        "jev_roles_consensus_compact_v1" => Some(4),
        _ => None,
    }
}

fn consensus_case(
    case: &Value,
    response: &Value,
    limit: usize,
    primary_veto: bool,
) -> Result<Value, String> {
    let candidates = array(&response["retrieval"], "candidates")?;
    if candidates.is_empty() {
        if response["classification_status"] != "skipped_empty"
            || !response["selected_id"].is_null()
            || !response["consensus"].is_null()
        {
            return Err("consenso vacío inconsistente".into());
        }
        return Ok(
            json!({"id":case["id"],"skipped_empty":true,"votes":null,"selected_id":null,"classified_candidates":0}),
        );
    }
    let trace = &response["consensus"];
    let votes = array(trace, "votes")?;
    let count = candidates.len().min(limit);
    if votes.len() != 2
        || trace["classified_candidate_count"] != count
        || votes
            .iter()
            .any(|v| !v.is_null() && !candidates[..count].iter().any(|c| c["memory_id"] == *v))
    {
        return Err("traza de consenso inválida".into());
    }
    let resolution = if votes[0].is_null() && votes[1].is_null() {
        "both_abstain"
    } else if votes[0] == votes[1] {
        "agreement"
    } else if votes[1].is_null() {
        "primary_without_second"
    } else if votes[0].is_null() {
        "second_without_primary"
    } else {
        "conflict"
    };
    let selected = if votes[0] == votes[1] || (primary_veto && votes[1].is_null()) {
        votes[0].clone()
    } else {
        Value::Null
    };
    if response["selected_id"] != selected {
        return Err("resultado no corresponde a votos".into());
    }
    Ok(
        json!({"id":case["id"],"votes":votes,"agreement":votes[0]==votes[1],"resolution":resolution,"selected_id":selected,
        "vote_permitted":votes.iter().map(|v|!v.is_null() && case["expected_ids"].as_array().is_some_and(|ids| ids.contains(v))).collect::<Vec<_>>(),"classified_candidates":count}),
    )
}
pub fn report_utility(root: &Path, directory: &Path, output: &Path) -> Result<(), String> {
    let (_, cases, _) = load(root)?;
    let paired_path = directory.join("paired-cases.jsonl");
    let journal_path = directory.join("journal.jsonl");
    let prep = read(&directory.join("preparation.json"))?;
    let candidate_policy = prep["candidate_policy"].as_str().unwrap_or("utility_v1");
    if prep["cases_sha256"] != digest(&root.join("development.jsonl"))?
        || prep["corpus_sha256"] != digest(&root.join("memories.json"))?
        || prep["paired_cases_sha256"] != digest(&paired_path)?
        || lines(&paired_path)? != utility_pairs(&cases, candidate_policy)?
    {
        return Err("casos emparejados modificados".into());
    }
    let audit_path = directory.join("audit.json");
    let audit = read(&audit_path)?;
    let campaigns = array(&audit, "campaigns")?;
    if campaigns.len() != 1
        || campaigns[0]["integrity"]["valid"] != true
        || campaigns[0]["sha256"]["journal"] != digest(&journal_path)?
        || campaigns[0]["sha256"]["cases"] != digest(&paired_path)?
        || campaigns[0]["sha256"]["corpus"] != digest(&root.join("memories.json"))?
    {
        return Err("falta auditoría íntegra de esta campaña".into());
    }
    let journal = lines(&journal_path)?;
    let manifest = journal.first().ok_or("diario vacío")?;
    if manifest["selection_policy"] != "per_case"
        || manifest["provider"]
            != if [
                "jev_roles_v1",
                "jev_roles_v2",
                "jev_roles_consensus_v1",
                "jev_roles_consensus_batch_v1",
                "jev_roles_consensus_shared_v1",
                "jev_roles_consensus_compact_v1",
                "jev_roles_consensus_mirrored_v1",
                "jev_roles_primary_veto_v1",
                "jev_per_memory_v1",
                "jev_per_memory_v2",
                "jev_per_memory_v3",
                "jev_per_memory_binary_v1",
                "jev_per_memory_binary_v2",
                "jev_per_memory_binary_v3",
                "jev_per_memory_binary_v4",
                "jev_per_memory_binary_v5",
                "jev_per_memory_examples_v1",
                "jev_per_memory_factors_v1",
                "jev_per_memory_scoped_v1",
                "jev_per_memory_scoped_v2",
                "jev_per_memory_scoped_v3",
                "jev_per_memory_scoped_anchor_v1",
                "jev_per_memory_scored_v1",
                "jev_per_memory_verified_v1",
                "jev_per_memory_correction_verified_v1",
                "jev_per_memory_correction_verified_v2",
                "jev_per_memory_correction_verified_v3",
                "jev_per_memory_correction_verified_v4",
                "jev_per_memory_correction_verified_v5",
                "jev_per_memory_role_verified_v1",
                "jev_per_memory_role_verified_v2",
                "jev_per_memory_role_batch_v1",
                "jev_per_memory_complete_premise_v1",
            ]
            .contains(&candidate_policy)
            {
                "per_case"
            } else {
                "decisions"
            }
        || manifest["cases_sha256"] != digest(&paired_path)?
    {
        return Err("manifiesto de política distinto".into());
    }
    let mut finished = BTreeMap::new();
    let mut started = BTreeSet::new();
    for event in journal.iter().skip(1) {
        let id = text(event, "id")?;
        match text(event, "kind")? {
            "started" if started.insert(id) => (),
            "finished" if started.contains(id) && finished.insert(id, event).is_none() => (),
            _ => return Err("eventos duplicados o fuera de orden".into()),
        }
    }
    if finished.len() != cases.len() * 2 || started.len() != finished.len() {
        return Err("campaña incompleta".into());
    }
    let mut by_policy: BTreeMap<String, BTreeMap<String, BTreeMap<String, usize>>> =
        BTreeMap::new();
    let mut usage: BTreeMap<String, Value> = BTreeMap::new();
    let mut elapsed: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut models: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut changes = Vec::new();
    let mut blocks = Vec::new();
    let mut gains = 0;
    let mut completeness_gains = 0;
    let mut additional_evidence_gains = 0;
    let mut seed_bytes: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut seed_counts: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut coverage_rows = Vec::new();
    let mut selected_relations: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut deltas = Vec::new();
    let mut operations_complete = true;
    for case in &cases {
        let id = text(case, "id")?;
        let baseline_key = format!("{id}__baseline");
        let candidate_key = format!("{id}__{candidate_policy}");
        let baseline = finished
            .get(baseline_key.as_str())
            .ok_or("falta par base")?;
        let candidate = finished
            .get(candidate_key.as_str())
            .ok_or("falta par candidato")?;
        if [
            "jev_roles_v1",
            "jev_roles_v2",
            "jev_roles_consensus_v1",
            "jev_roles_consensus_batch_v1",
            "jev_roles_consensus_shared_v1",
            "jev_roles_consensus_compact_v1",
            "jev_roles_consensus_mirrored_v1",
            "jev_roles_primary_veto_v1",
            "jev_per_memory_v1",
            "jev_per_memory_v2",
            "jev_per_memory_v3",
            "jev_per_memory_binary_v1",
            "jev_per_memory_binary_v2",
            "jev_per_memory_binary_v3",
            "jev_per_memory_binary_v4",
            "jev_per_memory_binary_v5",
            "jev_per_memory_examples_v1",
            "jev_per_memory_factors_v1",
            "jev_per_memory_scoped_v1",
            "jev_per_memory_scoped_v2",
            "jev_per_memory_scoped_v3",
            "jev_per_memory_scoped_anchor_v1",
            "jev_per_memory_scored_v1",
            "jev_per_memory_verified_v1",
            "jev_per_memory_correction_verified_v1",
            "jev_per_memory_correction_verified_v2",
            "jev_per_memory_correction_verified_v3",
            "jev_per_memory_correction_verified_v4",
            "jev_per_memory_correction_verified_v5",
            "jev_per_memory_role_verified_v1",
            "jev_per_memory_role_verified_v2",
            "jev_per_memory_role_batch_v1",
            "jev_per_memory_complete_premise_v1",
        ]
        .contains(&candidate_policy)
            && (baseline["response"]["requested_model"] != "openai/gpt-6-luna-decisions"
                || candidate["response"]["requested_model"] != "typesafe/jev-1.13")
        {
            return Err("modelos solicitados distintos del perfil emparejado".into());
        }
        if baseline["response"]["retrieval"]["candidates"]
            != candidate["response"]["retrieval"]["candidates"]
        {
            return Err(format!("listas de candidatos distintas: {id}"));
        }
        let left = selection_kind(case, &baseline["response"])?;
        let right = selection_kind(case, &candidate["response"])?;
        if let Some(reason) = utility_block(left, right) {
            blocks.push(json!({"id":id,"reason":reason,"baseline":left,"candidate_policy":candidate_policy,"candidate":right}));
        }
        if left != "permitted_evidence" && right == "permitted_evidence" {
            gains += 1;
        }
        let baseline_ids = selected_set(&baseline["response"])?;
        let candidate_ids = selected_set(&candidate["response"])?;
        if case["requested_parts"].is_array() {
            let base_coverage = coverage(case, &baseline_ids)?;
            let candidate_coverage = coverage(case, &candidate_ids)?;
            let base_parts = array(&base_coverage, "covered_parts")?;
            let other_parts = array(&candidate_coverage, "covered_parts")?;
            if base_parts.iter().any(|p| !other_parts.contains(p)) {
                blocks.push(json!({"id":id,"reason":"lost_baseline_requested_part","baseline":base_coverage,"candidate":candidate_coverage}));
            }
            coverage_rows
                .push(json!({"id":id,"baseline":base_coverage,"candidate":candidate_coverage}));
        }
        if per_memory_limit(candidate_policy).is_some()
            && left == "permitted_evidence"
            && right == "permitted_evidence"
        {
            if !baseline_ids.iter().all(|id| candidate_ids.contains(id)) {
                blocks.push(json!({"id":id,"reason":"lost_baseline_evidence_record","baseline_ids":baseline_ids,"candidate_ids":candidate_ids}));
            } else if candidate_ids.len() > baseline_ids.len() {
                additional_evidence_gains += 1;
            }
        }
        if per_memory_limit(candidate_policy).is_none()
            && left == "permitted_evidence"
            && right == "permitted_evidence"
        {
            let l = evidence_tier(selected_relation(case, &baseline["response"]));
            let r = evidence_tier(selected_relation(case, &candidate["response"]));
            if r < l {
                blocks.push(json!({"id":id,"reason":"lost_evidence_completeness","baseline_tier":l,"candidate_tier":r}));
            } else if r > l {
                completeness_gains += 1;
            }
        }
        if baseline_ids != candidate_ids || left != right {
            changes.push(json!({"id":id,"relation":case["relation"],"query":case["request"]["search"]["query"],
                "baseline_kind":left,"utility_kind":right,"baseline_selected_id":baseline["response"]["selected_id"],
                "utility_selected_id":candidate["response"]["selected_id"],"baseline_selected_ids":baseline_ids,"utility_selected_ids":candidate_ids,"baseline_seed":baseline["response"]["seed"]["seed_text"],
                "utility_seed":candidate["response"]["seed"]["seed_text"]}));
        }
        for (policy, row, kind) in [
            ("baseline", baseline, left),
            (candidate_policy, candidate, right),
        ] {
            if row["expected_ids"] != case["expected_ids"] {
                return Err("etiquetas alteradas en diario".into());
            }
            let relation = text(case, "relation")?;
            let counts = by_policy
                .entry(policy.into())
                .or_default()
                .entry(relation.into())
                .or_default();
            *counts.entry("cases".into()).or_default() += 1;
            *counts.entry(kind.into()).or_default() += 1;
            let response = &row["response"];
            seed_bytes.entry(policy.into()).or_default().push(
                response["seed"]["seed_text"]
                    .as_str()
                    .map_or(0, |s| s.len()) as f64,
            );
            seed_counts
                .entry(policy.into())
                .or_default()
                .push(array(&response["seed"], "memories")?.len() as f64);
            operations_complete &= response["status"] == "complete";
            let evidence_relation = if kind == "permitted_evidence" {
                selected_relation(case, response).unwrap_or("unknown")
            } else {
                kind
            };
            *selected_relations
                .entry(policy.into())
                .or_default()
                .entry(evidence_relation.into())
                .or_default() += 1;
            let u=usage.entry(policy.into()).or_insert(json!({"known_input_tokens":0,"input_token_receipts":0,"known_cost_usd":0.0,"cost_receipts":0,"cost_missing_cases":0,"skipped_empty_operations":0}));
            if let Some(tokens) = response["input_tokens"].as_u64() {
                u["known_input_tokens"] = json!(u["known_input_tokens"].as_u64().unwrap() + tokens);
                u["input_token_receipts"] = json!(u["input_token_receipts"].as_u64().unwrap() + 1);
            }
            if let Some(cost) = response["reported_cost_usd"].as_f64() {
                u["known_cost_usd"] = json!(u["known_cost_usd"].as_f64().unwrap() + cost);
                u["cost_receipts"] = json!(u["cost_receipts"].as_u64().unwrap() + 1);
            } else {
                if response["classification_status"] == "skipped_empty"
                    && response["retrieval"]["candidates"] == json!([])
                {
                    u["skipped_empty_operations"] =
                        json!(u["skipped_empty_operations"].as_u64().unwrap() + 1);
                } else {
                    if (policy.ends_with("per_memory_verified_v1")
                        || policy.ends_with("per_memory_correction_verified_v1")
                        || policy.ends_with("per_memory_correction_verified_v2")
                        || policy.ends_with("per_memory_correction_verified_v3")
                        || policy.ends_with("per_memory_correction_verified_v4")
                        || policy.ends_with("per_memory_correction_verified_v5")
                        || policy.ends_with("per_memory_role_verified_v1")
                        || policy.ends_with("per_memory_role_verified_v2")
                        || policy.ends_with("per_memory_role_batch_v1")
                        || policy.ends_with("per_memory_complete_premise_v1"))
                        && response["status"] == "complete"
                    {
                        let stages = if policy.ends_with("per_memory_complete_premise_v1") {
                            crate::verified::inspect_complete_premise(
                                response,
                                "typesafe/jev-1.13",
                            )?
                        } else if policy.ends_with("per_memory_role_batch_v1") {
                            crate::verified::inspect_role_batch(response, "typesafe/jev-1.13")?
                        } else if policy.ends_with("per_memory_role_verified_v2") {
                            crate::verified::inspect_roles_v2(response, "typesafe/jev-1.13")?
                        } else if policy.ends_with("per_memory_role_verified_v1") {
                            crate::verified::inspect_roles(response, "typesafe/jev-1.13")?
                        } else if policy.ends_with("per_memory_correction_verified_v5") {
                            crate::verified::inspect_corrections_v5(response, "typesafe/jev-1.13")?
                        } else if policy.ends_with("per_memory_correction_verified_v4") {
                            crate::verified::inspect_corrections_v4(response, "typesafe/jev-1.13")?
                        } else if policy.ends_with("per_memory_correction_verified_v3") {
                            crate::verified::inspect_corrections_v3(response, "typesafe/jev-1.13")?
                        } else if policy.ends_with("per_memory_correction_verified_v2") {
                            crate::verified::inspect_corrections_v2(response, "typesafe/jev-1.13")?
                        } else if policy.ends_with("per_memory_correction_verified_v1") {
                            crate::verified::inspect_corrections(response, "typesafe/jev-1.13")?
                        } else {
                            crate::verified::inspect(response, "typesafe/jev-1.13")?
                        };
                        u["known_cost_usd"] = json!(
                            u["known_cost_usd"].as_f64().unwrap()
                                + stages["usage"]["known_cost_usd"].as_f64().unwrap_or(0.0)
                        );
                    }
                    u["cost_missing_cases"] = json!(u["cost_missing_cases"].as_u64().unwrap() + 1);
                }
            }
            *models
                .entry(policy.into())
                .or_default()
                .entry(
                    response["served_model"]
                        .as_str()
                        .unwrap_or("not_reported")
                        .into(),
                )
                .or_default() += 1;
            elapsed
                .entry(policy.into())
                .or_default()
                .push(row["elapsed_ms"].as_f64().ok_or("falta tiempo")?);
        }
        deltas.push(
            candidate["elapsed_ms"].as_f64().ok_or("falta tiempo")?
                - baseline["elapsed_ms"].as_f64().ok_or("falta tiempo")?,
        );
    }
    let times: BTreeMap<_, _> = elapsed
        .iter()
        .map(|(k, v)| (k.clone(), time_distribution(v)))
        .collect();
    let mut witness_rows = Vec::new();
    let mut consensus_rows = Vec::new();
    if let Some(limit) = consensus_limit(candidate_policy) {
        for case in &cases {
            let key = format!("{}__{candidate_policy}", text(case, "id")?);
            let response = &finished.get(key.as_str()).ok_or("falta consenso")?["response"];
            if response["status"] == "complete" {
                consensus_rows.push(consensus_case(
                    case,
                    response,
                    limit,
                    candidate_policy == "jev_roles_primary_veto_v1",
                )?);
            }
        }
    }
    let mut per_memory_rows = Vec::new();
    let mut verification_rows = Vec::new();
    if per_memory_limit(candidate_policy).is_some() {
        for case in &cases {
            let key = format!("{}__{candidate_policy}", text(case, "id")?);
            let response = &finished.get(key.as_str()).ok_or("falta caso per_memory")?["response"];
            if response["status"] == "complete" {
                if (!response["per_memory"]["scoped_checks"].is_null())
                    != ((candidate_policy.ends_with("per_memory_scoped_v1")
                        || candidate_policy.ends_with("per_memory_scoped_v2")
                        || candidate_policy.ends_with("per_memory_scoped_v3")
                        || candidate_policy.ends_with("per_memory_scoped_anchor_v1"))
                        && response["classification_status"] != "skipped_empty")
                {
                    return Err("vías de otra política o ausentes".into());
                }
                if (!response["per_memory"]["factor_checks"].is_null())
                    != (candidate_policy.ends_with("per_memory_factors_v1")
                        && response["classification_status"] != "skipped_empty")
                {
                    return Err("factores de otra política o ausentes".into());
                }
                if candidate_policy.ends_with("per_memory_scored_v1") {
                    crate::score_diagnostic::inspect(response)?;
                } else if !response["per_memory"]["choice_scores"].is_null()
                    || !response["per_memory"]["request_sha256"].is_null()
                {
                    return Err("scores en política no instrumentada".into());
                }
                if candidate_policy.ends_with("per_memory_complete_premise_v1") {
                    verification_rows.push(json!({"id":case["id"],"verification":crate::verified::inspect_complete_premise(response,"typesafe/jev-1.13")?}));
                } else if candidate_policy.ends_with("per_memory_role_batch_v1") {
                    verification_rows.push(json!({"id":case["id"],"verification":crate::verified::inspect_role_batch(response, "typesafe/jev-1.13")?}));
                } else if candidate_policy.ends_with("per_memory_role_verified_v2") {
                    verification_rows.push(json!({"id":case["id"],"verification":crate::verified::inspect_roles_v2(response, "typesafe/jev-1.13")?}));
                } else if candidate_policy.ends_with("per_memory_role_verified_v1") {
                    verification_rows.push(json!({"id":case["id"],"verification":crate::verified::inspect_roles(response, "typesafe/jev-1.13")?}));
                } else if candidate_policy.ends_with("per_memory_correction_verified_v5") {
                    verification_rows.push(json!({"id":case["id"],"verification":crate::verified::inspect_corrections_v5(response, "typesafe/jev-1.13")?}));
                } else if candidate_policy.ends_with("per_memory_correction_verified_v4") {
                    verification_rows.push(json!({"id":case["id"],"verification":crate::verified::inspect_corrections_v4(response, "typesafe/jev-1.13")?}));
                } else if candidate_policy.ends_with("per_memory_correction_verified_v3") {
                    verification_rows.push(json!({"id":case["id"],"verification":crate::verified::inspect_corrections_v3(response, "typesafe/jev-1.13")?}));
                } else if candidate_policy.ends_with("per_memory_correction_verified_v2") {
                    verification_rows.push(json!({"id":case["id"],"verification":crate::verified::inspect_corrections_v2(response, "typesafe/jev-1.13")?}));
                } else if candidate_policy.ends_with("per_memory_correction_verified_v1") {
                    verification_rows.push(json!({"id":case["id"],"verification":crate::verified::inspect_corrections(response, "typesafe/jev-1.13")?}));
                } else if candidate_policy.ends_with("per_memory_verified_v1") {
                    verification_rows.push(json!({"id":case["id"],"verification":crate::verified::inspect(response, "typesafe/jev-1.13")?}));
                } else if !response["per_memory"]["verification"].is_null() {
                    return Err("traza de verificación en política sin verificador".into());
                }
                let mut policy_case = case.clone();
                if candidate_policy.ends_with("per_memory_scoped_anchor_v1") {
                    policy_case["request"]["selection_policy"] =
                        json!("per_memory_scoped_anchor_v1");
                }
                per_memory_rows.push(per_memory_case(
                    &policy_case,
                    response,
                    per_memory_limit(candidate_policy).unwrap(),
                    candidate_policy.ends_with("per_memory_binary_v1")
                        || candidate_policy.ends_with("per_memory_binary_v2")
                        || candidate_policy.ends_with("per_memory_binary_v3")
                        || candidate_policy.ends_with("per_memory_binary_v4")
                        || candidate_policy.ends_with("per_memory_binary_v5")
                        || candidate_policy.ends_with("per_memory_examples_v1")
                        || candidate_policy.ends_with("per_memory_factors_v1")
                        || candidate_policy.ends_with("per_memory_scoped_v1")
                        || candidate_policy.ends_with("per_memory_scoped_v2")
                        || candidate_policy.ends_with("per_memory_scoped_v3")
                        || candidate_policy.ends_with("per_memory_scoped_anchor_v1")
                        || candidate_policy.ends_with("per_memory_scored_v1")
                        || candidate_policy.ends_with("per_memory_verified_v1")
                        || candidate_policy.ends_with("per_memory_correction_verified_v1")
                        || candidate_policy.ends_with("per_memory_correction_verified_v2")
                        || candidate_policy.ends_with("per_memory_correction_verified_v3")
                        || candidate_policy.ends_with("per_memory_correction_verified_v4")
                        || candidate_policy.ends_with("per_memory_correction_verified_v5")
                        || candidate_policy.ends_with("per_memory_role_verified_v1")
                        || candidate_policy.ends_with("per_memory_role_verified_v2")
                        || candidate_policy.ends_with("per_memory_role_batch_v1")
                        || candidate_policy.ends_with("per_memory_complete_premise_v1"),
                )?);
            }
        }
    }
    let mut stages: BTreeMap<String, Value> = BTreeMap::new();
    if candidate_policy == "witness_v1" {
        for case in &cases {
            let id = text(case, "id")?;
            let key = format!("{id}__{candidate_policy}");
            let response = &finished.get(key.as_str()).ok_or("falta caso de testigo")?["response"];
            if let Some(trace) = response.get("applicability").filter(|v| !v.is_null()) {
                let proposed = trace["proposal_id"].clone();
                let proposal_permitted = array(case, "expected_ids")?.contains(&proposed);
                let selected = response["selected_id"].is_string();
                let actual = array(&trace["witness"], "query_bindings")?;
                let gold = array(case, "required_query_bindings")?;
                let missing: Vec<_> = gold
                    .iter()
                    .filter(|g| {
                        !actual.iter().any(|a| {
                            a["dimension"] == g["dimension"]
                                && a["quote"].as_str().is_some_and(|q| {
                                    q.to_lowercase()
                                        .contains(&g["quote"].as_str().unwrap_or("").to_lowercase())
                                })
                        })
                    })
                    .cloned()
                    .collect();
                witness_rows.push(json!({"id":id,"relation":case["relation"],"proposal_id":proposed,"proposal_permitted":proposal_permitted,"selected_after_gate":selected,"outcome":trace["outcome"],"reasons":trace["reasons"],"missing_gold_query_bindings":missing}));
                for stage in ["proposal_usage", "extraction_usage"] {
                    let u = &trace[stage];
                    let total=stages.entry(stage.into()).or_insert(json!({"receipts":0,"known_input_tokens":0,"known_output_tokens":0,"known_cost_usd":0.0,"missing_cost":0}));
                    if let Some(cost) = u["reported_cost_usd"].as_f64() {
                        total["receipts"] = json!(total["receipts"].as_u64().unwrap() + 1);
                        total["known_cost_usd"] =
                            json!(total["known_cost_usd"].as_f64().unwrap() + cost);
                    } else {
                        total["missing_cost"] = json!(total["missing_cost"].as_u64().unwrap() + 1);
                    }
                    for (field, target) in [
                        ("input_tokens", "known_input_tokens"),
                        ("output_tokens", "known_output_tokens"),
                    ] {
                        if let Some(n) = u[field].as_u64() {
                            total[target] = json!(total[target].as_u64().unwrap() + n);
                        }
                    }
                }
            }
        }
    }
    let resources = resource_screen(
        &usage["baseline"],
        &usage[candidate_policy],
        &times["baseline"],
        &times[candidate_policy],
        cases.len(),
        operations_complete,
    );
    let report = json!({"version":"utility-paired-report-v1","candidate_policy":candidate_policy,"cases_sha256":digest(&paired_path)?,
        "journal_sha256":digest(&journal_path)?,"audit_sha256":digest(&audit_path)?,"pairs":cases.len(),"by_policy":by_policy,
        "usage":usage,"models":models,"runner_elapsed_ms":times,"paired_delta_utility_minus_baseline_ms":time_distribution(&deltas),
        "witness_cases":witness_rows,"witness_stage_usage_on_traced_cases":stages,"changes":changes,"blocks":blocks,"useful_evidence_gains":gains,"evidence_completeness_gains":completeness_gains,"by_selected_evidence_relation":selected_relations,"selected_evidence_relation_denominator":"questions; relation of first selected memory only","passes_development_screen":blocks.is_empty() && (gains+completeness_gains+additional_evidence_gains)>0,
        "per_memory_cases":per_memory_rows,"verification_cases":verification_rows,"additional_evidence_gains":additional_evidence_gains,"baseline_record_preservation_required":per_memory_limit(candidate_policy).is_some(),
        "seed_bytes":seed_bytes.iter().map(|(k,v)|(k.clone(),time_distribution(v))).collect::<BTreeMap<_,_>>(),
        "seed_memory_counts":seed_counts.iter().map(|(k,v)|(k.clone(),time_distribution(v))).collect::<BTreeMap<_,_>>(),
        "requested_part_coverage":coverage_rows,"coverage_limitation":"Source-reviewed slot annotations; not factual answer truth or full verification of every context qualifier.",
        "consensus_cases":consensus_rows,"retrieved_candidates_identical":true,"candidate_classification_limit":per_memory_limit(candidate_policy).or_else(||consensus_limit(candidate_policy)),"baseline_classification_limit":20,
        "passes_quality_and_resource_screen":blocks.is_empty() && (gains+completeness_gains+additional_evidence_gains)>0 && resources["passes"]==true,"resource_screen":resources,
        "promoted":false,"independent_review":false,"answer_truth_measured":false,
        "warning":"Known synthetic development. A paired gain is not independent validation, factual answer truth or a population latency guarantee."});
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    file.write_all(
        (serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n").as_bytes(),
    )
    .map_err(|e| e.to_string())
}
