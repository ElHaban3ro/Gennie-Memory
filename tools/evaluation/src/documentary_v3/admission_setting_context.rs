//! Restrictive setting context for context-only admission evidence.
use super::{Engine, Relation, id, rows, s};
use serde_json::{Value, json};
impl Engine<'_> {
    pub(super) fn admission_setting_context(
        &self,
        actual: &Value,
        wanted: &Value,
        same_value: bool,
    ) -> bool {
        self.admission_setting_context_depth(actual, wanted, same_value, 0)
    }
    fn admission_setting_context_depth(
        &self,
        actual: &Value,
        wanted: &Value,
        same_value: bool,
        depth: usize,
    ) -> bool {
        if depth > 8 {
            return false;
        }
        if same_value
            && s(actual, "kind") == s(wanted, "kind")
            && self.term(self.source, actual, self.question, wanted) != Relation::Same
        {
            return false;
        }
        let links = |doc: &Value, term: &Value| -> Option<Vec<Value>> {
            if s(term, "kind") == "text" {
                return Some(vec![]);
            }
            if s(term, "kind") != "ref" {
                return None;
            }
            let reference = self.reference(doc, s(term, "id"))?;
            if !reference["anaphora"].is_null()
                || !reference["binding_scope_id"].is_null()
                || self.local(doc, id(reference), id(reference)) != Relation::Same
            {
                return None;
            }
            let aliases: Vec<_> = rows(doc, "references")
                .iter()
                .filter(|alias| self.local(doc, id(reference), id(alias)) == Relation::Same)
                .collect();
            if aliases.iter().any(|alias| {
                !alias["anaphora"].is_null()
                    || !alias["binding_scope_id"].is_null()
                    || !self
                        .dependency_missing(doc, "reference", id(alias))
                        .is_empty()
            }) {
                return None;
            }
            Some(
                aliases
                    .into_iter()
                    .flat_map(|alias| rows(alias, "links").iter().cloned())
                    .collect(),
            )
        };
        let (Some(actual_links), Some(wanted_links)) =
            (links(self.source, actual), links(self.question, wanted))
        else {
            return false;
        };
        // Compare every directed restriction, including those on local aliases.
        // A bare equal label cannot discharge a context named only on one side.
        actual_links.iter().all(|a| {
            wanted_links.iter().any(|b| {
                a["kind"] == b["kind"]
                    && self.admission_setting_context_depth(
                        &json!({"kind":"ref","id":a["target"]}),
                        &json!({"kind":"ref","id":b["target"]}),
                        true,
                        depth + 1,
                    )
            })
        }) && wanted_links.iter().all(|b| {
            actual_links.iter().any(|a| {
                a["kind"] == b["kind"]
                    && self.admission_setting_context_depth(
                        &json!({"kind":"ref","id":a["target"]}),
                        &json!({"kind":"ref","id":b["target"]}),
                        true,
                        depth + 1,
                    )
            })
        })
    }
}
