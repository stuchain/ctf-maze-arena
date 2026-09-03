use super::ServiceError;
use crate::maze::Maze;
use crate::replay;
use crate::solve::Solver;
use crate::store::{self, Identity, RunId};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Semaphore};

pub type StreamMap = Arc<RwLock<HashMap<RunId, broadcast::Sender<String>>>>;

pub struct StartRun<'a> {
    pub pool: &'a PgPool,
    pub streams: &'a StreamMap,
    pub concurrency: &'a Arc<Semaphore>,
    pub maze_id: uuid::Uuid,
    pub maze: Maze,
    pub solver_name: String,
    pub solver: Arc<dyn Solver>,
    pub request_id: &'a str,
    pub identity: Option<&'a Identity>,
}

pub async fn start(input: StartRun<'_>) -> Result<RunId, ServiceError> {
    let run_id = store::create_run(
        input.pool,
        input.maze_id,
        &input.solver_name,
        input.request_id,
        input.identity,
    )
    .await?;
    let (frame_tx, _) = broadcast::channel(4096);
    input.streams.write().await.insert(run_id, frame_tx.clone());

    let pool = input.pool.clone();
    let streams = Arc::clone(input.streams);
    let concurrency = Arc::clone(input.concurrency);
    let maze_id = input.maze_id;
    let solver_name = input.solver_name;
    let solver = input.solver;
    let maze = input.maze;
    tokio::spawn(async move {
        let permit = match concurrency.acquire_owned().await {
            Ok(permit) => permit,
            Err(error) => {
                tracing::error!(%run_id, %error, "solver concurrency gate closed");
                persist_failure(&pool, run_id, "worker_unavailable").await;
                streams.write().await.remove(&run_id);
                return;
            }
        };
        if let Err(error) = store::transition_to_running(&pool, run_id).await {
            tracing::error!(%run_id, %error, "failed to mark run running");
            persist_failure(&pool, run_id, "persistence_failed").await;
            streams.write().await.remove(&run_id);
            return;
        }
        let task = tokio::task::spawn_blocking(move || solver.solve(&maze));
        let result = match task.await {
            Ok(result) => result,
            Err(error) => {
                tracing::error!(%run_id, %error, "solver task failed");
                persist_failure(&pool, run_id, "solver_failed").await;
                streams.write().await.remove(&run_id);
                drop(permit);
                return;
            }
        };
        let stats = result.stats.clone();
        let replay = replay::build_replay(maze_id.to_string(), &solver_name, 0, result, 5);
        if let Err(error) = store::complete_run(&pool, run_id, &stats, &replay).await {
            tracing::error!(%run_id, %error, "failed to persist completed run");
            persist_failure(&pool, run_id, "persistence_failed").await;
        } else {
            for frame in &replay.frames {
                let _ =
                    frame_tx.send(serde_json::json!({"type": "frame", "data": frame}).to_string());
            }
            let _ = frame_tx.send(
                serde_json::json!({
                    "type": "finished", "path": replay.path, "stats": replay.stats
                })
                .to_string(),
            );
        }
        streams.write().await.remove(&run_id);
        drop(permit);
    });
    Ok(run_id)
}

async fn persist_failure(pool: &PgPool, run_id: RunId, code: &str) {
    if let Err(error) = store::fail_run(pool, run_id, code).await {
        tracing::error!(%run_id, %error, "failed to persist terminal run failure");
    }
}
