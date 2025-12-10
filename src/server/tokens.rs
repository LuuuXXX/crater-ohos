use crate::db::{Database, QueryUtils};
use crate::prelude::*;
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

/// API Token 管理
#[derive(Debug, Clone)]
pub struct ApiToken {
    pub token: String,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    ReadExperiments,
    WriteExperiments,
    ManageAgents,
    Admin,
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Permission::ReadExperiments => write!(f, "read_experiments"),
            Permission::WriteExperiments => write!(f, "write_experiments"),
            Permission::ManageAgents => write!(f, "manage_agents"),
            Permission::Admin => write!(f, "admin"),
        }
    }
}

pub trait TokenManager {
    fn create_token(&self, name: &str, permissions: Vec<Permission>) -> Fallible<ApiToken>;
    fn validate_token(&self, token: &str) -> Fallible<Option<ApiToken>>;
    fn revoke_token(&self, token: &str) -> Fallible<()>;
    fn list_tokens(&self) -> Fallible<Vec<ApiToken>>;
}

impl TokenManager for Database {
    fn create_token(&self, name: &str, permissions: Vec<Permission>) -> Fallible<ApiToken> {
        let conn = self.conn()?;
        let now = Utc::now();

        // Generate a random token
        let token = format!("crt_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));

        let permissions_json =
            serde_json::to_string(&permissions).context("failed to serialize permissions")?;

        conn.execute(
            "INSERT INTO api_tokens (token, name, permissions, created_at)
             VALUES (?, ?, ?, ?)",
            rusqlite::params![&token, name, &permissions_json, now.to_rfc3339()],
        )
        .context("failed to insert token")?;

        Ok(ApiToken {
            token,
            name: name.to_string(),
            permissions,
            created_at: now,
            expires_at: None,
        })
    }

    fn validate_token(&self, token: &str) -> Fallible<Option<ApiToken>> {
        let conn = self.conn()?;

        let result = conn
            .query_row(
                "SELECT token, name, permissions, created_at, expires_at
                 FROM api_tokens WHERE token = ?",
                [token],
                |row| {
                    let token: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let permissions_json: String = row.get(2)?;
                    let created_at_str: String = row.get(3)?;
                    let expires_at_str: Option<String> = row.get(4)?;

                    let permissions: Vec<Permission> = serde_json::from_str(&permissions_json)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?;

                    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                        .with_timezone(&Utc);

                    let expires_at = expires_at_str
                        .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
                        .transpose()
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?;

                    Ok(ApiToken {
                        token,
                        name,
                        permissions,
                        created_at,
                        expires_at,
                    })
                },
            )
            .optional()
            .context("failed to query token")?;

        // Check if token is expired
        if let Some(ref token_data) = result {
            if let Some(expires_at) = token_data.expires_at {
                if Utc::now() > expires_at {
                    return Ok(None); // Token is expired
                }
            }
        }

        Ok(result)
    }

    fn revoke_token(&self, token: &str) -> Fallible<()> {
        let conn = self.conn()?;

        let deleted = conn
            .execute("DELETE FROM api_tokens WHERE token = ?", [token])
            .context("failed to revoke token")?;

        if deleted == 0 {
            anyhow::bail!("token not found");
        }

        Ok(())
    }

    fn list_tokens(&self) -> Fallible<Vec<ApiToken>> {
        let conn = self.conn()?;

        let tokens = conn.query(
            "SELECT token, name, permissions, created_at, expires_at
             FROM api_tokens ORDER BY created_at DESC",
            std::iter::empty::<&dyn rusqlite::ToSql>(),
            |row| {
                let token: String = row.get(0)?;
                let name: String = row.get(1)?;
                let permissions_json: String = row.get(2)?;
                let created_at_str: String = row.get(3)?;
                let expires_at_str: Option<String> = row.get(4)?;

                let permissions: Vec<Permission> = serde_json::from_str(&permissions_json)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc);

                let expires_at = expires_at_str
                    .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
                    .transpose()
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                Ok(ApiToken {
                    token,
                    name,
                    permissions,
                    created_at,
                    expires_at,
                })
            },
        )?;

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_validation() {
        let db = Database::temp().unwrap();

        // Create token
        let permissions = vec![Permission::ReadExperiments, Permission::WriteExperiments];
        let token = db.create_token("test-token", permissions.clone()).unwrap();

        assert!(token.token.starts_with("crt_"));
        assert_eq!(token.name, "test-token");
        assert_eq!(token.permissions.len(), 2);

        // Validate token
        let validated = db.validate_token(&token.token).unwrap();
        assert!(validated.is_some());
        let validated = validated.unwrap();
        assert_eq!(validated.token, token.token);
        assert_eq!(validated.name, "test-token");

        // Validate non-existent token
        let result = db.validate_token("invalid-token").unwrap();
        assert!(result.is_none());

        // Revoke token
        db.revoke_token(&token.token).unwrap();

        // Validate revoked token
        let result = db.validate_token(&token.token).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_tokens() {
        let db = Database::temp().unwrap();

        // Create multiple tokens
        db.create_token("token1", vec![Permission::ReadExperiments])
            .unwrap();
        db.create_token("token2", vec![Permission::Admin])
            .unwrap();

        let tokens = db.list_tokens().unwrap();
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_permission_serialization() {
        let perm = Permission::Admin;
        let json = serde_json::to_string(&perm).unwrap();
        assert_eq!(json, "\"admin\"");

        let parsed: Permission = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Permission::Admin);
    }
}
