use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;
use regex::Regex;
use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;

/// A utility object that jointly covers conversion methods for various Boolean network formats,
/// including some information about the layout of network nodes.
///
/// You should never instantiate this object. Instead, just use the static methods it provides.
#[wasm_bindgen]
pub struct Conversions {
    _dummy: (), // Private field prevents instantiation.
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
pub fn read_layout(aeon_string: &str) -> HashMap<String, (f64, f64)> {
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
