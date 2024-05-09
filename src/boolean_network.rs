use crate::read_layout;
use crate::utils::infer_new_position;
use biodivine_lib_param_bn::{BooleanNetwork, Monotonicity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

/// A structure that is exported to JavaScript and provides a more JS-friendly abstraction of
/// a "Boolean network".
///
/// **Work in progress! Most of the actually useful APIs are still missing.**
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
pub(crate) struct VariableData {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) update_function: UpdateFunctionData,
    pub(crate) position: (f64, f64),
}

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct RegulationData {
    source: u64,
    target: u64,
    observable: bool,
    // Currently allowed values: "+", "-", ""
    monotonicity: String,
}

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct UpdateFunctionData {
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
                assert_eq!(x.len(), 2);
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
