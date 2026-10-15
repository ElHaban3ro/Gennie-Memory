use crate::core::AppModule;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::Duration;

pub fn connect(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("no se pudo crear directorio de datos: {e}"))?;
        }
    }
    let conn = Connection::open(path).map_err(|e| format!("no se pudo abrir SQLite: {e}"))?;
    configure(&conn).map_err(|e| format!("no se pudo configurar SQLite: {e}"))?;
    Ok(conn)
}

pub fn configure(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(())
}

pub fn migrate(conn: &mut Connection, modules: &[Box<dyn AppModule>]) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
           module TEXT NOT NULL,
           version INTEGER NOT NULL,
           checksum TEXT NOT NULL,
           PRIMARY KEY (module, version)
         );",
    )
    .map_err(|e| e.to_string())?;
    for module in modules {
        let mut previous_version = 0;
        for migration in module.migrations() {
            if migration.version <= previous_version {
                return Err(format!(
                    "versiones de migración inválidas en {}",
                    module.name()
                ));
            }
            previous_version = migration.version;
            let checksum = Sha256::digest(migration.sql.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            let old: Option<String> = conn
                .query_row(
                    "SELECT checksum FROM schema_migrations WHERE module = ?1 AND version = ?2",
                    params![module.name(), migration.version],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            if let Some(old) = old {
                if old != checksum {
                    return Err(format!(
                        "la migración aplicada {}:{} cambió",
                        module.name(),
                        migration.version
                    ));
                }
                continue;
            }
            let tx = conn.transaction().map_err(|e| e.to_string())?;
            tx.execute_batch(migration.sql).map_err(|e| e.to_string())?;
            tx.execute(
                "INSERT INTO schema_migrations (module, version, checksum) VALUES (?1, ?2, ?3)",
                params![module.name(), migration.version, checksum],
            )
            .map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Migration;

    struct TestModule(&'static str);
    impl AppModule for TestModule {
        fn name(&self) -> &'static str {
            "test"
        }
        fn migrations(&self) -> Vec<Migration> {
            vec![Migration {
                version: 1,
                sql: self.0,
            }]
        }
    }

    struct TwoMigrations(&'static str);
    impl AppModule for TwoMigrations {
        fn name(&self) -> &'static str {
            "two"
        }
        fn migrations(&self) -> Vec<Migration> {
            vec![
                Migration {
                    version: 1,
                    sql: self.0,
                },
                Migration {
                    version: 2,
                    sql: "CREATE TABLE y (id INTEGER);",
                },
            ]
        }
    }

    #[test]
    fn applied_migration_rejects_changed_bytes() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(
            &mut conn,
            &[Box::new(TestModule("CREATE TABLE x (id INTEGER);"))],
        )
        .unwrap();
        migrate(
            &mut conn,
            &[Box::new(TestModule("CREATE TABLE x (id INTEGER);"))],
        )
        .unwrap();
        let error = migrate(
            &mut conn,
            &[Box::new(TestModule("CREATE TABLE x (id TEXT);"))],
        )
        .unwrap_err();
        assert!(error.contains("cambió"));
    }

    #[test]
    fn multiple_migrations_preserve_old_checksums() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(
            &mut conn,
            &[Box::new(TwoMigrations("CREATE TABLE x (id INTEGER);"))],
        )
        .unwrap();
        migrate(
            &mut conn,
            &[Box::new(TwoMigrations("CREATE TABLE x (id INTEGER);"))],
        )
        .unwrap();
        let error = migrate(
            &mut conn,
            &[Box::new(TwoMigrations("CREATE TABLE x (id TEXT);"))],
        )
        .unwrap_err();
        assert!(error.contains("two:1"));
    }
}
