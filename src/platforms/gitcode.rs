use super::*;

pub struct GitCodeAdapter {
    config: PlatformConfig,
}

impl GitCodeAdapter {
    pub fn new(config: PlatformConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PlatformAdapter for GitCodeAdapter {
    fn platform_type(&self) -> PlatformType {
        PlatformType::GitCode
    }
    
    // GitCode-specific API implementation
    // GitCode API is similar to GitLab API
    
    async fn check_permission(&self, _user: &str, _permission: &str) -> Fallible<bool> {
        Ok(true)
    }
    
    async fn get_issue(&self, repo: &str, number: &str) -> Fallible<PlatformIssue> {
        let base_url = &self.config.api_base_url;
        // GitCode uses GitLab-compatible API structure
        Ok(PlatformIssue {
            platform: "gitcode".to_string(),
            api_url: format!("{}/projects/{}/issues/{}", base_url, repo, number),
            html_url: format!("{}/{}/-/issues/{}", base_url.replace("/api/v4", ""), repo, number),
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
        // GitCode webhook uses token-based verification similar to GitLab
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
    async fn test_gitcode_issue_url_generation() {
        let config = PlatformConfig {
            api_base_url: "https://gitcode.com/api/v4".to_string(),
            token: None,
            webhook_secret: None,
        };
        let adapter = GitCodeAdapter::new(config);
        let issue = adapter.get_issue("gitcode-org/project", "1").await.unwrap();
        assert_eq!(issue.platform, "gitcode");
        assert!(issue.html_url.contains("gitcode.com"));
    }

    #[test]
    fn test_gitcode_webhook_verification() {
        let config = PlatformConfig {
            api_base_url: "https://gitcode.com/api/v4".to_string(),
            token: None,
            webhook_secret: Some("test-secret".to_string()),
        };
        let adapter = GitCodeAdapter::new(config);
        assert!(adapter.verify_webhook_signature(b"test", "test-secret"));
        assert!(!adapter.verify_webhook_signature(b"test", "wrong-secret"));
    }
}
