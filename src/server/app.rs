use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use crate::server::{routes, state::AppState, websocket};

/// Create the Axum application with all routes
pub fn create_app() -> Router {
    let state = AppState::new();

    Router::new()
        // Model management routes
        .route("/api/models", get(routes::models::list_models))
        .route("/api/models", post(routes::models::upload_model))
        .route("/api/models/{id}/", get(routes::models::get_model))
        .route("/api/models/{id}/", delete(routes::models::delete_model))
        .route(
            "/api/models/{id}/structure",
            get(routes::models::get_model_structure),
        )
        // Simulation control routes
        .route(
            "/api/simulations",
            post(routes::simulations::start_simulation),
        )
        .route(
            "/api/simulations/{id}/",
            get(routes::simulations::get_status),
        )
        .route(
            "/api/simulations/{id}/",
            delete(routes::simulations::stop_simulation),
        )
        // WebSocket route
        .route("/ws/simulation/{id}/", get(websocket::handler))
        // Health check
        .route("/health", get(health_check))
        // CORS - allow all origins for development
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        // Logging
        .layer(TraceLayer::new_for_http())
        // Add state
        .with_state(state)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Start the server on the specified port
pub async fn serve(port: u16) {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,axum=debug,tower_http=debug".into()),
        )
        .init();

    let app = create_app();
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("API documentation:");
    tracing::info!("  GET  /health");
    tracing::info!("  GET  /api/models");
    tracing::info!("  POST /api/models");
    tracing::info!("  GET  /api/models/{{id}}/");
    tracing::info!("  GET  /api/models/{{id}}/structure");
    tracing::info!("  WS   /ws/simulation/{{id}}/");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
