#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export(unchecked_return_type = something)]
    pub async fn some_static_method(arg: String) -> Result<u8, Error> {
        Ok(1)
    }
}

fn main() {}