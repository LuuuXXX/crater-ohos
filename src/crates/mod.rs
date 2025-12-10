pub(crate) mod lists;
pub(crate) mod sources;

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

pub use crate::crates::sources::github::GitHubRepo;
pub use crate::crates::sources::gitcode::GitCodeRepo;
pub use crate::crates::sources::registry::RegistryCrate;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Clone)]
pub struct GitRepo {
    pub url: String,
    pub sha: Option<String>,
}

impl GitRepo {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            sha: None,
        }
    }

    pub fn with_sha(url: &str, sha: &str) -> Self {
        Self {
            url: url.to_string(),
            sha: Some(sha.to_string()),
        }
    }
}

impl fmt::Display for GitRepo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.url)?;
        if let Some(ref sha) = self.sha {
            write!(f, "#{}", sha)?;
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Crate {
    Registry(RegistryCrate),
    GitHub(GitHubRepo),
    GitCode(GitCodeRepo),
    Local(String),
    Path(String),
    Git(GitRepo),
}

impl Crate {
    pub fn id(&self) -> String {
        match self {
            Crate::Registry(krate) => format!("reg:{}", krate),
            Crate::GitHub(repo) => format!("gh:{}", repo.slug()),
            Crate::GitCode(repo) => format!("gc:{}", repo.slug()),
            Crate::Local(name) => format!("local:{}", name),
            Crate::Path(path) => format!("path:{}", path),
            Crate::Git(repo) => format!("git:{}", repo.url),
        }
    }
}

impl fmt::Display for Crate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Crate::Registry(krate) => write!(f, "{}", krate),
            Crate::GitHub(repo) => write!(f, "{}", repo),
            Crate::GitCode(repo) => write!(f, "{}", repo),
            Crate::Local(name) => write!(f, "local:{}", name),
            Crate::Path(path) => write!(f, "path:{}", path),
            Crate::Git(repo) => write!(f, "{}", repo),
        }
    }
}

impl FromStr for Crate {
    type Err = Error;

    fn from_str(s: &str) -> Fallible<Self> {
        if let Some(stripped) = s.strip_prefix("reg:") {
            if let Some((name, version)) = stripped.split_once('-') {
                Ok(Crate::Registry(RegistryCrate::new(name, version)))
            } else {
                anyhow::bail!("invalid registry crate format: {}", s)
            }
        } else if let Some(stripped) = s.strip_prefix("gh:") {
            if let Some((org, name)) = stripped.split_once('/') {
                Ok(Crate::GitHub(GitHubRepo::new(org, name)))
            } else {
                anyhow::bail!("invalid github repo format: {}", s)
            }
        } else if let Some(stripped) = s.strip_prefix("gc:") {
            if let Some((org, name)) = stripped.split_once('/') {
                Ok(Crate::GitCode(GitCodeRepo::new(org, name)))
            } else {
                anyhow::bail!("invalid gitcode repo format: {}", s)
            }
        } else if let Some(stripped) = s.strip_prefix("local:") {
            Ok(Crate::Local(stripped.to_string()))
        } else if let Some(stripped) = s.strip_prefix("path:") {
            Ok(Crate::Path(stripped.to_string()))
        } else if let Some(stripped) = s.strip_prefix("git:") {
            Ok(Crate::Git(GitRepo::new(stripped)))
        } else {
            anyhow::bail!("unknown crate format: {}", s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_id() {
        let krate = Crate::Registry(RegistryCrate::new("serde", "1.0.0"));
        assert_eq!(krate.id(), "reg:serde-1.0.0");

        let krate = Crate::GitHub(GitHubRepo::new("rust-lang", "rust"));
        assert_eq!(krate.id(), "gh:rust-lang/rust");

        let krate = Crate::GitCode(GitCodeRepo::new("rust-lang", "rust"));
        assert_eq!(krate.id(), "gc:rust-lang/rust");

        let krate = Crate::Local("my-crate".to_string());
        assert_eq!(krate.id(), "local:my-crate");
    }

    #[test]
    fn test_crate_display() {
        let krate = Crate::Registry(RegistryCrate::new("tokio", "1.0.0"));
        assert_eq!(krate.to_string(), "tokio-1.0.0");

        let krate = Crate::GitHub(GitHubRepo::new("tokio-rs", "tokio"));
        assert_eq!(krate.to_string(), "tokio-rs/tokio");

        let krate = Crate::GitCode(GitCodeRepo::new("tokio-rs", "tokio"));
        assert_eq!(krate.to_string(), "tokio-rs/tokio");
    }

    #[test]
    fn test_crate_parsing() {
        let krate: Crate = "reg:serde-1.0.0".parse().unwrap();
        assert!(matches!(krate, Crate::Registry(_)));

        let krate: Crate = "gh:rust-lang/rust".parse().unwrap();
        assert!(matches!(krate, Crate::GitHub(_)));

        let krate: Crate = "gc:rust-lang/rust".parse().unwrap();
        assert!(matches!(krate, Crate::GitCode(_)));

        let krate: Crate = "local:my-crate".parse().unwrap();
        assert!(matches!(krate, Crate::Local(_)));

        let krate: Crate = "path:/path/to/crate".parse().unwrap();
        assert!(matches!(krate, Crate::Path(_)));

        let krate: Crate = "git:https://github.com/rust-lang/rust".parse().unwrap();
        assert!(matches!(krate, Crate::Git(_)));
    }

    #[test]
    fn test_git_repo() {
        let repo = GitRepo::new("https://github.com/rust-lang/rust");
        assert_eq!(repo.url, "https://github.com/rust-lang/rust");
        assert!(repo.sha.is_none());

        let repo = GitRepo::with_sha("https://github.com/rust-lang/rust", "abc123");
        assert_eq!(repo.sha, Some("abc123".to_string()));
        assert_eq!(
            repo.to_string(),
            "https://github.com/rust-lang/rust#abc123"
        );
    }
}
