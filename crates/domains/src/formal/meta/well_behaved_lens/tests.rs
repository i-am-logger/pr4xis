//! Tests for the WellBehavedLens trait + canonical-form library.
//!
//! Three layers:
//!
//!   1. **Canonical-form idempotence** — `canonical(canonical(x)) == canonical(x)`
//!      for each per-source canonicalizer. (Per-form: deterministic
//!      unit test on small inputs + proptest on randomly-generated
//!      inputs.)
//!   2. **Signature determinism** — two evaluations of
//!      [`WellBehavedLens::signature`] on the same bytes yield the
//!      same digest.
//!   3. **PutGet law on synthetic sources** — for each canonical form
//!      we construct a deliberately-simple `WellBehavedLens`
//!      implementor whose `get` and `put` walk through a
//!      String / Value / Vec representation. The lens-law assertion
//!      then checks `sig(x) == sig(put(get(x)))` (Foster et al. 2007
//!      §3, Definition 3.2 PutGet).
//!
//! This file does *not* exercise real loaded sources (USLM, WordNet,
//! XSD, praxis.lock) — those land in the M4.θ.2 lens-law test
//! harness.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use proptest::prelude::*;

use super::canonical::{json, plain_text, toml as toml_canon, xml};
use super::lens_trait::{FailureStage, RoundTripFidelity, WellBehavedLens};

// ============================================================================
// Layer 1 — canonical-form idempotence
// ============================================================================

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn json_canonical_idempotent_simple() {
    let input = br#"{"b":2,"a":1,"c":[3,2,1]}"#;
    let c1 = json::canonicalize(input).expect("canonicalize");
    let c2 = json::canonicalize(&c1).expect("re-canonicalize");
    assert_eq!(c1, c2);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn json_canonical_sorts_keys() {
    let a = br#"{"b":2,"a":1}"#;
    let b = br#"{"a":1,"b":2}"#;
    let ca = json::canonicalize(a).expect("a");
    let cb = json::canonicalize(b).expect("b");
    assert_eq!(ca, cb);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn json_canonical_strips_whitespace() {
    let a = br#"{ "a" : 1 , "b" : 2 }"#;
    let b = br#"{"a":1,"b":2}"#;
    let ca = json::canonicalize(a).expect("a");
    let cb = json::canonicalize(b).expect("b");
    assert_eq!(ca, cb);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn json_canonical_escapes_control_chars() {
    // Input contains an *escaped* U+0001 (``); the canonical
    // form must keep the escape (RFC 8785 §3.2.1 + ECMA-262 string
    // serialization: U+0000–U+001F MUST be encoded as `\uXXXX`).
    let input: &[u8] = b"{\"k\":\"a\\u0001b\"}";
    let c = json::canonicalize(input).expect("canon");
    let s = String::from_utf8(c).unwrap();
    assert!(
        s.contains("\\u0001"),
        "control char U+0001 must be encoded as \\u0001 in canonical form, got: {:?}",
        s
    );
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn xml_canonical_idempotent_simple() {
    let input = br#"<r><a x="1" y="2"/></r>"#;
    let c1 = xml::canonicalize(input).expect("c1");
    let c2 = xml::canonicalize(&c1).expect("c2");
    assert_eq!(c1, c2);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn xml_canonical_sorts_attributes() {
    let a = br#"<r y="2" x="1"/>"#;
    let b = br#"<r x="1" y="2"/>"#;
    let ca = xml::canonicalize(a).expect("a");
    let cb = xml::canonicalize(b).expect("b");
    assert_eq!(ca, cb);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn xml_canonical_expands_empty_elements() {
    let input = br#"<r><a/></r>"#;
    let c = xml::canonicalize(input).expect("c");
    let s = String::from_utf8(c).expect("utf8");
    assert!(
        s.contains("<a></a>"),
        "empty element must expand to <a></a>, got {}",
        s
    );
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn xml_canonical_strips_xml_decl() {
    let with_decl = br#"<?xml version="1.0"?><r/>"#;
    let without_decl = br#"<r/>"#;
    let c_with = xml::canonicalize(with_decl).expect("with");
    let c_without = xml::canonicalize(without_decl).expect("without");
    assert_eq!(c_with, c_without);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn xml_canonical_strips_comments() {
    let with_comment = br#"<r><!-- hi --><a/></r>"#;
    let without_comment = br#"<r><a/></r>"#;
    let c_with = xml::canonicalize(with_comment).expect("with");
    let c_without = xml::canonicalize(without_comment).expect("without");
    assert_eq!(c_with, c_without);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn plain_text_canonical_idempotent() {
    let input = b"hello\r\nworld\r\n";
    let c1 = plain_text::canonicalize(input).expect("c1");
    let c2 = plain_text::canonicalize(&c1).expect("c2");
    assert_eq!(c1, c2);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn plain_text_canonical_folds_line_endings() {
    let crlf = b"a\r\nb";
    let lf = b"a\nb";
    let cr = b"a\rb";
    let c_crlf = plain_text::canonicalize(crlf).expect("crlf");
    let c_lf = plain_text::canonicalize(lf).expect("lf");
    let c_cr = plain_text::canonicalize(cr).expect("cr");
    assert_eq!(c_crlf, c_lf);
    assert_eq!(c_cr, c_lf);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn plain_text_canonical_strips_bom() {
    let with_bom = "\u{FEFF}hello".as_bytes();
    let without_bom = b"hello";
    let c_with = plain_text::canonicalize(with_bom).expect("with");
    let c_without = plain_text::canonicalize(without_bom).expect("without");
    assert_eq!(c_with, c_without);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn plain_text_canonical_applies_nfkc() {
    // U+00C5 (Å, composed) vs U+0041 U+030A (A + combining ring
    // above) are NFKC-equivalent. Their canonical forms must
    // coincide.
    let composed = "\u{00C5}".as_bytes();
    let decomposed = "A\u{030A}".as_bytes();
    let c_comp = plain_text::canonicalize(composed).expect("c");
    let c_decomp = plain_text::canonicalize(decomposed).expect("d");
    assert_eq!(c_comp, c_decomp);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn toml_canonical_idempotent() {
    let input = b"b = 2\na = 1\n";
    let c1 = toml_canon::canonicalize(input).expect("c1");
    let c2 = toml_canon::canonicalize(&c1).expect("c2");
    assert_eq!(c1, c2);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn toml_canonical_sorts_keys() {
    let a = b"b = 2\na = 1\n";
    let b = b"a = 1\nb = 2\n";
    let ca = toml_canon::canonicalize(a).expect("a");
    let cb = toml_canon::canonicalize(b).expect("b");
    assert_eq!(ca, cb);
}

#[pr4xis::praxis_value(Deterministic, Verifiable)]
#[test]
fn rdf_canonical_is_rdfc10_nquads() {
    use super::canonical::rdf;
    // A minimal RDF/XML graph: one class with a label.
    let doc = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns="http://example.org/o#">
  <owl:Class rdf:about="http://example.org/o#A">
    <rdfs:label>A</rdfs:label>
  </owl:Class>
</rdf:RDF>"#;
    let c1 = rdf::canonicalize(doc.as_bytes()).expect("RDFC-1.0 canonicalize");
    // Deterministic: a second run is byte-identical (RDFC §4.4.3).
    let c2 = rdf::canonicalize(doc.as_bytes()).expect("RDFC-1.0 canonicalize again");
    assert_eq!(c1, c2, "RDFC-1.0 canonical N-Quads must be deterministic");
    // The output is canonical N-Quads — `.`-terminated LF lines.
    let s = core::str::from_utf8(&c1).expect("UTF-8 N-Quads");
    assert!(
        s.lines().all(|l| l.is_empty() || l.ends_with(" .")),
        "RDFC-1.0 output is canonical N-Quads, got: {s:?}"
    );
    // The class's rdf:type triple is present in the canonical graph.
    assert!(
        s.contains(
            "<http://example.org/o#A> \
             <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> \
             <http://www.w3.org/2002/07/owl#Class> ."
        ),
        "canonical graph must carry the owl:Class type triple, got: {s}"
    );
}

// ============================================================================
// Layer 2 — signature determinism + the WellBehavedLens trait
// ============================================================================

/// A trivial `WellBehavedLens` impl whose source kind is "raw
/// UTF-8 string". Used as a witness that the trait machinery works.
struct StringSource;

impl WellBehavedLens for StringSource {
    type Target = String;
    type Error = super::canonical::CanonicalizationError;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        core::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|e| {
                super::canonical::CanonicalizationError::new(
                    "string-source",
                    format!("non-UTF-8: {}", e),
                )
            })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(target.as_bytes().to_vec())
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        plain_text::canonicalize(bytes)
    }
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn signature_is_deterministic() {
    let input = b"hello world";
    let s1 = StringSource::signature(input).expect("s1");
    let s2 = StringSource::signature(input).expect("s2");
    assert_eq!(s1, s2);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn assert_put_get_law_passes_for_identity_impl() {
    let input = b"hello world";
    StringSource::assert_put_get_law(input).expect("identity PutGet");
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn assert_put_get_law_passes_for_crlf_input() {
    // The CRLF gets normalized to LF in canonical form, but get +
    // put preserves it; the canonical-form sig still matches
    // because both sides canonicalize identically.
    let input = b"a\r\nb\r\nc";
    StringSource::assert_put_get_law(input).expect("crlf-eq");
}

/// A deliberately-broken `WellBehavedLens` impl that drops the
/// final character on `put`. Used to confirm that
/// `assert_put_get_law` actually detects ontology gaps.
struct DroppingStringSource;

impl WellBehavedLens for DroppingStringSource {
    type Target = String;
    type Error = super::canonical::CanonicalizationError;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        core::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|e| {
                super::canonical::CanonicalizationError::new(
                    "dropping-source",
                    format!("non-UTF-8: {}", e),
                )
            })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        // Drop the last byte if any.
        let bytes = target.as_bytes();
        if bytes.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(bytes[..bytes.len() - 1].to_vec())
        }
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        plain_text::canonicalize(bytes)
    }
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn assert_put_get_law_detects_dropped_byte() {
    let input = b"hello world";
    let err = DroppingStringSource::assert_put_get_law(input)
        .expect_err("dropping impl must fail PutGet");
    assert_eq!(err.stage, FailureStage::DigestMismatch);
    assert!(err.input_digest.is_some());
    assert!(err.roundtrip_digest.is_some());
    assert_ne!(err.input_digest, err.roundtrip_digest);
}

// ----------------------------------------------------------------------------
// Byte-exact law (M4.ι / #186) — strictly stronger than canonical PutGet.
// ----------------------------------------------------------------------------

/// An identity [`WellBehavedLens`] declaring byte-exact graph-faithful
/// fidelity. `get`/`put` round-trip valid UTF-8 verbatim, so the
/// byte-exact law `put(get(b)) == b` holds with no complement.
struct ByteExactStringSource;

impl WellBehavedLens for ByteExactStringSource {
    type Target = String;
    type Error = super::canonical::CanonicalizationError;

    const FIDELITY: RoundTripFidelity = RoundTripFidelity::ByteExactGraphFaithful;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        core::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|e| {
                super::canonical::CanonicalizationError::new(
                    "byte-exact-string-source",
                    format!("non-UTF-8: {}", e),
                )
            })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(target.as_bytes().to_vec())
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        plain_text::canonicalize(bytes)
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn fidelity_defaults_to_floor_and_can_be_overridden() {
    // Existing lenses inherit the conservative default so nothing flips
    // until they are migrated to graph-faithful byte-exactness.
    assert_eq!(
        StringSource::FIDELITY,
        RoundTripFidelity::RawBytesComplementFloor
    );
    assert_eq!(
        ByteExactStringSource::FIDELITY,
        RoundTripFidelity::ByteExactGraphFaithful
    );
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn assert_byte_exact_law_passes_for_identity_impl() {
    let input = b"hello world";
    ByteExactStringSource::assert_byte_exact_law(input).expect("identity byte-exact PutGet");
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn assert_byte_exact_law_preserves_crlf_unlike_canonical() {
    // The byte-exact law is strictly stronger: CRLF must survive
    // verbatim, where the canonical form would fold it to LF. The
    // identity lens reproduces the exact input bytes.
    let input = b"a\r\nb\r\nc";
    ByteExactStringSource::assert_byte_exact_law(input).expect("crlf byte-exact");
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn assert_byte_exact_law_detects_dropped_byte() {
    let input = b"hello world";
    let err = DroppingStringSource::assert_byte_exact_law(input)
        .expect_err("dropping impl must fail byte-exact PutGet");
    assert_eq!(err.stage, FailureStage::ByteMismatch);
    assert!(err.input_digest.is_some());
    assert!(err.roundtrip_digest.is_some());
    assert_ne!(err.input_digest, err.roundtrip_digest);
}

// ============================================================================
// Layer 3 — PutGet law on synthetic inputs through each canonical form
// ============================================================================

/// `WellBehavedLens` over JSON: get/put through serde_json,
/// canonicalize through RFC 8785.
struct JsonSource;

impl WellBehavedLens for JsonSource {
    type Target = serde_json::Value;
    type Error = super::canonical::CanonicalizationError;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        serde_json::from_slice(bytes).map_err(|e| {
            super::canonical::CanonicalizationError::new("json-source", format!("get: {}", e))
        })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(target).map_err(|e| {
            super::canonical::CanonicalizationError::new("json-source", format!("put: {}", e))
        })
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        json::canonicalize(bytes)
    }
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn json_put_get_law_synthetic() {
    let input = br#"{"name":"praxis","year":2026,"tags":["ontology","categories"]}"#;
    JsonSource::assert_put_get_law(input).expect("synthetic JSON PutGet");
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn json_put_get_law_unordered_keys() {
    // Same content, different key order. get -> put may produce
    // either order; canonical sorts both to the same form.
    let input = br#"{"z":1,"a":2,"m":3}"#;
    JsonSource::assert_put_get_law(input).expect("unordered keys");
}

/// `WellBehavedLens` over our subset of XML through quick-xml.
struct XmlSource;

impl WellBehavedLens for XmlSource {
    type Target = Vec<u8>;
    type Error = super::canonical::CanonicalizationError;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        // For the synthetic PutGet check the "ontology" is the byte
        // sequence itself; get is identity. Real source kinds
        // would get into an XSD-derived value.
        Ok(bytes.to_vec())
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(target.clone())
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        xml::canonicalize(bytes)
    }
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn xml_put_get_law_synthetic() {
    let input = br#"<root><child id="1">hello</child><child id="2">world</child></root>"#;
    XmlSource::assert_put_get_law(input).expect("synthetic XML PutGet");
}

/// `WellBehavedLens` over TOML.
struct TomlSource;

impl WellBehavedLens for TomlSource {
    type Target = ::toml::Value;
    type Error = super::canonical::CanonicalizationError;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        let s = core::str::from_utf8(bytes).map_err(|e| {
            super::canonical::CanonicalizationError::new("toml-source", format!("utf8: {}", e))
        })?;
        ::toml::from_str(s).map_err(|e| {
            super::canonical::CanonicalizationError::new("toml-source", format!("get: {}", e))
        })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        ::toml::to_string(target)
            .map(|s| s.into_bytes())
            .map_err(|e| {
                super::canonical::CanonicalizationError::new("toml-source", format!("put: {}", e))
            })
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        toml_canon::canonicalize(bytes)
    }
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn toml_put_get_law_synthetic() {
    let input =
        b"name = \"praxis\"\nyear = 2026\n[author]\nfirst = \"Ido\"\nlast = \"Samuelson\"\n";
    TomlSource::assert_put_get_law(input).expect("synthetic TOML PutGet");
}

// ============================================================================
// Layer 4 — proptest: canonical idempotence + signature determinism
// ============================================================================

proptest! {
    /// JSON canonicalize idempotence — for any valid JSON value
    /// produced by the strategy below, canonicalize twice equals
    /// once.
    #[test]
    fn proptest_json_canonical_idempotent(v in arb_simple_json_value()) {
        let bytes = serde_json::to_vec(&v).unwrap();
        let c1 = json::canonicalize(&bytes).unwrap();
        let c2 = json::canonicalize(&c1).unwrap();
        prop_assert_eq!(c1, c2);
    }

    /// Plain-text canonicalize idempotence on arbitrary UTF-8.
    #[test]
    fn proptest_plain_text_canonical_idempotent(s in ".*") {
        let bytes = s.as_bytes();
        let c1 = plain_text::canonicalize(bytes).unwrap();
        let c2 = plain_text::canonicalize(&c1).unwrap();
        prop_assert_eq!(c1, c2);
    }

    /// Signature determinism on arbitrary UTF-8.
    #[test]
    fn proptest_signature_deterministic(s in ".*") {
        let s1 = StringSource::signature(s.as_bytes()).unwrap();
        let s2 = StringSource::signature(s.as_bytes()).unwrap();
        prop_assert_eq!(s1, s2);
    }

    /// Plain-text PutGet via the StringSource is always faithful.
    #[test]
    fn proptest_string_source_put_get_law(s in ".*") {
        StringSource::assert_put_get_law(s.as_bytes())
            .unwrap_or_else(|e| panic!("PutGet failed on {:?}: {}", s, e));
    }

    /// The identity lens satisfies the strictly-stronger byte-exact
    /// law on arbitrary UTF-8 — `put(get(b)) == b` with no complement.
    #[test]
    fn proptest_byte_exact_string_source_law(s in ".*") {
        ByteExactStringSource::assert_byte_exact_law(s.as_bytes())
            .unwrap_or_else(|e| panic!("byte-exact law failed on {:?}: {}", s, e));
    }
}

pr4xis::register_praxis_value!(proptest_json_canonical_idempotent, Deterministic);
pr4xis::register_praxis_value!(proptest_plain_text_canonical_idempotent, Deterministic);
pr4xis::register_praxis_value!(proptest_signature_deterministic, Deterministic);
pr4xis::register_praxis_value!(proptest_string_source_put_get_law, Deterministic);
pr4xis::register_praxis_value!(proptest_byte_exact_string_source_law, Deterministic);

/// Strategy for arbitrary simple JSON values (no NaN/Infinity, no
/// floats with extreme precision that would make ECMA-262 number
/// rendering diverge from serde_json).
fn arb_simple_json_value() -> impl Strategy<Value = serde_json::Value> {
    use serde_json::{Map, Value};
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i32>().prop_map(|n| Value::Number(n.into())),
        "[a-zA-Z0-9_ ]{0,16}".prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 16, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::vec(("[a-z]{1,5}", inner), 0..4).prop_map(|kvs| {
                let mut m = Map::new();
                for (k, v) in kvs {
                    m.insert(k, v);
                }
                Value::Object(m)
            }),
        ]
    })
}
