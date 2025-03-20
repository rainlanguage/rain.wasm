#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    pub async fn some_static_method(arg: String) -> String {
        String::new()
    }
}

#[wasm_export]
impl TestStruct {
    pub async fn some_static_method() {}
}

fn main() {}