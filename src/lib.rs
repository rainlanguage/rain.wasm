//! Provides utilities, helpers and macros to easily build and customize [wasm_bindgen] bindings.

mod ser;
pub mod macros;

pub use ser::*;

// re-export wasm deps for version control on target
pub use paste;
pub use tsify;
pub use js_sys;
pub use wasm_bindgen;
pub use serde_wasm_bindgen;
pub use wasm_bindgen_futures;

// prelude exports
pub mod prelude {
    pub use paste;
    pub use tsify;
    pub use js_sys;
    pub use wasm_bindgen;
    pub use serde_wasm_bindgen;
    pub use wasm_bindgen_futures;
    pub use tsify::Tsify;
    pub use wasm_bindgen::prelude::*;
    pub use serde_wasm_bindgen::{to_value as to_js_value, from_value as from_js_value};
}
