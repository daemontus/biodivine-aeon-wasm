use biodivine_lib_bdd::Bdd;
use std::collections::HashMap;
use std::time::Duration;

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;
use instant::Instant;
use serde::Deserialize;
use serde::Serialize;

use crate::bdt::Bdt;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::graph_task_context::GraphTaskContext;
use crate::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use crate::scc::algo_xie_beerel::xie_beerel_attractors;
use crate::scc::{Behaviour, Class, Classifier};

#[wasm_bindgen]
pub struct ComputationResult {
    network: BooleanNetwork,
    graph: SymbolicAsyncGraph,
    classifier: Classifier,
    task: GraphTaskContext,
    elapsed: Duration,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct TreeData {
    network: String,
    data: HashMap<Class, Vec<u8>>,
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
        graph_task_context.restart(&graph);
        // Now we can actually start the computation...

        // First, perform ITGR reduction.
        let (universe, active_variables) = interleaved_transition_guided_reduction(
            &graph_task_context,
            &graph,
            graph.mk_unit_colored_vertices(),
            |task| {
                let elapsed = Instant::now() - task.started;
                on_progress
                    .call1(
                        on_progress,
                        &Self::get_results_internal(elapsed, task, &classifier),
                    )
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
                    .call1(
                        on_progress,
                        &Self::get_results_internal(elapsed, task, &classifier),
                    )
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

    pub fn get_tree_data(&self) -> TreeData {
        let mut serialized_data = HashMap::new();
        for (k, v) in self.classifier.export_result() {
            serialized_data.insert(k, v.into_bdd().to_bytes());
        }
        TreeData {
            network: self.network.to_string(),
            data: serialized_data,
        }
    }

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

#[wasm_bindgen]
impl TreeData {
    pub fn to_js(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self).unwrap()
    }

    pub fn from_js(value: JsValue) -> TreeData {
        serde_wasm_bindgen::from_value(value).unwrap()
    }
}

impl TreeData {

    pub fn build_tree(&self) -> Result<Bdt, String> {
        let network = BooleanNetwork::try_from(self.network.as_str())?;
        let graph = SymbolicAsyncGraph::new(&network)?;
        let mut native_data = HashMap::new();
        for (k, v) in &self.data {
            // WTF this conversion?
            let v_copy = v.clone();
            let mut slide = v_copy.as_slice();
            let native_v = GraphColors::new(Bdd::from_bytes(&mut slide), graph.symbolic_context());
            native_data.insert(k.clone(), native_v);
        }

        Ok(Bdt::new_from_graph(native_data, &graph, &network))
    }
}
