//! Posthoc scalar veto feasibility; never qualifies an old failed campaign.
use crate::{
    campaign::digest,
    isolated_provider_probe::{ids, lines, read},
    scoped_definition_probe::{selected_model, write_new},
    scoped_family_probe, score_diagnostic,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
fn fit(rows: &[(f64, bool)]) -> Value {
    if rows.is_empty() {
        return json!({"available":false,"reason":"no accepted scores"});
    }
    if !rows.iter().any(|(_, expected)| !*expected) {
        return json!({"available":true,"no_observed_accepted_false_positives":true,"threshold_needed":false,"threshold":null,"action":"keep current decisions","accepted_useful":rows.len()});
    }
    let mut result = score_diagnostic::fit(rows, true);
    result["available"] = json!(true);
    result["threshold_needed"] = json!(true);
    result
}
fn compose(
    a: &BTreeSet<String>,
    b: &BTreeSet<String>,
    j: &BTreeSet<String>,
    m: &BTreeSet<String>,
) -> BTreeSet<String> {
    a.union(
        &b.intersection(j)
            .filter(|id| m.contains(*id))
            .cloned()
            .collect(),
    )
    .cloned()
    .collect()
}
fn cascade_hypothesis(models: &[Value], inputs: &[Value], parent: &Value) -> Result<Value, String> {
    let get = |id: &str| {
        models
            .iter()
            .find(|m| m["model"] == id)
            .ok_or("falta modelo completo para hipótesis")
    };
    let j = get("typesafe/jev-1.13")?;
    let m = get("microsoft/microsoft-decision-1")?;
    let mercury = get("inception/mercury-decide")?;
    let collect = |model: &Value, query: &Value, role: Option<&str>| -> BTreeSet<String> {
        model["observations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["query"] == *query && role.is_none_or(|p| r["role"] == p))
            .map(|r| r["memory_id"].as_str().unwrap().to_owned())
            .collect()
    };
    let mut rows = Vec::new();
    let mut wrong = 0;
    let mut lost = 0;
    let mut retained = 0;
    for input in inputs {
        let a = collect(mercury, &input["id"], Some("A"));
        let b = collect(mercury, &input["id"], Some("B"));
        let selected = compose(
            &a,
            &b,
            &collect(j, &input["id"], None),
            &collect(m, &input["id"], None),
        );
        let expected = ids(&input["case"]["expected_ids"])?;
        let history = ids(&input["historical_selected_ids"])?;
        let bad = selected.difference(&expected).cloned().collect::<Vec<_>>();
        let missing = history
            .intersection(&expected)
            .filter(|id| !selected.contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        let omitted = input["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["memory_id"].as_str().unwrap())
            .filter(|id| expected.contains(*id) && !selected.contains(*id))
            .collect::<Vec<_>>();
        wrong += bad.len();
        lost += missing.len();
        retained += selected.intersection(&expected).count();
        rows.push(json!({"id":input["id"],"selected_ids":selected,"unpermitted_ids":bad,"lost_historical_useful_ids":missing,"omitted_gold_in_prefix":omitted}));
    }
    let f = parent["families"].as_array().unwrap();
    let used = f[1]["control_cost_usd"].as_f64().unwrap()
        + f[1]["candidate_cost_usd"].as_f64().unwrap()
        + f[2]["candidate_cost_usd"].as_f64().unwrap();
    Ok(
        json!({"posthoc":true,"new_provider_calls":0,"new_cost_usd":0.0,"rule":"Accept Mercury A; Mercury B requires useful acceptance by both Jev and Microsoft. Do not inspect gold during selection.","retained_allowed_judgments":retained,"unpermitted_ids":wrong,"lost_historical_useful_ids":lost,"cost_of_already_paid_source_streams_usd":used,"cost_of_sparse_runtime_not_measured":true,"latency_of_sparse_runtime_not_measured":true,"source_contexts":"All original seven candidates; fresh sparse critic contexts have not been tested.","fresh_validation":false,"runtime_implemented":false,"promoted":false,"cases":rows}),
    )
}
pub fn report(d: &Path, output: &Path) -> Result<(), String> {
    let checked = d.join(format!(
        "calibration-revalidation-{}.json",
        std::process::id()
    ));
    scoped_family_probe::report(d, &checked)?;
    let parent = read(&d.join("report-v2.json"))?;
    if read(&checked)? != parent {
        return Err("padre no reproducible".into());
    }
    fs::remove_file(&checked).map_err(|e| e.to_string())?;
    let requests = lines(&d.join("requests.jsonl"))?;
    let inputs = lines(&d.join("inputs.jsonl"))?;
    let events = lines(&d.join("journal.jsonl"))?;
    let ended = events
        .iter()
        .filter(|e| e["kind"] == "response")
        .map(|e| (e["id"].as_str().unwrap(), e))
        .collect::<BTreeMap<_, _>>();
    let families = parent["families"].as_array().ok_or("sin familias")?;
    let mut groups = Vec::new();
    if let Some(f) = families
        .iter()
        .find(|f| f["valid_control_operations"] == 18)
    {
        groups.push((f["family"].as_u64().unwrap(), "control"));
    }
    for f in families {
        if f["valid_candidate_operations"] == 18 {
            groups.push((f["family"].as_u64().unwrap(), "candidate"));
        }
    }
    let mut results = Vec::new();
    for (family, variant) in groups {
        let mut win: [Vec<(f64, bool)>; 3] = [vec![], vec![], vec![]];
        let mut margin: [Vec<(f64, bool)>; 3] = [vec![], vec![], vec![]];
        let mut observations = Vec::new();
        let mut omitted = Vec::new();
        let mut model = Value::Null;
        for input in &inputs {
            let id = format!(
                "{}__family_{family:02}_{variant}",
                input["id"].as_str().ok_or("sin ID")?
            );
            let r = requests
                .iter()
                .find(|r| r["id"] == id)
                .ok_or("petición ausente")?;
            let e = ended.get(id.as_str()).ok_or("respuesta ausente")?;
            model = r["payload"]["model"].clone();
            let expected = ids(&input["case"]["expected_ids"])?;
            let (selected, _) = selected_model(
                &e["body"],
                &r["payload"],
                input["candidates"].as_array().unwrap(),
                r["expected_served_model"].as_str().unwrap(),
            )?;
            for (n, c) in input["candidates"].as_array().unwrap().iter().enumerate() {
                let memory = c["memory_id"].as_str().unwrap();
                let useful = expected.contains(memory);
                if !selected.contains(memory) {
                    if useful {
                        omitted.push(json!({"query":input["id"],"memory_id":memory}));
                    }
                    continue;
                }
                let answer = &e["body"]["answers"][format!("memory_{n:02}_relation")];
                let role = answer["choice"].as_str().ok_or("sin rol")?;
                let slot = match role {
                    "A" => 1,
                    "B" => 2,
                    _ => return Err("positivo sin rol".into()),
                };
                let score = answer["probabilities"][role]
                    .as_f64()
                    .ok_or("sin probabilidad")?;
                let alternative = answer["probabilities"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter(|(key, _)| *key != role)
                    .map(|(_, value)| value.as_f64().unwrap())
                    .max_by(f64::total_cmp)
                    .ok_or("sin alternativa")?;
                let gap = score - alternative;
                for k in [0, slot] {
                    win[k].push((score, useful));
                    margin[k].push((gap, useful));
                }
                observations.push(json!({"query":input["id"],"memory_id":memory,"role":role,"allowed_by_original_gold":useful,"winning_probability":score,"margin_over_largest_alternative":gap}));
            }
        }
        results.push(json!({"model":model,"family":family,"source_variant":variant,"questions":18,"currently_accepted_judgments":observations.len(),"preexisting_omitted_gold_in_prefix":omitted,"features":{"winning_probability":{"global":fit(&win[0]),"role_A":fit(&win[1]),"role_B":fit(&win[2])},"margin_over_largest_alternative":{"global":fit(&margin[0]),"role_A":fit(&margin[1]),"role_B":fit(&margin[2])}},"observations":observations}));
    }
    let cascade = cascade_hypothesis(&results, &inputs, &parent)?;
    write_new(
        output,
        &json!({"version":"family-scalar-veto-feasibility-v1","posthoc":true,"new_provider_calls":0,"new_cost_usd":0.0,"fresh_validation":false,"promoted":false,"does_not_requalify_failed_campaign":true,"parent_report_sha256":digest(&d.join("report-v2.json"))?,"journal_sha256":digest(&d.join("journal.jsonl"))?,"interpretation":"Fits only a veto on currently accepted judgments. Omissions stay omitted. Separation on known labels is not a new performance result; earlier quality and latency failures remain.","models":results,"sparse_cascade_hypothesis":cascade}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_is_kept_and_b_needs_both_critics() {
        let set = |v: &[&str]| v.iter().map(|s| s.to_string()).collect();
        let result = compose(
            &set(&["a"]),
            &set(&["both", "only_j", "only_m", "neither"]),
            &set(&["both", "only_j"]),
            &set(&["both", "only_m"]),
        );
        assert_eq!(result, set(&["a", "both"]));
    }
}
