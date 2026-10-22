use super::*;
fn input(name: &str) -> Value {
    let base=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/documentary-v3-admission-request-v1");
    serde_json::from_slice(&std::fs::read(base.join(format!("{name}.json"))).unwrap()).unwrap()
}
fn classify(d:&Value,active:bool)->Value {
    let a=super::super::super::validate(&d["source"]["raw"],s(&d["source"],"text"),&d["vocabulary"]);
    let b=super::super::super::validate(&d["question"]["raw"],s(&d["question"],"text"),&d["vocabulary"]);
    for report in [&a.report,&b.report] { assert_ne!(report["status"],"invalid","{report}"); }
    if active {classify_pair_context(&a,&b,&d["vocabulary"],&d["pair_scope"])}else{classify_pair(&a,&b,&d["vocabulary"],&d["pair_scope"])}
}
#[test]
fn admission_request_context_frozen_positive_and_rejection_contract() {
 for name in ["negative","closed-described","profile-absent","link-absent","premise-negative","premise-description","no-premise-edge","source-positive","descriptor-extra","domain-missing"] {
    let d=input(name);
    if matches!(name,"no-premise-edge"|"descriptor-extra") {
        let side=if name=="descriptor-extra" {"source"}else{"question"};
        let result=super::super::super::validate(&d[side]["raw"],s(&d[side],"text"),&d["vocabulary"]);
        assert_eq!(result.report["status"],"invalid","{}",result.report);
        let path=if name=="descriptor-extra" {"/claims/1/body/members/0"}else{"/request/ask/dependencies/premise_ids"};
        assert!(rows(&result.report,"issues").iter().any(|i|s(i,"path")==path));
        continue;
    }
    let old=classify(&d,false);let result=classify(&d,true);
    let witnesses:Vec<_>=rows(&result,"requests").iter().flat_map(|r|rows(r,"partial")).filter(|w|s(w,"kind")=="partial_admission_request_context").collect();
    let expected=matches!(name,"negative"|"closed-described");
    assert_eq!(!witnesses.is_empty(),expected,"{name}: {result}");
    assert_eq!(result["complete_answer"],false,"{name}: {result}");
    assert!(rows(&old,"selected_claim_ids").is_empty(),"legacy/{name}: {old}");
    if !expected { assert!(rows(&result,"selected_claim_ids").is_empty(),"bypass/{name}: {result}"); }
    for w in witnesses {
      for key in ["answer_terms","answer_propositions","bindings"] {assert_eq!(w[key],json!([]),"{name}/{key}: {w}");}
      assert!(rows(w,"missing_constraints").contains(&json!("premise")),"{w}");
      assert_eq!(w["pending_question_premise"]["claim_body"],d["question"]["raw"]["claims"][0]["body"]);
      for key in ["actual_use_inferred","requested_value_inferred","premise_satisfied","premise_refuted","property_quantity_equated","membership_exclusion_inferred","descriptor_value_promoted"] {assert_eq!(w["admission_request_context"][key],false,"{key}: {w}");}
    }
 }
}
#[test]
fn admission_request_context_profile_rejects_malformed_declarations() {
    for control in ["extra_field","version","duplicate_id","unknown_attribute","slot","key","predicate","subject_role","setting_role","missing_link","binding","carry","conclusion","hops","reverse","transitive","role_type"] {
        let mut d=input("negative");
        let profile=&mut d["vocabulary"]["admission_request_context"];
        match control {
            "extra_field"=>profile["unexpected"]=json!(true),
            "version"=>profile["version"]=json!("unknown"),
            "duplicate_id"=>{let row=profile["rows"][0].clone();profile["rows"].as_array_mut().unwrap().push(row);},
            "unknown_attribute"=>profile["rows"][0]["target_attribute"]=json!("unregistered"),
            "slot"=>profile["rows"][0]["request_slot"]=json!("amount"),
            "key"=>profile["rows"][0]["setting_key"]=json!("program"),
            "predicate"=>profile["rows"][0]["premise_predicate"]=json!("uses"),
            "subject_role"=>profile["rows"][0]["premise_subject_role"]=json!("object"),
            "setting_role"=>profile["rows"][0]["premise_setting_role"]=json!("subject"),
            "missing_link"=>profile["rows"][0]["link_id"]=json!("missing"),
            "role_type"=>{let p=d["vocabulary"]["predicate_rows"].as_array_mut().unwrap().iter_mut().find(|p|p["id"]=="operates_mode").unwrap();p["roles"]["mode"]=json!("term");},
            _=>{let l=d["vocabulary"]["relevance_links"].as_array_mut().unwrap().last_mut().unwrap();match control {
                "binding"=>l["bindings"]=json!([["source.object","target.subject"]]),
                "carry"=>l["carry_roles"]=json!(["source.subject"]),
                "conclusion"=>l["conclusion"]=json!("answer"),
                "hops"=>l["max_hops"]=json!(2),
                "reverse"=>l["allow_reverse"]=json!(true),
                "transitive"=>l["allow_transitive_composition"]=json!(true),_=>unreachable!(),
            }}
        }
        d["source"]["raw"]["vocabulary_sha256"]=json!(digest(&serde_json::to_vec(&d["vocabulary"]).unwrap()));
        let result=super::super::super::validate(&d["source"]["raw"],s(&d["source"],"text"),&d["vocabulary"]);
        assert_eq!(result.report["status"],"invalid","{control}: {}",result.report);
        assert!(rows(&result.report,"issues").iter().any(|i|s(i,"path").starts_with("/vocabulary/admission_request_context")),"{control}: {}",result.report);
    }
}
#[test]
fn admission_request_context_rejects_known_member_via_question_alias() {
    let d=input("question-alias-to-member");
    let result=classify(&d,true);
    assert!(rows(&result,"requests").iter().flat_map(|r|rows(r,"partial")).all(|w|s(w,"kind")!="partial_admission_request_context"),"{result}");
    assert_eq!(result["complete_answer"],false,"{result}");
}
