//! A definitional lexicon naming the U.S. Code titles this deployment ACTUALLY
//! HOLDS — the residency surface that makes "what is title 42" answerable at
//! all.
//!
//! # The gap this closes
//!
//! A definitional answer is reached through an ENGLISH ATOM: the composed
//! reasoner indexes every `ontolex:Form` surface a loaded concept points at,
//! and "what is X" resolves when X is one of those surfaces
//! (`ComposedReasoner::new`'s Form-atom walk). `respite care` answers because
//! `caregiving_lexicon` is a WN-LMF lexicon, so the bridge minted a Form atom
//! for that exact two-word surface — multiword is no obstacle. A U.S. Code
//! TITLE had no atom of any kind: the corpus contributes one node per SECTION,
//! and nothing names the title itself, so "what is title 42" fell through to
//! the bare numeral and no lookup could fire.
//!
//! # What it contributes, and from where
//!
//! One synset per held title, glossed by that title's OWN registered
//! `description` (already data, already hash-pinned in the registry `.prx`),
//! lexicalized by every form
//! [`UsCodeTitleCitationForm::ALL`](crate::social::software::markup::xml::uslm::corpus::identifiers::UsCodeTitleCitationForm::ALL)
//! publishes. No citation syntax and no title's prose appear here: the surfaces
//! come from
//! [`UsCodeTitleId::citations`](crate::social::software::markup::xml::uslm::corpus::identifiers::UsCodeTitleId::citations)
//! and the gloss from `RegistryEntry::description`, so registering a tenth
//! title makes it answerable with no change to this file.
//!
//! # HELD, not registered
//!
//! The set is derived from LOADED CONTENT (`loaded_titles` reads the
//! materialized corpus's own section URNs), never from the registry's list of
//! titles it knows OF. Synthesizing an entry for a title the deployment does
//! not hold would let the engine answer about knowledge it does not have — the
//! precise dishonesty the abstention machinery exists to prevent — and it would
//! do so in the most convincing possible register, a confident definition.
//!
//! # Why WN-LMF and not a hand-built archive
//!
//! The lexicon is handed to the SAME generic bridge every registered
//! definitional lexicon rides
//! ([`lexicon_runtime_ontology_from_lmf`](crate::cognitive::linguistics::english::bridge::lexicon_runtime_ontology_from_lmf))
//! — `read_wordnet →
//! English::from_wordnet → project_lexicon_archive → apply(english_functor) →
//! materialize`. There is exactly one way a gloss becomes a chattable concept
//! in this engine, and a derived lexicon has no business being the second one.
//! The document is built as the typed WN-LMF ontology and serialized by WN-LMF's
//! own writer, so no markup is written down here either.

#[allow(unused_imports)]
use alloc::{collections::BTreeSet, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::ontology::RuntimeOntology;

use super::ontology::RegistryEntry;
use super::registry::data_sources;
use crate::cognitive::linguistics::english::bridge::{
    LexiconBridgeError, lexicon_runtime_ontology_from_lmf,
};
use crate::social::software::markup::xml::lmf::ontology::{
    Lemma, LexicalEntry, LexiconMetadata, LmfPos, Sense, Synset, WordNet,
};
use crate::social::software::markup::xml::lmf::writer::write_wordnet_document;
use crate::social::software::markup::xml::parser::serializer::serialize_document;
use crate::social::software::markup::xml::uslm::corpus::UsCode;
use crate::social::software::markup::xml::uslm::corpus::identifiers::{
    UsCodeTitleCitationForm, UsCodeTitleId,
};
use crate::social::software::markup::xml::uslm::title_number_of_urn;

/// The [`OntologyName`] the derived title lexicon materializes under.
///
/// Deliberately NOT a registered source name: this ontology has no fetchable
/// bytes and no hash pin of its own — it is composed at load time from sources
/// that do (the registry descriptions and the loaded corpus). Naming it after
/// a registry key would claim a provenance it does not have.
pub const USC_TITLE_LEXICON_NAME: &str = "usc_titles";

/// The BCP-47 language tag of the glosses — they are the registry's English
/// prose, and WN-LMF `<Lexicon language>` is where a lexicon states that
/// (WN-LMF 1.3 DTD line 9).
const LEXICON_LANGUAGE: &str = "en";

/// The U.S. Code titles the MATERIALIZED corpus actually holds.
///
/// Derived from the loaded sections' own URNs — `/us/usc/t42/s1983` is Title 42
/// being present, in the only way presence is observable — then lifted through
/// [`UsCodeTitleId::try_from_number`] so the result is typed and range-checked
/// (1 U.S.C. § 204). Deduplicated and returned in title-number order, so the
/// derived lexicon's content address does not depend on section load order.
///
/// This is emphatically not `registered_titles()`: the corpus skips a
/// registered title whose archive is absent on disk, and a deployment that
/// registers nine titles may hold one.
pub fn loaded_titles(usc: &UsCode) -> Vec<UsCodeTitleId> {
    usc.all_sections()
        .iter()
        .filter_map(|section| section.title_number())
        .collect::<BTreeSet<u32>>()
        .into_iter()
        .filter_map(|number| UsCodeTitleId::try_from_number(number).ok())
        .collect()
}

/// The U.S. Code titles held as INDEPENDENTLY LOADED ontologies, named by the
/// registry source name their bytes were published under.
///
/// The wasm host's shape of the same question [`loaded_titles`] answers for the
/// CLI. The derivation differs because the HOLDING differs, not because the
/// predicate is looser: the CLI merges every title into one `UsCode`, so the
/// only observation of presence is a section URN; the browser installs each
/// title as its own [`RuntimeOntology`] named `usc_title_42`, so the only
/// observation of presence is that name. Both read loaded content; neither
/// reads the registry's list of titles it merely knows OF.
///
/// A name that is not a U.S. Code title source key — an uploaded `.prx`, a
/// demo ontology, a test fixture — yields nothing, so a host cannot accidentally
/// mint an entry for something that is not a title.
pub fn titles_held_under_source_names<'a>(
    names: impl Iterator<Item = &'a str>,
) -> Vec<UsCodeTitleId> {
    names
        .filter_map(|name| UsCodeTitleId::try_from_source_name(name).ok())
        .map(|id| id.number())
        .collect::<BTreeSet<u32>>()
        .into_iter()
        .filter_map(|number| UsCodeTitleId::try_from_number(number).ok())
        .collect()
}

/// The titles held, read from the CONTENT rather than from what the ontologies
/// are called.
///
/// [`titles_held_under_source_names`] trusts the name a caller supplied, which
/// makes the engine's answer about a title a claim about a LABEL. Load Title 18's
/// USLM under the name `usc_title_5` and that version would have the engine
/// recite Title 5's registered gloss — a confident definition of a document it
/// does not hold, which is the one failure this codebase exists to prevent.
///
/// Every node a USC projection emits is named by its own USLM URN, so the title
/// a corpus actually contains is derivable from it: `title_number_of_urn` reads
/// `/us/usc/t42/...` back to 42. That is the same derivation the CLI already
/// performs over section URNs, so both hosts now answer the question the same
/// way and neither takes a name on trust.
pub fn titles_held_in<'a>(
    ontologies: impl Iterator<Item = &'a RuntimeOntology>,
) -> Vec<UsCodeTitleId> {
    ontologies
        .flat_map(|onto| {
            onto.archive()
                .nodes
                .iter()
                .filter_map(|node| title_number_of_urn(node.name.as_ref()))
                .collect::<Vec<u32>>()
        })
        .collect::<BTreeSet<u32>>()
        .into_iter()
        .filter_map(|number| UsCodeTitleId::try_from_number(number).ok())
        .collect()
}

/// Join the held titles against the registry, keeping only those that carry a
/// gloss to recite.
///
/// A registered `description` is `Option` (praxis.toml may omit it), and the
/// only honest response to its absence is to contribute NO entry: a synset with
/// no `<Definition>` is a name the engine can recognize but cannot explain,
/// which reads to a user as knowledge and answers as silence. Never synthesize
/// prose to fill the hole.
fn glossed_titles(loaded: &[UsCodeTitleId]) -> Vec<(&UsCodeTitleId, &'static str)> {
    loaded
        .iter()
        .filter_map(|id| {
            let source_name = id.source_name();
            let entry: &RegistryEntry = data_sources().iter().find(|e| e.name == source_name)?;
            let gloss = entry.description.as_deref()?;
            (!gloss.is_empty()).then_some((id, gloss))
        })
        .collect()
}

/// The typed WN-LMF lexicon for the held, glossed titles — one `Synset` per
/// title, one `LexicalEntry` per published citation form of it.
///
/// Identifier scheme, all derived so nothing collides and nothing is invented:
/// - the synset id is the title's canonical USLM URN (`/us/usc/t42`), the
///   identifier the LRC itself publishes it under. It is also, usefully, never
///   equal to any written form — `project_lexicon_archive` drops a
///   lexicalization edge whose form equals its concept's name, so a synset id
///   that collided with a surface would silently lose that surface;
/// - the entry id is that URN plus the form's ordinal, and the sense id the
///   entry id plus its own — the `cg-e-respite-care` / `cg-e-respite-care-1`
///   shape the shipped caregiving lexicon uses. These are internal document
///   ids, never surfaced.
///
/// Every lemma is a noun: a title citation is a proper nominal ("title 42 of
/// the United States Code" heads an NP in the enacted text it is quoted from),
/// which is the `n` tag of the WN-LMF `partOfSpeech` enumeration (DTD line 57).
pub fn usc_title_lexicon_wordnet(loaded: &[UsCodeTitleId]) -> WordNet {
    let glossed = glossed_titles(loaded);
    let mut synsets = Vec::with_capacity(glossed.len());
    let mut entries = Vec::with_capacity(glossed.len() * UsCodeTitleCitationForm::ALL.len());

    for (id, gloss) in glossed {
        let synset_id = id.urn().to_string();
        for (ordinal, written_form) in id.citations().enumerate() {
            let entry_id = format!("{synset_id}-e{ordinal}");
            let sense_id = format!("{entry_id}-1");
            entries.push(LexicalEntry {
                id: entry_id,
                lemma: Lemma {
                    written_form,
                    pos: LmfPos::Noun,
                    script: None,
                    pronunciations: Vec::new(),
                },
                senses: vec![Sense {
                    id: sense_id,
                    synset: synset_id.clone(),
                    relations: Vec::new(),
                    subcat: Vec::new(),
                    adjposition: None,
                    dc_source: None,
                    counts: Vec::new(),
                }],
                forms: Vec::new(),
                syntactic_behaviours: Vec::new(),
            });
        }
        synsets.push(Synset {
            id: synset_id,
            ili: None,
            pos: LmfPos::Noun,
            members: Vec::new(),
            definitions: vec![gloss.to_string()],
            ili_definition: None,
            examples: Vec::new(),
            relations: Vec::new(),
            lexfile: None,
            dc_source: None,
            confidence_score: None,
        });
    }

    WordNet {
        // Only what is true is carried. The lexicon has an id and a language;
        // it has no upstream `version`, `license` or `url`, because it is
        // composed here rather than published anywhere — and an absent
        // `#IMPLIED` attribute round-trips as absent (see `LexiconMetadata`).
        lexicon: LexiconMetadata {
            id: Some(USC_TITLE_LEXICON_NAME.to_string()),
            language: Some(LEXICON_LANGUAGE.to_string()),
            ..LexiconMetadata::default()
        },
        synsets,
        entries,
        syntactic_behaviours: Vec::new(),
    }
}

/// Materialize the held titles as a chat-composable [`RuntimeOntology`].
///
/// `None` when no held title carries a gloss — a deployment with no corpus, or
/// one whose titles registered no `description`, contributes nothing rather
/// than an empty ontology that would occupy a row in every catalog and answer
/// nothing.
///
/// The typed lexicon is serialized by WN-LMF's own writer and handed to the ONE
/// generic bridge, so escaping, element order and the `<Definition>` text are
/// the writer's concern and not this module's — a gloss containing `&`, `<` or
/// a newline is handled by the serializer that already round-trips the 89 MB
/// upstream WordNet.
pub fn usc_title_lexicon(
    loaded: &[UsCodeTitleId],
) -> Option<Result<RuntimeOntology, LexiconBridgeError>> {
    let wn = usc_title_lexicon_wordnet(loaded);
    if wn.synsets.is_empty() {
        return None;
    }
    let document = write_wordnet_document(&wn);
    let bytes = serialize_document(&document);
    let xml = String::from_utf8(bytes)
        .expect("the WN-LMF serializer emits UTF-8 (its input is Rust `String`s)");
    Some(lexicon_runtime_ontology_from_lmf(
        &xml,
        OntologyName::new(USC_TITLE_LEXICON_NAME.to_string()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    /// Every title this deployment REGISTERS, as the held set would be if the
    /// whole corpus were on disk. Used only to exercise the join; the
    /// production predicate is [`loaded_titles`], which reads the corpus.
    fn all_registered() -> Vec<UsCodeTitleId> {
        crate::social::software::markup::xml::uslm::corpus::identifiers::registered_titles()
            .collect()
    }

    fn title(number: u32) -> UsCodeTitleId {
        UsCodeTitleId::try_from_number(number).expect("a valid title number")
    }

    /// The surfaces a held title becomes answerable by are EXACTLY its own
    /// published citation forms — every leaf of `UsCodeTitleCitationForm::ALL`,
    /// obtained from the typed id. The test asserts set equality against
    /// `id.citations()` rather than against written-out strings, so it cannot
    /// pass by agreeing with a literal this module also wrote.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn surfaces_are_exactly_the_titles_own_citation_forms() {
        let held = vec![title(42)];
        let wn = usc_title_lexicon_wordnet(&held);

        let minted: BTreeSet<String> = wn
            .entries
            .iter()
            .map(|e| e.lemma.written_form.clone())
            .collect();
        let published: BTreeSet<String> = held[0].citations().collect();
        assert_eq!(
            minted, published,
            "every published citation form of the title, and nothing else, is a surface"
        );
        assert_eq!(
            published.len(),
            UsCodeTitleCitationForm::ALL.len(),
            "the forms are distinct, so the enumeration's size is the surface count"
        );
        // The bare designation is the phrasing that failed in chat; assert it
        // is genuinely among them, and that it came from the enumeration.
        assert!(
            minted.contains(&held[0].citation(UsCodeTitleCitationForm::Designation)),
            "the bare designation must be a surface — it is what a reader types"
        );
    }

    /// The gloss is the registry's own `description`, carried verbatim. A
    /// derived lexicon that paraphrased its source would be asserting something
    /// no source says.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn gloss_is_the_registered_description_verbatim() {
        let held = all_registered();
        assert!(
            !held.is_empty(),
            "precondition: this deployment registers at least one U.S. Code title"
        );
        let wn = usc_title_lexicon_wordnet(&held);
        for synset in &wn.synsets {
            let id = UsCodeTitleId::try_from_urn(synset.id.clone()).expect("synset id is the URN");
            let entry = data_sources()
                .iter()
                .find(|e| e.name == id.source_name())
                .expect("a held title is a registered source");
            assert_eq!(
                synset.definitions,
                vec![entry.description.clone().expect("gloss present")],
                "the gloss for {} is its registry description, unedited",
                id.source_name()
            );
        }
    }

    /// THE HONESTY PROPERTY: a title this deployment does not hold contributes
    /// no synset, no surface, and therefore no answer.
    ///
    /// Title 7 (Agriculture) is a real U.S. Code title that is neither
    /// registered nor loaded here. The check is structural — the held set is
    /// the input, so nothing outside it can appear in the output — and is run
    /// against a held set of exactly one title to prove the other eight
    /// REGISTERED titles are absent too. Registration is knowing OF a title;
    /// only loading is holding it.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_title_that_is_not_held_is_not_answerable() {
        let held = vec![title(42)];
        let wn = usc_title_lexicon_wordnet(&held);

        assert_eq!(wn.synsets.len(), 1, "exactly the one held title is glossed");
        let surfaces: BTreeSet<String> = wn
            .entries
            .iter()
            .map(|e| e.lemma.written_form.clone())
            .collect();

        // Title 7: real, unregistered, unheld.
        for form in title(7).citations() {
            assert!(
                !surfaces.contains(&form),
                "an unregistered title must contribute no surface, but {form:?} appeared"
            );
        }
        // Title 18: registered here, but NOT in the held set for this call.
        // Registration must not be mistaken for possession.
        for form in title(18).citations() {
            assert!(
                !surfaces.contains(&form),
                "a registered-but-unheld title must contribute no surface, but {form:?} appeared"
            );
        }
        assert!(
            usc_title_lexicon(&[]).is_none(),
            "holding no title contributes no ontology at all"
        );
    }

    /// The emitted document is well-formed WN-LMF that the reader recovers the
    /// same lexicon from — the precondition for the generic bridge accepting
    /// it. Exercises the two shapes that could silently break: a gloss carrying
    /// XML-significant characters, and the em-dash/`§` UTF-8 the real registry
    /// descriptions contain.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn the_emitted_document_round_trips_through_the_wn_lmf_reader() {
        let wn = usc_title_lexicon_wordnet(&all_registered());
        let bytes = serialize_document(&write_wordnet_document(&wn));
        let xml = String::from_utf8(bytes).expect("UTF-8");
        let reread = read_wordnet(&xml).expect("the emitted document is well-formed WN-LMF");
        assert_eq!(reread.synsets, wn.synsets, "synsets survive serialization");
        assert_eq!(reread.entries, wn.entries, "entries survive serialization");
        // Not vacuous: the real descriptions carry `§` and an em-dash, and the
        // reader records a `<Definition>` only when its text is non-empty.
        assert!(
            reread
                .synsets
                .iter()
                .all(|s| s.definitions.iter().all(|d| !d.trim().is_empty())),
            "every gloss survives as non-empty text"
        );
    }

    /// Ids are unique across the whole document. WN-LMF ids key maps in
    /// `English::from_wordnet`, so a collision would silently drop an entry
    /// rather than fail — worth gating even though the derivation makes it
    /// structurally impossible.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_minted_identifier_is_unique() {
        let wn = usc_title_lexicon_wordnet(&all_registered());
        let mut ids: BTreeSet<&str> = BTreeSet::new();
        for synset in &wn.synsets {
            assert!(ids.insert(&synset.id), "duplicate synset id {}", synset.id);
        }
        for entry in &wn.entries {
            assert!(ids.insert(&entry.id), "duplicate entry id {}", entry.id);
            for sense in &entry.senses {
                assert!(ids.insert(&sense.id), "duplicate sense id {}", sense.id);
                assert!(
                    wn.synsets.iter().any(|s| s.id == sense.synset),
                    "sense {} names a declared synset",
                    sense.id
                );
                // The lexicalization edge is dropped when a form equals its
                // concept's name (`project_lexicon_archive`), so a surface that
                // collided with the synset id would be silently unreachable.
                assert!(
                    !entry.lemma.written_form.eq_ignore_ascii_case(&sense.synset),
                    "surface {:?} must not collide with its synset id",
                    entry.lemma.written_form
                );
            }
        }
    }

    /// The held set comes from loaded CONTENT: `loaded_titles` reads the
    /// corpus's own section URNs. Against the real materialized corpus, every
    /// title it reports must be one whose sections are genuinely present, and
    /// the count must match the distinct title numbers those URNs carry.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn held_titles_are_derived_from_loaded_sections() {
        let usc = crate::social::software::markup::xml::uslm::corpus::loaded();
        let held = loaded_titles(usc);
        let distinct: BTreeSet<u32> = usc
            .all_sections()
            .iter()
            .filter_map(|s| s.title_number())
            .collect();
        assert_eq!(
            held.iter().map(|id| id.number()).collect::<BTreeSet<u32>>(),
            distinct,
            "the held set is exactly the distinct title numbers the loaded sections carry"
        );
        assert!(
            held.windows(2).all(|w| w[0].number() < w[1].number()),
            "held titles are deduplicated and in title-number order"
        );
    }
}
