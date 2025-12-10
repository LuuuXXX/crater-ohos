use crate::db::Database;
use crate::server::tokens::{Permission, TokenManager};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Authentication middleware that validates Bearer tokens
pub async fn auth(
    State(db): State<Arc<Database>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check for Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token
    let api_token = db
        .validate_token(token)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Store token in request extensions for later use
    req.extensions_mut().insert(api_token);

    Ok(next.run(req).await)
}

/// Middleware to check for specific permissions
pub fn require_permission(
    permission: Permission,
) -> impl Fn(Request, Next) -> Pin<Box<dyn Future<Output = Result<Response, StatusCode>> + Send>> + Clone
{
    move |req: Request, next: Next| {
        let permission = permission.clone();
        Box::pin(async move {
            // Get token from request extensions
            let api_token = req
                .extensions()
                .get::<crate::server::tokens::ApiToken>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            // Check if token has required permission or admin permission
            if !api_token.permissions.contains(&permission)
                && !api_token.permissions.contains(&Permission::Admin)
            {
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(req).await)
        })
    }
}
