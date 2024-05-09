use std::cmp::max;

use biodivine_lib_param_bn::FnUpdate;
use biodivine_lib_param_bn::{
    symbolic_async_graph::SymbolicAsyncGraph, BooleanNetwork, RegulatoryGraph,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub use conversions::read_layout;
pub use conversions::Conversions;

mod all;
mod bdt;
mod graph_task_context;
mod scc;
mod util;
mod utils;

mod boolean_network;
mod computation;
mod conversions;

#[derive(Serialize, Deserialize)]
pub struct CardinalityData {
    cardinality: f64,
}

#[wasm_bindgen]
pub fn check_update_function(model_string: &str) -> Result<JsValue, String> {
    let model_base = BooleanNetwork::try_from(model_string)?;

    let mut variable_names = Vec::new();
    for var in model_base.variables() {
        variable_names.push((var, model_base.get_variable_name(var).clone()));
    }
    variable_names.sort_by_cached_key(|(id, _)| model_base.as_graph().regulators(*id).len());

    let variable_names = variable_names
        .into_iter()
        .map(|(_, name)| name)
        .collect::<Vec<_>>();

    let mut new_rg = RegulatoryGraph::new(variable_names);
    for reg in model_base.as_graph().regulations() {
        let new_regulator = new_rg
            .find_variable(model_base.get_variable_name(reg.regulator))
            .unwrap();
        let new_target = new_rg
            .find_variable(model_base.get_variable_name(reg.target))
            .unwrap();
        let mut new_reg = reg.clone();
        new_reg.regulator = new_regulator;
        new_reg.target = new_target;
        new_rg.add_raw_regulation(new_reg).unwrap()
    }

    let mut model = BooleanNetwork::new(new_rg);

    for p in model_base.parameters() {
        let param = model_base.get_parameter(p);
        model
            .add_parameter(param.get_name(), param.get_arity())
            .unwrap();
    }

    for var in model.variables() {
        let var_name = model.get_variable_name(var).clone();
        let old_id = model_base
            .as_graph()
            .find_variable(var_name.as_str())
            .unwrap();
        if let Some(old_fn) = model_base.get_update_function(old_id) {
            let old_fn = old_fn.to_string(&model_base);
            model
                .add_string_update_function(var_name.as_str(), old_fn.as_str())
                .unwrap();
        }
    }

    // At this point, model should be sorted in the optimal ordering for the type of
    // function that we are exploring.

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
