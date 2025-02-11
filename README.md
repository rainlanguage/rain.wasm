# wasm-bindgen-utils

Provides utilities, helpers and macros to easily build and customize `wasm_bindgen` bindings. For more details please read the doumentation of the items of this lib.

Example:
```rust
use wasm_bindgen_utils::{prelude::*, impl_wasm_traits, impl_custom_tsify, add_ts_content};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SomeType {
    #[cfg_attr(target_family = "wasm", serde(serialize_with = "serialize_as_bytes"))]
    pub field: Vec<u8>,
    #[cfg_attr(target_family = "wasm", serde(serialize_with = "serialize_hashmap_as_object"))]
    pub other_field: HashMap<String, u8>,
}

// impl wasm traits for SomeType
impl_wasm_traits!(SomeType);

// impl tsify manually for SomeType (as an alternative to Tsify derive macro)
// the given string literal will become the typescript interface bindings for SomeType
impl_custom_tsify!(
    SomeType,
    "export interface SomeType {
        field: Uint8Array;
        otherField: Record<string, number>;
    }"
);

// appends a custom section to the .d.ts generated bindings
add_ts_content!("import { Something } from 'some-js-lib'")

// now someType can be used on functions and methods natively
#[wasm_bindgen]
pub fn some_fn(arg: SomeType) -> String {
    // body
}

#[wasm_bindgen]
pub async fn some_other_fn(arg1: Vec<u8>, arg2: HashMap<String, u8>) -> Result<SomeType, Error> {
    // body
    Ok(SomeType {
        field: arg1,
        other_field: arg2
    })
}
```
