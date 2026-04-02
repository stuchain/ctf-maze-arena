mod maze;
mod solve;
mod replay;
mod store;

use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    match init_db().await {
        Ok(_) => tracing::info!("database initialized"),
        Err(e) => tracing::warn!("database init failed: {e}"),
    }

    tracing::info!("server starting");
}

async fn init_db() -> Result<sqlx::SqlitePool, sqlx::Error> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/ctf_maze.db".into());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    // Apply migrations on startup so tables exist before we store anything.
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
