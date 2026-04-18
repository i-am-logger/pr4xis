#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use crate::social::software::markup::xml::ontology::XmlNode;
use crate::social::software::markup::xml::reader as xml_reader;

/// Read a WordNet LMF XML file into a WordNet ontology.
///
/// This reads XML through the XML ontology (understanding elements and
/// attributes), then interprets the content through the LMF ontology
/// (understanding what Synset, LexicalEntry, Sense MEAN).
pub fn read_wordnet(xml_text: &str) -> Result<WordNet, LmfReadError> {
    let xml_doc =
        xml_reader::read_xml(xml_text).map_err(|e| LmfReadError::Xml(e.message.clone()))?;

    // The root should be LexicalResource containing Lexicon
    let lexicon = xml_doc
        .find_all("Lexicon")
        .into_iter()
        .next()
        .ok_or_else(|| LmfReadError::Structure("no Lexicon element found".into()))?;

    let mut synsets = Vec::new();
    let mut entries = Vec::new();

    for child in &lexicon.children {
        if let XmlNode::Element(elem) = child {
            match elem.name.local.as_str() {
                "LexicalEntry" => {
                    if let Some(entry) = read_lexical_entry(elem) {
                        entries.push(entry);
                    }
                }
                "Synset" => {
                    if let Some(synset) = read_synset(elem) {
                        synsets.push(synset);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(WordNet { synsets, entries })
}

fn read_lexical_entry(
    elem: &crate::social::software::markup::xml::ontology::XmlElement,
) -> Option<LexicalEntry> {
    let id = elem
        .attributes
        .iter()
        .find(|a| a.name.local == "id")?
        .value
        .clone();

    let mut lemma = None;
    let mut senses = Vec::new();
    let mut forms = Vec::new();

    for child in &elem.children {
        if let XmlNode::Element(child_elem) = child {
            match child_elem.name.local.as_str() {
                "Lemma" => {
                    let written_form = attr_value(child_elem, "writtenForm")?;
                    let pos = LmfPos::parse(&attr_value(child_elem, "partOfSpeech")?);
                    lemma = Some(Lemma { written_form, pos });
                }
                "Sense" => {
                    let sense_id = attr_value(child_elem, "id").unwrap_or_default();
                    let synset = attr_value(child_elem, "synset").unwrap_or_default();
                    let subcat: Vec<String> = attr_value(child_elem, "subcat")
                        .map(|s| s.split_whitespace().map(String::from).collect())
                        .unwrap_or_default();
                    let mut relations = Vec::new();
                    for sense_child in &child_elem.children {
                        if let XmlNode::Element(rel_elem) = sense_child
                            && rel_elem.name.local == "SenseRelation"
                            && let (Some(rel_type), Some(target)) = (
                                attr_value(rel_elem, "relType"),
                                attr_value(rel_elem, "target"),
                            )
                        {
                            relations.push(SenseRelation {
                                rel_type: SenseRelationType::parse(&rel_type),
                                target,
                            });
                        }
                    }
                    senses.push(Sense {
                        id: sense_id,
                        synset,
                        relations,
                        subcat,
                    });
                }
                "Form" => {
                    if let Some(written_form) = attr_value(child_elem, "writtenForm") {
                        forms.push(Form { written_form });
                    }
                }
                _ => {}
            }
        }
    }

    Some(LexicalEntry {
        id,
        lemma: lemma?,
        senses,
        forms,
    })
}

fn read_synset(
    elem: &crate::social::software::markup::xml::ontology::XmlElement,
) -> Option<Synset> {
    let id = attr_value(elem, "id")?;
    let ili = attr_value(elem, "ili");
    let pos = LmfPos::parse(&attr_value(elem, "partOfSpeech").unwrap_or_default());
    let members: Vec<String> = attr_value(elem, "members")
        .map(|m| m.split_whitespace().map(String::from).collect())
        .unwrap_or_default();

    let mut definitions = Vec::new();
    let mut examples = Vec::new();
    let mut relations = Vec::new();

    for child in &elem.children {
        if let XmlNode::Element(child_elem) = child {
            match child_elem.name.local.as_str() {
                "Definition" => {
                    let text = child_elem
                        .children
                        .iter()
                        .map(|c| c.text_content())
                        .collect::<String>();
                    if !text.is_empty() {
                        definitions.push(text);
                    }
                }
                "Example" => {
                    let text = child_elem
                        .children
                        .iter()
                        .map(|c| c.text_content())
                        .collect::<String>();
                    if !text.is_empty() {
                        examples.push(text);
                    }
                }
                "SynsetRelation" => {
                    if let (Some(rel_type), Some(target)) = (
                        attr_value(child_elem, "relType"),
                        attr_value(child_elem, "target"),
                    ) {
                        relations.push(SynsetRelation {
                            rel_type: SynsetRelationType::parse(&rel_type),
                            target,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    Some(Synset {
        id,
        ili,
        pos,
        members,
        definitions,
        examples,
        relations,
    })
}

fn attr_value(
    elem: &crate::social::software::markup::xml::ontology::XmlElement,
    name: &str,
) -> Option<String> {
    elem.attributes
        .iter()
        .find(|a| a.name.local == name)
        .map(|a| a.value.clone())
}

#[derive(Debug)]
pub enum LmfReadError {
    Xml(String),
    Structure(String),
}

impl core::fmt::Display for LmfReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Xml(e) => write!(f, "XML error: {e}"),
            Self::Structure(e) => write!(f, "LMF structure error: {e}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LmfReadError {}
