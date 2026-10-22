//! Language ablation only. Reviewed equivalence is not behavioral correctness.
use serde_json::Value;
fn glossary() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/local-role-instructions-es-v1.json"
    ))
    .expect("reviewed glossary must be valid JSON")
}
fn render_template(template: &str, source: &str) -> String {
    let marker = "\"{{SOURCE_TEXT}}\"";
    assert_eq!(
        template.matches(marker).count(),
        1,
        "quoted source marker changed"
    );
    template.replacen(
        marker,
        &serde_json::to_string(source).expect("literal source encoding"),
        1,
    )
}
pub fn payload(query: &str, memories: &[Value]) -> Value {
    let table = glossary();
    let en = &table["english"];
    let es = &table["spanish"];
    let mut p = super::payload_negative_reasons(query, memories);
    assert_eq!(
        p["state"]["rules"], en["rules"],
        "English source rules changed"
    );
    p["state"]["rules"] = es["rules"].clone();
    if !memories.is_empty() {
        assert_eq!(
            p["state"]["role_definitions"], en["definitions"],
            "English source definitions changed"
        );
        p["state"]["role_definitions"] = es["definitions"].clone();
    }
    for (i, m) in memories.iter().enumerate() {
        let q = &mut p["questions"][format!("memory_{i:02}_relation")];
        let source = m["text"].as_str().expect("source text");
        assert_eq!(
            q["instructions"],
            render_template(en["template"].as_str().unwrap(), source),
            "English source template changed"
        );
        assert_eq!(
            q["criteria"], en["criteria"],
            "English source criteria changed"
        );
        q["instructions"] =
            Value::String(render_template(es["template"].as_str().unwrap(), source));
        q["criteria"] = es["criteria"].clone();
    }
    p
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn memories() -> Vec<Value> {
        vec![
            json!({"id":"private-id","revision":3,"text":"Un texto con comillas \"dato\" y {{SOURCE_TEXT}}."}),
            json!({"id":"second-id","revision":1,"text":"Otra fuente literal."}),
        ]
    }
    #[test]
    fn restoring_only_translated_fields_recovers_exact_english_payload() {
        let m = memories();
        let en = super::super::payload_negative_reasons("Pregunta literal", &m);
        let mut es = payload("Pregunta literal", &m);
        assert_ne!(es["state"]["rules"], en["state"]["rules"]);
        assert_eq!(
            es["state"]["contribution_examples"],
            en["state"]["contribution_examples"]
        );
        es["state"]["rules"] = en["state"]["rules"].clone();
        es["state"]["role_definitions"] = en["state"]["role_definitions"].clone();
        for (key, q) in es["questions"].as_object_mut().unwrap() {
            assert_eq!(
                q["criteria"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .collect::<Vec<_>>(),
                en["questions"][key]["criteria"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .collect::<Vec<_>>()
            );
            q["instructions"] = en["questions"][key]["instructions"].clone();
            q["criteria"] = en["questions"][key]["criteria"].clone();
        }
        assert_eq!(es, en);
    }
    #[test]
    fn source_quotes_markers_model_and_canonical_namespace_survive_translation() {
        let m = memories();
        let p = payload("Pregunta literal", &m);
        let table = glossary();
        assert_eq!(p["model"], super::super::Family::Perplexity.model());
        for (i, m) in m.iter().enumerate() {
            assert_eq!(
                p["questions"][format!("memory_{i:02}_relation")]["instructions"],
                render_template(
                    table["spanish"]["template"].as_str().unwrap(),
                    m["text"].as_str().unwrap()
                )
            );
        }
        let mut answers = serde_json::Map::new();
        for i in 0..m.len() {
            let probs = p["questions"][format!("memory_{i:02}_relation")]["criteria"]
                .as_object()
                .unwrap()
                .keys()
                .map(|k| (k.clone(), json!(if k == "N_subject" { 1.0 } else { 0.0 })))
                .collect::<serde_json::Map<_, _>>();
            answers.insert(
                format!("memory_{i:02}_relation"),
                json!({"type":"choice","choice":"N_subject","probabilities":probs}),
            );
        }
        let body = json!({"model":super::super::Family::Perplexity.revision(),"answers":answers});
        assert_eq!(
            super::super::decode_family(&body, &p, &m, super::super::Family::PerplexitySpanish)
                .unwrap(),
            vec!["none"; m.len()]
        );
    }
}
