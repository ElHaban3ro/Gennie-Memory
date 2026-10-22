//! Frozen utility analogies only. This is not a new source of grounding labels.
use super::contribution_role_contract as roles;
use serde_json::{Value, json};
pub fn examples() -> Value {
    roles::payload_local(
        "unused",
        &[json!({"id":"unused","revision":1,"text":"unused"})],
    )["state"]["contribution_examples"]
        .clone()
}
pub fn payload(mut full: Value, candidate: bool) -> Value {
    if candidate {
        full["state"]["contribution_examples"] = examples();
        full["state"]["contribution_examples_scope"] = json!(
            "These frozen [question, assertion, role] triplets illustrate ONLY the conditional utility role of an assertion assumed true. They are not source-to-proposition grounding judgments, not evidence for this query, and do not set a quota. Apply grounding to the current source and proposition independently."
        );
    }
    full
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_only_exact_frozen_utility_triplets_and_their_scope() {
        let f = json!({"model":"same","state":{"question":"q","source":"literal","propositions":{"claim_00":"a"}},"questions":{"claim_00_grounded":"unchanged","claim_00_role":"unchanged"}});
        assert_eq!(payload(f.clone(), false), f);
        let mut p = payload(f.clone(), true);
        assert_eq!(
            p["state"]["contribution_examples"]
                .as_array()
                .unwrap()
                .len(),
            8
        );
        assert_eq!(p["state"]["contribution_examples"], examples());
        p["state"]
            .as_object_mut()
            .unwrap()
            .remove("contribution_examples");
        p["state"]
            .as_object_mut()
            .unwrap()
            .remove("contribution_examples_scope");
        assert_eq!(p, f);
    }
}
