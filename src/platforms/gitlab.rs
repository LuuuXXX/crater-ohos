use super::*;

pub struct GitLabAdapter {
    config: PlatformConfig,
}

impl GitLabAdapter {
    pub fn new(config: PlatformConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PlatformAdapter for GitLabAdapter {
    fn platform_type(&self) -> PlatformType {
        PlatformType::GitLab
    }
    
    // GitLab-specific API implementation
    // GitLab API: https://docs.gitlab.com/ee/api/
    
    async fn check_permission(&self, _user: &str, _permission: &str) -> Fallible<bool> {
        Ok(true)
    }
    
    async fn get_issue(&self, repo: &str, number: &str) -> Fallible<PlatformIssue> {
        let base_url = &self.config.api_base_url;
        Ok(PlatformIssue {
            platform: "gitlab".to_string(),
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
        // GitLab webhook uses X-Gitlab-Token header
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
    async fn test_gitlab_issue_url_generation() {
        let config = PlatformConfig {
            api_base_url: "https://gitlab.com/api/v4".to_string(),
            token: None,
            webhook_secret: None,
        };
        let adapter = GitLabAdapter::new(config);
        let issue = adapter.get_issue("gitlab-org/gitlab", "1").await.unwrap();
        assert_eq!(issue.platform, "gitlab");
        assert!(issue.html_url.contains("gitlab.com"));
    }
}
