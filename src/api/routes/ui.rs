use crate::actions::experiments::ExperimentActions;
use crate::api::error::ApiError;
use crate::api::response::ApiResponse;
use crate::db::Database;
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct ExperimentListItem {
    pub name: String,
    pub assigned_to: Option<String>,
    pub requirement: Option<String>,
    pub mode: String,
    pub priority: i32,
    pub status: String,
    pub progress_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct ExperimentListResponse {
    pub experiments: Vec<ExperimentListItem>,
}

#[derive(Debug, Serialize)]
pub struct ExperimentDetailResponse {
    pub name: String,
    pub status: String,
    pub mode: String,
    pub priority: i32,
    pub assigned_to: Option<String>,
    pub requirement: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub report_url: Option<String>,
    pub platform_issue_url: Option<String>,
    pub progress_percentage: f64,
    pub completed_count: i64,
    pub total_count: i64,
    pub duration_seconds: Option<i64>,
    pub estimated_remaining_seconds: Option<i64>,
    pub average_task_seconds: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ProgressResponse {
    pub completed: i64,
    pub total: i64,
    pub percentage: f64,
}

/// GET /api/ui/experiments - Get all experiments for UI display
pub async fn list_experiments(
    State(db): State<Arc<Database>>,
) -> Result<Json<ApiResponse<ExperimentListResponse>>, ApiError> {
    let experiments = db
        .list()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let mut items = Vec::new();
    for exp in experiments {
        let (completed, total) = match db.get_experiment_progress(&exp.name) {
            Ok(progress) => progress,
            Err(e) => {
                log::warn!("Failed to get progress for experiment {}: {}", exp.name, e);
                (0, 0)
            }
        };
        
        let progress_percentage = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        items.push(ExperimentListItem {
            name: exp.name,
            assigned_to: exp.assigned_to.map(|a| a.to_string()),
            requirement: exp.requirement,
            mode: exp.mode.to_string(),
            priority: exp.priority,
            status: exp.status.to_string(),
            progress_percentage,
        });
    }

    Ok(Json(ApiResponse::success(ExperimentListResponse {
        experiments: items,
    })))
}

/// GET /api/ui/experiments/{name} - Get experiment detail for UI display
pub async fn get_experiment(
    State(db): State<Arc<Database>>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<ExperimentDetailResponse>>, ApiError> {
    let experiment = db
        .get(&name)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Experiment '{}' not found", name)))?;

    let (completed, total) = match db.get_experiment_progress(&name) {
        Ok(progress) => progress,
        Err(e) => {
            log::warn!("Failed to get progress for experiment {}: {}", name, e);
            (0, 0)
        }
    };
    
    let progress_percentage = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    // Calculate duration and estimates
    let duration_seconds = experiment.duration().map(|d| d.num_seconds());
    
    let (estimated_remaining_seconds, average_task_seconds) = if let Some(started) = experiment.started_at {
        let elapsed = Utc::now().signed_duration_since(started);
        let elapsed_seconds = elapsed.num_seconds();
        
        let avg_task_seconds = if completed > 0 {
            Some(elapsed_seconds as f64 / completed as f64)
        } else {
            None
        };
        
        let remaining = total - completed;
        let estimated_remaining = if let Some(avg) = avg_task_seconds {
            if remaining > 0 {
                Some((avg * remaining as f64) as i64)
            } else {
                None
            }
        } else {
            None
        };
        
        (estimated_remaining, avg_task_seconds)
    } else {
        (None, None)
    };

    let detail = ExperimentDetailResponse {
        name: experiment.name,
        status: experiment.status.to_string(),
        mode: experiment.mode.to_string(),
        priority: experiment.priority,
        assigned_to: experiment.assigned_to.map(|a| a.to_string()),
        requirement: experiment.requirement,
        created_at: experiment.created_at,
        started_at: experiment.started_at,
        completed_at: experiment.completed_at,
        report_url: experiment.report_url,
        platform_issue_url: experiment.platform_issue.map(|i| i.html_url),
        progress_percentage,
        completed_count: completed,
        total_count: total,
        duration_seconds,
        estimated_remaining_seconds,
        average_task_seconds,
    };

    Ok(Json(ApiResponse::success(detail)))
}

/// GET /api/ui/experiments/{name}/progress - Get experiment progress
pub async fn get_progress(
    State(db): State<Arc<Database>>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<ProgressResponse>>, ApiError> {
    // Check if experiment exists
    let _experiment = db
        .get(&name)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Experiment '{}' not found", name)))?;

    let (completed, total) = match db.get_experiment_progress(&name) {
        Ok(progress) => progress,
        Err(e) => {
            log::warn!("Failed to get progress for experiment {}: {}", name, e);
            (0, 0)
        }
    };
    
    let percentage = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(ApiResponse::success(ProgressResponse {
        completed,
        total,
        percentage,
    })))
}
