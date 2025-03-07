use tsify::Tsify;
use crate::impl_wasm_traits;
use serde::{Serialize, Deserialize};

/// A struct that holds info of a rust error that is serializable
/// natively to js through wasm bindgen, so [Result::Err] variants
/// of binding functions can return normaly in js instead of throwing.
///
/// Rust errors should impl [Into]/[From] trait to this struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WasmEncodedError {
    pub msg: String,
    pub readable_msg: String,
}
impl_wasm_traits!(WasmEncodedError);

/// A generic struct that either holds info of a rust [Result] that
/// is serializable natively to js through wasm bindgen, so binding
/// functions can return normaly in js instead of throwing by holding
/// either [Result::Ok] variant in its "value" prop or [Result::Err]
/// in its "error" prop.
/// Used in [wasm_bindgen_utils_macros::wasm_export] as the returning
/// type of exporting wasm binding functions.
///
/// [From] trait has been implemented for any [Result<T, E>] where
/// "E" implements [Into<WasmEncodedError>].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct WasmEncodedResult<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub value: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub error: Option<WasmEncodedError>,
}
impl_wasm_traits!(WasmEncodedResult<T>);

impl<T> WasmEncodedResult<T> {
    /// Creates a success instance from the given type
    pub fn success(value: T) -> Self {
        WasmEncodedResult {
            value: Some(value),
            error: None,
        }
    }
    /// Creates an error instance from the given WasmEncodedError
    pub fn error(err: WasmEncodedError) -> Self {
        WasmEncodedResult {
            value: None,
            error: Some(err),
        }
    }
}

impl<T, E: Into<WasmEncodedError>> From<Result<T, E>> for WasmEncodedResult<T> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => WasmEncodedResult {
                value: Some(value),
                error: None,
            },
            Err(err) => WasmEncodedResult {
                value: None,
                error: Some(err.into()),
            },
        }
    }
}
