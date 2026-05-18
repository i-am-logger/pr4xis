//! U.S. Code titles — whole-title statute corpora embedded as
//! compile-time `const` data.
//!
//! Each registered USC title (`[sources.usc_title_N]` in
//! `praxis.toml`) is parsed at build time by
//! [`pr4xis::codegen::uslm::generate_title_module_source`] into a
//! single Rust module emitted at `$OUT_DIR/usc_title_N_codegen.rs`.
//! The per-title `mod.rs` `include!`s it to expose every section's
//! structural ontology data as compile-time constants.
//!
//! Whole-title default — matches the WordNet/English pattern:
//! the entire corpus is embedded, not selected entries. Looking up
//! a statute means looking up its USLM identifier in the static
//! section table; the binary cost is paid once per title, not per
//! statute.
//!
//! ## Static type layout
//!
//! The codegen output references the [`StaticStatute`],
//! [`StaticTerm`], and [`StaticRelation`] types declared here.
//! All fields are `&'static str` so the entire structural data
//! lives in `.rodata` with zero allocation.
//!
//! ```ignore
//! pub static SECTIONS: &[StaticStatute] = &[
//!     StaticStatute {
//!         identifier: "/us/usc/t18/s1",
//!         num: "1",
//!         heading: "Repealed.",
//!         terms: &[],
//!         relations: &[],
//!     },
//!     StaticStatute {
//!         identifier: "/us/usc/t18/s1514A",
//!         num: "1514A",
//!         heading: "Civil action to protect against retaliation in fraud cases",
//!         terms: &[
//!             StaticTerm { id: "sox_1514a:a", name: "...", definition: "..." },
//!             // ...
//!         ],
//!         relations: &[
//!             StaticRelation {
//!                 from: "sox_1514a:a_1",
//!                 to: "sox_1514a:a",
//!                 kind: StaticRelationKind::Composes,
//!             },
//!         ],
//!     },
//! ];
//! ```
//!
//! Citation: LRC, *USLM XML User Guide*; 1 U.S.C. § 204.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, vec::Vec};

/// One statute embedded in a title — frozen const data per the
/// `English::CodegenData` precedent: raw `&'static str` storage at
/// the codegen layer, typed-value access via the public API.
///
/// Mirrors `pr4xis::codegen_data::CodegenData` (which uses
/// `entity_ids: &'static [&'static str]` for the same reason): the
/// storage IS the codegen output; consumers go through accessors
/// that return typed values (Identifier with format
/// IdentifierFormatConcept::UslmUrn for [`identifier_urn`], etc.).
///
/// [`identifier_urn`]: Self::identifier_urn
#[derive(Debug, Clone, Copy)]
pub struct StaticStatute {
    /// USLM URN raw string. Public only because the codegen output
    /// constructs StaticStatute via struct-literal syntax inside
    /// the `us_code::codegen` sub-module. Access via
    /// [`identifier_urn`][Self::identifier_urn] (returns typed
    /// `Identifier`) for type-safe consumers.
    #[doc(hidden)]
    pub identifier: &'static str,
    #[doc(hidden)]
    pub num: &'static str,
    #[doc(hidden)]
    pub heading: &'static str,
    #[doc(hidden)]
    pub terms: &'static [StaticTerm],
    #[doc(hidden)]
    pub relations: &'static [StaticRelation],
}

impl StaticStatute {
    /// USLM URN as a typed `Identifier` (format `UslmUrn`). Validates
    /// against the LRC User Guide §V grammar via
    /// [`crate::formal::meta::identifier_format::Identifier::uslm_urn`].
    ///
    /// LRC pl-119-90 USC titles include a small number of sections
    /// whose `identifier` attribute is a space-separated multi-URN
    /// (combined repealed-section ranges). For those, this method
    /// returns the validation error; use [`identifier_raw`] for the
    /// verbatim attribute value, or [`urns`] (typed Vec) for the
    /// constituent URNs when modeling that LRC convention.
    ///
    /// [`identifier_raw`]: Self::identifier_raw
    /// [`urns`]: Self::urns
    pub fn identifier_urn(
        &self,
    ) -> Result<
        crate::formal::meta::identifier_format::Identifier,
        crate::formal::meta::identifier_format::IdentifierParseError,
    > {
        crate::formal::meta::identifier_format::Identifier::uslm_urn(self.identifier)
    }

    /// Verbatim USLM `identifier` attribute. For single-URN sections
    /// this equals the result of [`identifier_urn`]'s `.value`; for
    /// LRC's multi-URN combined-range sections it's the
    /// space-separated string. The raw form exists because the LRC
    /// publishes both shapes; downstream callers that don't care
    /// about typing this discrimination use the raw form.
    ///
    /// [`identifier_urn`]: Self::identifier_urn
    pub fn identifier_raw(&self) -> &'static str {
        self.identifier
    }

    /// The `<num>` value verbatim (e.g. `"1514A"`). USLM does not
    /// constrain this token to a specific grammar — sections use
    /// arabic digits, letters, and digit-letter combinations.
    /// LITERATURE_GAP("a published ontology for USC section-number
    /// tokens with grammar"); for now exposed as a raw English text
    /// token in the section-number lexical role.
    pub fn num(&self) -> &'static str {
        self.num
    }

    /// `<heading>` plain text, in English (BCP-47 `en` per the LRC's
    /// `<uscDoc xml:lang="en">` declaration).
    pub fn heading(&self) -> &'static str {
        self.heading
    }

    /// Terms in this section (subsection-level + below).
    pub fn terms(&self) -> &'static [StaticTerm] {
        self.terms
    }

    /// Composes-mereology edges between this section's terms.
    pub fn relations(&self) -> &'static [StaticRelation] {
        self.relations
    }
}

/// One subdivision-term inside a statute. Storage is raw
/// `&'static str` per the [`StaticStatute`] precedent; typed-value
/// access is via the accessor methods.
#[derive(Debug, Clone, Copy)]
pub struct StaticTerm {
    #[doc(hidden)]
    pub id: &'static str,
    #[doc(hidden)]
    pub name: &'static str,
    #[doc(hidden)]
    pub definition: &'static str,
}

impl StaticTerm {
    /// The term's CURIE as a typed `Identifier` (format `Curie`).
    /// Validates the `prefix:local` shape per W3C CURIE Syntax 1.0 §2.
    pub fn id_curie(
        &self,
    ) -> Result<
        crate::formal::meta::identifier_format::Identifier,
        crate::formal::meta::identifier_format::IdentifierParseError,
    > {
        crate::formal::meta::identifier_format::Identifier::curie(self.id)
    }

    /// Verbatim CURIE string. Used by codegen-side renames in
    /// [`StaticStatute::to_statute`] that rewrite the URN-derived
    /// static prefix to the runtime statute name.
    pub fn id_raw(&self) -> &'static str {
        self.id
    }

    /// Term name — the section's `<heading>` text or a URN-derived
    /// subdivision marker. English text per LRC's `xml:lang="en"`.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Term definition — `<chapeau>`/`<content>` text with heading
    /// fallback. English text.
    pub fn definition(&self) -> &'static str {
        self.definition
    }
}

/// One Composes edge between terms. Both endpoints are CURIEs into
/// the same statute's term table.
#[derive(Debug, Clone, Copy)]
pub struct StaticRelation {
    #[doc(hidden)]
    pub from: &'static str,
    #[doc(hidden)]
    pub to: &'static str,
    #[doc(hidden)]
    pub kind: StaticRelationKind,
}

impl StaticRelation {
    /// The `from` endpoint as a typed CURIE `Identifier`.
    pub fn from_curie(
        &self,
    ) -> Result<
        crate::formal::meta::identifier_format::Identifier,
        crate::formal::meta::identifier_format::IdentifierParseError,
    > {
        crate::formal::meta::identifier_format::Identifier::curie(self.from)
    }

    /// The `to` endpoint as a typed CURIE `Identifier`.
    pub fn to_curie(
        &self,
    ) -> Result<
        crate::formal::meta::identifier_format::Identifier,
        crate::formal::meta::identifier_format::IdentifierParseError,
    > {
        crate::formal::meta::identifier_format::Identifier::curie(self.to)
    }

    /// Verbatim `from` CURIE.
    pub fn from_raw(&self) -> &'static str {
        self.from
    }

    /// Verbatim `to` CURIE.
    pub fn to_raw(&self) -> &'static str {
        self.to
    }

    /// Relation kind.
    pub fn kind(&self) -> StaticRelationKind {
        self.kind
    }
}

/// The kinded relation. Currently only Composes (mereology); the
/// USLM-derived structural data uses exclusively containment. Other
/// relation kinds may be added when downstream codegen passes lift
/// cross-references or burden-shifting into the static data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticRelationKind {
    Composes,
}

impl StaticStatute {
    /// Materialize a runtime [`super::Statute`] from this static
    /// data. Allocates on each call — callers that need a
    /// `&'static Statute` should cache via `OnceLock`.
    ///
    /// The section-root term (the entry whose CURIE has no local
    /// part) is filtered out because praxis CURIEs require a
    /// `prefix:local` shape; the § identity is carried by
    /// [`super::Statute::name()`], not by a term. Matches the
    /// semantics of [`super::from_uslm_section`].
    ///
    /// CURIE prefixes are rewritten from the static data's
    /// URN-derived prefix (e.g. `us_usc_t18_s1514a`) to the
    /// caller-supplied `statute_name` (e.g. `sox_1514a`). This
    /// lets a downstream module name a statute whatever fits the
    /// praxis registry while still reading the structural data
    /// from the title-level USLM source.
    pub fn to_statute(&self, statute_name: &str, version: &str) -> super::Statute {
        use crate::applied::data_provisioning::registry::{
            StructuralData, StructuralRelation, StructuralTerm,
        };
        use crate::formal::meta::identifier_format::Identifier;

        // CURIE-prefix rename: the codegen emits URN-derived prefixes
        // (e.g. `us_usc_t18_s1514a:a_1`) that the downstream runtime
        // module rebinds to its praxis-registry name (`sox_1514a`).
        // The first term's typed CURIE supplies the static prefix;
        // typed via Identifier::curie so the prefix is the URN-derived
        // namespace, not an ad-hoc string parse.
        let static_prefix: String = self
            .terms()
            .iter()
            .filter_map(|t| t.id_curie().ok())
            .next()
            .map(|id| id.value.split(':').next().unwrap_or("").to_string())
            .unwrap_or_default();

        let rename_curie = |raw: &str| -> String {
            match Identifier::curie(raw) {
                Ok(id) => {
                    if let Some(rest) = id.value.strip_prefix(&static_prefix)
                        && let Some(local) = rest.strip_prefix(':')
                    {
                        format!("{statute_name}:{local}")
                    } else {
                        id.value
                    }
                }
                // Non-CURIE rows (e.g. the section-root entry) pass
                // through unchanged; downstream filtering drops them.
                Err(_) => raw.to_string(),
            }
        };

        let data = StructuralData {
            description: format!("USLM source: {}", self.identifier_raw()),
            terms: self
                .terms()
                .iter()
                .filter(|t| t.id_curie().is_ok())
                .map(|t| StructuralTerm {
                    id: rename_curie(t.id_raw()),
                    name: t.name().to_string(),
                    definition: t.definition().to_string(),
                    lemmas: Vec::new(),
                })
                .collect(),
            relations: self
                .relations()
                .iter()
                .filter(|r| r.from_curie().is_ok() && r.to_curie().is_ok())
                .map(|r| StructuralRelation {
                    from: rename_curie(r.from_raw()),
                    to: rename_curie(r.to_raw()),
                    relation: match r.kind() {
                        StaticRelationKind::Composes => "Composes",
                    }
                    .to_string(),
                })
                .collect(),
        };

        // USLM-derived: provenance points at the section URN so
        // downstream consumers can trace each term back to the LRC
        // source, not the praxis-lock shim that wraps the legacy
        // hand-curated structural data.
        super::Statute::from_structural_with_context(
            statute_name,
            version,
            &data,
            self.identifier_raw(),
        )
        .expect("StaticStatute data must be valid (codegen-checked)")
    }
}

/// O(N) linear search for a section by USLM identifier. For ~1,400
/// sections this is comfortably under microseconds; if profiling
/// shows hotspot, swap for a phf::Map at codegen time.
///
/// Matches on the verbatim identifier attribute — for single-URN
/// sections this is the URN; for LRC's combined-range sections it's
/// the space-separated multi-URN string. Callers with a single
/// typed `Identifier` use [`find_section_by_urn`].
pub fn find_section<'a>(
    sections: &'a [StaticStatute],
    identifier: &str,
) -> Option<&'a StaticStatute> {
    sections.iter().find(|s| s.identifier_raw() == identifier)
}

/// Find a section by typed USLM URN `Identifier`. Matches against
/// the section's `identifier_urn()` for single-URN sections;
/// combined-range sections (whose raw `identifier` is a
/// whitespace-separated URN list) are not currently matched here —
/// they live in [`find_section`]'s raw-string path until the
/// multi-URN ontology lands.
pub fn find_section_by_urn<'a>(
    sections: &'a [StaticStatute],
    urn: &crate::formal::meta::identifier_format::Identifier,
) -> Option<&'a StaticStatute> {
    sections
        .iter()
        .find(|s| s.identifier_urn().is_ok_and(|got| got == *urn))
}

pub mod title_18;
pub mod title_49;

use crate::social::software::markup::xml::uslm::ontology::UsCodeTitleId;

/// Generic title-and-identifier dispatch. The caller passes a typed
/// [`UsCodeTitleId`] (no stringly-typed title-name parameter); the
/// function routes to the right SECTIONS array internally.
///
/// Returns `None` if the title isn't registered or the section isn't
/// present in that title.
pub fn section(title: &UsCodeTitleId, identifier: &str) -> Option<&'static StaticStatute> {
    let sections = sections_for_title(title)?;
    find_section(sections, identifier)
}

/// Lookup the embedded SECTIONS array for a given title.
///
/// This is the single dispatch point that the per-title sub-modules
/// feed into — when a new USC title is registered in `praxis.toml`,
/// adding it here is the only Rust-code change required (plus the
/// title's own `pub mod` declaration above).
fn sections_for_title(title: &UsCodeTitleId) -> Option<&'static [StaticStatute]> {
    match title.number() {
        18 => Some(title_18::SECTIONS),
        49 => Some(title_49::SECTIONS),
        _ => None,
    }
}

/// Every registered USC title, in title-number order. Useful for
/// global walks (e.g. "every published section across every loaded
/// title").
pub fn all_titles() -> &'static [(u32, &'static [StaticStatute])] {
    static TITLES: &[(u32, &[StaticStatute])] =
        &[(18, title_18::SECTIONS), (49, title_49::SECTIONS)];
    TITLES
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal::meta::identifier_format::Identifier;
    use crate::formal::meta::identifier_format::ontology::IdentifierFormatConcept;

    #[test]
    fn static_statute_identifier_urn_returns_typed_uslm_urn() {
        let t18 = UsCodeTitleId::try_from_number(18).unwrap();
        let s = section(&t18, "/us/usc/t18/s1514A").expect("SOX 1514A present");
        let id = s.identifier_urn().expect("SOX 1514A is a valid USLM URN");
        assert_eq!(id.format, IdentifierFormatConcept::UslmUrn);
        assert_eq!(id.value, "/us/usc/t18/s1514A");
    }

    #[test]
    fn static_term_id_curie_returns_typed_curie() {
        let t18 = UsCodeTitleId::try_from_number(18).unwrap();
        let s = section(&t18, "/us/usc/t18/s1514A").expect("SOX 1514A");
        let term = s
            .terms()
            .iter()
            .find(|t| t.id_curie().is_ok())
            .expect("section has ≥1 CURIE-shaped term");
        let id = term.id_curie().expect("term id parses as CURIE");
        assert_eq!(id.format, IdentifierFormatConcept::Curie);
    }

    #[test]
    fn static_relation_curie_endpoints_form_a_subset() {
        // The codegen emits relations including some whose endpoints
        // are not CURIEs (section-root entries with no local part).
        // The invariant: at least one relation has both endpoints
        // parsing as CURIE — the to_statute filter relies on that.
        let t18 = UsCodeTitleId::try_from_number(18).unwrap();
        let s = section(&t18, "/us/usc/t18/s1514A").expect("SOX 1514A");
        let both_curie_count = s
            .relations()
            .iter()
            .filter(|r| r.from_curie().is_ok() && r.to_curie().is_ok())
            .count();
        assert!(
            both_curie_count > 0,
            "§ 1514A must emit at least one relation with both endpoints in CURIE form"
        );
    }

    #[test]
    fn find_section_by_urn_matches_typed_identifier() {
        let t18 = UsCodeTitleId::try_from_number(18).unwrap();
        let sections = sections_for_title(&t18).expect("Title 18 registered");
        let urn = Identifier::uslm_urn("/us/usc/t18/s1514A").unwrap();
        let found = find_section_by_urn(sections, &urn).expect("SOX 1514A by URN");
        assert_eq!(found.identifier_raw(), "/us/usc/t18/s1514A");
    }

    #[test]
    fn static_statute_corpus_identifier_grammar_check_title_18() {
        // Every single-URN section's identifier must parse as a
        // typed USLM URN. Combined-range sections (whitespace-
        // separated multi-URN) are skipped — same convention as the
        // top-level tests.
        for s in title_18::SECTIONS.iter() {
            if s.identifier_raw().contains(' ') {
                continue;
            }
            assert!(
                s.identifier_urn().is_ok(),
                "Title 18 single-URN section identifier `{}` must parse",
                s.identifier_raw()
            );
        }
    }
}
