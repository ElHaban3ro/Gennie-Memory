//! Finite supply-graph satisfiability, not a natural-language role classifier.
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

type Edge = (usize, usize);
#[derive(Clone, Debug)]
struct Theory {
    nodes: Vec<String>,
    networks: BTreeSet<usize>,
    required: BTreeSet<Edge>,
    forbidden: BTreeSet<Edge>,
    immediate: Vec<(usize, BTreeSet<usize>)>,
    provenance: Vec<(usize, bool)>,
}
fn array<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    v.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("falta array {key}"))
}
fn keys(v: &Value, allowed: &[&str]) -> Result<(), String> {
    let object = v.as_object().ok_or("se esperaba objeto")?;
    if object.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("campo no permitido en teoría de flujo".into());
    }
    Ok(())
}
fn parse(v: &Value) -> Result<Theory, String> {
    keys(
        v,
        &[
            "nodes",
            "network_nodes",
            "required_edges",
            "forbidden_edges",
            "immediate_sources",
            "network_provenance",
        ],
    )?;
    let mut nodes = array(v, "nodes")?
        .iter()
        .map(|n| {
            n.as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .ok_or("nodo inválido".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let count = nodes.len();
    nodes.sort();
    nodes.dedup();
    if nodes.len() != count || nodes.is_empty() {
        return Err("nodos vacíos o duplicados".into());
    }
    // Bound parsing/allocation separately from the search-space limit.
    if nodes.len() > 8 {
        return Err("máximo ocho nodos para este diagnóstico".into());
    }
    let id = |x: &Value| -> Result<usize, String> {
        let s = x.as_str().ok_or("referencia de nodo inválida")?;
        nodes
            .binary_search_by(|n| n.as_str().cmp(s))
            .map_err(|_| format!("nodo desconocido: {s}"))
    };
    let edges = |key: &str| -> Result<BTreeSet<Edge>, String> {
        array(v, key)?
            .iter()
            .map(|e| {
                let a = e
                    .as_array()
                    .filter(|a| a.len() == 2)
                    .ok_or("arco debe tener dos nodos")?;
                let edge = (id(&a[0])?, id(&a[1])?);
                if edge.0 == edge.1 {
                    return Err("autoarco no permitido".into());
                }
                Ok(edge)
            })
            .collect()
    };
    let networks = array(v, "network_nodes")?
        .iter()
        .map(id)
        .collect::<Result<_, _>>()?;
    let required = edges("required_edges")?;
    let forbidden = edges("forbidden_edges")?;
    let immediate = array(v, "immediate_sources")?
        .iter()
        .map(|x| {
            keys(x, &["target", "allowed_sources"])?;
            Ok((
                id(&x["target"])?,
                array(x, "allowed_sources")?
                    .iter()
                    .map(id)
                    .collect::<Result<_, _>>()?,
            ))
        })
        .collect::<Result<_, String>>()?;
    let provenance = array(v, "network_provenance")?
        .iter()
        .map(|x| {
            keys(x, &["target", "required"])?;
            Ok((
                id(&x["target"])?,
                x["required"]
                    .as_bool()
                    .ok_or("required debe ser booleano")?,
            ))
        })
        .collect::<Result<_, String>>()?;
    Ok(Theory {
        nodes,
        networks,
        required,
        forbidden,
        immediate,
        provenance,
    })
}
fn path(t: &Theory, edges: &BTreeSet<Edge>, target: usize) -> Option<Vec<usize>> {
    // Start with an edge: provenance means a positive-length path, even in cycles.
    let mut queue = Vec::new();
    for &(from, to) in edges {
        if t.networks.contains(&from) {
            queue.push(vec![from, to]);
        }
    }
    let mut seen = BTreeSet::new();
    let mut cursor = 0;
    while cursor < queue.len() {
        let p = queue[cursor].clone();
        cursor += 1;
        let last = *p.last().unwrap();
        if last == target {
            return Some(p);
        }
        if !seen.insert(last) {
            continue;
        }
        for &(from, to) in edges {
            if from == last {
                let mut next = p.clone();
                next.push(to);
                queue.push(next);
            }
        }
    }
    None
}
fn solve(t: &Theory) -> Value {
    let permitted = |&(from, to): &Edge| {
        !t.forbidden.contains(&(from, to))
            && t.immediate
                .iter()
                .all(|(target, allowed)| to != *target || allowed.contains(&from))
    };
    if t.required.iter().any(|e| !permitted(e)) {
        return json!({"status":"no_model_in_finite_universe","checked_graphs":0});
    }
    let optional = (0..t.nodes.len())
        .flat_map(|from| (0..t.nodes.len()).map(move |to| (from, to)))
        .filter(|e| e.0 != e.1 && !t.required.contains(e) && permitted(e))
        .collect::<Vec<_>>();
    if optional.len() > 16 {
        return json!({"status":"unsupported_search_space","optional_edges":optional.len(),"limit":16});
    }
    let mut masks = (0u64..(1u64 << optional.len())).collect::<Vec<_>>();
    masks.sort_by_key(|m| (m.count_ones(), *m));
    let mut checked = 0;
    for mask in masks {
        let mut edges = t.required.clone();
        for (i, e) in optional.iter().enumerate() {
            if mask & (1 << i) != 0 {
                edges.insert(*e);
            }
        }
        checked += 1;
        if t.provenance
            .iter()
            .all(|(target, required)| path(t, &edges, *target).is_some() == *required)
        {
            let named = edges
                .iter()
                .map(|(a, b)| json!([t.nodes[*a], t.nodes[*b]]))
                .collect::<Vec<_>>();
            let paths=t.provenance.iter().filter(|(_,required)| *required).map(|(target,_)| json!({"target":t.nodes[*target],"path":path(t,&edges,*target).unwrap().iter().map(|i| &t.nodes[*i]).collect::<Vec<_>>()})).collect::<Vec<_>>();
            return json!({"status":"compatible","checked_graphs":checked,"witness_edges":named,"network_paths":paths});
        }
    }
    json!({"status":"no_model_in_finite_universe","checked_graphs":checked})
}
/// Accepts facts only. Invalid input and unrepresented search space are not `none` roles.
fn check(v: &Value) -> Result<Value, String> {
    Ok(solve(&parse(v)?))
}
fn required_text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("falta texto {key}"))
}
fn case_documentation(case: &Value) -> Result<(), String> {
    required_text(case, "interpretation")?;
    match case.get("quote") {
        Some(Value::Null) => Ok(()),
        Some(Value::String(s)) if !s.trim().is_empty() => Ok(()),
        _ => Err("quote debe ser texto o null explícito para control sin cita".into()),
    }
}
pub fn audit(input: &Path, output: &Path) -> Result<(), String> {
    let bytes = std::fs::read(input).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    keys(&v, &["version", "annotation_notice", "cases"])?;
    if v["version"] != "flow-compatibility-v1" {
        return Err("versión de flujo no admitida".into());
    }
    required_text(&v, "annotation_notice")?;
    let mut ids = BTreeSet::new();
    let cases = array(&v, "cases")?
        .iter()
        .map(|case| {
            keys(case, &["id", "quote", "interpretation", "theory"])?;
            case_documentation(case)?;
            let id = case["id"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or("falta id")?;
            if !ids.insert(id) {
                return Err("id duplicado".into());
            }
            Ok(json!({"id":id,"result":check(&case["theory"])?}))
        })
        .collect::<Result<Vec<_>, String>>()?;
    use sha2::{Digest, Sha256};
    let report = json!({"version":"flow-compatibility-report-v1","input_sha256":Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect::<String>(),"manual_formalization":true,"natural_language_ambiguity_resolved":false,"automatic_role_classifier":false,"provider_calls":0,"known_provider_cost_usd":0,"cases":cases});
    crate::scoped_definition_probe::write_new(output, &report)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> Value {
        json!({"nodes":["R","K","O"],"network_nodes":["R"],"required_edges":[["K","O"]],"forbidden_edges":[],"immediate_sources":[{"target":"O","allowed_sources":["K"]}],"network_provenance":[{"target":"O","required":true}]})
    }
    #[test]
    fn exclusive_direct_supplier_permits_upstream_network() {
        let r = check(&base()).unwrap();
        assert_eq!(r["status"], "compatible");
        assert_eq!(r["witness_edges"], json!([["K", "O"], ["R", "K"]]));
        assert_eq!(r["network_paths"][0]["path"], json!(["R", "K", "O"]));
    }
    #[test]
    fn explicit_no_network_blocks_even_indirect_provenance() {
        let mut v = base();
        v["network_provenance"]
            .as_array_mut()
            .unwrap()
            .push(json!({"target":"O","required":false}));
        assert_eq!(check(&v).unwrap()["status"], "no_model_in_finite_universe");
    }
    #[test]
    fn nonexclusive_supplier_is_compatible() {
        let mut v = base();
        v["immediate_sources"] = json!([]);
        assert_eq!(check(&v).unwrap()["status"], "compatible");
    }
    #[test]
    fn extra_origin_axiom_is_needed_to_remove_countermodel() {
        let mut v = base();
        v["network_provenance"]
            .as_array_mut()
            .unwrap()
            .push(json!({"target":"K","required":false}));
        assert_eq!(check(&v).unwrap()["status"], "no_model_in_finite_universe");
    }
    #[test]
    fn renaming_and_order_do_not_change_satisfiability() {
        let original = base();
        let renamed: Value = serde_json::from_str(
            &original
                .to_string()
                .replace("\"R\"", "\"Z\"")
                .replace("\"K\"", "\"A\"")
                .replace("\"O\"", "\"B\""),
        )
        .unwrap();
        assert_eq!(
            check(&original).unwrap()["status"],
            check(&renamed).unwrap()["status"]
        );
        let mut v = original.clone();
        v["nodes"].as_array_mut().unwrap().reverse();
        assert_eq!(check(&original).unwrap(), check(&v).unwrap());
    }
    #[test]
    fn invalid_facts_rejected_instead_of_interpreted_as_unrelated() {
        for nodes in [json!(["R", "R", "O"]), json!(["R", "O"])] {
            let mut v = base();
            v["nodes"] = nodes;
            assert!(check(&v).is_err());
        }
        let mut v = base();
        v["required_edges"] = json!([["K", "K"]]);
        assert!(check(&v).is_err());
        v = base();
        v["role"] = json!("B");
        assert!(check(&v).is_err());
    }
    #[test]
    fn excessive_search_is_explicitly_unsupported() {
        let mut v = base();
        v["nodes"] = json!(["R", "K", "O", "X", "Y", "Z"]);
        v["immediate_sources"] = json!([]);
        assert_eq!(check(&v).unwrap()["status"], "unsupported_search_space");
    }
    #[test]
    fn disconnected_cycle_does_not_create_network_provenance() {
        let mut t = parse(&base()).unwrap();
        t.required = BTreeSet::from([(0, 1), (1, 0)]);
        let network = *t.networks.first().unwrap();
        assert!(!t.required.iter().any(|e| e.0 == network));
        assert!(path(&t, &t.required, 0).is_none());
    }
    #[test]
    fn semantic_reading_documentation_is_required() {
        assert!(required_text(&json!({}), "annotation_notice").is_err());
        assert!(case_documentation(&json!({"quote":"text"})).is_err());
        assert!(case_documentation(&json!({"interpretation":"reading","quote":5})).is_err());
        assert!(case_documentation(&json!({"interpretation":"reading"})).is_err());
        assert!(case_documentation(&json!({"interpretation":"control","quote":null})).is_ok());
    }
    #[test]
    fn contradictory_edge_constraints_have_no_model() {
        let mut v = base();
        v["forbidden_edges"] = v["required_edges"].clone();
        assert_eq!(check(&v).unwrap()["status"], "no_model_in_finite_universe");
    }
}
