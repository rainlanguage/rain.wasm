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
pub async fn some_fn_preserve_class_async(arg: String) -> Result<TestStruct, Error> {
    Ok(TestStruct)
}
#[allow(non_snake_case)]
#[wasm_bindgen(
    js_name = "someFnPreserveClassAsync",
    unchecked_return_type = "WasmEncodedResult<TestStruct>"
)]
pub async fn some_fn_preserve_class_async__wasm_export(arg: String) -> JsValue {
    use js_sys::{Reflect, Object};
    let obj = Object::new();
    let result = some_fn_preserve_class_async(arg).await.into();
    match result {
        Ok(value) => {
            Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
            Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED)
                .unwrap();
        }
        Err(error) => {
            let wasm_error: WasmEncodedError = error.into();
            Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED)
                .unwrap();
            Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into()).unwrap();
        }
    };
    obj.into()
}
pub fn some_fn_preserve_class_sync(arg: String) -> Result<TestStruct, Error> {
    Ok(TestStruct)
}
#[allow(non_snake_case)]
#[wasm_bindgen(
    js_name = "someFnPreserveClassSync",
    unchecked_return_type = "WasmEncodedResult<TestStruct>"
)]
pub fn some_fn_preserve_class_sync__wasm_export(arg: String) -> JsValue {
    use js_sys::{Reflect, Object};
    let obj = Object::new();
    let result = some_fn_preserve_class_sync(arg).into();
    match result {
        Ok(value) => {
            Reflect::set(&obj, &JsValue::from_str("value"), &value.into()).unwrap();
            Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::UNDEFINED)
                .unwrap();
        }
        Err(error) => {
            let wasm_error: WasmEncodedError = error.into();
            Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::UNDEFINED)
                .unwrap();
            Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into()).unwrap();
        }
    };
    obj.into()
}
pub fn add_with_description(a: u32, b: u32) -> Result<u32, Error> {
    Ok(a + b)
}
#[allow(non_snake_case)]
#[wasm_bindgen(
    unchecked_return_type = "WasmEncodedResult<u32>",
    return_description = "returns the sum of two numbers"
)]
pub fn add_with_description__wasm_export(a: u32, b: u32) -> WasmEncodedResult<u32> {
    add_with_description(a, b).into()
}
pub async fn complex_calculation_with_desc(input: String) -> Result<i64, Error> {
    Ok(42)
}
#[allow(non_snake_case)]
#[wasm_bindgen(
    js_name = "complexCalc",
    catch,
    unchecked_return_type = "WasmEncodedResult<i64>",
    return_description = "performs complex calculation"
)]
pub async fn complex_calculation_with_desc__wasm_export(
    input: String,
) -> WasmEncodedResult<i64> {
    complex_calculation_with_desc(input).await.into()
}
