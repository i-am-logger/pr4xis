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

/// One statute embedded in a title — `&'static`-only fields so the
/// whole table lives in `.rodata`.
#[derive(Debug, Clone, Copy)]
pub struct StaticStatute {
    /// USLM URN, e.g. `/us/usc/t18/s1514A`.
    pub identifier: &'static str,
    /// `<num>` text, e.g. `"1514A"`.
    pub num: &'static str,
    /// `<heading>` plain text.
    pub heading: &'static str,
    /// Term table (subsection-level + below).
    pub terms: &'static [StaticTerm],
    /// Composes-mereology edges between terms.
    pub relations: &'static [StaticRelation],
}

/// One subdivision-term inside a statute. `id` is a CURIE
/// (`"sox_1514a:a_1_A"`); `name` is the heading or a derived
/// subdivision marker; `definition` is the chapeau or content text.
#[derive(Debug, Clone, Copy)]
pub struct StaticTerm {
    pub id: &'static str,
    pub name: &'static str,
    pub definition: &'static str,
}

/// One Composes edge between terms. Both endpoints are CURIEs into
/// the same statute's term table.
#[derive(Debug, Clone, Copy)]
pub struct StaticRelation {
    pub from: &'static str,
    pub to: &'static str,
    pub kind: StaticRelationKind,
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

        // Find the static-side prefix — the longest term-id common
        // prefix up to (but not including) the colon. This is the
        // URN-derived name (e.g. `us_usc_t18_s1514a`) that the
        // codegen emitter wrote.
        let static_prefix = self
            .terms
            .iter()
            .find(|t| t.id.contains(':'))
            .and_then(|t| t.id.split(':').next())
            .unwrap_or("");

        let rename = |curie: &str| -> String {
            if let Some(local) = curie.strip_prefix(static_prefix)
                && let Some(rest) = local.strip_prefix(':')
            {
                return format!("{statute_name}:{rest}");
            }
            curie.to_string()
        };

        let data = StructuralData {
            description: format!("USLM source: {}", self.identifier),
            terms: self
                .terms
                .iter()
                .filter(|t| t.id.contains(':'))
                .map(|t| StructuralTerm {
                    id: rename(t.id),
                    name: t.name.to_string(),
                    definition: t.definition.to_string(),
                    lemmas: Vec::new(),
                })
                .collect(),
            relations: self
                .relations
                .iter()
                .filter(|r| r.from.contains(':') && r.to.contains(':'))
                .map(|r| StructuralRelation {
                    from: rename(r.from),
                    to: rename(r.to),
                    relation: match r.kind {
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
        super::Statute::from_structural_with_context(statute_name, version, &data, self.identifier)
            .expect("StaticStatute data must be valid (codegen-checked)")
    }
}

/// O(N) linear search for a section by USLM identifier. For ~1,400
/// sections this is comfortably under microseconds; if profiling
/// shows hotspot, swap for a phf::Map at codegen time.
pub fn find_section<'a>(
    sections: &'a [StaticStatute],
    identifier: &str,
) -> Option<&'a StaticStatute> {
    sections.iter().find(|s| s.identifier == identifier)
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
