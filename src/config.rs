use crate::prelude::*;
use crate::utils::size::Size;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub demo_crates: DemoCrates,
    pub sandbox: SandboxConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub platforms: PlatformsConfig,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Fallible<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct DemoCrates {
    #[serde(default)]
    pub crates: Vec<String>,
    #[serde(default)]
    pub github_repos: Vec<String>,
    #[serde(default)]
    pub local_crates: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SandboxConfig {
    pub memory_limit: Size,
    pub build_log_max_size: Size,
    pub build_log_max_lines: u64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    pub acl: ACL,
    #[serde(default)]
    pub callback: CallbackConfig,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ACL {
    /// 允许的用户列表
    #[serde(default)]
    pub allowed_users: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CallbackConfig {
    /// 默认的 callback 超时时间（秒）
    pub timeout_secs: Option<u64>,
    /// 重试次数
    pub retry_count: Option<u32>,
}

impl CallbackConfig {
    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs.unwrap_or(30)
    }

    pub fn retry_count(&self) -> u32 {
        self.retry_count.unwrap_or(3)
    }
}

/// Platform configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct PlatformInstanceConfig {
    pub api_base_url: String,
    pub token: Option<String>,
    pub webhook_secret: Option<String>,
}

/// Multi-platform configuration
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct PlatformsConfig {
    pub github: Option<PlatformInstanceConfig>,
    pub gitee: Option<PlatformInstanceConfig>,
    pub gitlab: Option<PlatformInstanceConfig>,
    pub gitcode: Option<PlatformInstanceConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback_config_defaults() {
        let config = CallbackConfig {
            timeout_secs: None,
            retry_count: None,
        };
        assert_eq!(config.timeout_secs(), 30);
        assert_eq!(config.retry_count(), 3);
    }

    #[test]
    fn test_callback_config_custom() {
        let config = CallbackConfig {
            timeout_secs: Some(60),
            retry_count: Some(5),
        };
        assert_eq!(config.timeout_secs(), 60);
        assert_eq!(config.retry_count(), 5);
    }
}
