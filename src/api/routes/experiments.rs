use crate::actions::experiments::{CreateExperiment, EditExperiment, ExperimentActions};
use crate::api::error::ApiError;
use crate::api::response::ApiResponse;
use crate::db::Database;
use crate::experiments::{CrateSelect, Experiment, Mode, PlatformIssue};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CreateExperimentRequest {
    pub name: String,
    pub toolchains: [String; 2],
    pub mode: String,
    pub crate_select: String,
    pub platform_issue: Option<PlatformIssue>,
    pub callback_url: Option<String>,
    #[serde(default)]
    pub priority: i32,
}

#[derive(Debug, Deserialize)]
pub struct EditExperimentRequest {
    pub name: Option<String>,
    pub mode: Option<String>,
    pub crate_select: Option<String>,
    pub platform_issue: Option<PlatformIssue>,
    pub callback_url: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ExperimentResponse {
    pub experiment: Experiment,
}

#[derive(Debug, Serialize)]
pub struct ExperimentListResponse {
    pub experiments: Vec<Experiment>,
}

/// POST /api/v1/experiments - Create experiment
pub async fn create_experiment(
    State(db): State<Arc<Database>>,
    Json(req): Json<CreateExperimentRequest>,
) -> Result<Json<ApiResponse<ExperimentResponse>>, ApiError> {
    // Parse toolchains
    let toolchains = [
        req.toolchains[0]
            .parse()
            .map_err(|e: anyhow::Error| ApiError::BadRequest(e.to_string()))?,
        req.toolchains[1]
            .parse()
            .map_err(|e: anyhow::Error| ApiError::BadRequest(e.to_string()))?,
    ];

    // Parse mode
    let mode: Mode = req
        .mode
        .parse()
        .map_err(|e: anyhow::Error| ApiError::BadRequest(e.to_string()))?;

    // Parse crate_select
    let crate_select: CrateSelect = req
        .crate_select
        .parse()
        .map_err(|e: anyhow::Error| ApiError::BadRequest(e.to_string()))?;

    let create_req = CreateExperiment {
        name: req.name,
        toolchains,
        mode,
        crate_select,
        platform_issue: req.platform_issue,
        callback_url: req.callback_url,
        priority: req.priority,
    };

    let experiment = db
        .create(create_req)
        .map_err(|e| {
            if e.to_string().contains("already exists") {
                ApiError::Conflict(e.to_string())
            } else {
                ApiError::InternalServerError(e.to_string())
            }
        })?;

    Ok(Json(ApiResponse::success(ExperimentResponse { experiment })))
}

/// GET /api/v1/experiments - List all experiments
pub async fn list_experiments(
    State(db): State<Arc<Database>>,
) -> Result<Json<ApiResponse<ExperimentListResponse>>, ApiError> {
    let experiments = db
        .list()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Json(ApiResponse::success(ExperimentListResponse { experiments })))
}

/// GET /api/v1/experiments/{name} - Get experiment details
pub async fn get_experiment(
    State(db): State<Arc<Database>>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<ExperimentResponse>>, ApiError> {
    let experiment = db
        .get(&name)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Experiment '{}' not found", name)))?;

    Ok(Json(ApiResponse::success(ExperimentResponse { experiment })))
}

/// PUT /api/v1/experiments/{name} - Edit experiment
pub async fn edit_experiment(
    State(db): State<Arc<Database>>,
    Path(name): Path<String>,
    Json(req): Json<EditExperimentRequest>,
) -> Result<Json<ApiResponse<ExperimentResponse>>, ApiError> {
    let mode = req
        .mode
        .map(|m| m.parse())
        .transpose()
        .map_err(|e: anyhow::Error| ApiError::BadRequest(e.to_string()))?;

    let crate_select = req
        .crate_select
        .map(|cs| cs.parse())
        .transpose()
        .map_err(|e: anyhow::Error| ApiError::BadRequest(e.to_string()))?;

    let edit_req = EditExperiment {
        name: req.name,
        mode,
        crate_select,
        platform_issue: req.platform_issue,
        callback_url: req.callback_url,
        priority: req.priority,
    };

    let experiment = db
        .edit(&name, edit_req)
        .map_err(|e| {
            if e.to_string().contains("not found") {
                ApiError::NotFound(e.to_string())
            } else if e.to_string().contains("only be edited in 'queued' status") {
                ApiError::BadRequest(e.to_string())
            } else {
                ApiError::InternalServerError(e.to_string())
            }
        })?;

    Ok(Json(ApiResponse::success(ExperimentResponse { experiment })))
}

/// DELETE /api/v1/experiments/{name} - Delete experiment
pub async fn delete_experiment(
    State(db): State<Arc<Database>>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    db.delete(&name).map_err(|e| {
        if e.to_string().contains("not found") {
            ApiError::NotFound(e.to_string())
        } else if e.to_string().contains("only be deleted in 'queued' status") {
            ApiError::BadRequest(e.to_string())
        } else {
            ApiError::InternalServerError(e.to_string())
        }
    })?;

    Ok(Json(ApiResponse::success(())))
}

/// POST /api/v1/experiments/{name}/run - Run experiment
pub async fn run_experiment(
    State(db): State<Arc<Database>>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    db.run(&name).map_err(|e| {
        if e.to_string().contains("not found") {
            ApiError::NotFound(e.to_string())
        } else {
            ApiError::InternalServerError(e.to_string())
        }
    })?;

    Ok(Json(ApiResponse::success(())))
}

/// POST /api/v1/experiments/{name}/abort - Abort experiment
pub async fn abort_experiment(
    State(db): State<Arc<Database>>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    db.abort(&name, "Aborted by user").map_err(|e| {
        if e.to_string().contains("not found") {
            ApiError::NotFound(e.to_string())
        } else {
            ApiError::InternalServerError(e.to_string())
        }
    })?;

    Ok(Json(ApiResponse::success(())))
}
