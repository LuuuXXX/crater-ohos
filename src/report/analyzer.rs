use crate::report::{Comparison, CrateResult, RawTestResults};
use crate::results::TestResult;
use std::collections::HashMap;

/// Configuration for generating reports
#[derive(Debug, Clone)]
pub enum ReportConfig {
    Simple,
    Complete(ToolchainSelect),
}

/// Which toolchain to select for complete reports
#[derive(Debug, Clone, Copy)]
pub enum ToolchainSelect {
    Start,
    End,
}

/// Analyzed test results organized by category
#[derive(Serialize)]
pub struct TestResults {
    pub categories: HashMap<Comparison, Vec<CrateResult>>,
    pub summary: Summary,
}

#[derive(Serialize, Debug, Clone)]
pub struct Summary {
    pub total: usize,
    pub regressed: usize,
    pub fixed: usize,
    pub broken: usize,
    pub same_build_fail: usize,
    pub same_test_fail: usize,
    pub same_test_pass: usize,
    pub same_test_skipped: usize,
    pub skipped: usize,
    pub unknown: usize,
    pub errors: usize,
}

impl TestResults {
    /// Create TestResults from raw results
    pub fn from_raw(raw: RawTestResults) -> Self {
        let mut categories: HashMap<Comparison, Vec<CrateResult>> = HashMap::new();
        let mut summary = Summary {
            total: raw.crates.len(),
            regressed: 0,
            fixed: 0,
            broken: 0,
            same_build_fail: 0,
            same_test_fail: 0,
            same_test_pass: 0,
            same_test_skipped: 0,
            skipped: 0,
            unknown: 0,
            errors: 0,
        };

        for crate_result in raw.crates {
            // Update summary
            match crate_result.res {
                Comparison::Regressed => summary.regressed += 1,
                Comparison::Fixed => summary.fixed += 1,
                Comparison::Broken => summary.broken += 1,
                Comparison::SameBuildFail => summary.same_build_fail += 1,
                Comparison::SameTestFail => summary.same_test_fail += 1,
                Comparison::SameTestPass => summary.same_test_pass += 1,
                Comparison::SameTestSkipped => summary.same_test_skipped += 1,
                Comparison::Skipped => summary.skipped += 1,
                Comparison::Unknown => summary.unknown += 1,
                Comparison::Error => summary.errors += 1,
                _ => {}
            }

            // Add to category
            categories
                .entry(crate_result.res.clone())
                .or_insert_with(Vec::new)
                .push(crate_result);
        }

        TestResults {
            categories,
            summary,
        }
    }

    /// Get all results in a specific category
    pub fn get_category(&self, comparison: &Comparison) -> Option<&Vec<CrateResult>> {
        self.categories.get(comparison)
    }

    /// Get the total number of results
    pub fn total(&self) -> usize {
        self.summary.total
    }
}

/// Compare two test results and determine the comparison type
pub fn compare_results(
    start: &Option<TestResult>,
    end: &Option<TestResult>,
) -> Comparison {
    match (start, end) {
        (None, None) => Comparison::Skipped,
        (None, Some(_)) | (Some(_), None) => Comparison::Unknown,
        (Some(s), Some(e)) => {
            use TestResult::*;
            match (s, e) {
                // Both passed
                (TestPass, TestPass) => Comparison::SameTestPass,

                // Both failed in the same way
                (BuildFail(_), BuildFail(_)) => Comparison::SameBuildFail,
                (TestFail(_), TestFail(_)) => Comparison::SameTestFail,
                (TestSkipped, TestSkipped) => Comparison::SameTestSkipped,

                // Broken crates
                (BrokenCrate(_), _) | (_, BrokenCrate(_)) => Comparison::Broken,

                // Prepare failures
                (PrepareFail(_), _) | (_, PrepareFail(_)) => Comparison::PrepareFail,

                // Errors
                (Error, _) | (_, Error) => Comparison::Error,

                // Regression: passed -> failed
                (TestPass, BuildFail(_)) | (TestPass, TestFail(_)) => Comparison::Regressed,

                // Fix: failed -> passed
                (BuildFail(_), TestPass) | (TestFail(_), TestPass) => Comparison::Fixed,

                // Skipped
                (Skipped, _) | (_, Skipped) => Comparison::Skipped,

                // Everything else is unknown
                _ => Comparison::Unknown,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crates::Crate;
    use crate::results::FailureReason;

    #[test]
    fn test_compare_results_both_pass() {
        let result = compare_results(
            &Some(TestResult::TestPass),
            &Some(TestResult::TestPass),
        );
        assert_eq!(result, Comparison::SameTestPass);
    }

    #[test]
    fn test_compare_results_regressed() {
        let result = compare_results(
            &Some(TestResult::TestPass),
            &Some(TestResult::BuildFail(FailureReason::Unknown)),
        );
        assert_eq!(result, Comparison::Regressed);
    }

    #[test]
    fn test_compare_results_fixed() {
        let result = compare_results(
            &Some(TestResult::BuildFail(FailureReason::Unknown)),
            &Some(TestResult::TestPass),
        );
        assert_eq!(result, Comparison::Fixed);
    }

    #[test]
    fn test_compare_results_same_build_fail() {
        let result = compare_results(
            &Some(TestResult::BuildFail(FailureReason::Unknown)),
            &Some(TestResult::BuildFail(FailureReason::Timeout)),
        );
        assert_eq!(result, Comparison::SameBuildFail);
    }

    #[test]
    fn test_compare_results_skipped() {
        let result = compare_results(&None, &None);
        assert_eq!(result, Comparison::Skipped);
    }

    #[test]
    fn test_summary_counts() {
        let raw = RawTestResults {
            crates: vec![
                CrateResult {
                    name: "crate1".to_string(),
                    url: "".to_string(),
                    krate: Crate::Registry(crate::crates::sources::registry::RegistryCrate::new("test", "1.0.0")),
                    status: None,
                    res: Comparison::Regressed,
                    runs: [None, None],
                },
                CrateResult {
                    name: "crate2".to_string(),
                    url: "".to_string(),
                    krate: Crate::Registry(crate::crates::sources::registry::RegistryCrate::new("test2", "1.0.0")),
                    status: None,
                    res: Comparison::Fixed,
                    runs: [None, None],
                },
            ],
        };

        let results = TestResults::from_raw(raw);
        assert_eq!(results.summary.total, 2);
        assert_eq!(results.summary.regressed, 1);
        assert_eq!(results.summary.fixed, 1);
    }
}
