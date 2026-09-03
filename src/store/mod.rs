use crate::domain::RunStatus;
use crate::maze::Maze;
use crate::replay::Replay;
use crate::solve::SolveStats;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub type MazeId = Uuid;
pub type RunId = Uuid;

#[derive(Debug, Clone)]
pub struct Identity {
    pub github_subject: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunMetadata {
    pub id: RunId,
    pub maze_id: MazeId,
    pub solver: String,
    pub status: RunStatus,
    pub stats: Option<SolveStats>,
    pub error_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub run_id: RunId,
    pub solver: String,
    pub cost: u64,
    pub ms: u64,
    pub visited: u64,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmissionOutcome {
    Created,
    Existing,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database operation failed")]
    Database(#[from] sqlx::Error),
    #[error("stored data is invalid")]
    InvalidData(#[from] serde_json::Error),
    #[error("resource not found")]
    NotFound,
    #[error("run does not belong to this user")]
    Forbidden,
    #[error("run is not completed")]
    RunNotCompleted,
    #[error("invalid run state transition")]
    InvalidTransition,
    #[error("numeric value exceeds storage limits")]
    NumericOverflow,
}

pub async fn migrate(pool: &PgPool) -> Result<(), StoreError> {
    sqlx::migrate!()
        .run(pool)
        .await
        .map_err(sqlx::Error::from)?;
    Ok(())
}

pub async fn ping(pool: &PgPool) -> Result<(), StoreError> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}

/// A single-instance deployment cannot resume in-memory solver work after a process restart.
/// Convert orphaned active rows to an explicit terminal failure before accepting traffic.
pub async fn recover_interrupted_runs(pool: &PgPool) -> Result<u64, StoreError> {
    let result = sqlx::query(
        r#"UPDATE runs SET status = 'failed', error_code = 'worker_interrupted', completed_at = NOW()
           WHERE status IN ('queued', 'running')"#,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn store_maze(
    pool: &PgPool,
    maze: &Maze,
    seed: u64,
    algo: &str,
) -> Result<MazeId, StoreError> {
    let id = Uuid::new_v4();
    let payload = serde_json::to_value(maze)?;
    sqlx::query("INSERT INTO mazes (id, width, height, seed, generator_algo, payload) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(id)
        .bind(i16::try_from(maze.grid.width).map_err(|_| StoreError::NumericOverflow)?)
        .bind(i16::try_from(maze.grid.height).map_err(|_| StoreError::NumericOverflow)?)
        .bind(i64::try_from(seed).map_err(|_| StoreError::NumericOverflow)?)
        .bind(algo)
        .bind(payload)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn get_maze(pool: &PgPool, id: MazeId) -> Result<Option<Maze>, StoreError> {
    let payload =
        sqlx::query_scalar::<_, serde_json::Value>("SELECT payload FROM mazes WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    payload
        .map(serde_json::from_value)
        .transpose()
        .map_err(Into::into)
}

async fn upsert_user(
    tx: &mut Transaction<'_, Postgres>,
    identity: &Identity,
) -> Result<Uuid, StoreError> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO users (id, github_subject, display_name, avatar_url)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (github_subject) DO UPDATE SET display_name = EXCLUDED.display_name,
             avatar_url = EXCLUDED.avatar_url, updated_at = NOW()
           RETURNING id"#,
    )
    .bind(Uuid::new_v4())
    .bind(&identity.github_subject)
    .bind(&identity.display_name)
    .bind(&identity.avatar_url)
    .fetch_one(&mut **tx)
    .await?)
}

pub async fn create_run(
    pool: &PgPool,
    maze_id: MazeId,
    solver: &str,
    request_id: &str,
    identity: Option<&Identity>,
) -> Result<RunId, StoreError> {
    let mut tx = pool.begin().await?;
    let owner_user_id = match identity {
        Some(identity) => Some(upsert_user(&mut tx, identity).await?),
        None => None,
    };
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO runs (id, maze_id, owner_user_id, solver, status, request_id) VALUES ($1, $2, $3, $4, 'queued', $5)")
        .bind(id).bind(maze_id).bind(owner_user_id).bind(solver).bind(request_id)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn transition_to_running(pool: &PgPool, run_id: RunId) -> Result<(), StoreError> {
    let result = sqlx::query("UPDATE runs SET status = 'running', started_at = NOW() WHERE id = $1 AND status = 'queued'")
        .bind(run_id).execute(pool).await?;
    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(StoreError::InvalidTransition)
    }
}

pub async fn complete_run(
    pool: &PgPool,
    run_id: RunId,
    stats: &SolveStats,
    replay: &Replay,
) -> Result<(), StoreError> {
    let payload = serde_json::to_value(replay)?;
    let mut tx = pool.begin().await?;
    let result = sqlx::query(
        r#"UPDATE runs SET status = 'completed', visited = $2, cost = $3, duration_ms = $4,
           completed_at = NOW() WHERE id = $1 AND status = 'running'"#,
    )
    .bind(run_id)
    .bind(i64::try_from(stats.visited).map_err(|_| StoreError::NumericOverflow)?)
    .bind(i64::try_from(stats.cost).map_err(|_| StoreError::NumericOverflow)?)
    .bind(i64::try_from(stats.ms).map_err(|_| StoreError::NumericOverflow)?)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() != 1 {
        return Err(StoreError::InvalidTransition);
    }
    sqlx::query(
        "INSERT INTO replays (id, run_id, protocol_version, payload) VALUES ($1, $2, 1, $3)",
    )
    .bind(Uuid::new_v4())
    .bind(run_id)
    .bind(payload)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn fail_run(pool: &PgPool, run_id: RunId, error_code: &str) -> Result<(), StoreError> {
    let result = sqlx::query(
        "UPDATE runs SET status = 'failed', error_code = $2, completed_at = NOW() WHERE id = $1 AND status IN ('queued', 'running')")
        .bind(run_id).bind(error_code).execute(pool).await?;
    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(StoreError::InvalidTransition)
    }
}

pub async fn get_run(pool: &PgPool, run_id: RunId) -> Result<Option<RunMetadata>, StoreError> {
    type Row = (
        Uuid,
        Uuid,
        String,
        String,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<String>,
        DateTime<Utc>,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
    );
    let row: Option<Row> = sqlx::query_as(
        r#"SELECT id, maze_id, solver, status, visited, cost, duration_ms, error_code,
           created_at, started_at, completed_at FROM runs WHERE id = $1"#,
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await?;
    row.map(
        |(
            id,
            maze_id,
            solver,
            status,
            visited,
            cost,
            duration_ms,
            error_code,
            created_at,
            started_at,
            completed_at,
        )| {
            let status = status.parse().map_err(|_| StoreError::InvalidTransition)?;
            let stats = match (visited, cost, duration_ms) {
                (Some(visited), Some(cost), Some(ms)) => Some(SolveStats {
                    visited: usize::try_from(visited).map_err(|_| StoreError::NumericOverflow)?,
                    cost: usize::try_from(cost).map_err(|_| StoreError::NumericOverflow)?,
                    ms: u64::try_from(ms).map_err(|_| StoreError::NumericOverflow)?,
                }),
                _ => None,
            };
            Ok(RunMetadata {
                id,
                maze_id,
                solver,
                status,
                stats,
                error_code,
                created_at,
                started_at,
                completed_at,
            })
        },
    )
    .transpose()
}

pub async fn get_replay(pool: &PgPool, run_id: RunId) -> Result<Option<Replay>, StoreError> {
    let payload =
        sqlx::query_scalar::<_, serde_json::Value>("SELECT payload FROM replays WHERE run_id = $1")
            .bind(run_id)
            .fetch_optional(pool)
            .await?;
    payload
        .map(serde_json::from_value)
        .transpose()
        .map_err(Into::into)
}

pub async fn submit_leaderboard_run(
    pool: &PgPool,
    run_id: RunId,
    github_subject: &str,
) -> Result<SubmissionOutcome, StoreError> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as::<_, (String, Option<String>, Option<Uuid>)>(
        r#"SELECT r.status, u.github_subject, r.owner_user_id FROM runs r
           LEFT JOIN users u ON u.id = r.owner_user_id WHERE r.id = $1 FOR UPDATE OF r"#,
    )
    .bind(run_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(StoreError::NotFound)?;
    if row.0 != "completed" {
        return Err(StoreError::RunNotCompleted);
    }
    if row.1.as_deref() != Some(github_subject) {
        return Err(StoreError::Forbidden);
    }
    let user_id = row.2.ok_or(StoreError::Forbidden)?;
    let result = sqlx::query(
        "INSERT INTO leaderboard_submissions (run_id, user_id) VALUES ($1, $2) ON CONFLICT (run_id) DO NOTHING")
        .bind(run_id).bind(user_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(if result.rows_affected() == 1 {
        SubmissionOutcome::Created
    } else {
        SubmissionOutcome::Existing
    })
}

pub async fn list_leaderboard(
    pool: &PgPool,
    maze_id: MazeId,
    limit: i64,
    offset: i64,
) -> Result<Vec<LeaderboardEntry>, StoreError> {
    type Row = (Uuid, String, i64, i64, i64, Option<String>, Option<String>);
    let rows: Vec<Row> = sqlx::query_as(
        r#"SELECT r.id, r.solver, r.cost, r.duration_ms, r.visited, u.display_name, u.avatar_url
           FROM leaderboard_submissions s JOIN runs r ON r.id = s.run_id JOIN users u ON u.id = s.user_id
           WHERE r.maze_id = $1 AND r.status = 'completed'
           ORDER BY r.cost ASC, r.duration_ms ASC, r.visited ASC, s.accepted_at ASC, r.id ASC
           LIMIT $2 OFFSET $3"#)
        .bind(maze_id).bind(limit).bind(offset).fetch_all(pool).await?;
    rows.into_iter()
        .map(
            |(run_id, solver, cost, ms, visited, display_name, avatar_url)| {
                Ok(LeaderboardEntry {
                    run_id,
                    solver,
                    cost: u64::try_from(cost).map_err(|_| StoreError::NumericOverflow)?,
                    ms: u64::try_from(ms).map_err(|_| StoreError::NumericOverflow)?,
                    visited: u64::try_from(visited).map_err(|_| StoreError::NumericOverflow)?,
                    display_name,
                    avatar_url,
                })
            },
        )
        .collect()
}
