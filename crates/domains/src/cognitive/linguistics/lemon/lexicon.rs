#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

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

use alloc::collections::BTreeMap;

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// A Form — one grammatical realization (ontolex:Form).
/// Carries writtenRep (BCP 47 language-tagged).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Form {
    pub written_rep: String,
    pub lang: String,
}

/// A LexicalSense — bridges entry to ontology concept (ontolex:LexicalSense).
/// The `reference` identifies the ontology entity this sense points to; the
/// optional `domain` (dct:subject) marks the specialized register under which
/// this sense is the predominant reading.
///
/// Two senses on one entry — a general (`domain == None`) and a domain-specific
/// one — is the OntoLex one-entry-many-senses model (McCrae et al. 2017): the
/// word "person" carries its general sense AND a `"legal"`-domain sense on the
/// SAME shared entry, and which is predominant is re-ranked per query domain
/// (Koeling, McCarthy & Carroll 2005: the predominant sense is domain-dependent).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sense {
    pub reference: ConceptRef,
    pub domain: Option<String>,
}

impl Sense {
    /// A general, domain-unmarked sense.
    pub fn new(reference: ConceptRef) -> Self {
        Self {
            reference,
            domain: None,
        }
    }

    /// A sense predominant within `domain` (dct:subject) — e.g. the legal
    /// reading of a term a statute defines.
    pub fn in_domain(reference: ConceptRef, domain: impl Into<String>) -> Self {
        Self {
            reference,
            domain: Some(domain.into()),
        }
    }

    /// Domain-conditioned salience for a query `domain` (Koeling, McCarthy &
    /// Carroll 2005). A sense whose domain matches the query is most salient
    /// (elevated); a general (unmarked) sense is the default fall-through; an
    /// other-domain sense is least salient here. Higher = more predominant.
    pub fn salience_in(&self, domain: Option<&str>) -> Quantity {
        let salience = match (self.domain.as_deref(), domain) {
            (Some(d), Some(q)) if d == q => 2.0,
            (None, _) => 1.0,
            (Some(_), _) => 0.0,
        };
        Quantity::from_unit(salience, &unit::UNITLESS)
    }
}

/// Reference to an ontology concept — the target of ontolex:reference.
/// Identified by ontology name + concept name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConceptRef {
    pub ontology: String,
    pub concept: String,
}

/// A LexicalEntry — unit of the lexicon (ontolex:LexicalEntry).
/// Has a canonical form and senses connecting to ontology concepts.
#[derive(Debug, Clone)]
pub struct LexicalEntry {
    pub canonical_form: Form,
    pub senses: Vec<Sense>,
}

impl LexicalEntry {
    /// The predominant sense for a query `domain` — the OntoLex / Koeling,
    /// McCarthy & Carroll (2005) domain-dependent default reading. The most
    /// salient sense wins (a domain match outranks the general sense, which
    /// outranks an other-domain sense); ties resolve to source order (stable).
    /// `None` only for an entry that carries no senses.
    pub fn predominant_sense(&self, domain: Option<&str>) -> Option<&Sense> {
        self.senses
            .iter()
            .enumerate()
            .max_by(|(i1, s1), (i2, s2)| {
                s1.salience_in(domain)
                    .partial_cmp(&s2.salience_in(domain))
                    .expect("salience is unitless and always comparable")
                    .then(core::cmp::Reverse(*i1).cmp(&core::cmp::Reverse(*i2)))
            })
            .map(|(_, s)| s)
    }
}

/// A Lexicon — collection of entries for one language (lime:Lexicon).
///
/// Instance of LemonConcept::Lexicon. One lexicon per language.
/// The entries are indexed by their canonical form's written representation.
#[derive(Debug, Clone)]
pub struct Lexicon {
    pub lang: String,
    entries: BTreeMap<String, LexicalEntry>,
}

impl Lexicon {
    pub fn new(lang: impl Into<String>) -> Self {
        Self {
            lang: lang.into(),
            entries: BTreeMap::new(),
        }
    }

    /// Add a SENSE to the entry for `written_rep`, creating the entry if it is
    /// absent. Unlike a blind insert this APPENDS — so a surface that denotes
    /// several concepts (a general sense plus a domain-specific one) accumulates
    /// ALL its senses on the ONE shared entry (OntoLex one-entry-many-senses),
    /// rather than the last write silently clobbering the earlier senses.
    /// Idempotent: an identical `(reference, domain)` sense is not duplicated.
    pub fn add_sense(
        &mut self,
        written_rep: impl Into<String>,
        ontology: impl Into<String>,
        concept: impl Into<String>,
        domain: Option<String>,
    ) {
        let written_rep = written_rep.into();
        let lang = self.lang.clone();
        let sense = Sense {
            reference: ConceptRef {
                ontology: ontology.into(),
                concept: concept.into(),
            },
            domain,
        };
        let entry = self
            .entries
            .entry(written_rep.clone())
            .or_insert_with(|| LexicalEntry {
                canonical_form: Form { written_rep, lang },
                senses: Vec::new(),
            });
        if !entry.senses.contains(&sense) {
            entry.senses.push(sense);
        }
    }

    /// Add a general (domain-unmarked) sense for an ontology concept — the
    /// common case. A convenience for [`add_sense`](Self::add_sense) with no
    /// domain; repeated calls for one surface accumulate senses, they do not
    /// overwrite.
    pub fn add_entry(
        &mut self,
        written_rep: impl Into<String>,
        ontology: impl Into<String>,
        concept: impl Into<String>,
    ) {
        self.add_sense(written_rep, ontology, concept, None);
    }

    /// Resolve a word to its predominant sense in a query `domain` — the
    /// pointer a statement in that domain follows: the legal sense of "person"
    /// inside a legal corpus, the general sense by default.
    pub fn resolve(&self, written_rep: &str, domain: Option<&str>) -> Option<&Sense> {
        self.lookup(written_rep)
            .and_then(|e| e.predominant_sense(domain))
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

    pub fn entry_count(&self) -> Quantity {
        Quantity::from_unit(self.entries.len() as f64, &unit::UNITLESS)
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
    let mut lex = Lexicon::new("en");

    let descriptors = crate::formal::information::knowledge::describe_knowledge_base();
    for desc in &descriptors {
        let name = desc.name().to_string();
        lex.add_entry(name.clone(), name.clone(), name);
    }

    lex
}

#[cfg(test)]
mod lexicon_tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn english_terminology_is_nonempty() {
        let lex = build_english_terminology();
        assert!(
            lex.entry_count().value > 100.0,
            "expected >100 entries, got {}",
            lex.entry_count().value
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn can_lookup_knowledge_ontology() {
        let lex = build_english_terminology();
        assert!(lex.lookup("KnowledgeOntology").is_some());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn label_for_returns_canonical_form() {
        let lex = build_english_terminology();
        let label = lex.label_for("KnowledgeOntology", "KnowledgeOntology");
        assert_eq!(label, Some("KnowledgeOntology"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lexicon_language_is_english() {
        let lex = build_english_terminology();
        assert_eq!(lex.lang, "en");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn add_sense_accumulates_not_overwrites() {
        // The fixed bug: two concepts sharing one surface BOTH survive on the
        // single shared entry (the old `add_entry` insert silently clobbered).
        let mut lex = Lexicon::new("en");
        lex.add_sense("person", "english_wordnet", "person.n.01", None);
        lex.add_sense(
            "person",
            "us_legal_lexicon",
            "person",
            Some("legal".to_string()),
        );
        let e = lex.lookup("person").expect("entry");
        assert_eq!(
            e.senses.len(),
            2,
            "both senses live on the one shared entry"
        );
        // `add_entry` (the convenience) also accumulates rather than overwrites.
        let mut lex2 = Lexicon::new("en");
        lex2.add_entry("bank", "geo", "riverbank");
        lex2.add_entry("bank", "finance", "financial_institution");
        assert_eq!(lex2.lookup("bank").unwrap().senses.len(), 2);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn add_sense_is_idempotent() {
        let mut lex = Lexicon::new("en");
        lex.add_sense("person", "english_wordnet", "person.n.01", None);
        lex.add_sense("person", "english_wordnet", "person.n.01", None);
        assert_eq!(
            lex.lookup("person").unwrap().senses.len(),
            1,
            "an identical sense is not duplicated"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn legal_sense_elevated_in_legal_domain_general_by_default() {
        // The Q1 model: "person" is ONE shared atom carrying both its WordNet
        // sense and its Dictionary-Act legal sense; the legal sense is elevated
        // (predominant) in the legal register, the general sense governs by
        // default, and BOTH stay reachable ("having both").
        let mut lex = Lexicon::new("en");
        lex.add_sense("person", "english_wordnet", "person.n.01", None);
        lex.add_sense(
            "person",
            "us_legal_lexicon",
            "person",
            Some("legal".to_string()),
        );

        let legal = lex.resolve("person", Some("legal")).expect("a sense");
        assert_eq!(legal.reference.ontology, "us_legal_lexicon");

        let general = lex.resolve("person", None).expect("a sense");
        assert_eq!(general.reference.ontology, "english_wordnet");

        assert_eq!(lex.lookup("person").unwrap().senses.len(), 2);
    }
}
