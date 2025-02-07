mod helpers;
pub mod macros;

pub use helpers::*;

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
    pub use crate::helpers::*;
    pub use serde_wasm_bindgen::{from_value, to_value};
    pub use tsify::Tsify;
    pub use wasm_bindgen::{
        convert::*,
        prelude::*,
        JsCast, JsValue, UnwrapThrowExt,
        describe::{inform, WasmDescribe, WasmDescribeVector, VECTOR},
    };
}
