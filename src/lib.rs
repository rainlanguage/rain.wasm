mod helpers;
pub mod macros;

pub use helpers::*;

// re-export deps
pub use tsify;
pub use js_sys;
pub use wasm_bindgen;
pub use serde_wasm_bindgen;
pub use wasm_bindgen_futures;

// prelude exports
pub mod prelude {
    pub use js_sys;
    pub use paste::paste;
    pub use serde_wasm_bindgen::{from_value, to_value};
    pub use tsify::Tsify;
    pub use wasm_bindgen::{
        convert::*,
        describe::{inform, WasmDescribe, WasmDescribeVector, VECTOR},
        prelude::*,
        JsCast, JsValue, UnwrapThrowExt,
    };
}
