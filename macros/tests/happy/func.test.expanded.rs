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
pub fn add(arg1: u32, arg2: u32) -> Result<u32, Error> {
    Ok(arg1 + arg2)
}
#[allow(non_snake_case)]
#[wasm_bindgen(
    unchecked_return_type = "WasmEncodedResult<u32>",
    return_description = "the result of the addition of `arg1` and `arg2`"
)]
pub fn add__wasm_export(
    #[wasm_bindgen(param_description = "the first number")]
    arg1: u32,
    #[wasm_bindgen(param_description = "the second number")]
    arg2: u32,
) -> WasmEncodedResult<u32> {
    add(arg1, arg2).into()
}
pub fn mixed_params(input: String, count: u32) -> Result<String, Error> {
    Ok(input.repeat(count as usize))
}
#[allow(non_snake_case)]
#[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<String>")]
pub fn mixed_params__wasm_export(
    #[wasm_bindgen(param_description = "the input string")]
    input: String,
    count: u32,
) -> WasmEncodedResult<String> {
    mixed_params(input, count).into()
}
pub fn with_unchecked_param_type(
    custom_param: wasm_bindgen::JsValue,
    normal_param: String,
) -> Result<String, Error> {
    Ok(normal_param)
}
#[allow(non_snake_case)]
#[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<String>")]
pub fn with_unchecked_param_type__wasm_export(
    #[wasm_bindgen(unchecked_param_type = "CustomJSType")]
    custom_param: wasm_bindgen::JsValue,
    normal_param: String,
) -> WasmEncodedResult<String> {
    with_unchecked_param_type(custom_param, normal_param).into()
}
pub fn with_mixed_param_attrs(
    mixed_param: wasm_bindgen::JsValue,
    regular_param: String,
) -> Result<String, Error> {
    Ok(regular_param)
}
#[allow(non_snake_case)]
#[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<String>")]
pub fn with_mixed_param_attrs__wasm_export(
    #[wasm_bindgen(
        param_description = "a custom JS object",
        unchecked_param_type = "MyCustomType"
    )]
    mixed_param: wasm_bindgen::JsValue,
    #[wasm_bindgen(param_description = "a regular string")]
    regular_param: String,
) -> WasmEncodedResult<String> {
    with_mixed_param_attrs(mixed_param, regular_param).into()
}
pub fn with_unchecked_and_js_name(element: wasm_bindgen::JsValue) -> Result<u32, Error> {
    Ok(42)
}
#[allow(non_snake_case)]
#[wasm_bindgen(
    js_name = "customFunction",
    unchecked_return_type = "WasmEncodedResult<u32>"
)]
pub fn with_unchecked_and_js_name__wasm_export(
    #[wasm_bindgen(unchecked_param_type = "HTMLElement")]
    element: wasm_bindgen::JsValue,
) -> WasmEncodedResult<u32> {
    with_unchecked_and_js_name(element).into()
}
pub fn with_js_name_params(
    first_name: String,
    last_name: String,
) -> Result<String, Error> {
    Ok({
        let res = ::alloc::fmt::format(format_args!("{0} {1}", first_name, last_name));
        res
    })
}
#[allow(non_snake_case)]
#[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<String>")]
pub fn with_js_name_params__wasm_export(
    #[wasm_bindgen(js_name = "firstName")]
    first_name: String,
    #[wasm_bindgen(js_name = "lastName")]
    last_name: String,
) -> WasmEncodedResult<String> {
    with_js_name_params(first_name, last_name).into()
}
pub fn with_mixed_js_attributes(
    user_data: wasm_bindgen::JsValue,
    process_mode: String,
) -> Result<bool, Error> {
    Ok(true)
}
#[allow(non_snake_case)]
#[wasm_bindgen(
    js_name = "processUserData",
    unchecked_return_type = "WasmEncodedResult<bool>"
)]
pub fn with_mixed_js_attributes__wasm_export(
    #[wasm_bindgen(
        js_name = "userData",
        param_description = "the user's data object",
        unchecked_param_type = "UserData"
    )]
    user_data: wasm_bindgen::JsValue,
    #[wasm_bindgen(js_name = "processMode")]
    process_mode: String,
) -> WasmEncodedResult<bool> {
    with_mixed_js_attributes(user_data, process_mode).into()
}
pub fn snake_to_camel_conversion(
    user_id: u32,
    is_active: bool,
    created_at: String,
) -> Result<String, Error> {
    Ok({
        let res = ::alloc::fmt::format(
            format_args!("User {0} active: {1} at {2}", user_id, is_active, created_at),
        );
        res
    })
}
#[allow(non_snake_case)]
#[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<String>")]
pub fn snake_to_camel_conversion__wasm_export(
    #[wasm_bindgen(js_name = "userId")]
    user_id: u32,
    #[wasm_bindgen(js_name = "isActive")]
    is_active: bool,
    #[wasm_bindgen(js_name = "createdAt")]
    created_at: String,
) -> WasmEncodedResult<String> {
    snake_to_camel_conversion(user_id, is_active, created_at).into()
}
