pub mod migrations;

use crate::prelude::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params_from_iter, Connection, Row};
use std::path::Path;

pub type DatabasePool = Pool<SqliteConnectionManager>;

pub fn create_pool<P: AsRef<Path>>(db_path: P) -> Fallible<DatabasePool> {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(16)
        .build(manager)
        .context("failed to create database pool")?;

    // Run migrations
    {
        let conn = pool.get()?;
        migrations::execute(&conn)?;
    }

    Ok(pool)
}

pub fn create_memory_pool() -> Fallible<DatabasePool> {
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder()
        .max_size(16)
        .build(manager)
        .context("failed to create in-memory database pool")?;

    // Run migrations
    {
        let conn = pool.get()?;
        migrations::execute(&conn)?;
    }

    Ok(pool)
}

pub trait QueryUtils {
    fn execute_query<P>(&self, query: &str, params: P) -> Fallible<usize>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql;

    fn get_row<T, P, F>(&self, query: &str, params: P, f: F) -> Fallible<Option<T>>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
        F: FnOnce(&Row) -> rusqlite::Result<T>;

    fn query<T, P, F>(&self, query: &str, params: P, f: F) -> Fallible<Vec<T>>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
        F: FnMut(&Row) -> rusqlite::Result<T>;
}

impl QueryUtils for Connection {
    fn execute_query<P>(&self, query: &str, params: P) -> Fallible<usize>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
    {
        let mut stmt = self.prepare(query)?;
        let params: Vec<_> = params.into_iter().collect();
        let count = stmt.execute(params_from_iter(params))?;
        Ok(count)
    }

    fn get_row<T, P, F>(&self, query: &str, params: P, f: F) -> Fallible<Option<T>>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
        F: FnOnce(&Row) -> rusqlite::Result<T>,
    {
        let mut stmt = self.prepare(query)?;
        let params: Vec<_> = params.into_iter().collect();
        let mut rows = stmt.query(params_from_iter(params))?;

        if let Some(row) = rows.next()? {
            Ok(Some(f(row)?))
        } else {
            Ok(None)
        }
    }

    fn query<T, P, F>(&self, query: &str, params: P, mut f: F) -> Fallible<Vec<T>>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
        F: FnMut(&Row) -> rusqlite::Result<T>,
    {
        let mut stmt = self.prepare(query)?;
        let params: Vec<_> = params.into_iter().collect();
        let rows = stmt.query_map(params_from_iter(params), |row| f(row))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

pub struct Transaction<'a> {
    conn: &'a Connection,
    committed: bool,
}

impl<'a> Transaction<'a> {
    pub fn new(conn: &'a Connection) -> Fallible<Self> {
        conn.execute("BEGIN TRANSACTION", [])?;
        Ok(Transaction {
            conn,
            committed: false,
        })
    }

    pub fn commit(mut self) -> Fallible<()> {
        self.conn.execute("COMMIT", [])?;
        self.committed = true;
        Ok(())
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.conn.execute("ROLLBACK", []);
        }
    }
}

impl<'a> QueryUtils for Transaction<'a> {
    fn execute_query<P>(&self, query: &str, params: P) -> Fallible<usize>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
    {
        self.conn.execute_query(query, params)
    }

    fn get_row<T, P, F>(&self, query: &str, params: P, f: F) -> Fallible<Option<T>>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
        F: FnOnce(&Row) -> rusqlite::Result<T>,
    {
        self.conn.get_row(query, params, f)
    }

    fn query<T, P, F>(&self, query: &str, params: P, f: F) -> Fallible<Vec<T>>
    where
        P: IntoIterator,
        P::Item: rusqlite::ToSql,
        F: FnMut(&Row) -> rusqlite::Result<T>,
    {
        self.conn.query(query, params, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_memory_pool() {
        let pool = create_memory_pool().expect("failed to create memory pool");
        let conn = pool.get().expect("failed to get connection");

        // Verify migrations ran
        let tables: Vec<String> = conn
            .query(
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
                std::iter::empty::<&dyn rusqlite::ToSql>(),
                |row| row.get(0),
            )
            .expect("failed to query tables");

        assert!(tables.contains(&"experiments".to_string()));
        assert!(tables.contains(&"experiment_metadata".to_string()));
        assert!(tables.contains(&"migrations".to_string()));
    }

    #[test]
    fn test_query_utils() {
        let pool = create_memory_pool().expect("failed to create memory pool");
        let conn = pool.get().expect("failed to get connection");

        // Test execute_query
        let count = conn
            .execute_query(
                "INSERT INTO migrations (name, executed_at) VALUES (?, ?)",
                ["test_migration", "2024-01-01 00:00:00"],
            )
            .expect("failed to execute query");
        assert_eq!(count, 1);

        // Test get_row
        let name: Option<String> = conn
            .get_row(
                "SELECT name FROM migrations WHERE name = ?",
                ["test_migration"],
                |row| row.get(0),
            )
            .expect("failed to get row");
        assert_eq!(name, Some("test_migration".to_string()));

        // Test query
        let names: Vec<String> = conn
            .query(
                "SELECT name FROM migrations",
                std::iter::empty::<&dyn rusqlite::ToSql>(),
                |row| row.get(0),
            )
            .expect("failed to query");
        assert!(names.len() >= 1);
    }

    #[test]
    fn test_transaction_commit() {
        let pool = create_memory_pool().expect("failed to create memory pool");
        let conn = pool.get().expect("failed to get connection");

        {
            let tx = Transaction::new(&conn).expect("failed to create transaction");
            tx.execute_query(
                "INSERT INTO migrations (name, executed_at) VALUES (?, ?)",
                ["tx_test", "2024-01-01 00:00:00"],
            )
            .expect("failed to insert");
            tx.commit().expect("failed to commit");
        }

        let name: Option<String> = conn
            .get_row("SELECT name FROM migrations WHERE name = ?", ["tx_test"], |row| {
                row.get(0)
            })
            .expect("failed to get row");
        assert_eq!(name, Some("tx_test".to_string()));
    }

    #[test]
    fn test_transaction_rollback() {
        let pool = create_memory_pool().expect("failed to create memory pool");
        let conn = pool.get().expect("failed to get connection");

        {
            let tx = Transaction::new(&conn).expect("failed to create transaction");
            tx.execute_query(
                "INSERT INTO migrations (name, executed_at) VALUES (?, ?)",
                ["rollback_test", "2024-01-01 00:00:00"],
            )
            .expect("failed to insert");
            // Don't commit, let it drop
        }

        let name: Option<String> = conn
            .get_row(
                "SELECT name FROM migrations WHERE name = ?",
                ["rollback_test"],
                |row| row.get(0),
            )
            .expect("failed to get row");
        assert_eq!(name, None);
    }
}
