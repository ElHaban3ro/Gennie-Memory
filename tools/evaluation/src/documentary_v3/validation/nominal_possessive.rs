//! Closed lexical-use shape and unique incidence admission.
use super::*;
use crate::documentary_v3::nominal_possessive::KIND;
pub(super) fn node(c: &mut Check<'_>, t: &str, v: &Value, p: &str, depth: usize) -> bool {
    if t != "Link" || v["kind"] != KIND {
        return false;
    }
    if !super::super::request_context_profile::permits(c.vocab, KIND) {
        c.issue("invalid_shape", p);
        return true;
    }
    c.shape(v, p, &["kind", "target", "evidence"]);
    c.string(&v["target"], &format!("{p}/target"));
    if array(&v["evidence"]).len() != 1 {
        c.issue("invalid_shape", p);
    }
    c.list(&v["evidence"], &format!("{p}/evidence"), "Evidence", depth);
    true
}
fn span(v: &Value) -> Option<(u64, u64)> {
    Some((v["start_byte"].as_u64()?, v["end_byte"].as_u64()?))
}
fn contains(a: &Value, b: &Value) -> bool {
    span(a)
        .zip(span(b))
        .is_some_and(|((a, b), (c, d))| a <= c && c < d && d <= b)
}
fn prenominal_token(text: &str, owner: &Value, determiner: &Value, head: &Value) -> bool {
    let Some(((owner_start, _), ((start, end), (head_start, _)))) =
        span(owner).zip(span(determiner).zip(span(head)))
    else {
        return false;
    };
    if end >= head_start || owner_start > start || !super::super::context_budget::charge(1, 0) {
        return false;
    }
    let (Ok(owner_start), Ok(start), Ok(end), Ok(head_start)) = (
        usize::try_from(owner_start),
        usize::try_from(start),
        usize::try_from(end),
        usize::try_from(head_start),
    ) else {
        return false;
    };
    let (Some(before), Some(after)) = (text.get(owner_start..start), text.get(end..head_start))
    else {
        return false;
    };
    before
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_alphanumeric())
        && after.chars().next().is_some_and(char::is_whitespace)
}
pub(super) fn document(c: &mut Check<'_>, raw: &Value) {
    if !super::super::request_context_profile::permits(c.vocab, KIND) {
        return;
    }
    for r in charged(array(&raw["references"])) {
        let p = format!("/reference/{}", path_id(r["id"].as_str().unwrap_or("")));
        if !super::super::context_budget::charge(1, array(&r["links"]).len().saturating_mul(16)) {
            return;
        }
        let links: Vec<_> = charged(array(&r["links"]))
            .filter(|l| l["kind"] == KIND)
            .collect();
        if links.len() > 1 {
            c.issue("unsupported_nominal_possessive", &p);
        }
        if !links.is_empty() {
            if let Some(aliases) =
                super::super::nominal_possessive::component(raw, r["id"].as_str().unwrap_or(""))
            {
                let incidences = charged(array(&raw["references"]))
                    .filter(|a| aliases.contains(a["id"].as_str().unwrap_or("")))
                    .map(|a| {
                        charged(array(&a["links"]))
                            .filter(|l| l["kind"] == KIND)
                            .count()
                    })
                    .sum::<usize>();
                if incidences != 1 {
                    c.issue("unsupported_nominal_possessive", &p);
                }
            }
        }
        for l in links {
            let Some(u) = charged(array(&raw["references"])).find(|u| u["id"] == l["target"])
            else {
                c.issue("dangling_id", &p);
                continue;
            };
            let head = if !r["declared_class"].is_null() {
                &r["declared_class"]
            } else {
                &r["name"]
            };
            if r["denotation"] != "entity"
                || u["denotation"] != "lexical_use"
                || r["scope_id"] != u["scope_id"]
                || !contains(&r["mention"], head)
                || !contains(&r["mention"], &u["mention"])
                || !prenominal_token(c.text, &r["mention"], &u["mention"], head)
                || l["evidence"][0] != r["mention"]
            {
                c.issue("invalid_shape", &p);
            }
        }
        if r["denotation"] != "lexical_use" {
            continue;
        }
        if !super::super::context_budget::charge(
            1,
            r["mention"]["quote"]
                .as_str()
                .map_or(0, str::len)
                .saturating_mul(3),
        ) {
            return;
        }
        let determiner = r["mention"]["quote"].as_str().unwrap_or("").to_lowercase();
        if !matches!(
            determiner.as_str(),
            "mi" | "mis"
                | "tu"
                | "tus"
                | "su"
                | "sus"
                | "nuestro"
                | "nuestra"
                | "nuestros"
                | "nuestras"
                | "vuestro"
                | "vuestra"
                | "vuestros"
                | "vuestras"
        ) || [
            "name",
            "declared_class",
            "semantic_type",
            "binding_scope_id",
            "anaphora",
        ]
        .iter()
        .any(|k| !r[*k].is_null())
            || !array(&r["links"]).is_empty()
        {
            c.issue("invalid_shape", &p);
        }
        let mut owners = 0usize;
        for other in charged(array(&raw["references"])) {
            for l in charged(array(&other["links"])) {
                if l["target"] == r["id"] {
                    if l["kind"] == KIND {
                        owners += 1;
                    } else {
                        c.issue("invalid_shape", &p);
                    }
                }
            }
        }
        if owners != 1 {
            c.issue("unsupported_nominal_possessive", &p);
        }
        // All semantic occurrences are forbidden; only owning link and own
        // scope dependency can mention a lexical target.
        let rid = r["id"].as_str().unwrap_or("");
        for table in ["identity", "claims", "requests", "qualifiers", "scopes"] {
            for owner in charged(array(&raw[table])) {
                let mut used = BTreeSet::new();
                refs(owner, &mut used);
                if used.contains(rid) {
                    c.issue("invalid_shape", &p);
                }
            }
        }
        for other in charged(array(&raw["references"])) {
            if other["id"] != r["id"] {
                let mut used = BTreeSet::new();
                refs(&other["anaphora"], &mut used);
                if used.contains(rid) {
                    c.issue("invalid_shape", &p);
                }
            }
        }
        for d in charged(array(&raw["dependencies"])) {
            if d["consumer"]["kind"] == "reference"
                && d["consumer"]["id"] == r["id"]
                && !(d["requires"]["kind"] == "scope"
                    && d["requires"]["id"] == r["scope_id"]
                    && d["role"] == "binding")
            {
                c.issue("invalid_shape", &p);
            }
            if d["requires"]["kind"] == "reference" && d["requires"]["id"] == r["id"] {
                let owner =
                    charged(array(&raw["references"])).find(|o| o["id"] == d["consumer"]["id"]);
                if d["consumer"]["kind"] != "reference"
                    || d["role"] != "identity"
                    || !owner.is_some_and(|o| {
                        array(&o["links"])
                            .iter()
                            .any(|l| l["kind"] == KIND && l["target"] == r["id"])
                    })
                {
                    c.issue("invalid_shape", &p);
                }
            }
        }
    }
}
