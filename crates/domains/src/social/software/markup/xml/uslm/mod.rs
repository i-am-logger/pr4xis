//! USLM — United States Legislative Markup.
//!
//! USLM is an XML schema published by the U.S. House Office of
//! the Law Revision Counsel (LRC) for the United States Code. It
//! extends the [generic XML ontology](super) with **legislative**
//! meaning — `<title>`, `<section>`, `<subsection>`,
//! `<paragraph>`, `<subparagraph>`, `<clause>`, `<ref>`, etc.
//! denote the structural units of a legal text per Bluebook §3.3
//! statutory subdivision conventions.
//!
//! ## Authoritative source
//!
//! - U.S. House Office of the Law Revision Counsel, *USLM XML
//!   User Guide and Schema (USLM-1.0.15.xsd)*. Available at
//!   <https://uscode.house.gov/uslm/>.
//! - 1 U.S.C. § 204 — *Codes and Supplements; positive law
//!   titles*, the statute authorizing the U.S. Code itself.
//!
//! ## Layer position
//!
//! USLM sits between the generic XML ontology and the legal
//! Statute ontology, exactly as LMF sits between XML and the
//! lexical English ontology:
//!
//! ```text
//! generic XML  ─►  USLM (this module)  ─►  Statute  ─►  SOX 1514A instance
//! ```
//!
//! The build-time codegen path
//! ([`pr4xis::codegen::uslm`][crate-codegen]) consumes the same
//! data shape, slicing a section out and producing a `RawStatuteDoc`
//! that flows through the existing
//! [`pr4xis::codegen::statute`][crate-codegen-statute] pipeline
//! to emit static Rust modules at build time.
//!
//! [crate-codegen]: ../../../../../../../../pr4xis/codegen/uslm/index.html
//! [crate-codegen-statute]: ../../../../../../../../pr4xis/codegen/statute/index.html

pub mod corpus;
pub mod lens;

pub use corpus::*;
pub use lens::{
    UslmLensError, UslmTreeViewLens, UslmTypedTree, UslmXmlLens, read_section, read_uslm_title,
};

// USLM structural axiom validators — used by this crate's `#[cfg(test)]` modules
// AND the workspace heavy-corpus test crate (via `test-internals`), so a full
// title parsed once can be validated under `cargo test` instead of re-parsed per
// process-isolated nextest test.
#[cfg(any(test, feature = "test-internals"))]
pub mod axioms;

/// Test-only sourcing of the real 18 U.S.C. § 1514A USLM slice from the
/// fetched `usc_title_18` corpus — NOT from a committed standalone fixture
/// (the `sox_1514a-2002.xml` granule was removed when `pr4xis-domains` went
/// crates.io-publishable). § 1514A is codified at 18 U.S.C. § 1514A, and
/// `usc_title_18` is the registered authoritative source CI fetches via
/// `pr4xis update` and compiles to `.prx`; this slices § 1514A out of it.
///
/// Shared by `uslm::tests` and `uslm::lens::tests` (both descendants of
/// `uslm`). Loading FAILS LOUD — never skips — when the corpus is absent:
/// the data is fetched in CI, so an absent corpus is a real failure, not a
/// reason to false-green.
#[cfg(test)]
pub(crate) mod real_sox_1514a {
    use super::lens::read_uslm_title;
    use super::{HierarchyNode, UsCodeSection, UsCodeTitle};

    /// USLM identifier for 18 U.S.C. § 1514A (Sarbanes–Oxley § 806).
    pub(crate) const SECTION_IDENTIFIER: &str = "/us/usc/t18/s1514A";

    /// Absolute path to the fetched `usc_title_18` USLM XML, resolved from
    /// the praxis registry (`by_name("usc_title_18").local_path()`) against
    /// the workspace root — the same resolution `corpus::loaded` uses.
    /// Panics if the registry entry drifted away (a build-time invariant —
    /// `[sources.usc_title_18]` is in the workspace `praxis.toml`).
    fn title_18_path() -> std::path::PathBuf {
        let entry = crate::applied::data_provisioning::registry::by_name("usc_title_18")
            .expect("usc_title_18 must be a registered praxis.toml source");
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        workspace_root.join(entry.local_path())
    }

    /// The raw fetched `usc_title_18` USLM XML bytes-as-string. FAILS LOUD
    /// when the corpus is not on disk — CI fetches it via
    /// `pr4xis update usc_title_18`; tests do not skip. Used by call sites
    /// that drive a section-slicing parser (e.g.
    /// `pr4xis::codegen::uslm::parse_uslm_str`, which selects § 1514A
    /// internally by its USLM identifier) so both the runtime and codegen
    /// paths read the SAME source bytes.
    pub(crate) fn xml() -> alloc::string::String {
        let path = title_18_path();
        std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "run `pr4xis update usc_title_18` to fetch the Title 18 USLM corpus \
                 at {} — tests do not skip ({e})",
                path.display()
            )
        })
    }

    /// The real § 1514A as a single-section [`UsCodeTitle`] sliced out of the
    /// fetched Title 18 corpus — the title-sourced replacement for the deleted
    /// standalone `sox_1514a-2002.xml` slice. The returned title carries the
    /// real `/us/usc/t18` identifier and exactly one section (§ 1514A), so
    /// downstream assertions on `title.sections[0]` and the `axiom_*(&title)`
    /// validators behave identically to the old standalone-slice shape.
    ///
    /// FAILS LOUD when the corpus is not on disk — CI fetches it via
    /// `pr4xis update usc_title_18`; tests do not skip.
    pub(crate) fn title() -> UsCodeTitle {
        let path = title_18_path();
        let xml = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "run `pr4xis update usc_title_18` to fetch the Title 18 USLM corpus \
                 at {} — tests do not skip ({e})",
                path.display()
            )
        });
        let full = read_uslm_title(&xml)
            .expect("the fetched usc_title_18 USLM XML must parse into a UsCodeTitle");
        let section = section_from(&full);
        UsCodeTitle {
            sections: alloc::vec![section.clone()],
            hierarchy: alloc::vec![HierarchyNode::Section(alloc::boxed::Box::new(section))],
            ..full
        }
    }

    /// The verbatim `<section …identifier="/us/usc/t18/s1514A">…</section>`
    /// byte span sliced out of the fetched Title 18 USLM source — genuine
    /// published bytes, no transcription — wrapped in the canonical XML
    /// declaration so it is a well-formed mini-document. The title-sourced
    /// replacement for the deleted standalone `sox_1514a-2002.xml` byte
    /// stream, for byte-level lens-law checks (e.g. PutGet). Mirrors the
    /// in-repo `lens::writer::tests::real_title1_section` substring-slice
    /// pattern, but FAILS LOUD (panics) when the corpus or section is absent
    /// — CI fetches the corpus; tests do not skip.
    pub(crate) fn section_bytes() -> alloc::vec::Vec<u8> {
        const XML_DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>";
        let xml = xml();
        let needle = alloc::format!("identifier=\"{SECTION_IDENTIFIER}\"");
        let id_pos = xml.find(&needle).unwrap_or_else(|| {
            panic!(
                "§ 1514A ({SECTION_IDENTIFIER}) not found in the fetched usc_title_18 \
                 corpus — a real corpus regression"
            )
        });
        let start = xml[..id_pos]
            .rfind("<section")
            .expect("§ 1514A identifier must sit inside a <section> element");
        let end_tag = "</section>";
        let end_rel = xml[start..]
            .find(end_tag)
            .expect("§ 1514A <section> must have a closing </section>")
            + end_tag.len();
        alloc::format!("{XML_DECL}{}", &xml[start..start + end_rel]).into_bytes()
    }

    /// The real § 1514A as a single [`UsCodeSection`] sliced out of the
    /// fetched Title 18 corpus. Same sourcing as [`title`]; for call sites
    /// that operate on the section directly.
    pub(crate) fn section() -> UsCodeSection {
        let path = title_18_path();
        let xml = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "run `pr4xis update usc_title_18` to fetch the Title 18 USLM corpus \
                 at {} — tests do not skip ({e})",
                path.display()
            )
        });
        let full = read_uslm_title(&xml)
            .expect("the fetched usc_title_18 USLM XML must parse into a UsCodeTitle");
        section_from(&full)
    }

    /// Find § 1514A within a parsed Title 18 — `read_uslm_title` flattens
    /// every `<section>` (regardless of nesting) into `title.sections`, so
    /// the section is located by its USLM identifier. Panics if absent: a
    /// present-but-§1514A-less Title 18 is a corpus regression, not a skip.
    fn section_from(full: &UsCodeTitle) -> UsCodeSection {
        full.sections
            .iter()
            .find(|s| s.identifier == SECTION_IDENTIFIER)
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "§ 1514A ({SECTION_IDENTIFIER}) not found in the fetched usc_title_18 \
                     corpus — a real corpus regression"
                )
            })
    }
}

#[cfg(test)]
mod tests;
