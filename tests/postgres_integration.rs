use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware, Router,
};
use ctf_maze_arena::{
    api::{self, AppState, AuthClaims, AuthMode, JwtConfig},
    domain::RunStatus,
    maze::Maze,
    replay::{Replay, ReplayStats},
    services::run,
    solve::{self, ProgressSink, SolveError, SolveProgress, SolveResult, SolveStats, Solver},
    store::{self, Identity, StoreError},
};
use http_body_util::BodyExt;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{RwLock, Semaphore};
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "phase-02-integration-secret-32-bytes-minimum";

fn build_app(
    pool: PgPool,
    solvers: solve::SolverRegistry,
    concurrency: usize,
) -> (Router, Arc<AppState>) {
    let state = Arc::new(AppState {
        db: pool,
        solvers,
        stream_broadcasts: Arc::new(RwLock::new(HashMap::new())),
        solve_concurrency: Arc::new(Semaphore::new(concurrency)),
        active_solve_limits: run::ActiveSolveLimiter::new(10),
        accepting_solves: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        realtime_config: run::RealtimeConfig {
            terminal_retention: Duration::from_secs(2),
            ..Default::default()
        },
    });
    let router = api::router(Arc::clone(&state), 10_000, 10_000, 10_000, 10_000, false)
        .layer(middleware::from_fn_with_state(
            JwtConfig {
                secret: Some(JWT_SECRET.into()),
                clock_skew_secs: 60,
                auth_mode: AuthMode::OptionalJwt,
            },
            api::jwt_claims_middleware,
        ))
        .layer(middleware::from_fn(api::request_id_middleware));
    (router, state)
}

fn request(method: &str, uri: &str, body: Option<Value>, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("x-request-id", "phase02-test");
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let mut request = builder
        .body(Body::from(
            body.map_or_else(String::new, |value| value.to_string()),
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 32100))));
    request
}

async fn call(app: &Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    assert_eq!(
        response.headers().get("x-request-id").unwrap(),
        "phase02-test"
    );
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, body)
}

fn token(subject: &str) -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    encode(
        &Header::new(Algorithm::HS256),
        &AuthClaims {
            sub: subject.into(),
            name: Some(subject.into()),
            avatar_url: Some("https://example.test/avatar.png".into()),
            iat: now,
            exp: now + 300,
        },
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .unwrap()
}

async fn wait_for_status(pool: &PgPool, run_id: Uuid, expected: RunStatus) -> store::RunMetadata {
    for _ in 0..100 {
        let run = store::get_run(pool, run_id).await.unwrap().unwrap();
        if run.status == expected {
            return run;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("run {run_id} did not reach {expected}");
}

fn dummy_replay(maze_id: Uuid, stats: &SolveStats) -> Replay {
    Replay {
        protocol_version: 1,
        maze_id: maze_id.to_string(),
        solver: "BFS".into(),
        seed: 1,
        events: vec![],
        path: vec![],
        stats: ReplayStats {
            visited: stats.visited,
            cost: stats.cost,
            ms: stats.ms,
        },
    }
}

struct PanicSolver;
impl Solver for PanicSolver {
    fn name(&self) -> &'static str {
        "BFS"
    }
    fn solve(&self, _: &Maze) -> SolveResult {
        panic!("intentional integration-test panic")
    }
}

struct SlowSolver {
    active: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
}
impl Solver for SlowSolver {
    fn name(&self) -> &'static str {
        "BFS"
    }
    fn solve(&self, _: &Maze) -> SolveResult {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(active, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(75));
        self.active.fetch_sub(1, Ordering::SeqCst);
        SolveResult {
            path: vec![],
            stats: SolveStats {
                visited: 0,
                cost: 0,
                ms: 75,
            },
        }
    }
}

struct CancellableSolver;
impl Solver for CancellableSolver {
    fn name(&self) -> &'static str {
        "BFS"
    }
    fn solve(&self, _: &Maze) -> SolveResult {
        SolveResult {
            path: vec![],
            stats: SolveStats {
                visited: 200,
                cost: 0,
                ms: 400,
            },
        }
    }
    fn solve_with_progress(
        &self,
        _: &Maze,
        progress: &mut dyn ProgressSink,
        cancelled: &AtomicBool,
    ) -> Result<SolveResult, SolveError> {
        for step in 1..=200 {
            if cancelled.load(Ordering::Acquire) {
                return Err(SolveError::Cancelled);
            }
            progress.progress(SolveProgress {
                step,
                frontier: vec![[step % 50, 1]],
                visited: (0..step.min(50)).map(|x| [x, 0]).collect(),
                current: Some([step % 50, 0]),
            });
            std::thread::sleep(Duration::from_millis(2));
        }
        Ok(self.solve(&Maze::new(5, 5)))
    }
}

#[tokio::test]
async fn phase_03_realtime_cancellation_limits_and_shutdown_contracts() {
    let database_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("TEST_DATABASE_URL is not set; skipping PostgreSQL integration test");
            return;
        }
    };
    assert!(database_url.contains("ctf_maze_test"));
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .unwrap();
    store::migrate(&pool).await.unwrap();
    let maze =
        ctf_maze_arena::maze::generate(50, 50, 303, ctf_maze_arena::maze::GeneratorAlgo::Kruskal);
    let maze_id = store::store_maze(&pool, &maze, 303, "KRUSKAL")
        .await
        .unwrap();

    // A subscriber attached while work is active sees genuine intermediate progress.
    let mut registry = solve::default_registry();
    registry.insert("BFS".into(), Arc::new(CancellableSolver));
    let (app, state) = build_app(pool.clone(), registry, 1);
    let (status, body) = call(
        &app,
        request(
            "POST",
            "/api/solve",
            Some(json!({"mazeId": maze_id, "solver": "BFS"})),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let run_id = Uuid::parse_str(body["runId"].as_str().unwrap()).unwrap();
    let stream = state
        .stream_broadcasts
        .read()
        .await
        .get(&run_id)
        .cloned()
        .unwrap();
    let mut subscriber = stream.subscribe();
    let progress = tokio::time::timeout(Duration::from_secs(1), subscriber.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        progress,
        ctf_maze_arena::realtime::ServerMessage::Snapshot { .. }
            | ctf_maze_arena::realtime::ServerMessage::Delta { .. }
    ));

    let (status, cancelled) = call(
        &app,
        request("POST", &format!("/api/run/{run_id}/cancel"), None, None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled, json!({"cancelled": true}));
    wait_for_status(&pool, run_id, RunStatus::Cancelled).await;
    assert!(stream.resume(0).messages.iter().any(|message| matches!(
        message,
        ctf_maze_arena::realtime::ServerMessage::Cancelled { .. }
    )));

    // Ownership is checked before the cancellation signal can reach a solver.
    let owned_streams = Arc::new(RwLock::new(HashMap::new()));
    let owned_gate = Arc::new(Semaphore::new(0));
    let owned_limits = run::ActiveSolveLimiter::new(1);
    let owned_accepting = Arc::new(AtomicBool::new(true));
    let owned_config = run::RealtimeConfig::default();
    let owner = Identity {
        github_subject: "github-owner".into(),
        display_name: Some("Owner".into()),
        avatar_url: None,
    };
    let owned_run = run::start(run::StartRun {
        pool: &pool,
        streams: &owned_streams,
        concurrency: &owned_gate,
        active_limits: &owned_limits,
        accepting: &owned_accepting,
        config: &owned_config,
        actor: "user:github-owner".into(),
        maze_id,
        maze: store::get_maze(&pool, maze_id).await.unwrap().unwrap(),
        maze_seed: 123,
        solver_name: "BFS".into(),
        solver: Arc::new(CancellableSolver),
        request_id: "owned-cancellation",
        identity: Some(&owner),
    })
    .await
    .unwrap();
    let owned_stream = owned_streams.read().await.get(&owned_run).cloned().unwrap();
    assert!(matches!(
        run::cancel(&pool, &owned_streams, owned_run, Some("wrong-owner")).await,
        Err(ctf_maze_arena::services::ServiceError::Forbidden)
    ));
    assert!(!owned_stream.is_cancelled());
    assert!(
        run::cancel(&pool, &owned_streams, owned_run, Some("github-owner"))
            .await
            .unwrap()
    );
    wait_for_status(&pool, owned_run, RunStatus::Cancelled).await;
    run::shutdown(&pool, &owned_streams, &owned_accepting, &owned_gate).await;

    // Per-actor limits reject excess queued work, and graceful shutdown terminalizes it.
    let streams = Arc::new(RwLock::new(HashMap::new()));
    let gate = Arc::new(Semaphore::new(0));
    let limits = run::ActiveSolveLimiter::new(1);
    let accepting = Arc::new(AtomicBool::new(true));
    let config = run::RealtimeConfig {
        terminal_retention: Duration::from_millis(50),
        ..Default::default()
    };
    let first = run::start(run::StartRun {
        pool: &pool,
        streams: &streams,
        concurrency: &gate,
        active_limits: &limits,
        accepting: &accepting,
        config: &config,
        actor: "ip:test".into(),
        maze_id,
        maze: store::get_maze(&pool, maze_id).await.unwrap().unwrap(),
        maze_seed: 123,
        solver_name: "BFS".into(),
        solver: Arc::new(CancellableSolver),
        request_id: "queued-first",
        identity: None,
    })
    .await
    .unwrap();
    let rejected = run::start(run::StartRun {
        pool: &pool,
        streams: &streams,
        concurrency: &gate,
        active_limits: &limits,
        accepting: &accepting,
        config: &config,
        actor: "ip:test".into(),
        maze_id,
        maze: store::get_maze(&pool, maze_id).await.unwrap().unwrap(),
        maze_seed: 123,
        solver_name: "BFS".into(),
        solver: Arc::new(CancellableSolver),
        request_id: "queued-second",
        identity: None,
    })
    .await;
    assert!(matches!(
        rejected,
        Err(ctf_maze_arena::services::ServiceError::TooManyRequests)
    ));
    run::shutdown(&pool, &streams, &accepting, &gate).await;
    wait_for_status(&pool, first, RunStatus::Cancelled).await;
    assert!(!accepting.load(Ordering::Acquire));
    let rejected_shutdown = run::start(run::StartRun {
        pool: &pool,
        streams: &streams,
        concurrency: &gate,
        active_limits: &limits,
        accepting: &accepting,
        config: &config,
        actor: "other".into(),
        maze_id,
        maze: store::get_maze(&pool, maze_id).await.unwrap().unwrap(),
        maze_seed: 123,
        solver_name: "BFS".into(),
        solver: Arc::new(CancellableSolver),
        request_id: "after-shutdown",
        identity: None,
    })
    .await;
    assert!(matches!(
        rejected_shutdown,
        Err(ctf_maze_arena::services::ServiceError::ShuttingDown)
    ));
}

#[tokio::test]
async fn phase_02_postgres_http_and_lifecycle_contracts() {
    let database_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("TEST_DATABASE_URL is not set; skipping PostgreSQL integration test");
            return;
        }
    };
    assert!(
        database_url.contains("ctf_maze_test"),
        "integration tests require a dedicated ctf_maze_test database"
    );
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .unwrap();

    // A clean database migrates successfully, and startup migration is idempotent.
    store::migrate(&pool).await.unwrap();
    store::migrate(&pool).await.unwrap();
    assert_eq!(store::recover_interrupted_runs(&pool).await.unwrap(), 0);

    let (app, state) = build_app(pool.clone(), solve::default_registry(), 1);
    assert_eq!(
        call(&app, request("GET", "/api/health", None, None))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, request("GET", "/api/ready", None, None)).await.0,
        StatusCode::OK
    );

    let (status, invalid) = call(
        &app,
        request("POST", "/api/maze/generate", Some(json!({"w": 5})), None),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid["error"]["code"], "invalid_json");
    assert_eq!(invalid["requestId"], "phase02-test");

    let (status, generated) = call(
        &app,
        request(
            "POST",
            "/api/maze/generate",
            Some(json!({"w": 7, "h": 7, "seed": 42, "algo": "KRUSKAL"})),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let maze_id = Uuid::parse_str(generated["mazeId"].as_str().unwrap()).unwrap();
    assert_eq!(generated["maze"]["grid"]["width"], 7);
    assert!(store::get_maze(&pool, maze_id).await.unwrap().is_some());

    // Anonymous play is durable but cannot enter the authenticated leaderboard.
    let (status, anonymous) = call(
        &app,
        request(
            "POST",
            "/api/solve",
            Some(json!({"mazeId": maze_id, "solver": "BFS"})),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let anonymous_run = Uuid::parse_str(anonymous["runId"].as_str().unwrap()).unwrap();
    wait_for_status(&pool, anonymous_run, RunStatus::Completed).await;
    let retained = state
        .stream_broadcasts
        .read()
        .await
        .get(&anonymous_run)
        .cloned()
        .expect("fast completed stream retained");
    assert!(retained.resume(0).messages.iter().any(|message| matches!(
        message,
        ctf_maze_arena::realtime::ServerMessage::Completed { .. }
    )));
    assert_eq!(
        call(
            &app,
            request(
                "POST",
                "/api/leaderboard",
                Some(json!({"runId": anonymous_run})),
                None
            )
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(
            &app,
            request("GET", &format!("/api/replay/{anonymous_run}"), None, None)
        )
        .await
        .0,
        StatusCode::OK
    );

    let owner_subject = "github:phase02-owner";
    let owner_token = token(owner_subject);
    let (status, owned) = call(
        &app,
        request(
            "POST",
            "/api/solve",
            Some(json!({"mazeId": maze_id, "solver": "ASTAR"})),
            Some(&owner_token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let owned_run = Uuid::parse_str(owned["runId"].as_str().unwrap()).unwrap();
    let completed = wait_for_status(&pool, owned_run, RunStatus::Completed).await;
    assert!(completed.stats.is_some());
    assert!(completed.started_at.is_some() && completed.completed_at.is_some());

    let wrong_token = token("github:someone-else");
    let (status, forbidden) = call(
        &app,
        request(
            "POST",
            "/api/leaderboard",
            Some(json!({"runId": owned_run})),
            Some(&wrong_token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(forbidden["error"]["code"], "run_not_owned");
    let (status, accepted) = call(
        &app,
        request(
            "POST",
            "/api/leaderboard",
            Some(json!({"runId": owned_run})),
            Some(&owner_token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(accepted, json!({"accepted": true, "duplicate": false}));
    let (status, duplicate) = call(
        &app,
        request(
            "POST",
            "/api/leaderboard",
            Some(json!({"runId": owned_run})),
            Some(&owner_token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(duplicate, json!({"accepted": true, "duplicate": true}));
    let (status, board) = call(
        &app,
        request(
            "GET",
            &format!("/api/leaderboard?mazeId={maze_id}&limit=50&offset=0"),
            None,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(board.as_array().unwrap().len(), 1);
    assert_eq!(board[0]["runId"], owned_run.to_string());

    // Every state transition is guarded and completed metrics/replay commit atomically.
    let identity = Identity {
        github_subject: "github:state-test".into(),
        display_name: None,
        avatar_url: None,
    };
    let queued = store::create_run(&pool, maze_id, "BFS", "state-test", Some(&identity))
        .await
        .unwrap();
    assert!(matches!(
        store::submit_leaderboard_run(&pool, queued, &identity.github_subject).await,
        Err(StoreError::RunNotCompleted)
    ));
    store::transition_to_running(&pool, queued).await.unwrap();
    assert!(matches!(
        store::transition_to_running(&pool, queued).await,
        Err(StoreError::InvalidTransition)
    ));
    let stats = SolveStats {
        visited: 10,
        cost: 5,
        ms: 3,
    };
    store::complete_run(&pool, queued, &stats, &dummy_replay(maze_id, &stats))
        .await
        .unwrap();
    assert!(store::get_replay(&pool, queued).await.unwrap().is_some());
    store::submit_leaderboard_run(&pool, queued, &identity.github_subject)
        .await
        .unwrap();
    assert!(matches!(
        store::complete_run(&pool, queued, &stats, &dummy_replay(maze_id, &stats)).await,
        Err(StoreError::InvalidTransition)
    ));
    let failed = store::create_run(&pool, maze_id, "BFS", "failure-test", None)
        .await
        .unwrap();
    store::fail_run(&pool, failed, "test_failure")
        .await
        .unwrap();
    let failed = wait_for_status(&pool, failed, RunStatus::Failed).await;
    assert_eq!(failed.error_code.as_deref(), Some("test_failure"));

    // Equal scores have deterministic ordering and pagination validation is enforced.
    let tie_identity = Identity {
        github_subject: "github:tie-test".into(),
        display_name: Some("Tie".into()),
        avatar_url: None,
    };
    let tie = store::create_run(&pool, maze_id, "BFS", "tie-test", Some(&tie_identity))
        .await
        .unwrap();
    store::transition_to_running(&pool, tie).await.unwrap();
    store::complete_run(&pool, tie, &stats, &dummy_replay(maze_id, &stats))
        .await
        .unwrap();
    store::submit_leaderboard_run(&pool, tie, &tie_identity.github_subject)
        .await
        .unwrap();
    let first = store::list_leaderboard(&pool, maze_id, 100, 0)
        .await
        .unwrap();
    let second = store::list_leaderboard(&pool, maze_id, 100, 0)
        .await
        .unwrap();
    assert_eq!(
        first.iter().map(|entry| entry.run_id).collect::<Vec<_>>(),
        second.iter().map(|entry| entry.run_id).collect::<Vec<_>>()
    );
    assert_eq!(
        call(
            &app,
            request(
                "GET",
                &format!("/api/leaderboard?mazeId={maze_id}&limit=101"),
                None,
                None
            )
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );

    // A solver panic is converted to a durable failure rather than a stuck run.
    let mut panic_registry = solve::default_registry();
    panic_registry.insert("BFS".into(), Arc::new(PanicSolver));
    let (panic_app, panic_state) = build_app(pool.clone(), panic_registry, 1);
    let (_, panic_run) = call(
        &panic_app,
        request(
            "POST",
            "/api/solve",
            Some(json!({"mazeId": maze_id, "solver": "BFS"})),
            None,
        ),
    )
    .await;
    let panic_id = Uuid::parse_str(panic_run["runId"].as_str().unwrap()).unwrap();
    assert_eq!(
        wait_for_status(&pool, panic_id, RunStatus::Failed)
            .await
            .error_code
            .as_deref(),
        Some("solver_failed")
    );
    let failed_stream = panic_state
        .stream_broadcasts
        .read()
        .await
        .get(&panic_id)
        .cloned()
        .unwrap();
    assert!(failed_stream.resume(0).messages.iter().any(|message| matches!(message, ctf_maze_arena::realtime::ServerMessage::Failed { code, .. } if code == "solver_failed")));

    // CPU-bound tasks respect the configured concurrency cap.
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let slow: Arc<dyn Solver> = Arc::new(SlowSolver {
        active: Arc::clone(&active),
        peak: Arc::clone(&peak),
    });
    let streams = Arc::new(RwLock::new(HashMap::new()));
    let gate = Arc::new(Semaphore::new(1));
    let active_limits = run::ActiveSolveLimiter::new(10);
    let accepting = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let realtime_config = run::RealtimeConfig::default();
    let maze_a = store::get_maze(&pool, maze_id).await.unwrap().unwrap();
    let maze_b = store::get_maze(&pool, maze_id).await.unwrap().unwrap();
    let run_a = run::start(run::StartRun {
        pool: &pool,
        streams: &streams,
        concurrency: &gate,
        active_limits: &active_limits,
        accepting: &accepting,
        config: &realtime_config,
        actor: "bounded-a".into(),
        maze_id,
        maze: maze_a,
        maze_seed: 123,
        solver_name: "BFS".into(),
        solver: Arc::clone(&slow),
        request_id: "bounded-a",
        identity: None,
    })
    .await
    .unwrap();
    let run_b = run::start(run::StartRun {
        pool: &pool,
        streams: &streams,
        concurrency: &gate,
        active_limits: &active_limits,
        accepting: &accepting,
        config: &realtime_config,
        actor: "bounded-b".into(),
        maze_id,
        maze: maze_b,
        maze_seed: 123,
        solver_name: "BFS".into(),
        solver: slow,
        request_id: "bounded-b",
        identity: None,
    })
    .await
    .unwrap();
    wait_for_status(&pool, run_a, RunStatus::Completed).await;
    wait_for_status(&pool, run_b, RunStatus::Completed).await;
    assert_eq!(peak.load(Ordering::SeqCst), 1);

    // The ranking path has supporting indexes, verified through PostgreSQL's catalog and plan.
    let indexes: Vec<String> = sqlx::query_scalar("SELECT indexname FROM pg_indexes WHERE schemaname = 'public' AND tablename IN ('runs', 'leaderboard_submissions')")
        .fetch_all(&pool).await.unwrap();
    assert!(indexes
        .iter()
        .any(|name| name == "runs_leaderboard_sort_idx"));
    assert!(indexes
        .iter()
        .any(|name| name == "leaderboard_submissions_pkey"));
    let mut plan_tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL enable_seqscan = off")
        .execute(&mut *plan_tx)
        .await
        .unwrap();
    let plan: Vec<String> = sqlx::query_scalar(
        "EXPLAIN SELECT r.id FROM runs r JOIN leaderboard_submissions s ON s.run_id = r.id WHERE r.maze_id = $1 AND r.status = 'completed' ORDER BY r.cost, r.duration_ms, r.visited, r.id LIMIT 50")
        .bind(maze_id).fetch_all(&mut *plan_tx).await.unwrap();
    plan_tx.rollback().await.unwrap();
    assert!(plan.join("\n").contains("runs_leaderboard_sort_idx"));

    // Closed-pool failures stay safe and retain a request correlation ID.
    pool.close().await;
    let (status, unavailable) = call(&app, request("GET", "/api/ready", None, None)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(unavailable["error"]["code"], "database_unavailable");
    let serialized = unavailable.to_string().to_ascii_lowercase();
    assert!(
        !serialized.contains("sql")
            && !serialized.contains("pool")
            && !serialized.contains("postgres")
    );
}
