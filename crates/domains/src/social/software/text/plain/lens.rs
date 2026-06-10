//! [`PlainTextLens`] — the byte-exact, graph-faithful well-behaved lens
//! for UTF-8 plain text (M4.ι / #186, Phase 1).
//!
//! `get` decomposes the bytes into a [`PlainTextDocument`] recording
//! every byte-affecting concrete-syntax decision (leading BOM, per-line
//! terminator, final incomplete line); `put` replays them to reproduce
//! the input byte-for-byte. The lens therefore satisfies the strict
//! byte-exact PutGet law `put(get(b)) == b` with NO constant-complement,
//! and declares [`RoundTripFidelity::ByteExactGraphFaithful`].
//!
//! `canonical` keeps the existing NFKC + LF-fold + BOM-strip canonical
//! form (`canonical::plain_text`, UAX #15) untouched, so any
//! `[canonical_signatures]` pin still verifies; the byte-exact law is the
//! strictly-stronger guarantee layered above it.
//!
//! Registration with the round-trip harness (`register_lens!`) is
//! deferred until a real plain-text source is added to `praxis.toml`
//! under a `SourceTaxonomy` plain-text leaf; the law is proven directly
//! in `tests` meanwhile.

use alloc::string::String;
use alloc::vec::Vec;

use super::ontology::{ByteOrderMark, LineTerminator, PlainTextDocument, PlainTextLine};
use crate::formal::meta::well_behaved_lens::canonical::{self, CanonicalizationError};
use crate::formal::meta::well_behaved_lens::{RoundTripFidelity, WellBehavedLens};

/// The byte-exact plain-text lens. See the module documentation.
pub struct PlainTextLens;

const FORM: &str = "plain-text-byte-exact";

impl WellBehavedLens for PlainTextLens {
    type Target = PlainTextDocument;
    type Error = CanonicalizationError;

    const FIDELITY: RoundTripFidelity = RoundTripFidelity::ByteExactGraphFaithful;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        let s = core::str::from_utf8(bytes)
            .map_err(|e| CanonicalizationError::new(FORM, alloc::format!("non-UTF-8: {}", e)))?;
        // A single leading U+FEFF is the BOM signature; any further
        // U+FEFF is ordinary content (Unicode BOM FAQ).
        let (bom, body) = match s.strip_prefix('\u{FEFF}') {
            Some(rest) => (ByteOrderMark::Utf8, rest),
            None => (ByteOrderMark::Absent, s),
        };
        Ok(PlainTextDocument {
            bom,
            lines: split_lines(body),
        })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        let mut out = Vec::new();
        if target.bom == ByteOrderMark::Utf8 {
            // U+FEFF encoded as UTF-8 is EF BB BF (Unicode BOM FAQ);
            // the compiler derives those bytes from the code point.
            out.extend_from_slice("\u{FEFF}".as_bytes());
        }
        for line in &target.lines {
            out.extend_from_slice(line.content.as_bytes());
            if let Some(terminator) = line.terminator {
                out.extend_from_slice(terminator.as_bytes());
            }
        }
        Ok(out)
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        canonical::plain_text::canonicalize(bytes)
    }
}

/// Split `body` into lines, recording each line's exact terminator.
///
/// Splits on CR, LF and CRLF (Unicode §5.8 Table 5-1); NEL/LS/PS stay in
/// content. Trailing content with no terminator becomes a final
/// incomplete line (`terminator: None`, POSIX.1-2017 §3.195) — note an
/// incomplete line has, by that definition, one or more characters, so a
/// body ending in a terminator yields no trailing line.
fn split_lines(body: &str) -> Vec<PlainTextLine> {
    let mut lines = Vec::new();
    let mut content = String::new();
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\n' => lines.push(PlainTextLine {
                content: core::mem::take(&mut content),
                terminator: Some(LineTerminator::Lf),
            }),
            '\r' => {
                let terminator = if chars.peek() == Some(&'\n') {
                    chars.next();
                    LineTerminator::CrLf
                } else {
                    LineTerminator::Cr
                };
                lines.push(PlainTextLine {
                    content: core::mem::take(&mut content),
                    terminator: Some(terminator),
                });
            }
            other => content.push(other),
        }
    }
    if !content.is_empty() {
        lines.push(PlainTextLine {
            content,
            terminator: None,
        });
    }
    lines
}
