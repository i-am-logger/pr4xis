//! Praxis-native **OWL 2 RDF Mapping** reader — the W3C OWL 2 RDF
//! mapping (Patel-Schneider & Motik 2012) applied to the triple stream
//! emitted by [`crate::social::software::markup::xml::rdf::read_rdf_xml`].
//!
//! ## Pipeline
//!
//! ```text
//!   bytes  ──XML 1.0──▶  XmlElement  ──RDF/XML──▶  Vec<Triple>
//!                                                       │
//!                                          ┌────────────┘
//!                                          ▼
//!                                  OWL 2 RDF Mapping
//!                                  (Patel-Schneider &
//!                                   Motik 2012, §3)
//!                                          │
//!                                          ▼
//!                                    OwlOntology
//! ```
//!
//! The XML and RDF/XML legs are entirely outside this module; the OWL
//! reader consumes triples, not elements. All blank-node tracking, RDF
//! list materialisation (`rdf:first`/`rdf:rest`/`rdf:nil`),
//! `parseType="Resource"`/`Collection"`/`Literal"`, `xml:lang`
//! inheritance, and datatyped literals are the RDF/XML reader's
//! responsibility — the OWL layer only does the typed projection.
//!
//! ## Citations
//!
//! - **Patel-Schneider, P. F. & Motik, B. (eds.) (2012)** *OWL 2 Web
//!   Ontology Language: Mapping to RDF Graphs (2nd ed.)*, W3C
//!   Recommendation 11 December 2012 — §3.x mapping rules.
//!   <https://www.w3.org/TR/owl2-mapping-to-rdf/>.
//! - **Motik, B., Patel-Schneider, P. F. & Parsia, B. (eds.) (2012)**
//!   *OWL 2: Structural Specification and Functional-Style Syntax (2nd
//!   ed.)*, W3C Recommendation 11 December 2012 — §8 class
//!   expressions, §9 axioms, §10 annotations.
//!   <https://www.w3.org/TR/owl2-syntax/>.
//! - **Cyganiak, R., Wood, D. & Lanthaler, M. (eds.) (2014)** *RDF 1.1
//!   Concepts and Abstract Syntax*, W3C Recommendation 25 February 2014
//!   — §3.3 literals.
//!   <https://www.w3.org/TR/rdf11-concepts/>.
//! - **Brickley, D. & Guha, R. V. (eds.) (2014)** *RDF Schema 1.1*, W3C
//!   Recommendation 25 February 2014 — §2.1 subClassOf, §2.4 label,
//!   §2.5 comment, §5.1.7 subPropertyOf, §3.1 domain, §3.2 range.
//!   <https://www.w3.org/TR/rdf-schema/>.

#[allow(unused_imports)]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use super::ontology::{
    OwlAnnotation, OwlAnnotationValue, OwlClass, OwlClassExpression, OwlIndividual, OwlLiteral,
    OwlObjectProperty, OwlOntology, OwlRestriction, OwlRestrictionKind, OwlVocabulary,
};
use crate::social::software::markup::xml::rdf::{
    Quad, RdfReadError, RdfTerm, RdfVocabulary, Triple, read_rdf_xml,
};
use crate::social::software::markup::xml::reader as xml_reader;
use hashbrown::HashMap;

// =============================================================================
// Public entry point
// =============================================================================

/// Read an OWL/RDF ontology serialised as RDF/XML (Gandon & Schreiber
/// 2014). Composes the praxis XML reader with the praxis-native
/// RDF/XML triple reader, then applies the W3C OWL 2 RDF Mapping
/// (Patel-Schneider & Motik 2012) to project the triple set onto an
/// [`OwlOntology`].
///
/// All blank-node tracking, RDF list materialisation, and literal
/// shape (lang / datatype) are handled in
/// [`crate::social::software::markup::xml::rdf`]; this function never
/// walks the XML element tree directly.
pub fn read_owl(xml_text: &str) -> Result<OwlOntology, OwlReadError> {
    // 1. XML 1.0 well-formedness.
    let doc = xml_reader::read_xml(xml_text).map_err(|e| OwlReadError::Xml(e.message))?;

    // 2. Document base IRI — `xml:base` on the root takes precedence,
    //    then the default namespace `xmlns="..."` (RDF/XML §5.3 uses
    //    that as the implicit document base in the same way the
    //    historical reader did, so the SPAR vocabularies keep working).
    let base_iri = extract_base_iri(&doc.root);

    // 3. RDF/XML → triples (Gandon & Schreiber 2014).
    let triples = read_rdf_xml(&doc.root, &base_iri).map_err(OwlReadError::Rdf)?;

    // 4. OWL 2 RDF Mapping (Patel-Schneider & Motik 2012).
    Ok(project_triples_to_owl(&triples, &base_iri))
}

/// Read an OWL/RDF ontology serialised as RDF/XML into the **raw RDF
/// graph** — the [`Triple`] stream `read_rdf_xml` (Gandon & Schreiber
/// 2014) emits, lifted to the RDF 1.1 dataset model as [`Quad`]s in the
/// default graph — *below* the [`OwlOntology`] typed projection.
///
/// This is the true source graph the OWL 2 RDF Mapping
/// ([`read_owl`]/`project_triples_to_owl`) projects from. Where
/// [`read_owl`] keeps only the typed view (classes, properties,
/// individuals, the restriction/cardinality shapes it recognises),
/// `read_owl_to_quads` keeps **every** triple the document denotes —
/// the input to W3C RDF Dataset Canonicalization (RDFC-1.0,
/// REC-rdf-canon-20240521), which is the OWL lens's graph-identity
/// canonical form.
///
/// An RDF/XML document serialises a single RDF *graph*, not a dataset
/// (RDF/XML §6 has no syntax for named graphs); every triple therefore
/// belongs to the default graph. We lift each [`Triple`] into a
/// default-graph [`Quad`] (`graph = None`). The invariant is enforced
/// structurally: [`Triple`] (W3C RDF 1.1 §3.1) carries no graph-name
/// component, so a graph name *cannot* be present — `read_owl_to_quads`
/// can only ever produce default-graph quads, which RDFC-1.0 §4.4.3
/// treats as the dataset's default graph.
///
/// ## Citations
///
/// - **Cyganiak, Wood & Lanthaler (2014)** *RDF 1.1 Concepts and
///   Abstract Syntax*, §4 (RDF datasets: a default graph plus zero or
///   more named graphs). The default graph is the one a single RDF/XML
///   document denotes.
/// - **Longley, Kellogg & Yamamoto (2024)** *RDF Dataset
///   Canonicalization* (REC-rdf-canon-20240521), §4.4.3 — the
///   canonicalization algorithm over a dataset's quads.
///
/// [`Quad`]: crate::social::software::markup::xml::rdf::Quad
pub fn read_owl_to_quads(xml_text: &str) -> Result<Vec<Quad>, OwlReadError> {
    // 1. XML 1.0 well-formedness.
    let doc = xml_reader::read_xml(xml_text).map_err(|e| OwlReadError::Xml(e.message))?;

    // 2. Document base IRI (same resolution as `read_owl`).
    let base_iri = extract_base_iri(&doc.root);

    // 3. RDF/XML → triples (Gandon & Schreiber 2014).
    let triples = read_rdf_xml(&doc.root, &base_iri).map_err(OwlReadError::Rdf)?;

    // 4. Lift each triple into a default-graph quad. The `graph = None`
    //    is the only admissible value for a single-document RDF/XML
    //    graph (RDF 1.1 §4): `Triple` carries no graph-name component,
    //    so no triple can claim membership of a named graph. RDFC-1.0
    //    canonicalises this default-graph dataset.
    Ok(triples.into_iter().map(Quad::from_default_graph).collect())
}

/// Extract the document-level base IRI for relative `rdf:about` /
/// `rdf:ID` resolution. `xml:base` (XML Base §3) wins; the default
/// namespace `xmlns="..."` is the historical fallback used by the SPAR
/// vocabularies, with any trailing `#` stripped.
fn extract_base_iri(root: &crate::social::software::markup::xml::ontology::XmlElement) -> String {
    for attr in &root.attributes {
        if attr.name.prefix.as_deref() == Some("xml") && attr.name.local == "base" {
            return attr.value.clone();
        }
    }
    for attr in &root.attributes {
        if attr.name.prefix.is_none() && attr.name.local == "xmlns" {
            return attr.value.trim_end_matches('#').to_string();
        }
    }
    String::new()
}

// =============================================================================
// OWL 2 RDF Mapping — triples → OwlOntology
// =============================================================================

/// Apply the W3C OWL 2 RDF Mapping (Patel-Schneider & Motik 2012,
/// §3.x) to a triple set, producing the typed `OwlOntology` view.
fn project_triples_to_owl(triples: &[Triple], base_iri: &str) -> OwlOntology {
    let idx = TripleIndex::build(triples);
    let mut ont = OwlOntology {
        iri: base_iri.to_string(),
        ..Default::default()
    };

    // -- Ontology header (OWL 2 RDF Mapping §3.1, OWL 2 §3.5).
    // A subject typed `owl:Ontology` carries the header annotations.
    for subj in idx.subjects_by_type(OwlVocabulary::OWL_ONTOLOGY) {
        if let RdfTerm::Iri(iri) = subj {
            // The ontology IRI is the subject's IRI; if multiple
            // `owl:Ontology` nodes appear (extremely rare), the first
            // wins in document order to keep behaviour deterministic.
            if ont.iri.is_empty() || ont.iri == base_iri {
                ont.iri = iri.clone();
            }
            collect_header_annotations(subj, &idx, &mut ont.ontology_annotations);
        }
    }

    // -- Classes (OWL 2 RDF Mapping §3.2.1).
    let mut class_iris: Vec<&str> = Vec::new();
    let mut class_iri_set: hashbrown::HashSet<&str> = hashbrown::HashSet::new();
    for subj in idx.subjects_by_type_in_order(OwlVocabulary::OWL_CLASS) {
        if let RdfTerm::Iri(iri) = subj
            && !class_iri_set.contains(iri.as_str())
        {
            class_iri_set.insert(iri.as_str());
            class_iris.push(iri.as_str());
        }
    }
    for iri in class_iris {
        let mut class = build_class(iri, &idx);
        dedup_class(&mut class);
        ont.classes.push(class);
    }

    // -- Object properties (OWL 2 RDF Mapping §3.2.2).
    // A subject typed `owl:ObjectProperty` (or with any of the
    // property-characteristic types) is an object property.
    let property_type_iris = [
        OwlVocabulary::OWL_OBJECT_PROPERTY,
        OwlVocabulary::OWL_FUNCTIONAL_PROPERTY,
        OwlVocabulary::OWL_INVERSE_FUNCTIONAL,
        OwlVocabulary::OWL_TRANSITIVE_PROPERTY,
        OwlVocabulary::OWL_SYMMETRIC_PROPERTY,
    ];
    let mut prop_iris: Vec<&str> = Vec::new();
    let mut prop_iri_set: hashbrown::HashSet<&str> = hashbrown::HashSet::new();
    for type_iri in &property_type_iris {
        for subj in idx.subjects_by_type_in_order(type_iri) {
            if let RdfTerm::Iri(iri) = subj
                && !prop_iri_set.contains(iri.as_str())
            {
                prop_iri_set.insert(iri.as_str());
                prop_iris.push(iri.as_str());
            }
        }
    }
    for iri in prop_iris {
        let mut prop = build_property(iri, &idx);
        dedup_property(&mut prop);
        ont.properties.push(prop);
    }

    // -- Named individuals (OWL 2 RDF Mapping §3.2.3).
    let mut ind_iris: Vec<&str> = Vec::new();
    let mut ind_iri_set: hashbrown::HashSet<&str> = hashbrown::HashSet::new();
    for subj in idx.subjects_by_type_in_order(OwlVocabulary::OWL_NAMED_INDIVIDUAL) {
        if let RdfTerm::Iri(iri) = subj
            && !ind_iri_set.contains(iri.as_str())
        {
            ind_iri_set.insert(iri.as_str());
            ind_iris.push(iri.as_str());
        }
    }
    for iri in ind_iris {
        let mut ind = build_individual(iri, &idx);
        dedup_individual(&mut ind);
        ont.individuals.push(ind);
    }

    // -- Taxonomy (RDFS §2.1 / §5.1.7 — child→parent edges, classes
    //    only; an `rdfs:subClassOf` of an external `owl:Thing` is kept
    //    in the edge list per the historical reader).
    for class in &ont.classes {
        for sup in &class.superclasses {
            ont.taxonomy.push((class.iri.clone(), sup.clone()));
        }
    }
    for prop in &ont.properties {
        for sup in &prop.superproperties {
            ont.property_taxonomy.push((prop.iri.clone(), sup.clone()));
        }
    }
    ont.taxonomy.sort();
    ont.taxonomy.dedup();
    ont.property_taxonomy.sort();
    ont.property_taxonomy.dedup();
    dedup_annotations(&mut ont.ontology_annotations);

    ont
}

/// Build an [`OwlClass`] from the triples whose subject is `iri`.
/// Applies the relevant rows of the OWL 2 RDF Mapping table (§3.2.1).
fn build_class(iri: &str, idx: &TripleIndex) -> OwlClass {
    let mut class = OwlClass {
        iri: iri.to_string(),
        ..Default::default()
    };

    let subj = RdfTerm::Iri(iri.to_string());
    for (predicate, object) in idx.objects_for(&subj) {
        match predicate.as_str() {
            // RDFS §2.4 / §2.5 — labels and comments are first-class
            // annotations.
            p if p == RdfVocabulary::RDFS_LABEL => {
                if let Some(lit) = literal_of(object) {
                    if class.label.is_none() {
                        class.label = Some(lit.lexical.clone());
                    }
                    class.labels.push(lit);
                }
            }
            p if p == RdfVocabulary::RDFS_COMMENT => {
                if let Some(lit) = literal_of(object) {
                    if class.comment.is_none() {
                        class.comment = Some(lit.lexical.clone());
                    }
                    class.comments.push(lit);
                }
            }
            // RDFS §2.1 — class subsumption.
            p if p == RdfVocabulary::RDFS_SUB_CLASS_OF => {
                let expr = parse_class_expression(object, idx);
                if let OwlClassExpression::Named(named) = &expr
                    && !class.superclasses.iter().any(|s| s == named)
                {
                    class.superclasses.push(named.clone());
                }
                class.superclass_expressions.push(expr);
            }
            // OWL 2 §9.1.1 — equivalent classes.
            p if p == OwlVocabulary::OWL_EQUIVALENT_CLASS => {
                class
                    .equivalent_classes
                    .push(parse_class_expression(object, idx));
            }
            // OWL 2 §9.1.3 — disjoint classes (binary form via
            // `owl:disjointWith`; the n-ary `owl:AllDisjointClasses`
            // form is deferred).
            p if p == OwlVocabulary::OWL_DISJOINT_WITH => {
                class
                    .disjoint_classes
                    .push(parse_class_expression(object, idx));
            }
            // OWL 2 §10 — every other predicate is treated as a
            // free-form annotation on the class entity.
            p if p == RdfVocabulary::RDF_TYPE => {
                // rdf:type is the type triple itself; not an annotation.
            }
            _ => {
                if let Some(value) = annotation_value_of(object) {
                    class.annotations.push(OwlAnnotation {
                        predicate: predicate.clone(),
                        value,
                    });
                }
            }
        }
    }

    class
}

/// Build an [`OwlObjectProperty`] from the triples whose subject is
/// `iri`. Applies the relevant rows of OWL 2 RDF Mapping §3.2.2.
fn build_property(iri: &str, idx: &TripleIndex) -> OwlObjectProperty {
    let mut prop = OwlObjectProperty {
        iri: iri.to_string(),
        ..Default::default()
    };

    let subj = RdfTerm::Iri(iri.to_string());
    for (predicate, object) in idx.objects_for(&subj) {
        match predicate.as_str() {
            p if p == RdfVocabulary::RDFS_LABEL => {
                if let Some(lit) = literal_of(object) {
                    if prop.label.is_none() {
                        prop.label = Some(lit.lexical.clone());
                    }
                    prop.labels.push(lit);
                }
            }
            p if p == RdfVocabulary::RDFS_COMMENT => {
                if let Some(lit) = literal_of(object) {
                    if prop.comment.is_none() {
                        prop.comment = Some(lit.lexical.clone());
                    }
                    prop.comments.push(lit);
                }
            }
            // RDFS §3.1 — domain; §3.2 — range. The simple-projection
            // fields preserve the first seen IRI; further values are
            // dropped at this layer (multi-valued domain/range is
            // expressed as a union expression in OWL DL, and is not
            // yet projected to the simple field shape).
            p if p == RdfVocabulary::RDFS_DOMAIN => {
                if let RdfTerm::Iri(target) = object
                    && prop.domain.is_none()
                {
                    prop.domain = Some(target.clone());
                }
            }
            p if p == RdfVocabulary::RDFS_RANGE => {
                if let RdfTerm::Iri(target) = object
                    && prop.range.is_none()
                {
                    prop.range = Some(target.clone());
                }
            }
            // RDFS §5.1.7 — property subsumption.
            p if p == RdfVocabulary::RDFS_SUB_PROPERTY_OF => {
                if let RdfTerm::Iri(target) = object
                    && !prop.superproperties.iter().any(|s| s == target)
                {
                    prop.superproperties.push(target.clone());
                }
            }
            // OWL 2 §9.2.7 — inverse property.
            p if p == OwlVocabulary::OWL_INVERSE_OF => {
                if let RdfTerm::Iri(target) = object
                    && prop.inverse_of.is_none()
                {
                    prop.inverse_of = Some(target.clone());
                }
            }
            // OWL 2 §9.2.1 — equivalent properties.
            "http://www.w3.org/2002/07/owl#equivalentProperty" => {
                if let RdfTerm::Iri(target) = object
                    && !prop.equivalent_properties.iter().any(|s| s == target)
                {
                    prop.equivalent_properties.push(target.clone());
                }
            }
            // OWL 2 §9.2.2 — disjoint properties.
            "http://www.w3.org/2002/07/owl#propertyDisjointWith" => {
                if let RdfTerm::Iri(target) = object
                    && !prop.disjoint_properties.iter().any(|s| s == target)
                {
                    prop.disjoint_properties.push(target.clone());
                }
            }
            p if p == RdfVocabulary::RDF_TYPE => {
                // rdf:type carries no entity-level annotation content.
            }
            _ => {
                if let Some(value) = annotation_value_of(object) {
                    prop.annotations.push(OwlAnnotation {
                        predicate: predicate.clone(),
                        value,
                    });
                }
            }
        }
    }

    prop
}

/// Build an [`OwlIndividual`] from the triples whose subject is `iri`.
fn build_individual(iri: &str, idx: &TripleIndex) -> OwlIndividual {
    let mut ind = OwlIndividual {
        iri: iri.to_string(),
        ..Default::default()
    };

    let subj = RdfTerm::Iri(iri.to_string());
    for (predicate, object) in idx.objects_for(&subj) {
        match predicate.as_str() {
            p if p == RdfVocabulary::RDF_TYPE => {
                if let RdfTerm::Iri(t) = object
                    && t != OwlVocabulary::OWL_NAMED_INDIVIDUAL
                    && !ind.types.iter().any(|s| s == t)
                {
                    ind.types.push(t.clone());
                }
            }
            p if p == RdfVocabulary::RDFS_LABEL => {
                if let Some(lit) = literal_of(object) {
                    if ind.label.is_none() {
                        ind.label = Some(lit.lexical.clone());
                    }
                    ind.labels.push(lit);
                }
            }
            p if p == RdfVocabulary::RDFS_COMMENT => {
                if let Some(lit) = literal_of(object) {
                    ind.comments.push(lit);
                }
            }
            _ => {
                if let Some(value) = annotation_value_of(object) {
                    ind.annotations.push(OwlAnnotation {
                        predicate: predicate.clone(),
                        value,
                    });
                }
            }
        }
    }

    ind
}

/// De-duplicate label / comment / annotation / class-expression
/// content on a class.
///
/// The OWL 2 RDF Mapping (Patel-Schneider & Motik 2012) is set-
/// theoretic over the triple graph: a `(s, p, o)` triple appearing
/// twice in the document is the *same* triple. The praxis projector
/// reads triples in document order, so a repeated triple (e.g.
/// `rdfs:label "EmptyCollection"@en` on both a class and a same-IRI
/// individual — OWL 2 punning, §5.2) would be carried twice. This
/// dedupe collapses content-equal entries down to one occurrence,
/// keeping the projection set-theoretic and the round-trip stable
/// across the class / individual / property entity views.
fn dedup_class(c: &mut OwlClass) {
    dedup_literals(&mut c.labels);
    dedup_literals(&mut c.comments);
    if let Some(first) = c.labels.first() {
        c.label = Some(first.lexical.clone());
    } else {
        c.label = None;
    }
    if let Some(first) = c.comments.first() {
        c.comment = Some(first.lexical.clone());
    } else {
        c.comment = None;
    }
    dedup_annotations(&mut c.annotations);
    dedup_class_expressions(&mut c.superclass_expressions);
    dedup_class_expressions(&mut c.equivalent_classes);
    dedup_class_expressions(&mut c.disjoint_classes);
    // Re-derive the simple superclass-IRI list from the deduped
    // expression set.
    let mut named: Vec<String> = Vec::new();
    for e in &c.superclass_expressions {
        if let OwlClassExpression::Named(iri) = e
            && !named.iter().any(|s| s == iri)
        {
            named.push(iri.clone());
        }
    }
    c.superclasses = named;
}

fn dedup_property(p: &mut OwlObjectProperty) {
    dedup_literals(&mut p.labels);
    dedup_literals(&mut p.comments);
    if let Some(first) = p.labels.first() {
        p.label = Some(first.lexical.clone());
    } else {
        p.label = None;
    }
    if let Some(first) = p.comments.first() {
        p.comment = Some(first.lexical.clone());
    } else {
        p.comment = None;
    }
    dedup_annotations(&mut p.annotations);
    dedup_strings(&mut p.superproperties);
    dedup_strings(&mut p.equivalent_properties);
    dedup_strings(&mut p.disjoint_properties);
}

fn dedup_individual(i: &mut OwlIndividual) {
    dedup_literals(&mut i.labels);
    dedup_literals(&mut i.comments);
    if let Some(first) = i.labels.first() {
        i.label = Some(first.lexical.clone());
    } else {
        i.label = None;
    }
    dedup_annotations(&mut i.annotations);
    dedup_strings(&mut i.types);
}

fn dedup_literals(lits: &mut Vec<OwlLiteral>) {
    let mut seen: hashbrown::HashSet<(String, Option<String>, Option<String>)> =
        hashbrown::HashSet::new();
    lits.retain(|l| seen.insert((l.lexical.clone(), l.lang.clone(), l.datatype.clone())));
}

fn dedup_annotations(anns: &mut Vec<OwlAnnotation>) {
    let mut seen: hashbrown::HashSet<(String, String)> = hashbrown::HashSet::new();
    anns.retain(|a| {
        let key = (
            a.predicate.clone(),
            match &a.value {
                OwlAnnotationValue::Iri(i) => format!("I:{i}"),
                OwlAnnotationValue::Literal(l) => format!(
                    "L:{}:{}:{}",
                    l.lang.as_deref().unwrap_or(""),
                    l.datatype.as_deref().unwrap_or(""),
                    l.lexical
                ),
            },
        );
        seen.insert(key)
    });
}

fn dedup_class_expressions(exprs: &mut Vec<OwlClassExpression>) {
    let mut seen: hashbrown::HashSet<String> = hashbrown::HashSet::new();
    exprs.retain(|e| seen.insert(canonical_expr_form(e)));
}

fn dedup_strings(v: &mut Vec<String>) {
    let mut seen: hashbrown::HashSet<String> = hashbrown::HashSet::new();
    v.retain(|s| seen.insert(s.clone()));
}

/// Walk a subject typed `owl:Ontology` and collect its annotation
/// predicates (OWL 2 §3.5). Skips `rdf:type` itself.
fn collect_header_annotations(subj: &RdfTerm, idx: &TripleIndex, out: &mut Vec<OwlAnnotation>) {
    for (predicate, object) in idx.objects_for(subj) {
        if predicate == RdfVocabulary::RDF_TYPE {
            continue;
        }
        if let Some(value) = annotation_value_of(object) {
            out.push(OwlAnnotation {
                predicate: predicate.clone(),
                value,
            });
        }
    }
}

/// Parse the object of an `rdfs:subClassOf` / `owl:equivalentClass` /
/// `owl:disjointWith` triple as an [`OwlClassExpression`] (W3C OWL 2
/// §8). An IRI object is `Named`; a blank-node object is followed
/// through its triples to discover whether it is a restriction or one
/// of the class-construction shapes.
fn parse_class_expression(object: &RdfTerm, idx: &TripleIndex) -> OwlClassExpression {
    match object {
        RdfTerm::Iri(iri) => OwlClassExpression::Named(iri.clone()),
        RdfTerm::Blank(_) => parse_anonymous_class_expression(object, idx),
        RdfTerm::Literal { lexical, .. } => {
            // OWL 2 §8 disallows literal class expressions; the
            // typed view falls back to a Named expression on the
            // literal's lexical form so the triple is never silently
            // dropped (W3C `LiteralsCannotBeSubjects` is upstream).
            OwlClassExpression::Named(lexical.clone())
        }
    }
}

/// Dispatch a blank-node class expression by inspecting its triples.
/// OWL 2 RDF Mapping §3.2 / §3.3 maps:
///   - `_:b rdf:type owl:Restriction` ⇒ a property restriction
///   - `_:b owl:unionOf <list>`        ⇒ `ObjectUnionOf`
///   - `_:b owl:intersectionOf <list>` ⇒ `ObjectIntersectionOf`
///   - `_:b owl:complementOf X`        ⇒ `ObjectComplementOf`
fn parse_anonymous_class_expression(subj: &RdfTerm, idx: &TripleIndex) -> OwlClassExpression {
    let triples = idx.objects_for(subj);
    let mut on_property: Option<String> = None;
    let mut some_values_from: Option<RdfTerm> = None;
    let mut all_values_from: Option<RdfTerm> = None;
    let mut has_value: Option<RdfTerm> = None;
    let mut min_cardinality: Option<u32> = None;
    let mut max_cardinality: Option<u32> = None;
    let mut exact_cardinality: Option<u32> = None;
    let mut min_qualified: Option<u32> = None;
    let mut max_qualified: Option<u32> = None;
    let mut exact_qualified: Option<u32> = None;
    let mut on_class: Option<RdfTerm> = None;
    let mut union_of: Option<RdfTerm> = None;
    let mut intersection_of: Option<RdfTerm> = None;
    let mut complement_of: Option<RdfTerm> = None;
    let mut is_restriction = false;

    for (predicate, object) in &triples {
        match predicate.as_str() {
            p if p == RdfVocabulary::RDF_TYPE => {
                if let RdfTerm::Iri(t) = object
                    && t == OwlVocabulary::OWL_RESTRICTION
                {
                    is_restriction = true;
                }
            }
            p if p == OwlVocabulary::OWL_ON_PROPERTY => {
                if let RdfTerm::Iri(iri) = object {
                    on_property = Some(iri.clone());
                }
            }
            p if p == OwlVocabulary::OWL_SOME_VALUES_FROM => {
                some_values_from = Some((*object).clone());
            }
            p if p == OwlVocabulary::OWL_ALL_VALUES_FROM => {
                all_values_from = Some((*object).clone());
            }
            p if p == OwlVocabulary::OWL_HAS_VALUE => {
                has_value = Some((*object).clone());
            }
            p if p == OwlVocabulary::OWL_MIN_CARDINALITY => {
                if let Some(n) = parse_nonnegative_integer(object) {
                    min_cardinality = Some(n);
                }
            }
            p if p == OwlVocabulary::OWL_MAX_CARDINALITY => {
                if let Some(n) = parse_nonnegative_integer(object) {
                    max_cardinality = Some(n);
                }
            }
            p if p == OwlVocabulary::OWL_CARDINALITY => {
                if let Some(n) = parse_nonnegative_integer(object) {
                    exact_cardinality = Some(n);
                }
            }
            // OWL 2 §8.3.x qualified cardinality variants — same
            // semantics with an `owl:onClass` filler.
            "http://www.w3.org/2002/07/owl#minQualifiedCardinality" => {
                if let Some(n) = parse_nonnegative_integer(object) {
                    min_qualified = Some(n);
                }
            }
            "http://www.w3.org/2002/07/owl#maxQualifiedCardinality" => {
                if let Some(n) = parse_nonnegative_integer(object) {
                    max_qualified = Some(n);
                }
            }
            "http://www.w3.org/2002/07/owl#qualifiedCardinality" => {
                if let Some(n) = parse_nonnegative_integer(object) {
                    exact_qualified = Some(n);
                }
            }
            "http://www.w3.org/2002/07/owl#onClass" => {
                on_class = Some((*object).clone());
            }
            p if p == OwlVocabulary::OWL_UNION_OF => {
                union_of = Some((*object).clone());
            }
            p if p == OwlVocabulary::OWL_INTERSECTION_OF => {
                intersection_of = Some((*object).clone());
            }
            p if p == OwlVocabulary::OWL_COMPLEMENT_OF => {
                complement_of = Some((*object).clone());
            }
            _ => {}
        }
    }

    // Restriction dispatch (W3C OWL 2 §8.2).
    if is_restriction && let Some(prop) = on_property {
        let kind = if let Some(filler) = some_values_from {
            OwlRestrictionKind::SomeValuesFrom(Box::new(parse_class_expression(&filler, idx)))
        } else if let Some(filler) = all_values_from {
            OwlRestrictionKind::AllValuesFrom(Box::new(parse_class_expression(&filler, idx)))
        } else if let Some(value) = has_value {
            OwlRestrictionKind::HasValue(annotation_value_of(&value).unwrap_or(
                OwlAnnotationValue::Literal(OwlLiteral {
                    lexical: String::new(),
                    lang: None,
                    datatype: None,
                }),
            ))
        } else if let (Some(n), filler) = (exact_qualified, on_class.clone()) {
            OwlRestrictionKind::ExactCardinality {
                n,
                on_class: filler.map(|f| Box::new(parse_class_expression(&f, idx))),
            }
        } else if let (Some(n), filler) = (min_qualified, on_class.clone()) {
            OwlRestrictionKind::MinCardinality {
                n,
                on_class: filler.map(|f| Box::new(parse_class_expression(&f, idx))),
            }
        } else if let (Some(n), filler) = (max_qualified, on_class) {
            OwlRestrictionKind::MaxCardinality {
                n,
                on_class: filler.map(|f| Box::new(parse_class_expression(&f, idx))),
            }
        } else if let Some(n) = exact_cardinality {
            OwlRestrictionKind::ExactCardinality { n, on_class: None }
        } else if let Some(n) = min_cardinality {
            OwlRestrictionKind::MinCardinality { n, on_class: None }
        } else if let Some(n) = max_cardinality {
            OwlRestrictionKind::MaxCardinality { n, on_class: None }
        } else {
            // No recognisable filler — fall back to a degenerate
            // `someValuesFrom owl:Thing` so the restriction is
            // preserved structurally rather than silently dropped.
            OwlRestrictionKind::SomeValuesFrom(Box::new(OwlClassExpression::Named(
                OwlVocabulary::OWL_THING.to_string(),
            )))
        };
        return OwlClassExpression::Restriction(OwlRestriction {
            on_property: prop,
            kind,
        });
    }

    if let Some(list_head) = union_of {
        let elems = materialise_rdf_list(&list_head, idx)
            .into_iter()
            .map(|t| parse_class_expression(&t, idx))
            .collect();
        return OwlClassExpression::Union(elems);
    }
    if let Some(list_head) = intersection_of {
        let elems = materialise_rdf_list(&list_head, idx)
            .into_iter()
            .map(|t| parse_class_expression(&t, idx))
            .collect();
        return OwlClassExpression::Intersection(elems);
    }
    if let Some(target) = complement_of {
        return OwlClassExpression::Complement(Box::new(parse_class_expression(&target, idx)));
    }

    // A blank-node class expression with no recognised shape — keep
    // it as a Named expression on the blank label so the triple set
    // is not silently dropped (the canonical writer will discard
    // unreferenced unknown blanks).
    OwlClassExpression::Named(match subj {
        RdfTerm::Blank(b) => b.clone(),
        RdfTerm::Iri(i) => i.clone(),
        RdfTerm::Literal { lexical, .. } => lexical.clone(),
    })
}

/// Materialise an `rdf:first`/`rdf:rest`/`rdf:nil` cons list (RDF 1.1
/// §5.1) starting at `head`. Stops at `rdf:nil` or at any node that
/// does not continue the list.
fn materialise_rdf_list(head: &RdfTerm, idx: &TripleIndex) -> Vec<RdfTerm> {
    let nil = format!("{}{}", RdfVocabulary::RDF_NS, "nil");
    let mut out: Vec<RdfTerm> = Vec::new();
    let mut cursor = head.clone();
    let mut seen: hashbrown::HashSet<RdfTerm> = hashbrown::HashSet::new();
    loop {
        if let RdfTerm::Iri(i) = &cursor
            && *i == nil
        {
            break;
        }
        if !seen.insert(cursor.clone()) {
            break; // cycle guard
        }
        let triples = idx.objects_for(&cursor);
        let mut first: Option<RdfTerm> = None;
        let mut rest: Option<RdfTerm> = None;
        for (p, o) in &triples {
            if p.as_str() == RdfVocabulary::RDF_FIRST {
                first = Some((*o).clone());
            } else if p.as_str() == RdfVocabulary::RDF_REST {
                rest = Some((*o).clone());
            }
        }
        match (first, rest) {
            (Some(f), Some(r)) => {
                out.push(f);
                cursor = r;
            }
            (Some(f), None) => {
                out.push(f);
                break;
            }
            _ => break,
        }
    }
    out
}

/// Extract a literal from an RDF term object, if it is one.
fn literal_of(term: &RdfTerm) -> Option<OwlLiteral> {
    match term {
        RdfTerm::Literal {
            lexical,
            lang,
            datatype,
        } => Some(OwlLiteral {
            lexical: lexical.clone(),
            lang: lang.clone(),
            datatype: datatype.clone(),
        }),
        _ => None,
    }
}

/// Map an RDF object term to an [`OwlAnnotationValue`] (W3C OWL 2 §10.1).
fn annotation_value_of(term: &RdfTerm) -> Option<OwlAnnotationValue> {
    match term {
        RdfTerm::Iri(iri) => Some(OwlAnnotationValue::Iri(iri.clone())),
        RdfTerm::Blank(b) => Some(OwlAnnotationValue::Iri(b.clone())),
        RdfTerm::Literal {
            lexical,
            lang,
            datatype,
        } => Some(OwlAnnotationValue::Literal(OwlLiteral {
            lexical: lexical.clone(),
            lang: lang.clone(),
            datatype: datatype.clone(),
        })),
    }
}

/// Parse an xsd:nonNegativeInteger / xsd:int cardinality literal.
fn parse_nonnegative_integer(term: &RdfTerm) -> Option<u32> {
    match term {
        RdfTerm::Literal { lexical, .. } => lexical.parse::<u32>().ok(),
        _ => None,
    }
}

// =============================================================================
// Triple index — subject-keyed map plus type-indexed selection
// =============================================================================

/// Subject → (predicate, object) lookup, preserving document order
/// per subject. The OWL projection scans subjects keyed by IRI and
/// blank-node label; this index makes each `objects_for(subject)`
/// call O(1) for the lookup and O(k) for the iteration.
struct TripleIndex<'a> {
    by_subject: HashMap<RdfTerm, Vec<(&'a String, &'a RdfTerm)>>,
    /// `type-IRI → ordered list of subject terms` for fast §3.2.1 /
    /// §3.2.2 entity-selection passes.
    by_type: HashMap<String, Vec<&'a RdfTerm>>,
}

impl<'a> TripleIndex<'a> {
    fn build(triples: &'a [Triple]) -> Self {
        let mut by_subject: HashMap<RdfTerm, Vec<(&'a String, &'a RdfTerm)>> = HashMap::new();
        let mut by_type: HashMap<String, Vec<&'a RdfTerm>> = HashMap::new();
        for t in triples {
            by_subject
                .entry(t.subject.clone())
                .or_default()
                .push((&t.predicate, &t.object));
            if t.predicate == RdfVocabulary::RDF_TYPE
                && let RdfTerm::Iri(type_iri) = &t.object
            {
                by_type
                    .entry(type_iri.clone())
                    .or_default()
                    .push(&t.subject);
            }
        }
        Self {
            by_subject,
            by_type,
        }
    }

    /// All `(predicate, object)` pairs whose subject is `subj`, in
    /// document order. Returns owned tuples of references so the
    /// callers can pattern-match without re-borrowing the index.
    fn objects_for(&self, subj: &RdfTerm) -> Vec<(&'a String, &'a RdfTerm)> {
        self.by_subject.get(subj).cloned().unwrap_or_default()
    }

    /// All subjects typed `type_iri`, in *unspecified* (hash) order.
    /// Callers that need stable order use
    /// [`Self::subjects_by_type_in_order`] instead.
    fn subjects_by_type(&self, type_iri: &str) -> Vec<&'a RdfTerm> {
        self.by_type.get(type_iri).cloned().unwrap_or_default()
    }

    /// All subjects typed `type_iri`, in document order — the order
    /// the triple stream surfaced them.
    fn subjects_by_type_in_order(&self, type_iri: &str) -> Vec<&'a RdfTerm> {
        self.subjects_by_type(type_iri)
    }
}

// =============================================================================
// Error type
// =============================================================================

/// Structured error from [`read_owl`] — distinguishes a XML
/// well-formedness failure (`Xml`) from an RDF/XML projection failure
/// (`Rdf`). Both carry their underlying messages.
#[derive(Debug)]
pub enum OwlReadError {
    /// XML 1.0 well-formedness rejected the bytes.
    Xml(String),
    /// The RDF/XML triple reader rejected the document.
    Rdf(RdfReadError),
}

impl OwlReadError {
    /// Compatibility constructor for callers that historically built
    /// the error from a single message string.
    pub fn from_message(message: String) -> Self {
        Self::Xml(message)
    }
}

impl core::fmt::Display for OwlReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Xml(m) => write!(f, "OWL read error (XML): {m}"),
            Self::Rdf(e) => write!(f, "OWL read error (RDF/XML): {e}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OwlReadError {}

// =============================================================================
// Graph-equivalence helper — used by Phase 2's round-trip test.
// =============================================================================

/// Compare two `OwlOntology` values as RDF graphs, not by Vec
/// position. Two ontologies are considered equivalent iff:
///
///  - they declare the same set of class IRIs;
///  - they declare the same set of object-property IRIs;
///  - they declare the same set of individual IRIs;
///  - their `taxonomy` and `property_taxonomy` edge sets are equal;
///  - per-class `superclasses` / `equivalent_classes` /
///    `disjoint_classes` collapse to equal *sets* of class
///    expressions (compared by structural content, not Vec position);
///  - per-property `superproperties` / `inverse_of` /
///    `equivalent_properties` / `disjoint_properties` are equal sets;
///  - the multi-set of literals (labels + comments) on each entity
///    is the same.
///
/// This is the RDF-graph equality the OWL 2 RDF Mapping promises
/// (Patel-Schneider & Motik 2012 §3 — the mapping is set-theoretic
/// over the triple set), simplified to the praxis OWL projection's
/// fields. Used by the Phase 2 round-trip test to assert
/// `read_owl(write_owl(read_owl(b))) ≡_graph read_owl(b)`.
pub fn owl_equivalent(a: &OwlOntology, b: &OwlOntology) -> bool {
    if a.iri != b.iri {
        return false;
    }
    fn class_set(o: &OwlOntology) -> hashbrown::HashSet<&str> {
        o.classes.iter().map(|c| c.iri.as_str()).collect()
    }
    fn prop_set(o: &OwlOntology) -> hashbrown::HashSet<&str> {
        o.properties.iter().map(|p| p.iri.as_str()).collect()
    }
    fn ind_set(o: &OwlOntology) -> hashbrown::HashSet<&str> {
        o.individuals.iter().map(|i| i.iri.as_str()).collect()
    }
    if class_set(a) != class_set(b) {
        return false;
    }
    if prop_set(a) != prop_set(b) {
        return false;
    }
    if ind_set(a) != ind_set(b) {
        return false;
    }

    fn taxonomy_set(o: &OwlOntology) -> hashbrown::HashSet<(&str, &str)> {
        o.taxonomy
            .iter()
            .map(|(c, p)| (c.as_str(), p.as_str()))
            .collect()
    }
    fn prop_taxonomy_set(o: &OwlOntology) -> hashbrown::HashSet<(&str, &str)> {
        o.property_taxonomy
            .iter()
            .map(|(c, p)| (c.as_str(), p.as_str()))
            .collect()
    }
    if taxonomy_set(a) != taxonomy_set(b) {
        return false;
    }
    if prop_taxonomy_set(a) != prop_taxonomy_set(b) {
        return false;
    }

    // Per-class label/comment multisets and class-expression sets.
    let by_iri_a: HashMap<&str, &OwlClass> =
        a.classes.iter().map(|c| (c.iri.as_str(), c)).collect();
    let by_iri_b: HashMap<&str, &OwlClass> =
        b.classes.iter().map(|c| (c.iri.as_str(), c)).collect();
    for (iri, ca) in &by_iri_a {
        let cb = by_iri_b.get(iri).expect("class set already equal");
        if literal_multiset(&ca.labels) != literal_multiset(&cb.labels) {
            return false;
        }
        if literal_multiset(&ca.comments) != literal_multiset(&cb.comments) {
            return false;
        }
        if expr_multiset(&ca.superclass_expressions) != expr_multiset(&cb.superclass_expressions) {
            return false;
        }
        if expr_multiset(&ca.equivalent_classes) != expr_multiset(&cb.equivalent_classes) {
            return false;
        }
        if expr_multiset(&ca.disjoint_classes) != expr_multiset(&cb.disjoint_classes) {
            return false;
        }
    }

    // Per-property label/comment multisets and IRI sets.
    let pa: HashMap<&str, &OwlObjectProperty> =
        a.properties.iter().map(|p| (p.iri.as_str(), p)).collect();
    let pb: HashMap<&str, &OwlObjectProperty> =
        b.properties.iter().map(|p| (p.iri.as_str(), p)).collect();
    for (iri, p_a) in &pa {
        let p_b = pb.get(iri).expect("property set already equal");
        if literal_multiset(&p_a.labels) != literal_multiset(&p_b.labels) {
            return false;
        }
        if literal_multiset(&p_a.comments) != literal_multiset(&p_b.comments) {
            return false;
        }
        if p_a.inverse_of != p_b.inverse_of {
            return false;
        }
        let set = |v: &Vec<String>| v.iter().cloned().collect::<hashbrown::HashSet<String>>();
        if set(&p_a.equivalent_properties) != set(&p_b.equivalent_properties) {
            return false;
        }
        if set(&p_a.disjoint_properties) != set(&p_b.disjoint_properties) {
            return false;
        }
    }

    true
}

fn literal_multiset(lits: &[OwlLiteral]) -> Vec<OwlLiteral> {
    let mut v: Vec<OwlLiteral> = lits.to_vec();
    v.sort_by(|x, y| {
        (x.lang.as_deref(), x.datatype.as_deref(), x.lexical.as_str()).cmp(&(
            y.lang.as_deref(),
            y.datatype.as_deref(),
            y.lexical.as_str(),
        ))
    });
    v
}

fn expr_multiset(exprs: &[OwlClassExpression]) -> Vec<String> {
    let mut v: Vec<String> = exprs.iter().map(canonical_expr_form).collect();
    v.sort();
    v
}

/// A deterministic textual encoding of an [`OwlClassExpression`] for
/// multiset comparison. Named classes encode as their IRI;
/// constructors encode as `(kind elem₁ elem₂ …)`; restrictions encode
/// as `(restr <prop> <kind> <filler>)`.
pub(super) fn canonical_expr_form(expr: &OwlClassExpression) -> String {
    match expr {
        OwlClassExpression::Named(iri) => format!("N:{iri}"),
        OwlClassExpression::Union(parts) => {
            let mut s: Vec<String> = parts.iter().map(canonical_expr_form).collect();
            s.sort();
            format!("U:[{}]", s.join("|"))
        }
        OwlClassExpression::Intersection(parts) => {
            let mut s: Vec<String> = parts.iter().map(canonical_expr_form).collect();
            s.sort();
            format!("I:[{}]", s.join("|"))
        }
        OwlClassExpression::Complement(inner) => {
            format!("C:[{}]", canonical_expr_form(inner))
        }
        OwlClassExpression::Restriction(r) => {
            let kind_form = match &r.kind {
                OwlRestrictionKind::SomeValuesFrom(inner) => {
                    format!("svf:{}", canonical_expr_form(inner))
                }
                OwlRestrictionKind::AllValuesFrom(inner) => {
                    format!("avf:{}", canonical_expr_form(inner))
                }
                OwlRestrictionKind::HasValue(v) => match v {
                    OwlAnnotationValue::Iri(i) => format!("hv-iri:{i}"),
                    OwlAnnotationValue::Literal(l) => format!(
                        "hv-lit:{}:{}:{}",
                        l.lang.as_deref().unwrap_or(""),
                        l.datatype.as_deref().unwrap_or(""),
                        l.lexical
                    ),
                },
                OwlRestrictionKind::MinCardinality { n, on_class } => {
                    let inner = on_class
                        .as_ref()
                        .map(|e| canonical_expr_form(e))
                        .unwrap_or_default();
                    format!("min:{n}:{inner}")
                }
                OwlRestrictionKind::MaxCardinality { n, on_class } => {
                    let inner = on_class
                        .as_ref()
                        .map(|e| canonical_expr_form(e))
                        .unwrap_or_default();
                    format!("max:{n}:{inner}")
                }
                OwlRestrictionKind::ExactCardinality { n, on_class } => {
                    let inner = on_class
                        .as_ref()
                        .map(|e| canonical_expr_form(e))
                        .unwrap_or_default();
                    format!("exact:{n}:{inner}")
                }
            };
            format!("R:({})[{}]", r.on_property, kind_form)
        }
    }
}
