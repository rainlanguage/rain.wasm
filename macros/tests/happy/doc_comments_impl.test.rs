#[macro_use]
extern crate wasm_bindgen_utils_macros;

struct Calculator {
    value: u32,
}

#[wasm_export]
impl Calculator {
    /// Creates a new Calculator with the given initial value
    /// 
    /// # Arguments
    /// * `initial` - The starting value for the calculator
    #[wasm_export]
    pub fn new(initial: u32) -> Result<Calculator, Error> {
        Ok(Calculator { value: initial })
    }

    /// Adds a value to the current calculator value
    /// Returns a new Calculator instance with the updated value
    #[wasm_export(preserve_js_class)]
    pub fn add(&self, other: u32) -> Result<Calculator, Error> {
        Ok(Calculator { value: self.value + other })
    }

    /// Gets the current value of the calculator
    /// 
    /// This method returns the internal value as a u32.
    /// The value represents the current state of the calculator.
    #[wasm_export(js_name = "getValue")]
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
    #[wasm_export(return_description = "result of complex calculation")]
    pub fn complex_calc(&self, factor: u32, offset: u32) -> Result<u32, Error> {
        Ok(((self.value * factor) + offset) % 1000)
    }
}