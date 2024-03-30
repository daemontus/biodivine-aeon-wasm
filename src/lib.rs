use std::cmp::max;
use std::collections::HashMap;

use crate::utils::infer_new_position;
use biodivine_lib_param_bn::FnUpdate;
use biodivine_lib_param_bn::{
    symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork, Monotonicity,
};
use graph_task_context::GraphTaskContext;
use regex::Regex;
use scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use scc::algo_xie_beerel::xie_beerel_attractors;
use scc::{Behaviour, Classifier};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

mod all;
mod bdt;
mod graph_task_context;
mod scc;
mod util;
mod utils;

/// A structure that is exported to JavaScript and provides a more JS-friendly abstraction of
/// a "Boolean network".
///
/// **Work in progress, this does really have enough useful API yet.**
///
/// **Note that you have to explicitly call `free` to release a `BooleanNetworkModel` object
/// on the JS side once it is no longer needed.**
#[wasm_bindgen]
#[derive(Clone)]
pub struct BooleanNetworkModel {
    id_counter: u64,
    variable_data: HashMap<u64, VariableData>,
    // Variable to a list of regulators.
    regulation_data: HashMap<u64, Vec<RegulationData>>,
}

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
struct VariableData {
    id: u64,
    name: String,
    update_function: UpdateFunctionData,
    position: (f64, f64),
}

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
struct RegulationData {
    source: u64,
    target: u64,
    observable: bool,
    // Currently allowed values: "+", "-", ""
    monotonicity: String,
}

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
struct UpdateFunctionData {
    raw_string: String,
}

#[wasm_bindgen]
impl BooleanNetworkModel {
    pub fn new() -> BooleanNetworkModel {
        BooleanNetworkModel {
            id_counter: 0,
            variable_data: HashMap::new(),
            regulation_data: HashMap::new(),
        }
    }

    pub fn from_aeon(aeon_string: &str) -> Result<BooleanNetworkModel, String> {
        let bn = BooleanNetwork::try_from(aeon_string)?;
        let layout = read_layout(aeon_string);

        let mut variable_data = HashMap::new();
        let mut regulation_data = HashMap::new();

        for var in bn.variables() {
            let id = var.to_index() as u64;
            let fn_string = match bn.get_update_function(var).as_ref() {
                Some(fun) => fun.to_string(&bn),
                None => String::new(),
            };
            let position = layout
                .get(bn.get_variable_name(var))
                .cloned()
                .unwrap_or_else(|| infer_new_position(&variable_data));

            let data = VariableData {
                id,
                name: bn.get_variable_name(var).clone(),
                update_function: UpdateFunctionData {
                    raw_string: fn_string,
                },
                position,
            };

            variable_data.insert(id, data);
            regulation_data.insert(id, Vec::new());
        }

        for reg in bn.as_graph().regulations() {
            let data = RegulationData {
                source: reg.regulator.to_index() as u64,
                target: reg.target.to_index() as u64,
                observable: reg.is_observable(),
                monotonicity: match reg.monotonicity {
                    None => String::new(),
                    Some(Monotonicity::Activation) => "+".to_string(),
                    Some(Monotonicity::Inhibition) => "-".to_string(),
                },
            };

            let target = reg.target.to_index() as u64;
            regulation_data.get_mut(&target).unwrap().push(data);
        }

        Ok(BooleanNetworkModel {
            id_counter: bn.num_vars() as u64,
            variable_data,
            regulation_data,
        })
    }

    /// Returns true if the model has zero variables.
    pub fn is_empty(&self) -> bool {
        self.variable_data.is_empty()
    }

    /// Return the name of the variable with the given integer `id`.
    pub fn get_variable_name(&self, id: u64) -> Result<String, String> {
        if let Some(var) = self.variable_data.get(&id) {
            Ok(var.name.clone())
        } else {
            Err(format!("Value {} is not a valid variable ID.", id))
        }
    }

    pub fn add_variable(&mut self, name: Option<String>, position: Option<Vec<f64>>) -> JsValue {
        self.id_counter += 1;
        let id = self.id_counter;

        let name = match name {
            Some(x) => x,
            None => format!("v_{}", id),
        };

        let position = match position {
            Some(x) => {
                assert!(x.len() == 2);
                (x[0], x[1])
            }
            None => infer_new_position(&self.variable_data),
        };

        let update_function = UpdateFunctionData {
            raw_string: String::new(),
        };

        let data = VariableData {
            id,
            name,
            position,
            update_function,
        };

        let result = serde_wasm_bindgen::to_value(&data).unwrap();
        self.variable_data.insert(id, data);
        self.regulation_data.insert(id, Vec::new());
        result
    }
}

/// A utility object that jointly covers conversion methods for various Boolean network formats,
/// including some information about the layout of network nodes.
///
/// You should never instantiate this object. Instead, just use the static methods it provides.
#[wasm_bindgen]
pub struct Conversions {
    _dummy: (),
}

#[wasm_bindgen]
impl Conversions {
    /// Convert a Boolean network model encoded as an `.sbml` string into a model encoded
    /// as an `.aeon` string.
    ///
    /// The conversion preserves the node layout present in the `.sbml` file.
    ///
    /// When the model is not valid, a `string` error is thrown.
    pub fn sbml_to_aeon(sbml_string: &str) -> Result<String, String> {
        let (model, layout) = BooleanNetwork::try_from_sbml(sbml_string)?;
        let mut model_string = format!("{}", model); // convert back to aeon
        model_string += "\n";
        for (var, (x, y)) in layout {
            model_string += format!("#position:{}:{},{}\n", var, x, y).as_str();
        }
        Ok(model_string)
    }

    /// Convert a Boolean network model encoded as a `.bnet` string into a model encoded
    /// as an `.aeon` string.
    ///
    /// There is no layout or regulation monotonicity information in a `.bnet` model, so you
    /// need to then compute some layout manually and infer monotonicity/essentiality as well.
    ///
    /// When the model is not valid, a `string` error is thrown.
    pub fn bnet_to_aeon(bnet_string: &str) -> Result<String, String> {
        let network = BooleanNetwork::try_from_bnet(bnet_string)?;
        Ok(network.to_string())
    }

    /// Convert a Boolean network model encoded as an `.aeon` string into a model encoded
    /// as an `.sbml` string.
    ///
    /// The conversion preserves the node layout present in the `.aeon` file.
    ///
    /// When the model is not valid, a `string` error is thrown.
    #[wasm_bindgen]
    pub fn aeon_to_sbml(aeon_string: &str) -> Result<String, String> {
        let network = BooleanNetwork::try_from(aeon_string)?;
        let layout = read_layout(aeon_string);
        let sbml_string = network.to_sbml(Some(&layout));
        Ok(sbml_string)
    }

    /// Convert a Boolean network model encoded as an `.aeon` string into a model encoded
    /// as a `.bnet` string.
    ///
    /// There is no layout or regulation monotonicity information in a `.bnet` model, so these
    /// are simply discarded.
    ///
    /// When the model is not valid, a `string` error is thrown.
    #[wasm_bindgen]
    pub fn aeon_to_bnet(aeon_string: &str) -> Result<String, String> {
        let network = BooleanNetwork::try_from(aeon_string)?;
        network.to_bnet(false)
    }

    /// Deprecated: Use only for backwards-compatibility reasons.
    #[wasm_bindgen]
    pub fn aeon_to_sbml_instantiated(aeon_string: &str) -> Result<String, String> {
        let graph =
            BooleanNetwork::try_from(aeon_string).and_then(|bn| SymbolicAsyncGraph::new(&bn))?;

        let witness = graph.pick_witness(graph.unit_colors());
        let layout = read_layout(aeon_string);
        Ok(witness.to_sbml(Some(&layout)).to_string())
    }
}

/// Try to read the model layout metadata from the given aeon file.
fn read_layout(aeon_string: &str) -> HashMap<String, (f64, f64)> {
    let re = Regex::new(r"^\s*#position:(?P<var>[a-zA-Z0-9_]+):(?P<x>.+?),(?P<y>.+?)\s*$").unwrap();
    let mut layout = HashMap::new();
    for line in aeon_string.lines() {
        if let Some(captures) = re.captures(line) {
            let var = captures["var"].to_string();
            let x = captures["x"].parse::<f64>();
            let y = captures["y"].parse::<f64>();
            if let (Ok(x), Ok(y)) = (x, y) {
                layout.insert(var, (x, y));
            }
        }
    }
    layout
}

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
    pub fn start(model: &str) -> Result<ComputationResult, String> {
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

#[derive(Serialize, Deserialize)]
pub struct CardinalityData {
    cardinality: f64,
}

#[wasm_bindgen]
pub fn check_update_function(model_string: &str) -> Result<JsValue, String> {
    let model = BooleanNetwork::try_from(model_string)?;
    let mut max_size = 0;
    for v in model.variables() {
        if let Some(update_function) = model.get_update_function(v) {
            max_size = max(max_size, max_parameter_cardinality(update_function));
        } else {
            max_size = max(max_size, model.regulators(v).len())
        }
    }
    let graph = if max_size <= 5 {
        SymbolicAsyncGraph::new(&model)?
    } else {
        return Err("Function too large for on-the-fly analysis.".to_string());
    };
    let cardinality = graph.unit_colors().approx_cardinality();
    let result = CardinalityData { cardinality };
    Ok(serde_wasm_bindgen::to_value(&result).unwrap())
}

fn max_parameter_cardinality(function: &FnUpdate) -> usize {
    match function {
        FnUpdate::Const(_) | FnUpdate::Var(_) => 0,
        FnUpdate::Param(_, args) => {
            let mut max = args.len();
            for arg in args {
                let x = max_parameter_cardinality(arg);
                if x > max {
                    max = x;
                }
            }
            max
        }
        FnUpdate::Not(inner) => max_parameter_cardinality(inner),
        FnUpdate::Binary(_, left, right) => max(
            max_parameter_cardinality(left),
            max_parameter_cardinality(right),
        ),
    }
}
