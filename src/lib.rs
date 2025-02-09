//! Provides utilities, helpers and macros to easily build and customize [wasm_bindgen] bindings.
//!
//! ## Features
//! - `alloy`: enabled by default, adds [mod@alloy_primitives] dependency and provides
//!   a trait implementation for converting [alloy_primitives::U256] to [js_sys::BigInt]

mod ser;
pub mod macros;
#[cfg(feature = "alloy")]
pub mod alloy;

pub use ser::*;

// re-export wasm deps for version control on target
pub use tsify;
pub use js_sys;
pub use wasm_bindgen;
pub use serde_wasm_bindgen;
pub use wasm_bindgen_futures;

// prelude exports
pub mod prelude {
    pub use js_sys;
    pub use paste::paste;
    pub use crate::ser::*;
    pub use serde_wasm_bindgen::{from_value, to_value};
    pub use tsify::Tsify;
    pub use wasm_bindgen::{
        convert::*,
        prelude::*,
        JsCast, JsValue, UnwrapThrowExt,
        describe::{inform, WasmDescribe, WasmDescribeVector, VECTOR},
    };
}
