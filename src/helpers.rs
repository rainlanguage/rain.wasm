use alloy_primitives::{U256, ruint::ParseError};
use std::{error::Error, fmt::Display, str::FromStr};

// A trait for converting a given type to U256
pub trait TryIntoU256<T: FromStr<Err = E>, E: Error>
where
    Self: Display,
{
    fn try_into_u256(&self) -> Result<T, E> {
        T::from_str(&format!("{}", &self))
    }
}
impl TryIntoU256<U256, ParseError> for js_sys::BigInt {}

// a serializer fn for serializing Vec<u8> as bytes (Uint8Array for js)
pub fn bytes_serilializer<S: serde::Serializer>(
    val: &[u8],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(val)
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
