//! Documentary graph checks with an operation-wide opt-in budget.
use super::*;

fn ancestor(scopes: &BTreeMap<String, &Value>, from: &str, to: &str) -> bool {
    let mut next = Some(from);
    let mut seen = BTreeSet::new();
    while let Some(s) = next {
        if s == to {
            return true;
        }
        if !seen.insert(s) {
            return false;
        }
        next = scopes.get(s).and_then(|v| v["parent_id"].as_str());
    }
    false
}
pub(super) fn graph(c: &mut Check<'_>, raw: &Value) {
    let mut all = BTreeMap::new();
    for (table, kind) in [
        ("scopes", "scope"),
        ("references", "reference"),
        ("claims", "claim"),
        ("requests", "request"),
        ("qualifiers", "qualifier"),
    ] {
        for (i, row) in charged(
            array(&raw[table])
                .iter()
                .take_while(|_| budget_visit())
                .enumerate(),
        ) {
            if let Some(id) = row["id"].as_str() {
                if all.insert(id.to_owned(), (kind, row)).is_some() {
                    c.issue("invalid_shape", &format!("/{table}/{i}/id"));
                }
            }
        }
    }
    let scopes: BTreeMap<_, _> = all
        .iter()
        .take_while(|_| budget_visit())
        .filter(|(_, x)| x.0 == "scope")
        .map(|(id, x)| (id.clone(), x.1))
        .collect();
    let roots: Vec<_> = scopes
        .values()
        .take_while(|_| budget_visit())
        .filter(|s| s["parent_id"].is_null())
        .collect();
    if roots.len() != 1 || roots.first().is_some_and(|s| s["kind"] != "document") {
        c.issue("invalid_shape", "/scopes");
    }
    for (id, s) in charged(&scopes) {
        let p = format!("/scope/{}", path_id(id));
        let mut seen = BTreeSet::new();
        let mut cur = Some(id.as_str());
        let mut forall = 0;
        let mut quantifiers = Vec::new();
        let mut after_forall_exists = false;
        let mut second_forall = false;
        while let Some(x) = cur {
            if !budget_visit() {
                return;
            }
            if !seen.insert(x) {
                c.issue("invalid_shape", &p);
                break;
            }
            let Some(row) = scopes.get(x) else {
                c.issue("dangling_id", &p);
                break;
            };
            if let Some(q) = row["binding"]["quantifier"].as_str() {
                if quantifiers.last() != Some(&q) {
                    quantifiers.push(q);
                }
            }
            if row["binding"]["quantifier"] == "forall" {
                forall += 1;
                if after_forall_exists {
                    second_forall = true
                }
            } else if row["binding"]["quantifier"] == "exists" && forall > 0 {
                after_forall_exists = true
            }
            cur = row["parent_id"].as_str();
        }
        if forall > 1 || second_forall || quantifiers.len() > 2 {
            c.issue("unsupported_combination", &p);
        }
        if (s["kind"] == "binder") != !s["binding"].is_null()
            || (s["kind"] == "branch") != !s["branch_id"].is_null()
        {
            c.issue("invalid_shape", &p);
        }
        let b = &s["binding"];
        if !b.is_null() {
            let variable = b["variable"].as_str().unwrap_or("");
            if !all.get(variable).is_some_and(|(k, r)| {
                *k == "reference"
                    && r["binding_scope_id"] == id.as_str()
                    && r["scope_id"] == id.as_str()
            }) {
                c.issue("dangling_id", &p);
            }
            let d = b["domain_ref"].as_str().and_then(|r| all.get(r));
            if b["quantifier"] == "forall"
                && !d.is_some_and(|(k, r)| {
                    *k == "reference"
                        && (r["denotation"] == "group" || r["denotation"] == "class")
                        && (!r["declared_class"].is_null() || !b["cardinality"].is_null())
                })
            {
                c.issue("unsupported_combination", &p);
            }
            if !b["cardinality"].is_null()
                && b["cardinality"]["domain"] != b["domain_ref"]
                && b["cardinality"]["domain"] != b["variable"]
            {
                c.issue("invalid_shape", &p);
            }
            if b["witness_relation"] == "same_across_domain"
                && !scopes
                    .iter()
                    .take_while(|_| budget_visit())
                    .any(|(other, row)| {
                        other != id
                            && row["binding"]["quantifier"] == "forall"
                            && ancestor(&scopes, other, id)
                    })
            {
                c.issue("unsupported_combination", &p);
            }
            if b["witness_relation"] == "distinct_across_domain"
                && (b["quantifier"] != "exists"
                    || !scopes
                        .iter()
                        .take_while(|_| budget_visit())
                        .any(|(other, row)| {
                            other != id
                                && row["binding"]["quantifier"] == "forall"
                                && ancestor(&scopes, id, other)
                        }))
            {
                c.issue("unsupported_combination", &p);
            }
            if b["witness_relation"] == "distinct_across_domain" && array(&b["evidence"]).is_empty()
            {
                c.issue("invalid_shape", &p);
            }
            if b["witness_relation"] == "same_across_domain" && b["quantifier"] != "exists" {
                c.issue("unsupported_combination", &p);
            }
        }
    }
    for (id, (kind, row)) in charged(&all) {
        let p = format!("/{kind}/{}", path_id(id));
        if *kind == "request" && row["mode"] == "configuration_slot" {
            for qid in array(&row["qualifier_ids"])
                .iter()
                .take_while(|_| budget_visit())
                .chain(array(&row["target"]["qualifier_ids"]))
            {
                if qid
                    .as_str()
                    .and_then(|id| all.get(id))
                    .is_some_and(|(kind, q)| {
                        *kind == "qualifier"
                            && q["kind"] == "configuration"
                            && q["body"]["key"] == row["slot"]
                    })
                {
                    c.issue("invalid_shape", &p);
                }
            }
        }
        let scope = if *kind == "scope" {
            Some(id.as_str())
        } else {
            row["scope_id"].as_str()
        };
        if *kind != "scope" && !scope.is_some_and(|s| scopes.contains_key(s)) {
            c.issue("dangling_id", &p);
        }
        let mut used = BTreeSet::new();
        refs(row, &mut used);
        let mut quals = Vec::new();
        collect(row, "qualifier_ids", &mut quals);
        // qualifier_ids are arrays, collected separately to retain their exact IDs.
        fn qids(v: &Value, out: &mut Vec<String>) {
            if let Some(o) = v.as_object() {
                for (k, x) in charged(o) {
                    if k == "qualifier_ids" {
                        out.extend(
                            array(x)
                                .iter()
                                .take_while(|_| budget_visit())
                                .filter_map(Value::as_str)
                                .map(str::to_owned),
                        );
                    }
                    qids(x, out)
                }
            } else if let Some(a) = v.as_array() {
                for x in charged(a) {
                    qids(x, out)
                }
            }
        }
        qids(row, &mut quals);
        for r in used
            .iter()
            .take_while(|_| budget_visit())
            .chain(quals.iter().take_while(|_| budget_visit()))
        {
            let Some((rk, target)) = all.get(r) else {
                c.issue("dangling_id", &p);
                continue;
            };
            if used.contains(r) && *rk != "reference" {
                c.issue("dangling_id", &p);
            }
            if quals.contains(r) && *rk != "qualifier" {
                c.issue("dangling_id", &p);
            }
            if let (Some(s), Some(t)) = (scope, target["scope_id"].as_str()) {
                if !ancestor(&scopes, s, t) {
                    c.issue("scope_escape", &p);
                }
            }
        }
        if *kind == "request" && row["class_filter"]["kind"] == "reference_pattern" {
            let filter_id = row["class_filter"]["reference_id"].as_str().unwrap_or("");
            let pattern = all
                .get(filter_id)
                .filter(|(kind, _)| *kind == "reference")
                .map(|(_, r)| *r);
            let valid = raw["kind"] == "question"
                && row["mode"] == "answer"
                && pattern.is_some_and(|r| {
                    r["name"].is_null()
                        && r["denotation"] == "entity"
                        && r["binding_scope_id"].is_null()
                        && r["anaphora"].is_null()
                        && (!r["declared_class"].is_null() || !r["semantic_type"].is_null())
                })
                && array(&row["target"]["atom"]["arguments"])
                    .iter()
                    .take_while(|_| budget_visit())
                    .all(|a| a["role"] != row["slot"])
                && array(&c.vocab["predicate_rows"])
                    .iter()
                    .take_while(|_| budget_visit())
                    .any(|pred| {
                        pred["id"] == row["target"]["atom"]["predicate"]
                            && pred["roles"][row["slot"].as_str().unwrap_or("")] == "ref"
                    });
            if !valid {
                c.issue("invalid_shape", &format!("{p}/class_filter"));
            }
            let mut target_refs = BTreeSet::new();
            refs(&row["target"], &mut target_refs);
            if target_refs.contains(filter_id)
                || array(&raw["identity"])
                    .iter()
                    .take_while(|_| budget_visit())
                    .any(|edge| edge["left"] == filter_id || edge["right"] == filter_id)
            {
                c.issue("invalid_shape", &format!("{p}/class_filter"));
            }
            // The requested variable may constrain this request, never assert an existing answer.
            for claim in charged(array(&raw["claims"])) {
                let mut claim_refs = BTreeSet::new();
                refs(claim, &mut claim_refs);
                if claim_refs.contains(filter_id) {
                    c.issue("invalid_shape", &format!("{p}/class_filter"));
                }
            }
        }
        if *kind == "reference"
            && !row["binding_scope_id"].is_null()
            && !row["binding_scope_id"]
                .as_str()
                .and_then(|s| scopes.get(s))
                .is_some_and(|b| {
                    b["kind"] == "binder"
                        && b["binding"]["variable"] == id.as_str()
                        && b["id"] == row["scope_id"]
                })
        {
            c.issue("dangling_id", &p);
        }
        for key in ["premise_ids", "rule_targets"] {
            let values = if key == "rule_targets" {
                &row["body"][key]
            } else {
                &row[key]
            };
            for target in charged(array(values)) {
                if !target
                    .as_str()
                    .and_then(|id| all.get(id))
                    .is_some_and(|x| x.0 == "claim")
                {
                    c.issue("dangling_id", &p);
                }
                if !array(&raw["dependencies"])
                    .iter()
                    .take_while(|_| budget_visit())
                    .any(|d| {
                        d["consumer"]["kind"] == *kind
                            && d["consumer"]["id"] == id.as_str()
                            && d["requires"]["kind"] == "claim"
                            && d["requires"]["id"] == *target
                    })
                {
                    c.issue("invalid_shape", &format!("{p}/dependencies/{key}"));
                }
            }
        }
        if row["anaphora"]["status"] == "asserted"
            && (array(&row["anaphora"]["antecedent_ids"]).len() != 1
                || array(&row["anaphora"]["evidence"]).is_empty())
        {
            c.issue("ambiguous_identity", &p);
        }
        if row["anaphora"]["status"] == "unresolved" {
            c.issue("unresolved_target", &p);
        }
        if *kind == "qualifier" && row["resolution"] == "unknown" {
            c.issue("unknown_qualifier", &p);
        }
        if matches!(*kind, "claim" | "request")
            && scope.is_some_and(|scope| {
                scopes.iter().take_while(|_| budget_visit()).any(|(id, s)| {
                    s["binding"]["quantifier"] == "forall" && ancestor(&scopes, scope, id)
                })
            })
        {
            let mut predicates = Vec::new();
            collect(row, "predicate", &mut predicates);
            for pred in charged(predicates) {
                if !array(&c.vocab["predicate_rows"])
                    .iter()
                    .take_while(|_| budget_visit())
                    .any(|r| r["id"] == pred && r["distributive"] == true)
                {
                    c.issue("unsupported_combination", &p);
                }
            }
        }
        if *kind == "claim" || *kind == "request" || *kind == "reference" {
            for (required, role) in used
                .iter()
                .take_while(|_| budget_visit())
                .map(|r| (r, "reference"))
                .chain(
                    quals
                        .iter()
                        .take_while(|_| budget_visit())
                        .map(|r| (r, "qualifier")),
                )
                .chain(
                    scope
                        .into_iter()
                        .map(|s| (all.get_key_value(s).map(|x| x.0).unwrap_or(id), "scope")),
                )
            {
                if required == id {
                    continue;
                }
                if super::super::request_context_profile::enabled(c.vocab)
                    && request_context::has_carrier(row)
                    && (role == "scope"
                        || role == "reference"
                            && request_context::implicit_refs(row).contains(required))
                {
                    continue;
                }
                if !array(&raw["dependencies"])
                    .iter()
                    .take_while(|_| budget_visit())
                    .any(|d| {
                        d["consumer"]["kind"] == *kind
                            && d["consumer"]["id"] == id.as_str()
                            && d["requires"]["kind"] == role
                            && d["requires"]["id"] == required.as_str()
                    })
                {
                    c.issue(
                        "invalid_shape",
                        &format!("{p}/dependencies/{role}/{required}"),
                    );
                }
            }
        }
    }
    for (i, d) in charged(
        array(&raw["dependencies"])
            .iter()
            .take_while(|_| budget_visit())
            .enumerate(),
    ) {
        for (key, allowed) in [
            ("consumer", &["claim", "request", "reference"][..]),
            (
                "requires",
                &["claim", "reference", "qualifier", "scope"][..],
            ),
        ] {
            let ep = &d[key];
            let bounded_qualifier_consumer = key == "consumer"
                && ep["kind"] == "qualifier"
                && super::super::request_context_profile::permits(
                    c.vocab,
                    "relative_nonoccurrence_bound",
                )
                && ep["id"]
                    .as_str()
                    .and_then(|id| all.get(id))
                    .is_some_and(|(kind, row)| {
                        *kind == "qualifier" && row["kind"] == "relative_nonoccurrence_bound"
                    });
            if (!allowed.contains(&ep["kind"].as_str().unwrap_or(""))
                && !bounded_qualifier_consumer)
                || !ep["id"]
                    .as_str()
                    .and_then(|s| all.get(s))
                    .is_some_and(|r| Some(r.0) == ep["kind"].as_str())
            {
                c.issue("dangling_id", &format!("/dependencies/{i}/{key}"));
            }
        }
    }
    // Directed dependency cycles cannot resolve themselves.
    fn cyclic<'a>(
        id: &'a str,
        deps: &'a [Value],
        path: &mut BTreeSet<&'a str>,
        done: &mut BTreeSet<&'a str>,
    ) -> bool {
        if !budget_visit() {
            return false;
        }
        if path.len() > MAX_DEPTH || path.contains(id) {
            return true;
        }
        if done.contains(id) {
            return false;
        }
        path.insert(id);
        for d in deps
            .iter()
            .take_while(|_| budget_visit())
            .filter(|d| d["consumer"]["id"] == id)
        {
            if let Some(to) = d["requires"]["id"].as_str() {
                if cyclic(to, deps, path, done) {
                    return true;
                }
            }
        }
        path.remove(id);
        done.insert(id);
        false
    }
    let mut done = BTreeSet::new();
    for id in all.keys().take_while(|_| budget_visit()) {
        if cyclic(
            id,
            array(&raw["dependencies"]),
            &mut BTreeSet::new(),
            &mut done,
        ) {
            c.issue("invalid_shape", "/dependencies");
            break;
        }
    }
    // same closure is used only to detect contradictions, never to merge references.
    let mut same: BTreeMap<String, BTreeSet<String>> = all
        .iter()
        .take_while(|_| budget_visit())
        .filter(|(_, x)| x.0 == "reference")
        .map(|(id, _)| (id.clone(), BTreeSet::from([id.clone()])))
        .collect();
    let identity_bytes =
        super::super::context_budget::serialized_bytes(&raw["identity"]).unwrap_or(usize::MAX);
    if !super::super::context_budget::charge(array(&raw["identity"]).len(), identity_bytes) {
        return;
    }
    let mut identity_edges = array(&raw["identity"]).to_vec();
    for reference in charged(array(&raw["references"])) {
        if reference["anaphora"]["status"] == "asserted"
            && array(&reference["anaphora"]["antecedent_ids"]).len() == 1
        {
            identity_edges.push(json!({"kind":"same","left":reference["id"],"right":reference["anaphora"]["antecedent_ids"][0]}));
        }
    }
    for edge in charged(&identity_edges) {
        for k in ["left", "right"] {
            if !edge[k].as_str().is_some_and(|s| same.contains_key(s)) {
                c.issue("dangling_id", "/identity");
            }
        }
        if let (Some(l), Some(r)) = (
            edge["left"].as_str().and_then(|id| all.get(id)),
            edge["right"].as_str().and_then(|id| all.get(id)),
        ) {
            if let (Some(ls), Some(rs)) = (l.1["scope_id"].as_str(), r.1["scope_id"].as_str()) {
                if !ancestor(&scopes, ls, rs) && !ancestor(&scopes, rs, ls) {
                    c.issue("scope_escape", "/identity");
                }
            }
        }
        if edge["kind"] == "same" {
            let l = edge["left"].as_str().unwrap_or("");
            let r = edge["right"].as_str().unwrap_or("");
            let mut union = BTreeSet::new();
            for value in same
                .get(l)
                .into_iter()
                .flatten()
                .chain(same.get(r).into_iter().flatten())
            {
                // Charge before each String clone, including entries later deduplicated.
                if !super::super::context_budget::charge(1, value.len()) {
                    return;
                }
                union.insert(value.clone());
            }
            let union_bytes = union
                .iter()
                .take_while(|_| budget_visit())
                .try_fold(0usize, |n, id| n.checked_add(id.len()))
                .unwrap_or(usize::MAX);
            for id in charged(&union) {
                // Cloning a component copies every member, not only the map key.
                let Some(bytes) = union_bytes.checked_add(id.len()) else {
                    super::super::context_budget::charge(usize::MAX, usize::MAX);
                    return;
                };
                if !super::super::context_budget::charge(union.len().saturating_add(1), bytes) {
                    return;
                }
                same.insert(id.clone(), union.clone());
            }
        }
    }
    for edge in charged(&identity_edges) {
        if edge["kind"] == "different"
            && edge["left"]
                .as_str()
                .and_then(|s| same.get(s))
                .is_some_and(|a| edge["right"].as_str().is_some_and(|r| a.contains(r)))
        {
            c.issue("ambiguous_identity", "/identity");
            for key in ["left", "right"] {
                if let Some(id) = edge[key].as_str() {
                    c.issue("ambiguous_identity", &format!("/reference/{}", path_id(id)));
                }
            }
        }
    }
    for cov in charged(array(&raw["coverage"])) {
        if cov["state"] != "represented" {
            c.issue(
                if cov["state"] == "unsupported" {
                    "unsupported_combination"
                } else {
                    "unresolved_target"
                },
                "/coverage",
            );
        }
        if !cov["owner"].is_null()
            && !cov["owner"]["id"]
                .as_str()
                .is_some_and(|id| all.contains_key(id))
        {
            c.issue("dangling_id", "/coverage/owner");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(reference_count: usize) -> (Value, Value) {
        let vocabulary = json!({"version":"budget-test","predicate_rows":[{"id":"owns","roles":{"owner":"ref","object":"ref"},"distributive":true}],"fixed_attribute_rows":[],"fixed_action_rows":[],"semantic_type_rows":[],"relevance_links":[],"request_context_profile":{"version":"documentary_request_context_v1","features":["nominal_during"]}});
        let evidence = json!({"start_byte":0,"end_byte":1,"quote":"a"});
        let mut references = vec![];
        let mut dependencies = vec![];
        for index in 0..reference_count {
            let id = format!("ref-{index:04}");
            references.push(json!({"id":id,"scope_id":"root","mention":evidence,"name":evidence,"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null}));
            dependencies.push(json!({"consumer":{"kind":"reference","id":id},"requires":{"kind":"scope","id":"root"},"role":"binding"}));
        }
        let document = json!({"version":"documentary_evidence_v3","kind":"source","text_sha256":digest(b"a"),"vocabulary_sha256":digest(&serde_json::to_vec(&vocabulary).unwrap()),"domain_namespace":null,"scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":references,"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":dependencies,"coverage":[]});
        (vocabulary, document)
    }
    #[test]
    fn same_component_clone_work_is_global_even_without_context_owners() {
        let (mut vocabulary, mut document) = fixture(192);
        document["identity"] = json!((1..192).map(|index| json!({"kind":"same","left":format!("ref-{:04}",index-1),"right":format!("ref-{index:04}"),"evidence":[{"start_byte":0,"end_byte":1,"quote":"a"}]})).collect::<Vec<_>>());
        let report =
            serde_json::to_value(crate::documentary_v3::validate(&document, "a", &vocabulary))
                .unwrap();
        assert_eq!(report["status"], "invalid", "{report}");
        assert!(
            array(&report["issues"])
                .iter()
                .any(|issue| issue["code"] == "request_context_limit")
        );
        // Removing the opt-in budget preserves the established graph result.
        vocabulary
            .as_object_mut()
            .unwrap()
            .remove("request_context_profile");
        document["vocabulary_sha256"] = json!(digest(&serde_json::to_vec(&vocabulary).unwrap()));
        let report =
            serde_json::to_value(crate::documentary_v3::validate(&document, "a", &vocabulary))
                .unwrap();
        assert_eq!(report["status"], "complete");
    }
    #[test]
    fn legacy_owners_under_profile_cannot_repeat_dependency_scans_without_charge() {
        let (vocabulary, mut document) = fixture(32);
        let mut claims = vec![];
        for index in 1..32 {
            document["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"reference","id":format!("ref-{index:04}")},"requires":{"kind":"reference","id":format!("ref-{:04}",index-1)},"role":"identity"}));
        }
        for index in 0..256 {
            let id = format!("claim-{index:04}");
            claims.push(json!({"id":id,"scope_id":"root","qualifier_ids":[],"evidence":[{"start_byte":0,"end_byte":1,"quote":"a"}],"body":{"kind":"proposition","value":{"atom":{"predicate":"owns","arguments":[{"role":"owner","value":{"kind":"ref","id":"ref-0031"}},{"role":"object","value":{"kind":"ref","id":"ref-0000"}}],"polarity":"positive","evidence":[{"start_byte":0,"end_byte":1,"quote":"a"}]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]}}}));
            for (kind, target) in [
                ("scope", "root"),
                ("reference", "ref-0031"),
                ("reference", "ref-0000"),
            ] {
                document["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":id},"requires":{"kind":kind,"id":target},"role":"identity"}));
            }
        }
        document["claims"] = json!(claims);
        let report =
            serde_json::to_value(crate::documentary_v3::validate(&document, "a", &vocabulary))
                .unwrap();
        assert_eq!(report["status"], "invalid");
        assert!(
            array(&report["issues"])
                .iter()
                .any(|issue| issue["code"] == "request_context_limit"),
            "{report}"
        );
    }
}
