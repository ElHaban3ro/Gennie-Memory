//! Frozen compatibility metadata only; never applies retired schema to new DBs.
use gennie_memory_kernel::digest::sha256_hex;
use rusqlite::Connection;

const DASHBOARD_V1: &str = include_str!("../migrations/retired_dashboard/0001_review.sql");

pub fn verify(conn: &Connection) -> Result<(), String> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='schema_migrations')",
        [], |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if !exists {
        return Ok(());
    }
    let mut query = conn
        .prepare("SELECT version, checksum FROM schema_migrations WHERE module='dashboard'")
        .map_err(|e| e.to_string())?;
    let rows = query
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    let expected = sha256_hex(DASHBOARD_V1.as_bytes());
    for row in rows {
        let (version, checksum) = row.map_err(|e| e.to_string())?;
        if version != 1 {
            return Err(format!(
                "migración histórica dashboard:{version} no reconocida"
            ));
        }
        if checksum != expected {
            return Err("la migración aplicada dashboard:1 cambió".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gennie_memory_kernel::core::{AppModule, Migration};
    struct Historical;
    impl AppModule for Historical {
        fn name(&self) -> &'static str {
            "dashboard"
        }
        fn migrations(&self) -> Vec<Migration> {
            vec![Migration {
                version: 1,
                sql: DASHBOARD_V1,
            }]
        }
    }
    #[test]
    fn fresh_database_never_creates_retired_tables() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::registry::migrate(&mut conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name='dashboard_reviews'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
        let modules = crate::registry::modules().unwrap();
        assert_eq!(
            modules.iter().map(|m| m.name()).collect::<Vec<_>>(),
            vec!["retrieval"]
        );
    }
    #[test]
    fn existing_reviews_are_preserved_but_checksum_stays_enforced() {
        assert_eq!(
            sha256_hex(DASHBOARD_V1.as_bytes()),
            "25f4c37d020ddee77d725221459c706aedc71427d52439c73c1a1eb6a8b044d2"
        );
        let mut conn = Connection::open_in_memory().unwrap();
        gennie_memory_kernel::db::migrate(&mut conn, &[Box::new(Historical)]).unwrap();
        conn.execute("INSERT INTO dashboard_reviews(query_key,query,scope_kind,scope_id,as_of,query_mode,candidates_json,decisions_json,outcome,created_at) VALUES ('k','q','project','p','2026-10-23T00:00:00.000Z','keywords','[]','[]','none','2026-10-23T00:00:00.000Z')",[]).unwrap();
        crate::registry::migrate(&mut conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM dashboard_reviews", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
        conn.execute(
            "UPDATE schema_migrations SET checksum='tampered' WHERE module='dashboard'",
            [],
        )
        .unwrap();
        assert!(
            crate::registry::migrate(&mut conn)
                .unwrap_err()
                .contains("dashboard:1 cambió")
        );
    }
}
