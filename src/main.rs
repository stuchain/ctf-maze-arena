use ctf_maze_arena::api;
use ctf_maze_arena::solve;
use axum::http::{header, HeaderName, HeaderValue, Request};
use axum::middleware;
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
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
        let per_second = parse_u64_env(
            "RATE_LIMIT_PER_SECOND",
            RateLimitConfig::DEFAULT_PER_SECOND,
        );
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
}

impl AuthConfig {
    const DEFAULT_CLOCK_SKEW_SECS: u64 = 60;

    fn from_env() -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let clock_skew_secs = parse_u64_env("JWT_CLOCK_SKEW_SECS", Self::DEFAULT_CLOCK_SKEW_SECS);

        Self {
            jwt_secret,
            clock_skew_secs,
        }
    }
}

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

    let state = Arc::new(api::AppState {
        db: pool,
        solvers: solve::default_registry(),
        stream_broadcasts: Arc::new(RwLock::new(HashMap::new())),
    });

    let rate_limit = RateLimitConfig::from_env();
    let auth_config = AuthConfig::from_env();
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
        state,
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
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
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
        .allow_headers([axum::http::header::CONTENT_TYPE]);

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
                    Err(_) => tracing::warn!("Ignoring invalid CORS origin in ALLOWED_ORIGINS: {}", origin),
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

fn parse_allowed_origins(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|origin| origin.trim_end_matches('/').to_string())
        .collect()
}

async fn init_db() -> Result<sqlx::SqlitePool, sqlx::Error> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data/ctf_maze.db".into());

    // If the DB path points to a file inside a folder (like `./data/ctf_maze.db`),
    // make sure the folder exists and the DB file exists.
    //
    // This prevents startup failures like:
    // `unable to open database file` when `data/ctf_maze.db` (or its parent dir)
    // doesn't exist yet.
    if let Some(path_part) = url.strip_prefix("sqlite:") {
        // `sqlite://./data/ctf_maze.db` -> `./data/ctf_maze.db`
        let path_part = path_part.trim_start_matches('/');
        let db_path = Path::new(path_part);

        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).map_err(sqlx::Error::Io)?;
        }
        if !db_path.exists() {
            fs::File::create(db_path).map_err(sqlx::Error::Io)?;
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    // Apply migrations on startup so tables exist before we store anything.
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_allowed_origins, parse_allowed_origins_env, parse_bool_env, parse_u32_env,
        parse_u64_env, AllowedOriginsSetting, RateLimitConfig,
    };

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
        assert_eq!(parse_allowed_origins_env(None), AllowedOriginsSetting::Unset);
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
}
