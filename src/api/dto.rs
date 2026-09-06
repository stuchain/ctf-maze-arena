use crate::store::LeaderboardEntry;
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
    pub challenge_id: Uuid,
    pub seed: u64,
    pub date: String,
    pub version: u16,
    pub w: u32,
    pub h: u32,
    pub algo: String,
    pub feature_preset: String,
    pub seconds_until_reset: i64,
    pub personal_best: Option<LeaderboardEntry>,
    pub streak: u32,
    pub completed: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct GenerateRequest {
    pub w: usize,
    pub h: usize,
    pub seed: u64,
    pub algo: String,
    #[serde(default = "default_feature_preset", rename = "featurePreset")]
    pub feature_preset: String,
    #[serde(default, rename = "dailyChallengeId")]
    pub daily_challenge_id: Option<Uuid>,
}

fn default_feature_preset() -> String {
    "classic".to_string()
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
pub(super) struct RaceRequest {
    pub maze_id: String,
    pub solvers: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RaceRunResponse {
    pub solver: String,
    pub run_id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RaceResponse {
    pub race_id: Uuid,
    pub runs: Vec<RaceRunResponse>,
    pub execution_mode: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct CancelResponse {
    pub cancelled: bool,
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
    #[serde(default)]
    pub maze_id: Option<String>,
    #[serde(default)]
    pub daily_date: Option<String>,
    #[serde(default)]
    pub race_id: Option<String>,
    #[serde(default)]
    pub solver: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DeleteProfileResponse {
    pub deleted: bool,
    pub leaderboard_policy: &'static str,
}

fn default_limit() -> u32 {
    50
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StreamQuery {
    pub run_id: String,
    #[serde(default)]
    pub after_sequence: u64,
}
