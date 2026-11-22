//! Progress and cancellation of one conversion (spec 08, "Execução").

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use ts_rs::TS;

/// What a conversion is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Stage {
    Reading,
    Writing,
}

/// A step of a running conversion; `fraction` (0 to 1) when it can be
/// measured.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Progress {
    pub stage: Stage,
    pub fraction: Option<f64>,
}

/// The channel between a running conversion and whoever asked for it: the
/// conversion reports progress and checks for a cancel request.
pub struct Control {
    cancelled: AtomicBool,
    report: Box<dyn Fn(Progress) + Send + Sync>,
}

impl Control {
    /// A control that hands every progress to `report`.
    pub fn new(report: impl Fn(Progress) + Send + Sync + 'static) -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            report: Box::new(report),
        }
    }

    /// A control nobody listens to.
    pub fn silent() -> Self {
        Self::new(|_| {})
    }

    /// Asks the conversion to stop at its next check.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub(crate) fn report(&self, stage: Stage, fraction: Option<f64>) {
        (self.report)(Progress { stage, fraction });
    }
}
