#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export(preserve_js_class)]
impl TestStruct {
    pub fn some_static_method(arg: String) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }
}

fn main() {}
