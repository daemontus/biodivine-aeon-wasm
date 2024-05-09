use crate::boolean_network::VariableData;
use std::collections::HashMap;

pub fn infer_new_position(data: &HashMap<u64, VariableData>) -> (f64, f64) {
    if data.is_empty() {
        return (0.0, 0.0);
    }

    let mut low_corner = (f64::INFINITY, f64::INFINITY);
    let mut high_corner = (f64::NEG_INFINITY, f64::NEG_INFINITY);

    for var in data.values() {
        low_corner.0 = f64_min(var.position.0, low_corner.0);
        low_corner.1 = f64_min(var.position.1, low_corner.1);
        high_corner.0 = f64_max(var.position.0, high_corner.0);
        high_corner.1 = f64_max(var.position.1, high_corner.1);
    }

    let x = high_corner.0 + 10.0;
    let y = low_corner.1 + (high_corner.1 - low_corner.1) / 2.0;

    (x, y)
}

/// An "unsafe" implementation of minimum for f64 that assumes the numbers are never NaN
fn f64_min(x: f64, y: f64) -> f64 {
    if x < y {
        x
    } else {
        y
    }
}

fn f64_max(x: f64, y: f64) -> f64 {
    if x > y {
        x
    } else {
        y
    }
}
