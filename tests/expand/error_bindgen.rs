#[macro_use]
extern crate wasm_bindgen_utils;

#[error_bindgen]
#[derive(Error, Debug)]
pub enum A {
    X(String),
    Y(Vec<u8>),
}