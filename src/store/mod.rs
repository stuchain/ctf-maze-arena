use crate::maze::Maze;
use sqlx::SqlitePool;
use uuid::Uuid;

pub type MazeId = String;
pub type RunId = String;

pub async fn store_maze(
    pool: &SqlitePool,
    maze: &Maze,
    seed: u64,
    algo: &str,
) -> Result<MazeId, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    // Serialization should not fail for our in-memory structures, so we keep it simple.
    let walls_json = serde_json::to_string(&maze.walls).unwrap();
    let keys_json = serde_json::to_string(&maze.keys).unwrap();
    let doors_json = serde_json::to_string(&maze.doors).unwrap();

    sqlx::query(
        r#"
        INSERT INTO mazes (id, width, height, seed, generator_algo, walls_json, keys_json, doors_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(maze.grid.width as i64)
    .bind(maze.grid.height as i64)
    .bind(seed as i64)
    .bind(algo)
    .bind(&walls_json)
    .bind(&keys_json)
    .bind(&doors_json)
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn get_maze(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<Maze>, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64, i64, String, String, String)>(
        "SELECT width, height, walls_json, keys_json, doors_json FROM mazes WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some((w, h, walls_json, keys_json, doors_json)) = row else {
        return Ok(None);
    };

    // If DB data was produced by `store_maze`, this should succeed.
    let maze = Maze::from_json(w as usize, h as usize, &walls_json, &keys_json, &doors_json)
        .expect("maze JSON should deserialize");

    Ok(Some(maze))
}

