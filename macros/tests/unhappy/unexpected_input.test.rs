#[macro_use]
extern crate wasm_bindgen_utils_macros;

#[wasm_export]
struct TestStruct;

#[wasm_export]
enum TestEnum {}

#[wasm_export]
type TestType = u8;

#[wasm_export]
mod test_mod {}

#[wasm_export]
const TEST_COST: u8 = 1;

#[wasm_export]
static TEST_STATIC: u8 = 1;

fn main() {}
