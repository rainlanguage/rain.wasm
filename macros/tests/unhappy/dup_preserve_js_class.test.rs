#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export(preserve_js_class, preserve_js_class)]
    pub async fn some_static_method(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

#[wasm_export(preserve_js_class, preserve_js_class)]
pub fn some_static_method(arg: String) -> Result<String, Error> {
    Ok(String::new())
}

fn main() {}
