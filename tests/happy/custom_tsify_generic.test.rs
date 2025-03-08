#[macro_use]
extern crate wasm_bindgen_utils;

pub struct A<T, E> {
    pub field1: T,
    pub field2: E,
}
impl_custom_tsify!(
    A,
    "export interface A<T, E> {
        field1: T;
        field2: E;
    };"
);
