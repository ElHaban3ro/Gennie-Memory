pub trait AppModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn active_by_default(&self) -> bool {
        true
    }
    fn depends_on(&self) -> &'static [&'static str] {
        &[]
    }
    fn optional_dependencies(&self) -> &'static [&'static str] {
        &[]
    }
    fn migrations(&self) -> Vec<Migration> {
        Vec::new()
    }
    fn start(&self) -> Result<(), String> {
        Ok(())
    }
    fn stop(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Migration {
    pub version: i64,
    pub sql: &'static str,
}
