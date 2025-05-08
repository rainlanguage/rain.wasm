#[macro_use]
extern crate wasm_bindgen_utils_macros;

#[wasm_export]
fn some_fn(arg: String) -> Result<String, Error> {
    Ok(String::new())
}

#[wasm_export]
pub(crate) fn some_other_fn(arg: String) -> Result<String, Error> {
    Ok(String::new())
}

fn main() {}
