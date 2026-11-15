//! Schema migrations: embedded SQL, numbered and applied in order (spec 02).

use rusqlite::Connection;

/// Applies the migrations after the one in `PRAGMA user_version`. Migration
/// `i` of the slice takes the schema to version `i + 1`.
///
/// A published migration is never edited: a fix is a new migration.
pub fn migrate(conn: &mut Connection, migrations: &[&str]) -> rusqlite::Result<()> {
    let applied = schema_version(conn)?;
    let pending = (1..)
        .zip(migrations)
        .skip_while(|(version, _)| *version <= applied);
    for (version, sql) in pending {
        apply(conn, version, sql)?;
    }
    Ok(())
}

/// One migration and its `user_version` in a single transaction: a failure
/// leaves the database at the previous version.
fn apply(conn: &mut Connection, version: i64, sql: &str) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute_batch(sql)?;
    tx.pragma_update(None, "user_version", version)?;
    tx.commit()
}

/// The schema version recorded in `PRAGMA user_version`.
pub fn schema_version(conn: &Connection) -> rusqlite::Result<i64> {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user_version(conn: &Connection) -> i64 {
        conn.pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap()
    }

    fn tables(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    #[test]
    fn migrate_applies_pending_migrations_in_order() {
        let mut conn = Connection::open_in_memory().unwrap();

        migrate(
            &mut conn,
            &["CREATE TABLE a (x)", "ALTER TABLE a RENAME TO b"],
        )
        .unwrap();

        assert_eq!(
            (user_version(&conn), tables(&conn)),
            (2, vec!["b".to_string()])
        );
    }

    #[test]
    fn migrate_skips_already_applied_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, &["CREATE TABLE a (x)"]).unwrap();

        migrate(&mut conn, &["CREATE TABLE a (x)", "CREATE TABLE b (y)"]).unwrap();

        assert_eq!(
            (user_version(&conn), tables(&conn)),
            (2, vec!["a".into(), "b".into()])
        );
    }

    #[test]
    fn failed_migration_keeps_database_at_previous_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        let broken = "CREATE TABLE b (y); INSERT INTO tabela_que_nao_existe VALUES (1);";

        let result = migrate(&mut conn, &["CREATE TABLE a (x)", broken]);

        assert!(result.is_err());
        assert_eq!(
            (user_version(&conn), tables(&conn)),
            (1, vec!["a".to_string()])
        );
    }
}
