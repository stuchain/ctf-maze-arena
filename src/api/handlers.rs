use super::{dto::*, error::ApiError, AppState, AuthClaims};
use crate::{
    services::{leaderboard as leaderboard_service, maze, run, ServiceError},
    store::{self, Identity, SubmissionOutcome},
};
use axum::{
    extract::{rejection::JsonRejection, ConnectInfo, Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

const BUILD_VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_GIT_SHA: &str = env!("GIT_SHA");

pub(super) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: BUILD_VERSION,
        git_sha: BUILD_GIT_SHA,
    })
}

pub(super) async fn ready(
    Extension(state): Extension<Arc<AppState>>,
    Extension(request_id): Extension<String>,
) -> Result<Json<Value>, ApiError> {
    store::ping(&state.db).await.map_err(|error| {
        tracing::error!(%error, "readiness database check failed");
        ApiError::from_service(ServiceError::Unavailable, &request_id)
    })?;
    Ok(Json(json!({"status": "ready"})))
}

pub(super) async fn daily() -> Json<DailyResponse> {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let seed = date.bytes().fold(0u64, |acc, byte| {
        acc.wrapping_mul(31).wrapping_add(byte as u64)
    });
    Json(DailyResponse {
        seed,
        date,
        w: 15,
        h: 15,
    })
}

pub(super) async fn generate(
    Extension(state): Extension<Arc<AppState>>,
    Extension(request_id): Extension<String>,
    payload: Result<Json<GenerateRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<GenerateResponse>), ApiError> {
    let Json(request) = payload.map_err(|error| ApiError::invalid_json(error, &request_id))?;
    let (maze_id, maze) = maze::generate_and_store(
        &state.db,
        request.w,
        request.h,
        request.seed,
        &request.algo,
        &request.feature_preset,
    )
    .await
    .map_err(|error| ApiError::from_service(error, &request_id))?;
    let maze = serde_json::to_value(maze).map_err(|error| {
        tracing::error!(%error, "maze serialization failed");
        ApiError::from_service(ServiceError::Internal, &request_id)
    })?;
    Ok((
        StatusCode::CREATED,
        Json(GenerateResponse { maze_id, maze }),
    ))
}

pub(super) async fn race(
    Extension(state): Extension<Arc<AppState>>,
    Extension(request_id): Extension<String>,
    claims: Option<Extension<AuthClaims>>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    payload: Result<Json<RaceRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<RaceResponse>), ApiError> {
    let Json(request) = payload.map_err(|error| ApiError::invalid_json(error, &request_id))?;
    if !(2..=4).contains(&request.solvers.len()) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid_race",
            "A race requires two to four solvers.",
            &request_id,
        ));
    }
    let mut unique = std::collections::HashSet::new();
    if request
        .solvers
        .iter()
        .any(|solver| !unique.insert(solver.as_str()))
    {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid_race",
            "Race solvers must be unique.",
            &request_id,
        ));
    }
    let maze_id = parse_uuid(&request.maze_id, "mazeId", &request_id)?;
    let (maze, maze_seed) = maze::get_with_seed(&state.db, maze_id)
        .await
        .map_err(|error| ApiError::from_service(error, &request_id))?;
    let identity = claims.as_ref().map(|Extension(claims)| Identity {
        github_subject: claims.sub.clone(),
        display_name: claims.name.clone(),
        avatar_url: claims.avatar_url.clone(),
    });
    let race_id = Uuid::new_v4();
    let actor = claims.as_ref().map_or_else(
        || format!("ip:{}", peer.ip()),
        |Extension(claims)| format!("user:{}", claims.sub),
    );
    let solver_names = request.solvers;
    let mut starts = Vec::with_capacity(solver_names.len());
    for solver_name in &solver_names {
        let solver = state
            .solvers
            .get(solver_name.as_str())
            .cloned()
            .ok_or_else(|| {
                ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "unknown_solver",
                    "solver must be BFS, DFS, ASTAR, or DP_KEYS.",
                    &request_id,
                )
            })?;
        starts.push(run::StartRun {
            pool: &state.db,
            streams: &state.stream_broadcasts,
            concurrency: &state.solve_concurrency,
            active_limits: &state.active_solve_limits,
            accepting: &state.accepting_solves,
            config: &state.realtime_config,
            actor: actor.clone(),
            maze_id,
            maze: maze.clone(),
            maze_seed,
            solver_name: solver_name.clone(),
            solver,
            request_id: &request_id,
            identity: identity.as_ref(),
        });
    }
    let run_ids = run::start_race(starts)
        .await
        .map_err(|error| ApiError::from_service(error, &request_id))?;
    let runs = solver_names
        .into_iter()
        .zip(run_ids)
        .map(|(solver, run_id)| RaceRunResponse { solver, run_id })
        .collect();
    Ok((
        StatusCode::ACCEPTED,
        Json(RaceResponse {
            race_id,
            runs,
            execution_mode: "sequential_compute_synchronized_playback",
        }),
    ))
}

fn parse_uuid(raw: &str, field: &str, request_id: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw).map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid_identifier",
            format!("{field} must be a valid UUID."),
            request_id,
        )
    })
}

pub(super) async fn get_maze(
    Extension(state): Extension<Arc<AppState>>,
    Extension(request_id): Extension<String>,
    Path(raw): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let maze_id = parse_uuid(&raw, "mazeId", &request_id)?;
    let value = maze::get(&state.db, maze_id)
        .await
        .map_err(|error| ApiError::from_service(error, &request_id))?;
    Ok(Json(serde_json::to_value(value).map_err(|_| {
        ApiError::from_service(ServiceError::Internal, &request_id)
    })?))
}

pub(super) async fn solve(
    Extension(state): Extension<Arc<AppState>>,
    Extension(request_id): Extension<String>,
    claims: Option<Extension<AuthClaims>>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    payload: Result<Json<SolveRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<SolveResponse>), ApiError> {
    let Json(request) = payload.map_err(|error| ApiError::invalid_json(error, &request_id))?;
    let maze_id = parse_uuid(&request.maze_id, "mazeId", &request_id)?;
    let (maze, maze_seed) = maze::get_with_seed(&state.db, maze_id)
        .await
        .map_err(|error| ApiError::from_service(error, &request_id))?;
    let solver = state.solvers.get(&request.solver).cloned().ok_or_else(|| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            "unknown_solver",
            "solver must be BFS, DFS, ASTAR, or DP_KEYS.",
            &request_id,
        )
    })?;
    let identity = claims.as_ref().map(|Extension(claims)| Identity {
        github_subject: claims.sub.clone(),
        display_name: claims.name.clone(),
        avatar_url: claims.avatar_url.clone(),
    });
    let actor = claims.as_ref().map_or_else(
        || format!("ip:{}", peer.ip()),
        |Extension(claims)| format!("user:{}", claims.sub),
    );
    let run_id = run::start(run::StartRun {
        pool: &state.db,
        streams: &state.stream_broadcasts,
        concurrency: &state.solve_concurrency,
        active_limits: &state.active_solve_limits,
        accepting: &state.accepting_solves,
        config: &state.realtime_config,
        actor,
        maze_id,
        maze,
        maze_seed,
        solver_name: request.solver,
        solver,
        request_id: &request_id,
        identity: identity.as_ref(),
    })
    .await
    .map_err(|error| ApiError::from_service(error, &request_id))?;
    Ok((StatusCode::ACCEPTED, Json(SolveResponse { run_id })))
}

pub(super) async fn cancel_run(
    Extension(state): Extension<Arc<AppState>>,
    Extension(request_id): Extension<String>,
    claims: Option<Extension<AuthClaims>>,
    Path(raw): Path<String>,
) -> Result<Json<CancelResponse>, ApiError> {
    let run_id = parse_uuid(&raw, "runId", &request_id)?;
    let subject = claims.as_ref().map(|Extension(claims)| claims.sub.as_str());
    let cancelled = run::cancel(&state.db, &state.stream_broadcasts, run_id, subject)
        .await
        .map_err(|error| ApiError::from_service(error, &request_id))?;
    Ok(Json(CancelResponse { cancelled }))
}

pub(super) async fn get_run(
    Extension(state): Extension<Arc<AppState>>,
    Extension(request_id): Extension<String>,
    Path(raw): Path<String>,
) -> Result<Json<store::RunMetadata>, ApiError> {
    let run_id = parse_uuid(&raw, "runId", &request_id)?;
    let run = store::get_run(&state.db, run_id)
        .await
        .map_err(|error| ApiError::from_service(error.into(), &request_id))?
        .ok_or_else(|| ApiError::from_service(ServiceError::NotFound, &request_id))?;
    Ok(Json(run))
}

pub(super) async fn submit_leaderboard(
    Extension(state): Extension<Arc<AppState>>,
    Extension(request_id): Extension<String>,
    claims: Option<Extension<AuthClaims>>,
    payload: Result<Json<LeaderboardSubmitRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<LeaderboardSubmitResponse>), ApiError> {
    let claims = claims
        .ok_or_else(|| ApiError::from_service(ServiceError::Unauthorized, &request_id))?
        .0;
    let Json(request) = payload.map_err(|error| ApiError::invalid_json(error, &request_id))?;
    let run_id = parse_uuid(&request.run_id, "runId", &request_id)?;
    let outcome = leaderboard_service::submit(&state.db, run_id, &claims.sub)
        .await
        .map_err(|error| ApiError::from_service(error, &request_id))?;
    let duplicate = outcome == SubmissionOutcome::Existing;
    Ok((
        if duplicate {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(LeaderboardSubmitResponse {
            accepted: true,
            duplicate,
        }),
    ))
}

pub(super) async fn get_replay(
    Extension(state): Extension<Arc<AppState>>,
    Extension(request_id): Extension<String>,
    Path(raw): Path<String>,
) -> Result<Json<crate::replay::Replay>, ApiError> {
    let run_id = parse_uuid(&raw, "runId", &request_id)?;
    let replay = store::get_replay(&state.db, run_id)
        .await
        .map_err(|error| ApiError::from_service(error.into(), &request_id))?
        .ok_or_else(|| ApiError::from_service(ServiceError::NotFound, &request_id))?;
    Ok(Json(replay))
}

pub(super) async fn leaderboard(
    Extension(state): Extension<Arc<AppState>>,
    Extension(request_id): Extension<String>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<Vec<store::LeaderboardEntry>>, ApiError> {
    let maze_id = parse_uuid(&query.maze_id, "mazeId", &request_id)?;
    let entries = leaderboard_service::list(&state.db, maze_id, query.limit, query.offset)
        .await
        .map_err(|error| ApiError::from_service(error, &request_id))?;
    Ok(Json(entries))
}
