use ctf_maze_arena::api;
use ctf_maze_arena::solve;
use axum::http::{header, HeaderName, HeaderValue};
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, PartialEq, Eq)]
enum AllowedOriginsSetting {
    Unset,
    Explicit(Vec<String>),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = init_db().await?;
    tracing::info!("database initialized");

    let state = Arc::new(api::AppState {
        db: pool,
        solvers: solve::default_registry(),
        stream_broadcasts: Arc::new(RwLock::new(HashMap::new())),
    });

    let cors = cors_layer_from_env();

    let app = api::router(state)
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
    axum::serve(listener, app).await?;
    Ok(())
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
        parse_allowed_origins, parse_allowed_origins_env, parse_bool_env, AllowedOriginsSetting,
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
}
