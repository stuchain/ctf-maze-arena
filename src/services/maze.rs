use super::ServiceError;
use crate::maze::{generate, Cell, Edge, GeneratorAlgo, Maze};
use crate::solve::{BfsSolver, Solver};
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
    feature_preset: &str,
    daily_challenge_id: Option<uuid::Uuid>,
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
    let mut maze = generate(width, height, seed, algo);
    match feature_preset {
        "classic" => {}
        "keys" => add_key_and_door(&mut maze)?,
        _ => {
            return Err(ServiceError::InvalidInput(
                "featurePreset must be classic or keys".into(),
            ))
        }
    }
    if let Some(challenge_id) = daily_challenge_id {
        store::validate_daily_challenge(
            pool,
            challenge_id,
            seed,
            width,
            height,
            algorithm,
            feature_preset,
        )
        .await
        .map_err(|error| match error {
            store::StoreError::NotFound => ServiceError::InvalidInput(
                "dailyChallengeId does not match the immutable challenge definition".into(),
            ),
            other => other.into(),
        })?;
    }
    let id =
        store::store_maze_for_challenge(pool, &maze, seed, algorithm, daily_challenge_id).await?;
    Ok((id, maze))
}

fn add_key_and_door(maze: &mut Maze) -> Result<(), ServiceError> {
    let path = BfsSolver.solve(maze).path;
    if path.len() < 4 {
        return Err(ServiceError::InvalidInput(
            "maze is too short for the keys preset".into(),
        ));
    }
    let door_index = (path.len() * 2 / 3).clamp(2, path.len() - 1);
    let key_index = (door_index / 2).max(1);
    let key_cell: Cell = path[key_index];
    let door = Edge::normalized(path[door_index - 1], path[door_index]);
    maze.keys.insert(key_cell, 0);
    maze.doors.insert(door, 0);
    Ok(())
}

pub async fn get(pool: &PgPool, id: MazeId) -> Result<Maze, ServiceError> {
    get_with_seed(pool, id).await.map(|(maze, _seed)| maze)
}

pub async fn get_with_seed(pool: &PgPool, id: MazeId) -> Result<(Maze, u64), ServiceError> {
    store::get_maze_with_seed(pool, id)
        .await?
        .ok_or(ServiceError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::add_key_and_door;
    use crate::maze::{generate, GeneratorAlgo};
    use crate::solve::{DpKeysSolver, Solver};

    #[test]
    fn key_preset_is_deterministic_and_solvable() {
        let mut first = generate(12, 12, 42, GeneratorAlgo::Kruskal);
        let mut second = generate(12, 12, 42, GeneratorAlgo::Kruskal);
        add_key_and_door(&mut first).unwrap();
        add_key_and_door(&mut second).unwrap();
        assert_eq!(first.keys, second.keys);
        assert_eq!(first.doors, second.doors);
        assert!(!DpKeysSolver.solve(&first).path.is_empty());
    }
}
