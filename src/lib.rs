//! Provides utilities, helpers and macros to easily build and customize [wasm_bindgen] bindings,
//! such as [impl_wasm_traits] macro that will implement wasm traits for a give type and
//! [serialize_hashmap_as_object] serializer function to serialize a hashmap as object used with
//! serde `serialize_with` attribute.
//! For more details please read the doumentation of the items of this lib.
//!
//! Example:
//! ```ignore
//! #[derive(Serialize, Deserialize)]
//! #[serde(rename_all = "camelCase")]
//! pub struct SomeType {
//!     #[cfg_attr(target_family = "wasm", serde(serialize_with = "bytes_serilializer"))]
//!     pub field: Vec<u8>,
//!     #[cfg_attr(target_family = "wasm", serde(serialize_with = "serialize_hashmap_as_object"))]
//!     pub other_field: HashMap<String, u8>,
//! }
//!
//! // impl wasm traits for SomeType
//! impl_wasm_traits!(SomeType);
//!
//! // impl tsify manually for SomeType (as an alternative to Tsify derive macro)
//! // the given string literal will become the typescript interface bindings for SomeType
//! impl_custom_tsify!(
//!     SomeType,
//!     "export interface SomeType {
//!         field: Uint8Array;
//!         otherField: Record<string, number>;
//!     }"
//! );
//!
//! // appends a custom section to the .d.ts generated bindings
//! add_ts_content!("import { Something } from 'some-js-lib'")
//!
//! // now someType can be used on functions and methods natively
//! #[wasm_bindgen]
//! pub fn some_fn(arg: SomeType) -> String {
//!     // body
//! }
//!
//! #[wasm_bindgen]
//! pub async fn some_other_fn(arg1: Vec<u8>, arg2: HashMap<String, u8>) -> Result<SomeType, Error> {
//!     // body
//!     Ok(SomeType {
//!         field: arg1,
//!         other_field: arg2
//!     })
//! }
//! ```

mod ser;
pub mod macros;

pub use ser::*;

// prelude exports
pub mod prelude {
    pub use paste;
    pub use js_sys;
    pub use tsify_next;
    pub use wasm_bindgen;
    pub use serde_wasm_bindgen;
    pub use wasm_bindgen_futures;
    pub use tsify_next::Tsify;
    pub use wasm_bindgen::prelude::*;
    pub use serde_wasm_bindgen::{to_value as to_js_value, from_value as from_js_value};
}
