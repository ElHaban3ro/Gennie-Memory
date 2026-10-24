use gennie_memory_kernel::core::{AppModule, Migration};

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
            Migration {
                version: 5,
                sql: include_str!("../migrations/0005_write_operations.sql"),
            },
            Migration {
                version: 6,
                sql: include_str!("../migrations/0006_exact_dedup_and_evidence.sql"),
            },
            Migration {
                version: 7,
                sql: include_str!("../migrations/0007_revision_write_outcomes.sql"),
            },
            Migration {
                version: 8,
                sql: include_str!("../migrations/0008_retrieval_isolation.sql"),
            },
            Migration {
                version: 9,
                sql: include_str!("../migrations/0009_local_embedding_indexes.sql"),
            },
            Migration {
                version: 10,
                sql: include_str!("../migrations/0010_source_verification.sql"),
            },
            Migration {
                version: 11,
                sql: include_str!("../migrations/0011_write_reconciliation.sql"),
            },
            Migration {
                version: 12,
                sql: include_str!("../migrations/0012_memory_lifecycle.sql"),
            },
        ]
    }
}
