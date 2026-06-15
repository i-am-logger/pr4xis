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

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Text",
    source: "Hellmann NIF (2013); Chiarcos OLiA (2015); Coecke DisCoCat (2010)",

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
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts: Vec<_> = TextCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == TextRelationKind::Parthood)
            .collect();
        let targets = [
            TextConcept::LexiconReference,
            TextConcept::GrammaticalType,
            TextConcept::MeaningReference,
            TextConcept::Annotation,
        ];
        // Parthood is part→whole (BFO:0000050): each reference/type/annotation is
        // a PART of the Word, so the part is the source and Word the target.
        if targets.iter().all(|t| {
            parts
                .iter()
                .any(|m| m.source() == *t && m.target() == TextConcept::Word)
        }) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "WordIsFullyConnected",
        "Word has LexiconReference, GrammaticalType, MeaningReference, Annotation",
        "Hellmann et al. (2013) NIF 2.0 Core Ontology; Chiarcos & Sukhareva (2015) OLiA"
    );
}
pr4xis::register_axiom!(
    WordIsFullyConnected,
    "Hellmann et al. (2013) NIF 2.0 Core Ontology; Chiarcos & Sukhareva (2015) OLiA"
);

/// Context contains Sentences which contain Words (two-level mereology).
#[derive(Debug)]
pub struct TwoLevelContainment;

impl Axiom for TwoLevelContainment {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts: Vec<_> = TextCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == TextRelationKind::Parthood)
            .collect();
        // part→whole: a Sentence is PART of the Context, a Word PART of the Sentence.
        let ctx_has_sent = parts
            .iter()
            .any(|m| m.source() == TextConcept::Sentence && m.target() == TextConcept::Context);
        let sent_has_word = parts
            .iter()
            .any(|m| m.source() == TextConcept::Word && m.target() == TextConcept::Sentence);
        if ctx_has_sent && sent_has_word {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TwoLevelContainment",
        "Context contains Sentences, Sentences contain Words (NIF structure)",
        "Hellmann et al. (2013) NIF 2.0 Core Ontology"
    );
}
pr4xis::register_axiom!(
    TwoLevelContainment,
    "Hellmann et al. (2013) NIF 2.0 Core Ontology"
);

impl Ontology for TextOntology {
    type Cat = TextCategory;
    type Qual = IsStructural;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(WordIsFullyConnected));
        axioms.push(Box::new(TwoLevelContainment));
        axioms
    }
}
