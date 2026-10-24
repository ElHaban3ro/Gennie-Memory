//! Development regression cohort, distinct from the consumed three-way suite.
use super::*;
use std::process::Stdio;
use std::time::{Duration, Instant};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    contract_version: u32,
    corpus: binding::Artifact,
    policy_path: String,
    policy_sha256: String,
    cases: CaseFile,
    protocol: Protocol,
    per_request_reserve_usd: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseFile {
    path: String,
    sha256: String,
    count: usize,
}
fn artifact(root: &Path, path: &str, sha: &str) -> Result<Vec<u8>, String> {
    if path.is_empty()
        || !hash_valid(sha)
        || Path::new(path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("ruta/hash de regresión inválido".into());
    }
    let file = root.join(path).canonicalize().map_err(|e| e.to_string())?;
    if !file.starts_with(root) {
        return Err("artefacto fuera del banco".into());
    }
    let bytes = read(&file)?;
    if digest(&bytes) != sha {
        return Err("hash de regresión distinto del congelado".into());
    }
    Ok(bytes)
}
fn load(
    path: &Path,
) -> Result<(Manifest, binding::Binding, Vec<Case>, std::path::PathBuf), String> {
    let m: Manifest = serde_json::from_slice(&read(path)?).map_err(|e| e.to_string())?;
    if m.contract_version != 1
        || m.cases.count != 30
        || m.protocol.max_calls != 30
        || !(1..=180).contains(&m.protocol.max_seconds)
        || m.protocol
            .max_cost_usd
            .is_none_or(|c| !c.is_finite() || c <= 0.0 || c > 0.02)
        || m.protocol.unlimited_cost_authorized
        || !m.per_request_reserve_usd.is_finite()
        || m.per_request_reserve_usd <= 0.0
        || m.per_request_reserve_usd > m.protocol.max_cost_usd.unwrap()
        || !(0.0..=1.0).contains(&m.protocol.min_useful_recall)
        || !m.protocol.max_p95_ms.is_finite()
        || m.protocol.max_p95_ms <= 0.0
        || m.protocol.label_source != "agent_synthetic_unreviewed"
    {
        return Err("protocolo de regresión inválido".into());
    }
    let root = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let policy: Value = serde_json::from_slice(&artifact(&root, &m.policy_path, &m.policy_sha256)?)
        .map_err(|e| e.to_string())?;
    if policy["provider"] != "decisions" || policy["selection_policy"] != "baseline" {
        return Err("regresión exige Decisions/baseline".into());
    }
    let binding = binding::Binding::load(&root, &m.corpus, policy)?;
    let content = artifact(&root, &m.cases.path, &m.cases.sha256)?;
    let mut cases = vec![];
    let mut ids = HashSet::new();
    for line in std::str::from_utf8(&content)
        .map_err(|_| "casos no UTF8")?
        .lines()
        .filter(|l| !l.trim().is_empty())
    {
        let case: Case = serde_json::from_str(line).map_err(|e| e.to_string())?;
        if case.id.trim().is_empty()
            || !ids.insert(case.id.clone())
            || case
                .expected_evidence
                .iter()
                .any(|e| e.relation != "unassessed")
            || !matches!(
                case.expected_outcome.as_str(),
                "evidence_selected" | "abstained" | "no_candidates"
            )
        {
            return Err("etiqueta de regresión inválida".into());
        }
        binding.validate_case(&case)?;
        cases.push(case);
    }
    if cases.len() != 30
        || !cases.iter().any(|c| {
            c.expected_evidence
                .iter()
                .any(|e| e.memory_id == "pcv1-aliso-1")
        })
    {
        return Err("se requieren 30 casos, incluido Aliso".into());
    }
    Ok((m, binding, cases, root))
}

pub(super) fn run(path: &Path, binary: &Path, output: &Path) -> Result<Value, String> {
    let (manifest, binding, cases, root) = load(path)?;
    let binary = binary.canonicalize().map_err(|e| e.to_string())?;
    std::fs::create_dir(output)
        .map_err(|e| format!("directorio de ejecución debe ser nuevo: {e}"))?;
    let output = output.canonicalize().map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    let mut journal = run::fresh(&output.join("journal.jsonl"))?;
    run::append(
        &mut journal,
        &json!({"kind":"manifest","format":"baseline_regression_30_v1","input_sha256":digest(&read(path)?),"binary_sha256":crate::campaign::digest(&binary)?,"cases_sha256":manifest.cases.sha256,"policy_sha256":manifest.policy_sha256,"development_only":true,"label_source":manifest.protocol.label_source,"max_calls":manifest.protocol.max_calls,"max_seconds":manifest.protocol.max_seconds,"max_cost_usd":manifest.protocol.max_cost_usd,"per_request_reserve_usd":manifest.per_request_reserve_usd,"automatic_retries":false}),
    )?;
    let started = Instant::now();
    let deadline = started + Duration::from_secs(manifest.protocol.max_seconds);
    let mut observations = vec![];
    let mut calls = 0;
    let mut cost = 0.0;
    let mut uncertain = false;
    let result = (|| {
        let database = output.join("corpus.db");
        if !run::command(&binary, &database, &output)
            .arg("load-fixtures")
            .arg(root.join(&manifest.corpus.path))
            .stdout(Stdio::null())
            .status()
            .map_err(|_| "no se pudo cargar corpus")?
            .success()
        {
            return Err("producto rechazó corpus".into());
        }
        let mut session = run::Session::start(&binary, &database, &output)?;
        for case in &cases {
            if Instant::now() >= deadline
                || calls >= manifest.protocol.max_calls
                || cost + manifest.per_request_reserve_usd > manifest.protocol.max_cost_usd.unwrap()
            {
                return Err("presupuesto agotado".into());
            }
            run::append(
                &mut journal,
                &json!({"kind":"started","id":case.id,"request_sha256":digest(&serde_json::to_vec(&case.request).map_err(|e|e.to_string())?),"elapsed_ms":started.elapsed().as_secs_f64()*1000.0}),
            )?;
            uncertain = true;
            calls += 1;
            let response = session.request(
                "recall_v1",
                &case.request,
                deadline
                    .saturating_duration_since(Instant::now())
                    .min(Duration::from_secs(30)),
            )?;
            run::append(
                &mut journal,
                &json!({"kind":"finished","id":case.id,"response":response,"elapsed_ms":started.elapsed().as_secs_f64()*1000.0}),
            )?;
            let usage = &response["usage"];
            if usage["receipt_status"] == "not_called" {
                calls -= 1;
            } else {
                let charged = usage["reported_cost_usd"]
                    .as_f64()
                    .filter(|c| c.is_finite() && *c >= 0.0)
                    .ok_or("coste desconocido; no reintentar")?;
                cost += charged;
                if usage["receipt_status"] != "complete"
                    || charged > manifest.per_request_reserve_usd
                    || cost > manifest.protocol.max_cost_usd.unwrap()
                {
                    return Err("recibo incompleto o límite de coste excedido".into());
                }
            }
            uncertain = false;
            let mut grade = run::grade(case, &response, &binding.memories)?;
            let retrieved: HashSet<_> = response["retrieval"]["candidates"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v["memory_id"].as_str())
                .collect();
            grade["retrieved_expected_evidence"] = json!(
                case.expected_evidence
                    .iter()
                    .filter(|e| retrieved.contains(e.memory_id.as_str()))
                    .count()
            );
            observations.push(grade);
        }
        Ok::<(), String>(())
    })();
    let mut summary = run::summarize("development", &observations, &manifest.protocol);
    summary["status"] = json!(if result.is_ok() {
        "completed"
    } else {
        "stopped"
    });
    summary["stop_reason"] = json!(result.err());
    summary["calls"] = json!(calls);
    summary["reported_cost_usd"] = json!(cost);
    summary["total_cost_usd"] = if uncertain { Value::Null } else { json!(cost) };
    summary["cost_complete"] = json!(!uncertain);
    summary["wall_ms"] = json!(started.elapsed().as_secs_f64() * 1000.0);
    summary["completed_cases"] = json!(observations.len());
    summary["requested_cases"] = json!(30);
    summary["final_partition_used"] = json!(false);
    summary["outcomes_correct"] = json!(
        observations
            .iter()
            .filter(|r| r["outcome_matches"] == true
                && r["unrelated_selections"] == 0
                && r["useful_selected"] == r["expected_evidence"]
                && r["seed_faithful"] == true
                && r["execution_status"] == "completed")
            .count()
    );
    run::append(
        &mut journal,
        &json!({"kind":"campaign_terminal","summary":summary}),
    )?;
    run::save(&output.join("summary.json"), &summary)?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regression_cohort_has_thirty_bound_cases_and_original_failure() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/baseline-evidence-30-v1/manifest.json");
        let (_, _, cases, _) = load(&path).unwrap();
        assert_eq!(cases.len(), 30);
        assert_eq!(
            cases
                .iter()
                .filter(|c| !c.expected_evidence.is_empty())
                .count(),
            20
        );
        let aliso = cases
            .iter()
            .find(|c| {
                c.expected_evidence
                    .iter()
                    .any(|e| e.memory_id == "pcv1-aliso-1")
            })
            .unwrap();
        let original: Case = serde_json::from_str(
            std::fs::read_to_string(
                path.parent()
                    .unwrap()
                    .join("../product-contract-v1/development.jsonl"),
            )
            .unwrap()
            .lines()
            .next()
            .unwrap(),
        )
        .unwrap();
        assert_eq!(aliso.request["search"], original.request["search"]);
        assert_eq!(aliso.request["context"], original.request["context"]);
    }
}
