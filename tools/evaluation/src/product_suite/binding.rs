//! Corpus and effective-policy binding for prospective product suites.
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Artifact {
    pub(super) path: String,
    sha256: String,
}

pub(super) struct Binding {
    pub(super) memories: Vec<Value>,
    policy: Value,
}

impl Binding {
    pub(super) fn load(root: &Path, artifact: &Artifact, policy: Value) -> Result<Self, String> {
        if !hash_valid(&artifact.sha256)
            || artifact.path.is_empty()
            || Path::new(&artifact.path)
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err("ruta o hash de corpus inválido".into());
        }
        let path = root.join(&artifact.path);
        if !path
            .canonicalize()
            .map_err(|e| e.to_string())?
            .starts_with(root.canonicalize().map_err(|e| e.to_string())?)
        {
            return Err("corpus fuera del banco".into());
        }
        let bytes = read(&path)?;
        if digest(&bytes) != artifact.sha256 {
            return Err("hash de corpus distinto del congelado".into());
        }
        let memories: Vec<Value> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if memories.is_empty() {
            return Err("corpus vacío".into());
        }
        let mut identities = HashMap::new();
        let mut revisions = HashSet::new();
        for m in &memories {
            let id = required(m, "id")?;
            let scope = (
                required(m, "scope_kind")?,
                required(m, "scope_id")?,
                required(m, "entity_id")?,
            );
            required(m, "original_text")?;
            required(m, "summary")?;
            required(&m["source"], "ref")?;
            let revision = m["revision"]
                .as_u64()
                .filter(|r| *r > 0)
                .ok_or("revisión de corpus inválida")?;
            if !revisions.insert((id, revision)) {
                return Err("revisión duplicada en corpus".into());
            }
            if identities
                .insert(id, scope)
                .is_some_and(|previous| previous != scope)
            {
                return Err("identidad de corpus cambia de ámbito o entidad".into());
            }
        }
        if policy.as_object().is_none_or(|p| p.len() != 4)
            || !matches!(
                policy["provider"].as_str(),
                Some("decisions" | "jev" | "jev114")
            )
            || !matches!(
                policy["selection_policy"].as_str(),
                Some("baseline" | "per_memory_v1" | "per_memory_v2")
            )
            || policy["search"].as_object().is_none_or(|p| p.len() != 2)
            || policy["search"]["limit"]
                .as_u64()
                .is_none_or(|n| !(1..=20).contains(&n))
            || !matches!(
                policy["search"]["query_mode"].as_str(),
                Some("keywords" | "natural")
            )
            || policy["budgets"].as_object().is_none_or(|p| p.len() != 3)
            || policy["budgets"]["input_bytes"]
                .as_u64()
                .is_none_or(|n| !(1..=32768).contains(&n))
            || policy["budgets"]["seed_bytes"]
                .as_u64()
                .is_none_or(|n| !(1..=65536).contains(&n))
            || policy["budgets"]["deadline_ms"]
                .as_u64()
                .is_none_or(|n| !(1..=25000).contains(&n))
        {
            return Err("política congelada inválida para banco ligado a corpus".into());
        }
        Ok(Self { memories, policy })
    }

    pub(super) fn validate_case(&self, case: &Case) -> Result<(), String> {
        let r = &case.request;
        let effective = json!({
            "provider":r.get("provider").cloned().unwrap_or(json!("decisions")),
            "selection_policy":r.get("selection_policy").cloned().unwrap_or(json!("baseline")),
            "search":{"limit":r["search"].get("limit").filter(|v| !v.is_null()).cloned().unwrap_or(json!(20)),
                "query_mode":r["search"].get("query_mode").filter(|v| !v.is_null()).cloned().unwrap_or(json!("keywords"))},
            "budgets":{"input_bytes":r["budgets"].get("input_bytes").cloned().unwrap_or(json!(32768)),
                "seed_bytes":r["budgets"].get("seed_bytes").cloned().unwrap_or(json!(16384)),
                "deadline_ms":r["budgets"].get("deadline_ms").cloned().unwrap_or(json!(25000))}
        });
        if effective != self.policy {
            return Err(format!(
                "{}: solicitud contradice política congelada",
                case.id
            ));
        }
        let search = &r["search"];
        required(search, "query")?;
        required(search, "scope_kind")?;
        required(search, "scope_id")?;
        let as_of = required(search, "as_of")?;
        if !utc_shape(as_of) {
            return Err("as_of debe usar UTC con milisegundos".into());
        }
        let scoped: Vec<_> = self
            .memories
            .iter()
            .filter(|m| {
                m["scope_kind"] == search["scope_kind"] && m["scope_id"] == search["scope_id"]
            })
            .collect();
        // All records in the scope bind groups, including distractors and old revisions.
        let sources: HashSet<_> = scoped
            .iter()
            .filter_map(|m| m["source"]["ref"].as_str())
            .collect();
        let entities: HashSet<_> = scoped
            .iter()
            .filter_map(|m| m["entity_id"].as_str())
            .collect();
        if sources != case.source_groups.iter().map(String::as_str).collect()
            || entities != case.entity_groups.iter().map(String::as_str).collect()
        {
            return Err(format!(
                "{}: grupos no corresponden al corpus del ámbito",
                case.id
            ));
        }
        if (case.expected_outcome == "evidence_selected") != !case.expected_evidence.is_empty() {
            return Err("resultado esperado contradice evidencia anotada".into());
        }
        let mut seen = HashSet::new();
        for e in &case.expected_evidence {
            if !seen.insert(&e.memory_id) {
                return Err("evidencia gold duplicada".into());
            }
            let m = scoped
                .iter()
                .filter(|m| m["id"] == e.memory_id)
                .max_by_key(|m| m["revision"].as_u64())
                .ok_or("evidencia gold ausente del ámbito")?;
            if m["revision"].as_u64() != Some(e.revision)
                || m["state"] != "active"
                || (!search["entity_id"].is_null() && search["entity_id"] != m["entity_id"])
            {
                return Err("evidencia gold obsoleta, inactiva o de otra entidad".into());
            }
            for key in ["valid_from", "valid_until"] {
                if !m[key].is_null() {
                    let date = m[key]
                        .as_str()
                        .filter(|s| utc_shape(s))
                        .ok_or("fecha de corpus inválida")?;
                    if (key == "valid_from" && date > as_of)
                        || (key == "valid_until" && date <= as_of)
                    {
                        return Err("evidencia gold fuera de vigencia".into());
                    }
                }
            }
        }
        Ok(())
    }
}
fn required<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("falta campo textual {key}"))
}
fn utc_shape(s: &str) -> bool {
    s.len() == 24
        && s.bytes().enumerate().all(|(i, b)| match i {
            4 | 7 => b == b'-',
            10 => b == b'T',
            13 | 16 => b == b':',
            19 => b == b'.',
            23 => b == b'Z',
            _ => b.is_ascii_digit(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Binding, Case) {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/product-contract-v1");
        let suite: Suite =
            serde_json::from_slice(&read(&root.join("suite.json")).unwrap()).unwrap();
        let policy = serde_json::from_slice(&read(&root.join("policy.json")).unwrap()).unwrap();
        let binding = Binding::load(&root, suite.corpus.as_ref().unwrap(), policy).unwrap();
        let text = std::fs::read_to_string(root.join("development.jsonl")).unwrap();
        (
            binding,
            serde_json::from_str(text.lines().next().unwrap()).unwrap(),
        )
    }
    #[test]
    fn prospective_bank_is_bound_without_executing_final() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/product-contract-v1/suite.json");
        let report = validate(&root).unwrap();
        assert_eq!(report["corpus_binding"], "passed");
        assert_eq!(report["policy_binding"], "passed");
        assert_eq!(report["partitions"]["final"], 6);
        assert_eq!(report["final_executed"], false);
    }
    #[test]
    fn rejects_policy_drift_and_contradictory_labels() {
        let (binding, mut case) = fixture();
        binding.validate_case(&case).unwrap();
        case.request["provider"] = json!("jev");
        assert!(
            binding
                .validate_case(&case)
                .unwrap_err()
                .contains("política")
        );
        case.request["provider"] = json!("decisions");
        case.expected_outcome = "abstained".into();
        assert!(
            binding
                .validate_case(&case)
                .unwrap_err()
                .contains("contradice evidencia")
        );
    }
    #[test]
    fn rejects_foreign_stale_inactive_and_duplicate_gold() {
        let (mut binding, mut case) = fixture();
        case.expected_evidence[0].revision = 2;
        assert!(
            binding
                .validate_case(&case)
                .unwrap_err()
                .contains("obsoleta")
        );
        case.expected_evidence[0].revision = 1;
        let id = case.expected_evidence[0].memory_id.clone();
        case.expected_evidence[0].memory_id = "unknown".into();
        assert!(
            binding
                .validate_case(&case)
                .unwrap_err()
                .contains("ausente")
        );
        case.expected_evidence[0].memory_id = id;
        binding.memories[0]["state"] = json!("archived");
        assert!(
            binding
                .validate_case(&case)
                .unwrap_err()
                .contains("inactiva")
        );
        binding.memories[0]["state"] = json!("active");
        binding.memories[0]["valid_until"] = case.request["search"]["as_of"].clone();
        assert!(
            binding
                .validate_case(&case)
                .unwrap_err()
                .contains("vigencia")
        );
        binding.memories[0]["valid_until"] = Value::Null;
        case.expected_evidence.push(ExpectedEvidence {
            memory_id: case.expected_evidence[0].memory_id.clone(),
            revision: 1,
            relation: "supports_answer".into(),
        });
        assert!(
            binding
                .validate_case(&case)
                .unwrap_err()
                .contains("duplicada")
        );
    }
    #[test]
    fn groups_must_include_distractors_not_only_selected_memory() {
        let (mut binding, case) = fixture();
        binding.memories[1]["source"]["ref"] = json!("synthetic://unlisted-source");
        assert!(binding.validate_case(&case).unwrap_err().contains("grupos"));
    }
}
