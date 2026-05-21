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
/// when no USC title XML is on disk. Materialized at runtime via
/// [`loaded`] (canonical singleton) and at test time via
/// [`UsCode::cached_full`].
// The LRC's USLM source preserves zero-width and soft-hyphen Unicode
// characters that appear verbatim in published statute text. They are
// part of the authoritative bytes; stripping them would diverge from
// the source. Suppress clippy's invisible-character lint at the module
// boundary so the lint can still catch hand-written code elsewhere in
// the crate.
#[allow(dead_code, clippy::invisible_characters)]
mod full_corpus {
    include!(concat!(env!("OUT_DIR"), "/usc_corpus_codegen.rs"));
}

/// The canonical loaded U.S. Code corpus — every section + subdivision
/// from every registered USC title whose USLM XML was on disk at build
/// time. Backed by the codegen output emitted by
/// `crates/domains/build.rs::write_usc_corpus_codegen`, materialised
/// once per process behind a `OnceLock`.
///
/// Replaces the legacy per-title `us_code::title_N::section(...)`
/// dispatch — downstream consumers query the unified corpus by typed
/// USLM URN through [`UsCode::section_by_urn`].
///
/// Citation: 1 U.S.C. § 204 (Code authority); LRC, *USLM XML User
/// Guide* §V (USC URN hierarchy).
pub fn loaded() -> &'static UsCode {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<UsCode> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        UsCode::from_codegen_with_aux(&full_corpus::CODEGEN_DATA, full_corpus::USC_SECTION_AUX)
    })
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
/// Every subdivision URN is a typed [`Identifier`] grounded in the
/// LRC USLM XML User Guide §V hierarchy.
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
    /// from this section's subdivision tree. Downstream consumers
    /// (e.g. `sox_1514a::statute`) call this directly via
    /// [`crate::social::software::markup::xml::uslm::corpus::loaded`].
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
    /// Delegates to the canonical [`loaded`] singleton; kept for
    /// existing test-suite call sites.
    ///
    /// Mirrors
    /// [`crate::social::judicial::statute_structure::english_adjunction::test_helpers::cached_english`]
    /// — same OnceLock pattern, build-time codegen instead of
    /// runtime XML parsing.
    #[cfg(test)]
    pub fn cached_full() -> &'static Self {
        loaded()
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
}
