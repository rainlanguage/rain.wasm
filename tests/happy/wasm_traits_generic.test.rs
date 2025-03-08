#[macro_use]
extern crate wasm_bindgen_utils;

pub struct SomeGenericType<T, E, K> {
    pub field: T,
    pub other_field: E,
    pub another_field: K,
}
impl_wasm_traits!(SomeGenericType<T, E, K>);
