use axum::{
    Extension, Json, Router,
    extract::{Query, ws::WebSocketUpgrade},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::maze::gen::{GeneratorAlgo, generate};
use crate::store;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub solvers: crate::solve::SolverRegistry,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new().nest(
        "/api",
        api_routes().layer(Extension(state)),
    )
}

fn api_routes() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/maze/generate", post(generate_handler))
        .route("/solve", post(solve_handler))
        .route("/solve/stream", get(stream_handler))
}

async fn health_handler() -> &'static str {
    "ok"
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
    Ok(Json(GenerateResponse { maze_id, maze: maze_json }))
}

#[derive(Debug, Deserialize)]
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
    tokio::spawn(async move {
        let result = solver.solve(&maze);
        let _ = store::update_run_stats(&db, &run_id_bg, &result.stats).await;
        let replay = crate::replay::build_replay(&maze_id, &solver_name, 0, result, 5);
        let _ = store::save_replay(&db, &run_id_bg, &replay).await;
    });

    Ok(Json(SolveResponse { run_id }))
}

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    #[serde(rename = "runId")]
    pub run_id: String,
}

async fn stream_handler(
    ws: WebSocketUpgrade,
    Extension(_state): Extension<Arc<AppState>>,
    Query(q): Query<StreamQuery>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(socket, q.run_id))
}

async fn handle_socket(mut socket: axum::extract::ws::WebSocket, run_id: String) {
    let msg = serde_json::json!({"type": "connected", "runId": run_id}).to_string();
    let _ = socket
        .send(axum::extract::ws::Message::Text(msg))
        .await;
}
