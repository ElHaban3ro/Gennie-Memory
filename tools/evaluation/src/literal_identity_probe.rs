//! Posthoc identity-anchor screen, not fresh model quality or an alias resolver.
use crate::isolated_provider_probe::{lines, read};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::Path,
};
fn words(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}
fn missing(query: &str, memory: &str, identity: &Value) -> Result<Vec<String>, String> {
    let first = query.split_whitespace().next().unwrap_or("");
    let have = words(memory);
    let mut anchors = BTreeSet::new();
    for token in identity.as_array().ok_or("identidad ausente")? {
        let token = token.as_str().ok_or("token no textual")?;
        if token == first {
            continue;
        }
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.chars().next().is_some_and(char::is_uppercase) {
            anchors.extend(words(clean));
        }
    }
    Ok(anchors.into_iter().filter(|a| !have.contains(a)).collect())
}
pub fn report(dir: &Path, output: &Path) -> Result<(), String> {
    let source = read(&dir.join("report.json"))?;
    let revalidation = dir.join(format!("role-revalidation-{}.json", std::process::id()));
    crate::independent_role_audit::report(dir, &revalidation)?;
    if read(&revalidation)? != source {
        return Err("informe de origen no reproducible".into());
    }
    std::fs::remove_file(revalidation).map_err(|e| e.to_string())?;
    let requests = lines(&dir.join("requests.jsonl"))?;
    if source["new_operations"] != 128
        || source["complete"] != true
        || source["journal_sha256"] != crate::campaign::digest(&dir.join("journal.jsonl"))?
    {
        return Err("fuente incompleta o alterada".into());
    }
    let pre = read(&dir.join("preflight.json"))?;
    if pre["requests_sha256"] != crate::campaign::digest(&dir.join("requests.jsonl"))? {
        return Err("payloads alterados".into());
    }
    let lookup = requests
        .iter()
        .map(|r| (r["id"].as_str().unwrap(), r))
        .collect::<BTreeMap<_, _>>();
    let mut rows = Vec::new();
    let mut counts = BTreeMap::<String, usize>::new();
    let mut lost = Vec::new();
    let mut removed = Vec::new();
    let mut controls = 0;
    for observation in source["observations"]
        .as_array()
        .ok_or("observaciones ausentes")?
    {
        let id = observation["id"].as_str().ok_or("ID ausente")?;
        let r = lookup.get(id).ok_or("par sin payload")?;
        let input: Value =
            serde_json::from_str(r["payload"]["messages"][1]["content"].as_str().unwrap())
                .map_err(|e| e.to_string())?;
        let absent = missing(
            input["query"].as_str().unwrap(),
            input["saved_memory"].as_str().unwrap(),
            &input["query_projection"]["identity_tokens"],
        )?;
        let old = observation["selected"]
            .as_bool()
            .ok_or("elección inválida")?;
        let selected = old && absent.is_empty();
        let expected = observation["expected_useful"].as_bool().unwrap();
        let class = if selected {
            if expected {
                "true_positive"
            } else {
                "false_positive"
            }
        } else if expected {
            "false_negative"
        } else {
            "true_negative"
        };
        if observation["is_control"] == true {
            if selected == expected {
                controls += 1;
            }
        } else {
            *counts.entry(class.into()).or_default() += 1;
            if old && !selected {
                if expected {
                    lost.push(id);
                } else {
                    removed.push(id);
                }
            }
        }
        rows.push(json!({"id":id,"is_control":observation["is_control"],"expected_useful":expected,"original_selected":old,"recomposed_selected":selected,"missing_literal_named_identity":absent,"classification":class}));
    }
    let result = json!({"version":"literal-identity-posthoc-probe-v1","posthoc":true,"new_provider_calls":0,"source_report_sha256":crate::campaign::digest(&dir.join("report.json"))?,"requests_sha256":pre["requests_sha256"],"candidate_counts":counts,"controls_correct":controls,"previously_accepted_useful_lost":lost,"previous_wrong_accepts_removed":removed,"observations":rows,"original_gold_changed":false,"original_results_changed":false,"promoted":false,"warning":"Exploratory recomposition of existing outputs. Only capitalized tokens in the fixed query identity are tested, excluding the sentence-initial token. Literal presence does not prove identity; aliases/coreference or misprojected names may cause losses. Fresh full tests required; not a general semantic verifier or adoption gate."});
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .map_err(|e| e.to_string())?
        .write_all(&serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_qualifiers_block_other_person_but_do_not_require_lowercase_descriptors() {
        let id = json!(["Dalia", "censo", "Ribera", "octubre?"]);
        assert_eq!(
            missing(
                "¿Hora de Dalia en Ribera?",
                "Bruno observa Ribera en octubre.",
                &id
            )
            .unwrap(),
            vec!["dalia"]
        );
        assert!(
            missing("¿Hora de Dalia en Ribera?", "Dalia observa Ribera.", &id)
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn first_word_capitalization_and_unicode_punctuation_are_safe() {
        assert!(
            missing(
                "¿Qué Índigo I4?",
                "Índigo I4.",
                &json!(["¿Qué", "Índigo", "I4?"])
            )
            .unwrap()
            .is_empty()
        );
        assert_eq!(
            missing("¿Qué Orbe?", "Vega pesa ocho.", &json!(["Orbe?"])).unwrap(),
            vec!["orbe"]
        );
    }
}
