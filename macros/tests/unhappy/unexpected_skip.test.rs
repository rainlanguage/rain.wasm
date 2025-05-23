#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export(skip)]
impl TestStruct {
    pub async fn some_static_method(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

#[wasm_export(skip)]
pub fn some_fn(arg: String) -> Result<String, Error> {
    Ok(String::new())
}

fn main() {}