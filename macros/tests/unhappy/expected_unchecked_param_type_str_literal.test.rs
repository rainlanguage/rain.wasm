#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export]
    pub async fn test_method(
        #[wasm_export(unchecked_param_type = notStringLiteral)]
        arg: wasm_bindgen::JsValue
    ) -> Result<String, Error> {
        Ok("test".to_string())
    }
}

fn main() {}