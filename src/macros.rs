/// A macro that implements main wasm traits for the given type.
/// These traits are the necessary ones to be able to send/receive
/// the given type through wasm bindgen bounry.
/// The type needs to implement [serde::Serialize], [serde::Deserialize]
/// and [tsify::Tsify].
///
/// Example:
/// ```ignore
/// #[derive(Serialize, Deserialize, Tsify)]
/// #[serde(rename_all = "camelCase")]
/// pub struct A {
///     pub field: String,
///     pub other_field: u8,
/// }
/// impl_main_wasm_traits!(A);
///
/// #[wasm_bindgen]
/// pub fn some_fn(arg: A) -> String {
///     // body
/// }
///
/// #[wasm_bindgen]
/// pub fn some_other_fn(arg: String) -> Option<A> {
///     // body
/// }
/// ```
#[macro_export]
macro_rules! impl_main_wasm_traits {
    ($type_name:path) => {
        impl $type_name {
            const TYPE_NAME: &'static str = stringify!($type_name);
        }
        impl $crate::prelude::wasm_bindgen::describe::WasmDescribe for $type_name {
            #[inline]
            fn describe() {
                <Self as $crate::prelude::Tsify>::JsType::describe()
            }
        }
        impl $crate::prelude::wasm_bindgen::convert::IntoWasmAbi for $type_name {
            type Abi = <<Self as $crate::prelude::Tsify>::JsType as $crate::prelude::wasm_bindgen::convert::IntoWasmAbi>::Abi;

            #[inline]
            fn into_abi(self) -> Self::Abi {
                let mut err = Self::TYPE_NAME.to_string();
                err.push_str(": ");
                let result = $crate::prelude::to_js_value(&self);
                $crate::prelude::UnwrapThrowExt::expect_throw(result.inspect_err(|e| err.push_str(&e.to_string())), &err).into_abi()
            }
        }
        impl $crate::prelude::wasm_bindgen::convert::OptionIntoWasmAbi for $type_name {
            #[inline]
            fn none() -> Self::Abi {
                <<Self as $crate::prelude::Tsify>::JsType as $crate::prelude::wasm_bindgen::convert::OptionIntoWasmAbi>::none()
            }
        }
        impl $crate::prelude::wasm_bindgen::convert::FromWasmAbi for $type_name {
            type Abi = <<Self as $crate::prelude::Tsify>::JsType as $crate::prelude::wasm_bindgen::convert::FromWasmAbi>::Abi;

            #[inline]
            unsafe fn from_abi(js: Self::Abi) -> Self {
                let mut err = Self::TYPE_NAME.to_string();
                err.push_str(": ");
                let result = $crate::prelude::from_js_value(js.into());
                $crate::prelude::UnwrapThrowExt::expect_throw(result.inspect_err(|e| err.push_str(&e.to_string())), &err)
            }
        }
        impl $crate::prelude::wasm_bindgen::convert::OptionFromWasmAbi for $type_name {
            #[inline]
            fn is_none(js: &Self::Abi) -> bool {
                <<Self as $crate::prelude::Tsify>::JsType as $crate::prelude::wasm_bindgen::convert::OptionFromWasmAbi>::is_none(js)
            }
        }
    };
}

/// Implements complementary wasm traits for the given type.
/// Needs [impl_main_wasm_traits] to be implemented first.
/// It allows a type to be used on async functions normally or
/// as ref or as Vec<> etc.
/// The type needs to implement [serde::Serialize], [serde::Deserialize]
/// and [tsify::Tsify].
///
/// Example:
/// ```ignore
/// #[derive(Serialize, Deserialize, Tsify)]
/// #[serde(rename_all = "camelCase")]
/// pub struct A {
///     pub field: String,
///     pub other_field: u8,
/// }
/// impl_main_wasm_traits!(A);
/// impl_complementary_wasm_traits!(A);
///
/// #[wasm_bindgen]
/// pub async fn some_fn(arg: &A) -> Result<String, Error> {
///     // body
/// }
/// ```
#[macro_export]
macro_rules! impl_complementary_wasm_traits {
    ($type_name:path) => {
        impl $crate::prelude::wasm_bindgen::convert::RefFromWasmAbi for $type_name {
            type Abi = <$crate::prelude::wasm_bindgen::JsValue as $crate::prelude::wasm_bindgen::convert::RefFromWasmAbi>::Abi;
            type Anchor = Box<$type_name>;
            unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                Box::new(<$type_name as $crate::prelude::wasm_bindgen::convert::FromWasmAbi>::from_abi(js))
            }
        }
        impl $crate::prelude::wasm_bindgen::convert::LongRefFromWasmAbi for $type_name {
            type Abi = <$crate::prelude::wasm_bindgen::JsValue as $crate::prelude::wasm_bindgen::convert::RefFromWasmAbi>::Abi;
            type Anchor = Box<$type_name>;
            unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                Box::new(<$type_name as $crate::prelude::wasm_bindgen::convert::FromWasmAbi>::from_abi(js))
            }
        }
        impl $crate::prelude::wasm_bindgen::convert::VectorIntoWasmAbi for $type_name {
            type Abi = <Box<[$crate::prelude::wasm_bindgen::JsValue]> as $crate::prelude::wasm_bindgen::convert::IntoWasmAbi>::Abi;
            fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
                $crate::prelude::wasm_bindgen::convert::js_value_vector_into_abi(vector)
            }
        }
        impl $crate::prelude::wasm_bindgen::convert::VectorFromWasmAbi for $type_name {
            type Abi = <Box<[$crate::prelude::wasm_bindgen::JsValue]> as $crate::prelude::wasm_bindgen::convert::IntoWasmAbi>::Abi;
            unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
                $crate::prelude::wasm_bindgen::convert::js_value_vector_from_abi(js)
            }
        }
        impl $crate::prelude::wasm_bindgen::describe::WasmDescribeVector for $type_name {
            fn describe_vector() {
                $crate::prelude::wasm_bindgen::describe::inform($crate::prelude::wasm_bindgen::describe::VECTOR);
                <$type_name as $crate::prelude::wasm_bindgen::describe::WasmDescribe>::describe();
            }
        }
        impl From<$type_name> for $crate::prelude::wasm_bindgen::JsValue {
            fn from(value: $type_name) -> Self {
                let mut err = <$type_name>::TYPE_NAME.to_string();
                err.push_str(": ");
                let result = $crate::prelude::to_js_value(&value);
                $crate::prelude::UnwrapThrowExt::expect_throw(
                    result.inspect_err(|e| err.push_str(&e.to_string())),
                    &err,
                )
            }
        }
        impl $crate::prelude::wasm_bindgen::convert::TryFromJsValue for $type_name {
            type Error = $crate::prelude::serde_wasm_bindgen::Error;
            fn try_from_js_value(value: $crate::prelude::wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
                $crate::prelude::from_js_value(value)
            }
        }
    };
}

/// Implement all wasm traits for the given type.
/// that is [impl_main_wasm_traits] and [impl_complementary_wasm_traits].
/// The type needs to implement [serde::Serialize], [serde::Deserialize]
/// and [tsify::Tsify].
///
/// Example:
/// ```ignore
/// #[derive(Serialize, Deserialize, Tsify)]
/// #[serde(rename_all = "camelCase")]
/// pub struct A {
///     pub field: String,
///     pub other_field: u8,
/// }
/// impl_all_wasm_traits!(A);
///
/// #[wasm_bindgen]
/// pub fn some_fn(arg: Vec<A>) -> String {
///     // body
/// }
/// ```
#[macro_export]
macro_rules! impl_all_wasm_traits {
    ($type_name:path) => {
        $crate::impl_main_wasm_traits!($type_name);
        $crate::impl_complementary_wasm_traits!($type_name);
    };
}

/// Implements [tsify::Tsify] with the given type declaration for the given rust
/// type(struct, enum, type, ...) identifier.
/// This is the same as what [tsify::Tsify] derive macro does internally for a
/// given type but with full customization capability, as both are a sugar
/// for [wasm_bindgen] `typescript_custom_section` attr plus `extern C` block
/// defining a wrapped [wasm_bindgen::JsValue] for the given type.
/// Therefore, this macro (unlike tsify derive macro) puts representative
/// [wasm_bindgen::JsValue] of the given type on the current scope identified
/// by prepending "Js" to the orginial type identifier, meaning it would be
/// accessible by for example:
/// `JsSomeType` when original type is `SomeType`.
/// This allows to manually serialize/deserialize the [wasm_bindgen::JsValue]
/// to/from js side from/to the rust type, for example with custom serializers
/// and deserializers.
///
/// Example:
/// ```ignore
/// #[derive(Serialize, Deserialize)]
/// #[serde(rename_all = "camelCase")]
/// pub struct SomeType {
///     pub field: String,
///     pub other_field: u8,
/// }
/// impl_custom_tsify!(
///     SomeType,
///     // this will become the typescript
///     // interface bindings for SomeType
///     "export interface SomeType {
///         field: string;
///         otherField: number
///     };"
/// );
///
/// #[wasm_bindgen]
/// pub fn some_fn(arg: JsSomeType) -> JsSomeType {
///     // deserialize the arg which is a wrapped `JsValue`
///     // into rust `SomeType` using serde_wasm_bindgen
///     let val = serde_wasm_bindgen::from_value::<SomeType>(arg.obj).unwrap_throw();
///
///     // body
///
///     // serialize to JsValue optionally with serializer available
///     // options and wrap it in JsSomeType for return
///     let ser = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
///     JsSomeType { obj: val.serialize(ser).unwrap_throw() }
/// }
/// ```
#[macro_export]
macro_rules! impl_custom_tsify {
    ($type_name:ident, $decl:literal) => {
        $crate::prelude::paste::paste! {
            #[$crate::prelude::wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(typescript_type = [<$type_name>])]
                pub type [<Js $type_name>];
            }

            #[$crate::prelude::wasm_bindgen(typescript_custom_section)]
            const TYPESCRIPT_CONTENT: &'static str = $decl;

            impl $crate::prelude::Tsify for $type_name {
                type JsType = [<Js $type_name>];
                const DECL: &'static str = $decl;
            }
        }
    };
}

/// Adds/appends the given string literal to wasm bindgen typescript bindings.
/// This is just a sugar for [wasm_bindgen] `typescript_custom_section`, so
/// the given text can be anything, from typescript comment to type declarations
/// or any other valid .d.ts content.
///
/// Example:
/// ```ignore
/// // add some custom type to .d.ts bindings output
/// add_ts_content!("export type SomeType = { field: string; otherField: number };");
///
/// // add some comment to .d.ts bindings output
/// add_ts_content!("// this is some comment");
/// ```
#[macro_export]
macro_rules! add_ts_content {
    ($decl:literal) => {
        $crate::prelude::paste::paste! {
            #[$crate::prelude::wasm_bindgen(typescript_custom_section)]
            const TYPESCRIPT_CONTENT: &'static str = $decl;
        }
    };
}

#[cfg(target_family = "wasm")]
#[cfg(test)]
mod tests {
    use crate::*;
    use js_sys::JsString;
    use wasm_bindgen_test::wasm_bindgen_test;
    use std::{collections::HashMap, str::FromStr};

    #[derive(serde::Deserialize, serde::Serialize, Default)]
    pub struct A {
        pub field1: String,
        #[serde(serialize_with = "bytes_serilializer")]
        pub field2: Vec<u8>,
        #[serde(serialize_with = "serialize_hashmap_as_object")]
        pub field3: HashMap<String, u64>,
    }

    // ensures macros validity at compile time
    // impl tsify manualy for "A" that needs it
    // before being able to impl all wasm traits
    impl_custom_tsify!(
        A,
        "export interface A {
            field1: String;
            field2: Uint8Array;
            field3: Record<string, bigint>;
        };"
    );
    impl_all_wasm_traits!(A);
    add_ts_content!("export type SomeType = string;");

    #[wasm_bindgen_test]
    fn test_macros() {
        let res = serde_wasm_bindgen::to_value(&A::default()).unwrap();

        // should exist
        assert!(JsString::from_str("field1").unwrap().js_in(&res));
        assert!(JsString::from_str("field2").unwrap().js_in(&res));
        assert!(JsString::from_str("field3").unwrap().js_in(&res));

        // should not exist
        assert!(!JsString::from_str("field4").unwrap().js_in(&res));
    }
}
