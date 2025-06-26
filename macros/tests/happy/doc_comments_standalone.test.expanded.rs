#[macro_use]
extern crate wasm_bindgen_utils_macros;
struct TestStruct;
/// This function calculates the sum of two numbers
/// It's a simple addition operation
pub fn add(a: u32, b: u32) -> Result<u32, Error> {
    Ok(a + b)
}
/// This function calculates the sum of two numbers
/// It's a simple addition operation
#[allow(non_snake_case)]
#[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<u32>")]
pub fn add__wasm_export(a: u32, b: u32) -> WasmEncodedResult<u32> {
    add(a, b).into()
}
/// Multiply two numbers together
///
/// # Arguments
/// * `x` - The first number
/// * `y` - The second number
///
/// # Returns
/// The product of x and y
pub fn mul(x: u32, y: u32) -> Result<u32, Error> {
    Ok(x * y)
}
/// Multiply two numbers together
///
/// # Arguments
/// * `x` - The first number
/// * `y` - The second number
///
/// # Returns
/// The product of x and y
#[allow(non_snake_case)]
#[wasm_bindgen(js_name = "multiply", unchecked_return_type = "WasmEncodedResult<u32>")]
pub fn mul__wasm_export(x: u32, y: u32) -> WasmEncodedResult<u32> {
    mul(x, y).into()
}
/// Creates a new TestStruct instance
///
/// This is a more complex example with multiple
/// lines of documentation that should be preserved
/// in the generated wasm_bindgen function.
pub fn create_test_struct() -> Result<TestStruct, Error> {
    Ok(TestStruct)
}
/// Creates a new TestStruct instance
///
/// This is a more complex example with multiple
/// lines of documentation that should be preserved
/// in the generated wasm_bindgen function.
#[allow(non_snake_case)]
#[wasm_bindgen(
    unchecked_return_type = "WasmEncodedResult<TestStruct>",
    return_description = "new TestStruct instance"
)]
pub fn create_test_struct__wasm_export() -> WasmEncodedResult<TestStruct> {
    create_test_struct().into()
}
/// Async function with doc comments
/// This function demonstrates that doc comments work with async functions too
pub async fn async_with_docs(input: String) -> Result<TestStruct, Error> {
    Ok(TestStruct)
}
/// Async function with doc comments
/// This function demonstrates that doc comments work with async functions too
#[allow(non_snake_case)]
#[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<TestStruct>")]
pub async fn async_with_docs__wasm_export(input: String) -> JsValue {
    use js_sys::{Reflect, Object};
    let obj = Object::new();
    let result = async_with_docs(input).await.into();
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
