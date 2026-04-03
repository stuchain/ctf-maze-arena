use axum::{Extension, Router, routing::get};
use std::sync::Arc;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub solvers: crate::solve::SolverRegistry,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new().nest(
        "/api",
        api_routes().layer(Extension(state)),
    )
}

fn api_routes() -> Router {
    Router::new().route("/health", get(health_handler))
}

async fn health_handler() -> &'static str {
    "ok"
}
