use axum::{
    extract::{ws::WebSocketUpgrade, Path, Query},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

use crate::maze::gen::{generate, GeneratorAlgo};
use crate::solve::SolveStats;
use crate::store;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub solvers: crate::solve::SolverRegistry,
    /// Per-run broadcast senders so WebSocket clients can subscribe to frame JSON lines.
    pub stream_broadcasts: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
}

pub fn router(
    state: Arc<AppState>,
    global_rate_limit_per_second: u64,
    global_rate_limit_burst: u32,
) -> Router {
    Router::new().nest(
        "/api",
        api_routes(global_rate_limit_per_second, global_rate_limit_burst).layer(Extension(state)),
    )
}

fn api_routes(global_rate_limit_per_second: u64, global_rate_limit_burst: u32) -> Router {
    let global_limiter = GovernorConfigBuilder::default()
        .per_second(global_rate_limit_per_second)
        .burst_size(global_rate_limit_burst)
        .use_headers()
        .finish()
        .expect("valid global rate limit config");

    let exempt_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/solve/stream", get(stream_handler));

    let limited_routes = Router::new()
        .route("/maze/generate", post(generate_handler))
        .route("/maze/:maze_id", get(get_maze_handler))
        .route("/solve", post(solve_handler))
        .route("/replay/:run_id", get(replay_handler))
        .route("/leaderboard", get(leaderboard_handler))
        .route("/daily", get(daily_handler))
        .layer(GovernorLayer {
            config: Arc::new(global_limiter),
        });

    Router::new().merge(exempt_routes).merge(limited_routes)
}

async fn health_handler() -> &'static str {
    "ok"
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DailyResponse {
    seed: u64,
    date: String,
    w: u32,
    h: u32,
}

async fn daily_handler() -> Json<DailyResponse> {
    let now = chrono::Utc::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let seed = date_str
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    Json(DailyResponse {
        seed,
        date: date_str,
        w: 15,
        h: 15,
    })
}

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub w: usize,
    pub h: usize,
    pub seed: u64,
    pub algo: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResponse {
    pub maze_id: String,
    pub maze: Value,
}

const MIN_SIZE: usize = 5;
const MAX_SIZE: usize = 100;

fn validate_generate(req: &GenerateRequest) -> Result<(), String> {
    if req.w < MIN_SIZE || req.w > MAX_SIZE {
        return Err(format!("w must be {}..{}", MIN_SIZE, MAX_SIZE));
    }
    if req.h < MIN_SIZE || req.h > MAX_SIZE {
        return Err(format!("h must be {}..{}", MIN_SIZE, MAX_SIZE));
    }
    if !["KRUSKAL", "PRIM", "DFS"].contains(&req.algo.as_str()) {
        return Err("algo must be KRUSKAL, PRIM, or DFS".into());
    }
    Ok(())
}

async fn generate_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, (StatusCode, Json<Value>)> {
    if let Err(msg) = validate_generate(&req) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))));
    }
    let algo = match req.algo.as_str() {
        "KRUSKAL" => GeneratorAlgo::Kruskal,
        "PRIM" => GeneratorAlgo::Prim,
        "DFS" => GeneratorAlgo::Dfs,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "unknown algo"})),
            ))
        }
    };
    let maze = generate(req.w, req.h, req.seed, algo);
    let maze_id = store::store_maze(&state.db, &maze, req.seed, &req.algo)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    let maze_json = serde_json::to_value(&maze).unwrap();
    Ok(Json(GenerateResponse {
        maze_id,
        maze: maze_json,
    }))
}

async fn get_maze_handler(
    Extension(state): Extension<Arc<AppState>>,
    Path(maze_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let maze = store::get_maze(&state.db, &maze_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db"})),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "maze not found"})),
        ))?;

    let maze_json = serde_json::to_value(&maze).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "serialize"})),
        )
    })?;

    Ok(Json(maze_json))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolveRequest {
    pub maze_id: String,
    pub solver: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolveResponse {
    pub run_id: String,
}

async fn solve_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<SolveRequest>,
) -> Result<Json<SolveResponse>, (StatusCode, Json<Value>)> {
    let maze = store::get_maze(&state.db, &req.maze_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db"})),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "maze not found"})),
        ))?;

    let solver = state
        .solvers
        .get(&req.solver)
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "unknown solver"})),
        ))?
        .clone();

    let run_id = store::create_run(&state.db, &req.maze_id, &req.solver)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    let db = state.db.clone();
    let maze_id = req.maze_id.clone();
    let solver_name = req.solver.clone();
    let run_id_bg = run_id.clone();
    let (frame_tx, _) = broadcast::channel::<String>(4096);
    {
        let mut guard = state.stream_broadcasts.write().await;
        guard.insert(run_id.clone(), frame_tx.clone());
    }

    let stream_map = state.stream_broadcasts.clone();
    let frame_tx_bg = frame_tx.clone();
    tokio::spawn(async move {
        let result = solver.solve(&maze);
        let _ = store::update_run_stats(&db, &run_id_bg, &result.stats).await;
        let replay = crate::replay::build_replay(&maze_id, &solver_name, 0, result, 5);
        for f in &replay.frames {
            let line = json!({"type": "frame", "data": f}).to_string();
            let _ = frame_tx_bg.send(line);
        }
        let _ = store::save_replay(&db, &run_id_bg, &replay).await;
        let finished = json!({
            "type": "finished",
            "path": replay.path,
            "stats": replay.stats,
        })
        .to_string();
        let _ = frame_tx_bg.send(finished);
        stream_map.write().await.remove(&run_id_bg);
    });

    Ok(Json(SolveResponse { run_id }))
}

async fn replay_handler(
    Extension(state): Extension<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Result<Json<crate::replay::Replay>, (StatusCode, Json<Value>)> {
    let replay = store::get_replay(&state.db, &run_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db"})),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "replay not found"})),
        ))?;
    Ok(Json(replay))
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    #[serde(rename = "mazeId")]
    pub maze_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub run_id: String,
    pub solver: String,
    pub cost: usize,
    pub ms: u64,
    pub visited: usize,
}

async fn leaderboard_handler(
    Extension(state): Extension<Arc<AppState>>,
    Query(q): Query<LeaderboardQuery>,
) -> Result<Json<Vec<LeaderboardEntry>>, StatusCode> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT r.id, r.solver, r.stats_json FROM runs r WHERE r.maze_id = ? AND r.status = 'completed'",
    )
    .bind(&q.maze_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut entries: Vec<LeaderboardEntry> = rows
        .into_iter()
        .filter_map(|(id, solver, stats_json)| {
            let stats_json = stats_json?;
            let stats: SolveStats = serde_json::from_str(&stats_json).ok()?;
            Some(LeaderboardEntry {
                run_id: id,
                solver,
                cost: stats.cost,
                ms: stats.ms,
                visited: stats.visited,
            })
        })
        .collect();
    entries.sort_by(|a, b| {
        a.cost
            .cmp(&b.cost)
            .then(a.ms.cmp(&b.ms))
            .then(a.visited.cmp(&b.visited))
    });
    Ok(Json(entries.into_iter().take(50).collect()))
}

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    #[serde(rename = "runId")]
    pub run_id: String,
}

async fn stream_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
    Query(q): Query<StreamQuery>,
) -> axum::response::Response {
    let state = Arc::clone(&state);
    let run_id = q.run_id.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, state, run_id))
}

async fn handle_socket(
    mut socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
    run_id: String,
) {
    use axum::extract::ws::Message;
    use broadcast::error::RecvError;

    let hello = json!({"type": "connected", "runId": run_id}).to_string();
    if socket.send(Message::Text(hello.into())).await.is_err() {
        return;
    }

    let mut rx = {
        let map = state.stream_broadcasts.read().await;
        match map.get(&run_id) {
            Some(tx) => tx.subscribe(),
            None => {
                let err =
                    json!({"type": "error", "error": "unknown or completed runId"}).to_string();
                let _ = socket.send(Message::Text(err.into())).await;
                return;
            }
        }
    };

    loop {
        match rx.recv().await {
            Ok(text) => {
                // Client disconnected: stop forwarding; solver task may still finish and persist replay.
                if socket.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            Err(RecvError::Lagged(_)) => continue,
            Err(RecvError::Closed) => break,
        }
    }
}
