use std::sync::atomic::{AtomicBool, Ordering};

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use instant::Instant;

use crate::scc::ProgressTracker;

/// A context object which aggregates all necessary information about a running task working with
/// a symbolic graph.
///
/// We use this to avoid passing each context variable as a (mutable) reference. It is also easier
/// to implement some utility methods this way.
pub struct GraphTaskContext {
    pub started: Instant,
    pub is_cancelled: AtomicBool,
    pub progress: ProgressTracker,
}

impl Default for GraphTaskContext {
    fn default() -> Self {
        GraphTaskContext::new()
    }
}

impl GraphTaskContext {
    /// Create a new task context.
    pub fn new() -> GraphTaskContext {
        GraphTaskContext {
            started: Instant::now(),
            is_cancelled: AtomicBool::new(false),
            progress: ProgressTracker::new(),
        }
    }

    /// Re-initialize the task context with a fresh graph.
    pub fn restart(&self, graph: &SymbolicAsyncGraph) {
        self.progress.init_from_graph(graph);
        self.is_cancelled.store(false, Ordering::SeqCst);
    }

    /// True if the task is cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::SeqCst)
    }

    /*
    /// Set the status of this task to cancel.
    ///
    /// Return true if the computation was set to cancelled by this call, false if it was
    /// cancelled previously.
    pub fn cancel(&self) -> bool {
        !self.is_cancelled.swap(true, Ordering::SeqCst)
    }
     */

    /// Indicate that the given set still needs to be processed by the task.
    pub fn update_remaining(&self, remaining: &GraphColoredVertices) {
        self.progress.update_remaining(remaining);
    }

    /*
    /// Output a string which represent the percentage of remaining state space.
    pub fn get_percent_string(&self) -> String {
        self.progress.get_percent_string()
    }
     */
}
