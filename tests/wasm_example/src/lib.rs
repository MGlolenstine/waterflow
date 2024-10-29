use wasm_bindgen::prelude::*;
use waterflow_bindings::waterflow_binding;
use waterflow_plugin_interface::prelude::*;

#[waterflow_binding]
pub fn reverse_join(input: Vec<String>) -> String {
    let return_value = input.into_iter().rev().collect::<Vec<String>>().join(", ");
    return_value
}

#[waterflow_binding]
pub fn normal_join(input: Vec<String>) -> String {
    let return_value = input.into_iter().collect::<Vec<String>>().join(", ");
    return_value
}
