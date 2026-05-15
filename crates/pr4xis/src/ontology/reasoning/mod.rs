pub mod analogy;
pub mod catalog;
pub mod context;
pub mod ontology;
pub mod structural;

pub use ontology::{ReasoningCategory, ReasoningConcept, ReasoningOntology, ReasoningTradition};

pub use analogy::Analogy;
pub use catalog::structural_axioms_for;
pub use structural::{
    AntisymmetricOnKind, AsymmetricOnKind, IrreflexiveOnKind, NoCyclesOnKind, SymmetricOnKind,
};
