#[macro_use]
extern crate wasm_bindgen_utils;

pub struct SomeType {
    pub msg: Option<String>,
    pub bytes: Vec<u8>,
}
impl_wasm_traits!(SomeType);
