//! Ontology Algebra — categorical operations on ontologies.
//!
//! Ontologies compose via categorical constructs: coproduct (union),
//! product (intersection), pushout (merge), pullback (shared structure),
//! and the Spivak migration triple ΣF ⊣ ΔF ⊣ ΠF.
//!
//! Every query is a composition of these operations: "Is a dog a
//! mammal?" is a Δ-migration pulling `dog` through the taxonomy functor.
//!
//! # Literature
//!
//! - **Goguen & Burstall (1992)** "Institutions: Abstract Model Theory
//!   for Specification and Programming", *Journal of the ACM*
//!   39(1):95-146 — "colimits are how to compose systems"; the
//!   institutional framework for ontology operations.
//! - **Zimmermann, Krötzsch, Euzenat & Hitzler (2006)** "Formalizing
//!   Ontology Alignment and its Operations with Category Theory",
//!   *FOIS 2006* — pushout = merge; pullback = shared structure.
//! - **Spivak (2012)** "Functorial Data Migration", *Information and
//!   Computation* 217:31-51 — the ΣF ⊣ ΔF ⊣ ΠF adjoint triple.
//! - **Smith (2006)** "Composition by Colimit", in Goguen & Malcolm
//!   *Algebraic Approaches to Program Semantics* — composition of
//!   software-engineering structures via colimit.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Algebra",
    source: "Goguen & Burstall (1992) Institutions: Abstract Model Theory for Specification and Programming, JACM 39(1):95-146; Zimmermann, Krötzsch, Euzenat & Hitzler (2006) Formalizing Ontology Alignment and its Operations with Category Theory, FOIS; Spivak (2012) Functorial Data Migration, Information and Computation 217:31-51; Smith (2006) Composition by Colimit",

    concepts: [
        Coproduct,
        Product,
        Pushout,
        Pullback,
        Colimit,
        Limit,
        DeltaMigration,
        SigmaMigration,
        PiMigration,
        Diagram,
        Span,
        Cospan,
        Ontology,
        Mapping,
    ],

    labels: {
        Coproduct: ("en", "Coproduct",
            "Goguen & Burstall (1992): A || B - disjoint union; colimit of the discrete diagram {A, B}."),
        Product: ("en", "Product",
            "A & B - pullback / shared structure; Zimmermann (2006) intersection of alignments."),
        Pushout: ("en", "Pushout",
            "Zimmermann (2006): A ⊕ B along S - the merge of two ontologies over a shared sub-ontology. Goguen (1992) composition by colimit."),
        Pullback: ("en", "Pullback",
            "The shared sub-ontology that two ontologies have in common - the source of a span (V-alignment)."),
        Colimit: ("en", "Colimit",
            "Goguen & Burstall (1992): the general composition of a diagram - 'colimits are how to compose systems.'"),
        Limit: ("en", "Limit",
            "Mac Lane (1971): the general shared structure of a diagram."),
        DeltaMigration: ("en", "Delta migration",
            "Spivak (2012): ΔF - pullback migration; restricts/projects data from target schema to source."),
        SigmaMigration: ("en", "Sigma migration",
            "Spivak (2012): ΣF - left pushforward via coproduct (union). Left adjoint of Δ."),
        PiMigration: ("en", "Pi migration",
            "Spivak (2012): ΠF - right pushforward via product. Right adjoint of Δ."),
        Diagram: ("en", "Diagram",
            "A collection of ontologies connected by functors - the input to colimit/limit operations."),
        Span: ("en", "Span",
            "Zimmermann (2006): two functors with common domain - the basis of ontology alignment."),
        Cospan: ("en", "Cospan",
            "Two functors with common codomain - the basis of pushout composition."),
        Ontology: ("en", "Ontology",
            "An object in the category of ontologies."),
        Mapping: ("en", "Mapping",
            "A functor between ontologies - a morphism in the category."),
    },

    is_a: [
        // Coproducts and pushouts are colimits.
        (Coproduct, Colimit),
        (Pushout, Colimit),
        // Products and pullbacks are limits.
        (Product, Limit),
        (Pullback, Limit),
        // The Spivak migration functors are mappings.
        (DeltaMigration, Mapping),
        (SigmaMigration, Mapping),
        (PiMigration, Mapping),
        // Spans and cospans are diagrams.
        (Span, Diagram),
        (Cospan, Diagram),
    ],

    has_a: [
        // A diagram contains ontologies and mappings.
        (Diagram, Ontology),
        (Diagram, Mapping),
        // Coproduct / Product take two ontologies.
        (Coproduct, Ontology),
        (Product, Ontology),
        // Pushout consumes a span; pullback consumes a cospan.
        (Pushout, Span),
        (Pullback, Cospan),
    ],

    opposes: [
        // Union vs intersection (Zimmermann 2006).
        (Coproduct, Product),
        (Product, Coproduct),
        // Synthesis vs analysis (Mac Lane 1971).
        (Colimit, Limit),
        (Limit, Colimit),
        // Left vs right pushforward (Spivak 2012).
        (SigmaMigration, PiMigration),
        (PiMigration, SigmaMigration),
    ],
}

/// Quality: whether a concept is an operation (vs a structural element).
#[derive(Debug, Clone)]
pub struct IsOperation;

impl Quality for IsOperation {
    type Individual = AlgebraConcept;
    type Value = bool;

    fn get(&self, c: &AlgebraConcept) -> Option<bool> {
        use AlgebraConcept as A;
        Some(matches!(
            c,
            A::Coproduct
                | A::Product
                | A::Pushout
                | A::Pullback
                | A::Colimit
                | A::Limit
                | A::DeltaMigration
                | A::SigmaMigration
                | A::PiMigration
        ))
    }
}

impl Ontology for AlgebraOntology {
    type Cat = AlgebraCategory;
    type Qual = IsOperation;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(AdjointTriple));
        axioms.push(Box::new(CoproductProductDual));
        axioms
    }
}

/// Spivak (2012): ΣF ⊣ ΔF ⊣ ΠF is an adjoint triple. We assert the
/// pairwise opposition between SigmaMigration and PiMigration via the
/// Opposition kind to keep the structural assertion in the category.
pub struct AdjointTriple;

impl Axiom for AdjointTriple {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let opp: Vec<_> = AlgebraCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AlgebraRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        if opp.contains(&(AlgebraConcept::SigmaMigration, AlgebraConcept::PiMigration)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AdjointTriple",
        "Sigma migration and Pi migration are opposed (left vs right adjoint of Delta)",
        "Spivak (2012) Functorial Data Migration, Information and Computation 217:31-51"
    );
}

pr4xis::register_axiom!(
    AdjointTriple,
    "Spivak (2012) Functorial Data Migration, Information and Computation 217:31-51"
);

/// Zimmermann (2006): Coproduct (union) and Product (intersection) are
/// dual categorical operations.
pub struct CoproductProductDual;

impl Axiom for CoproductProductDual {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let opp: Vec<_> = AlgebraCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AlgebraRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        if opp.contains(&(AlgebraConcept::Coproduct, AlgebraConcept::Product)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CoproductProductDual",
        "Coproduct and Product are dual categorical operations (union vs intersection)",
        "Zimmermann, Krötzsch, Euzenat & Hitzler (2006) Formalizing Ontology Alignment and its Operations with Category Theory, FOIS"
    );
}

pr4xis::register_axiom!(
    CoproductProductDual,
    "Zimmermann, Krötzsch, Euzenat & Hitzler (2006) Formalizing Ontology Alignment and its Operations with Category Theory, FOIS"
);
