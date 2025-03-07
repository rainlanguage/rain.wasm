#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export(skip = "string")]
    pub async fn some_static_method(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

fn main() {}
