//! TOML canonicalization — Praxis-defined.
//!
//! TOML has no IETF / ISO canonical-form specification at the time
//! of the M4.θ.0 survey. The closest published reference is the
//! TOML 1.0.0 spec itself (Preston-Werner et al., 2021,
//! <https://toml.io/en/v1.0.0>), which describes value semantics
//! but not a unique canonical serialization.
//!
//! We define Praxis's canonical form for TOML as the following
//! transformation:
//!
//!   1. Parse the input as [`::toml::Value`].
//!   2. Walk the value tree depth-first; for every table, emit
//!      members sorted by ASCII key order.
//!   3. Emit values using the [`::toml::ser`] serializer's standard
//!      output. The `toml` crate emits a consistent shape
//!      (double-quoted strings; integers without leading zeros;
//!      `true`/`false`; floats with a decimal point), so the
//!      pre-sort + re-serialize sequence yields a deterministic
//!      byte sequence for any given parsed value.
//!
//! ## Caveats (documented; relevant for the round-trip gate)
//!
//! - **Comment loss.** TOML comments are not modelled by
//!   [`::toml::Value`]; canonicalization drops them. Two TOML files
//!   that differ only in comments share a canonical form. Praxis's
//!   loaded TOML sources (`praxis.toml`, `praxis.lock`,
//!   `citations.toml`) are operationally meaningful only at the
//!   value level — comments are documentation for humans.
//! - **Table vs. inline-table layout.** TOML supports two
//!   equivalent serializations for the same value: standard
//!   `[table]` headers vs. `table = { … }` inline form. The
//!   canonical form is whichever the `toml` crate chooses by
//!   default at re-emit time. New `toml` crate versions may
//!   change the choice, in which case re-canonicalize the
//!   sources.
//!
//! When TOML acquires a published canonical-form RFC, this
//! implementation moves to it and the doc-comment is rewritten.

use alloc::format;
use alloc::vec::Vec;

use ::toml::Value;

use super::CanonicalizationError;

const FORM: &str = "toml-praxis-canonical";

/// Canonicalize `bytes` (a TOML document) per the Praxis-defined
/// canonical form documented in the module header.
pub fn canonicalize(bytes: &[u8]) -> Result<Vec<u8>, CanonicalizationError> {
    let s = core::str::from_utf8(bytes)
        .map_err(|e| CanonicalizationError::new(FORM, format!("non-UTF-8: {}", e)))?;
    let parsed: Value = ::toml::from_str(s)
        .map_err(|e| CanonicalizationError::new(FORM, format!("parse: {}", e)))?;
    let sorted = sort_value(parsed);
    let out = ::toml::to_string(&sorted)
        .map_err(|e| CanonicalizationError::new(FORM, format!("emit: {}", e)))?;
    Ok(out.into_bytes())
}

/// Recursively sort table keys.
fn sort_value(v: Value) -> Value {
    match v {
        Value::Table(t) => {
            // BTreeMap is sorted by key; rebuild the table from it.
            let mut entries: alloc::vec::Vec<(alloc::string::String, Value)> =
                t.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut out = ::toml::map::Map::new();
            for (k, vv) in entries {
                out.insert(k, sort_value(vv));
            }
            Value::Table(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_value).collect()),
        other => other,
    }
}
