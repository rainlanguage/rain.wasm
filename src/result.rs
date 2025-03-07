use tsify::Tsify;
use crate::impl_wasm_traits;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WasmEncodedError {
    msg: String,
    readable_msg: String,
}
impl_wasm_traits!(WasmEncodedError);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct WasmEncodedResult<T> {
    value: Option<T>,
    error: Option<WasmEncodedError>,
}
impl_wasm_traits!(WasmEncodedResult<T>);

impl<T> WasmEncodedResult<T> {
    pub fn success(data: T) -> Self {
        WasmEncodedResult {
            value: Some(data),
            error: None,
        }
    }
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
            Ok(data) => WasmEncodedResult {
                value: Some(data),
                error: None,
            },
            Err(err) => WasmEncodedResult {
                value: None,
                error: Some(err.into()),
            },
        }
    }
}
