use bypar::ToBytes as _;
use bypar::{
    prelude::{IntoSizedString, SizedString, SizedVec},
    FromBytes as _,
};
use bypar_derive::{FromBytes, ToBytes};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, ItemFn};
use waterflow_plugin_interface::Communication;

fn get_input_strings(ptr: *const u8, len: u32) -> Option<Vec<String>> {
    // Convert the raw pointer and length into a slice
    let input = unsafe { std::slice::from_raw_parts(ptr, len as usize) };

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

fn pack_into_output(output: String) -> *const u8 {
    let slice = Communication::Output(output.into_sized()).to_vec();
    // Allocate memory in WASM and return a pointer to the reversed data
    let boxed_slice = slice.into_boxed_slice();
    let ptr = boxed_slice.as_ptr();
    std::mem::forget(boxed_slice); // Prevent deallocation
    ptr
}

#[proc_macro_attribute]
pub fn waterflow_binding(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Get the original function signature
    let fn_name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;

    // Generate the new function body
    let gen = quote! {
        #[wasm_bindgen]
        #input

        fn #fn_name(ptr: *const u8, len: u32) -> *const u8 {
            let input_strings = get_input_strings(ptr, len).unwrap_or_default();
            let return_value = (|input: Vec<String>| {
                let return_value = #fn_name(input);
                return_value
            })(input_strings);
            pack_into_output(return_value)
        }
    };

    gen.into()
}
