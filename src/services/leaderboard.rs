use super::ServiceError;
use crate::store::{self, LeaderboardEntry, MazeId, RunId, SubmissionOutcome};
use sqlx::PgPool;

pub async fn submit(
    pool: &PgPool,
    run_id: RunId,
    subject: &str,
) -> Result<SubmissionOutcome, ServiceError> {
    Ok(store::submit_leaderboard_run(pool, run_id, subject).await?)
}

pub async fn list(
    pool: &PgPool,
    maze_id: MazeId,
    limit: u32,
    offset: u32,
) -> Result<Vec<LeaderboardEntry>, ServiceError> {
    if limit == 0 || limit > 100 {
        return Err(ServiceError::InvalidInput("limit must be 1..100".into()));
    }
    if offset > 10_000 {
        return Err(ServiceError::InvalidInput("offset must be 0..10000".into()));
    }
    Ok(store::list_leaderboard(pool, maze_id, i64::from(limit), i64::from(offset)).await?)
}
