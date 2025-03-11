use proc_macro::TokenStream;

mod wasm_export;

/// A proc macro that generates the wasm function bindings with `WasmEncodedResult`
/// return type from rust functions that natively return [Result<T, E>], this makes
/// it possible to avoid throwing on js when return value is [Result::Err] variant,
/// instead it will return `WasmEncodedResult<T>` normally on js where either of
/// [Result::Ok] or [Result::Err] variants are included within the `WasmEncodedResult`
/// properties.
///
/// All of the `wasm_bindgen` attributes are available for this macro and are forwarded
/// directly to `wasm_bindgen` macro on expansion.
///
/// Example:
/// ```ignore
/// use wasm_bindgen_utils::prelude::*;
///
/// struct TestStruct;
///
/// #[wasm_export]
/// impl TestStruct {
///     #[wasm_export(js_name = "someStaticMethod", unchecked_return_type = "string")]
///     pub async fn some_static_method((arg1, arg2): (String, u8)) -> Result<String, Error> {
///         Ok(String::new())
///     }
///     #[wasm_export(skip)]
///     pub async fn some_skip_fn() -> Result<String, Error> {
///         Ok(String::new())
///     }
///     #[some_external_macro]
///     #[wasm_export(some_other_wbg_attrs)]
///     pub fn some_self_method(&self, arg: String) -> Result<TestStruct, Error> {
///         Ok(TestStruct)
///     }
/// }
/// ```
/// above will basically translate to the following:
/// ```ignore
/// impl TestStruct {
///     pub async fn some_static_method((arg1, arg2): (String, u8)) -> Result<String, Error> {
///         Ok(String::new())
///     }
///     pub async fn some_skip_fn() -> Result<String, Error> {
///         Ok(String::new())
///     }
///     #[some_external_macro]
///     pub fn some_self_method(&self, arg: String) -> Result<TestStruct, Error> {
///         Ok(TestStruct)
///     }
/// }
/// #[wasm_bindgen]
/// impl TestStruct {
///     #[wasm_bindgen(js_name = "someStaticMethod", unchecked_return_type = "WasmEncodedResult<string>")]
///     pub async fn some_static_method__wasm_export((arg1, arg2): (String, u8)) -> WasmEncodedResult<String> {
///         Self::some_static_method((arg1, arg2)).await.into()
///     }
///     #[wasm_bindgen(some_other_wbg_attrs, unchecked_return_type = "WasmEncodedResult<TestStruct>")]
///     pub fn some_self_method__wasm_export(&self, arg: String) -> WasmEncodedResult<TestStruct> {
///         self.some_self_method(arg).into()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn wasm_export(attr: TokenStream, item: TokenStream) -> TokenStream {
    match wasm_export::expand(attr, item) {
        Ok(tokens) => tokens,
        Err(e) => e.into_compile_error().into(),
    }
}
