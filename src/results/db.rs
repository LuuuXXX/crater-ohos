use crate::crates::Crate;
use crate::db::DatabasePool;
use crate::prelude::*;
use crate::results::{EncodedLog, TestResult};
use crate::toolchain::Toolchain;

pub struct DatabaseDB {
    pool: DatabasePool,
}

impl DatabaseDB {
    pub fn new(pool: DatabasePool) -> Self {
        DatabaseDB { pool }
    }

    pub fn store_result(
        &self,
        experiment: &str,
        krate: &Crate,
        toolchain: &Toolchain,
        result: &TestResult,
        log: Option<&EncodedLog>,
    ) -> Fallible<()> {
        let conn = self.pool.get()?;

        let krate_str = krate.to_string();
        let toolchain_str = toolchain.to_string();
        let result_json = serde_json::to_string(result)?;
        let log_bytes = log.map(|l| l.to_bytes().to_vec());

        conn.execute(
            "INSERT OR REPLACE INTO results (experiment, crate, toolchain, result, log) 
             VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![experiment, krate_str, toolchain_str, result_json, log_bytes],
        )?;

        Ok(())
    }

    pub fn get_result(
        &self,
        experiment: &str,
        krate: &Crate,
        toolchain: &Toolchain,
    ) -> Fallible<Option<(TestResult, Option<EncodedLog>)>> {
        let conn = self.pool.get()?;

        let krate_str = krate.to_string();
        let toolchain_str = toolchain.to_string();

        let result = conn.query_row(
            "SELECT result, log FROM results WHERE experiment = ? AND crate = ? AND toolchain = ?",
            rusqlite::params![experiment, krate_str, toolchain_str],
            |row| {
                let result_json: String = row.get(0)?;
                let log_bytes: Option<Vec<u8>> = row.get(1)?;
                Ok((result_json, log_bytes))
            },
        );

        match result {
            Ok((result_json, log_bytes)) => {
                let test_result: TestResult = serde_json::from_str(&result_json)?;
                let log = log_bytes.map(EncodedLog::from_plain);
                Ok(Some((test_result, log)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn delete_all_results(&self, experiment: &str) -> Fallible<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM results WHERE experiment = ?", [experiment])?;
        Ok(())
    }

    pub fn get_result_count(&self, experiment: &str) -> Fallible<usize> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM results WHERE experiment = ?",
            [experiment],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }
}

#[derive(Debug, Clone)]
pub struct ProgressData {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub skipped: usize,
}

impl ProgressData {
    pub fn new() -> Self {
        ProgressData {
            total: 0,
            completed: 0,
            failed: 0,
            skipped: 0,
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed as f64 / self.total as f64) * 100.0
        }
    }
}

impl Default for ProgressData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crates::sources::registry::RegistryCrate;
    use crate::db::create_memory_pool;
    use crate::toolchain::RustwideToolchain;

    #[test]
    fn test_store_and_get_result() {
        let pool = create_memory_pool().unwrap();
        let db = DatabaseDB::new(pool.clone());

        // Create a dummy experiment first to satisfy foreign key constraint
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO experiments (name, mode, cap_lints, priority, created_at, status, ignore_blacklist)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params!["exp1", "build-and-test", "warn", 0, "2024-01-01 00:00:00", "queued", 0],
        ).unwrap();
        drop(conn);

        let krate = Crate::Registry(RegistryCrate::new("test", "1.0.0"));
        let toolchain = Toolchain {
            source: RustwideToolchain::Dist("stable".to_string()),
            target: None,
            rustflags: None,
            rustdocflags: None,
            cargoflags: None,
            ci_try: false,
            patches: vec![],
        };

        let result = TestResult::TestPass;
        let log = EncodedLog::from_plain(b"test log".to_vec());

        db.store_result("exp1", &krate, &toolchain, &result, Some(&log))
            .unwrap();

        let (retrieved_result, retrieved_log) =
            db.get_result("exp1", &krate, &toolchain).unwrap().unwrap();

        assert_eq!(retrieved_result, TestResult::TestPass);
        assert!(retrieved_log.is_some());
        assert_eq!(retrieved_log.unwrap().decode().unwrap(), "test log");
    }

    #[test]
    fn test_get_nonexistent_result() {
        let pool = create_memory_pool().unwrap();
        let db = DatabaseDB::new(pool);

        let krate = Crate::Registry(RegistryCrate::new("nonexistent", "1.0.0"));
        let toolchain = Toolchain {
            source: RustwideToolchain::Dist("stable".to_string()),
            target: None,
            rustflags: None,
            rustdocflags: None,
            cargoflags: None,
            ci_try: false,
            patches: vec![],
        };

        let result = db.get_result("exp1", &krate, &toolchain).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_all_results() {
        let pool = create_memory_pool().unwrap();
        let db = DatabaseDB::new(pool.clone());

        // Create a dummy experiment first to satisfy foreign key constraint
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO experiments (name, mode, cap_lints, priority, created_at, status, ignore_blacklist)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params!["exp1", "build-and-test", "warn", 0, "2024-01-01 00:00:00", "queued", 0],
        ).unwrap();
        drop(conn);

        let krate = Crate::Registry(RegistryCrate::new("test", "1.0.0"));
        let toolchain = Toolchain {
            source: RustwideToolchain::Dist("stable".to_string()),
            target: None,
            rustflags: None,
            rustdocflags: None,
            cargoflags: None,
            ci_try: false,
            patches: vec![],
        };

        db.store_result("exp1", &krate, &toolchain, &TestResult::TestPass, None)
            .unwrap();

        let count = db.get_result_count("exp1").unwrap();
        assert_eq!(count, 1);

        db.delete_all_results("exp1").unwrap();

        let count = db.get_result_count("exp1").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_progress_data() {
        let mut progress = ProgressData::new();
        assert_eq!(progress.percentage(), 0.0);

        progress.total = 100;
        progress.completed = 50;
        assert_eq!(progress.percentage(), 50.0);

        progress.completed = 100;
        assert_eq!(progress.percentage(), 100.0);
    }
}
