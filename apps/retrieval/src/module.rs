use iromem_kernel::core::{AppModule, Migration};

pub struct RetrievalModule;

impl AppModule for RetrievalModule {
    fn name(&self) -> &'static str {
        "retrieval"
    }

    fn migrations(&self) -> Vec<Migration> {
        vec![
            Migration {
                version: 1,
                sql: include_str!("../migrations/0001_initial.sql"),
            },
            Migration {
                version: 2,
                sql: include_str!("../migrations/0002_embeddings.sql"),
            },
            Migration {
                version: 3,
                sql: include_str!("../migrations/0003_entity_filter.sql"),
            },
            Migration {
                version: 4,
                sql: include_str!("../migrations/0004_seed_provenance.sql"),
            },
        ]
    }
}
