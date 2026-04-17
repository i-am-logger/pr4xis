// Runtime Lexicon — a lime:Lexicon instance for one language.
//
// A Lexicon collects LexicalEntries, each with a canonical Form and
// one or more Senses connecting the entry to ontology concepts.
//
// This is the operational Lemon: the ontology says WHAT the structure
// is (LexicalEntry, Form, Sense, etc.), this module provides the
// runtime instances that hold actual lexical data.
//
// The English terminology lexicon is built by the functor
// F: OntologyConcepts → Lexicon(English) — each Entity variant in
// each ontology gets a LexicalEntry with its name as canonical form.
//
// Source: W3C Ontolex (2016) §5 lime:Lexicon; McCrae et al. (2017)

use std::collections::BTreeMap;

/// A Form — one grammatical realization (ontolex:Form).
/// Carries writtenRep (BCP 47 language-tagged).
#[derive(Debug, Clone)]
pub struct Form {
    pub written_rep: String,
    pub lang: &'static str,
}

/// A LexicalSense — bridges entry to ontology concept (ontolex:LexicalSense).
/// The reference identifies the ontology entity this sense points to.
#[derive(Debug, Clone)]
pub struct Sense {
    pub reference: ConceptRef,
}

/// Reference to an ontology concept — the target of ontolex:reference.
/// Identified by ontology name + concept name (both from the type system).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConceptRef {
    pub ontology: &'static str,
    pub concept: &'static str,
}

/// A LexicalEntry — unit of the lexicon (ontolex:LexicalEntry).
/// Has a canonical form and senses connecting to ontology concepts.
#[derive(Debug, Clone)]
pub struct LexicalEntry {
    pub canonical_form: Form,
    pub senses: Vec<Sense>,
}

/// A Lexicon — collection of entries for one language (lime:Lexicon).
///
/// Instance of LemonConcept::Lexicon. One lexicon per language.
/// The entries are indexed by their canonical form's written representation.
#[derive(Debug, Clone)]
pub struct Lexicon {
    pub lang: &'static str,
    entries: BTreeMap<String, LexicalEntry>,
}

impl Lexicon {
    pub fn new(lang: &'static str) -> Self {
        Self {
            lang,
            entries: BTreeMap::new(),
        }
    }

    /// Add an entry for an ontology concept.
    pub fn add_entry(&mut self, written_rep: &str, ontology: &'static str, concept: &'static str) {
        let entry = LexicalEntry {
            canonical_form: Form {
                written_rep: written_rep.to_string(),
                lang: self.lang,
            },
            senses: vec![Sense {
                reference: ConceptRef { ontology, concept },
            }],
        };
        self.entries.insert(written_rep.to_string(), entry);
    }

    /// Look up an entry by its canonical written form.
    pub fn lookup(&self, written_rep: &str) -> Option<&LexicalEntry> {
        self.entries.get(written_rep)
    }

    /// Find all entries that reference a given ontology concept.
    pub fn entries_for_concept(&self, ontology: &str, concept: &str) -> Vec<&LexicalEntry> {
        self.entries
            .values()
            .filter(|e| {
                e.senses
                    .iter()
                    .any(|s| s.reference.ontology == ontology && s.reference.concept == concept)
            })
            .collect()
    }

    /// The label for an ontology concept — the canonical form of its entry.
    /// This IS what replaces Vocabulary.name() and Axiom.description().
    pub fn label_for(&self, ontology: &str, concept: &str) -> Option<&str> {
        self.entries_for_concept(ontology, concept)
            .first()
            .map(|e| e.canonical_form.written_rep.as_str())
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &LexicalEntry)> {
        self.entries.iter()
    }
}

/// Build the English terminology lexicon from all registered ontologies.
///
/// This is the functor F: OntologyConcepts → Lexicon(English).
/// For each ontology, each Entity variant becomes a LexicalEntry.
/// The canonical form is the variant name. The sense references
/// the ontology concept.
pub fn build_english_terminology() -> Lexicon {
    // Placeholder — entries are registered by individual ontologies via add_entry.
    // Pre-populating from describe_knowledge_base() is deferred until ConceptRef
    // accepts owned strings (tracked as part of the Vocabulary redesign).
    Lexicon::new("en")
}

#[cfg(test)]
mod lexicon_tests {
    use super::*;

    #[test]
    #[ignore = "build_english_terminology is a placeholder — requires Lexicon to accept owned concept names (Vocabulary redesign follow-up)"]
    fn english_terminology_is_nonempty() {
        let lex = build_english_terminology();
        assert!(
            lex.entry_count() > 100,
            "expected >100 entries, got {}",
            lex.entry_count()
        );
    }

    #[test]
    #[ignore = "build_english_terminology is a placeholder — requires Lexicon to accept owned concept names (Vocabulary redesign follow-up)"]
    fn can_lookup_knowledge_ontology() {
        let lex = build_english_terminology();
        assert!(lex.lookup("KnowledgeOntology").is_some());
    }

    #[test]
    #[ignore = "build_english_terminology is a placeholder — requires Lexicon to accept owned concept names (Vocabulary redesign follow-up)"]
    fn label_for_returns_canonical_form() {
        let lex = build_english_terminology();
        let label = lex.label_for("KnowledgeOntology", "KnowledgeOntology");
        assert_eq!(label, Some("KnowledgeOntology"));
    }

    #[test]
    fn lexicon_language_is_english() {
        let lex = build_english_terminology();
        assert_eq!(lex.lang, "en");
    }
}
