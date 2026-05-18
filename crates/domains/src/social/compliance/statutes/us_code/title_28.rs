//! 28 U.S.C. — Judiciary and Judicial Procedure.
//!
//! Whole-title statute corpus parsed at build time from the LRC's
//! USLM XML at release point pl-119-90. Title 28 USC App. holds the
//! Federal Rules of Civil Procedure, Federal Rules of Evidence,
//! Federal Rules of Appellate Procedure, and Federal Rules of
//! Bankruptcy Procedure — the LRC codifies them as appendix
//! subdivisions of Title 28 per 28 U.S.C. § 2072 (Rules Enabling
//! Act). The `include!` below pulls in `$OUT_DIR/usc_title_28_codegen.rs`
//! which defines `pub static SECTIONS: &[StaticStatute] = &[...]`
//! covering every section + appendix subdivision in the title.
//!
//! The loaded sections are the vocabulary the Layer 3 legal-frame
//! resolver in
//! [`crate::social::judicial::statute_structure::statute_understanding`]
//! looks up term names against (FRE 702 Expert Witness, FRCP Rule 17
//! Party Capacity, FRCP Rule 38 Jury, etc.) — populated from the
//! LRC's authoritative XML, not hand-coded.
//!
//! Citation: 1 U.S.C. § 204; 28 U.S.C. § 2072; LRC,
//! *USLM XML User Guide*.
//! Source: `https://uscode.house.gov/download/releasepoints/us/pl/119/90/xml_usc28@119-90.zip`.

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
    include!(concat!(env!("OUT_DIR"), "/usc_title_28_codegen.rs"));
}
pub use codegen::SECTIONS;

/// Look up a section by USLM identifier.
///
/// Example: `section("/us/usc/t28/s1331")` returns the
/// federal-question jurisdiction section.
pub fn section(identifier: &str) -> Option<&'static StaticStatute> {
    super::find_section(SECTIONS, identifier)
}

/// All sections in Title 28, in the order they appear in the LRC
/// publication.
pub fn all_sections() -> &'static [StaticStatute] {
    SECTIONS
}
