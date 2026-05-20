//! Plain-text canonicalization per Unicode Technical Report #15
//! Normalization Forms (Davis & Whistler 2024, Unicode Consortium,
//! <https://www.unicode.org/reports/tr15/>), specifically NFKC per
//! §6, with three additional shape rules layered on top:
//!
//!   1. **BOM strip.** A leading U+FEFF byte-order-mark is removed.
//!      Per Unicode TR #15 a BOM does not change the represented
//!      text; we strip it so a BOM'd and a non-BOM'd encoding of
//!      the same text share a canonical form.
//!   2. **Line-ending normalization.** CRLF (`\r\n`) and bare CR
//!      (`\r`) are folded to LF (`\n`), matching the
//!      WHATWG Encoding Standard §3.4 line-feed convention and
//!      preventing the round-trip from being sensitive to whether
//!      a loader returns Windows or Unix newlines.
//!   3. **NFKC** is applied last so any compatibility decomposition
//!      that introduces CR or BOM (which it never does in TR #15
//!      §6, but the ordering is defensive) wouldn't bypass the
//!      shape rules.
//!
//! No BiDi controls / line-break opportunities are touched — those
//! are display-layer concerns not part of the textual content per
//! TR #15.

use alloc::string::String;
use alloc::vec::Vec;

use unicode_normalization::UnicodeNormalization;

use super::CanonicalizationError;

const FORM: &str = "plain-text-tr15-nfkc";

/// Canonicalize `bytes` as plain UTF-8 text.
///
/// Returns the NFKC + LF-normalized + BOM-stripped UTF-8 bytes.
/// Errors only if the input is not valid UTF-8.
pub fn canonicalize(bytes: &[u8]) -> Result<Vec<u8>, CanonicalizationError> {
    let s = core::str::from_utf8(bytes)
        .map_err(|e| CanonicalizationError::new(FORM, alloc::format!("non-UTF-8: {}", e)))?;
    // Strip all leading BOMs in a loop. A document re-fed through
    // canonicalize must yield the same output (idempotence); a
    // single-pass strip would leave the second BOM in
    // `\u{FEFF}\u{FEFF}…` and break idempotence on the second
    // canonicalization. Interior U+FEFF (deprecated ZWNBSP usage) is
    // preserved as a regular character.
    let mut stripped = s;
    while let Some(rest) = stripped.strip_prefix('\u{FEFF}') {
        stripped = rest;
    }
    // Normalize line endings: CRLF → LF, then bare CR → LF.
    let mut lf_normalized = String::with_capacity(stripped.len());
    let mut chars = stripped.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next(); // consume LF
            }
            lf_normalized.push('\n');
        } else {
            lf_normalized.push(c);
        }
    }
    // Apply NFKC. `nfkc()` returns an iterator of char; collect to
    // String.
    let nfkc: String = lf_normalized.nfkc().collect();
    Ok(nfkc.into_bytes())
}
