#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export]
    pub fn test_method1(
        #[wasm_export(param_description = "first"; param_description = "second")]
        arg: String,
    ) -> Result<String, Error> {
        Ok(arg)
    }
}

#[wasm_export] 
impl TestStruct {
    #[wasm_export]
    pub fn test_method2(
        #[wasm_export(param_description = "test" - something_else)]
        arg: String,
    ) -> Result<String, Error> {
        Ok(arg)
    }
}

fn main() {}