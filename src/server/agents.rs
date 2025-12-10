use crate::db::{Database, QueryUtils};
use crate::prelude::*;
use chrono::{DateTime, Duration, Utc};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

/// Agent 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<String>,
    pub last_heartbeat: DateTime<Utc>,
    pub current_experiment: Option<String>,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Busy,
    Offline,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AgentStatus::Idle => write!(f, "idle"),
            AgentStatus::Busy => write!(f, "busy"),
            AgentStatus::Offline => write!(f, "offline"),
        }
    }
}

/// Agent 注册请求
#[derive(Debug, Clone)]
pub struct RegisterAgent {
    pub name: String,
    pub capabilities: Vec<String>,
}

/// Agent 管理 trait
pub trait AgentManager {
    /// 注册新 Agent
    fn register_agent(&self, req: RegisterAgent) -> Fallible<Agent>;

    /// Agent 心跳
    fn heartbeat(&self, agent_id: &str) -> Fallible<()>;

    /// 获取 Agent 信息
    fn get_agent(&self, agent_id: &str) -> Fallible<Option<Agent>>;

    /// 列出所有 Agent
    fn list_agents(&self) -> Fallible<Vec<Agent>>;

    /// 为 Agent 分配任务
    fn assign_task(&self, agent_id: &str, experiment: &str) -> Fallible<()>;

    /// 标记任务完成
    fn complete_task(&self, agent_id: &str) -> Fallible<()>;

    /// 移除离线 Agent
    fn cleanup_offline_agents(&self, timeout_secs: i64) -> Fallible<usize>;
}

impl AgentManager for Database {
    fn register_agent(&self, req: RegisterAgent) -> Fallible<Agent> {
        let conn = self.conn()?;
        let now = Utc::now();
        let agent_id = format!("agent-{}", uuid::Uuid::new_v4());

        let capabilities_json = serde_json::to_string(&req.capabilities)
            .context("failed to serialize capabilities")?;

        conn.execute(
            "INSERT INTO agents (id, name, capabilities, last_heartbeat, status)
             VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![
                &agent_id,
                &req.name,
                &capabilities_json,
                now.to_rfc3339(),
                AgentStatus::Idle.to_string(),
            ],
        )
        .context("failed to insert agent")?;

        Ok(Agent {
            id: agent_id,
            name: req.name,
            capabilities: req.capabilities,
            last_heartbeat: now,
            current_experiment: None,
            status: AgentStatus::Idle,
        })
    }

    fn heartbeat(&self, agent_id: &str) -> Fallible<()> {
        let conn = self.conn()?;
        let now = Utc::now();

        let updated = conn
            .execute(
                "UPDATE agents SET last_heartbeat = ? WHERE id = ?",
                rusqlite::params![now.to_rfc3339(), agent_id],
            )
            .context("failed to update agent heartbeat")?;

        if updated == 0 {
            anyhow::bail!("agent '{}' not found", agent_id);
        }

        Ok(())
    }

    fn get_agent(&self, agent_id: &str) -> Fallible<Option<Agent>> {
        let conn = self.conn()?;

        let result = conn
            .query_row(
                "SELECT id, name, capabilities, last_heartbeat, current_experiment, status
                 FROM agents WHERE id = ?",
                [agent_id],
                |row| {
                    let id: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let capabilities_json: String = row.get(2)?;
                    let last_heartbeat_str: String = row.get(3)?;
                    let current_experiment: Option<String> = row.get(4)?;
                    let status_str: String = row.get(5)?;

                    let capabilities: Vec<String> = serde_json::from_str(&capabilities_json)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?;

                    let last_heartbeat = DateTime::parse_from_rfc3339(&last_heartbeat_str)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                        .with_timezone(&Utc);

                    let status = match status_str.as_str() {
                        "idle" => AgentStatus::Idle,
                        "busy" => AgentStatus::Busy,
                        "offline" => AgentStatus::Offline,
                        _ => AgentStatus::Offline,
                    };

                    Ok(Agent {
                        id,
                        name,
                        capabilities,
                        last_heartbeat,
                        current_experiment,
                        status,
                    })
                },
            )
            .optional()
            .context("failed to query agent")?;

        Ok(result)
    }

    fn list_agents(&self) -> Fallible<Vec<Agent>> {
        let conn = self.conn()?;

        let agents = conn.query(
            "SELECT id, name, capabilities, last_heartbeat, current_experiment, status
             FROM agents ORDER BY name",
            std::iter::empty::<&dyn rusqlite::ToSql>(),
            |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let capabilities_json: String = row.get(2)?;
                let last_heartbeat_str: String = row.get(3)?;
                let current_experiment: Option<String> = row.get(4)?;
                let status_str: String = row.get(5)?;

                let capabilities: Vec<String> = serde_json::from_str(&capabilities_json)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                let last_heartbeat = DateTime::parse_from_rfc3339(&last_heartbeat_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc);

                let status = match status_str.as_str() {
                    "idle" => AgentStatus::Idle,
                    "busy" => AgentStatus::Busy,
                    "offline" => AgentStatus::Offline,
                    _ => AgentStatus::Offline,
                };

                Ok(Agent {
                    id,
                    name,
                    capabilities,
                    last_heartbeat,
                    current_experiment,
                    status,
                })
            },
        )?;

        Ok(agents)
    }

    fn assign_task(&self, agent_id: &str, experiment: &str) -> Fallible<()> {
        let conn = self.conn()?;

        // Check if agent exists and is idle
        let agent = self
            .get_agent(agent_id)?
            .ok_or_else(|| anyhow::anyhow!("agent '{}' not found", agent_id))?;

        if agent.status != AgentStatus::Idle {
            anyhow::bail!(
                "agent '{}' is not idle, current status: {:?}",
                agent_id,
                agent.status
            );
        }

        // Update agent status
        conn.execute(
            "UPDATE agents SET current_experiment = ?, status = ? WHERE id = ?",
            rusqlite::params![experiment, AgentStatus::Busy.to_string(), agent_id],
        )
        .context("failed to assign task to agent")?;

        Ok(())
    }

    fn complete_task(&self, agent_id: &str) -> Fallible<()> {
        let conn = self.conn()?;

        let updated = conn
            .execute(
                "UPDATE agents SET current_experiment = NULL, status = ? WHERE id = ?",
                rusqlite::params![AgentStatus::Idle.to_string(), agent_id],
            )
            .context("failed to complete task")?;

        if updated == 0 {
            anyhow::bail!("agent '{}' not found", agent_id);
        }

        Ok(())
    }

    fn cleanup_offline_agents(&self, timeout_secs: i64) -> Fallible<usize> {
        let conn = self.conn()?;
        let cutoff = Utc::now() - Duration::seconds(timeout_secs);

        let deleted = conn
            .execute(
                "DELETE FROM agents WHERE last_heartbeat < ?",
                [cutoff.to_rfc3339()],
            )
            .context("failed to cleanup offline agents")?;

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_lifecycle() {
        let db = Database::temp().unwrap();

        // Register agent
        let req = RegisterAgent {
            name: "test-agent".to_string(),
            capabilities: vec!["build".to_string(), "test".to_string()],
        };
        let agent = db.register_agent(req).unwrap();
        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.capabilities.len(), 2);

        // Heartbeat
        std::thread::sleep(std::time::Duration::from_millis(10));
        let result = db.heartbeat(&agent.id);
        assert!(result.is_ok());

        // Get agent
        let retrieved = db.get_agent(&agent.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, agent.id);
        assert!(retrieved.last_heartbeat > agent.last_heartbeat);

        // List agents
        let agents = db.list_agents().unwrap();
        assert_eq!(agents.len(), 1);
    }

    #[test]
    fn test_agent_task_assignment() {
        let db = Database::temp().unwrap();

        // Create an experiment first
        use crate::actions::experiments::{CreateExperiment, ExperimentActions};
        use crate::experiments::{CrateSelect, Mode};
        use crate::toolchain::Toolchain;
        use std::str::FromStr;

        let exp_req = CreateExperiment {
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
        db.create(exp_req).unwrap();

        // Register agent
        let req = RegisterAgent {
            name: "test-agent".to_string(),
            capabilities: vec!["build".to_string()],
        };
        let agent = db.register_agent(req).unwrap();

        // Assign task
        let result = db.assign_task(&agent.id, "test-exp");
        assert!(result.is_ok());

        // Check agent status
        let agent = db.get_agent(&agent.id).unwrap().unwrap();
        assert_eq!(agent.status, AgentStatus::Busy);
        assert_eq!(agent.current_experiment, Some("test-exp".to_string()));

        // Complete task
        db.complete_task(&agent.id).unwrap();

        // Check agent status
        let agent = db.get_agent(&agent.id).unwrap().unwrap();
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.current_experiment, None);
    }

    #[test]
    fn test_cleanup_offline_agents() {
        let db = Database::temp().unwrap();

        // Register agent
        let req = RegisterAgent {
            name: "test-agent".to_string(),
            capabilities: vec!["build".to_string()],
        };
        db.register_agent(req).unwrap();

        // Cleanup with very short timeout (should delete the agent)
        let deleted = db.cleanup_offline_agents(0).unwrap();
        assert_eq!(deleted, 1);

        // Verify deletion
        let agents = db.list_agents().unwrap();
        assert_eq!(agents.len(), 0);
    }
}
