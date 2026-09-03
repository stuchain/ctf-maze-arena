use super::ServiceError;
use crate::maze::Maze;
use crate::realtime::{ProgressRecorder, RunStream};
use crate::replay;
use crate::solve::{SolveError, Solver};
use crate::store::{self, Identity, RunId};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};

pub type StreamMap = Arc<RwLock<HashMap<RunId, Arc<RunStream>>>>;

#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    pub history_capacity: usize,
    pub client_channel_capacity: usize,
    pub sample_every: u32,
    pub snapshot_every: u32,
    pub max_replay_events: usize,
    pub terminal_retention: Duration,
    pub heartbeat_interval: Duration,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            history_capacity: 256,
            client_channel_capacity: 32,
            sample_every: 2,
            snapshot_every: 32,
            max_replay_events: 2048,
            terminal_retention: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(10),
        }
    }
}

pub struct ActiveSolveLimiter {
    counts: Mutex<HashMap<String, usize>>,
    max_per_actor: usize,
}

impl ActiveSolveLimiter {
    pub fn new(max_per_actor: usize) -> Arc<Self> {
        Arc::new(Self {
            counts: Mutex::new(HashMap::new()),
            max_per_actor: max_per_actor.max(1),
        })
    }
    fn acquire(self: &Arc<Self>, actor: String) -> Option<ActiveSolveLease> {
        let mut counts = self
            .counts
            .lock()
            .expect("active solve limiter mutex poisoned");
        let count = counts.entry(actor.clone()).or_default();
        if *count >= self.max_per_actor {
            return None;
        }
        *count += 1;
        Some(ActiveSolveLease {
            limiter: Arc::clone(self),
            actor,
        })
    }
}

struct ActiveSolveLease {
    limiter: Arc<ActiveSolveLimiter>,
    actor: String,
}
impl Drop for ActiveSolveLease {
    fn drop(&mut self) {
        let mut counts = self
            .limiter
            .counts
            .lock()
            .expect("active solve limiter mutex poisoned");
        if let Some(count) = counts.get_mut(&self.actor) {
            *count -= 1;
            if *count == 0 {
                counts.remove(&self.actor);
            }
        }
    }
}

pub struct StartRun<'a> {
    pub pool: &'a PgPool,
    pub streams: &'a StreamMap,
    pub concurrency: &'a Arc<Semaphore>,
    pub active_limits: &'a Arc<ActiveSolveLimiter>,
    pub accepting: &'a Arc<AtomicBool>,
    pub config: &'a RealtimeConfig,
    pub actor: String,
    pub maze_id: uuid::Uuid,
    pub maze: Maze,
    pub maze_seed: u64,
    pub solver_name: String,
    pub solver: Arc<dyn Solver>,
    pub request_id: &'a str,
    pub identity: Option<&'a Identity>,
}

pub async fn start(input: StartRun<'_>) -> Result<RunId, ServiceError> {
    if !input.accepting.load(Ordering::Acquire) {
        return Err(ServiceError::ShuttingDown);
    }
    let lease = input
        .active_limits
        .acquire(input.actor)
        .ok_or(ServiceError::TooManyRequests)?;
    let run_id = store::create_run(
        input.pool,
        input.maze_id,
        &input.solver_name,
        input.request_id,
        input.identity,
    )
    .await?;
    let stream = RunStream::new(
        run_id,
        input.config.history_capacity,
        input.config.client_channel_capacity,
    );
    input
        .streams
        .write()
        .await
        .insert(run_id, Arc::clone(&stream));

    let pool = input.pool.clone();
    let streams = Arc::clone(input.streams);
    let concurrency = Arc::clone(input.concurrency);
    let config = input.config.clone();
    let maze_id = input.maze_id;
    let solver_name = input.solver_name;
    let solver = input.solver;
    let maze = input.maze;
    let maze_seed = input.maze_seed;
    tokio::spawn(async move {
        let _lease = lease;
        let permit = match concurrency.acquire_owned().await {
            Ok(permit) => permit,
            Err(error) => {
                tracing::error!(%run_id, %error, "solver concurrency gate closed");
                if stream.is_cancelled() {
                    let _ = store::cancel_run_system(&pool, run_id).await;
                    stream.cancelled();
                } else {
                    persist_failure(&pool, run_id, "worker_unavailable").await;
                    stream.fail(
                        "worker_unavailable",
                        "Solver capacity is temporarily unavailable.",
                    );
                }
                schedule_cleanup(streams, run_id, stream, config.terminal_retention);
                return;
            }
        };
        if stream.is_cancelled() {
            let _ = store::cancel_run_system(&pool, run_id).await;
            stream.cancelled();
            schedule_cleanup(streams, run_id, stream, config.terminal_retention);
            drop(permit);
            return;
        }
        if let Err(error) = store::transition_to_running(&pool, run_id).await {
            tracing::error!(%run_id, %error, "failed to mark run running");
            persist_failure(&pool, run_id, "persistence_failed").await;
            stream.fail("persistence_failed", "The run could not be started.");
            schedule_cleanup(streams, run_id, stream, config.terminal_retention);
            return;
        }

        let solver_stream = Arc::clone(&stream);
        let recorder_config = config.clone();
        let task = tokio::task::spawn_blocking(move || {
            let mut recorder = ProgressRecorder::new(
                Arc::clone(&solver_stream),
                recorder_config.sample_every,
                recorder_config.snapshot_every,
                recorder_config.max_replay_events,
            );
            let result =
                solver.solve_with_progress(&maze, &mut recorder, solver_stream.cancellation());
            let events = recorder.finish();
            (result, events)
        });
        match task.await {
            Ok((Ok(result), events)) => {
                if stream.is_cancelled() {
                    if let Err(error) = store::cancel_run_system(&pool, run_id).await {
                        tracing::error!(%run_id, %error, "failed to persist cancellation");
                    }
                    stream.cancelled();
                    drop(permit);
                    schedule_cleanup(streams, run_id, stream, config.terminal_retention);
                    return;
                }
                let stats = result.stats.clone();
                let replay = replay::build_replay(
                    maze_id.to_string(),
                    &solver_name,
                    maze_seed,
                    result,
                    events,
                );
                if let Err(error) = store::complete_run(&pool, run_id, &stats, &replay).await {
                    tracing::error!(%run_id, %error, "failed to persist completed run");
                    persist_failure(&pool, run_id, "persistence_failed").await;
                    stream.fail(
                        "persistence_failed",
                        "The completed run could not be saved.",
                    );
                } else {
                    stream.complete(replay.path.clone(), replay.stats.clone());
                }
            }
            Ok((Err(SolveError::Cancelled), _)) => {
                if let Err(error) = store::cancel_run_system(&pool, run_id).await {
                    tracing::error!(%run_id, %error, "failed to persist cancellation");
                }
                stream.cancelled();
            }
            Err(error) => {
                tracing::error!(%run_id, %error, "solver task failed");
                persist_failure(&pool, run_id, "solver_failed").await;
                stream.fail("solver_failed", "The solver stopped unexpectedly.");
            }
        }
        drop(permit);
        schedule_cleanup(streams, run_id, stream, config.terminal_retention);
    });
    Ok(run_id)
}

pub async fn cancel(
    pool: &PgPool,
    streams: &StreamMap,
    run_id: RunId,
    github_subject: Option<&str>,
) -> Result<bool, ServiceError> {
    let stream = streams
        .read()
        .await
        .get(&run_id)
        .cloned()
        .ok_or(ServiceError::NotFound)?;
    store::authorize_run_cancellation(pool, run_id, github_subject).await?;
    stream.request_cancel();
    let changed = store::cancel_run(pool, run_id, github_subject).await?;
    stream.cancelled();
    Ok(changed)
}

pub async fn shutdown(
    pool: &PgPool,
    streams: &StreamMap,
    accepting: &AtomicBool,
    concurrency: &Semaphore,
) {
    accepting.store(false, Ordering::Release);
    let active: Vec<_> = streams
        .read()
        .await
        .iter()
        .map(|(id, stream)| (*id, Arc::clone(stream)))
        .collect();
    for (run_id, stream) in active {
        if stream.is_terminal() {
            continue;
        }
        stream.request_cancel();
        if let Err(error) = store::cancel_run_system(pool, run_id).await {
            tracing::error!(%run_id, %error, "failed to persist shutdown cancellation");
        }
        stream.cancelled();
    }
    concurrency.close();
}

fn schedule_cleanup(
    streams: StreamMap,
    run_id: RunId,
    stream: Arc<RunStream>,
    retention: Duration,
) {
    tokio::spawn(async move {
        tokio::time::sleep(retention).await;
        let mut map = streams.write().await;
        if map
            .get(&run_id)
            .is_some_and(|current| Arc::ptr_eq(current, &stream))
        {
            map.remove(&run_id);
        }
    });
}

async fn persist_failure(pool: &PgPool, run_id: RunId, code: &str) {
    if let Err(error) = store::fail_run(pool, run_id, code).await {
        tracing::error!(%run_id, %error, "failed to persist terminal run failure");
    }
}
