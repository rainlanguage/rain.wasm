#[macro_use]
extern crate wasm_bindgen_utils;

pub struct A {
    pub field1: String,
    pub field2: Vec<u8>,
    pub field3: HashMap<String, u64>,
}
impl_custom_tsify!(
    A,
    "export interface A {
        field1: String;
        field2: Uint8Array;
        field3: Record<string, bigint>;
    };"
);
