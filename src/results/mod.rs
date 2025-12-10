mod db;
#[cfg(test)]
mod dummy;

use crate::crates::Crate;
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub use crate::results::db::{DatabaseDB, ProgressData};
#[cfg(test)]
pub use crate::results::dummy::DummyDB;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestResult {
    BrokenCrate(BrokenReason),
    PrepareFail(FailureReason),
    BuildFail(FailureReason),
    TestFail(FailureReason),
    TestSkipped,
    TestPass,
    Skipped,
    Error,
}

impl TestResult {
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            TestResult::BuildFail(_) | TestResult::TestFail(_)
        )
    }

    pub fn is_success(&self) -> bool {
        matches!(self, TestResult::TestPass)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureReason {
    Unknown,
    OOM,
    NoSpace,
    Timeout,
    ICE,
    NetworkAccess,
    Docker,
    CompilerDiagnosticChange,
    CompilerError(BTreeSet<DiagnosticCode>),
    DependsOn(BTreeSet<Crate>),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct DiagnosticCode(pub String);

impl DiagnosticCode {
    pub fn new(code: &str) -> Self {
        DiagnosticCode(code.to_string())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrokenReason {
    Unknown,
    CargoToml,
    Yanked,
    MissingDependencies,
    MissingGitRepository,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EncodedLog {
    Plain(Vec<u8>),
    Gzip(Vec<u8>),
}

impl EncodedLog {
    pub fn to_bytes(&self) -> &[u8] {
        match self {
            EncodedLog::Plain(data) => data,
            EncodedLog::Gzip(data) => data,
        }
    }

    pub fn encoding_type(&self) -> EncodingType {
        match self {
            EncodedLog::Plain(_) => EncodingType::Plain,
            EncodedLog::Gzip(_) => EncodingType::Gzip,
        }
    }

    pub fn from_plain_slice(data: &[u8]) -> Self {
        EncodedLog::Plain(data.to_vec())
    }

    pub fn from_plain(data: Vec<u8>) -> Self {
        EncodedLog::Plain(data)
    }

    pub fn from_gzip(data: Vec<u8>) -> Self {
        EncodedLog::Gzip(data)
    }

    pub fn decode(&self) -> Fallible<String> {
        match self {
            EncodedLog::Plain(data) => Ok(String::from_utf8_lossy(data).to_string()),
            EncodedLog::Gzip(data) => {
                use flate2::read::GzDecoder;
                use std::io::Read;

                let mut decoder = GzDecoder::new(&data[..]);
                let mut result = String::new();
                decoder.read_to_string(&mut result)?;
                Ok(result)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EncodingType {
    Plain,
    Gzip,
}

impl EncodingType {
    pub fn to_str(&self) -> &'static str {
        match self {
            EncodingType::Plain => "plain",
            EncodingType::Gzip => "gzip",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_result_serialization() {
        let result = TestResult::TestPass;
        let json = serde_json::to_string(&result).unwrap();
        let parsed: TestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TestResult::TestPass);

        let result = TestResult::BuildFail(FailureReason::Timeout);
        let json = serde_json::to_string(&result).unwrap();
        let parsed: TestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TestResult::BuildFail(FailureReason::Timeout));
    }

    #[test]
    fn test_test_result_is_failure() {
        assert!(TestResult::BuildFail(FailureReason::Unknown).is_failure());
        assert!(TestResult::TestFail(FailureReason::Timeout).is_failure());
        assert!(!TestResult::TestPass.is_failure());
        assert!(!TestResult::TestSkipped.is_failure());
    }

    #[test]
    fn test_test_result_is_success() {
        assert!(TestResult::TestPass.is_success());
        assert!(!TestResult::BuildFail(FailureReason::Unknown).is_success());
        assert!(!TestResult::TestSkipped.is_success());
    }

    #[test]
    fn test_encoded_log_plain() {
        let log = EncodedLog::from_plain(b"test log".to_vec());
        assert_eq!(log.to_bytes(), b"test log");
        assert_eq!(log.decode().unwrap(), "test log");
    }

    #[test]
    fn test_encoded_log_gzip() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"test log").unwrap();
        let compressed = encoder.finish().unwrap();

        let log = EncodedLog::from_gzip(compressed);
        assert_eq!(log.decode().unwrap(), "test log");
    }

    #[test]
    fn test_diagnostic_code() {
        let code = DiagnosticCode::new("E0308");
        assert_eq!(code.0, "E0308");

        let json = serde_json::to_string(&code).unwrap();
        let parsed: DiagnosticCode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.0, "E0308");
    }
}
