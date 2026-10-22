//! Evaluation prototype: preserve catalog references by substitution, never infer coreference.
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: String,
    reference: Value,
}
fn expand(
    v: &Value,
    catalog: &BTreeMap<String, Value>,
    used: &mut BTreeSet<String>,
    depth: usize,
) -> Result<Value, String> {
    if depth > 64 {
        return Err("estructura demasiado profunda".into());
    }
    match v {
        Value::Object(m) => {
            if m.contains_key("reference_id") {
                return Err("identificador fuera de un slot de referencia".into());
            }
            let mut result = m.clone();
            for (k, x) in m {
                result[k] = if ["subject", "reference"].contains(&k.as_str()) {
                    let marker = x
                        .as_object()
                        .filter(|m| m.len() == 1)
                        .ok_or("slot con referencia copiada o campos añadidos")?;
                    let id = marker
                        .get("reference_id")
                        .and_then(Value::as_str)
                        .ok_or("slot sin identificador")?;
                    let reference = catalog
                        .get(id)
                        .ok_or("referencia no presentada en catálogo")?;
                    used.insert(id.to_owned());
                    reference.clone()
                } else {
                    expand(x, catalog, used, depth + 1)?
                };
            }
            Ok(Value::Object(result))
        }
        Value::Array(a) => a
            .iter()
            .map(|v| expand(v, catalog, used, depth + 1))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        _ => Ok(v.clone()),
    }
}
pub fn materialize(catalog: &Value, stage2: &Value) -> Result<Value, String> {
    let entries: Vec<Entry> =
        serde_json::from_value(catalog.clone()).map_err(|e| format!("catálogo inválido: {e}"))?;
    if entries.is_empty() || entries.len() > 64 {
        return Err("catálogo vacío o demasiado extenso".into());
    }
    let mut map = BTreeMap::new();
    for e in entries {
        if e.id.is_empty()
            || e.id.len() > 48
            || !e.id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || !e.reference.is_object()
            || map.insert(e.id, e.reference).is_some()
        {
            return Err("ID duplicado/inválido o referencia no es objeto".into());
        }
    }
    let mut used = BTreeSet::new();
    let raw = expand(stage2, &map, &mut used, 0)?;
    // Unused entries remain observable; lack of use does not prove an omitted clause.
    Ok(
        serde_json::json!({"raw":raw,"used_ids":used,"unused_ids":map.keys().filter(|id|!used.contains(*id)).collect::<Vec<_>>(),"preserved_by_substitution":true,"semantic_fidelity_certified":false}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn all_reference_dimensions_survive_and_input_is_not_mutated() {
        let reference = json!({"mention_quote":"caja K de X","type_quote":"caja","name":"K","owners":[{"mention_quote":"X","type_quote":null,"name":"X"}],"scope":"fixed_group","count":3,"part_of":{"name":"Y"},"member_of":null});
        let catalog = json!([{"id":"r","reference":reference}]);
        let before = catalog.clone();
        let stage = json!({"question":{"subject":{"reference_id":"r"},"assumptions":[{"subject":{"reference_id":"r"}}],"requested":[]}});
        let a = materialize(&catalog, &stage).unwrap();
        assert_eq!(a["raw"]["question"]["subject"], reference);
        assert_eq!(a["raw"]["question"]["assumptions"][0]["subject"], reference);
        assert_eq!(catalog, before);
        assert_eq!(a["semantic_fidelity_certified"], false);
    }
    #[test]
    fn unknown_new_inline_modified_or_foreign_markers_are_rejected() {
        let c = json!([{"id":"r","reference":{"name":"X"}}]);
        for bad in [
            json!({"subject":{"reference_id":"other"}}),
            json!({"subject":{"name":"Y"}}),
            json!({"subject":{"reference_id":"r","name":"Y"}}),
            json!({"note":{"reference_id":"r"}}),
        ] {
            assert!(materialize(&c, &bad).is_err());
        }
        assert!(
            materialize(
                &json!([{"id":"r","reference":{}},{"id":"r","reference":{}}]),
                &json!({})
            )
            .is_err()
        );
    }
    #[test]
    fn nested_entity_values_and_all_readings_use_exact_references() {
        let c = json!([{"id":"r","reference":{"name":"R4"}}]);
        let x = json!({"readings":[{"clauses":[{"statement":{"values":[{"type":"entity","reference":{"reference_id":"r"}}]}}]}]});
        let a = materialize(&c, &x).unwrap();
        assert_eq!(
            a["raw"]["readings"][0]["clauses"][0]["statement"]["values"][0]["reference"]["name"],
            "R4"
        );
    }
    #[test]
    fn unused_references_visible_and_same_local_id_is_not_cross_text_identity() {
        let c =
            json!([{"id":"r","reference":{"name":"X"}},{"id":"extra","reference":{"name":"Z"}}]);
        let d = json!([{"id":"r","reference":{"name":"Y"}}]);
        let x = json!({"subject":{"reference_id":"r"}});
        let a = materialize(&c, &x).unwrap();
        assert_eq!(a["unused_ids"], json!(["extra"]));
        assert_ne!(a["raw"], materialize(&d, &x).unwrap()["raw"]);
    }
    #[test]
    fn wrong_reference_choice_and_wrong_predicate_are_not_certified_by_conservation() {
        let c = json!([{"id":"x","reference":{"name":"X"}},{"id":"y","reference":{"name":"Y"}}]);
        let a = materialize(
            &c,
            &json!({"subject":{"reference_id":"y"},"polarity":"wrong"}),
        )
        .unwrap();
        assert_eq!(a["raw"]["subject"]["name"], "Y");
        assert_eq!(a["raw"]["polarity"], "wrong");
        assert_eq!(a["semantic_fidelity_certified"], false);
    }
}
