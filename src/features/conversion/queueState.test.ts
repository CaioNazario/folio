import { expect, test } from "vitest";
import { applyEvent, EMPTY_QUEUE, withName } from "./queueState";

const named = withName(EMPTY_QUEUE, "doc-1", "Aula.txt");
const queued = applyEvent(named, {
  kind: "queued",
  job: 1,
  document: "doc-1",
  target: "md",
});

test("a queued job takes the name asked for and waits", () => {
  expect(queued.jobs[1]).toEqual({ name: "Aula.txt", target: "md" });
  expect(queued.waiting).toEqual([1]);
});

test("a started job runs and leaves the waiting list", () => {
  const started = applyEvent(queued, { kind: "started", job: 1 });

  expect(started.running).toEqual({ job: 1, fraction: null });
  expect(started.waiting).toEqual([]);
});

test("progress updates the running fraction", () => {
  const started = applyEvent(queued, { kind: "started", job: 1 });
  const progress = applyEvent(started, {
    kind: "progress",
    job: 1,
    progress: { stage: "reading", fraction: 0.4 },
  });

  expect(progress.running).toEqual({ job: 1, fraction: 0.4 });
});

test("a cancelled waiting job leaves the queue", () => {
  const cancelled = applyEvent(queued, { kind: "cancelled", job: 1 });

  expect(cancelled.waiting).toEqual([]);
  expect(cancelled.running).toBeNull();
});
