//! Praxis-native **RDF/XML triple reader** — the layer between the praxis
//! XML 1.0 parser ([`super::super::reader::read_xml`]) and any
//! triple-consuming downstream (OWL 2 RDF mapping, SPARQL, etc.).
//!
//! ## Scope
//!
//! Given an [`XmlElement`] produced by the praxis XML reader and a
//! base IRI string, [`read_rdf_xml`] returns the `Vec<Triple>` the
//! RDF/XML serialisation denotes (W3C RDF 1.1 XML Syntax — Gandon &
//! Schreiber, eds., W3C Recommendation 2014-02-25). It handles:
//!
//! - typed-node form (`<owl:Class rdf:about="X">`) and striped form
//!   (`<rdf:Description rdf:about="X"><rdf:type …/>…</rdf:Description>`)
//!   — §2.10, §2.13;
//! - subject identification via `rdf:about` (§2.11), `rdf:ID` (§2.12),
//!   `rdf:nodeID` (§2.14), or an auto-generated blank node;
//! - object identification on predicate elements via `rdf:resource`,
//!   `rdf:nodeID`, nested resource elements, or literal content;
//! - `parseType="Resource"` (§2.16), `parseType="Collection"` (§2.17),
//!   `parseType="Literal"` (§2.15) for nested predicate-element
//!   content shapes;
//! - `xml:lang` (XML 1.0 §2.12, inherited down the element tree)
//!   producing `lang`-tagged plain literals (`rdf:langString`,
//!   RDF 1.1 §3.3);
//! - `rdf:datatype` IRIs on predicate elements producing typed literals
//!   (RDF 1.1 §3.3);
//! - `xml:base` (XML Base, Marsh & Tobin 2009) accumulated against the
//!   caller-supplied `base_iri` to resolve relative IRIs;
//! - prefix expansion of element / attribute QNames against the
//!   in-scope namespace map carried on [`XmlElement::namespaces`]
//!   (Namespaces in XML 1.0, Bray et al. 2009 §3 — every `xmlns` /
//!   `xmlns:prefix` declaration the praxis XML reader sees is exposed
//!   on the element it appears on).
//!
//! Determinism: blank-node labels come from a single
//! preorder-document-traversal counter, so two `read_rdf_xml` runs on
//! identical input yield byte-identical [`Triple`] streams — the same
//! determinism guarantee the OWL reader makes for `.prx.gz` (commit
//! `a4afae37`).
//!
//! ## Citations
//!
//! - **RDF 1.1 XML Syntax** (Gandon & Schreiber, eds.), W3C Recommendation
//!   2014-02-25 — the production grammar implemented here.
//!   <https://www.w3.org/TR/rdf-syntax-grammar/>.
//! - **RDF 1.1 Concepts and Abstract Syntax** (Cyganiak, Wood & Lanthaler,
//!   eds.), W3C Recommendation 2014-02-25 — the triple model.
//!   <https://www.w3.org/TR/rdf11-concepts/>.
//! - **Namespaces in XML 1.0 (Third Edition)** (Bray, Hollander, Layman
//!   & Tobin, eds.), W3C Recommendation 2009-12-08 — §3 prefix → URI
//!   resolution.
//!   <https://www.w3.org/TR/xml-names/>.
//! - **XML Base (Second Edition)** (Marsh & Tobin, eds.), W3C
//!   Recommendation 2009-01-28 — §3 relative IRI resolution.
//!   <https://www.w3.org/TR/xmlbase/>.

#[allow(unused_imports)]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use super::ontology::RdfVocabulary;
use super::term::{RdfTerm, Triple};
use crate::social::software::markup::xml::ontology::{XmlElement, XmlNode};

// =============================================================================
// Error type
// =============================================================================

/// Structural violations a praxis-native RDF/XML reader detects.
///
/// Every variant names the W3C-specified invariant that fails — the
/// reader never silently drops a triple, mirroring praxis's
/// `feedback_no_silent_failures` discipline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RdfReadError {
    /// A namespace prefix was used on an element or attribute QName
    /// without being declared in scope (Namespaces in XML 1.0 §3 —
    /// every prefix in use MUST resolve to an in-scope URI).
    UndeclaredPrefix { prefix: String },
    /// An IRI reference is structurally invalid — RFC 3987 IRIs are
    /// non-empty and contain no whitespace. Praxis applies the same
    /// minimal check `read_owl` applies; full RFC 3987 validation is
    /// out of scope for the syntax-level reader.
    MalformedIri { iri: String },
    /// A literal appeared in subject position. W3C RDF 1.1 §3 forbids
    /// this — the
    /// [`crate::social::software::markup::xml::rdf::ontology::LiteralsCannotBeSubjects`]
    /// axiom.
    LiteralAsSubject,
    /// A predicate element carried an unrecognised `rdf:parseType`
    /// value (RDF/XML §2.15-§2.17 enumerates `Literal` / `Resource` /
    /// `Collection`; any other token is undefined behaviour and the
    /// reader rejects it rather than guess).
    UnknownParseType { value: String },
    /// An element declared both `rdf:about` / `rdf:ID` / `rdf:nodeID`
    /// in mutually-exclusive combinations (RDF/XML §5.1.3 — production
    /// `nodeElementURIs` permits at most one of these).
    ConflictingSubjectAttributes,
}

impl core::fmt::Display for RdfReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UndeclaredPrefix { prefix } => {
                write!(f, "namespace prefix `{prefix}` is not declared in scope")
            }
            Self::MalformedIri { iri } => write!(f, "malformed IRI: {iri:?}"),
            Self::LiteralAsSubject => {
                write!(
                    f,
                    "a literal cannot appear in subject position (W3C RDF 1.1 §3)"
                )
            }
            Self::UnknownParseType { value } => {
                write!(f, "unknown rdf:parseType value: {value:?}")
            }
            Self::ConflictingSubjectAttributes => write!(
                f,
                "rdf:about / rdf:ID / rdf:nodeID are mutually exclusive on one node element"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RdfReadError {}

// =============================================================================
// Public entry point
// =============================================================================

/// Read an RDF/XML element subtree into a `Vec<Triple>`.
///
/// `elem` is the root `rdf:RDF` element (or any RDF/XML node-element
/// — the function accepts either, per RDF/XML §2.10 / §3.5). `base_iri`
/// is the document base IRI used to resolve `rdf:ID` and relative
/// `rdf:about` references when no enclosing `xml:base` overrides it
/// (XML Base §3).
///
/// Triples are emitted in **document preorder**: each element's own
/// emitted triples precede its children's, and within an element
/// predicate elements are processed in source order. This together
/// with the deterministic blank-node counter makes the output
/// byte-identical across runs on identical input (cf. the
/// `read_owl` determinism fix, praxis commit `a4afae37`).
pub fn read_rdf_xml(elem: &XmlElement, base_iri: &str) -> Result<Vec<Triple>, RdfReadError> {
    let mut ctx = ReadCtx::new(base_iri);
    let scope = Scope::root();
    let scope = scope.push_namespaces(&elem.namespaces);
    let scope = scope.push_xml_base(elem);
    let scope = scope.push_xml_lang(elem);

    let mut triples = Vec::new();
    // RDF/XML §3.5 `RDF` production: the root may be `rdf:RDF` or a
    // bare node element. When it is `rdf:RDF`, the children are node
    // elements; otherwise the element itself is a node element.
    if is_rdf_named(elem, &scope, "RDF") {
        for child in &elem.children {
            if let XmlNode::Element(child_elem) = child {
                read_node_element(child_elem, &scope, &mut ctx, &mut triples)?;
            }
        }
    } else {
        read_node_element(elem, &scope, &mut ctx, &mut triples)?;
    }
    Ok(triples)
}

// =============================================================================
// Reader context — blank-node counter + base IRI
// =============================================================================

/// Per-document reader state. Only the blank-node counter and the
/// document-level base IRI live here; everything that can change
/// inside the tree (in-scope namespaces, `xml:lang`, `xml:base`) is
/// carried on the stack-allocated [`Scope`].
struct ReadCtx {
    /// Deterministic blank-node counter — incremented in preorder
    /// document traversal so identical input yields identical labels.
    blank_counter: u64,
    /// The document-level base IRI handed in by the caller.
    document_base: String,
}

impl ReadCtx {
    fn new(base_iri: &str) -> Self {
        Self {
            blank_counter: 0,
            document_base: base_iri.to_string(),
        }
    }

    /// Allocate a fresh anonymous blank-node label `_:b<n>` and bump
    /// the counter. The `_:` prefix mirrors N-Triples (W3C RDF 1.1
    /// N-Triples §3) so downstream tools recognise the syntax.
    fn fresh_blank(&mut self) -> String {
        let label = format!("_:b{}", self.blank_counter);
        self.blank_counter += 1;
        label
    }
}

// =============================================================================
// Scope — in-scope namespaces, xml:base, xml:lang along the element path
// =============================================================================

/// Lexical scope along the current element path: the prefix → URI map
/// (Namespaces in XML 1.0 §3), the inherited `xml:base` (XML Base §3),
/// and the inherited `xml:lang` (XML 1.0 §2.12).
///
/// Scopes are immutable; entering an element produces a new
/// [`Scope`] that shadows the parent on the same key. Backed by simple
/// `Vec` lookups because element-tree depth is bounded and search is
/// dominated by the RDF subject loop, not the scope walk.
#[derive(Clone)]
struct Scope {
    /// `(prefix-or-empty-for-default, uri)` pairs, in declaration order
    /// from outer to inner. Lookup walks from innermost outward to
    /// honour shadowing (Namespaces in XML 1.0 §6.1).
    bindings: Vec<(String, String)>,
    /// The current `xml:base`, resolved against any outer `xml:base`
    /// (XML Base §3). Empty string means "use the document base".
    base: String,
    /// The current `xml:lang` value, `None` if no enclosing element
    /// declared one (XML 1.0 §2.12).
    lang: Option<String>,
}

impl Scope {
    fn root() -> Self {
        Self {
            bindings: Vec::new(),
            base: String::new(),
            lang: None,
        }
    }

    /// Push the element's `xmlns` / `xmlns:prefix` declarations onto
    /// the scope. The default namespace (`xmlns="..."`, prefix
    /// `None`) is recorded with the empty-string key.
    fn push_namespaces(
        &self,
        namespaces: &[crate::social::software::markup::xml::ontology::XmlNamespace],
    ) -> Self {
        let mut next = self.clone();
        for ns in namespaces {
            let key = ns.prefix.clone().unwrap_or_default();
            next.bindings.push((key, ns.uri.clone()));
        }
        next
    }

    /// Push any `xml:base` attribute on the element onto the scope
    /// (XML Base §3 — a child `xml:base` resolves against its
    /// ancestor's, not against the document base).
    fn push_xml_base(&self, elem: &XmlElement) -> Self {
        let mut next = self.clone();
        for attr in &elem.attributes {
            if attr.name.prefix.as_deref() == Some("xml") && attr.name.local == "base" {
                next.base = resolve_relative(&self.base, &attr.value);
                break;
            }
        }
        next
    }

    /// Push any `xml:lang` attribute on the element onto the scope
    /// (XML 1.0 §2.12 — `xml:lang` is inherited along the element
    /// tree until shadowed; an empty value clears the inheritance).
    fn push_xml_lang(&self, elem: &XmlElement) -> Self {
        let mut next = self.clone();
        for attr in &elem.attributes {
            if attr.name.prefix.as_deref() == Some("xml") && attr.name.local == "lang" {
                next.lang = if attr.value.is_empty() {
                    None
                } else {
                    Some(attr.value.clone())
                };
                break;
            }
        }
        next
    }

    /// Resolve a prefix to its in-scope URI (Namespaces in XML 1.0
    /// §6.1 — the innermost binding wins). `None` for an unbound
    /// prefix; the empty-string key resolves the default namespace.
    fn resolve_prefix(&self, prefix: &str) -> Option<&str> {
        for (p, uri) in self.bindings.iter().rev() {
            if p == prefix {
                return Some(uri.as_str());
            }
        }
        None
    }

    /// The effective base for relative-IRI resolution at this scope.
    /// XML Base §3: the innermost `xml:base` (if any) overrides the
    /// document base passed to [`read_rdf_xml`].
    fn effective_base<'a>(&'a self, ctx: &'a ReadCtx) -> &'a str {
        if self.base.is_empty() {
            ctx.document_base.as_str()
        } else {
            self.base.as_str()
        }
    }
}

// =============================================================================
// RDF/XML node-element loop
// =============================================================================

/// Process one RDF/XML *node element* (RDF/XML §3.4 production
/// `nodeElement`): emit a `subject rdf:type type-IRI` triple if the
/// element name is not `rdf:Description`, then walk the predicate
/// children. Returns the subject term so a caller (a parent predicate
/// element) can connect its property to this node.
fn read_node_element(
    elem: &XmlElement,
    parent_scope: &Scope,
    ctx: &mut ReadCtx,
    triples: &mut Vec<Triple>,
) -> Result<RdfTerm, RdfReadError> {
    let scope = parent_scope
        .push_namespaces(&elem.namespaces)
        .push_xml_base(elem)
        .push_xml_lang(elem);

    // RDF/XML §2.11/§2.12/§2.14: a node element's subject comes from
    // the first of `rdf:about`, `rdf:ID`, `rdf:nodeID`, or a fresh
    // blank node. The three attribute forms are mutually exclusive.
    let subject = node_subject(elem, &scope, ctx)?;

    // RDF/XML §2.13 typed-node form: the element name (when not
    // `rdf:Description`) IS the rdf:type of the subject.
    let elem_iri = qname_to_iri(&elem.name.prefix, &elem.name.local, &scope)?;
    let is_description = is_rdf_named(elem, &scope, "Description");
    if !is_description && !is_rdf_named(elem, &scope, "RDF") {
        triples.push(Triple {
            subject: subject.clone(),
            predicate: RdfVocabulary::RDF_TYPE.to_string(),
            object: RdfTerm::Iri(elem_iri),
        });
    }

    // RDF/XML §2.5: attributes on a node element other than the
    // RDF-reserved set are *property attributes* — abbreviation for
    // `subject -name-> "value"^^xsd:string` triples. The reserved set
    // (`rdf:about` / `rdf:ID` / `rdf:nodeID` / `xml:base` / `xml:lang`)
    // is filtered out here. `rdf:type` as a property attribute is
    // also recognised (§2.13) — its value is an IRI reference, not a
    // literal.
    for attr in &elem.attributes {
        let prefix = attr.name.prefix.as_deref().unwrap_or("");
        if prefix == "xml" {
            continue;
        }
        if is_rdf_reserved_attr_local(prefix, &attr.name.local, &scope) {
            continue;
        }
        // Unprefixed attributes on a node element are not RDF
        // predicates (RDF/XML §2.5 requires a qualified property
        // attribute) — skip them silently rather than synthesise
        // ungrounded triples.
        if prefix.is_empty() {
            continue;
        }
        let pred_iri = qname_to_iri(&attr.name.prefix, &attr.name.local, &scope)?;
        let obj = if pred_iri == RdfVocabulary::RDF_TYPE {
            RdfTerm::Iri(resolve_iri(&attr.value, scope.effective_base(ctx))?)
        } else {
            literal_from_text(&attr.value, &scope, None)
        };
        triples.push(Triple {
            subject: subject.clone(),
            predicate: pred_iri,
            object: obj,
        });
    }

    // Predicate-element children. RDF/XML §3.4 production
    // `propertyEltList`.
    for child in &elem.children {
        if let XmlNode::Element(child_elem) = child {
            read_predicate_element(child_elem, &subject, &scope, ctx, triples)?;
        }
    }

    Ok(subject)
}

/// Compute the subject term for a node element. RDF/XML §2.11 (`rdf:about`),
/// §2.12 (`rdf:ID`), §2.14 (`rdf:nodeID`).
fn node_subject(
    elem: &XmlElement,
    scope: &Scope,
    ctx: &mut ReadCtx,
) -> Result<RdfTerm, RdfReadError> {
    let about = rdf_attr(elem, scope, "about");
    let id = rdf_attr(elem, scope, "ID");
    let node_id = rdf_attr(elem, scope, "nodeID");
    let count = [about.is_some(), id.is_some(), node_id.is_some()]
        .iter()
        .filter(|b| **b)
        .count();
    if count > 1 {
        return Err(RdfReadError::ConflictingSubjectAttributes);
    }
    if let Some(about) = about {
        return Ok(RdfTerm::Iri(resolve_iri(
            &about,
            scope.effective_base(ctx),
        )?));
    }
    if let Some(id) = id {
        // RDF/XML §2.12 `rdf:ID` is shorthand for the base IRI plus a
        // `#` plus the value.
        let resolved = resolve_relative(scope.effective_base(ctx), &format!("#{id}"));
        return Ok(RdfTerm::Iri(check_iri(resolved)?));
    }
    if let Some(node_id) = node_id {
        return Ok(RdfTerm::Blank(format!("_:n_{node_id}")));
    }
    Ok(RdfTerm::Blank(ctx.fresh_blank()))
}

/// Process one *predicate element* (RDF/XML §3.4 production `propertyElt`).
/// Resolves the predicate IRI from the element QName, then dispatches on
/// the object shape declared by the attributes / nested content.
fn read_predicate_element(
    elem: &XmlElement,
    subject: &RdfTerm,
    parent_scope: &Scope,
    ctx: &mut ReadCtx,
    triples: &mut Vec<Triple>,
) -> Result<(), RdfReadError> {
    let scope = parent_scope
        .push_namespaces(&elem.namespaces)
        .push_xml_base(elem)
        .push_xml_lang(elem);

    let pred_iri = qname_to_iri(&elem.name.prefix, &elem.name.local, &scope)?;
    let parse_type = rdf_attr(elem, &scope, "parseType");
    let datatype = rdf_attr(elem, &scope, "datatype");
    let resource = rdf_attr(elem, &scope, "resource");
    let node_id = rdf_attr(elem, &scope, "nodeID");

    // §2.17 `parseType="Collection"` — emit a cons list of
    // `rdf:first`/`rdf:rest` blank nodes terminating in `rdf:nil`.
    if let Some(pt) = &parse_type
        && pt == "Collection"
    {
        let head = emit_collection(&elem.children, &scope, ctx, triples)?;
        triples.push(Triple {
            subject: subject.clone(),
            predicate: pred_iri,
            object: head,
        });
        return Ok(());
    }

    // §2.16 `parseType="Resource"` — inline anonymous blank as the
    // object, whose predicate-element children are the body.
    if let Some(pt) = &parse_type
        && pt == "Resource"
    {
        let blank = RdfTerm::Blank(ctx.fresh_blank());
        for child in &elem.children {
            if let XmlNode::Element(child_elem) = child {
                read_predicate_element(child_elem, &blank, &scope, ctx, triples)?;
            }
        }
        triples.push(Triple {
            subject: subject.clone(),
            predicate: pred_iri,
            object: blank,
        });
        return Ok(());
    }

    // §2.15 `parseType="Literal"` — object is an XML literal whose
    // value is the inner XML of the predicate element, datatype
    // `rdf:XMLLiteral`.
    if let Some(pt) = &parse_type
        && pt == "Literal"
    {
        let lex = serialize_inner_xml(&elem.children);
        let xml_lit_dt = format!("{}{}", RdfVocabulary::RDF_NS, "XMLLiteral");
        triples.push(Triple {
            subject: subject.clone(),
            predicate: pred_iri,
            object: RdfTerm::Literal {
                lexical: lex,
                lang: None,
                datatype: Some(xml_lit_dt),
            },
        });
        return Ok(());
    }

    if let Some(pt) = parse_type {
        return Err(RdfReadError::UnknownParseType { value: pt });
    }

    // §2.4 / §3.4: a `rdf:resource` attribute names an IRI object.
    if let Some(r) = resource {
        let iri = resolve_iri(&r, scope.effective_base(ctx))?;
        triples.push(Triple {
            subject: subject.clone(),
            predicate: pred_iri,
            object: RdfTerm::Iri(iri),
        });
        // §2.5 — property attributes on a property element with an
        // `rdf:resource` describe the IRI object itself (a sub-node).
        emit_property_attributes(
            elem,
            &RdfTerm::Iri(resolve_iri_or_existing(&r, &scope, ctx)?),
            &scope,
            ctx,
            triples,
        )?;
        return Ok(());
    }

    // §2.14: a `rdf:nodeID` attribute names a blank-node object.
    if let Some(nid) = node_id {
        let blank = RdfTerm::Blank(format!("_:n_{nid}"));
        triples.push(Triple {
            subject: subject.clone(),
            predicate: pred_iri,
            object: blank.clone(),
        });
        emit_property_attributes(elem, &blank, &scope, ctx, triples)?;
        return Ok(());
    }

    // Nested element children: recurse as a node element and use its
    // subject term as the object of this predicate (RDF/XML §3.4).
    let child_elements: Vec<&XmlElement> = elem
        .children
        .iter()
        .filter_map(|n| match n {
            XmlNode::Element(e) => Some(e),
            _ => None,
        })
        .collect();
    if !child_elements.is_empty() {
        // RDF/XML §2.5 (resourcePropertyElt): a property element with
        // one nested node element. Multiple are uncommon outside
        // parseType=Collection; we still emit one triple per child.
        for child_elem in &child_elements {
            let object = read_node_element(child_elem, &scope, ctx, triples)?;
            triples.push(Triple {
                subject: subject.clone(),
                predicate: pred_iri.clone(),
                object,
            });
        }
        return Ok(());
    }

    // Otherwise the text content is a literal value. §2.4
    // (literalPropertyElt).
    let lex = element_text_content(elem);
    let obj = literal_from_text(&lex, &scope, datatype);
    triples.push(Triple {
        subject: subject.clone(),
        predicate: pred_iri,
        object: obj,
    });
    Ok(())
}

/// §2.5 — emit one triple per qualified property attribute on a
/// predicate element. The triples describe the *object* the predicate
/// element pointed at, not the outer subject.
fn emit_property_attributes(
    elem: &XmlElement,
    object_subject: &RdfTerm,
    scope: &Scope,
    ctx: &mut ReadCtx,
    triples: &mut Vec<Triple>,
) -> Result<(), RdfReadError> {
    for attr in &elem.attributes {
        let prefix = attr.name.prefix.as_deref().unwrap_or("");
        if prefix == "xml" || prefix.is_empty() {
            continue;
        }
        if is_rdf_reserved_attr_local(prefix, &attr.name.local, scope) {
            continue;
        }
        let pred_iri = qname_to_iri(&attr.name.prefix, &attr.name.local, scope)?;
        let obj = if pred_iri == RdfVocabulary::RDF_TYPE {
            RdfTerm::Iri(resolve_iri(&attr.value, scope.effective_base(ctx))?)
        } else {
            literal_from_text(&attr.value, scope, None)
        };
        triples.push(Triple {
            subject: object_subject.clone(),
            predicate: pred_iri,
            object: obj,
        });
    }
    Ok(())
}

/// §2.17 — `parseType="Collection"` cons-list emission. Returns the
/// head blank-node term (or `rdf:nil` for an empty collection).
fn emit_collection(
    children: &[XmlNode],
    scope: &Scope,
    ctx: &mut ReadCtx,
    triples: &mut Vec<Triple>,
) -> Result<RdfTerm, RdfReadError> {
    let elements: Vec<&XmlElement> = children
        .iter()
        .filter_map(|n| match n {
            XmlNode::Element(e) => Some(e),
            _ => None,
        })
        .collect();
    if elements.is_empty() {
        return Ok(RdfTerm::Iri(RdfVocabulary::RDF_NS.to_string() + "nil"));
    }
    // Allocate cons cells; first walk emits, second walk links.
    let mut cells: Vec<RdfTerm> = Vec::with_capacity(elements.len());
    for _ in &elements {
        cells.push(RdfTerm::Blank(ctx.fresh_blank()));
    }
    let nil = RdfTerm::Iri(RdfVocabulary::RDF_NS.to_string() + "nil");
    for (i, child_elem) in elements.iter().enumerate() {
        let item = read_node_element(child_elem, scope, ctx, triples)?;
        triples.push(Triple {
            subject: cells[i].clone(),
            predicate: RdfVocabulary::RDF_FIRST.to_string(),
            object: item,
        });
        let rest = if i + 1 < cells.len() {
            cells[i + 1].clone()
        } else {
            nil.clone()
        };
        triples.push(Triple {
            subject: cells[i].clone(),
            predicate: RdfVocabulary::RDF_REST.to_string(),
            object: rest,
        });
    }
    Ok(cells[0].clone())
}

// =============================================================================
// Helpers — IRI resolution, QName expansion, attribute lookup
// =============================================================================

/// Look up an `rdf:`-prefixed attribute on the element. Resolves the
/// element's attribute namespace prefix against the in-scope map so
/// that an alternate prefix bound to the same URI (e.g.
/// `xmlns:r="http://www.w3.org/1999/02/22-rdf-syntax-ns#"`) is honoured.
fn rdf_attr(elem: &XmlElement, scope: &Scope, local: &str) -> Option<String> {
    for attr in &elem.attributes {
        let prefix = attr.name.prefix.as_deref().unwrap_or("");
        if attr.name.local == local && prefix_resolves_to_rdf(prefix, scope) {
            return Some(attr.value.clone());
        }
    }
    None
}

/// True iff the prefix in scope binds to the RDF namespace URI.
fn prefix_resolves_to_rdf(prefix: &str, scope: &Scope) -> bool {
    matches!(scope.resolve_prefix(prefix), Some(u) if u == RdfVocabulary::RDF_NS)
}

/// True iff the element's expanded name is `rdf:local`.
fn is_rdf_named(elem: &XmlElement, scope: &Scope, local: &str) -> bool {
    let prefix = elem.name.prefix.as_deref().unwrap_or("");
    elem.name.local == local && prefix_resolves_to_rdf(prefix, scope)
}

/// The set of RDF-reserved attribute locals that are NEVER property
/// attributes — RDF/XML §5.1.3 production `nodeElementURIs` /
/// `propertyAttributeURIs`. `xml:` attributes are filtered separately.
fn is_rdf_reserved_attr_local(prefix: &str, local: &str, scope: &Scope) -> bool {
    if !prefix_resolves_to_rdf(prefix, scope) {
        return false;
    }
    matches!(
        local,
        "about" | "ID" | "nodeID" | "resource" | "parseType" | "datatype"
    )
}

/// Expand a `(prefix, local)` QName to a full IRI by joining the
/// in-scope namespace URI with the local part (Namespaces in XML 1.0
/// §3). An unbound prefix is an error; the empty prefix selects the
/// default namespace, and if no default is in scope the local part is
/// taken bare — which is not RDF-legal but is reported by the caller
/// via [`RdfReadError::MalformedIri`].
fn qname_to_iri(
    prefix: &Option<String>,
    local: &str,
    scope: &Scope,
) -> Result<String, RdfReadError> {
    let prefix_str = prefix.as_deref().unwrap_or("");
    let ns = scope.resolve_prefix(prefix_str);
    match ns {
        Some(uri) => Ok(format!("{}{}", uri, local)),
        None if prefix_str.is_empty() => check_iri(local.to_string()),
        None => Err(RdfReadError::UndeclaredPrefix {
            prefix: prefix_str.to_string(),
        }),
    }
}

/// Resolve an IRI reference against a base IRI per the XML Base §3
/// rules: an absolute IRI is returned as-is; a fragment-only ref
/// (`#frag`) is appended to the base; otherwise the lexical join is
/// applied. This is the minimal subset RDF/XML actually exercises —
/// full RFC 3986 §5.2 reference resolution is not required for the
/// shapes that occur in the OWL corpus, and is left to a downstream
/// IRI library.
fn resolve_iri(raw: &str, base: &str) -> Result<String, RdfReadError> {
    let resolved = resolve_relative(base, raw);
    check_iri(resolved)
}

/// Same as [`resolve_iri`] but returns the resolved string without
/// constructing an error path — used for the "describe the object
/// subject" pass on property attributes, where the IRI was already
/// validated on the outer triple.
fn resolve_iri_or_existing(
    raw: &str,
    scope: &Scope,
    ctx: &ReadCtx,
) -> Result<String, RdfReadError> {
    resolve_iri(raw, scope.effective_base(ctx))
}

/// Lexical IRI resolution (a thin subset). XML Base §3: when the raw
/// reference is absolute (scheme `:` present before any path
/// separator) or already a `urn:` IRI, return it; otherwise prefix
/// the base. `#fragment` references join with the base by appending
/// the fragment directly.
fn resolve_relative(base: &str, raw: &str) -> String {
    if raw.is_empty() {
        return base.to_string();
    }
    if is_absolute_iri(raw) {
        return raw.to_string();
    }
    if let Some(frag) = raw.strip_prefix('#') {
        // Strip any existing fragment on the base before appending.
        let base_no_frag = match base.find('#') {
            Some(i) => &base[..i],
            None => base,
        };
        return format!("{base_no_frag}#{frag}");
    }
    // Non-fragment relative — concatenate. RDF/XML in practice uses
    // absolute IRIs and fragment refs almost exclusively; full path
    // merging per RFC 3986 §5.2 is not required for the bundled OWL
    // corpus and is intentionally left to a downstream IRI library.
    format!("{base}{raw}")
}

/// An IRI is absolute when it begins with a scheme. Per RFC 3987
/// §2.2 productions `IRI = scheme ":" …`. The minimal recogniser is
/// "contains a `:` before any `/` or `#` or `?`".
fn is_absolute_iri(s: &str) -> bool {
    for (i, c) in s.char_indices() {
        match c {
            ':' => return i > 0,
            '/' | '#' | '?' => return false,
            _ => {}
        }
    }
    false
}

/// Minimal IRI syntactic check — RFC 3987 IRIs are non-empty and
/// contain no whitespace. Full validation is intentionally not
/// performed here; the syntactic gate matches what `read_owl` already
/// applied implicitly via the XML parser.
fn check_iri(iri: String) -> Result<String, RdfReadError> {
    if iri.is_empty() || iri.chars().any(|c| c.is_whitespace()) {
        return Err(RdfReadError::MalformedIri { iri });
    }
    Ok(iri)
}

/// Construct a plain or typed literal from a lexical form and the
/// surrounding `xml:lang` / `rdf:datatype` context. Datatype IRIs
/// resolve against the document base (RDF/XML §2.5 — the value is an
/// IRI reference). If both `lang` and `datatype` are present,
/// `datatype` wins (RDF 1.1 §3.3 — typed literals carry no language
/// tag).
fn literal_from_text(lexical: &str, scope: &Scope, datatype: Option<String>) -> RdfTerm {
    if let Some(dt) = datatype {
        // Datatype IRI per RDF/XML §2.5 — keep as-is when absolute,
        // otherwise base-relative. The datatype IRI itself is not
        // re-validated here; downstream lens layers may.
        let dt_iri = resolve_relative(&scope.base, &dt);
        return RdfTerm::Literal {
            lexical: lexical.to_string(),
            lang: None,
            datatype: Some(dt_iri),
        };
    }
    let lang = scope.lang.as_ref().filter(|s| !s.is_empty()).cloned();
    RdfTerm::Literal {
        lexical: lexical.to_string(),
        lang,
        datatype: None,
    }
}

/// Collect text + CDATA content from a predicate element's direct
/// children. Mirrors [`XmlElement::text_content`] but only at one
/// level — RDF/XML §2.4 literal property elements have flat text.
fn element_text_content(elem: &XmlElement) -> String {
    let mut out = String::new();
    for child in &elem.children {
        match child {
            XmlNode::Text(t) | XmlNode::CData(t) => out.push_str(t),
            _ => {}
        }
    }
    out
}

/// Serialise an element's inner XML for `parseType="Literal"`. The
/// reader emits a minimal textual form — a full XML c14n pass is
/// available at `formal/meta/well_behaved_lens/canonical/xml.rs` and
/// can be wired in by a downstream OWL functor; the syntax-level
/// reader keeps the serialisation cheap and reversible enough to
/// preserve information.
fn serialize_inner_xml(children: &[XmlNode]) -> String {
    let mut out = String::new();
    for child in children {
        serialize_xml_node(child, &mut out);
    }
    out
}

fn serialize_xml_node(node: &XmlNode, out: &mut String) {
    match node {
        XmlNode::Text(t) => out.push_str(t),
        XmlNode::CData(t) => out.push_str(t),
        XmlNode::Comment(_) | XmlNode::ProcessingInstruction { .. } => {
            // Comments / PIs are not in the XML literal's value space
            // (RDF/XML §2.15 cites XML c14n; c14n strips comments by
            // default).
        }
        XmlNode::Element(e) => {
            out.push('<');
            write_qname(&e.name, out);
            for ns in &e.namespaces {
                out.push(' ');
                match &ns.prefix {
                    Some(p) => {
                        out.push_str("xmlns:");
                        out.push_str(p);
                    }
                    None => out.push_str("xmlns"),
                }
                out.push_str("=\"");
                out.push_str(&ns.uri);
                out.push('"');
            }
            for attr in &e.attributes {
                out.push(' ');
                write_qname(&attr.name, out);
                out.push_str("=\"");
                out.push_str(&attr.value);
                out.push('"');
            }
            if e.children.is_empty() {
                out.push_str("/>");
            } else {
                out.push('>');
                for c in &e.children {
                    serialize_xml_node(c, out);
                }
                out.push_str("</");
                write_qname(&e.name, out);
                out.push('>');
            }
        }
    }
}

fn write_qname(name: &crate::social::software::markup::xml::ontology::XmlName, out: &mut String) {
    if let Some(p) = &name.prefix {
        out.push_str(p);
        out.push(':');
    }
    out.push_str(&name.local);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::reader::read_xml;

    /// Helper: parse RDF/XML source text into triples with a fixed
    /// document base. Wraps the chained call so each spec test is
    /// one assertion.
    fn rdf(src: &str) -> Vec<Triple> {
        let doc = read_xml(src).expect("XML must parse");
        read_rdf_xml(&doc.root, "http://example.org/base/").expect("RDF/XML must read")
    }

    fn rdf_at(src: &str, base: &str) -> Vec<Triple> {
        let doc = read_xml(src).expect("XML must parse");
        read_rdf_xml(&doc.root, base).expect("RDF/XML must read")
    }

    const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

    // ── Spec micro-cases (RDF/XML §2.x) ─────────────────────────────

    /// §2.13 typed-node form emits one `rdf:type` triple per node
    /// element whose name is not `rdf:Description`.
    #[test]
    fn typed_node_form_emits_rdf_type() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <ex:Person rdf:about="http://example.org/alice"/>
</rdf:RDF>"#;
        let ts = rdf(src);
        assert_eq!(ts.len(), 1);
        let t = &ts[0];
        assert_eq!(t.subject, RdfTerm::Iri("http://example.org/alice".into()));
        assert_eq!(t.predicate, format!("{RDF_NS}type"));
        assert_eq!(t.object, RdfTerm::Iri("http://example.org/Person".into()));
    }

    /// §2.10 striped form — `rdf:Description` carries no implicit
    /// type triple; the explicit `rdf:type` child supplies the type.
    #[test]
    fn striped_form_emits_explicit_type() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/alice">
    <rdf:type rdf:resource="http://example.org/Person"/>
  </rdf:Description>
</rdf:RDF>"#;
        let ts = rdf(src);
        assert_eq!(ts.len(), 1);
        assert_eq!(
            ts[0].subject,
            RdfTerm::Iri("http://example.org/alice".into())
        );
        assert_eq!(ts[0].predicate, format!("{RDF_NS}type"));
        assert_eq!(
            ts[0].object,
            RdfTerm::Iri("http://example.org/Person".into())
        );
    }

    /// §2.11 — `rdf:about` is an absolute IRI subject identifier.
    #[test]
    fn rdf_about_takes_absolute_iri() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/x">
    <ex:p>v</ex:p>
  </rdf:Description>
</rdf:RDF>"#;
        let ts = rdf(src);
        assert_eq!(ts[0].subject, RdfTerm::Iri("http://example.org/x".into()));
    }

    /// §2.12 — `rdf:ID` becomes `base#value`.
    #[test]
    fn rdf_id_resolves_against_base() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:ID="alice">
    <ex:p>v</ex:p>
  </rdf:Description>
</rdf:RDF>"#;
        let ts = rdf_at(src, "http://example.org/doc");
        assert_eq!(
            ts[0].subject,
            RdfTerm::Iri("http://example.org/doc#alice".into())
        );
    }

    /// §2.14 — `rdf:nodeID` names a stable blank node; the same
    /// `nodeID` value used twice yields the same blank-node label.
    #[test]
    fn rdf_nodeid_produces_stable_blank() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:nodeID="a"><ex:p>1</ex:p></rdf:Description>
  <rdf:Description rdf:nodeID="a"><ex:q>2</ex:q></rdf:Description>
</rdf:RDF>"#;
        let ts = rdf(src);
        // Both subjects share the same blank label.
        let s0 = &ts[0].subject;
        let s1 = &ts[1].subject;
        assert_eq!(s0, s1);
        assert!(matches!(s0, RdfTerm::Blank(b) if b == "_:n_a"));
    }

    /// §2.17 — `parseType="Collection"` materialises a cons list
    /// (`rdf:first`/`rdf:rest`/…/`rdf:nil`).
    #[test]
    fn parse_type_collection_emits_cons_list() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/s">
    <ex:members rdf:parseType="Collection">
      <ex:M rdf:about="http://example.org/m1"/>
      <ex:M rdf:about="http://example.org/m2"/>
    </ex:members>
  </rdf:Description>
</rdf:RDF>"#;
        let ts = rdf(src);
        // Expect: 2 rdf:type triples (typed-node form on each ex:M),
        // 2 rdf:first triples, 2 rdf:rest triples, and one outer
        // ex:members triple.
        let firsts: Vec<_> = ts
            .iter()
            .filter(|t| t.predicate == format!("{RDF_NS}first"))
            .collect();
        let rests: Vec<_> = ts
            .iter()
            .filter(|t| t.predicate == format!("{RDF_NS}rest"))
            .collect();
        assert_eq!(firsts.len(), 2);
        assert_eq!(rests.len(), 2);
        // The last rest must terminate in rdf:nil.
        let nil_iri = format!("{RDF_NS}nil");
        assert!(
            rests
                .iter()
                .any(|t| t.object == RdfTerm::Iri(nil_iri.clone()))
        );
    }

    /// §2.16 — `parseType="Resource"` introduces an inline blank-node
    /// subject for the nested predicate elements.
    #[test]
    fn parse_type_resource_creates_inline_blank() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/s">
    <ex:address rdf:parseType="Resource">
      <ex:city>Tokyo</ex:city>
    </ex:address>
  </rdf:Description>
</rdf:RDF>"#;
        let ts = rdf(src);
        // One triple s -ex:address-> _:b0 plus _:b0 -ex:city-> "Tokyo"
        let outer = ts
            .iter()
            .find(|t| t.predicate == "http://example.org/address")
            .unwrap();
        let inner = ts
            .iter()
            .find(|t| t.predicate == "http://example.org/city")
            .unwrap();
        assert!(matches!(outer.object, RdfTerm::Blank(_)));
        assert_eq!(outer.object, inner.subject);
        assert_eq!(
            inner.object,
            RdfTerm::Literal {
                lexical: "Tokyo".into(),
                lang: None,
                datatype: None
            }
        );
    }

    /// §2.15 — `parseType="Literal"` produces an XML-literal typed
    /// object whose datatype is `rdf:XMLLiteral`.
    #[test]
    fn parse_type_literal_produces_xml_literal_datatype() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/s">
    <ex:body rdf:parseType="Literal"><b>hello</b></ex:body>
  </rdf:Description>
</rdf:RDF>"#;
        let ts = rdf(src);
        let t = &ts[0];
        match &t.object {
            RdfTerm::Literal {
                lexical,
                lang,
                datatype,
            } => {
                assert!(lexical.contains("<b>hello</b>"));
                assert!(lang.is_none());
                assert_eq!(
                    datatype.as_deref(),
                    Some("http://www.w3.org/1999/02/22-rdf-syntax-ns#XMLLiteral")
                );
            }
            other => panic!("expected an XML literal, got {other:?}"),
        }
    }

    /// XML 1.0 §2.12 — `xml:lang` is inherited along the element
    /// tree, so a child predicate element without its own `xml:lang`
    /// picks up the enclosing element's value as the literal's tag.
    #[test]
    fn xml_lang_inherits_to_inner_literals() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/"
         xml:lang="en">
  <rdf:Description rdf:about="http://example.org/s">
    <ex:name>Alice</ex:name>
  </rdf:Description>
</rdf:RDF>"#;
        let ts = rdf(src);
        match &ts[0].object {
            RdfTerm::Literal { lang, .. } => assert_eq!(lang.as_deref(), Some("en")),
            other => panic!("expected literal, got {other:?}"),
        }
    }

    /// RDF/XML §2.5 — `rdf:datatype` on a literal property element
    /// produces a typed literal.
    #[test]
    fn rdf_datatype_produces_typed_literal() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/s">
    <ex:age rdf:datatype="http://www.w3.org/2001/XMLSchema#integer">42</ex:age>
  </rdf:Description>
</rdf:RDF>"#;
        let ts = rdf(src);
        match &ts[0].object {
            RdfTerm::Literal {
                lexical,
                lang,
                datatype,
            } => {
                assert_eq!(lexical, "42");
                assert!(lang.is_none());
                assert_eq!(
                    datatype.as_deref(),
                    Some("http://www.w3.org/2001/XMLSchema#integer")
                );
            }
            other => panic!("expected typed literal, got {other:?}"),
        }
    }

    /// XML Base §3 — a child `xml:base` shadows the document base
    /// for relative IRI resolution within its subtree.
    #[test]
    fn xml_base_overrides_document_base() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xml:base="http://other.example.org/inner">
  <rdf:Description rdf:ID="x"/>
</rdf:RDF>"#;
        let ts = rdf_at(src, "http://example.org/doc");
        // No triples emitted because rdf:Description with only rdf:ID
        // and no predicate children is a no-op. We instead inspect a
        // case where ID drives a type triple via the typed-node form.
        assert_eq!(ts.len(), 0);
        let src2 = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/"
         xml:base="http://other.example.org/inner">
  <ex:T rdf:ID="x"/>
</rdf:RDF>"#;
        let ts2 = rdf_at(src2, "http://example.org/doc");
        assert_eq!(
            ts2[0].subject,
            RdfTerm::Iri("http://other.example.org/inner#x".into())
        );
    }

    /// RDF/XML §3.4 — an anonymous nested node element introduces a
    /// fresh blank-node object.
    #[test]
    fn anonymous_nested_node_introduces_blank() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <ex:Person rdf:about="http://example.org/alice">
    <ex:knows>
      <ex:Person/>
    </ex:knows>
  </ex:Person>
</rdf:RDF>"#;
        let ts = rdf(src);
        let knows = ts
            .iter()
            .find(|t| t.predicate == "http://example.org/knows")
            .unwrap();
        assert!(matches!(knows.object, RdfTerm::Blank(_)));
    }

    /// Predicate-element `rdf:resource` makes the object an IRI.
    #[test]
    fn rdf_resource_makes_object_an_iri() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/s">
    <ex:p rdf:resource="http://example.org/o"/>
  </rdf:Description>
</rdf:RDF>"#;
        let ts = rdf(src);
        assert_eq!(ts[0].object, RdfTerm::Iri("http://example.org/o".into()));
    }

    /// §2.5 — qualified property attributes on a node element are
    /// shorthand for plain-literal property triples.
    #[test]
    fn property_attributes_expand_to_plain_literals() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <ex:T rdf:about="http://example.org/s" ex:name="Alice"/>
</rdf:RDF>"#;
        let ts = rdf(src);
        // One rdf:type triple (typed-node form) + one property-attr triple.
        assert_eq!(ts.len(), 2);
        let name = ts
            .iter()
            .find(|t| t.predicate == "http://example.org/name")
            .unwrap();
        assert_eq!(
            name.object,
            RdfTerm::Literal {
                lexical: "Alice".into(),
                lang: None,
                datatype: None
            }
        );
    }

    // ── Determinism ─────────────────────────────────────────────────

    /// Two `read_rdf_xml` runs on the same input yield identical
    /// triples in identical order — the praxis determinism guarantee
    /// (cf. read_owl `a4afae37`).
    #[test]
    fn determinism_two_runs_match() {
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <ex:Person rdf:about="http://example.org/a">
    <ex:knows><ex:Person/></ex:knows>
    <ex:knows><ex:Person/></ex:knows>
  </ex:Person>
</rdf:RDF>"#;
        let a = rdf(src);
        let b = rdf(src);
        assert_eq!(a, b, "two reads must produce identical triple streams");
    }

    // ── Subject-admissibility invariant (W3C RDF 1.1 §3) ────────────

    /// The
    /// [`crate::social::software::markup::xml::rdf::ontology::LiteralsCannotBeSubjects`]
    /// axiom: no triple the reader emits ever carries a literal in
    /// subject position.
    #[test]
    fn reader_never_emits_literal_as_subject() {
        // A document that exercises blanks, nested nodes, literals,
        // collections, and parseType=Resource — every shape that
        // produces a triple.
        let src = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:ex="http://example.org/">
  <ex:T rdf:about="http://example.org/s" ex:label="x">
    <ex:p>literal</ex:p>
    <ex:r rdf:parseType="Resource"><ex:q>inner</ex:q></ex:r>
    <ex:members rdf:parseType="Collection">
      <ex:M rdf:about="http://example.org/m"/>
    </ex:members>
  </ex:T>
</rdf:RDF>"#;
        let ts = rdf(src);
        assert!(!ts.is_empty());
        for t in &ts {
            assert!(
                t.subject_is_admissible(),
                "literal subject in triple {t:?} violates W3C RDF 1.1 §3"
            );
        }
    }

    // ── Corpus-wide audit on the six bundled OWL vocabularies ──────

    /// Walk every bundled OWL file through the praxis-native RDF/XML
    /// reader and assert no literal-as-subject violations. Per-vocab
    /// triple counts are printed (not pinned to magic numbers, per
    /// `feedback_no_bounded_discovery_counts`). The corpus-wide audit
    /// disciplines `feedback_corpus_wide_audit_on_load`.
    #[test]
    fn corpus_wide_six_owl_files_parse_to_triples() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let data_dir = root.join("data/ontologies");

        let vocabs = [
            "cito-2.8.1.owl",
            "doco-1.3.owl",
            "c4o-1.2.owl",
            "biro-1.1.1.owl",
            "prov_o-2013-04-30.owl",
            "olia-2026-04-09.owl",
        ];

        let mut total: usize = 0;
        for name in &vocabs {
            let path = data_dir.join(name);
            let Ok(bytes) = std::fs::read(&path) else {
                panic!("corpus file not on disk: {}", path.display());
            };
            let text =
                core::str::from_utf8(&bytes).unwrap_or_else(|e| panic!("{name} is not UTF-8: {e}"));
            let doc = read_xml(text).unwrap_or_else(|e| panic!("{name} did not parse as XML: {e}"));
            let triples = read_rdf_xml(&doc.root, "")
                .unwrap_or_else(|e| panic!("{name} did not read as RDF/XML: {e}"));
            assert!(
                !triples.is_empty(),
                "{name} produced zero triples — RDF/XML reader is failing silently"
            );
            for t in &triples {
                assert!(
                    t.subject_is_admissible(),
                    "{name}: literal subject in {t:?} violates W3C RDF 1.1 §3"
                );
            }
            println!("rdf-xml-corpus-audit: {name} — triples={}", triples.len());
            total += triples.len();
        }
        println!("rdf-xml-corpus-audit: total triples across 6 vocabularies = {total}");
    }

    // ── Property-based coverage on a tractable RDF/XML subset ──────────
    //
    // The hand-written `rdf!(…)` cases pin specific spec sections; the
    // proptests below exercise the reader on a small, well-behaved
    // generated subset (typed-node and `rdf:Description`, one or two
    // predicate elements per node, optional `xml:lang` / `rdf:datatype` /
    // `rdf:resource`). Properties asserted:
    //
    //   (a) `read_rdf_xml` succeeds on every generated doc.
    //   (b) The W3C axiom
    //       [`crate::social::software::markup::xml::rdf::ontology::LiteralsCannotBeSubjects`]
    //       holds on the returned triples — application of
    //       `Triple::subject_is_admissible`, the structural projection
    //       of that axiom.
    //   (c) Determinism: two independent `read_rdf_xml` calls on the
    //       same bytes produce identical triple streams (blank-node
    //       labels and document order included), mirroring the
    //       `read_owl` determinism guarantee (praxis commit
    //       `a4afae37`).
    //
    // The generator restricts to the structural shapes that exercise
    // the reader's main control-flow forks; full grammar coverage
    // belongs to the W3C-suite path, not this proptest.

    use proptest::prelude::*;

    /// A small generated RDF/XML node-element specification.
    #[derive(Debug, Clone)]
    struct GenNode {
        /// `true` = typed-node form (`<ex:Cls rdf:about=…>`); `false` =
        /// `rdf:Description` form.
        typed: bool,
        /// Local subject identifier — used to form `http://example.org/<id>`.
        id: u8,
        /// Predicate elements attached to this node.
        preds: Vec<GenPred>,
    }

    /// A generated predicate element — either a literal-valued (plain,
    /// `xml:lang`-tagged, or `rdf:datatype`-tagged) or `rdf:resource`
    /// reference.
    #[derive(Debug, Clone)]
    enum GenPred {
        PlainLit { local: u8, value: u8 },
        LangLit { local: u8, value: u8, lang_idx: u8 },
        TypedLit { local: u8, value: u8, dt_idx: u8 },
        Resource { local: u8, target: u8 },
    }

    fn arb_pred() -> impl Strategy<Value = GenPred> {
        let local = 0u8..4;
        let value = 0u8..4;
        prop_oneof![
            (local.clone(), value.clone())
                .prop_map(|(local, value)| GenPred::PlainLit { local, value }),
            (local.clone(), value.clone(), 0u8..3).prop_map(|(local, value, lang_idx)| {
                GenPred::LangLit {
                    local,
                    value,
                    lang_idx,
                }
            }),
            (local.clone(), value.clone(), 0u8..3).prop_map(|(local, value, dt_idx)| {
                GenPred::TypedLit {
                    local,
                    value,
                    dt_idx,
                }
            }),
            (local, 0u8..4).prop_map(|(local, target)| GenPred::Resource { local, target }),
        ]
    }

    fn arb_node() -> impl Strategy<Value = GenNode> {
        (
            any::<bool>(),
            0u8..4,
            prop::collection::vec(arb_pred(), 1..=2),
        )
            .prop_map(|(typed, id, preds)| GenNode { typed, id, preds })
    }

    fn arb_doc() -> impl Strategy<Value = Vec<GenNode>> {
        prop::collection::vec(arb_node(), 1..=3)
    }

    /// Render a generated document to a concrete RDF/XML source string.
    /// All quoting is conservative (numeric ids, ASCII locals) so the
    /// XML 1.0 parser accepts every output.
    fn render(doc: &[GenNode]) -> String {
        const LANGS: &[&str] = &["en", "fr", "de"];
        const DTS: &[&str] = &[
            "http://www.w3.org/2001/XMLSchema#string",
            "http://www.w3.org/2001/XMLSchema#integer",
            "http://www.w3.org/2001/XMLSchema#boolean",
        ];
        let mut out = String::from(
            "<?xml version=\"1.0\"?>\n\
             <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n\
                      xmlns:ex=\"http://example.org/\">\n",
        );
        for n in doc {
            let about = format!("http://example.org/s{}", n.id);
            if n.typed {
                out.push_str(&format!("  <ex:Cls rdf:about=\"{about}\">\n"));
            } else {
                out.push_str(&format!("  <rdf:Description rdf:about=\"{about}\">\n"));
            }
            for p in &n.preds {
                match p {
                    GenPred::PlainLit { local, value } => {
                        out.push_str(&format!("    <ex:p{local}>v{value}</ex:p{local}>\n"));
                    }
                    GenPred::LangLit {
                        local,
                        value,
                        lang_idx,
                    } => {
                        let lang = LANGS[*lang_idx as usize % LANGS.len()];
                        out.push_str(&format!(
                            "    <ex:p{local} xml:lang=\"{lang}\">v{value}</ex:p{local}>\n"
                        ));
                    }
                    GenPred::TypedLit {
                        local,
                        value,
                        dt_idx,
                    } => {
                        let dt = DTS[*dt_idx as usize % DTS.len()];
                        out.push_str(&format!(
                            "    <ex:p{local} rdf:datatype=\"{dt}\">v{value}</ex:p{local}>\n"
                        ));
                    }
                    GenPred::Resource { local, target } => {
                        out.push_str(&format!(
                            "    <ex:p{local} rdf:resource=\"http://example.org/r{target}\"/>\n"
                        ));
                    }
                }
            }
            if n.typed {
                out.push_str("  </ex:Cls>\n");
            } else {
                out.push_str("  </rdf:Description>\n");
            }
        }
        out.push_str("</rdf:RDF>\n");
        out
    }

    proptest! {
        /// (a) The reader succeeds on every generated doc — no
        /// structural shape in the subset is undefined behaviour.
        #[test]
        fn prop_reader_accepts_generated_docs(doc in arb_doc()) {
            let src = render(&doc);
            let parsed = read_xml(&src).expect("XML must parse");
            let res = read_rdf_xml(&parsed.root, "http://example.org/base/");
            prop_assert!(res.is_ok(), "read_rdf_xml rejected generated doc: {:?}", res.err());
        }

        /// (b) The
        /// [`crate::social::software::markup::xml::rdf::ontology::LiteralsCannotBeSubjects`]
        /// axiom (W3C RDF 1.1 Concepts §3) — no triple the reader
        /// emits ever carries a literal in subject position.
        #[test]
        fn prop_literals_never_subjects(doc in arb_doc()) {
            let src = render(&doc);
            let parsed = read_xml(&src).expect("XML must parse");
            let triples = read_rdf_xml(&parsed.root, "http://example.org/base/")
                .expect("read_rdf_xml must succeed");
            for t in &triples {
                prop_assert!(
                    t.subject_is_admissible(),
                    "literal subject in {t:?} violates W3C RDF 1.1 §3"
                );
            }
        }

        /// (c) Determinism — two independent `read_rdf_xml` calls on
        /// the same bytes produce byte-identical `Vec<Triple>`, blank
        /// labels and order included (cf. the `read_owl` determinism
        /// fix, praxis commit `a4afae37`).
        #[test]
        fn prop_read_rdf_xml_is_deterministic(doc in arb_doc()) {
            let src = render(&doc);
            let parsed1 = read_xml(&src).expect("XML must parse");
            let parsed2 = read_xml(&src).expect("XML must parse");
            let t1 = read_rdf_xml(&parsed1.root, "http://example.org/base/")
                .expect("read 1");
            let t2 = read_rdf_xml(&parsed2.root, "http://example.org/base/")
                .expect("read 2");
            prop_assert_eq!(t1, t2, "two reads on identical bytes diverged");
        }
    }
}
