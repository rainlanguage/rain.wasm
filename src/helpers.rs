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
pub fn bytes_serilializer<S: Serializer>(val: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(val)
}

/// Serializer fn that serializes HashMap as k/v object.
/// in js it would be plain js object and not js Map.
///
/// The [HashMap]'s keys should either be String or impl
/// [ToString] trait so that they can be converted to a
/// valid string key when serialized.
/// The [HashMap]'s entry values should themselves impl
/// [Serialize] as well.
///
/// This serializer fn idealy is meant to be used with
/// [serde_wasm_bindgen::Serializer] with wasm bindgen traits
/// in [crate::macros] implemented for its parent type.
///
/// Example:
/// ```ignore
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct A {
///     #[cfg_attr(target_family = "wasm", serde(serialize_with = "serialize_hashmap_as_object"))]
///     field: HashMap<String, u8>,
/// }
/// #[cfg(target_family = "wasm")]
/// impl_all_wasm_traits!(A);
/// ```
pub fn serialize_hashmap_as_object<K, V, S>(
    val: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    K: ToString,
    V: Serialize,
    S: Serializer,
{
    let mut ser = serializer.serialize_struct("HashMap", val.len())?;
    for (key, value) in val {
        // static str is not actually needed since we are dealing
        // with a hashmap which its keys can change at runtime
        // so we can safely deref the &str for this purpose
        let key = unsafe { &*(key.to_string().as_str() as *const str) };
        ser.serialize_field(key, value)?;
    }
    ser.end()
}

/// Serializer fn that serializes BTreeMap as k/v object.
/// in js it would be plain js object and not js Map.
///
/// The [BTreeMap]'s keys should either be String or impl
/// [ToString] trait so that they can be converted to a
/// valid string key when serialized.
/// The [BTreeMap]'s entry values should themselves impl
/// [Serialize] as well.
///
/// This serializer fn idealy is meant to be used with
/// [serde_wasm_bindgen::Serializer] with wasm bindgen traits
/// in [crate::macros] implemented for its parent type.
///
/// Example:
/// ```ignore
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct A {
///     #[cfg_attr(target_family = "wasm", serde(serialize_with = "serialize_btreemap_as_object"))]
///     field: BTreeMap<String, u8>,
/// }
/// #[cfg(target_family = "wasm")]
/// impl_all_wasm_traits!(A);
/// ```
pub fn serialize_btreemap_as_object<K, V, S>(
    val: &BTreeMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    K: ToString,
    V: Serialize,
    S: Serializer,
{
    let mut ser = serializer.serialize_struct("BTreeMap", val.len())?;
    for (key, value) in val {
        // static str is not actually needed since we are dealing
        // with a btreemap which its keys can change at runtime
        // so we can safely deref the &str for this purpose
        let key = unsafe { &*(key.to_string().as_str() as *const str) };
        ser.serialize_field(key, value)?;
    }
    ser.end()
}

#[cfg(target_family = "wasm")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use wasm_bindgen_test::wasm_bindgen_test;
    use serde_test::{assert_de_tokens, assert_ser_tokens, Token};

    #[wasm_bindgen_test]
    fn test_js_bigint_to_u256() {
        let bigint = js_sys::BigInt::from_str("12").unwrap();
        let result = bigint.try_into_u256().unwrap();
        let expected = U256::from(12);
        assert_eq!(result, expected);
    }

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

    #[wasm_bindgen_test]
    fn test_map_serializer() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Test {
            #[serde(serialize_with = "serialize_hashmap_as_object")]
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

    #[wasm_bindgen_test]
    fn test_bmap_serializer() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Test {
            #[serde(serialize_with = "serialize_btreemap_as_object")]
            field: BTreeMap<String, u8>,
        }

        let mut bmap = BTreeMap::new();
        bmap.insert("key1".to_string(), 8);
        bmap.insert("key2".to_string(), 9);
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
                Token::U8(8),
                Token::Str("key2"),
                Token::U8(9),
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
                Token::U8(8),
                Token::Str("key2"),
                Token::U8(9),
                Token::MapEnd,
                Token::StructEnd,
            ],
        );
    }
}
