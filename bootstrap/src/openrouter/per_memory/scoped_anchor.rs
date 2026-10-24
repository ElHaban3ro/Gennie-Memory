//! One request: original ScopedV1 paths plus a shared, literal query-case witness.
use super::*;
use gennie_memory_retrieval::per_memory::anchor::{LiteralAnchor, present, query_names};
pub(in crate::openrouter) fn payload(
    query: &str,
    scope: &str,
    candidates: &[Candidate],
    provider: ClassifierProvider,
) -> Value {
    let candidates = &candidates[..candidates.len().min(7)];
    let mut p = super::scoped::payload(query, scope, candidates, provider);
    let names = query_names(query);
    if !names.is_empty() {
        let mut criteria = serde_json::Map::new();
        for (i, name) in names.iter().enumerate() {
            criteria.insert(((b'A' + i as u8) as char).to_string(), json!(name));
        }
        criteria.insert("none".into(), json!("No reliable literal name"));
        p["questions"]["case_anchor"] = json!({"type":"choice","instructions":"Choose the fixed named case in state.user_prompt, not an assumed author/owner or unknown answer. Choose none for uncertain, implicit or alias naming. This is a case name, not evidence.","criteria":criteria});
    }
    p
}
pub(in crate::openrouter) fn parse(
    mut body: Value,
    query: &str,
    candidates: &[Candidate],
) -> Result<DecisionResult, DecisionError> {
    let candidates = &candidates[..candidates.len().min(7)];
    let invalid = || DecisionError {
        code: "scoped_anchor_invalid",
        http_status: None,
        partial_usage: known_receipt(&body),
    };
    let names = query_names(query);
    let token = if names.is_empty() {
        if body["answers"].get("case_anchor").is_some() {
            return Err(invalid());
        }
        None
    } else {
        let a = &body["answers"]["case_anchor"];
        let mut one = body.clone();
        one["answers"] = json!({"relevance":a});
        super::super::parse(one, &names).map_err(|_| invalid())?;
        if a["type"] != "choice" {
            return Err(invalid());
        }
        match a["choice"].as_str() {
            Some("none") => None,
            Some(s) if s.len() == 1 => {
                let i = s.as_bytes()[0].checked_sub(b'A').ok_or_else(invalid)? as usize;
                Some(names.get(i).ok_or_else(invalid)?.clone())
            }
            _ => return Err(invalid()),
        }
    };
    if !names.is_empty() {
        body["answers"]
            .as_object_mut()
            .ok_or_else(|| failure("scoped_anchor_invalid"))?
            .remove("case_anchor");
    }
    let mut d = super::scoped::parse(body, candidates)?;
    let trace = d
        .per_memory
        .as_mut()
        .ok_or_else(|| failure("scoped_anchor_invalid"))?;
    for ((check, item), candidate) in trace
        .scoped_checks
        .as_mut()
        .unwrap()
        .iter_mut()
        .zip(&mut trace.items)
        .zip(candidates)
    {
        check.literal_anchor = Some(LiteralAnchor {
            query_token: token.clone(),
            present_in_source: token.as_deref().map(|t| present(t, &candidate.summary)),
        });
        item.relation = if check.accepted() {
            EvidenceRelation::UsefulEvidence
        } else {
            EvidenceRelation::Unrelated
        };
    }
    d.selected_id = trace.selected_ids().first().map(|s| (*s).to_string());
    Ok(d)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn candidates() -> Vec<Candidate> {
        [
            "Teo diseñó Neblina.",
            "Sara escribió el libreto Luciérnaga.",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, s)| Candidate {
            memory_id: format!("m{i}"),
            revision: 1,
            summary: s.into(),
            conditions: None,
            source: None,
            state: "active".into(),
            bm25: 0.,
        })
        .collect()
    }
    fn body() -> Value {
        let a = |s| json!({"type":"choice","choice":s,"probabilities":{"A":0.5,"none":0.5}});
        let mut result = json!({"model":"test","usage":{"input_tokens":20,"output_tokens":0,"cost":0.001},"answers":{"case_anchor":a("B"),"memory_00_fact":a("none"),"memory_00_correction":a("A"),"memory_01_fact":a("none"),"memory_01_correction":a("A")}});
        result["answers"]["case_anchor"]["probabilities"] = json!({"A":0.25,"B":0.5,"none":0.25});
        result
    }
    #[test]
    fn corrected_author_is_not_required_as_fixed_identity_and_receipt_is_not_multiplied() {
        let q = "¿Cuándo escribió Teo Luciérnaga?";
        let c = candidates();
        let d = parse(body(), q, &c).unwrap();
        let t = d.per_memory.unwrap();
        assert_eq!(t.selected_ids(), ["m1"]);
        assert!(t.valid(&c, Some("m1")));
        assert!(t.anchors_valid(q, &c, true));
        assert_eq!(d.reported_cost_usd, Some(0.001));
        let mut p = payload(q, "s", &c, ClassifierProvider::Jev);
        p["questions"]
            .as_object_mut()
            .unwrap()
            .remove("case_anchor");
        assert_eq!(
            p,
            super::super::scoped::payload(q, "s", &c, ClassifierProvider::Jev)
        );
    }
    #[test]
    fn missing_or_out_of_namespace_anchor_fails_with_known_charge() {
        let q = "¿Cuándo escribió Teo Luciérnaga?";
        for value in ["Z", ""] {
            let mut b = body();
            b["answers"]["case_anchor"]["choice"] = json!(value);
            assert_eq!(
                parse(b, q, &candidates())
                    .unwrap_err()
                    .partial_usage
                    .unwrap()
                    .reported_cost_usd,
                Some(0.001)
            );
        }
        let mut b = body();
        b["answers"].as_object_mut().unwrap().remove("case_anchor");
        assert!(parse(b, q, &candidates()).is_err());
    }
    #[test]
    fn no_name_or_uncertain_name_preserves_scoped_fallback_without_manufactured_presence() {
        let mut b = body();
        b["answers"].as_object_mut().unwrap().remove("case_anchor");
        let d = parse(b, "¿qué ocurrió?", &candidates()).unwrap();
        assert_eq!(d.per_memory.unwrap().selected_ids(), ["m0", "m1"]);
        let mut b = body();
        b["answers"]["case_anchor"]["choice"] = json!("none");
        assert_eq!(
            parse(b, "¿Cuándo escribió Teo Luciérnaga?", &candidates())
                .unwrap()
                .per_memory
                .unwrap()
                .selected_ids(),
            ["m0", "m1"]
        );
    }
}
