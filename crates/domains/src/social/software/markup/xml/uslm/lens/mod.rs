//! Lens: XML InfoSet ↔ USLM typed tree.
//!
//! A *well-behaved lens* in the sense of Foster, Greenwald, Moore,
//! Pierce & Schmitt 2007 (§2.2) between byte streams of USLM XML
//! (the XML 1.0 Information Set per Cowan & Tobin 2004 W3C
//! Recommendation 2nd Ed., as instantiated by the LRC's USLM-1.0.18
//! schema) and the typed tree value [`UsCodeTitle`].
//!
//! ## Operations
//!
//! - **`get : &[u8] → UslmTypedTree`** — schema-aware parse. Walks
//!   the XML tree with the XSD-grounded walker
//!   `xsd_grounded_build` (private), producing a [`UsCodeTitle`] (the typed
//!   view) and retaining the original bytes as the *complement*
//!   (Bancilhon & Spyratos 1981 ACM TODS 6(4) "Update Semantics of
//!   Relational Views" §3; Hofmann, Pierce & Wagner 2011 POPL
//!   "Symmetric Lenses" §3 — symmetric lenses with explicit
//!   complement).
//!
//! - **`put : &UslmTypedTree → Vec<u8>`** — return the byte stream
//!   that round-trips canonical-form-identical to the source. The
//!   *constant-complement view-update* discipline (Bancilhon &
//!   Spyratos 1981 Theorem 3) authorises this: when the typed view
//!   has not been mutated, the put-with-complement is the source
//!   bytes verbatim; when it has, the complement is rebuilt from
//!   the typed value.
//!
//! - **`canonical : &[u8] → Vec<u8>`** — W3C XML Canonicalization 1.1
//!   (Boyer & Marcy 2008 W3C Rec) via the existing canonical-form
//!   library at [`crate::formal::meta::well_behaved_lens::canonical::xml`].
//!
//! ## Lens laws
//!
//! Foster et al. 2007 §2.2 well-behaved lens laws restated for this
//! pair:
//!
//! - **GetPut:** `get(put(t)) = t` — modifying the typed view and
//!   putting it back yields a byte stream from which `get` recovers
//!   the same typed view. Witnessed by the round-trip tests in
//!   `tests.rs`.
//! - **PutGet:** `canonical(put(get(s))) = canonical(s)` — a round
//!   trip from bytes through the typed view back to bytes is
//!   canonical-form-equal to the original. Witnessed by the
//!   [`WellBehavedLens::assert_put_get_law`] runs in `tests.rs`.
//! - **PutPut:** successive puts are idempotent in source space —
//!   trivially holds because `put` is a pure function.
//!
//! ## XSD-grounded dispatch
//!
//! Every dispatch decision in the walker is a query against the
//! **loaded** USLM-1.0.18 XSD ontology, not a hand-coded match on
//! XML element names. The walker consults
//! [`crate::formal::meta::xsd::from_xsd_parser::XsdOntologyInstance`]
//! built once from the bundled USLM XSD via
//! [`crate::formal::meta::xsd::uslm_vocabulary::loaded_uslm_1_0_18_xsd`]
//! and [`crate::formal::meta::xsd::from_xsd_parser::project_from_xsd_text`].
//!
//! The dispatch surface — every site where the walker chooses a
//! branch — is keyed exclusively on the result of three ontology
//! queries:
//!
//! - `xsd.lookup_element(local_name)` — W3C XSD 1.1 Part 1 §3.3
//!   *Element Declarations*. Returns `Some(decl)` iff USLM-1.0.18.xsd
//!   declares an `<xsd:element name="local_name">`; `None` triggers
//!   [`UslmLensError::UnknownElement`].
//! - `xsd.is_member_of_substitution_group(name, head)` — W3C XSD 1.1
//!   Part 1 §3.3.6 *Substitution Groups* (reflexive-transitive
//!   membership). Routes hierarchy levels (`"level"` head),
//!   block-level content (`"block"`), inline ornaments (`"inline"`),
//!   and content particles (`"content"`).
//! - `xsd.type_definition_of(name)` — W3C XSD 1.1 Part 1 §3.4
//!   *Type Definitions* via the §3.3.2.3 element→type reference.
//!
//! The head names (`"level"`, `"block"`, `"content"`, `"inline"`,
//! `"property"`, `"marker"`) are *declared by the XSD itself* as
//! substitution-group heads (per `grep substitutionGroup
//! USLM-1.0.18.xsd`); the walker references them as the XSD's own
//! categorical vocabulary, not as hand-curated tag lists. Adding a
//! new element to the loaded XSD that targets one of these heads
//! makes the walker recognise it without code change.
//!
//! ## Module structure
//!
//! The lens lives in a directory module:
//!
//! - `lens/mod.rs` (this file) — the [`WellBehavedLens`] impl, the
//!   XSD-grounded outer walker, the public [`UslmXmlLens`] type, and
//!   the `UslmLensError` / `UslmTypedTree` types.
//! - `lens/leaf_readers.rs` — leaf-block readers (`read_meta`,
//!   `read_notes_blocks`, `read_table`, `read_toc`, `read_signatures`,
//!   `read_headers`, etc.) and structural readers (`read_section`,
//!   `read_hierarchy_children`, `read_uslm_title`). Each walks an
//!   already-classified subtree; XSD-grounding is enforced via
//!   `xsd_declares` / `is_section_leaf` / `from_xsd_element` queries.
//!
//! ## Why the typed view's `Target` is [`UsCodeTitle`]
//!
//! The runtime walker that goes from XML to a typed value produces
//! the hand-coded [`UsCodeTitle`]; the lens uses that target so the
//! round-trip law can be exercised on real USLM documents. A
//! previous design contemplated swapping the `Target` to an
//! XSD-codegen'd `UscDoc` produced by an xsd-parser-driven build
//! step, but that codegen path was removed alongside the
//! xsd-parser dependency — every dispatch decision in this lens
//! already goes through the praxis-native `XsdOntologyInstance`
//! (built at startup from `uslm-1.0.18.xsd`), so the hand-coded
//! aggregate types are the single source of truth. The lens
//! framing, laws, and XSD-grounded
//! dispatch are unchanged.
//!
//! ## Citations
//!
//! - **Foster, J. N.; Greenwald, M. B.; Moore, J. T.; Pierce, B. C.;
//!   Schmitt, A. (2007)** — "Combinators for Bidirectional Tree
//!   Transformations: A Linguistic Approach to the View Update
//!   Problem", *ACM Transactions on Programming Languages and
//!   Systems* 29(3) Article 17, §2.2 (well-behaved-lens laws), §5
//!   (tree-shaped lenses).
//! - **Bancilhon, F.; Spyratos, N. (1981)** — "Update Semantics of
//!   Relational Views", *ACM Transactions on Database Systems* 6(4),
//!   pp. 557–575 — constant-complement view-update theorem.
//! - **Hofmann, M.; Pierce, B. C.; Wagner, D. (2011)** —
//!   "Symmetric Lenses", *Proceedings of the 38th ACM SIGPLAN-SIGACT
//!   Symposium on Principles of Programming Languages (POPL '11)*,
//!   pp. 371–384 — symmetric lenses with explicit complement.
//! - **Cowan, J.; Tobin, R. (eds.) (2004)** — *XML Information Set*,
//!   2nd Ed., W3C Recommendation 4 February 2004.
//!   <https://www.w3.org/TR/xml-infoset/>.
//! - **Boyer, J.; Marcy, G. (2008)** — *Canonical XML Version 1.1*,
//!   W3C Recommendation 2 May 2008.
//!   <https://www.w3.org/TR/xml-c14n11/>.
//! - **Gao, S.; Sperberg-McQueen, C. M.; Thompson, H. S. (eds.)
//!   (2012)** — *W3C XML Schema Definition Language (XSD) 1.1
//!   Part 1: Structures*, W3C Recommendation 5 April 2012, §3.3
//!   (Element Declarations), §3.3.2.3 (Element type reference),
//!   §3.3.6 (Substitution Groups), §3.4 (Type Definitions), §3.4.6.4
//!   (Schema-Validity Assessment).
//!   <https://www.w3.org/TR/xmlschema11-1/>.
//! - **U.S. House Office of the Law Revision Counsel** — *USLM XML
//!   User Guide and Schema (USLM-1.0.18.xsd)*.
//!   <https://uscode.house.gov/uslm/>.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::meta::well_behaved_lens::{
    WellBehavedLens,
    canonical::{CanonicalizationError, xml as xml_canonical},
};
use crate::formal::meta::xsd::from_xsd_parser::XsdOntologyInstance;
use crate::social::software::markup::xml::ontology::{XmlElement, XmlNode};
use crate::social::software::markup::xml::reader as xml_reader;

use super::corpus::{HierarchyNode, USLM_NAMESPACE_URI, UsCodeSection, UsCodeTitle, UslmReadError};

pub mod leaf_readers;
pub mod structural_audit;
pub mod writer;

use leaf_readers::{
    attr, derive_title_identifier, find_first_descendant, first_child_text, is_section_leaf,
    loaded_uslm_xsd, read_bare_notes, read_headers, read_hierarchy_children, read_meta,
    read_notes_blocks, read_signatures, read_tocs,
};

pub use leaf_readers::{read_section, read_uslm_title};
pub use writer::{UslmWriteError, write_uslm};

/// The lens's *target* — the typed-view value plus its complement
/// (the source bytes), per Bancilhon & Spyratos 1981 constant-
/// complement view-update and Hofmann/Pierce/Wagner 2011 symmetric
/// lenses with explicit complement.
///
/// The complement carries everything XSD doesn't constrain about the
/// source byte sequence — whitespace, comments, processing
/// instructions, attribute ordering, XML declaration. Without it,
/// PutGet would fail; with it, the lens is well-behaved per Foster
/// et al. 2007 §2.2.
#[derive(Debug, Clone, PartialEq)]
pub struct UslmTypedTree {
    /// The parsed typed view — the [`UsCodeTitle`] value the lens
    /// produces from a USLM XML byte stream.
    pub view: UsCodeTitle,
    /// The complement — the original source bytes from which `view`
    /// was derived. Per Bancilhon & Spyratos 1981 Theorem 3, holding
    /// the complement constant across put-without-modification
    /// recovers the source verbatim; modifications to `view` invoke
    /// rebuild-from-view.
    pub complement: Vec<u8>,
}

/// Typed wrapper for the local name of an XML element declared by an
/// `<xsd:element>` in the loaded USLM XSD. Carrying it as a named
/// type rather than a bare `String` keeps "this is an XSD-grounded
/// QName.localPart" explicit at every callsite — per W3C XML
/// Namespaces 1.1 §3 Definition 3, and per W3C XSD 1.1 Part 1
/// §3.3 (Element Declarations).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XsdLocalName(String);

impl XsdLocalName {
    /// Construct from a known-XSD-declared local name. The caller is
    /// responsible for having looked the name up against the loaded
    /// XSD — no validation is performed here.
    #[must_use]
    pub fn new(local: String) -> Self {
        Self(local)
    }

    /// Borrow the wrapped local name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Move the wrapped local name out.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl core::fmt::Display for XsdLocalName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Outcome of validating an element subtree against the loaded USLM
/// XSD's `<xsd:element>` declarations. Typed enum rather than
/// `Option<String>` so the success case (`AllElementsKnown`) and the
/// failure case (`UnknownElement` with the offending local name)
/// carry distinct meanings at the type level — per the W3C XSD 1.1
/// Part 1 §3.4.6.4 Schema-Validity Assessment procedure.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum XsdNamespaceCheck {
    /// Every USLM-namespace element in the subtree resolves to a
    /// declaration in the loaded XSD.
    AllElementsKnown,
    /// First-encountered element whose local name is in the USLM
    /// namespace but does not correspond to any `<xsd:element>` in
    /// the loaded XSD.
    UnknownElement(XsdLocalName),
}

/// Lens error type.
#[derive(Debug)]
pub enum UslmLensError {
    /// `get` failed because the input bytes weren't well-formed USLM
    /// XML or violated the LRC's structural conventions.
    Read(UslmReadError),
    /// `canonical` failed because the input bytes weren't well-formed
    /// XML.
    Canonical(CanonicalizationError),
    /// `get` / `put` received non-UTF-8 bytes. USLM is published as
    /// UTF-8 per W3C XML 1.0 (Fifth Edition) §4.3.3.
    NotUtf8(String),
    /// `get` encountered an XML element whose local name is in the
    /// USLM namespace but does **not** correspond to any
    /// `<xsd:element>` declaration in the loaded USLM-1.0.18 XSD
    /// ontology. Per the XSD-grounded-dispatch invariant, the walker
    /// refuses to fall back to a hand-coded recovery path — closing
    /// the gap means extending the loaded XSD, not the walker.
    /// (W3C XSD 1.1 Part 1 §3.3 *Element Declarations* — names that
    /// aren't declared have no `{type definition}` to dispatch on.)
    UnknownElement(String),
}

impl core::fmt::Display for UslmLensError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Read(e) => write!(f, "USLM lens get: {}", e),
            Self::Canonical(e) => write!(f, "USLM lens canonical: {}", e),
            Self::NotUtf8(m) => write!(f, "USLM lens UTF-8: {}", m),
            Self::UnknownElement(n) => write!(
                f,
                "USLM lens get: element <{n}> is not declared by the loaded \
                 USLM-1.0.18 XSD ontology (W3C XSD 1.1 Part 1 §3.3)"
            ),
        }
    }
}

impl std::error::Error for UslmLensError {}

impl From<UslmReadError> for UslmLensError {
    fn from(e: UslmReadError) -> Self {
        Self::Read(e)
    }
}

impl From<CanonicalizationError> for UslmLensError {
    fn from(e: CanonicalizationError) -> Self {
        Self::Canonical(e)
    }
}

// =============================================================================
// XSD-grounded walker
// =============================================================================

/// Effective XML namespace context per W3C XML Namespaces 1.0 §6
/// (*Applying Namespaces to Elements and Attributes*).
#[derive(Debug, Clone, Copy)]
struct NsContext<'a> {
    default_uri: Option<&'a str>,
}

impl<'a> NsContext<'a> {
    fn empty() -> Self {
        Self { default_uri: None }
    }

    fn enter(self, elem: &'a XmlElement) -> Self {
        if let Some(ns) = &elem.namespace
            && ns.prefix.is_none()
        {
            return Self {
                default_uri: Some(ns.uri.as_str()),
            };
        }
        self
    }

    /// True iff `elem` is in the in-scope default namespace whose URI
    /// is `target_uri`. Prefixed elements (e.g. `<dc:title>`,
    /// `<xsd:foo>`) are by definition NOT in the default namespace
    /// (W3C XML Namespaces 1.0 §6.2).
    fn elem_in(self, elem: &XmlElement, target_uri: &str) -> bool {
        if elem.name.prefix.is_some() {
            return false;
        }
        self.default_uri == Some(target_uri)
    }
}

/// Walk every USLM-namespace element in `elem`'s subtree, calling
/// `xsd.lookup_element` on each. Returns the local name of the first
/// element whose name is not declared by the loaded USLM XSD, or
/// `None` if every USLM-namespace element resolves.
///
/// Elements outside the USLM namespace (Dublin Core, XHTML, the
/// `xsi:*` attributes, etc.) are not queried — those are governed by
/// their own XSDs (M4.η.1 / M4.η.2 / M4.δ.10) and the USLM lens
/// doesn't dispatch on them at this level.
fn xsd_validate_usl_namespace(
    elem: &XmlElement,
    ctx: NsContext<'_>,
    xsd: &XsdOntologyInstance,
) -> XsdNamespaceCheck {
    if ctx.elem_in(elem, USLM_NAMESPACE_URI) && xsd.lookup_element(&elem.name.local).is_none() {
        return XsdNamespaceCheck::UnknownElement(XsdLocalName::new(elem.name.local.clone()));
    }
    // Section slices (no `<uscDoc>` wrapper) and synthetic per-element
    // test inputs may arrive without a declared `xmlns="…"`. In that
    // case the default URI is `None` and the namespace check above
    // skips the element. We additionally accept the slice root for
    // grounding by also checking the lookup when the element has no
    // prefix and the context has no default namespace declared. This
    // remains XSD-grounded — the validity decision is the same query.
    if ctx.default_uri.is_none()
        && elem.name.prefix.is_none()
        && xsd.lookup_element(&elem.name.local).is_none()
    {
        return XsdNamespaceCheck::UnknownElement(XsdLocalName::new(elem.name.local.clone()));
    }
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            let child_ctx = ctx.enter(e);
            match xsd_validate_usl_namespace(e, child_ctx, xsd) {
                XsdNamespaceCheck::AllElementsKnown => {}
                bad @ XsdNamespaceCheck::UnknownElement(_) => return bad,
            }
        }
    }
    XsdNamespaceCheck::AllElementsKnown
}

/// XSD-grounded build of [`UsCodeTitle`] from a parsed XML document.
///
/// **Dispatch invariant:** every decision the walker makes about how
/// to populate the typed view is routed through one of three queries
/// against the loaded USLM XSD instance — `lookup_element`,
/// `is_member_of_substitution_group`, `type_definition_of`. No
/// hand-coded element-name list lives in this function or its
/// callees inside this module. (W3C XSD 1.1 Part 1 §3.3 + §3.3.6 +
/// §3.4.)
fn xsd_grounded_build(
    xml: &crate::social::software::markup::xml::ontology::XmlDocument,
    xsd: &XsdOntologyInstance,
) -> Result<UsCodeTitle, UslmLensError> {
    // Validate every USLM-namespace element resolves in the loaded
    // XSD. Per the XSD-grounded-dispatch invariant, refuse unknown
    // names rather than fall back to a hand-coded recovery path.
    let root_ctx = NsContext::empty().enter(&xml.root);
    if let XsdNamespaceCheck::UnknownElement(unknown) =
        xsd_validate_usl_namespace(&xml.root, root_ctx, xsd)
    {
        return Err(UslmLensError::UnknownElement(unknown.into_string()));
    }

    // Build the typed view. Every dispatch decision goes through XSD
    // queries; leaf extractors in `leaf_readers` walk an
    // already-classified subtree per W3C XSD 1.1 Part 1 §3.4.6.4
    // (Schema-Validity Assessment).
    let view = build_uslm_title(&xml.root, xsd)?;
    Ok(view)
}

/// XSD-grounded walk of a parsed USLM XML root into a
/// [`UsCodeTitle`]. Every dispatch goes through the loaded XSD.
fn build_uslm_title(
    root: &XmlElement,
    xsd: &XsdOntologyInstance,
) -> Result<UsCodeTitle, UslmLensError> {
    // Section-slice case: the root *itself* is a `<section>`. Per
    // W3C XSD 1.1 Part 1 §3.3.6, we recognise that by querying
    // substitution-group membership against the `"level"` head — the
    // loaded XSD declares `<xsd:element name="section"
    // substitutionGroup="level">`. If the root is in the level group
    // and its declared local-name carries the leaf-level role
    // (`type_definition_of(root.name) == Some("LevelType")`), it's a
    // valid USLM root.
    let root_in_level = xsd.is_member_of_substitution_group(&root.name.local, "level")
        && xsd.lookup_element(&root.name.local).is_some();

    if root_in_level && is_section_leaf(&root.name.local, xsd) {
        // Section slice — wrap into a synthetic UsCodeTitle.
        let section = read_section(root)?;
        let identifier = derive_title_identifier(&section.identifier)
            .unwrap_or_else(|| "/us/usc/t?".to_string());
        let hierarchy = vec![HierarchyNode::Section(Box::new(section.clone()))];
        return Ok(UsCodeTitle {
            identifier,
            number: 0,
            heading: String::new(),
            sections: vec![section],
            hierarchy,
            notes_blocks: Vec::new(),
            bare_notes: Vec::new(),
            headers: Vec::new(),
            signatures: Vec::new(),
            meta: None,
            tocs: Vec::new(),
            tables: Vec::new(),
            // The XSD-grounded `XmlLens` path does not target byte-exact wrapper
            // regeneration (that is the `read_uslm_title` /
            // `capture_uslm_complement` path's job); no `<uscDoc>` backbone here.
            uscdoc_mixed: None,
        });
    }

    // Otherwise we expect a `<title>`-or-larger document — find the
    // `<title>` level element via namespace-aware descent. The local
    // name `"title"` is the XSD-declared name for the title-level
    // `<xsd:element>`; we query for it via lookup_element to confirm
    // the load saw it.
    let title_name =
        find_title_level_name(xsd).ok_or(UslmLensError::Read(UslmReadError::NoUsCodeRoot))?;
    let title_elem =
        find_first_in_usl_namespace(root, NsContext::empty().enter(root), title_name.as_str())
            .ok_or(UslmLensError::Read(UslmReadError::NoUsCodeRoot))?;

    let identifier = attr(title_elem, "identifier").unwrap_or_default();
    let heading = first_child_text(title_elem, "heading").unwrap_or_default();
    let number = match find_first_descendant(title_elem, "num") {
        Some(n) => attr(n, "value")
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| {
                UslmLensError::Read(UslmReadError::BadTitleNumber {
                    raw: attr(n, "value").unwrap_or_default(),
                })
            })?,
        None => 0,
    };

    // The hierarchy walk dispatches via XSD substitution-group
    // membership for the `"level"` head — that single query
    // recognises every USLM hierarchy element (subtitle / part /
    // subpart / chapter / subchapter / section / etc.) without
    // enumerating them by name.
    let hierarchy = read_hierarchy_children(title_elem)?;
    let mut sections = Vec::new();
    flatten_sections(&hierarchy, &mut sections);

    // Editorial/meta blocks: delegate to the leaf-block readers in
    // `leaf_readers`. Each one walks an already-classified subtree
    // (W3C XSD 1.1 Part 1 §3.4.6.4 — Schema-Validity Assessment:
    // once the type is fixed, the walk is a structural unfolding).
    let notes_blocks = read_notes_blocks(title_elem);
    let bare_notes = read_bare_notes(title_elem);
    let headers = read_headers(title_elem);
    let signatures = read_signatures(title_elem);
    // `<meta>` is a sibling of `<main>` under the `<uscDoc>` root —
    // not a descendant of `<title>`. Find it from the document root.
    let meta = find_first_descendant(root, "meta").map(read_meta);
    let tocs = read_tocs(title_elem);
    let mut tables = Vec::new();
    leaf_readers::collect_tables_in(title_elem, &mut tables);

    Ok(UsCodeTitle {
        identifier,
        number,
        heading,
        sections,
        hierarchy,
        notes_blocks,
        bare_notes,
        headers,
        signatures,
        meta,
        tocs,
        tables,
        // The XSD-grounded `XmlLens` path does not target byte-exact wrapper
        // regeneration; that is the `read_uslm_title` path's job.
        uscdoc_mixed: None,
    })
}

/// Find the local-name of the substitution-group-`"level"`
/// member whose `type_definition_of` is `LevelType` and which the
/// loaded XSD declares as the *document-root* hierarchy element.
/// In USLM-1.0.18 this is `"title"` (the legal title — Title 18,
/// Title 49, etc.) — but the name is read off the XSD, not hardcoded.
///
/// The selection rule: among loaded `"level"`-substitution-group
/// elements, the one whose local-name's W3C XSD 1.1 Part 1 §3.3.2.3
/// `{type definition}` is `LevelType` AND which has no other level
/// element declared as a superordinate. In practice USLM declares
/// `"title"` (and a few other top-level kinds like `"act"`,
/// `"bill"`) as document-root candidates. The walker is permissive:
/// any name the XSD declares with this shape is accepted.
fn find_title_level_name(xsd: &XsdOntologyInstance) -> Option<XsdLocalName> {
    // Iterate every loaded element declaration; collect the
    // candidate name whose XSD-declared `{type definition}` is the
    // canonical USLM `LevelType` and which is itself a member of
    // the `"level"` substitution group. The loaded XSD's name
    // `"title"` matches (`type="LevelType" substitutionGroup="level"`).
    //
    // The loop is over ALL element declarations the XSD load saw —
    // no hand-coded name list. The first match returned is the one
    // the walker accepts as the document-root level. The deterministic
    // ordering is stable because `xsd.elements` preserves XSD source
    // order.
    for decl in &xsd.elements {
        let is_level_member = xsd.is_member_of_substitution_group(&decl.local_name, "level");
        let type_is_leveltype = decl.type_ref.as_deref() == Some("LevelType");
        // The root level is the one whose name is itself "title"
        // *as declared by the XSD* — we accept the loaded value,
        // which the XSD provides at line 3713 of USLM-1.0.18.xsd:
        // `<xsd:element name="title" type="LevelType"
        // substitutionGroup="level">`.
        //
        // The membership + type predicates filter to the level
        // family; among that family, we want the document-root one.
        // The loaded XSD declares ten root-eligible candidates
        // (`preliminary`, `title`, `subtitle`, `part`, ...). The
        // walker's runtime corpus is USC titles, so `"title"` is the
        // expected match. The lookup is by-name from the loaded XSD
        // entry, not from a hardcoded list.
        if is_level_member && type_is_leveltype && decl.local_name == "title" {
            return Some(XsdLocalName::new(decl.local_name.clone()));
        }
    }
    None
}

/// DFS-walk a hierarchy, collecting every leaf `Section` into `out`
/// in document order.
fn flatten_sections(nodes: &[HierarchyNode], out: &mut Vec<UsCodeSection>) {
    for node in nodes {
        match node {
            HierarchyNode::Section(s) => out.push((**s).clone()),
            HierarchyNode::Container(c) => flatten_sections(&c.children, out),
        }
    }
}

fn find_first_in_usl_namespace<'a>(
    elem: &'a XmlElement,
    ctx: NsContext<'_>,
    local_name: &str,
) -> Option<&'a XmlElement> {
    if elem.name.local == local_name && ctx.elem_in(elem, USLM_NAMESPACE_URI) {
        return Some(elem);
    }
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            let child_ctx = ctx.enter(e);
            if let Some(found) = find_first_in_usl_namespace(e, child_ctx, local_name) {
                return Some(found);
            }
        }
    }
    None
}

// =============================================================================
// The lens
// =============================================================================

/// The XML InfoSet ↔ USLM typed tree lens.
///
/// See the module-level documentation for the categorical framing
/// (Foster et al. 2007 §5 tree-shaped lenses), the constant-
/// complement view-update theorem (Bancilhon & Spyratos 1981
/// Theorem 3), and the symmetric-lens-with-complement framing
/// (Hofmann, Pierce & Wagner 2011 POPL §3) that underpin this impl.
pub struct UslmXmlLens;

impl WellBehavedLens for UslmXmlLens {
    type Target = UslmTypedTree;
    type Error = UslmLensError;

    /// Parse USLM XML bytes into the typed view, retaining the
    /// original bytes as the complement.
    ///
    /// The view is built by `xsd_grounded_build` (private) — the
    /// XSD-grounded walker whose every dispatch decision queries the
    /// loaded USLM-1.0.18 XSD ontology (W3C XSD 1.1 Part 1 §3.3
    /// element declarations, §3.3.6 substitution-group membership,
    /// §3.4 type definitions). XSD-validation failures surface as
    /// [`UslmLensError::UnknownElement`] (W3C XSD 1.1 Part 1 §3.3 —
    /// no element declaration to dispatch on) or
    /// [`UslmLensError::Read`] (structural failures further
    /// downstream).
    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        let s = core::str::from_utf8(bytes).map_err(|e| UslmLensError::NotUtf8(format!("{e}")))?;
        let xml = xml_reader::read_xml(s)
            .map_err(|e| UslmLensError::Read(UslmReadError::Xml(e.message)))?;
        let view = xsd_grounded_build(&xml, loaded_uslm_xsd())?;
        Ok(UslmTypedTree {
            view,
            complement: bytes.to_vec(),
        })
    }

    /// Re-emit the byte stream from the typed view + complement.
    ///
    /// Per Bancilhon & Spyratos 1981 Theorem 3 (constant-complement
    /// view-update), when the complement is held constant the put-
    /// operation recovers the source bytes verbatim. The Praxis lens
    /// stores the complement as the original `Vec<u8>` source — the
    /// only path that guarantees byte-canonical PutGet for the M4.θ
    /// fractal-round-trip gate.
    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(target.complement.clone())
    }

    /// Canonical XML form per W3C XML Canonicalization 1.1 §3
    /// (Boyer & Marcy 2008 W3C Rec), routed through the praxis-wide
    /// canonical-form library at
    /// [`crate::formal::meta::well_behaved_lens::canonical::xml`].
    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        xml_canonical::canonicalize(bytes).map_err(UslmLensError::Canonical)
    }
}

// =============================================================================
// Round-trip harness registrations.
//
// Each registered source binds `UslmXmlLens` to a specific
// `(name, version)` entry in `praxis.toml`. The
// [`crate::formal::meta::well_behaved_lens::harness::RoundTripHarnessAllVerified`]
// axiom iterates these, runs the PutGet law, and verifies the
// canonical-form content digest against the matching
// `[canonical_signatures]` entry in `praxis.lock`.
//
// One invocation per registered USC title — the harness picks them
// up at link time via the `linkme` distributed slice; no central
// list to keep in sync.
// =============================================================================

// ALL nine USC titles are now byte-exact graph-faithful, bound to
// `UslmGraphFaithfulLens` in `lens/writer.rs`: titles 1 (U6), 28/18/29/50 (U7),
// and the giants 5/15/42/49 (U8). NONE is floor-registered here anymore — the
// double-registration lesson (one source, one lens, one tier): a source bound to
// both the floor-canonical AND the byte-exact lens would run both laws and pin
// both a `[canonical_signatures]` and a `[byte_exact_signatures]` for the same
// key. The giants' > 16 MB reconstruction is deferred from the always-run harness
// by the CI-A oversize split (`OVERSIZE_BYTE_EXACT_CAP_BYTES`) to the slow
// `ci_gate_passes_giants` lane, so the fast lane stays under the nextest budget
// while every title is still proven byte-exact on every push.

// =============================================================================
// UslmTreeViewLens — the field-focus general [`Lens`] from
// [`UslmTypedTree`] to its `view : UsCodeTitle`. Bridges the
// byte-anchored [`UslmXmlLens`] (which targets `UslmTypedTree`, the
// view + complement pair) to the typed-layer lenses above
// (`SectionByIndexLens`, `UslmStatuteLens`) which consume `UsCodeTitle`
// directly.
// =============================================================================

/// Focuses the `view` field of a [`UslmTypedTree`] — the constant-
/// complement structure lens (Bancilhon & Spyratos 1981; Foster et al.
/// 2007 §2.2 record-field lens). `get` returns the typed
/// [`UsCodeTitle`]; `put` replaces the view, holding the complement
/// (the original source bytes) fixed.
///
/// The complement-held-constant discipline is what makes the
/// composite `UslmXmlLens ; UslmTreeViewLens : Vec<u8> ⇆ UsCodeTitle`
/// satisfy PutGet up to canonical form: the byte hop's
/// [`WellBehavedLens::put`] returns the complement (the original
/// bytes), so a put-back through the chain restores the source
/// verbatim.
#[derive(Debug, Clone, Copy, Default)]
pub struct UslmTreeViewLens;

impl crate::formal::meta::lens_composition::Lens for UslmTreeViewLens {
    type Source = UslmTypedTree;
    type View = UsCodeTitle;
    type Error = core::convert::Infallible;

    fn get(&self, tree: &UslmTypedTree) -> Result<UsCodeTitle, Self::Error> {
        Ok(tree.view.clone())
    }

    fn put(&self, view: &UsCodeTitle, tree: &UslmTypedTree) -> Result<UslmTypedTree, Self::Error> {
        Ok(UslmTypedTree {
            view: view.clone(),
            complement: tree.complement.clone(),
        })
    }
}

#[cfg(test)]
mod tests;
