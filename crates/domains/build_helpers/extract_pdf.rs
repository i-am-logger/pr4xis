//! Build-time PDF → text extraction helper.
//!
//! Mirrors the runtime extractor at
//! `crates/domains/src/social/software/binary/pdf/extract.rs`
//! (M4.γ Phase 6) but is a focused text-only subset that can run
//! from `build.rs` without depending on the crate being built.
//! The runtime extractor carries the property-test surface;
//! this file's tests cover the build-time path independently.
//!
//! The outcome of extraction is a [`PdfExtractOutcome`] enum
//! whose variants the build script translates into corresponding
//! `PdfBuildExtraction` codegen constants (defined at
//! `crates/domains/src/applied/data_provisioning/build_extraction.rs`).
//! Both enums are typed and exhaustive — no `Option<&str>` gaps.
//!
//! Both extractors use the same underlying `lopdf` byte parser
//! and content-stream operator semantics (ISO 32000-2:2020 §9.4),
//! so output for the same PDF + same lopdf version is byte-equal
//! across runtime and build-time paths.
//!
//! Spec references:
//!
//! - ISO 32000-2:2020 §7.5 — file structure.
//! - §7.8.2 — content streams.
//! - §9.4.3 — text-showing operators (Tj, TJ, ', ").
//! - Annex D.5 — WinAnsiEncoding (the only standard encoding
//!   for which lopdf publicly dispatches a table at this version).

use std::path::Path;

/// Outcome of a build-time PDF extraction.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfExtractOutcome {
    /// PDF parsed, text-showing operators decoded successfully.
    Extracted(String),
    /// PDF file at the given path doesn't exist on disk.
    NotOnDisk,
    /// PDF was on disk but couldn't be parsed.
    ParseFailed(String),
    /// PDF parsed but encryption blocked decode.
    Encrypted,
}

/// Extract plain text from a PDF on disk by walking every page's
/// content stream and concatenating the bytes of Tj / TJ / ' / "
/// operators decoded through WinAnsiEncoding.
///
/// This is a focused build-time subset: no font-encoding
/// resolution, no image flagging, no section slicing. The
/// runtime extractor (Phase 6) does all of those. For text
/// from a govinfo-style USCODE PDF whose fonts ship the actual
/// Latin glyphs at WinAnsi codepoints, this is sufficient.
pub fn extract_pdf_to_text(path: &Path) -> PdfExtractOutcome {
    if !path.exists() {
        return PdfExtractOutcome::NotOnDisk;
    }
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => return PdfExtractOutcome::ParseFailed(format!("read {path:?}: {e}")),
    };
    extract_pdf_bytes(&bytes)
}

/// Extract plain text from PDF bytes in memory.
pub fn extract_pdf_bytes(bytes: &[u8]) -> PdfExtractOutcome {
    let doc = match lopdf::Document::load_mem(bytes) {
        Ok(d) => d,
        Err(e) => return PdfExtractOutcome::ParseFailed(format!("{e}")),
    };
    if doc.is_encrypted() {
        return PdfExtractOutcome::Encrypted;
    }

    let pages = doc.get_pages();
    let mut out = String::new();
    let mut first_page = true;
    for &page_id in pages.values() {
        let content = match doc.get_page_content(page_id) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let parsed = match lopdf::content::Content::decode(&content) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if !first_page {
            out.push_str("\n\n");
        }
        first_page = false;
        for op in &parsed.operations {
            match op.operator.as_str() {
                "Tj" | "'" => {
                    if let Some(bytes) = op.operands.first().and_then(string_bytes) {
                        out.push_str(&decode_winansi(&bytes));
                    }
                    out.push(' ');
                }
                "\"" => {
                    if let Some(bytes) = op.operands.get(2).and_then(string_bytes) {
                        out.push_str(&decode_winansi(&bytes));
                    }
                    out.push(' ');
                }
                "TJ" => {
                    if let Some(lopdf::Object::Array(items)) = op.operands.first() {
                        for item in items {
                            if let Some(bytes) = string_bytes(item) {
                                out.push_str(&decode_winansi(&bytes));
                            }
                        }
                        out.push(' ');
                    }
                }
                _ => {}
            }
        }
    }

    PdfExtractOutcome::Extracted(out)
}

fn string_bytes(o: &lopdf::Object) -> Option<Vec<u8>> {
    match o {
        lopdf::Object::String(bytes, _) => Some(bytes.clone()),
        _ => None,
    }
}

fn decode_winansi(bytes: &[u8]) -> String {
    lopdf::Encoding::SimpleEncoding(b"WinAnsiEncoding")
        .bytes_to_string(bytes)
        .unwrap_or_else(|_| {
            // Fall back to lossy Latin-1 for any byte the WinAnsi
            // dispatcher refuses; the goal at build time is best-
            // effort embedding, not the runtime's strict typed-
            // fail-closed behavior.
            bytes.iter().map(|&b| b as char).collect()
        })
}

/// Escape a `&str` for use inside a Rust raw string literal that
/// the build script emits. We use `r#"..."#` quoting to avoid the
/// need to backslash-escape arbitrary content; the only thing we
/// must guarantee is that `"#` doesn't appear in the input.
pub fn escape_for_raw_string(s: &str) -> String {
    // Replace any `"#` sequence with `"\u{0023}` (the `#` rendered
    // as a Unicode escape) so the raw string boundary can't be
    // forged. Idempotent on inputs that don't contain `"#`.
    s.replace("\"#", "\"\\u{0023}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic 1-page PDF programmatically via lopdf
    /// so the extractor has a known-shape input to verify
    /// against. Mirrors the test fixture pattern used in
    /// `src/social/software/binary/pdf/extract.rs` (Phase 6).
    fn pdf_with_text(text: &str) -> Vec<u8> {
        use lopdf::{Document, Object, Stream, dictionary};
        let mut doc = Document::with_version("1.4");
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
            "Encoding" => "WinAnsiEncoding",
        });
        let cs = format!("BT\n/F1 12 Tf\n({text}) Tj\nET\n");
        let content_id = doc.add_object(Stream::new(dictionary! {}, cs.into_bytes()));
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
            "Resources" => dictionary! {
                "Font" => dictionary! { "F1" => font_id },
            },
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).expect("serialize");
        bytes
    }

    // ── Unit tests ───────────────────────────────────────────────

    #[test]
    fn extract_winansi_ascii_text() {
        let bytes = pdf_with_text("Hello world");
        match extract_pdf_bytes(&bytes) {
            PdfExtractOutcome::Extracted(text) => {
                assert!(text.contains("Hello world"));
            }
            other => panic!("expected Extracted; got {other:?}"),
        }
    }

    #[test]
    fn missing_file_returns_not_on_disk() {
        let result =
            extract_pdf_to_text(std::path::Path::new("/tmp/definitely_does_not_exist.pdf"));
        assert_eq!(result, PdfExtractOutcome::NotOnDisk);
    }

    #[test]
    fn malformed_bytes_return_parse_failed() {
        let result = extract_pdf_bytes(b"not a PDF at all");
        match result {
            PdfExtractOutcome::ParseFailed(_) => {}
            other => panic!("expected ParseFailed; got {other:?}"),
        }
    }

    #[test]
    fn escape_for_raw_string_preserves_normal_text() {
        let s = "normal text with quotes \" and hashes # but no boundary";
        assert_eq!(escape_for_raw_string(s), s);
    }

    #[test]
    fn escape_for_raw_string_neutralizes_boundary_sequence() {
        // The raw string here uses double-hash boundaries `r##"..."##`
        // because the test content contains the single-hash boundary
        // sequence `"#` that the production code is supposed to
        // neutralize.
        let s = r##"text with "# inside"##;
        let escaped = escape_for_raw_string(s);
        assert!(!escaped.contains("\"#"));
        assert!(escaped.contains("\"\\u{0023}"));
    }

    #[test]
    fn escape_for_raw_string_is_idempotent_when_no_boundary() {
        let s = "plain text";
        assert_eq!(escape_for_raw_string(&escape_for_raw_string(s)), s);
    }

    // ── Property-based ───────────────────────────────────────────

    use proptest::prelude::*;

    /// Generate printable-ASCII text safe to embed in a PDF
    /// string literal (matches Phase 6's generator constraints).
    fn arb_safe_text() -> impl Strategy<Value = String> {
        proptest::collection::vec(0x20u8..=0x7Eu8, 0..32).prop_map(|bytes| {
            bytes
                .into_iter()
                .filter(|b| !matches!(*b, b'(' | b')' | b'\\'))
                .map(|b| b as char)
                .collect()
        })
    }

    proptest! {
        /// ASCII text round-trips through the build-time
        /// extractor: PDF-encoded then extracted yields the
        /// original text as a substring of the output.
        #[test]
        fn prop_ascii_round_trips_through_build_extractor(text in arb_safe_text()) {
            let bytes = pdf_with_text(&text);
            match extract_pdf_bytes(&bytes) {
                PdfExtractOutcome::Extracted(out) => {
                    prop_assert!(
                        out.contains(&text) || text.is_empty(),
                        "extracted {out:?} should contain {text:?}",
                    );
                }
                other => {
                    return Err(proptest::test_runner::TestCaseError::fail(format!(
                        "expected Extracted; got {other:?}"
                    )));
                }
            }
        }

        /// Extraction is deterministic: same PDF bytes → identical
        /// output across repeated calls.
        #[test]
        fn prop_extract_is_deterministic(text in arb_safe_text()) {
            let bytes = pdf_with_text(&text);
            let r1 = extract_pdf_bytes(&bytes);
            let r2 = extract_pdf_bytes(&bytes);
            prop_assert_eq!(r1, r2);
        }

        /// `escape_for_raw_string` is idempotent on inputs that
        /// don't contain the boundary sequence — `f(f(s)) == f(s)`
        /// for those inputs.
        #[test]
        fn prop_escape_idempotent_on_clean_input(
            s in "[A-Za-z0-9 .,;:!?/\\\\]{0,64}",
        ) {
            prop_assume!(!s.contains("\"#"));
            let once = escape_for_raw_string(&s);
            let twice = escape_for_raw_string(&once);
            prop_assert_eq!(once, twice);
        }

        /// `escape_for_raw_string` always produces a string that
        /// never contains the raw-string-closing sequence `"#`,
        /// regardless of input.
        #[test]
        fn prop_escape_eliminates_boundary_sequence(
            s in "[\\x20-\\x7E]{0,64}",
        ) {
            let escaped = escape_for_raw_string(&s);
            prop_assert!(!escaped.contains("\"#"));
        }

        /// Multi-page PDF extraction concatenates pages with
        /// `\n\n` separators. n pages of identical text yield an
        /// output where the text appears n times.
        #[test]
        fn prop_extract_handles_multi_page_count(n in 1u8..6) {
            use lopdf::{Document, Object, Stream, dictionary};
            let mut doc = Document::with_version("1.4");
            let font_id = doc.add_object(dictionary! {
                "Type" => "Font",
                "Subtype" => "Type1",
                "BaseFont" => "Helvetica",
                "Encoding" => "WinAnsiEncoding",
            });
            let pages_id = doc.new_object_id();
            let mut kids = Vec::new();
            for _ in 0..n {
                let cs = b"BT\n/F1 12 Tf\n(marker) Tj\nET\n".to_vec();
                let content_id = doc.add_object(Stream::new(dictionary! {}, cs));
                let p = doc.add_object(dictionary! {
                    "Type" => "Page",
                    "Parent" => pages_id,
                    "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
                    "Contents" => content_id,
                    "Resources" => dictionary! {
                        "Font" => dictionary! { "F1" => font_id },
                    },
                });
                kids.push(Object::Reference(p));
            }
            let pages = dictionary! {
                "Type" => "Pages",
                "Kids" => kids,
                "Count" => n as i64,
            };
            doc.objects.insert(pages_id, Object::Dictionary(pages));
            let catalog = doc.add_object(dictionary! {
                "Type" => "Catalog",
                "Pages" => pages_id,
            });
            doc.trailer.set("Root", catalog);
            let mut bytes = Vec::new();
            doc.save_to(&mut bytes).expect("serialize");

            match extract_pdf_bytes(&bytes) {
                PdfExtractOutcome::Extracted(text) => {
                    let occurrences = text.matches("marker").count();
                    prop_assert_eq!(occurrences, n as usize);
                }
                other => {
                    return Err(proptest::test_runner::TestCaseError::fail(format!(
                        "expected Extracted; got {other:?}"
                    )));
                }
            }
        }

        /// Adversarial: arbitrary bytes through the build-time
        /// extractor must never panic — either return a typed
        /// outcome (Extracted, NotOnDisk, ParseFailed, Encrypted)
        /// or fail gracefully. Build scripts are not allowed to
        /// crash, so this is a hard invariant.
        #[test]
        fn prop_arbitrary_bytes_never_panic(
            bytes in proptest::collection::vec(any::<u8>(), 0..512),
        ) {
            let outcome = extract_pdf_bytes(&bytes);
            // The outcome must be one of the four typed variants;
            // matching exhaustively is the proof of totality.
            match outcome {
                PdfExtractOutcome::Extracted(_)
                | PdfExtractOutcome::NotOnDisk
                | PdfExtractOutcome::ParseFailed(_)
                | PdfExtractOutcome::Encrypted => {}
            }
        }

        /// Adversarial: a truncated PDF (valid header, missing
        /// body / xref) must produce ParseFailed (or Extracted
        /// with empty text if lopdf is lenient), never panic.
        #[test]
        fn prop_truncated_pdf_never_panics(cutoff_pct in 0u8..90) {
            let bytes = pdf_with_text("test content");
            let cutoff = (bytes.len() * cutoff_pct as usize) / 100;
            let truncated = &bytes[..cutoff];
            match extract_pdf_bytes(truncated) {
                PdfExtractOutcome::Extracted(_)
                | PdfExtractOutcome::ParseFailed(_) => {}
                other => {
                    return Err(proptest::test_runner::TestCaseError::fail(format!(
                        "unexpected variant on truncated input: {other:?}"
                    )));
                }
            }
        }
    }
}
