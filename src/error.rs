use serde::Serialize;
use std::error::Error;
use crate::{prelude::*, TypedResult};
use std::fmt::{self, Display, Formatter};

/// A generic error type that holds a rust error instance as well as its human readable msg.
/// Main purpose of this struct is to be used with combination of [TypedResult] as return type
/// for wasm_bindgen function bindings.
///
/// Sine rust errors are not generally deserialize friendly as their inner types usually
/// consist of external crates' error types, and as we dont really need for errors that are
/// sent to js side to be deserializable back into rust again, we only need to implement
/// `into wasm abi` traits for this struct, which enables us to natively use on a function's
/// return that is exposed to js by wasm_bindgen macro.
/// So the only constraint on the generic is that it should implement [Serialize] and
/// [Error] traits.
#[derive(Debug, Serialize, Tsify)]
pub struct TypedError<E: Error> {
    pub typ: E,
    pub msg: String,
}

impl<E: Error> From<E> for TypedError<E> {
    fn from(value: E) -> Self {
        TypedError {
            msg: value.to_string(),
            typ: value,
        }
    }
}
impl<T, E: Error> From<TypedError<E>> for TypedResult<T, TypedError<E>> {
    fn from(value: TypedError<E>) -> Self {
        TypedResult {
            value: None,
            error: Some(value),
        }
    }
}

// impl Error trait for TypedError
impl<E: Error> Display for TypedError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl<E: Error> Error for TypedError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.typ.source()
    }
}

// impl only into_wasm_abi traits for TypedError
impl<E: Serialize + Error> TypedError<E> {
    const TYPE_NAME: &'static str = "TypedJsError";
    /// A simple helpful warpper for serde_wasm_bindgen::to_value
    /// as self method for easy accessible conversion
    pub fn try_into_js_value(&self) -> Result<JsValue, serde_wasm_bindgen::Error> {
        to_js_value(&self)
    }
}
impl<E: Serialize + Error> wasm_bindgen::describe::WasmDescribe for TypedError<E> {
    #[inline]
    fn describe() {
        <Self as Tsify>::JsType::describe()
    }
}
impl<E: Serialize + Error> wasm_bindgen::convert::IntoWasmAbi for TypedError<E> {
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
impl<E: Serialize + Error> wasm_bindgen::convert::OptionIntoWasmAbi for TypedError<E> {
    #[inline]
    fn none() -> Self::Abi {
        <<Self as Tsify>::JsType as wasm_bindgen::convert::OptionIntoWasmAbi>::none()
    }
}
impl<E: Serialize + Error> wasm_bindgen::convert::VectorIntoWasmAbi for TypedError<E> {
    type Abi = <Box<[<Self as Tsify>::JsType]> as wasm_bindgen::convert::IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        wasm_bindgen::convert::js_value_vector_into_abi(vector)
    }
}
impl<E: Serialize + Error> wasm_bindgen::describe::WasmDescribeVector for TypedError<E> {
    fn describe_vector() {
        wasm_bindgen::describe::inform(wasm_bindgen::describe::VECTOR);
        <Self as wasm_bindgen::describe::WasmDescribe>::describe();
    }
}
impl<E: Serialize + Error> From<TypedError<E>> for JsValue {
    fn from(value: TypedError<E>) -> Self {
        let mut err = String::new();
        err.push_str(<TypedError<E>>::TYPE_NAME);
        err.push_str(": ");
        let result = value.try_into_js_value();
        UnwrapThrowExt::expect_throw(result.inspect_err(|e| err.push_str(&e.to_string())), &err)
    }
}
impl<E: Serialize + Error> wasm_bindgen::__rt::VectorIntoJsValue for TypedError<E> {
    fn vector_into_jsvalue(vector: Box<[Self]>) -> JsValue {
        wasm_bindgen::__rt::js_value_vector_into_jsvalue(vector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_from_any_error() {
        let err = TestError::A;

        let result = <TypedError<TestError>>::from(err);
        assert!(result.msg == "A");
        assert!(matches!(result.typ, TestError::A));
    }

    #[test]
    fn test_from_typederr_for_typedres() {
        let err = TypedError::from(TestError::A);
        let result = <TypedResult<String, TypedError<TestError>>>::from(err);
        assert!(result.value.is_none());
        assert!(result
            .error
            .is_some_and(|v| matches!(v.typ, TestError::A) && v.msg == "A"));
    }
}
