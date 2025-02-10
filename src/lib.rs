//! Provides utilities, helpers and macros to easily build and customize [wasm_bindgen] bindings.

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
