#[macro_use]
extern crate wasm_bindgen_utils_macros;
struct Calculator {
    value: u32,
}
impl Calculator {
    /// Creates a new Calculator with the given initial value
    ///
    /// # Arguments
    /// * `initial` - The starting value for the calculator
    pub fn new(initial: u32) -> Result<Calculator, Error> {
        Ok(Calculator { value: initial })
    }
    /// Adds a value to the current calculator value
    /// Returns a new Calculator instance with the updated value
    pub fn add(&self, other: u32) -> Result<Calculator, Error> {
        Ok(Calculator {
            value: self.value + other,
        })
    }
    /// Gets the current value of the calculator
    ///
    /// This method returns the internal value as a u32.
    /// The value represents the current state of the calculator.
    pub fn get_value(&self) -> Result<u32, Error> {
        Ok(self.value)
    }
    /// Complex calculation method with detailed documentation
    ///
    /// This method performs a complex calculation involving:
    /// - Multiplication by the input factor
    /// - Addition of a constant offset
    /// - Modulo operation for bounds checking
    ///
    /// # Arguments
    /// * `factor` - The multiplication factor
    /// * `offset` - The offset to add
    ///
    /// # Returns
    /// The result of the complex calculation
    pub fn complex_calc(&self, factor: u32, offset: u32) -> Result<u32, Error> {
        Ok(((self.value * factor) + offset) % 1000)
    }
}
#[wasm_bindgen]
impl Calculator {
    /// Creates a new Calculator with the given initial value
    ///
    /// # Arguments
    /// * `initial` - The starting value for the calculator
    #[allow(non_snake_case)]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<Calculator>")]
    pub fn new__wasm_export(initial: u32) -> WasmEncodedResult<Calculator> {
        Self::new(initial).into()
    }
    /// Adds a value to the current calculator value
    /// Returns a new Calculator instance with the updated value
    #[allow(non_snake_case)]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<Calculator>")]
    pub fn add__wasm_export(&self, other: u32) -> JsValue {
        use js_sys::{Reflect, Object};
        let obj = Object::new();
        let result = self.add(other).into();
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
    /// Gets the current value of the calculator
    ///
    /// This method returns the internal value as a u32.
    /// The value represents the current state of the calculator.
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        js_name = "getValue",
        unchecked_return_type = "WasmEncodedResult<u32>"
    )]
    pub fn get_value__wasm_export(&self) -> WasmEncodedResult<u32> {
        self.get_value().into()
    }
    /// Complex calculation method with detailed documentation
    ///
    /// This method performs a complex calculation involving:
    /// - Multiplication by the input factor
    /// - Addition of a constant offset
    /// - Modulo operation for bounds checking
    ///
    /// # Arguments
    /// * `factor` - The multiplication factor
    /// * `offset` - The offset to add
    ///
    /// # Returns
    /// The result of the complex calculation
    #[allow(non_snake_case)]
    #[wasm_bindgen(
        unchecked_return_type = "WasmEncodedResult<u32>",
        return_description = "result of complex calculation"
    )]
    pub fn complex_calc__wasm_export(
        &self,
        factor: u32,
        offset: u32,
    ) -> WasmEncodedResult<u32> {
        self.complex_calc(factor, offset).into()
    }
}
