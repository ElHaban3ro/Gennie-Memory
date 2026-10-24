mod tests {
    use super::*;
    fn vocabulary() -> Value {
        json!({"version":"reading-controls-v1","term_description_profile":{"version":"local_reference_descriptions_v1"},"predicate_rows":[{"id":"operates_mode","roles":{"subject":"ref","mode":"ref"},"distributive":true},{"id":"quantity","roles":{"subject":"ref","attribute":"attribute_id","amount":"quantity"},"distributive":true}],"fixed_attribute_rows":["resolution"],"fixed_action_rows":[],"semantic_type_rows":[],"relevance_links":[]})
    }
    fn q(text: &str, part: &str) -> Value {
        let i = text.find(part).unwrap();
        json!({"start_byte":i,"end_byte":i+part.len(),"quote":part})
    }
    fn reference(id: &str) -> Value {
        json!({"kind":"ref","id":id})
    }
    fn doc(text: &str, kind: &str, names: &[&str]) -> Value {
        json!({"version":"documentary_evidence_v3","kind":kind,"text_sha256":hash(text.as_bytes()),"vocabulary_sha256":hash(&serde_json::to_vec(&vocabulary()).unwrap()),"domain_namespace":"readings-controls","scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":names.iter().map(|name|json!({"id":name,"scope_id":"root","mention":q(text,name),"name":q(text,name),"declared_class":if *name=="Suave" {q(text,"modo")} else {Value::Null},"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null})).collect::<Vec<_>>(),"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":[],"coverage":[]})
    }
    fn dependencies(d: &mut Value) {
        let mut ds = vec![];
        for rf in arr(d, "references") {
            ds.push(json!({"consumer":{"kind":"reference","id":rf["id"]},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
        }
        for (table, kind) in [("claims", "claim"), ("requests", "request")] {
            for row in arr(d, table) {
                ds.push(json!({"consumer":{"kind":kind,"id":row["id"]},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
                for rf in arr(d, "references") {
                    ds.push(json!({"consumer":{"kind":kind,"id":row["id"]},"requires":{"kind":"reference","id":rf["id"]},"role":"identity"}));
                }
            }
        }
        d["dependencies"] = json!(ds);
    }
    fn described(text: &str) -> Value {
        json!({"kind":"described_ref","id":"Suave","constraints":[{"atom":{"predicate":"quantity","arguments":[{"role":"subject","value":reference("Suave")},{"role":"attribute","value":{"kind":"symbol","id":"resolution","evidence":[q(text,text)]}},{"role":"amount","value":{"kind":"quantity","decimal":"240","unit":"ppp","bound":"exact","evidence":[q(text,"240 ppp")]}}],"polarity":"positive","evidence":[q(text,text)]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]}],"resolution":"known","evidence":[q(text,"modo Suave a 240 ppp")]})
    }
    fn atom(text: &str, mode: Option<Value>) -> Value {
        let mut args = vec![json!({"role":"subject","value":reference("Prisma")})];
        if let Some(v) = mode {
            args.push(json!({"role":"mode","value":v}));
        }
        json!({"predicate":"operates_mode","arguments":args,"polarity":"positive","evidence":[q(text,text)]})
    }
    fn fixture(specific: bool) -> Value {
        let st = "Prisma funciona únicamente en el modo Suave a 240 ppp.";
        let qt = if specific {
            "¿Prisma puede funcionar en el modo Suave a 240 ppp?"
        } else {
            "¿Qué restricción de modo expresa Prisma?"
        };
        let mut s = doc(st, "source", &["Prisma", "Suave"]);
        s["claims"] = json!([
            {"id":"limit","scope_id":"root","qualifier_ids":[],"evidence":[q(st,st)],"body":{"kind":"value_restriction","target":atom(st,Some(reference("Suave"))),"answer_role":"mode","answer_domain":{"kind":"lexical","class_quote":q(st,"modo")},"closure":"closed","members":[described(st)],"judgment":"fact","status":"state","basis":"assignment","normative_scope":null,"existence":"not_asserted"}},
            {"id":"positive","scope_id":"root","qualifier_ids":[],"evidence":[q(st,st)],"body":{"kind":"proposition","value":{"atom":atom(st,Some(described(st))),"judgment":"fact","status":"state","basis":"assignment","normative_scope":null,"qualifier_ids":[]}}}
        ]);
        dependencies(&mut s);
        let mut qq = doc(
            qt,
            "question",
            if specific {
                &["Prisma", "Suave"]
            } else {
                &["Prisma"]
            },
        );
        qq["requests"] = json!([{"id":"ask","scope_id":"root","mode":if specific {"truth"}else{"constraint"},"target":{"atom":atom(qt,if specific {Some(described(qt))}else{None}),"judgment":"fact","status":"state","qualifier_ids":[]},"slot":if specific {"status"}else{"mode"},"class_filter":if specific {Value::Null}else{json!({"kind":"lexical","class_quote":q(qt,"modo")})},"unit":null,"basis_requirement":if specific {json!({"kind":"exact","value":"capability"})}else{json!({"kind":"any"})},"required_answer_extent":if specific {"truth_status"}else{"all_values"},"normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[q(qt,qt)]}]);
        dependencies(&mut qq);
        let mut cap = s.clone();
        cap["claims"][0]["body"]["basis"] = json!("capability");
        cap["claims"][1]["body"]["value"]["basis"] = json!("capability");
        json!({"vocabulary":vocabulary(),"source":{"text":st,"raw":wrap(&[s,cap],st)},"question":{"text":qt,"raw":qq},"pair_scope":{"namespace":"readings-controls","configuration":[],"time_selector":null,"closed_world":false}})
    }
    fn wrap(docs: &[Value], text: &str) -> Value {
        json!({"version":VERSION,"kind":docs[0]["kind"],"text_sha256":docs[0]["text_sha256"],"vocabulary_sha256":docs[0]["vocabulary_sha256"],"domain_namespace":docs[0]["domain_namespace"],"closure":"closed","readings":docs.iter().enumerate().map(|(i,d)|json!({"id":format!("reading-{i}"),"document":d})).collect::<Vec<_>>(),"ambiguities":if docs.len()>1 {json!([{"id":"modal","span":q(text,text),"dimension":"modal_basis","candidate_reading_ids":(0..docs.len()).map(|i|format!("reading-{i}")).collect::<Vec<_>>(),"closure":"closed","evidence":[q(text,text)]}])}else{json!([])},"unresolved_dimensions":[]})
    }
    fn rename(v: &mut Value, from: &str, to: &str) {
        match v {
            Value::Array(xs) => {
                for x in xs {
                    rename(x, from, to)
                }
            }
            Value::Object(m) => {
                for (k, x) in m {
                    if k == "id" && x == from {
                        *x = json!(to)
                    } else {
                        rename(x, from, to)
                    }
                }
            }
            _ => {}
        }
    }
    #[test]
    fn generic_constraint_preserves_bases_and_specific_capability_stays_conditional() {
        let a = fixture(false);
        let r = classify(&a);
        assert_eq!(r["source"]["status"], "complete", "{r}");
        assert_eq!(r["result"]["complete_answer"], true, "{r}");
        assert_ne!(
            r["result"]["reading_matrix"][0]["result"]["requests"][0]["support"][0]["source_constraint"]
                ["basis"],
            r["result"]["reading_matrix"][1]["result"]["requests"][0]["support"][0]["source_constraint"]
                ["basis"]
        );
        let b = classify(&fixture(true));
        assert_eq!(b["result"]["complete_answer"], false);
        assert_eq!(
            b["result"]["requests"][0]["coverage"],
            "conditional_reading"
        );
    }
    #[test]
    fn named_identity_can_use_different_local_ids_but_anonymous_identity_cannot() {
        let mut a = fixture(false);
        rename(
            &mut a["source"]["raw"]["readings"][1]["document"],
            "Suave",
            "other-mode-id",
        );
        assert_eq!(classify(&a)["result"]["complete_answer"], true);
        a["source"]["raw"]["readings"][1]["document"]["references"][1]["name"] = Value::Null;
        assert_eq!(classify(&a)["result"]["complete_answer"], false);
    }
    #[test]
    fn open_missing_invalid_and_limits_never_promote() {
        for mutation in ["open", "missing", "invalid", "limit"] {
            let mut a = fixture(false);
            match mutation {
                "open" => a["source"]["raw"]["closure"] = json!("open"),
                "missing" => a["source"]["raw"]["readings"][1]["document"] = Value::Null,
                "invalid" => {
                    a["source"]["raw"]["readings"][1]["document"]["text_sha256"] = json!("bad")
                }
                _ => {
                    let rd = a["source"]["raw"]["readings"][0].clone();
                    a["source"]["raw"]["readings"] = json!(vec![rd; 9]);
                }
            }
            assert_eq!(
                classify(&a)["result"]["complete_answer"],
                false,
                "{mutation}"
            );
        }
    }
    #[test]
    fn different_configuration_values_and_same_ids_do_not_pass() {
        let mut a = fixture(false);
        a["source"]["raw"]["readings"][1]["document"]["claims"][0]["body"]["members"][0]["constraints"]
            [0]["atom"]["arguments"][2]["value"]["decimal"] = json!("480");
        assert_eq!(classify(&a)["result"]["complete_answer"], false);
        let mut b = fixture(false);
        b["source"]["raw"]["readings"][1]["document"]["references"][1]["name"] =
            json!(q(b["source"]["text"].as_str().unwrap(), "Prisma"));
        assert_eq!(classify(&b)["result"]["complete_answer"], false);
    }
    #[test]
    fn question_request_drop_or_reorder_blocks_global_completion_and_retains_cells() {
        for reorder in [false, true] {
            let mut a = fixture(false);
            let qt = a["question"]["text"].as_str().unwrap().to_owned();
            let mut d = a["question"]["raw"].clone();
            let mut ask = d["requests"][0].clone();
            ask["id"] = json!("second");
            d["requests"].as_array_mut().unwrap().push(ask);
            dependencies(&mut d);
            let mut other = d.clone();
            if reorder {
                other["requests"].as_array_mut().unwrap().reverse();
            } else {
                other["requests"].as_array_mut().unwrap().pop();
                dependencies(&mut other);
            }
            a["question"]["raw"] = wrap(&[d, other], &qt);
            let r = classify(&a);
            assert_eq!(r["result"]["complete_answer"], false);
            assert_eq!(arr(&r["result"], "reading_matrix").len(), 4);
        }
    }
    #[test]
    fn singleton_legacy_validation_and_file_no_replacement() {
        let mut a = fixture(false);
        let source = a["source"]["raw"]["readings"][0]["document"].clone();
        a["source"]["raw"] = source.clone();
        assert_eq!(
            validate(
                &source,
                a["source"]["text"].as_str().unwrap(),
                &a["vocabulary"]
            ),
            serde_json::to_value(documentary_v3::validate(
                &source,
                a["source"]["text"].as_str().unwrap(),
                &a["vocabulary"]
            ))
            .unwrap()
        );
        let dir = std::env::temp_dir().join(format!(
            "gennie-memory-reading-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let input = dir.join("input.json");
        let old = dir.join("legacy.json");
        let new = dir.join("new.json");
        std::fs::write(&input, serde_json::to_vec(&a).unwrap()).unwrap();
        super::super::run(&input, &old, true).unwrap();
        run(&input, &new, true).unwrap();
        assert_eq!(std::fs::read(&old).unwrap(), std::fs::read(&new).unwrap());
        assert!(run(&input, &new, true).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn same_name_at_different_occurrences_is_not_documentary_identity() {
        let mut input = fixture(false);
        let old = input["source"]["text"].as_str().unwrap().to_owned();
        let text = format!("{old} Otra unidad se llama Prisma.");
        input["source"]["text"] = json!(text);
        input["source"]["raw"]["text_sha256"] = json!(hash(text.as_bytes()));
        for r in input["source"]["raw"]["readings"].as_array_mut().unwrap() {
            r["document"]["text_sha256"] = json!(hash(text.as_bytes()));
        }
        let start = text.rfind("Prisma").unwrap();
        let anchor = json!({"start_byte":start,"end_byte":start+6,"quote":"Prisma"});
        input["source"]["raw"]["readings"][1]["document"]["references"][0]["name"] = anchor.clone();
        input["source"]["raw"]["readings"][1]["document"]["references"][0]["mention"] = anchor;
        let report = classify(&input);
        assert_eq!(report["source"]["status"], "complete", "{report}");
        assert_eq!(report["result"]["complete_answer"], false, "{report}");
    }
    #[test]
    fn qualifier_cycles_and_deep_nesting_do_not_recurse_unboundedly() {
        let document = json!({"qualifiers":[{"id":"a","scope_id":"root","kind":"configuration","body":{"qualifier_ids":["b"]}},{"id":"b","scope_id":"root","kind":"configuration","body":{"qualifier_ids":["a"]}}]});
        assert!(canonical(&json!({"qualifier_ids":["a"]}), &document).is_none());
        let mut value = json!("leaf");
        for _ in 0..70 {
            value = json!([value]);
        }
        assert!(canonical(&value, &json!({})).is_none());
    }
    #[test]
    fn stable_first_request_survives_an_independent_elliptical_second_request() {
        let mut input = fixture(false);
        input["vocabulary"]["fixed_attribute_rows"]
            .as_array_mut()
            .unwrap()
            .push(json!("pending_pages"));
        let vh = hash(&serde_json::to_vec(&input["vocabulary"]).unwrap());
        let old = input["source"]["text"].as_str().unwrap().to_owned();
        let st = format!("{old} Prisma tiene 7 páginas pendientes.");
        input["source"]["text"] = json!(st);
        input["source"]["raw"]["text_sha256"] = json!(hash(st.as_bytes()));
        input["source"]["raw"]["vocabulary_sha256"] = json!(vh);
        for r in input["source"]["raw"]["readings"].as_array_mut().unwrap() {
            let d = &mut r["document"];
            d["text_sha256"] = json!(hash(st.as_bytes()));
            d["vocabulary_sha256"] = json!(vh);
            d["claims"].as_array_mut().unwrap().push(json!({"id":"remaining-pages","scope_id":"root","qualifier_ids":[],"evidence":[q(&st,"Prisma tiene 7 páginas pendientes.")],"body":{"kind":"proposition","value":{"atom":{"predicate":"quantity","arguments":[{"role":"subject","value":reference("Prisma")},{"role":"attribute","value":{"kind":"symbol","id":"pending_pages","evidence":[q(&st,"páginas pendientes")]}},{"role":"amount","value":{"kind":"quantity","decimal":"7","unit":"páginas","bound":"exact","evidence":[q(&st,"7 páginas")]}}],"polarity":"positive","evidence":[q(&st,"Prisma tiene 7 páginas pendientes.")]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]}}}));
            dependencies(d);
        }
        let oldq = input["question"]["text"].as_str().unwrap().to_owned();
        let qt = format!("{oldq} ¿Cuántas páginas quedan pendientes?");
        input["question"]["text"] = json!(qt);
        let mut context = input["question"]["raw"].clone();
        context["text_sha256"] = json!(hash(qt.as_bytes()));
        context["vocabulary_sha256"] = json!(vh);
        context["requests"].as_array_mut().unwrap().push(json!({"id":"pages","scope_id":"root","mode":"answer","target":{"atom":{"predicate":"quantity","arguments":[{"role":"subject","value":reference("Prisma")},{"role":"attribute","value":{"kind":"symbol","id":"pending_pages","evidence":[q(&qt,"páginas quedan pendientes")]}}],"polarity":"positive","evidence":[q(&qt,"¿Cuántas páginas quedan pendientes?")]},"judgment":"fact","status":"state","qualifier_ids":[]},"slot":"amount","class_filter":null,"unit":{"symbol":"páginas","evidence":[q(&qt,"páginas")]},"basis_requirement":{"kind":"any"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[q(&qt,"¿Cuántas páginas quedan pendientes?")]}));
        dependencies(&mut context);
        let mut independent = context.clone();
        independent["references"].as_array_mut().unwrap().push(json!({"id":"unidentified-pages","scope_id":"root","mention":q(&qt,"páginas"),"name":null,"declared_class":q(&qt,"páginas"),"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}));
        independent["requests"][1]["target"]["atom"]["arguments"][0]["value"] =
            reference("unidentified-pages");
        dependencies(&mut independent);
        input["question"]["raw"] = wrap(&[context, independent], &qt);
        input["question"]["raw"]["ambiguities"][0]["dimension"] = json!("reference");
        let report = classify(&input);
        assert_eq!(report["source"]["status"], "complete", "{report}");
        assert_eq!(report["question"]["status"], "complete", "{report}");
        assert_eq!(
            report["result"]["requests"][0]["complete_answer"], true,
            "{report}"
        );
        assert_eq!(
            report["result"]["requests"][1]["coverage"], "conditional_reading",
            "{report}"
        );
        assert_eq!(report["result"]["complete_answer"], false);
    }
    #[test]
    fn reference_link_depth_is_bounded_even_without_cycle() {
        let mut d = doc("Prisma", "source", &["Prisma"]);
        let prototype = d["references"][0].clone();
        let mut refs = vec![];
        for i in 0..66 {
            let mut r = prototype.clone();
            r["id"] = json!(format!("id-{i}"));
            r["name"]["quote"] = json!(format!("name-{i}"));
            r["links"] = if i < 65 {
                json!([{"kind":"associated_with","target":format!("id-{}",i+1),"evidence":[]}])
            } else {
                json!([])
            };
            refs.push(r);
        }
        d["references"] = json!(refs);
        assert!(reference_key(&d, "id-0", &mut BTreeSet::new()).is_none());
    }
    #[test]
    fn convergent_reference_dag_has_shared_node_budget_and_small_control() {
        fn dag(count: usize) -> Value {
            let mut d = doc("Prisma", "source", &["Prisma"]);
            let prototype = d["references"][0].clone();
            let mut refs = vec![];
            for i in 0..count {
                let mut r = prototype.clone();
                r["id"] = json!(format!("id-{i}"));
                r["name"]["quote"] = json!(format!("name-{i}"));
                let links=(1..=2).filter(|delta|i+delta<count).map(|delta|json!({"kind":"associated_with","target":format!("id-{}",i+delta),"evidence":[]})).collect::<Vec<_>>();
                r["links"] = json!(links);
                refs.push(r);
            }
            d["references"] = json!(refs);
            d
        }
        assert!(reference_key(&dag(5), "id-0", &mut BTreeSet::new()).is_some());
        // 18 vertices are enough to exhaust 4096 visits; never construct the huge DAG expansion.
        assert!(reference_key(&dag(18), "id-0", &mut BTreeSet::new()).is_none());
    }
    #[test]
    fn convergent_qualifier_expansion_uses_one_shared_budget() {
        let mut qs = vec![];
        for i in 0..18 {
            let links = (1..=2)
                .filter(|delta| i + delta < 18)
                .map(|delta| json!(format!("q-{}", i + delta)))
                .collect::<Vec<_>>();
            qs.push(json!({"id":format!("q-{i}"),"scope_id":"root","kind":"guard","body":{"qualifier_ids":links}}));
        }
        let document = json!({"qualifiers":qs});
        assert!(canonical(&json!({"qualifier_ids":["q-0"]}), &document).is_none());
    }
    #[test]
    fn contextual_owner_key_ignores_only_local_context_ids_and_detects_condition_swap() {
        let a = fixture(false);
        let doc = &a["source"]["raw"]["readings"][0]["document"];
        let mut owner = doc["claims"][0].clone();
        owner["context_bindings"] = json!([{"id":"local-a","kind":"nominal_during","anchor":reference("Suave"),"evidence":[]}]);
        let key = context_owner_key(&owner, doc).unwrap();
        owner["id"] = json!("renamed-owner");
        owner["context_bindings"][0]["id"] = json!("renamed-context");
        assert_eq!(Some(key.clone()), context_owner_key(&owner, doc));
        owner["context_bindings"][0]["anchor"] = reference("Prisma");
        assert_ne!(Some(key), context_owner_key(&owner, doc));
    }
    #[test]
    fn context_readings_budget_is_global_and_keeps_original_payload_on_exhaustion() {
        let mut a = fixture(false);
        a["vocabulary"]["request_context_profile"] =
            json!({"version":"documentary_request_context_v1","features":["nominal_during"]});
        a["source"]["raw"]["readings"][0]["document"]["claims"] = json!(vec![json!({}); 513]);
        let report = classify(&a);
        assert_eq!(report["result"]["complete_answer"], false);
        assert_eq!(
            report["result"]["issues"][0]["code"],
            "request_context_limit"
        );
        assert_eq!(report["source"], a["source"]);
        assert!(arr(&report["result"], "reading_matrix").is_empty());
    }
    #[test]
    fn contextual_reference_components_are_id_invariant_and_conflicting_aliases_fail() {
        let a = fixture(false);
        let mut doc = a["source"]["raw"]["readings"][0]["document"].clone();
        let mut alias = doc["references"][1].clone();
        alias["id"] = json!("alias");
        doc["references"].as_array_mut().unwrap().push(alias);
        doc["identity"] = json!([{"kind":"same","left":"Suave","right":"alias"}]);
        let vocab = json!({"request_context_profile":{}});
        documentary_v3::context_budget::with_operation(&vocab, &doc, || {
            let left = reference_key(&doc, "Suave", &mut BTreeSet::new()).unwrap();
            assert_eq!(
                Some(left.clone()),
                reference_key(&doc, "alias", &mut BTreeSet::new())
            );
            let mut other = doc.clone();
            rename(&mut other, "alias", "renamed");
            other["identity"][0]["right"] = json!("renamed");
            assert_eq!(
                Some(left),
                reference_key(&other, "renamed", &mut BTreeSet::new())
            );
            other["identity"]
                .as_array_mut()
                .unwrap()
                .push(json!({"kind":"different","left":"Suave","right":"renamed"}));
            assert!(reference_key(&other, "Suave", &mut BTreeSet::new()).is_none());
        })
        .unwrap();
    }
}
