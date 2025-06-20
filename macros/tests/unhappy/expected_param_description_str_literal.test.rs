#[macro_use]
extern crate wasm_bindgen_utils_macros;

#[wasm_export]
impl TestStruct {
    #[wasm_export]
    pub fn test_method(
        #[wasm_export(param_description = something)]
        arg: String,
    ) -> Result<String, Error> {
        Ok(arg)
    }
}

struct TestStruct;

fn main() {}