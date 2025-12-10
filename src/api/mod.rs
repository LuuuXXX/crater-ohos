pub mod error;
pub mod middleware;
pub mod response;
pub mod routes;

use crate::db::Database;
use axum::{
    middleware as axum_middleware,
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// Build the API router
pub fn build_router(db: Arc<Database>) -> Router {
    // Health and config routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(routes::health::health))
        .route("/config", get(routes::health::config));

    // Experiment routes (require authentication)
    let experiment_routes = Router::new()
        .route("/experiments", post(routes::experiments::create_experiment))
        .route("/experiments", get(routes::experiments::list_experiments))
        .route("/experiments/:name", get(routes::experiments::get_experiment))
        .route("/experiments/:name", put(routes::experiments::edit_experiment))
        .route("/experiments/:name", delete(routes::experiments::delete_experiment))
        .route("/experiments/:name/run", post(routes::experiments::run_experiment))
        .route("/experiments/:name/abort", post(routes::experiments::abort_experiment))
        .layer(axum_middleware::from_fn_with_state(
            db.clone(),
            middleware::auth::auth,
        ));

    // Agent routes (require authentication)
    let agent_routes = Router::new()
        .route("/agents/register", post(routes::agents::register_agent))
        .route("/agents/:id/heartbeat", post(routes::agents::agent_heartbeat))
        .route("/agents", get(routes::agents::list_agents))
        .route("/agents/:id", get(routes::agents::get_agent))
        .layer(axum_middleware::from_fn_with_state(
            db.clone(),
            middleware::auth::auth,
        ));

    // Combine all routes under /api/v1
    Router::new()
        .nest("/api/v1", public_routes)
        .nest("/api/v1", experiment_routes)
        .nest("/api/v1", agent_routes)
        .layer(CorsLayer::permissive())
        .with_state(db)
}
