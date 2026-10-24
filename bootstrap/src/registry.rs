use gennie_memory_dashboard::module::DashboardModule;
use gennie_memory_kernel::core::AppModule;
use gennie_memory_retrieval::module::RetrievalModule;
use std::collections::{HashMap, HashSet};

pub fn modules() -> Result<Vec<Box<dyn AppModule>>, String> {
    order(vec![Box::new(RetrievalModule), Box::new(DashboardModule)])
}

fn order(input: Vec<Box<dyn AppModule>>) -> Result<Vec<Box<dyn AppModule>>, String> {
    let names: HashSet<&str> = input.iter().map(|module| module.name()).collect();
    if names.len() != input.len() {
        return Err("nombre de módulo duplicado".into());
    }
    let mut remaining: HashMap<&str, Box<dyn AppModule>> = input
        .into_iter()
        .map(|module| (module.name(), module))
        .collect();
    let mut result = Vec::new();
    while !remaining.is_empty() {
        let next = remaining
            .iter()
            .filter(|(_, module)| {
                module.depends_on().iter().all(|name| {
                    result
                        .iter()
                        .any(|done: &Box<dyn AppModule>| done.name() == *name)
                })
            })
            .map(|(name, _)| *name)
            .min();
        let Some(name) = next else {
            let missing: Vec<_> = remaining
                .values()
                .flat_map(|module| module.depends_on())
                .filter(|name| !names.contains(**name))
                .copied()
                .collect();
            return if missing.is_empty() {
                Err("ciclo de dependencias".into())
            } else {
                Err(format!(
                    "dependencia de módulo ausente: {}",
                    missing.join(", ")
                ))
            };
        };
        result.push(remaining.remove(name).expect("nombre presente"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Module(&'static str, &'static [&'static str]);
    impl AppModule for Module {
        fn name(&self) -> &'static str {
            self.0
        }
        fn depends_on(&self) -> &'static [&'static str] {
            self.1
        }
    }

    #[test]
    fn orders_dependencies_and_rejects_invalid_graph() {
        let ordered = order(vec![
            Box::new(Module("consumer", &["source"])),
            Box::new(Module("source", &[])),
        ])
        .unwrap();
        assert_eq!(
            ordered.iter().map(|m| m.name()).collect::<Vec<_>>(),
            vec!["source", "consumer"]
        );
        assert!(
            order(vec![Box::new(Module("a", &["missing"]))])
                .err()
                .unwrap()
                .contains("ausente")
        );
        assert!(
            order(vec![
                Box::new(Module("a", &["b"])),
                Box::new(Module("b", &["a"]))
            ])
            .err()
            .unwrap()
            .contains("ciclo")
        );
    }
}
