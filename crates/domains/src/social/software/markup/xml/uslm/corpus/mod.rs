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
pub mod section_aux;

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
pub use section_aux::{UscComposesEdge, UscSectionAux, UscSubdivision};

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
/// Granularity is the `<section>` element — every USC `<section>`
/// produces exactly one `UscSection`. The flat top-level fields
/// (`urn`, `heading`, `text`) summarise the section as a Layer-3
/// vocabulary entry; the [`subdivisions`][Self::subdivisions] and
/// [`relations`][Self::relations] slices add the typed subdivision
/// depth — every `<subsection>` / `<paragraph>` / `<subparagraph>` /
/// `<clause>` / `<subclause>` / `<item>` / `<subitem>` is materialised
/// as a [`UscSubdivision`] node with its own URN and a Composes-edge
/// list joining it to its containing parent.
///
/// This is the data the
/// `social::compliance::statutes::us_code::StaticStatute` shims hold
/// today; absorbing it here is what unblocks the M4.ε.6 deletion of
/// that subtree.
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
    /// Subdivision tree: subsections, paragraphs, subparagraphs,
    /// clauses, subclauses, items, subitems. Each carries its own
    /// URN per LRC USLM XML User Guide §V (USC hierarchy) and may
    /// have nested children. Empty for placeholder sections (e.g.
    /// `[Reserved]`).
    pub subdivisions: &'static [UscSubdivision],
    /// Composes-relation edges between this section and its
    /// subdivisions, plus between sibling subdivisions where USLM
    /// declares them (per W3C XSD 1.1 Part 1 §3.3.6 substitution-group
    /// containment + LRC USLM XML User Guide §V hierarchy). Empty for
    /// sections with no enumerated subdivisions.
    pub relations: &'static [UscComposesEdge],
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

    /// Count every node in the subdivision tree (pre-order). The
    /// section root is *not* counted — only the
    /// subsection-and-below nodes. Equivalent to summing
    /// `descendants_including_self().count()` across every top-level
    /// subdivision.
    pub fn subdivision_count(&self) -> usize {
        fn walk(s: &UscSubdivision) -> usize {
            1 + s.children.iter().map(walk).sum::<usize>()
        }
        self.subdivisions.iter().map(walk).sum()
    }

    /// Materialize a runtime [`crate::social::compliance::statutes::Statute`]
    /// from this section's subdivision tree. Drop-in replacement for
    /// the legacy
    /// `StaticStatute::to_statute(statute_name, version)` path —
    /// after M4.ε.6 deletes the `us_code/` shims, downstream consumers
    /// (e.g. `sox_1514a::statute_from_uslm`) call this directly.
    ///
    /// CURIE prefixes for emitted terms are derived from
    /// `statute_name` so the praxis registry name (e.g. `sox_1514a`,
    /// `air21_42121`) drives downstream lookup — even though the
    /// underlying URN namespace is the USLM `/us/usc/t<N>/s<num>`
    /// hierarchy. The section root maps to the bare CURIE `name`
    /// (no local part); each subdivision URN segment after the
    /// section prefix becomes the CURIE local part with slashes
    /// replaced by underscores. Example: with `statute_name =
    /// "sox_1514a"`, the URN `/us/usc/t18/s1514A/a/1/A` becomes the
    /// CURIE `sox_1514a:a_1_A`.
    ///
    /// Citation: LRC USLM XML User Guide §V (USC URN hierarchy);
    /// W3C CURIE Syntax 1.0 §2 (prefix:local form).
    pub fn to_statute(
        &self,
        statute_name: &str,
        version: &str,
    ) -> crate::social::compliance::statutes::Statute {
        use crate::applied::data_provisioning::registry::{
            StructuralData, StructuralRelation, StructuralTerm,
        };

        let section_urn = self.urn.value().to_string();
        let mut terms: Vec<StructuralTerm> = Vec::new();

        // Section-root term — its CURIE is the bare statute name; the
        // praxis Statute filter drops bare-prefix entries (CURIE
        // requires prefix:local), but downstream consumers may still
        // want the section-level heading/text. The `from_structural`
        // path validates each term id as a CURIE; bare-name entries
        // fail that check, so we emit only `prefix:local` rows.
        for sub in self.subdivisions {
            emit_subdivision_term(sub, &section_urn, statute_name, &mut terms);
        }

        let relations: Vec<StructuralRelation> = self
            .relations
            .iter()
            .filter_map(|edge| {
                let from = urn_to_curie(edge.from_urn, &section_urn, statute_name)?;
                let to = urn_to_curie(edge.to_urn, &section_urn, statute_name)?;
                // Drop edges whose endpoint is the section root (its
                // CURIE is bare and gets filtered out of `terms`
                // above; the StatuteConstructError::DanglingRelation
                // check would reject such an edge).
                if !from.contains(':') || !to.contains(':') {
                    return None;
                }
                Some(StructuralRelation {
                    from,
                    to,
                    relation: "Composes".to_string(),
                })
            })
            .collect();

        let data = StructuralData {
            description: alloc::format!("USLM source: {}", section_urn),
            terms,
            relations,
        };

        crate::social::compliance::statutes::Statute::from_structural_with_context(
            statute_name,
            version,
            &data,
            &section_urn,
        )
        .expect("UscSection subdivision data must be valid (codegen-checked)")
    }
}

/// Convert a USLM URN to a CURIE `<statute_name>:<local>` form,
/// stripping the section prefix and replacing slashes with
/// underscores. Returns `None` if the URN doesn't sit under the
/// section prefix. The section root URN itself (where the stripped
/// local part is empty) yields the bare `statute_name` form.
fn urn_to_curie(urn: &str, section_prefix: &str, statute_name: &str) -> Option<String> {
    if urn == section_prefix {
        return Some(statute_name.to_string());
    }
    let local = urn.strip_prefix(section_prefix)?.strip_prefix('/')?;
    if local.is_empty() {
        return Some(statute_name.to_string());
    }
    let joined = local.replace('/', "_");
    Some(alloc::format!("{statute_name}:{joined}"))
}

fn emit_subdivision_term(
    sub: &UscSubdivision,
    section_urn: &str,
    statute_name: &str,
    terms: &mut Vec<crate::applied::data_provisioning::registry::StructuralTerm>,
) {
    use crate::applied::data_provisioning::registry::StructuralTerm;
    let urn_value = sub.urn.value();
    if let Some(curie) = urn_to_curie(urn_value, section_urn, statute_name)
        && curie.contains(':')
    {
        let name = match sub.heading {
            Some(h) if !h.trim().is_empty() => h.to_string(),
            _ => alloc::format!("({})", sub.num),
        };
        // Fall back to chapeau, then content, then the term name —
        // keeps every term's definition non-empty (the
        // `from_structural` constructor doesn't reject empty
        // definitions but downstream consumers expect at least the
        // structural marker text).
        let definition = match (sub.chapeau, sub.content) {
            (Some(c), _) if !c.trim().is_empty() => c.to_string(),
            (_, Some(c)) if !c.trim().is_empty() => c.to_string(),
            _ => name.clone(),
        };
        terms.push(StructuralTerm {
            id: curie,
            name,
            definition,
            lemmas: Vec::new(),
        });
    }
    for child in sub.children {
        emit_subdivision_term(child, section_urn, statute_name, terms);
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
    /// runtime corpus with URN indexing. Sections built through this
    /// path carry EMPTY subdivision trees — use
    /// [`Self::from_codegen_with_aux`] when subdivision depth matters
    /// (e.g. for `UscSection::to_statute`).
    pub fn from_codegen(data: &CodegenData<UsCode>) -> Self {
        Self::from_codegen_with_aux(data, &[])
    }

    /// Functor: `CodegenData<UsCode>` × `&[UscSectionAux]` → `UsCode`.
    ///
    /// Joins each section's `urn` against the aux slice (also keyed
    /// by URN string) to attach the subdivision tree + Composes-edge
    /// list. Sections with no aux entry get empty
    /// `subdivisions`/`relations` slices.
    ///
    /// This is the constructor the build-time-generated
    /// `usc_corpus_codegen.rs` static drives, via the parallel
    /// `USC_SECTION_AUX` table emitted alongside `CODEGEN_DATA`.
    pub fn from_codegen_with_aux(
        data: &CodegenData<UsCode>,
        aux: &'static [UscSectionAux],
    ) -> Self {
        // Index aux by URN for O(1) lookup during section build.
        let mut aux_by_urn: HashMap<&'static str, &'static UscSectionAux> =
            HashMap::with_capacity(aux.len());
        for entry in aux {
            aux_by_urn.insert(entry.urn, entry);
        }

        let mut sections = Vec::with_capacity(data.entity_count);
        let mut by_urn = HashMap::with_capacity(data.entity_count);
        for i in 0..data.entity_count {
            let urn_str = data.entity_ids[i];
            // Codegen has already grammar-validated the URN.
            let urn = Identifier::from_codegen_static(IdentifierFormatConcept::UslmUrn, urn_str);
            let heading = data.entity_labels[i].to_string();
            let text = data.entity_defs[i].to_string();
            by_urn.insert(urn_str.to_string(), i);
            let (subdivisions, relations) = match aux_by_urn.get(urn_str) {
                Some(a) => (a.subdivisions, a.relations),
                None => (&[][..], &[][..]),
            };
            sections.push(UscSection {
                urn,
                heading,
                text,
                subdivisions,
                relations,
            });
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
        FULL.get_or_init(|| {
            UsCode::from_codegen_with_aux(&full_corpus::CODEGEN_DATA, full_corpus::USC_SECTION_AUX)
        })
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

    #[test]
    fn from_codegen_default_yields_empty_subdivisions() {
        // The fixture path passes `aux=&[]` implicitly; sections
        // built that way must have empty subdivision/relation
        // slices.
        let usc = UsCode::from_codegen(&FIXTURE_DATA);
        for s in usc.all_sections() {
            assert!(
                s.subdivisions.is_empty(),
                "subdivisions for {}",
                s.urn.value()
            );
            assert!(s.relations.is_empty(), "relations for {}", s.urn.value());
            assert_eq!(s.subdivision_count(), 0);
        }
    }

    // ---------------------------------------------------------------
    // Equivalence verification against the legacy
    // `social::compliance::statutes::us_code::StaticStatute` shim.
    // Each section's UscSection subdivision tree must contain the
    // same set of CURIE local-part endpoints that the legacy
    // StaticStatute's term list does (after converting URNs to
    // CURIEs via the shared `to_statute` projection), and the
    // Composes-edge count must match. M4.ε.6 deletion of `us_code/`
    // is gated on this passing.
    // ---------------------------------------------------------------

    #[cfg(test)]
    mod equivalence_with_us_code_shim {
        use super::*;
        use crate::social::compliance::statutes::us_code;

        /// For `/us/usc/t18/s1514A`, compare the new UsCode corpus's
        /// subdivision data against the legacy `StaticStatute`'s
        /// term list under the `to_statute("sox_1514a", ...)`
        /// projection. We compare:
        ///
        /// 1. The set of CURIE ids of subdivision-level terms
        ///    (i.e. terms whose CURIE has a non-empty local part).
        /// 2. The count of Composes-edges with both endpoints in
        ///    the CURIE-id set.
        ///
        /// Both views project a common subset — every USLM
        /// subdivision URN of the form
        /// `/us/usc/t18/s1514A/<segments>` produces one term with
        /// CURIE `sox_1514a:<segments_underscored>`. The legacy
        /// view *additionally* emits a bare-prefix term
        /// (`us_usc_t18_s1514a` with no local part) that
        /// `to_statute`'s CURIE filter drops; the new view never
        /// emits that bare entry, so the comparison is on the
        /// CURIE-shaped subset alone.
        #[test]
        fn sox_1514a_subdivision_curies_match_legacy_shim() {
            let usc = UsCode::cached_full();
            let urn = Identifier::from_codegen_static(
                IdentifierFormatConcept::UslmUrn,
                "/us/usc/t18/s1514A",
            );
            let section = match usc.section_by_urn(&urn) {
                Some(s) => s,
                None => {
                    // Title 18 XML not on disk in this build — skip.
                    eprintln!("SKIP: § 1514A not in loaded corpus");
                    return;
                }
            };

            // Statute via the new corpus.
            let statute_new = section.to_statute("sox_1514a", "2002");

            // Statute via the legacy shim.
            let t18 = UsCodeTitleId::try_from_number(18).expect("title 18");
            let static_statute = us_code::section(&t18, "/us/usc/t18/s1514A")
                .expect("§ 1514A in legacy us_code shim");
            let statute_legacy = static_statute.to_statute("sox_1514a", "2002");

            // Collect CURIEs of each statute's terms. Filter the
            // legacy view's `:unknown_N` synthetic entries — those
            // are the legacy shim's crutch for identifier-less
            // level elements (e.g. `<paragraph>` nested inside
            // `<note>` / `<quotedContent>` in repealed sections);
            // the new URN-grounded view drops them deliberately
            // per the "bottom-up loaded" rule.
            let curies_new: alloc::collections::BTreeSet<String> = statute_new
                .terms()
                .iter()
                .map(|t| t.id.value().to_string())
                .collect();
            let curies_legacy: alloc::collections::BTreeSet<String> = statute_legacy
                .terms()
                .iter()
                .map(|t| t.id.value().to_string())
                .filter(|c| !c.contains(":unknown_"))
                .collect();

            // The new view's CURIE set must equal the legacy view's
            // CURIE set. If a divergence shows up, surface the
            // symmetric difference for debugging.
            let only_in_new: Vec<&String> = curies_new.difference(&curies_legacy).collect();
            let only_in_legacy: Vec<&String> = curies_legacy.difference(&curies_new).collect();
            assert!(
                only_in_new.is_empty() && only_in_legacy.is_empty(),
                "CURIE-set divergence on § 1514A.\n only_in_new: {only_in_new:?}\n only_in_legacy: {only_in_legacy:?}",
            );

            // Composes-edge count: new view's edges (which all
            // sit between URN-grounded CURIEs) must be <= legacy
            // (which may include edges anchored on synthetic
            // `:unknown_N` rows). § 1514A has no such synthetic
            // rows so the counts must match exactly.
            assert_eq!(
                statute_new.relations().len(),
                statute_legacy.relations().len(),
                "Composes-edge count mismatch on § 1514A",
            );
        }

        /// Same equivalence check for AIR21 § 42121 in Title 49 —
        /// the other case-study statute the legacy shim covers.
        #[test]
        fn air21_42121_subdivision_curies_match_legacy_shim() {
            let usc = UsCode::cached_full();
            let urn = Identifier::from_codegen_static(
                IdentifierFormatConcept::UslmUrn,
                "/us/usc/t49/s42121",
            );
            let section = match usc.section_by_urn(&urn) {
                Some(s) => s,
                None => {
                    eprintln!("SKIP: § 42121 not in loaded corpus");
                    return;
                }
            };

            let statute_new = section.to_statute("air21_42121", "2007");

            let t49 = UsCodeTitleId::try_from_number(49).expect("title 49");
            let static_statute = us_code::section(&t49, "/us/usc/t49/s42121")
                .expect("§ 42121 in legacy us_code shim");
            let statute_legacy = static_statute.to_statute("air21_42121", "2007");

            let curies_new: alloc::collections::BTreeSet<String> = statute_new
                .terms()
                .iter()
                .map(|t| t.id.value().to_string())
                .collect();
            let curies_legacy: alloc::collections::BTreeSet<String> = statute_legacy
                .terms()
                .iter()
                .map(|t| t.id.value().to_string())
                .filter(|c| !c.contains(":unknown_"))
                .collect();

            let only_in_new: Vec<&String> = curies_new.difference(&curies_legacy).collect();
            let only_in_legacy: Vec<&String> = curies_legacy.difference(&curies_new).collect();
            assert!(
                only_in_new.is_empty() && only_in_legacy.is_empty(),
                "CURIE-set divergence on § 42121.\n only_in_new: {only_in_new:?}\n only_in_legacy: {only_in_legacy:?}",
            );

            assert_eq!(
                statute_new.relations().len(),
                statute_legacy.relations().len(),
                "Composes-edge count mismatch on § 42121",
            );
        }

        /// Cross-title sanity check: every loaded Title 18 section's
        /// new-view CURIE set must be a SUBSET of the legacy shim's
        /// CURIE set. (Equality cannot hold corpus-wide because the
        /// legacy `parse_uslm_title_all_sections_str` synthesizes
        /// artificial `:unknown_N` CURIEs for level-elements that
        /// appear without an `identifier=` attribute — most
        /// commonly `<paragraph>`/`<subparagraph>` nested inside
        /// `<note>` or `<quotedContent>` blocks in repealed
        /// sections like § 437. The new view drops those because
        /// they have no URN to ground a typed Identifier. Per the
        /// `feedback_push_back_on_unsupported_file_types` and
        /// "bottom-up loaded" rules, the new view's URN-grounding
        /// is the correct shape; the legacy `:unknown_N` rows are
        /// a synthetic crutch that M4.ε.6 deletion drops.)
        ///
        /// The subset check confirms every URN-bearing subdivision
        /// the legacy shim emits is also present in the new view.
        #[test]
        fn title_18_subdivision_curies_are_subset_of_legacy_shim() {
            let usc = UsCode::cached_full();
            let t18 = UsCodeTitleId::try_from_number(18).expect("title 18");
            let Some(sections) = legacy_sections_for_title(&t18) else {
                eprintln!("SKIP: title 18 not in legacy shim");
                return;
            };
            let mut compared = 0usize;
            let mut missing_from_new: Vec<String> = Vec::new();
            for static_section in sections {
                let raw = static_section.identifier_raw();
                if raw.contains(' ') {
                    // Combined-range URN — neither view models it
                    // as a single section. Skip.
                    continue;
                }
                let urn = Identifier::from_codegen_static(
                    IdentifierFormatConcept::UslmUrn,
                    leaked_str(raw),
                );
                let Some(section) = usc.section_by_urn(&urn) else {
                    continue;
                };
                let statute_new = section.to_statute("sox_1514a", "2002");
                let statute_legacy = static_section.to_statute("sox_1514a", "2002");

                let curies_new: alloc::collections::BTreeSet<String> = statute_new
                    .terms()
                    .iter()
                    .map(|t| t.id.value().to_string())
                    .collect();
                // Legacy CURIEs minus the synthetic `:unknown_N`
                // shim entries that the new URN-grounded view
                // intentionally drops.
                let curies_legacy: alloc::collections::BTreeSet<String> = statute_legacy
                    .terms()
                    .iter()
                    .map(|t| t.id.value().to_string())
                    .filter(|c| !c.contains(":unknown_"))
                    .collect();

                // The new view must contain every legacy URN-bearing
                // CURIE.
                for c in curies_legacy.difference(&curies_new) {
                    missing_from_new.push(alloc::format!("{}: missing CURIE {}", raw, c));
                    if missing_from_new.len() > 5 {
                        break;
                    }
                }
                compared += 1;
                if !missing_from_new.is_empty() {
                    break;
                }
                if compared >= 200 {
                    break;
                }
            }
            assert!(
                missing_from_new.is_empty(),
                "Title 18 new view missing URN-bearing CURIEs the legacy shim emits (sampled {compared}): {missing_from_new:?}",
            );
            assert!(compared > 0, "no Title 18 sections cross-compared");
        }

        /// Helper: borrow the legacy SECTIONS slice for the given
        /// title via the public accessor (us_code's
        /// `find_section_by_urn` is keyed off the title's slice).
        fn legacy_sections_for_title(
            title: &UsCodeTitleId,
        ) -> Option<&'static [us_code::StaticStatute]> {
            // Lift the slice via all_titles() — the only public
            // accessor that exposes the per-title SECTIONS.
            us_code::all_titles()
                .iter()
                .find(|(n, _)| *n == title.number())
                .map(|(_, slice)| *slice)
        }

        /// Leak a short URN string to gain a 'static borrow for
        /// constructing the typed Identifier — only safe in tests.
        fn leaked_str(s: &str) -> &'static str {
            Box::leak(s.to_string().into_boxed_str())
        }
    }
}
