//! Plain-text / TSV decoder — the typed-record-stream codec for every
//! `ContentType::Plaintext` source.
//!
//! Phase 2 left `ContentType::Plaintext` with NO decoder: a TSV vocabulary
//! source could not satisfy [`DecoderTotalityPerKind`] and so could not be
//! registered or carry a committed `.prx`. This decoder closes that gap — it
//! turns the raw TSV bytes into a generic typed record stream
//! ([`TsvRecords`]), the shape every TSV consumer parses further (the EWMH
//! `_NET_WM_STATE` window-state vocabulary into `Vec<StateBitDef>`, the
//! AGID-derived irregular slice into `Vec<IrregularForm>`, the OLiA→CCG
//! projection into a loaded `Connection`). The decoder is GENERIC — one codec
//! for the whole `Plaintext` content type, never one per file: it parses the
//! line/`\t`-field grid that every praxis TSV shares (the `#`-comment + blank-
//! line convention each consumer already follows) and hands the typed rows back
//! to the consumer's own field interpreter.
//!
//! ## File format (the praxis-TSV convention)
//!
//! UTF-8 text, one record per line. A line whose first non-whitespace
//! character is `#`, or that is blank, is a comment / spacer and is dropped.
//! Every other line is split on the ASCII TAB (`\t`) into fields; leading /
//! trailing ASCII whitespace is trimmed from each field. The record stream is
//! the in-order list of the surviving rows' fields — the structure-preserving
//! image of the file the consumers then map onto their typed records.
//!
//! ## Relationship to the raw-source `.prx` path
//!
//! This decoder is the *parse* half; the *byte-materialization* half is the
//! generalized [`raw_source_prx`] gate (a TSV's committed `.prx` carries the
//! raw bytes, content-address-pinned). A consumer loads
//! `raw_source_text_embedded(name, version, PRX)` → these bytes → `parse(...)`
//! → its typed record. The registry-side entry point [`decode`] is what the
//! [`DecoderTotalityPerKind`] axiom consults via [`super::has_decoder_for`].
//!
//! [`DecoderTotalityPerKind`]: super::super::ontology::DecoderTotalityPerKind
//! [`raw_source_prx`]: crate::applied::data_provisioning::raw_source_prx

#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes — the single declaration of which content type this file
/// decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::Plaintext;

/// One decoded TSV record: the in-order, trimmed `\t`-separated fields of a
/// single non-comment line.
pub type TsvRecord = Vec<String>;

/// The decoded record stream of a whole TSV source — the generic typed image
/// each consumer maps onto its own record type.
pub type TsvRecords = Vec<TsvRecord>;

/// Decode TSV bytes into the generic typed record stream: drop `#`-comment and
/// blank lines, split each surviving line on TAB, trim each field. This is the
/// one structure the `Plaintext` content type decodes to; the per-consumer
/// field interpretation (which column is the lemma, which the spec atom, …)
/// stays with the consumer.
#[must_use]
pub fn parse(text: &str) -> TsvRecords {
    text.lines()
        // `lines()` strips the `\n`; drop any CRLF `\r` residue too.
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .filter(|line| {
            let t = line.trim_start();
            !t.is_empty() && !t.starts_with('#')
        })
        .map(|line| line.split('\t').map(|f| f.trim().to_string()).collect())
        .collect()
}

/// The registry-side decoder entry point — the typed transformation
/// `RawBytes → TsvRecords` the data-provisioning machinery uses to witness
/// "every registered `Plaintext` source has a runtime decoder"
/// ([`DecoderTotalityPerKind`](super::super::ontology::DecoderTotalityPerKind)).
/// Fails closed if the bytes are not UTF-8 (a TSV vocabulary is text by
/// definition).
pub fn decode(bytes: &[u8]) -> Result<TsvRecords, PlaintextTsvError> {
    let text =
        core::str::from_utf8(bytes).map_err(|e| PlaintextTsvError::NotUtf8(e.to_string()))?;
    Ok(parse(text))
}

/// A failure decoding a `Plaintext` source — fail-closed, naming the cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaintextTsvError {
    /// The source bytes are not UTF-8 — a TSV vocabulary must be text.
    NotUtf8(String),
}

impl core::fmt::Display for PlaintextTsvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PlaintextTsvError::NotUtf8(e) => write!(f, "plaintext/TSV source is not UTF-8: {e}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PlaintextTsvError {}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parses_tab_separated_rows() {
        let input = "# a comment\n\
                     fullscreen\t_NET_WM_STATE_FULLSCREEN\tEWMH 1.5 §5\n\
                     \n\
                     above\t_NET_WM_STATE_ABOVE\tEWMH 1.5 §5\n";
        let rows = parse(input);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            vec!["fullscreen", "_NET_WM_STATE_FULLSCREEN", "EWMH 1.5 §5"]
        );
        assert_eq!(rows[1], vec!["above", "_NET_WM_STATE_ABOVE", "EWMH 1.5 §5"]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn comments_and_blanks_dropped() {
        let input = "\n#first\n  # indented comment\nx\ty\n\n";
        let rows = parse(input);
        assert_eq!(rows, vec![vec!["x".to_string(), "y".to_string()]]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fields_are_trimmed_and_crlf_safe() {
        let input = "a \t b\t c\r\n";
        let rows = parse(input);
        assert_eq!(rows, vec![vec!["a", "b", "c"]]);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn decode_rejects_non_utf8_fail_closed() {
        let err = decode(&[0xff, 0xfe, 0x00]).expect_err("invalid UTF-8 must fail");
        assert!(matches!(err, PlaintextTsvError::NotUtf8(_)), "got {err:?}");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn decode_round_trips_utf8_text() {
        let bytes = "k1\tv1\nk2\tv2\n".as_bytes();
        let rows = decode(bytes).expect("valid utf8 decodes");
        assert_eq!(rows.len(), 2);
    }

    proptest! {
        /// TSV ENCODE/DECODE ROUND-TRIP: forall well-formed record stream of
        /// non-empty, tab/newline/hash-free fields, rendering it to the TSV
        /// wire form and decoding recovers the SAME records — the GetPut leg
        /// of the records ⇄ TSV lens. (We render a canonical TSV from the
        /// records: it is the inverse of `parse` on data rows.)
        #[test]
        fn prop_tsv_records_round_trip(
            rows in proptest::collection::vec(
                proptest::collection::vec("[a-zA-Z0-9_§-]{1,12}", 1..5),
                0..40,
            )
        ) {
            // Render the canonical TSV: one `\t`-joined line per record.
            let mut wire = String::new();
            for row in &rows {
                wire.push_str(&row.join("\t"));
                wire.push('\n');
            }
            let decoded = parse(&wire);
            prop_assert_eq!(&decoded, &rows);
            // And the byte-level `decode` agrees with `parse`.
            let via_bytes = decode(wire.as_bytes())
                .map_err(|e| TestCaseError::fail(format!("decode: {e}")))?;
            prop_assert_eq!(via_bytes, rows);
        }

        /// COMMENT/BLANK INVARIANCE: forall data record stream, interleaving
        /// `#`-comment and blank lines NEVER changes the decoded records — the
        /// decoder's structural drop rule is robust to provenance headers
        /// (every praxis TSV carries a citation comment block).
        #[test]
        fn prop_comments_and_blanks_are_invisible(
            rows in proptest::collection::vec(
                proptest::collection::vec("[a-zA-Z0-9_]{1,8}", 1..4),
                1..20,
            ),
            comment in "[a-zA-Z0-9 ]{0,20}",
        ) {
            let mut wire = String::new();
            wire.push_str(&format!("# {comment}\n\n"));
            for row in &rows {
                wire.push_str(&row.join("\t"));
                wire.push('\n');
                // A blank + comment line between every record.
                wire.push('\n');
                wire.push_str(&format!("#{comment}\n"));
            }
            prop_assert_eq!(parse(&wire), rows);
        }

        /// FULL-CHAIN `.prx` ROUND-TRIP: forall well-formed TSV record stream,
        /// rendering it to TSV bytes, wrapping those bytes in the raw-source
        /// `.prx` envelope, loading the envelope back through the fail-closed
        /// content gate, and DECODING the recovered bytes through this codec
        /// yields the SAME records. This composes the records ⇄ TSV lens with the
        /// bytes ⇄ `.prx` lens (`raw_source_prx`) — the exact load path every
        /// registered TSV source uses (`raw_source_text_embedded → parse`), so a
        /// break anywhere in {render, encode, gate, decode} fails HERE.
        #[test]
        fn prop_tsv_round_trips_through_raw_source_prx(
            rows in proptest::collection::vec(
                proptest::collection::vec("[a-zA-Z0-9_§-]{1,12}", 1..5),
                0..40,
            )
        ) {
            use crate::applied::data_provisioning::raw_source_prx::{
                emit_raw_source_prx, load_raw_source_prx_gated, raw_source_archive_address,
            };
            use crate::applied::data_provisioning::registry::LockDigest;

            // Render the canonical TSV bytes from the records.
            let mut wire = String::new();
            for row in &rows {
                wire.push_str(&row.join("\t"));
                wire.push('\n');
            }
            // Wrap → pin → gated load → decode, the registered-source load path.
            let prx = emit_raw_source_prx("plaintext_tsv_prop", "1", wire.as_bytes());
            let pin = LockDigest::address(raw_source_archive_address(&prx));
            let bytes = load_raw_source_prx_gated(&prx, &pin, "plaintext_tsv_prop@1")
                .map_err(|e| TestCaseError::fail(format!("gated load: {e}")))?;
            let decoded = decode(&bytes)
                .map_err(|e| TestCaseError::fail(format!("decode: {e}")))?;
            prop_assert_eq!(decoded, rows);
        }
    }

    pr4xis::register_praxis_value!(prop_tsv_records_round_trip, Deterministic);
    pr4xis::register_praxis_value!(prop_comments_and_blanks_are_invisible, Deterministic);
    pr4xis::register_praxis_value!(
        prop_tsv_round_trips_through_raw_source_prx,
        Deterministic,
        Extensible
    );
}
