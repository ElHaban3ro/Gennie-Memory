use super::DocumentResult;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[path = "validation/admission_request_context.rs"]
mod admission_request_context;
mod graph;
pub(super) mod nominal_de;
mod nominal_possessive;
mod relative_bound;
mod request_context;
use graph::graph;

/// Shared operand graph: matching must not drop edges granted implicitly by validation.
pub(super) fn effective_context_dependencies(
    raw: &Value,
    kind: &str,
    id: &str,
) -> Vec<(String, String)> {
    request_context::implicit_dependencies(raw, kind, id)
}

pub(super) fn selection_term_valid(term: &Value, text: &str, vocab: &Value) -> bool {
    let mut check = Check {
        text,
        vocab,
        issues: Vec::new(),
        nodes: 0,
        limit_exceeded: false,
    };
    check.node("Term", term, "/selection/term", 0);
    check.issues.is_empty() && !check.limit_exceeded
}

const MAX_NODES: usize = 100_000;
const MAX_DEPTH: usize = 64;
struct Check<'a> {
    text: &'a str,
    vocab: &'a Value,
    issues: Vec<Value>,
    nodes: usize,
    limit_exceeded: bool,
}
impl Check<'_> {
    fn issue(&mut self, code: &str, path: &str) {
        if !super::context_budget::charge(
            1,
            code.len().saturating_add(path.len()).saturating_add(32),
        ) {
            self.limit_exceeded = true;
            return;
        }
        self.issues.push(json!({"id":format!("issue-{}",self.issues.len()+1),"code":code,"path":path,"evidence_refs":[]}));
    }
    fn shape(&mut self, v: &Value, p: &str, required: &[&str]) -> bool {
        let Some(o) = v.as_object() else {
            self.issue("invalid_shape", p);
            return false;
        };
        let valid = o.len() == required.len()
            && required
                .iter()
                .take_while(|_| budget_visit())
                .all(|k| o.contains_key(*k));
        if !valid {
            self.issue("invalid_shape", p);
        }
        valid
    }
    fn string(&mut self, v: &Value, p: &str) {
        if !v.is_string() || v.as_str() == Some("") {
            self.issue("invalid_shape", p);
        }
    }
    fn choice(&mut self, v: &Value, p: &str, options: &[&str]) {
        if !v.as_str().is_some_and(|s| options.contains(&s)) {
            self.issue("invalid_shape", p);
        }
    }
    fn list(&mut self, v: &Value, p: &str, t: &str, depth: usize) {
        if let Some(a) = v.as_array() {
            for (i, x) in a.iter().take_while(|_| budget_visit()).enumerate() {
                self.node(t, x, &format!("{p}/{i}"), depth + 1);
            }
        } else {
            self.issue("invalid_shape", p);
        }
    }
    fn evidence(&mut self, v: &Value, p: &str) {
        self.shape(v, p, &["start_byte", "end_byte", "quote"]);
        let range = v["start_byte"].as_u64().zip(v["end_byte"].as_u64());
        let valid = range.is_some_and(|(s, e)| {
            s < e
                && e <= u32::MAX as u64
                && self
                    .text
                    .get(s as usize..e as usize)
                    .is_some_and(|q| Some(q) == v["quote"].as_str())
        });
        if !valid {
            self.issue("invalid_quote", p);
        }
    }
    fn node(&mut self, t: &str, v: &Value, p: &str, depth: usize) {
        self.nodes += 1;
        if super::request_context_profile::enabled(self.vocab)
            && !super::context_budget::charge(1, 0)
        {
            self.limit_exceeded = true;
            self.issue("request_context_limit", p);
            return;
        }
        if depth > MAX_DEPTH || self.nodes > MAX_NODES {
            self.limit_exceeded = true;
            self.issue("unsupported_combination", p);
            return;
        }
        if t == "Body" && v["kind"] == super::linguistic_composition::KIND {
            let nominal = super::nominal_event_context::carries(v);
            let permitted = super::request_context_profile::permits(
                self.vocab,
                if nominal {
                    super::nominal_event_context::FEATURE
                } else {
                    super::linguistic_composition::FEATURE
                },
            );
            let valid = if nominal {
                super::nominal_event_context::valid(v, self.text, None)
            } else {
                super::linguistic_composition::valid(v, self.text, None)
            };
            if !permitted || !valid {
                self.issue("invalid_shape", p);
            }
            return;
        }
        if nominal_possessive::node(self, t, v, p, depth)
            || nominal_de::node(self, t, v, p, depth)
            || relative_bound::node(self, t, v, p, depth)
            || request_context::node(self, t, v, p, depth)
        {
            return;
        }
        if t == "Evidence" {
            self.evidence(v, p);
            return;
        }
        if t == "Id" {
            self.string(v, p);
            return;
        }
        let fields: &[&str] = match t {
            "Document" => &[
                "version",
                "text_sha256",
                "kind",
                "vocabulary_sha256",
                "domain_namespace",
                "scopes",
                "references",
                "identity",
                "qualifiers",
                "claims",
                "requests",
                "dependencies",
                "coverage",
            ],
            "Scope" => &["id", "parent_id", "kind", "branch_id", "binding"],
            "Binding" => &[
                "variable",
                "quantifier",
                "domain_ref",
                "cardinality",
                "witness_relation",
                "relation_closures",
                "evidence",
            ],
            "Cardinality" => &["relation", "value", "domain", "evidence"],
            "Reference" => &[
                "id",
                "scope_id",
                "mention",
                "name",
                "declared_class",
                "semantic_type",
                "denotation",
                "links",
                "binding_scope_id",
                "anaphora",
            ],
            "SemanticType" => &["type_id", "evidence"],
            "Link" => &["kind", "target", "evidence"],
            "Anaphora" => &["status", "antecedent_ids", "evidence"],
            "IdentityEdge" => &["kind", "left", "right", "evidence"],
            "Term" => match v["kind"].as_str() {
                Some("ref") => &["kind", "id"],
                Some("described_ref") => &["kind", "id", "constraints", "resolution", "evidence"],
                Some("text") => &["kind", "value", "evidence"],
                Some("quantity") => &["kind", "decimal", "unit", "bound", "evidence"],
                Some("time") => &["kind", "value", "format", "evidence"],
                Some("symbol") => &["kind", "id", "evidence"],
                _ => {
                    self.issue("invalid_shape", p);
                    return;
                }
            },
            "Atom" | "AtomPattern" => &["predicate", "arguments", "polarity", "evidence"],
            "Argument" => &["role", "value"],
            "Proposition" => &[
                "atom",
                "judgment",
                "status",
                "basis",
                "normative_scope",
                "qualifier_ids",
            ],
            "Pattern" => &["atom", "judgment", "status", "qualifier_ids"],
            "Qualifier" => &["id", "scope_id", "kind", "body", "resolution", "evidence"],
            "Configuration" => &["key", "value"],
            "Time" => &["role", "value"],
            "Claim" => &["id", "scope_id", "qualifier_ids", "evidence", "body"],
            "Body" => match v["kind"].as_str() {
                Some("normative_open_pattern")
                    if super::request_context_profile::permits(
                        self.vocab,
                        "normative_open_pattern",
                    ) =>
                {
                    &[
                        "kind",
                        "target",
                        "open_role",
                        "judgment",
                        "status",
                        "basis",
                        "normative_scope",
                    ]
                }
                Some("proposition") => &["kind", "value"],
                Some("value_restriction") => &[
                    "kind",
                    "target",
                    "answer_role",
                    "closure",
                    "members",
                    "judgment",
                    "status",
                    "basis",
                    "normative_scope",
                    "existence",
                ],
                Some("event") => &[
                    "kind",
                    "event_ref",
                    "event_kind",
                    "subject",
                    "aspect",
                    "status",
                    "method",
                    "times",
                    "result",
                    "basis",
                ],
                Some("rule") => &[
                    "kind",
                    "rule_ref",
                    "normative_scope",
                    "direction",
                    "action",
                    "conditions",
                ],
                Some("exemption") => &[
                    "kind",
                    "rule_targets",
                    "normative_scope",
                    "beneficiary",
                    "waived_conditions",
                    "target_resolution",
                ],
                Some("alternatives") => &["kind", "closure", "exclusive", "branches"],
                _ => {
                    self.issue("invalid_shape", p);
                    return;
                }
            },
            "Method" => {
                if v["state"] == "specified" {
                    &["state", "value"]
                } else {
                    &["state"]
                }
            }
            "Branch" => &["id", "scope_id", "claim_ids"],
            "Request" => &[
                "id",
                "scope_id",
                "mode",
                "target",
                "slot",
                "class_filter",
                "unit",
                "basis_requirement",
                "required_answer_extent",
                "normative_scope_filter",
                "qualifier_ids",
                "premise_ids",
                "evidence",
            ],
            "EventPattern" => &[
                "event_ref",
                "event_kind",
                "subject",
                "aspect",
                "status",
                "time_selector",
                "method_selector",
            ],
            "MethodSelector" => {
                if v["kind"] == "specified" {
                    &["kind", "value"]
                } else {
                    &["kind"]
                }
            }
            "ClassFilter" => {
                if v["kind"] == "reference_pattern" {
                    &["kind", "reference_id"]
                } else if v["kind"] == "lexical" {
                    &["kind", "class_quote"]
                } else {
                    &["kind", "type_id", "evidence"]
                }
            }
            "AnswerDomain" => match v["kind"].as_str() {
                Some("all") => &["kind"],
                Some("lexical") => &["kind", "class_quote"],
                _ => &["kind", "type_id", "evidence"],
            },
            "Unit" => &["symbol", "evidence"],
            "BasisRequirement" => {
                if v["kind"] == "exact" {
                    &["kind", "value"]
                } else {
                    &["kind"]
                }
            }
            "Dependency" => &["consumer", "requires", "role"],
            "Endpoint" => &["kind", "id"],
            "Coverage" => &["span", "state", "owner", "issue_id"],
            "Closure" => &[
                "id",
                "scope_id",
                "domain_binding",
                "object_binding",
                "target",
                "closed_role",
                "evidence",
            ],
            _ => {
                self.issue("invalid_shape", p);
                return;
            }
        };
        let context_fields;
        let fields = if matches!(t, "Claim" | "Request")
            && v.get("context_bindings").is_some()
            && super::request_context_profile::enabled(self.vocab)
        {
            let mut expanded = fields.to_vec();
            expanded.push("context_bindings");
            context_fields = expanded;
            context_fields.as_slice()
        } else {
            fields
        };
        let domain_fields;
        let fields = if t == "Body"
            && v["kind"] == "value_restriction"
            && v.get("answer_domain").is_some()
        {
            let mut expanded = fields.to_vec();
            expanded.push("answer_domain");
            domain_fields = expanded;
            domain_fields.as_slice()
        } else {
            fields
        };
        if t == "Body"
            && v["kind"] == "value_restriction"
            && self.vocab["restriction_domain_profile"]["version"] == "explicit_answer_domains_v1"
            && v.get("answer_domain").is_none()
        {
            self.issue("unsupported_combination", &format!("{p}/answer_domain"));
        }
        if !self.shape(v, p, fields) {
            return;
        }
        for k in charged(fields) {
            let x = &v[*k];
            let q = format!("{p}/{k}");
            if *k == "answer_domain" && x.is_null() {
                self.issue("unsupported_combination", &q);
                continue;
            }
            if x.is_null()
                && matches!(
                    *k,
                    "parent_id"
                        | "branch_id"
                        | "binding"
                        | "domain_ref"
                        | "cardinality"
                        | "name"
                        | "declared_class"
                        | "semantic_type"
                        | "binding_scope_id"
                        | "anaphora"
                        | "normative_scope"
                        | "aspect"
                        | "result"
                        | "event_ref"
                        | "time_selector"
                        | "class_filter"
                        | "unit"
                        | "normative_scope_filter"
                        | "owner"
                        | "issue_id"
                        | "domain_namespace"
                )
            {
                continue;
            }
            if *k == "context_bindings" {
                request_context::contexts(self, v, &q, depth);
                continue;
            }
            let child = match *k {
                "open_role" => Some("NormativeOpenRole"),
                "mention" | "name" | "declared_class" | "class_quote" | "span" => Some("Evidence"),
                "binding" => Some("Binding"),
                "cardinality" => Some("Cardinality"),
                "semantic_type" => Some("SemanticType"),
                "anaphora" => Some("Anaphora"),
                "atom" => Some(if t == "Pattern" {
                    "AtomPattern"
                } else {
                    "Atom"
                }),
                "body" => Some(match t {
                    "Claim" => "Body",
                    "Qualifier" => match v["kind"].as_str() {
                        Some("guard") => "Proposition",
                        Some("configuration") => "Configuration",
                        _ => "Time",
                    },
                    _ => "Body",
                }),
                "target" if t == "Request" => Some(if v["mode"] == "event_slot" {
                    "EventPattern"
                } else {
                    "Pattern"
                }),
                "target" if t == "Closure" => Some("Proposition"),
                "target" if t == "Body" && v["kind"] == "normative_open_pattern" => {
                    Some("AtomPattern")
                }
                "target" if t == "Body" => Some("Atom"),
                "value"
                    if matches!(
                        t,
                        "Argument" | "Method" | "Time" | "Configuration" | "MethodSelector"
                    ) =>
                {
                    Some("Term")
                }
                "value" if t == "Body" => Some("Proposition"),
                "action" => Some("Proposition"),
                "result" => Some("Term"),
                "method" => Some("Method"),
                "time_selector" => Some("Time"),
                "method_selector" => Some("MethodSelector"),
                "class_filter" => Some("ClassFilter"),
                "answer_domain" => Some("AnswerDomain"),
                "unit" if t == "Request" => Some("Unit"),
                "basis_requirement" => Some("BasisRequirement"),
                "consumer" | "requires" | "owner" => Some("Endpoint"),
                _ => None,
            };
            if let Some(child) = child {
                if child == "Term" && x["kind"] == "described_ref" && t != "Argument" {
                    self.issue("invalid_shape", &q);
                }
                self.node(child, x, &q, depth + 1);
                continue;
            }
            let arr = match *k {
                "scopes" => Some("Scope"),
                "references" => Some("Reference"),
                "identity" => Some("IdentityEdge"),
                "qualifiers" => Some("Qualifier"),
                "claims" => Some("Claim"),
                "requests" => Some("Request"),
                "dependencies" => Some("Dependency"),
                "coverage" => Some("Coverage"),
                "evidence" => Some("Evidence"),
                "links" => Some("Link"),
                "arguments" => Some("Argument"),
                "conditions" | "constraints" => Some("Proposition"),
                "waived_conditions" => Some("Atom"),
                "branches" => Some("Branch"),
                "members" => Some("Term"),
                "times" => Some("Time"),
                "relation_closures" => Some("Closure"),
                "qualifier_ids" | "premise_ids" | "rule_targets" | "claim_ids"
                | "antecedent_ids" => Some("Id"),
                _ => None,
            };
            if let Some(a) = arr {
                self.list(x, &q, a, depth);
                continue;
            }
            let options: Option<&[&str]> = match *k {
                "kind" => Some(match t {
                    "Document" => &["source", "question"],
                    "Scope" => &["document", "branch", "binder"],
                    "Link" => &[
                        "owned_by",
                        "located_at",
                        "part_of",
                        "member_of",
                        "associated_with",
                    ],
                    "IdentityEdge" => &["same", "different"],
                    "Term" => &["ref", "described_ref", "text", "quantity", "time", "symbol"],
                    "Qualifier" => &["guard", "configuration", "time"],
                    "Body"
                        if v["kind"] == "normative_open_pattern"
                            && super::request_context_profile::permits(
                                self.vocab,
                                "normative_open_pattern",
                            ) =>
                    {
                        &["normative_open_pattern"]
                    }
                    "Body" => &[
                        "proposition",
                        "value_restriction",
                        "event",
                        "rule",
                        "exemption",
                        "alternatives",
                    ],
                    "ClassFilter" => &["lexical", "semantic", "reference_pattern"],
                    "AnswerDomain" => &["all", "lexical", "semantic"],
                    "MethodSelector" => &["any", "specified"],
                    "BasisRequirement" => &["any", "exact"],
                    "Endpoint" => &["claim", "request", "reference", "qualifier", "scope"],
                    _ => &[],
                }),
                "quantifier" => Some(&["exists", "forall"]),
                "witness_relation" => Some(&[
                    "unspecified",
                    "same_across_domain",
                    "distinct_across_domain",
                ]),
                "relation" => Some(&["exact", "at_least", "at_most"]),
                "denotation"
                    if super::request_context_profile::permits(
                        self.vocab,
                        "nominal_possessive",
                    ) =>
                {
                    Some(&["entity", "group", "class", "lexical_use"])
                }
                "denotation" => Some(&["entity", "group", "class"]),
                "polarity" => Some(&["positive", "negative"]),
                "judgment" => Some(&["fact", "allowed", "required", "forbidden"]),
                "basis" => Some(&[
                    "description",
                    "observation",
                    "assignment",
                    "capability",
                    "normative",
                    "specification",
                ]),
                "resolution" => Some(&["known", "unknown"]),
                "closure" => Some(if v["kind"] == "value_restriction" {
                    &["closed"]
                } else {
                    &["closed", "open"]
                }),
                "exclusive" => Some(&["asserted", "not_asserted"]),
                "existence" => Some(&["asserted", "not_asserted"]),
                "direction" => Some(&["necessary", "sufficient"]),
                "target_resolution" => Some(&["exact", "unresolved"]),
                "event_kind" => Some(&["measurement", "inspection"]),
                "format" => Some(&["rfc3339", "date", "literal", "year"]),
                "bound" => Some(&["exact", "at_least", "at_most"]),
                "mode" => Some(&[
                    "answer",
                    "truth",
                    "constraint",
                    "event_slot",
                    "configuration_slot",
                    "rule_condition",
                    "exemption_status",
                ]),
                "required_answer_extent" => Some(&["one_value", "all_values", "truth_status"]),
                "status" => Some(match t {
                    "Anaphora" => &["asserted", "unresolved"],
                    "EventPattern" => &["planned", "occurred", "any"],
                    "Body" if v["kind"] == "event" => {
                        &["planned", "occurred", "negated_occurrence"]
                    }
                    _ => &["state", "planned", "occurred"],
                }),
                "state" => Some(if t == "Coverage" {
                    &["represented", "unresolved", "unsupported"]
                } else {
                    &["specified", "unspecified"]
                }),
                "role" if t == "Time" => {
                    Some(&["holds_at", "scheduled_at", "planned_for", "occurred_at"])
                }
                "role" if t == "Dependency" => Some(&[
                    "identity",
                    "guard",
                    "configuration",
                    "temporal",
                    "binding",
                    "premise",
                    "normative_target",
                    "branch_membership",
                ]),
                _ => None,
            };
            if let Some(opts) = options {
                self.choice(x, &q, opts)
            } else if *k == "value" && t == "Cardinality" {
                if !x.as_u64().is_some_and(|n| n <= u32::MAX as u64) {
                    self.issue("invalid_shape", &q)
                }
            } else {
                self.string(x, &q)
            }
        }
        self.local(t, v, p);
    }
    fn local(&mut self, t: &str, v: &Value, p: &str) {
        if matches!(
            t,
            "Atom"
                | "AtomPattern"
                | "Claim"
                | "IdentityEdge"
                | "Binding"
                | "Cardinality"
                | "Qualifier"
                | "SemanticType"
                | "Closure"
                | "Link"
                | "Anaphora"
                | "Unit"
                | "Request"
        ) || (t == "Term" && v["kind"] != "ref")
        {
            if array(&v["evidence"]).is_empty() {
                self.issue("invalid_shape", &format!("{p}/evidence"));
            }
        }
        if t == "BasisRequirement" && v["kind"] == "exact" {
            self.choice(
                &v["value"],
                p,
                &[
                    "description",
                    "observation",
                    "assignment",
                    "capability",
                    "normative",
                    "specification",
                ],
            );
        }
        if t == "Request" && v["mode"] == "configuration_slot" {
            if !super::configuration_profile::permits(self.vocab, v["slot"].as_str().unwrap_or(""))
                || v["required_answer_extent"] != "one_value"
                || v["target"]["judgment"] != "fact"
                || v["target"]["atom"]["polarity"] != "positive"
            {
                self.issue("invalid_shape", p);
            }
        }
        if t == "Request" && v["mode"] == "event_slot" {
            self.choice(
                &v["slot"],
                p,
                &[
                    "status",
                    "method",
                    "aspect",
                    "result",
                    "scheduled_at",
                    "planned_for",
                    "occurred_at",
                    "scheduled_year",
                    "planned_year",
                    "occurred_year",
                ],
            );
        }
        if t == "Term" && v["kind"] == "described_ref" {
            for code in super::descriptions::issues(v, self.vocab) {
                self.issue(code, p);
            }
        }
        if t == "Term" && v["kind"] == "time" && v["format"] == "year" {
            let year = v["value"].as_str().unwrap_or("");
            if year.len() != 4
                || !year.bytes().all(|b| b.is_ascii_digit())
                || !array(&v["evidence"])
                    .iter()
                    .take_while(|_| budget_visit())
                    .any(|e| e["quote"] == year)
            {
                self.issue("invalid_shape", p);
            }
        }
        if t == "Time" && v["value"]["kind"] != "time" {
            self.issue("invalid_shape", p);
        }
        if matches!(t, "Proposition" | "Body")
            && matches!(
                v["judgment"].as_str(),
                Some("allowed" | "required" | "forbidden")
            )
            && !v["normative_scope"].is_string()
        {
            self.issue("invalid_shape", &format!("{p}/normative_scope"));
        }
        if t == "Term" && v["kind"] == "quantity" {
            let s = v["decimal"].as_str().unwrap_or("");
            let s = s.strip_prefix('-').unwrap_or(s);
            let parts: Vec<_> = s.split('.').collect();
            if parts.len() > 2
                || parts
                    .iter()
                    .take_while(|_| budget_visit())
                    .any(|x| x.is_empty() || !x.bytes().all(|b| b.is_ascii_digit()))
            {
                self.issue("invalid_shape", &format!("{p}/decimal"));
            }
        }
        if t == "Atom" || t == "AtomPattern" {
            let pred = v["predicate"].as_str().unwrap_or("");
            let row = self.vocab["predicate_rows"]
                .as_array()
                .and_then(|a| {
                    a.iter()
                        .take_while(|_| budget_visit())
                        .find(|r| r["id"] == pred)
                })
                .cloned();
            let Some(row) = row else {
                self.issue("unknown_vocabulary", p);
                return;
            };
            let mut seen = BTreeSet::new();
            for (i, arg) in charged(
                array(&v["arguments"])
                    .iter()
                    .take_while(|_| budget_visit())
                    .enumerate(),
            ) {
                let role = arg["role"].as_str().unwrap_or("");
                let term = &arg["value"];
                let typ = row["roles"][role].as_str();
                if !seen.insert(role) || typ.is_none() {
                    self.issue("unknown_vocabulary", &format!("{p}/arguments/{i}"));
                }
                let good = match typ {
                    Some("ref") => matches!(term["kind"].as_str(), Some("ref" | "described_ref")),
                    Some("quantity") => term["kind"] == "quantity",
                    Some("attribute_id") => {
                        term["kind"] == "symbol"
                            && array(&self.vocab["fixed_attribute_rows"]).contains(&term["id"])
                    }
                    Some("action_id") => {
                        term["kind"] == "symbol"
                            && array(&self.vocab["fixed_action_rows"]).contains(&term["id"])
                    }
                    Some("term") => {
                        term["kind"] != "symbol"
                            && (term["kind"] != "described_ref"
                                || super::descriptions::term_roles_enabled(self.vocab))
                    }
                    _ => false,
                };
                if !good {
                    self.issue("unknown_vocabulary", &format!("{p}/arguments/{i}/value"));
                }
            }
            if t == "Atom"
                && row["roles"]
                    .as_object()
                    .is_some_and(|r| r.len() != seen.len())
            {
                self.issue("invalid_shape", p);
            }
        }
        if t == "Reference" && !v["semantic_type"].is_null() {
            let st = &v["semantic_type"];
            let row = array(&self.vocab["semantic_type_rows"])
                .iter()
                .take_while(|_| budget_visit())
                .find(|r| r["id"] == st["type_id"]);
            if !row.is_some_and(|r| {
                array(&st["evidence"])
                    .iter()
                    .take_while(|_| budget_visit())
                    .all(|e| array(&r["class_quotes"]).contains(&e["quote"]))
            }) {
                self.issue("unknown_vocabulary", p);
            }
        }
        if matches!(t, "ClassFilter" | "AnswerDomain")
            && v["kind"] == "semantic"
            && !array(&self.vocab["semantic_type_rows"])
                .iter()
                .take_while(|_| budget_visit())
                .any(|r| r["id"] == v["type_id"])
        {
            self.issue("unknown_vocabulary", p);
        }
        if matches!(t, "ClassFilter" | "AnswerDomain") && v["kind"] == "semantic" {
            let row = array(&self.vocab["semantic_type_rows"])
                .iter()
                .take_while(|_| budget_visit())
                .find(|r| r["id"] == v["type_id"]);
            if array(&v["evidence"]).is_empty()
                || !row.is_some_and(|r| {
                    array(&v["evidence"])
                        .iter()
                        .take_while(|_| budget_visit())
                        .all(|e| array(&r["class_quotes"]).contains(&e["quote"]))
                })
            {
                self.issue("unknown_vocabulary", p);
            }
        }
        if t == "Body"
            && v["kind"] == "value_restriction"
            && array(&v["members"])
                .iter()
                .take_while(|_| budget_visit())
                .any(|m| m["kind"] == "described_ref")
        {
            let role = v["answer_role"].as_str().unwrap_or("");
            if !array(&self.vocab["predicate_rows"])
                .iter()
                .take_while(|_| budget_visit())
                .any(|r| {
                    r["id"] == v["target"]["predicate"]
                        && (r["roles"][role] == "ref"
                            || (r["roles"][role] == "term"
                                && super::descriptions::term_roles_enabled(self.vocab)))
                })
            {
                self.issue("invalid_shape", &format!("{p}/members"));
            }
        }
        if t == "Body"
            && v["kind"] == "value_restriction"
            && v.get("answer_domain")
                .is_some_and(|d| !d.is_null() && d["kind"] != "all")
        {
            let role = v["answer_role"].as_str().unwrap_or("");
            if !array(&self.vocab["predicate_rows"])
                .iter()
                .take_while(|_| budget_visit())
                .any(|row| row["id"] == v["target"]["predicate"] && row["roles"][role] == "ref")
            {
                self.issue("unsupported_combination", &format!("{p}/answer_domain"));
            }
        }
        if t == "Coverage" && !v["owner"].is_null() {
            self.choice(&v["owner"]["kind"], p, &["claim", "request", "reference"]);
        }
        if t == "Body" && v["kind"] == "event" {
            if !v["aspect"].is_null()
                && !array(&self.vocab["fixed_attribute_rows"]).contains(&v["aspect"])
            {
                self.issue("unknown_vocabulary", p);
            }
            let mut roles = BTreeSet::new();
            for time in charged(array(&v["times"])) {
                if !roles.insert(time["role"].as_str().unwrap_or(""))
                    || time["value"]["kind"] != "time"
                    || (time["role"] == "occurred_at" && v["status"] != "occurred")
                    || (time["role"] == "planned_for" && v["status"] != "planned")
                    || time["role"] == "holds_at"
                {
                    self.issue("invalid_shape", p);
                }
            }
        }
        if t == "Request" && v["mode"] != "event_slot" {
            let a = &v["target"]["atom"];
            let pred = &a["predicate"];
            if let Some(row) = array(&self.vocab["predicate_rows"])
                .iter()
                .take_while(|_| budget_visit())
                .find(|r| r["id"] == *pred)
            {
                let args = array(&a["arguments"]);
                let roles = row["roles"].as_object();
                let missing: Vec<_> = roles
                    .into_iter()
                    .flat_map(|r| r.keys().take_while(|_| budget_visit()))
                    .filter(|r| {
                        !args
                            .iter()
                            .take_while(|_| budget_visit())
                            .any(|x| x["role"] == r.as_str())
                    })
                    .collect();
                if v["mode"] == "truth"
                    || v["mode"] == "configuration_slot"
                    || (v["mode"] == "answer"
                        && matches!(
                            v["slot"].as_str(),
                            Some(
                                "holds_at"
                                    | "scheduled_at"
                                    | "planned_for"
                                    | "occurred_at"
                                    | "holds_year"
                                    | "scheduled_year"
                                    | "planned_year"
                                    | "occurred_year"
                            )
                        ))
                    || (matches!(
                        v["mode"].as_str(),
                        Some("rule_condition" | "exemption_status")
                    ) && !row["roles"]
                        .as_object()
                        .is_some_and(|roles| roles.contains_key(v["slot"].as_str().unwrap_or(""))))
                {
                    if !missing.is_empty() {
                        self.issue("invalid_shape", p);
                    }
                } else if missing.len() != 1
                    || missing[0].as_str() != v["slot"].as_str().unwrap_or("")
                {
                    self.issue("invalid_shape", p);
                }
            }
        }
    }
}
fn budget_visit() -> bool {
    super::context_budget::charge(1, 0)
}
fn charged<I: IntoIterator>(iter: I) -> impl Iterator<Item = I::Item> {
    iter.into_iter().take_while(|_| budget_visit())
}

fn path_id(id: &str) -> String {
    id.replace('~', "~0").replace('/', "~1")
}
fn array(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn digest(v: &[u8]) -> String {
    Sha256::digest(v)
        .iter()
        .take_while(|_| budget_visit())
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn collect(v: &Value, key: &str, out: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in charged(o) {
                if k == key {
                    if let Some(s) = x.as_str() {
                        if super::context_budget::charge(1, s.len()) {
                            out.push(s.to_owned());
                        }
                    }
                }
                collect(x, key, out);
            }
        }
        Value::Array(a) => {
            for x in charged(a) {
                collect(x, key, out)
            }
        }
        _ => {}
    }
}
fn refs(v: &Value, out: &mut BTreeSet<String>) {
    if let Some(o) = v.as_object() {
        for (k, x) in charged(o) {
            if (k == "id" && matches!(v["kind"].as_str(), Some("ref" | "described_ref")))
                || matches!(k.as_str(),"boundary_subject"|"reference_id"|"subject"|"event_ref"|"rule_ref"|"beneficiary"|"normative_scope"|"normative_scope_filter"|"domain_ref"|"variable"|"domain"|"domain_binding"|"object_binding"|"left"|"right"|"target" if x.is_string())
            {
                if let Some(s) = x.as_str() {
                    if super::context_budget::charge(1, s.len()) {
                        out.insert(s.to_owned());
                    }
                }
            }
            if k == "antecedent_ids" {
                for s in charged(
                    array(x)
                        .iter()
                        .take_while(|_| budget_visit())
                        .filter_map(Value::as_str),
                ) {
                    if super::context_budget::charge(1, s.len()) {
                        out.insert(s.to_owned());
                    }
                }
            }
            refs(x, out);
        }
    } else if let Some(a) = v.as_array() {
        for x in charged(a) {
            refs(x, out)
        }
    }
}
fn vocabulary(c: &mut Check<'_>, v: &Value) {
    let mut fields = vec![
        "version",
        "predicate_rows",
        "fixed_attribute_rows",
        "fixed_action_rows",
        "semantic_type_rows",
        "relevance_links",
    ];
    if v.get("attribute_profiles").is_some() {
        fields.push("attribute_profiles");
    }
    if v.get("request_context_profile").is_some() {
        fields.push("request_context_profile");
        if !super::request_context_profile::valid(v) {
            c.issue("unknown_vocabulary", "/vocabulary/request_context_profile");
        }
    }
    if v.get("assignment_projection").is_some() {
        fields.push("assignment_projection");
    }
    if v.get("admission_request_context").is_some() {
        fields.push("admission_request_context");
    }
    if v.get("relevance_profiles").is_some() {
        fields.push("relevance_profiles");
    }
    if v.get("measurement_context").is_some() {
        fields.push("measurement_context");
    }
    if v.get("documentary_anchor_context").is_some() {
        fields.push("documentary_anchor_context");
    }
    if v.get("restriction_domain_profile").is_some() {
        fields.push("restriction_domain_profile");
    }
    if v.get("configuration_request_profile").is_some() {
        fields.push("configuration_request_profile");
        if !super::configuration_profile::valid(v) {
            c.issue(
                "unknown_vocabulary",
                "/vocabulary/configuration_request_profile",
            );
        }
    }
    if let Some(profile) = v.get("term_description_profile") {
        fields.push("term_description_profile");
        if !c.shape(
            profile,
            "/vocabulary/term_description_profile",
            &["version"],
        ) || profile["version"] != super::descriptions::PROFILE
        {
            c.issue("unknown_vocabulary", "/vocabulary/term_description_profile");
        }
    }
    if let Some(profile) = v.get("described_term_role_profile") {
        fields.push("described_term_role_profile");
        if !c.shape(
            profile,
            "/vocabulary/described_term_role_profile",
            &["version"],
        ) || profile["version"] != super::descriptions::TERM_ROLES_PROFILE
            || !super::descriptions::enabled(v)
        {
            c.issue(
                "unknown_vocabulary",
                "/vocabulary/described_term_role_profile",
            );
        }
    }
    if !c.shape(v, "/vocabulary", &fields) {
        return;
    }
    c.string(&v["version"], "/vocabulary/version");
    let mut ids = BTreeSet::new();
    for key in ["fixed_attribute_rows", "fixed_action_rows"] {
        ids.clear();
        if !v[key].is_array() {
            c.issue("invalid_shape", "/vocabulary");
        }
        for x in charged(array(&v[key])) {
            c.string(x, "/vocabulary");
            if !ids.insert(x.clone().to_string()) {
                c.issue("invalid_shape", "/vocabulary");
            }
        }
    }
    let mut predicates = BTreeMap::new();
    if let Some(profile) = v.get("restriction_domain_profile") {
        let path = "/vocabulary/restriction_domain_profile";
        if !c.shape(profile, path, &["version"])
            || profile["version"] != "explicit_answer_domains_v1"
        {
            c.issue("invalid_shape", path);
        }
    }
    for r in charged(array(&v["predicate_rows"])) {
        c.shape(
            r,
            "/vocabulary/predicate_rows",
            &["id", "roles", "distributive"],
        );
        c.string(&r["id"], "/vocabulary/predicate_rows/id");
        if !r["distributive"].is_boolean()
            || !r["roles"].as_object().is_some_and(|o| {
                !o.is_empty()
                    && o.iter().take_while(|_| budget_visit()).all(|(k, v)| {
                        !k.is_empty()
                            && matches!(
                                v.as_str(),
                                Some("ref" | "attribute_id" | "action_id" | "term" | "quantity")
                            )
                    })
            })
        {
            c.issue("invalid_shape", "/vocabulary/predicate_rows");
        }
        if predicates
            .insert(r["id"].as_str().unwrap_or(""), r)
            .is_some()
        {
            c.issue("invalid_shape", "/vocabulary/predicate_rows");
        }
    }
    if let Some(profile) = v.get("assignment_projection") {
        let path = "/vocabulary/assignment_projection";
        c.shape(profile, path, &["version", "context_keys"]);
        let keys = array(&profile["context_keys"]);
        let unique: BTreeSet<_> = keys
            .iter()
            .take_while(|_| budget_visit())
            .filter_map(Value::as_str)
            .collect();
        if profile["version"] != "uses_qualifier_assignment_v1"
            || keys.is_empty()
            || unique.len() != keys.len()
            || unique
                .iter()
                .take_while(|_| budget_visit())
                .any(|key| key.is_empty())
            || predicates.get("uses").map(|r| &r["roles"])
                != Some(&json!({"subject":"ref","object":"ref"}))
            || predicates.get("quantity").map(|r| &r["roles"])
                != Some(&json!({"subject":"ref","attribute":"attribute_id","amount":"quantity"}))
        {
            c.issue("invalid_shape", path);
        }
    }
    let mut types = BTreeSet::new();
    for r in charged(array(&v["semantic_type_rows"])) {
        c.shape(r, "/vocabulary/semantic_type_rows", &["id", "class_quotes"]);
        c.string(&r["id"], "/vocabulary/semantic_type_rows/id");
        if !types.insert(r["id"].to_string())
            || !r["class_quotes"].as_array().is_some_and(|a| {
                !a.is_empty()
                    && a.iter()
                        .take_while(|_| budget_visit())
                        .all(|x| x.as_str().is_some_and(|s| !s.is_empty()))
            })
        {
            c.issue("invalid_shape", "/vocabulary/semantic_type_rows");
        }
    }
    let mut links = BTreeSet::new();
    for r in charged(array(&v["relevance_links"])) {
        c.shape(
            r,
            "/vocabulary/relevance_links",
            &[
                "id",
                "source_predicate",
                "target_predicate",
                "bindings",
                "carry_roles",
                "conclusion",
                "max_hops",
                "allow_reverse",
                "allow_transitive_composition",
            ],
        );
        if !links.insert(r["id"].to_string())
            || r["conclusion"] != "partial_intermediate_only"
            || r["max_hops"] != 1
            || r["allow_reverse"] != false
            || r["allow_transitive_composition"] != false
        {
            c.issue("invalid_shape", "/vocabulary/relevance_links");
        }
        let s = predicates.get(r["source_predicate"].as_str().unwrap_or(""));
        let t = predicates.get(r["target_predicate"].as_str().unwrap_or(""));
        let role = |value: &Value, prefix: &str, row: Option<&&Value>| {
            value
                .as_str()
                .and_then(|v| v.strip_prefix(prefix))
                .is_some_and(|k| row.is_some_and(|r| r["roles"].get(k).is_some()))
        };
        if s.is_none()
            || t.is_none()
            || !r["bindings"].as_array().is_some_and(|a| {
                !a.is_empty()
                    && a.iter().take_while(|_| budget_visit()).all(|b| {
                        b.as_array().is_some_and(|b| {
                            b.len() == 2 && role(&b[0], "source.", s) && role(&b[1], "target.", t)
                        })
                    })
            })
            || !r["carry_roles"].as_array().is_some_and(|a| {
                !a.is_empty()
                    && a.iter()
                        .take_while(|_| budget_visit())
                        .all(|r| role(r, "source.", s))
            })
        {
            c.issue("unknown_vocabulary", "/vocabulary/relevance_links");
        }
    }
    for k in ["predicate_rows", "semantic_type_rows", "relevance_links"] {
        if !v[k].is_array() {
            c.issue("invalid_shape", "/vocabulary");
        }
    }
    admission_request_context::validate(c, v);
    relevance_profiles(c, v);
    measurement_context(c, v);
    documentary_anchor_context(c, v);
}
fn documentary_anchor_context(c: &mut Check<'_>, v: &Value) {
    let Some(profile) = v.get("documentary_anchor_context") else {
        return;
    };
    let path = "/vocabulary/documentary_anchor_context";
    if !c.shape(profile, path, &["version", "rows"])
        || profile["version"] != "anchored_documentary_context_v1"
        || !profile["rows"].is_array()
    {
        c.issue("invalid_shape", path);
        return;
    }
    let mut ids = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for row in charged(array(&profile["rows"])) {
        let link = array(&v["relevance_links"])
            .iter()
            .take_while(|_| budget_visit())
            .find(|l| l["id"] == row["link_id"]);
        if !c.shape(
            row,
            path,
            &[
                "id",
                "link_id",
                "attribute_id",
                "target_status",
                "source_link_kind",
                "question_link_kind",
                "source_configuration",
            ],
        ) || !row["id"].as_str().is_some_and(|s| !s.is_empty())
            || !ids.insert(row["id"].to_string())
            || !targets.insert(format!(
                "{}:{}:{}",
                row["link_id"], row["attribute_id"], row["target_status"]
            ))
            || !array(&v["fixed_attribute_rows"]).contains(&row["attribute_id"])
            || row["target_status"] != "occurred"
            || !matches!(
                row["source_link_kind"].as_str(),
                Some("owned_by" | "part_of")
            )
            || !matches!(
                row["question_link_kind"].as_str(),
                Some("owned_by" | "part_of")
            )
            || !link.is_some_and(|l| {
                matches!(
                    l["source_predicate"].as_str(),
                    Some("information_state" | "documentary_availability")
                ) && matches!(
                    l["target_predicate"].as_str(),
                    Some("property" | "quantity")
                ) && array(&l["bindings"]).len() == 2
                    && array(&l["bindings"]).contains(&json!(["source.subject", "target.subject"]))
                    && array(&l["bindings"])
                        .contains(&json!(["source.attribute", "target.attribute"]))
                    && array(&l["carry_roles"]).contains(&json!("source.state"))
            })
            || !row["source_configuration"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
        {
            c.issue("invalid_shape", path);
        }
        let mut keys = BTreeSet::new();
        for config in charged(array(&row["source_configuration"])) {
            if !c.shape(config, path, &["key", "value"])
                || !config["key"].as_str().is_some_and(|s| !s.is_empty())
                || !config["value"].as_str().is_some_and(|s| !s.is_empty())
                || !keys.insert(config["key"].to_string())
            {
                c.issue("invalid_shape", path);
            }
        }
    }
}
fn measurement_context(c: &mut Check<'_>, v: &Value) {
    let Some(profile) = v.get("measurement_context") else {
        return;
    };
    let path = "/vocabulary/measurement_context";
    if !c.shape(profile, path, &["version", "rows"])
        || profile["version"] != "negative_measurement_context_v1"
        || !profile["rows"].is_array()
        || !array(&v["predicate_rows"])
            .iter()
            .take_while(|_| budget_visit())
            .any(|p| {
                p["id"] == "quantity"
                    && p["roles"]
                        == json!({"subject":"ref","attribute":"attribute_id","amount":"quantity"})
            })
    {
        c.issue("invalid_shape", path);
        return;
    }
    let mut ids = BTreeSet::new();
    for row in charged(array(&profile["rows"])) {
        if !c.shape(
            row,
            path,
            &[
                "id",
                "event_aspect",
                "target_attribute",
                "source_configuration",
                "target_configuration",
                "context_bindings",
            ],
        ) || !row["id"].as_str().is_some_and(|s| !s.is_empty())
            || !ids.insert(row["id"].to_string())
            || !row["event_aspect"].as_str().is_some_and(|s| !s.is_empty())
            || !array(&v["fixed_attribute_rows"]).contains(&row["target_attribute"])
            || !row["context_bindings"].is_array()
        {
            c.issue("invalid_shape", path);
        }
        let mut source_keys = BTreeSet::new();
        let mut target_keys = BTreeSet::new();
        for (field, keys) in [
            ("source_configuration", &mut source_keys),
            ("target_configuration", &mut target_keys),
        ] {
            if !row[field].is_array() {
                c.issue("invalid_shape", path);
            }
            for config in charged(array(&row[field])) {
                if !c.shape(config, path, &["key", "value"])
                    || !config["key"].as_str().is_some_and(|s| !s.is_empty())
                    || !config["value"].as_str().is_some_and(|s| !s.is_empty())
                    || !keys.insert(config["key"].to_string())
                {
                    c.issue("invalid_shape", path);
                }
            }
        }
        if array(&row["target_configuration"]).is_empty() {
            c.issue("invalid_shape", path);
        }
        for binding in charged(array(&row["context_bindings"])) {
            if !binding.as_array().is_some_and(|b| {
                b.len() == 2
                    && b.iter()
                        .take_while(|_| budget_visit())
                        .all(|v| v.as_str().is_some_and(|s| !s.is_empty()))
            }) || !source_keys.insert(binding[0].to_string())
                || !target_keys.insert(binding[1].to_string())
            {
                c.issue("invalid_shape", path);
            }
        }
    }
}
fn relevance_profiles(c: &mut Check<'_>, v: &Value) {
    let Some(profile) = v.get("relevance_profiles") else {
        return;
    };
    let path = "/vocabulary/relevance_profiles";
    if !c.shape(profile, path, &["version", "rows"])
        || profile["version"] != "documentary_relevance_profiles_v1"
        || !profile["rows"].is_array()
    {
        c.issue("invalid_shape", path);
        return;
    }
    let predicates: BTreeMap<_, _> = array(&v["predicate_rows"])
        .iter()
        .take_while(|_| budget_visit())
        .filter_map(|p| Some((p["id"].as_str()?, p)))
        .collect();
    let mut links = BTreeSet::new();
    for row in charged(array(&profile["rows"])) {
        if !c.shape(row, path, &["link_id", "alternatives"])
            || !links.insert(row["link_id"].to_string())
            || !row["alternatives"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
        {
            c.issue("invalid_shape", path);
            continue;
        }
        let Some(link) = array(&v["relevance_links"])
            .iter()
            .take_while(|_| budget_visit())
            .find(|l| l["id"] == row["link_id"])
        else {
            c.issue("unknown_vocabulary", path);
            continue;
        };
        let source = predicates.get(link["source_predicate"].as_str().unwrap_or(""));
        let target = predicates.get(link["target_predicate"].as_str().unwrap_or(""));
        let role = |p: Option<&&Value>, name: &Value| {
            name.as_str()
                .is_some_and(|n| p.is_some_and(|p| p["roles"][n] == "ref"))
        };
        let classes = |value: &Value| {
            value.as_array().is_some_and(|a| {
                !a.is_empty()
                    && a.iter()
                        .take_while(|_| budget_visit())
                        .all(|s| s.as_str().is_some_and(|s| !s.trim().is_empty()))
                    && a.iter()
                        .take_while(|_| budget_visit())
                        .map(Value::to_string)
                        .collect::<BTreeSet<_>>()
                        .len()
                        == a.len()
            })
        };
        let mut alternatives = BTreeSet::new();
        for guard in charged(array(&row["alternatives"])) {
            if !alternatives.insert(guard.to_string()) {
                c.issue("invalid_shape", path);
            }
            match guard["kind"].as_str() {
                Some("class_context") => {
                    if !c.shape(
                        guard,
                        path,
                        &[
                            "kind",
                            "source_role",
                            "source_classes",
                            "target_role",
                            "target_classes",
                        ],
                    ) || !role(source, &guard["source_role"])
                        || !role(target, &guard["target_role"])
                        || !classes(&guard["source_classes"])
                        || !classes(&guard["target_classes"])
                    {
                        c.issue("invalid_shape", path);
                    }
                }
                Some("answer_filter") => {
                    if !c.shape(guard, path, &["kind", "source_role"])
                        || !role(source, &guard["source_role"])
                    {
                        c.issue("invalid_shape", path);
                    }
                }
                Some("corroborating_claim") => {
                    let evidence = predicates.get(guard["predicate"].as_str().unwrap_or(""));
                    let mut source_anchor = false;
                    let mut target_anchor = false;
                    let mut unique = BTreeSet::new();
                    let valid = guard["bindings"].as_array().is_some_and(|a| {
                        !a.is_empty()
                            && a.iter().take_while(|_| budget_visit()).all(|b| {
                                if !b.as_array().is_some_and(|b| b.len() == 2)
                                    || !unique.insert(b.to_string())
                                {
                                    return false;
                                }
                                let a = b[0].as_str().unwrap_or("");
                                let e = b[1].as_str().unwrap_or("").strip_prefix("evidence.");
                                let good = if let Some(a) = a.strip_prefix("source.") {
                                    source_anchor = true;
                                    role(source, &Value::String(a.into()))
                                } else if let Some(a) = a.strip_prefix("target.") {
                                    target_anchor = true;
                                    role(target, &Value::String(a.into()))
                                } else {
                                    false
                                };
                                good && e.is_some_and(|e| role(evidence, &Value::String(e.into())))
                            })
                    });
                    if !c.shape(guard, path, &["kind", "predicate", "bindings"])
                        || evidence.is_none()
                        || !valid
                        || !source_anchor
                        || !target_anchor
                    {
                        c.issue("invalid_shape", path);
                    }
                }
                _ => c.issue("invalid_shape", path),
            }
        }
    }
}
// Only documentary identity edges and asserted anaphora establish equality;
// scope labels and fresh IDs never establish equality or inequality.
fn scope_identity(raw: &Value, left: &Value, right: &Value) -> &'static str {
    let (Some(left), Some(right)) = (left.as_str(), right.as_str()) else {
        return "unknown";
    };
    let mut edges = array(&raw["identity"])
        .iter()
        .take_while(|_| budget_visit())
        .filter(|e| e["kind"] == "same")
        .filter_map(|e| Some((e["left"].as_str()?, e["right"].as_str()?)))
        .collect::<Vec<_>>();
    for reference in charged(array(&raw["references"])) {
        if reference["anaphora"]["status"] == "asserted"
            && array(&reference["anaphora"]["antecedent_ids"]).len() == 1
        {
            if let (Some(id), Some(antecedent)) = (
                reference["id"].as_str(),
                reference["anaphora"]["antecedent_ids"][0].as_str(),
            ) {
                edges.push((id, antecedent));
            }
        }
    }
    let component = |start| {
        let mut result = BTreeSet::from([start]);
        let mut pending = vec![start];
        while let Some(id) = pending.pop() {
            if !budget_visit() {
                break;
            }
            for &(l, r) in charged(&edges) {
                let target = if id == l {
                    Some(r)
                } else if id == r {
                    Some(l)
                } else {
                    None
                };
                if let Some(target) = target {
                    if result.insert(target) {
                        pending.push(target);
                    }
                }
            }
        }
        result
    };
    let l = component(left);
    let r = component(right);
    let same = l.contains(right);
    let different = array(&raw["identity"])
        .iter()
        .take_while(|_| budget_visit())
        .any(|edge| {
            edge["kind"] == "different"
                && edge["left"]
                    .as_str()
                    .zip(edge["right"].as_str())
                    .is_some_and(|(a, b)| {
                        (l.contains(a) && r.contains(b)) || (l.contains(b) && r.contains(a))
                    })
        });
    match (same, different) {
        (true, true) => "conflict",
        (true, false) => "same",
        (false, true) => "different",
        _ => "unknown",
    }
}
fn combinations(c: &mut Check<'_>, raw: &Value) {
    let claims: BTreeMap<_, _> = array(&raw["claims"])
        .iter()
        .take_while(|_| budget_visit())
        .filter_map(|r| r["id"].as_str().map(|id| (id, r)))
        .collect();
    let scopes: BTreeMap<_, _> = array(&raw["scopes"])
        .iter()
        .take_while(|_| budget_visit())
        .filter_map(|r| r["id"].as_str().map(|id| (id.to_owned(), r)))
        .collect();
    let references: BTreeMap<_, _> = array(&raw["references"])
        .iter()
        .take_while(|_| budget_visit())
        .filter_map(|r| r["id"].as_str().map(|id| (id, r)))
        .collect();
    let mut branch_ids = BTreeSet::new();
    for (id, claim) in charged(&claims) {
        let b = &claim["body"];
        let p = format!("/claim/{}/body", path_id(id));
        if b["kind"] == "alternatives" {
            if array(&b["branches"]).len() < 2 {
                c.issue("invalid_shape", &p);
            }
            for br in charged(array(&b["branches"])) {
                let scope = br["scope_id"].as_str().unwrap_or("");
                if !branch_ids.insert(br["id"].to_string())
                    || !scopes.get(scope).is_some_and(|s| {
                        s["kind"] == "branch"
                            && s["branch_id"] == br["id"]
                            && s["parent_id"] == claim["scope_id"]
                    })
                {
                    c.issue("scope_escape", &p);
                }
                if array(&br["claim_ids"]).is_empty() {
                    c.issue("invalid_shape", &p);
                }
                for cid in charged(array(&br["claim_ids"])) {
                    let child = cid.as_str().and_then(|x| claims.get(x));
                    // The alternatives carrier imposes its qualifiers on every
                    // child. Membership is structural; children need not copy
                    // those qualifiers into their own additional qualifier list.
                    if !child.is_some_and(|cc| cc["scope_id"] == scope) {
                        c.issue("scope_escape", &p);
                    }
                }
            }
        }
        if b["kind"] == "exemption" {
            if b["target_resolution"] == "unresolved" {
                c.issue("unresolved_target", &p);
                continue;
            }
            if array(&b["rule_targets"]).is_empty()
                || array(&b["waived_conditions"]).is_empty()
                || b["normative_scope"].is_null()
            {
                c.issue("unresolved_target", &p);
            }
            for target in charged(array(&b["rule_targets"])) {
                let rule = target.as_str().and_then(|x| claims.get(x));
                if !rule.is_some_and(|r| {
                    r["body"]["kind"] == "rule"
                        && r["body"]["direction"] == "necessary"
                        && scope_identity(raw, &r["body"]["normative_scope"], &b["normative_scope"])
                            == "same"
                        && array(&b["waived_conditions"])
                            .iter()
                            .take_while(|_| budget_visit())
                            .all(|a| {
                                array(&r["body"]["conditions"])
                                    .iter()
                                    .take_while(|_| budget_visit())
                                    .any(|p| {
                                        p["atom"]["predicate"] == a["predicate"]
                                            && p["atom"]["polarity"] == a["polarity"]
                                            && p["atom"]["arguments"] == a["arguments"]
                                    })
                            })
                }) {
                    c.issue("unresolved_target", &p);
                }
            }
        }
        if b["kind"] == "rule" {
            if array(&b["conditions"]).is_empty() {
                c.issue("invalid_shape", &p);
            }
            if matches!(
                b["action"]["judgment"].as_str(),
                Some("allowed" | "required" | "forbidden")
            ) {
                match scope_identity(raw, &b["normative_scope"], &b["action"]["normative_scope"]) {
                    "same" => {}
                    "different" => c.issue("unsupported_combination", &p),
                    "conflict" => c.issue("ambiguous_identity", &p),
                    _ => c.issue("unresolved_target", &p),
                }
            }
        }
    }
    for s in scopes.values().take_while(|_| budget_visit()) {
        for closure in charged(array(&s["binding"]["relation_closures"])) {
            let p = format!(
                "/scope/{}/binding/relation_closures",
                path_id(s["id"].as_str().unwrap_or(""))
            );
            let role = closure["closed_role"].as_str().unwrap_or("");
            let d = closure["domain_binding"]
                .as_str()
                .and_then(|r| references.get(r));
            let o = closure["object_binding"]
                .as_str()
                .and_then(|r| references.get(r));
            let args = array(&closure["target"]["atom"]["arguments"]);
            let domain_binder = d
                .and_then(|r| r["binding_scope_id"].as_str())
                .and_then(|id| scopes.get(id));
            let object_binder = o
                .and_then(|r| r["binding_scope_id"].as_str())
                .and_then(|id| scopes.get(id));
            if !domain_binder.is_some_and(|b| b["binding"]["quantifier"] == "forall")
                || !object_binder.is_some_and(|b| b["binding"]["quantifier"] == "exists")
            {
                c.issue("unsupported_combination", &p);
            }
            if closure["scope_id"] != s["id"]
                || d.is_none()
                || o.is_none()
                || !args.iter().take_while(|_| budget_visit()).any(|a| {
                    a["role"] == role
                        && a["value"]["kind"] == "ref"
                        && a["value"]["id"] == closure["object_binding"]
                })
                || !args.iter().take_while(|_| budget_visit()).any(|a| {
                    a["value"]["kind"] == "ref" && a["value"]["id"] == closure["domain_binding"]
                })
                || array(&closure["evidence"]).is_empty()
            {
                c.issue("unsupported_combination", &p);
            }
        }
    }
}
pub(super) fn validate(raw: &Value, text: &str, vocab: &Value) -> DocumentResult {
    let vh = digest(&serde_json::to_vec(vocab).unwrap_or_default());
    let mut c = Check {
        text,
        vocab,
        issues: Vec::new(),
        nodes: 0,
        limit_exceeded: false,
    };
    vocabulary(&mut c, vocab);
    for (code, path) in super::attribute_profiles::vocabulary_issues(vocab) {
        c.issue(&code, &path);
    }
    c.node("Document", raw, "", 0);
    if raw["version"] != "documentary_evidence_v3" {
        c.issue("invalid_shape", "/version");
    }
    if raw["text_sha256"] != digest(text.as_bytes()) {
        c.issue("invalid_quote", "/text_sha256");
    }
    if raw["vocabulary_sha256"] != vh {
        c.issue("unknown_vocabulary", "/vocabulary_sha256");
    }
    if c.nodes <= MAX_NODES {
        graph(&mut c, raw);
        if !super::context_budget::exhausted() {
            combinations(&mut c, raw);
        }
        if !super::context_budget::exhausted() {
            request_context::document(&mut c, raw);
            relative_bound::document(&mut c, raw);
            nominal_de::document(&mut c, raw);
            nominal_possessive::document(&mut c, raw);
            if super::request_context_profile::permits(
                vocab,
                super::linguistic_composition::FEATURE,
            ) || super::request_context_profile::permits(
                vocab,
                super::nominal_event_context::FEATURE,
            ) {
                for (index, claim) in array(&raw["claims"]).iter().enumerate() {
                    if !budget_visit() {
                        break;
                    }
                    if super::linguistic_composition::carries(claim)
                        && (raw["kind"] != "source"
                            || !if super::nominal_event_context::carries(&claim["body"]) {
                                super::nominal_event_context::valid(&claim["body"], text, Some(raw))
                            } else {
                                super::linguistic_composition::valid(
                                    &claim["body"],
                                    text,
                                    Some(raw),
                                )
                            })
                    {
                        c.issue("invalid_shape", &format!("/claims/{index}/body"));
                    }
                }
            }
        }
        if !c.limit_exceeded && !super::context_budget::exhausted() {
            for (code, path) in super::attribute_profiles::document_issues(raw, vocab) {
                c.issue(&code, &path);
            }
            for (code, path) in super::descriptions::document_issues(raw) {
                c.issue(&code, &path);
            }
        }
    }
    if raw["kind"] == "question" && array(&raw["requests"]).is_empty() {
        c.issue("invalid_shape", "/requests");
    }
    let invalid = c.limit_exceeded
        || c.issues.iter().take_while(|_| budget_visit()).any(|i| {
            matches!(
                i["code"].as_str(),
                Some("invalid_shape" | "invalid_quote" | "dangling_id" | "scope_escape")
            ) || (i["code"] == "unknown_vocabulary"
                && i["path"]
                    .as_str()
                    .is_some_and(|p| p.starts_with("/vocabulary")))
        });
    let mut candidates = Vec::new();
    for (table, claim) in [("claims", true), ("requests", false)] {
        for row in charged(array(&raw[table])) {
            let mut r = BTreeSet::new();
            refs(row, &mut r);
            let id = &row["id"];
            let own = format!(
                "/{}/{}/",
                if claim { "claim" } else { "request" },
                path_id(id.as_str().unwrap_or(""))
            );
            let mut relevant: Vec<_> = c
                .issues
                .iter()
                .take_while(|_| budget_visit())
                .filter(|i| {
                    i["path"]
                        .as_str()
                        .is_some_and(|p| p.starts_with(&own) || p == own.trim_end_matches('/'))
                })
                .collect();
            let mut pending = vec![(
                if claim { "claim" } else { "request" }.to_owned(),
                id.as_str().unwrap_or("").to_owned(),
            )];
            let mut visited = BTreeSet::new();
            while let Some((kind, node_id)) = pending.pop() {
                if !budget_visit() {
                    break;
                }
                if !visited.insert((kind.clone(), node_id.clone())) {
                    continue;
                }
                for issue in charged(&c.issues) {
                    let singular = format!("/{kind}/{}", path_id(&node_id));
                    // Recursive shape checks use real JSON pointers (array indices).
                    // Graph checks use singular kind/ID paths. Match both without
                    // interpreting a technical ID as an array index.
                    let table = match kind.as_str() {
                        "claim" => "claims",
                        "request" => "requests",
                        "reference" => "references",
                        "qualifier" => "qualifiers",
                        "scope" => "scopes",
                        _ => "",
                    };
                    let indexed = array(&raw[table])
                        .iter()
                        .take_while(|_| budget_visit())
                        .position(|row| row["id"] == node_id)
                        .map(|index| format!("/{table}/{index}"));
                    if issue["path"].as_str().is_some_and(|p| {
                        p == singular
                            || p.starts_with(&(singular.clone() + "/"))
                            || indexed.as_ref().is_some_and(|prefix| {
                                p == prefix || p.starts_with(&(prefix.clone() + "/"))
                            })
                    }) && !relevant.contains(&issue)
                    {
                        relevant.push(issue);
                    }
                }
                if super::request_context_profile::enabled(vocab)
                    && request_context::has_carrier(row)
                {
                    if !super::context_budget::charge(1, 0) {
                        // Budget exhaustion is checked at the public operation boundary too.
                        break;
                    }
                    pending.extend(request_context::implicit_dependencies(raw, &kind, &node_id));
                }
                for d in array(&raw["dependencies"])
                    .iter()
                    .take_while(|_| budget_visit())
                    .filter(|d| d["consumer"]["kind"] == kind && d["consumer"]["id"] == node_id)
                {
                    if let (Some(k), Some(i)) =
                        (d["requires"]["kind"].as_str(), d["requires"]["id"].as_str())
                    {
                        if !super::context_budget::charge(1, k.len().saturating_add(i.len())) {
                            break;
                        }
                        pending.push((k.to_owned(), i.to_owned()));
                    }
                }
            }
            let codes: Vec<_> = relevant
                .iter()
                .take_while(|_| budget_visit())
                .map(|i| &i["code"])
                .collect();
            let conditional = !array(&row["qualifier_ids"]).is_empty()
                || scopes_branch(raw, row["scope_id"].as_str().unwrap_or(""));
            // Reserve serialized candidate contents before json! copies any borrowed values.
            let evidence_bytes =
                super::context_budget::serialized_bytes(&row["evidence"]).unwrap_or(usize::MAX);
            let refs_bytes = r
                .iter()
                .try_fold(0usize, |n, rid| n.checked_add(rid.len().saturating_add(4)))
                .unwrap_or(usize::MAX);
            let issue_bytes = relevant
                .iter()
                .try_fold(0usize, |n, issue| {
                    n.checked_add(
                        issue["code"]
                            .as_str()
                            .map_or(0, str::len)
                            .saturating_add(issue["path"].as_str().map_or(0, str::len))
                            .saturating_add(16),
                    )
                })
                .unwrap_or(usize::MAX);
            let candidate_bytes = evidence_bytes
                .checked_add(refs_bytes)
                .and_then(|n| n.checked_add(issue_bytes))
                .and_then(|n| {
                    n.checked_add(
                        id.as_str()
                            .map_or(0, str::len)
                            .saturating_mul(2)
                            .saturating_add(512),
                    )
                })
                .unwrap_or(usize::MAX);
            if !super::context_budget::charge(
                r.len().saturating_add(relevant.len()).saturating_add(1),
                candidate_bytes,
            ) {
                break;
            }
            candidates.push(json!({"claim_ids":if claim{vec![id]}else{vec![]},"request_id":if claim{None}else{Some(id)},"reference_ids":r,"state":if invalid{"blocked"}else if !codes.is_empty(){"unresolved"}else if conditional{"conditional"}else{"eligible"},"reason_codes":codes,"dependency_paths":relevant.iter().take_while(|_| budget_visit()).map(|i|&i["path"]).collect::<Vec<_>>(),"evidence_refs":array(&row["evidence"])}));
        }
    }
    if super::request_context_profile::enabled(vocab)
        && !invalid
        && !super::context_budget::exhausted()
    {
        let bytes = super::context_budget::serialized_bytes(raw).unwrap_or(usize::MAX);
        super::context_budget::charge(1, bytes);
    }
    let budget_exhausted =
        super::request_context_profile::enabled(vocab) && super::context_budget::exhausted();
    if budget_exhausted {
        c.issue("request_context_limit", "");
        for candidate in &mut candidates {
            candidate["state"] = json!("blocked");
        }
    }
    let invalid = invalid || budget_exhausted;
    let status = if invalid {
        "invalid"
    } else if c.issues.is_empty() {
        "complete"
    } else {
        "partial"
    };
    // Move already-budgeted reports instead of cloning them again through json!.
    let mut report = json!({"status":status,"document":null,"candidates":null,"issues":null});
    if !invalid {
        report["document"] = raw.clone();
    }
    report["candidates"] = Value::Array(candidates);
    report["issues"] = Value::Array(c.issues);
    super::normative_indeterminacy::decorate(&mut report, raw, vocab);
    DocumentResult {
        report,
        vocabulary_sha256: vh,
    }
}
fn scopes_branch(raw: &Value, id: &str) -> bool {
    let mut next = Some(id);
    let mut seen = BTreeSet::new();
    while let Some(id) = next {
        if !budget_visit() {
            return false;
        }
        if !seen.insert(id) {
            break;
        }
        let Some(s) = array(&raw["scopes"])
            .iter()
            .take_while(|_| budget_visit())
            .find(|s| s["id"] == id)
        else {
            break;
        };
        if s["kind"] == "branch" {
            return true;
        }
        next = s["parent_id"].as_str();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    include!("described_term_roles_tests.rs");
    fn vocabulary() -> Value {
        json!({"version":"test-v1","predicate_rows":[{"id":"owns","roles":{"owner":"ref","object":"ref"},"distributive":true}],"fixed_attribute_rows":[],"fixed_action_rows":[],"semantic_type_rows":[],"relevance_links":[]})
    }
    fn doc(text: &str) -> Value {
        json!({"version":"documentary_evidence_v3","text_sha256":digest(text.as_bytes()),"kind":"source","vocabulary_sha256":digest(&serde_json::to_vec(&vocabulary()).unwrap()),"domain_namespace":null,"scopes":[{"id":"root","parent_id":null,"kind":"document","branch_id":null,"binding":null}],"references":[],"identity":[],"qualifiers":[],"claims":[],"requests":[],"dependencies":[],"coverage":[]})
    }
    fn quote() -> Value {
        json!({"start_byte":0,"end_byte":1,"quote":"a"})
    }
    fn useful() -> Value {
        let mut d = doc("a");
        d["references"]=json!(["owner","object"].map(|id|json!({"id":id,"scope_id":"root","mention":quote(),"name":null,"declared_class":null,"semantic_type":null,"denotation":"entity","links":[],"binding_scope_id":null,"anaphora":null})));
        d["claims"] = json!([{"id":"c","scope_id":"root","qualifier_ids":[],"evidence":[quote()],"body":{"kind":"proposition","value":{"atom":{"predicate":"owns","arguments":[{"role":"owner","value":{"kind":"ref","id":"owner"}},{"role":"object","value":{"kind":"ref","id":"object"}}],"polarity":"positive","evidence":[quote()]},"judgment":"fact","status":"state","basis":"description","normative_scope":null,"qualifier_ids":[]}}}]);
        d["dependencies"] = json!([
            {"consumer":{"kind":"reference","id":"owner"},"requires":{"kind":"scope","id":"root"},"role":"binding"},
            {"consumer":{"kind":"reference","id":"object"},"requires":{"kind":"scope","id":"root"},"role":"binding"},
            {"consumer":{"kind":"claim","id":"c"},"requires":{"kind":"scope","id":"root"},"role":"binding"},
            {"consumer":{"kind":"claim","id":"c"},"requires":{"kind":"reference","id":"owner"},"role":"identity"},
            {"consumer":{"kind":"claim","id":"c"},"requires":{"kind":"reference","id":"object"},"role":"identity"}]);
        d
    }
    #[test]
    fn associated_link_uses_existing_literal_scope_and_dependency_validation() {
        let mut d = useful();
        d["references"][0]["links"] =
            json!([{"kind":"associated_with","target":"object","evidence":[quote()]}]);
        let dep = json!({"consumer":{"kind":"reference","id":"owner"},"requires":{"kind":"reference","id":"object"},"role":"identity"});
        d["dependencies"].as_array_mut().unwrap().push(dep);
        assert_eq!(
            validate(&d, "a", &vocabulary()).report["status"],
            "complete"
        );
        let mut missing = d.clone();
        missing["dependencies"].as_array_mut().unwrap().pop();
        assert_eq!(
            validate(&missing, "a", &vocabulary()).report["status"],
            "invalid"
        );
        let mut dangling = d.clone();
        dangling["references"][0]["links"][0]["target"] = json!("absent");
        assert_eq!(
            validate(&dangling, "a", &vocabulary()).report["status"],
            "invalid"
        );
        let mut unquoted = d.clone();
        unquoted["references"][0]["links"][0]["evidence"][0]["quote"] = json!("invented");
        assert_eq!(
            validate(&unquoted, "a", &vocabulary()).report["status"],
            "invalid"
        );
        d["scopes"].as_array_mut().unwrap().push(json!({"id":"branch","parent_id":"root","kind":"branch","branch_id":"b","binding":null}));
        d["references"][1]["scope_id"] = json!("branch");
        d["dependencies"][1]["requires"]["id"] = json!("branch");
        let result = validate(&d, "a", &vocabulary());
        assert_eq!(result.report["status"], "invalid");
        assert!(
            array(&result.report["issues"])
                .iter()
                .any(|i| i["code"] == "scope_escape")
        );
    }
    #[test]
    fn useful_claim_eligible_and_identity_not_collapsed() {
        let d = useful();
        let r = validate(&d, "a", &vocabulary());
        assert_eq!(r.report["issues"], json!([]));
        assert_eq!(r.report["candidates"][0]["state"], "eligible");
        assert_eq!(
            r.report["document"]["references"].as_array().unwrap().len(),
            2
        );
    }
    #[test]
    fn missing_dependency_and_same_different_conflict() {
        let mut d = useful();
        d["dependencies"].as_array_mut().unwrap().pop();
        assert_eq!(validate(&d, "a", &vocabulary()).report["status"], "invalid");
        let mut d = useful();
        d["identity"] = json!([{"kind":"same","left":"owner","right":"object","evidence":[quote()]},{"kind":"different","left":"owner","right":"object","evidence":[quote()]}]);
        assert_eq!(validate(&d, "a", &vocabulary()).report["status"], "partial");
    }
    #[test]
    fn normative_scope_is_mandatory() {
        let mut d = useful();
        d["claims"][0]["body"]["value"]["judgment"] = json!("allowed");
        assert_eq!(validate(&d, "a", &vocabulary()).report["status"], "invalid");
    }
    #[test]
    fn event_preserves_missing_method_and_temporal_role() {
        let mut d = useful();
        d["claims"][0]["body"] = json!({"kind":"event","event_ref":"owner","event_kind":"measurement","subject":"object","aspect":null,"status":"planned","method":{"state":"unspecified"},"times":[{"role":"planned_for","value":{"kind":"time","value":"tomorrow","format":"literal","evidence":[quote()]}}],"result":null,"basis":"description"});
        assert_eq!(
            validate(&d, "a", &vocabulary()).report["status"],
            "complete"
        );
        d["claims"][0]["body"]["times"][0]["role"] = json!("occurred_at");
        assert_eq!(validate(&d, "a", &vocabulary()).report["status"], "invalid");
    }
    #[test]
    fn dependency_cycles_are_invalid() {
        let mut d = useful();
        d["dependencies"].as_array_mut().unwrap().extend([json!({"consumer":{"kind":"reference","id":"owner"},"requires":{"kind":"reference","id":"object"},"role":"identity"}),json!({"consumer":{"kind":"reference","id":"object"},"requires":{"kind":"reference","id":"owner"},"role":"identity"})]);
        assert_eq!(validate(&d, "a", &vocabulary()).report["status"], "invalid");
    }
    #[test]
    fn branch_reference_cannot_escape_to_parent() {
        let mut d = useful();
        d["scopes"].as_array_mut().unwrap().push(json!({"id":"branch","parent_id":"root","kind":"branch","branch_id":"b","binding":null}));
        d["references"][1]["scope_id"] = json!("branch");
        d["dependencies"][1]["requires"]["id"] = json!("branch");
        let result = validate(&d, "a", &vocabulary());
        assert_eq!(result.report["status"], "invalid");
        assert!(
            array(&result.report["issues"])
                .iter()
                .any(|i| i["code"] == "scope_escape")
        );
    }
    #[test]
    fn unknown_qualifier_propagates_only_to_consumer() {
        let mut d = useful();
        let clone = d["claims"][0].clone();
        d["claims"].as_array_mut().unwrap().push(clone);
        d["claims"][1]["id"] = json!("independent");
        let deps = array(&d["dependencies"])
            .iter()
            .filter(|d| d["consumer"]["kind"] == "claim")
            .cloned()
            .map(|mut d| {
                d["consumer"]["id"] = json!("independent");
                d
            })
            .collect::<Vec<_>>();
        d["dependencies"].as_array_mut().unwrap().extend(deps);
        d["qualifiers"] = json!([{"id":"q","scope_id":"root","kind":"configuration","body":{"key":"mode","value":{"kind":"text","value":"unknown","evidence":[quote()]}},"resolution":"unknown","evidence":[quote()]}]);
        d["claims"][0]["qualifier_ids"] = json!(["q"]);
        d["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":"c"},"requires":{"kind":"qualifier","id":"q"},"role":"configuration"}));
        let result = validate(&d, "a", &vocabulary());
        assert_eq!(result.report["status"], "partial");
        assert_eq!(result.report["candidates"][0]["state"], "unresolved");
        assert_eq!(result.report["candidates"][1]["state"], "eligible");
    }
    #[test]
    fn unknown_claim_vocabulary_preserves_independent_and_propagates_to_consumer() {
        let mut d = useful();
        let claim = d["claims"][0].clone();
        let deps = array(&d["dependencies"])
            .iter()
            .filter(|d| d["consumer"]["kind"] == "claim")
            .cloned()
            .collect::<Vec<_>>();
        for id in ["unknown", "dependent"] {
            let mut row = claim.clone();
            row["id"] = json!(id);
            d["claims"].as_array_mut().unwrap().push(row);
            d["dependencies"]
                .as_array_mut()
                .unwrap()
                .extend(deps.iter().cloned().map(|mut d| {
                    d["consumer"]["id"] = json!(id);
                    d
                }));
        }
        d["claims"][1]["body"]["value"]["atom"]["predicate"] = json!("undeclared_predicate");
        d["dependencies"].as_array_mut().unwrap().push(json!({"consumer":{"kind":"claim","id":"dependent"},"requires":{"kind":"claim","id":"unknown"},"role":"premise"}));
        let result = validate(&d, "a", &vocabulary());
        assert_eq!(result.report["status"], "partial");
        assert_eq!(result.report["document"], d);
        assert_eq!(result.report["candidates"][0]["state"], "eligible");
        for index in [1, 2] {
            let candidate = &result.report["candidates"][index];
            assert_eq!(candidate["state"], "unresolved");
            assert!(array(&candidate["reason_codes"]).contains(&json!("unknown_vocabulary")));
            assert!(
                array(&candidate["dependency_paths"])
                    .iter()
                    .any(|p| p.as_str().is_some_and(|p| p.starts_with("/claims/1/")))
            );
        }
    }
    #[test]
    fn unknown_request_type_stays_local_and_preserves_request() {
        let mut d = useful();
        let atom = d["claims"][0]["body"]["value"]["atom"].clone();
        d["kind"] = json!("question");
        d["claims"] = json!([]);
        d["requests"] = json!([{"id":"r","scope_id":"root","mode":"truth","target":{"atom":atom,"judgment":"fact","status":"state","qualifier_ids":[]},"slot":"status","class_filter":{"kind":"semantic","type_id":"not_declared","evidence":[quote()]},"unit":null,"basis_requirement":{"kind":"any"},"required_answer_extent":"truth_status","normative_scope_filter":null,"qualifier_ids":[],"premise_ids":[],"evidence":[quote()]}]);
        for dep in d["dependencies"].as_array_mut().unwrap() {
            if dep["consumer"]["kind"] == "claim" {
                dep["consumer"] = json!({"kind":"request","id":"r"});
            }
        }
        let result = validate(&d, "a", &vocabulary());
        assert_eq!(result.report["status"], "partial");
        assert_eq!(result.report["document"], d);
        assert_eq!(result.report["candidates"][0]["request_id"], "r");
        assert_eq!(result.report["candidates"][0]["state"], "unresolved");
        assert!(
            array(&result.report["candidates"][0]["dependency_paths"])
                .iter()
                .any(|p| p.as_str().is_some_and(|p| p.starts_with("/requests/0/")))
        );
    }
    #[test]
    fn unknown_reference_type_propagates_and_vocabulary_hash_stays_fatal() {
        let mut d = useful();
        d["references"][0]["semantic_type"] =
            json!({"type_id":"unknown_type","evidence":[quote()]});
        let result = validate(&d, "a", &vocabulary());
        assert_eq!(result.report["status"], "partial");
        assert_eq!(result.report["candidates"][0]["state"], "unresolved");
        d["vocabulary_sha256"] = json!("incorrect");
        assert_eq!(validate(&d, "a", &vocabulary()).report["status"], "invalid");
        let mut v = vocabulary();
        v["unexpected"] = json!(true);
        assert_eq!(validate(&useful(), "a", &v).report["status"], "invalid");
    }
    #[test]
    fn rule_normative_scope_requires_documentary_same_and_preserves_independents() {
        let mut d = useful();
        let fact = d["claims"][0]["body"]["value"].clone();
        let mut action = fact.clone();
        action["judgment"] = json!("allowed");
        action["normative_scope"] = json!("object");
        d["claims"][0]["body"] = json!({"kind":"rule","rule_ref":"owner","normative_scope":"owner","direction":"sufficient","action":action,"conditions":[fact]});
        let result = validate(&d, "a", &vocabulary());
        assert_eq!(result.report["status"], "partial");
        assert_eq!(result.report["candidates"][0]["state"], "unresolved");
        assert!(
            array(&result.report["issues"])
                .iter()
                .any(|i| i["code"] == "unresolved_target")
        );
        d["identity"] =
            json!([{"kind":"same","left":"owner","right":"object","evidence":[quote()]}]);
        assert_eq!(
            validate(&d, "a", &vocabulary()).report["status"],
            "complete"
        );
        d["identity"][0]["kind"] = json!("different");
        let result = validate(&d, "a", &vocabulary());
        assert_eq!(result.report["status"], "partial");
        assert!(
            array(&result.report["issues"])
                .iter()
                .any(|i| i["code"] == "unsupported_combination")
        );
    }
    #[test]
    fn accepts_empty_source_but_not_empty_question() {
        let mut d = doc("á");
        assert_eq!(
            validate(&d, "á", &vocabulary()).report["status"],
            "complete"
        );
        d["kind"] = json!("question");
        assert_eq!(validate(&d, "á", &vocabulary()).report["status"], "invalid");
    }
    #[test]
    fn rejects_utf8_interior_and_unknown_fields() {
        let mut d = doc("á");
        d["coverage"] = json!([{"span":{"start_byte":1,"end_byte":2,"quote":"á"},"state":"represented","owner":null,"issue_id":null}]);
        assert_eq!(validate(&d, "á", &vocabulary()).report["status"], "invalid");
        let mut d = doc("");
        d["extra"] = json!(true);
        assert_eq!(validate(&d, "", &vocabulary()).report["status"], "invalid");
    }
    #[test]
    fn preserves_invalid_candidates() {
        let mut d = doc("a");
        d["claims"] = json!([{"id":"bad","garbage":true}]);
        let r = validate(&d, "a", &vocabulary());
        assert_eq!(r.report["status"], "invalid");
        assert_eq!(r.report["candidates"][0]["claim_ids"], json!(["bad"]));
        assert_eq!(r.report["candidates"][0]["state"], "blocked");
    }
    #[test]
    fn coverage_unknown_is_partial_not_invalid() {
        let mut d = doc("a");
        d["coverage"] = json!([{"span":{"start_byte":0,"end_byte":1,"quote":"a"},"state":"unsupported","owner":null,"issue_id":"pending"}]);
        assert_eq!(validate(&d, "a", &vocabulary()).report["status"], "partial");
    }
}
