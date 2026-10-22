//! Opt-in owner-local context matching. A contextual owner never reaches legacy fallback.
use super::*;

fn carries(owner: &Value) -> bool {
    owner.get("context_bindings").is_some() || s(&owner["body"], "kind") == "normative_open_pattern"
}
fn row(out: Evidence, r: &Value) -> Value {
    let complete = !out.support.is_empty() && out.unresolved.is_empty() && out.conflicts.is_empty();
    let coverage = if !out.conflicts.is_empty() {
        "conflict"
    } else if complete {
        "complete"
    } else if !out.partial.is_empty() || !out.support.is_empty() {
        "partial"
    } else {
        "none"
    };
    json!({"request_id":id(r),"support":out.support,"refutation":[],"partial":out.partial,
        "unresolved":out.unresolved,"conflicts":out.conflicts,"missing_constraints":out.missing,"coverage":coverage})
}
impl Engine<'_> {
    fn context_class_filter_equal(&self, a: &Value, b: &Value) -> bool {
        match (s(a, "kind"), s(b, "kind")) {
            ("lexical", "lexical") => {
                !s(&a["class_quote"], "quote").is_empty()
                    && a["class_quote"]["quote"] == b["class_quote"]["quote"]
            }
            ("semantic", "semantic") => !s(a, "type_id").is_empty() && a["type_id"] == b["type_id"],
            _ => false,
        }
    }
    fn context_dependency_carrier(&self, r: &Value) -> bool {
        let mut pending = vec![("request".to_owned(), id(r).to_owned())];
        let mut visited = BTreeSet::new();
        while let Some((kind, rid)) = pending.pop() {
            if !super::super::context_budget::charge(1, 0) {
                return true;
            }
            if !visited.insert((kind.clone(), rid.clone())) {
                continue;
            }
            let table = match kind.as_str() {
                "request" => "requests",
                "claim" => "claims",
                _ => "",
            };
            if rows(self.question, table)
                .iter()
                .any(|o| id(o) == rid && carries(o))
            {
                return true;
            }
            for dep in rows(self.question, "dependencies") {
                if !super::super::context_budget::charge(1, 0) {
                    return true;
                }
                if s(&dep["consumer"], "kind") == kind && s(&dep["consumer"], "id") == rid {
                    pending.push((
                        s(&dep["requires"], "kind").into(),
                        s(&dep["requires"], "id").into(),
                    ));
                }
            }
        }
        rows(r, "premise_ids")
            .iter()
            .filter_map(Value::as_str)
            .any(|cid| {
                rows(self.question, "claims")
                    .iter()
                    .any(|c| id(c) == cid && carries(c))
            })
    }
    fn context_pending(&self, c: &Value, r: &Value, out: &mut Evidence, reason: &str) {
        let bytes = super::super::context_budget::serialized_bytes(c)
            .unwrap_or(usize::MAX)
            .saturating_add(super::super::context_budget::serialized_bytes(r).unwrap_or(usize::MAX))
            .saturating_mul(2);
        if !super::super::context_budget::charge(1, bytes) {
            out.missing.push("request_context_limit".into());
            return;
        }
        let mut candidate = self.candidate(c, r, &[reason.into()]);
        candidate["source_constraint"] = c.clone();
        candidate["request_constraint"] = r.clone();
        out.unresolved.push(candidate);
        out.missing.push(reason.into());
    }
    pub(super) fn context_clean_reference(&self, doc: &Value, rid: &str) -> bool {
        if self.reference(doc, rid).is_none() || self.local(doc, rid, rid) != Relation::Same {
            return false;
        }
        rows(doc, "references")
            .iter()
            .filter(|rf| self.local(doc, rid, id(rf)) == Relation::Same)
            .all(|rf| {
                super::super::context_budget::charge(1, 0)
                    && s(&rf["anaphora"], "status") != "unresolved"
                    && rf["binding_scope_id"].is_null()
                    && self.branch(doc, s(rf, "scope_id")).is_empty()
                    && self.binders(doc, s(rf, "scope_id")).is_empty()
                    && self.dependency_missing(doc, "reference", id(rf)).is_empty()
            })
    }
    fn context_terms(&self, a: &Value, b: &Value, map: &mut Vec<Value>) -> Relation {
        if !super::super::context_budget::charge(1, 0) {
            return Relation::Unknown;
        }
        let relation = self.term(self.source, a, self.question, b);
        if matches!(s(a, "kind"), "ref" | "described_ref")
            && matches!(s(b, "kind"), "ref" | "described_ref")
        {
            if !self.context_clean_reference(self.source, id(a))
                || !self.context_clean_reference(self.question, id(b))
            {
                return Relation::Unknown;
            }
            if relation == Relation::Same {
                map.push(json!({"source":id(a),"question":id(b)}));
            }
        }
        relation
    }
    fn context_participant(
        &self,
        source: &Value,
        question: &Value,
        p: &Value,
        open: bool,
        map: &mut Vec<Value>,
    ) -> Relation {
        match (s(source, "kind"), s(question, "kind")) {
            ("reference", "reference") => {
                self.context_terms(&source["value"], &question["value"], map)
            }
            ("reference", "owner_slot") if !open => {
                let Some(actor) = argument(&p["atom"], "actor") else {
                    return Relation::Unknown;
                };
                if s(actor, "kind") != "ref"
                    || !self.context_clean_reference(self.source, id(actor))
                    || !self.context_clean_reference(self.source, id(&source["value"]))
                {
                    return Relation::Unknown;
                }
                self.local(self.source, id(actor), id(&source["value"]))
            }
            ("owner_slot", "owner_slot") if open => Relation::Same,
            _ => Relation::Unknown,
        }
    }
    fn context_payload(
        &self,
        c: &Value,
        r: &Value,
        map: &[Value],
        kind: &str,
        answer: &str,
    ) -> Value {
        json!({"kind":kind,"source_owner":{"kind":"claim","id":id(c)},"question_owner":{"kind":"request","id":id(r)},
            "source_context_bindings":c["context_bindings"],"question_context_bindings":r["context_bindings"],
            "source_body":c["body"],"question_target":r["target"],"reference_map":map,
            "unresolved_answer":answer,"requested_identity_inferred":false,"requested_value_inferred":false})
    }
    fn context_provenance(&self, w: &mut Value, c: &Value, r: &Value, map: &[Value]) {
        w["request_context_witness"] = json!({"source_owner":c,"question_owner":r,"reference_map":map,
            "source_scope":rows(self.source,"scopes"),"question_scope":rows(self.question,"scopes"),
            "source_qualifiers":rows(self.source,"qualifiers"),"question_qualifiers":rows(self.question,"qualifiers"),
            "source_evidence":c["evidence"],"question_evidence":r["evidence"],"source_dependencies":rows(self.source,"dependencies"),
            "question_dependencies":rows(self.question,"dependencies"),
            "control":{"source_participant":c["context_bindings"][0]["action"]["actor"],"question_participant":r["context_bindings"][0]["action"]["actor"],
            "source_actor":if s(&r["context_bindings"][0]["action"]["actor"],"kind")=="owner_slot" && s(&c["body"],"kind")=="proposition" {argument(&c["body"]["value"]["atom"],"actor").cloned().unwrap_or(Value::Null)}else{Value::Null},"substitution":if s(&c["body"],"kind")=="normative_open_pattern" {"open_roles"} else if s(&r["context_bindings"][0]["action"]["actor"],"kind")=="owner_slot" {"owner_slot_to_source_actor"}else{"reference_comparison"}}});
    }
    fn contextual_claim(&self, c: &Value, r: &Value, out: &mut Evidence) {
        if !super::super::context_budget::charge(1, 0) {
            self.context_pending(c, r, out, "request_context_budget");
            return;
        }
        let source_atom = if s(&c["body"], "kind") == "normative_open_pattern" {
            &c["body"]["target"]
        } else {
            &c["body"]["value"]["atom"]
        };
        if source_atom.get("predicate").is_some()
            && source_atom["predicate"] != r["target"]["atom"]["predicate"]
        {
            return;
        }
        let sc = rows(c, "context_bindings");
        let qc = rows(r, "context_bindings");
        if sc.len() != 1 || qc.len() != 1 {
            self.context_pending(c, r, out, "request_context_missing_or_unsupported");
            return;
        }
        if sc[0]["kind"] != qc[0]["kind"] {
            return;
        }
        let nominal = s(&sc[0], "kind") == "nominal_during";
        let open = s(&c["body"], "kind") == "normative_open_pattern";
        let p = if open {
            json!({"atom":c["body"]["target"],"judgment":c["body"]["judgment"],"status":c["body"]["status"],"basis":c["body"]["basis"],"normative_scope":c["body"]["normative_scope"],"qualifier_ids":[]})
        } else {
            c["body"]["value"].clone()
        };
        let target = &r["target"];
        if (!open && s(&c["body"], "kind") != "proposition")
            || s(r, "mode") != "answer"
            || s(&p["atom"], "polarity") != "positive"
            || s(&target["atom"], "polarity") != "positive"
            || p["judgment"] != target["judgment"]
            || p["status"] != target["status"]
        {
            self.context_pending(c, r, out, "request_context_unsupported");
            return;
        }
        if !self.branch(self.source, s(c, "scope_id")).is_empty()
            || !self.binders(self.source, s(c, "scope_id")).is_empty()
            || !self.branch(self.question, s(r, "scope_id")).is_empty()
            || !self.binders(self.question, s(r, "scope_id")).is_empty()
            || !self
                .dependency_missing(self.source, "claim", id(c))
                .is_empty()
            || !self
                .dependency_missing(self.question, "request", id(r))
                .is_empty()
        {
            self.context_pending(c, r, out, "dependency");
            return;
        }
        let mut map = vec![];
        let mut relation = Relation::Same;
        if nominal {
            relation = self.context_terms(&sc[0]["anchor"], &qc[0]["anchor"], &mut map);
        } else if s(&sc[0], "kind") == "normative_action_relation" {
            let sr = &sc[0]["relation"];
            let qr = &qc[0]["relation"];
            if s(sr, "kind") == "unresolved" || s(qr, "kind") == "unresolved" {
                relation = Relation::Unknown;
            } else if sr["kind"] != qr["kind"] {
                relation = Relation::Different;
            }
            let sa = &sc[0]["action"];
            let qa = &qc[0]["action"];
            relation = relation.merge(self.context_terms(
                &sa["action_kind"],
                &qa["action_kind"],
                &mut map,
            ));
            relation = relation.merge(self.context_terms(
                &sa["destination"],
                &qa["destination"],
                &mut map,
            ));
            relation = relation.merge(self.context_participant(
                &sa["actor"],
                &qa["actor"],
                &p,
                open,
                &mut map,
            ));
        } else {
            relation = Relation::Unknown;
        }
        if relation == Relation::Different {
            return;
        }
        if relation != Relation::Same {
            self.context_pending(c, r, out, "request_context_control_or_identity");
            return;
        }
        if p["atom"]["predicate"] != target["atom"]["predicate"] {
            return;
        }
        let slot = s(r, "slot");
        for wanted in rows(&target["atom"], "arguments") {
            if s(wanted, "role") == slot {
                continue;
            }
            let Some(actual) = argument(&p["atom"], s(wanted, "role")) else {
                self.context_pending(c, r, out, "request_context_target");
                return;
            };
            relation = relation.merge(self.context_terms(actual, &wanted["value"], &mut map));
        }
        if relation == Relation::Different {
            return;
        }
        if relation != Relation::Same || self.scope_match(r, &p) != Relation::Same {
            self.context_pending(c, r, out, "request_context_target");
            return;
        }
        // Every known role must be represented on both sides; actor/amount is the sole hole.
        if rows(&p["atom"], "arguments")
            .iter()
            .any(|a| s(a, "role") != slot && argument(&target["atom"], s(a, "role")).is_none())
        {
            self.context_pending(c, r, out, "request_context_target");
            return;
        }
        let sq = self.claim_qualifiers(c, &p);
        let qq = self.request_qualifiers(r);
        let (qualifiers, missing) = self.context(self.source, &sq, self.question, &qq, false);
        let mut partial_kind = None;
        if nominal && qualifiers != Relation::Same {
            // Only the declared maximum extent may remain unestablished; other
            // conditions, source conditions and the observation basis stay intact.
            let is_max = |q: &&Value| {
                s(q, "kind") == "configuration"
                    && q["body"]["key"] == "quantity_extent"
                    && q["body"]["value"]["value"] == "maximum"
            };
            let max_ids: Vec<_> = qq
                .iter()
                .filter(|id| {
                    id.as_str()
                        .and_then(|id| self.qualifier(self.question, id))
                        .as_ref()
                        .is_some_and(is_max)
                })
                .cloned()
                .collect();
            let rest: Vec<_> = qq
                .iter()
                .filter(|id| !max_ids.contains(id))
                .cloned()
                .collect();
            if s(
                argument(&p["atom"], "attribute").unwrap_or(&Value::Null),
                "id",
            ) == "autonomy"
                && s(&p, "basis") == "observation"
                && max_ids.len() == 1
                && self
                    .context(self.source, &sq, self.question, &rest, false)
                    .0
                    == Relation::Same
            {
                partial_kind = Some((
                    "observed_same_attribute_extent_pending",
                    "requested_extent_not_established",
                ));
            } else {
                self.context_pending(
                    c,
                    r,
                    out,
                    missing
                        .first()
                        .map(String::as_str)
                        .unwrap_or("request_context_qualifiers"),
                );
                return;
            }
        } else if qualifiers != Relation::Same {
            self.context_pending(c, r, out, "request_context_qualifiers");
            return;
        }
        if s(r, "required_answer_extent") == "all_values" {
            self.context_pending(c, r, out, "answer_extent");
            return;
        }
        if !self.premises(r).is_empty() {
            self.context_pending(c, r, out, "premise");
            return;
        }
        if open {
            if !self.context_class_filter_equal(
                &c["body"]["open_role"]["class_filter"],
                &r["class_filter"],
            ) {
                self.context_pending(c, r, out, "class_filter");
                return;
            }
            partial_kind = Some((
                "normative_open_pattern",
                "individual_not_supplied_by_pattern",
            ));
        }
        let terms: Vec<_> = argument(&p["atom"], slot).cloned().into_iter().collect();
        if terms.iter().any(|t| {
            matches!(s(t, "kind"), "ref" | "described_ref")
                && !self.context_clean_reference(self.source, id(t))
        }) {
            self.context_pending(c, r, out, "identity");
            return;
        }
        let filters = self.filters(r, &p, &terms);
        if (!filters.is_empty() && !(open && filters == vec!["class_filter".to_owned()]))
            && !(partial_kind.is_some() && filters.iter().all(|f| f == "basis"))
        {
            self.context_pending(
                c,
                r,
                out,
                filters.first().map(String::as_str).unwrap_or("answer_slot"),
            );
            return;
        }
        if partial_kind.is_none() && terms.is_empty() {
            self.context_pending(c, r, out, "answer_slot");
            return;
        }
        // Debit transport and provenance before materializing them.
        let size =
            |v: &Value| super::super::context_budget::serialized_bytes(v).unwrap_or(usize::MAX);
        let bytes = size(c)
            .saturating_add(size(r))
            .saturating_add(size(self.source))
            .saturating_add(size(self.question))
            .saturating_mul(3);
        if !super::super::context_budget::charge(1, bytes) {
            self.context_pending(c, r, out, "request_context_budget");
            return;
        }
        map.sort_by_key(Value::to_string);
        map.dedup();
        let mut w = self.witness(
            c,
            r,
            &p,
            if partial_kind.is_some() {
                "partial_request_context_v1"
            } else {
                "request_context_v1"
            },
            if partial_kind.is_some() {
                vec![]
            } else {
                terms
            },
            vec![],
            &[],
        );
        self.context_provenance(&mut w, c, r, &map);
        if let Some((kind, answer)) = partial_kind {
            w["context_transport"] = self.context_payload(c, r, &map, kind, answer);
            w["answer_terms"] = json!([]);
            w["answer_propositions"] = json!([]);
            w["bindings"] = json!([]);
            w["missing_constraints"] = json!([answer]);
            out.partial(w);
        } else {
            out.publish(w, false);
        }
    }
    pub(super) fn request_context_request(&self, r: &Value) -> Option<Value> {
        let present = super::super::request_context_profile::enabled(self.vocabulary);
        if !present {
            return None;
        }
        if !carries(r) && self.context_dependency_carrier(r) {
            return Some(
                json!({"request_id":id(r),"support":[],"refutation":[],"partial":[],"unresolved":[{"request_id":id(r),"claim_ids":[],"state":"unresolved","reason_codes":["request_context_dependency"],"request_constraint":r}],"conflicts":[],"missing_constraints":["request_context_dependency"],"coverage":"none"}),
            );
        }
        let source_has = rows(self.source, "claims").iter().any(carries);
        if !carries(r) && !source_has {
            return None;
        }
        if !carries(r) {
            // Keep independent legacy facts, but remove conditioned owners from every
            // legacy event/guard/branch path before any fallback can inspect them.
            if !super::super::context_budget::charge(
                1,
                super::super::context_budget::serialized_bytes(self.source).unwrap_or(usize::MAX),
            ) {
                return Some(
                    json!({"request_id":id(r),"support":[],"refutation":[],"partial":[],"unresolved":[],"conflicts":[],"missing_constraints":["request_context_limit"],"coverage":"none"}),
                );
            }
            let mut filtered = self.source.clone();
            filtered["claims"] = json!(
                rows(self.source, "claims")
                    .iter()
                    .filter(|c| !carries(c))
                    .collect::<Vec<_>>()
            );
            let engine = Engine {
                possessive_source: self.possessive_source,
                lexical_source: self.lexical_source,
                possessive_question: self.possessive_question,
                source: &filtered,
                question: self.question,
                source_report: self.source_report,
                pending_claims: self.pending_claims,
                question_report: self.question_report,
                vocabulary: self.vocabulary,
                scope: self.scope,
                context_v2: self.context_v2,
            };
            return Some(engine.request_legacy(r));
        }
        let mut out = Evidence::default();
        for c in self.pending_claims {
            self.context_pending(c, r, &mut out, "request_context_unsupported");
        }
        for c in rows(self.source, "claims") {
            self.contextual_claim(c, r, &mut out);
        }
        self.constrain_answer_pattern(r, &mut out);
        if out.support.len() > 1 {
            let first = &out.support[0]["answer_terms"];
            if out
                .support
                .iter()
                .skip(1)
                .any(|w| semantic(&w["answer_terms"]) != semantic(first))
            {
                out.conflicts.push(json!({"kind":"potential","reason":"request_context_multiple_answers","support_witness_ids":out.support.iter().map(|w|w["id"].clone()).collect::<Vec<_>>(),"refutation_witness_ids":[]}));
            }
        }
        self.coverage_candidates(r, &mut out);
        Some(row(out, r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    fn hash(v: &Value) -> String {
        Sha256::digest(serde_json::to_vec(v).unwrap())
            .iter()
            .map(|x| format!("{x:02x}"))
            .collect()
    }
    fn nominal() -> Value {
        serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/request-context-contract-v1/control-preparation-v2/proposed-nominal-positive.json"))).unwrap()
    }
    fn run(mut input: Value) -> Value {
        let vocab = input["vocabulary"].clone();
        let h = hash(&vocab);
        for side in ["source", "question"] {
            input[side]["raw"]["vocabulary_sha256"] = json!(h);
        }
        let a = super::super::super::validate(
            &input["source"]["raw"],
            s(&input["source"], "text"),
            &vocab,
        );
        let b = super::super::super::validate(
            &input["question"]["raw"],
            s(&input["question"], "text"),
            &vocab,
        );
        assert_eq!(a.report["status"], "complete", "source {}", a.report);
        assert_eq!(b.report["status"], "complete", "question {}", b.report);
        classify_pair_context(
            &a,
            &b,
            &vocab,
            &json!({"namespace":"request-context-prospective-controls-v1","configuration":[],"time_selector":null,"closed_world":false}),
        )
    }
    #[test]
    fn nominal_positive_retains_provenance_and_removal_blocks_fallback() {
        let original = nominal();
        let good = run(original.clone());
        assert_eq!(good["complete_answer"], true, "{good}");
        assert_eq!(
            good["requests"][0]["support"][0]["request_context_witness"]["source_owner"]["context_bindings"],
            original["source"]["raw"]["claims"][0]["context_bindings"]
        );
        for side in ["source", "question"] {
            let mut changed = original.clone();
            let table = if side == "source" {
                "claims"
            } else {
                "requests"
            };
            changed[side]["raw"][table][0]
                .as_object_mut()
                .unwrap()
                .remove("context_bindings");
            let result = run(changed);
            assert_eq!(result["complete_answer"], false, "{side}: {result}");
            assert_eq!(result["selected_claim_ids"], json!([]));
        }
    }
    #[test]
    fn nominal_unknown_anchor_and_class_homonym_never_answer() {
        let input = nominal();
        for field in ["name", "declared_class"] {
            let mut changed = input.clone();
            // A distinct valid span changes the anchor/class rather than relying
            // on an invalid quotation to obtain the rejection.
            let span = changed["source"]["raw"]["references"][0][field].clone();
            changed["source"]["raw"]["references"][1][field] = span;
            let result = run(changed);
            assert_eq!(result["complete_answer"], false, "{result}");
            assert_eq!(result["selected_claim_ids"], json!([]));
        }
    }
    #[test]
    fn observed_maximum_context_is_partial_with_no_answer_leak() {
        let mut input = nominal();
        input["source"]["raw"]["claims"][0]["body"]["value"]["basis"] = json!("observation");
        input["source"]["raw"]["claims"][0]["qualifier_ids"] = json!([]);
        input["source"]["raw"]["claims"][0]["body"]["value"]["qualifier_ids"] = json!([]);
        input["source"]["raw"]["dependencies"]
            .as_array_mut()
            .unwrap()
            .retain(|d| d["requires"]["kind"] != "qualifier");
        let result = run(input);
        assert_eq!(result["complete_answer"], false, "{result}");
        assert_eq!(result["requests"][0]["coverage"], "partial", "{result}");
        let w = &result["requests"][0]["partial"][0];
        for field in ["answer_terms", "answer_propositions", "bindings"] {
            assert_eq!(w[field], json!([]), "{w}");
        }
        assert_eq!(w["context_transport"]["requested_value_inferred"], false);
        assert_eq!(
            w["context_transport"]["source_body"]["value"]["basis"],
            "observation"
        );
    }
    fn normative(open: bool, owner_slot: bool) -> Value {
        let mut input = nominal();
        input["vocabulary"]["predicate_rows"] = json!([{"id":"action","roles":{"actor":"ref","action_kind":"action_id","object":"ref"},"distributive":true}]);
        input["vocabulary"]["fixed_action_rows"] = json!(["withdraw", "enroll"]);
        input["vocabulary"]["request_context_profile"]["features"] =
            json!(["normative_action_relation", "normative_open_pattern"]);
        let text = "Tutor T must withdraw Person P for Person P enroll Course B.";
        let quote = |part: &str| {
            let start = text.find(part).unwrap();
            json!({"start_byte":start,"end_byte":start+part.len(),"quote":part})
        };
        let reference = |rid: &str, name: &str, class: &str| json!({"id":rid,"scope_id":"root","mention":quote(name),"name":quote(name),"declared_class":quote(class),"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null});
        for side in ["source", "question"] {
            input[side]["text"] = json!(text);
            let d = &mut input[side]["raw"];
            d["text_sha256"] = json!(
                Sha256::digest(text.as_bytes())
                    .iter()
                    .map(|x| format!("{x:02x}"))
                    .collect::<String>()
            );
            d["references"] = json!([
                reference("T", "Tutor T", "Tutor"),
                reference("P", "Person P", "Person"),
                reference("B", "Course B", "Course")
            ]);
            d["qualifiers"] = json!([]);
            d["dependencies"] = json!([]);
            d["coverage"] =
                json!([{"span":quote(text),"state":"represented","owner":null,"issue_id":null}]);
            let mut args = vec![
                json!({"role":"action_kind","value":{"kind":"symbol","id":"withdraw","evidence":[quote("withdraw")]}}),
                json!({"role":"object","value":{"kind":"ref","id":"P"}}),
            ];
            if side == "source" && !open {
                args.push(json!({"role":"actor","value":{"kind":"ref","id":"T"}}));
            }
            let atom = json!({"predicate":"action","arguments":args,"polarity":"positive","evidence":[quote(text)]});
            let target =
                json!({"atom":atom,"judgment":"required","status":"state","qualifier_ids":[]});
            let filter = json!({"kind":"lexical","class_quote":quote("Tutor")});
            let mut owner = if side == "source" {
                json!({"id":"c","scope_id":"root","qualifier_ids":[],"evidence":[quote(text)],"body":if open {json!({"kind":"normative_open_pattern","target":atom,"open_role":{"role":"actor","class_filter":filter,"evidence":[quote("Tutor")]},"judgment":"required","status":"state","basis":"normative","normative_scope":"B"})} else {let mut p=target.clone();p["basis"]=json!("normative");p["normative_scope"]=json!("B");json!({"kind":"proposition","value":p})}})
            } else {
                json!({"id":"q","scope_id":"root","mode":"answer","target":target,"slot":"actor","class_filter":filter,"unit":null,"basis_requirement":{"kind":"exact","value":"normative"},"required_answer_extent":"one_value","normative_scope_filter":"B","qualifier_ids":[],"premise_ids":[],"evidence":[quote(text)]})
            };
            let actor = if owner_slot && (side == "question" || open) {
                json!({"kind":"owner_slot","role":"actor","evidence":[quote(text)]})
            } else {
                json!({"kind":"reference","value":{"kind":"ref","id":if owner_slot {"T"}else{"P"}},"evidence":[quote(text)]})
            };
            owner["context_bindings"] = json!([{"id":"ctx","kind":"normative_action_relation","relation":{"kind":"purpose","evidence":[quote(text)]},"action":{"action_kind":{"kind":"symbol","id":"enroll","evidence":[quote("enroll")]},"actor":actor,"destination":{"kind":"ref","id":"B"},"evidence":[quote(text)]},"evidence":[quote(text)]}]);
            let table = if side == "source" {
                "claims"
            } else {
                "requests"
            };
            d[table] = json!([owner]);
            let owner_kind = if side == "source" { "claim" } else { "request" };
            let owner_id = if side == "source" { "c" } else { "q" };
            let mut deps = vec![];
            for rid in ["T", "P", "B"] {
                deps.push(json!({"consumer":{"kind":"reference","id":rid},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
                deps.push(json!({"consumer":{"kind":owner_kind,"id":owner_id},"requires":{"kind":"reference","id":rid},"role":"identity"}));
            }
            deps.push(json!({"consumer":{"kind":owner_kind,"id":owner_id},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
            deps.push(json!({"consumer":{"kind":owner_kind,"id":owner_id},"requires":{"kind":"reference","id":"B"},"role":"normative_target"}));
            d["dependencies"] = json!(deps);
        }
        input
    }
    #[test]
    fn normative_actor_control_class_and_unknown_are_distinct() {
        for owner_slot in [false, true] {
            let input = normative(false, owner_slot);
            let result = run(input.clone());
            assert_eq!(result["complete_answer"], true, "{result}");
            let mut changed = input.clone();
            changed["source"]["raw"]["claims"][0]["context_bindings"][0]["action"]["actor"]["value"]
                ["id"] = json!(if owner_slot { "P" } else { "T" });
            let result = run(changed);
            assert_eq!(result["selected_claim_ids"], json!([]), "{result}");
            let mut changed = input.clone();
            changed["source"]["raw"]["references"][0]["declared_class"] = Value::Null;
            let result = run(changed);
            assert_eq!(result["selected_claim_ids"], json!([]), "{result}");
            let mut changed = input;
            changed["source"]["raw"]["claims"][0]["context_bindings"][0]["action"]["actor"] = json!({"kind":"unresolved","reason":"unspecified","evidence":changed["source"]["raw"]["claims"][0]["evidence"]});
            let result = run(changed);
            assert_eq!(result["selected_claim_ids"], json!([]), "{result}");
        }
    }
    #[test]
    fn normative_open_pattern_preserves_partial_and_never_fills_actor() {
        for owner_slot in [false, true] {
            let result = run(normative(true, owner_slot));
            assert_eq!(result["requests"][0]["coverage"], "partial", "{result}");
            let w = &result["requests"][0]["partial"][0];
            for field in ["answer_terms", "answer_propositions", "bindings"] {
                assert_eq!(w[field], json!([]));
            }
            assert_eq!(
                w["context_transport"]["source_body"]["kind"],
                "normative_open_pattern"
            );
        }
    }
    #[test]
    fn normative_owner_slot_alias_requires_same_and_keeps_private_binding_local() {
        let mut input = normative(false, true);
        let mut alias = input["source"]["raw"]["references"][0].clone();
        alias["id"] = json!("alias-T");
        input["source"]["raw"]["references"]
            .as_array_mut()
            .unwrap()
            .push(alias);
        input["source"]["raw"]["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":"alias-T"},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
        input["source"]["raw"]["claims"][0]["context_bindings"][0]["action"]["actor"]["value"]["id"] =
            json!("alias-T");
        let evidence = input["source"]["raw"]["claims"][0]["evidence"].clone();
        input["source"]["raw"]["identity"] =
            json!([{"kind":"same","left":"T","right":"alias-T","evidence":evidence}]);
        let positive = run(input.clone());
        assert_eq!(positive["complete_answer"], true, "{positive}");
        let control = &positive["requests"][0]["support"][0]["request_context_witness"]["control"];
        assert_eq!(control["substitution"], "owner_slot_to_source_actor");
        assert_eq!(control["source_actor"]["id"], "T");
        input["source"]["raw"]["identity"][0]["kind"] = json!("different");
        let negative = run(input);
        assert_eq!(negative["selected_claim_ids"], json!([]), "{negative}");
    }
    #[test]
    fn normative_reference_pattern_cannot_bypass_class_evidence() {
        let mut input = normative(false, false);
        let mut wanted = input["question"]["raw"]["references"][0].clone();
        wanted["id"] = json!("wanted");
        wanted["name"] = Value::Null;
        wanted["mention"] = wanted["declared_class"].clone();
        input["question"]["raw"]["references"]
            .as_array_mut()
            .unwrap()
            .push(wanted);
        input["question"]["raw"]["dependencies"].as_array_mut().unwrap().extend([
            json!({"consumer":{"kind":"reference","id":"wanted"},"requires":{"kind":"scope","id":"root"},"role":"binding"}),
            json!({"consumer":{"kind":"request","id":"q"},"requires":{"kind":"reference","id":"wanted"},"role":"identity"})
        ]);
        input["question"]["raw"]["requests"][0]["class_filter"] =
            json!({"kind":"reference_pattern","reference_id":"wanted"});
        let good = run(input.clone());
        assert_eq!(good["complete_answer"], true, "{good}");
        for actual_class in [
            Value::Null,
            input["source"]["raw"]["references"][1]["declared_class"].clone(),
        ] {
            let mut changed = input.clone();
            changed["source"]["raw"]["references"][0]["declared_class"] = actual_class;
            let result = run(changed);
            assert_eq!(result["complete_answer"], false, "{result}");
            assert_eq!(result["selected_claim_ids"], json!([]), "{result}");
            let candidate = &result["requests"][0]["unresolved"][0];
            assert_eq!(
                candidate["reason_codes"],
                json!(["answer_reference_pattern"])
            );
            assert_eq!(
                candidate["request_context_witness"]["source_owner"]["context_bindings"],
                input["source"]["raw"]["claims"][0]["context_bindings"]
            );
        }
    }
}
