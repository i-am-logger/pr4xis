//! JSON canonicalization per RFC 8785 *JSON Canonicalization Scheme*
//! (Rundgren, Jordan & Erdtman 2020, IETF,
//! <https://www.rfc-editor.org/rfc/rfc8785.html>).
//!
//! No `serde_jcs` crate is in our dependency tree at the time of the
//! M4.θ.0 survey; we implement the JCS subset directly on top of
//! [`serde_json::Value`].
//!
//! ## Spec coverage (RFC 8785 §3)
//!
//! - **§3.2.1 Serialization of primitive data types.** Strings are
//!   re-emitted with the RFC 8785 escape rules (the JSON.stringify
//!   subset from ECMA-262: `\\`, `\"`, control chars `<= U+001F`
//!   via `\uXXXX`).
//! - **§3.2.2 Serialization of objects.** Members emitted sorted by
//!   key (UTF-16 code-unit lexical comparison per the spec).
//! - **§3.2.3 Serialization of arrays.** Elements emitted in source
//!   order.
//! - **§3.2.4 Serialization of `null`, `true`, `false`.**
//! - **§3.2.5 Serialization of numbers.** Integers and floats are
//!   serialized per the ECMA-262 `Number.prototype.toString`
//!   algorithm, which is *not* what `serde_json` does by default.
//!   We approximate this for the cases the round-trip gate
//!   exercises (integers and finite floats with reasonable
//!   precision). NaN/Infinity are rejected (JSON does not represent
//!   them per RFC 8259).
//!
//! The ECMA-262 number serialization is the only subtle path. A
//! full RFC 8785 implementation would link `Number.prototype.toString`
//! exactly; our approximation re-uses `serde_json`'s `to_string`
//! which is sufficient for integers and simple floats produced by
//! JSON loaders. Round-trip tests on real loaded sources surface
//! cases where the approximation diverges.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use serde_json::Value;

use super::CanonicalizationError;

const FORM: &str = "json-jcs-rfc-8785";

/// Canonicalize `bytes` (a JSON document) per RFC 8785.
pub fn canonicalize(bytes: &[u8]) -> Result<Vec<u8>, CanonicalizationError> {
    let v: Value = serde_json::from_slice(bytes)
        .map_err(|e| CanonicalizationError::new(FORM, format!("parse: {}", e)))?;
    let mut out = String::new();
    write_value(&v, &mut out)?;
    Ok(out.into_bytes())
}

fn write_value(v: &Value, out: &mut String) -> Result<(), CanonicalizationError> {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            // RFC 8785 §3.2.5: numbers use ECMA-262 toString. NaN/Inf
            // are not JSON-representable. serde_json::Number rejects
            // those at parse time, so we just emit its Display form.
            out.push_str(&n.to_string());
        }
        Value::String(s) => {
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\u{0008}' => out.push_str("\\b"),
                    '\u{000C}' => out.push_str("\\f"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if (c as u32) < 0x20 => {
                        out.push_str(&format!("\\u{:04x}", c as u32));
                    }
                    c => out.push(c),
                }
            }
            out.push('"');
        }
        Value::Array(arr) => {
            out.push('[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(item, out)?;
            }
            out.push(']');
        }
        Value::Object(obj) => {
            // RFC 8785 §3.2.2: members sorted by UTF-16 code-unit
            // lexical comparison. For ASCII / BMP-only keys this is
            // the same as char order. Praxis's loaded JSON sources
            // are ASCII-keyed; we use Rust's default lexicographic
            // String ordering (UTF-8 byte order). For purely-BMP /
            // ASCII keys this coincides with UTF-16 code-unit order.
            // A future commit can switch to true UTF-16 ordering
            // when round-trip on a real source surfaces a key with
            // non-BMP characters.
            let mut entries: Vec<(&String, &Value)> = obj.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            out.push('{');
            for (i, (k, vv)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                // Reuse string-emit for the key.
                let key_value = Value::String((*k).clone());
                write_value(&key_value, out)?;
                out.push(':');
                write_value(vv, out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}
