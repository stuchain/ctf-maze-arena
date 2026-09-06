use crate::domain::RunStatus;
use crate::maze::Maze;
use crate::replay::Replay;
use crate::solve::SolveStats;
use chrono::{DateTime, NaiveDate, Utc};
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
    pub rank: u64,
    pub tied: bool,
    pub run_id: RunId,
    pub solver: String,
    pub cost: u64,
    pub ms: u64,
    pub visited: u64,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub accepted_at: DateTime<Utc>,
    pub is_personal: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyChallenge {
    pub id: Uuid,
    pub date: NaiveDate,
    pub version: u16,
    pub seed: u64,
    pub width: u16,
    pub height: u16,
    pub generator_algo: String,
    pub feature_preset: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementAward {
    pub key: String,
    pub version: u16,
    pub name: String,
    pub description: String,
    pub awarded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRun {
    pub run_id: Uuid,
    pub solver: String,
    pub cost: u64,
    pub ms: u64,
    pub visited: u64,
    pub accepted_at: DateTime<Utc>,
    pub challenge_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub provider_subject: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub total_submissions: u64,
    pub daily_streak: u32,
    pub achievements: Vec<AchievementAward>,
    pub history: Vec<ProfileRun>,
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
    #[error("submission rate limit exceeded")]
    RateLimited,
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
    store_maze_for_challenge(pool, maze, seed, algo, None).await
}

pub async fn store_maze_for_challenge(
    pool: &PgPool,
    maze: &Maze,
    seed: u64,
    algo: &str,
    daily_challenge_id: Option<Uuid>,
) -> Result<MazeId, StoreError> {
    let id = Uuid::new_v4();
    let payload = serde_json::to_value(maze)?;
    sqlx::query("INSERT INTO mazes (id, width, height, seed, generator_algo, payload, daily_challenge_id) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(id)
        .bind(i16::try_from(maze.grid.width).map_err(|_| StoreError::NumericOverflow)?)
        .bind(i16::try_from(maze.grid.height).map_err(|_| StoreError::NumericOverflow)?)
        .bind(i64::try_from(seed).map_err(|_| StoreError::NumericOverflow)?)
        .bind(algo)
        .bind(payload)
        .bind(daily_challenge_id)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn get_maze(pool: &PgPool, id: MazeId) -> Result<Option<Maze>, StoreError> {
    Ok(get_maze_with_seed(pool, id)
        .await?
        .map(|(maze, _seed)| maze))
}

pub async fn get_maze_with_seed(
    pool: &PgPool,
    id: MazeId,
) -> Result<Option<(Maze, u64)>, StoreError> {
    let row = sqlx::query_as::<_, (serde_json::Value, i64)>(
        "SELECT payload, seed FROM mazes WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    row.map(|(payload, seed)| {
        let maze = serde_json::from_value(payload)?;
        let seed = u64::try_from(seed).map_err(|_| StoreError::NumericOverflow)?;
        Ok((maze, seed))
    })
    .transpose()
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

pub async fn upsert_identity(pool: &PgPool, identity: &Identity) -> Result<Uuid, StoreError> {
    let mut tx = pool.begin().await?;
    let id = upsert_user(&mut tx, identity).await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn create_run(
    pool: &PgPool,
    maze_id: MazeId,
    solver: &str,
    request_id: &str,
    identity: Option<&Identity>,
) -> Result<RunId, StoreError> {
    Ok(create_runs(pool, maze_id, &[solver], request_id, identity)
        .await?
        .remove(0))
}

pub async fn create_runs(
    pool: &PgPool,
    maze_id: MazeId,
    solvers: &[&str],
    request_id: &str,
    identity: Option<&Identity>,
) -> Result<Vec<RunId>, StoreError> {
    create_runs_with_race(pool, maze_id, solvers, request_id, identity, None).await
}

pub async fn create_runs_with_race(
    pool: &PgPool,
    maze_id: MazeId,
    solvers: &[&str],
    request_id: &str,
    identity: Option<&Identity>,
    race_id: Option<Uuid>,
) -> Result<Vec<RunId>, StoreError> {
    let mut tx = pool.begin().await?;
    let owner_user_id = match identity {
        Some(identity) => Some(upsert_user(&mut tx, identity).await?),
        None => None,
    };
    let mut ids = Vec::with_capacity(solvers.len());
    for solver in solvers {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO runs (id, maze_id, owner_user_id, solver, status, request_id, race_id) VALUES ($1, $2, $3, $4, 'queued', $5, $6)")
            .bind(id).bind(maze_id).bind(owner_user_id).bind(solver).bind(request_id).bind(race_id)
            .execute(&mut *tx).await?;
        ids.push(id);
    }
    tx.commit().await?;
    Ok(ids)
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
    let payload_bytes = i32::try_from(serde_json::to_vec(replay)?.len())
        .map_err(|_| StoreError::NumericOverflow)?;
    let payload = serde_json::to_value(replay)?;
    let mut tx = pool.begin().await?;
    let result = sqlx::query(
        r#"UPDATE runs SET status = 'completed', visited = $2, cost = $3, duration_ms = $4, peak_frontier = $5,
           completed_at = NOW() WHERE id = $1 AND status = 'running'"#,
    )
    .bind(run_id)
    .bind(i64::try_from(stats.visited).map_err(|_| StoreError::NumericOverflow)?)
    .bind(i64::try_from(stats.cost).map_err(|_| StoreError::NumericOverflow)?)
    .bind(i64::try_from(stats.ms).map_err(|_| StoreError::NumericOverflow)?)
    .bind(i64::try_from(stats.peak_frontier).map_err(|_| StoreError::NumericOverflow)?)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() != 1 {
        return Err(StoreError::InvalidTransition);
    }
    sqlx::query(
        "INSERT INTO replays (id, run_id, protocol_version, payload, payload_bytes) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(run_id)
    .bind(i16::try_from(replay.protocol_version).map_err(|_| StoreError::NumericOverflow)?)
    .bind(payload)
    .bind(payload_bytes)
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

pub async fn cancel_run(
    pool: &PgPool,
    run_id: RunId,
    github_subject: Option<&str>,
) -> Result<bool, StoreError> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        r#"SELECT r.status, u.github_subject FROM runs r
           LEFT JOIN users u ON u.id = r.owner_user_id WHERE r.id = $1 FOR UPDATE OF r"#,
    )
    .bind(run_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(StoreError::NotFound)?;
    if row.1.is_some() && row.1.as_deref() != github_subject {
        return Err(StoreError::Forbidden);
    }
    if row.0 == "cancelled" {
        return Ok(false);
    }
    if row.0 != "queued" && row.0 != "running" {
        return Err(StoreError::InvalidTransition);
    }
    sqlx::query("UPDATE runs SET status = 'cancelled', completed_at = NOW() WHERE id = $1")
        .bind(run_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(true)
}

pub async fn authorize_run_cancellation(
    pool: &PgPool,
    run_id: RunId,
    github_subject: Option<&str>,
) -> Result<(), StoreError> {
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        r#"SELECT r.status, u.github_subject FROM runs r
           LEFT JOIN users u ON u.id = r.owner_user_id WHERE r.id = $1"#,
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await?
    .ok_or(StoreError::NotFound)?;
    if row.1.is_some() && row.1.as_deref() != github_subject {
        return Err(StoreError::Forbidden);
    }
    if row.0 != "queued" && row.0 != "running" && row.0 != "cancelled" {
        return Err(StoreError::InvalidTransition);
    }
    Ok(())
}

pub async fn cancel_run_system(pool: &PgPool, run_id: RunId) -> Result<bool, StoreError> {
    let result = sqlx::query(
        "UPDATE runs SET status = 'cancelled', completed_at = NOW() WHERE id = $1 AND status IN ('queued', 'running')",
    )
    .bind(run_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}

pub async fn delete_expired_replays(pool: &PgPool) -> Result<u64, StoreError> {
    Ok(sqlx::query("DELETE FROM replays WHERE expires_at <= NOW()")
        .execute(pool)
        .await?
        .rows_affected())
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
        Option<i64>,
        Option<String>,
        DateTime<Utc>,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
    );
    let row: Option<Row> = sqlx::query_as(
        r#"SELECT id, maze_id, solver, status, visited, cost, duration_ms, peak_frontier, error_code,
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
            peak_frontier,
            error_code,
            created_at,
            started_at,
            completed_at,
        )| {
            let status = status.parse().map_err(|_| StoreError::InvalidTransition)?;
            let stats = match (visited, cost, duration_ms, peak_frontier) {
                (Some(visited), Some(cost), Some(ms), Some(peak_frontier)) => Some(SolveStats {
                    visited: usize::try_from(visited).map_err(|_| StoreError::NumericOverflow)?,
                    cost: usize::try_from(cost).map_err(|_| StoreError::NumericOverflow)?,
                    ms: u64::try_from(ms).map_err(|_| StoreError::NumericOverflow)?,
                    peak_frontier: usize::try_from(peak_frontier)
                        .map_err(|_| StoreError::NumericOverflow)?,
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
    let payload = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT payload FROM replays WHERE run_id = $1 AND expires_at > NOW()",
    )
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
    let row = sqlx::query_as::<_, (String, Option<String>, Option<Uuid>, String, Option<i64>, Option<Uuid>)>(
        r#"SELECT r.status, u.github_subject, r.owner_user_id, r.solver, r.visited, m.daily_challenge_id FROM runs r
           JOIN mazes m ON m.id = r.maze_id
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
    let already_submitted: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM leaderboard_submissions WHERE run_id = $1)",
    )
    .bind(run_id)
    .fetch_one(&mut *tx)
    .await?;
    if already_submitted {
        return Ok(SubmissionOutcome::Existing);
    }
    let recent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM leaderboard_submissions WHERE user_id = $1 AND accepted_at >= NOW() - INTERVAL '24 hours'",
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;
    if recent >= 100 {
        return Err(StoreError::RateLimited);
    }
    let result = sqlx::query(
        "INSERT INTO leaderboard_submissions (run_id, user_id) VALUES ($1, $2) ON CONFLICT (run_id) DO NOTHING")
        .bind(run_id).bind(user_id).execute(&mut *tx).await?;
    if result.rows_affected() == 1 {
        let mut keys = vec!["first_finish"];
        if row.4.is_some_and(|visited| visited < 100) {
            keys.push("efficient");
        }
        if row.3 == "ASTAR" {
            keys.push("astar_optimal");
        }
        if row.3 == "DP_KEYS" {
            keys.push("key_master");
        }
        if row.5.is_some() {
            keys.push("daily_challenger");
        }
        for key in keys {
            sqlx::query(
                r#"INSERT INTO user_achievements (user_id, achievement_key, achievement_version, run_id)
                   VALUES ($1, $2, 1, $3) ON CONFLICT DO NOTHING"#,
            )
            .bind(user_id)
            .bind(key)
            .bind(run_id)
            .execute(&mut *tx)
            .await?;
        }
    }
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
    list_leaderboard_filtered(pool, Some(maze_id), None, None, None, None, limit, offset).await
}

#[allow(clippy::too_many_arguments)]
pub async fn list_leaderboard_filtered(
    pool: &PgPool,
    maze_id: Option<MazeId>,
    daily_date: Option<NaiveDate>,
    race_id: Option<Uuid>,
    solver: Option<&str>,
    personal_subject: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LeaderboardEntry>, StoreError> {
    type Row = (
        i64,
        i64,
        Uuid,
        String,
        i64,
        i64,
        i64,
        Option<String>,
        Option<String>,
        DateTime<Utc>,
        bool,
    );
    let rows: Vec<Row> = sqlx::query_as(
        r#"WITH ranked AS (
             SELECT
               DENSE_RANK() OVER (ORDER BY r.cost, r.duration_ms, r.visited)::BIGINT AS rank,
               COUNT(*) OVER (PARTITION BY r.cost, r.duration_ms, r.visited)::BIGINT AS tie_count,
               r.id, r.solver, r.cost, r.duration_ms, r.visited,
               CASE WHEN u.deleted_at IS NULL THEN u.display_name ELSE 'Deleted player' END AS display_name,
               CASE WHEN u.deleted_at IS NULL THEN u.avatar_url ELSE NULL END AS avatar_url,
               s.accepted_at, COALESCE(u.github_subject = $5, false) AS is_personal
             FROM leaderboard_submissions s
             JOIN runs r ON r.id = s.run_id
             JOIN mazes m ON m.id = r.maze_id
             LEFT JOIN daily_challenges d ON d.id = m.daily_challenge_id
             JOIN users u ON u.id = s.user_id
             WHERE r.status = 'completed'
               AND ($1::UUID IS NULL OR r.maze_id = $1)
               AND ($2::DATE IS NULL OR d.challenge_date = $2)
               AND ($3::UUID IS NULL OR r.race_id = $3)
               AND ($4::TEXT IS NULL OR r.solver = $4)
           )
           SELECT rank, tie_count, id, solver, cost, duration_ms, visited, display_name,
             avatar_url, accepted_at, is_personal
           FROM ranked WHERE ($5::TEXT IS NULL OR is_personal)
           ORDER BY cost, duration_ms, visited, accepted_at, id
           LIMIT $6 OFFSET $7"#,
    )
    .bind(maze_id)
    .bind(daily_date)
    .bind(race_id)
    .bind(solver)
    .bind(personal_subject)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(
            |(
                rank,
                tie_count,
                run_id,
                solver,
                cost,
                ms,
                visited,
                display_name,
                avatar_url,
                accepted_at,
                is_personal,
            )| {
                Ok(LeaderboardEntry {
                    rank: u64::try_from(rank).map_err(|_| StoreError::NumericOverflow)?,
                    tied: tie_count > 1,
                    run_id,
                    solver,
                    cost: u64::try_from(cost).map_err(|_| StoreError::NumericOverflow)?,
                    ms: u64::try_from(ms).map_err(|_| StoreError::NumericOverflow)?,
                    visited: u64::try_from(visited).map_err(|_| StoreError::NumericOverflow)?,
                    display_name,
                    avatar_url,
                    accepted_at,
                    is_personal,
                })
            },
        )
        .collect()
}

pub async fn get_or_create_daily_challenge(
    pool: &PgPool,
    date: NaiveDate,
    version: u16,
    seed: u64,
) -> Result<DailyChallenge, StoreError> {
    type Row = (Uuid, NaiveDate, i16, i64, i16, i16, String, String);
    let row: Row = sqlx::query_as(
        r#"INSERT INTO daily_challenges
             (id, challenge_date, version, seed, width, height, generator_algo, feature_preset)
           VALUES ($1, $2, $3, $4, 15, 15, 'KRUSKAL', 'classic')
           ON CONFLICT (challenge_date, version) DO UPDATE SET challenge_date = EXCLUDED.challenge_date
           RETURNING id, challenge_date, version, seed, width, height, generator_algo, feature_preset"#,
    )
    .bind(Uuid::new_v4())
    .bind(date)
    .bind(i16::try_from(version).map_err(|_| StoreError::NumericOverflow)?)
    .bind(i64::try_from(seed).map_err(|_| StoreError::NumericOverflow)?)
    .fetch_one(pool)
    .await?;
    Ok(DailyChallenge {
        id: row.0,
        date: row.1,
        version: u16::try_from(row.2).map_err(|_| StoreError::NumericOverflow)?,
        seed: u64::try_from(row.3).map_err(|_| StoreError::NumericOverflow)?,
        width: u16::try_from(row.4).map_err(|_| StoreError::NumericOverflow)?,
        height: u16::try_from(row.5).map_err(|_| StoreError::NumericOverflow)?,
        generator_algo: row.6,
        feature_preset: row.7,
    })
}

pub async fn validate_daily_challenge(
    pool: &PgPool,
    id: Uuid,
    seed: u64,
    width: usize,
    height: usize,
    algorithm: &str,
    feature_preset: &str,
) -> Result<(), StoreError> {
    let matches: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
             SELECT 1 FROM daily_challenges WHERE id = $1 AND seed = $2 AND width = $3
             AND height = $4 AND generator_algo = $5 AND feature_preset = $6
           )"#,
    )
    .bind(id)
    .bind(i64::try_from(seed).map_err(|_| StoreError::NumericOverflow)?)
    .bind(i16::try_from(width).map_err(|_| StoreError::NumericOverflow)?)
    .bind(i16::try_from(height).map_err(|_| StoreError::NumericOverflow)?)
    .bind(algorithm)
    .bind(feature_preset)
    .fetch_one(pool)
    .await?;
    if matches {
        Ok(())
    } else {
        Err(StoreError::NotFound)
    }
}

pub async fn daily_personal_state(
    pool: &PgPool,
    challenge_id: Uuid,
    subject: &str,
) -> Result<(Option<LeaderboardEntry>, u32), StoreError> {
    let best = sqlx::query_as::<_, (Uuid, String, i64, i64, i64, Option<String>, Option<String>, DateTime<Utc>)>(
        r#"SELECT r.id, r.solver, r.cost, r.duration_ms, r.visited, u.display_name, u.avatar_url, s.accepted_at
           FROM leaderboard_submissions s JOIN runs r ON r.id = s.run_id
           JOIN mazes m ON m.id = r.maze_id JOIN users u ON u.id = s.user_id
           WHERE m.daily_challenge_id = $1 AND u.github_subject = $2 AND r.status = 'completed'
           ORDER BY r.cost, r.duration_ms, r.visited, s.accepted_at, r.id LIMIT 1"#,
    )
    .bind(challenge_id)
    .bind(subject)
    .fetch_optional(pool)
    .await?
    .map(|row| -> Result<LeaderboardEntry, StoreError> {
        Ok(LeaderboardEntry {
            rank: 0,
            tied: false,
            run_id: row.0,
            solver: row.1,
            cost: u64::try_from(row.2).map_err(|_| StoreError::NumericOverflow)?,
            ms: u64::try_from(row.3).map_err(|_| StoreError::NumericOverflow)?,
            visited: u64::try_from(row.4).map_err(|_| StoreError::NumericOverflow)?,
            display_name: row.5,
            avatar_url: row.6,
            accepted_at: row.7,
            is_personal: true,
        })
    })
    .transpose()?;
    let dates: Vec<NaiveDate> = sqlx::query_scalar(
        r#"SELECT DISTINCT d.challenge_date FROM leaderboard_submissions s
           JOIN runs r ON r.id = s.run_id JOIN mazes m ON m.id = r.maze_id
           JOIN daily_challenges d ON d.id = m.daily_challenge_id JOIN users u ON u.id = s.user_id
           WHERE u.github_subject = $1 ORDER BY d.challenge_date DESC"#,
    )
    .bind(subject)
    .fetch_all(pool)
    .await?;
    Ok((best, calculate_streak(&dates, Utc::now().date_naive())))
}

fn calculate_streak(dates: &[NaiveDate], today: NaiveDate) -> u32 {
    let Some(first) = dates.first().copied() else {
        return 0;
    };
    if first != today && first != today.pred_opt().unwrap_or(today) {
        return 0;
    }
    let mut streak = 1;
    let mut expected = first;
    for date in dates.iter().copied().skip(1) {
        expected = expected.pred_opt().unwrap_or(expected);
        if date != expected {
            break;
        }
        streak += 1;
    }
    streak
}

pub async fn get_profile(pool: &PgPool, identity: &Identity) -> Result<UserProfile, StoreError> {
    type HistoryRow = (
        Uuid,
        String,
        i64,
        i64,
        i64,
        DateTime<Utc>,
        Option<NaiveDate>,
    );
    let user_id = upsert_identity(pool, identity).await?;
    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM leaderboard_submissions WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    let achievements: Vec<(String, i16, String, String, DateTime<Utc>)> = sqlx::query_as(
        r#"SELECT a.achievement_key, a.achievement_version, d.name, d.description, a.awarded_at
           FROM user_achievements a JOIN achievement_definitions d
           ON d.achievement_key = a.achievement_key AND d.version = a.achievement_version
           WHERE a.user_id = $1 ORDER BY a.awarded_at, a.achievement_key"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let history: Vec<HistoryRow> = sqlx::query_as(
        r#"SELECT r.id, r.solver, r.cost, r.duration_ms, r.visited, s.accepted_at, d.challenge_date
           FROM leaderboard_submissions s JOIN runs r ON r.id = s.run_id
           JOIN mazes m ON m.id = r.maze_id LEFT JOIN daily_challenges d ON d.id = m.daily_challenge_id
           WHERE s.user_id = $1 ORDER BY s.accepted_at DESC, r.id DESC LIMIT 50"#,
    ).bind(user_id).fetch_all(pool).await?;
    let dates: Vec<NaiveDate> = sqlx::query_scalar(
        r#"SELECT DISTINCT d.challenge_date FROM leaderboard_submissions s JOIN runs r ON r.id = s.run_id
           JOIN mazes m ON m.id = r.maze_id JOIN daily_challenges d ON d.id = m.daily_challenge_id
           WHERE s.user_id = $1 ORDER BY d.challenge_date DESC"#,
    ).bind(user_id).fetch_all(pool).await?;
    Ok(UserProfile {
        provider_subject: identity.github_subject.clone(),
        display_name: identity.display_name.clone(),
        avatar_url: identity.avatar_url.clone(),
        total_submissions: u64::try_from(total).map_err(|_| StoreError::NumericOverflow)?,
        daily_streak: calculate_streak(&dates, Utc::now().date_naive()),
        achievements: achievements
            .into_iter()
            .map(|a| AchievementAward {
                key: a.0,
                version: u16::try_from(a.1).unwrap_or_default(),
                name: a.2,
                description: a.3,
                awarded_at: a.4,
            })
            .collect(),
        history: history
            .into_iter()
            .map(|r| -> Result<ProfileRun, StoreError> {
                Ok(ProfileRun {
                    run_id: r.0,
                    solver: r.1,
                    cost: u64::try_from(r.2).map_err(|_| StoreError::NumericOverflow)?,
                    ms: u64::try_from(r.3).map_err(|_| StoreError::NumericOverflow)?,
                    visited: u64::try_from(r.4).map_err(|_| StoreError::NumericOverflow)?,
                    accepted_at: r.5,
                    challenge_date: r.6,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub async fn delete_profile(pool: &PgPool, subject: &str) -> Result<bool, StoreError> {
    let mut tx = pool.begin().await?;
    let user_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM users WHERE github_subject = $1 AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(subject)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(user_id) = user_id else {
        return Ok(false);
    };
    sqlx::query("DELETE FROM user_achievements WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        r#"UPDATE runs SET owner_user_id = NULL WHERE owner_user_id = $1
           AND NOT EXISTS (SELECT 1 FROM leaderboard_submissions s WHERE s.run_id = runs.id)"#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE users SET github_subject = $2, display_name = NULL, avatar_url = NULL, deleted_at = NOW(), updated_at = NOW() WHERE id = $1",
    ).bind(user_id).bind(format!("deleted:{user_id}")).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(true)
}

#[cfg(test)]
mod community_tests {
    use super::calculate_streak;
    use chrono::NaiveDate;

    #[test]
    fn streak_accepts_today_or_yesterday_and_stops_at_gap() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 5).unwrap();
        assert_eq!(
            calculate_streak(&[today, today.pred_opt().unwrap()], today),
            2
        );
        assert_eq!(calculate_streak(&[today.pred_opt().unwrap()], today), 1);
        assert_eq!(calculate_streak(&[today - chrono::Days::new(2)], today), 0);
        assert_eq!(
            calculate_streak(&[today, today - chrono::Days::new(2)], today),
            1
        );
    }
}
