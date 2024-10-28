use bypar::ToBytes as _;
use bypar::{
    prelude::{IntoSizedString, SizedString, SizedVec},
    FromBytes as _,
};
use bypar_derive::{FromBytes, ToBytes};
use wasm_bindgen::prelude::*;

#[derive(ToBytes, FromBytes)]
pub enum Communication {
    #[enum_index(0)]
    Inputs(SizedVec<u32, SizedString<u32>>),
    #[enum_index(1)]
    Output(SizedString<u32>),
}

fn get_input_strings(input: &[u8]) -> Option<Vec<String>> {
    let Ok(Communication::Inputs(input_strings)) = Communication::from_bytes(input) else {
        return None;
    };

    Some(
        Vec::from(input_strings)
            .into_iter()
            .fold(vec![], |mut c, a| {
                c.push(String::from(a.clone()));
                c
            }),
    )
}

fn pack_into_output(output: String) -> Vec<u8> {
    Communication::Output(output.into_sized()).to_vec()
}

#[wasm_bindgen]
pub fn reverse_join(input: &[u8]) -> Vec<u8> {
    let input_strings = get_input_strings(input).unwrap_or_default();

    let return_value = input_strings
        .into_iter()
        .rev()
        .collect::<Vec<String>>()
        .join(", ");
    pack_into_output(return_value)
}

#[wasm_bindgen]
pub fn normal_join(input: &[u8]) -> Vec<u8> {
    let input_strings = get_input_strings(input).unwrap_or_default();

    let return_value = input_strings
        .into_iter()
        .collect::<Vec<String>>()
        .join(", ");
    pack_into_output(return_value)
}
