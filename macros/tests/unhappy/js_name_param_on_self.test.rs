#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export]
    pub async fn test_method(
        #[wasm_export(js_name = "selfParam")]
        &self,
        arg: String
    ) -> Result<String, Error> {
        Ok(arg)
    }
}

fn main() {}