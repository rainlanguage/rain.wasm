#[macro_use]
extern crate wasm_bindgen_utils;

pub struct SomeType {
    pub msg: Option<String>,
    pub bytes: Vec<u8>,
}
impl_wasm_traits!(SomeType);

pub struct SomeGenericType<T, E> {
    pub value: T,
    pub error: E,
}
impl_wasm_traits!(SomeGenericType<T, E>);
