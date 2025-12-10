pub mod github;
pub mod gitee;
pub mod gitlab;

use crate::experiments::PlatformIssue;
use crate::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 平台类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlatformType {
    GitHub,
    Gitee,
    GitLab,
    GitCode,
    Custom(String),
}

impl PlatformType {
    pub fn as_str(&self) -> &str {
        match self {
            PlatformType::GitHub => "github",
            PlatformType::Gitee => "gitee",
            PlatformType::GitLab => "gitlab",
            PlatformType::GitCode => "gitcode",
            PlatformType::Custom(s) => s,
        }
    }
}

/// 平台用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformUser {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// 平台仓库信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRepo {
    pub id: String,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub clone_url: String,
    pub default_branch: String,
}

/// 平台评论
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformComment {
    pub id: String,
    pub body: String,
    pub author: PlatformUser,
    pub created_at: String,
}

/// 平台适配器 trait
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    /// 获取平台类型
    fn platform_type(&self) -> PlatformType;
    
    /// 验证用户是否有权限
    async fn check_permission(&self, user: &str, permission: &str) -> Fallible<bool>;
    
    /// 获取 Issue/PR 信息
    async fn get_issue(&self, repo: &str, number: &str) -> Fallible<PlatformIssue>;
    
    /// 发表评论
    async fn post_comment(&self, repo: &str, issue_number: &str, body: &str) -> Fallible<PlatformComment>;
    
    /// 更新评论
    async fn update_comment(&self, repo: &str, comment_id: &str, body: &str) -> Fallible<PlatformComment>;
    
    /// 获取仓库信息
    async fn get_repo(&self, owner: &str, name: &str) -> Fallible<PlatformRepo>;
    
    /// 获取用户信息
    async fn get_user(&self, username: &str) -> Fallible<PlatformUser>;
    
    /// 验证 webhook 签名
    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> bool;
}

/// 平台配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub api_base_url: String,
    pub token: Option<String>,
    pub webhook_secret: Option<String>,
}

/// 平台工厂
pub struct PlatformFactory;

impl PlatformFactory {
    pub fn create(platform_type: PlatformType, config: PlatformConfig) -> Box<dyn PlatformAdapter> {
        match platform_type {
            PlatformType::GitHub => Box::new(github::GitHubAdapter::new(config)),
            PlatformType::Gitee => Box::new(gitee::GiteeAdapter::new(config)),
            PlatformType::GitLab => Box::new(gitlab::GitLabAdapter::new(config)),
            PlatformType::GitCode => Box::new(gitlab::GitLabAdapter::new(config)), // GitCode 基于 GitLab
            _ => Box::new(github::GitHubAdapter::new(config)), // 默认
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_type_serialization() {
        assert_eq!(PlatformType::GitHub.as_str(), "github");
        assert_eq!(PlatformType::Gitee.as_str(), "gitee");
        assert_eq!(PlatformType::GitLab.as_str(), "gitlab");
        assert_eq!(PlatformType::GitCode.as_str(), "gitcode");
    }

    #[test]
    fn test_platform_factory() {
        let config = PlatformConfig {
            api_base_url: "https://api.github.com".to_string(),
            token: None,
            webhook_secret: None,
        };
        let adapter = PlatformFactory::create(PlatformType::GitHub, config);
        assert_eq!(adapter.platform_type(), PlatformType::GitHub);
    }

    #[tokio::test]
    async fn test_github_issue_url_generation() {
        let config = PlatformConfig {
            api_base_url: "https://api.github.com".to_string(),
            token: None,
            webhook_secret: None,
        };
        let adapter = github::GitHubAdapter::new(config);
        let issue = adapter.get_issue("rust-lang/rust", "12345").await.unwrap();
        assert_eq!(issue.platform, "github");
        assert!(issue.html_url.contains("github.com"));
    }
}
