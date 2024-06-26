use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphColors};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::Mutex;

mod _impl_behaviour;
/// **(internal)** Utility methods for the behaviour `Class`.
mod _impl_class;
/// **(internal)** Implementation of `Behaviour` classification in `Classifier`.
mod _impl_classifier;
mod _impl_progress_tracker;
pub mod algo_interleaved_transition_guided_reduction;
pub mod algo_saturated_reachability;
pub mod algo_stability_analysis;
pub mod algo_xie_beerel;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Behaviour {
    Stability,
    Oscillation,
    Disorder,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Class(Vec<Behaviour>);

pub struct Classifier {
    classes: Mutex<HashMap<Class, GraphColors>>,
    attractors: Mutex<Vec<(GraphColoredVertices, HashMap<Behaviour, GraphColors>)>>,
}

#[derive(Serialize, Deserialize)]
pub struct ProgressTracker {
    total: Mutex<f64>,
    remaining: Mutex<f64>,
    processes: AtomicU32,
}
