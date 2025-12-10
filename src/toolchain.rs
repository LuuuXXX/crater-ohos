use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Toolchain {
    pub source: RustwideToolchain,
    pub target: Option<String>,
    pub rustflags: Option<String>,
    pub rustdocflags: Option<String>,
    pub cargoflags: Option<String>,
    pub ci_try: bool,
    pub patches: Vec<CratePatch>,
}

impl Toolchain {
    pub fn to_path_component(&self) -> String {
        let mut component = self.source.to_string();
        if let Some(ref target) = self.target {
            component.push_str("--");
            component.push_str(target);
        }
        component
    }
}

impl fmt::Display for Toolchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.source)?;
        if let Some(ref target) = self.target {
            write!(f, " (target: {})", target)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct CratePatch {
    pub name: String,
    pub repo: url::Url,
    pub branch: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum RustwideToolchain {
    Dist(String),
    Master { sha: Option<String> },
    Try { sha: String },
    CI { sha: String, alt: bool },
}

impl fmt::Display for RustwideToolchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RustwideToolchain::Dist(name) => write!(f, "{}", name),
            RustwideToolchain::Master { sha: Some(sha) } => write!(f, "master#{}", sha),
            RustwideToolchain::Master { sha: None } => write!(f, "master"),
            RustwideToolchain::Try { sha } => write!(f, "try#{}", sha),
            RustwideToolchain::CI { sha, alt: false } => write!(f, "ci#{}", sha),
            RustwideToolchain::CI { sha, alt: true } => write!(f, "ci-alt#{}", sha),
        }
    }
}

impl FromStr for RustwideToolchain {
    type Err = Error;

    fn from_str(input: &str) -> Fallible<Self> {
        if let Some(hash_idx) = input.find('#') {
            let (prefix, sha) = input.split_at(hash_idx);
            let sha = &sha[1..]; // Remove the '#'

            match prefix {
                "master" => Ok(RustwideToolchain::Master {
                    sha: Some(sha.to_string()),
                }),
                "try" => Ok(RustwideToolchain::Try {
                    sha: sha.to_string(),
                }),
                "ci" => Ok(RustwideToolchain::CI {
                    sha: sha.to_string(),
                    alt: false,
                }),
                "ci-alt" => Ok(RustwideToolchain::CI {
                    sha: sha.to_string(),
                    alt: true,
                }),
                _ => anyhow::bail!("unknown toolchain type: {}", prefix),
            }
        } else {
            match input {
                "master" => Ok(RustwideToolchain::Master { sha: None }),
                name => Ok(RustwideToolchain::Dist(name.to_string())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rustwide_toolchain_parsing() {
        assert_eq!(
            "stable".parse::<RustwideToolchain>().unwrap(),
            RustwideToolchain::Dist("stable".to_string())
        );
        assert_eq!(
            "beta".parse::<RustwideToolchain>().unwrap(),
            RustwideToolchain::Dist("beta".to_string())
        );
        assert_eq!(
            "nightly".parse::<RustwideToolchain>().unwrap(),
            RustwideToolchain::Dist("nightly".to_string())
        );
        assert_eq!(
            "master".parse::<RustwideToolchain>().unwrap(),
            RustwideToolchain::Master { sha: None }
        );
        assert_eq!(
            "master#abc123".parse::<RustwideToolchain>().unwrap(),
            RustwideToolchain::Master {
                sha: Some("abc123".to_string())
            }
        );
        assert_eq!(
            "try#abc123".parse::<RustwideToolchain>().unwrap(),
            RustwideToolchain::Try {
                sha: "abc123".to_string()
            }
        );
        assert_eq!(
            "ci#abc123".parse::<RustwideToolchain>().unwrap(),
            RustwideToolchain::CI {
                sha: "abc123".to_string(),
                alt: false
            }
        );
        assert_eq!(
            "ci-alt#abc123".parse::<RustwideToolchain>().unwrap(),
            RustwideToolchain::CI {
                sha: "abc123".to_string(),
                alt: true
            }
        );
    }

    #[test]
    fn test_toolchain_display() {
        let tc = Toolchain {
            source: RustwideToolchain::Dist("stable".to_string()),
            target: None,
            rustflags: None,
            rustdocflags: None,
            cargoflags: None,
            ci_try: false,
            patches: vec![],
        };
        assert_eq!(tc.to_string(), "stable");

        let tc = Toolchain {
            source: RustwideToolchain::Dist("stable".to_string()),
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            rustflags: None,
            rustdocflags: None,
            cargoflags: None,
            ci_try: false,
            patches: vec![],
        };
        assert_eq!(tc.to_string(), "stable (target: x86_64-unknown-linux-gnu)");
    }
}
