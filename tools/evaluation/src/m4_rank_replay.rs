//! Component interventions on COMPLETE frozen rank pools, not a live encoder run.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::Path,
};
fn sha(b: &[u8]) -> String {
    Sha256::digest(b)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn rank(c: &Value, key: &str) -> Option<u64> {
    c[key].as_u64()
}
fn rrf(c: &Value, threshold: Option<f64>) -> f64 {
    let text = rank(c, "text_rank").map_or(0., |r| 1. / (60. + r as f64));
    let semantic = rank(c, "semantic_rank")
        .filter(|_| threshold.is_none_or(|t| c["cosine"].as_f64().is_some_and(|v| v >= t)))
        .map_or(0., |r| 1. / (60. + r as f64));
    text + semantic
}
fn pick<'a>(cs: &'a [Value], arm: &str) -> Option<&'a str> {
    if arm == "no_retrieval" {
        return None;
    }
    let mut candidates: Vec<_> = cs
        .iter()
        .filter(|c| match arm {
            "without_embeddings" => rank(c, "text_rank").is_some(),
            "semantic_only" => rank(c, "semantic_rank").is_some(),
            _ => true,
        })
        .collect();
    let threshold = match arm {
        "cosine_floor_0_7" => Some(0.7),
        "cosine_floor_0_8" => Some(0.8),
        "cosine_floor_0_9" => Some(0.9),
        _ => None,
    };
    if threshold.is_some() {
        candidates.retain(|c| rrf(c, threshold) > 0.);
    }
    candidates.sort_by(|a, b| {
        let score = |c: &Value| match arm {
            "without_embeddings" => 1. / rank(c, "text_rank").unwrap() as f64,
            "semantic_only" => 1. / rank(c, "semantic_rank").unwrap() as f64,
            "without_rrf_best_rank" => {
                1. / rank(c, "text_rank")
                    .unwrap_or(u64::MAX)
                    .min(rank(c, "semantic_rank").unwrap_or(u64::MAX)) as f64
            }
            _ => rrf(c, threshold),
        };
        score(b)
            .total_cmp(&score(a))
            .then_with(|| {
                rank(a, "text_rank")
                    .unwrap_or(u64::MAX)
                    .cmp(&rank(b, "text_rank").unwrap_or(u64::MAX))
            })
            .then_with(|| {
                rank(a, "semantic_rank")
                    .unwrap_or(u64::MAX)
                    .cmp(&rank(b, "semantic_rank").unwrap_or(u64::MAX))
            })
            .then_with(|| a["memory_id"].as_str().cmp(&b["memory_id"].as_str()))
    });
    candidates.first().and_then(|c| c["memory_id"].as_str())
}
pub fn run(input: &Path, out: &Path) -> Result<(), String> {
    let bytes = fs::read(input).map_err(|e| e.to_string())?;
    let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
    let mut rows = vec![];
    let mut ids = BTreeSet::new();
    let mut splits = BTreeSet::new();
    for line in text.lines() {
        let row: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
        if !ids.insert(row["id"].as_str().ok_or("row_id")?.to_owned()) {
            return Err("duplicate_id".into());
        }
        let split = row["split"].as_str().ok_or("split")?;
        if !["development", "calibration"].contains(&split) {
            return Err("only_observed_development_or_calibration".into());
        }
        splits.insert(split.to_owned());
        if row["label_source"] != "deterministic_fixture_rule" {
            return Err("unexpected_label_provenance".into());
        }
        let r = &row["response"];
        let n = r["eligible_memories"].as_u64().ok_or("eligible")?;
        let cs = r["candidates"].as_array().ok_or("candidates")?;
        if r["status"] != "complete"
            || r["semantic_coverage"] != "complete"
            || r["indexed_memories"] != n
            || n > 20
            || cs.len() != n as usize
        {
            return Err("incomplete_rank_pool".into());
        }
        let mut mids = BTreeSet::new();
        let mut semantic = BTreeSet::new();
        let mut lexical = BTreeSet::new();
        for c in cs {
            if !mids.insert(c["memory_id"].as_str().ok_or("memory_id")?) {
                return Err("duplicate_candidate".into());
            }
            let rank = rank(c, "semantic_rank").ok_or("missing_semantic_rank")?;
            if !(1..=n).contains(&rank) || !semantic.insert(rank) {
                return Err("semantic_rank_roster".into());
            }
            let cosine = c["cosine"].as_f64().ok_or("cosine")?;
            if !cosine.is_finite() || !(-1.0..=1.0).contains(&cosine) {
                return Err("cosine_invalid".into());
            }
            if let Some(r) = self::rank(c, "text_rank") {
                if r == 0 || r > 20 || !lexical.insert(r) {
                    return Err("text_rank_roster".into());
                }
            }
            if (c["rrf"].as_f64().ok_or("rrf")? - rrf(c, None)).abs() > 1e-12 {
                return Err("rrf_reconstruction".into());
            }
        }
        if cs.first().and_then(|c| c["memory_id"].as_str()) != pick(cs, "frozen_rrf") {
            return Err("original_order_not_reconstructed".into());
        }
        if row["expected_ids"].as_array().is_none() || row["forbidden_ids"].as_array().is_none() {
            return Err("labels".into());
        }
        rows.push(row);
    }
    if rows.is_empty() || splits.len() != 1 {
        return Err("empty_or_mixed_split".into());
    }
    let arms = [
        "no_retrieval",
        "without_embeddings",
        "semantic_only",
        "without_rrf_best_rank",
        "frozen_rrf",
        "cosine_floor_0_7",
        "cosine_floor_0_8",
        "cosine_floor_0_9",
    ];
    let mut metrics = serde_json::Map::new();
    for arm in arms {
        let mut correct = 0;
        let mut positive = 0;
        let mut negatives = 0;
        let mut useful = 0;
        let mut abstained = 0;
        let mut forbidden = 0;
        let mut changes = vec![];
        let mut segments: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        for row in &rows {
            let cs = row["response"]["candidates"].as_array().unwrap();
            let chosen = pick(cs, arm);
            let base = pick(cs, "frozen_rrf");
            let expected = row["expected_ids"].as_array().unwrap();
            let ok = match chosen {
                Some(id) => expected.iter().any(|v| v == id),
                None => expected.is_empty(),
            };
            correct += usize::from(ok);
            if expected.is_empty() {
                negatives += 1;
                abstained += usize::from(chosen.is_none());
            } else {
                positive += 1;
                useful += usize::from(ok);
            }
            forbidden += usize::from(chosen.is_some_and(|id| {
                row["forbidden_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v == id)
            }));
            if chosen != base {
                changes.push(json!({"id":row["id"],"frozen_rrf":base,"selected":chosen,"legacy_id_correct":ok}));
            }
            let x = segments
                .entry(row["segment"].as_str().ok_or("segment")?.to_owned())
                .or_default();
            x.0 += 1;
            x.1 += usize::from(ok);
        }
        metrics.insert(arm.into(),json!({"cases":rows.len(),"legacy_id_correct":correct,"positive_cases":positive,"positive_id_correct":useful,"negative_cases":negatives,"negative_abstained":abstained,"forbidden_id_selections":forbidden,"changed_cases":changes,"segments":segments.into_iter().map(|(s,(n,k))|(s,json!({"cases":n,"legacy_id_correct":k}))).collect::<BTreeMap<_,_>>() }));
    }
    let report = json!({"schema_version":1,"source":input.to_string_lossy(),"source_sha256":sha(&bytes),"cases":rows.len(),"split":splits.iter().next(),"arms":metrics,"interpretation":"Rank-pool interventions on frozen historical Laya vectors. No active encoder, remote classifier, seed composition, timing or human semantic validation. Complete <=20-memory pools only. without_rrf_best_rank replaces reciprocal-rank sum with best individual rank on the same union; fixed cosine floors remove only semantic contributions and retain lexical matches. No structured-filter ablation can be inferred from filtered pools. Historical ID labels are not factual answer truth. Parameters fixed before reading results; do not choose a production threshold from these observed development/calibration data."});
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(out)
        .map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn threshold_removes_semantics_but_keeps_lexical() {
        let cs = vec![
            json!({"memory_id":"a","text_rank":2,"semantic_rank":1,"cosine":0.3}),
            json!({"memory_id":"b","text_rank":1,"semantic_rank":2,"cosine":0.95}),
        ];
        assert_eq!(pick(&cs, "cosine_floor_0_9"), Some("b"));
        assert_eq!(pick(&cs, "no_retrieval"), None);
        assert_eq!(pick(&cs, "semantic_only"), Some("a"));
        assert_eq!(pick(&cs, "without_embeddings"), Some("b"));
    }
}
