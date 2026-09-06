use super::ServiceError;
use crate::store::{self, LeaderboardEntry, MazeId, RunId, SubmissionOutcome};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn submit(
    pool: &PgPool,
    run_id: RunId,
    subject: &str,
) -> Result<SubmissionOutcome, ServiceError> {
    Ok(store::submit_leaderboard_run(pool, run_id, subject).await?)
}

pub struct Filters<'a> {
    pub maze_id: Option<MazeId>,
    pub daily_date: Option<NaiveDate>,
    pub race_id: Option<Uuid>,
    pub solver: Option<&'a str>,
    pub personal_subject: Option<&'a str>,
    pub limit: u32,
    pub offset: u32,
}

pub async fn list_filtered(
    pool: &PgPool,
    filters: Filters<'_>,
) -> Result<Vec<LeaderboardEntry>, ServiceError> {
    if filters.maze_id.is_none() && filters.daily_date.is_none() && filters.race_id.is_none() {
        return Err(ServiceError::InvalidInput(
            "mazeId, dailyDate, or raceId is required".into(),
        ));
    }
    if filters.limit == 0 || filters.limit > 100 {
        return Err(ServiceError::InvalidInput("limit must be 1..100".into()));
    }
    if filters.offset > 10_000 {
        return Err(ServiceError::InvalidInput("offset must be 0..10000".into()));
    }
    if let Some(solver) = filters.solver {
        if !matches!(solver, "BFS" | "DFS" | "ASTAR" | "DP_KEYS") {
            return Err(ServiceError::InvalidInput(
                "solver must be BFS, DFS, ASTAR, or DP_KEYS".into(),
            ));
        }
    }
    Ok(store::list_leaderboard_filtered(
        pool,
        filters.maze_id,
        filters.daily_date,
        filters.race_id,
        filters.solver,
        filters.personal_subject,
        i64::from(filters.limit),
        i64::from(filters.offset),
    )
    .await?)
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
