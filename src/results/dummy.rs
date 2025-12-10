use crate::crates::Crate;
use crate::results::{EncodedLog, TestResult};
use crate::toolchain::Toolchain;
use std::collections::HashMap;

/// Dummy database for testing purposes
pub struct DummyDB {
    results: HashMap<String, (TestResult, Option<EncodedLog>)>,
}

impl DummyDB {
    pub fn new() -> Self {
        DummyDB {
            results: HashMap::new(),
        }
    }

    fn make_key(experiment: &str, krate: &Crate, toolchain: &Toolchain) -> String {
        format!("{}:{}:{}", experiment, krate.id(), toolchain)
    }

    pub fn store_result(
        &mut self,
        experiment: &str,
        krate: &Crate,
        toolchain: &Toolchain,
        result: TestResult,
        log: Option<EncodedLog>,
    ) {
        let key = Self::make_key(experiment, krate, toolchain);
        self.results.insert(key, (result, log));
    }

    pub fn get_result(
        &self,
        experiment: &str,
        krate: &Crate,
        toolchain: &Toolchain,
    ) -> Option<(TestResult, Option<EncodedLog>)> {
        let key = Self::make_key(experiment, krate, toolchain);
        self.results.get(&key).cloned()
    }

    pub fn delete_all_results(&mut self, experiment: &str) {
        let prefix = format!("{}:", experiment);
        self.results.retain(|k, _| !k.starts_with(&prefix));
    }

    pub fn get_result_count(&self, experiment: &str) -> usize {
        let prefix = format!("{}:", experiment);
        self.results.keys().filter(|k| k.starts_with(&prefix)).count()
    }
}

impl Default for DummyDB {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crates::sources::registry::RegistryCrate;
    use crate::toolchain::RustwideToolchain;

    #[test]
    fn test_dummy_db_store_and_get() {
        let mut db = DummyDB::new();

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

        db.store_result(
            "exp1",
            &krate,
            &toolchain,
            TestResult::TestPass,
            Some(EncodedLog::from_plain(b"test log".to_vec())),
        );

        let (result, log) = db.get_result("exp1", &krate, &toolchain).unwrap();
        assert_eq!(result, TestResult::TestPass);
        assert!(log.is_some());
    }

    #[test]
    fn test_dummy_db_delete_all() {
        let mut db = DummyDB::new();

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

        db.store_result("exp1", &krate, &toolchain, TestResult::TestPass, None);
        db.store_result("exp2", &krate, &toolchain, TestResult::TestPass, None);

        assert_eq!(db.get_result_count("exp1"), 1);
        assert_eq!(db.get_result_count("exp2"), 1);

        db.delete_all_results("exp1");

        assert_eq!(db.get_result_count("exp1"), 0);
        assert_eq!(db.get_result_count("exp2"), 1);
    }
}
