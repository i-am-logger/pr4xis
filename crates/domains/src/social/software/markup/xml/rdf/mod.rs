pub mod canon;
pub mod ontology;
pub mod reader;
pub mod term;

pub use canon::{
    CanonError, CanonLimits, HashAlgorithm, Quad, canonicalize, canonicalize_nquads,
    canonicalize_with,
};
pub use ontology::*;
pub use reader::{RdfReadError, read_rdf_xml};
pub use term::{RdfTerm, Triple};

#[cfg(test)]
mod tests;
