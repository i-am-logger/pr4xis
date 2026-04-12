//! The linear algebra ontology — vector spaces, matrices, determinants, eigenvalues, positive definiteness
pub mod decomposition;
pub mod determinant;
pub mod eigenvalue;
pub mod matrix;
pub mod ontology;
pub mod positive_definite;
pub mod vector_space;

#[cfg(test)]
mod tests;
