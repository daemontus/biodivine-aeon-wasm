use std::time::Duration;

use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;
use instant::Instant;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::graph_task_context::GraphTaskContext;
use crate::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use crate::scc::algo_xie_beerel::xie_beerel_attractors;
use crate::scc::{Behaviour, Classifier};

#[wasm_bindgen]
pub struct ComputationResult {
    network: BooleanNetwork,
    graph: SymbolicAsyncGraph,
    classifier: Classifier,
    task: GraphTaskContext,
    elapsed: Duration,
}

#[derive(Serialize, Deserialize)]
pub struct ResultsSummary {
    is_finished: bool,
    progress: String,
    data: Vec<ResultsSummaryRow>,
    elapsed: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ResultsSummaryRow {
    sat_count: f64,
    phenotype: Vec<Behaviour>,
}

#[wasm_bindgen]
impl ComputationResult {
    pub fn compute(
        model: &str,
        on_progress: &js_sys::Function,
    ) -> Result<ComputationResult, String> {
        let bn = BooleanNetwork::try_from(model)?;
        let graph = SymbolicAsyncGraph::new(&bn)?;
        let classifier = Classifier::new(&graph);

        let graph_task_context = GraphTaskContext::new();
        // Now we can actually start the computation...

        // First, perform ITGR reduction.
        let (universe, active_variables) = interleaved_transition_guided_reduction(
            &graph_task_context,
            &graph,
            graph.mk_unit_colored_vertices(),
            |task| {
                let elapsed = Instant::now() - task.started;
                on_progress
                    .call0(&Self::get_results_internal(elapsed, task, &classifier))
                    .unwrap();
            },
        );

        // Then run Xie-Beerel to actually detect the components.
        xie_beerel_attractors(
            &graph_task_context,
            &graph,
            &universe,
            &active_variables,
            |component| {
                classifier.add_component(component, &graph);
            },
            |task| {
                let elapsed = Instant::now() - task.started;
                on_progress
                    .call0(&Self::get_results_internal(elapsed, task, &classifier))
                    .unwrap();
            },
        );

        let elapsed = Instant::now() - graph_task_context.started;
        Ok(ComputationResult {
            network: bn,
            graph,
            classifier,
            task: graph_task_context,
            elapsed,
        })
    }

    pub fn get_results(&self) -> JsValue {
        Self::get_results_internal(self.elapsed, &self.task, &self.classifier)
    }

    /*pub fn build_tree(&self) -> Bdt {
        Bdt::new_from_graph(self.classifier.export_result(), &self.graph, &self.network)
    }*/

    fn get_results_internal(
        elapsed: Duration,
        task: &GraphTaskContext,
        classifier: &Classifier,
    ) -> JsValue {
        let data = classifier.export_result();

        let mut data_result = Vec::new();
        for (k, v) in &data {
            data_result.push(ResultsSummaryRow {
                sat_count: v.approx_cardinality(),
                phenotype: k.get_vector(),
            })
        }

        let result = ResultsSummary {
            is_finished: task.progress.is_finished(),
            progress: task.progress.get_percent_string(),
            elapsed: elapsed.as_millis() as u64,
            data: data_result,
        };

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}
