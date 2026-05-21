//! USLM runtime corpus — the domain-projected typed aggregate of a
//! loaded USLM XML title.
//!
//! Not to be confused with "USLM ontology": the proper USLM
//! ontology is the XSD ontology (formal::meta::xsd) projected
//! through the loaded USLM-1.0.18.xsd. The types in this module are
//! the RUNTIME AGGREGATES — domain-meaningful projections of the
//! parsed XML, held as Rust structs until xsd-parser-types upstream
//! gains serde derives (held PR) at which point this module
//! transitions to holding only the corpus-level UsCode struct and
//! delegating field-level types to xsd-parser-generated output.
//!
//! Citation: LRC USLM XML User Guide §V (USC structure); 1 U.S.C.
//! § 204 (USC authority); W3C XSD 1.1 Part 1 §3.3.6 (substitution
//! groups — the grounding for ContainerKind / SubdivisionKind /
//! UsCodeAdditionalContainer dispatch via from_xsd_element).
//!
//! ## Layer position
//!
//! `UsCode` is the root corpus aggregate: every U.S. Code section
//! loaded from USLM XML at build time. Parallel to
//! [`crate::cognitive::linguistics::english::English`] for WordNet.
//!
//! ```text
//! USLM XML  ─►  pr4xis::codegen::usc_corpus  ─►  CodegenData<UsCode>
//!                                                       │
//!                                                       ▼
//!                                                UsCode::from_codegen
//! ```
//!
//! Identical shape to the English/WordNet pipeline; the phantom marker
//! on `CodegenData` is the local `UsCode` type so handles emitted for
//! USC sections cannot accidentally be used to index into an English
//! corpus and vice versa.
//!
//! ## Literature
//!
//! - U.S. House Office of the Law Revision Counsel, *USLM XML User
//!   Guide (USLM-1.0.18.xsd)*. <https://uscode.house.gov/uslm/>.
//! - 1 U.S.C. § 204 — *Codes and Supplements; positive law titles*.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use hashbrown::HashMap;

use pr4xis::codegen_data::CodegenData;

use crate::formal::meta::identifier_format::Identifier;
use crate::formal::meta::identifier_format::ontology::IdentifierFormatConcept;

pub mod identifiers;
pub mod kinds;
pub mod namespaces;
pub mod runtime_types;

pub use identifiers::{UsCodeTitleId, UsCodeTitleIdError};
pub use kinds::{
    ContainerKind, InlineKind, SubdivisionKind, UsCodeAdditionalContainer, UsCodeAmendmentKind,
    UsCodeFormElement, UsCodeHeadingVariant, UsCodeLegislativeFormula, UsCodeNoteKind,
    UsCodeQuotedVariant, UsCodeTableCellKind,
};
pub use namespaces::{DUBLIN_CORE_NAMESPACE_URI, USLM_NAMESPACE_URI, XHTML_NAMESPACE_URI};
pub use runtime_types::{
    HierarchyNode, UsCodeAmendmentMarkup, UsCodeContainer, UsCodeContinuation, UsCodeDate,
    UsCodeDefBlock, UsCodeHeader, UsCodeInlineRun, UsCodeMarker, UsCodeMeta, UsCodeMetaProperty,
    UsCodeName, UsCodeNote, UsCodeNotesBlock, UsCodeProviso, UsCodeQuotedContent, UsCodeRef,
    UsCodeSection, UsCodeSectionRef, UsCodeSignature, UsCodeSourceCredit, UsCodeSubdivision,
    UsCodeTable, UsCodeTableCell, UsCodeTableRow, UsCodeTerm, UsCodeTitle, UsCodeToc,
    UsCodeTocItem, UslmReadError,
};

/// Build-time aggregate `CodegenData<UsCode>` emitted by
/// `crates/domains/build.rs::write_usc_corpus_codegen`. Empty stub
/// when no USC title XML is on disk. Test-only — accessed via
/// [`UsCode::cached_full`].
// The LRC's USLM source preserves zero-width and soft-hyphen Unicode
// characters that appear verbatim in published statute text. They are
// part of the authoritative bytes; stripping them would diverge from
// the source. Suppress clippy's invisible-character lint at the module
// boundary so the lint can still catch hand-written code elsewhere in
// the crate. Mirrors the pattern in
// `social::compliance::statutes::us_code::title_18` /
// `title_49` / `title_28`.
#[cfg(test)]
#[allow(dead_code, clippy::invisible_characters)]
mod full_corpus {
    include!(concat!(env!("OUT_DIR"), "/usc_corpus_codegen.rs"));
}

/// One USC section.
///
/// Granularity is intentionally the `<section>` element — every USC
/// `<section>` produces exactly one `UscSection`. The internal
/// subdivision hierarchy (`<subsection>` / `<paragraph>` / …) is
/// flattened into the section body text per the M4.ε.3 scope. A
/// future enhancement may surface those as typed substructure.
#[derive(Debug, Clone)]
pub struct UscSection {
    /// Typed USLM URN (e.g. `/us/usc/t18/s1514A`). Grammar-validated at
    /// build time by the codegen emitter; const-constructed here.
    pub urn: Identifier,
    /// Section heading text, English (LRC USLM `<heading>`).
    pub heading: String,
    /// Section body text — chapeau + content text concatenated for
    /// every container within the section. Empty if the source had no
    /// `<chapeau>` / `<content>` (e.g. a placeholder reservation).
    pub text: String,
}

impl UscSection {
    /// Title number derived from the URN (e.g. `/us/usc/t18/...` → 18).
    /// Returns `None` if the URN doesn't match the canonical
    /// `/us/usc/t<N>/...` shape.
    pub fn title_number(&self) -> Option<u32> {
        let v = self.urn.value();
        let segments: Vec<&str> = v.trim_start_matches('/').split('/').collect();
        if segments.first().copied() != Some("us") || segments.get(1).copied() != Some("usc") {
            return None;
        }
        segments
            .get(2)
            .and_then(|s| s.strip_prefix('t'))?
            .parse()
            .ok()
    }
}

/// The loaded U.S. Code corpus.
///
/// Materialised by [`UsCode::from_codegen`] from the build-time
/// [`CodegenData<UsCode>`] static. The phantom marker on the data
/// type is this same `UsCode` struct, so the codegen and runtime
/// types are kept aligned at compile time.
#[derive(Debug)]
pub struct UsCode {
    sections: Vec<UscSection>,
    by_urn: HashMap<String, usize>,
}

impl UsCode {
    /// Functor: `CodegenData<UsCode>` → `UsCode`.
    ///
    /// Mirrors [`crate::cognitive::linguistics::language::from_codegen`]
    /// for English/WordNet. Walks the static slices and materialises a
    /// runtime corpus with URN indexing.
    pub fn from_codegen(data: &CodegenData<UsCode>) -> Self {
        let mut sections = Vec::with_capacity(data.entity_count);
        let mut by_urn = HashMap::with_capacity(data.entity_count);
        for i in 0..data.entity_count {
            let urn_str = data.entity_ids[i];
            // Codegen has already grammar-validated the URN.
            let urn = Identifier::from_codegen_static(IdentifierFormatConcept::UslmUrn, urn_str);
            let heading = data.entity_labels[i].to_string();
            let text = data.entity_defs[i].to_string();
            by_urn.insert(urn_str.to_string(), i);
            sections.push(UscSection { urn, heading, text });
        }
        Self { sections, by_urn }
    }

    /// Look up a section by typed URN. Returns `None` if the
    /// identifier's format isn't USLM URN, or no section with that URN
    /// is loaded.
    pub fn section_by_urn(&self, urn: &Identifier) -> Option<&UscSection> {
        if urn.format != IdentifierFormatConcept::UslmUrn {
            return None;
        }
        self.by_urn.get(urn.value()).map(|&i| &self.sections[i])
    }

    /// All loaded sections, in load order (title-then-section).
    pub fn all_sections(&self) -> &[UscSection] {
        &self.sections
    }

    /// Number of loaded sections.
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// Test-only accessor for the full corpus emitted by
    /// `crates/domains/build.rs`'s `write_usc_corpus_codegen` step.
    /// Cached behind a `OnceLock` so the ~2770-section corpus is
    /// materialised once per test process. Used by Layer 3
    /// corpus-wide gap audits that need every registered USC
    /// title's section headings (not just the two synthetic
    /// sections in [`UsCode::sample`]).
    ///
    /// Mirrors
    /// [`crate::social::judicial::statute_structure::english_adjunction::test_helpers::cached_english`]
    /// — same OnceLock pattern, build-time codegen instead of
    /// runtime XML parsing.
    #[cfg(test)]
    pub fn cached_full() -> &'static Self {
        use std::sync::OnceLock;
        static FULL: OnceLock<UsCode> = OnceLock::new();
        FULL.get_or_init(|| UsCode::from_codegen(&full_corpus::CODEGEN_DATA))
    }

    /// Minimal sample U.S. Code corpus for testing — two synthetic
    /// sections (Title 18 § 1514A and Title 49 § 42121). Mirrors the
    /// fixture pattern used by [`crate::cognitive::linguistics::english::English::sample`]
    /// for unit tests that exercise a downstream consumer without
    /// requiring the full LRC USLM XML to be present on disk.
    pub fn sample() -> Self {
        static SAMPLE_DATA: CodegenData<UsCode> = CodegenData {
            entity_count: 2,
            entity_ids: &["/us/usc/t18/s1514A", "/us/usc/t49/s42121"],
            entity_kind: &["section", "section"],
            entity_labels: &[
                "Civil action to protect against retaliation in fraud cases",
                "Whistleblower protection program",
            ],
            entity_defs: &[
                "No company may discriminate against an employee who provides information about fraud.",
                "No air carrier may discriminate against an employee who provides information about an air-safety violation.",
            ],
            word_index: &[],
            taxonomy: &[],
            mereology: &[],
            opposition: &[],
            equivalence: &[],
            causation: &[],
            references: &[],
        };
        Self::from_codegen(&SAMPLE_DATA)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Static fixture mirroring the shape of codegen output for two
    /// synthetic sections — verifies `from_codegen` builds an indexed
    /// corpus without going through actual XML.
    static FIXTURE_DATA: CodegenData<UsCode> = CodegenData {
        entity_count: 2,
        entity_ids: &["/us/usc/t18/s1514A", "/us/usc/t49/s42121"],
        entity_kind: &["section", "section"],
        entity_labels: &[
            "Civil action to protect against retaliation in fraud cases",
            "Whistleblower protection program",
        ],
        entity_defs: &[
            "No company may discriminate.",
            "Discharge or otherwise discriminate.",
        ],
        word_index: &[],
        taxonomy: &[],
        mereology: &[],
        opposition: &[],
        equivalence: &[],
        causation: &[],
        references: &[],
    };

    #[test]
    fn from_codegen_materialises_every_section() {
        let usc = UsCode::from_codegen(&FIXTURE_DATA);
        assert_eq!(usc.section_count(), 2);
        assert_eq!(usc.all_sections().len(), 2);
    }

    #[test]
    fn section_urn_is_typed_uslm() {
        let usc = UsCode::from_codegen(&FIXTURE_DATA);
        let s = &usc.all_sections()[0];
        assert_eq!(s.urn.format, IdentifierFormatConcept::UslmUrn);
        assert_eq!(s.urn.value(), "/us/usc/t18/s1514A");
    }

    #[test]
    fn section_by_urn_finds_loaded_section() {
        let usc = UsCode::from_codegen(&FIXTURE_DATA);
        let urn =
            Identifier::from_codegen_static(IdentifierFormatConcept::UslmUrn, "/us/usc/t49/s42121");
        let s = usc.section_by_urn(&urn).expect("section present");
        assert_eq!(s.heading, "Whistleblower protection program");
    }

    #[test]
    fn section_by_urn_rejects_non_uslm_identifier() {
        let usc = UsCode::from_codegen(&FIXTURE_DATA);
        // CURIE-format identifier with the same string value must not
        // resolve — the typed format guards against mixing URN with
        // CURIE-shaped strings that happen to match textually.
        let curie = Identifier::curie("sox_1514a:a").expect("CURIE valid");
        assert!(usc.section_by_urn(&curie).is_none());
    }

    #[test]
    fn title_number_extracts_from_urn() {
        let usc = UsCode::from_codegen(&FIXTURE_DATA);
        assert_eq!(usc.all_sections()[0].title_number(), Some(18));
        assert_eq!(usc.all_sections()[1].title_number(), Some(49));
    }
}
