use super::*;
use reqwest::Client;

pub struct GitHubAdapter {
    config: PlatformConfig,
    _client: Client,
}

impl GitHubAdapter {
    pub fn new(config: PlatformConfig) -> Self {
        Self {
            config,
            _client: Client::new(),
        }
    }
}

#[async_trait]
impl PlatformAdapter for GitHubAdapter {
    fn platform_type(&self) -> PlatformType {
        PlatformType::GitHub
    }
    
    async fn check_permission(&self, _user: &str, _permission: &str) -> Fallible<bool> {
        // GitHub permission check implementation
        Ok(true) // Simplified implementation
    }
    
    async fn get_issue(&self, repo: &str, number: &str) -> Fallible<PlatformIssue> {
        // Get GitHub Issue
        Ok(PlatformIssue {
            platform: "github".to_string(),
            api_url: format!("https://api.github.com/repos/{}/issues/{}", repo, number),
            html_url: format!("https://github.com/{}/issues/{}", repo, number),
            identifier: number.to_string(),
        })
    }
    
    async fn post_comment(&self, _repo: &str, _issue_number: &str, _body: &str) -> Fallible<PlatformComment> {
        // Post comment implementation
        anyhow::bail!("post_comment not yet implemented")
    }
    
    async fn update_comment(&self, _repo: &str, _comment_id: &str, _body: &str) -> Fallible<PlatformComment> {
        // Update comment implementation
        anyhow::bail!("update_comment not yet implemented")
    }
    
    async fn get_repo(&self, _owner: &str, _name: &str) -> Fallible<PlatformRepo> {
        // Get repository information
        anyhow::bail!("get_repo not yet implemented")
    }
    
    async fn get_user(&self, _username: &str) -> Fallible<PlatformUser> {
        // Get user information
        anyhow::bail!("get_user not yet implemented")
    }
    
    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> bool {
        // GitHub webhook signature verification (HMAC-SHA256)
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        if let Some(secret) = &self.config.webhook_secret {
            if let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
                mac.update(payload);
                let expected = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
                return signature == expected;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_signature_verification() {
        let config = PlatformConfig {
            api_base_url: "https://api.github.com".to_string(),
            token: None,
            webhook_secret: Some("test-secret".to_string()),
        };
        let adapter = GitHubAdapter::new(config);
        
        // Test payload and signature
        let payload = b"test payload";
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        let mut mac = Hmac::<Sha256>::new_from_slice(b"test-secret").unwrap();
        mac.update(payload);
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        
        assert!(adapter.verify_webhook_signature(payload, &signature));
        assert!(!adapter.verify_webhook_signature(payload, "invalid"));
    }
}
