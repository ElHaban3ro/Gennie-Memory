//! Prospective split integrity, not semantic grading or a network runner.
mod binding;
mod regression;
mod review;
mod run;
pub fn run_baseline_regression(
    manifest: &Path,
    binary: &Path,
    output: &Path,
) -> Result<Value, String> {
    regression::run(manifest, binary, output)
}
pub fn audit_campaign(suite: &Path, binary: &Path, output: &Path) -> Result<Value, String> {
    run::audit(suite, binary, output)
}
pub fn run_campaign(suite: &Path, binary: &Path, output: &Path) -> Result<Value, String> {
    run::run(suite, binary, output)
}
pub fn audit_reviews(paths: &[&Path]) -> Result<Value, String> {
    review::audit(paths)
}
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Component, Path};

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    contract_version: u32,
    corpus: Option<binding::Artifact>,
    policy_path: String,
    policy_sha256: String,
    protocol: Protocol,
    partitions: Vec<Partition>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Protocol {
    max_calls: u64,
    max_seconds: u64,
    max_cost_usd: Option<f64>,
    unlimited_cost_authorized: bool,
    min_useful_recall: f64,
    max_unrelated_selections: u64,
    max_degradations: u64,
    max_p95_ms: f64,
    label_source: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Partition {
    split: String,
    path: String,
    sha256: String,
    count: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    source_groups: Vec<String>,
    entity_groups: Vec<String>,
    family_groups: Vec<String>,
    request: Value,
    expected_outcome: String,
    expected_evidence: Vec<ExpectedEvidence>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedEvidence {
    memory_id: String,
    revision: u64,
    relation: String,
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    let mut bytes = vec![];
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(8_388_609)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 8_388_608 {
        return Err("archivo supera 8 MiB".into());
    }
    Ok(bytes)
}
fn hash_valid(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn validate(path: &Path) -> Result<Value, String> {
    let bytes = read(path)?;
    let suite: Suite = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let p = &suite.protocol;
    if suite.contract_version != 1
        || !hash_valid(&suite.policy_sha256)
        || p.max_calls == 0
        || p.max_seconds == 0
        || !(0.0..=1.0).contains(&p.min_useful_recall)
        || !p.max_p95_ms.is_finite()
        || p.max_p95_ms <= 0.0
        || p.max_cost_usd.is_some_and(|c| !c.is_finite() || c <= 0.0)
        || (p.max_cost_usd.is_none() != p.unlimited_cost_authorized)
        || !matches!(
            p.label_source.as_str(),
            "agent_synthetic_unreviewed" | "blind_synthetic_adjudication" | "human_reviewed"
        )
    {
        return Err("protocolo inválido: versión, huella, métricas y presupuestos deben fijarse antes de evaluar".into());
    }
    let mut splits = HashSet::new();
    let mut ids = HashSet::new();
    let mut fingerprints = HashSet::new();
    let mut groups: HashMap<(usize, String), String> = HashMap::new();
    let mut counts = serde_json::Map::new();
    let root = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if Path::new(&suite.policy_path)
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("ruta de política inválida".into());
    }
    let policy = root.join(&suite.policy_path);
    if !policy
        .canonicalize()
        .map_err(|e| e.to_string())?
        .starts_with(root.canonicalize().map_err(|e| e.to_string())?)
        || digest(&read(&policy)?) != suite.policy_sha256
    {
        return Err("la política no coincide con su huella congelada".into());
    }
    let binding = suite
        .corpus
        .as_ref()
        .map(|corpus| {
            let policy_value =
                serde_json::from_slice(&read(&policy)?).map_err(|e| e.to_string())?;
            binding::Binding::load(root, corpus, policy_value)
        })
        .transpose()?;
    for partition in &suite.partitions {
        if !matches!(
            partition.split.as_str(),
            "development" | "calibration" | "final"
        ) || !splits.insert(&partition.split)
            || !hash_valid(&partition.sha256)
            || partition.count == 0
            || Path::new(&partition.path)
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err("partición inválida, duplicada o ruta fuera del banco".into());
        }
        let file = root.join(&partition.path);
        if !file
            .canonicalize()
            .map_err(|e| e.to_string())?
            .starts_with(root.canonicalize().map_err(|e| e.to_string())?)
        {
            return Err("partición fuera del banco".into());
        }
        let content = read(&file)?;
        if digest(&content) != partition.sha256 {
            return Err("hash de partición distinto del congelado".into());
        }
        let text = std::str::from_utf8(&content).map_err(|_| "partición no UTF-8")?;
        let mut count = 0;
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let case: Case = serde_json::from_str(line).map_err(|e| e.to_string())?;
            if case.id.trim().is_empty() || !ids.insert(case.id.clone()) {
                return Err("ID vacío o repetido".into());
            }
            if !matches!(
                case.expected_outcome.as_str(),
                "evidence_selected"
                    | "no_candidates"
                    | "abstained"
                    | "clarification_required"
                    | "not_evaluated"
            ) {
                return Err("resultado esperado desconocido".into());
            }
            if case.expected_evidence.iter().any(|e| {
                e.memory_id.trim().is_empty()
                    || e.revision == 0
                    || !matches!(
                        e.relation.as_str(),
                        "supports_answer" | "refutes_premise" | "partial_context" | "unassessed"
                    )
            }) {
                return Err("etiqueta de evidencia inválida".into());
            }
            if case.request["contract_version"] != 1
                || !case.request["search"]["query"].is_string()
                || case.request.as_object().is_none_or(|r| {
                    r.keys().any(|k| {
                        ![
                            "contract_version",
                            "search",
                            "provider",
                            "selection_policy",
                            "request_id",
                            "context",
                            "unresolved_references",
                            "budgets",
                        ]
                        .contains(&k.as_str())
                    })
                })
            {
                return Err(
                    "la solicitud runtime debe excluir etiquetas de evaluación y usar contrato v1"
                        .into(),
                );
            }
            if let Some(binding) = &binding {
                binding.validate_case(&case)?;
            }
            let mut fingerprint = case.request.clone();
            fingerprint.as_object_mut().unwrap().remove("request_id");
            if !fingerprints.insert(digest(
                &serde_json::to_vec(&fingerprint).map_err(|e| e.to_string())?,
            )) {
                return Err("consulta runtime repetida".into());
            }
            for (kind, values) in [
                &case.source_groups,
                &case.entity_groups,
                &case.family_groups,
            ]
            .iter()
            .enumerate()
            {
                if values.is_empty() || values.iter().any(|v| v.trim().is_empty()) {
                    return Err("faltan grupos de fuente, entidad o familia".into());
                }
                for value in values.iter() {
                    let previous = groups
                        .entry((kind, value.clone()))
                        .or_insert_with(|| partition.split.clone());
                    if *previous != partition.split {
                        return Err("fuga entre particiones por fuente, entidad o familia".into());
                    }
                }
            }
            count += 1;
        }
        if count != partition.count {
            return Err("cantidad de casos distinta del manifiesto".into());
        }
        counts.insert(partition.split.clone(), json!(count));
    }
    if splits.len() != 3 {
        return Err("se requieren desarrollo, calibración y final separados".into());
    }
    Ok(
        json!({"contract_version":1,"suite_sha256":digest(&bytes),"policy_sha256":suite.policy_sha256,
        "corpus_binding":if binding.is_some(){"passed"}else{"not_provided"},"policy_binding":if binding.is_some(){"passed"}else{"not_evaluated"},
        "partitions":counts,"structural_validation":"passed","semantic_quality":"not_evaluated","runtime_schema_validation":"deferred_to_product",
        "label_source":p.label_source,"calls_executed":0,"final_executed":false,
        "thresholds":{"min_useful_recall":p.min_useful_recall,"max_unrelated_selections":p.max_unrelated_selections,
            "max_degradations":p.max_degradations,"max_p95_ms":p.max_p95_ms},
        "limits":{"max_calls":p.max_calls,"max_seconds":p.max_seconds,"max_cost_usd":p.max_cost_usd,"unlimited_cost_authorized":p.unlimited_cost_authorized}}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_leakage_tampering_and_gold_in_runtime_request() {
        let root = std::env::temp_dir().join(format!(
            "gennie-memory-product-suite-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let mut partitions = vec![];
        for split in ["development", "calibration", "final"] {
            let row = json!({"id":split,"source_groups":[format!("source-{split}")],"entity_groups":[format!("entity-{split}")],"family_groups":[format!("family-{split}")],
                "request":{"contract_version":1,"search":{"query":format!("consulta {split}")}},"expected_outcome":"abstained","expected_evidence":[]});
            let bytes = serde_json::to_vec(&row).unwrap();
            std::fs::write(root.join(format!("{split}.jsonl")), &bytes).unwrap();
            partitions.push(json!({"split":split,"path":format!("{split}.jsonl"),"sha256":digest(&bytes),"count":1}));
        }
        let mut manifest = json!({"contract_version":1,"policy_path":"policy.json","policy_sha256":digest(b"{}"),"protocol":{"max_calls":6,"max_seconds":30,"max_cost_usd":0.1,"unlimited_cost_authorized":false,"min_useful_recall":0.95,"max_unrelated_selections":0,"max_degradations":0,"max_p95_ms":1500,"label_source":"agent_synthetic_unreviewed"},"partitions":partitions});
        std::fs::write(root.join("policy.json"), b"{}").unwrap();
        let path = root.join("suite.json");
        std::fs::write(&path, manifest.to_string()).unwrap();
        assert_eq!(validate(&path).unwrap()["final_executed"], false);
        let target = root.join("final.jsonl");
        let original = std::fs::read(&target).unwrap();
        let mut row: Value = serde_json::from_slice(&original).unwrap();
        row["family_groups"] = json!(["family-development"]);
        let bytes = serde_json::to_vec(&row).unwrap();
        std::fs::write(&target, &bytes).unwrap();
        assert!(validate(&path).unwrap_err().contains("hash"));
        manifest["partitions"][2]["sha256"] = json!(digest(&bytes));
        std::fs::write(&path, manifest.to_string()).unwrap();
        assert!(validate(&path).unwrap_err().contains("fuga"));
        row["family_groups"] = json!(["family-final"]);
        row["request"]["expected_ids"] = json!(["leak"]);
        let bytes = serde_json::to_vec(&row).unwrap();
        std::fs::write(&target, &bytes).unwrap();
        manifest["partitions"][2]["sha256"] = json!(digest(&bytes));
        std::fs::write(&path, manifest.to_string()).unwrap();
        assert!(validate(&path).unwrap_err().contains("excluir etiquetas"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
