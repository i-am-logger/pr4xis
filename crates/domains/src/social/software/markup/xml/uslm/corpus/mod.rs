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

/// The USC → generic [`Archive`](pr4xis_runtime::archive::Archive) projection —
/// statute provisions as content-addressed `Definition` nodes carrying typed
/// `Parthood` / grounding edges (the analog of the English bridge).
pub mod bridge;
pub mod identifiers;
pub mod kinds;
pub mod namespaces;
pub mod runtime_types;
pub mod section_aux;

// The self-describing, load-validated `.prx.gz` distribution envelope for
// the loaded U.S. Code corpus — the USC second consumer of the
// `OntologyArchiveStorage` ontology, the legislative twin of `owl::prx`.
// Gated on `feature = "prx"` (rkyv archival + RFC 1952 gzip), prx-gated and
// NOT codegen-gated: USC emit parses via `read_uslm_title` (quick-xml only,
// the path `loaded()` already uses), so it needs no `pr4xis/codegen`
// `xsd-parser` substrate, unlike `owl::prx`'s `owl_to_builder`.
#[cfg(feature = "prx")]
pub mod prx;

pub use identifiers::{UsCodeTitleId, UsCodeTitleIdError};
pub use kinds::{
    ContainerKind, InlineKind, SubdivisionKind, UsCodeAdditionalContainer, UsCodeAmendmentKind,
    UsCodeFormElement, UsCodeHeadingVariant, UsCodeLegislativeFormula, UsCodeNoteKind,
    UsCodeQuotedVariant, UsCodeTableCellKind,
};
pub use namespaces::{DUBLIN_CORE_NAMESPACE_URI, USLM_NAMESPACE_URI, XHTML_NAMESPACE_URI};
pub use runtime_types::{
    HierarchyNode, UsCodeAmendmentMarkup, UsCodeContainer, UsCodeContentAttr, UsCodeContentNode,
    UsCodeContinuation, UsCodeDate, UsCodeDefBlock, UsCodeHeader, UsCodeInlineRun, UsCodeMarker,
    UsCodeMeta, UsCodeMetaProperty, UsCodeMixed, UsCodeName, UsCodeNote, UsCodeNotesBlock,
    UsCodeProviso, UsCodeQuotedContent, UsCodeRef, UsCodeSection, UsCodeSectionRef,
    UsCodeSignature, UsCodeSourceCredit, UsCodeSubdivision, UsCodeTable, UsCodeTableCell,
    UsCodeTableRow, UsCodeTerm, UsCodeTitle, UsCodeToc, UsCodeTocItem, UslmReadError,
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
        // Parse-once corpus load. Each registered USC title is loaded
        // from its compiled `.prx` archive when one is present — the
        // fast rkyv path `pr4xis compile` produces, admitted through the
        // fail-closed `praxis.lock` gate — and parsed from USLM XML
        // otherwise. Per-title corpora compose via `UsCode::concat`.
        //
        // On a fresh checkout no archive exists, so the XML path runs
        // and pays the ~85 MB parse — the cost the build-time aggregate
        // static once paid before it hit rustc's compile-time memory
        // ceiling (M4.δ.7.a). `pr4xis compile` turns that into an
        // O(rkyv) load: CI compiles once up front so each nextest
        // process loads in ~ms instead of re-parsing the XML per
        // process (nextest is process-per-test, so the OnceLock only
        // amortizes within a single test).
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
        use crate::social::software::markup::xml::uslm::lens::read_uslm_title;
        let workspace_root: std::path::PathBuf =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(|p| p.parent())
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("."));
        let mut parts: alloc::vec::Vec<UsCode> = alloc::vec::Vec::new();
        for entry in data_sources() {
            if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
                continue;
            }
            // Fastest path: a content-addressed COMPACT archive (portable
            // dependency-free succinct bytes), admitted through the fail-closed
            // `[compact_archive_signatures]` gate — gunzip + hash-check + decode,
            // with NO source reconstruction (the cost the rkyv-envelope gate pays
            // on giant titles). Tried before the envelope; an absent or unpinned
            // compact archive falls through to it.
            #[cfg(feature = "prx")]
            {
                use crate::applied::data_provisioning::registry::lock_compact_archive_signature;
                let cprx_path = prx::usc_compact_prx_cache_dir(&workspace_root)
                    .join(alloc::format!("{}-{}.cprx.gz", entry.name, entry.version));
                if let Ok(cprx_gz) = std::fs::read(&cprx_path)
                    && let Some(pin) = lock_compact_archive_signature(&entry.name, &entry.version)
                {
                    let key = alloc::format!("{}@{}", entry.name, entry.version);
                    match prx::load_compact_usc_prx_gz_gated(&cprx_gz, pin, &key) {
                        Ok(corpus) => {
                            parts.push(corpus);
                            continue;
                        }
                        // Pinned but the compact archive failed the content gate —
                        // the committed pin and emitted bytes disagree (a stale
                        // source or tampering). Fail LOUD; silently falling
                        // through would mask the contract violation.
                        Err(e) => panic!(
                            "loaded(): compact archive {} is pinned but failed the \
                             content gate: {e}",
                            cprx_path.display()
                        ),
                    }
                }
            }
            // Fast path: a compiled `.prx` archive carries the
            // already-parsed corpus (rkyv), admitted through the
            // fail-closed lock gate. A present-but-bad archive is a hard
            // error — never a silent fall back to XML, which would mask
            // tampering. Only an ABSENT archive falls through to XML.
            #[cfg(feature = "prx")]
            {
                let prx_path = prx::usc_prx_cache_dir(&workspace_root).join(alloc::format!(
                    "{}-{}.prx.gz",
                    entry.name,
                    entry.version
                ));
                if let Ok(prx_gz) = std::fs::read(&prx_path) {
                    use crate::social::software::markup::xml::owl::prx::PrxError;
                    match prx::load_usc_prx_gz_from_lock(&prx_gz) {
                        Ok(corpus) => {
                            parts.push(corpus);
                            continue;
                        }
                        // Archive present but NOT pinned in `praxis.lock` — it
                        // is not a trust anchor yet, so fall through to the
                        // authoritative (`[hashes]`-verified) XML source. Run
                        // `pr4xis compile --lock` to pin it and take the fast
                        // path. (Native only: the source XML is the truth and
                        // the archive is an optimization. The browser, with no
                        // source to fall back to, has no such graceful path —
                        // its gate is the only anchor.)
                        Err(PrxError::NoArchivePin { .. } | PrxError::NoLockPin { .. }) => {}
                        // Pinned, but the gate rejected it: the committed pin
                        // and the emitted archive disagree (toolchain drift, a
                        // stale source, or tampering). Fail LOUD — silently
                        // parsing XML would mask a real contract violation and
                        // re-create the per-process parse cost the cache exists
                        // to remove. Re-run `pr4xis compile`.
                        Err(e) => panic!(
                            "loaded(): compiled archive {} is pinned but failed \
                             the load gate: {e}",
                            prx_path.display()
                        ),
                    }
                }
            }
            let path = workspace_root.join(entry.local_path());
            let Ok(xml) = std::fs::read_to_string(&path) else {
                // Title XML not on disk — skip gracefully (the same
                // behavior the prior codegen path had: build script
                // emitted a stub when no XML was present).
                continue;
            };
            match read_uslm_title(&xml) {
                Ok(title) => parts.push(UsCode::from_uslm_titles_owned(alloc::vec![title])),
                Err(e) => panic!(
                    "loaded() failed parsing registered title {}: {e}",
                    entry.name
                ),
            }
        }
        UsCode::concat(parts)
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

    /// Runtime constructor: assemble a [`UsCode`] from parsed
    /// [`UsCodeTitle`] instances obtained via [`read_uslm_title`].
    /// Per M4.δ.7.a (`docs/m4-delta-7-a-design.md`), this is the
    /// canonical path [`loaded()`] uses to materialize the corpus
    /// from on-disk USLM XML — the WordNet pattern, mirroring
    /// [`crate::cognitive::linguistics::english::English::from_wordnet`].
    ///
    /// Replaces the build-time codegen aggregate static that hit
    /// rustc's compile-time memory ceiling at ~85 MB of input XML.
    ///
    /// [`Box::leak`] converts owned strings + slices into the
    /// `&'static` lifetimes the existing [`UscSection`] /
    /// [`UscSubdivision`] API requires. The leaks persist for
    /// process lifetime, same as the [`OnceLock`]-cached singleton
    /// — equivalent to build-time-emitted statics.
    ///
    /// Section text accumulates chapeau + content at every depth
    /// (matching `pr4xis::codegen::usc_corpus`'s
    /// `section_bodies_concatenate_chapeau_and_content_text` test
    /// invariant) so that downstream Layer-3 lemma resolution sees
    /// the same body text it saw under the codegen path.
    ///
    /// [`OnceLock`]: std::sync::OnceLock
    /// [`read_uslm_title`]: super::lens::leaf_readers::read_uslm_title
    pub fn from_uslm_titles_owned(titles: alloc::vec::Vec<UsCodeTitle>) -> Self {
        let mut sections: Vec<UscSection> = Vec::new();
        let mut by_urn: HashMap<String, usize> = HashMap::new();
        for title in titles {
            for section in title.sections {
                let urn_str: &'static str =
                    alloc::boxed::Box::leak(section.identifier.clone().into_boxed_str());
                let urn =
                    Identifier::from_codegen_static(IdentifierFormatConcept::UslmUrn, urn_str);
                // The section's PROSE heading — the title text minus the
                // editorial footnote annotation the LRC nests inside the
                // `<heading>` (a typed `<note type="footnote">` plus its
                // `<ref class="footnoteRef">` marker). The flat
                // `section.heading` (`heading_mixed.plain_text()`) flattens
                // that footnote's own sentence INTO the title; the runtime
                // vocabulary entry — and the lexical-understanding pipeline
                // that reads it — wants the title, not the editor's note, so
                // it projects from the typed tree via `prose_text()`. The
                // corpus `section.heading` field itself stays untouched (the
                // byte-exact writer path depends on it).
                let heading = section.heading_mixed.prose_text();
                let text = section_body_text(&section);
                let (sub_vec, rel_vec) = subdivisions_to_static(&section.children, urn_str);
                let subdivisions: &'static [UscSubdivision] =
                    alloc::boxed::Box::leak(sub_vec.into_boxed_slice());
                let relations: &'static [UscComposesEdge] =
                    alloc::boxed::Box::leak(rel_vec.into_boxed_slice());
                by_urn.insert(section.identifier.clone(), sections.len());
                sections.push(UscSection {
                    urn,
                    heading,
                    text,
                    subdivisions,
                    relations,
                });
            }
        }
        Self { sections, by_urn }
    }

    /// Compose per-title corpora into the unified corpus. Each part is a
    /// single title materialised either from its compiled `.prx` archive
    /// (rkyv, via the fail-closed lock gate) or from parsed USLM XML;
    /// [`loaded`] picks the source per title and feeds the parts here.
    ///
    /// Section order is preserved (title-then-section, the order
    /// [`data_sources`][crate::applied::data_provisioning::registry::data_sources]
    /// yields registered titles); `by_urn` is re-indexed into the
    /// concatenated `sections`. Composition — not a third corpus walker —
    /// so an archive-loaded title and an XML-parsed title land in exactly
    /// the same shape.
    fn concat(parts: alloc::vec::Vec<UsCode>) -> Self {
        let mut sections: Vec<UscSection> = Vec::new();
        let mut by_urn: HashMap<String, usize> = HashMap::new();
        for part in parts {
            let base = sections.len();
            for (urn, idx) in part.by_urn {
                by_urn.insert(urn, base + idx);
            }
            sections.extend(part.sections);
        }
        Self { sections, by_urn }
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

/// Flatten a section's body text the way `pr4xis::codegen::usc_corpus`
/// does at build time: every chapeau + content node at any depth,
/// space-joined. The `section_bodies_concatenate_chapeau_and_content_text`
/// test in the codegen pins this behavior; the runtime path must
/// produce the same body so downstream Layer-3 lemma resolution
/// sees identical input.
fn section_body_text(s: &UsCodeSection) -> String {
    let mut out = String::new();
    if let Some(c) = &s.chapeau {
        push_with_space(&mut out, c);
    }
    if let Some(c) = &s.content {
        push_with_space(&mut out, c);
    }
    for child in &s.children {
        push_subdivision_text(&mut out, child);
    }
    out
}

/// Recursively append a subdivision's body text to `out`.
fn push_subdivision_text(out: &mut String, s: &UsCodeSubdivision) {
    if let Some(c) = &s.chapeau {
        push_with_space(out, c);
    }
    if let Some(c) = &s.content {
        push_with_space(out, c);
    }
    for child in &s.children {
        push_subdivision_text(out, child);
    }
}

/// Append `s` to `out`, separating with a single space if `out` is
/// non-empty. Matches the codegen's space-join convention.
fn push_with_space(out: &mut String, s: &str) {
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(s);
}

/// Convert a `Vec<UsCodeSubdivision>` tree (owned, runtime) into
/// the static-lifetime [`UscSubdivision`] tree the corpus API
/// requires, plus the parallel [`UscComposesEdge`] list. Uses
/// [`Box::leak`] to convert owned strings + slices to `&'static`;
/// the leaks live for process lifetime (same as the OnceLock-cached
/// singleton).
///
/// `parent_urn` is the URN of the immediate parent (the section or
/// containing subdivision); each child emits one
/// `UscComposesEdge { from_urn: child, to_urn: parent }` edge.
fn subdivisions_to_static(
    subs: &[UsCodeSubdivision],
    parent_urn: &'static str,
) -> (Vec<UscSubdivision>, Vec<UscComposesEdge>) {
    let mut result_subs = Vec::with_capacity(subs.len());
    let mut all_edges = Vec::new();
    for sub in subs {
        let sub_urn: &'static str =
            alloc::boxed::Box::leak(sub.identifier.clone().into_boxed_str());
        all_edges.push(UscComposesEdge {
            from_urn: sub_urn,
            to_urn: parent_urn,
        });
        let (child_subs, child_edges) = subdivisions_to_static(&sub.children, sub_urn);
        all_edges.extend(child_edges);
        let num_leaked: &'static str = alloc::boxed::Box::leak(sub.num.clone().into_boxed_str());
        let heading_leaked: Option<&'static str> = sub
            .heading
            .as_ref()
            .map(|h| -> &'static str { alloc::boxed::Box::leak(h.clone().into_boxed_str()) });
        let chapeau_leaked: Option<&'static str> = sub
            .chapeau
            .as_ref()
            .map(|c| -> &'static str { alloc::boxed::Box::leak(c.clone().into_boxed_str()) });
        let content_leaked: Option<&'static str> = sub
            .content
            .as_ref()
            .map(|c| -> &'static str { alloc::boxed::Box::leak(c.clone().into_boxed_str()) });
        let children_leaked: &'static [UscSubdivision] =
            alloc::boxed::Box::leak(child_subs.into_boxed_slice());
        let urn = Identifier::from_codegen_static(IdentifierFormatConcept::UslmUrn, sub_urn);
        result_subs.push(UscSubdivision {
            urn,
            kind: sub.kind,
            num: num_leaked,
            heading: heading_leaked,
            chapeau: chapeau_leaked,
            content: content_leaked,
            children: children_leaked,
        });
    }
    (result_subs, all_edges)
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
