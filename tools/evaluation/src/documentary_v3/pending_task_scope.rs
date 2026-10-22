//! Closed task equality for pending-page evidence. No missing selector is a wildcard.
use super::*;

const REASON: &str = "pending_task_scope";
fn charged() -> bool {
    super::super::context_budget::charge(1, 0)
}
fn contains_pending(value: &Value) -> bool {
    if !charged() {
        return true;
    }
    if s(value, "predicate") == "quantity"
        && rows(value, "arguments").iter().any(|a| {
            s(a, "role") == "attribute"
                && s(&a["value"], "kind") == "symbol"
                && id(&a["value"]) == "pending_pages"
        })
    {
        return true;
    }
    match value {
        Value::Object(m) => m.values().any(contains_pending),
        Value::Array(a) => a.iter().any(contains_pending),
        _ => false,
    }
}
fn contains_task_key(value: &Value) -> bool {
    if !charged() {
        return true;
    }
    if s(value, "key") == "pending_action" {
        return true;
    }
    match value {
        Value::Object(m) => m.values().any(contains_task_key),
        Value::Array(a) => a.iter().any(contains_task_key),
        _ => false,
    }
}
fn collect_claim_ids(value: &Value, ids: &mut BTreeSet<String>) {
    if !charged() {
        return;
    }
    match value {
        Value::Object(m) => {
            if m.contains_key("body") && !id(value).is_empty() {
                ids.insert(id(value).into());
            }
            for (key, child) in m {
                if key == "claim_ids" {
                    ids.extend(strings(child.as_array().map(Vec::as_slice).unwrap_or(&[])));
                }
                collect_claim_ids(child, ids);
            }
        }
        Value::Array(a) => {
            for child in a {
                collect_claim_ids(child, ids);
            }
        }
        _ => {}
    }
}
impl Engine<'_> {
    // Repeated references to one qualifier are ordinary carrier duplication;
    // two distinct declarations, even with equal values, are ambiguous here.
    fn pending_selector(&self, doc: &Value, ids: &[Value]) -> Result<Option<String>, ()> {
        let mut declarations = vec![];
        for qid in ids {
            if !charged() {
                return Err(());
            }
            let Some(q) = self.qualifier(doc, qid.as_str().ok_or(())?) else {
                return Err(());
            };
            if s(&q["body"], "key") != "pending_action" {
                if contains_task_key(q) {
                    return Err(());
                }
                continue;
            }
            if s(q, "kind") != "configuration"
                || s(q, "resolution") != "known"
                || s(&q["body"]["value"], "kind") != "symbol"
                || !rows(self.vocabulary, "fixed_action_rows")
                    .iter()
                    .any(|a| a.as_str() == Some(id(&q["body"]["value"])))
            {
                return Err(());
            }
            declarations.push(id(&q["body"]["value"]).to_owned());
        }
        if declarations.len() > 1 {
            return Err(());
        }
        Ok(declarations.pop())
    }
    fn pending_claim_safe(&self, c: &Value, r: &Value) -> bool {
        if s(&c["body"], "kind") != "proposition"
            || !contains_pending(&c["body"]["value"]["atom"])
            || s(&c["body"]["value"]["atom"], "predicate") != "quantity"
            || s(&r["target"]["atom"], "predicate") != "quantity"
            || !contains_pending(&r["target"]["atom"])
            || !matches!(
                (s(r, "mode"), s(r, "slot")),
                ("answer", "amount") | ("truth", "status")
            )
            || !self.branch(self.source, s(c, "scope_id")).is_empty()
            || !self.binders(self.source, s(c, "scope_id")).is_empty()
            || !self.branch(self.question, s(r, "scope_id")).is_empty()
            || !self.binders(self.question, s(r, "scope_id")).is_empty()
            || rows(self.scope, "configuration")
                .iter()
                .any(|q| s(q, "key") == "pending_action")
        {
            return false;
        }
        // Embedded atoms/conditions in a nominally flat carrier remain unsupported.
        let flat = |p: &Value| {
            rows(&p["atom"], "arguments")
                .iter()
                .all(|a| matches!(s(&a["value"], "kind"), "ref" | "symbol" | "quantity"))
        };
        let p = &c["body"]["value"];
        if !flat(p) || !flat(&r["target"]) {
            return false;
        }
        let a = self.pending_selector(self.source, &self.claim_qualifiers(c, p));
        let b = self.pending_selector(self.question, &self.request_qualifiers(r));
        matches!((a,b), (Ok(a),Ok(b)) if a == b)
    }
    fn pending_provenance(&self, w: &Value) -> Vec<&Value> {
        let mut ids = BTreeSet::new();
        collect_claim_ids(w, &mut ids);
        // A rule or dependent claim cannot launder the task of its premises.
        let mut pending: Vec<_> = ids
            .iter()
            .cloned()
            .map(|id| ("claim".to_owned(), id))
            .collect();
        let mut visited = BTreeSet::new();
        while let Some((kind, cid)) = pending.pop() {
            if !charged() || !visited.insert((kind.clone(), cid.clone())) {
                continue;
            }
            if kind == "claim" {
                ids.insert(cid.clone());
            }
            for dep in rows(self.source, "dependencies") {
                if !charged() {
                    break;
                }
                if s(&dep["consumer"], "kind") == kind && s(&dep["consumer"], "id") == cid {
                    pending.push((
                        s(&dep["requires"], "kind").into(),
                        s(&dep["requires"], "id").into(),
                    ));
                }
            }
        }
        rows(self.source, "claims")
            .iter()
            .filter(|c| charged() && ids.contains(id(c)))
            .collect()
    }
    fn pending_rejection(&self, w: &Value, r: &Value) -> Option<Value> {
        let claims = self.pending_provenance(w);
        let sensitive: Vec<_> = claims
            .iter()
            .copied()
            .filter(|c| contains_pending(&c["body"]))
            .collect();
        if sensitive.is_empty()
            || claims.len() == 1 && sensitive.iter().all(|c| self.pending_claim_safe(c, r))
        {
            return None;
        }
        let ids: Vec<_> = claims.iter().map(|c| id(c)).collect();
        let evidence: Vec<_> = claims
            .iter()
            .flat_map(|c| rows(c, "evidence").iter().cloned())
            .collect();
        let bytes = super::super::context_budget::serialized_bytes(&json!([&ids, &evidence]))
            .unwrap_or(usize::MAX);
        super::super::context_budget::charge(1, bytes);
        Some(
            json!({"claim_ids":ids,"request_id":id(r),"reference_ids":[],"state":"unresolved","reason_codes":[REASON],"evidence_refs":evidence}),
        )
    }
    pub(super) fn pending_task_scope(&self, r: &Value, mut result: Value) -> Value {
        if !super::super::request_context_profile::permits(self.vocabulary, REASON) {
            return result;
        }
        let mut rejected_ids = BTreeSet::new();
        let mut rejected = vec![];
        let mut changed = false;
        for key in ["support", "refutation", "partial", "unresolved"] {
            let mut kept = vec![];
            let Value::Array(witnesses) = result[key].take() else {
                result[key] = json!([]);
                continue;
            };
            for witness in witnesses {
                if let Some(candidate) = self.pending_rejection(&witness, r) {
                    rejected_ids.insert(id(&witness).to_owned());
                    rejected.push(candidate);
                    changed = true;
                } else {
                    kept.push(witness);
                }
            }
            result[key] = Value::Array(kept);
        }
        let mut conflicts = vec![];
        let previous_conflicts = result["conflicts"].take();
        let previous_conflicts = match previous_conflicts {
            Value::Array(conflicts) => conflicts,
            _ => vec![],
        };
        for conflict in previous_conflicts {
            let depends_on_rejected = rows(&conflict, "support_witness_ids")
                .iter()
                .chain(rows(&conflict, "refutation_witness_ids"))
                .filter_map(Value::as_str)
                .any(|wid| rejected_ids.contains(wid));
            if let Some(candidate) = self.pending_rejection(&conflict, r) {
                rejected.push(candidate);
                changed = true;
            } else if depends_on_rejected {
                changed = true;
            } else {
                conflicts.push(conflict);
            }
        }
        result["conflicts"] = Value::Array(conflicts);
        if !changed {
            return result;
        }
        result["unresolved"]
            .as_array_mut()
            .unwrap()
            .extend(rejected);
        let mut missing = strings(rows(&result, "missing_constraints"));
        missing.push(REASON.into());
        unique(&mut missing);
        result["missing_constraints"] = json!(missing);
        let useful = ["support", "refutation", "partial"]
            .iter()
            .any(|k| !rows(&result, k).is_empty());
        result["coverage"] = json!(if !rows(&result, "conflicts").is_empty() {
            "conflict"
        } else if useful {
            "partial"
        } else {
            "none"
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Value, Value, Value, Value) {
        let vocabulary = json!({"fixed_attribute_rows":["pending_pages","mass"],"fixed_action_rows":["digitize","review"],"request_context_profile":{"version":"documentary_request_context_v1","features":["pending_task_scope"]}});
        let atom = json!({"predicate":"quantity","arguments":[{"role":"subject","value":{"kind":"ref","id":"file"}},{"role":"attribute","value":{"kind":"symbol","id":"pending_pages"}},{"role":"amount","value":{"kind":"quantity","decimal":"0","unit":"pages","bound":"exact"}}],"polarity":"positive"});
        let p = json!({"atom":atom,"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]});
        let source = json!({"claims":[{"id":"c","scope_id":"root","qualifier_ids":[],"body":{"kind":"proposition","value":p},"evidence":[{"quote":"zero pending pages"}]}],"qualifiers":[],"dependencies":[],"scopes":[{"id":"root","parent_id":null}]});
        let request = json!({"id":"r","mode":"answer","slot":"amount","scope_id":"root","qualifier_ids":[],"target":p});
        let question = json!({"requests":[request],"claims":[],"qualifiers":[],"dependencies":[],"scopes":[{"id":"root","parent_id":null}]});
        (source, question, vocabulary, json!({"configuration":[]}))
    }
    fn task(doc: &mut Value, source: bool, selector: Option<&str>) {
        let Some(selector) = selector else {
            return;
        };
        doc["qualifiers"] = json!([{"id":"task","kind":"configuration","resolution":"known","body":{"key":"pending_action","value":{"kind":"symbol","id":selector}}}]);
        if source {
            doc["claims"][0]["qualifier_ids"] = json!(["task"]);
        } else {
            doc["requests"][0]["qualifier_ids"] = json!(["task"]);
        }
    }
    fn witness(cid: &str, wid: &str) -> Value {
        json!({"id":wid,"claim_ids":[cid],"request_id":"r","answer_terms":[{"kind":"quantity","decimal":"0"}],"answer_propositions":[{"numerical":"payload"}],"bindings":[{"source_ref":"file","question_ref":"file"}],"evidence_refs":[],"missing_constraints":[]})
    }
    fn output() -> Value {
        json!({"request_id":"r","support":[witness("c","w")],"refutation":[],"partial":[],"unresolved":[],"conflicts":[],"missing_constraints":[],"coverage":"complete"})
    }
    fn guard(
        source: &Value,
        question: &Value,
        vocabulary: &Value,
        scope: &Value,
        result: Value,
    ) -> Value {
        let report = json!({});
        let e = Engine {
            possessive_source: &BTreeSet::new(),
            lexical_source: &BTreeSet::new(),
            possessive_question: &BTreeSet::new(),
            source,
            question,
            vocabulary,
            scope,
            source_report: &report,
            question_report: &report,
            pending_claims: &[],
            context_v2: true,
        };
        e.pending_task_scope(&question["requests"][0], result)
    }
    fn assert_empty_payload(result: &Value) {
        for candidate in rows(result, "unresolved") {
            assert!(candidate.get("answer_terms").is_none(), "{result}");
            assert!(candidate.get("answer_propositions").is_none(), "{result}");
            assert!(candidate.get("bindings").is_none(), "{result}");
        }
    }
    #[test]
    fn all_nine_task_combinations_have_exact_selector_equality() {
        for a in [None, Some("digitize"), Some("review")] {
            for b in [None, Some("digitize"), Some("review")] {
                let (mut s, mut q, v, scope) = fixture();
                task(&mut s, true, a);
                task(&mut q, false, b);
                let original = output();
                let result = guard(&s, &q, &v, &scope, original.clone());
                if a == b {
                    assert_eq!(result, original);
                } else {
                    assert_eq!(result["coverage"], "none", "{a:?}/{b:?}: {result}");
                    assert_empty_payload(&result);
                }
            }
        }
    }
    #[test]
    fn unsafe_nested_and_unsupported_modes_block_all_witness_channels() {
        for body in ["rule", "alternatives", "value_restriction"] {
            for channel in ["support", "refutation", "partial", "unresolved"] {
                let (mut s, q, v, scope) = fixture();
                let p = s["claims"][0]["body"]["value"].clone();
                s["claims"][0]["body"] = json!({"kind":body,"nested":p});
                let mut out = output();
                out["support"] = json!([]);
                out[channel] = json!([witness("c", "w")]);
                let result = guard(&s, &q, &v, &scope, out);
                assert_eq!(result["coverage"], "none", "{body}/{channel}: {result}");
                assert_empty_payload(&result);
            }
        }
        for mode in [
            "configuration_slot",
            "event_slot",
            "requirements",
            "constraint",
        ] {
            let (s, mut q, v, scope) = fixture();
            q["requests"][0]["mode"] = json!(mode);
            assert_eq!(guard(&s, &q, &v, &scope, output())["coverage"], "none");
        }
    }
    #[test]
    fn malformed_unknown_duplicate_and_global_selectors_are_not_absence() {
        for change in ["unknown", "duplicate", "malformed", "undeclared", "global"] {
            let (mut s, mut q, v, mut scope) = fixture();
            task(&mut s, true, Some("digitize"));
            task(&mut q, false, Some("digitize"));
            match change {
                "unknown" => s["qualifiers"][0]["resolution"] = json!("unknown"),
                "malformed" => s["qualifiers"][0]["body"]["value"]["kind"] = json!("text"),
                "undeclared" => s["qualifiers"][0]["body"]["value"]["id"] = json!("unregistered"),
                "duplicate" => {
                    let mut other = s["qualifiers"][0].clone();
                    other["id"] = json!("task2");
                    s["qualifiers"].as_array_mut().unwrap().push(other);
                    s["claims"][0]["body"]["value"]["qualifier_ids"] = json!(["task2"]);
                }
                _ => {
                    scope["configuration"] =
                        json!([{"key":"pending_action","value":{"kind":"symbol","id":"digitize"}}])
                }
            }
            let result = guard(&s, &q, &v, &scope, output());
            assert_eq!(result["coverage"], "none", "{change}: {result}");
            assert_empty_payload(&result);
        }
    }
    #[test]
    fn independent_witness_and_unrelated_attribute_survive_with_conflicts_pruned() {
        let (mut s, mut q, v, scope) = fixture();
        task(&mut q, false, Some("digitize"));
        let mut other = s["claims"][0].clone();
        other["id"] = json!("mass");
        other["body"]["value"]["atom"]["arguments"][1]["value"]["id"] = json!("mass");
        s["claims"].as_array_mut().unwrap().push(other);
        let good = witness("mass", "good");
        let mut out = output();
        out["support"].as_array_mut().unwrap().push(good.clone());
        out["conflicts"] =
            json!([{"support_witness_ids":["w","good"],"refutation_witness_ids":[]}]);
        let result = guard(&s, &q, &v, &scope, out);
        assert_eq!(result["support"], json!([good]));
        assert_eq!(result["conflicts"], json!([]));
        assert_eq!(result["coverage"], "partial");
        assert_empty_payload(&result);
    }
    #[test]
    fn aggregate_and_dependency_provenance_cannot_launder_pending_claim() {
        for nested in [false, true] {
            let (mut s, mut q, v, scope) = fixture();
            task(&mut q, false, Some("review"));
            let mut carrier = s["claims"][0].clone();
            carrier["id"] = json!("carrier");
            carrier["body"] = json!({"kind":"event"});
            s["claims"].as_array_mut().unwrap().push(carrier);
            s["dependencies"] = json!([{"consumer":{"kind":"claim","id":"carrier"},"requires":{"kind":"claim","id":"c"},"role":"premise"}]);
            let mut out = output();
            out["support"][0]["claim_ids"] = json!(["carrier"]);
            if nested {
                out["support"][0]["pending_evidence"] = json!({"claim_ids":["c"]});
                s["dependencies"] = json!([]);
            }
            let result = guard(&s, &q, &v, &scope, out);
            assert_eq!(result["coverage"], "none");
            assert_empty_payload(&result);
        }
    }
    #[test]
    fn proposition_carrier_truth_constraint_and_historical_rows_are_preserved() {
        for mode in ["answer", "truth"] {
            let (mut s, mut q, v, scope) = fixture();
            task(&mut s, true, Some("review"));
            task(&mut q, false, Some("review"));
            s["claims"][0]["qualifier_ids"] = json!([]);
            s["claims"][0]["body"]["value"]["qualifier_ids"] = json!(["task"]);
            q["requests"][0]["mode"] = json!(mode);
            q["requests"][0]["slot"] = json!(if mode == "truth" { "status" } else { "amount" });
            assert_eq!(guard(&s, &q, &v, &scope, output()), output());
        }
        let (s, mut q, mut v, scope) = fixture();
        task(&mut q, false, Some("digitize"));
        v.as_object_mut().unwrap().remove("request_context_profile");
        assert_eq!(guard(&s, &q, &v, &scope, output()), output());
    }

    #[test]
    fn conflict_only_payload_is_guarded_and_independent_conflicts_survive() {
        let (s, mut q, v, scope) = fixture();
        task(&mut q, false, Some("digitize"));
        let independent = json!({"kind":"potential","reason":"unrelated_conflict"});
        let mut out = output();
        out["support"] = json!([]);
        out["conflicts"] = json!([witness("c", "blocked_conflict"), independent]);
        let result = guard(&s, &q, &v, &scope, out);
        assert_eq!(result["conflicts"], json!([independent]));
        assert_eq!(result["coverage"], "conflict");
        assert_eq!(rows(&result, "unresolved").len(), 1);
        assert_empty_payload(&result);
    }
}
