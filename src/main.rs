use axum::http::{header, HeaderName, HeaderValue, Request};
use axum::middleware;
use ctf_maze_arena::api;
use ctf_maze_arena::solve;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::fmt;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use tower_http::cors::{Any, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

#[derive(Debug, PartialEq, Eq)]
enum AllowedOriginsSetting {
    Unset,
    Explicit(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RateLimitConfig {
    per_second: u64,
    burst: u32,
    expensive_per_second: u64,
    expensive_burst: u32,
    trust_proxy: bool,
}

impl RateLimitConfig {
    const DEFAULT_PER_SECOND: u64 = 20;
    const DEFAULT_BURST: u32 = 40;
    const DEFAULT_EXPENSIVE_PER_SECOND: u64 = 5;
    const DEFAULT_EXPENSIVE_BURST: u32 = 10;

    fn from_env() -> Self {
        let per_second =
            parse_u64_env("RATE_LIMIT_PER_SECOND", RateLimitConfig::DEFAULT_PER_SECOND);
        let burst = parse_u32_env("RATE_LIMIT_BURST", RateLimitConfig::DEFAULT_BURST);
        let expensive_per_second = parse_u64_env(
            "RATE_LIMIT_EXPENSIVE_PER_SECOND",
            RateLimitConfig::DEFAULT_EXPENSIVE_PER_SECOND,
        );
        let expensive_burst = parse_u32_env(
            "RATE_LIMIT_EXPENSIVE_BURST",
            RateLimitConfig::DEFAULT_EXPENSIVE_BURST,
        );
        let trust_proxy = parse_bool_env(std::env::var("TRUST_PROXY").ok().as_deref());

        Self {
            per_second,
            burst,
            expensive_per_second,
            expensive_burst,
            trust_proxy,
        }
    }
}

#[derive(Debug, Clone)]
struct AuthConfig {
    jwt_secret: Option<String>,
    clock_skew_secs: u64,
    mode: api::AuthMode,
}

impl AuthConfig {
    const DEFAULT_CLOCK_SKEW_SECS: u64 = 60;

    fn from_env() -> Result<Self, ConfigError> {
        let jwt_secret = std::env::var("JWT_SECRET")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let clock_skew_secs = parse_u64_env("JWT_CLOCK_SKEW_SECS", Self::DEFAULT_CLOCK_SKEW_SECS);
        let mode = parse_auth_mode_env(std::env::var("AUTH_MODE").ok().as_deref());

        validate_jwt_secret(mode, jwt_secret.as_deref())?;

        Ok(Self {
            jwt_secret,
            clock_skew_secs,
            mode,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogFormat {
    Pretty,
    Json,
}

impl LogFormat {
    fn from_env() -> Self {
        match std::env::var("LOG_FORMAT") {
            Ok(v) if v.trim().eq_ignore_ascii_case("json") => LogFormat::Json,
            Ok(v) if !v.trim().is_empty() && !v.trim().eq_ignore_ascii_case("pretty") => {
                tracing::warn!(
                    "LOG_FORMAT has unsupported value {:?}; using pretty formatter",
                    v
                );
                LogFormat::Pretty
            }
            _ => LogFormat::Pretty,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    init_logging();

    let pool = init_db().await?;
    tracing::info!("database initialized");

    let realtime_config = ctf_maze_arena::services::run::RealtimeConfig {
        history_capacity: parse_usize_env("STREAM_HISTORY_CAPACITY", 256),
        client_channel_capacity: parse_usize_env("STREAM_CLIENT_CAPACITY", 32),
        sample_every: parse_u32_env("STREAM_SAMPLE_EVERY", 2),
        snapshot_every: parse_u32_env("STREAM_SNAPSHOT_EVERY", 32),
        max_replay_events: parse_usize_env("MAX_REPLAY_EVENTS", 2048),
        terminal_retention: Duration::from_secs(parse_u64_env("STREAM_RETENTION_SECS", 30)),
        heartbeat_interval: Duration::from_secs(parse_u64_env("STREAM_HEARTBEAT_SECS", 10)),
    };
    let state = Arc::new(api::AppState {
        db: pool,
        solvers: solve::default_registry(),
        stream_broadcasts: Arc::new(RwLock::new(HashMap::new())),
        solve_concurrency: Arc::new(Semaphore::new(parse_usize_env("MAX_CONCURRENT_SOLVES", 1))),
        active_solve_limits: ctf_maze_arena::services::run::ActiveSolveLimiter::new(
            parse_usize_env("MAX_ACTIVE_SOLVES_PER_ACTOR", 2),
        ),
        accepting_solves: Arc::new(AtomicBool::new(true)),
        realtime_config,
    });

    let rate_limit = RateLimitConfig::from_env();
    let auth_config = AuthConfig::from_env()?;
    tracing::info!(
        rate_limit_per_second = rate_limit.per_second,
        rate_limit_burst = rate_limit.burst,
        rate_limit_expensive_per_second = rate_limit.expensive_per_second,
        rate_limit_expensive_burst = rate_limit.expensive_burst,
        trust_proxy = rate_limit.trust_proxy,
        "loaded rate limit config"
    );
    tracing::info!(
        jwt_secret_configured = auth_config.jwt_secret.is_some(),
        jwt_clock_skew_secs = auth_config.clock_skew_secs,
        auth_mode = ?auth_config.mode,
        "loaded auth config"
    );

    let cors = cors_layer_from_env();

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let request_id = request
                .headers()
                .get(api::REQUEST_ID_HEADER)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown");
            tracing::info_span!(
                "http_request",
                request_id = %request_id,
                method = %request.method(),
                path = %request.uri().path()
            )
        })
        .on_response(
            |response: &axum::response::Response, latency: Duration, _span: &tracing::Span| {
                tracing::info!(
                    status = response.status().as_u16(),
                    latency_ms = latency.as_millis() as u64,
                    "request completed"
                );
            },
        );

    let app = api::router(
        Arc::clone(&state),
        rate_limit.per_second,
        rate_limit.burst,
        rate_limit.expensive_per_second,
        rate_limit.expensive_burst,
        rate_limit.trust_proxy,
    )
    .layer(middleware::from_fn_with_state(
        api::JwtConfig {
            secret: auth_config.jwt_secret.clone(),
            clock_skew_secs: auth_config.clock_skew_secs,
            auth_mode: auth_config.mode,
        },
        api::jwt_claims_middleware,
    ))
    .layer(trace_layer)
    .layer(middleware::from_fn(api::request_id_middleware))
    .layer(SetResponseHeaderLayer::if_not_present(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    ))
    .layer(cors);
    let port = parse_port_env(std::env::var("PORT").ok().as_deref())?;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal(state))
    .await?;
    Ok(())
}

fn init_logging() {
    let env_filter = tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
    );
    match LogFormat::from_env() {
        LogFormat::Json => tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .flatten_event(true)
            .init(),
        LogFormat::Pretty => tracing_subscriber::fmt().with_env_filter(env_filter).init(),
    }
}

fn cors_layer_from_env() -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    let allowed_origins_raw = std::env::var("ALLOWED_ORIGINS").ok();
    match parse_allowed_origins_env(allowed_origins_raw.as_deref()) {
        AllowedOriginsSetting::Unset => {
            if cfg!(debug_assertions) {
                tracing::info!("ALLOWED_ORIGINS is unset; using permissive CORS for local/dev.");
                return base.allow_origin(Any);
            }

            if is_permissive_override_enabled() {
                tracing::warn!(
                    "ALLOWED_ORIGINS is unset in release build and CORS_PERMISSIVE=true; allowing permissive CORS by explicit override."
                );
                return base.allow_origin(Any);
            }

            tracing::warn!(
                "ALLOWED_ORIGINS is unset in release build; permissive CORS is disabled by default."
            );
            base
        }
        AllowedOriginsSetting::Explicit(origins) if origins.is_empty() => {
            tracing::info!(
                "ALLOWED_ORIGINS is set but empty; cross-origin CORS is disabled by default."
            );
            base
        }
        AllowedOriginsSetting::Explicit(origins) => {
            let mut header_values = Vec::with_capacity(origins.len());
            for origin in origins {
                match HeaderValue::from_str(&origin) {
                    Ok(value) => header_values.push(value),
                    Err(_) => tracing::warn!(
                        "Ignoring invalid CORS origin in ALLOWED_ORIGINS: {}",
                        origin
                    ),
                }
            }

            if header_values.is_empty() {
                tracing::warn!(
                    "ALLOWED_ORIGINS did not contain any valid origins; cross-origin CORS is disabled."
                );
                return base;
            }

            base.allow_origin(header_values)
        }
    }
}

fn is_permissive_override_enabled() -> bool {
    parse_bool_env(std::env::var("CORS_PERMISSIVE").ok().as_deref())
}

fn parse_bool_env(value: Option<&str>) -> bool {
    value
        .map(str::trim)
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn parse_u64_env(key: &str, default: u64) -> u64 {
    match std::env::var(key) {
        Ok(v) => match v.trim().parse::<u64>() {
            Ok(parsed) if parsed > 0 => parsed,
            _ => {
                tracing::warn!(
                    "{} is invalid (must be a positive integer): {:?}; using default {}",
                    key,
                    v,
                    default
                );
                default
            }
        },
        Err(_) => default,
    }
}

fn parse_u32_env(key: &str, default: u32) -> u32 {
    match std::env::var(key) {
        Ok(v) => match v.trim().parse::<u32>() {
            Ok(parsed) if parsed > 0 => parsed,
            _ => {
                tracing::warn!(
                    "{} is invalid (must be a positive integer): {:?}; using default {}",
                    key,
                    v,
                    default
                );
                default
            }
        },
        Err(_) => default,
    }
}

fn parse_allowed_origins_env(value: Option<&str>) -> AllowedOriginsSetting {
    match value {
        None => AllowedOriginsSetting::Unset,
        Some(raw) => AllowedOriginsSetting::Explicit(parse_allowed_origins(raw)),
    }
}

fn parse_auth_mode_env(value: Option<&str>) -> api::AuthMode {
    match value.map(str::trim) {
        Some(v) if v.eq_ignore_ascii_case("jwt") => api::AuthMode::Jwt,
        Some(v) if v.eq_ignore_ascii_case("optional_jwt") => api::AuthMode::OptionalJwt,
        Some(v) if !v.is_empty() && !v.eq_ignore_ascii_case("anonymous") => {
            tracing::warn!(
                "AUTH_MODE has unsupported value {:?}; defaulting to anonymous",
                v
            );
            api::AuthMode::Anonymous
        }
        _ => api::AuthMode::Anonymous,
    }
}

fn validate_jwt_secret(mode: api::AuthMode, secret: Option<&str>) -> Result<(), ConfigError> {
    if mode == api::AuthMode::Anonymous {
        return Ok(());
    }
    match secret {
        Some(value) if value.len() >= 32 => Ok(()),
        Some(_) => Err(ConfigError(
            "JWT_SECRET must contain at least 32 characters when AUTH_MODE enables JWTs".into(),
        )),
        None => Err(ConfigError(
            "JWT_SECRET is required when AUTH_MODE enables JWTs".into(),
        )),
    }
}

fn parse_port_env(value: Option<&str>) -> Result<u16, ConfigError> {
    match value {
        None => Ok(8080),
        Some(raw) => raw
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|port| *port > 0)
            .ok_or_else(|| {
                ConfigError(format!(
                    "PORT must be an integer between 1 and 65535, got {raw:?}"
                ))
            }),
    }
}

fn parse_allowed_origins(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|origin| origin.trim_end_matches('/').to_string())
        .collect()
}

async fn init_db() -> Result<sqlx::PgPool, Box<dyn std::error::Error + Send + Sync>> {
    let url = std::env::var("DATABASE_URL")
        .map_err(|_| ConfigError("DATABASE_URL is required and must point to Postgres".into()))?;
    if !url.starts_with("postgres://") && !url.starts_with("postgresql://") {
        return Err(ConfigError(
            "DATABASE_URL must use the postgres:// or postgresql:// scheme".into(),
        )
        .into());
    }
    let max_connections = parse_u32_env("DB_MAX_CONNECTIONS", 5);
    let attempts = parse_u32_env("DB_CONNECT_ATTEMPTS", 5);
    let options = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(10));
    let mut last_error = None;
    for attempt in 1..=attempts {
        match options.clone().connect(&url).await {
            Ok(pool) => {
                ctf_maze_arena::store::migrate(&pool).await?;
                let recovered = ctf_maze_arena::store::recover_interrupted_runs(&pool).await?;
                if recovered > 0 {
                    tracing::warn!(recovered, "marked interrupted solver runs as failed");
                }
                let deleted = ctf_maze_arena::store::delete_expired_replays(&pool).await?;
                if deleted > 0 {
                    tracing::info!(deleted, "deleted expired replay payloads");
                }
                return Ok(pool);
            }
            Err(error) => {
                tracing::warn!(attempt, attempts, %error, "database connection failed");
                last_error = Some(error);
                if attempt < attempts {
                    tokio::time::sleep(Duration::from_millis(250 * u64::from(attempt))).await;
                }
            }
        }
    }
    Err(last_error
        .expect("at least one database connection attempt")
        .into())
}

fn parse_usize_env(key: &str, default: usize) -> usize {
    match std::env::var(key) {
        Ok(value) => value
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|value| *value > 0)
            .unwrap_or_else(|| {
                tracing::warn!(
                    key,
                    value,
                    default,
                    "invalid positive integer; using default"
                );
                default
            }),
        Err(_) => default,
    }
}

async fn shutdown_signal(state: Arc<api::AppState>) {
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install terminate signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        result = tokio::signal::ctrl_c() => {
            if let Err(error) = result {
                tracing::error!(%error, "failed to listen for ctrl-c");
            }
        }
        _ = terminate => {}
    }
    tracing::info!("graceful shutdown requested");
    ctf_maze_arena::services::run::shutdown(
        &state.db,
        &state.stream_broadcasts,
        &state.accepting_solves,
        &state.solve_concurrency,
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::{
        parse_allowed_origins, parse_allowed_origins_env, parse_auth_mode_env, parse_bool_env,
        parse_port_env, parse_u32_env, parse_u64_env, validate_jwt_secret, AllowedOriginsSetting,
        RateLimitConfig,
    };
    use crate::api::AuthMode;

    #[test]
    fn parse_allowed_origins_splits_and_trims() {
        let parsed = parse_allowed_origins(" https://app.example.com,https://www.example.com ");
        assert_eq!(
            parsed,
            vec![
                "https://app.example.com".to_string(),
                "https://www.example.com".to_string()
            ]
        );
    }

    #[test]
    fn parse_allowed_origins_normalizes_trailing_slashes() {
        let parsed = parse_allowed_origins("https://app.example.com/,http://localhost:3000/");
        assert_eq!(
            parsed,
            vec![
                "https://app.example.com".to_string(),
                "http://localhost:3000".to_string()
            ]
        );
    }

    #[test]
    fn parse_allowed_origins_unset_is_distinct_from_empty() {
        assert_eq!(
            parse_allowed_origins_env(None),
            AllowedOriginsSetting::Unset
        );
        assert_eq!(
            parse_allowed_origins_env(Some("")),
            AllowedOriginsSetting::Explicit(Vec::new())
        );
    }

    #[test]
    fn parse_bool_env_accepts_true_case_insensitively() {
        assert!(parse_bool_env(Some("true")));
        assert!(parse_bool_env(Some("TRUE")));
        assert!(parse_bool_env(Some(" True ")));
        assert!(!parse_bool_env(Some("false")));
        assert!(!parse_bool_env(None));
    }

    #[test]
    fn parse_u64_env_uses_default_for_missing_or_invalid_values() {
        let key = "TEST_RATE_LIMIT_PER_SECOND";
        std::env::remove_var(key);
        assert_eq!(parse_u64_env(key, 20), 20);

        std::env::set_var(key, "abc");
        assert_eq!(parse_u64_env(key, 20), 20);

        std::env::set_var(key, "0");
        assert_eq!(parse_u64_env(key, 20), 20);

        std::env::set_var(key, "25");
        assert_eq!(parse_u64_env(key, 20), 25);
        std::env::remove_var(key);
    }

    #[test]
    fn parse_u32_env_uses_default_for_missing_or_invalid_values() {
        let key = "TEST_RATE_LIMIT_BURST";
        std::env::remove_var(key);
        assert_eq!(parse_u32_env(key, 40), 40);

        std::env::set_var(key, "-1");
        assert_eq!(parse_u32_env(key, 40), 40);

        std::env::set_var(key, "0");
        assert_eq!(parse_u32_env(key, 40), 40);

        std::env::set_var(key, "80");
        assert_eq!(parse_u32_env(key, 40), 80);
        std::env::remove_var(key);
    }

    #[test]
    fn rate_limit_config_from_env_reads_values() {
        std::env::set_var("RATE_LIMIT_PER_SECOND", "18");
        std::env::set_var("RATE_LIMIT_BURST", "36");
        std::env::set_var("RATE_LIMIT_EXPENSIVE_PER_SECOND", "4");
        std::env::set_var("RATE_LIMIT_EXPENSIVE_BURST", "8");
        std::env::set_var("TRUST_PROXY", "true");

        let config = RateLimitConfig::from_env();
        assert_eq!(config.per_second, 18);
        assert_eq!(config.burst, 36);
        assert_eq!(config.expensive_per_second, 4);
        assert_eq!(config.expensive_burst, 8);
        assert!(config.trust_proxy);

        std::env::remove_var("RATE_LIMIT_PER_SECOND");
        std::env::remove_var("RATE_LIMIT_BURST");
        std::env::remove_var("RATE_LIMIT_EXPENSIVE_PER_SECOND");
        std::env::remove_var("RATE_LIMIT_EXPENSIVE_BURST");
        std::env::remove_var("TRUST_PROXY");
    }

    #[test]
    fn parse_auth_mode_env_maps_supported_values() {
        assert_eq!(parse_auth_mode_env(None), AuthMode::Anonymous);
        assert_eq!(parse_auth_mode_env(Some("anonymous")), AuthMode::Anonymous);
        assert_eq!(parse_auth_mode_env(Some("jwt")), AuthMode::Jwt);
        assert_eq!(
            parse_auth_mode_env(Some("optional_jwt")),
            AuthMode::OptionalJwt
        );
    }

    #[test]
    fn jwt_modes_require_a_strong_configured_secret() {
        assert!(validate_jwt_secret(AuthMode::Anonymous, None).is_ok());
        assert!(validate_jwt_secret(AuthMode::Jwt, None).is_err());
        assert!(validate_jwt_secret(AuthMode::OptionalJwt, Some("too-short")).is_err());
        assert!(validate_jwt_secret(AuthMode::Jwt, Some(&"x".repeat(32))).is_ok());
    }

    #[test]
    fn port_parser_rejects_invalid_values() {
        assert_eq!(parse_port_env(None).expect("default port"), 8080);
        assert_eq!(parse_port_env(Some("3000")).expect("valid port"), 3000);
        assert!(parse_port_env(Some("0")).is_err());
        assert!(parse_port_env(Some("70000")).is_err());
        assert!(parse_port_env(Some("abc")).is_err());
    }
}
