//! 18 U.S.C. — Crimes and Criminal Procedure.
//!
//! Whole-title statute corpus parsed at build time from the LRC's
//! USLM XML at release point pl-119-90. The `include!` below pulls
//! in `$OUT_DIR/usc_title_18_codegen.rs` which defines
//! `pub static SECTIONS: &[StaticStatute] = &[...]` covering every
//! section in the title.
//!
//! Citation: 1 U.S.C. § 204; LRC, *USLM XML User Guide*.
//! Source: `https://uscode.house.gov/download/releasepoints/us/pl/119/90/xml_usc18@119-90.zip`.

#[allow(unused_imports)]
use super::{StaticRelation, StaticRelationKind, StaticStatute, StaticTerm};

// The LRC's USLM source preserves zero-width and soft-hyphen
// Unicode characters that appear verbatim in published statute
// text. They are part of the authoritative bytes; stripping them
// would diverge from the source. Suppress clippy's
// invisible-character lint at the module boundary so the lint can
// still catch hand-written code elsewhere in the crate.
#[allow(clippy::invisible_characters)]
mod codegen {
    use super::*;
    include!(concat!(env!("OUT_DIR"), "/usc_title_18_codegen.rs"));
}
pub use codegen::SECTIONS;

/// Look up a section by USLM identifier.
///
/// Example: `section("/us/usc/t18/s1514A")` returns the SOX
/// whistleblower section.
pub fn section(identifier: &str) -> Option<&'static StaticStatute> {
    super::find_section(SECTIONS, identifier)
}

/// All sections in Title 18, in the order they appear in the LRC
/// publication.
pub fn all_sections() -> &'static [StaticStatute] {
    SECTIONS
}
