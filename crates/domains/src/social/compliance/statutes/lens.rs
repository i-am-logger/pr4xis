//! Domain lenses for the statute layer — the final hops of the
//! `file ⇄ xml ⇄ xsd ⇄ statute` pipeline, expressed as composable
//! [`Lens`]es (Foster et al. 2007).
//!
//! The byte hop `file ⇄ UsCodeTitle` is the XSD-grounded
//! [`UslmXmlLens`](crate::social::software::markup::xml::uslm::UslmXmlLens)
//! (a [`WellBehavedLens`](crate::formal::meta::well_behaved_lens)).
//! This module adds the two *typed* hops above the byte boundary:
//!
//! - [`SectionByIndexLens`] : `UsCodeTitle ⇄ UsCodeSection` — focuses
//!   one section of a title (the list-element lens, Foster et al. 2007
//!   §2.2).
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
//!   Transformations", *ACM TOPLAS* 29(3), 2007. §2.2, §3.
//! - **Bancilhon, F. & Spyratos, N.** "Update Semantics of Relational
//!   Views", *ACM TODS* 6(4), 1981 (constant complement).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::fmt;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::from_uslm::from_uslm_section;
use super::statute::{Statute, StatuteConstructError};
use crate::formal::meta::lens_composition::{Compose, Lens, get_put_holds};
use crate::social::software::markup::xml::uslm::{UsCodeSection, UsCodeTitle};

// =============================================================================
// SectionByIndexLens : UsCodeTitle ⇄ UsCodeSection (list-element focus).
// =============================================================================

/// Focuses the `index`-th section of a [`UsCodeTitle`] — the
/// list-element lens (Foster et al. 2007 §2.2). `get` reads the
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
// Sample fixtures — a minimal USLM section/title for the lens axioms.
// =============================================================================

/// A minimal, childless USLM section — enough to exercise the domain
/// lens laws (it projects to a `Statute` with the lens's name/version
/// and no subdivision terms).
pub fn sample_uslm_section() -> UsCodeSection {
    UsCodeSection {
        identifier: "/us/usc/t18/s1514A".to_string(),
        num: "1514A".to_string(),
        heading: "Civil action to protect against retaliation".to_string(),
        heading_runs: Vec::new(),
        chapeau: None,
        chapeau_runs: Vec::new(),
        content: None,
        content_runs: Vec::new(),
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
/// §2.2; Bancilhon & Spyratos 1981).
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
        "Foster et al. (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2; Bancilhon & Spyratos (1981) ACM TODS 6(4)"
    );
}

pr4xis::register_axiom!(
    UslmStatuteLensWellBehaved,
    "Foster et al. (2007) ACM TOPLAS 29(3) §2.2; Bancilhon & Spyratos (1981) ACM TODS 6(4)"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal::meta::lens_composition::{put_get_holds, put_put_holds};

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

    #[test]
    fn section_lens_out_of_range() {
        let title = sample_uslm_title();
        assert!(SectionByIndexLens { index: 5 }.get(&title).is_err());
    }

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

    #[test]
    fn composed_chain_title_to_statute() {
        let chain = title_to_statute_lens(0, "sox_1514a", "2002");
        let title = sample_uslm_title();
        let st = chain.get(&title).unwrap();
        assert_eq!(st.name(), "sox_1514a");
        assert!(get_put_holds(&chain, &title));
    }

    #[test]
    fn axiom_domain_lens_well_behaved() {
        assert!(UslmStatuteLensWellBehaved.verify().is_ok());
    }

    #[test]
    fn axiom_chain_composes() {
        assert!(StatuteChainComposes.verify().is_ok());
    }
}
