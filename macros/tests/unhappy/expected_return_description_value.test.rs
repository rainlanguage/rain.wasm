#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export(return_description)]
    pub async fn some_method_missing_return_desc_value(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

fn main() {}