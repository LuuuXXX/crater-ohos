use crate::config::Config;
use crate::crates::Crate;
use crate::experiments::Experiment;
use crate::prelude::*;
use crate::results::TestResult;
use mime::Mime;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

mod analyzer;
mod archives;
mod display;
mod html;
mod markdown;

pub use self::display::{Color, ResultColor, ResultName};
pub use analyzer::TestResults;

/// Raw test results before analysis
#[derive(Serialize, Deserialize)]
pub struct RawTestResults {
    pub crates: Vec<CrateResult>,
}

/// Result for a single crate
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct CrateResult {
    pub name: String,
    pub url: String,
    pub krate: Crate,
    pub status: Option<String>,
    pub res: Comparison,
    pub runs: [Option<RunResult>; 2],
}

/// Result of a single run (one toolchain)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct RunResult {
    pub result: TestResult,
    pub log: String,
}

string_enum! {
    pub enum Comparison {
        Regressed => "regressed",
        Fixed => "fixed",
        Skipped => "skipped",
        Unknown => "unknown",
        Error => "error",
        Broken => "broken",
        PrepareFail => "prepare-fail",
        SameBuildFail => "build-fail",
        SameTestFail => "test-fail",
        SameTestSkipped => "test-skipped",
        SameTestPass => "test-pass",
        SpuriousRegressed => "spurious-regressed",
        SpuriousFixed => "spurious-fixed",
    }
}

from_into_string!(Comparison);

impl Comparison {
    pub fn show_in_summary(&self) -> bool {
        matches!(
            self,
            Comparison::Regressed | Comparison::Fixed | Comparison::SpuriousRegressed | Comparison::SpuriousFixed
        )
    }
}

/// Trait for writing report files
pub trait ReportWriter {
    fn write_bytes<P: AsRef<Path>>(
        &self,
        path: P,
        content: Vec<u8>,
        mime: &Mime,
    ) -> Fallible<()>;

    fn write_string<P: AsRef<Path>>(&self, path: P, content: Cow<str>) -> Fallible<()> {
        self.write_bytes(path, content.as_bytes().to_vec(), &mime::TEXT_PLAIN)
    }
}

/// File-based report writer
pub struct FileWriter(PathBuf);

impl FileWriter {
    pub fn create<P: AsRef<Path>>(dest: P) -> Fallible<Self> {
        let dest = dest.as_ref().to_path_buf();
        fs::create_dir_all(&dest)?;
        Ok(FileWriter(dest))
    }
}

impl ReportWriter for FileWriter {
    fn write_bytes<P: AsRef<Path>>(
        &self,
        path: P,
        content: Vec<u8>,
        _mime: &Mime,
    ) -> Fallible<()> {
        let full_path = self.0.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, content)?;
        Ok(())
    }
}

/// Generate a report from experiment results
pub fn gen<W: ReportWriter>(
    ex: &Experiment,
    results: &TestResults,
    writer: &W,
    _config: &Config,
) -> Fallible<()> {
    info!("Generating report for experiment: {}", ex.name);

    // Generate HTML report
    let html = html::generate_html_report(results)?;
    writer.write_string("index.html", Cow::Borrowed(&html))?;

    // Generate Markdown report
    let markdown = markdown::generate_markdown_report(results)?;
    writer.write_string("report.md", Cow::Borrowed(&markdown))?;

    // Generate JSON summary
    let json = serde_json::to_string_pretty(&results.summary)?;
    writer.write_string("summary.json", Cow::Borrowed(&json))?;

    info!("Report generated successfully");
    Ok(())
}

/// Trait for reading results from storage
pub trait ReadResults {
    fn load_all_results(&self, ex: &Experiment) -> Fallible<RawTestResults>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crates::sources::registry::RegistryCrate;
    use tempfile::tempdir;

    #[test]
    fn test_file_writer() {
        let dir = tempdir().unwrap();
        let writer = FileWriter::create(dir.path()).unwrap();

        let result = writer.write_string("test.txt", Cow::Borrowed("Hello, World!"));
        assert!(result.is_ok());

        let content = fs::read_to_string(dir.path().join("test.txt")).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_file_writer_nested_path() {
        let dir = tempdir().unwrap();
        let writer = FileWriter::create(dir.path()).unwrap();

        let result = writer.write_string("nested/dir/test.txt", Cow::Borrowed("Hello!"));
        assert!(result.is_ok());

        let content = fs::read_to_string(dir.path().join("nested/dir/test.txt")).unwrap();
        assert_eq!(content, "Hello!");
    }

    #[test]
    fn test_comparison_display() {
        assert_eq!(Comparison::Regressed.to_string(), "regressed");
        assert_eq!(Comparison::Fixed.to_string(), "fixed");
        assert_eq!(Comparison::SameTestPass.to_string(), "test-pass");
    }

    #[test]
    fn test_comparison() {
        assert!(Comparison::Regressed.show_in_summary());
        assert!(Comparison::Fixed.show_in_summary());
        assert!(!Comparison::SameTestPass.show_in_summary());
        assert!(!Comparison::Skipped.show_in_summary());
        assert!(Comparison::SpuriousRegressed.show_in_summary());
        assert!(Comparison::SpuriousFixed.show_in_summary());
    }

    #[test]
    fn test_crate_result_serialization() {
        let result = CrateResult {
            name: "test".to_string(),
            url: "https://example.com".to_string(),
            krate: Crate::Registry(RegistryCrate::new("test", "1.0.0")),
            status: None,
            res: Comparison::SameTestPass,
            runs: [None, None],
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: CrateResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.res, Comparison::SameTestPass);
    }
}
