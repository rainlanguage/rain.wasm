use tsify::Tsify;
use crate::impl_wasm_traits;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WasmEncodedError {
    pub msg: String,
    pub readable_msg: String,
}
impl_wasm_traits!(WasmEncodedError);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct WasmEncodedResult<T> {
    pub value: Option<T>,
    pub error: Option<WasmEncodedError>,
}
impl_wasm_traits!(WasmEncodedResult<T>);

impl<T> WasmEncodedResult<T> {
    pub fn success(value: T) -> Self {
        WasmEncodedResult {
            value: Some(value),
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
