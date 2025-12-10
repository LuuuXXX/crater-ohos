use crate::actions::experiments::ExperimentActions;
use crate::db::Database;
use crate::ui::templates::TEMPLATES;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;

/// GET /ui/queue - Show experiment queue page
pub async fn queue_page(
    State(db): State<Arc<Database>>,
) -> Response {
    let experiments = match db.list() {
        Ok(exps) => exps,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("<h1>Error loading experiments</h1><p>{}</p>", e))
            ).into_response();
        }
    };

    // Calculate progress for each experiment
    let mut exp_items = Vec::new();
    for exp in experiments {
        let (completed, total) = db.get_experiment_progress(&exp.name).unwrap_or((0, 0));
        let progress_percentage = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        exp_items.push(json!({
            "name": exp.name,
            "assigned_to": exp.assigned_to.map(|a| a.to_string()).unwrap_or_else(|| "None".to_string()),
            "requirement": exp.requirement.unwrap_or_else(|| "None".to_string()),
            "mode": exp.mode.to_string(),
            "priority": exp.priority,
            "status": exp.status.to_string(),
            "progress_percentage": format!("{:.1}", progress_percentage),
        }));
    }

    let context = json!({
        "experiments": exp_items,
        "page_title": "Experiment Queue",
    });

    match TEMPLATES.render("queue.html", &tera::Context::from_serialize(&context).unwrap()) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("<h1>Template Error</h1><p>{}</p>", e))
            ).into_response()
        }
    }
}

/// GET /ui/ex/{name} - Show experiment detail page
pub async fn experiment_page(
    State(db): State<Arc<Database>>,
    Path(name): Path<String>,
) -> Response {
    let experiment = match db.get(&name) {
        Ok(Some(exp)) => exp,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Html(format!("<h1>Experiment Not Found</h1><p>Experiment '{}' does not exist</p>", name))
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("<h1>Error loading experiment</h1><p>{}</p>", e))
            ).into_response();
        }
    };

    let (completed, total) = db.get_experiment_progress(&name).unwrap_or((0, 0));
    let progress_percentage = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    // Calculate duration and estimates
    let duration_str = experiment.duration()
        .map(|d| format_duration(d.num_seconds()))
        .unwrap_or_else(|| "N/A".to_string());

    let (estimated_remaining_str, average_task_str) = if let Some(started) = experiment.started_at {
        let elapsed = chrono::Utc::now().signed_duration_since(started);
        let elapsed_seconds = elapsed.num_seconds();
        
        let avg_task_seconds = if completed > 0 {
            Some(elapsed_seconds as f64 / completed as f64)
        } else {
            None
        };
        
        let avg_str = avg_task_seconds
            .map(|avg| format!("{:.1}s", avg))
            .unwrap_or_else(|| "N/A".to_string());
        
        let remaining = total - completed;
        let est_str = if let Some(avg) = avg_task_seconds {
            if remaining > 0 && experiment.completed_at.is_none() {
                format_duration((avg * remaining as f64) as i64)
            } else {
                "N/A".to_string()
            }
        } else {
            "N/A".to_string()
        };
        
        (est_str, avg_str)
    } else {
        ("N/A".to_string(), "N/A".to_string())
    };

    let context = json!({
        "name": experiment.name,
        "status": experiment.status.to_string(),
        "mode": experiment.mode.to_string(),
        "priority": experiment.priority,
        "assigned_to": experiment.assigned_to.map(|a| a.to_string()).unwrap_or_else(|| "None".to_string()),
        "requirement": experiment.requirement.unwrap_or_else(|| "None".to_string()),
        "created_at": experiment.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        "started_at": experiment.started_at.map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string()).unwrap_or_else(|| "Not started".to_string()),
        "completed_at": experiment.completed_at.map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string()).unwrap_or_else(|| "Not completed".to_string()),
        "progress_percentage": format!("{:.1}", progress_percentage),
        "completed_count": completed,
        "total_count": total,
        "duration": duration_str,
        "estimated_remaining": estimated_remaining_str,
        "average_task_time": average_task_str,
        "report_url": experiment.report_url.unwrap_or_else(|| "N/A".to_string()),
        "platform_issue_url": experiment.platform_issue.map(|i| i.html_url).unwrap_or_else(|| "N/A".to_string()),
        "page_title": format!("Experiment: {}", experiment.name),
    });

    match TEMPLATES.render("experiment.html", &tera::Context::from_serialize(&context).unwrap()) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("<h1>Template Error</h1><p>{}</p>", e))
            ).into_response()
        }
    }
}

/// Format duration in seconds to human-readable format
fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{}m {}s", minutes, secs)
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    } else {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}
