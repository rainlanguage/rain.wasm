use std::collections::{BTreeMap, HashMap};
use serde::{ser::SerializeStruct, Serialize, Serializer};

/// Serializer fn for serializing Vec\<u8\> as bytes (Uint8Array for js)
/// Example:
/// ```ignore
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct A {
///     #[serde(serialize_with = "serialize_as_bytes")]
///     field: Vec<u8>,
/// }
/// ```
pub fn serialize_as_bytes<S: Serializer>(val: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(val)
}

/// Serializer fn for serializing u64 as js bigint
/// Example:
/// ```ignore
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct A {
///     #[serde(serialize_with = "serialize_u64_as_bigint")]
///     field: u64,
/// }
/// ```
pub fn serialize_u64_as_bigint<S: Serializer>(val: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u128(*val as u128)
}

/// Serializer fn for serializing i64 as js bigint
/// Example:
/// ```ignore
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct A {
///     #[serde(serialize_with = "serialize_i64_as_bigint")]
///     field: i64,
/// }
/// ```
pub fn serialize_i64_as_bigint<S: Serializer>(val: &i64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i128(*val as i128)
}

/// Serializer fn that serializes HashMap as k/v object.
/// in js it would be plain js object and not js Map.
///
/// The [HashMap]'s entry values should themselves impl
/// [Serialize] as well.
///
/// This provides great level of flexibilty to specify a
/// specific property of the given type and not all of the
/// properties to be serialized as js plain object instead
/// of js Map when wasm_bindgen convert traits are implemented
/// for the given type by using [impl_wasm_traits](crate::impl_wasm_traits)
///
/// Example:
/// ```ignore
/// #[derive(serde::Serialize, serde::Deserialize, Tsify)]
/// struct A {
///     #[cfg_attr(
///         target_family = "wasm",
///         serde(serialize_with = "serialize_hashmap_as_object"),
///         tsify(type = "Record<string, number>")
///     )]
///     field: HashMap<String, u8>,
/// }
/// #[cfg(target_family = "wasm")]
/// impl_all_wasm_traits!(A);
///
/// #[wasm_bindgen]
/// pub fn some_fn() -> A {
///     let mut rust_map = HashMap::new();
///     rust_map.insert("key".to_string(), 1);
///     rust_map.insert("otherKey".to_string(), 2);
///
///     // in js when some_fn() is called the result will be:
///     // { field: { key: 1, otherKey: 2 } }
///     A { field: rust_map }
/// }
/// ```
pub fn serialize_hashmap_as_object<V, S>(
    val: &HashMap<String, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    // K: ToString,
    V: Serialize,
    S: Serializer,
{
    let mut ser = serializer.serialize_struct("HashMap", val.len())?;
    for (key, value) in val {
        // static str is not actually needed since we are dealing
        // with a hashmap which its keys can change at runtime
        // so we can safely deref the &str for this purpose
        let key = unsafe { &*(key.as_str() as *const str) };
        ser.serialize_field(key, value)?;
    }
    ser.end()
}

/// Same as [serialize_hashmap_as_object] but for `Option<HashMap>`
pub fn serialize_opt_hashmap_as_object<V, S>(
    val: &Option<HashMap<String, V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    // K: ToString,
    V: Serialize,
    S: Serializer,
{
    match val {
        Some(ref val) => serialize_hashmap_as_object(val, serializer),
        None => serializer.serialize_none(),
    }
}

/// Serializer fn that serializes BTreeMap as k/v object.
/// in js it would be plain js object and not js Map.
///
/// The [BTreeMap]'s entry values should themselves impl
/// [Serialize] as well.
///
/// This provides great level of flexibilty to specify a
/// specific property of the given type and not all of the
/// properties to be serialized as js plain object instead
/// of js Map when wasm_bindgen convert traits are implemented
/// for the given type by using [impl_wasm_traits](crate::impl_wasm_traits)
///
/// Example:
/// ```ignore
/// #[derive(serde::Serialize, serde::Deserialize, Tsify)]
/// struct A {
///     #[cfg_attr(
///         target_family = "wasm",
///         serde(serialize_with = "serialize_hashmap_as_object"),
///         tsify(type = "Record<string, number>")
///     )]
///     field: BTreeMap<String, u8>,
/// }
/// #[cfg(target_family = "wasm")]
/// impl_all_wasm_traits!(A);
///
/// #[wasm_bindgen]
/// pub fn some_fn() -> A {
///     let mut rust_map = BTreeMAp::new();
///     rust_map.insert("key".to_string(), 1);
///     rust_map.insert("otherKey".to_string(), 2);
///
///     // in js when some_fn() is called the result will be:
///     // { field: { key: 1, otherKey: 2 } }
///     A { field: rust_map }
/// }
/// ```
pub fn serialize_btreemap_as_object<V, S>(
    val: &BTreeMap<String, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    // K: ToString,
    V: Serialize,
    S: Serializer,
{
    let mut ser = serializer.serialize_struct("BTreeMap", val.len())?;
    for (key, value) in val {
        // static str is not actually needed since we are dealing
        // with a btreemap which its keys can change at runtime
        // so we can safely deref the &str for this purpose
        let key = unsafe { &*(key.as_str() as *const str) };
        ser.serialize_field(key, value)?;
    }
    ser.end()
}

/// Same as [serialize_btreemap_as_object] but for `Option<BTreeMap>`
pub fn serialize_opt_btreemap_as_object<V, S>(
    val: &Option<BTreeMap<String, V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    // K: ToString,
    V: Serialize,
    S: Serializer,
{
    match val {
        Some(ref val) => serialize_btreemap_as_object(val, serializer),
        None => serializer.serialize_none(),
    }
}

#[cfg(target_family = "wasm")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use wasm_bindgen_test::wasm_bindgen_test;
    use serde_test::{assert_de_tokens, assert_ser_tokens, Token};

    #[wasm_bindgen_test]
    fn test_byte_serializer() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Bytes {
            #[serde(serialize_with = "serialize_as_bytes")]
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
    fn test_u64_serializer() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Bytes {
            #[serde(serialize_with = "serialize_u64_as_bigint")]
            field: u64,
        }

        let bytes = Bytes { field: 123 };

        assert_de_tokens(
            &bytes,
            &[
                Token::Struct {
                    name: "Bytes",
                    len: 1,
                },
                Token::Str("field"),
                Token::U64(123),
                Token::StructEnd,
            ],
        );
    }

    #[wasm_bindgen_test]
    fn test_i64_serializer() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Bytes {
            #[serde(serialize_with = "serialize_i64_as_bigint")]
            field: i64,
        }

        let bytes = Bytes { field: 123 };

        assert_de_tokens(
            &bytes,
            &[
                Token::Struct {
                    name: "Bytes",
                    len: 1,
                },
                Token::Str("field"),
                Token::I64(123),
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
