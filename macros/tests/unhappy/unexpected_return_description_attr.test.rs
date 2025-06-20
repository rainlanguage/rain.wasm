#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export(return_description = "unexpected description")]
impl TestStruct {
    pub async fn some_method(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

fn main() {}