//! WordNet as a pr4xis ontology.
//!
//! George A. Miller's WordNet — the lexical database organizing words
//! into sets of cognitive synonyms (synsets), each expressing a distinct
//! concept, linked by semantic relations. This is the foundation of
//! pr4xis's lexical lookup, taxonomy traversal, and common-ancestor
//! steps (`PipelineStep::EntityLookup`, `TaxonomyTraversal`,
//! `CommonAncestor`), making their `ontology_name()` resolvable to
//! `WordNetOntology::meta().name`.
//!
//! References:
//! - Miller, G. A. (1995). *WordNet: A Lexical Database for English*.
//!   Communications of the ACM 38(11).
//! - Fellbaum, C. (ed.) (1998). *WordNet: An Electronic Lexical Database*.
//!   MIT Press.
//! - Miller, G. A. et al. (1990). *Introduction to WordNet: An On-line
//!   Lexical Database*. International Journal of Lexicography 3(4).

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "WordNet",
    source: "Miller (1995) CACM 38(11); Fellbaum (ed.) WordNet (MIT Press 1998)",
    being: SocialObject,

    concepts: [
        Synset,
        Word,
        Sense,
        Hypernym,
        Hyponym,
        Meronym,
        Holonym,
        Antonym,
    ],

    labels: {
        Synset: ("en", "Synset", "A set of cognitive synonyms — words that denote the same concept. The atomic unit of WordNet. Miller (1995)."),
        Word: ("en", "Word", "A lexical form. A word can belong to multiple synsets (polysemy). Miller (1995)."),
        Sense: ("en", "Sense", "A (word, synset) pair — the meaning of a specific word in a specific context. Miller (1995)."),
        Hypernym: ("en", "Hypernym", "A superordinate synset (is-a parent). Dog ⊑ Mammal. The relation that makes taxonomy traversal possible. Miller (1995)."),
        Hyponym: ("en", "Hyponym", "A subordinate synset (is-a child). The inverse of Hypernym. Miller (1995)."),
        Meronym: ("en", "Meronym", "A part-of relation: wheel is a meronym of car. Miller (1995)."),
        Holonym: ("en", "Holonym", "The whole-of inverse of Meronym. Miller (1995)."),
        Antonym: ("en", "Antonym", "A lexical opposition relation between words — not synsets. Miller (1995): antonymy is word-level, not concept-level."),
    },

    is_a: [
        (Hypernym, Synset),
        (Hyponym, Synset),
    ],

    edges: [
        (Word, Sense, HasSense),
        (Sense, Synset, DenotesSynset),
        (Synset, Hypernym, HasHypernym),
        (Synset, Hyponym, HasHyponym),
        (Synset, Meronym, HasMeronym),
        (Synset, Holonym, HasHolonym),
        (Word, Antonym, HasAntonym),
    ],
}

/// Whether a concept is a lexical unit, a synset, or a lexical relation.
#[derive(Debug, Clone)]
pub struct WordNetRole;

impl Quality for WordNetRole {
    type Individual = WordNetConcept;
    type Value = &'static str;

    fn get(&self, c: &WordNetConcept) -> Option<&'static str> {
        use WordNetConcept as W;
        Some(match c {
            W::Synset | W::Hypernym | W::Hyponym => "conceptual",
            W::Word | W::Sense => "lexical",
            W::Meronym | W::Holonym | W::Antonym => "relation",
        })
    }
}

/// Axiom: Hypernym and Hyponym are inverses — the defining property of
/// the taxonomy relation in WordNet (Miller 1995 §3).
pub struct WordNetTaxonomyHasInverses;

impl Axiom for WordNetTaxonomyHasInverses {
    fn description(&self) -> &str {
        "Hypernym and Hyponym are inverse relations on synsets (Miller 1995): A Hypernym B ⇔ B Hyponym A"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
        let rels = WordNetTaxonomy::relations();
        rels.iter()
            .any(|(c, p)| *c == WordNetConcept::Hypernym && *p == WordNetConcept::Synset)
            && rels
                .iter()
                .any(|(c, p)| *c == WordNetConcept::Hyponym && *p == WordNetConcept::Synset)
    }
}

impl Ontology for WordNetOntology {
    type Cat = WordNetCategory;
    type Qual = WordNetRole;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        WordNetOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(WordNetTaxonomyHasInverses)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<WordNetCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        WordNetOntology::validate().unwrap();
    }

    #[test]
    fn wordnet_taxonomy_has_inverses_holds() {
        assert!(
            WordNetTaxonomyHasInverses.holds(),
            "{}",
            WordNetTaxonomyHasInverses.description()
        );
    }
}
