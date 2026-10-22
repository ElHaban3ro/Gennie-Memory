//! Known development only; never substitutes this measurement for extraction quality.
use super::*;
fn relation_label(p: &Value) -> Result<Value, String> {
    let mut v = json!({});
    for key in ["relation", "selected", "world_roles", "review_required"] {
        v[key] = p.get(key).ok_or("etiqueta de relación incompleta")?.clone();
    }
    text(&v, "relation")?;
    boolean(&v, "selected")?;
    boolean(&v, "review_required")?;
    let roles = list(&v, "world_roles")?
        .iter()
        .map(|r| r.as_str().ok_or("rol inválido"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if roles.is_empty()
        || roles.len() != list(&v, "world_roles")?.len()
        || roles.iter().any(|r| !["A", "B", "C", "none"].contains(r))
    {
        return Err("roles inválidos".into());
    }
    v["world_roles"] = json!(roles);
    Ok(v)
}
fn indexed(v: &Value) -> Result<BTreeMap<String, Value>, String> {
    let mut map = BTreeMap::new();
    for p in list(v, "pairs")? {
        if map.insert(text(p, "id")?, p.clone()).is_some() {
            return Err("anotación duplicada".into());
        }
    }
    Ok(map)
}
pub fn score(bank: &Path, raw: &Path, output: &Path) -> Result<(), String> {
    crate::scoped_definition_probe::write_new(output, &reconstruct_score(bank, raw)?)
}
pub(super) fn reconstruct_score(bank: &Path, raw: &Path) -> Result<Value, String> {
    let manifest = read(&bank.join("manifest.json"))?;
    if manifest["version"] != "documentary-known100-v1-reviewed" {
        return Err("banco de suficiencia no revisado".into());
    }
    let files = manifest["files_sha256"]
        .as_object()
        .ok_or("faltan hashes")?;
    for required in [
        "source-only-reviewed.json",
        "author-labels.json",
        "labels-reviewed.json",
        "blind-review.json",
        "blind-review-final.json",
        "blind-review-counts.json",
        "adjudication.json",
    ] {
        if !files.contains_key(required) {
            return Err("archivo requerido no ligado".into());
        }
    }
    for (p, h) in files {
        if p.contains('/') || p.contains("..") || h.as_str() != Some(hash(&bank.join(p))?.as_str())
        {
            return Err("hash de banco inválido".into());
        }
    }
    let external = manifest["external_sha256"]
        .as_object()
        .ok_or("faltan contratos")?;
    for required in [
        "docs/evaluation/evidence-contribution-v3-contract.md",
        "docs/evaluation/evidence-contribution-v4-contract.md",
        "docs/evaluation/epistemic-binding-v5-contract.md",
        "docs/evaluation/measurement-identity-v1-extension.md",
        "docs/evaluation/request-basis-v1-extension.md",
    ] {
        if !external.contains_key(required) {
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
    let s = read(&source)?;
    if r["version"] != "epistemic-binding-v5-report"
        || r["input_sha256"] != hash(&source)?
        || r["pairs"] != reconstruct(&s)?["pairs"]
    {
        return Err("resultado no reproduce fuente/código".into());
    }
    let witnessed = audit_assertion_witnesses(&s, &r)?;
    let author = read(&bank.join("author-labels.json"))?;
    let review = read(&bank.join("blind-review-final.json"))?;
    let counts = read(&bank.join("blind-review-counts.json"))?;
    let adj = read(&bank.join("adjudication.json"))?;
    if review["ready_for_manual_execution"] != true
        || adj["reconciled_before_execution"] != true
        || adj["source_sha256"] != hash(&source)?
        || adj["author_labels_sha256"] != hash(&bank.join("author-labels.json"))?
        || adj["blind_review_sha256"] != hash(&bank.join("blind-review-final.json"))?
        || adj["blind_counts_sha256"] != hash(&bank.join("blind-review-counts.json"))?
        || adj["resolved_labels_sha256"] != hash(&bank.join("labels-reviewed.json"))?
    {
        return Err("revisión no conciliada".into());
    }
    let mut expected = indexed(&author)?;
    let mut resolved = indexed(&read(&bank.join("labels-reviewed.json"))?)?;
    let mut reviewed = indexed(&review)?;
    let mut counted = indexed(&counts)?;
    if expected.len() != reviewed.len() || expected.len() != counted.len() {
        return Err("cobertura de revisión desigual".into());
    }
    let mut errors = Vec::new();
    let mut selected = 0;
    let mut review_required = 0;
    let mut qualified = 0;
    let mut extra_qualification = 0;
    let mut assertions = 0;
    for p in list(&r, "pairs")? {
        let id = text(p, "id")?;
        let gold = expected.remove(&id).ok_or("falta etiqueta")?;
        let blind = reviewed.remove(&id).ok_or("falta revisión")?;
        let count = counted.remove(&id).ok_or("falta cuenta revisada")?;
        let target = relation_label(&gold)?;
        if target != relation_label(&blind)?
            || gold["assertable_facts"].as_u64().is_none()
            || gold["assertable_facts"] != count["assertable_facts"]
        {
            return Err("etiquetas y revisión no conciliadas".into());
        }
        let a = &p["result"];
        let minimum =
            boolean(&gold, "qualification_required")? || boolean(&blind, "qualification_required")?;
        let resolved_label = resolved.remove(&id).ok_or("falta etiqueta conciliada")?;
        if relation_label(&resolved_label)? != target
            || resolved_label["assertable_facts"] != gold["assertable_facts"]
            || boolean(&resolved_label, "qualification_required")? != minimum
        {
            return Err("mínimo obligatorio no conciliado".into());
        }
        if relation_label(a)? != target
            || a["assertable_facts"] != gold["assertable_facts"]
            || minimum && a["qualification_required"] != true
        {
            errors.push(json!({"id":id,"expected":gold,"review":blind,"actual":a}));
        }
        selected += usize::from(a["selected"] == true);
        review_required += usize::from(a["review_required"] == true);
        qualified += usize::from(a["qualification_required"] == true);
        extra_qualification += usize::from(!minimum && a["qualification_required"] == true);
        assertions += a["assertable_facts"]
            .as_u64()
            .ok_or("cuenta de salida inválida")?;
    }
    if !resolved.is_empty()
        || !expected.is_empty()
        || !reviewed.is_empty()
        || !counted.is_empty()
        || manifest["pairs"].as_u64() != Some(list(&r, "pairs")?.len() as u64)
    {
        return Err("sobran etiquetas/revisión".into());
    }
    Ok(
        json!({"version":"documentary-known100-v1-score","bank_manifest_sha256":hash(&bank.join("manifest.json"))?,"raw_report_sha256":hash(raw)?,"pairs":list(&r,"pairs")?.len(),"correct_boundary_outcomes":list(&r,"pairs")?.len()-errors.len(),"errors":errors,"passes_manual_representation":errors.is_empty(),"selected":selected,"review_required":review_required,"qualification_required":qualified,"additional_conservative_qualifications":extra_qualification,"invariant_requested_assertions":assertions,"independently_witnessed_assertions":witnessed,"manual_facts_not_extraction":true,"passes_autonomous_quality":false,"fresh_validation":false,"historical_metrics_modified":false,"provider_calls":0,"known_provider_cost_usd":0}),
    )
}
