use super::*;

pub struct GiteeAdapter {
    config: PlatformConfig,
}

impl GiteeAdapter {
    pub fn new(config: PlatformConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PlatformAdapter for GiteeAdapter {
    fn platform_type(&self) -> PlatformType {
        PlatformType::Gitee
    }
    
    // 实现 Gitee 特定的 API 调用
    // Gitee API: https://gitee.com/api/v5/swagger
    
    async fn check_permission(&self, _user: &str, _permission: &str) -> Fallible<bool> {
        Ok(true)
    }
    
    async fn get_issue(&self, repo: &str, number: &str) -> Fallible<PlatformIssue> {
        Ok(PlatformIssue {
            platform: "gitee".to_string(),
            api_url: format!("https://gitee.com/api/v5/repos/{}/issues/{}", repo, number),
            html_url: format!("https://gitee.com/{}/issues/{}", repo, number),
            identifier: number.to_string(),
        })
    }
    
    async fn post_comment(&self, _repo: &str, _issue_number: &str, _body: &str) -> Fallible<PlatformComment> {
        anyhow::bail!("post_comment not yet implemented")
    }
    
    async fn update_comment(&self, _repo: &str, _comment_id: &str, _body: &str) -> Fallible<PlatformComment> {
        anyhow::bail!("update_comment not yet implemented")
    }
    
    async fn get_repo(&self, _owner: &str, _name: &str) -> Fallible<PlatformRepo> {
        anyhow::bail!("get_repo not yet implemented")
    }
    
    async fn get_user(&self, _username: &str) -> Fallible<PlatformUser> {
        anyhow::bail!("get_user not yet implemented")
    }
    
    fn verify_webhook_signature(&self, _payload: &[u8], signature: &str) -> bool {
        // Gitee webhook 签名验证
        if let Some(secret) = &self.config.webhook_secret {
            return signature == *secret;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gitee_issue_url_generation() {
        let config = PlatformConfig {
            api_base_url: "https://gitee.com/api/v5".to_string(),
            token: None,
            webhook_secret: None,
        };
        let adapter = GiteeAdapter::new(config);
        let issue = adapter.get_issue("openharmony/rust", "1").await.unwrap();
        assert_eq!(issue.platform, "gitee");
        assert!(issue.html_url.contains("gitee.com"));
    }
}
