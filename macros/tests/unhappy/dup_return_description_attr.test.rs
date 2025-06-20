#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export(return_description = "first description", return_description = "second description")]
    pub async fn some_method_with_dup_return_desc(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

fn main() {}