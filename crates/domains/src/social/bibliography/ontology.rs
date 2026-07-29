//! FRBR — the bibliographic Work/Expression/Manifestation hierarchy (the
//! cataloging-standard "WEMI" model, Group 1 entities), plus Genre as a
//! Group-3-adjacent classification grounded against WordNet's loaded
//! `has_domain_topic`/`domain_topic` relations (Turing-benchmark B2). The
//! classifier, keystone axiom, fixture-composition axiom, and the
//! real-corpus generated-test axiom all live in
//! [`wordnet_grounding`](super::wordnet_grounding) — see that module's doc
//! for the corrected relation direction and the (honestly negative)
//! `exemplifies` finding this ontology no longer claims.
//!
//! # Literature
//!
//! - **IFLA Study Group on the Functional Requirements for Bibliographic
//!   Records (1998)** *Functional Requirements for Bibliographic Records:
//!   Final Report* — §3.2.1 Work, §3.2.2 Expression, §3.2.3 Manifestation
//!   (Group 1 "products of intellectual or artistic endeavour");
//!   §3.4 Concept (Group 3, the classification entities a Work's Genre
//!   specializes).
//! - **Miller (1995)** *WordNet: A Lexical Database for English*,
//!   Communications of the ACM 38(11) — the general WordNet source.
//! - **Bentivogli & Pianta (2004)** "Extending WordNet with Syntagmatic
//!   Information" *Proc. GWC 2004* — the `has_domain_topic`/`domain_topic`
//!   relations this ontology grounds Genre against.
//! - **Magnini & Cavaglià (2000)** *Integrating Subject Field Codes into
//!   WordNet*, Proc. LREC 2000 — the domain-topic annotation methodology
//!   WordNet's `has_domain_topic`/`domain_topic` edges implement.

use pr4xis::ontology::{Axiom, Ontology, Quality};

use super::work::{self, ExpressionRecord, WorkRecord};

pr4xis::ontology! {
    name: "Frbr",
    source: "IFLA FRBR Study Group (1998) Functional Requirements for Bibliographic Records; Miller (1995) WordNet CACM 38(11); Magnini & Cavaglià (2000) Proc. LREC 2000",

    concepts: [Work, Expression, Manifestation, Genre],

    labels: {
        Work: ("en", "Work",
            "A distinct intellectual or artistic creation — the abstract notion underlying every realization of it (the Iliad, as opposed to any particular translation or edition). IFLA FRBR (1998) §3.2.1."),
        Expression: ("en", "Expression",
            "The specific intellectual or artistic realization a Work takes (a particular translation, a particular performance). IFLA FRBR (1998) §3.2.2."),
        Manifestation: ("en", "Manifestation",
            "The physical embodiment of an Expression (an edition, a printing). IFLA FRBR (1998) §3.2.3."),
        Genre: ("en", "Genre",
            "A classification of a Work by literary type (epic, tragedy, novel) — a Group-3-adjacent Concept a Work is a subject of, grounded here against WordNet's loaded has_domain_topic/domain_topic edges rather than invented (e.g. the \"literature\" domain synset's real has_domain_topic membership). IFLA FRBR (1998) §3.4."),
    },

    // A Work is realized through its Expressions; an Expression is embodied
    // in its Manifestations — the WEMI containment chain (IFLA FRBR 1998
    // §3.2, Group 1).
    has_a: [
        (Work, Expression),
        (Expression, Manifestation),
    ],
}

/// Quality: is this concept an FRBR Group 1 entity (a "product of
/// intellectual or artistic endeavour", IFLA FRBR 1998 §3.1)? Work,
/// Expression, and Manifestation are; Genre is a classification (Group 3
/// Concept), not a Group 1 product.
#[derive(Debug, Clone)]
pub struct IsGroup1Entity;

impl Quality for IsGroup1Entity {
    type Individual = FrbrConcept;
    type Value = bool;

    fn get(&self, c: &FrbrConcept) -> Option<bool> {
        use FrbrConcept as C;
        Some(matches!(c, C::Work | C::Expression | C::Manifestation))
    }
}

impl Ontology for FrbrOntology {
    type Cat = FrbrCategory;
    type Qual = IsGroup1Entity;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        use super::wordnet_grounding::{
            GenreDomainTopicAgreesAcrossLoadedEnglishWordNet,
            GenreDomainTopicRoundTripsOnFixtureWordNet, GenreGroundsInWordNetDomainTopic,
        };
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(WorkIsRealizedThroughAtLeastOneExpression));
        axioms.push(Box::new(ExpressionIsEmbodiedInAtLeastOneManifestation));
        axioms.push(Box::new(GenreGroundsInWordNetDomainTopic));
        axioms.push(Box::new(GenreDomainTopicRoundTripsOnFixtureWordNet));
        axioms.push(Box::new(GenreDomainTopicAgreesAcrossLoadedEnglishWordNet));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: a Work is realized through at least one Expression. IFLA FRBR
/// (1998) §3.2.1: "a work is always realized through one or more
/// expressions."
pub struct WorkIsRealizedThroughAtLeastOneExpression;

impl Axiom for WorkIsRealizedThroughAtLeastOneExpression {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let realized = WorkRecord {
            title: "Iliad".into(),
            author: "Homer".into(),
            expressions: vec!["Lattimore translation".into()],
        };
        let unrealized = WorkRecord {
            expressions: vec![],
            ..realized.clone()
        };
        if work::is_realized(&realized) && !work::is_realized(&unrealized) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "WorkIsRealizedThroughAtLeastOneExpression",
        "a work is realized through at least one expression",
        "IFLA FRBR Study Group (1998) \u{00a7}3.2.1"
    );
}

pr4xis::register_axiom!(
    WorkIsRealizedThroughAtLeastOneExpression,
    "IFLA FRBR Study Group (1998) \u{00a7}3.2.1"
);

/// Axiom: an Expression is embodied in at least one Manifestation. IFLA
/// FRBR (1998) §3.2.2.
pub struct ExpressionIsEmbodiedInAtLeastOneManifestation;

impl Axiom for ExpressionIsEmbodiedInAtLeastOneManifestation {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let embodied = ExpressionRecord {
            title: "Lattimore translation".into(),
            manifestations: vec!["1951 University of Chicago Press edition".into()],
        };
        let disembodied = ExpressionRecord {
            manifestations: vec![],
            ..embodied.clone()
        };
        if work::is_embodied(&embodied) && !work::is_embodied(&disembodied) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ExpressionIsEmbodiedInAtLeastOneManifestation",
        "an expression is embodied in at least one manifestation",
        "IFLA FRBR Study Group (1998) \u{00a7}3.2.2"
    );
}

pr4xis::register_axiom!(
    ExpressionIsEmbodiedInAtLeastOneManifestation,
    "IFLA FRBR Study Group (1998) \u{00a7}3.2.2"
);

// Genre's WordNet grounding (classifier + keystone axiom + fixture
// composition + real-corpus generated sweep) lives in
// `super::wordnet_grounding` — see that module's doc for the corrected
// has_domain_topic/domain_topic direction and the honestly-dropped
// `exemplifies` claim (no real edge exists under genre/literature in the
// loaded corpus).

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<FrbrCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        FrbrOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_concepts() {
        assert_eq!(FrbrConcept::variants().len(), 4);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn group_1_entities_are_correctly_partitioned() {
        let q = IsGroup1Entity;
        for c in FrbrConcept::variants() {
            assert!(q.get(&c).is_some(), "{c:?} has no IsGroup1Entity");
        }
        assert_eq!(q.get(&FrbrConcept::Work), Some(true));
        assert_eq!(q.get(&FrbrConcept::Genre), Some(false));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn work_is_realized_through_at_least_one_expression_holds() {
        assert!(WorkIsRealizedThroughAtLeastOneExpression.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn expression_is_embodied_in_at_least_one_manifestation_holds() {
        assert!(
            ExpressionIsEmbodiedInAtLeastOneManifestation
                .verify()
                .is_ok()
        );
    }

    // Genre's WordNet-grounding axioms (keystone, fixture-composition, and
    // the real-corpus generated sweep) are tested in
    // `super::wordnet_grounding::tests`.
}
