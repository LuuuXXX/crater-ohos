use crate::api::response::ApiResponse;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthStatus {
    status: String,
    version: String,
}

/// GET /api/v1/health - Health check endpoint
pub async fn health() -> Json<ApiResponse<HealthStatus>> {
    Json(ApiResponse::success(HealthStatus {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

#[derive(Serialize)]
pub struct ConfigInfo {
    version: String,
    // Add more config info as needed
}

/// GET /api/v1/config - Get configuration information
pub async fn config() -> Json<ApiResponse<ConfigInfo>> {
    Json(ApiResponse::success(ConfigInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}
