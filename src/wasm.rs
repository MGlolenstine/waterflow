use crate::error::Error;
use crate::Result;
use bypar::ToBytes as _;
use bypar::{
    prelude::{IntoSizedString, IntoSizedVec, SizedString, SizedVec},
    FromBytes as _,
};
use bypar_derive::{FromBytes, ToBytes};
use std::sync::OnceLock;
use wasmtime::*;

static ENGINE: OnceLock<Engine> = OnceLock::new();

#[derive(ToBytes, FromBytes)]
pub enum Communication {
    #[enum_index(0)]
    Inputs(SizedVec<u32, SizedString<u32>>),
    #[enum_index(1)]
    Output(SizedString<u32>),
}

pub(crate) fn run_wasm_code(
    function_name: &str,
    file_name: &str,
    inputs: &[String],
) -> Result<String> {
    let engine = ENGINE.get_or_init(Engine::default);
    let module = Module::from_file(engine, file_name)
        .map_err(|_| panic!())
        .unwrap();

    let mut store = Store::new(engine, ());
    let mut linker = Linker::<()>::new(engine);

    // Implement core `wasm-bindgen` placeholder functions
    linker.func_wrap(
        "__wbindgen_placeholder__",
        "__wbindgen_describe",
        |_: i32| {},
    )?;

    // Mimic `__wbindgen_throw` for throwing errors
    linker.func_wrap(
        "__wbindgen_placeholder__",
        "__wbindgen_throw",
        |ptr: i32, len: i32| {
            let message = format!("WASM throw at ptr: {} len: {}", ptr, len);
            eprintln!("{}", message); // Log error message to stderr
        },
    )?;

    // `__wbindgen_externref_xform__` functions for managing references
    linker.func_wrap(
        "__wbindgen_externref_xform__",
        "__wbindgen_externref_table_grow",
        |_: i32| -> i32 {
            0 // Return a default value for this placeholder
        },
    )?;
    linker.func_wrap(
        "__wbindgen_externref_xform__",
        "__wbindgen_externref_table_set_null",
        |_: i32| {},
    )?;

    // let instance = Instance::new(&mut store, &module, &[]).unwrap();
    let instance = linker.instantiate(&mut store, &module)?;

    let input_bytes = get_input_bytes(inputs);

    let memory = instance
        .get_memory(&mut store, "memory")
        .expect("The WASM module doesn't contain 'memory' section!");

    // Allocate memory in the WebAssembly instance
    let input_len = input_bytes.len() as u32;
    let input_ptr = memory.data_size(&mut store) as u32; // Get the current size of the memory
    memory.grow(&mut store, (input_len as u64 + 1) / 64 + 1)?; // Grow memory if necessary

    // Copy the input bytes to the WebAssembly memory
    memory.write(&mut store, input_ptr as usize, &input_bytes)?;

    // Call the function with the pointer to the byte array and its length
    let my_function =
        instance.get_typed_func::<(i32, i32), (i32, i32)>(&mut store, function_name)?;
    let (result_ptr, result_len) =
        my_function.call(&mut store, (input_ptr as i32, input_len as i32))?;

    // Retrieve the resulting byte array from the WebAssembly memory
    let mut result_slice = vec![0u8; result_len as usize];
    memory.read(&mut store, result_ptr as usize, &mut result_slice)?;

    let output = Communication::from_bytes(&result_slice)?;

    let Communication::Output(output) = output else {
        return Err(Error::WasmWrongTypeReturned);
    };

    Ok(output.into())
}

fn get_input_bytes(inputs: &[String]) -> Vec<u8> {
    let inputs = Communication::Inputs(
        inputs
            .iter()
            .map(|s| s.to_string().into_sized())
            .collect::<Vec<_>>()
            .into_sized(),
    );

    inputs.to_vec()
}
