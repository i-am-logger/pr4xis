// Adjunction: KnowledgeToLemon ⊣ LemonToKnowledge
//
// Left adjoint F = KnowledgeToLemon: maps knowledge structure to linguistic structure.
// Right adjoint G = LemonToKnowledge: maps linguistic structure back to knowledge.
//
// Unit η_A: A → G(F(A)) — embeds a knowledge concept into its round-trip form.
// Counit ε_B: F(G(B)) → B — projects the linguistic round-trip back.
//
// Gap analysis: the unit/counit collapse reveals which knowledge concepts
// lose distinction when linguistically realized, and which linguistic
// concepts have no knowledge-base counterpart.

use pr4xis::category::Adjunction;
use pr4xis::category::Functor;

use super::lemon_functor::KnowledgeToLemon;
use super::ontology::{KnowledgeConcept, KnowledgeRelation, KnowledgeRelationKind};
use crate::cognitive::linguistics::lemon::knowledge_functor::LemonToKnowledge;
use crate::cognitive::linguistics::lemon::ontology::{
    LemonConcept, LemonRelation, LemonRelationKind,
};

pub struct KnowledgeLemonAdjunction;

impl Adjunction for KnowledgeLemonAdjunction {
    type Left = KnowledgeToLemon;
    type Right = LemonToKnowledge;

    fn unit(obj: &KnowledgeConcept) -> KnowledgeRelation {
        let round_trip = LemonToKnowledge::map_object(&KnowledgeToLemon::map_object(obj));
        KnowledgeRelation {
            from: *obj,
            to: round_trip,
            kind: KnowledgeRelationKind::Composed,
        }
    }

    fn counit(obj: &LemonConcept) -> LemonRelation {
        let round_trip = KnowledgeToLemon::map_object(&LemonToKnowledge::map_object(obj));
        LemonRelation {
            from: round_trip,
            to: *obj,
            kind: LemonRelationKind::Composed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::entity::Entity;

    #[test]
    fn unit_is_valid() {
        for obj in KnowledgeConcept::variants() {
            let u = KnowledgeLemonAdjunction::unit(&obj);
            assert!(
                KnowledgeConcept::variants().contains(&u.from),
                "unit source {:?} not in Knowledge",
                u.from
            );
            assert!(
                KnowledgeConcept::variants().contains(&u.to),
                "unit target {:?} not in Knowledge",
                u.to
            );
        }
    }

    #[test]
    fn counit_is_valid() {
        for obj in LemonConcept::variants() {
            let c = KnowledgeLemonAdjunction::counit(&obj);
            assert!(
                LemonConcept::variants().contains(&c.from),
                "counit source {:?} not in Lemon",
                c.from
            );
            assert!(
                LemonConcept::variants().contains(&c.to),
                "counit target {:?} not in Lemon",
                c.to
            );
        }
    }

    #[test]
    fn gap_analysis() {
        let knowledge_variants = KnowledgeConcept::variants();
        let lemon_variants = LemonConcept::variants();

        let mut unit_collapses = Vec::new();
        for obj in &knowledge_variants {
            let u = KnowledgeLemonAdjunction::unit(obj);
            if u.from != u.to {
                unit_collapses.push((*obj, u.to));
            }
        }

        let mut counit_collapses = Vec::new();
        for obj in &lemon_variants {
            let c = KnowledgeLemonAdjunction::counit(obj);
            if c.from != c.to {
                counit_collapses.push((*obj, c.from));
            }
        }

        let unit_loss = unit_collapses.len() as f64 / knowledge_variants.len() as f64;
        let counit_loss = counit_collapses.len() as f64 / lemon_variants.len() as f64;

        assert!(
            unit_loss < 1.0,
            "total unit collapse — Knowledge and Lemon share no structure"
        );
        assert!(
            counit_loss < 1.0,
            "total counit collapse — Lemon and Knowledge share no structure"
        );
    }
}
