//! Byte-exact round-trip tests for [`PlainTextLens`].
//!
//! Each case proves `put(get(b)) == b` byte-for-byte for a class of
//! concrete-syntax decisions, citing the production it covers.

use alloc::string::String;

use proptest::prelude::*;

use super::lens::PlainTextLens;
use super::ontology::{ByteOrderMark, LineTerminator};
use crate::formal::meta::well_behaved_lens::{RoundTripFidelity, WellBehavedLens};

/// Assert the byte-exact PutGet law holds for `bytes`, both via the
/// harness entry point and via a direct `put(get(b))` comparison.
fn assert_byte_exact(bytes: &[u8]) {
    PlainTextLens::assert_byte_exact_law(bytes)
        .unwrap_or_else(|e| panic!("byte-exact law failed on {:?}: {}", bytes, e));
    let doc = PlainTextLens::get(bytes).expect("get");
    let out = PlainTextLens::put(&doc).expect("put");
    assert_eq!(out, bytes, "put(get(b)) != b for {:?}", bytes);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn fidelity_is_byte_exact() {
    assert_eq!(
        PlainTextLens::FIDELITY,
        RoundTripFidelity::ByteExactGraphFaithful
    );
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn empty_input() {
    assert_byte_exact(b"");
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn mixed_line_endings_preserved() {
    // Unix LF, Windows CRLF and classic-Mac CR survive verbatim — the
    // canonical form folds all three to LF (Unicode §5.8), the
    // byte-exact law must not.
    assert_byte_exact(b"unix\nwin\r\nmac\rtail");
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn final_newline_present_and_absent() {
    assert_byte_exact(b"a\nb\n"); // terminated last line
    assert_byte_exact(b"a\nb"); // final incomplete line (POSIX §3.195)
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn empty_lines_preserved() {
    assert_byte_exact(b"a\n\n\nb\n");
}

#[pr4xis::praxis_value(Deterministic, Verifiable)]
#[test]
fn utf8_bom_preserved() {
    let with_bom = "\u{FEFF}hello\n".as_bytes();
    assert_byte_exact(with_bom);
    assert_eq!(
        PlainTextLens::get(with_bom).unwrap().bom,
        ByteOrderMark::Utf8
    );
}

#[pr4xis::praxis_value(Deterministic, Verifiable)]
#[test]
fn no_bom_recorded_as_absent() {
    let no_bom = b"hello\n";
    assert_byte_exact(no_bom);
    assert_eq!(
        PlainTextLens::get(no_bom).unwrap().bom,
        ByteOrderMark::Absent
    );
}

#[pr4xis::praxis_value(Deterministic, Verifiable)]
#[test]
fn only_first_feff_is_bom_rest_is_content() {
    // The first U+FEFF is the signature; a second is ordinary content
    // (Unicode BOM FAQ). Both must round-trip exactly.
    let two = "\u{FEFF}\u{FEFF}x".as_bytes();
    assert_byte_exact(two);
    let doc = PlainTextLens::get(two).unwrap();
    assert_eq!(doc.bom, ByteOrderMark::Utf8);
    assert!(doc.lines[0].content.starts_with('\u{FEFF}'));
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn bom_only_document() {
    assert_byte_exact("\u{FEFF}".as_bytes());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn non_utf8_is_rejected_not_silently_dropped() {
    // A lone 0xFF is not valid UTF-8; get must error rather than lose
    // the byte (a silent drop would violate byte-exactness).
    PlainTextLens::get(&[0xFF]).expect_err("invalid UTF-8 must error");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn terminator_byte_sequences() {
    assert_eq!(LineTerminator::Lf.as_bytes(), b"\n");
    assert_eq!(LineTerminator::Cr.as_bytes(), b"\r");
    assert_eq!(LineTerminator::CrLf.as_bytes(), b"\r\n");
}

proptest! {
    /// Byte-exactness on arbitrary UTF-8: `.*` generates valid UTF-8
    /// strings whose bytes must round-trip exactly.
    #[test]
    fn proptest_byte_exact_arbitrary_utf8(s in ".*") {
        let bytes = s.as_bytes();
        let doc = PlainTextLens::get(bytes).expect("get");
        let out = PlainTextLens::put(&doc).expect("put");
        prop_assert_eq!(out, bytes.to_vec());
    }

    /// Byte-exactness over arbitrary mixes of line endings, optional
    /// BOM, and a possibly-unterminated final line.
    #[test]
    fn proptest_byte_exact_line_mixes(
        bom in any::<bool>(),
        segs in prop::collection::vec(("[^\r\n]{0,8}", 0usize..4), 0..8),
    ) {
        let mut input = String::new();
        if bom {
            input.push('\u{FEFF}');
        }
        for (content, term) in segs {
            input.push_str(&content);
            match term {
                1 => input.push('\n'),
                2 => input.push('\r'),
                3 => input.push_str("\r\n"),
                _ => {} // no terminator — merges with the next segment
            }
        }
        let bytes = input.as_bytes();
        let doc = PlainTextLens::get(bytes).expect("get");
        let out = PlainTextLens::put(&doc).expect("put");
        prop_assert_eq!(out, bytes.to_vec());
    }
}

pr4xis::register_praxis_value!(proptest_byte_exact_arbitrary_utf8, Deterministic);
pr4xis::register_praxis_value!(proptest_byte_exact_line_mixes, Deterministic);
