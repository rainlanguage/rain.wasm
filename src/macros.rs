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

/// Implements tsify with the given type declaration as literal, optionally
/// for the given rust type(struct, enum, type, ...) identifier.
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
    ($decl:literal) => {
        $crate::prelude::paste! {
            #[$crate::prelude::wasm_bindgen(typescript_custom_section)]
            const TYPESCRIPT_CONTENT: &'static str = $decl;
        }
    };
}
