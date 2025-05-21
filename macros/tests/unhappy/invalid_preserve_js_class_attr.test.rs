#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export(preserve_js_class = "string")]
    pub async fn some_static_method(arg: String) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }
}

#[wasm_export(preserve_js_class = "string")]
pub async fn some_fn(arg: String) -> Result<TestStruct, Error> {
    Ok(TestStruct)
}

fn main() {}
