#[macro_use]
extern crate wasm_bindgen_utils_macros;

#[wasm_export]
struct TestStruct;

#[wasm_export]
enum TestEnum {}

#[wasm_export]
fn test() {}

#[wasm_export]
type SomeType = u8;

#[wasm_export]
mod SomeMod {}

#[wasm_export]
const SOME_COST: u8 = 1;

#[wasm_export]
static SOME_STATIC: u8 = 1;

fn main() {}
