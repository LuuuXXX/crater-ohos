use crate::api::error::ApiError;
use crate::api::response::ApiResponse;
use crate::db::Database;
use crate::server::agents::{Agent, AgentManager, RegisterAgent};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct RegisterAgentRequest {
    pub name: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentResponse {
    pub agent: Agent,
}

#[derive(Debug, Serialize)]
pub struct AgentListResponse {
    pub agents: Vec<Agent>,
}

/// POST /api/v1/agents/register - Register agent
pub async fn register_agent(
    State(db): State<Arc<Database>>,
    Json(req): Json<RegisterAgentRequest>,
) -> Result<Json<ApiResponse<AgentResponse>>, ApiError> {
    let register_req = RegisterAgent {
        name: req.name,
        capabilities: req.capabilities,
    };

    let agent = db
        .register_agent(register_req)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Json(ApiResponse::success(AgentResponse { agent })))
}

/// POST /api/v1/agents/{id}/heartbeat - Agent heartbeat
pub async fn agent_heartbeat(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    db.heartbeat(&id).map_err(|e| {
        if e.to_string().contains("not found") {
            ApiError::NotFound(e.to_string())
        } else {
            ApiError::InternalServerError(e.to_string())
        }
    })?;

    Ok(Json(ApiResponse::success(())))
}

/// GET /api/v1/agents - List all agents
pub async fn list_agents(
    State(db): State<Arc<Database>>,
) -> Result<Json<ApiResponse<AgentListResponse>>, ApiError> {
    let agents = db
        .list_agents()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Json(ApiResponse::success(AgentListResponse { agents })))
}

/// GET /api/v1/agents/{id} - Get agent details
pub async fn get_agent(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<AgentResponse>>, ApiError> {
    let agent = db
        .get_agent(&id)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{}' not found", id)))?;

    Ok(Json(ApiResponse::success(AgentResponse { agent })))
}
