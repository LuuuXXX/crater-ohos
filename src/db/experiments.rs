use crate::db::DatabasePool;
use crate::experiments::{Assignee, CapLints, Experiment, Mode, PlatformIssue, Status};
use crate::prelude::*;
use crate::toolchain::Toolchain;
use chrono::{DateTime, Utc};
use rusqlite::Row;
use std::str::FromStr;

/// Database wrapper for experiment operations
pub struct Database {
    pool: DatabasePool,
}

impl Database {
    pub fn new(pool: DatabasePool) -> Self {
        Database { pool }
    }

    /// Create a temporary in-memory database for testing
    pub fn temp() -> Fallible<Self> {
        let pool = crate::db::create_memory_pool()?;
        Ok(Database::new(pool))
    }

    /// Get a connection from the pool
    pub fn conn(&self) -> Fallible<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>> {
        Ok(self.pool.get()?)
    }

    /// Helper to parse experiment from database row
    pub(crate) fn parse_experiment(row: &Row) -> rusqlite::Result<Experiment> {
        let name: String = row.get("name")?;
        let mode_str: String = row.get("mode")?;
        let cap_lints_str: String = row.get("cap_lints")?;
        let toolchain_start: Option<String> = row.get("toolchain_start")?;
        let toolchain_end: Option<String> = row.get("toolchain_end")?;
        let priority: i32 = row.get("priority")?;
        let created_at_str: String = row.get("created_at")?;
        let started_at_str: Option<String> = row.get("started_at")?;
        let completed_at_str: Option<String> = row.get("completed_at")?;
        let platform_issue_str: Option<String> = row.get("platform_issue")?;
        let platform_issue_url_str: Option<String> = row.get("platform_issue_url")?;
        let platform_issue_identifier_str: Option<String> = row.get("platform_issue_identifier")?;
        let status_str: String = row.get("status")?;
        let assigned_to_str: Option<String> = row.get("assigned_to")?;
        let report_url: Option<String> = row.get("report_url")?;
        let ignore_blacklist: i32 = row.get("ignore_blacklist")?;
        let requirement: Option<String> = row.get("requirement")?;

        let mode = Mode::from_str(&mode_str).map_err(|_| {
            rusqlite::Error::InvalidQuery
        })?;

        let cap_lints = CapLints::from_str(&cap_lints_str).map_err(|_| {
            rusqlite::Error::InvalidQuery
        })?;

        let status = Status::from_str(&status_str).map_err(|_| {
            rusqlite::Error::InvalidQuery
        })?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        let started_at = started_at_str
            .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

        let completed_at = completed_at_str
            .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

        let platform_issue = if let (Some(platform), Some(api_url), Some(html_url), Some(identifier)) =
            (platform_issue_str, platform_issue_url_str.clone(), platform_issue_url_str, platform_issue_identifier_str)
        {
            Some(PlatformIssue {
                platform,
                api_url,
                html_url,
                identifier,
            })
        } else {
            None
        };

        let assigned_to = assigned_to_str
            .map(|s| Assignee::from_str(&s))
            .transpose()
            .map_err(|_| rusqlite::Error::InvalidQuery)?;

        let toolchains = if let (Some(start), Some(end)) = (toolchain_start, toolchain_end) {
            let t_start = Toolchain::from_str(&start).map_err(|_| rusqlite::Error::InvalidQuery)?;
            let t_end = Toolchain::from_str(&end).map_err(|_| rusqlite::Error::InvalidQuery)?;
            [t_start, t_end]
        } else {
            // Default toolchains if not set
            [
                Toolchain::from_str("stable").unwrap(),
                Toolchain::from_str("beta").unwrap(),
            ]
        };

        Ok(Experiment {
            name,
            toolchains,
            mode,
            cap_lints,
            priority,
            created_at,
            started_at,
            completed_at,
            platform_issue,
            status,
            assigned_to,
            report_url,
            ignore_blacklist: ignore_blacklist != 0,
            requirement,
        })
    }
}
