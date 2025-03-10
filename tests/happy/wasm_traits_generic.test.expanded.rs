#[macro_use]
extern crate wasm_bindgen_utils;
pub struct SomeGenericType<T, E, K> {
    pub field: T,
    pub other_field: E,
    pub another_field: K,
}
impl<T, E, K> SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    const TYPE_NAME: &'static str = "SomeGenericType";
    /// A simple helpful wrapper for serde_wasm_bindgen::to_value
    /// as self method for easy accessible conversion
    pub fn try_into_js_value(
        &self,
    ) -> Result<
        ::wasm_bindgen_utils::prelude::JsValue,
        ::wasm_bindgen_utils::prelude::serde_wasm_bindgen::Error,
    > {
        ::wasm_bindgen_utils::prelude::to_js_value(&self)
    }
    /// A simple helpful warpper for serde_wasm_bindgen::from_value
    /// as Self method for easy accessible conversion
    pub fn try_from_js_value(
        js: ::wasm_bindgen_utils::prelude::JsValue,
    ) -> Result<Self, ::wasm_bindgen_utils::prelude::serde_wasm_bindgen::Error> {
        ::wasm_bindgen_utils::prelude::from_js_value(js)
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::describe::WasmDescribe
for SomeGenericType<T, E, K> {
    #[inline]
    fn describe() {
        <Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType::describe()
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::IntoWasmAbi
for SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Abi = <<Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType as ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::IntoWasmAbi>::Abi;
    #[inline]
    fn into_abi(self) -> Self::Abi {
        let mut err = String::new();
        err.push_str(Self::TYPE_NAME);
        err.push_str(": ");
        let result = self
            .try_into_js_value()
            .map(
                <<Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType as ::wasm_bindgen_utils::prelude::JsCast>::unchecked_from_js,
            );
        ::wasm_bindgen_utils::prelude::UnwrapThrowExt::expect_throw(
                result.inspect_err(|e| err.push_str(&e.to_string())),
                &err,
            )
            .into_abi()
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::OptionIntoWasmAbi
for SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    #[inline]
    fn none() -> Self::Abi {
        <<Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType as ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::OptionIntoWasmAbi>::none()
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::FromWasmAbi
for SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Abi = <<Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType as ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::FromWasmAbi>::Abi;
    #[inline]
    unsafe fn from_abi(js: Self::Abi) -> Self {
        let mut err = String::new();
        err.push_str(Self::TYPE_NAME);
        err.push_str(": ");
        let result = Self::try_from_js_value(
            <Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType::from_abi(js).into(),
        );
        ::wasm_bindgen_utils::prelude::UnwrapThrowExt::expect_throw(
            result.inspect_err(|e| err.push_str(&e.to_string())),
            &err,
        )
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::OptionFromWasmAbi
for SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    #[inline]
    fn is_none(js: &Self::Abi) -> bool {
        <<Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType as ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::OptionFromWasmAbi>::is_none(
            js,
        )
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::RefFromWasmAbi
for SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Abi = <<Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType as ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::RefFromWasmAbi>::Abi;
    type Anchor = Box<Self>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(
            <Self as ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::FromWasmAbi>::from_abi(
                js,
            ),
        )
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::LongRefFromWasmAbi
for SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Abi = <<Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType as ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::LongRefFromWasmAbi>::Abi;
    type Anchor = Box<Self>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(
            <Self as ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::FromWasmAbi>::from_abi(
                js,
            ),
        )
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::VectorIntoWasmAbi
for SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Abi = <Box<
        [<Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType],
    > as ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::js_value_vector_into_abi(
            vector,
        )
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::VectorFromWasmAbi
for SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Abi = <Box<
        [<Self as ::wasm_bindgen_utils::prelude::Tsify>::JsType],
    > as ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::FromWasmAbi>::Abi;
    unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
        ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::js_value_vector_from_abi(
            js,
        )
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::describe::WasmDescribeVector
for SomeGenericType<T, E, K> {
    fn describe_vector() {
        ::wasm_bindgen_utils::prelude::wasm_bindgen::describe::inform(
            ::wasm_bindgen_utils::prelude::wasm_bindgen::describe::VECTOR,
        );
        <Self as ::wasm_bindgen_utils::prelude::wasm_bindgen::describe::WasmDescribe>::describe();
    }
}
impl<T, E, K> From<SomeGenericType<T, E, K>> for ::wasm_bindgen_utils::prelude::JsValue
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn from(value: SomeGenericType<T, E, K>) -> Self {
        let mut err = String::new();
        err.push_str(<SomeGenericType<T, E, K>>::TYPE_NAME);
        err.push_str(": ");
        let result = value.try_into_js_value();
        ::wasm_bindgen_utils::prelude::UnwrapThrowExt::expect_throw(
            result.inspect_err(|e| err.push_str(&e.to_string())),
            &err,
        )
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::convert::TryFromJsValue
for SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Error = ::wasm_bindgen_utils::prelude::serde_wasm_bindgen::Error;
    fn try_from_js_value(
        value: ::wasm_bindgen_utils::prelude::JsValue,
    ) -> Result<Self, Self::Error> {
        Self::try_from_js_value(value)
    }
}
impl<T, E, K> ::wasm_bindgen_utils::prelude::wasm_bindgen::__rt::VectorIntoJsValue
for SomeGenericType<T, E, K>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    E: serde::Serialize + for<'de> serde::Deserialize<'de>,
    K: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn vector_into_jsvalue(
        vector: Box<[Self]>,
    ) -> ::wasm_bindgen_utils::prelude::JsValue {
        ::wasm_bindgen_utils::prelude::wasm_bindgen::__rt::js_value_vector_into_jsvalue(
            vector,
        )
    }
}
