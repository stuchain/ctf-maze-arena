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
    solve::{self, SolveResult, SolveStats, Solver},
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
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{RwLock, Semaphore};
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "phase-02-integration-secret-32-bytes-minimum";

fn build_app(pool: PgPool, solvers: solve::SolverRegistry, concurrency: usize) -> Router {
    let state = Arc::new(AppState {
        db: pool,
        solvers,
        stream_broadcasts: Arc::new(RwLock::new(HashMap::new())),
        solve_concurrency: Arc::new(Semaphore::new(concurrency)),
    });
    api::router(state, 10_000, 10_000, 10_000, 10_000, false)
        .layer(middleware::from_fn_with_state(
            JwtConfig {
                secret: Some(JWT_SECRET.into()),
                clock_skew_secs: 60,
                auth_mode: AuthMode::OptionalJwt,
            },
            api::jwt_claims_middleware,
        ))
        .layer(middleware::from_fn(api::request_id_middleware))
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
        maze_id: maze_id.to_string(),
        solver: "BFS".into(),
        seed: 1,
        frames: vec![],
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
            frames: vec![],
        }
    }
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

    let app = build_app(pool.clone(), solve::default_registry(), 1);
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
    let panic_app = build_app(pool.clone(), panic_registry, 1);
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

    // CPU-bound tasks respect the configured concurrency cap.
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let slow: Arc<dyn Solver> = Arc::new(SlowSolver {
        active: Arc::clone(&active),
        peak: Arc::clone(&peak),
    });
    let streams = Arc::new(RwLock::new(HashMap::new()));
    let gate = Arc::new(Semaphore::new(1));
    let maze_a = store::get_maze(&pool, maze_id).await.unwrap().unwrap();
    let maze_b = store::get_maze(&pool, maze_id).await.unwrap().unwrap();
    let run_a = run::start(run::StartRun {
        pool: &pool,
        streams: &streams,
        concurrency: &gate,
        maze_id,
        maze: maze_a,
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
        maze_id,
        maze: maze_b,
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
