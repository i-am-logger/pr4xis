/// Science of structural analysis methods.
///
/// Formalizes the reasoning processes used to analyze structure in data
/// and extract patterns, clusters, and anomalies. Also provides the
/// *runtime* counterparts: a `FormalContext`/`ConceptLattice`
/// implementation using Ganter's NextClosure algorithm, and a
/// `ConceptLatticeFibration` over the Classification ontology
/// (Grothendieck 1971 / Jacobs 1999).
///
/// Grounded in:
/// - Wille 1982: Formal Concept Analysis (concept lattices from binary relations)
/// - Ganter & Wille 1999: Formal Concept Analysis (comprehensive treatment)
/// - Ganter 1984: NextClosure algorithm
/// - Birkhoff 1940: Lattice Theory (algebraic foundations)
/// - Davey & Priestley 2002: Hasse diagrams, transitive reduction
/// - Grothendieck 1971 / Jacobs 1999: fibrations and the lift from a
///   concept lattice to the Classification taxonomy
pub mod classification_fibration;
pub mod fca;
pub mod ontology;

pub use classification_fibration::ConceptLatticeFibration;
pub use fca::{BitSet, ConceptLattice, FormalConcept, FormalContext};
