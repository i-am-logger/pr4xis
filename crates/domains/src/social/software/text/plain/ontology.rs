//! Plain-text concrete-syntax ontology — the byte-affecting decisions a
//! UTF-8 plain-text document encodes, modeled as first-class typed nodes
//! so the byte-exact lens can reconstruct the exact input bytes from the
//! graph alone (M4.ι / #186) with no constant-complement side-channel.
//!
//! Each node cites the authority that fixes its byte freedom:
//!
//! - [`ByteOrderMark`] — the Unicode BOM (U+FEFF) as a UTF-8 signature.
//! - [`LineTerminator`] — the line-ending code points of Unicode §5.8.
//! - a final incomplete line (no trailing terminator) — POSIX §3.195.
//!
//! Unicode normalization (UAX #15) is deliberately NOT modeled here: the
//! byte-exact lens stores the exact decoded scalar sequence, whose UTF-8
//! re-encoding is unique, so no normalization choice affects the bytes.
//! NFKC belongs to the *canonical* form (`canonical::plain_text`), not
//! the byte layer.
//!
//! ## Citations
//!
//! - **The Unicode Standard, Version 16.0.0**, §5.8 "Newline Guidelines",
//!   Table 5-1 "Hex Values for Acronyms" — CR = U+000D, LF = U+000A,
//!   CRLF = <U+000D U+000A>
//!   (<https://www.unicode.org/versions/Unicode16.0.0/core-spec/chapter-5/>).
//! - **Unicode BOM FAQ** — the UTF-8 encoding of U+FEFF is the byte
//!   sequence EF BB BF (<https://www.unicode.org/faq/utf_bom.html>).
//! - **IEEE Std 1003.1-2017 (POSIX.1-2017)** Base Definitions §3.206
//!   "Line" (a sequence of non-`<newline>` characters plus a terminating
//!   `<newline>`) and §3.195 "Incomplete Line" (non-`<newline>` characters
//!   at end of file)
//!   (<https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html>).

use alloc::string::String;
use alloc::vec::Vec;

/// The byte-order mark at the head of a UTF-8 plain-text document.
///
/// U+FEFF encoded as UTF-8 is the three bytes `EF BB BF` (Unicode BOM
/// FAQ). Only a single leading U+FEFF is treated as the signature; any
/// further U+FEFF is ordinary content (deprecated ZWNBSP usage).
/// Non-UTF-8 BOMs (UTF-16/UTF-32) are out of scope for this UTF-8 lens —
/// such inputs fail to decode and are not byte-exact here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrderMark {
    /// No leading BOM.
    Absent,
    /// A leading UTF-8 BOM (`EF BB BF`).
    Utf8,
}

/// A line terminator.
///
/// The Unicode Standard §5.8 "Newline Guidelines" (Table 5-1) fixes the
/// code points: `CR = U+000D`, `LF = U+000A`, `CRLF = <U+000D, U+000A>`.
/// These three are the platform conventions §5.8 enumerates (Unix = LF,
/// Windows = CRLF, classic Mac OS = CR).
///
/// NEL (U+0085), LS (U+2028) and PS (U+2029) — also listed in §5.8 — are
/// preserved verbatim as line *content* by this lens rather than modeled
/// as boundaries; byte-exactness holds regardless, and promoting them to
/// first-class terminators is a faithful refinement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineTerminator {
    /// Line feed, U+000A (`\n`).
    Lf,
    /// Carriage return, U+000D (`\r`).
    Cr,
    /// Carriage return + line feed, `<U+000D, U+000A>` (`\r\n`).
    CrLf,
}

impl LineTerminator {
    /// The exact bytes this terminator emits. CR/LF are ASCII, so their
    /// UTF-8 encoding is the single byte itself (Unicode §5.8 Table 5-1).
    #[must_use]
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            LineTerminator::Lf => b"\n",
            LineTerminator::Cr => b"\r",
            LineTerminator::CrLf => b"\r\n",
        }
    }
}

/// One line of a plain-text document: its content followed by an
/// optional terminator.
///
/// A `terminator` of `None` marks a final *incomplete line* — content at
/// end-of-file with no trailing newline (POSIX.1-2017 §3.195). Only the
/// last line of a document may have `terminator == None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainTextLine {
    /// The line's exact decoded content (the scalar values between
    /// terminators), stored verbatim — no normalization, so the unique
    /// UTF-8 re-encoding reproduces the original bytes.
    pub content: String,
    /// The terminator that ended this line, or `None` for a final
    /// incomplete line (POSIX.1-2017 §3.195 Incomplete Line).
    pub terminator: Option<LineTerminator>,
}

/// A UTF-8 plain-text document decomposed into the concrete-syntax
/// decisions that fix its exact bytes: a leading [`ByteOrderMark`] and an
/// ordered sequence of [`PlainTextLine`]s, each carrying its own
/// [`LineTerminator`]. Reconstructed byte-for-byte by the lens `put`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainTextDocument {
    /// The leading byte-order mark (or [`ByteOrderMark::Absent`]).
    pub bom: ByteOrderMark,
    /// The document's lines in order. Empty for empty input.
    pub lines: Vec<PlainTextLine>,
}
