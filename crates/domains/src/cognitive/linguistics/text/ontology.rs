//! Text Occurrence — where linguistic units live in text.
//!
//! Bridges NIF (text position), Lemon (lexicon), OLiA (annotation),
//! and Lambek (grammar) into a unified model of typed tokens.
//!
//! A Word is NOT a string. It's a text occurrence at a position in a
//! Context, connected to a LexicalEntry in a Lexicon (via Lemon),
//! carrying a grammatical type (via Lambek), and annotated with
//! linguistic features (via OLiA).
//!
//! Source: Hellmann et al. NIF (2013); Chiarcos & Sukhareva OLiA (2015);
//!         Coecke, Sadrzadeh & Clark DisCoCat (2010)

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Text",
    source: "Hellmann NIF (2013); Chiarcos OLiA (2015); Coecke DisCoCat (2010)",
    being: AbstractObject,

    concepts: [
        // NIF structural concepts (Hellmann 2013)
        Context,
        Word,
        Sentence,
        Phrase,
        Span,
        // Bridging concepts (functors to other ontologies)
        LexiconReference,
        GrammaticalType,
        MeaningReference,
        Annotation,
    ],

    labels: {
        Context: ("en", "Context", "nif:Context — the reference text containing all occurrences."),
        Word: ("en", "Word", "nif:Word — a token occurrence at a position in Context."),
        Sentence: ("en", "Sentence", "nif:Sentence — a sequence of Words forming a grammatical unit."),
        Phrase: ("en", "Phrase", "nif:Phrase — a contiguous span of Words (NP, VP, etc.)."),
        Span: ("en", "Span", "A position range in the Context (beginIndex, endIndex)."),
        LexiconReference: ("en", "Lexicon reference", "The lexicon entry this Word maps to (Lemon functor target)."),
        GrammaticalType: ("en", "Grammatical type", "The grammatical type assigned (Lambek functor target)."),
        MeaningReference: ("en", "Meaning reference", "The ontology concept referenced through meaning (DisCoCat target)."),
        Annotation: ("en", "Annotation", "Linguistic annotation — POS, morphology, dependency (OLiA)."),
    },

    is_a: [
        (Word, Span),
        (Sentence, Span),
        (Phrase, Span),
    ],

    has_a: [
        (Context, Sentence),
        (Sentence, Word),
        (Phrase, Word),
        (Word, Span),
        (Word, LexiconReference),
        (Word, GrammaticalType),
        (Word, MeaningReference),
        (Word, Annotation),
    ],
}

/// Whether a concept is NIF-structural vs a bridging reference.
#[derive(Debug, Clone)]
pub struct IsStructural;

impl Quality for IsStructural {
    type Individual = TextConcept;
    type Value = bool;

    fn get(&self, individual: &TextConcept) -> Option<bool> {
        Some(matches!(
            individual,
            TextConcept::Context
                | TextConcept::Word
                | TextConcept::Sentence
                | TextConcept::Phrase
                | TextConcept::Span
        ))
    }
}

/// Word has all four bridging connections.
#[derive(Debug)]
pub struct WordIsFullyConnected;

impl Axiom for WordIsFullyConnected {
    fn description(&self) -> &str {
        "Word has LexiconReference, GrammaticalType, MeaningReference, Annotation"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::mereology::MereologyDef;
        let parts = TextMereology::relations();
        let targets = [
            TextConcept::LexiconReference,
            TextConcept::GrammaticalType,
            TextConcept::MeaningReference,
            TextConcept::Annotation,
        ];
        targets.iter().all(|t| {
            parts
                .iter()
                .any(|(whole, part)| *whole == TextConcept::Word && part == t)
        })
    }
}
pr4xis::register_axiom!(WordIsFullyConnected);

/// Context contains Sentences which contain Words (two-level mereology).
#[derive(Debug)]
pub struct TwoLevelContainment;

impl Axiom for TwoLevelContainment {
    fn description(&self) -> &str {
        "Context contains Sentences, Sentences contain Words (NIF structure)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::mereology::MereologyDef;
        let parts = TextMereology::relations();
        let ctx_has_sent = parts
            .iter()
            .any(|(w, p)| *w == TextConcept::Context && *p == TextConcept::Sentence);
        let sent_has_word = parts
            .iter()
            .any(|(w, p)| *w == TextConcept::Sentence && *p == TextConcept::Word);
        ctx_has_sent && sent_has_word
    }
}
pr4xis::register_axiom!(TwoLevelContainment);

impl Ontology for TextOntology {
    type Cat = TextCategory;
    type Qual = IsStructural;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        TextOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(WordIsFullyConnected),
            Box::new(TwoLevelContainment),
        ]
    }
}
