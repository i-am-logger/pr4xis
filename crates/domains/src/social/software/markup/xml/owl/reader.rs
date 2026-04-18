#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use crate::social::software::markup::xml::ontology::{XmlElement, XmlNode};
use crate::social::software::markup::xml::reader as xml_reader;

/// RDFS predicate local names — the ontological meaning of child elements.
/// These are not arbitrary strings; each is a well-known RDFS/OWL predicate.
fn is_rdfs_label(name: &str) -> bool {
    name == "label"
}
fn is_rdfs_comment(name: &str) -> bool {
    name == "comment"
}
fn is_rdfs_subclass_of(name: &str) -> bool {
    name == "subClassOf"
}
fn is_rdfs_domain(name: &str) -> bool {
    name == "domain"
}
fn is_rdfs_range(name: &str) -> bool {
    name == "range"
}
fn is_rdf_type(name: &str) -> bool {
    name == "type"
}

/// Read an OWL/RDF ontology through the XML ontology.
///
/// OWL is built on RDF which is built on XML.
/// We read the XML structure, then interpret the RDF/OWL elements
/// through their ontological meaning.
pub fn read_owl(xml_text: &str) -> Result<OwlOntology, OwlReadError> {
    let doc = xml_reader::read_xml(xml_text).map_err(|e| OwlReadError(e.message))?;

    // Extract the base namespace from the rdf:RDF root element.
    // rdf:ID="Foo" is shorthand for base_ns + "#Foo".
    let base_ns = extract_base_ns(&doc.root);

    let mut ontology = OwlOntology {
        iri: String::new(),
        classes: Vec::new(),
        properties: Vec::new(),
        individuals: Vec::new(),
        taxonomy: Vec::new(),
    };

    // The root is rdf:RDF
    process_element(&doc.root, &mut ontology, &base_ns);

    if ontology.iri.is_empty() {
        ontology.iri.clone_from(&base_ns);
    }

    // OWL allows reopening a class to add annotations.
    // Merge duplicate IRIs into a single OwlClass.
    deduplicate_classes(&mut ontology);
    deduplicate_taxonomy(&mut ontology);

    Ok(ontology)
}

/// Extract the base namespace from the rdf:RDF root element.
fn extract_base_ns(root: &XmlElement) -> String {
    // xml:base takes priority
    for attr in &root.attributes {
        if attr.name.local == "base" {
            return attr.value.clone();
        }
    }
    // Default namespace (xmlns="...") is the ontology base
    for attr in &root.attributes {
        if attr.name.prefix.is_none() && attr.name.local == "xmlns" {
            return attr.value.trim_end_matches('#').to_string();
        }
    }
    String::new()
}

/// Resolve an IRI reference to an absolute IRI.
/// - `rdf:about="#Foo"` → `base_ns#Foo`
/// - `rdf:ID="Foo"` → `base_ns#Foo`
/// - `rdf:about="http://..."` → used as-is
fn resolve_iri(raw: &str, base_ns: &str) -> String {
    if raw.starts_with('#') {
        format!("{}{}", base_ns, raw)
    } else if raw.starts_with("http") || raw.starts_with("urn:") {
        raw.to_string()
    } else if !raw.is_empty() {
        // rdf:ID value — no # prefix, needs base + #
        format!("{}#{}", base_ns, raw)
    } else {
        String::new()
    }
}

fn get_iri(elem: &XmlElement, base_ns: &str) -> String {
    if let Some(about) = elem.attribute("about") {
        resolve_iri(about, base_ns)
    } else if let Some(id) = elem.attribute("ID") {
        resolve_iri(id, base_ns)
    } else {
        String::new()
    }
}

fn get_resource(elem: &XmlElement, base_ns: &str) -> Option<String> {
    elem.attribute("resource").map(|r| resolve_iri(r, base_ns))
}

fn process_element(elem: &XmlElement, ont: &mut OwlOntology, base_ns: &str) {
    // Use the OWL vocabulary to identify what this element IS,
    // rather than matching on raw strings.
    let concept = OwlVocabulary::from_local_name(&elem.name.local);

    match concept {
        Some(OwlConcept::Ontology) => {
            let iri = get_iri(elem, base_ns);
            if !iri.is_empty() {
                ont.iri = iri;
            }
        }
        Some(OwlConcept::Class) => {
            read_class(elem, ont, base_ns);
        }
        Some(OwlConcept::ObjectProperty) => {
            read_property(elem, ont, base_ns);
        }
        Some(OwlConcept::NamedIndividual) => {
            read_individual(elem, ont, base_ns);
        }
        _ => {}
    }

    // Recurse into children
    for child in &elem.children {
        if let XmlNode::Element(child_elem) = child {
            process_element(child_elem, ont, base_ns);
        }
    }
}

fn read_class(elem: &XmlElement, ont: &mut OwlOntology, base_ns: &str) {
    let iri = get_iri(elem, base_ns);

    if iri.is_empty() {
        return;
    }

    let mut label = None;
    let mut comment = None;
    let mut superclasses = Vec::new();

    for child in &elem.children {
        if let XmlNode::Element(child_elem) = child {
            let local = &child_elem.name.local;
            if is_rdfs_label(local) {
                label = Some(child_elem.text_content());
            } else if is_rdfs_comment(local) {
                comment = Some(child_elem.text_content());
            } else if is_rdfs_subclass_of(local) {
                if let Some(resource) = get_resource(child_elem, base_ns) {
                    superclasses.push(resource.clone());
                    ont.taxonomy.push((iri.clone(), resource));
                }
                // Also check for nested Class/Restriction references
                for grandchild in &child_elem.children {
                    if let XmlNode::Element(gc_elem) = grandchild {
                        let is_class_or_restriction =
                            OwlVocabulary::from_local_name(&gc_elem.name.local)
                                .is_some_and(|c| c.is_class_expression());
                        if is_class_or_restriction {
                            let gc_iri = get_iri(gc_elem, base_ns);
                            if !gc_iri.is_empty() {
                                superclasses.push(gc_iri.clone());
                                ont.taxonomy.push((iri.clone(), gc_iri));
                            }
                        }
                    }
                }
            }
        }
    }

    ont.classes.push(OwlClass {
        iri,
        label,
        comment,
        superclasses,
    });
}

fn read_property(elem: &XmlElement, ont: &mut OwlOntology, base_ns: &str) {
    let iri = get_iri(elem, base_ns);

    if iri.is_empty() {
        return;
    }

    let mut label = None;
    let mut domain = None;
    let mut range = None;

    for child in &elem.children {
        if let XmlNode::Element(child_elem) = child {
            let local = &child_elem.name.local;
            if is_rdfs_label(local) {
                label = Some(child_elem.text_content());
            } else if is_rdfs_domain(local) {
                domain = get_resource(child_elem, base_ns);
            } else if is_rdfs_range(local) {
                range = get_resource(child_elem, base_ns);
            }
        }
    }

    ont.properties.push(OwlObjectProperty {
        iri,
        label,
        domain,
        range,
    });
}

fn read_individual(elem: &XmlElement, ont: &mut OwlOntology, base_ns: &str) {
    let iri = get_iri(elem, base_ns);

    if iri.is_empty() {
        return;
    }

    let mut types = Vec::new();
    let mut label = None;

    for child in &elem.children {
        if let XmlNode::Element(child_elem) = child {
            let local = &child_elem.name.local;
            if is_rdf_type(local) {
                if let Some(resource) = get_resource(child_elem, base_ns) {
                    types.push(resource);
                }
            } else if is_rdfs_label(local) {
                label = Some(child_elem.text_content());
            }
        }
    }

    ont.individuals.push(OwlIndividual { iri, types, label });
}

#[derive(Debug)]
pub struct OwlReadError(pub String);

impl core::fmt::Display for OwlReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OWL read error: {}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OwlReadError {}

/// Merge duplicate class IRIs — OWL files reopen classes to add properties.
fn deduplicate_classes(ont: &mut OwlOntology) {
    use hashbrown::HashMap;
    let mut by_iri: HashMap<String, OwlClass> = HashMap::new();

    for class in ont.classes.drain(..) {
        by_iri
            .entry(class.iri.clone())
            .and_modify(|existing| {
                if existing.label.is_none() {
                    existing.label.clone_from(&class.label);
                }
                if existing.comment.is_none() {
                    existing.comment.clone_from(&class.comment);
                }
                for sc in &class.superclasses {
                    if !existing.superclasses.contains(sc) {
                        existing.superclasses.push(sc.clone());
                    }
                }
            })
            .or_insert(class);
    }

    ont.classes = by_iri.into_values().collect();
}

/// Remove duplicate taxonomy pairs.
fn deduplicate_taxonomy(ont: &mut OwlOntology) {
    ont.taxonomy.sort();
    ont.taxonomy.dedup();
}
