use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::linear_algebra::decomposition;
use crate::formal::math::linear_algebra::determinant;
use crate::formal::math::linear_algebra::eigenvalue;
use crate::formal::math::linear_algebra::matrix::{self, Matrix};
use crate::formal::math::linear_algebra::positive_definite;
use crate::formal::math::linear_algebra::vector_space::Vector;

// ---------------------------------------------------------------------------
// Entity: algebraic structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum AlgebraicStructure {
    Scalar,
    Vector,
    Matrix,
    SymmetricMatrix,
    PositiveDefiniteMatrix,
    DiagonalMatrix,
    IdentityMatrix,
    LowerTriangular,
    UpperTriangular,
}

define_ontology! {
    /// Discrete category over algebraic structure entities.
    pub LinearAlgebraOntology for LinearAlgebraCategory {
        concepts: AlgebraicStructure,
        relation: AlgebraicRelation,
    }
}

#[derive(Debug, Clone)]
pub struct StructureDimension;

impl Quality for StructureDimension {
    type Individual = AlgebraicStructure;
    type Value = &'static str;

    fn get(&self, s: &AlgebraicStructure) -> Option<&'static str> {
        Some(match s {
            AlgebraicStructure::Scalar => "0 (field element)",
            AlgebraicStructure::Vector => "n (n-dimensional)",
            AlgebraicStructure::Matrix => "n×m",
            AlgebraicStructure::SymmetricMatrix => "n×n, n(n+1)/2 free",
            AlgebraicStructure::PositiveDefiniteMatrix => "n×n, n(n+1)/2 free, all λ>0",
            AlgebraicStructure::DiagonalMatrix => "n×n, n free",
            AlgebraicStructure::IdentityMatrix => "n×n, 0 free",
            AlgebraicStructure::LowerTriangular => "n×n, n(n+1)/2 free",
            AlgebraicStructure::UpperTriangular => "n×n, n(n+1)/2 free",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Matrix multiplication is associative: (AB)C = A(BC).
pub struct MultiplicationAssociativity;

impl Axiom for MultiplicationAssociativity {
    fn description(&self) -> &str {
        "matrix multiplication is associative: (AB)C = A(BC)"
    }

    fn holds(&self) -> bool {
        for (a, b, c) in &canonical_matrix_triples() {
            let ab_c = a.multiply(b).multiply(c);
            let a_bc = a.multiply(&b.multiply(c));
            if !matrix::approx_eq(&ab_c, &a_bc, 1e-8) {
                return false;
            }
        }
        true
    }
}

/// Identity matrix: AI = IA = A.
pub struct MultiplicationIdentity;

impl Axiom for MultiplicationIdentity {
    fn description(&self) -> &str {
        "identity matrix: AI = IA = A"
    }

    fn holds(&self) -> bool {
        for m in &canonical_square_matrices() {
            let n = m.rows;
            let i = Matrix::identity(n);
            if !matrix::approx_eq(&m.multiply(&i), m, 1e-12) {
                return false;
            }
            if !matrix::approx_eq(&i.multiply(m), m, 1e-12) {
                return false;
            }
        }
        true
    }
}

/// Transpose of transpose: (A^T)^T = A.
pub struct TransposeInvolution;

impl Axiom for TransposeInvolution {
    fn description(&self) -> &str {
        "(A^T)^T = A (transpose is an involution)"
    }

    fn holds(&self) -> bool {
        for m in &canonical_square_matrices() {
            if !matrix::approx_eq(&m.transpose().transpose(), m, 1e-15) {
                return false;
            }
        }
        true
    }
}

/// Transpose of product: (AB)^T = B^T A^T.
pub struct TransposeProduct;

impl Axiom for TransposeProduct {
    fn description(&self) -> &str {
        "(AB)^T = B^T A^T"
    }

    fn holds(&self) -> bool {
        let matrices = canonical_square_matrices();
        for a in &matrices {
            for b in &matrices {
                if a.cols != b.rows {
                    continue;
                }
                let lhs = a.multiply(b).transpose();
                let rhs = b.transpose().multiply(&a.transpose());
                if !matrix::approx_eq(&lhs, &rhs, 1e-10) {
                    return false;
                }
            }
        }
        true
    }
}

/// det(I) = 1 (normalization axiom of determinant).
pub struct DetNormalization;

impl Axiom for DetNormalization {
    fn description(&self) -> &str {
        "det(I) = 1 (determinant normalization axiom)"
    }

    fn holds(&self) -> bool {
        for n in 1..=5 {
            let i = Matrix::identity(n);
            if (determinant::det(&i) - 1.0).abs() > 1e-15 {
                return false;
            }
        }
        true
    }
}

/// det(AB) = det(A) * det(B) (multiplicativity).
pub struct DetMultiplicativity;

impl Axiom for DetMultiplicativity {
    fn description(&self) -> &str {
        "det(AB) = det(A) * det(B) (multiplicativity)"
    }

    fn holds(&self) -> bool {
        let matrices = canonical_square_matrices();
        for a in &matrices {
            for b in &matrices {
                if a.rows != b.rows {
                    continue;
                }
                let lhs = determinant::det(&a.multiply(b));
                let rhs = determinant::det(a) * determinant::det(b);
                if (lhs - rhs).abs() > 1e-6 {
                    return false;
                }
            }
        }
        true
    }
}

/// det(A^T) = det(A) (transpose invariance).
pub struct DetTranspose;

impl Axiom for DetTranspose {
    fn description(&self) -> &str {
        "det(A^T) = det(A) (transpose invariance)"
    }

    fn holds(&self) -> bool {
        for m in &canonical_square_matrices() {
            let d = determinant::det(m);
            let dt = determinant::det(&m.transpose());
            if (d - dt).abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}

/// Trace is the sum of eigenvalues.
pub struct TraceEigenvalueSum;

impl Axiom for TraceEigenvalueSum {
    fn description(&self) -> &str {
        "tr(A) = sum of eigenvalues"
    }

    fn holds(&self) -> bool {
        for m in &canonical_symmetric_matrices() {
            let tr = m.trace();
            let evs = eigenvalue::eigenvalues_symmetric(m);
            let ev_sum: f64 = evs.iter().sum();
            if (tr - ev_sum).abs() > 1e-6 {
                return false;
            }
        }
        true
    }
}

/// Determinant is the product of eigenvalues.
pub struct DetEigenvalueProduct;

impl Axiom for DetEigenvalueProduct {
    fn description(&self) -> &str {
        "det(A) = product of eigenvalues"
    }

    fn holds(&self) -> bool {
        for m in &canonical_symmetric_matrices() {
            let d = determinant::det(m);
            let evs = eigenvalue::eigenvalues_symmetric(m);
            let ev_prod: f64 = evs.iter().product();
            if (d - ev_prod).abs() > 1e-4 {
                return false;
            }
        }
        true
    }
}

/// Cholesky: A = L L^T for PD matrices.
pub struct CholeskyFactorization;

impl Axiom for CholeskyFactorization {
    fn description(&self) -> &str {
        "A = L L^T for positive definite matrices (Cholesky)"
    }

    fn holds(&self) -> bool {
        for m in &canonical_pd_matrices() {
            let l = match decomposition::cholesky(m) {
                Some(l) => l,
                None => return false,
            };
            let reconstructed = l.multiply(&l.transpose());
            if !matrix::approx_eq(m, &reconstructed, 1e-10) {
                return false;
            }
        }
        true
    }
}

/// PSD: x^T A x >= 0 for all x.
pub struct PsdQuadraticForm;

impl Axiom for PsdQuadraticForm {
    fn description(&self) -> &str {
        "x^T A x >= 0 for PSD matrices (positive semi-definiteness)"
    }

    fn holds(&self) -> bool {
        let test_vectors = canonical_vectors();
        for m in &canonical_pd_matrices() {
            for x in &test_vectors {
                if x.dim() != m.rows {
                    continue;
                }
                let q = positive_definite::quadratic_form(m, x);
                if q < -1e-10 {
                    return false;
                }
            }
        }
        true
    }
}

/// Joseph form update preserves PSD.
pub struct JosephPreservesPsd;

impl Axiom for JosephPreservesPsd {
    fn description(&self) -> &str {
        "Joseph form covariance update preserves positive semi-definiteness"
    }

    fn holds(&self) -> bool {
        // 2x2 test case
        let p = Matrix::new(2, 2, vec![4.0, 1.0, 1.0, 3.0]);
        let h = Matrix::new(1, 2, vec![1.0, 0.0]);
        let r = Matrix::new(1, 1, vec![1.0]);
        // K = P H^T (H P H^T + R)^{-1}
        let pht = p.multiply(&h.transpose());
        let s = h.multiply(&pht).add(&r);
        let s_inv = 1.0 / s.get(0, 0);
        let k = pht.scale(s_inv);
        let p_new = positive_definite::joseph_update(&p, &k, &h, &r);
        positive_definite::is_positive_semidefinite(&p_new)
    }
}

impl Ontology for LinearAlgebraOntology {
    type Cat = LinearAlgebraCategory;
    type Qual = StructureDimension;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(MultiplicationAssociativity),
            Box::new(MultiplicationIdentity),
            Box::new(TransposeInvolution),
            Box::new(TransposeProduct),
            Box::new(DetNormalization),
            Box::new(DetMultiplicativity),
            Box::new(DetTranspose),
            Box::new(TraceEigenvalueSum),
            Box::new(DetEigenvalueProduct),
            Box::new(CholeskyFactorization),
            Box::new(PsdQuadraticForm),
            Box::new(JosephPreservesPsd),
        ]
    }
}

// ---------------------------------------------------------------------------
// Canonical test data
// ---------------------------------------------------------------------------

fn canonical_square_matrices() -> Vec<Matrix> {
    vec![
        Matrix::identity(2),
        Matrix::identity(3),
        Matrix::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]),
        Matrix::new(2, 2, vec![2.0, 1.0, 1.0, 3.0]),
        Matrix::new(3, 3, vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0, 5.0]),
        Matrix::diagonal(&[2.0, 3.0, 5.0]),
    ]
}

fn canonical_symmetric_matrices() -> Vec<Matrix> {
    vec![
        Matrix::identity(2),
        Matrix::identity(3),
        Matrix::new(2, 2, vec![2.0, 1.0, 1.0, 3.0]),
        Matrix::new(3, 3, vec![4.0, 2.0, 1.0, 2.0, 5.0, 3.0, 1.0, 3.0, 6.0]),
        Matrix::diagonal(&[1.0, 2.0, 3.0]),
    ]
}

fn canonical_pd_matrices() -> Vec<Matrix> {
    vec![
        Matrix::identity(2),
        Matrix::identity(3),
        Matrix::new(2, 2, vec![2.0, 1.0, 1.0, 3.0]),
        Matrix::new(3, 3, vec![4.0, 2.0, 1.0, 2.0, 5.0, 3.0, 1.0, 3.0, 6.0]),
        Matrix::diagonal(&[1.0, 2.0, 3.0]),
        Matrix::diagonal(&[10.0, 20.0]),
    ]
}

fn canonical_matrix_triples() -> Vec<(Matrix, Matrix, Matrix)> {
    let a = Matrix::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]);
    let b = Matrix::new(2, 2, vec![2.0, 0.0, 1.0, 3.0]);
    let c = Matrix::new(2, 2, vec![1.0, 1.0, 0.0, 2.0]);
    let d = Matrix::new(3, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0]);
    let e = Matrix::new(3, 3, vec![2.0, 0.0, 1.0, 0.0, 3.0, 0.0, 1.0, 0.0, 4.0]);
    let f = Matrix::diagonal(&[1.0, 2.0, 3.0]);
    vec![(a, b, c), (d, e, f)]
}

fn canonical_vectors() -> Vec<Vector> {
    vec![
        Vector::new(vec![1.0, 0.0]),
        Vector::new(vec![0.0, 1.0]),
        Vector::new(vec![1.0, 1.0]),
        Vector::new(vec![-1.0, 2.0]),
        Vector::new(vec![1.0, 0.0, 0.0]),
        Vector::new(vec![0.0, 1.0, 0.0]),
        Vector::new(vec![1.0, 2.0, 3.0]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<LinearAlgebraCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        LinearAlgebraOntology::validate().unwrap();
    }
}
