use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub git_sha: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DailyResponse {
    pub seed: u64,
    pub date: String,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize)]
pub(super) struct GenerateRequest {
    pub w: usize,
    pub h: usize,
    pub seed: u64,
    pub algo: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GenerateResponse {
    pub maze_id: Uuid,
    pub maze: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SolveRequest {
    pub maze_id: String,
    pub solver: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SolveResponse {
    pub run_id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LeaderboardSubmitRequest {
    pub run_id: String,
}

#[derive(Debug, Serialize)]
pub(super) struct LeaderboardSubmitResponse {
    pub accepted: bool,
    pub duplicate: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LeaderboardQuery {
    pub maze_id: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    50
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StreamQuery {
    pub run_id: String,
}
