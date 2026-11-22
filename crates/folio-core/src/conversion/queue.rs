//! The conversion queue: one conversion at a time, in a background thread,
//! in the order asked (spec 08, "Execução").

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;

use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use super::{Control, Progress};
use crate::bridge_error::BridgeError;
use crate::documents::{Document, Format};
use crate::workspace::Workspace;
use crate::{Error, Result};

/// A conversion asked for, waiting or running.
pub struct Job {
    pub id: u32,
    pub workspace: Arc<Workspace>,
    pub document: Uuid,
    pub target: Format,
    pub password: Option<String>,
}

/// What the queue tells the UI about each job.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ConversionEvent {
    Queued { job: u32 },
    Started { job: u32 },
    Progress { job: u32, progress: Progress },
    Finished { job: u32, document: Document },
    Failed { job: u32, error: BridgeError },
    Cancelled { job: u32 },
}

/// Runs one job, reporting to its control.
pub type Runner = dyn Fn(&Job, &Control) -> Result<Document> + Send + Sync;
/// Receives the queue's events.
pub type Emitter = dyn Fn(ConversionEvent) + Send + Sync;

/// The queue. Dropping it stops the worker after the running job.
pub struct ConversionQueue {
    shared: Arc<Shared>,
}

/// What the queue and its worker thread share.
struct Shared {
    state: Mutex<State>,
    wake: Condvar,
    run: Box<Runner>,
    emit: Box<Emitter>,
}

#[derive(Default)]
struct State {
    last_id: u32,
    waiting: VecDeque<Job>,
    running: Option<(u32, Arc<Control>)>,
    closed: bool,
}

impl ConversionQueue {
    /// A queue that converts with [`Workspace::convert_with`].
    pub fn new(emit: impl Fn(ConversionEvent) + Send + Sync + 'static) -> Self {
        Self::with_runner(Box::new(run_conversion), Box::new(emit))
    }

    /// A queue that runs its jobs with `run`.
    pub fn with_runner(run: Box<Runner>, emit: Box<Emitter>) -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            wake: Condvar::new(),
            run,
            emit,
        });
        let worker = shared.clone();
        thread::spawn(move || Shared::work(&worker));
        Self { shared }
    }

    /// Puts a conversion of `document` to `target` at the end of the queue
    /// and returns its job id.
    pub fn enqueue(
        &self,
        workspace: Arc<Workspace>,
        document: Uuid,
        target: Format,
        password: Option<String>,
    ) -> u32 {
        let mut state = self.shared.lock();
        state.last_id += 1;
        let id = state.last_id;
        (self.shared.emit)(ConversionEvent::Queued { job: id });
        let job = Job {
            id,
            workspace,
            document,
            target,
            password,
        };
        state.waiting.push_back(job);
        self.shared.wake.notify_one();
        id
    }

    /// Cancels the job `id`: a waiting job leaves the queue, a running one
    /// stops at its next check.
    pub fn cancel(&self, id: u32) {
        let mut state = self.shared.lock();
        if let Some((_, control)) = state.running.as_ref().filter(|(job, _)| *job == id) {
            control.cancel();
            return;
        }
        let before = state.waiting.len();
        state.waiting.retain(|job| job.id != id);
        if state.waiting.len() < before {
            (self.shared.emit)(ConversionEvent::Cancelled { job: id });
        }
    }
}

impl Drop for ConversionQueue {
    fn drop(&mut self) {
        self.shared.lock().closed = true;
        self.shared.wake.notify_one();
    }
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The worker: runs the jobs one by one until the queue is dropped.
    fn work(this: &Arc<Self>) {
        while let Some((job, control)) = Self::next_job(this) {
            (this.emit)(ConversionEvent::Started { job: job.id });
            let result = (this.run)(&job, &control);
            this.lock().running = None;
            (this.emit)(finished(job.id, result));
        }
    }

    /// Waits for the next job and marks it running; `None` once closed.
    fn next_job(this: &Arc<Self>) -> Option<(Job, Arc<Control>)> {
        let mut state = this.lock();
        loop {
            if state.closed {
                return None;
            }
            if let Some(job) = state.waiting.pop_front() {
                let control = Arc::new(Self::control_for(this, job.id));
                state.running = Some((job.id, control.clone()));
                return Some((job, control));
            }
            state = this
                .wake
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// A control that forwards the progress of job `id` as events.
    fn control_for(this: &Arc<Self>, id: u32) -> Control {
        let shared = Arc::downgrade(this);
        Control::new(move |progress| {
            if let Some(shared) = shared.upgrade() {
                (shared.emit)(ConversionEvent::Progress { job: id, progress });
            }
        })
    }
}

/// The event that closes a job.
fn finished(job: u32, result: Result<Document>) -> ConversionEvent {
    match result {
        Ok(document) => ConversionEvent::Finished { job, document },
        Err(Error::ConversionCancelled(_)) => ConversionEvent::Cancelled { job },
        Err(error) => ConversionEvent::Failed {
            job,
            error: BridgeError::from(&error),
        },
    }
}

fn run_conversion(job: &Job, control: &Control) -> Result<Document> {
    let password = job.password.as_deref();
    job.workspace
        .convert_with(job.document, job.target, password, control)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::mpsc::{Receiver, channel};
    use std::time::Duration;

    use super::*;
    use crate::documents::test_docs::write_doc;
    use crate::workspace::test_env::TestEnv;

    fn indexed(workspace: &Workspace, path: &str, content: &[u8]) -> Uuid {
        let path = write_doc(workspace, path, content);
        workspace.index_file(&path).unwrap().unwrap().id
    }

    /// The next event, failing the test after a few seconds.
    fn next(events: &Receiver<ConversionEvent>) -> ConversionEvent {
        events.recv_timeout(Duration::from_secs(10)).unwrap()
    }

    #[test]
    fn job_is_queued_started_and_finished_with_its_derivative() {
        let env = TestEnv::new();
        let workspace = Arc::new(env.open_workspace());
        let txt = indexed(&workspace, "Aula.txt", b"texto\n");
        let (sender, events) = channel();
        let queue = ConversionQueue::new(move |event| sender.send(event).unwrap());

        let job = queue.enqueue(workspace.clone(), txt, Format::Md, None);

        assert_eq!(next(&events), ConversionEvent::Queued { job });
        assert_eq!(next(&events), ConversionEvent::Started { job });
        let finished = loop {
            match next(&events) {
                ConversionEvent::Progress { .. } => continue,
                other => break other,
            }
        };
        let ConversionEvent::Finished { document, .. } = finished else {
            panic!("expected Finished, got {finished:?}");
        };
        assert_eq!(document.path.display(), "Aula.md");
    }

    /// A runner that ends a job as failed when `gate` receives a `()`, or as
    /// cancelled once its control is cancelled.
    fn gated_runner(
        gate: Mutex<Receiver<()>>,
    ) -> impl Fn(&Job, &Control) -> Result<Document> + Send + Sync {
        move |job, control| loop {
            if control.is_cancelled() {
                return Err(Error::ConversionCancelled(job.document));
            }
            let opened = gate.lock().unwrap().recv_timeout(Duration::from_millis(5));
            if opened.is_ok() {
                return Err(Error::WriteFailed(job.document));
            }
        }
    }

    /// A queue whose jobs wait for a `()` on the returned sender before
    /// finishing, or stop as cancelled once their control is cancelled.
    fn gated_queue() -> (
        ConversionQueue,
        std::sync::mpsc::Sender<()>,
        Receiver<ConversionEvent>,
    ) {
        let (open, gate) = channel::<()>();
        let run = gated_runner(Mutex::new(gate));
        let (sender, events) = channel();
        let emit = move |event| sender.send(event).unwrap();
        (
            ConversionQueue::with_runner(Box::new(run), Box::new(emit)),
            open,
            events,
        )
    }

    struct Env {
        _env: TestEnv,
        workspace: Arc<Workspace>,
    }

    fn env() -> Env {
        let env = TestEnv::new();
        let workspace = Arc::new(env.open_workspace());
        Env {
            _env: env,
            workspace,
        }
    }

    fn enqueue(queue: &ConversionQueue, env: &Env) -> u32 {
        queue.enqueue(env.workspace.clone(), Uuid::from_u128(1), Format::Md, None)
    }

    #[test]
    fn second_job_starts_only_after_first_ends() {
        let (queue, open, events) = gated_queue();
        let env = env();
        let (first, second) = (enqueue(&queue, &env), enqueue(&queue, &env));
        open.send(()).unwrap();
        open.send(()).unwrap();

        let kinds: Vec<_> = (0..6).map(|_| next(&events)).collect();

        let failed = |job| ConversionEvent::Failed {
            job,
            error: BridgeError::WriteFailed {
                id: Uuid::from_u128(1).to_string(),
            },
        };
        let started_second = kinds
            .iter()
            .position(|e| *e == ConversionEvent::Started { job: second });
        let failed_first = kinds.iter().position(|e| *e == failed(first));
        assert!(failed_first.unwrap() < started_second.unwrap(), "{kinds:?}");
        assert_eq!(kinds.last(), Some(&failed(second)));
    }

    #[test]
    fn cancelling_waiting_job_removes_it_without_starting() {
        let (queue, open, events) = gated_queue();
        let env = env();
        let (first, second) = (enqueue(&queue, &env), enqueue(&queue, &env));

        queue.cancel(second);
        open.send(()).unwrap();

        let kinds: Vec<_> = (0..5).map(|_| next(&events)).collect();
        assert!(kinds.contains(&ConversionEvent::Cancelled { job: second }));
        assert!(!kinds.contains(&ConversionEvent::Started { job: second }));
        assert!(matches!(kinds.last(), Some(ConversionEvent::Failed { job, .. }) if *job == first));
    }

    #[test]
    fn cancelling_running_job_stops_it_as_cancelled() {
        let (queue, _open, events) = gated_queue();
        let env = env();
        let job = enqueue(&queue, &env);
        assert_eq!(next(&events), ConversionEvent::Queued { job });
        assert_eq!(next(&events), ConversionEvent::Started { job });

        queue.cancel(job);

        assert_eq!(next(&events), ConversionEvent::Cancelled { job });
    }

    #[test]
    fn job_progress_reaches_events() {
        let env = env();
        let pdf = indexed(
            &env.workspace,
            "Ata.pdf",
            &crate::readers::test_pdf::pdf(&[vec![crate::readers::test_pdf::text(
                "Ata", 72.0, 700.0,
            )]]),
        );
        let (sender, events) = channel();
        let queue = ConversionQueue::new(move |event| sender.send(event).unwrap());

        let job = queue.enqueue(env.workspace.clone(), pdf, Format::Txt, None);

        let progress = (0..4).map(|_| next(&events)).find(|e| {
            matches!(e, ConversionEvent::Progress { progress, .. } if progress.fraction == Some(1.0))
        });
        assert!(matches!(progress, Some(ConversionEvent::Progress { job: j, .. }) if j == job));
    }
}
