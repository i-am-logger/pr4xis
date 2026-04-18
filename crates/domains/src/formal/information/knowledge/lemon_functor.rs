// Knowledge → Lemon functor.
//
// Proof that the Knowledge Base ontology (VoID/DCAT) maps structurally
// to the Ontolex-Lemon model. This is how self-description connects
// to linguistic realization: a Vocabulary's entries become LexicalEntries,
// its schema becomes the conceptual layer, its descriptors become Forms.
//
// The functor makes explicit what Ontolex-Lemon assumes: every ontology
// element (Vocabulary, Entry, DataSource) has a linguistic realization
// reachable through the Lemon bridge.
//
// Source: W3C VoID (2011) × W3C Ontolex (2016)

use pr4xis::category::Functor;

use super::ontology::*;
use crate::cognitive::linguistics::lemon::ontology::*;

pub struct KnowledgeToLemon;

impl Functor for KnowledgeToLemon {
    type Source = KnowledgeBaseCategory;
    type Target = LemonCategory;

    fn map_object(obj: &KnowledgeConcept) -> LemonConcept {
        match obj {
            // KnowledgeBase collects Vocabularies; Lexicon collects LexicalEntries.
            // Both are the container concept for their respective domains.
            KnowledgeConcept::KnowledgeBase => LemonConcept::Lexicon,

            // A Vocabulary IS the ontology entity that Lemon's LexicalSense
            // points to via reference. It's what gets linguistically described.
            KnowledgeConcept::Vocabulary => LemonConcept::OntologyReference,

            // An Entry (skos:Concept) maps to LexicalEntry — the linguistic
            // realization of a concept. This is the core Lemon insight:
            // every concept in a vocabulary has a lexical entry that names it.
            KnowledgeConcept::Entry => LemonConcept::LexicalEntry,

            // A Schema (skos:ConceptScheme) maps to LexicalConcept — the
            // mental abstraction layer. The schema defines the conceptual
            // structure; LexicalConcept captures concepts independent of
            // any particular language.
            KnowledgeConcept::Schema => LemonConcept::LexicalConcept,

            // A Descriptor (void statistics) maps to Form — the surface
            // realization of structural metadata. Counts, labels, annotations
            // are the "written representation" of the ontology's structure.
            KnowledgeConcept::Descriptor => LemonConcept::Form,

            // A DataSource maps to OntologyReference — external sources
            // are themselves ontology entities that can be referenced.
            KnowledgeConcept::DataSource => LemonConcept::OntologyReference,
        }
    }

    fn map_morphism(m: &KnowledgeRelation) -> LemonRelation {
        LemonRelation {
            from: Self::map_object(&m.from),
            to: Self::map_object(&m.to),
            kind: map_kind(&m.kind),
        }
    }
}
pr4xis::register_functor!(KnowledgeToLemon);

fn map_kind(kind: &KnowledgeRelationKind) -> LemonRelationKind {
    match kind {
        // Catalogs → Entry: KnowledgeBase catalogs Vocabulary
        // maps to Lexicon contains LexicalEntry (via Entry relation kind)
        KnowledgeRelationKind::Catalogs => LemonRelationKind::Entry,

        // Contains → Sense: Vocabulary contains Entry
        // maps to LexicalEntry has Sense (entry IS the linguistic unit)
        KnowledgeRelationKind::Contains => LemonRelationKind::Sense,

        // ConformsTo → IsConceptOf: Vocabulary conforms to Schema
        // maps to OntologyReference isConceptOf LexicalConcept
        KnowledgeRelationKind::ConformsTo => LemonRelationKind::IsConceptOf,

        // DescribedBy → CanonicalForm: Vocabulary described by Descriptor
        // maps to OntologyReference's canonical form (the preferred label)
        KnowledgeRelationKind::DescribedBy => LemonRelationKind::CanonicalForm,

        // DerivedFrom → Reference: Vocabulary derived from DataSource
        // maps to LexicalSense references OntologyReference
        KnowledgeRelationKind::DerivedFrom => LemonRelationKind::Reference,

        // Defines → Evokes: Schema defines Entry
        // maps to LexicalConcept evokes LexicalEntry
        KnowledgeRelationKind::Defines => LemonRelationKind::Evokes,

        // Structural morphisms
        KnowledgeRelationKind::Identity => LemonRelationKind::Identity,
        KnowledgeRelationKind::Composed => LemonRelationKind::Composed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<KnowledgeToLemon>().unwrap();
    }
}
