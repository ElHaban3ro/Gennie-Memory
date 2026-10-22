//! Byte receiver retains last-key legacy semantics but rejects duplicate keys for V1.
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use std::fmt;

struct Parsed {
    value: Value,
    duplicate: bool,
    contextual: bool,
}
impl Parsed {
    fn scalar(value: Value) -> Self {
        Self {
            value,
            duplicate: false,
            contextual: false,
        }
    }
}
impl<'de> Deserialize<'de> for Parsed {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Parsed;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON value")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Parsed, E> {
                Ok(Parsed::scalar(Value::Bool(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Parsed, E> {
                Ok(Parsed::scalar(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Parsed, E> {
                Ok(Parsed::scalar(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Parsed, E> {
                serde_json::Number::from_f64(v)
                    .map(|v| Parsed::scalar(Value::Number(v)))
                    .ok_or_else(|| E::custom("invalid number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Parsed, E> {
                Ok(Parsed::scalar(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Parsed, E> {
                Ok(Parsed::scalar(v.into()))
            }
            fn visit_none<E: de::Error>(self) -> Result<Parsed, E> {
                Ok(Parsed::scalar(Value::Null))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Parsed, E> {
                Ok(Parsed::scalar(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Parsed, A::Error> {
                let (mut values, mut duplicate, mut contextual) = (Vec::new(), false, false);
                while let Some(p) = a.next_element::<Parsed>()? {
                    duplicate |= p.duplicate;
                    contextual |= p.contextual;
                    values.push(p.value);
                }
                Ok(Parsed {
                    value: Value::Array(values),
                    duplicate,
                    contextual,
                })
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Parsed, A::Error> {
                let (mut values, mut duplicate, mut contextual) = (Map::new(), false, false);
                while let Some(k) = a.next_key::<String>()? {
                    contextual |=
                        matches!(k.as_str(), "request_context_profile" | "context_bindings");
                    let p = a.next_value::<Parsed>()?;
                    duplicate |= p.duplicate || values.contains_key(&k);
                    contextual |= p.contextual;
                    values.insert(k, p.value);
                }
                Ok(Parsed {
                    value: Value::Object(values),
                    duplicate,
                    contextual,
                })
            }
        }
        d.deserialize_any(V)
    }
}
pub(super) fn parse(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() > super::packet::LIMIT {
        return Err(format!(
            "entrada V3 supera 16 MiB; sha256={}",
            super::packet::digest(bytes)
        ));
    }
    let p: Parsed = serde_json::from_slice(bytes)
        .map_err(|e| format!("JSON V3: {e}; sha256={}", super::packet::digest(bytes)))?;
    if p.contextual && p.duplicate {
        return Err(format!(
            "claves JSON duplicadas en contrato request_context V1; sha256={}",
            super::packet::digest(bytes)
        ));
    }
    Ok(p.value)
}
/// Closed new transports reject duplicate keys even without a contextual profile.
pub(super) fn parse_strict(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() > super::packet::LIMIT {
        return Err("entrada V3 supera 16 MiB".into());
    }
    let parsed: Parsed = serde_json::from_slice(bytes).map_err(|e| format!("JSON V3: {e}"))?;
    if parsed.duplicate {
        return Err("claves JSON duplicadas en contrato léxico V1".into());
    }
    Ok(parsed.value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recursive_duplicate_is_detected_before_value_can_hide_the_profile() {
        for bytes in [
            br#"{"vocabulary":{"request_context_profile":{},"request_context_profile":null}}"#
                .as_slice(),
            br#"{"vocabulary":{"request_context_profile":{}},"raw":{"x":{"a":1,"a":2}}}"#,
            br#"{"vocabulary":{"request_context_profile":{}},"vocabulary":{}}"#,
            br#"{"raw":{"context_bindings":[{"kind":"a","kind":"b"}]}}"#,
        ] {
            let error = parse(bytes).unwrap_err();
            assert!(error.contains("duplicadas"));
            assert!(error.contains("sha256="));
        }
    }
    #[test]
    fn strict_receiver_rejects_nested_manifest_and_intent_duplicates_without_profile() {
        for input in [
            br#"{"review_anchors":[{"review_anchor_id":"a","review_anchor_id":"b"}]}"#.as_slice(),
            br#"{"selection":{"predicate":"a","predicate":"b"}}"#,
        ] {
            assert!(parse_strict(input).unwrap_err().contains("duplicadas"));
        }
    }
    #[test]
    fn legacy_last_key_behavior_and_numbers_are_preserved() {
        let bytes = br#"{"a":1,"a":2,"b":[true,null,"text",-2,1.25,18446744073709551615]}"#;
        assert_eq!(
            parse(bytes).unwrap(),
            serde_json::from_slice::<Value>(bytes).unwrap()
        );
        let bytes = br#"{"request_context_profile":{},"raw":{"a":1}}"#;
        assert_eq!(
            parse(bytes).unwrap(),
            serde_json::from_slice::<Value>(bytes).unwrap()
        );
    }
}
