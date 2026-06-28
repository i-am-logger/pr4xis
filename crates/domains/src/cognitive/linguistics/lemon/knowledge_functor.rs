// Lemon → Knowledge functor.
//
// The reverse direction: linguistic structure maps to knowledge structure.
// A Lexicon IS a knowledge base (it catalogs lexical entries).
// A LexicalEntry IS an entry in a vocabulary.
// An OntologyReference IS the vocabulary being described.
//
// Together with Knowledge → Lemon, this forms a candidate adjunction.
// The gap analysis reveals what each ontology is missing from the other.
//
// Source: W3C Ontolex (2016) × W3C VoID (2011)

use pr4xis::category::Functor;

use super::ontology::*;
use crate::formal::information::knowledge::ontology::*;

pub struct LemonToKnowledge;

impl Functor for LemonToKnowledge {
    type Source = LemonCategory;
    type Target = KnowledgeCategory;

    fn map_object(obj: &LemonConcept) -> KnowledgeConcept {
        match obj {
            // Lexicon collects entries → KnowledgeBase collects vocabularies.
            LemonConcept::Lexicon => KnowledgeConcept::KnowledgeBase,

            // LexicalEntry is a concept in the lexicon → Entry in the knowledge base.
            LemonConcept::LexicalEntry => KnowledgeConcept::Entry,

            // Form is surface realization → Descriptor (structural metadata).
            LemonConcept::Form => KnowledgeConcept::Descriptor,

            // LexicalSense bridges entry to ontology → Vocabulary
            // (the sense IS the vocabulary's self-description of what it means).
            LemonConcept::LexicalSense => KnowledgeConcept::Vocabulary,

            // LexicalConcept is mental abstraction → Schema
            // (the conceptual layer that entries conform to).
            LemonConcept::LexicalConcept => KnowledgeConcept::Schema,

            // OntologyReference is what gets described → Vocabulary.
            LemonConcept::OntologyReference => KnowledgeConcept::Vocabulary,
        }
    }

    fn map_morphism(m: &LemonRelation) -> KnowledgeRelation {
        KnowledgeRelation {
            from: Self::map_object(&m.from),
            to: Self::map_object(&m.to),
            kind: map_kind(&m.kind),
        }
    }
}
pr4xis::register_functor!(LemonToKnowledge);

fn map_kind(kind: &LemonRelationKind) -> KnowledgeRelationKind {
    match kind {
        // Lexicon → LexicalEntry (Entry) maps to KnowledgeBase → Vocabulary (Catalogs)
        LemonRelationKind::Entry => KnowledgeRelationKind::Catalogs,

        // LexicalEntry → Form (CanonicalForm) maps to Entry described by Descriptor
        LemonRelationKind::CanonicalForm => KnowledgeRelationKind::DescribedBy,
        LemonRelationKind::OtherForm => KnowledgeRelationKind::DescribedBy,

        // LexicalEntry → LexicalSense (Sense) maps to Entry contained in Vocabulary
        LemonRelationKind::Sense => KnowledgeRelationKind::Contains,

        // LexicalSense → OntologyReference (Reference) maps to Vocabulary derived from DataSource
        LemonRelationKind::Reference => KnowledgeRelationKind::DerivedFrom,

        // LexicalEntry → OntologyReference (Denotes) — the canonical
        // Lemon "denotes" relation maps to the knowledge-base Subsumption
        // (Smith 2005 OBO-RO): the lexical entry IS-A reference to the
        // ontology entry. Identity would be wrong: the source and target
        // images differ, and target's identity-aware compose would then
        // break FunctorCompositionLaw (Mac Lane CWM Ch. II §1).
        LemonRelationKind::Denotes => KnowledgeRelationKind::Subsumption,

        // LexicalEntry → LexicalConcept (Evokes) maps to Entry conforming to Schema
        LemonRelationKind::Evokes => KnowledgeRelationKind::ConformsTo,

        // LexicalConcept → OntologyReference (IsConceptOf) maps to Schema defines Entry
        LemonRelationKind::IsConceptOf => KnowledgeRelationKind::Defines,

        // LexicalConcept → LexicalSense (LexicalizedSense) maps to Schema → Vocabulary
        LemonRelationKind::LexicalizedSense => KnowledgeRelationKind::ConformsTo,

        LemonRelationKind::Identity => KnowledgeRelationKind::Identity,

        // Canonical Relations-ontology kinds (Smith 2005 OBO-RO) — unreachable
        // when no source edges use these kinds.
        LemonRelationKind::Subsumption
        | LemonRelationKind::Parthood
        | LemonRelationKind::Causation
        | LemonRelationKind::Opposition => KnowledgeRelationKind::Identity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws() {
        assert_functor_laws::<LemonToKnowledge>();
    }
}
