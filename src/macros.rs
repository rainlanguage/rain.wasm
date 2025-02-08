/// A macro that implements main wasm traits for the given type
#[macro_export]
macro_rules! impl_main_wasm_traits {
    ($type_name:path) => {
        impl $crate::prelude::WasmDescribe for $type_name {
            #[inline]
            fn describe() {
                <Self as $crate::prelude::Tsify>::JsType::describe()
            }
        }
        impl $crate::prelude::IntoWasmAbi for $type_name {
            type Abi = <<Self as $crate::prelude::Tsify>::JsType as $crate::prelude::IntoWasmAbi>::Abi;

            #[inline]
            fn into_abi(self) -> Self::Abi {
                let mut err = "".to_string();
                let result = $crate::prelude::Tsify::into_js(&self);
                $crate::prelude::UnwrapThrowExt::expect_throw(result.inspect_err(|e| err.push_str(&e.to_string())), &err).into_abi()
            }
        }
        impl $crate::prelude::OptionIntoWasmAbi for $type_name {
            #[inline]
            fn none() -> Self::Abi {
                <<Self as $crate::prelude::Tsify>::JsType as $crate::prelude::OptionIntoWasmAbi>::none()
            }
        }
        impl $crate::prelude::FromWasmAbi for $type_name {
            type Abi = <<Self as $crate::prelude::Tsify>::JsType as $crate::prelude::FromWasmAbi>::Abi;

            #[inline]
            unsafe fn from_abi(js: Self::Abi) -> Self {
                let mut err = "".to_string();
                let result = <Self as $crate::prelude::Tsify>::from_js(<Self as $crate::prelude::Tsify>::JsType::from_abi(js));
                $crate::prelude::UnwrapThrowExt::expect_throw(result.inspect_err(|e| err.push_str(&e.to_string())), &err)
            }
        }
        impl $crate::prelude::OptionFromWasmAbi for $type_name {
            #[inline]
            fn is_none(js: &Self::Abi) -> bool {
                <<Self as $crate::prelude::Tsify>::JsType as $crate::prelude::OptionFromWasmAbi>::is_none(js)
            }
        }
    };
}

/// Implements complementary wasm traits for the given type
#[macro_export]
macro_rules! impl_complementary_wasm_traits {
    ($type_name:path) => {
        impl $crate::prelude::RefFromWasmAbi for $type_name {
            type Abi = <$crate::prelude::JsValue as $crate::prelude::RefFromWasmAbi>::Abi;
            type Anchor = Box<$type_name>;
            unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                Box::new(<$type_name as $crate::prelude::FromWasmAbi>::from_abi(js))
            }
        }
        impl $crate::prelude::LongRefFromWasmAbi for $type_name {
            type Abi = <$crate::prelude::JsValue as $crate::prelude::RefFromWasmAbi>::Abi;
            type Anchor = Box<$type_name>;
            unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                Box::new(<$type_name as $crate::prelude::FromWasmAbi>::from_abi(js))
            }
        }
        impl $crate::prelude::VectorIntoWasmAbi for $type_name {
            type Abi = <Box<[$crate::prelude::JsValue]> as $crate::prelude::IntoWasmAbi>::Abi;
            fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
                $crate::prelude::js_value_vector_into_abi(vector)
            }
        }
        impl $crate::prelude::VectorFromWasmAbi for $type_name {
            type Abi = <Box<[$crate::prelude::JsValue]> as $crate::prelude::IntoWasmAbi>::Abi;
            unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
                $crate::prelude::js_value_vector_from_abi(js)
            }
        }
        impl $crate::prelude::WasmDescribeVector for $type_name {
            fn describe_vector() {
                $crate::prelude::inform($crate::prelude::VECTOR);
                <$type_name as $crate::prelude::WasmDescribe>::describe();
            }
        }
        impl From<$type_name> for $crate::prelude::JsValue {
            fn from(value: $type_name) -> Self {
                let mut err = "".to_string();
                let result = $crate::prelude::to_value(&value);
                $crate::prelude::UnwrapThrowExt::expect_throw(
                    result.inspect_err(|e| err.push_str(&e.to_string())),
                    &err,
                )
            }
        }
        impl $crate::prelude::TryFromJsValue for $type_name {
            type Error = serde_wasm_bindgen::Error;
            fn try_from_js_value(value: $crate::prelude::JsValue) -> Result<Self, Self::Error> {
                $crate::prelude::from_value(value)
            }
        }
    };
}

/// Implement all wasm traits for the given type
#[macro_export]
macro_rules! impl_all_wasm_traits {
    ($type_name:path) => {
        $crate::impl_main_wasm_traits!($type_name);
        $crate::impl_complementary_wasm_traits!($type_name);
    };
}

/// Implements tsify with the given type declaration for the given rust
/// type(struct, enum, type, ...) identifier.
/// This is the same as what [tsify::Tsify] "derive" does internally for a
/// given type but with full customization accessibility, as both are a shortcut
/// for wasm_bindgen typescript_custom_section and the latter also puts
/// representative js type of the given type on the current scope identified by
/// prepending "Js" to the orginial type identifier, meaning it would be
/// accessible by for example: "JsSomeType" when original type is "SomeType".
#[macro_export]
macro_rules! impl_custom_tsify {
    ($type_name:ident, $decl:literal) => {
        $crate::prelude::paste! {
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

/// Adds/appends the given string literal to wasm bindgen typescript bindings
/// The given text can be anything, from typescript comment to type declarations
/// or any other valid .d.ts content
#[macro_export]
macro_rules! add_ts_content {
    ($decl:literal) => {
        $crate::prelude::paste! {
            #[$crate::prelude::wasm_bindgen(typescript_custom_section)]
            const TYPESCRIPT_CONTENT: &'static str = $decl;
        }
    };
}

#[cfg(target_family = "wasm")]
#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use js_sys::{JsString, Object};
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
        let res = to_value(&A::default()).unwrap();

        // should exist
        assert!(JsString::from_str("field1").unwrap().js_in(&res));
        assert!(JsString::from_str("field2").unwrap().js_in(&res));
        assert!(JsString::from_str("field3").unwrap().js_in(&res));

        // should not exist
        assert!(!JsString::from_str("field4").unwrap().js_in(&res));
    }
}
