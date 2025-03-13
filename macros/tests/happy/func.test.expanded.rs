#[macro_use]
extern crate wasm_bindgen_utils_macros;
struct TestStruct;
#[some_external_macro]
pub async fn some_fn(arg: String) -> Result<TestStruct, Error> {
    Ok(TestStruct)
}
#[allow(non_snake_case)]
#[wasm_bindgen(
    js_name = "someSelfMethod",
    some_wbg_attr,
    some_other_wbg_attr = something,
    unchecked_return_type = "WasmEncodedResult<TestStruct>"
)]
pub async fn some_fn__wasm_export(arg: String) -> WasmEncodedResult<TestStruct> {
    some_fn(arg).await.into()
}
pub fn some_other_fn() -> Result<Vec<u8>, Error> {
    Ok(::alloc::vec::Vec::new())
}
#[allow(non_snake_case)]
#[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<number[]>")]
pub fn some_other_fn__wasm_export() -> WasmEncodedResult<Vec<u8>> {
    some_other_fn().into()
}
