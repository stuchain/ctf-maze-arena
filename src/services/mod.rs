pub mod leaderboard;
pub mod maze;
pub mod run;

use crate::store::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("resource not found")]
    NotFound,
    #[error("authentication is required")]
    Unauthorized,
    #[error("you do not own this run")]
    Forbidden,
    #[error("run is not completed")]
    Conflict,
    #[error("service is temporarily unavailable")]
    Unavailable,
    #[error("too many active solves")]
    TooManyRequests,
    #[error("service is shutting down")]
    ShuttingDown,
    #[error("internal operation failed")]
    Internal,
}

impl From<StoreError> for ServiceError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::NotFound => Self::NotFound,
            StoreError::Forbidden => Self::Forbidden,
            StoreError::RunNotCompleted | StoreError::InvalidTransition => Self::Conflict,
            StoreError::Database(_) => Self::Unavailable,
            StoreError::InvalidData(_) | StoreError::NumericOverflow => Self::Internal,
        }
    }
}
