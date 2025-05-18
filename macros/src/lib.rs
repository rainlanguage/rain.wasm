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
///
/// ### Preserving JS Class In WasmEncodedResult
/// By default, `WasmEncodedResult` is serialized to JS/TS using `serde_wasm_bindgen`
/// due to wasm_bindgen's limitations with generic types, this is fine when the Ok variant
/// is not a JS/TS class instance, however, this approach does not preserve JS/TS class
/// instances if a Rust struct corresponds to one, but rather `serde_wasm_bindgen` serialization
/// will convert it to a plain object, similar to JSON stringification.
///
/// To address this, you can use the `preserve_js_class` attribute on a method or function;
/// when enabled, the macro (using `js_sys` lib helpers) bypasses `serde_wasm_bindgen`
/// serialization and manually populates the `value` and `error` fields of an empty JS
/// object to resemble `WasmEncodedResult`, ensuring that class instances are preserved
/// in JS/TS as expected. As a result, the macro generated exporting function in Rust has
/// to return a `JsValue` rather than `WasmEncodedResult<T>`, but it is typed with
/// `unchecked_return_type = "WasmEncodedResult<T>"` (this is just a technicality of it,
/// it doesn't affect the users of the macro)
///
/// example:
/// in rust we will use it like:
/// ```ignore
/// #[wasm_bindgen]
/// struct TestStruct;
///
/// #[wasm_export]
/// impl TestStruct {
///     #[wasm_export(js_name = "new", preserve_js_class)]
///     pub fn new() -> Result<TestStruct, Error> {
///         Ok(TestStruct)
///     }
/// }
/// ```
///
/// and we will get the following on JS/TS:
/// ```ts
/// const result = TestStruct.new();
/// if (result.error) {
///     // handle error
/// } else {
///     const testStruct = result.value;
///     assert(
///         testStruct instanceof TestStruct,
///         "Expected to be instance of TestStruct, but it is not"
///     );
///     // do stuff
/// }
/// ```
#[proc_macro_attribute]
pub fn wasm_export(attr: TokenStream, item: TokenStream) -> TokenStream {
    match wasm_export::expand(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
