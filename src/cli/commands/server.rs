use crate::api::build_router;
use crate::db::Database;
use crate::prelude::*;
use std::sync::Arc;

pub async fn server(port: u16, config_path: String) -> Fallible<()> {
    println!("Starting crater-ohos API server...");
    println!("  Port: {}", port);
    println!("  Config: {}", config_path);

    // Load configuration
    let _config = crate::config::Config::load(&config_path)
        .context("Failed to load configuration")?;

    // Initialize database
    let db = Database::open()?;
    let db = Arc::new(db);

    // Build router
    let app = build_router(db);

    // Bind to address
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("✓ Server listening on http://{}", addr);
    println!("  Health check: http://{}/api/v1/health", addr);
    println!("\nPress Ctrl+C to stop the server");

    // Start server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    println!("\nServer stopped");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}
