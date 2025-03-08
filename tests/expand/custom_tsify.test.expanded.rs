#[macro_use]
extern crate wasm_bindgen_utils;
pub struct A {
    pub field1: String,
    pub field2: Vec<u8>,
    pub field3: HashMap<String, u64>,
}
#[automatically_derived]
#[repr(transparent)]
pub struct JsA {
    obj: wasm_bindgen::JsValue,
}
#[automatically_derived]
const _: () = {
    use wasm_bindgen::convert::TryFromJsValue;
    use wasm_bindgen::convert::{IntoWasmAbi, FromWasmAbi};
    use wasm_bindgen::convert::{OptionIntoWasmAbi, OptionFromWasmAbi};
    use wasm_bindgen::convert::{RefFromWasmAbi, LongRefFromWasmAbi};
    use wasm_bindgen::describe::WasmDescribe;
    use wasm_bindgen::{JsValue, JsCast, JsObject};
    use wasm_bindgen::__rt::core;
    #[automatically_derived]
    impl WasmDescribe for JsA {
        fn describe() {
            use wasm_bindgen::describe::*;
            inform(NAMED_EXTERNREF);
            inform(1u32);
            inform(65u32);
        }
    }
    #[automatically_derived]
    impl IntoWasmAbi for JsA {
        type Abi = <JsValue as IntoWasmAbi>::Abi;
        #[inline]
        fn into_abi(self) -> Self::Abi {
            self.obj.into_abi()
        }
    }
    #[automatically_derived]
    impl OptionIntoWasmAbi for JsA {
        #[inline]
        fn none() -> Self::Abi {
            0
        }
    }
    #[automatically_derived]
    impl<'a> OptionIntoWasmAbi for &'a JsA {
        #[inline]
        fn none() -> Self::Abi {
            0
        }
    }
    #[automatically_derived]
    impl FromWasmAbi for JsA {
        type Abi = <JsValue as FromWasmAbi>::Abi;
        #[inline]
        unsafe fn from_abi(js: Self::Abi) -> Self {
            JsA {
                obj: JsValue::from_abi(js).into(),
            }
        }
    }
    #[automatically_derived]
    impl OptionFromWasmAbi for JsA {
        #[inline]
        fn is_none(abi: &Self::Abi) -> bool {
            *abi == 0
        }
    }
    #[automatically_derived]
    impl<'a> IntoWasmAbi for &'a JsA {
        type Abi = <&'a JsValue as IntoWasmAbi>::Abi;
        #[inline]
        fn into_abi(self) -> Self::Abi {
            (&self.obj).into_abi()
        }
    }
    #[automatically_derived]
    impl RefFromWasmAbi for JsA {
        type Abi = <JsValue as RefFromWasmAbi>::Abi;
        type Anchor = core::mem::ManuallyDrop<JsA>;
        #[inline]
        unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
            let tmp = <JsValue as RefFromWasmAbi>::ref_from_abi(js);
            core::mem::ManuallyDrop::new(JsA {
                obj: core::mem::ManuallyDrop::into_inner(tmp).into(),
            })
        }
    }
    #[automatically_derived]
    impl LongRefFromWasmAbi for JsA {
        type Abi = <JsValue as LongRefFromWasmAbi>::Abi;
        type Anchor = JsA;
        #[inline]
        unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
            let tmp = <JsValue as LongRefFromWasmAbi>::long_ref_from_abi(js);
            JsA { obj: tmp.into() }
        }
    }
    #[automatically_derived]
    impl From<JsValue> for JsA {
        #[inline]
        fn from(obj: JsValue) -> JsA {
            JsA { obj: obj.into() }
        }
    }
    #[automatically_derived]
    impl AsRef<JsValue> for JsA {
        #[inline]
        fn as_ref(&self) -> &JsValue {
            self.obj.as_ref()
        }
    }
    #[automatically_derived]
    impl AsRef<JsA> for JsA {
        #[inline]
        fn as_ref(&self) -> &JsA {
            self
        }
    }
    #[automatically_derived]
    impl From<JsA> for JsValue {
        #[inline]
        fn from(obj: JsA) -> JsValue {
            obj.obj.into()
        }
    }
    #[automatically_derived]
    impl JsCast for JsA {
        fn instanceof(val: &JsValue) -> bool {
            #[cfg(
                not(
                    all(
                        target_arch = "wasm32",
                        any(target_os = "unknown", target_os = "none")
                    )
                )
            )]
            unsafe fn __wbg_instanceof_JsA_0e3b7e439dec8d02(_: u32) -> u32 {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("cannot check instanceof on non-wasm targets"),
                    );
                };
            }
            unsafe {
                let idx = val.into_abi();
                __wbg_instanceof_JsA_0e3b7e439dec8d02(idx) != 0
            }
        }
        #[inline]
        fn unchecked_from_js(val: JsValue) -> Self {
            JsA { obj: val.into() }
        }
        #[inline]
        fn unchecked_from_js_ref(val: &JsValue) -> &Self {
            unsafe { &*(val as *const JsValue as *const JsA) }
        }
    }
    impl JsObject for JsA {}
};
#[automatically_derived]
impl core::ops::Deref for JsA {
    type Target = wasm_bindgen::JsValue;
    #[inline]
    fn deref(&self) -> &wasm_bindgen::JsValue {
        &self.obj
    }
}
impl ::wasm_bindgen_utils::prelude::Tsify for A {
    type JsType = JsA;
    const DECL: &'static str = "export interface A {
        field1: String;
        field2: Uint8Array;
        field3: Record<string, bigint>;
    };";
}
