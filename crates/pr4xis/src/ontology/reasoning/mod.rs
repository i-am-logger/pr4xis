pub mod analogy;
pub mod catalog;
pub mod causation;
pub mod context;
pub mod equivalence;
pub mod mereology;
pub mod opposition;
pub mod structural;
pub mod taxonomy;

pub use analogy::Analogy;
pub use catalog::structural_axioms_for;
pub use structural::{
    AntisymmetricOnKind, AsymmetricOnKind, IrreflexiveOnKind, NoCyclesOnKind, SymmetricOnKind,
};
