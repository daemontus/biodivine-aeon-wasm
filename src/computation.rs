use crate::graph_task_context::GraphTaskContext;
use crate::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use crate::scc::algo_xie_beerel::xie_beerel_attractors;
use crate::scc::{Behaviour, Classifier};
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub struct ComputationResult {
    graph: SymbolicAsyncGraph,
    classifier: Classifier,
    elapsed: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ResultsSummary {
    is_partial: bool,
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
    pub fn compute(model: &str) -> Result<ComputationResult, String> {
        let start = instant::Instant::now();

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
        );

        let elapsed = instant::Instant::now() - start;

        Ok(ComputationResult {
            graph,
            classifier,
            elapsed: elapsed.as_millis() as u64,
        })
    }

    pub fn get_results(&self) -> JsValue {
        let data = self.classifier.export_result();

        let mut data_result = Vec::new();
        for (k, v) in &data {
            data_result.push(ResultsSummaryRow {
                sat_count: v.approx_cardinality(),
                phenotype: k.get_vector(),
            })
        }

        let result = ResultsSummary {
            is_partial: false,
            elapsed: self.elapsed,
            data: data_result,
        };

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}
