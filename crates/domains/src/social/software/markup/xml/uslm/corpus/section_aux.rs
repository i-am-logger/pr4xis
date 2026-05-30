//! Section-level subdivision tree and Composes-relation graph for
//! [`super::UscSection`].
//!
//! The flat per-section `{ urn, heading, text }` carries the section as
//! a single leaf in the Layer-3 vocabulary. The aux types in this
//! module add the SUBDIVISION DEPTH: every subsection / paragraph /
//! subparagraph / clause / subclause / item / subitem within a section
//! is a typed [`UscSubdivision`] node with its own USLM URN, and each
//! parent ↔ child containment is one [`UscComposesEdge`].
//!
//! ## Layer position
//!
//! ```text
//! UscSection
//!  ├── urn        Identifier (USLM URN of the §)
//!  ├── heading    section heading text
//!  ├── text       chapeau + content text concatenated
//!  ├── subdivisions: &[UscSubdivision]  ← *new*
//!  └── relations:   &[UscComposesEdge]  ← *new*
//! ```
//!
//! Each subdivision carries its own URN per LRC USLM XML User Guide §V
//! (USC hierarchy) — e.g. `/us/usc/t18/s1514A/a/1/A`. The
//! [`SubdivisionKind`] tag comes from cee1f68's XSD-grounded
//! `SubdivisionKind::from_xsd_element` (W3C XSD 1.1 Part 1 §3.3.6
//! substitution groups, `substitutionGroup="level"`); the codegen
//! emitter projects element names verbatim and the runtime accepts
//! whatever the type validates.
//!
//! ## Literature
//!
//! - U.S. House Office of the Law Revision Counsel, *USLM XML User
//!   Guide (USLM-1.0.18.xsd)* §V (USC hierarchy).
//!   <https://uscode.house.gov/uslm/>.
//! - W3C, *XML Schema 1.1 Part 1: Structures* §3.3.6 (Substitution
//!   Groups). <https://www.w3.org/TR/xmlschema11-1/#cElement_Declarations>.

use super::SubdivisionKind;
use crate::formal::meta::identifier_format::Identifier;

/// One subdivision node inside a `<section>`. Every USLM level-group
/// element below the section leaf (subsection / paragraph /
/// subparagraph / clause / subclause / item / subitem) produces one
/// node — recursing through children.
///
/// All slices are `&'static` because the codegen emits this data as
/// frozen tables in the generated module; downstream consumers borrow
/// these slices directly without allocation.
///
/// Not `Copy` because [`Identifier`] holds a `Cow<'static, str>` —
/// the Copy bound is rejected by the compiler for that variant.
/// Cloning a UscSubdivision still copies all slice handles (which
/// are 'static) and the inner Cow::Borrowed (which is itself
/// trivially-cloneable).
#[derive(Debug, Clone)]
pub struct UscSubdivision {
    /// Typed USLM URN within the section's URN space — e.g.
    /// `/us/usc/t18/s1514A/a/1/A`. Grammar-validated at codegen time
    /// (the URN comes verbatim from the LRC `identifier` attribute);
    /// const-constructed here via
    /// [`Identifier::from_codegen_static`].
    pub urn: Identifier,
    /// Subdivision kind — `Subsection` / `Paragraph` / `Subparagraph`
    /// / `Clause` / `Subclause` / `Item` / `Subitem`. Per cee1f68
    /// (M4.ε.5.a XSD grounding) the dispatch from USLM element name
    /// to this enum runs through
    /// [`SubdivisionKind::from_xsd_element`] — codegen emits the
    /// variant directly, runtime can re-validate against the loaded
    /// XSD ontology.
    pub kind: SubdivisionKind,
    /// `<num>` value verbatim — e.g. `"a"`, `"1"`, `"A"`.
    pub num: &'static str,
    /// `<heading>` plain text, when present. `None` for subdivisions
    /// that carry no heading (most paragraphs / subparagraphs).
    pub heading: Option<&'static str>,
    /// `<chapeau>` text — the introductory phrase a subdivision uses
    /// to introduce its enumerated children. `None` if the
    /// subdivision is a leaf.
    pub chapeau: Option<&'static str>,
    /// `<content>` text — the body of a leaf subdivision. `None` if
    /// the subdivision only contains children (its content lives in
    /// `chapeau` plus the children's content).
    pub content: Option<&'static str>,
    /// Nested subdivisions, in USLM document order.
    pub children: &'static [UscSubdivision],
}

impl UscSubdivision {
    /// Iterate over self plus every descendant in pre-order. Useful
    /// for axiom tests that count subdivision nodes per section.
    pub fn descendants_including_self(&'static self) -> SubdivisionWalk {
        SubdivisionWalk {
            stack: alloc::vec![self],
        }
    }
}

/// One Composes-mereology edge between a parent URN and a child URN
/// within a section's subdivision tree. Both endpoints are raw URN
/// strings (the canonical USLM identifier attribute); callers that
/// need a typed [`Identifier`] reconstruct it via
/// [`Identifier::uslm_urn`].
///
/// Direction: `from_urn` is the CHILD (the component); `to_urn` is
/// the PARENT (the whole). This matches the praxis mereology
/// convention where the relation reads "child Composes-into
/// parent".
#[derive(Debug, Clone, Copy)]
pub struct UscComposesEdge {
    /// Child URN — the subdivision that is a component of the
    /// parent. For example `/us/usc/t18/s1514A/a/1/A`.
    pub from_urn: &'static str,
    /// Parent URN — the section or larger subdivision that contains
    /// the child. For example `/us/usc/t18/s1514A/a/1`.
    pub to_urn: &'static str,
}

/// The aux record for one section — kept as a parallel array next to
/// the [`super::CodegenData<UsCode>`] static so the runtime functor
/// can attach subdivision data after materialising the flat section
/// list. One entry per section, indexed by URN-lookup against the
/// CodegenData entity ids.
#[derive(Debug, Clone, Copy)]
pub struct UscSectionAux {
    /// USLM URN of the section this aux record describes — used to
    /// join against the CodegenData entity ids at runtime.
    pub urn: &'static str,
    /// Subdivision tree rooted at the section. Empty for sections
    /// with no enumerated subdivisions (placeholder reservations, or
    /// short prose-only sections).
    pub subdivisions: &'static [UscSubdivision],
    /// Composes edges across the whole tree — every parent↔child
    /// containment within the section flattened into a single edge
    /// list. The section root is implicit (its URN equals
    /// [`Self::urn`]).
    pub relations: &'static [UscComposesEdge],
}

/// Pre-order walker over a static subdivision subtree.
pub struct SubdivisionWalk {
    stack: alloc::vec::Vec<&'static UscSubdivision>,
}

impl Iterator for SubdivisionWalk {
    type Item = &'static UscSubdivision;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        // Push children in reverse so pre-order iteration sees them
        // in source order.
        for child in node.children.iter().rev() {
            self.stack.push(child);
        }
        Some(node)
    }
}
