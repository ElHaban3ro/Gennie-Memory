//! Small, prospective factor reliability audit. No inference and no gold runtime.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
const FACTORS: [&str; 3] = ["same_context", "requested_fact", "premise_correction"];
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn read(path: &Path) -> Result<(Vec<u8>, Value), String> {
    let b = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v = serde_json::from_slice(&b).map_err(|e| e.to_string())?;
    Ok((b, v))
}
fn write_new(path: &Path, v: &Value) -> Result<(), String> {
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())
}
struct Bank {
    labels: BTreeMap<String, Value>,
    cases: BTreeMap<String, Value>,
    corpus: BTreeMap<String, Value>,
    hashes: Value,
}
fn bank(dir: &Path) -> Result<Bank, String> {
    let (_, manifest) = read(&dir.join("manifest.json"))?;
    for name in [
        "corpus.json",
        "cases.json",
        "labels.json",
        "blind-review.json",
    ] {
        let b = fs::read(dir.join(name)).map_err(|e| e.to_string())?;
        if manifest["sha256"][name] != hash(&b) {
            return Err(format!("bank_hash_mismatch:{name}"));
        }
    }
    let (_, cs) = read(&dir.join("cases.json"))?;
    let (_, ls) = read(&dir.join("labels.json"))?;
    let map = |v: Value| -> Result<BTreeMap<String, Value>, String> {
        let mut out = BTreeMap::new();
        for item in v.as_array().ok_or("bank_array")? {
            let id = item["id"].as_str().ok_or("bank_id")?.to_owned();
            if out.insert(id, item.clone()).is_some() {
                return Err("bank_duplicate_id".into());
            }
        }
        Ok(out)
    };
    let cases = map(cs)?;
    let labels = map(ls)?;
    let (_, corpus) = read(&dir.join("corpus.json"))?;
    let corpus = map(corpus)?;
    if cases.len() != 24 || cases.keys().ne(labels.keys()) {
        return Err("bank_roster".into());
    }
    let mut scopes = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for (id, c) in &cases {
        let label = &labels[id];
        let memory = corpus
            .get(c["memory_id"].as_str().ok_or("case_memory")?)
            .ok_or("missing_memory")?;
        if label["memory_id"] != memory["id"]
            || label["revision"] != memory["revision"]
            || c["revision"] != memory["revision"]
            || c["request"]["search"]["scope_id"] != memory["scope_id"]
            || label["source_sha256"]
                != hash(
                    memory["original_text"]
                        .as_str()
                        .ok_or("source_text")?
                        .as_bytes(),
                )
        {
            return Err("source_label_binding".into());
        }
        if !scopes.insert(c["request"]["search"]["scope_id"].as_str().ok_or("scope")?)
            || !sources.insert(labels[id]["source_sha256"].as_str().ok_or("source_hash")?)
        {
            return Err("source_overlap".into());
        }
    }
    for split in ["development", "calibration", "final"] {
        if cases.values().filter(|c| c["split"] == split).count() != 8 {
            return Err("split_count".into());
        }
    }
    Ok(Bank {
        labels,
        cases,
        corpus,
        hashes: manifest["sha256"].clone(),
    })
}
fn rows(path: &Path) -> Result<(Vec<u8>, Vec<Value>), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
    let mut out = vec![];
    for line in text.lines() {
        out.push(serde_json::from_str(line).map_err(|e| e.to_string())?);
    }
    Ok((bytes, out))
}
fn roster<'a>(
    bank: &Bank,
    rows: &'a [Value],
    split: &str,
    provider: &str,
) -> Result<Vec<&'a Value>, String> {
    let expected: BTreeSet<_> = bank
        .cases
        .iter()
        .filter(|(_, c)| c["split"] == split)
        .map(|(id, _)| id.as_str())
        .collect();
    let mut seen = BTreeSet::new();
    let mut implementations = BTreeSet::new();
    let mut out = vec![];
    for r in rows {
        let id = r["id"].as_str().ok_or("record_id")?;
        if !expected.contains(id) || !seen.insert(id) || r["provider"] != provider {
            return Err("record_roster_or_provider".into());
        }
        if let Some(raw) = r["raw_request"].as_str() {
            if r["request_sha256"] != hash(raw.as_bytes())
                || r["response"]["input_sha256"] != hash(raw.as_bytes())
            {
                return Err(format!("request_byte_hash_binding:{id}"));
            }
            let parsed: Value = serde_json::from_str(raw).map_err(|_| "raw_request_json")?;
            if !contains_fields(&r["response"]["effective_request"], &parsed) {
                return Err(format!("raw_effective_request_binding:{id}"));
            }
        }
        if r["response"]["requested_model"] != model(provider)? {
            return Err(format!("requested_model_binding:{id}"));
        }
        let mut expected_request = bank.cases[id]["request"].clone();
        expected_request["provider"] = json!(provider);
        if !contains_fields(&r["response"]["effective_request"], &expected_request) {
            return Err(format!("response_request_binding:{id}"));
        }
        let implementation = r["response"]["implementation_sha256"]
            .as_str()
            .ok_or("implementation_hash")?;
        if implementation.len() != 64 || !implementation.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("implementation_hash".into());
        }
        implementations.insert(implementation);
        if let Some(candidates) = r["response"]["retrieval"]["candidates"].as_array() {
            let m = &bank.corpus[bank.labels[id]["memory_id"].as_str().unwrap()];
            if candidates.len() != 1
                || candidates[0]["memory_id"] != m["id"]
                || candidates[0]["revision"] != m["revision"]
                || candidates[0]["summary"] != m["summary"]
            {
                return Err(format!("response_source_binding:{id}"));
            }
        } else if r["response"]["execution_status"] == "completed" {
            return Err(format!("response_source_missing:{id}"));
        }
        out.push(r);
    }
    if seen != expected {
        return Err("record_incomplete_split".into());
    }
    if implementations.len() != 1 {
        return Err("mixed_implementations".into());
    }
    Ok(out)
}
fn contains_fields(actual: &Value, expected: &Value) -> bool {
    if let Some(object) = expected.as_object() {
        object.iter().all(|(k, v)| contains_fields(&actual[k], v))
    } else {
        actual == expected
    }
}
fn assessment<'a>(r: &'a Value, label: &Value, model: &str) -> Result<&'a Value, String> {
    let p = &r["response"];
    if p["execution_status"] != "completed"
        || p["usage"]["served_model"] != model
        || p["coverage"]["classification"] != "complete"
    {
        return Err(format!("record_degraded_or_wrong_model:{}", r["id"]));
    }
    let aa = p["assessments"].as_array().ok_or("assessments")?;
    if aa.len() != 1
        || aa[0]["memory_id"] != label["memory_id"]
        || aa[0]["revision"] != label["revision"]
    {
        return Err("candidate_roster".into());
    }
    Ok(&aa[0])
}
fn score(a: &Value, f: &str) -> Result<f64, String> {
    let n = a["factor_scores"][f]
        .as_f64()
        .ok_or_else(|| format!("missing_factor_score:{f}"))?;
    if !n.is_finite() || !(0.0..=1.0).contains(&n) {
        return Err("invalid_factor_score".into());
    }
    Ok(n)
}
fn model(provider: &str) -> Result<&'static str, String> {
    match provider {
        "decisions" => Ok("openai/gpt-6-luna-decisions"),
        "jev" => Ok("typesafe/jev-1.13"),
        _ => Err("unsupported_provider".into()),
    }
}
/// Fixed TWO bins per factor, chosen before inference for this tiny sample.
/// Unsupported bins reject publication; no tuning after opening final.
fn fitted(dir: &Path, records: &Path, profile: &Path, provider: &str) -> Result<Value, String> {
    let b = bank(dir)?;
    let (bytes, rs) = rows(records)?;
    let rs = roster(&b, &rs, "calibration", provider)?;
    let requested_model = model(provider)?;
    let served_models: BTreeSet<_> = rs
        .iter()
        .map(|r| {
            r["response"]["usage"]["served_model"]
                .as_str()
                .ok_or("served_model_missing")
        })
        .collect::<Result<_, _>>()?;
    if served_models.len() != 1 {
        return Err("mixed_training_served_models".into());
    }
    let model = *served_models.iter().next().unwrap();
    let dated = model
        .strip_prefix(requested_model)
        .and_then(|v| v.strip_prefix('-'))
        .is_some_and(|v| v.len() == 8 && v.bytes().all(|b| b.is_ascii_digit()));
    if model != requested_model && !dated {
        return Err("unexpected_served_model".into());
    }
    let profile_bytes = fs::read(profile).map_err(|e| e.to_string())?;
    if rs
        .iter()
        .any(|r| r["factor_profile_sha256"] != hash(&profile_bytes))
    {
        return Err("record_factor_profile_mismatch".into());
    }
    let mut factors = serde_json::Map::new();
    for f in FACTORS {
        let mut counts = [(0usize, 0usize); 2];
        for r in &rs {
            let label = &b.labels[r["id"].as_str().unwrap()];
            let a = assessment(r, label, model)?;
            let p = score(a, f)?;
            let bin = usize::from(p > 0.5);
            counts[bin].0 += 1;
            counts[bin].1 += usize::from(label["factors"][f].as_bool().ok_or("label_factor")?);
        }
        if counts.iter().any(|&(n, _)| n == 0) {
            return Err(format!("unsupported_calibration_bin:{f}"));
        }
        factors.insert(f.into(),json!([{"upper":0.5,"positives":counts[0].1,"n":counts[0].0},{"upper":1.0,"positives":counts[1].1,"n":counts[1].0}]));
    }
    let profile_bytes = fs::read(profile).map_err(|e| e.to_string())?;
    Ok(
        json!({"schema_version":1,"profile":"factor_empirical_bins_v1","profile_sha256":hash(&profile_bytes),"provider":provider,"selection_policy":"per_memory_factors_v1","requested_model":requested_model,"served_model":model,"training_sha256":hash(&bytes),"factors":factors}),
    )
}
pub fn calibrate(
    dir: &Path,
    records: &Path,
    profile: &Path,
    provider: &str,
    out: &Path,
) -> Result<(), String> {
    write_new(out, &fitted(dir, records, profile, provider)?)
}

fn metrics(samples: &[(f64, bool)]) -> Value {
    if samples.is_empty() {
        return json!({"n":0,"brier":null,"ece_5_fixed_bins":null});
    }
    let mut buckets = [(0usize, 0f64, 0f64); 5];
    let mut brier = 0.;
    for &(p, y) in samples {
        let y = if y { 1. } else { 0. };
        brier += (p - y).powi(2);
        let i = ((p * 5.).floor() as usize).min(4);
        buckets[i].0 += 1;
        buckets[i].1 += p;
        buckets[i].2 += y;
    }
    let n = samples.len() as f64;
    let ece = buckets
        .iter()
        .filter(|x| x.0 > 0)
        .map(|&(_k, p, y)| (p - y).abs() / n)
        .sum::<f64>();
    json!({"n":samples.len(),"brier":brier/n,"ece_5_fixed_bins":ece,"bins":buckets.iter().enumerate().map(|(i,&(k,p,y))|json!({"lower":i as f64/5.,"upper":(i+1) as f64/5.,"n":k,"mean_score":if k>0{Some(p/k as f64)}else{None},"positive_rate":if k>0{Some(y/k as f64)}else{None}})).collect::<Vec<_>>()})
}
/// Evaluation preserves incomplete/degraded cases in the denominator, separately.
pub fn report(
    dir: &Path,
    records: &Path,
    calibration: &Path,
    training: &Path,
    profile: &Path,
    provider: &str,
    split: &str,
    out: &Path,
) -> Result<(), String> {
    if !["development", "calibration", "final"].contains(&split) {
        return Err("split".into());
    }
    let b = bank(dir)?;
    let (bytes, rs) = rows(records)?;
    let rs = roster(&b, &rs, split, provider)?;
    let (calbytes, cal) = read(calibration)?;
    let requested_model = model(provider)?;
    let model = cal["served_model"]
        .as_str()
        .ok_or("calibration_served_model")?;
    if cal != fitted(dir, training, profile, provider)? {
        return Err("calibration_not_reproducible".into());
    }
    let profile_bytes = fs::read(profile).map_err(|e| e.to_string())?;
    if rs
        .iter()
        .any(|r| r["factor_profile_sha256"] != hash(&profile_bytes))
    {
        return Err("record_factor_profile_mismatch".into());
    }
    let (_, training_rows) = rows(training)?;
    if rs.first().map(|r| &r["response"]["implementation_sha256"])
        != training_rows
            .first()
            .map(|r| &r["response"]["implementation_sha256"])
    {
        return Err("calibration_implementation_mismatch".into());
    }
    if split == "final"
        && rs
            .iter()
            .any(|r| r["response"]["effective_request"]["calibration"] != cal)
    {
        return Err("final_missing_exact_calibration".into());
    }
    if cal["provider"] != provider
        || cal["requested_model"] != requested_model
        || cal["profile"] != "factor_empirical_bins_v1"
    {
        return Err("calibration_identity".into());
    }
    let mut results = serde_json::Map::new();
    let mut degraded = vec![];
    let mut correct = 0;
    let mut per_case = vec![];
    for r in &rs {
        let label = &b.labels[r["id"].as_str().unwrap()];
        match assessment(r, label, model) {
            Ok(a) => {
                let selected = a["selected"].as_bool().ok_or("selected")?;
                let ok = label["expected_selected"] == selected;
                correct += usize::from(ok);
                per_case.push(json!({"id":r["id"],"segment":b.cases[r["id"].as_str().unwrap()]["segment"],"expected_selected":label["expected_selected"],"selected":selected,"selection_correct":ok,"role":label["role"]}));
            }
            Err(e) => degraded.push(json!({"id":r["id"],"reason":e})),
        }
    }
    for f in FACTORS {
        let bins = cal["factors"][f].as_array().ok_or("calibration_bins")?;
        if bins.len() != 2 || bins[0]["upper"] != 0.5 || bins[1]["upper"] != 1.0 {
            return Err("fixed_two_bins_required".into());
        }
        let mut rates = vec![];
        let mut total = 0;
        for bin in bins {
            let n = bin["n"].as_u64().ok_or("n")?;
            let positives = bin["positives"].as_u64().ok_or("positives")?;
            if n == 0 || positives > n {
                return Err("calibration_counts".into());
            }
            total += n;
            rates.push((positives as f64 + 1.) / (n as f64 + 2.));
        }
        if total != 8 {
            return Err("calibration_n".into());
        }
        let mut raw = vec![];
        let mut mapped = vec![];
        let mut missing = vec![];
        let mut flags_correct = 0;
        for r in &rs {
            let label = &b.labels[r["id"].as_str().unwrap()];
            let actual_field = if f == "same_context" {
                "applicability"
            } else {
                f
            };
            match assessment(r, label, model).and_then(|a| {
                Ok((
                    score(a, f)?,
                    a[actual_field].as_bool().ok_or("factor_flag")?,
                ))
            }) {
                Ok((p, flag)) => {
                    let y = label["factors"][f].as_bool().ok_or("gold")?;
                    flags_correct += usize::from(y == flag);
                    raw.push((p, y));
                    let mapped_score = rates[usize::from(p > 0.5)];
                    if split == "final" {
                        let assessment = assessment(r, label, model)?;
                        if assessment["calibration_status"] != "empirical_profile_bound"
                            || assessment["calibrated_factor_scores"][f]
                                .as_f64()
                                .is_none_or(|actual| (actual - mapped_score).abs() > 1e-12)
                        {
                            return Err(format!(
                                "final_calibration_publication_mismatch:{}:{f}",
                                r["id"]
                            ));
                        }
                    }
                    mapped.push((mapped_score, y));
                }
                Err(e) => missing.push(json!({"id":r["id"],"reason":e})),
            }
        }
        results.insert(f.into(),json!({"raw":metrics(&raw),"empirical_two_bins":metrics(&mapped),"correct_boolean_judgments":flags_correct,"expected_cases":rs.len(),"unscored":missing,"calibrated_values":rates}));
    }
    write_new(
        out,
        &json!({"schema_version":1,"split":split,"provider":provider,"requested_model":requested_model,"served_model":model,"bank_sha256":b.hashes,"records_sha256":hash(&bytes),"calibration_sha256":hash(&calbytes),"expected_cases":rs.len(),"selection_correct":correct,"degraded":degraded,"factors":results,"cases":per_case,"interpretation":"Descriptive synthetic sample only. Eight cases per split; sources disjoint but template families shared. Fixed two-bin Laplace rates have tiny support and do not certify facts or authorize general use. Missing scores and degraded operations are not successes."}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn brier_and_ece_use_probabilities_not_boolean_choices() {
        let m = metrics(&[(0.8, true), (0.2, false)]);
        assert!((m["brier"].as_f64().unwrap() - 0.04).abs() < 1e-12);
        assert!((m["ece_5_fixed_bins"].as_f64().unwrap() - 0.2).abs() < 1e-12);
        assert!(metrics(&[])["brier"].is_null());
    }
    #[test]
    fn missing_probability_does_not_become_boolean_confidence() {
        assert!(score(&json!({"requested_fact":true}), "requested_fact").is_err());
        assert!(
            score(
                &json!({"factor_scores":{"requested_fact":1.01}}),
                "requested_fact"
            )
            .is_err()
        );
    }
    #[test]
    fn fixture_source_partitions_and_hashes_are_valid() {
        let d = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/m4-factor-v1");
        bank(&d).unwrap();
    }
    fn synthetic_records(b: &Bank, split: &str) -> Vec<Value> {
        b.cases.iter().filter(|(_,c)|c["split"]==split).map(|(id,c)|{
            let g=&b.labels[id]; let m=&b.corpus[g["memory_id"].as_str().unwrap()];
            let mut scores=serde_json::Map::new();for f in FACTORS {scores.insert(f.into(),json!(if g["factors"][f]==true{0.8}else{0.2}));}
            json!({"id":id,"provider":"decisions","factor_profile_sha256":hash(b"synthetic test profile"),"response":{"requested_model":"openai/gpt-6-luna-decisions","execution_status":"completed","effective_request":c["request"],"implementation_sha256":"a".repeat(64),"coverage":{"classification":"complete"},"usage":{"served_model":"openai/gpt-6-luna-decisions-20261006"},"retrieval":{"candidates":[{"memory_id":m["id"],"revision":m["revision"],"summary":m["summary"]}]},"assessments":[{"memory_id":m["id"],"revision":m["revision"],"selected":g["expected_selected"],"applicability":g["factors"]["same_context"],"requested_fact":g["factors"]["requested_fact"],"premise_correction":g["factors"]["premise_correction"],"factor_scores":scores}]}})
        }).collect()
    }
    #[test]
    fn replay_cannot_relabel_another_question_source_or_implementation() {
        let d = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/m4-factor-v1");
        let b = bank(&d).unwrap();
        let rs = synthetic_records(&b, "calibration");
        roster(&b, &rs, "calibration", "decisions").unwrap();
        let mut changed = rs.clone();
        changed[0]["response"]["effective_request"]["search"]["query"] = json!("another question");
        assert!(roster(&b, &changed, "calibration", "decisions").is_err());
        let mut changed = rs.clone();
        changed[0]["response"]["retrieval"]["candidates"][0]["summary"] = json!("other source");
        assert!(roster(&b, &changed, "calibration", "decisions").is_err());
        let mut changed = rs.clone();
        changed[0]["response"]["implementation_sha256"] = json!("b".repeat(64));
        assert!(roster(&b, &changed, "calibration", "decisions").is_err());
        let mut changed = rs.clone();
        changed[0]["response"]["retrieval"]["candidates"] = Value::Null;
        assert!(roster(&b, &changed, "calibration", "decisions").is_err());
        let mut changed = rs.clone();
        changed[0]["response"]["effective_request"]["selection_policy"] = json!("baseline");
        assert!(roster(&b, &changed, "calibration", "decisions").is_err());
    }
    #[test]
    fn empirical_profile_is_rebuilt_and_existing_reports_are_preserved() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let d = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/m4-factor-v1");
        let b = bank(&d).unwrap();
        let temp = std::env::temp_dir().join(format!(
            "m4-calibration-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&temp).unwrap();
        let training = temp.join("training.jsonl");
        let reserve = temp.join("final.jsonl");
        let profile = temp.join("profile.rs");
        let artifact = temp.join("artifact.json");
        let output = temp.join("report.json");
        let serialized = |split: &str| {
            synthetic_records(&b, split)
                .iter()
                .map(|r| serde_json::to_string(r).unwrap() + "\n")
                .collect::<String>()
        };
        fs::write(&training, serialized("calibration")).unwrap();
        fs::write(&reserve, serialized("final")).unwrap();
        fs::write(&profile, "synthetic test profile").unwrap();
        calibrate(&d, &training, &profile, "decisions", &artifact).unwrap();
        let (_, cal) = read(&artifact).unwrap();
        let mut final_records = synthetic_records(&b, "final");
        for r in &mut final_records {
            r["response"]["effective_request"]["calibration"] = cal.clone();
            r["response"]["assessments"][0]["calibration_status"] =
                json!("empirical_profile_bound");
            let mut scores = serde_json::Map::new();
            for f in FACTORS {
                let p = r["response"]["assessments"][0]["factor_scores"][f]
                    .as_f64()
                    .unwrap();
                let bin = &cal["factors"][f][usize::from(p > 0.5)];
                scores.insert(
                    f.into(),
                    json!(
                        (bin["positives"].as_f64().unwrap() + 1.)
                            / (bin["n"].as_f64().unwrap() + 2.)
                    ),
                );
            }
            r["response"]["assessments"][0]["calibrated_factor_scores"] = json!(scores);
        }
        fs::write(
            &reserve,
            final_records
                .iter()
                .map(|r| serde_json::to_string(r).unwrap() + "\n")
                .collect::<String>(),
        )
        .unwrap();
        report(
            &d,
            &reserve,
            &artifact,
            &training,
            &profile,
            "decisions",
            "final",
            &output,
        )
        .unwrap();
        let valid_final = fs::read(&reserve).unwrap();
        let mut changed = final_records.clone();
        for r in &mut changed {
            r["response"]["implementation_sha256"] = json!("b".repeat(64));
        }
        fs::write(
            &reserve,
            changed
                .iter()
                .map(|r| serde_json::to_string(r).unwrap() + "\n")
                .collect::<String>(),
        )
        .unwrap();
        assert!(
            report(
                &d,
                &reserve,
                &artifact,
                &training,
                &profile,
                "decisions",
                "final",
                &temp.join("wrong-implementation.json")
            )
            .unwrap_err()
            .contains("implementation_mismatch")
        );
        let mut changed = final_records.clone();
        changed[0]["response"]["assessments"][0]["calibrated_factor_scores"]["same_context"] =
            json!(0.123);
        fs::write(
            &reserve,
            changed
                .iter()
                .map(|r| serde_json::to_string(r).unwrap() + "\n")
                .collect::<String>(),
        )
        .unwrap();
        assert!(
            report(
                &d,
                &reserve,
                &artifact,
                &training,
                &profile,
                "decisions",
                "final",
                &temp.join("wrong-calibrated-score.json")
            )
            .unwrap_err()
            .contains("publication_mismatch")
        );
        fs::write(&reserve, valid_final).unwrap();
        let original = fs::read(&output).unwrap();
        assert!(
            report(
                &d,
                &reserve,
                &artifact,
                &training,
                &profile,
                "decisions",
                "final",
                &output
            )
            .is_err()
        );
        assert_eq!(fs::read(&output).unwrap(), original);
        let (_, mut tampered) = read(&artifact).unwrap();
        tampered["factors"]["requested_fact"][0]["positives"] = json!(1);
        fs::write(&artifact, serde_json::to_vec(&tampered).unwrap()).unwrap();
        assert!(
            report(
                &d,
                &reserve,
                &artifact,
                &training,
                &profile,
                "decisions",
                "final",
                &temp.join("invalid.json")
            )
            .is_err()
        );
        assert!(!temp.join("invalid.json").exists());
        fs::remove_dir_all(temp).unwrap();
    }
}
