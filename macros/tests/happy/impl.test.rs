#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[some_external_macro]
#[wasm_export]
impl TestStruct {
    #[wasm_export(js_name = "someStaticMethod")]
    #[wasm_export(unchecked_return_type = "string")]
    pub async fn some_static_method((arg1, arg2): (String, u8)) -> Result<String, Error> {
        Ok(String::new())
    }

    #[wasm_export(skip)]
    pub async fn some_skip_method() -> Result<String, Error> {
        Ok(String::new())
    }

    #[wasm_export(js_name = "someSelfMethod", some_wbg_attr, some_other_wbg_attr = something)]
    pub async fn some_self_method(&self, arg: String) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }

    #[wasm_export(js_name = "someMethodPreserveClassAsync", preserve_js_class)]
    pub async fn some_method_preserve_class_async(arg: String) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }

    #[wasm_export(js_name = "someMethodPreserveClassSync", preserve_js_class)]
    pub fn some_method_preserve_class_sync(&self, arg: String) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }
}

#[wasm_export(some_top_wbg_attr = "something", some_other_wbg_attr)]
impl TestStruct {
    #[wasm_export]
    pub fn returns_num_array(&mut self) -> Result<Vec<u8>, Error> {
        Ok(Vec::new())
    }

    #[wasm_export(return_description = "gets the user's name")]
    pub fn get_name(&self) -> Result<String, Error> {
        Ok("test".to_string())
    }

    #[wasm_export(js_name = "getUserAge", return_description = "returns user age in years", catch)]
    pub fn get_age(&self, user_id: u32) -> Result<u32, Error> {
        Ok(25)
    }

    #[wasm_export(return_description = "the number at the given index")]
    pub fn number(
        &self,
        #[wasm_export(param_description = "the index of the number to be returned")]
        index: u32,
    ) -> Result<u32, Error> {
        Ok(index * 2)
    }

    #[wasm_export]
    pub fn with_unchecked_param(
        &self,
        #[wasm_export(unchecked_param_type = "CustomJSType")]
        custom_param: wasm_bindgen::JsValue,
    ) -> Result<String, Error> {
        Ok("success".to_string())
    }

    #[wasm_export(js_name = "processElement")]
    pub fn process_element(
        &mut self,
        #[wasm_export(unchecked_param_type = "HTMLElement", param_description = "the DOM element to process")]
        element: wasm_bindgen::JsValue,
        #[wasm_export(param_description = "processing options")]
        options: String,
    ) -> Result<bool, Error> {
        Ok(true)
    }

    #[wasm_export]
    pub fn with_js_name_params(
        &self,
        #[wasm_export(js_name = "primaryKey")]
        primary_key: u32,
        #[wasm_export(js_name = "displayName")]
        display_name: String,
    ) -> Result<String, Error> {
        Ok(format!("Item {}: {}", primary_key, display_name))
    }

    #[wasm_export(js_name = "updateRecord")]
    pub fn update_record(
        &mut self,
        #[wasm_export(js_name = "recordId", param_description = "unique identifier for the record")]
        record_id: u32,
        #[wasm_export(js_name = "newData", unchecked_param_type = "RecordData")]
        new_data: wasm_bindgen::JsValue,
        #[wasm_export(js_name = "saveOptions", param_description = "options for saving")]
        save_options: String,
    ) -> Result<bool, Error> {
        Ok(true)
    }
}