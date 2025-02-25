use serde::Serialize;
use std::error::Error;
use crate::{prelude::*, TypedError};

/// A generic return type that can either contain value of the a given type or an error.
/// Main purpose of this struct is to be used instead of rust `Result` for returned data
/// from wasm_bindgen function bindings, so instead of throwing in js for an error, it
/// will return this struct with a filled error property and vice versa for resolved value.
///
/// Sine rust errors are not generally deserialize friendly as their inner types usually
/// consist of external crates' error types, and as we dont really need for errors that are
/// sent to js side to be deserializable back into rust again, we only need to implement
/// `into wasm abi` traits for this struct, which enables us to natively use on a function's
/// return that is exposed to js by wasm_bindgen macro.
/// So the only constraint on generics is that both should implement [Serialize], and error
/// generic `E` should implement rust [Error] trait as well.
#[derive(Debug, Serialize, Tsify)]
pub struct TypedResult<T, E: Error> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub value: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub error: Option<E>,
}

impl<T, E: Error> From<Result<T, E>> for TypedResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => TypedResult {
                value: Some(v),
                error: None,
            },
            Err(e) => TypedResult {
                value: None,
                error: Some(e),
            },
        }
    }
}
impl<T, E: Error> From<Result<T, E>> for TypedResult<T, TypedError<E>> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => TypedResult {
                value: Some(v),
                error: None,
            },
            Err(e) => TypedResult {
                value: None,
                error: Some(e.into()),
            },
        }
    }
}

// impl only into_wasm_abi traits for TypedResult
impl<T: Serialize, E: Serialize + Error> TypedResult<T, E> {
    const TYPE_NAME: &'static str = "TypedResult";
    /// A simple helpful warpper for serde_wasm_bindgen::to_value
    /// as self method for easy accessible conversion
    pub fn try_into_js_value(&self) -> Result<JsValue, serde_wasm_bindgen::Error> {
        to_js_value(&self)
    }
}
impl<T: Serialize, E: Serialize + Error> wasm_bindgen::describe::WasmDescribe
    for TypedResult<T, E>
{
    #[inline]
    fn describe() {
        <Self as Tsify>::JsType::describe()
    }
}
impl<T: Serialize, E: Serialize + Error> wasm_bindgen::convert::IntoWasmAbi for TypedResult<T, E> {
    type Abi = <<Self as Tsify>::JsType as wasm_bindgen::convert::IntoWasmAbi>::Abi;

    #[inline]
    fn into_abi(self) -> Self::Abi {
        let mut err = String::new();
        err.push_str(Self::TYPE_NAME);
        err.push_str(": ");
        let result = self
            .try_into_js_value()
            .map(<<Self as Tsify>::JsType as JsCast>::unchecked_from_js);
        UnwrapThrowExt::expect_throw(result.inspect_err(|e| err.push_str(&e.to_string())), &err)
            .into_abi()
    }
}
impl<T: Serialize, E: Serialize + Error> wasm_bindgen::convert::OptionIntoWasmAbi
    for TypedResult<T, E>
{
    #[inline]
    fn none() -> Self::Abi {
        <<Self as Tsify>::JsType as wasm_bindgen::convert::OptionIntoWasmAbi>::none()
    }
}
impl<T: Serialize, E: Serialize + Error> wasm_bindgen::convert::VectorIntoWasmAbi
    for TypedResult<T, E>
{
    type Abi = <Box<[<Self as Tsify>::JsType]> as wasm_bindgen::convert::IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        wasm_bindgen::convert::js_value_vector_into_abi(vector)
    }
}
impl<T: Serialize, E: Serialize + Error> wasm_bindgen::describe::WasmDescribeVector
    for TypedResult<T, E>
{
    fn describe_vector() {
        wasm_bindgen::describe::inform(wasm_bindgen::describe::VECTOR);
        <Self as wasm_bindgen::describe::WasmDescribe>::describe();
    }
}
impl<T: Serialize, E: Serialize + Error> From<TypedResult<T, E>> for JsValue {
    fn from(value: TypedResult<T, E>) -> Self {
        let mut err = String::new();
        err.push_str(<TypedResult<T, E>>::TYPE_NAME);
        err.push_str(": ");
        let result = value.try_into_js_value();
        UnwrapThrowExt::expect_throw(result.inspect_err(|e| err.push_str(&e.to_string())), &err)
    }
}
impl<T: Serialize, E: Serialize + Error> wasm_bindgen::__rt::VectorIntoJsValue
    for TypedResult<T, E>
{
    fn vector_into_jsvalue(vector: Box<[Self]>) -> JsValue {
        wasm_bindgen::__rt::js_value_vector_into_jsvalue(vector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fmt::{self, Display, Formatter};

    #[derive(Debug)]
    enum TestError {
        A,
    }
    impl Display for TestError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "A")
        }
    }
    impl Error for TestError {}

    #[test]
    fn test_from_result() {
        let err_res = Err(TestError::A);
        let ok_res = Ok("something".to_string());

        let result = <TypedResult<String, TestError>>::from(ok_res);
        assert!(result.error.is_none());
        assert!(result.value.is_some_and(|v| v == "something"));

        let result = <TypedResult<String, TestError>>::from(err_res);
        assert!(result.value.is_none());
        assert!(result.error.is_some_and(|v| matches!(v, TestError::A)));

        let err_res = Err(TypedError::from(TestError::A));
        let ok_res: Result<String, TypedError<TestError>> = Ok("something".to_string());

        let result = <TypedResult<String, TypedError<TestError>>>::from(ok_res);
        assert!(result.error.is_none());
        assert!(result.value.is_some_and(|v| v == "something"));

        let result = <TypedResult<String, TypedError<TestError>>>::from(err_res);
        assert!(result.value.is_none());
        assert!(result
            .error
            .is_some_and(|v| matches!(v.typ, TestError::A) && v.msg == "A"));
    }
}
