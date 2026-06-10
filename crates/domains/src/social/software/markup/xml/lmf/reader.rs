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

    let lexicon_meta = read_lexicon_metadata(lexicon);

    let mut synsets = Vec::new();
    let mut entries = Vec::new();
    let mut syntactic_behaviours = Vec::new();

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
                // `SyntacticBehaviour*` at lexicon scope (DTD line 5) —
                // where Open English WordNet 2025 places its 39 frames.
                "SyntacticBehaviour" => {
                    if let Some(sb) = read_syntactic_behaviour(elem) {
                        syntactic_behaviours.push(sb);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(WordNet {
        lexicon: lexicon_meta,
        synsets,
        entries,
        syntactic_behaviours,
    })
}

/// Read the `<!ATTLIST Lexicon>` metadata (DTD lines 6-32) — the named
/// required/implied attrs plus the Dublin Core `dc:*` set (preserved in
/// declaration order under their prefixed names so the writer is a
/// faithful inverse).
fn read_lexicon_metadata(
    elem: &crate::social::software::markup::xml::ontology::XmlElement,
) -> LexiconMetadata {
    // The Dublin Core attrs, in the order the DTD declares them (lines
    // 16-29) — loaded from the DTD rather than hand-listed so the field
    // set is grounded in the schema, not encoded.
    let dc: Vec<(String, String)> = lexicon_dc_attr_names()
        .iter()
        .filter_map(|name| attr_value(elem, name).map(|v| ((*name).to_string(), v)))
        .collect();
    LexiconMetadata {
        id: attr_value(elem, "id"),
        label: attr_value(elem, "label"),
        language: attr_value(elem, "language"),
        email: attr_value(elem, "email"),
        license: attr_value(elem, "license"),
        version: attr_value(elem, "version"),
        url: attr_value(elem, "url"),
        citation: attr_value(elem, "citation"),
        logo: attr_value(elem, "logo"),
        status: attr_value(elem, "status"),
        confidence_score: attr_value(elem, "confidenceScore"),
        dc,
    }
}

/// The Dublin Core `dc:*` attribute names declared on `<!ATTLIST
/// Lexicon>`, in DTD declaration order (lines 16-29). The full list is
/// the standard `/dc` terms WN-LMF imports; the writer re-emits them in
/// this same order, so an absent attr round-trips as absent.
fn lexicon_dc_attr_names() -> &'static [&'static str] {
    &[
        "dc:contributor",
        "dc:coverage",
        "dc:creator",
        "dc:date",
        "dc:description",
        "dc:format",
        "dc:identifier",
        "dc:publisher",
        "dc:relation",
        "dc:rights",
        "dc:source",
        "dc:subject",
        "dc:title",
        "dc:type",
    ]
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
    let mut syntactic_behaviours = Vec::new();

    for child in &elem.children {
        if let XmlNode::Element(child_elem) = child {
            match child_elem.name.local.as_str() {
                "Lemma" => {
                    let written_form = attr_value(child_elem, "writtenForm")?;
                    let pos = LmfPos::parse(&attr_value(child_elem, "partOfSpeech")?);
                    lemma = Some(Lemma {
                        written_form,
                        pos,
                        script: attr_value(child_elem, "script"),
                        pronunciations: read_pronunciations(child_elem),
                    });
                }
                "Sense" => {
                    senses.push(read_sense(child_elem));
                }
                "Form" => {
                    if let Some(written_form) = attr_value(child_elem, "writtenForm") {
                        forms.push(Form {
                            written_form,
                            id: attr_value(child_elem, "id"),
                            script: attr_value(child_elem, "script"),
                            pronunciations: read_pronunciations(child_elem),
                        });
                    }
                }
                // `SyntacticBehaviour*` at entry scope (DTD line 33).
                "SyntacticBehaviour" => {
                    if let Some(sb) = read_syntactic_behaviour(child_elem) {
                        syntactic_behaviours.push(sb);
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
        syntactic_behaviours,
    })
}

/// Read a `<Sense>` (DTD lines 74-97): its `SenseRelation*`, `Count*`
/// children and the corpus-confirmed attrs (`subcat`, `adjposition`,
/// `dc:source`).
fn read_sense(elem: &crate::social::software::markup::xml::ontology::XmlElement) -> Sense {
    let id = attr_value(elem, "id").unwrap_or_default();
    let synset = attr_value(elem, "synset").unwrap_or_default();
    let subcat: Vec<String> = attr_value(elem, "subcat")
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_default();
    let mut relations = Vec::new();
    let mut counts = Vec::new();
    for sense_child in &elem.children {
        if let XmlNode::Element(child) = sense_child {
            match child.name.local.as_str() {
                "SenseRelation" => {
                    if let (Some(rel_type), Some(target)) =
                        (attr_value(child, "relType"), attr_value(child, "target"))
                    {
                        relations.push(SenseRelation {
                            rel_type: SenseRelationType::parse(&rel_type),
                            target,
                        });
                    }
                }
                // `<Count>` (#PCDATA) — corpus frequency (DTD line 233).
                "Count" => {
                    counts.push(Count {
                        value: text_content(child),
                    });
                }
                _ => {}
            }
        }
    }
    Sense {
        id,
        synset,
        relations,
        subcat,
        adjposition: attr_value(elem, "adjposition"),
        dc_source: attr_value(elem, "dc:source"),
        counts,
    }
}

/// Read the `<Pronunciation>` children of a `<Lemma>` or `<Form>` (DTD
/// lines 63-69) — the transcription text plus the declared attrs.
fn read_pronunciations(
    elem: &crate::social::software::markup::xml::ontology::XmlElement,
) -> Vec<Pronunciation> {
    let mut out = Vec::new();
    for child in &elem.children {
        if let XmlNode::Element(p) = child
            && p.name.local == "Pronunciation"
        {
            out.push(Pronunciation {
                text: text_content(p),
                variety: attr_value(p, "variety"),
                notation: attr_value(p, "notation"),
                phonemic: attr_value(p, "phonemic"),
                audio: attr_value(p, "audio"),
            });
        }
    }
    out
}

/// Read a `<SyntacticBehaviour>` (EMPTY, DTD lines 228-232): the
/// `subcategorizationFrame` (required), the optional `id`, and the
/// whitespace-separated `senses` IDREFS.
fn read_syntactic_behaviour(
    elem: &crate::social::software::markup::xml::ontology::XmlElement,
) -> Option<SyntacticBehaviour> {
    let subcategorization_frame = attr_value(elem, "subcategorizationFrame")?;
    let senses: Vec<String> = attr_value(elem, "senses")
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_default();
    Some(SyntacticBehaviour {
        id: attr_value(elem, "id"),
        subcategorization_frame,
        senses,
    })
}

/// Collect an element's direct `#PCDATA` text content.
fn text_content(elem: &crate::social::software::markup::xml::ontology::XmlElement) -> String {
    elem.children
        .iter()
        .map(|c| c.text_content())
        .collect::<String>()
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
    let mut ili_definition = None;
    let mut examples = Vec::new();
    let mut relations = Vec::new();

    for child in &elem.children {
        if let XmlNode::Element(child_elem) = child {
            match child_elem.name.local.as_str() {
                "Definition" => {
                    let text = text_content(child_elem);
                    if !text.is_empty() {
                        definitions.push(text);
                    }
                }
                // `ILIDefinition?` (DTD lines 98, 145) — at most one; the
                // Interlingual Index gloss. Captured whether or not it is
                // empty (an empty ILIDefinition is still a present element,
                // unlike Definition which the reader filters when empty for
                // back-compat).
                "ILIDefinition" => {
                    ili_definition = Some(text_content(child_elem));
                }
                "Example" => {
                    let text = text_content(child_elem);
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
        ili_definition,
        examples,
        relations,
        lexfile: attr_value(elem, "lexfile"),
        dc_source: attr_value(elem, "dc:source"),
        confidence_score: attr_value(elem, "confidenceScore"),
    })
}

/// Look up an attribute by its QUALIFIED name (the prefixed form, e.g.
/// `dc:source`). The XML parser splits a prefixed attribute name into
/// `prefix`/`local` (W3C XML 1.0 §3.1), so matching on
/// [`XmlName::qualified`](crate::social::software::markup::xml::ontology::XmlName::qualified)
/// finds both bare (`writtenForm`) and prefixed (`dc:source`) attrs;
/// matching on `local` alone would confuse `dc:source` with a bare
/// `source`. The writer rebuilds the same qualified name so the
/// attribute round-trips.
fn attr_value(
    elem: &crate::social::software::markup::xml::ontology::XmlElement,
    name: &str,
) -> Option<String> {
    elem.attributes
        .iter()
        .find(|a| a.name.qualified() == name)
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
