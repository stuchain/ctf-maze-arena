mod auth;
mod dto;
mod error;
mod handlers;
mod realtime;
mod request_id;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::Semaphore;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::{PeerIpKeyExtractor, SmartIpKeyExtractor},
    GovernorLayer,
};

pub use auth::{jwt_claims_middleware, AuthClaims, AuthMode, JwtConfig};
pub use request_id::{request_id_middleware, REQUEST_ID_HEADER};

pub struct AppState {
    pub db: sqlx::PgPool,
    pub solvers: crate::solve::SolverRegistry,
    pub stream_broadcasts: crate::services::run::StreamMap,
    pub solve_concurrency: Arc<Semaphore>,
    pub active_solve_limits: Arc<crate::services::run::ActiveSolveLimiter>,
    pub accepting_solves: Arc<AtomicBool>,
    pub realtime_config: crate::services::run::RealtimeConfig,
}

pub fn router(
    state: Arc<AppState>,
    global_per_second: u64,
    global_burst: u32,
    expensive_per_second: u64,
    expensive_burst: u32,
    trust_proxy: bool,
) -> Router {
    let exempt = Router::new()
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::ready))
        .route("/solve/stream", get(realtime::stream));
    let expensive = Router::new()
        .route("/maze/generate", post(handlers::generate))
        .route("/solve", post(handlers::solve));
    let baseline = Router::new()
        .route("/maze/{maze_id}", get(handlers::get_maze))
        .route("/run/{run_id}", get(handlers::get_run))
        .route("/run/{run_id}/cancel", post(handlers::cancel_run))
        .route("/replay/{run_id}", get(handlers::get_replay))
        .route(
            "/leaderboard",
            get(handlers::leaderboard).post(handlers::submit_leaderboard),
        )
        .route("/daily", get(handlers::daily));

    let routes = if trust_proxy {
        tracing::warn!(
            "TRUST_PROXY=true; forwarded client IP headers must be overwritten by a trusted proxy"
        );
        let global = GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_second(global_per_second)
            .burst_size(global_burst)
            .use_headers()
            .finish()
            .expect("rate limit");
        let expensive_limit = GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_second(expensive_per_second)
            .burst_size(expensive_burst)
            .use_headers()
            .finish()
            .expect("rate limit");
        Router::new()
            .merge(exempt)
            .merge(expensive.layer(GovernorLayer {
                config: Arc::new(expensive_limit),
            }))
            .merge(baseline.layer(GovernorLayer {
                config: Arc::new(global),
            }))
    } else {
        let global = GovernorConfigBuilder::default()
            .key_extractor(PeerIpKeyExtractor)
            .per_second(global_per_second)
            .burst_size(global_burst)
            .use_headers()
            .finish()
            .expect("rate limit");
        let expensive_limit = GovernorConfigBuilder::default()
            .key_extractor(PeerIpKeyExtractor)
            .per_second(expensive_per_second)
            .burst_size(expensive_burst)
            .use_headers()
            .finish()
            .expect("rate limit");
        Router::new()
            .merge(exempt)
            .merge(expensive.layer(GovernorLayer {
                config: Arc::new(expensive_limit),
            }))
            .merge(baseline.layer(GovernorLayer {
                config: Arc::new(global),
            }))
    };
    Router::new().nest("/api", routes.layer(Extension(state)))
}
