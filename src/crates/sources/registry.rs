use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Clone)]
pub struct RegistryCrate {
    pub name: SmolStr,
    pub version: SmolStr,
}

impl RegistryCrate {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: SmolStr::new(name),
            version: SmolStr::new(version),
        }
    }
}

impl std::fmt::Display for RegistryCrate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}-{}", self.name, self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_crate_creation() {
        let krate = RegistryCrate::new("serde", "1.0.0");
        assert_eq!(krate.name.as_str(), "serde");
        assert_eq!(krate.version.as_str(), "1.0.0");
    }

    #[test]
    fn test_registry_crate_display() {
        let krate = RegistryCrate::new("tokio", "1.0.0");
        assert_eq!(krate.to_string(), "tokio-1.0.0");
    }

    #[test]
    fn test_registry_crate_serialization() {
        let krate = RegistryCrate::new("regex", "1.5.4");
        let json = serde_json::to_string(&krate).unwrap();
        let parsed: RegistryCrate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name.as_str(), "regex");
        assert_eq!(parsed.version.as_str(), "1.5.4");
    }
}
