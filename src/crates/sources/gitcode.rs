use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Clone)]
pub struct GitCodeRepo {
    pub org: String,
    pub name: String,
    pub sha: Option<String>,
}

impl GitCodeRepo {
    pub fn slug(&self) -> String {
        format!("{}/{}", self.org, self.name)
    }

    pub fn new(org: &str, name: &str) -> Self {
        Self {
            org: org.to_string(),
            name: name.to_string(),
            sha: None,
        }
    }

    pub fn with_sha(org: &str, name: &str, sha: &str) -> Self {
        Self {
            org: org.to_string(),
            name: name.to_string(),
            sha: Some(sha.to_string()),
        }
    }
}

impl std::fmt::Display for GitCodeRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.slug())?;
        if let Some(ref sha) = self.sha {
            write!(f, "#{}", sha)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitcode_repo_slug() {
        let repo = GitCodeRepo::new("rust-lang", "rust");
        assert_eq!(repo.slug(), "rust-lang/rust");
    }

    #[test]
    fn test_gitcode_repo_display() {
        let repo = GitCodeRepo::new("tokio-rs", "tokio");
        assert_eq!(repo.to_string(), "tokio-rs/tokio");

        let repo = GitCodeRepo::with_sha("serde-rs", "serde", "abc123");
        assert_eq!(repo.to_string(), "serde-rs/serde#abc123");
    }

    #[test]
    fn test_gitcode_repo_serialization() {
        let repo = GitCodeRepo::with_sha("rust-lang", "crater", "def456");
        let json = serde_json::to_string(&repo).unwrap();
        let parsed: GitCodeRepo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.org, "rust-lang");
        assert_eq!(parsed.name, "crater");
        assert_eq!(parsed.sha, Some("def456".to_string()));
    }
}
