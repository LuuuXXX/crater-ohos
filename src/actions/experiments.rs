use crate::db::{Database, QueryUtils};
use crate::experiments::{CrateSelect, Experiment, Mode, PlatformIssue, Status};
use crate::prelude::*;
use crate::toolchain::Toolchain;
use chrono::Utc;
use rusqlite::OptionalExtension;

/// 创建实验的请求参数
#[derive(Debug, Clone)]
pub struct CreateExperiment {
    pub name: String,
    pub toolchains: [Toolchain; 2],
    pub mode: Mode,
    pub crate_select: CrateSelect,
    pub platform_issue: Option<PlatformIssue>,
    pub callback_url: Option<String>,
    pub priority: i32,
}

/// 编辑实验的请求参数
#[derive(Debug, Clone, Default)]
pub struct EditExperiment {
    pub name: Option<String>,
    pub mode: Option<Mode>,
    pub crate_select: Option<CrateSelect>,
    pub platform_issue: Option<PlatformIssue>,
    pub callback_url: Option<String>,
    pub priority: Option<i32>,
}

/// Actions trait 定义实验操作接口
pub trait ExperimentActions {
    /// 创建新实验
    fn create(&self, req: CreateExperiment) -> Fallible<Experiment>;

    /// 编辑已存在的实验（仅限 queued 状态）
    fn edit(&self, name: &str, req: EditExperiment) -> Fallible<Experiment>;

    /// 删除实验（仅限 queued 状态）
    fn delete(&self, name: &str) -> Fallible<()>;

    /// 获取实验详情
    fn get(&self, name: &str) -> Fallible<Option<Experiment>>;

    /// 列出所有实验
    fn list(&self) -> Fallible<Vec<Experiment>>;

    /// 运行实验（将状态改为 running）
    fn run(&self, name: &str) -> Fallible<()>;

    /// 完成实验
    fn complete(&self, name: &str) -> Fallible<()>;

    /// 中止实验
    fn abort(&self, name: &str, reason: &str) -> Fallible<()>;
}

impl ExperimentActions for Database {
    fn create(&self, req: CreateExperiment) -> Fallible<Experiment> {
        let conn = self.conn()?;
        let now = Utc::now();

        // Check if experiment already exists
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM experiments WHERE name = ?",
                [&req.name],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .context("failed to check experiment existence")?;

        if exists {
            anyhow::bail!("experiment '{}' already exists", req.name);
        }

        let toolchain_start = req.toolchains[0].to_string();
        let toolchain_end = req.toolchains[1].to_string();
        let mode_str = req.mode.to_string();
        let status_str = Status::Queued.to_string();
        let created_at_str = now.to_rfc3339();

        // Insert experiment
        let (platform_issue, platform_issue_url, platform_issue_identifier) =
            if let Some(ref issue) = req.platform_issue {
                (
                    Some(issue.platform.clone()),
                    Some(issue.html_url.clone()),
                    Some(issue.identifier.clone()),
                )
            } else {
                (None, None, None)
            };

        conn.execute(
            "INSERT INTO experiments 
             (name, mode, cap_lints, toolchain_start, toolchain_end, priority, created_at, 
              platform_issue, platform_issue_url, platform_issue_identifier, status, ignore_blacklist)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                &req.name,
                &mode_str,
                "warn", // default cap_lints
                &toolchain_start,
                &toolchain_end,
                req.priority,
                &created_at_str,
                platform_issue,
                platform_issue_url,
                platform_issue_identifier,
                &status_str,
                0, // ignore_blacklist = false
            ],
        )
        .context("failed to insert experiment")?;

        // Insert metadata if callback_url is provided
        if let Some(ref callback_url) = req.callback_url {
            let platform = req
                .platform_issue
                .as_ref()
                .map(|i| i.platform.as_str())
                .unwrap_or("unknown");

            conn.execute(
                "INSERT INTO experiment_metadata (experiment, callback_url, platform, created_at)
                 VALUES (?, ?, ?, ?)",
                rusqlite::params![&req.name, callback_url, platform, &created_at_str],
            )
            .context("failed to insert experiment metadata")?;
        }

        // Get and return the created experiment
        self.get(&req.name)?
            .ok_or_else(|| anyhow::anyhow!("failed to retrieve created experiment"))
    }

    fn edit(&self, name: &str, req: EditExperiment) -> Fallible<Experiment> {
        let conn = self.conn()?;

        // Check if experiment exists and is in queued state
        let current_status: Option<String> = conn
            .query_row(
                "SELECT status FROM experiments WHERE name = ?",
                [name],
                |row| row.get(0),
            )
            .optional()
            .context("failed to get experiment status")?;

        let current_status = current_status
            .ok_or_else(|| anyhow::anyhow!("experiment '{}' not found", name))?;

        if current_status != Status::Queued.to_string() {
            anyhow::bail!(
                "can only edit experiments in 'queued' state, current state: {}",
                current_status
            );
        }

        // Build update query dynamically
        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(new_name) = &req.name {
            updates.push("name = ?");
            params.push(Box::new(new_name.clone()));
        }

        if let Some(mode) = &req.mode {
            updates.push("mode = ?");
            params.push(Box::new(mode.to_string()));
        }

        if let Some(ref issue) = req.platform_issue {
            updates.push("platform_issue = ?");
            updates.push("platform_issue_url = ?");
            updates.push("platform_issue_identifier = ?");
            params.push(Box::new(issue.platform.clone()));
            params.push(Box::new(issue.html_url.clone()));
            params.push(Box::new(issue.identifier.clone()));
        }

        if let Some(priority) = req.priority {
            updates.push("priority = ?");
            params.push(Box::new(priority));
        }

        if updates.is_empty() {
            // Nothing to update, just return current experiment
            return self
                .get(name)?
                .ok_or_else(|| anyhow::anyhow!("experiment '{}' not found", name));
        }

        // Add the WHERE clause parameter
        params.push(Box::new(name.to_string()));

        let query = format!("UPDATE experiments SET {} WHERE name = ?", updates.join(", "));

        conn.execute(
            &query,
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        )
        .context("failed to update experiment")?;

        // Update metadata if callback_url is provided
        if let Some(ref callback_url) = req.callback_url {
            conn.execute(
                "UPDATE experiment_metadata SET callback_url = ? WHERE experiment = ?",
                rusqlite::params![callback_url, name],
            )
            .context("failed to update experiment metadata")?;
        }

        // Get the final name (might have changed)
        let final_name = req.name.as_deref().unwrap_or(name);

        self.get(final_name)?
            .ok_or_else(|| anyhow::anyhow!("experiment '{}' not found after edit", final_name))
    }

    fn delete(&self, name: &str) -> Fallible<()> {
        let conn = self.conn()?;

        // Check if experiment exists and is in queued state
        let current_status: Option<String> = conn
            .query_row(
                "SELECT status FROM experiments WHERE name = ?",
                [name],
                |row| row.get(0),
            )
            .optional()
            .context("failed to get experiment status")?;

        let current_status = current_status
            .ok_or_else(|| anyhow::anyhow!("experiment '{}' not found", name))?;

        if current_status != Status::Queued.to_string() {
            anyhow::bail!(
                "can only delete experiments in 'queued' state, current state: {}",
                current_status
            );
        }

        // Delete experiment (cascades to metadata, results, etc.)
        conn.execute("DELETE FROM experiments WHERE name = ?", [name])
            .context("failed to delete experiment")?;

        Ok(())
    }

    fn get(&self, name: &str) -> Fallible<Option<Experiment>> {
        let conn = self.conn()?;

        let result = conn
            .query_row(
                "SELECT name, mode, cap_lints, toolchain_start, toolchain_end, priority, 
                        created_at, started_at, completed_at, platform_issue, platform_issue_url, 
                        platform_issue_identifier, status, assigned_to, report_url, 
                        ignore_blacklist, requirement
                 FROM experiments WHERE name = ?",
                [name],
                Database::parse_experiment,
            )
            .optional()
            .context("failed to query experiment")?;

        Ok(result)
    }

    fn list(&self) -> Fallible<Vec<Experiment>> {
        let conn = self.conn()?;

        let experiments = conn.query(
            "SELECT name, mode, cap_lints, toolchain_start, toolchain_end, priority, 
                    created_at, started_at, completed_at, platform_issue, platform_issue_url, 
                    platform_issue_identifier, status, assigned_to, report_url, 
                    ignore_blacklist, requirement
             FROM experiments ORDER BY created_at DESC",
            std::iter::empty::<&dyn rusqlite::ToSql>(),
            Database::parse_experiment,
        )?;

        Ok(experiments)
    }

    fn run(&self, name: &str) -> Fallible<()> {
        let conn = self.conn()?;
        let now = Utc::now();

        // Check if experiment exists and is in queued state
        let current_status: Option<String> = conn
            .query_row(
                "SELECT status FROM experiments WHERE name = ?",
                [name],
                |row| row.get(0),
            )
            .optional()
            .context("failed to get experiment status")?;

        let current_status = current_status
            .ok_or_else(|| anyhow::anyhow!("experiment '{}' not found", name))?;

        if current_status != Status::Queued.to_string() {
            anyhow::bail!(
                "can only run experiments in 'queued' state, current state: {}",
                current_status
            );
        }

        // Update status to running and set started_at
        conn.execute(
            "UPDATE experiments SET status = ?, started_at = ? WHERE name = ?",
            rusqlite::params![Status::Running.to_string(), now.to_rfc3339(), name],
        )
        .context("failed to update experiment status")?;

        Ok(())
    }

    fn complete(&self, name: &str) -> Fallible<()> {
        let conn = self.conn()?;
        let now = Utc::now();

        // Check if experiment exists
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM experiments WHERE name = ?",
                [name],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .context("failed to check experiment existence")?;

        if !exists {
            anyhow::bail!("experiment '{}' not found", name);
        }

        // Update status to completed and set completed_at
        conn.execute(
            "UPDATE experiments SET status = ?, completed_at = ? WHERE name = ?",
            rusqlite::params![Status::Completed.to_string(), now.to_rfc3339(), name],
        )
        .context("failed to update experiment status")?;

        Ok(())
    }

    fn abort(&self, name: &str, _reason: &str) -> Fallible<()> {
        let conn = self.conn()?;

        // Check if experiment exists
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM experiments WHERE name = ?",
                [name],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .context("failed to check experiment existence")?;

        if !exists {
            anyhow::bail!("experiment '{}' not found", name);
        }

        // For now, we'll use ReportFailed as the aborted state
        // We could add an "Aborted" status to the Status enum if needed
        conn.execute(
            "UPDATE experiments SET status = ? WHERE name = ?",
            rusqlite::params![Status::ReportFailed.to_string(), name],
        )
        .context("failed to update experiment status")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toolchain::RustwideToolchain;
    use std::str::FromStr;

    #[test]
    fn test_create_experiment() {
        let db = Database::temp().unwrap();
        let req = CreateExperiment {
            name: "test-exp".to_string(),
            toolchains: [
                Toolchain {
                    source: RustwideToolchain::Dist("stable".to_string()),
                    target: None,
                    rustflags: None,
                    rustdocflags: None,
                    cargoflags: None,
                    ci_try: false,
                    patches: vec![],
                },
                Toolchain {
                    source: RustwideToolchain::Dist("beta".to_string()),
                    target: None,
                    rustflags: None,
                    rustdocflags: None,
                    cargoflags: None,
                    ci_try: false,
                    patches: vec![],
                },
            ],
            mode: Mode::BuildAndTest,
            crate_select: CrateSelect::Demo,
            platform_issue: None,
            callback_url: Some("https://example.com/callback".to_string()),
            priority: 0,
        };
        let exp = db.create(req).unwrap();
        assert_eq!(exp.name, "test-exp");
        assert_eq!(exp.status, Status::Queued);
        assert_eq!(exp.mode, Mode::BuildAndTest);
    }

    #[test]
    fn test_edit_experiment_only_queued() {
        let db = Database::temp().unwrap();

        // Create experiment
        let req = CreateExperiment {
            name: "test-exp".to_string(),
            toolchains: [
                Toolchain::from_str("stable").unwrap(),
                Toolchain::from_str("beta").unwrap(),
            ],
            mode: Mode::BuildAndTest,
            crate_select: CrateSelect::Demo,
            platform_issue: None,
            callback_url: None,
            priority: 0,
        };
        db.create(req).unwrap();

        // Edit should work in queued state
        let edit_req = EditExperiment {
            priority: Some(10),
            ..Default::default()
        };
        let result = db.edit("test-exp", edit_req);
        assert!(result.is_ok());

        // Run the experiment (changes status to running)
        db.run("test-exp").unwrap();

        // Edit should fail in running state
        let edit_req = EditExperiment {
            priority: Some(20),
            ..Default::default()
        };
        let result = db.edit("test-exp", edit_req);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_experiment_only_queued() {
        let db = Database::temp().unwrap();

        // Create experiment
        let req = CreateExperiment {
            name: "test-exp".to_string(),
            toolchains: [
                Toolchain::from_str("stable").unwrap(),
                Toolchain::from_str("beta").unwrap(),
            ],
            mode: Mode::BuildAndTest,
            crate_select: CrateSelect::Demo,
            platform_issue: None,
            callback_url: None,
            priority: 0,
        };
        db.create(req).unwrap();

        // Delete should work in queued state
        let result = db.delete("test-exp");
        assert!(result.is_ok());

        // Verify deletion
        let exp = db.get("test-exp").unwrap();
        assert!(exp.is_none());
    }

    #[test]
    fn test_experiment_lifecycle() {
        let db = Database::temp().unwrap();

        // Create
        let req = CreateExperiment {
            name: "lifecycle-test".to_string(),
            toolchains: [
                Toolchain::from_str("stable").unwrap(),
                Toolchain::from_str("beta").unwrap(),
            ],
            mode: Mode::BuildAndTest,
            crate_select: CrateSelect::Demo,
            platform_issue: None,
            callback_url: None,
            priority: 0,
        };
        db.create(req).unwrap();

        // Get
        let exp = db.get("lifecycle-test").unwrap();
        assert!(exp.is_some());
        assert_eq!(exp.unwrap().status, Status::Queued);

        // Run
        db.run("lifecycle-test").unwrap();
        let exp = db.get("lifecycle-test").unwrap();
        assert_eq!(exp.unwrap().status, Status::Running);

        // Complete
        db.complete("lifecycle-test").unwrap();
        let exp = db.get("lifecycle-test").unwrap();
        assert_eq!(exp.unwrap().status, Status::Completed);
    }

    #[test]
    fn test_list_experiments() {
        let db = Database::temp().unwrap();

        // Create multiple experiments
        for i in 0..3 {
            let req = CreateExperiment {
                name: format!("exp-{}", i),
                toolchains: [
                    Toolchain::from_str("stable").unwrap(),
                    Toolchain::from_str("beta").unwrap(),
                ],
                mode: Mode::BuildAndTest,
                crate_select: CrateSelect::Demo,
                platform_issue: None,
                callback_url: None,
                priority: i,
            };
            db.create(req).unwrap();
        }

        let experiments = db.list().unwrap();
        assert_eq!(experiments.len(), 3);
    }
}
