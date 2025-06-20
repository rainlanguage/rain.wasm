#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
pub fn test_function(
    #[wasm_export(param_description)]
    arg: String,
) -> Result<String, Error> {
    Ok(arg)
}

fn main() {}