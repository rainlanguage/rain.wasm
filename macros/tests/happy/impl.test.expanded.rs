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
    pub async fn some_method_preserve_class_async(
        arg: String,
    ) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }
    pub fn some_method_preserve_class_sync(
        &self,
        arg: String,
    ) -> Result<TestStruct, Error> {
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
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        js_name = "someMethodPreserveClassAsync",
        unchecked_return_type = "WasmEncodedResult<TestStruct>"
    )]
    pub async fn some_method_preserve_class_async__wasm_export(arg: String) -> JsValue {
        use js_sys::{Reflect, Object};
        let obj = Object::new();
        let result = Self::some_method_preserve_class_async(arg).await.into();
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
                Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into())
                    .unwrap();
            }
        };
        obj.into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        js_name = "someMethodPreserveClassSync",
        unchecked_return_type = "WasmEncodedResult<TestStruct>"
    )]
    pub fn some_method_preserve_class_sync__wasm_export(&self, arg: String) -> JsValue {
        use js_sys::{Reflect, Object};
        let obj = Object::new();
        let result = self.some_method_preserve_class_sync(arg).into();
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
                Reflect::set(&obj, &JsValue::from_str("error"), &wasm_error.into())
                    .unwrap();
            }
        };
        obj.into()
    }
}
impl TestStruct {
    pub fn returns_num_array(&mut self) -> Result<Vec<u8>, Error> {
        Ok(Vec::new())
    }
    pub fn get_name(&self) -> Result<String, Error> {
        Ok("test".to_string())
    }
    pub fn get_age(&self, user_id: u32) -> Result<u32, Error> {
        Ok(25)
    }
    pub fn number(&self, index: u32) -> Result<u32, Error> {
        Ok(index * 2)
    }
    pub fn with_unchecked_param(
        &self,
        custom_param: wasm_bindgen::JsValue,
    ) -> Result<String, Error> {
        Ok("success".to_string())
    }
    pub fn process_element(
        &mut self,
        element: wasm_bindgen::JsValue,
        options: String,
    ) -> Result<bool, Error> {
        Ok(true)
    }
    pub fn with_js_name_params(
        &self,
        primary_key: u32,
        display_name: String,
    ) -> Result<String, Error> {
        Ok({
            let res = ::alloc::fmt::format(
                format_args!("Item {0}: {1}", primary_key, display_name),
            );
            res
        })
    }
    pub fn update_record(
        &mut self,
        record_id: u32,
        new_data: wasm_bindgen::JsValue,
        save_options: String,
    ) -> Result<bool, Error> {
        Ok(true)
    }
}
#[wasm_bindgen(some_top_wbg_attr = "something", some_other_wbg_attr)]
impl TestStruct {
    #[allow(non_snake_case)]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<Vec < u8 >>")]
    pub fn returns_num_array__wasm_export(&mut self) -> WasmEncodedResult<Vec<u8>> {
        self.returns_num_array().into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        unchecked_return_type = "WasmEncodedResult<String>",
        return_description = "gets the user's name"
    )]
    pub fn get_name__wasm_export(&self) -> WasmEncodedResult<String> {
        self.get_name().into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        js_name = "getUserAge",
        catch,
        unchecked_return_type = "WasmEncodedResult<u32>",
        return_description = "returns user age in years"
    )]
    pub fn get_age__wasm_export(&self, user_id: u32) -> WasmEncodedResult<u32> {
        self.get_age(user_id).into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        unchecked_return_type = "WasmEncodedResult<u32>",
        return_description = "the number at the given index"
    )]
    pub fn number__wasm_export(
        &self,
        #[wasm_bindgen(param_description = "the index of the number to be returned")]
        index: u32,
    ) -> WasmEncodedResult<u32> {
        self.number(index).into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<String>")]
    pub fn with_unchecked_param__wasm_export(
        &self,
        #[wasm_bindgen(unchecked_param_type = "CustomJSType")]
        custom_param: wasm_bindgen::JsValue,
    ) -> WasmEncodedResult<String> {
        self.with_unchecked_param(custom_param).into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        js_name = "processElement",
        unchecked_return_type = "WasmEncodedResult<bool>"
    )]
    pub fn process_element__wasm_export(
        &mut self,
        #[wasm_bindgen(
            unchecked_param_type = "HTMLElement",
            param_description = "the DOM element to process"
        )]
        element: wasm_bindgen::JsValue,
        #[wasm_bindgen(param_description = "processing options")]
        options: String,
    ) -> WasmEncodedResult<bool> {
        self.process_element(element, options).into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<String>")]
    pub fn with_js_name_params__wasm_export(
        &self,
        #[wasm_bindgen(js_name = "primaryKey")]
        primary_key: u32,
        #[wasm_bindgen(js_name = "displayName")]
        display_name: String,
    ) -> WasmEncodedResult<String> {
        self.with_js_name_params(primary_key, display_name).into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        js_name = "updateRecord",
        unchecked_return_type = "WasmEncodedResult<bool>"
    )]
    pub fn update_record__wasm_export(
        &mut self,
        #[wasm_bindgen(
            js_name = "recordId",
            param_description = "unique identifier for the record"
        )]
        record_id: u32,
        #[wasm_bindgen(js_name = "newData", unchecked_param_type = "RecordData")]
        new_data: wasm_bindgen::JsValue,
        #[wasm_bindgen(
            js_name = "saveOptions",
            param_description = "options for saving"
        )]
        save_options: String,
    ) -> WasmEncodedResult<bool> {
        self.update_record(record_id, new_data, save_options).into()
    }
}
