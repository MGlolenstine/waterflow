use bypar::ToBytes as _;
use bypar::{
    prelude::{IntoSizedString, SizedString, SizedVec},
    FromBytes as _,
};
use bypar_derive::{FromBytes, ToBytes};
use wasm_bindgen::prelude::*;
use waterflow_plugin_interface::Communication;

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
