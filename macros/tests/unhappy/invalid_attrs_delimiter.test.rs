#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

#[wasm_export]
impl TestStruct {
    #[wasm_export(skip; unchecked_return_type = "string")]
    pub async fn some_static_method1(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

#[wasm_export]
impl TestStruct {
    #[wasm_export(skip - unchecked_return_type = "string")]
    pub async fn some_static_method2(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

#[wasm_export]
impl TestStruct {
    #[wasm_export(skip. unchecked_return_type = "string")]
    pub async fn some_static_method3(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

#[wasm_export]
impl TestStruct {
    #[wasm_export(skip/ unchecked_return_type = "string")]
    pub async fn some_static_method4(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

#[wasm_export]
impl TestStruct {
    #[wasm_export(skip unchecked_return_type = "string")]
    pub async fn some_static_method5(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }
}

fn main() {}