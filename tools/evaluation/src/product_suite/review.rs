//! Audit independent review artifacts without manufacturing semantic judgments.
use super::*;

pub(super) fn audit(paths: &[&Path]) -> Result<Value, String> {
    if paths.len() != 5 {
        return Err("se requieren suite, paquete, rúbrica y dos revisiones".into());
    }
    let structural = validate(paths[0])?;
    let bytes: Vec<_> = paths.iter().map(|p| read(p)).collect::<Result<_, _>>()?;
    let suite: Suite = serde_json::from_slice(&bytes[0]).map_err(|e| e.to_string())?;
    let root = paths[0]
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let artifact = suite
        .corpus
        .as_ref()
        .ok_or("revisión requiere corpus ligado")?;
    let binding = binding::Binding::load(
        root,
        artifact,
        serde_json::from_slice(&read(&root.join(&suite.policy_path))?)
            .map_err(|e| e.to_string())?,
    )?;
    let mut cases = HashMap::new();
    for partition in &suite.partitions {
        let data = read(&root.join(&partition.path))?;
        for line in std::str::from_utf8(&data)
            .map_err(|_| "partición no UTF-8")?
            .lines()
            .filter(|s| !s.trim().is_empty())
        {
            let case: Case = serde_json::from_str(line).map_err(|e| e.to_string())?;
            cases.insert(case.id.clone(), case);
        }
    }
    let mut packet = HashMap::new();
    for line in std::str::from_utf8(&bytes[1])
        .map_err(|_| "paquete no UTF-8")?
        .lines()
        .filter(|s| !s.trim().is_empty())
    {
        let row: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
        let id = row["case_id"].as_str().ok_or("paquete sin ID")?.to_string();
        let case = cases.get(&id).ok_or("caso ajeno en paquete")?;
        let memories: Vec<_> = binding
            .memories
            .iter()
            .filter(|m| {
                m["scope_kind"] == case.request["search"]["scope_kind"]
                    && m["scope_id"] == case.request["search"]["scope_id"]
            })
            .cloned()
            .collect();
        if row.as_object().is_none_or(|o| o.len() != 3)
            || row["request"] != case.request
            || row["memories"] != json!(memories)
            || packet.insert(id, row).is_some()
        {
            return Err("paquete alterado, duplicado o con campos de etiquetas".into());
        }
    }
    if packet.len() != cases.len() {
        return Err("paquete incompleto".into());
    }
    let packet_hash = digest(&bytes[1]);
    let rubric_hash = digest(&bytes[2]);
    let reviews: Vec<Value> = bytes[3..]
        .iter()
        .map(|b| serde_json::from_slice(b).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    let mut reviewer_ids = HashSet::new();
    let mut disagreements = vec![];
    let mut concerns = vec![];
    for review in &reviews {
        let reviewer = review["reviewer"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or("revisor sin identidad")?;
        if !reviewer_ids.insert(reviewer) {
            return Err("se requieren dos identidades de revisión distintas".into());
        }
        if review["packet_sha256"] != packet_hash || review["rubric_sha256"] != rubric_hash {
            return Err("revisión no corresponde al paquete y rúbrica vigentes".into());
        }
        let rows = review["cases"].as_array().ok_or("revisión sin casos")?;
        let mut seen = HashSet::new();
        for row in rows {
            let id = row["case_id"].as_str().ok_or("revisión sin ID de caso")?;
            let case = cases.get(id).ok_or("revisión de caso ajeno")?;
            if !seen.insert(id) {
                return Err("caso duplicado en revisión".into());
            }
            let expected: HashMap<_, _> = case
                .expected_evidence
                .iter()
                .map(|e| (e.memory_id.as_str(), (e.revision, e.relation.as_str())))
                .collect();
            let judgments = row["assessments"].as_array().ok_or("faltan juicios")?;
            let selected = row["selected_ids"].as_array().ok_or("faltan selecciones")?;
            let selected: Vec<_> = selected
                .iter()
                .map(|v| v.as_str().ok_or("selección inválida"))
                .collect::<Result<_, _>>()?;
            if selected.iter().copied().collect::<HashSet<_>>().len() != selected.len() {
                return Err("selección duplicada".into());
            }
            let memories = packet[id]["memories"].as_array().unwrap();
            let mut assessed = HashSet::new();
            let mut labels = HashMap::new();
            for j in judgments {
                let memory = j["memory_id"].as_str().ok_or("juicio sin memoria")?;
                let revision = j["revision"].as_u64().ok_or("juicio sin revisión")?;
                let relation = j["relation"].as_str().ok_or("juicio sin relación")?;
                if !assessed.insert(memory)
                    || !memories
                        .iter()
                        .any(|m| m["id"] == memory && m["revision"].as_u64() == Some(revision))
                    || !matches!(
                        relation,
                        "supports_answer" | "refutes_premise" | "partial_context" | "unrelated"
                    )
                {
                    return Err("juicio inválido, ajeno o duplicado".into());
                }
                if selected.contains(&memory) != (relation != "unrelated") {
                    return Err("juicio y selección contradictorios".into());
                }
                if relation != "unrelated" {
                    labels.insert(memory, (revision, relation));
                }
            }
            if assessed.len() != memories.len() || selected.iter().any(|id| !assessed.contains(id))
            {
                return Err("juicios incompletos o selección ajena".into());
            }
            if row["rationale"]
                .as_str()
                .is_none_or(|s| s.trim().is_empty())
            {
                return Err("revisión sin fundamento".into());
            }
            let notes = row["concerns"]
                .as_array()
                .ok_or("faltan observaciones de revisión")?;
            if notes.iter().any(|v| v.as_str().is_none()) {
                return Err("observación inválida".into());
            }
            if !notes.is_empty() {
                concerns.push(json!({"reviewer":reviewer,"case_id":id,"concerns":notes}));
            }
            if row["outcome"] != case.expected_outcome || labels != expected {
                disagreements.push(json!({"reviewer":reviewer,"case_id":id,"proposed_outcome":row["outcome"],"expected_outcome":case.expected_outcome,"assessments":judgments}));
            }
        }
        if seen.len() != cases.len() {
            return Err("revisión incompleta".into());
        }
    }
    Ok(
        json!({"structural_validation":structural,"packet_sha256":packet_hash,"rubric_sha256":rubric_hash,
        "review_sha256":[digest(&bytes[3]),digest(&bytes[4])],"reviewers":reviews.iter().map(|r|r["reviewer"].clone()).collect::<Vec<_>>(),
        "cases":cases.len(),"label_agreement":disagreements.is_empty(),"disagreements":disagreements,"concerns":concerns,
        "semantic_adoption":"not_evaluated","human_review":false,"model_family_independence":"not_established",
        "api_calls":0,"review_cost_usd":Value::Null,"final_executed":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audits_preserved_blind_reviews_without_claiming_adoption() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/product-contract-v1/history/draft-1");
        let files = [
            "suite.json",
            "review-packet.jsonl",
            "review-rubric.md",
            "review-a.json",
            "review-b.json",
        ]
        .map(|p| root.join(p));
        let paths: Vec<_> = files.iter().map(|p| p.as_path()).collect();
        let report = audit(&paths).unwrap();
        assert_eq!(report["label_agreement"], true);
        assert_eq!(report["final_executed"], false);
        assert!(!report["concerns"].as_array().unwrap().is_empty());
        let repeated = [paths[0], paths[1], paths[2], paths[3], paths[3]];
        assert!(audit(&repeated).unwrap_err().contains("distintas"));
        let changed_rubric = root.join("../../review-rubric.md");
        let stale = [
            paths[0],
            paths[1],
            changed_rubric.as_path(),
            paths[3],
            paths[4],
        ];
        assert!(audit(&stale).unwrap_err().contains("vigentes"));
    }
}
