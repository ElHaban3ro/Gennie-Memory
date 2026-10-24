//! Advisory lexical residuals, never a semantic validator, identity key or repair.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::Write, path::Path};

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
struct Leaf {
    path: String,
    literal: String,
    containers: Vec<Option<String>>,
}
fn occurrences(haystack: &str, needle: &str) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    haystack
        .char_indices()
        .filter_map(|(i, _)| haystack[i..].starts_with(needle).then_some(i))
        .collect()
}
fn atoms(
    v: &Value,
    path: &str,
    out: &mut Vec<Leaf>,
    outer: &[Option<String>],
    depth: usize,
) -> Result<(), String> {
    if depth > 64 {
        return Err("referencia demasiado profunda para auditar".into());
    }
    if v.get("owners").is_some_and(|x| !x.is_array()) {
        return Err(format!("propietarios {path} no son una lista"));
    }
    for key in ["member_of", "part_of"] {
        if v.get(key).is_some_and(|x| !x.is_null() && !x.is_object()) {
            return Err(format!("enlace {path}/{key} no es objeto ni ausencia"));
        }
    }
    if v["mention_quote"].as_str().is_none_or(str::is_empty) {
        return Err(format!("mención ausente en {path}"));
    }
    let mut containers = outer.to_vec();
    containers.push(v["mention_quote"].as_str().map(str::to_owned));
    for key in ["type_quote", "name"] {
        if let Some(s) = v[key].as_str() {
            out.push(Leaf {
                path: format!("{path}/{key}"),
                literal: s.to_owned(),
                containers: containers.clone(),
            });
        } else if v.get(key).is_some_and(|x| !x.is_null()) {
            return Err(format!("campo {path}/{key} no es texto ni ausencia"));
        }
    }
    if let Some(owners) = v["owners"].as_array() {
        for (i, owner) in owners.iter().enumerate() {
            if !owner.is_object() {
                return Err(format!("propietario {path}/{i} no es objeto"));
            }
            atoms(
                owner,
                &format!("{path}/owners/{i}"),
                out,
                &containers,
                depth + 1,
            )?;
        }
    }
    for key in ["member_of", "part_of"] {
        if v[key].is_object() {
            atoms(
                &v[key],
                &format!("{path}/{key}"),
                out,
                &containers,
                depth + 1,
            )?;
        }
    }
    Ok(())
}
fn inspect(text: &str, reference: &Value, path: &str) -> Result<Value, String> {
    let mention = reference["mention_quote"].as_str().ok_or("falta mención")?;
    if mention.is_empty() || mention.len() > 4096 {
        return Err("mención vacía o demasiado larga".into());
    }
    if !text.contains(mention) {
        return Err("mención no literal en el texto".into());
    }
    let mut fields = Vec::new();
    atoms(reference, path, &mut fields, &[], 0)?;
    let mut covered = vec![false; mention.len()];
    let mut bindings = Vec::new();
    for leaf in fields {
        let candidates = occurrences(mention, &leaf.literal);
        let mut begin = 0;
        let mut end = mention.len();
        let mut container_state = "unique_literal_containers";
        for container in &leaf.containers {
            let Some(container) = container else {
                container_state = "container_missing_not_assigned";
                break;
            };
            match occurrences(&mention[begin..end], container).as_slice() {
                [offset] => {
                    begin += offset;
                    end = begin + container.len();
                }
                [] => {
                    container_state = "container_not_in_this_mention";
                    break;
                }
                _ => {
                    container_state = "ambiguous_container_not_assigned";
                    break;
                }
            }
        }
        let eligible = if container_state == "unique_literal_containers" {
            occurrences(&mention[begin..end], &leaf.literal)
                .into_iter()
                .map(|i| begin + i)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let state = if container_state != "unique_literal_containers" {
            container_state
        } else {
            match eligible.as_slice() {
                [start] => {
                    covered[*start..*start + leaf.literal.len()].fill(true);
                    "unique_literal_occurrence"
                }
                [] => "empty_or_not_in_this_mention",
                _ => "ambiguous_occurrence_not_assigned",
            }
        };
        bindings.push(json!({"field":leaf.path,"literal":leaf.literal,"byte_starts":candidates,"container_mentions":leaf.containers,"eligible_byte_starts":eligible,"state":state}));
    }
    let mut residuals = Vec::new();
    let mut open = None;
    for (i, _) in mention.char_indices() {
        if !covered[i] && open.is_none() {
            open = Some(i);
        }
        if covered[i] {
            if let Some(start) = open.take() {
                residuals.push(json!({"byte_start":start,"byte_end":i,"text":&mention[start..i]}));
            }
        }
    }
    if let Some(start) = open {
        residuals
            .push(json!({"byte_start":start,"byte_end":mention.len(),"text":&mention[start..]}));
    }
    let mut partition = Vec::new();
    let mut from = 0;
    let mut previous = covered[0];
    for (i, _) in mention.char_indices().skip(1) {
        if covered[i] != previous {
            partition.push(json!({"byte_start":from,"byte_end":i,"text":&mention[from..i],"covered_by_unique_leaf":previous}));
            from = i;
            previous = covered[i];
        }
    }
    partition.push(json!({"byte_start":from,"byte_end":mention.len(),"text":&mention[from..],"covered_by_unique_leaf":previous}));
    Ok(
        json!({"path":path,"mention_quote":mention,"mention_literal_in_text":text.contains(mention),"field_occurrences":bindings,"uncovered_spans":residuals,"semantic_fidelity_certified":false,"zero_residual_is_not_a_pass":true,"audit_state":"advisory_observed_mention","offset_unit":"utf8_bytes_half_open","partition":partition}),
    )
}
fn walk(
    text: &str,
    v: &Value,
    path: &str,
    rows: &mut Vec<Value>,
    depth: usize,
) -> Result<(), String> {
    if depth > 64 || rows.len() > 10000 {
        return Err("estructura demasiado profunda o extensa".into());
    }
    match v {
        Value::Object(m) => {
            if ["mention_quote", "type_quote", "owners", "scope"]
                .iter()
                .any(|k| m.contains_key(*k))
                || path.contains("/owners/")
                || path.ends_with("/member_of")
                || path.ends_with("/part_of")
            {
                rows.push(match inspect(text, v, path) {
                    Ok(row) => row,
                    Err(reason) => json!({"path":path,"audit_state":"not_audited_invalid_reference","reason":reason,"mention_quote":v.get("mention_quote"),"uncovered_spans":null,"semantic_fidelity_certified":false}),
                });
            }
            for (k, x) in m {
                let escaped = k.replace('~', "~0").replace('/', "~1");
                walk(text, x, &format!("{path}/{escaped}"), rows, depth + 1)?;
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                walk(text, x, &format!("{path}/{i}"), rows, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}
pub fn inspect_raw(text: &str, raw: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    walk(text, raw, "", &mut rows, 0)?;
    let state = if raw["status"] == "unsupported" {
        "not_audited_unsupported"
    } else if rows.is_empty() {
        "no_references_observed_not_audited"
    } else {
        "emitted_references_only"
    };
    Ok(
        json!({"audit_state":state,"references":rows,"semantic_fidelity_certified":false,"decision_or_repair":null,"unverified_dimensions":["scope","count","owner_order","member_of_meaning","part_of_meaning","ontology_binding","assumptions","context","configuration","omitted_clauses_or_mentions"]}),
    )
}
pub fn audit(attempt: &Path, output: &Path) -> Result<(), String> {
    let requests = fs::read(attempt.join("requests.jsonl")).map_err(|e| e.to_string())?;
    let journal = fs::read(attempt.join("journal.jsonl")).map_err(|e| e.to_string())?;
    let prep: Value = serde_json::from_slice(
        &fs::read(attempt.join("preparation.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if prep["requests_sha256"] != hash(&requests) {
        return Err("peticiones no coinciden con su preparación".into());
    }
    let parse = |bytes: &[u8]| -> Result<Vec<Value>, String> {
        String::from_utf8(bytes.to_vec())
            .map_err(|e| e.to_string())?
            .lines()
            .map(|s| serde_json::from_str(s).map_err(|e| e.to_string()))
            .collect()
    };
    let req = parse(&requests)?;
    if prep["max_http_requests"].as_u64() != Some(req.len() as u64)
        || prep["source_operations"].as_u64()
            != Some(req.iter().filter(|r| r["kind"] == "source").count() as u64)
        || prep["question_operations"].as_u64()
            != Some(req.iter().filter(|r| r["kind"] == "question").count() as u64)
        || req
            .iter()
            .any(|r| r["kind"] != "source" && r["kind"] != "question")
    {
        return Err("cantidad o tipos de operaciones no coinciden con preparación".into());
    }

    let mut starts = BTreeMap::new();
    let mut ends = BTreeMap::new();
    for e in parse(&journal)? {
        let id = e["id"].as_str().ok_or("evento sin ID")?.to_owned();
        if !req.iter().any(|r| r["id"] == id) {
            return Err("evento ajeno".into());
        }
        if e["kind"] == "started" {
            if starts.insert(id, ()).is_some() {
                return Err("inicio duplicado".into());
            }
        } else {
            if !starts.contains_key(&id) || ends.insert(id, e).is_some() {
                return Err("fin duplicado o sin inicio".into());
            }
        }
    }
    let mut operations = Vec::new();
    let mut seen = BTreeMap::new();
    for r in req {
        let id = r["id"].as_str().ok_or("petición sin ID")?;
        if seen.insert(id.to_owned(), ()).is_some() {
            return Err("petición duplicada".into());
        }
        let text = r["text"].as_str().ok_or("petición sin texto")?;
        let audit = match ends.get(id) {
            Some(e)
                if e["kind"] == "response"
                    && e["http_status"] == 200
                    && e["body"]["choices"]
                        .as_array()
                        .is_some_and(|a| a.len() == 1)
                    && e["body"]["choices"][0]["finish_reason"] == "stop" =>
            {
                match e["body"]["choices"][0]["message"]["content"]
                    .as_str()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok())
                {
                    Some(raw) => {
                        json!({"state":"observed_raw","raw_sha256":hash(raw.to_string().as_bytes()),"audit":inspect_raw(text, &raw)?})
                    }
                    None => json!({"state":"unavailable_invalid_json","audit":null}),
                }
            }
            _ => json!({"state":"unavailable_no_completed_http200","audit":null}),
        };
        operations.push(json!({"id":id,"kind":r["kind"],"result":audit}));
    }
    let report = json!({"version":"reference-coverage-v1-advisory","requests_sha256":hash(&requests),"journal_sha256":hash(&journal),"operations":operations,"provider_calls":0,"no_stopword_whitelist":true,"no_normalization":true,"semantic_fidelity_certified":false,"quality_gate":null,"input_labels_or_review_read":false});
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| e.to_string())?;
    file.write_all(
        serde_json::to_string_pretty(&report)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn r(mention: &str, noun: Option<&str>, name: Option<&str>) -> Value {
        json!({"mention_quote":mention,"type_quote":noun,"name":name,"owners":[],"scope":"single"})
    }
    fn residual(reference: Value) -> String {
        let result = inspect(reference["mention_quote"].as_str().unwrap(), &reference, "").unwrap();
        result["uncovered_spans"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["text"].as_str().unwrap())
            .collect()
    }
    #[test]
    fn missing_intermediate_owner_and_declared_name_are_visible_without_repair() {
        let mut x = r(
            "El formulario del pedido O7 de Darsa",
            Some("formulario"),
            None,
        );
        x["owners"] = json!([{"mention_quote":"Darsa","type_quote":null,"name":"Darsa"}]);
        let before = x.clone();
        let gaps = residual(x.clone());
        assert!(gaps.contains("pedido O7"));
        assert_eq!(x, before);
        assert_eq!(residual(r("Cima", None, None)), "Cima");
        assert_eq!(
            residual(r("expediente Atlas", None, Some("Atlas"))),
            "expediente "
        );
    }
    #[test]
    fn no_stopwords_case_folding_or_unicode_byte_corruption() {
        assert_eq!(residual(r("La Paz", None, Some("Paz"))), "La ");
        assert_eq!(residual(r("Árbol 東京", Some("Árbol"), Some("東京"))), " ");
        assert_eq!(residual(r("Atlas", None, Some("atlas"))), "Atlas");
        assert_eq!(residual(r("aaa", None, Some("aa"))), "aaa");
    }
    #[test]
    fn evidence_container_and_repeated_names_do_not_hide_missing_fields() {
        let mut x = r("red del huerto Verbena", Some("red"), None);
        x["owners"] =
            json!([{"mention_quote":"huerto Verbena","type_quote":null,"name":"Verbena"}]);
        assert!(residual(x).contains("huerto"));
        let v = inspect("A de A", &r("A de A", None, Some("A")), "").unwrap();
        assert_eq!(
            v["field_occurrences"][0]["state"],
            "ambiguous_occurrence_not_assigned"
        );
        assert_eq!(v["uncovered_spans"][0]["text"], "A de A");
    }
    #[test]
    fn nested_links_and_all_readings_are_visited_and_never_rewritten() {
        let mut x = r("sensor S del equipo X", Some("sensor"), Some("S"));
        x["part_of"] = r("equipo X", Some("equipo"), Some("X"));
        let raw = json!({"certain":[{"subject":x}],"readings":[{"clauses":[{"subject":r("equipo Y",Some("equipo"),Some("Y"))}]}]});
        let before = raw.clone();
        let a = inspect_raw("sensor S del equipo X; equipo Y", &raw).unwrap();
        assert_eq!(a["references"].as_array().unwrap().len(), 3);
        assert_eq!(raw, before);
        assert_eq!(a["semantic_fidelity_certified"], false);
        assert!(a["decision_or_repair"].is_null());
    }
    #[test]
    fn zero_residual_cannot_certify_owner_role_scope_or_ontology() {
        let mut x = r("Atlas", None, Some("Atlas"));
        x["scope"] = json!("all_members");
        let a = inspect("Atlas", &x, "").unwrap();
        assert!(a["uncovered_spans"].as_array().unwrap().is_empty());
        assert_eq!(a["semantic_fidelity_certified"], false);
        let mut role = r("caja K", Some("K"), Some("caja"));
        role["count"] = json!(3);
        assert_eq!(residual(role), " "); // words preserved despite swapped semantic roles
    }
    #[test]
    fn grammatical_and_corrigible_residuals_are_not_declared_errors() {
        let x = r("la caja roja K", Some("caja"), Some("K"));
        let a = inspect("la caja roja K", &x, "").unwrap();
        assert!(residual(x).contains("roja"));
        assert!(a.get("missing_identity").is_none());
        assert_eq!(a["semantic_fidelity_certified"], false);
    }
    #[test]
    fn partition_reconstructs_unicode_and_does_not_normalize_combining_forms() {
        let phrase = "El Árbol 東京 e\u{301}";
        let v = inspect(phrase, &r(phrase, Some("Árbol"), Some("東京")), "").unwrap();
        let mut bytes = 0;
        let rebuilt = v["partition"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| {
                assert_eq!(x["byte_start"].as_u64().unwrap() as usize, bytes);
                let end = x["byte_end"].as_u64().unwrap() as usize;
                assert_eq!(&phrase[bytes..end], x["text"].as_str().unwrap());
                bytes = end;
                x["text"].as_str().unwrap()
            })
            .collect::<String>();
        assert_eq!(rebuilt, phrase);
        assert_eq!(bytes, phrase.len());
        assert_eq!(residual(r("e\u{301}", None, Some("é"))), "e\u{301}");
    }
    #[test]
    fn distributed_identity_and_external_parent_are_explicit_limits() {
        let mut x = r("sensor S", Some("sensor"), Some("S"));
        x["part_of"] = r("equipo X", Some("equipo"), Some("X"));
        let a = inspect_raw("sensor S pertenece a equipo X", &x).unwrap();
        let parent = &a["references"][0];
        assert!(
            parent["field_occurrences"]
                .as_array()
                .unwrap()
                .iter()
                .any(|x| x["literal"] == "X" && x["state"] == "container_not_in_this_mention")
        );
        assert_eq!(a["references"].as_array().unwrap().len(), 2);
        let mut y = x.clone();
        y.as_object_mut().unwrap().remove("part_of");
        let short = inspect_raw("sensor S pertenece a equipo X", &y).unwrap();
        assert_eq!(
            short["references"][0]["uncovered_spans"],
            parent["uncovered_spans"]
        );
        // A missing externally declared link does not necessarily change the local residual.
    }
    #[test]
    fn universal_member_cardinality_and_owner_order_cannot_be_validated_by_coverage() {
        let mut x = r("todos los sensores de L", Some("sensores"), None);
        x["owners"] = json!([{"mention_quote":"L","type_quote":null,"name":"L"}]);
        x["scope"] = json!("all_members");
        let a = residual(x.clone());
        x["scope"] = json!("single");
        x["count"] = json!(3);
        assert_eq!(a, residual(x));
        let mut y = r("sensor S de A para B", Some("sensor"), Some("S"));
        y["owners"] = json!([{"mention_quote":"A","name":"A","type_quote":null},{"mention_quote":"B","name":"B","type_quote":null}]);
        let before = residual(y.clone());
        y["owners"].as_array_mut().unwrap().reverse();
        assert_eq!(before, residual(y));
        assert_ne!(residual(r("sensores", Some("sensor"), None)), "");
    }
    #[test]
    fn ontology_roles_and_additive_link_meaning_are_outside_lexical_evidence() {
        let mut x = r("depósito R4", Some("depósito"), Some("R4"));
        x["ontology_type"] = json!({"type_id":"network","quote":"depósito"});
        let baseline = residual(x.clone());
        x["ontology_type"]["type_id"] = json!("reservoir");
        assert_eq!(baseline, residual(x));
        let link = r("equipo X", Some("equipo"), Some("X"));
        let mut y = r("sensor S del equipo X", Some("sensor"), Some("S"));
        y["member_of"] = link.clone();
        let before = residual(y.clone());
        y.as_object_mut().unwrap().remove("member_of");
        y["part_of"] = link;
        assert_eq!(before, residual(y));
    }
    #[test]
    fn missing_nonliteral_and_unsupported_are_not_zero_residual_passes() {
        let a = inspect_raw(
            "Atlas",
            &json!({"subject":{"name":"Atlas","scope":"single"}}),
        )
        .unwrap();
        assert_eq!(
            a["references"][0]["audit_state"],
            "not_audited_invalid_reference"
        );
        assert!(a["references"][0]["uncovered_spans"].is_null());
        let a = inspect_raw("Atlas", &r("Cima", None, Some("Cima"))).unwrap();
        assert_eq!(
            a["references"][0]["audit_state"],
            "not_audited_invalid_reference"
        );
        let a = inspect_raw("Atlas", &json!({"status":"unsupported"})).unwrap();
        assert_eq!(a["audit_state"], "not_audited_unsupported");
        let a = inspect_raw("Atlas", &json!({"document":{"certain":[]}})).unwrap();
        assert_eq!(a["audit_state"], "no_references_observed_not_audited");
        let mut x = r("Atlas", None, Some("Atlas"));
        x["type_quote"] = json!(42);
        assert_eq!(
            inspect_raw("Atlas", &x).unwrap()["references"][0]["audit_state"],
            "not_audited_invalid_reference"
        );
    }
    #[test]
    fn external_or_ambiguous_container_cannot_cover_an_unrelated_local_name() {
        let mut x = r("sensor X", Some("sensor"), None);
        x["part_of"] = r("equipo X", Some("equipo"), Some("X"));
        let a = inspect("sensor X pertenece al equipo X", &x, "").unwrap();
        assert_eq!(residual(x), " X");
        let leaf = a["field_occurrences"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["field"] == "/part_of/name")
            .unwrap();
        assert_eq!(leaf["byte_starts"], json!([7]));
        assert_eq!(leaf["state"], "container_not_in_this_mention");
        assert_eq!(leaf["eligible_byte_starts"], json!([]));
        let mut y = r("Atlas de Atlas", None, None);
        y["owners"] = json!([{"mention_quote":"Atlas","name":"Atlas","type_quote":null}]);
        assert_eq!(residual(y), "Atlas de Atlas");
    }
    #[test]
    fn malformed_links_or_owner_containers_are_not_audited_as_complete() {
        for (key, bad) in [
            ("owners", json!("bad")),
            ("owners", json!([42])),
            ("member_of", json!("bad")),
            ("part_of", json!(false)),
        ] {
            let mut x = r("Atlas", None, Some("Atlas"));
            x[key] = bad;
            let a = inspect_raw("Atlas", &x).unwrap();
            assert_eq!(
                a["references"][0]["audit_state"],
                "not_audited_invalid_reference"
            );
            assert!(a["references"][0]["uncovered_spans"].is_null());
        }
    }
    #[test]
    fn file_audit_keeps_missing_operations_rejects_roster_mismatch_and_never_overwrites() {
        let temp = std::env::temp_dir().join(format!(
            "gennie-memory-reference-audit-{}",
            std::process::id()
        ));
        fs::create_dir(&temp).unwrap();
        let req = json!({"id":"op0","kind":"source","text":"Atlas"}).to_string() + "\n";
        fs::write(temp.join("requests.jsonl"), &req).unwrap();
        fs::write(temp.join("journal.jsonl"), "").unwrap();
        let mut prep = json!({"requests_sha256":hash(req.as_bytes()),"max_http_requests":2,"source_operations":1,"question_operations":0});
        fs::write(temp.join("preparation.json"), prep.to_string()).unwrap();
        let out = temp.join("audit.json");
        assert!(audit(&temp, &out).is_err());
        assert!(!out.exists());
        prep["max_http_requests"] = json!(1);
        fs::write(temp.join("preparation.json"), prep.to_string()).unwrap();
        audit(&temp, &out).unwrap();
        let bytes = fs::read(&out).unwrap();
        let a: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(a["operations"].as_array().unwrap().len(), 1);
        assert_eq!(
            a["operations"][0]["result"]["state"],
            "unavailable_no_completed_http200"
        );
        assert!(audit(&temp, &out).is_err());
        assert_eq!(fs::read(&out).unwrap(), bytes);
        fs::write(
            temp.join("journal.jsonl"),
            "{\"id\":\"foreign\",\"kind\":\"started\"}\n",
        )
        .unwrap();
        assert!(audit(&temp, &temp.join("foreign.json")).is_err());
        fs::remove_dir_all(&temp).unwrap();
    }
    #[test]
    fn empty_or_nameless_nested_reference_is_retained_as_not_audited() {
        for key in ["member_of", "part_of"] {
            for child in [json!({}), json!({"name":"X"})] {
                let mut x = r("Atlas", None, Some("Atlas"));
                x[key] = child;
                let a = inspect_raw("Atlas", &x).unwrap();
                assert_eq!(a["references"].as_array().unwrap().len(), 2);
                assert!(
                    a["references"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|r| r["audit_state"] == "not_audited_invalid_reference")
                );
                assert!(
                    a["references"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|r| r["path"] == format!("/{key}"))
                );
            }
        }
    }
}
