#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export(return_description = something)]
    pub async fn some_method_with_invalid_return_desc(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

fn main() {}