use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Queued, Self::Running | Self::Cancelled | Self::Failed)
                | (
                    Self::Running,
                    Self::Completed | Self::Failed | Self::Cancelled
                )
        )
    }
}

impl fmt::Display for RunStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        formatter.write_str(value)
    }
}

impl FromStr for RunStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("unknown run status: {value}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RunStatus;

    #[test]
    fn run_state_machine_allows_only_documented_transitions() {
        let statuses = [
            RunStatus::Queued,
            RunStatus::Running,
            RunStatus::Completed,
            RunStatus::Failed,
            RunStatus::Cancelled,
        ];
        let allowed = [
            (RunStatus::Queued, RunStatus::Running),
            (RunStatus::Queued, RunStatus::Failed),
            (RunStatus::Queued, RunStatus::Cancelled),
            (RunStatus::Running, RunStatus::Completed),
            (RunStatus::Running, RunStatus::Failed),
            (RunStatus::Running, RunStatus::Cancelled),
        ];

        for from in statuses {
            for to in statuses {
                assert_eq!(
                    from.can_transition_to(to),
                    allowed.contains(&(from, to)),
                    "unexpected transition result for {from} -> {to}"
                );
            }
        }
    }
}
