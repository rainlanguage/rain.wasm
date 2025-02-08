use alloy_primitives::{U256, ruint::ParseError};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt::Display,
    str::FromStr,
};

/// A trait for converting a given type to U256
pub trait TryIntoU256<T: FromStr<Err = E>, E: Error>
where
    Self: Display,
{
    fn try_into_u256(&self) -> Result<T, E> {
        T::from_str(&format!("{}", &self))
    }
}
impl TryIntoU256<U256, ParseError> for js_sys::BigInt {}

/// Serializer fn for serializing Vec\<u8\> as bytes (Uint8Array for js)
/// Example:
/// ```ignore
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct A {
///     #[serde(serialize_with = "bytes_serilializer")]
///     field: Vec<u8>,
/// }
/// ```
pub fn bytes_serilializer<S: serde::Serializer>(
    val: &[u8],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(val)
}

/// Serializer fn that serializes HashMap as k/v object.
/// in js it would be plain js object and not js Map
/// Example:
/// ```ignore
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct A {
///     #[serde(serialize_with = "serialize_map_as_object")]
///     field: HashMap<String, String>,
/// }
/// ```
pub fn serialize_map_as_object<S: Serializer, T: Serialize>(
    value: &HashMap<String, T>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut ser = serializer.serialize_struct("HashMap", value.len())?;
    for (k, v) in value {
        let key: &'static str = Box::leak(k.clone().into_boxed_str());
        ser.serialize_field(key, v)?;
    }
    ser.end()
}

/// Serializer fn that serializes BTreeMap as k/v object.
/// in js it would be plain js object and not js Map
/// Example:
/// ```ignore
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct A {
///     #[serde(serialize_with = "serialize_btreemap_as_object")]
///     field: BTreeMap<String, String>,
/// }
/// ```
pub fn serialize_btreemap_as_object<S: Serializer, T: Serialize>(
    value: &BTreeMap<String, T>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut ser = serializer.serialize_struct("BTreeMap", value.len())?;
    for (k, v) in value {
        let key: &'static str = Box::leak(k.clone().into_boxed_str());
        ser.serialize_field(key, v)?;
    }
    ser.end()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use wasm_bindgen_test::wasm_bindgen_test;
    use serde_test::{assert_de_tokens, assert_ser_tokens, Token};

    #[cfg(target_family = "wasm")]
    #[wasm_bindgen_test]
    fn test_js_bigint_to_u256() {
        let bigint = js_sys::BigInt::from_str("12").unwrap();
        let result = bigint.try_into_u256().unwrap();
        let expected = U256::from(12);
        assert_eq!(result, expected);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_byte_serializer() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Bytes {
            #[serde(serialize_with = "bytes_serilializer")]
            field: Vec<u8>,
        }

        let bytes = Bytes {
            field: vec![1, 2, 3, 4, 5, 6],
        };

        assert_ser_tokens(
            &bytes,
            &[
                Token::Struct {
                    name: "Bytes",
                    len: 1,
                },
                Token::Str("field"),
                Token::Bytes(&[1, 2, 3, 4, 5, 6]),
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &bytes,
            &[
                Token::Struct {
                    name: "Bytes",
                    len: 1,
                },
                Token::Str("field"),
                Token::Seq { len: Some(6) },
                Token::U8(1),
                Token::U8(2),
                Token::U8(3),
                Token::U8(4),
                Token::U8(5),
                Token::U8(6),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_map_serializer() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Test {
            #[serde(serialize_with = "serialize_map_as_object")]
            field: HashMap<String, String>,
        }

        let mut hashmap = HashMap::new();
        hashmap.insert("key1".to_string(), "some value".to_string());
        hashmap.insert("key2".to_string(), "some other value".to_string());
        let test = Test { field: hashmap };

        assert_ser_tokens(
            &test,
            &[
                Token::Struct {
                    name: "Test",
                    len: 1,
                },
                Token::Str("field"),
                Token::Struct {
                    name: "HashMap",
                    len: 2,
                },
                Token::Str("key1"),
                Token::Str("some value"),
                Token::Str("key2"),
                Token::Str("some other value"),
                Token::StructEnd,
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &test,
            &[
                Token::Struct {
                    name: "Test",
                    len: 1,
                },
                Token::Str("field"),
                Token::Map { len: Some(2) },
                Token::Str("key1"),
                Token::Str("some value"),
                Token::Str("key2"),
                Token::Str("some other value"),
                Token::MapEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_bmap_serializer() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Test {
            #[serde(serialize_with = "serialize_btreemap_as_object")]
            field: BTreeMap<String, String>,
        }

        let mut bmap = BTreeMap::new();
        bmap.insert("key1".to_string(), "some value".to_string());
        bmap.insert("key2".to_string(), "some other value".to_string());
        let test = Test { field: bmap };

        assert_ser_tokens(
            &test,
            &[
                Token::Struct {
                    name: "Test",
                    len: 1,
                },
                Token::Str("field"),
                Token::Struct {
                    name: "BTreeMap",
                    len: 2,
                },
                Token::Str("key1"),
                Token::Str("some value"),
                Token::Str("key2"),
                Token::Str("some other value"),
                Token::StructEnd,
                Token::StructEnd,
            ],
        );

        assert_de_tokens(
            &test,
            &[
                Token::Struct {
                    name: "Test",
                    len: 1,
                },
                Token::Str("field"),
                Token::Map { len: Some(2) },
                Token::Str("key1"),
                Token::Str("some value"),
                Token::Str("key2"),
                Token::Str("some other value"),
                Token::MapEnd,
                Token::StructEnd,
            ],
        );
    }
}
