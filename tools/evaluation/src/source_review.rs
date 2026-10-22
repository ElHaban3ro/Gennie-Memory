//! Retrospective source-only review reconciliation. No metric rescoring or relabeling.
use crate::campaign::digest;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
fn read(p: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(p).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn array<'a>(v: &'a Value, k: &str) -> Result<&'a Vec<Value>, String> {
    v[k].as_array().ok_or_else(|| format!("falta {k}"))
}
fn text<'a>(v: &'a Value, k: &str) -> Result<&'a str, String> {
    v[k].as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("falta {k}"))
}
fn ids(v: &Value, key: &str) -> Result<BTreeSet<String>, String> {
    let rows = array(v, key)?;
    let set = rows
        .iter()
        .map(|id| {
            id.as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .ok_or("ID inválido".into())
        })
        .collect::<Result<BTreeSet<_>, String>>()?;
    if set.len() != rows.len() {
        return Err("ID repetido".into());
    }
    Ok(set)
}
fn check(packet: &Value, review: &Value) -> Result<(), String> {
    let mut scopes = BTreeMap::new();
    for c in array(packet, "conversations")? {
        let scope = text(c, "scope_id")?;
        let statements = array(c, "statements")?
            .iter()
            .map(|s| Ok(text(s, "id")?.to_string()))
            .collect::<Result<BTreeSet<_>, String>>()?;
        if scopes.insert(scope, statements).is_some() {
            return Err("ámbito repetido".into());
        }
    }
    let questions = array(packet, "questions")?;
    let rows = array(review, "questions")?;
    if questions.len() != 64 || rows.len() != 64 {
        return Err("la revisión requiere las 64 preguntas".into());
    }
    let mut seen = BTreeSet::new();
    for row in rows {
        let id = text(row, "id")?;
        if !seen.insert(id) {
            return Err("pregunta repetida".into());
        }
        let original = questions
            .iter()
            .find(|q| q["id"] == id)
            .ok_or("pregunta ajena")?;
        if row["scope_id"] != original["scope_id"] {
            return Err("ámbito de revisión alterado".into());
        }
        let allowed = scopes
            .get(text(original, "scope_id")?)
            .ok_or("ámbito inexistente")?;
        let useful = ids(row, "useful_statement_ids")?;
        let context = ids(row, "related_context_statement_ids")?;
        if !useful.is_disjoint(&context)
            || !useful.is_subset(allowed)
            || !context.is_subset(allowed)
        {
            return Err("ID ajeno o contexto mezclado con evidencia".into());
        }
        let roles = row["relation_por_id"].as_object().ok_or("faltan roles")?;
        let rationales = row["rationale_por_id"]
            .as_object()
            .ok_or("faltan razones")?;
        if roles.keys().cloned().collect::<BTreeSet<_>>() != useful
            || rationales.keys().cloned().collect::<BTreeSet<_>>() != useful
        {
            return Err("roles/razones no corresponden a todos los IDs útiles".into());
        }
        for id in &useful {
            if !["supports_answer", "refutes_premise", "pending_or_partial"]
                .contains(&roles[id].as_str().ok_or("rol inválido")?)
                || rationales[id].as_str().is_none_or(|s| s.trim().is_empty())
            {
                return Err("rol o justificación inválidos".into());
            }
        }
        text(row, "answer_boundary")?;
        for a in array(row, "ambiguous_statement_ids")? {
            let id = a
                .as_str()
                .or_else(|| a["id"].as_str())
                .ok_or("ID ambiguo inválido")?;
            if !allowed.contains(id) {
                return Err("ambigüedad de fuente ajena".into());
            }
        }
    }
    Ok(())
}
pub fn run(directory: &Path, output: &Path) -> Result<(), String> {
    let root = Path::new("tests/fixtures/conversation-evidence-v5");
    let (_, cases, _) = crate::evidence::load(root)?;
    let bindings = read(&directory.join("source-bindings.json"))?;
    let diagnostic_path = Path::new(text(&bindings, "diagnostic_spec")?);
    if bindings["diagnostic_spec_sha256"] != digest(diagnostic_path)? {
        return Err("especificación previa alterada".into());
    }
    let diagnostic = read(diagnostic_path)?;
    for name in ["development.jsonl", "memories.json", "manifest.json"] {
        let path = root.join(name);
        let key = path.to_str().ok_or("ruta inválida")?;
        let expected = bindings["bound_inputs"][key]
            .as_str()
            .ok_or("falta vínculo previo de etiquetas")?;
        if diagnostic["inputs"][key] != expected || digest(&path)? != expected {
            return Err("etiquetas no corresponden al banco previo a la revisión".into());
        }
    }
    let spec = read(&directory.join("full-audit-protocol.json"))?;
    let packet = read(&directory.join("full-packet.json"))?;
    let review = read(&directory.join("full-source-review.json"))?;
    if spec["source_sha256"] != digest(&root.join("source.json"))?
        || spec["questions_sha256"] != digest(&root.join("blind-questions.json"))?
        || spec["full_packet_sha256"] != digest(&directory.join("full-packet.json"))?
    {
        return Err("paquete/fuente alterados".into());
    }
    let source = read(&root.join("source.json"))?;
    let questions = read(&root.join("blind-questions.json"))?;
    if packet
        != json!({"synthetic":true,"as_of":source["as_of"],"conversations":source["conversations"],"questions":questions})
    {
        return Err("paquete no corresponde exactamente a fuentes y preguntas".into());
    }
    check(&packet, &review)?;
    let rows = array(&review, "questions")?;
    let mut disagreements = Vec::new();
    let mut relation_disagreements = Vec::new();
    for case in &cases {
        let row = rows
            .iter()
            .find(|r| r["id"] == case["id"])
            .ok_or("pregunta omitida")?;
        let original = array(case, "evidence")?
            .iter()
            .map(|e| Ok(text(e, "statement_id")?.to_string()))
            .collect::<Result<BTreeSet<_>, String>>()?;
        let revised = ids(row, "useful_statement_ids")?;
        for evidence in array(case, "evidence")? {
            let id = text(evidence, "statement_id")?;
            if revised.contains(id) && evidence["relation"] != row["relation_por_id"][id] {
                relation_disagreements.push(json!({"question_id":case["id"],"statement_id":id,"original_relation":evidence["relation"],"review_relation":row["relation_por_id"][id]}));
            }
        }
        if original != revised {
            disagreements.push(json!({"id":case["id"],"query":case["request"]["search"]["query"],"original_useful_statement_ids":original,"review_useful_statement_ids":revised,"only_original":original.difference(&revised).collect::<Vec<_>>(),"only_review":revised.difference(&original).collect::<Vec<_>>(),"review_related_context":row["related_context_statement_ids"],"review_ambiguities":row["ambiguous_statement_ids"],"review_answer_boundary":row["answer_boundary"]}));
        }
    }
    let report = json!({"version":"source-only-retrospective-reconciliation-v1","structural_integrity_valid":true,"questions":64,"disagreements":disagreements,"disagreement_cases":disagreements.len(),"original_labels_changed":false,"original_scores_recomputed":false,"fresh_validation":false,"human_review":false,"same_model_family":true,"review_blind_to_scores_and_author_labels":true,"reviewer_previously_saw_four_source_cases_and_criterion_challenge":true,"packet_sha256":digest(&directory.join("full-packet.json"))?,"review_sha256":digest(&directory.join("full-source-review.json"))?,"warning":"Structural checks and source-only agent judgment do not establish semantic truth. Disagreements require resolution under a fixed rubric; no retroactive metric improvement."});
    let mut report = report;
    report["relation_disagreements"] = json!(relation_disagreements);
    report["provenance_flags_are_declared_not_structurally_proven"] = json!(true);
    report["source_bindings_sha256"] = json!(digest(&directory.join("source-bindings.json"))?);
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    f.write_all(
        serde_json::to_string_pretty(&report)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn review_requires_all_questions_and_same_scope_sources_without_evidence_context_overlap() {
        let questions: Vec<_> = (0..64)
            .map(|i| json!({"id":format!("q{i}"),"scope_id":"s"}))
            .collect();
        let packet = json!({"questions":questions,"conversations":[{"scope_id":"s","statements":[{"id":"x","text":"fact"}]}]});
        let mut review = json!({"questions":(0..64).map(|i|json!({"id":format!("q{i}"),"scope_id":"s","useful_statement_ids":["x"],"relation_por_id":{"x":"supports_answer"},"rationale_por_id":{"x":"requested fact"},"related_context_statement_ids":[],"ambiguous_statement_ids":[],"answer_boundary":"fact"})).collect::<Vec<_>>()});
        assert!(check(&packet, &review).is_ok());
        review["questions"][0]["related_context_statement_ids"] = json!(["x"]);
        assert!(check(&packet, &review).is_err());
        review["questions"][0]["related_context_statement_ids"] = json!([]);
        review["questions"][0]["scope_id"] = json!("other");
        assert!(check(&packet, &review).is_err());
        review["questions"].as_array_mut().unwrap().pop();
        assert!(check(&packet, &review).is_err());
    }
}
