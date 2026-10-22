//! Reviewed minimal-pair development screen, not population validation.
use crate::{
    campaign::digest,
    contribution_role_contract as contract,
    isolated_provider_probe::{lines, read},
    scoped_definition_probe::write_new,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
const PROTOCOL: &str = "docs/evaluation/contribution-role-v3-minimal-protocol-2026-10-20.md";

pub(crate) fn bank(d: &Path) -> Result<Vec<Value>, String> {
    let manifest = read(&d.join("manifest.json"))?;
    if manifest["version"] != "evidence-contribution-v3-reviewed"
        || manifest["provider_calls_before_freeze"] != 0
    {
        return Err("banco no congelado antes de red".into());
    }
    let contract_path = "docs/evaluation/evidence-contribution-v3-contract.md";
    let contract_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(contract_path);
    if manifest["contract_path"] != contract_path
        || manifest["contract_sha256"] != digest(&contract_file)?
    {
        return Err("contrato de revisión alterado".into());
    }
    let files = manifest["files_sha256"].as_object().ok_or("sin hashes")?;
    for required in [
        "author-initial.json",
        "source-only.json",
        "initial-manifest.json",
        "blind-review-initial.json",
        "development.json",
        "source-only-reviewed.json",
        "blind-review-final.json",
        "adjudication.json",
    ] {
        if !files.contains_key(required) {
            return Err("procedencia de banco incompleta".into());
        }
    }
    for (file, h) in manifest["files_sha256"]
        .as_object()
        .ok_or("sin hashes de banco")?
    {
        let p = Path::new(file);
        if p.components().count() != 1 || digest(&d.join(p))? != *h {
            return Err("archivo de banco alterado".into());
        }
    }
    let b = read(&d.join("development.json"))?;
    let reviewed = read(&d.join("blind-review-final.json"))?;
    let source = read(&d.join("source-only-reviewed.json"))?;
    if b["version"] != "evidence-contribution-v3-development-reviewed"
        || b["synthetic"] != true
        || b["fresh_validation"] != false
    {
        return Err("versión de banco inválida".into());
    }
    let cases = b["cases"]
        .as_array()
        .filter(|c| c.len() == 16)
        .ok_or("banco no completo")?;
    let reviews = reviewed
        .as_array()
        .filter(|r| r.len() == cases.len())
        .ok_or("revisión incompleta")?;
    let sources = source
        .as_array()
        .filter(|r| r.len() == cases.len())
        .ok_or("fuentes incompletas")?;
    let mut seen = BTreeSet::new();
    for ((c, r), s) in cases.iter().zip(reviews).zip(sources) {
        let id = c["id"].as_str().ok_or("sin id")?;
        if !seen.insert(id)
            || r["id"] != id
            || s["id"] != id
            || c["question"] != s["question"]
            || c["memories"] != s["memories"]
        {
            return Err("banco/revisión/fuente no corresponden".into());
        }
        let query = c["question"]
            .as_str()
            .filter(|q| !q.is_empty())
            .ok_or("sin pregunta")?;
        let memories = c["memories"]
            .as_array()
            .filter(|m| m.len() == 4)
            .ok_or("sin cuatro fuentes")?;
        let labels = c["labels"]
            .as_array()
            .filter(|m| m.len() == 4)
            .ok_or("etiquetas incompletas")?;
        let judgments = r["judgments"]
            .as_array()
            .filter(|m| m.len() == 4)
            .ok_or("juicios incompletos")?;
        let mut ids = BTreeSet::new();
        for ((m, l), j) in memories.iter().zip(labels).zip(judgments) {
            let mid = m["id"].as_str().ok_or("sin id de memoria")?;
            let text = m["text"]
                .as_str()
                .filter(|t| !t.is_empty())
                .ok_or("sin fuente")?;
            if !ids.insert(mid)
                || m["revision"].as_u64() != Some(1)
                || l["id"] != mid
                || j["id"] != mid
                || l["role"] != j["role"]
                || j["ambiguous"] != false
            {
                return Err("juicio sin conciliar o identidad inválida".into());
            }
            if !matches!(l["role"].as_str(), Some("A" | "B" | "C" | "none")) {
                return Err("rol inválido".into());
            }
            for q in [&l["source_quote"], &j["source_quote"]] {
                if !q
                    .as_str()
                    .is_some_and(|q| !q.is_empty() && text.contains(q))
                {
                    return Err("cita no literal".into());
                }
            }
            if !j["question_quote"]
                .as_str()
                .is_some_and(|q| !q.is_empty() && query.contains(q))
            {
                return Err("cita de pregunta no literal".into());
            }
        }
    }
    Ok(cases.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reviewed_bank_rejects_tampering_and_missing_provenance_hashes() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/evidence-contribution-v3");
        assert_eq!(bank(&source).unwrap().len(), 16);
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "iromem-contribution-bank-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&temp).unwrap();
        for p in fs::read_dir(&source).unwrap() {
            let p = p.unwrap().path();
            if p.is_file() {
                fs::copy(&p, temp.join(p.file_name().unwrap())).unwrap();
            }
        }
        assert!(bank(&temp).is_ok());
        fs::write(temp.join("blind-review-final.json"), "[]").unwrap();
        assert!(bank(&temp).is_err());
        let mut manifest = read(&temp.join("manifest.json")).unwrap();
        manifest["files_sha256"]
            .as_object_mut()
            .unwrap()
            .remove("blind-review-final.json");
        fs::write(
            temp.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(bank(&temp).unwrap_err().contains("procedencia"));
        fs::remove_dir_all(temp).unwrap();
    }
}
fn requests(cases: &[Value], profile: contract::Profile, family: contract::Family) -> Vec<Value> {
    cases.iter().map(|c|json!({"id":c["id"],"payload":family.payload(profile,c["question"].as_str().unwrap(),c["memories"].as_array().unwrap())})).collect()
}
pub(crate) fn validate_spanish_glossary() -> Result<(), String> {
    let base =
        Path::new("outputs/iromem/evaluation/instruction-language-es-v1-preparation-2026-10-20");
    let review = read(&base.join("review-final.json"))?;
    let prepared = read(&base.join("preparation.json"))?;
    if review["equivalent"] != true
        || review["reviewed_fields"]
            .as_array()
            .is_none_or(|r| r.len() != 15)
        || review["issues"].as_array().is_none_or(|r| !r.is_empty())
        || prepared["glossary_sha256"]
            != digest(Path::new(
                "tests/fixtures/local-role-instructions-es-v1.json",
            ))?
        || prepared["final_review_sha256"] != digest(&base.join("review-final.json"))?
    {
        return Err("glosario español sin revisión equivalente vinculada".into());
    }
    Ok(())
}
pub fn prepare(input: &Path, d: &Path) -> Result<(), String> {
    prepare_profile(input, d, contract::Profile::Initial, contract::Family::Jev)
}
pub fn prepare_complete(input: &Path, d: &Path) -> Result<(), String> {
    prepare_profile(input, d, contract::Profile::Complete, contract::Family::Jev)
}
pub fn prepare_strict(input: &Path, d: &Path) -> Result<(), String> {
    prepare_profile(input, d, contract::Profile::Strict, contract::Family::Jev)
}
pub fn prepare_shared(input: &Path, d: &Path) -> Result<(), String> {
    prepare_profile(input, d, contract::Profile::Shared, contract::Family::Jev)
}
pub fn prepare_local(input: &Path, d: &Path) -> Result<(), String> {
    prepare_profile(input, d, contract::Profile::Local, contract::Family::Jev)
}
pub fn prepare_family(input: &Path, d: &Path, family: contract::Family) -> Result<(), String> {
    prepare_profile(input, d, contract::Profile::Local, family)
}
fn prepare_profile(
    input: &Path,
    d: &Path,
    profile: contract::Profile,
    family: contract::Family,
) -> Result<(), String> {
    if family != contract::Family::Jev && profile != contract::Profile::Local {
        return Err("familia solo admite LocalV5".into());
    }
    if family == contract::Family::PerplexitySpanish {
        validate_spanish_glossary()?;
    }
    let complete = profile != contract::Profile::Initial;
    let strict = matches!(
        profile,
        contract::Profile::Strict | contract::Profile::Shared | contract::Profile::Local
    );
    let shared = matches!(
        profile,
        contract::Profile::Shared | contract::Profile::Local
    );
    let cases = bank(input)?;
    let reqs = requests(&cases, profile, family);
    fs::create_dir_all(d).map_err(|e| e.to_string())?;
    for (name, data) in [("inputs.jsonl", &cases), ("requests.jsonl", &reqs)] {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(d.join(name))
            .map_err(|e| e.to_string())?;
        for row in data {
            writeln!(f, "{row}").map_err(|e| e.to_string())?;
        }
        f.sync_all().map_err(|e| e.to_string())?;
    }
    let mut hashes = serde_json::Map::new();
    for p in [
        PROTOCOL,
        "docs/evaluation/evidence-contribution-v3-contract.md",
        "tools/evaluation/src/contribution_role_contract.rs",
        "tools/evaluation/src/contribution_role_probe.rs",
        "bootstrap/src/openrouter/per_memory/contribution_role_probe.rs",
    ] {
        hashes.insert(p.into(), json!(digest(Path::new(p))?));
    }
    if complete {
        let p = "docs/evaluation/contribution-role-complete-v2-protocol-2026-10-20.md";
        hashes.insert(p.into(), json!(digest(Path::new(p))?));
    }
    if strict {
        let p = "docs/evaluation/contribution-role-strict-v3-protocol-2026-10-20.md";
        hashes.insert(p.into(), json!(digest(Path::new(p))?));
    }
    if shared {
        let p = "docs/evaluation/contribution-role-shared-v4-protocol-2026-10-20.md";
        hashes.insert(p.into(), json!(digest(Path::new(p))?));
    }
    let mut prep = json!({"bank_directory":input,"bank_manifest_sha256":digest(&input.join("manifest.json"))?,"inputs_sha256":digest(&d.join("inputs.jsonl"))?,"requests_sha256":digest(&d.join("requests.jsonl"))?,"source_hashes":hashes,"operations":16,"candidate_judgments":64,"max_http_requests":16,"fresh_validation":false,"promoted":false});
    if profile == contract::Profile::Local {
        let path = "docs/evaluation/contribution-local-v5-protocol-2026-10-20.md";
        prep["source_hashes"][path] = json!(digest(Path::new(path))?);
    }
    if family != contract::Family::Jev {
        let path = if family == contract::Family::PerplexitySpanish {
            "docs/evaluation/instruction-language-es-v1-protocol-2026-10-20.md"
        } else if family == contract::Family::MicrosoftContrast {
            "docs/evaluation/negative-choice-microsoft-v1-protocol-2026-10-20.md"
        } else if family == contract::Family::PerplexityContrast {
            "docs/evaluation/negative-choice-contrast-v1-protocol-2026-10-20.md"
        } else if family == contract::Family::GeminiExamples {
            "docs/evaluation/generative-contribution-v3-examples-protocol-2026-10-20.md"
        } else if family == contract::Family::Qwen {
            "docs/evaluation/qwen9b-classifier-v1-protocol-2026-10-22.md"
        } else if family == contract::Family::Gemma {
            "docs/evaluation/gemma-classifier-v1-protocol-2026-10-22.md"
        } else if family == contract::Family::Gemini {
            "docs/evaluation/generative-contribution-v2-protocol-2026-10-20.md"
        } else {
            "docs/evaluation/contribution-local-family-protocol-2026-10-20.md"
        };
        prep["source_hashes"][path] = json!(digest(Path::new(path))?);
    }
    if matches!(
        family,
        contract::Family::Gemini
            | contract::Family::GeminiExamples
            | contract::Family::Gemma
            | contract::Family::Qwen
    ) {
        let path = "tools/evaluation/src/generative_contribution_contract.rs";
        prep["source_hashes"][path] = json!(digest(Path::new(path))?);
        let catalog = "outputs/iromem/evaluation/generative-role-catalog-2026-10-20.json";
        prep["source_hashes"][catalog] = json!(digest(Path::new(catalog))?);
    }
    if family == contract::Family::Qwen {
        let path = "docs/evaluation/qwen9b-classifier-recovery-protocol-2026-10-22.md";
        prep["source_hashes"][path] = json!(digest(Path::new(path))?);
        let path = "outputs/iromem/evaluation/qwen9b-endpoints-2026-10-22.json";
        prep["source_hashes"][path] = json!(digest(Path::new(path))?);
    }
    if family == contract::Family::Gemma {
        let path =
            "outputs/iromem/evaluation/source-text-evidence-design-2026-10-22/gemma-endpoints.json";
        prep["source_hashes"][path] = json!(digest(Path::new(path))?);
    }
    if family == contract::Family::PerplexitySpanish {
        for path in [
            "tools/evaluation/src/spanish_contribution_contract.rs",
            "tests/fixtures/local-role-instructions-es-v1.json",
            "outputs/iromem/evaluation/instruction-language-es-v1-preparation-2026-10-20/review-final.json",
            "outputs/iromem/evaluation/instruction-language-es-v1-preparation-2026-10-20/preparation.json",
        ] {
            prep["source_hashes"][path] = json!(digest(Path::new(path))?);
        }
    }
    profile.set_flags(&mut prep);
    family.set_preparation(&mut prep);
    write_new(&d.join("preparation.json"), &prep)
}
pub fn report(d: &Path, out: &Path) -> Result<(), String> {
    write_new(out, &reconstruct(d)?)
}
pub(crate) fn reconstruct(d: &Path) -> Result<Value, String> {
    let prep = read(&d.join("preparation.json"))?;
    let pre = read(&d.join("preflight.json"))?;
    let bankdir = Path::new(prep["bank_directory"].as_str().ok_or("sin banco")?);
    let cases = bank(bankdir)?;
    let profile = contract::Profile::from_preparation(&prep)?;
    let family = contract::Family::from_preparation(&prep)?;
    if family != contract::Family::Jev && profile != contract::Profile::Local {
        return Err("familia solo admite LocalV5".into());
    }
    let complete = profile != contract::Profile::Initial;
    let strict = matches!(
        profile,
        contract::Profile::Strict | contract::Profile::Shared | contract::Profile::Local
    );
    let shared = matches!(
        profile,
        contract::Profile::Shared | contract::Profile::Local
    );
    let reqs = requests(&cases, profile, family);
    if digest(&bankdir.join("manifest.json"))? != prep["bank_manifest_sha256"]
        || lines(&d.join("inputs.jsonl"))? != cases
        || lines(&d.join("requests.jsonl"))? != reqs
        || digest(&d.join("inputs.jsonl"))? != prep["inputs_sha256"]
        || digest(&d.join("requests.jsonl"))? != prep["requests_sha256"]
        || digest(&d.join("preparation.json"))? != pre["preparation_sha256"]
    {
        return Err("preparación no vinculada".into());
    }
    for (p, h) in pre["sources_sha256"]
        .as_object()
        .ok_or("sin fuentes ejecutadas")?
    {
        if digest(&d.join("source-executed").join(p))? != *h {
            return Err("fuente ejecutada alterada".into());
        }
    }
    for (p, h) in prep["source_hashes"]
        .as_object()
        .ok_or("sin fuentes preparadas")?
    {
        if pre["sources_sha256"][p] != *h {
            return Err("fuente preparada distinta de ejecutada".into());
        }
    }
    let events = lines(&d.join("journal.jsonl"))?;
    let mut starts = BTreeSet::new();
    let mut ended = BTreeMap::new();
    let mut cursor = 0;
    let mut cost = 0.;
    let mut unknown = 0;
    for e in &events {
        let id = e["id"].as_str().ok_or("sin id de evento")?;
        if e["kind"] == "started" {
            if cursor >= reqs.len() || reqs[cursor]["id"] != id || !starts.insert(id) {
                return Err("inicio repetido/fuera de orden".into());
            }
            cursor += 1;
        } else if e["kind"] == "response" || e["kind"] == "failed" {
            if !starts.contains(id) || ended.insert(id, e).is_some() {
                return Err("fin repetido/sin inicio".into());
            }
            let c = if e["kind"] == "response" {
                e["body"]["usage"]["cost"].as_f64()
            } else {
                e["partial_usage"]["reported_cost_usd"].as_f64()
            };
            if let Some(c) = c.filter(|c| c.is_finite() && *c >= 0.) {
                cost += c;
            } else {
                unknown += 1;
            }
        } else {
            return Err("evento desconocido".into());
        }
    }
    unknown += starts.len() - ended.len();
    let mut results = Vec::new();
    let (mut valid, mut wrong, mut lost, mut role_errors, mut exact) = (0, 0, 0, 0, 0);
    let mut confusion = BTreeMap::<String, usize>::new();
    for (c, r) in cases.iter().zip(&reqs) {
        let id = c["id"].as_str().unwrap();
        let m = c["memories"].as_array().unwrap();
        let decoded = ended
            .get(id)
            .filter(|e| e["kind"] == "response")
            .map(|e| contract::decode_family(&e["body"], &r["payload"], m, family));
        if let Some(Ok(roles)) = decoded {
            valid += 1;
            let mut details = Vec::new();
            let mut errors = 0;
            for ((role, l), memory) in roles.iter().zip(c["labels"].as_array().unwrap()).zip(m) {
                let expected = l["role"].as_str().unwrap();
                let mismatch = role != expected;
                errors += usize::from(mismatch);
                wrong += usize::from(role != "none" && expected == "none");
                lost += usize::from(role == "none" && expected != "none");
                *confusion.entry(format!("{expected}->{role}")).or_default() += 1;
                details.push(json!({"id":memory["id"],"revision":memory["revision"],"expected_role":expected,"actual_role":role,"role_correct":!mismatch}));
            }
            role_errors += errors;
            exact += usize::from(errors == 0);
            results.push(json!({"id":id,"valid":true,"selected":contract::selected(&roles,m)?,"judgments":details}));
        } else {
            results.push(json!({"id":id,"valid":false,"error":decoded.map(|r|r.err())}));
        }
    }
    let version = if family != contract::Family::Jev {
        format!("contribution-local-{}-minimal-v1", family.name())
    } else {
        (if profile == contract::Profile::Local {
            "contribution-local-v5-minimal-screen"
        } else if shared {
            "contribution-role-shared-v4-screen"
        } else if strict {
            "contribution-role-strict-v3-screen"
        } else if complete {
            "contribution-role-complete-v2-screen"
        } else {
            "contribution-role-v3-minimal-screen"
        })
        .to_owned()
    };
    Ok(
        json!({"version":version,"http_requests":starts.len(),"ended_http_requests":ended.len(),"unrun_questions":16-starts.len(),"uncertain_http_requests":starts.len()-ended.len(),"valid_questions":valid,"valid_judgments":valid*4,"role_errors":if valid==0{None}else{Some(role_errors)},"unpermitted_ids":if valid==0{None}else{Some(wrong)},"lost_useful_ids":if valid==0{None}else{Some(lost)},"exact_role_sets":exact,"confusion":confusion,"known_cost_usd":cost,"unknown_cost_operations":unknown,"screen_passed":valid==16&&role_errors==0&&wrong==0&&lost==0&&unknown==0&&(family!=contract::Family::Gemma||cost<=0.005)&&(family!=contract::Family::Qwen||cost<=0.01),"fresh_validation":false,"resource_gate_measured":false,"runtime_recall_measured":false,"promoted":false,"journal_sha256":digest(&d.join("journal.jsonl"))?,"cases":results}),
    )
}
