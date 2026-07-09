//! Domain lenses for the statute layer — the final hops of the
//! `file ⇄ xml ⇄ xsd ⇄ statute` pipeline, expressed as composable
//! [`Lens`]es (Foster et al. 2007).
//!
//! The byte hop `file ⇄ UsCodeTitle` is the XSD-grounded
//! [`UslmXmlLens`]
//! (a [`WellBehavedLens`](crate::formal::meta::well_behaved_lens)).
//! This module adds the two *typed* hops above the byte boundary:
//!
//! - [`SectionByIndexLens`] : `UsCodeTitle ⇄ UsCodeSection` — focuses
//!   one section of a title (the list-element lens, Foster et al. 2007
//!   §3, Definition 3.2).
//! - [`UslmStatuteLens`] : `UsCodeSection ⇄ Statute` — the **domain
//!   lens** projecting a USLM section to its typed [`Statute`] via
//!   [`from_uslm_section`], with a *constant-complement* put-back
//!   (Bancilhon & Spyratos 1981): the `Statute` is a derived view of
//!   the authoritative USLM source, so a put of the get-image restores
//!   the source unchanged; updates outside the image are not propagated
//!   (USLM is the source of truth).
//!
//! Composed (`SectionByIndexLens ; UslmStatuteLens`) they give a
//! `UsCodeTitle ⇄ Statute` lens, which composes after the byte lens to
//! complete `file → statute`.
//!
//! ## Citation
//!
//! - **Foster, J. N., Greenwald, M. B., Moore, J. T., Pierce, B. C. &
//!   Schmitt, A.** "Combinators for Bidirectional Tree
//!   Transformations", *ACM TOPLAS* 29(3), 2007. §3 (Definition 3.2).
//! - **Bancilhon, F. & Spyratos, N.** "Update Semantics of Relational
//!   Views", *ACM TODS* 6(4), 1981 (constant complement).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::fmt;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::from_uslm::from_uslm_section;
use super::statute::{Statute, StatuteConstructError};
use crate::formal::meta::lens_composition::{Compose, Lens, WellBehavedLensAdapter, get_put_holds};
use crate::social::software::markup::xml::uslm::{
    UsCodeMixed, UsCodeSection, UsCodeTitle, UslmTreeViewLens, UslmXmlLens,
};

// =============================================================================
// SectionByIndexLens : UsCodeTitle ⇄ UsCodeSection (list-element focus).
// =============================================================================

/// Focuses the `index`-th section of a [`UsCodeTitle`] — the
/// list-element lens (Foster et al. 2007 §3, Definition 3.2). `get` reads the
/// section; `put` writes an updated section back at the same index.
#[derive(Debug, Clone, Copy)]
pub struct SectionByIndexLens {
    /// Zero-based position in [`UsCodeTitle::sections`].
    pub index: usize,
}

/// Error of [`SectionByIndexLens`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionLensError {
    /// The index is outside the title's section list.
    IndexOutOfRange { index: usize, len: usize },
}

impl fmt::Display for SectionLensError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SectionLensError::IndexOutOfRange { index, len } => {
                write!(
                    f,
                    "section index {index} out of range (title has {len} sections)"
                )
            }
        }
    }
}

impl Lens for SectionByIndexLens {
    type Source = UsCodeTitle;
    type View = UsCodeSection;
    type Error = SectionLensError;

    fn get(&self, title: &UsCodeTitle) -> Result<UsCodeSection, Self::Error> {
        title
            .sections
            .get(self.index)
            .cloned()
            .ok_or(SectionLensError::IndexOutOfRange {
                index: self.index,
                len: title.sections.len(),
            })
    }

    fn put(
        &self,
        section: &UsCodeSection,
        title: &UsCodeTitle,
    ) -> Result<UsCodeTitle, Self::Error> {
        if self.index >= title.sections.len() {
            return Err(SectionLensError::IndexOutOfRange {
                index: self.index,
                len: title.sections.len(),
            });
        }
        let mut updated = title.clone();
        updated.sections[self.index] = section.clone();
        Ok(updated)
    }
}

// =============================================================================
// UslmStatuteLens : UsCodeSection ⇄ Statute (the domain lens).
// =============================================================================

/// Projects a USLM [`UsCodeSection`] to its typed [`Statute`]. `get`
/// runs [`from_uslm_section`] under the lens's registry `name` /
/// `version`. `put` is the *constant-complement* view-update
/// (Bancilhon & Spyratos 1981): the `Statute` is a derived read-only
/// view of the authoritative USLM source, so putting back the get-image
/// restores the source unchanged; a view outside the image is rejected
/// (USLM is the source of truth, not the projection).
#[derive(Debug, Clone)]
pub struct UslmStatuteLens {
    /// The praxis-registry statute name used as the CURIE prefix
    /// (e.g. `"sox_1514a"`).
    pub name: String,
    /// The statute version (mirrors `praxis.toml`).
    pub version: String,
}

/// Error of [`UslmStatuteLens`].
#[derive(Debug, Clone)]
pub enum UslmStatuteLensError {
    /// `get` failed to construct the `Statute` from the USLM section.
    Construct(StatuteConstructError),
    /// `put` was asked to write back a `Statute` not equal to
    /// `get(source)` — outside the constant-complement image; USLM is
    /// the source of truth and such an update is not propagated.
    ViewOutsideImage,
}

impl fmt::Display for UslmStatuteLensError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UslmStatuteLensError::Construct(e) => write!(f, "statute construction: {e}"),
            UslmStatuteLensError::ViewOutsideImage => write!(
                f,
                "statute view is outside the lens's get-image; the constant-complement \
                 discipline (Bancilhon & Spyratos 1981) does not propagate it back to USLM"
            ),
        }
    }
}

/// A stable identity for a [`Statute`] view — name, version, and the
/// term / relation counts. `Statute` is not `PartialEq`; two statutes
/// agreeing on this signature are the same view for the lens's
/// constant-complement check.
fn statute_signature(s: &Statute) -> (String, String, usize, usize) {
    (
        s.name().to_string(),
        s.version().to_string(),
        s.terms().len(),
        s.relations().len(),
    )
}

impl Lens for UslmStatuteLens {
    type Source = UsCodeSection;
    type View = Statute;
    type Error = UslmStatuteLensError;

    fn get(&self, section: &UsCodeSection) -> Result<Statute, Self::Error> {
        from_uslm_section(&self.name, &self.version, section)
            .map_err(UslmStatuteLensError::Construct)
    }

    fn put(&self, view: &Statute, section: &UsCodeSection) -> Result<UsCodeSection, Self::Error> {
        let current = self.get(section)?;
        if statute_signature(view) == statute_signature(&current) {
            // Constant complement: the view is unchanged, so the source
            // (the whole USLM section) round-trips verbatim.
            Ok(section.clone())
        } else {
            Err(UslmStatuteLensError::ViewOutsideImage)
        }
    }
}

/// The composed typed chain `UsCodeTitle ⇄ Statute`: focus the
/// `index`-th section, then view it as a [`Statute`]. Composes after
/// the byte lens `file ⇄ UsCodeTitle` to complete `file → statute`.
pub fn title_to_statute_lens(
    index: usize,
    name: &str,
    version: &str,
) -> Compose<SectionByIndexLens, UslmStatuteLens> {
    Compose::new(
        SectionByIndexLens { index },
        UslmStatuteLens {
            name: name.to_string(),
            version: version.to_string(),
        },
    )
}

// =============================================================================
// SectionByNumLens : UsCodeTitle ⇄ UsCodeSection (lookup by num).
// =============================================================================

/// Focuses the first section of a [`UsCodeTitle`] whose `num` matches
/// `self.num` — the *find-by-key* lens (Foster et al. 2007 §3, Definition 3.2,
/// total-on-the-domain-of-definition).
///
/// `get` returns the matching section; `put` writes an updated section
/// back at the same position. Distinct from [`SectionByIndexLens`]: the
/// index-based lens is total on the index but ignores the section's
/// identifier, while this one is total on the *named* section regardless
/// of its position.
#[derive(Debug, Clone)]
pub struct SectionByNumLens {
    /// The `num` value of the section to focus (e.g. `"1514A"`).
    pub num: String,
}

/// Error of [`SectionByNumLens`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionNumLensError {
    /// No section in the title carries this `num`.
    NotFound { num: String },
}

impl fmt::Display for SectionNumLensError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SectionNumLensError::NotFound { num } => {
                write!(f, "no section with num = `{num}` in the title")
            }
        }
    }
}

impl Lens for SectionByNumLens {
    type Source = UsCodeTitle;
    type View = UsCodeSection;
    type Error = SectionNumLensError;

    fn get(&self, title: &UsCodeTitle) -> Result<UsCodeSection, Self::Error> {
        title
            .sections
            .iter()
            .find(|s| s.num == self.num)
            .cloned()
            .ok_or(SectionNumLensError::NotFound {
                num: self.num.clone(),
            })
    }

    fn put(
        &self,
        section: &UsCodeSection,
        title: &UsCodeTitle,
    ) -> Result<UsCodeTitle, Self::Error> {
        let idx = title
            .sections
            .iter()
            .position(|s| s.num == self.num)
            .ok_or_else(|| SectionNumLensError::NotFound {
                num: self.num.clone(),
            })?;
        let mut updated = title.clone();
        updated.sections[idx] = section.clone();
        Ok(updated)
    }
}

// =============================================================================
// The full composed chain Vec<u8> ⇄ Statute — the goal of the lens
// architecture: read a USLM-XML byte stream, focus a named section,
// project to its typed `Statute`. Every hop is a verified lens; the
// whole composite is verified by composition (Foster et al. 2007 §3).
// =============================================================================

/// Type alias for the byte hop adapted to the general [`Lens`] —
/// `Vec<u8> ⇄ UslmTypedTree`.
pub type ByteHop = WellBehavedLensAdapter<UslmXmlLens>;

/// The full bytes-to-statute composed lens type:
///
/// ```text
/// Vec<u8>
///   ─WellBehavedLensAdapter<UslmXmlLens>─►  UslmTypedTree
///   ─UslmTreeViewLens─►                     UsCodeTitle
///   ─SectionByNumLens─►                     UsCodeSection
///   ─UslmStatuteLens─►                      Statute
/// ```
pub type BytesToStatuteLens =
    Compose<Compose<Compose<ByteHop, UslmTreeViewLens>, SectionByNumLens>, UslmStatuteLens>;

/// Build the full `Vec<u8> ⇄ Statute` lens chain over a USLM source.
///
/// - `section_num` selects which `<section>` of the title to focus
///   (e.g. `"1514A"`).
/// - `name` / `version` parameterise the [`UslmStatuteLens`] (matched
///   against the praxis statute-registry CURIE).
///
/// The composite is a well-behaved lens (Foster et al. 2007 §3:
/// composition of well-behaved lenses is well-behaved) anchored at the
/// bytes — `put(get(bytes), bytes) == bytes` follows from the byte
/// hop's constant-complement discipline (Bancilhon & Spyratos 1981).
pub fn bytes_to_statute_lens(section_num: &str, name: &str, version: &str) -> BytesToStatuteLens {
    Compose::new(
        Compose::new(
            Compose::new(ByteHop::new(), UslmTreeViewLens),
            SectionByNumLens {
                num: section_num.to_string(),
            },
        ),
        UslmStatuteLens {
            name: name.to_string(),
            version: version.to_string(),
        },
    )
}

// =============================================================================
// Sample fixtures — a minimal USLM section/title for the lens axioms.
// =============================================================================

/// A minimal, childless USLM section — enough to exercise the domain
/// lens laws (it projects to a `Statute` with the lens's name/version
/// and no subdivision terms).
pub fn sample_uslm_section() -> UsCodeSection {
    UsCodeSection {
        identifier: "/us/usc/t18/s1514A".to_string(),
        num: "1514A".to_string(),
        num_text: String::new(),
        num_footnote: None,
        heading: "Civil action to protect against retaliation".to_string(),
        heading_runs: Vec::new(),
        heading_mixed: UsCodeMixed::new(),
        chapeau: None,
        chapeau_runs: Vec::new(),
        chapeau_mixed: None,
        content: None,
        content_runs: Vec::new(),
        content_mixed: None,
        children: Vec::new(),
        refs: Vec::new(),
        notes_blocks: Vec::new(),
        bare_notes: Vec::new(),
        source_credits: Vec::new(),
        continuations: Vec::new(),
        def_blocks: Vec::new(),
        markers: Vec::new(),
        amendments: Vec::new(),
    }
}

/// A minimal USLM title wrapping [`sample_uslm_section`].
pub fn sample_uslm_title() -> UsCodeTitle {
    UsCodeTitle {
        identifier: "/us/usc/t18".to_string(),
        number: 18,
        heading: "CRIMES AND CRIMINAL PROCEDURE".to_string(),
        sections: vec![sample_uslm_section()],
        hierarchy: Vec::new(),
        notes_blocks: Vec::new(),
        bare_notes: Vec::new(),
        headers: Vec::new(),
        signatures: Vec::new(),
        meta: None,
        tocs: Vec::new(),
        tables: Vec::new(),
        uscdoc_mixed: None,
    }
}

// =============================================================================
// Axioms.
// =============================================================================

/// Verify PutGet on the lens's get-image by signature equality
/// (`Statute` is not `PartialEq`): `get(put(get(s), s))` has the same
/// signature as `get(s)`.
fn put_get_on_image_holds(lens: &UslmStatuteLens, section: &UsCodeSection) -> bool {
    let Ok(v) = lens.get(section) else {
        return false;
    };
    let Ok(s2) = lens.put(&v, section) else {
        return false;
    };
    matches!(lens.get(&s2), Ok(v2) if statute_signature(&v2) == statute_signature(&v))
}

/// Axiom: `UslmStatuteLens` is a well-behaved domain lens on the sample
/// USLM section — GetPut holds (constant-complement put-back restores
/// the source) and PutGet holds on the get-image (Foster et al. 2007
/// §3, Definition 3.2; Bancilhon & Spyratos 1981).
pub struct UslmStatuteLensWellBehaved;

impl Axiom for UslmStatuteLensWellBehaved {
    fn verify(&self) -> Verdict {
        let lens = UslmStatuteLens {
            name: "sox_1514a".to_string(),
            version: "2002".to_string(),
        };
        let section = sample_uslm_section();
        let get_put = get_put_holds(&lens, &section);
        let put_get = put_get_on_image_holds(&lens, &section);
        // The projected statute carries the lens's identity.
        let projects =
            matches!(lens.get(&section), Ok(s) if s.name() == "sox_1514a" && s.version() == "2002");
        if get_put && put_get && projects {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "UslmStatuteLensWellBehaved",
        "the USLM→Statute domain lens satisfies GetPut (constant-complement put-back restores the source) and PutGet on its get-image, and projects a section to a statute carrying the lens's name/version",
        "Foster et al. (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2; Bancilhon & Spyratos (1981) ACM TODS 6(4)"
    );
}

pr4xis::register_axiom!(
    UslmStatuteLensWellBehaved,
    "Foster et al. (2007) ACM TOPLAS 29(3) §3, Definition 3.2; Bancilhon & Spyratos (1981) ACM TODS 6(4)"
);

/// Axiom: the composed typed chain `UsCodeTitle ⇄ Statute`
/// (`SectionByIndexLens ; UslmStatuteLens`) is well-behaved — GetPut
/// holds on the sample title, and `get` yields the section's statute
/// (Foster et al. 2007 §3: composition preserves well-behavedness).
pub struct StatuteChainComposes;

impl Axiom for StatuteChainComposes {
    fn verify(&self) -> Verdict {
        let chain = title_to_statute_lens(0, "sox_1514a", "2002");
        let title = sample_uslm_title();
        let get_put = get_put_holds(&chain, &title);
        let projects = matches!(chain.get(&title), Ok(s) if s.name() == "sox_1514a");
        // An out-of-range section index fails closed.
        let oob = title_to_statute_lens(99, "sox_1514a", "2002")
            .get(&title)
            .is_err();
        if get_put && projects && oob {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StatuteChainComposes",
        "the composed UsCodeTitle⇄Statute lens (SectionByIndexLens ; UslmStatuteLens) is well-behaved (GetPut on the sample title) and get yields the focused section's statute; an out-of-range index fails closed",
        "Foster et al. (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3 (composition)"
    );
}

pr4xis::register_axiom!(
    StatuteChainComposes,
    "Foster et al. (2007) ACM TOPLAS 29(3) §3 (composition)"
);

// =============================================================================
// Real-data round-trip axiom — the full `bytes ⇄ Statute` chain run
// against the actual USC Title 18 (P.L. 119-90) USLM bytes, not a
// hand-built fixture. The "real round-trip" deliverable (M4.ε.5.a.6.3).
// =============================================================================

/// Locate the on-disk path for a praxis-registry source. Returns the
/// absolute path the file *would* live at, regardless of whether the
/// bytes are present yet — caller checks existence.
fn resolve_source_path(name: &str, version: &str) -> Option<std::path::PathBuf> {
    let entry = crate::applied::data_provisioning::registry::by_name_version(name, version)?;
    // Mirrors the resolution logic in
    // `crate::formal::meta::well_behaved_lens::harness::resolve_source_bytes`:
    // workspace-relative path → absolute via CARGO_MANIFEST_DIR + two
    // `parent()` calls. Keeping the logic local avoids exposing the
    // harness's private helper.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path_str = entry.local_path();
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .and_then(std::path::Path::parent);
    Some(
        workspace_root
            .map(|root| root.join(&path_str))
            .unwrap_or_else(|| std::path::PathBuf::from(&path_str)),
    )
}

/// Axiom: the full `bytes ⇄ Statute` lens chain is well-behaved on
/// the *real* USC Title 18 (P.L. 119-90) USLM bytes — not the
/// hand-built sample title. The chain focuses the actual §1514A
/// (SOX whistleblower civil action) section, projects it to a
/// [`Statute`] with the lens's name/version, and GetPut holds:
/// `put(get(bytes), bytes) == bytes` (Bancilhon & Spyratos 1981
/// constant complement; Foster et al. 2007 §3 composition preserves
/// well-behavedness).
///
/// `SourceNotOnDisk` is a soft pass — committers without
/// `pr4xis update`-ed corpora don't break the build (mirrors
/// `RoundTripHarnessAllVerified`). Any *real* lens-law violation or
/// projection failure on present bytes fails the axiom.
pub struct BytesToStatuteOnRealTitle18;

impl Axiom for BytesToStatuteOnRealTitle18 {
    fn verify(&self) -> Verdict {
        let Some(path) = resolve_source_path("usc_title_18", "pl-119-90") else {
            // Source not registered — the registry mistakenly drifted.
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Soft pass: no `pr4xis update` yet. The byte-anchored
                // harness's `RoundTripHarnessAllVerified` already
                // separately reports this; this axiom intentionally
                // does not duplicate the failure surface.
                return Ok(Box::new(SimpleProof::new(self.meta())));
            }
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };

        let chain = bytes_to_statute_lens("1514A", "sox_1514a", "2002");

        // get(bytes) yields a Statute carrying the lens's identity.
        let statute = match chain.get(&bytes) {
            Ok(s) => s,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let identity_ok = statute.name() == "sox_1514a" && statute.version() == "2002";

        // GetPut on the real bytes: the byte-hop's constant complement
        // propagates through composition; put(get(bytes), bytes) ==
        // bytes (Bancilhon & Spyratos 1981 Theorem 3, lifted through
        // Foster et al. 2007 §3).
        let get_put = get_put_holds(&chain, &bytes);

        if identity_ok && get_put {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BytesToStatuteOnRealTitle18",
        "the full bytes ⇄ Statute composed lens reads the actual USC Title 18 (P.L. 119-90) USLM bytes, focuses §1514A, projects to a Statute with the lens's identity, and GetPut holds at the byte boundary",
        "Foster et al. (2007) ACM TOPLAS 29(3) §3, Definition 3.2; Bancilhon & Spyratos (1981) ACM TODS 6(4) Theorem 3"
    );
}

pr4xis::register_axiom!(
    BytesToStatuteOnRealTitle18,
    "Foster et al. (2007) ACM TOPLAS 29(3) §3, Definition 3.2; Bancilhon & Spyratos (1981) ACM TODS 6(4) Theorem 3"
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal::meta::lens_composition::{put_get_holds, put_put_holds};

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn section_lens_is_well_behaved() {
        let title = sample_uslm_title();
        let lens = SectionByIndexLens { index: 0 };
        let other = {
            let mut s = sample_uslm_section();
            s.heading = "Amended heading".to_string();
            s
        };
        assert!(get_put_holds(&lens, &title));
        assert!(put_get_holds(&lens, &other, &title)); // View is UsCodeSection: PartialEq
        assert!(put_put_holds(&lens, &other, &sample_uslm_section(), &title));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn section_lens_out_of_range() {
        let title = sample_uslm_title();
        assert!(SectionByIndexLens { index: 5 }.get(&title).is_err());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn domain_lens_projects_statute() {
        let lens = UslmStatuteLens {
            name: "sox_1514a".to_string(),
            version: "2002".to_string(),
        };
        let st = lens.get(&sample_uslm_section()).unwrap();
        assert_eq!(st.name(), "sox_1514a");
        assert_eq!(st.version(), "2002");
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn domain_lens_get_put_and_put_get() {
        let lens = UslmStatuteLens {
            name: "sox_1514a".to_string(),
            version: "2002".to_string(),
        };
        let section = sample_uslm_section();
        assert!(get_put_holds(&lens, &section));
        assert!(put_get_on_image_holds(&lens, &section));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn composed_chain_title_to_statute() {
        let chain = title_to_statute_lens(0, "sox_1514a", "2002");
        let title = sample_uslm_title();
        let st = chain.get(&title).unwrap();
        assert_eq!(st.name(), "sox_1514a");
        assert!(get_put_holds(&chain, &title));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn axiom_domain_lens_well_behaved() {
        assert!(UslmStatuteLensWellBehaved.verify().is_ok());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn axiom_chain_composes() {
        assert!(StatuteChainComposes.verify().is_ok());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn section_by_num_lens_is_well_behaved() {
        let title = sample_uslm_title();
        let lens = SectionByNumLens {
            num: "1514A".to_string(),
        };
        let other = {
            let mut s = sample_uslm_section();
            s.heading = "Amended heading".to_string();
            s
        };
        assert_eq!(lens.get(&title).unwrap().num, "1514A");
        assert!(get_put_holds(&lens, &title));
        assert!(put_get_holds(&lens, &other, &title));
        assert!(put_put_holds(&lens, &other, &sample_uslm_section(), &title));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn section_by_num_lens_not_found() {
        let title = sample_uslm_title();
        let lens = SectionByNumLens {
            num: "does-not-exist".to_string(),
        };
        match lens.get(&title) {
            Err(SectionNumLensError::NotFound { num }) => assert_eq!(num, "does-not-exist"),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    // `BytesToStatuteOnRealTitle18` parses the real ~12 MB USC Title 18 USLM
    // bytes, so its `#[test]` driver is a heavy-corpus producer: it lives in the
    // heavy-corpus lane — see
    // `crates/praxis-corpus-tests/tests/statute_axioms.rs::
    // axiom_bytes_to_statute_on_real_title_18`. One process there parses the
    // corpus once; the fast nextest lane no longer pays the parse per
    // process-isolated test. The axiom itself (`pub` above) is unchanged.
}
