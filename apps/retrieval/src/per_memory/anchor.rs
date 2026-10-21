//! Literal witness for a model-chosen fixed case name, not semantic identity certification.
use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LiteralAnchor {
    pub query_token: Option<String>,
    pub present_in_source: Option<bool>,
}
pub fn query_names(query: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, t) in query.split_whitespace().enumerate() {
        if i == 0 {
            continue;
        }
        let s = t.trim_matches(|c: char| !c.is_alphanumeric());
        if s.chars().next().is_some_and(char::is_uppercase) && !out.iter().any(|x: &String| x == s)
        {
            out.push(s.into());
            if out.len() == 16 {
                break;
            }
        }
    }
    out
}
fn words(s: &str) -> std::collections::BTreeSet<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}
pub fn present(token: &str, source: &str) -> bool {
    let needed = words(token);
    !needed.is_empty() && needed.is_subset(&words(source))
}
impl LiteralAnchor {
    pub fn valid(&self, query: &str, source: &str) -> bool {
        self.query_token
            .as_ref()
            .is_none_or(|t| query_names(query).contains(t))
            && self.present_in_source == self.query_token.as_deref().map(|t| present(t, source))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_name_is_not_a_property_or_proof_of_identity() {
        assert_eq!(
            query_names("¿Cuándo escribió Teo Luciérnaga?"),
            vec!["Teo", "Luciérnaga"]
        );
        assert!(present(
            "Luciérnaga",
            "Sara escribió el libreto Luciérnaga."
        ));
        assert!(!present("Luciérnaga", "Teo diseñó Neblina."));
        assert!(
            LiteralAnchor {
                query_token: None,
                present_in_source: None
            }
            .valid("¿qué?", "implicit alias")
        );
    }
    #[test]
    fn invalid_query_names_or_fabricated_source_presence_fail() {
        let a = LiteralAnchor {
            query_token: Some("Orbe".into()),
            present_in_source: Some(true),
        };
        assert!(!a.valid("¿Peso Orbe?", "Vega pesa ocho."));
        assert!(!a.valid("¿Peso Vega?", "Orbe pesa ocho."));
        assert!(a.valid("¿Peso Orbe?", "ORBE pesa ocho."));
        assert!(!present("", "Orbe"));
    }
}
