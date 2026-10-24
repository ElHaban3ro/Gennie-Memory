//! Baseline presents saved facts as evidence as well as answer choices.
use super::*;

pub(super) fn payload(
    query: &str,
    scope: &str,
    options: serde_json::Map<String, Value>,
    provider: ClassifierProvider,
) -> Value {
    let memories: serde_json::Map<String, Value> = options
        .iter()
        .filter(|(label, _)| label.as_str() != "none")
        .map(|(label, fact)| (label.clone(), fact.clone()))
        .collect();
    let mut body = choice_payload(query, scope, options, INSTRUCTIONS, provider);
    body["state"]["memories"] = json!(memories);
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_contains_saved_facts_not_the_abstention_option() {
        let options=serde_json::from_value(json!({"A":"Aliso usa una tabla de tarea, responsable y fecha.","B":"Aliso conserva informes seis meses.","none":"No listed memory contains the requested fact."})).unwrap();
        let query = r#"{"query":"¿Qué formato usa Aliso?","context":[{"text":"Si no existe un formato registrado, no inventes uno."}]}"#;
        let body = payload(query, "aliso", options, ClassifierProvider::Decisions);
        assert_eq!(
            body["state"]["memories"],
            json!({"A":"Aliso usa una tabla de tarea, responsable y fecha.","B":"Aliso conserva informes seis meses."})
        );
        assert!(body["state"]["memories"].get("none").is_none());
        assert_eq!(body["state"]["user_prompt"], query);
        assert_eq!(body["questions"]["relevance"]["instructions"], INSTRUCTIONS);
    }
    #[test]
    fn only_evidence_is_added_to_legacy_payload_for_each_typed_model() {
        for provider in [
            ClassifierProvider::Decisions,
            ClassifierProvider::Jev,
            ClassifierProvider::Jev114,
        ] {
            let options:serde_json::Map<String,Value>=serde_json::from_value(json!({"A":"No enviar sin permiso.","none":"No listed memory contains the requested fact."})).unwrap();
            let legacy = choice_payload(
                "¿Qué se permite?",
                "scope",
                options.clone(),
                INSTRUCTIONS,
                provider,
            );
            let mut fixed = payload("¿Qué se permite?", "scope", options, provider);
            fixed["state"].as_object_mut().unwrap().remove("memories");
            assert_eq!(fixed, legacy);
        }
    }
}
