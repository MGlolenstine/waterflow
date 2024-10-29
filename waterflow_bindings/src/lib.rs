use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{self, parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn waterflow_binding(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Get the original function signature
    let fn_name = &input.sig.ident;
    let wrapped_fn_name = format_ident!("{}_impl", fn_name); // Create new name for wrapped function
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let body = &input.block;

    // Generate the new function body
    let gen = quote! {
        // Define the original function with the new name
        fn #wrapped_fn_name(#inputs) #output {
            #body
        }

        // Define the wrapper function with the original name
        #[wasm_bindgen]
        pub fn #fn_name(ptr: *const u8, len: u32) -> *const u8 {
            let input_strings = get_input_strings(ptr, len).unwrap_or_default();
            let return_value = (|input: Vec<String>| {
                let return_value = #wrapped_fn_name(input);
                return_value
            })(input_strings);
            pack_into_output(return_value)
        }
    };

    gen.into()
}
