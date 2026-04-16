// Ontology Algebra — categorical operations on ontologies.
//
// Ontologies compose via categorical constructs: coproduct (union),
// product (intersection), pushout (merge), pullback (shared structure),
// and the Sigma/Delta/Pi migration triple.
//
// A || B = coproduct — disjoint union, both ontologies side by side.
// A & B  = product/pullback — shared structure between A and B.
// A -> B = functor — structure-preserving map.
// A ⊣ B  = adjunction — optimal inverse pair.
//
// Every query is a composition of these operations. "Is a dog a mammal?"
// is a Delta migration pulling "dog" through the taxonomy functor.
//
// Source: Goguen & Burstall "Institutions" (1992);
//         Zimmermann et al. "Ontology Alignment" (FOIS 2006);
//         Spivak "Functorial Data Migration" (2012);
//         Smith "Composition by Colimit" (2006)

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Concepts in the Ontology Algebra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum AlgebraConcept {
    // === Core operations ===
    /// A || B — coproduct (disjoint union) of two ontologies.
    /// Goguen: colimit of the discrete diagram {A, B}.
    /// All concepts from both, no identification.
    Coproduct,

    /// A & B — product (pullback) of two ontologies.
    /// The shared structure: only what's common survives.
    /// Zimmermann: intersection of alignments.
    Product,

    /// A ⊕ B along S — pushout over a shared sub-ontology S.
    /// Zimmermann: ontology merge. The minimal union respecting alignment.
    /// Goguen: composition by colimit.
    Pushout,

    /// The shared sub-ontology that two ontologies have in common.
    /// The source of a span (V-alignment). Computed by pullback.
    Pullback,

    /// A colimit — the general composition of a diagram of ontologies.
    /// Goguen: "colimits are how to compose systems."
    Colimit,

    /// A limit — the general shared structure of a diagram.
    Limit,

    // === Migration functors (Spivak/CQL) ===
    /// ΔF: pullback migration. Restricts data from target to source.
    /// "Project backward along the functor."
    DeltaMigration,

    /// ΣF: left pushforward. Pushes data forward via coproduct (union).
    /// Left adjoint of Delta.
    SigmaMigration,

    /// ΠF: right pushforward. Pushes data forward via product.
    /// Right adjoint of Delta.
    PiMigration,

    // === Structural concepts ===
    /// A diagram — a collection of ontologies connected by functors.
    /// The input to colimit/limit operations.
    Diagram,

    /// A span — two functors with common domain (V-alignment).
    /// Zimmermann: the basis of ontology alignment.
    Span,

    /// A cospan — two functors with common codomain.
    /// The basis of pushout composition.
    Cospan,

    /// An ontology itself — an object in the category of ontologies.
    Ontology,

    /// A functor between ontologies — a morphism in the category.
    Mapping,
}

define_ontology! {
    /// Ontology Algebra — categorical operations on ontologies.
    pub AlgebraOntology for AlgebraCategory {
        concepts: AlgebraConcept,
        relation: AlgebraRelation,

        being: AbstractObject,
        source: "Goguen & Burstall (1992); Zimmermann et al. (2006); Spivak (2012); Smith (2006)",

        is_a: AlgebraTaxonomy [
            // Coproduct and Product are special cases of Colimit and Limit
            (Coproduct, Colimit),
            (Pushout, Colimit),
            (Product, Limit),
            (Pullback, Limit),
            // Migration functors are Mappings
            (DeltaMigration, Mapping),
            (SigmaMigration, Mapping),
            (PiMigration, Mapping),
            // Span and Cospan are Diagrams
            (Span, Diagram),
            (Cospan, Diagram),
        ],

        has_a: AlgebraMereology [
            // A Diagram contains Ontologies and Mappings
            (Diagram, Ontology),
            (Diagram, Mapping),
            // Coproduct/Product take two Ontologies
            (Coproduct, Ontology),
            (Product, Ontology),
            // Pushout needs a Span (shared base)
            (Pushout, Span),
            // Pullback needs a Cospan
            (Pullback, Cospan),
        ],

        opposes: AlgebraOpposition [
            // Coproduct vs Product (union vs intersection)
            (Coproduct, Product),
            // Colimit vs Limit (synthesis vs analysis)
            (Colimit, Limit),
            // Sigma vs Pi (left vs right pushforward)
            (SigmaMigration, PiMigration),
        ],
    }
}

/// Whether a concept is an operation vs a structural element.
#[derive(Debug, Clone)]
pub struct IsOperation;

impl Quality for IsOperation {
    type Individual = AlgebraConcept;
    type Value = bool;

    fn get(&self, individual: &AlgebraConcept) -> Option<bool> {
        Some(matches!(
            individual,
            AlgebraConcept::Coproduct
                | AlgebraConcept::Product
                | AlgebraConcept::Pushout
                | AlgebraConcept::Pullback
                | AlgebraConcept::Colimit
                | AlgebraConcept::Limit
                | AlgebraConcept::DeltaMigration
                | AlgebraConcept::SigmaMigration
                | AlgebraConcept::PiMigration
        ))
    }
}

/// ΣF ⊣ ΔF ⊣ ΠF — the adjoint triple (Spivak 2012).
#[derive(Debug)]
pub struct AdjointTriple;

impl Axiom for AdjointTriple {
    fn description(&self) -> &str {
        "ΣF ⊣ ΔF ⊣ ΠF: the migration functors form an adjoint triple (Spivak 2012)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::opposition::OppositionDef;
        AlgebraOpposition::pairs()
            .iter()
            .any(|(a, b)| *a == AlgebraConcept::SigmaMigration && *b == AlgebraConcept::PiMigration)
    }
}

/// Coproduct and Product are dual (colimit vs limit).
#[derive(Debug)]
pub struct CoproductProductDual;

impl Axiom for CoproductProductDual {
    fn description(&self) -> &str {
        "Coproduct ⊥ Product: union and intersection are dual (Zimmermann 2006)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::opposition::OppositionDef;
        AlgebraOpposition::pairs()
            .iter()
            .any(|(a, b)| *a == AlgebraConcept::Coproduct && *b == AlgebraConcept::Product)
    }
}

impl Ontology for AlgebraOntology {
    type Cat = AlgebraCategory;
    type Qual = IsOperation;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        AlgebraOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(AdjointTriple), Box::new(CoproductProductDual)]
    }
}
