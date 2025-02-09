use alloy_primitives::{ruint::ParseError, U256};
use std::{error::Error, fmt::Display, str::FromStr};

/// A trait for converting a given type to U256
pub trait TryIntoU256<T: FromStr<Err = E>, E: Error>
where
    Self: Display,
{
    fn try_into_u256(&self) -> Result<T, E> {
        T::from_str(&format!("{}", &self))
    }
}
impl TryIntoU256<U256, ParseError> for js_sys::BigInt {}

#[cfg(target_family = "wasm")]
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_js_bigint_to_u256() {
        let bigint = js_sys::BigInt::from_str("12").unwrap();
        let result = bigint.try_into_u256().unwrap();
        let expected = alloy_primitives::U256::from(12);
        assert_eq!(result, expected);
    }
}
