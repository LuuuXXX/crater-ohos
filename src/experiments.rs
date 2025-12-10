use crate::prelude::*;
use crate::toolchain::Toolchain;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlatformIssue {
    /// Platform identifier: "github", "gitcode", "gitlab" etc.
    pub platform: String,
    /// API URL
    pub api_url: String,
    /// User-accessible URL
    pub html_url: String,
    /// Generic identifier (e.g., issue number or PR number)
    pub identifier: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Experiment {
    pub name: String,
    pub toolchains: [Toolchain; 2],
    pub mode: Mode,
    pub cap_lints: CapLints,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub platform_issue: Option<PlatformIssue>,
    pub status: Status,
    pub assigned_to: Option<Assignee>,
    pub report_url: Option<String>,
    pub ignore_blacklist: bool,
    pub requirement: Option<String>,
}

impl Experiment {
    pub fn duration(&self) -> Option<chrono::Duration> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            Some(completed.signed_duration_since(started))
        } else {
            None
        }
    }
}

string_enum! {
    pub enum Status {
        Queued => "queued",
        Running => "running",
        NeedsReport => "needs-report",
        GeneratingReport => "generating-report",
        ReportFailed => "report-failed",
        Completed => "completed",
    }
}

from_into_string!(Status);

string_enum! {
    pub enum Mode {
        BuildAndTest => "build-and-test",
        BuildOnly => "build-only",
        CheckOnly => "check-only",
        Clippy => "clippy",
        Rustdoc => "rustdoc",
        UnstableFeatures => "unstable-features",
        Fix => "fix",
    }
}

from_into_string!(Mode);

string_enum! {
    pub enum CapLints {
        Allow => "allow",
        Warn => "warn",
        Deny => "deny",
        Forbid => "forbid",
    }
}

from_into_string!(CapLints);

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum CrateSelect {
    Full,
    Demo,
    Top(u32),
    Local,
    Dummy,
    Random(u32),
    List(HashSet<String>),
}

impl fmt::Display for CrateSelect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CrateSelect::Full => write!(f, "full"),
            CrateSelect::Demo => write!(f, "demo"),
            CrateSelect::Top(count) => write!(f, "top-{}", count),
            CrateSelect::Local => write!(f, "local"),
            CrateSelect::Dummy => write!(f, "dummy"),
            CrateSelect::Random(count) => write!(f, "random-{}", count),
            CrateSelect::List(_) => write!(f, "list"),
        }
    }
}

impl FromStr for CrateSelect {
    type Err = Error;

    fn from_str(input: &str) -> Fallible<Self> {
        match input {
            "full" => Ok(CrateSelect::Full),
            "demo" => Ok(CrateSelect::Demo),
            "local" => Ok(CrateSelect::Local),
            "dummy" => Ok(CrateSelect::Dummy),
            s if s.starts_with("top-") => {
                let count = s[4..].parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("invalid top count: {}", s))?;
                Ok(CrateSelect::Top(count))
            }
            s if s.starts_with("random-") => {
                let count = s[7..].parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("invalid random count: {}", s))?;
                Ok(CrateSelect::Random(count))
            }
            _ => anyhow::bail!("unknown crate select: {}", input),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Assignee {
    Agent(String),
    Distributed,
    CLI,
}

impl fmt::Display for Assignee {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Assignee::Agent(name) => write!(f, "{}", name),
            Assignee::Distributed => write!(f, "distributed"),
            Assignee::CLI => write!(f, "cli"),
        }
    }
}

impl FromStr for Assignee {
    type Err = Error;

    fn from_str(input: &str) -> Fallible<Self> {
        match input {
            "distributed" => Ok(Assignee::Distributed),
            "cli" => Ok(Assignee::CLI),
            other => Ok(Assignee::Agent(other.to_string())),
        }
    }
}

// Database record structures for serialization
#[derive(Serialize, Deserialize)]
pub struct ExperimentDBRecord {
    pub name: String,
    pub mode: String,
    pub cap_lints: String,
    pub toolchain_start: Option<String>,
    pub toolchain_end: Option<String>,
    pub priority: i32,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub platform_issue: Option<String>,
    pub platform_issue_url: Option<String>,
    pub platform_issue_identifier: Option<String>,
    pub status: String,
    pub assigned_to: Option<String>,
    pub report_url: Option<String>,
    pub ignore_blacklist: bool,
    pub requirement: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_parsing() {
        assert_eq!("queued".parse::<Status>().unwrap(), Status::Queued);
        assert_eq!("running".parse::<Status>().unwrap(), Status::Running);
        assert_eq!(
            "needs-report".parse::<Status>().unwrap(),
            Status::NeedsReport
        );
        assert_eq!(
            "generating-report".parse::<Status>().unwrap(),
            Status::GeneratingReport
        );
        assert_eq!(
            "report-failed".parse::<Status>().unwrap(),
            Status::ReportFailed
        );
        assert_eq!("completed".parse::<Status>().unwrap(), Status::Completed);
    }

    #[test]
    fn test_mode_parsing() {
        assert_eq!(
            "build-and-test".parse::<Mode>().unwrap(),
            Mode::BuildAndTest
        );
        assert_eq!("build-only".parse::<Mode>().unwrap(), Mode::BuildOnly);
        assert_eq!("check-only".parse::<Mode>().unwrap(), Mode::CheckOnly);
        assert_eq!("clippy".parse::<Mode>().unwrap(), Mode::Clippy);
        assert_eq!("rustdoc".parse::<Mode>().unwrap(), Mode::Rustdoc);
        assert_eq!(
            "unstable-features".parse::<Mode>().unwrap(),
            Mode::UnstableFeatures
        );
        assert_eq!("fix".parse::<Mode>().unwrap(), Mode::Fix);
    }

    #[test]
    fn test_cap_lints_parsing() {
        assert_eq!("allow".parse::<CapLints>().unwrap(), CapLints::Allow);
        assert_eq!("warn".parse::<CapLints>().unwrap(), CapLints::Warn);
        assert_eq!("deny".parse::<CapLints>().unwrap(), CapLints::Deny);
        assert_eq!("forbid".parse::<CapLints>().unwrap(), CapLints::Forbid);
    }

    #[test]
    fn test_assignee_parsing() {
        assert_eq!(
            "distributed".parse::<Assignee>().unwrap(),
            Assignee::Distributed
        );
        assert_eq!("cli".parse::<Assignee>().unwrap(), Assignee::CLI);
        assert_eq!(
            "agent-1".parse::<Assignee>().unwrap(),
            Assignee::Agent("agent-1".to_string())
        );
    }

    #[test]
    fn test_crate_select_parsing() {
        assert_eq!("demo".parse::<CrateSelect>().unwrap(), CrateSelect::Demo);
        assert_eq!("full".parse::<CrateSelect>().unwrap(), CrateSelect::Full);
        assert_eq!("top-100".parse::<CrateSelect>().unwrap(), CrateSelect::Top(100));
        assert_eq!("local".parse::<CrateSelect>().unwrap(), CrateSelect::Local);
        assert_eq!("dummy".parse::<CrateSelect>().unwrap(), CrateSelect::Dummy);
        assert_eq!("random-50".parse::<CrateSelect>().unwrap(), CrateSelect::Random(50));
    }

    #[test]
    fn test_platform_issue() {
        let issue = PlatformIssue {
            platform: "github".to_string(),
            api_url: "https://api.github.com/repos/test/test/issues/1".to_string(),
            html_url: "https://github.com/test/test/issues/1".to_string(),
            identifier: "1".to_string(),
        };

        let json = serde_json::to_string(&issue).unwrap();
        let parsed: PlatformIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.platform, "github");
        assert_eq!(parsed.identifier, "1");
    }
}
