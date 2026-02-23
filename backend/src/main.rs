use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Extension, Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod auth;
mod config;
mod db;
mod error;
mod graphql;
mod llm;

use crate::auth::middleware::auth_middleware;
use crate::config::AppConfig;
use crate::db::init_db;
use crate::graphql::build_schema;
use crate::llm::manager::LlmManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting FerrisMind (OpenNotebookLM) backend...");

    // Load configuration
    let config = AppConfig::from_env()?;

    // Initialize Database
    let db = init_db(&config.surreal).await?;

    // Initialize LLM Manager
    let llm_manager = Arc::new(LlmManager::new(&config.llm));

    // Build GraphQL Schema
    let schema = build_schema(db.clone(), config.jwt.clone(), llm_manager.clone());

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build Axum Router
    let app = Router::new()
        .route("/graphql", post(crate::graphql::graphql_handler))
        .route("/graphiql", get(crate::graphql::graphiql_handler))
        .route(
            "/api/chat/stream",
            post(crate::llm::sse::chat_stream_handler),
        )
        .layer(Extension(schema))
        .layer(Extension(config.jwt.clone()))
        .layer(Extension(llm_manager))
        .layer(middleware::from_fn(auth_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
