pub mod prelude;
use prelude::*;

#[derive(ToBytes, FromBytes)]
pub enum Communication {
    #[enum_index(0)]
    Inputs(SizedVec<u32, SizedString<u32>>),
    #[enum_index(1)]
    Output(SizedString<u32>),
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn get_input_strings(ptr: *const u8, len: u32) -> Option<Vec<String>> {
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

pub fn pack_into_output(output: String) -> *const u8 {
    let slice = Communication::Output(output.into_sized()).to_vec();
    // Allocate memory in WASM and return a pointer to the reversed data
    let boxed_slice = slice.into_boxed_slice();
    let ptr = boxed_slice.as_ptr();
    std::mem::forget(boxed_slice); // Prevent deallocation
    ptr
}
