//! Lexical / value / canonical mappings for the binary, URI, and
//! qualified-name XSD datatypes (W3C XML Schema 1.1 Part 2
//! §3.3.15-§3.3.19): `hexBinary`, `base64Binary`, `anyURI`, `QName`,
//! `NOTATION`.
//!
//! - **hexBinary** (§3.3.15): value = a byte sequence; lexical =
//!   `(hexDigit hexDigit)*`; canonical = uppercase hex octets
//!   (`·hexDigitCanonical·` maps to `0-9A-F`).
//! - **base64Binary** (§3.3.16): value = a byte sequence; lexical =
//!   the Base64 grammar (`#x20` permitted between characters, final
//!   quantum restricted to canonical padding chars); canonical =
//!   RFC 2045 Base64 with no whitespace. A literal whose padding bits
//!   are non-zero is outside the lexical space (enforced by requiring
//!   the decode to re-encode identically).
//! - **anyURI** (§3.3.17): value = the character sequence after
//!   `whiteSpace = collapse`; XSD 1.1's lexical space is permissive.
//! - **QName** (§3.3.18) / **NOTATION** (§3.3.19): lexical =
//!   `(Prefix ':')? LocalPart` with `Prefix` / `LocalPart` NCNames.
//!   The full value depends on in-scope namespace bindings (out of
//!   scope here); this module validates the lexical structure and
//!   echoes the collapsed literal as its context-free canonical form.
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05. §3.3.15-§3.3.19, Appendix E.
//! - **IETF RFC 2045** §6.8 (Base64 Content-Transfer-Encoding).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::strings::{WhiteSpace, apply_white_space, is_ncname};

// =============================================================================
// hexBinary — §3.3.15.
// =============================================================================

/// Parse a `·hexBinary·` literal (§3.3.15.1): an even-length run of
/// hex digits (after `whiteSpace = collapse`), decoded to bytes.
/// Returns `None` if it is not a whole number of hex octets.
pub fn parse_hex_binary(lex: &str) -> Option<Vec<u8>> {
    let collapsed = apply_white_space(lex, WhiteSpace::Collapse);
    let b = collapsed.as_bytes();
    if !b.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(b.len() / 2);
    for pair in b.chunks(2) {
        let hi = hex_value(pair[0])?;
        let lo = hex_value(pair[1])?;
        out.push((hi << 4) | lo);
    }
    Some(out)
}

/// `·hexBinaryCanonical·` (Appendix E): uppercase hex, two digits per
/// octet (`·hexDigitCanonical·` → `0-9A-F`).
pub fn canonical_hex_binary(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02X}"));
    }
    s
}

/// Value of a single `hexDigit` (§3.3.15.1 `[0-9a-fA-F]`).
fn hex_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// =============================================================================
// base64Binary — §3.3.16.
// =============================================================================

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Parse a `·base64Binary·` literal (§3.3.16.1) to bytes. Whitespace is
/// removed (the grammar permits `#x20` between characters; `whiteSpace`
/// is `collapse`); the result must be canonical Base64 — a literal
/// with non-zero padding bits (outside the `B16char`/`B04char` final
/// quantum) is rejected by requiring the decode to re-encode identically.
pub fn parse_base64_binary(lex: &str) -> Option<Vec<u8>> {
    let stripped: String = lex
        .chars()
        .filter(|c| !matches!(c, ' ' | '\t' | '\n' | '\r'))
        .collect();
    let bytes = base64_decode(&stripped)?;
    // Enforce canonical padding bits: the literal must be exactly what
    // the canonical encoder would produce (modulo the removed
    // whitespace).
    (canonical_base64_binary(&bytes) == stripped).then_some(bytes)
}

/// `·base64BinaryCanonical·` (Appendix E): RFC 2045 Base64 with no
/// whitespace and canonical final-quantum padding.
pub fn canonical_base64_binary(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        match chunk {
            [b0, b1, b2] => {
                out.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
                out.push(BASE64_ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
                out.push(BASE64_ALPHABET[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
                out.push(BASE64_ALPHABET[(b2 & 0x3F) as usize] as char);
            }
            [b0, b1] => {
                out.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
                out.push(BASE64_ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
                out.push(BASE64_ALPHABET[((b1 & 0x0F) << 2) as usize] as char);
                out.push('=');
            }
            [b0] => {
                out.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
                out.push(BASE64_ALPHABET[((b0 & 0x03) << 4) as usize] as char);
                out.push('=');
                out.push('=');
            }
            _ => {}
        }
    }
    out
}

/// Decode a whitespace-free Base64 string to bytes, validating length,
/// alphabet, and `=` padding placement.
fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let b = s.as_bytes();
    if !b.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::new();
    let n = b.len();
    let mut i = 0;
    while i < n {
        let quad = &b[i..i + 4];
        let is_last = i + 4 == n;
        let c0 = base64_value(quad[0])?;
        let c1 = base64_value(quad[1])?;
        if quad[2] == b'=' {
            // Single octet: `xx==`, only in the final quad.
            if !is_last || quad[3] != b'=' {
                return None;
            }
            out.push((c0 << 2) | (c1 >> 4));
        } else {
            let c2 = base64_value(quad[2])?;
            if quad[3] == b'=' {
                // Two octets: `xxx=`, only in the final quad.
                if !is_last {
                    return None;
                }
                out.push((c0 << 2) | (c1 >> 4));
                out.push((c1 << 4) | (c2 >> 2));
            } else {
                let c3 = base64_value(quad[3])?;
                out.push((c0 << 2) | (c1 >> 4));
                out.push((c1 << 4) | (c2 >> 2));
                out.push((c2 << 6) | c3);
            }
        }
        i += 4;
    }
    Some(out)
}

/// The 6-bit value of a Base64 alphabet character (RFC 2045 §6.8).
fn base64_value(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

// =============================================================================
// anyURI — §3.3.17.
// =============================================================================

/// Parse an `·anyURI·` literal (§3.3.17.1): `whiteSpace = collapse`
/// applied; XSD 1.1's lexical space admits any resulting character
/// sequence. The value is the collapsed string.
pub fn parse_any_uri(lex: &str) -> String {
    apply_white_space(lex, WhiteSpace::Collapse)
}

// =============================================================================
// QName / NOTATION — §3.3.18 / §3.3.19.
// =============================================================================

/// The lexical structure of an `xs:QName` (§3.3.18): an optional NCName
/// prefix and an NCName local part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QNameValue {
    pub prefix: Option<String>,
    pub local: String,
}

impl QNameValue {
    /// The context-free canonical literal: `prefix:local` or `local`.
    /// (The fully resolved value requires in-scope namespace bindings,
    /// which are outside this datatype module.)
    pub fn canonical(&self) -> String {
        match &self.prefix {
            Some(p) => format!("{p}:{}", self.local),
            None => self.local.clone(),
        }
    }
}

/// Parse an `xs:QName` literal (§3.3.18.1): `(Prefix ':')? LocalPart`,
/// each part an NCName, after `whiteSpace = collapse`.
pub fn parse_qname(lex: &str) -> Option<QNameValue> {
    let v = apply_white_space(lex, WhiteSpace::Collapse);
    match v.split_once(':') {
        Some((prefix, local)) => {
            if is_ncname(prefix) && is_ncname(local) {
                Some(QNameValue {
                    prefix: Some(prefix.to_string()),
                    local: local.to_string(),
                })
            } else {
                None
            }
        }
        None => is_ncname(&v).then_some(QNameValue {
            prefix: None,
            local: v,
        }),
    }
}

/// Parse an `xs:NOTATION` literal (§3.3.19.1). `NOTATION` shares
/// `QName`'s lexical space (its values are the QNames of declared
/// notations).
pub fn parse_notation(lex: &str) -> Option<QNameValue> {
    parse_qname(lex)
}

// =============================================================================
// Axioms.
// =============================================================================

/// Axiom: hexBinary round-trips bytes, canonicalizes to uppercase, is
/// case-insensitive on input, and rejects odd-length literals
/// (§3.3.15).
pub struct HexBinaryRoundTrips;

impl Axiom for HexBinaryRoundTrips {
    fn verify(&self) -> Verdict {
        let ok = parse_hex_binary("0FB7") == Some(vec![0x0F, 0xB7])
            && parse_hex_binary("0fb7") == parse_hex_binary("0FB7") // case-insensitive
            && canonical_hex_binary(&[0x0F, 0xB7]) == "0FB7"        // uppercase canonical
            && parse_hex_binary("") == Some(vec![])
            && parse_hex_binary("ABC").is_none()                    // odd length
            && parse_hex_binary("GG").is_none(); // not hex
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "HexBinaryRoundTrips",
        "hexBinary decodes case-insensitively, canonicalizes to uppercase octets, round-trips bytes, and rejects odd-length/non-hex literals",
        "W3C XSD 1.1 Part 2 §3.3.15, Appendix E (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(HexBinaryRoundTrips, "W3C XSD 1.1 Part 2 §3.3.15");

/// Axiom: base64Binary round-trips bytes, tolerates internal
/// whitespace, canonicalizes without whitespace, and rejects
/// non-canonical padding bits (§3.3.16).
pub struct Base64BinaryRoundTrips;

impl Axiom for Base64BinaryRoundTrips {
    fn verify(&self) -> Verdict {
        // "Man" -> "TWFu"; one/two-octet padding cases.
        let ok = parse_base64_binary("TWFu") == Some(b"Man".to_vec())
            && canonical_base64_binary(b"Man") == "TWFu"
            && parse_base64_binary("TW Fu") == Some(b"Man".to_vec())   // whitespace tolerated
            && canonical_base64_binary(b"M") == "TQ=="
            && canonical_base64_binary(b"Ma") == "TWE="
            && parse_base64_binary("TQ==") == Some(b"M".to_vec())
            && parse_base64_binary("") == Some(vec![])
            && parse_base64_binary("TQ=A").is_none()                   // '=' mid-quad
            && parse_base64_binary("TR==").is_none(); // non-zero padding bits
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "Base64BinaryRoundTrips",
        "base64Binary decodes (tolerating whitespace), round-trips bytes, canonicalizes without whitespace, and rejects misplaced '=' and non-canonical padding bits",
        "W3C XSD 1.1 Part 2 §3.3.16, Appendix E; RFC 2045 §6.8 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(Base64BinaryRoundTrips, "W3C XSD 1.1 Part 2 §3.3.16");

/// Axiom: anyURI collapses whitespace and admits URI references; QName
/// accepts `prefix:local` and bare `local` of NCNames, rejecting two
/// colons and non-NCName parts (§3.3.17-§3.3.18).
pub struct UriAndQNameLexical;

impl Axiom for UriAndQNameLexical {
    fn verify(&self) -> Verdict {
        let uri_ok = parse_any_uri("  http://example.org/a b  ") == "http://example.org/a b"
            && parse_any_uri("urn:isbn:0451450523") == "urn:isbn:0451450523";
        let qn_ok = parse_qname("xs:element").map(|q| q.canonical()).as_deref()
            == Some("xs:element")
            && parse_qname("element").map(|q| q.prefix.clone()) == Some(None)
            && parse_qname("a:b:c").is_none()    // two colons
            && parse_qname("1bad:x").is_none()   // prefix not an NCName
            && parse_qname(":x").is_none()       // empty prefix
            && parse_notation("img:png").is_some();
        if uri_ok && qn_ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "UriAndQNameLexical",
        "anyURI collapses whitespace and admits URI references; QName/NOTATION accept prefix:local and bare local NCNames and reject two colons or non-NCName parts",
        "W3C XSD 1.1 Part 2 §3.3.17, §3.3.18, §3.3.19 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    UriAndQNameLexical,
    "W3C XSD 1.1 Part 2 §3.3.17, §3.3.18, §3.3.19"
);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hex_binary_basics() {
        assert_eq!(
            parse_hex_binary("DEADBEEF").unwrap(),
            vec![0xDE, 0xAD, 0xBE, 0xEF]
        );
        assert_eq!(
            parse_hex_binary("deadbeef").unwrap(),
            vec![0xDE, 0xAD, 0xBE, 0xEF]
        );
        assert_eq!(canonical_hex_binary(&[0xDE, 0xAD]), "DEAD");
        assert!(parse_hex_binary("XY").is_none());
        assert!(parse_hex_binary("F").is_none());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn base64_basics() {
        assert_eq!(parse_base64_binary("TWFu").unwrap(), b"Man");
        assert_eq!(canonical_base64_binary(b"Man"), "TWFu");
        assert_eq!(canonical_base64_binary(b"M"), "TQ==");
        assert_eq!(canonical_base64_binary(b"Ma"), "TWE=");
        assert_eq!(parse_base64_binary("TWE=").unwrap(), b"Ma");
        assert!(parse_base64_binary("TR==").is_none()); // non-canonical padding
        assert!(parse_base64_binary("====").is_none());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn any_uri_collapses() {
        assert_eq!(parse_any_uri("  a  b "), "a b");
        assert_eq!(parse_any_uri("http://x/y"), "http://x/y");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn qname_structure() {
        let q = parse_qname("xs:string").unwrap();
        assert_eq!(q.prefix.as_deref(), Some("xs"));
        assert_eq!(q.local, "string");
        assert_eq!(q.canonical(), "xs:string");
        assert!(parse_qname("noPrefix").unwrap().prefix.is_none());
        assert!(parse_qname("a:b:c").is_none());
        assert!(parse_qname("9:x").is_none());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn axiom_hex() {
        assert!(HexBinaryRoundTrips.verify().is_ok());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn axiom_base64() {
        assert!(Base64BinaryRoundTrips.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_uri_qname() {
        assert!(UriAndQNameLexical.verify().is_ok());
    }

    proptest! {
        /// hexBinary: canonical(bytes) re-parses to the same bytes, and
        /// canonical is uppercase (§3.3.15, Appendix E).
        #[test]
        fn prop_hex_round_trip(bytes in prop::collection::vec(any::<u8>(), 0..64)) {
            let canon = canonical_hex_binary(&bytes);
            prop_assert!(canon.bytes().all(|c| c.is_ascii_digit() || (b'A'..=b'F').contains(&c)));
            prop_assert_eq!(parse_hex_binary(&canon), Some(bytes));
        }

        /// base64Binary: canonical(bytes) re-parses to the same bytes
        /// and re-canonicalizes to itself (§3.3.16, RFC 2045).
        #[test]
        fn prop_base64_round_trip(bytes in prop::collection::vec(any::<u8>(), 0..64)) {
            let canon = canonical_base64_binary(&bytes);
            let back = parse_base64_binary(&canon).expect("canonical base64 re-parses");
            prop_assert_eq!(&back, &bytes);
            prop_assert_eq!(canonical_base64_binary(&back), canon);
        }
    }

    pr4xis::register_praxis_value!(prop_hex_round_trip, Deterministic);
    pr4xis::register_praxis_value!(prop_base64_round_trip, Deterministic);
}
