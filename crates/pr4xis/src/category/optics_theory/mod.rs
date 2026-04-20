//! Optics — substrate ontology grounding `category/optics.rs`. Names
//! the hierarchy (Optic, Lens, Prism, Iso, Traversal, Getter, Setter,
//! Fold, Optional, Review, Profunctor optics) per Foster et al. (2007),
//! van Laarhoven (2009), Kmett's lens library, Pickering-Gibbons-Wu (2017).

pub mod ontology;

pub use ontology::{OpticsCategory, OpticsConcept, OpticsLineage, OpticsOntology};
