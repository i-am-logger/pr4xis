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
use super::UsCodeRef;
use crate::formal::meta::identifier_format::Identifier;

/// One subdivision node inside a `<section>`. Every USLM level-group
/// element below the section leaf (subsection / paragraph /
/// subparagraph / clause / subclause / item / subitem) produces one
/// node — recursing through children.
///
/// OWNED, not `&'static`: the runtime corpus constructors
/// (`from_uslm_titles_owned`, the compact/rkyv archive loads) build these
/// trees from parsed source, and a TRANSIENT corpus — the wasm load path
/// projects a title into its `RuntimeOntology` and then drops the owned
/// `UsCode` — must actually return its memory. The former `&'static`
/// fields forced every runtime constructor through `Box::leak`, pinning
/// ~24 MiB per loaded title forever (the audit-5 post-install retention
/// finding); the build-time codegen that once justified frozen static
/// tables is retired (M4.δ.7.a).
#[derive(Debug, Clone)]
pub struct UscSubdivision {
    /// Typed USLM URN within the section's URN space — e.g.
    /// `/us/usc/t18/s1514A/a/1/A`. The URN comes verbatim from the LRC
    /// `identifier` attribute and is grammar-validated on construction
    /// via [`Identifier::uslm_urn`].
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
    pub num: String,
    /// `<heading>` plain text, when present. `None` for subdivisions
    /// that carry no heading (most paragraphs / subparagraphs).
    pub heading: Option<String>,
    /// `<chapeau>` text — the introductory phrase a subdivision uses
    /// to introduce its enumerated children. `None` if the
    /// subdivision is a leaf.
    pub chapeau: Option<String>,
    /// `<content>` text — the body of a leaf subdivision. `None` if
    /// the subdivision only contains children (its content lives in
    /// `chapeau` plus the children's content).
    pub content: Option<String>,
    /// Nested subdivisions, in USLM document order.
    pub children: Vec<UscSubdivision>,
    /// `<ref href="…">` cross-references collected from THIS
    /// subdivision's own body surfaces (its `heading` / `chapeau` /
    /// `content`), in USLM document order — NOT from its
    /// [`children`][Self::children], which carry their own. Scoped
    /// per-node exactly as `heading` / `chapeau` / `content` are, so a
    /// citation is attributed to the smallest provision that literally
    /// contains it (LRC USLM XML User Guide §V hierarchy). Each entry is
    /// the parse-faithful [`UsCodeRef`] `{ href, text }` — the `href` a
    /// USLM identifier URN (`/us/usc/t15/s78`), carried VERBATIM; it may
    /// point OUTSIDE the loaded corpus (a sister title, a repealed
    /// provision), so it is a raw citation surface, not a resolved edge.
    pub refs: Vec<UsCodeRef>,
}

impl UscSubdivision {
    /// Iterate over self plus every descendant in pre-order. Useful
    /// for axiom tests that count subdivision nodes per section.
    pub fn descendants_including_self(&self) -> SubdivisionWalk<'_> {
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
#[derive(Debug, Clone)]
pub struct UscComposesEdge {
    /// Child URN — the subdivision that is a component of the
    /// parent. For example `/us/usc/t18/s1514A/a/1/A`.
    pub from_urn: String,
    /// Parent URN — the section or larger subdivision that contains
    /// the child. For example `/us/usc/t18/s1514A/a/1`.
    pub to_urn: String,
}

/// The aux record for one section — kept as a parallel array next to
/// the [`super::CodegenData<UsCode>`] static so the runtime functor
/// can attach subdivision data after materialising the flat section
/// list. One entry per section, indexed by URN-lookup against the
/// CodegenData entity ids.
#[derive(Debug, Clone)]
pub struct UscSectionAux {
    /// USLM URN of the section this aux record describes — used to
    /// join against the CodegenData entity ids at runtime.
    pub urn: String,
    /// Subdivision tree rooted at the section. Empty for sections
    /// with no enumerated subdivisions (placeholder reservations, or
    /// short prose-only sections).
    pub subdivisions: Vec<UscSubdivision>,
    /// Composes edges across the whole tree — every parent↔child
    /// containment within the section flattened into a single edge
    /// list. The section root is implicit (its URN equals
    /// [`Self::urn`]).
    pub relations: Vec<UscComposesEdge>,
    /// `<ref href="…">` cross-references collected from the SECTION
    /// root's own body surfaces (its `heading` / `chapeau` / `content`),
    /// in USLM document order — the section-scoped counterpart of
    /// [`UscSubdivision::refs`], carried on the aux record so the archive
    /// load path ([`super::UsCode::from_codegen_with_aux`]) can attach a
    /// section's own citations without a raw parse. A subdivision's
    /// citations ride its own [`UscSubdivision::refs`], not this list.
    pub refs: Vec<UsCodeRef>,
}

/// Pre-order walker over a borrowed subdivision subtree.
pub struct SubdivisionWalk<'a> {
    stack: alloc::vec::Vec<&'a UscSubdivision>,
}

impl<'a> Iterator for SubdivisionWalk<'a> {
    type Item = &'a UscSubdivision;

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
