#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[some_external_macro]
#[wasm_export(js_name = "someSelfMethod", some_wbg_attr, some_other_wbg_attr = something)]
pub async fn some_fn(arg: String) -> Result<TestStruct, Error> {
    Ok(TestStruct)
}

#[wasm_export(unchecked_return_type = "number[]")]
pub fn some_other_fn() -> Result<Vec<u8>, Error> {
    Ok(vec![])
}
