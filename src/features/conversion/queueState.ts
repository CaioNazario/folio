import type { ConversionEvent } from "@/lib/bindings/ConversionEvent";
import type { Format } from "@/lib/bindings/Format";

// What the UI knows about the conversion queue, kept from its events.

export type Job = { name: string; target: Format };

export type QueueState = {
  /** Names of the documents asked for, by id: events only carry the id. */
  names: Record<string, string>;
  jobs: Record<number, Job>;
  running: { job: number; fraction: number | null } | null;
  /** Jobs queued and not started yet. */
  waiting: number[];
};

export const EMPTY_QUEUE: QueueState = {
  names: {},
  jobs: {},
  running: null,
  waiting: [],
};

/** Remembers the name of `id` before asking for its conversion. */
export function withName(
  state: QueueState,
  id: string,
  name: string,
): QueueState {
  return { ...state, names: { ...state.names, [id]: name } };
}

function withoutJob(state: QueueState, job: number): QueueState {
  const running = state.running?.job === job ? null : state.running
  return { ...state, running, waiting: state.waiting.filter((id) => id !== job) }
}

function queued(state: QueueState, event: Extract<ConversionEvent, { kind: 'queued' }>): QueueState {
  const job = { name: state.names[event.document] ?? '', target: event.target }
  return { ...state, jobs: { ...state.jobs, [event.job]: job }, waiting: [...state.waiting, event.job] }
}

/** The queue after `event`. */
export function applyEvent(state: QueueState, event: ConversionEvent): QueueState {
  if (event.kind === 'queued') return queued(state, event)
  if (event.kind === 'started') return { ...withoutJob(state, event.job), running: { job: event.job, fraction: null } }
  if (event.kind === 'progress') return { ...state, running: { job: event.job, fraction: event.progress.fraction } }
  return withoutJob(state, event.job)
}
