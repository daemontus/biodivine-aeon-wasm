use json::{array, object, JsonValue};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::bdt::{AttributeId, Bdt, BdtNodeId};
use crate::computation::TreeData;
use crate::util::index_type::IndexType;

#[wasm_bindgen]
pub struct DecisionTree {
    inner: Bdt,
}

#[wasm_bindgen]
impl DecisionTree {
    pub fn from_tree_data(result: JsValue) -> Result<DecisionTree, String> {
        let data = serde_wasm_bindgen::from_value::<TreeData>(result).unwrap();
        let inner = data.build_tree()?;
        Ok(DecisionTree { inner })
    }

    pub fn get_full_tree(&self) -> JsValue {
        JsValue::from_str(self.inner.to_json().to_string().as_str())
    }

    pub fn get_attributes(&self, node_id: usize) -> JsValue {
        let id = BdtNodeId::try_from_index(node_id, &self.inner).unwrap();
        JsValue::from_str(self.inner.attribute_gains_json(id).to_string().as_str())
    }

    pub fn get_stability_data(&self) {
        unimplemented!()
    }

    pub fn apply_attribute(&mut self, node_id: usize, attribute_id: usize) -> JsValue {
        let node_id = BdtNodeId::try_from_index(node_id, &self.inner).unwrap();
        let attribute_id = AttributeId::try_from_index(attribute_id, &self.inner).unwrap();
        let (left, right) = self.inner.make_decision(node_id, attribute_id).unwrap();
        let changes = array![
            self.inner.node_to_json(node_id),
            self.inner.node_to_json(left),
            self.inner.node_to_json(right),
        ];
        JsValue::from_str(changes.to_string().as_str())
    }

    pub fn revert_decision(&mut self, node_id: usize) -> JsValue {
        let node_id = BdtNodeId::try_from_index(node_id, &self.inner).unwrap();
        let removed = self.inner.revert_decision(node_id);
        let removed = removed
            .into_iter()
            .map(|v| v.to_index())
            .collect::<Vec<_>>();
        let response = object! {
            "node": self.inner.node_to_json(node_id),
            "removed": JsonValue::from(removed)
        };
        JsValue::from_str(response.to_string().as_str())
    }

    pub fn auto_expand(&mut self, node_id: usize, depth: u32) -> JsValue {
        let node_id = BdtNodeId::try_from_index(node_id, &self.inner).unwrap();
        let depth = if depth > 10 { 10 } else { depth };
        let changed = self.inner.auto_expand(node_id, depth);
        JsValue::from_str(self.inner.to_json_partial(&changed).to_string().as_str())
    }

    pub fn apply_tree_precision(&mut self, precision: u32) {
        self.inner.set_precision(precision)
    }

    pub fn get_tree_precision(&self) -> u32 {
        self.inner.get_precision()
    }
}
