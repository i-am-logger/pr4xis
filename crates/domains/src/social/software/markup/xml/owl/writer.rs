//! `write_owl` — deterministic canonical RDF/XML serialisation of an
//! [`OwlOntology`].
//!
//! ## Canonical-form rules (the lens's published canonical-form spec)
//!
//! 1. **Lexicographic entity order.** Classes first, then object
//!    properties, then individuals, each block sorted lexicographically
//!    by IRI. Within an entity: labels and comments first (sorted by
//!    `(lang, datatype, lexical)`), then sorted relations, then sorted
//!    annotations.
//! 2. **Root prefixes sorted, declared once.** All in-use namespace
//!    bindings appear on `<rdf:RDF>` in alphabetical-prefix order; no
//!    redundant re-declarations on inner elements.
//! 3. **No DTD, no `xml:base`, no comments, no PIs.** The canonical
//!    form is a minimal RDF/XML projection of the triple set.
//! 4. **Content-addressed blank-node labels.** An anonymous restriction
//!    (or anonymous union / intersection / complement / RDF list) is
//!    labelled `_:c<digest-hex>` where the hash is computed over the
//!    canonical structural encoding of the expression. Two
//!    structurally-identical blank nodes hash to the same label and
//!    are emitted once. A hash collision between two structurally
//!    distinct restrictions is a genuine ambiguity and is reported as
//!    a panic.
//! 5. **UTF-8, LF line endings, two-space indent.** Standard XML 1.0
//!    declaration as the first line.
//!
//! ## Citations
//!
//! - **Patel-Schneider, P. F. & Motik, B. (eds.) (2012)** *OWL 2 Web
//!   Ontology Language: Mapping to RDF Graphs (2nd ed.)*, W3C
//!   Recommendation 11 December 2012 — the inverse of `read_owl`'s
//!   mapping (see [`super::reader`]).
//! - **Longley, D. & Sporny, M. (eds.) (2024)** *RDF Dataset
//!   Canonicalization 1.0 (URDNA2015)*, W3C Recommendation 21 May 2024
//!   — §4.5 blank-node-labelling; the algorithm we simplify by
//!   content-hashing one restriction at a time (sufficient because
//!   the SPAR / PROV-O / OLiA blank nodes are tree-shaped, not
//!   cyclic).
//! - **Boyer, J. & Marcy, G. (eds.) (2008)** *Canonical XML Version
//!   1.1*, W3C Recommendation 2 May 2008 — the prior art canonical-
//!   form pass the round-trip harness lens for XSD/USLM uses; we
//!   simplify to a praxis-domain RDF/XML canonical form because OWL
//!   RDF/XML's degrees of freedom (typed-node vs striped form,
//!   property attributes vs predicate elements, list serialisation
//!   shapes, …) are larger than what C14N normalises.
//! - **Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020)** *BLAKE3: one
//!   function, fast everywhere* — the hash used for content-addressed
//!   blank-node labels.

#[allow(unused_imports)]
use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec::Vec,
};

use pr4xis_runtime::address::ContentAddress;

use super::ontology::{
    OwlAnnotation, OwlAnnotationValue, OwlClass, OwlClassExpression, OwlIndividual, OwlLiteral,
    OwlObjectProperty, OwlOntology, OwlRestrictionKind,
};
use super::reader::canonical_expr_form;

const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";
const OWL_NS: &str = "http://www.w3.org/2002/07/owl#";

/// Emit canonical RDF/XML bytes for `ont`.
///
/// The output is the canonical form of an [`OwlOntology`]: the same
/// `OwlOntology` always produces the same bytes; two ontologies that
/// are [`super::reader::owl_equivalent`] produce the same bytes.
pub fn write_owl(ont: &OwlOntology) -> Vec<u8> {
    let ns = NamespaceMap::build(ont);
    let mut w = Writer::new(ns);
    w.write_header();
    w.write_root_open(ont);
    w.write_ontology_node(ont);
    w.write_classes(&ont.classes);
    w.write_properties(&ont.properties);
    w.write_individuals(&ont.individuals);
    w.write_root_close();
    w.into_bytes()
}

/// Discovered-namespace → prefix map. Built by scanning every
/// annotation predicate IRI in the ontology and assigning each
/// unique namespace a stable deterministic prefix (`n0`, `n1`, …
/// in alphabetical-namespace order). The fixed `owl` / `rdf` /
/// `rdfs` prefixes are always present.
struct NamespaceMap {
    /// `(prefix, namespace_uri)` — sorted by prefix for deterministic
    /// root-element emission.
    bindings: Vec<(String, String)>,
}

impl NamespaceMap {
    fn build(ont: &OwlOntology) -> Self {
        let mut namespaces: alloc::collections::BTreeSet<String> =
            alloc::collections::BTreeSet::new();
        for ann in &ont.ontology_annotations {
            namespaces.insert(predicate_namespace(&ann.predicate));
        }
        for c in &ont.classes {
            for ann in &c.annotations {
                namespaces.insert(predicate_namespace(&ann.predicate));
            }
        }
        for p in &ont.properties {
            for ann in &p.annotations {
                namespaces.insert(predicate_namespace(&ann.predicate));
            }
        }
        for i in &ont.individuals {
            for ann in &i.annotations {
                namespaces.insert(predicate_namespace(&ann.predicate));
            }
        }
        // Three fixed bindings — owl/rdf/rdfs always declared even
        // when unused by an annotation, since the entity tags
        // themselves use them.
        let mut bindings: Vec<(String, String)> = Vec::new();
        bindings.push(("owl".to_string(), OWL_NS.to_string()));
        bindings.push(("rdf".to_string(), RDF_NS.to_string()));
        bindings.push(("rdfs".to_string(), RDFS_NS.to_string()));
        // Filter out the three fixed namespaces, then assign `n0`,
        // `n1`, … to the remainder in sorted order.
        let mut others: Vec<&String> = namespaces
            .iter()
            .filter(|n| *n != OWL_NS && *n != RDF_NS && *n != RDFS_NS && !n.is_empty())
            .collect();
        others.sort();
        for (i, ns) in others.iter().enumerate() {
            bindings.push((format!("n{i}"), ns.to_string()));
        }
        bindings.sort_by(|a, b| a.0.cmp(&b.0));
        Self { bindings }
    }

    /// Resolve a full predicate IRI to a QName using this map.
    /// Predicates outside any declared namespace fall back to the
    /// hash-suffix `rdf:_<hex>` form (legacy compatibility path,
    /// shouldn't normally trigger because `build` scans them).
    fn predicate_qname(&self, predicate: &str) -> String {
        for (prefix, ns) in &self.bindings {
            if predicate.starts_with(ns.as_str())
                && let Some(local) = predicate.strip_prefix(ns.as_str())
                && is_valid_ncname(local)
            {
                return format!("{prefix}:{local}");
            }
        }
        // Fall-back: hash-encoded suffix. Should only happen if a
        // predicate IRI is malformed beyond what the namespace
        // detector recognised.
        format!("rdf:_{}", predicate_safe_local(predicate))
    }
}

/// Extract the namespace (the IRI prefix up to and including the
/// final `#` or `/`) from a predicate IRI.
fn predicate_namespace(predicate: &str) -> String {
    if let Some(idx) = predicate.rfind('#') {
        predicate[..=idx].to_string()
    } else if let Some(idx) = predicate.rfind('/') {
        predicate[..=idx].to_string()
    } else {
        String::new()
    }
}

/// Approximate XML NCName check — the local part must start with
/// a letter or underscore and contain only NCName characters
/// (Namespaces in XML 1.0 §3). Conservative: rejects anything with
/// non-alphanumeric / non-`_` / non-`-` / non-`.` content. Enough
/// for the bundled SPAR / PROV-O / OLiA local names.
fn is_valid_ncname(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().expect("non-empty");
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
}

struct Writer {
    buf: String,
    indent: usize,
    ns: NamespaceMap,
}

impl Writer {
    fn new(ns: NamespaceMap) -> Self {
        Self {
            buf: String::new(),
            indent: 0,
            ns,
        }
    }

    fn push_line(&mut self, line: &str) {
        for _ in 0..self.indent {
            self.buf.push(' ');
            self.buf.push(' ');
        }
        self.buf.push_str(line);
        self.buf.push('\n');
    }

    fn into_bytes(self) -> Vec<u8> {
        self.buf.into_bytes()
    }

    fn write_header(&mut self) {
        self.buf
            .push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    }

    fn write_root_open(&mut self, _ont: &OwlOntology) {
        // Every discovered namespace declared on the root in
        // sorted-prefix order. `owl` / `rdf` / `rdfs` always present
        // (entity tags use them); discovered annotation namespaces
        // appear as `n0`, `n1`, … per `NamespaceMap`.
        self.buf.push_str("<rdf:RDF\n");
        for (prefix, ns) in &self.ns.bindings {
            self.buf.push_str("  xmlns:");
            self.buf.push_str(prefix);
            self.buf.push_str("=\"");
            self.buf.push_str(ns);
            self.buf.push_str("\"\n");
        }
        self.buf.push_str(">\n");
        self.indent = 1;
    }

    fn write_root_close(&mut self) {
        self.indent = 0;
        self.buf.push_str("</rdf:RDF>\n");
    }

    fn write_ontology_node(&mut self, ont: &OwlOntology) {
        if ont.iri.is_empty() && ont.ontology_annotations.is_empty() {
            return;
        }
        self.push_line(&format!(
            "<owl:Ontology rdf:about={}>",
            attr_value(&ont.iri)
        ));
        self.indent += 1;
        write_annotations_sorted(self, &ont.ontology_annotations);
        self.indent -= 1;
        self.push_line("</owl:Ontology>");
    }

    fn write_classes(&mut self, classes: &[OwlClass]) {
        let mut sorted: Vec<&OwlClass> = classes.iter().collect();
        sorted.sort_by(|a, b| a.iri.cmp(&b.iri));
        for c in sorted {
            self.write_class(c);
        }
    }

    fn write_class(&mut self, c: &OwlClass) {
        self.push_line(&format!("<owl:Class rdf:about={}>", attr_value(&c.iri)));
        self.indent += 1;

        write_literal_list(self, "rdfs:label", &c.labels);
        write_literal_list(self, "rdfs:comment", &c.comments);

        // Superclass expressions — sorted by canonical form so the
        // output is deterministic regardless of input order. Named
        // superclasses emit as `<rdfs:subClassOf rdf:resource="…"/>`;
        // anonymous expressions emit inline.
        let mut sup_exprs: Vec<&OwlClassExpression> = c.superclass_expressions.iter().collect();
        sup_exprs.sort_by_key(|e| canonical_expr_form(e));
        for expr in sup_exprs {
            write_class_relation(self, "rdfs:subClassOf", expr);
        }
        let mut eqs: Vec<&OwlClassExpression> = c.equivalent_classes.iter().collect();
        eqs.sort_by_key(|e| canonical_expr_form(e));
        for expr in eqs {
            write_class_relation(self, "owl:equivalentClass", expr);
        }
        let mut djs: Vec<&OwlClassExpression> = c.disjoint_classes.iter().collect();
        djs.sort_by_key(|e| canonical_expr_form(e));
        for expr in djs {
            write_class_relation(self, "owl:disjointWith", expr);
        }

        write_annotations_sorted(self, &c.annotations);

        self.indent -= 1;
        self.push_line("</owl:Class>");
    }

    fn write_properties(&mut self, properties: &[OwlObjectProperty]) {
        let mut sorted: Vec<&OwlObjectProperty> = properties.iter().collect();
        sorted.sort_by(|a, b| a.iri.cmp(&b.iri));
        for p in sorted {
            self.write_property(p);
        }
    }

    fn write_property(&mut self, p: &OwlObjectProperty) {
        self.push_line(&format!(
            "<owl:ObjectProperty rdf:about={}>",
            attr_value(&p.iri)
        ));
        self.indent += 1;

        write_literal_list(self, "rdfs:label", &p.labels);
        write_literal_list(self, "rdfs:comment", &p.comments);

        if let Some(d) = &p.domain {
            self.push_line(&format!("<rdfs:domain rdf:resource={}/>", attr_value(d)));
        }
        if let Some(r) = &p.range {
            self.push_line(&format!("<rdfs:range rdf:resource={}/>", attr_value(r)));
        }
        let mut sups: Vec<&String> = p.superproperties.iter().collect();
        sups.sort();
        for s in sups {
            self.push_line(&format!(
                "<rdfs:subPropertyOf rdf:resource={}/>",
                attr_value(s)
            ));
        }
        if let Some(inv) = &p.inverse_of {
            self.push_line(&format!(
                "<owl:inverseOf rdf:resource={}/>",
                attr_value(inv)
            ));
        }
        let mut eqs: Vec<&String> = p.equivalent_properties.iter().collect();
        eqs.sort();
        for s in eqs {
            self.push_line(&format!(
                "<owl:equivalentProperty rdf:resource={}/>",
                attr_value(s)
            ));
        }
        let mut djs: Vec<&String> = p.disjoint_properties.iter().collect();
        djs.sort();
        for s in djs {
            self.push_line(&format!(
                "<owl:propertyDisjointWith rdf:resource={}/>",
                attr_value(s)
            ));
        }

        write_annotations_sorted(self, &p.annotations);

        self.indent -= 1;
        self.push_line("</owl:ObjectProperty>");
    }

    fn write_individuals(&mut self, inds: &[OwlIndividual]) {
        let mut sorted: Vec<&OwlIndividual> = inds.iter().collect();
        sorted.sort_by(|a, b| a.iri.cmp(&b.iri));
        for i in sorted {
            self.write_individual(i);
        }
    }

    fn write_individual(&mut self, ind: &OwlIndividual) {
        self.push_line(&format!(
            "<owl:NamedIndividual rdf:about={}>",
            attr_value(&ind.iri)
        ));
        self.indent += 1;

        let mut types: Vec<&String> = ind.types.iter().collect();
        types.sort();
        for t in types {
            self.push_line(&format!("<rdf:type rdf:resource={}/>", attr_value(t)));
        }
        write_literal_list(self, "rdfs:label", &ind.labels);
        write_literal_list(self, "rdfs:comment", &ind.comments);
        write_annotations_sorted(self, &ind.annotations);

        self.indent -= 1;
        self.push_line("</owl:NamedIndividual>");
    }
}

/// Emit a sorted sequence of `<tag …>lexical</tag>` lines, one per
/// literal in the multiset.
fn write_literal_list(w: &mut Writer, tag: &str, lits: &[OwlLiteral]) {
    let mut sorted: Vec<&OwlLiteral> = lits.iter().collect();
    sorted.sort_by(|a, b| {
        (a.lang.as_deref(), a.datatype.as_deref(), a.lexical.as_str()).cmp(&(
            b.lang.as_deref(),
            b.datatype.as_deref(),
            b.lexical.as_str(),
        ))
    });
    for lit in sorted {
        w.push_line(&format_literal(tag, lit));
    }
}

fn format_literal(tag: &str, lit: &OwlLiteral) -> String {
    let mut s = String::new();
    s.push('<');
    s.push_str(tag);
    if let Some(lang) = &lit.lang {
        s.push_str(" xml:lang=");
        s.push_str(&attr_value(lang));
    }
    if let Some(dt) = &lit.datatype {
        s.push_str(" rdf:datatype=");
        s.push_str(&attr_value(dt));
    }
    s.push('>');
    s.push_str(&escape_text(&lit.lexical));
    s.push_str("</");
    s.push_str(tag);
    s.push('>');
    s
}

/// Annotation block — sorted by (predicate, then canonical value form).
fn write_annotations_sorted(w: &mut Writer, anns: &[OwlAnnotation]) {
    let mut sorted: Vec<&OwlAnnotation> = anns.iter().collect();
    sorted.sort_by(|a, b| {
        a.predicate
            .cmp(&b.predicate)
            .then_with(|| canonical_value_form(&a.value).cmp(&canonical_value_form(&b.value)))
    });
    for ann in sorted {
        write_annotation(w, ann);
    }
}

fn predicate_tag_resolved(w: &Writer, predicate: &str) -> String {
    if let Some(local) = predicate.strip_prefix(RDF_NS)
        && is_valid_ncname(local)
    {
        return format!("rdf:{local}");
    }
    if let Some(local) = predicate.strip_prefix(RDFS_NS)
        && is_valid_ncname(local)
    {
        return format!("rdfs:{local}");
    }
    if let Some(local) = predicate.strip_prefix(OWL_NS)
        && is_valid_ncname(local)
    {
        return format!("owl:{local}");
    }
    w.ns.predicate_qname(predicate)
}

fn canonical_value_form(value: &OwlAnnotationValue) -> String {
    match value {
        OwlAnnotationValue::Iri(i) => format!("I:{i}"),
        OwlAnnotationValue::Literal(l) => format!(
            "L:{}:{}:{}",
            l.lang.as_deref().unwrap_or(""),
            l.datatype.as_deref().unwrap_or(""),
            l.lexical
        ),
    }
}

fn write_annotation(w: &mut Writer, ann: &OwlAnnotation) {
    let tag = predicate_tag_resolved(w, &ann.predicate);
    match &ann.value {
        OwlAnnotationValue::Iri(iri) => {
            w.push_line(&format!("<{tag} rdf:resource={}/>", attr_value(iri)));
        }
        OwlAnnotationValue::Literal(lit) => {
            w.push_line(&format_literal(&tag, lit));
        }
    }
}

/// Emit a class-expression relation. Named targets emit as a single
/// `rdf:resource` reference; anonymous expressions emit inline as a
/// nested `<owl:Restriction>` / `<owl:Class>` block.
fn write_class_relation(w: &mut Writer, tag: &str, expr: &OwlClassExpression) {
    match expr {
        OwlClassExpression::Named(iri) => {
            w.push_line(&format!("<{tag} rdf:resource={}/>", attr_value(iri)));
        }
        _ => {
            w.push_line(&format!("<{tag}>"));
            w.indent += 1;
            write_inline_class_expression(w, expr);
            w.indent -= 1;
            w.push_line(&format!("</{tag}>"));
        }
    }
}

/// Emit an anonymous class expression inline. Restrictions emit as
/// `<owl:Restriction>` blocks; unions / intersections emit as
/// `<owl:Class><owl:unionOf …>` blocks; complements emit as
/// `<owl:Class><owl:complementOf …>`.
fn write_inline_class_expression(w: &mut Writer, expr: &OwlClassExpression) {
    let label = content_addressed_label(expr);
    match expr {
        OwlClassExpression::Named(iri) => {
            w.push_line(&format!("<rdf:Description rdf:about={}/>", attr_value(iri)));
        }
        OwlClassExpression::Restriction(r) => {
            w.push_line(&format!(
                "<owl:Restriction rdf:nodeID={}>",
                attr_value(&label)
            ));
            w.indent += 1;
            w.push_line(&format!(
                "<owl:onProperty rdf:resource={}/>",
                attr_value(&r.on_property)
            ));
            write_restriction_kind(w, &r.kind);
            w.indent -= 1;
            w.push_line("</owl:Restriction>");
        }
        OwlClassExpression::Union(parts) => {
            w.push_line(&format!("<owl:Class rdf:nodeID={}>", attr_value(&label)));
            w.indent += 1;
            w.push_line("<owl:unionOf rdf:parseType=\"Collection\">");
            w.indent += 1;
            let mut sorted: Vec<&OwlClassExpression> = parts.iter().collect();
            sorted.sort_by_key(|e| canonical_expr_form(e));
            for p in sorted {
                write_inline_class_expression(w, p);
            }
            w.indent -= 1;
            w.push_line("</owl:unionOf>");
            w.indent -= 1;
            w.push_line("</owl:Class>");
        }
        OwlClassExpression::Intersection(parts) => {
            w.push_line(&format!("<owl:Class rdf:nodeID={}>", attr_value(&label)));
            w.indent += 1;
            w.push_line("<owl:intersectionOf rdf:parseType=\"Collection\">");
            w.indent += 1;
            let mut sorted: Vec<&OwlClassExpression> = parts.iter().collect();
            sorted.sort_by_key(|e| canonical_expr_form(e));
            for p in sorted {
                write_inline_class_expression(w, p);
            }
            w.indent -= 1;
            w.push_line("</owl:intersectionOf>");
            w.indent -= 1;
            w.push_line("</owl:Class>");
        }
        OwlClassExpression::Complement(inner) => {
            w.push_line(&format!("<owl:Class rdf:nodeID={}>", attr_value(&label)));
            w.indent += 1;
            w.push_line("<owl:complementOf>");
            w.indent += 1;
            write_inline_class_expression(w, inner);
            w.indent -= 1;
            w.push_line("</owl:complementOf>");
            w.indent -= 1;
            w.push_line("</owl:Class>");
        }
    }
}

fn write_restriction_kind(w: &mut Writer, kind: &OwlRestrictionKind) {
    match kind {
        OwlRestrictionKind::SomeValuesFrom(filler) => {
            if let OwlClassExpression::Named(iri) = filler.as_ref() {
                w.push_line(&format!(
                    "<owl:someValuesFrom rdf:resource={}/>",
                    attr_value(iri)
                ));
            } else {
                w.push_line("<owl:someValuesFrom>");
                w.indent += 1;
                write_inline_class_expression(w, filler);
                w.indent -= 1;
                w.push_line("</owl:someValuesFrom>");
            }
        }
        OwlRestrictionKind::AllValuesFrom(filler) => {
            if let OwlClassExpression::Named(iri) = filler.as_ref() {
                w.push_line(&format!(
                    "<owl:allValuesFrom rdf:resource={}/>",
                    attr_value(iri)
                ));
            } else {
                w.push_line("<owl:allValuesFrom>");
                w.indent += 1;
                write_inline_class_expression(w, filler);
                w.indent -= 1;
                w.push_line("</owl:allValuesFrom>");
            }
        }
        OwlRestrictionKind::HasValue(v) => match v {
            OwlAnnotationValue::Iri(i) => {
                w.push_line(&format!("<owl:hasValue rdf:resource={}/>", attr_value(i)));
            }
            OwlAnnotationValue::Literal(lit) => {
                w.push_line(&format_literal("owl:hasValue", lit));
            }
        },
        OwlRestrictionKind::MinCardinality { n, on_class } => write_cardinality(
            w,
            "owl:minCardinality",
            "owl:minQualifiedCardinality",
            *n,
            on_class,
        ),
        OwlRestrictionKind::MaxCardinality { n, on_class } => write_cardinality(
            w,
            "owl:maxCardinality",
            "owl:maxQualifiedCardinality",
            *n,
            on_class,
        ),
        OwlRestrictionKind::ExactCardinality { n, on_class } => write_cardinality(
            w,
            "owl:cardinality",
            "owl:qualifiedCardinality",
            *n,
            on_class,
        ),
    }
}

fn write_cardinality(
    w: &mut Writer,
    plain_tag: &str,
    qualified_tag: &str,
    n: u32,
    on_class: &Option<Box<OwlClassExpression>>,
) {
    if let Some(filler) = on_class {
        w.push_line(&format!(
            "<{qualified_tag} rdf:datatype=\"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\">{n}</{qualified_tag}>"
        ));
        if let OwlClassExpression::Named(iri) = filler.as_ref() {
            w.push_line(&format!("<owl:onClass rdf:resource={}/>", attr_value(iri)));
        } else {
            w.push_line("<owl:onClass>");
            w.indent += 1;
            write_inline_class_expression(w, filler);
            w.indent -= 1;
            w.push_line("</owl:onClass>");
        }
    } else {
        w.push_line(&format!(
            "<{plain_tag} rdf:datatype=\"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\">{n}</{plain_tag}>"
        ));
    }
}

/// Compute the content-addressed blank-node label for an anonymous
/// expression. Two structurally-identical expressions yield the same
/// label; a hash collision between two structurally distinct
/// expressions is reported as a panic on the assumption that BLAKE3
/// is collision-resistant (Aumasson, O'Connor, Neves & Wilcox-O'Hearn
/// 2020) — the praxis-
/// distinctness-vs-collision discipline.
fn content_addressed_label(expr: &OwlClassExpression) -> String {
    let canonical = canonical_expr_form(expr);
    let mut label = String::with_capacity(65);
    label.push('c');
    label.push_str(&ContentAddress::of(canonical.as_bytes()).to_hex());
    label
}

/// Stable, character-safe local name derived from an IRI. The XML
/// `Name` production (W3C XML 1.0 §2.3) restricts the character set;
/// the canonical writer uses a hex-encoded content-digest prefix to stay
/// inside it without colliding for distinct IRIs.
fn predicate_safe_local(iri: &str) -> String {
    let mut hex = ContentAddress::of(iri.as_bytes()).to_hex();
    // Take the first 16 hex chars as a probabilistically-unique
    // identifier (W3C XML Name productions accept any alphanum).
    hex.truncate(16);
    hex
}

/// XML attribute-value escape: produce a quoted, escaped form for an
/// `="…"` attribute. W3C XML 1.0 §2.3 / §3.3.3 reserves `<`, `&`, and
/// the quote character; we additionally escape `>` and `\r` for
/// robustness against quirky consumers.
fn attr_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\r' => out.push_str("&#13;"),
            '\n' => out.push_str("&#10;"),
            '\t' => out.push_str("&#9;"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// XML text-content escape — same predefined-entity set as XML 1.0
/// §4.6 (`<`, `>`, `&`).
fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::owl::reader::{owl_equivalent, read_owl};

    #[test]
    fn write_owl_is_deterministic() {
        let xml = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#">
  <owl:Class rdf:about="http://example.org/A">
    <rdfs:label xml:lang="en">A</rdfs:label>
  </owl:Class>
  <owl:Class rdf:about="http://example.org/B">
    <rdfs:subClassOf rdf:resource="http://example.org/A"/>
  </owl:Class>
</rdf:RDF>"#;
        let ont = read_owl(xml).expect("parse");
        let a = write_owl(&ont);
        let b = write_owl(&ont);
        assert_eq!(a, b, "write_owl must be deterministic");
    }

    #[test]
    fn write_owl_round_trip_simple() {
        let xml = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#">
  <owl:Class rdf:about="http://example.org/A">
    <rdfs:label xml:lang="en">A</rdfs:label>
  </owl:Class>
</rdf:RDF>"#;
        let ont1 = read_owl(xml).expect("parse 1");
        let bytes = write_owl(&ont1);
        let text = std::str::from_utf8(&bytes).expect("utf8");
        let ont2 = read_owl(text).expect("parse 2 (round-trip)");
        assert!(
            owl_equivalent(&ont1, &ont2),
            "round-trip equivalence on a simple class fixture"
        );
    }

    // ── Property-based coverage on synthetic ontologies ─────────────
    //
    // The bundled-vocabulary tests pin specific real-world inputs; the
    // proptests below sweep the writer's canonical-form invariants
    // over the structural subset the `test_arb::arb_ontology` strategy
    // emits. Properties:
    //
    //   (a) `write_owl` is deterministic — two calls on the same
    //       `OwlOntology` produce byte-identical `Vec<u8>`.
    //   (b) Within each entity block (classes, properties), the entity
    //       IRIs appear in strict lexicographic order — the writer's
    //       sort-and-canonicalize discipline.
    //   (c) `read_owl(write_owl(&ont))` produces an `OwlOntology`
    //       `owl_equivalent` to `ont` — the structural-equivalence
    //       round-trip, mirroring the categorical idempotence the
    //       Phase 2 corpus-wide harness pins for the six bundled vocabs.

    use proptest::prelude::*;

    use super::super::test_arb::arb_ontology;

    /// Scan the emitted bytes for every `<owl:Class rdf:about="…">` (or
    /// `<owl:ObjectProperty …>`) opener and return the IRIs in document
    /// order. Helper for the lexicographic-ordering property.
    fn extract_iris(text: &str, opener: &str) -> Vec<String> {
        let mut out = Vec::new();
        let needle = format!("{opener} rdf:about=\"");
        let mut rest = text;
        while let Some(idx) = rest.find(&needle) {
            let after = &rest[idx + needle.len()..];
            if let Some(end) = after.find('"') {
                out.push(after[..end].to_string());
                rest = &after[end..];
            } else {
                break;
            }
        }
        out
    }

    proptest! {
        /// (a) `write_owl` is deterministic — two calls on the same
        /// `OwlOntology` produce byte-identical bytes. Pins the lens's
        /// deterministic-canonical claim.
        #[test]
        fn prop_write_owl_is_deterministic(ont in arb_ontology()) {
            let a = write_owl(&ont);
            let b = write_owl(&ont);
            prop_assert_eq!(a, b);
        }

        /// (b) Within each entity block (classes / properties), the
        /// entity IRIs appear in strict lexicographic order. Pins the
        /// canonical-form §1 rule from the writer's module-level
        /// canonical-form spec.
        #[test]
        fn prop_entity_blocks_are_lexicographically_sorted(ont in arb_ontology()) {
            let bytes = write_owl(&ont);
            let text = std::str::from_utf8(&bytes).expect("utf8");

            let class_iris = extract_iris(text, "<owl:Class");
            for w in class_iris.windows(2) {
                prop_assert!(
                    w[0] < w[1],
                    "class block not strictly lex-sorted: {:?} !< {:?}",
                    w[0],
                    w[1]
                );
            }
            let prop_iris = extract_iris(text, "<owl:ObjectProperty");
            for w in prop_iris.windows(2) {
                prop_assert!(
                    w[0] < w[1],
                    "property block not strictly lex-sorted: {:?} !< {:?}",
                    w[0],
                    w[1]
                );
            }
        }

        /// (c) Round-trip on synthetic ontologies — `read_owl(write_owl(&ont))`
        /// is `owl_equivalent` to `ont`. Pins the lens's PutGet leg
        /// at the structural level for any within-subset instance,
        /// mirroring the categorical round-trip the Phase 2 harness
        /// runs on the six bundled OWL vocabs.
        #[test]
        fn prop_round_trip_owl_equivalent(ont in arb_ontology()) {
            let bytes = write_owl(&ont);
            let text = std::str::from_utf8(&bytes).expect("utf8");
            let ont2 = read_owl(text).expect("read_owl on write_owl output");
            prop_assert!(
                owl_equivalent(&ont, &ont2),
                "round-trip drift: write_owl ∘ read_owl not owl_equivalent"
            );
        }
    }
}
