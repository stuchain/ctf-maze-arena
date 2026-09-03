use crate::solve::{ProgressSink, SolveProgress};
use crate::store::RunId;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::broadcast;

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualState {
    pub step: u32,
    pub frontier: Vec<[u32; 2]>,
    pub visited: Vec<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<[u32; 2]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualDelta {
    pub step: u32,
    pub frontier_added: Vec<[u32; 2]>,
    pub frontier_removed: Vec<[u32; 2]>,
    pub visited_added: Vec<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<[u32; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ServerMessage {
    Connected {
        protocol_version: u16,
        run_id: RunId,
        sequence: u64,
        latest_sequence: u64,
    },
    Snapshot {
        protocol_version: u16,
        run_id: RunId,
        sequence: u64,
        state: VisualState,
    },
    Delta {
        protocol_version: u16,
        run_id: RunId,
        sequence: u64,
        delta: VisualDelta,
    },
    Completed {
        protocol_version: u16,
        run_id: RunId,
        sequence: u64,
        path: Vec<[u32; 2]>,
        stats: crate::replay::ReplayStats,
    },
    Failed {
        protocol_version: u16,
        run_id: RunId,
        sequence: u64,
        code: String,
        message: String,
    },
    Cancelled {
        protocol_version: u16,
        run_id: RunId,
        sequence: u64,
    },
    Heartbeat {
        protocol_version: u16,
        run_id: RunId,
        sequence: u64,
    },
}

impl ServerMessage {
    pub fn sequence(&self) -> u64 {
        match self {
            Self::Connected { sequence, .. }
            | Self::Snapshot { sequence, .. }
            | Self::Delta { sequence, .. }
            | Self::Completed { sequence, .. }
            | Self::Failed { sequence, .. }
            | Self::Cancelled { sequence, .. }
            | Self::Heartbeat { sequence, .. } => *sequence,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ReplayEvent {
    Snapshot { sequence: u64, state: VisualState },
    Delta { sequence: u64, delta: VisualDelta },
}

impl ReplayEvent {
    pub fn sequence(&self) -> u64 {
        match self {
            Self::Snapshot { sequence, .. } | Self::Delta { sequence, .. } => *sequence,
        }
    }
}

struct Inner {
    sequence: u64,
    state_sequence: u64,
    state: VisualState,
    history: VecDeque<ServerMessage>,
    terminal: Option<ServerMessage>,
}

pub struct RunStream {
    run_id: RunId,
    sender: broadcast::Sender<ServerMessage>,
    inner: Mutex<Inner>,
    cancelled: AtomicBool,
    history_capacity: usize,
}

pub struct ResumeBatch {
    pub latest_sequence: u64,
    pub messages: Vec<ServerMessage>,
}

impl RunStream {
    pub fn new(run_id: RunId, history_capacity: usize, client_capacity: usize) -> Arc<Self> {
        let (sender, _) = broadcast::channel(client_capacity.max(1));
        Arc::new(Self {
            run_id,
            sender,
            inner: Mutex::new(Inner {
                sequence: 0,
                state_sequence: 0,
                state: VisualState::default(),
                history: VecDeque::new(),
                terminal: None,
            }),
            cancelled: AtomicBool::new(false),
            history_capacity: history_capacity.max(1),
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerMessage> {
        self.sender.subscribe()
    }
    pub fn cancellation(&self) -> &AtomicBool {
        &self.cancelled
    }
    pub fn request_cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
    pub fn latest_sequence(&self) -> u64 {
        self.inner
            .lock()
            .expect("run stream mutex poisoned")
            .sequence
    }
    pub fn is_terminal(&self) -> bool {
        self.inner
            .lock()
            .expect("run stream mutex poisoned")
            .terminal
            .is_some()
    }

    pub fn publish_progress(&self, progress: SolveProgress, snapshot: bool) -> Option<ReplayEvent> {
        let mut inner = self.inner.lock().expect("run stream mutex poisoned");
        if inner.terminal.is_some() {
            return None;
        }
        inner.sequence += 1;
        let sequence = inner.sequence;
        let mut next = VisualState {
            step: progress.step,
            frontier: progress.frontier,
            visited: progress.visited,
            current: progress.current,
        };
        next.frontier.sort_unstable();
        next.visited.sort_unstable();
        let replay_event = if snapshot || sequence == 1 {
            ReplayEvent::Snapshot {
                sequence,
                state: next.clone(),
            }
        } else {
            ReplayEvent::Delta {
                sequence,
                delta: delta_between(&inner.state, &next),
            }
        };
        inner.state = next;
        inner.state_sequence = sequence;
        let message = match &replay_event {
            ReplayEvent::Snapshot { state, .. } => ServerMessage::Snapshot {
                protocol_version: PROTOCOL_VERSION,
                run_id: self.run_id,
                sequence,
                state: state.clone(),
            },
            ReplayEvent::Delta { delta, .. } => ServerMessage::Delta {
                protocol_version: PROTOCOL_VERSION,
                run_id: self.run_id,
                sequence,
                delta: delta.clone(),
            },
        };
        self.retain_and_send(&mut inner, message);
        Some(replay_event)
    }

    pub fn complete(&self, path: Vec<[u32; 2]>, stats: crate::replay::ReplayStats) {
        self.publish_terminal(|sequence| ServerMessage::Completed {
            protocol_version: PROTOCOL_VERSION,
            run_id: self.run_id,
            sequence,
            path,
            stats,
        });
    }
    pub fn fail(&self, code: impl Into<String>, message: impl Into<String>) {
        let code = code.into();
        let message = message.into();
        self.publish_terminal(|sequence| ServerMessage::Failed {
            protocol_version: PROTOCOL_VERSION,
            run_id: self.run_id,
            sequence,
            code,
            message,
        });
    }
    pub fn cancelled(&self) {
        self.publish_terminal(|sequence| ServerMessage::Cancelled {
            protocol_version: PROTOCOL_VERSION,
            run_id: self.run_id,
            sequence,
        });
    }

    fn publish_terminal(&self, build: impl FnOnce(u64) -> ServerMessage) {
        let mut inner = self.inner.lock().expect("run stream mutex poisoned");
        if inner.terminal.is_some() {
            return;
        }
        inner.sequence += 1;
        let message = build(inner.sequence);
        inner.terminal = Some(message.clone());
        self.retain_and_send(&mut inner, message);
    }

    fn retain_and_send(&self, inner: &mut Inner, message: ServerMessage) {
        inner.history.push_back(message.clone());
        while inner.history.len() > self.history_capacity {
            inner.history.pop_front();
        }
        let _ = self.sender.send(message);
    }

    pub fn resume(&self, after_sequence: u64) -> ResumeBatch {
        let inner = self.inner.lock().expect("run stream mutex poisoned");
        let oldest = inner
            .history
            .front()
            .map(ServerMessage::sequence)
            .unwrap_or(inner.sequence + 1);
        let covered = after_sequence.saturating_add(1) >= oldest;
        let messages = if covered {
            inner
                .history
                .iter()
                .filter(|message| message.sequence() > after_sequence)
                .cloned()
                .collect()
        } else {
            let mut messages = vec![ServerMessage::Snapshot {
                protocol_version: PROTOCOL_VERSION,
                run_id: self.run_id,
                sequence: inner.state_sequence,
                state: inner.state.clone(),
            }];
            if let Some(terminal) = &inner.terminal {
                messages.push(terminal.clone());
            }
            messages
        };
        ResumeBatch {
            latest_sequence: inner.sequence,
            messages,
        }
    }

    pub fn connected(&self, after_sequence: u64) -> ServerMessage {
        let latest_sequence = self.latest_sequence();
        ServerMessage::Connected {
            protocol_version: PROTOCOL_VERSION,
            run_id: self.run_id,
            sequence: after_sequence.min(latest_sequence),
            latest_sequence,
        }
    }
    pub fn heartbeat(&self) -> ServerMessage {
        ServerMessage::Heartbeat {
            protocol_version: PROTOCOL_VERSION,
            run_id: self.run_id,
            sequence: self.latest_sequence(),
        }
    }
}

fn delta_between(previous: &VisualState, next: &VisualState) -> VisualDelta {
    let old_frontier: HashSet<_> = previous.frontier.iter().copied().collect();
    let new_frontier: HashSet<_> = next.frontier.iter().copied().collect();
    let old_visited: HashSet<_> = previous.visited.iter().copied().collect();
    let mut frontier_added: Vec<_> = new_frontier.difference(&old_frontier).copied().collect();
    let mut frontier_removed: Vec<_> = old_frontier.difference(&new_frontier).copied().collect();
    let mut visited_added: Vec<_> = next
        .visited
        .iter()
        .copied()
        .filter(|cell| !old_visited.contains(cell))
        .collect();
    frontier_added.sort_unstable();
    frontier_removed.sort_unstable();
    visited_added.sort_unstable();
    VisualDelta {
        step: next.step,
        frontier_added,
        frontier_removed,
        visited_added,
        current: next.current,
    }
}

pub fn apply_replay_event(state: &mut VisualState, event: &ReplayEvent) {
    match event {
        ReplayEvent::Snapshot {
            state: snapshot, ..
        } => *state = snapshot.clone(),
        ReplayEvent::Delta { delta, .. } => {
            let removed: HashSet<_> = delta.frontier_removed.iter().copied().collect();
            state.frontier.retain(|cell| !removed.contains(cell));
            for cell in &delta.frontier_added {
                if !state.frontier.contains(cell) {
                    state.frontier.push(*cell);
                }
            }
            for cell in &delta.visited_added {
                if !state.visited.contains(cell) {
                    state.visited.push(*cell);
                }
            }
            state.frontier.sort_unstable();
            state.visited.sort_unstable();
            state.step = delta.step;
            state.current = delta.current;
        }
    }
}

pub struct ProgressRecorder {
    stream: Arc<RunStream>,
    sample_every: u32,
    snapshot_every: u32,
    max_events: usize,
    events: Vec<ReplayEvent>,
    last_progress: Option<SolveProgress>,
}

impl ProgressRecorder {
    pub fn new(
        stream: Arc<RunStream>,
        sample_every: u32,
        snapshot_every: u32,
        max_events: usize,
    ) -> Self {
        Self {
            stream,
            sample_every: sample_every.max(1),
            snapshot_every: snapshot_every.max(1),
            max_events: max_events.max(2),
            events: Vec::new(),
            last_progress: None,
        }
    }
    pub fn finish(mut self) -> Vec<ReplayEvent> {
        if let Some(progress) = self.last_progress.take() {
            let already_recorded = self.events.last().is_some_and(|event| match event {
                ReplayEvent::Snapshot { state, .. } => state.step == progress.step,
                ReplayEvent::Delta { delta, .. } => delta.step == progress.step,
            });
            if already_recorded {
                return self.events;
            }
            if let Some(event) = self.stream.publish_progress(progress, true) {
                if self.events.len() == self.max_events {
                    self.events.pop();
                }
                self.events.push(event);
            }
        }
        self.events
    }
}

impl ProgressSink for ProgressRecorder {
    fn progress(&mut self, progress: SolveProgress) {
        self.last_progress = Some(progress.clone());
        if !progress.step.is_multiple_of(self.sample_every) && progress.step != 1 {
            return;
        }
        let snapshot = progress.step.is_multiple_of(self.snapshot_every);
        if let Some(event) = self.stream.publish_progress(progress, snapshot) {
            if self.events.len() < self.max_events {
                self.events.push(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::{generate, GeneratorAlgo};
    use crate::replay::build_replay;
    use crate::solve::{AstarSolver, BfsSolver, DfsSolver, DpKeysSolver, Solver};
    use uuid::Uuid;
    fn progress(step: u32, frontier: &[[u32; 2]], visited: &[[u32; 2]]) -> SolveProgress {
        SolveProgress {
            step,
            frontier: frontier.to_vec(),
            visited: visited.to_vec(),
            current: visited.last().copied(),
        }
    }
    #[test]
    fn deltas_reconstruct_the_latest_snapshot() {
        let stream = RunStream::new(Uuid::new_v4(), 8, 2);
        let events = [
            stream
                .publish_progress(progress(1, &[[1, 0]], &[[0, 0]]), true)
                .unwrap(),
            stream
                .publish_progress(progress(2, &[[2, 0]], &[[0, 0], [1, 0]]), false)
                .unwrap(),
            stream
                .publish_progress(progress(3, &[], &[[0, 0], [1, 0], [2, 0]]), false)
                .unwrap(),
        ];
        let mut reconstructed = VisualState::default();
        for event in &events {
            apply_replay_event(&mut reconstructed, event);
        }
        assert_eq!(reconstructed, stream.inner.lock().unwrap().state);
    }
    #[test]
    fn resume_uses_snapshot_when_history_gap_is_too_old() {
        let stream = RunStream::new(Uuid::new_v4(), 2, 1);
        for step in 1..=4 {
            let _ = stream.publish_progress(progress(step, &[], &[[step, 0]]), step == 1);
        }
        let resumed = stream.resume(0);
        assert!(matches!(
            resumed.messages.first(),
            Some(ServerMessage::Snapshot { sequence: 4, .. })
        ));
        assert!(stream
            .resume(3)
            .messages
            .iter()
            .all(|message| message.sequence() > 3));
    }
    #[test]
    fn terminal_state_is_idempotent_and_retained() {
        let stream = RunStream::new(Uuid::new_v4(), 2, 1);
        stream.cancelled();
        stream.fail("ignored", "ignored");
        assert!(stream.is_terminal());
        assert!(stream
            .publish_progress(progress(2, &[], &[[0, 0]]), false)
            .is_none());
        assert_eq!(stream.latest_sequence(), 1);
        assert!(matches!(
            stream.resume(0).messages.last(),
            Some(ServerMessage::Cancelled { .. })
        ));
    }

    #[test]
    fn protocol_messages_serialize_with_camel_case_fields() {
        let run_id = Uuid::new_v4();
        let value = serde_json::to_value(ServerMessage::Connected {
            protocol_version: PROTOCOL_VERSION,
            run_id,
            sequence: 3,
            latest_sequence: 7,
        })
        .unwrap();
        assert_eq!(value["type"], "connected");
        assert_eq!(value["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(value["runId"], run_id.to_string());
        assert_eq!(value["latestSequence"], 7);
        assert!(value.get("protocol_version").is_none());
    }

    #[test]
    fn gap_resume_delivers_visual_state_before_terminal() {
        let stream = RunStream::new(Uuid::new_v4(), 2, 1);
        for step in 1..=4 {
            let _ = stream.publish_progress(progress(step, &[], &[[step, 0]]), step == 1);
        }
        stream.complete(
            vec![[0, 0], [1, 0]],
            crate::replay::ReplayStats {
                visited: 4,
                cost: 1,
                ms: 1,
            },
        );
        let resumed = stream.resume(0);
        assert_eq!(resumed.messages.len(), 2);
        assert!(matches!(
            &resumed.messages[0],
            ServerMessage::Snapshot { sequence: 4, .. }
        ));
        assert!(matches!(
            &resumed.messages[1],
            ServerMessage::Completed { sequence: 5, .. }
        ));
    }

    #[tokio::test]
    async fn subscriber_receives_progress_published_after_subscribe() {
        let stream = RunStream::new(Uuid::new_v4(), 8, 2);
        let mut receiver = stream.subscribe();
        let _ = stream.publish_progress(progress(1, &[[1, 0]], &[[0, 0]]), true);
        let message = receiver.recv().await.unwrap();
        assert!(matches!(
            message,
            ServerMessage::Snapshot { sequence: 1, .. }
        ));
    }

    #[tokio::test]
    async fn lagged_subscriber_recovers_from_bounded_history() {
        let stream = RunStream::new(Uuid::new_v4(), 3, 1);
        let mut receiver = stream.subscribe();
        for step in 1..=20 {
            let _ = stream.publish_progress(progress(step, &[], &[[step, 0]]), step == 1);
        }
        assert!(matches!(
            receiver.recv().await,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_))
        ));
        assert_eq!(stream.inner.lock().unwrap().history.len(), 3);
        let resumed = stream.resume(0);
        assert!(matches!(
            resumed.messages.first(),
            Some(ServerMessage::Snapshot { sequence: 20, .. })
        ));
    }

    #[test]
    fn recorder_caps_replay_events_and_keeps_final_state() {
        let stream = RunStream::new(Uuid::new_v4(), 4, 1);
        let mut recorder = ProgressRecorder::new(Arc::clone(&stream), 1, 4, 5);
        for step in 1..=50 {
            recorder.progress(progress(step, &[], &[[step, 0]]));
        }
        let events = recorder.finish();
        assert!(events.len() <= 5);
        let mut state = VisualState::default();
        for event in &events {
            apply_replay_event(&mut state, event);
        }
        assert_eq!(state.step, 50);
    }

    #[test]
    fn concurrent_target_sized_solves_stay_within_stream_and_replay_budgets() {
        let maze = Arc::new(generate(50, 50, 303, GeneratorAlgo::Kruskal));
        let solvers: Vec<Arc<dyn Solver>> = vec![
            Arc::new(BfsSolver),
            Arc::new(DfsSolver),
            Arc::new(AstarSolver),
            Arc::new(DpKeysSolver),
        ];

        std::thread::scope(|scope| {
            let handles: Vec<_> = solvers
                .into_iter()
                .map(|solver| {
                    let maze = Arc::clone(&maze);
                    scope.spawn(move || {
                        let stream = RunStream::new(Uuid::new_v4(), 256, 32);
                        let mut recorder = ProgressRecorder::new(Arc::clone(&stream), 2, 32, 2_048);
                        let result = solver
                            .solve_with_progress(&maze, &mut recorder, &AtomicBool::new(false))
                            .unwrap();
                        let events = recorder.finish();
                        let mut reconstructed = VisualState::default();
                        for event in &events {
                            apply_replay_event(&mut reconstructed, event);
                        }
                        let replay =
                            build_replay("target-maze", solver.name(), 303, result, events);
                        let payload = serde_json::to_vec(&replay).unwrap();

                        assert!(
                            replay.events.len() <= 2_048,
                            "{} event budget",
                            solver.name()
                        );
                        assert!(
                            payload.len() <= 8 * 1_024 * 1_024,
                            "{} payload budget",
                            solver.name()
                        );
                        let inner = stream.inner.lock().unwrap();
                        assert!(
                            inner.history.len() <= 256,
                            "{} history budget",
                            solver.name()
                        );
                        assert_eq!(reconstructed, inner.state, "{} replay state", solver.name());
                    })
                })
                .collect();
            for handle in handles {
                handle.join().unwrap();
            }
        });
    }
}
