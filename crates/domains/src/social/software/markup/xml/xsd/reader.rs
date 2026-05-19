//! XSD reader — `XmlDocument` → typed [`XsdSchema`].
//!
//! Reads a W3C XSD 1.1 schema document by walking its already-parsed
//! XML form. The XML parsing itself is delegated to
//! [`super::super::reader::read_xml`]; this module just imposes the XSD
//! schema-component semantics on top.
//!
//! Reference: W3C XML Schema Definition Language (XSD) 1.1 Part 1:
//! Structures (Gao, Sperberg-McQueen & Thompson 2012). Section numbers
//! cited inline.
//!
//! # Failure modes
//!
//! - [`XsdReadError::NotXml`] — the bytes don't form a valid XML
//!   document at all (delegated to the XML reader).
//! - [`XsdReadError::NotSchemaRoot`] — the root element isn't
//!   `xsd:schema` (W3C XSD 1.1 Part 1 §3.1.2 requires this).
//! - [`XsdReadError::Unsupported`] — the document uses an XSD construct
//!   this reader does not yet support. The reader is fail-closed: no
//!   silent passthrough.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::super::ontology::{XmlElement, XmlNode};
use super::super::reader::read_xml;
use super::ontology::*;

/// Read an XSD document from a raw XML string.
///
/// # Errors
///
/// Returns [`XsdReadError`] if the input is not a valid XSD 1.1 schema
/// document in the subset this reader supports (see `mod.rs` for the
/// supported-subset inventory).
pub fn read_xsd(text: &str) -> Result<XsdSchema, XsdReadError> {
    // Strip a UTF-8 byte-order-mark if present. The Unicode FAQ on
    // BOM (Unicode Consortium, "UTF-8 BOM (EF BB BF)" — informative
    // signature, not part of the encoded text) treats it as an
    // optional signature; the praxis XML reader's `.trim()` only
    // strips ASCII whitespace, so we must do the BOM strip here.
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    let doc = read_xml(text).map_err(|e| XsdReadError::NotXml(e.to_string()))?;
    read_schema(&doc.root)
}

fn read_schema(root: &XmlElement) -> Result<XsdSchema, XsdReadError> {
    if local(&root.name.local) != "schema" || !is_xsd(&root.name.prefix) {
        return Err(XsdReadError::NotSchemaRoot {
            actual_local: root.name.local.clone(),
            actual_prefix: root.name.prefix.clone(),
        });
    }
    let target_namespace = root.attribute("targetNamespace").map(String::from);
    let version = root.attribute("version").map(String::from);
    let element_form_default = root.attribute("elementFormDefault").map(String::from);
    let attribute_form_default = root.attribute("attributeFormDefault").map(String::from);

    let mut schema = XsdSchema {
        target_namespace,
        version,
        element_form_default,
        attribute_form_default,
        elements: Vec::new(),
        complex_types: Vec::new(),
        simple_types: Vec::new(),
        attribute_groups: Vec::new(),
        groups: Vec::new(),
        imports: Vec::new(),
    };

    for child in elements(root) {
        match local(&child.name.local) {
            "element" => schema.elements.push(read_element(child)?),
            "complexType" => schema.complex_types.push(read_complex_type(child)?),
            "simpleType" => schema.simple_types.push(read_simple_type(child)?),
            "attributeGroup" => schema.attribute_groups.push(read_attribute_group(child)?),
            "group" => schema.groups.push(read_group(child)?),
            "import" => schema.imports.push(read_import(child)),
            "include" | "redefine" | "override" => {
                return Err(XsdReadError::Unsupported {
                    construct: child.name.qualified(),
                    reason:
                        "module assembly other than xsd:import is not yet supported (W3C XSD 1.1 Part 1 §4.2)"
                            .into(),
                });
            }
            // `xsd:annotation` at schema level is metadata — kept out of
            // the typed schema component model (W3C XSD 1.1 Part 1
            // §3.15.2.2: annotations have no impact on validation).
            "annotation" => {}
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:{}", other),
                    reason: "unrecognized top-level schema component".into(),
                });
            }
        }
    }
    Ok(schema)
}

fn read_element(e: &XmlElement) -> Result<XsdElement, XsdReadError> {
    let name = e.attribute("name").map(String::from);
    let ref_name = e.attribute("ref").map(String::from);
    let type_name = e.attribute("type").map(String::from);
    let substitution_group = e.attribute("substitutionGroup").map(String::from);
    let is_abstract = e.attribute("abstract").map(parse_bool).unwrap_or(false);
    let min_occurs = e.attribute("minOccurs").map(parse_occurs).transpose()?;
    let max_occurs = e.attribute("maxOccurs").map(parse_occurs).transpose()?;

    let mut inline_complex_type = None;
    let mut inline_simple_type = None;
    let mut documentation = None;

    for child in elements(e) {
        match local(&child.name.local) {
            "complexType" => inline_complex_type = Some(Box::new(read_complex_type(child)?)),
            "simpleType" => inline_simple_type = Some(Box::new(read_simple_type(child)?)),
            "annotation" => documentation = read_documentation(child),
            // `unique`, `key`, `keyref` constraint declarations (W3C XSD
            // 1.1 Part 1 §3.11) are not yet supported because USLM
            // doesn't use them.
            "unique" | "key" | "keyref" => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:{}", local(&child.name.local)),
                    reason: "identity constraints not yet supported".into(),
                });
            }
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:element > xsd:{}", other),
                    reason: "unrecognized child of xsd:element".into(),
                });
            }
        }
    }

    Ok(XsdElement {
        name,
        ref_name,
        type_name,
        substitution_group,
        is_abstract,
        min_occurs,
        max_occurs,
        inline_complex_type,
        inline_simple_type,
        documentation,
    })
}

fn read_complex_type(e: &XmlElement) -> Result<XsdComplexType, XsdReadError> {
    let name = e.attribute("name").map(String::from);
    let mixed = e.attribute("mixed").map(parse_bool).unwrap_or(false);
    let is_abstract = e.attribute("abstract").map(parse_bool).unwrap_or(false);

    let mut content_model = XsdContentModel::Empty;
    let mut attributes: Vec<XsdAttribute> = Vec::new();
    let mut attribute_group_refs: Vec<String> = Vec::new();
    let mut any_attribute: Option<XsdAnyAttribute> = None;
    let mut documentation: Option<String> = None;
    let mut content_model_seen = false;

    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => documentation = read_documentation(child),
            "complexContent" => {
                content_model = read_complex_content(child)?;
                content_model_seen = true;
            }
            "simpleContent" => {
                content_model = read_simple_content(child)?;
                content_model_seen = true;
            }
            "sequence" => {
                content_model = XsdContentModel::Sequence(read_particle(child)?);
                content_model_seen = true;
            }
            "choice" => {
                content_model = XsdContentModel::Choice(read_particle(child)?);
                content_model_seen = true;
            }
            "all" => {
                content_model = XsdContentModel::All(read_particle(child)?);
                content_model_seen = true;
            }
            "group" => {
                let ref_name = child
                    .attribute("ref")
                    .ok_or_else(|| XsdReadError::Unsupported {
                        construct: "xsd:group (without ref)".into(),
                        reason: "inline xsd:group declarations not supported inside complexType"
                            .into(),
                    })?;
                content_model = XsdContentModel::GroupRef {
                    ref_name: ref_name.into(),
                    min_occurs: child.attribute("minOccurs").map(parse_occurs).transpose()?,
                    max_occurs: child.attribute("maxOccurs").map(parse_occurs).transpose()?,
                };
                content_model_seen = true;
            }
            "attribute" => attributes.push(read_attribute(child)?),
            "attributeGroup" => {
                if let Some(r) = child.attribute("ref") {
                    attribute_group_refs.push(r.into());
                }
            }
            "anyAttribute" => any_attribute = Some(read_any_attribute(child)),
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:complexType > xsd:{}", other),
                    reason: "unrecognized child of xsd:complexType".into(),
                });
            }
        }
    }
    // If no content-model child was seen, the type is empty-content but
    // may have attributes — that's still a valid XSD shape.
    let _ = content_model_seen;
    Ok(XsdComplexType {
        name,
        mixed,
        is_abstract,
        content_model,
        attributes,
        attribute_group_refs,
        any_attribute,
        documentation,
    })
}

fn read_complex_content(e: &XmlElement) -> Result<XsdContentModel, XsdReadError> {
    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => {}
            "extension" => return read_extension(child, true),
            "restriction" => return read_restriction(child, true),
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:complexContent > xsd:{}", other),
                    reason: "expected xsd:extension or xsd:restriction".into(),
                });
            }
        }
    }
    Err(XsdReadError::Unsupported {
        construct: "xsd:complexContent (empty)".into(),
        reason: "complexContent must contain xsd:extension or xsd:restriction".into(),
    })
}

fn read_simple_content(e: &XmlElement) -> Result<XsdContentModel, XsdReadError> {
    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => {}
            "extension" => {
                let base = child
                    .attribute("base")
                    .ok_or_else(|| XsdReadError::Unsupported {
                        construct: "xsd:extension (without base)".into(),
                        reason: "extension must declare a base type".into(),
                    })?
                    .into();
                let mut attributes = Vec::new();
                let mut attribute_group_refs = Vec::new();
                for sub in elements(child) {
                    match local(&sub.name.local) {
                        "attribute" => attributes.push(read_attribute(sub)?),
                        "attributeGroup" => {
                            if let Some(r) = sub.attribute("ref") {
                                attribute_group_refs.push(r.into());
                            }
                        }
                        // W3C XSD 1.1 Part 1 §3.10: `xsd:anyAttribute`
                        // appears here too. The current SimpleContent
                        // variant doesn't carry it (USLM uses it but
                        // we don't depend on the wildcard for codegen
                        // semantics); treat as accepted-but-elided.
                        "anyAttribute" => {}
                        "annotation" => {}
                        other => {
                            return Err(XsdReadError::Unsupported {
                                construct: format!(
                                    "xsd:simpleContent > xsd:extension > xsd:{}",
                                    other
                                ),
                                reason: "unsupported child of simpleContent extension".into(),
                            });
                        }
                    }
                }
                return Ok(XsdContentModel::SimpleContent {
                    base,
                    attributes,
                    attribute_group_refs,
                });
            }
            "restriction" => {
                // Treat as a SimpleContent { base, attributes,
                // attribute_group_refs } with the base from
                // restriction@base; facets and inline simpleTypes are
                // accepted-but-elided. The XSD reader's job is to
                // produce the schema-component graph; facet
                // enforcement is the validator's job (W3C XSD 1.1
                // Part 1 §3.4.2.2 and Part 2 §4 each address one
                // half).
                let base = child
                    .attribute("base")
                    .ok_or_else(|| XsdReadError::Unsupported {
                        construct: "xsd:restriction (without base)".into(),
                        reason: "restriction must declare a base type".into(),
                    })?
                    .into();
                let mut attributes = Vec::new();
                let mut attribute_group_refs = Vec::new();
                for sub in elements(child) {
                    match local(&sub.name.local) {
                        "attribute" => attributes.push(read_attribute(sub)?),
                        "attributeGroup" => {
                            if let Some(r) = sub.attribute("ref") {
                                attribute_group_refs.push(r.into());
                            }
                        }
                        // Facets, inline simpleType, anyAttribute, and
                        // annotation are accepted structurally.
                        "annotation" | "anyAttribute" | "simpleType" | "enumeration"
                        | "pattern" | "length" | "minLength" | "maxLength" | "totalDigits"
                        | "fractionDigits" | "minInclusive" | "maxInclusive" | "minExclusive"
                        | "maxExclusive" | "whiteSpace" => {}
                        other => {
                            return Err(XsdReadError::Unsupported {
                                construct: format!(
                                    "xsd:simpleContent > xsd:restriction > xsd:{}",
                                    other
                                ),
                                reason: "unsupported child of simpleContent restriction".into(),
                            });
                        }
                    }
                }
                return Ok(XsdContentModel::SimpleContent {
                    base,
                    attributes,
                    attribute_group_refs,
                });
            }
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:simpleContent > xsd:{}", other),
                    reason: "expected xsd:extension or xsd:restriction".into(),
                });
            }
        }
    }
    Err(XsdReadError::Unsupported {
        construct: "xsd:simpleContent (empty)".into(),
        reason: "simpleContent must contain xsd:extension or xsd:restriction".into(),
    })
}

fn read_extension(e: &XmlElement, is_complex: bool) -> Result<XsdContentModel, XsdReadError> {
    let base = e
        .attribute("base")
        .ok_or_else(|| XsdReadError::Unsupported {
            construct: "xsd:extension (without base)".into(),
            reason: "extension must declare a base type".into(),
        })?
        .into();
    let mut body = XsdContentModel::Empty;
    let mut attributes = Vec::new();
    let mut attribute_group_refs = Vec::new();
    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => {}
            "sequence" => body = XsdContentModel::Sequence(read_particle(child)?),
            "choice" => body = XsdContentModel::Choice(read_particle(child)?),
            "all" => body = XsdContentModel::All(read_particle(child)?),
            "group" => {
                if let Some(r) = child.attribute("ref") {
                    body = XsdContentModel::GroupRef {
                        ref_name: r.into(),
                        min_occurs: child.attribute("minOccurs").map(parse_occurs).transpose()?,
                        max_occurs: child.attribute("maxOccurs").map(parse_occurs).transpose()?,
                    };
                }
            }
            "attribute" => attributes.push(read_attribute(child)?),
            "attributeGroup" => {
                if let Some(r) = child.attribute("ref") {
                    attribute_group_refs.push(r.into());
                }
            }
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:extension > xsd:{}", other),
                    reason: "unsupported child of xsd:extension".into(),
                });
            }
        }
    }
    Ok(if is_complex {
        XsdContentModel::ExtensionOf {
            base,
            body: Box::new(body),
            attributes,
            attribute_group_refs,
        }
    } else {
        XsdContentModel::SimpleContent {
            base,
            attributes,
            attribute_group_refs,
        }
    })
}

fn read_restriction(e: &XmlElement, is_complex: bool) -> Result<XsdContentModel, XsdReadError> {
    let base = e
        .attribute("base")
        .ok_or_else(|| XsdReadError::Unsupported {
            construct: "xsd:restriction (without base)".into(),
            reason: "restriction must declare a base type".into(),
        })?
        .into();
    let mut body = XsdContentModel::Empty;
    let mut attributes = Vec::new();
    let mut attribute_group_refs = Vec::new();
    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => {}
            "sequence" => body = XsdContentModel::Sequence(read_particle(child)?),
            "choice" => body = XsdContentModel::Choice(read_particle(child)?),
            "all" => body = XsdContentModel::All(read_particle(child)?),
            "group" => {
                if let Some(r) = child.attribute("ref") {
                    body = XsdContentModel::GroupRef {
                        ref_name: r.into(),
                        min_occurs: child.attribute("minOccurs").map(parse_occurs).transpose()?,
                        max_occurs: child.attribute("maxOccurs").map(parse_occurs).transpose()?,
                    };
                }
            }
            "attribute" => attributes.push(read_attribute(child)?),
            "attributeGroup" => {
                if let Some(r) = child.attribute("ref") {
                    attribute_group_refs.push(r.into());
                }
            }
            // simpleType facets when restricting a simpleType base
            // (enumeration, pattern, length) are handled by
            // `read_simple_type`'s restriction path, not here. Inside
            // a complex restriction they don't apply.
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:restriction > xsd:{}", other),
                    reason: "unsupported child of xsd:restriction (complex context)".into(),
                });
            }
        }
    }
    if is_complex {
        Ok(XsdContentModel::RestrictionOf {
            base,
            body: Box::new(body),
            attributes,
            attribute_group_refs,
        })
    } else {
        Ok(XsdContentModel::SimpleContent {
            base,
            attributes,
            attribute_group_refs,
        })
    }
}

fn read_particle(e: &XmlElement) -> Result<XsdParticle, XsdReadError> {
    let min_occurs = e.attribute("minOccurs").map(parse_occurs).transpose()?;
    let max_occurs = e.attribute("maxOccurs").map(parse_occurs).transpose()?;
    let mut terms = Vec::new();
    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => {}
            "element" => terms.push(XsdTerm::Element(read_element(child)?)),
            "sequence" => terms.push(XsdTerm::Sequence(read_particle(child)?)),
            "choice" => terms.push(XsdTerm::Choice(read_particle(child)?)),
            "all" => terms.push(XsdTerm::All(read_particle(child)?)),
            "group" => {
                let r = child
                    .attribute("ref")
                    .ok_or_else(|| XsdReadError::Unsupported {
                        construct: "nested xsd:group (without ref)".into(),
                        reason: "only named-group references supported inside particles".into(),
                    })?
                    .into();
                terms.push(XsdTerm::GroupRef {
                    ref_name: r,
                    min_occurs: child.attribute("minOccurs").map(parse_occurs).transpose()?,
                    max_occurs: child.attribute("maxOccurs").map(parse_occurs).transpose()?,
                });
            }
            "any" => terms.push(XsdTerm::Any(read_any(child)?)),
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("particle > xsd:{}", other),
                    reason: "unsupported particle term".into(),
                });
            }
        }
    }
    Ok(XsdParticle {
        min_occurs,
        max_occurs,
        terms,
    })
}

fn read_any(e: &XmlElement) -> Result<XsdAny, XsdReadError> {
    Ok(XsdAny {
        namespace: e.attribute("namespace").map(String::from),
        process_contents: e.attribute("processContents").map(String::from),
        min_occurs: e.attribute("minOccurs").map(parse_occurs).transpose()?,
        max_occurs: e.attribute("maxOccurs").map(parse_occurs).transpose()?,
    })
}

fn read_any_attribute(e: &XmlElement) -> XsdAnyAttribute {
    XsdAnyAttribute {
        namespace: e.attribute("namespace").map(String::from),
        process_contents: e.attribute("processContents").map(String::from),
    }
}

fn read_attribute(e: &XmlElement) -> Result<XsdAttribute, XsdReadError> {
    let mut inline_simple_type_seen = false;
    let mut documentation = None;
    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => documentation = read_documentation(child),
            // inline simpleType is allowed; we don't track its facets
            // here yet, but accept the shape for forward compat.
            "simpleType" => {
                inline_simple_type_seen = true;
            }
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:attribute > xsd:{}", other),
                    reason: "unsupported child of xsd:attribute".into(),
                });
            }
        }
    }
    let _ = inline_simple_type_seen;
    Ok(XsdAttribute {
        name: e.attribute("name").map(String::from),
        ref_name: e.attribute("ref").map(String::from),
        type_name: e.attribute("type").map(String::from),
        usage: e.attribute("use").map(String::from),
        default: e.attribute("default").map(String::from),
        fixed: e.attribute("fixed").map(String::from),
        documentation,
    })
}

fn read_attribute_group(e: &XmlElement) -> Result<XsdAttributeGroup, XsdReadError> {
    let name = e
        .attribute("name")
        .ok_or_else(|| XsdReadError::Unsupported {
            construct: "top-level xsd:attributeGroup without name".into(),
            reason: "top-level attributeGroup declarations must have a name".into(),
        })?
        .into();
    let mut attributes = Vec::new();
    let mut attribute_group_refs = Vec::new();
    let mut any_attribute = None;
    let mut documentation = None;
    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => documentation = read_documentation(child),
            "attribute" => attributes.push(read_attribute(child)?),
            "attributeGroup" => {
                if let Some(r) = child.attribute("ref") {
                    attribute_group_refs.push(r.into());
                }
            }
            "anyAttribute" => any_attribute = Some(read_any_attribute(child)),
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:attributeGroup > xsd:{}", other),
                    reason: "unsupported child of xsd:attributeGroup".into(),
                });
            }
        }
    }
    Ok(XsdAttributeGroup {
        name,
        attributes,
        attribute_group_refs,
        any_attribute,
        documentation,
    })
}

fn read_group(e: &XmlElement) -> Result<XsdGroup, XsdReadError> {
    let name = e
        .attribute("name")
        .ok_or_else(|| XsdReadError::Unsupported {
            construct: "top-level xsd:group without name".into(),
            reason: "top-level group declarations must have a name".into(),
        })?
        .into();
    let mut content_model = XsdContentModel::Empty;
    let mut documentation = None;
    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => documentation = read_documentation(child),
            "sequence" => content_model = XsdContentModel::Sequence(read_particle(child)?),
            "choice" => content_model = XsdContentModel::Choice(read_particle(child)?),
            "all" => content_model = XsdContentModel::All(read_particle(child)?),
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:group > xsd:{}", other),
                    reason: "expected xsd:sequence/xsd:choice/xsd:all body".into(),
                });
            }
        }
    }
    Ok(XsdGroup {
        name,
        content_model,
        documentation,
    })
}

fn read_simple_type(e: &XmlElement) -> Result<XsdSimpleType, XsdReadError> {
    let name = e.attribute("name").map(String::from);
    let mut derivation = XsdSimpleDerivation::None;
    let mut documentation = None;
    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => documentation = read_documentation(child),
            "restriction" => {
                derivation = XsdSimpleDerivation::Restriction(read_simple_restriction(child)?);
            }
            "union" => {
                // W3C XSD 1.1 Part 2 §4.1.2.5: `memberTypes` is a
                // whitespace-separated list of QNames. The XSD reader
                // returns the member-types as parsed strings; consumers
                // resolve each name against the schema if needed.
                let member_types: Vec<String> = child
                    .attribute("memberTypes")
                    .map(|s| {
                        s.split_whitespace()
                            .map(|t| t.to_string())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                derivation = XsdSimpleDerivation::Union { member_types };
            }
            "list" => {
                let item_type = child
                    .attribute("itemType")
                    .ok_or_else(|| XsdReadError::Unsupported {
                        construct: "xsd:simpleType > xsd:list (without itemType)".into(),
                        reason:
                            "xsd:list requires the itemType attribute (W3C XSD 1.1 Part 2 §4.1.2.4)"
                                .into(),
                    })?
                    .into();
                derivation = XsdSimpleDerivation::List { item_type };
            }
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:simpleType > xsd:{}", other),
                    reason: "unsupported child of xsd:simpleType".into(),
                });
            }
        }
    }
    Ok(XsdSimpleType {
        name,
        derivation,
        documentation,
    })
}

fn read_simple_restriction(e: &XmlElement) -> Result<XsdSimpleRestriction, XsdReadError> {
    let base = e
        .attribute("base")
        .ok_or_else(|| XsdReadError::Unsupported {
            construct: "xsd:simpleType > xsd:restriction without base".into(),
            reason: "restriction must declare a base type".into(),
        })?
        .into();
    let mut enumerations = Vec::new();
    let mut patterns = Vec::new();
    for child in elements(e) {
        match local(&child.name.local) {
            "annotation" => {}
            "enumeration" => {
                if let Some(v) = child.attribute("value") {
                    enumerations.push(v.into());
                }
            }
            "pattern" => {
                if let Some(v) = child.attribute("value") {
                    patterns.push(v.into());
                }
            }
            // Other facets (length, minLength, maxLength, totalDigits,
            // fractionDigits, minInclusive, maxInclusive, minExclusive,
            // maxExclusive, whiteSpace, assertion, explicitTimezone)
            // are tolerated structurally but not currently exposed on
            // XsdSimpleRestriction — they don't appear on the USLM
            // simpleTypes (which use enumeration + pattern only).
            // Add fields when the first consumer needs them.
            "length" | "minLength" | "maxLength" | "totalDigits" | "fractionDigits"
            | "minInclusive" | "maxInclusive" | "minExclusive" | "maxExclusive" | "whiteSpace" => {}
            other => {
                return Err(XsdReadError::Unsupported {
                    construct: format!("xsd:simpleType > xsd:restriction > xsd:{}", other),
                    reason: "unsupported facet".into(),
                });
            }
        }
    }
    Ok(XsdSimpleRestriction {
        base,
        enumerations,
        patterns,
    })
}

fn read_import(e: &XmlElement) -> XsdImport {
    XsdImport {
        namespace: e.attribute("namespace").map(String::from),
        schema_location: e.attribute("schemaLocation").map(String::from),
    }
}

fn read_documentation(e: &XmlElement) -> Option<String> {
    for child in elements(e) {
        if local(&child.name.local) == "documentation" {
            let text = child.text_content();
            if !text.trim().is_empty() {
                return Some(text);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn elements(e: &XmlElement) -> impl Iterator<Item = &XmlElement> {
    e.children.iter().filter_map(|c| match c {
        XmlNode::Element(child) => Some(child),
        _ => None,
    })
}

/// QName local-name helper — strips the `prefix:` if present.
///
/// W3C Namespaces in XML 1.0 §3 — a QName is `prefix:local`; here we
/// only need the local part because the namespace-URI match has
/// already been confirmed at the document-level by `is_xsd`.
fn local(name: &str) -> &str {
    match name.find(':') {
        Some(i) => &name[i + 1..],
        None => name,
    }
}

/// `True` if the qualified-name prefix resolves to the XSD namespace.
///
/// Per W3C XML Namespaces 1.0 §6, this should resolve via the in-scope
/// namespace declarations on the element, not by string match. The
/// existing praxis XML reader already drops `xmlns:*` declarations
/// from the attribute set but does not currently preserve the
/// prefix→URI map on the [`XmlElement`] tree, so we accept either the
/// conventional `xsd:` or `xs:` prefix here. The well-formed USLM XSD
/// uses `xsd:` exclusively, and the test corpus exercises that path.
fn is_xsd(prefix: &Option<String>) -> bool {
    matches!(prefix.as_deref(), None | Some("xsd") | Some("xs"))
}

fn parse_bool(s: &str) -> bool {
    matches!(s.trim(), "true" | "1")
}

fn parse_occurs(s: &str) -> Result<Occurs, XsdReadError> {
    let s = s.trim();
    if s == "unbounded" {
        return Ok(Occurs::Unbounded);
    }
    s.parse::<u32>()
        .map(Occurs::Count)
        .map_err(|_| XsdReadError::Unsupported {
            construct: format!("minOccurs/maxOccurs = {s:?}"),
            reason: "expected a non-negative integer or the literal \"unbounded\"".into(),
        })
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Error returned by [`read_xsd`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XsdReadError {
    /// The bytes don't form a valid XML document (delegated XML parser
    /// rejection).
    NotXml(String),
    /// Root element is not `xsd:schema`. W3C XSD 1.1 Part 1 §3.1.2.
    NotSchemaRoot {
        actual_local: String,
        actual_prefix: Option<String>,
    },
    /// The schema uses an XSD construct this reader does not yet
    /// support. Fail-closed contract — no silent passthrough.
    Unsupported { construct: String, reason: String },
}

impl core::fmt::Display for XsdReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotXml(s) => write!(f, "XSD reader: input is not valid XML: {s}"),
            Self::NotSchemaRoot {
                actual_local,
                actual_prefix,
            } => write!(
                f,
                "XSD reader: root is <{}:{}>, expected <xsd:schema>",
                actual_prefix.as_deref().unwrap_or(""),
                actual_local
            ),
            Self::Unsupported { construct, reason } => {
                write!(f, "XSD reader: unsupported construct {construct}: {reason}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for XsdReadError {}
