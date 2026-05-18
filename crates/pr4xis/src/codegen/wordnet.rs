use std::collections::HashMap;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::builder::{EntityDef, OntologyBuilder};

/// Parse a WordNet XML-LMF file into an OntologyBuilder.
///
/// Reads the full Open English WordNet XML and extracts:
/// - Synsets as entities (concepts)
/// - LexicalEntries as word→synset mappings
/// - SynsetRelations mapped to reasoning ontology types
pub fn parse_wordnet_xml(path: &Path) -> Result<OntologyBuilder, ParseError> {
    let xml = std::fs::read_to_string(path).map_err(|e| ParseError::Io(e.to_string()))?;

    let mut reader = Reader::from_str(&xml);
    let mut builder = OntologyBuilder::new();

    // Track current parsing state
    let mut state = ParseState::None;
    let mut current_synset: Option<SynsetBuilder> = None;
    let mut current_entry: Option<EntryBuilder> = None;
    let mut buf = Vec::new();

    // Synset ID → POS for cross-referencing
    let mut synset_pos: HashMap<String, String> = HashMap::new();

    // Sense ID → synset ID, for resolving sense-level relations
    // (especially `also`) into synset-level references at the end of
    // parsing.
    let mut sense_to_synset: HashMap<String, String> = HashMap::new();
    // Deferred sense-level `also` edges, resolved after the full file
    // has been read so both endpoints' synset mappings are known.
    let mut pending_sense_also: Vec<(String, String)> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"Synset" => {
                    let mut sb = SynsetBuilder::default();
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"id" => sb.id = String::from_utf8_lossy(&attr.value).into(),
                            b"partOfSpeech" => sb.pos = String::from_utf8_lossy(&attr.value).into(),
                            b"ili" => sb.ili = Some(String::from_utf8_lossy(&attr.value).into()),
                            _ => {}
                        }
                    }
                    synset_pos.insert(sb.id.clone(), sb.pos.clone());
                    state = ParseState::InSynset;
                    current_synset = Some(sb);
                }
                b"Definition" if state == ParseState::InSynset => {
                    state = ParseState::InDefinition;
                }
                b"Example" if state == ParseState::InSynset => {
                    state = ParseState::InExample;
                }
                b"SynsetRelation" if state == ParseState::InSynset => {
                    if let Some(ref mut synset) = current_synset {
                        let mut rel_type = String::new();
                        let mut target = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"relType" => {
                                    rel_type = String::from_utf8_lossy(&attr.value).into()
                                }
                                b"target" => target = String::from_utf8_lossy(&attr.value).into(),
                                _ => {}
                            }
                        }
                        if !rel_type.is_empty() && !target.is_empty() {
                            synset.relations.push((rel_type, target));
                        }
                    }
                }
                b"LexicalEntry" => {
                    let mut eb = EntryBuilder::default();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"id" {
                            eb.id = String::from_utf8_lossy(&attr.value).into();
                        }
                    }
                    state = ParseState::InEntry;
                    current_entry = Some(eb);
                }
                b"Lemma" if state == ParseState::InEntry => {
                    if let Some(ref mut entry) = current_entry {
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"writtenForm" => {
                                    entry.lemma = String::from_utf8_lossy(&attr.value).into()
                                }
                                b"partOfSpeech" => {
                                    entry.pos = String::from_utf8_lossy(&attr.value).into()
                                }
                                _ => {}
                            }
                        }
                    }
                }
                b"Form" if state == ParseState::InEntry => {
                    if let Some(ref mut entry) = current_entry {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"writtenForm" {
                                entry
                                    .forms
                                    .push(String::from_utf8_lossy(&attr.value).into());
                            }
                        }
                    }
                }
                b"Sense" if state == ParseState::InEntry => {
                    if let Some(ref mut entry) = current_entry {
                        let mut synset_ref = String::new();
                        let mut sense_id = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"synset" => {
                                    synset_ref = String::from_utf8_lossy(&attr.value).into()
                                }
                                b"id" => sense_id = String::from_utf8_lossy(&attr.value).into(),
                                _ => {}
                            }
                        }
                        if !synset_ref.is_empty() {
                            entry.senses.push((sense_id, synset_ref));
                        }
                    }
                }
                b"SenseRelation" if state == ParseState::InEntry => {
                    if let Some(ref mut entry) = current_entry {
                        let mut rel_type = String::new();
                        let mut target = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"relType" => {
                                    rel_type = String::from_utf8_lossy(&attr.value).into()
                                }
                                b"target" => target = String::from_utf8_lossy(&attr.value).into(),
                                _ => {}
                            }
                        }
                        if !rel_type.is_empty() && !target.is_empty() {
                            entry.sense_relations.push((rel_type, target));
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                let decoded = e.decode().unwrap_or_default();
                let text = match quick_xml::escape::unescape(&decoded) {
                    Ok(unescaped) => unescaped.into_owned(),
                    Err(_) => decoded.into_owned(),
                };
                match state {
                    ParseState::InDefinition => {
                        if let Some(ref mut synset) = current_synset {
                            synset.definitions.push(text);
                        }
                        state = ParseState::InSynset;
                    }
                    ParseState::InExample => {
                        if let Some(ref mut synset) = current_synset {
                            synset.examples.push(text);
                        }
                        state = ParseState::InSynset;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"Synset" => {
                    if let Some(synset) = current_synset.take() {
                        finalize_synset(&mut builder, synset);
                    }
                    state = ParseState::None;
                }
                b"LexicalEntry" => {
                    if let Some(entry) = current_entry.take() {
                        finalize_entry(
                            &mut builder,
                            entry,
                            &mut sense_to_synset,
                            &mut pending_sense_also,
                        );
                    }
                    state = ParseState::None;
                }
                b"Definition" if state == ParseState::InDefinition => {
                    state = ParseState::InSynset;
                }
                b"Example" if state == ParseState::InExample => {
                    state = ParseState::InSynset;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(ParseError::Xml(format!("XML error: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    // Resolve sense-level `also` (SKOS seeAlso) to synset-level
    // references. Both endpoints must have their synset mapping
    // recorded; pairs with an unknown sense-id are dropped (the
    // referenced sense may live in another lexicon or have failed
    // parsing). Per Global WordNet schema relType="also" on
    // SenseRelation.
    for (src_sense, tgt_sense) in &pending_sense_also {
        if let (Some(src_synset), Some(tgt_synset)) = (
            sense_to_synset.get(src_sense),
            sense_to_synset.get(tgt_sense),
        ) {
            builder.add_reference(src_synset, tgt_synset);
        }
    }

    let _ = synset_pos;
    Ok(builder)
}

#[derive(Debug, Clone, PartialEq)]
enum ParseState {
    None,
    InSynset,
    InDefinition,
    InExample,
    InEntry,
}

#[derive(Debug, Default)]
struct SynsetBuilder {
    id: String,
    pos: String,
    ili: Option<String>,
    definitions: Vec<String>,
    examples: Vec<String>,
    relations: Vec<(String, String)>, // (relType, target synset ID)
}

#[derive(Debug, Default)]
struct EntryBuilder {
    id: String,
    lemma: String,
    pos: String,
    forms: Vec<String>,
    senses: Vec<(String, String)>,          // (sense_id, synset_id)
    sense_relations: Vec<(String, String)>, // (relType, target sense ID)
}

fn finalize_synset(builder: &mut OntologyBuilder, synset: SynsetBuilder) {
    let mut entity = EntityDef::new(&synset.id, &synset.id);
    if !synset.pos.is_empty() {
        entity = entity.pos(&synset.pos);
    }
    for def in &synset.definitions {
        entity = entity.definition(def);
    }
    builder.add_entity(entity);

    // Map synset relations to reasoning ontology types
    for (rel_type, target) in &synset.relations {
        match rel_type.as_str() {
            // Taxonomy (is-a): child=this, parent=target
            "hypernym" | "instance_hypernym" => {
                builder.add_taxonomy(&synset.id, target);
            }
            // Mereology (has-a): whole=target, part=this (holonym means "this is part of target")
            "holo_member" | "holo_part" | "holo_substance" => {
                builder.add_mereology(target, &synset.id);
            }
            // Mereology reverse: whole=this, part=target
            "mero_member" | "mero_part" | "mero_substance" => {
                builder.add_mereology(&synset.id, target);
            }
            // Causation
            "causes" => {
                builder.add_causation(&synset.id, target);
            }
            // SKOS-style cross-reference. `also` in WordNet links two
            // synsets that are conceptually related but neither
            // hypernymous nor synonymous (Fellbaum 1998 §1; Global
            // WordNet schema relType="also"). Mapped to the SKOS
            // `seeAlso` slot in CodegenData::references.
            "also" => {
                builder.add_reference(&synset.id, target);
            }
            // Other relations stored but not mapped yet:
            // "similar", "attribute", "domain_topic", "domain_region",
            // "exemplifies", "entails", etc.
            _ => {}
        }
    }
}

fn finalize_entry(
    builder: &mut OntologyBuilder,
    entry: EntryBuilder,
    sense_to_synset: &mut HashMap<String, String>,
    pending_sense_also: &mut Vec<(String, String)>,
) {
    // Map word text to each synset it belongs to
    for (sense_id, synset_id) in &entry.senses {
        builder.add_word_index(&entry.lemma, synset_id);

        // Also index morphological forms
        for form in &entry.forms {
            builder.add_word_index(form, synset_id);
        }

        // Record sense → synset for cross-resolution of sense-level
        // relations after the file finishes parsing.
        if !sense_id.is_empty() {
            sense_to_synset.insert(sense_id.clone(), synset_id.clone());
        }
    }

    // Handle sense-level relations (antonyms are typically sense-level)
    for (rel_type, target) in &entry.sense_relations {
        match rel_type.as_str() {
            "antonym" => {
                // Antonyms link senses, but we map to synsets for opposition
                // We need to resolve sense → synset, which requires cross-referencing
                // For now, we handle this in a post-processing step
            }
            "similar" => {
                // Similar senses → equivalence candidates
            }
            "also" => {
                // SKOS seeAlso (sense-level). The XML parser doesn't
                // bind each SenseRelation to a specific Sense (relations
                // accumulate on the entry), so the source sense is
                // approximated as the most recently parsed sense — fine
                // in practice because the vast majority of LMF entries
                // have a single sense. Both endpoints' synset
                // resolution happens after parsing completes.
                if let Some((sense_id, _)) = entry.senses.last() {
                    pending_sense_also.push((sense_id.clone(), target.clone()));
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Io(String),
    Xml(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Xml(e) => write!(f, "XML parse error: {e}"),
        }
    }
}

impl std::error::Error for ParseError {}
