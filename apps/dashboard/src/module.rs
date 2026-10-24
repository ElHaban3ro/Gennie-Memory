use gennie_memory_kernel::core::{AppModule, Migration};

pub struct DashboardModule;

impl AppModule for DashboardModule {
    fn name(&self) -> &'static str {
        "dashboard"
    }
    fn depends_on(&self) -> &'static [&'static str] {
        &["retrieval"]
    }
    fn migrations(&self) -> Vec<Migration> {
        vec![Migration {
            version: 1,
            sql: include_str!("../migrations/0001_review.sql"),
        }]
    }
}
