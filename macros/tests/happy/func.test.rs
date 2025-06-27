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

#[wasm_export(js_name = "someFnPreserveClassAsync", preserve_js_class)]
pub async fn some_fn_preserve_class_async(arg: String) -> Result<TestStruct, Error> {
    Ok(TestStruct)
}

#[wasm_export(js_name = "someFnPreserveClassSync", preserve_js_class)]
pub fn some_fn_preserve_class_sync(arg: String) -> Result<TestStruct, Error> {
    Ok(TestStruct)
}

#[wasm_export(return_description = "returns the sum of two numbers")]
pub fn add_with_description(a: u32, b: u32) -> Result<u32, Error> {
    Ok(a + b)
}

#[wasm_export(js_name = "complexCalc", return_description = "performs complex calculation", catch)]
pub async fn complex_calculation_with_desc(input: String) -> Result<i64, Error> {
    Ok(42)
}

#[wasm_export(return_description = "the result of the addition of `arg1` and `arg2`")]
pub fn add(
    #[wasm_export(param_description = "the first number")]
    arg1: u32,
    #[wasm_export(param_description = "the second number")]
    arg2: u32,
) -> Result<u32, Error> {
    Ok(arg1 + arg2)
}

#[wasm_export]
pub fn mixed_params(
    #[wasm_export(param_description = "the input string")]
    input: String,
    count: u32,  // no description
) -> Result<String, Error> {
    Ok(input.repeat(count as usize))
}

#[wasm_export]
pub fn with_unchecked_param_type(
    #[wasm_export(unchecked_param_type = "CustomJSType")]
    custom_param: wasm_bindgen::JsValue,
    normal_param: String,
) -> Result<String, Error> {
    Ok(normal_param)
}

#[wasm_export]
pub fn with_mixed_param_attrs(
    #[wasm_export(param_description = "a custom JS object", unchecked_param_type = "MyCustomType")]
    mixed_param: wasm_bindgen::JsValue,
    #[wasm_export(param_description = "a regular string")]
    regular_param: String,
) -> Result<String, Error> {
    Ok(regular_param)
}

#[wasm_export(js_name = "customFunction")]
pub fn with_unchecked_and_js_name(
    #[wasm_export(unchecked_param_type = "HTMLElement")]
    element: wasm_bindgen::JsValue,
) -> Result<u32, Error> {
    Ok(42)
}
