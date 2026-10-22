//! Bounded negative events never enter an unconditional legacy inference path.
use super::*;
const KIND: &str = "relative_nonoccurrence_bound";
fn charged() -> bool {
    super::super::context_budget::charge(1, 0)
}
type Node = (String, String);
fn reverse_edge(
    graph: &mut BTreeMap<Node, BTreeSet<Node>>,
    required: Node,
    consumer: &Node,
) -> bool {
    let bytes = required
        .0
        .len()
        .saturating_add(required.1.len())
        .saturating_add(consumer.0.len())
        .saturating_add(consumer.1.len())
        .saturating_add(128);
    if !super::super::context_budget::charge(1, bytes) {
        return false;
    }
    graph.entry(required).or_default().insert(consumer.clone());
    true
}
fn nested_claims(value: &Value, ids: &mut BTreeSet<String>) {
    if !charged() {
        return;
    }
    match value {
        Value::Object(m) => {
            for (key, child) in m {
                if key == "claim_ids" {
                    ids.extend(strings(child.as_array().map(Vec::as_slice).unwrap_or(&[])));
                }
                nested_claims(child, ids);
            }
        }
        Value::Array(a) => {
            for child in a {
                nested_claims(child, ids);
            }
        }
        _ => {}
    }
}
impl Engine<'_> {
    fn relative_bound(&self, c: &Value) -> Option<&Value> {
        rows(c, "qualifier_ids")
            .iter()
            .filter_map(Value::as_str)
            .filter_map(|qid| self.qualifier(self.source, qid))
            .find(|q| charged() && s(q, "kind") == KIND)
    }
    fn bounded_nodes(&self) -> BTreeSet<(String, String)> {
        let mut blocked = BTreeSet::new();
        for q in rows(self.source, "qualifiers") {
            if charged() && s(q, "kind") == KIND {
                blocked.insert(("qualifier".into(), id(q).into()));
            }
        }
        for c in rows(self.source, "claims")
            .iter()
            .chain(self.pending_claims)
        {
            if charged() && self.relative_bound(c).is_some() {
                blocked.insert(("claim".into(), id(c).into()));
            }
        }
        let mut reverse: BTreeMap<(String, String), BTreeSet<(String, String)>> = BTreeMap::new();
        for d in rows(self.source, "dependencies") {
            if !charged() {
                break;
            }
            if !reverse_edge(
                &mut reverse,
                (s(&d["requires"], "kind").into(), id(&d["requires"]).into()),
                &(s(&d["consumer"], "kind").into(), id(&d["consumer"]).into()),
            ) {
                break;
            }
        }
        for (table, kind) in [
            ("claims", "claim"),
            ("references", "reference"),
            ("qualifiers", "qualifier"),
            ("scopes", "scope"),
        ] {
            for owner in rows(self.source, table) {
                if !charged() {
                    break;
                }
                let consumer = (kind.to_owned(), id(owner).to_owned());
                for required in super::super::validation::effective_context_dependencies(
                    self.source,
                    kind,
                    id(owner),
                ) {
                    if !charged() {
                        break;
                    }
                    if !reverse_edge(&mut reverse, required, &consumer) {
                        break;
                    }
                }
                if kind == "claim" {
                    let mut nested = BTreeSet::new();
                    nested_claims(&owner["body"], &mut nested);
                    for cid in nested {
                        if !reverse_edge(&mut reverse, ("claim".into(), cid), &consumer) {
                            break;
                        }
                    }
                }
            }
        }
        let mut pending: Vec<_> = blocked.iter().cloned().collect();
        while let Some(required) = pending.pop() {
            if !charged() {
                break;
            }
            if let Some(consumers) = reverse.get(&required) {
                for consumer in consumers {
                    if !charged() {
                        break;
                    }
                    if blocked.insert(consumer.clone()) {
                        pending.push(consumer.clone());
                    }
                }
            }
        }
        blocked
    }
    fn bounded_compatible(&self, c: &Value, r: &Value) -> bool {
        if !charged()
            || !self.branch(self.source, s(c, "scope_id")).is_empty()
            || !self.binders(self.source, s(c, "scope_id")).is_empty()
            || !self.branch(self.question, s(r, "scope_id")).is_empty()
            || !self.binders(self.question, s(r, "scope_id")).is_empty()
            || !self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
            || !self
                .dependency_missing(self.question, "request", id(r))
                .is_empty()
            || !rows(r, "premise_ids").is_empty()
            || !self.request_qualifiers(r).is_empty()
            || r.get("context_bindings").is_some()
            || !rows(self.scope, "configuration").is_empty()
            || !self.scope["time_selector"].is_null()
        {
            return false;
        }
        let b = &c["body"];
        let subject = if s(r, "mode") == "event_slot" {
            let t = &r["target"];
            if t["event_kind"] != "measurement"
                || t["aspect"] != "duration"
                || !matches!(s(t, "status"), "occurred" | "any")
                || !t["time_selector"].is_null()
                || s(&t["method_selector"], "kind") != "any"
                || (!t["event_ref"].is_null()
                    && self.identity(
                        self.source,
                        s(b, "event_ref"),
                        self.question,
                        s(t, "event_ref"),
                    ) != Relation::Same)
            {
                return false;
            }
            s(t, "subject")
        } else {
            let t = &r["target"]["atom"];
            if s(r, "mode") != "answer"
                || s(r, "slot") != "amount"
                || t["predicate"] != "quantity"
                || t["polarity"] != "positive"
                || r["target"]["judgment"] != "fact"
                || r["target"]["status"] != "state"
                || argument(t, "attribute")
                    .is_none_or(|a| s(a, "kind") != "symbol" || id(a) != "duration")
            {
                return false;
            }
            let Some(a) = argument(t, "subject") else {
                return false;
            };
            if s(a, "kind") != "ref" {
                return false;
            }
            id(a)
        };
        (s(&r["basis_requirement"], "kind") == "any"
            || r["basis_requirement"]["value"] == b["basis"])
            && self.context_clean_reference(self.source, s(b, "subject"))
            && self.context_clean_reference(self.question, subject)
            && self.identity(self.source, s(b, "subject"), self.question, subject) == Relation::Same
    }
    pub(super) fn relative_bound_request(&self, r: &Value) -> Option<Value> {
        if !super::super::request_context_profile::permits(self.vocabulary, KIND) {
            return None;
        }
        let blocked = self.bounded_nodes();
        if blocked.is_empty() {
            return None;
        }
        let bytes =
            super::super::context_budget::serialized_bytes(self.source).unwrap_or(usize::MAX);
        if !super::super::context_budget::charge(1, bytes) {
            return Some(
                json!({"request_id":id(r),"support":[],"refutation":[],"partial":[],"unresolved":[],"conflicts":[],"missing_constraints":["request_context_limit"],"coverage":"none"}),
            );
        }
        let mut filtered = self.source.clone();
        filtered["claims"] = json!(
            rows(self.source, "claims")
                .iter()
                .filter(|c| !blocked.contains(&("claim".into(), id(c).into())))
                .collect::<Vec<_>>()
        );
        let pending: Vec<_> = self
            .pending_claims
            .iter()
            .filter(|c| !blocked.contains(&("claim".into(), id(c).into())))
            .cloned()
            .collect();
        let engine = Engine {
            source: &filtered,
            pending_claims: &pending,
            ..*self
        };
        let mut result = engine.request_without_relative_bound(r);
        for c in rows(self.source, "claims") {
            if !charged() {
                break;
            }
            let Some(q) = self.relative_bound(c) else {
                continue;
            };
            if !self.bounded_compatible(c, r) {
                continue;
            }
            let copy_bytes = [
                c,
                r,
                q,
                &self.source["scopes"],
                &self.question["scopes"],
                &self.source["dependencies"],
                &self.question["dependencies"],
                &self.source["qualifiers"],
                &self.question["qualifiers"],
            ]
            .iter()
            .try_fold(1024usize, |total, v| {
                total.checked_add(super::super::context_budget::serialized_bytes(v)?)
            })
            .unwrap_or(usize::MAX);
            if !super::super::context_budget::charge(1, copy_bytes) {
                break;
            }
            let payload = json!({"source_owner":c,"source_qualifier":q,"question_owner":r,"source_scopes":rows(self.source,"scopes"),"question_scopes":rows(self.question,"scopes"),"source_dependencies":rows(self.source,"dependencies"),"question_dependencies":rows(self.question,"dependencies"),"source_qualifiers":rows(self.source,"qualifiers"),"question_qualifiers":rows(self.question,"qualifiers"),"boundary_subject":self.reference(self.source,s(&q["body"],"boundary_subject")),"boundary_occurrence_inferred":false,"subsequent_measurement_inferred":false,"requested_value_inferred":false});
            if !super::super::context_budget::charge(
                1,
                super::super::context_budget::serialized_bytes(&payload).unwrap_or(usize::MAX),
            ) {
                break;
            }
            let w = json!({"id":format!("{}:{}:relative-bound",id(r),id(c)),"kind":"partial_relative_nonoccurrence_bound","claim_ids":[id(c)],"request_id":id(r),"qualifier_ids":c["qualifier_ids"],"answer_terms":[],"answer_propositions":[],"bindings":[],"evidence_refs":c["evidence"],"missing_constraints":["relative_nonoccurrence_bound","requested_value"],"relative_bound_witness":payload});
            if let Some(partials) = result["partial"].as_array_mut() {
                partials.push(w);
            }
            let mut missing = strings(rows(&result, "missing_constraints"));
            missing.extend([KIND.into(), "requested_value".into()]);
            unique(&mut missing);
            result["missing_constraints"] = json!(missing);
            if result["coverage"] == "none" {
                result["coverage"] = json!("partial");
            }
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    const ST: &str = "No se midió la duración de Vela hasta apagarse. Vela pesa ocho gramos.";
    const QT: &str = "¿Qué duración tiene Vela?";
    fn hash(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
    fn quote(t: &str) -> Value {
        json!({"start_byte":0,"end_byte":t.len(),"quote":t})
    }
    fn vocab() -> Value {
        json!({"version":"relative-bound-controls-v1","predicate_rows":[{"id":"quantity","roles":{"subject":"ref","attribute":"attribute_id","amount":"quantity"},"distributive":true}],"fixed_attribute_rows":["duration","mass"],"fixed_action_rows":["switch_off"],"semantic_type_rows":[],"relevance_links":[],"request_context_profile":{"version":"documentary_request_context_v1","features":[KIND]}})
    }
    fn document(t: &str, kind: &str, v: &Value) -> Value {
        let start = t.find("Vela").unwrap();
        let name = json!({"start_byte":start,"end_byte":start+4,"quote":"Vela"});
        json!({"version":"documentary_evidence_v3","text_sha256":hash(t.as_bytes()),"vocabulary_sha256":hash(&serde_json::to_vec(v).unwrap()),"kind":kind,"domain_namespace":"relative-bound-controls","scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":[{"id":"device","scope_id":"root","mention":name,"name":name,"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}],"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":[{"consumer":{"kind":"reference","id":"device"},"requires":{"kind":"scope","id":"root"},"role":"binding"}],"coverage":[{"span":quote(t),"state":"represented","owner":null,"issue_id":null}]})
    }
    fn fixture() -> (Value, Value, Value) {
        let v = vocab();
        let mut a = document(ST, "source", &v);
        let mut b = document(QT, "question", &v);
        a["qualifiers"] = json!([{"id":"bound","scope_id":"root","kind":KIND,"body":{"relation":"until","boundary_subject":"device","boundary_action":"switch_off"},"resolution":"known","evidence":[quote(ST)]}]);
        a["claims"] = json!([{"id":"measurement","scope_id":"root","qualifier_ids":["bound"],"evidence":[quote(ST)],"body":{"kind":"event","event_ref":null,"event_kind":"measurement","subject":"device","aspect":"duration","status":"negated_occurrence","method":{"state":"unspecified"},"times":[],"result":null,"basis":"description"}}]);
        b["requests"] = json!([{"id":"ask","scope_id":"root","mode":"answer","target":{"atom":{"predicate":"quantity","arguments":[{"role":"subject","value":{"kind":"ref","id":"device"}},{"role":"attribute","value":{"kind":"symbol","id":"duration","evidence":[quote(QT)]}}],"polarity":"positive","evidence":[quote(QT)]},"judgment":"fact","status":"state","qualifier_ids":[]},"slot":"amount","class_filter":null,"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":"one_value","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[quote(QT)]}]);
        for (doc, kind, cid) in [(&mut a, "claim", "measurement"), (&mut b, "request", "ask")] {
            for (required_kind, rid, role) in [
                ("scope", "root", "binding"),
                ("reference", "device", "identity"),
            ] {
                doc["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":kind,"id":cid},"requires":{"kind":required_kind,"id":rid},"role":role}));
            }
        }
        a["dependencies"].as_array_mut().unwrap().extend([
            json!({"consumer":{"kind":"claim","id":"measurement"},"requires":{"kind":"qualifier","id":"bound"},"role":"configuration"}),
            json!({"consumer":{"kind":"qualifier","id":"bound"},"requires":{"kind":"reference","id":"device"},"role":"identity"}),
            json!({"consumer":{"kind":"qualifier","id":"bound"},"requires":{"kind":"scope","id":"root"},"role":"binding"}),
        ]);
        (a, b, v)
    }
    fn run(a: &Value, b: &Value, v: &Value) -> Value {
        let sa = super::super::super::validate(a, ST, v);
        let qb = super::super::super::validate(b, QT, v);
        assert_eq!(sa.report["status"], "complete", "{}", sa.report);
        assert_eq!(qb.report["status"], "complete", "{}", qb.report);
        classify_pair_context(
            &sa,
            &qb,
            v,
            &json!({"namespace":"relative-bound-controls","configuration":[],"time_selector":null,"closed_world":false}),
        )
    }
    #[test]
    fn relative_bound_complete_representation_only_publishes_empty_payload_partial() {
        let (a, b, v) = fixture();
        let r = run(&a, &b, &v);
        assert_eq!(r["complete_answer"], false, "{r}");
        assert_eq!(r["requests"][0]["coverage"], "partial", "{r}");
        let w = &r["requests"][0]["partial"][0];
        for field in ["answer_terms", "answer_propositions", "bindings"] {
            assert_eq!(w[field], json!([]), "{r}");
        }
        assert_eq!(w["relative_bound_witness"]["source_owner"], a["claims"][0]);
        assert_eq!(
            w["relative_bound_witness"]["source_qualifier"],
            a["qualifiers"][0]
        );
        assert_eq!(
            w["relative_bound_witness"]["question_owner"],
            b["requests"][0]
        );
        assert_eq!(
            w["relative_bound_witness"]["boundary_occurrence_inferred"],
            false
        );
        assert_eq!(
            w["relative_bound_witness"]["subsequent_measurement_inferred"],
            false
        );
    }
    #[test]
    fn relative_bound_closed_shape_ownership_and_explicit_dependencies() {
        for mutation in [
            "status",
            "aspect",
            "subject",
            "relation",
            "action",
            "resolution",
            "extra",
            "bodytype",
            "result",
            "unowned",
            "reused",
            "duplicate",
            "scope",
            "identity_edge",
            "binding_edge",
            "owner_edge",
            "dangling",
            "terminal",
        ] {
            let (mut a, _, v) = fixture();
            match mutation {
                "status" => a["claims"][0]["body"]["status"] = json!("occurred"),
                "aspect" => a["claims"][0]["body"]["aspect"] = json!("mass"),
                "subject" => a["claims"][0]["body"]["subject"] = Value::Null,
                "relation" => a["qualifiers"][0]["body"]["relation"] = json!("before"),
                "action" => a["qualifiers"][0]["body"]["boundary_action"] = json!("start"),
                "resolution" => a["qualifiers"][0]["resolution"] = json!("unknown"),
                "extra" => a["qualifiers"][0]["body"]["inclusive"] = json!(true),
                "bodytype" => {
                    a["qualifiers"][0]["body"]["boundary_subject"] =
                        json!({"kind":"ref","id":"device"})
                }
                "result" => {
                    a["claims"][0]["body"]["result"] = json!({"kind":"quantity","decimal":"8","unit":"h","bound":"exact","evidence":[quote(ST)]})
                }
                "unowned" => a["claims"][0]["qualifier_ids"] = json!([]),
                "reused" => {
                    let mut c = a["claims"][0].clone();
                    c["id"] = json!("other");
                    a["claims"].as_array_mut().unwrap().push(c);
                }
                "duplicate" => a["claims"][0]["qualifier_ids"] = json!(["bound", "bound"]),
                "scope" => a["qualifiers"][0]["scope_id"] = json!("elsewhere"),
                "identity_edge" | "binding_edge" | "owner_edge" => a["dependencies"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|d| match mutation {
                        "identity_edge" => {
                            !(d["consumer"]["id"] == "bound" && d["role"] == "identity")
                        }
                        "binding_edge" => {
                            !(d["consumer"]["id"] == "bound" && d["role"] == "binding")
                        }
                        _ => {
                            !(d["consumer"]["id"] == "measurement"
                                && d["requires"]["id"] == "bound")
                        }
                    }),
                "dangling" => a["qualifiers"][0]["body"]["boundary_subject"] = json!("missing"),
                _ => {
                    a["qualifiers"].as_array_mut().unwrap().push(json!({"id":"terminal","scope_id":"root","kind":"configuration","body":{"key":"terminal_condition","value":{"kind":"text","value":"apagarse","evidence":[quote(ST)]}},"resolution":"known","evidence":[quote(ST)]}));
                    a["claims"][0]["qualifier_ids"] = json!(["bound", "terminal"]);
                    a["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":"measurement"},"requires":{"kind":"qualifier","id":"terminal"},"role":"configuration"}));
                }
            }
            let checked = super::super::super::validate(&a, ST, &v);
            assert_eq!(
                checked.report["status"], "invalid",
                "{mutation}: {}",
                checked.report
            );
        }
    }
    #[test]
    fn relative_bound_is_opt_in_and_never_infers_an_unbounded_event_refutation() {
        let (a, mut b, mut v) = fixture();
        b["requests"][0]["mode"] = json!("event_slot");
        b["requests"][0]["slot"] = json!("status");
        b["requests"][0]["target"] = json!({"event_ref":null,"event_kind":"measurement","subject":"device","aspect":"duration","status":"occurred","time_selector":null,"method_selector":{"kind":"any"}});
        let r = run(&a, &b, &v);
        assert_eq!(r["requests"][0]["support"], json!([]));
        assert_eq!(r["requests"][0]["refutation"], json!([]));
        assert_eq!(r["complete_answer"], false);
        v.as_object_mut().unwrap().remove("request_context_profile");
        let mut old = a.clone();
        old["vocabulary_sha256"] = json!(hash(&serde_json::to_vec(&v).unwrap()));
        assert_eq!(
            super::super::super::validate(&old, ST, &v).report["status"],
            "invalid"
        );
    }
    #[test]
    fn relative_bound_unrelated_subject_aspect_and_external_context_are_not_partial() {
        for mutation in ["aspect", "identity", "configuration", "time"] {
            let (a, mut b, v) = fixture();
            if mutation == "aspect" {
                b["requests"][0]["target"]["atom"]["arguments"][1]["value"]["id"] = json!("mass");
            }
            if mutation == "identity" {
                b["references"][0]["name"] = Value::Null;
            }
            let sa = super::super::super::validate(&a, ST, &v);
            let qb = super::super::super::validate(&b, QT, &v);
            assert_eq!(sa.report["status"], "complete");
            assert_eq!(qb.report["status"], "complete");
            let mut scope = json!({"namespace":"relative-bound-controls","configuration":[],"time_selector":null,"closed_world":false});
            if mutation == "configuration" {
                scope["configuration"] = json!([{"key":"context","value":"external"}]);
            }
            if mutation == "time" {
                scope["time_selector"] = json!({"role":"occurred_at"});
            }
            let r = classify_pair_context(&sa, &qb, &v, &scope);
            assert_eq!(r["selected_claim_ids"], json!([]), "{mutation}: {r}");
            assert_eq!(r["complete_answer"], false, "{r}");
        }
    }
    #[test]
    fn relative_bound_dependency_veto_keeps_independent_fact() {
        let (mut a, mut b, v) = fixture();
        b["requests"][0]["target"]["atom"]["arguments"][1]["value"]["id"] = json!("mass");
        for (cid, amount) in [("independent", "8"), ("dependent", "9")] {
            a["claims"].as_array_mut().unwrap().push(json!({"id":cid,"scope_id":"root","qualifier_ids":[],"evidence":[quote(ST)],"body":{"kind":"proposition","value":{"atom":{"predicate":"quantity","arguments":[{"role":"subject","value":{"kind":"ref","id":"device"}},{"role":"attribute","value":{"kind":"symbol","id":"mass","evidence":[quote(ST)]}},{"role":"amount","value":{"kind":"quantity","decimal":amount,"unit":"g","bound":"exact","evidence":[quote(ST)]}}],"polarity":"positive","evidence":[quote(ST)]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]}}}));
            for (required_kind, rid, role) in [
                ("scope", "root", "binding"),
                ("reference", "device", "identity"),
            ] {
                a["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":cid},"requires":{"kind":required_kind,"id":rid},"role":role}));
            }
        }
        a["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":"dependent"},"requires":{"kind":"claim","id":"measurement"},"role":"premise"}));
        let r = run(&a, &b, &v);
        assert_eq!(r["selected_claim_ids"], json!(["independent"]), "{r}");
        assert_eq!(
            r["requests"][0]["support"][0]["answer_terms"][0]["decimal"], "8",
            "{r}"
        );
        assert_eq!(r["requests"][0]["conflicts"], json!([]), "{r}");
    }

    #[test]
    fn relative_bound_absence_preserves_historical_matching_and_legacy_api_rejects_profile() {
        let (mut a, mut b, mut v) = fixture();
        let sa = super::super::super::validate(&a, ST, &v);
        let qb = super::super::super::validate(&b, QT, &v);
        let scope = json!({"namespace":"relative-bound-controls","configuration":[],"time_selector":null,"closed_world":false});
        assert_eq!(classify_pair(&sa, &qb, &v, &scope)["status"], "invalid");
        a["qualifiers"] = json!([]);
        a["claims"][0]["qualifier_ids"] = json!([]);
        a["dependencies"].as_array_mut().unwrap().retain(|d| {
            d["consumer"]["kind"] != "qualifier" && d["requires"]["kind"] != "qualifier"
        });
        let with_profile = run(&a, &b, &v);
        v.as_object_mut().unwrap().remove("request_context_profile");
        for d in [&mut a, &mut b] {
            d["vocabulary_sha256"] = json!(hash(&serde_json::to_vec(&v).unwrap()));
        }
        assert_eq!(with_profile, run(&a, &b, &v));
    }
    #[test]
    fn relative_bound_validation_and_matching_share_outer_budget() {
        let (a, b, v) = fixture();
        let sa = super::super::super::validate(&a, ST, &v);
        let qb = super::super::super::validate(&b, QT, &v);
        let input = json!({"source":a,"question":b,"vocabulary":v});
        let exhausted = super::super::super::context_budget::with_operation(&v, &input, || {
            assert!(super::super::super::context_budget::charge(999_000, 0));
            assert!(!super::super::super::context_budget::charge(1_001, 0));
            let scope = json!({"namespace":"relative-bound-controls","configuration":[],"time_selector":null,"closed_world":false});
            let _ = classify_pair_context(&sa, &qb, &v, &scope);
            super::super::super::validate(&a, ST, &v)
        });
        assert!(exhausted.is_err());
        assert_eq!(
            super::super::super::validate(&a, ST, &v).report["status"],
            "complete"
        );
    }
    #[test]
    fn relative_bound_implicit_normative_scope_and_alias_dependency_cannot_bypass_veto() {
        for use_alias in [false, true] {
            let (mut a, mut b, mut v) = fixture();
            let st = format!(
                "{ST} El tutor debe acompañar a Nube para inscribirse en Ronda. Marco Cobalto."
            );
            let qt = "¿Qué tutor debe acompañar a Nube para inscribirse en Ronda? Según Cobalto.";
            let span = |text: &str, part: &str| {
                let start = text.find(part).unwrap();
                json!({"start_byte":start,"end_byte":start+part.len(),"quote":part})
            };
            v["predicate_rows"].as_array_mut().unwrap().push(json!({"id":"action","roles":{"actor":"ref","action_kind":"action_id","object":"ref"},"distributive":true}));
            v["fixed_action_rows"] = json!(["switch_off", "accompany", "enroll"]);
            v["request_context_profile"]["features"] =
                json!([KIND, "normative_action_relation", "normative_open_pattern"]);
            for (doc, text) in [(&mut a, st.as_str()), (&mut b, qt)] {
                doc["text_sha256"] = json!(hash(text.as_bytes()));
                doc["vocabulary_sha256"] = json!(hash(&serde_json::to_vec(&v).unwrap()));
                for (rid, name) in [
                    ("person", "Nube"),
                    ("course", "Ronda"),
                    ("rule-source", "Cobalto"),
                ] {
                    doc["references"].as_array_mut().unwrap().push(json!({"id":rid,"scope_id":"root","mention":span(text,name),"name":span(text,name),"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}));
                    doc["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":rid},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
                }
            }
            // Keep every original quotation valid in the extended source; replace the
            // unrelated original question with the synthetic normative request.
            b["references"]
                .as_array_mut()
                .unwrap()
                .retain(|r| id(r) != "device");
            b["dependencies"] = json!([]);
            for rid in ["person", "course", "rule-source"] {
                b["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":rid},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
            }
            let mut dependent = "rule-source";
            if use_alias {
                let mut alias = a["references"][3].clone();
                alias["id"] = json!("rule-alias");
                a["references"].as_array_mut().unwrap().push(alias);
                a["identity"] = json!([{"kind":"same","left":"rule-source","right":"rule-alias","evidence":[span(&st,"Cobalto")]}]);
                a["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":"rule-alias"},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
                dependent = "rule-alias";
            }
            a["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":dependent},"requires":{"kind":"claim","id":"measurement"},"role":"premise"}));
            for (doc, text, is_source) in [(&mut a, st.as_str(), true), (&mut b, qt, false)] {
                let atom = json!({"predicate":"action","arguments":[{"role":"action_kind","value":{"kind":"symbol","id":"accompany","evidence":[span(text,"acompañar")]}},{"role":"object","value":{"kind":"ref","id":"person"}}],"polarity":"positive","evidence":[span(text,text)]});
                let filter = json!({"kind":"lexical","class_quote":span(text,"tutor")});
                let context = json!([{"id":"purpose","kind":"normative_action_relation","relation":{"kind":"purpose","evidence":[span(text,"para")]},"action":{"action_kind":{"kind":"symbol","id":"enroll","evidence":[span(text,"inscribirse")]},"actor":{"kind":"owner_slot","role":"actor","evidence":[span(text,"inscribirse")]},"destination":{"kind":"ref","id":"course"},"evidence":[span(text,"inscribirse en Ronda")]},"evidence":[span(text,"para inscribirse en Ronda")]}]);
                if is_source {
                    doc["claims"].as_array_mut().unwrap().push(json!({"id":"norm","scope_id":"root","qualifier_ids":[],"evidence":[span(text,"El tutor debe acompañar a Nube para inscribirse en Ronda.")],"body":{"kind":"normative_open_pattern","target":atom,"open_role":{"role":"actor","class_filter":filter,"evidence":[span(text,"tutor")]},"judgment":"required","status":"state","basis":"normative","normative_scope":"rule-source"},"context_bindings":context}));
                } else {
                    doc["requests"] = json!([{"id":"ask","scope_id":"root","mode":"answer","target":{"atom":atom,"judgment":"required","status":"state","qualifier_ids":[]},"slot":"actor","class_filter":filter,"unit":null,"basis_requirement":{"kind":"exact","value":"normative"},"required_answer_extent":"one_value","normative_scope_filter":"rule-source","qualifier_ids":[],"premise_ids":[],"evidence":[span(text,text)],"context_bindings":context}]);
                    doc["coverage"] = json!([{"span":span(text,text),"state":"represented","owner":null,"issue_id":null}]);
                }
            }
            // The source pattern omits permitted implicit operand edges. The
            // request still declares its ordinary target and normative filter.
            for (rid, role) in [("person", "identity"), ("rule-source", "normative_target")] {
                b["dependencies"].as_array_mut().unwrap().push(json!({
                    "consumer":{"kind":"request","id":"ask"},
                    "requires":{"kind":"reference","id":rid},"role":role
                }));
            }
            let sa = super::super::super::validate(&a, &st, &v);
            let qb = super::super::super::validate(&b, qt, &v);
            assert_eq!(sa.report["status"], "complete", "{}", sa.report);
            assert_eq!(qb.report["status"], "complete", "{}", qb.report);
            let result = classify_pair_context(
                &sa,
                &qb,
                &v,
                &json!({"namespace":"relative-bound-controls","configuration":[],"time_selector":null,"closed_world":false}),
            );
            assert_eq!(
                result["selected_claim_ids"],
                json!([]),
                "alias={use_alias}: {result}"
            );
            assert_eq!(result["complete_answer"], false, "{result}");
            assert_eq!(result["requests"][0]["partial"], json!([]), "{result}");
        }
    }
}
