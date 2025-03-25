#[macro_use]
extern crate wasm_bindgen_utils_macros;
struct TestStruct;
#[some_external_macro]
impl TestStruct {
    pub async fn some_static_method(
        (arg1, arg2): (String, u8),
    ) -> Result<String, Error> {
        Ok(String::new())
    }
    pub async fn some_skip_method() -> Result<String, Error> {
        Ok(String::new())
    }
    pub async fn some_self_method(&self, arg: String) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }
}
#[wasm_bindgen]
impl TestStruct {
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        js_name = "someStaticMethod",
        unchecked_return_type = "WasmEncodedResult<string>"
    )]
    pub async fn some_static_method__wasm_export(
        (arg1, arg2): (String, u8),
    ) -> WasmEncodedResult<String> {
        Self::some_static_method((arg1, arg2)).await.into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        js_name = "someSelfMethod",
        some_wbg_attr,
        some_other_wbg_attr = something,
        unchecked_return_type = "WasmEncodedResult<TestStruct>"
    )]
    pub async fn some_self_method__wasm_export(
        &self,
        arg: String,
    ) -> WasmEncodedResult<TestStruct> {
        self.some_self_method(arg).await.into()
    }
}
impl TestStruct {
    pub fn returns_num_array(&mut self) -> Result<Vec<u8>, Error> {
        Ok(Vec::new())
    }
}
#[wasm_bindgen(some_top_wbg_attr = "something", some_other_wbg_attr)]
impl TestStruct {
    #[allow(non_snake_case)]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<Vec < u8 >>")]
    pub fn returns_num_array__wasm_export(&mut self) -> WasmEncodedResult<Vec<u8>> {
        self.returns_num_array().into()
    }
}
