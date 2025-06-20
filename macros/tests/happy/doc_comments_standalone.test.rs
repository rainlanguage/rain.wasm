#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct TestStruct;

/// This function calculates the sum of two numbers
/// It's a simple addition operation
#[wasm_export]
pub fn add(a: u32, b: u32) -> Result<u32, Error> {
    Ok(a + b)
}

/// Multiply two numbers together
/// 
/// # Arguments
/// * `x` - The first number
/// * `y` - The second number
/// 
/// # Returns
/// The product of x and y
#[wasm_export(js_name = "multiply")]
pub fn mul(x: u32, y: u32) -> Result<u32, Error> {
    Ok(x * y)
}

/// Creates a new TestStruct instance
/// 
/// This is a more complex example with multiple
/// lines of documentation that should be preserved
/// in the generated wasm_bindgen function.
#[wasm_export(return_description = "new TestStruct instance")]
pub fn create_test_struct() -> Result<TestStruct, Error> {
    Ok(TestStruct)
}

/// Async function with doc comments
/// This function demonstrates that doc comments work with async functions too
#[wasm_export(preserve_js_class)]
pub async fn async_with_docs(input: String) -> Result<TestStruct, Error> {
    Ok(TestStruct)
}