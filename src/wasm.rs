use crate::error::Error;
use crate::Result;
use bypar::ToBytes as _;
use bypar::{
    prelude::{IntoSizedString, IntoSizedVec},
    FromBytes as _,
};
use std::sync::OnceLock;
use wasmtime::*;
use waterflow_plugin_interface::Communication;

static ENGINE: OnceLock<Engine> = OnceLock::new();

pub(crate) fn run_wasm_code(
    function_name: &str,
    file_name: &str,
    inputs: &[String],
) -> Result<String> {
    let input = get_input_bytes(inputs);
    let engine = ENGINE.get_or_init(Engine::default);
    let module = Module::from_file(engine, file_name).expect("Failed to load WASM module");
    let mut store = Store::new(engine, ());

    // Instantiate the WASM module
    let instance = Instance::new(&mut store, &module, &[]).expect("Failed to instantiate WASM");

    // Get the `reverse_bytes` function
    let reverse_bytes = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, function_name)
        .expect("Failed to get function");

    // Allocate memory for the input data
    let memory = instance
        .get_memory(&mut store, "memory")
        .expect("Failed to find memory");

    // Copy input data to WASM memory
    memory.data_mut(&mut store)[..input.len()].copy_from_slice(&input);

    // Call the WASM function
    let output_ptr = reverse_bytes
        .call(&mut store, (0, input.len() as i32))
        .expect("Failed to call function");

    // Retrieve the output data from WASM memory
    let output_len = input.len(); // Output length is the same as input length
    let output =
        memory.data(&store)[output_ptr as usize..(output_ptr as usize + output_len)].to_vec();

    let output = Communication::from_bytes(&output)?;

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
