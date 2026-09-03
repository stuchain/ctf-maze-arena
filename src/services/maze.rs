use super::ServiceError;
use crate::maze::{generate, GeneratorAlgo, Maze};
use crate::store::{self, MazeId};
use sqlx::PgPool;

pub const MIN_SIZE: usize = 5;
pub const MAX_SIZE: usize = 100;

pub async fn generate_and_store(
    pool: &PgPool,
    width: usize,
    height: usize,
    seed: u64,
    algorithm: &str,
) -> Result<(MazeId, Maze), ServiceError> {
    if !(MIN_SIZE..=MAX_SIZE).contains(&width) {
        return Err(ServiceError::InvalidInput(format!(
            "w must be {MIN_SIZE}..{MAX_SIZE}"
        )));
    }
    if !(MIN_SIZE..=MAX_SIZE).contains(&height) {
        return Err(ServiceError::InvalidInput(format!(
            "h must be {MIN_SIZE}..{MAX_SIZE}"
        )));
    }
    if seed > i64::MAX as u64 {
        return Err(ServiceError::InvalidInput(
            "seed must be at most 9223372036854775807".into(),
        ));
    }
    let algo = match algorithm {
        "KRUSKAL" => GeneratorAlgo::Kruskal,
        "PRIM" => GeneratorAlgo::Prim,
        "DFS" => GeneratorAlgo::Dfs,
        _ => {
            return Err(ServiceError::InvalidInput(
                "algo must be KRUSKAL, PRIM, or DFS".into(),
            ))
        }
    };
    let maze = generate(width, height, seed, algo);
    let id = store::store_maze(pool, &maze, seed, algorithm).await?;
    Ok((id, maze))
}

pub async fn get(pool: &PgPool, id: MazeId) -> Result<Maze, ServiceError> {
    store::get_maze(pool, id)
        .await?
        .ok_or(ServiceError::NotFound)
}
