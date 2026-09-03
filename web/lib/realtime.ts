export const PROTOCOL_VERSION = 1;

export type StreamStatus =
  | 'idle'
  | 'waking'
  | 'connecting'
  | 'live'
  | 'reconnecting'
  | 'completed'
  | 'failed'
  | 'cancelled';

export interface VisualState {
  step: number;
  frontier: [number, number][];
  visited: [number, number][];
  current?: [number, number];
}

export interface SolveStats { visited: number; cost: number; ms: number }

export interface StreamState {
  runId: string | null;
  status: StreamStatus;
  sequence: number;
  visual: VisualState | null;
  path: [number, number][];
  stats: SolveStats | null;
  error: string | null;
}

export const idleStreamState: StreamState = {
  runId: null, status: 'idle', sequence: 0, visual: null, path: [], stats: null, error: null,
};

export function connectingState(runId: string): StreamState {
  return { ...idleStreamState, runId, status: 'connecting' };
}

function cell(value: unknown): [number, number] {
  return Array.isArray(value) && value.length >= 2 ? [Number(value[0]), Number(value[1])] : [0, 0];
}
function cells(value: unknown): [number, number][] { return Array.isArray(value) ? value.map(cell) : []; }
function visual(value: unknown): VisualState {
  const state = (value ?? {}) as Record<string, unknown>;
  return { step: Number(state.step ?? 0), frontier: cells(state.frontier), visited: cells(state.visited), current: state.current == null ? undefined : cell(state.current) };
}
function stats(value: unknown): SolveStats | null {
  if (!value || typeof value !== 'object') return null;
  const item = value as Record<string, unknown>;
  return { visited: Number(item.visited ?? 0), cost: Number(item.cost ?? 0), ms: Number(item.ms ?? 0) };
}
function key(value: [number, number]) { return `${value[0]}:${value[1]}`; }

export function reduceServerMessage(state: StreamState, raw: unknown): StreamState {
  if (!raw || typeof raw !== 'object') return { ...state, status: 'failed', error: 'Invalid realtime message.' };
  const message = raw as Record<string, unknown>;
  if (message.protocolVersion !== PROTOCOL_VERSION || message.runId !== state.runId) return state;
  const type = String(message.type ?? '');
  const sequence = Number(message.sequence ?? 0);
  if (type !== 'connected' && type !== 'heartbeat' && sequence <= state.sequence) return state;

  if (type === 'connected') return { ...state, status: state.status === 'completed' ? state.status : 'live', error: null };
  if (type === 'heartbeat') return state;
  if (type === 'snapshot') return { ...state, status: 'live', sequence, visual: visual(message.state), error: null };
  if (type === 'delta') {
    const delta = (message.delta ?? {}) as Record<string, unknown>;
    const previous = state.visual ?? { step: 0, frontier: [], visited: [] };
    const removed = new Set(cells(delta.frontierRemoved).map(key));
    const frontier = previous.frontier.filter((item) => !removed.has(key(item)));
    const frontierKeys = new Set(frontier.map(key));
    for (const item of cells(delta.frontierAdded)) if (!frontierKeys.has(key(item))) { frontier.push(item); frontierKeys.add(key(item)); }
    const visited = [...previous.visited];
    const visitedKeys = new Set(visited.map(key));
    for (const item of cells(delta.visitedAdded)) if (!visitedKeys.has(key(item))) { visited.push(item); visitedKeys.add(key(item)); }
    return { ...state, status: 'live', sequence, visual: { step: Number(delta.step ?? previous.step), frontier, visited, current: delta.current == null ? undefined : cell(delta.current) }, error: null };
  }
  if (type === 'completed') return { ...state, status: 'completed', sequence, path: cells(message.path), stats: stats(message.stats), error: null };
  if (type === 'failed') return { ...state, status: 'failed', sequence, error: typeof message.message === 'string' ? message.message : 'The solve failed.' };
  if (type === 'cancelled') return { ...state, status: 'cancelled', sequence, error: null };
  return state;
}

export function replayStates(events: unknown[]): VisualState[] {
  let state = connectingState('replay');
  const states: VisualState[] = [];
  for (const raw of events) {
    if (!raw || typeof raw !== 'object') continue;
    const event = raw as Record<string, unknown>;
    const message = { ...event, protocolVersion: PROTOCOL_VERSION, runId: 'replay' };
    state = reduceServerMessage(state, message);
    if (state.visual) states.push(state.visual);
  }
  return states;
}
