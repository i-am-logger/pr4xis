//! Linear-algebra ontology — algebraic-structure taxonomy plus the
//! canonical matrix-algebra axioms.
//!
//! The nine concepts model the standard subtype hierarchy of matrix
//! algebra used in numerical computation (Strang 2016, Golub & Van Loan
//! 2013): scalars, vectors, and the matrix family — general matrices,
//! the symmetric and positive-definite specialisations, diagonal,
//! identity, and the two triangular forms.
//!
//! # Literature
//!
//! - **Strang (2016)** *Introduction to Linear Algebra*, 5th ed. —
//!   foundational treatment of the matrix subtype hierarchy, vector
//!   spaces, determinants, and eigenvalues.
//! - **Golub & Van Loan (2013)** *Matrix Computations*, 4th ed. — the
//!   numerical-linear-algebra reference: Cholesky factorisation (§4.2),
//!   determinant invariants (§14.6), Joseph-form covariance update.
//! - **Horn & Johnson (2013)** *Matrix Analysis*, 2nd ed. — the
//!   positive-definite quadratic-form characterisation and trace /
//!   eigenvalue identities.

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::linear_algebra::decomposition;
use crate::formal::math::linear_algebra::determinant;
use crate::formal::math::linear_algebra::eigenvalue;
use crate::formal::math::linear_algebra::matrix::{self, Matrix};
use crate::formal::math::linear_algebra::positive_definite;
use crate::formal::math::linear_algebra::vector_space::Vector;

pr4xis::ontology! {
    name: "LinearAlgebra",
    source: "Strang (2016) Introduction to Linear Algebra; Golub & Van Loan (2013) Matrix Computations; Horn & Johnson (2013) Matrix Analysis",

    concepts: [
        Scalar,
        Vector,
        Matrix,
        SymmetricMatrix,
        PositiveDefiniteMatrix,
        DiagonalMatrix,
        IdentityMatrix,
        LowerTriangular,
        UpperTriangular,
    ],

    labels: {
        Scalar: ("en", "Scalar",
            "Strang (2016) Ch. 1: a single field element (real or complex), the 0-dimensional ground for vectors and matrices."),
        Vector: ("en", "Vector",
            "Strang (2016) Ch. 1: an n-tuple in F^n; the basic object of a vector space (Kahan's vector-space axioms)."),
        Matrix: ("en", "Matrix",
            "Strang (2016) Ch. 2: a rectangular array of field elements; a linear map between finite-dimensional vector spaces."),
        SymmetricMatrix: ("en", "Symmetric matrix",
            "Strang (2016) §5.1: a square matrix A with A = A^T; the spectral theorem guarantees real eigenvalues and an orthonormal eigenbasis."),
        PositiveDefiniteMatrix: ("en", "Positive-definite matrix",
            "Horn & Johnson (2013) §7.1: a symmetric matrix A with x^T A x > 0 for all x != 0; equivalently, all eigenvalues are strictly positive."),
        DiagonalMatrix: ("en", "Diagonal matrix",
            "Strang (2016) §1.4: a square matrix with non-zero entries only on the main diagonal; diagonal matrices commute and form an abelian sub-ring."),
        IdentityMatrix: ("en", "Identity matrix",
            "Strang (2016) §1.4: the diagonal matrix with all 1s on the diagonal; the multiplicative identity I with AI = IA = A."),
        LowerTriangular: ("en", "Lower-triangular matrix",
            "Golub & Van Loan (2013) §1.3.5: a square matrix whose entries above the main diagonal are zero; the Cholesky factor L."),
        UpperTriangular: ("en", "Upper-triangular matrix",
            "Golub & Van Loan (2013) §1.3.5: a square matrix whose entries below the main diagonal are zero; the back-substitution target in solve(Ux = b)."),
    },

    is_a: [
        // Strang (2016) §5.1 / Horn & Johnson §7.1 subtype chain.
        (PositiveDefiniteMatrix, SymmetricMatrix),
        (SymmetricMatrix, Matrix),
        (DiagonalMatrix, SymmetricMatrix),
        (IdentityMatrix, DiagonalMatrix),
        (IdentityMatrix, PositiveDefiniteMatrix),
        (DiagonalMatrix, LowerTriangular),
        (DiagonalMatrix, UpperTriangular),
        (LowerTriangular, Matrix),
        (UpperTriangular, Matrix),
    ],
}

/// Quality: free-parameter count (degrees of freedom) for each
/// algebraic-structure subtype — Strang (2016) Ch. 5 counts.
#[derive(Debug, Clone)]
pub struct StructureDimension;

impl Quality for StructureDimension {
    type Individual = LinearAlgebraConcept;
    type Value = &'static str;

    fn get(&self, s: &LinearAlgebraConcept) -> Option<&'static str> {
        use LinearAlgebraConcept as L;
        Some(match s {
            L::Scalar => "0 (field element)",
            L::Vector => "n (n-dimensional)",
            L::Matrix => "n x m",
            L::SymmetricMatrix => "n x n, n(n+1)/2 free",
            L::PositiveDefiniteMatrix => "n x n, n(n+1)/2 free, all eigenvalues > 0",
            L::DiagonalMatrix => "n x n, n free",
            L::IdentityMatrix => "n x n, 0 free",
            L::LowerTriangular => "n x n, n(n+1)/2 free",
            L::UpperTriangular => "n x n, n(n+1)/2 free",
        })
    }
}

impl Ontology for LinearAlgebraOntology {
    type Cat = LinearAlgebraCategory;
    type Qual = StructureDimension;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(MultiplicationAssociativity));
        axioms.push(Box::new(MultiplicationIdentity));
        axioms.push(Box::new(TransposeInvolution));
        axioms.push(Box::new(TransposeProduct));
        axioms.push(Box::new(DetNormalization));
        axioms.push(Box::new(DetMultiplicativity));
        axioms.push(Box::new(DetTranspose));
        axioms.push(Box::new(TraceEigenvalueSum));
        axioms.push(Box::new(DetEigenvalueProduct));
        axioms.push(Box::new(CholeskyFactorization));
        axioms.push(Box::new(PsdQuadraticForm));
        axioms.push(Box::new(JosephPreservesPsd));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Domain axioms — matrix algebra, determinants, eigenvalues, PD theory.
// ---------------------------------------------------------------------------

/// Matrix multiplication is associative: (AB)C = A(BC).
/// Strang (2016) §1.4.
pub struct MultiplicationAssociativity;

impl Axiom for MultiplicationAssociativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for (a, b, c) in &canonical_matrix_triples() {
            let ab_c = a.multiply(b).multiply(c);
            let a_bc = a.multiply(&b.multiply(c));
            if !matrix::approx_eq(&ab_c, &a_bc, 1e-8) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MultiplicationAssociativity",
        "matrix multiplication is associative: (AB)C = A(BC)",
        "Strang (2016) Introduction to Linear Algebra §1.4"
    );
}

pr4xis::register_axiom!(
    MultiplicationAssociativity,
    "Strang (2016) Introduction to Linear Algebra §1.4"
);

/// Identity matrix: AI = IA = A. Strang (2016) §1.4.
pub struct MultiplicationIdentity;

impl Axiom for MultiplicationIdentity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for m in &canonical_square_matrices() {
            let n = m.rows;
            let i = Matrix::identity(n);
            if !matrix::approx_eq(&m.multiply(&i), m, 1e-12)
                || !matrix::approx_eq(&i.multiply(m), m, 1e-12)
            {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MultiplicationIdentity",
        "identity matrix: AI = IA = A",
        "Strang (2016) Introduction to Linear Algebra §1.4"
    );
}

pr4xis::register_axiom!(
    MultiplicationIdentity,
    "Strang (2016) Introduction to Linear Algebra §1.4"
);

/// Transpose is an involution: (A^T)^T = A. Strang (2016) §2.7.
pub struct TransposeInvolution;

impl Axiom for TransposeInvolution {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for m in &canonical_square_matrices() {
            if !matrix::approx_eq(&m.transpose().transpose(), m, 1e-15) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "TransposeInvolution",
        "(A^T)^T = A (transpose is an involution)",
        "Strang (2016) Introduction to Linear Algebra §2.7"
    );
}

pr4xis::register_axiom!(
    TransposeInvolution,
    "Strang (2016) Introduction to Linear Algebra §2.7"
);

/// Transpose distributes over product (reversing order):
/// (AB)^T = B^T A^T. Strang (2016) §2.7.
pub struct TransposeProduct;

impl Axiom for TransposeProduct {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let matrices = canonical_square_matrices();
        for a in &matrices {
            for b in &matrices {
                if a.cols != b.rows {
                    continue;
                }
                let lhs = a.multiply(b).transpose();
                let rhs = b.transpose().multiply(&a.transpose());
                if !matrix::approx_eq(&lhs, &rhs, 1e-10) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "TransposeProduct",
        "(AB)^T = B^T A^T",
        "Strang (2016) Introduction to Linear Algebra §2.7"
    );
}

pr4xis::register_axiom!(
    TransposeProduct,
    "Strang (2016) Introduction to Linear Algebra §2.7"
);

/// Determinant normalisation: det(I) = 1.
/// Golub & Van Loan (2013) §14.6.
pub struct DetNormalization;

impl Axiom for DetNormalization {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for n in 1..=5 {
            let i = Matrix::identity(n);
            if (determinant::det(&i) - 1.0).abs() > 1e-15 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DetNormalization",
        "det(I) = 1 (determinant normalisation)",
        "Golub & Van Loan (2013) Matrix Computations §14.6"
    );
}

pr4xis::register_axiom!(
    DetNormalization,
    "Golub & Van Loan (2013) Matrix Computations §14.6"
);

/// Determinant multiplicativity: det(AB) = det(A) det(B).
/// Strang (2016) §5.1.
pub struct DetMultiplicativity;

impl Axiom for DetMultiplicativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let matrices = canonical_square_matrices();
        for a in &matrices {
            for b in &matrices {
                if a.rows != b.rows {
                    continue;
                }
                let lhs = determinant::det(&a.multiply(b));
                let rhs = determinant::det(a) * determinant::det(b);
                if (lhs - rhs).abs() > 1e-6 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DetMultiplicativity",
        "det(AB) = det(A) * det(B)",
        "Strang (2016) Introduction to Linear Algebra §5.1"
    );
}

pr4xis::register_axiom!(
    DetMultiplicativity,
    "Strang (2016) Introduction to Linear Algebra §5.1"
);

/// Determinant transpose invariance: det(A^T) = det(A).
/// Strang (2016) §5.1.
pub struct DetTranspose;

impl Axiom for DetTranspose {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for m in &canonical_square_matrices() {
            let d = determinant::det(m);
            let dt = determinant::det(&m.transpose());
            if (d - dt).abs() > 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DetTranspose",
        "det(A^T) = det(A) (transpose invariance)",
        "Strang (2016) Introduction to Linear Algebra §5.1"
    );
}

pr4xis::register_axiom!(
    DetTranspose,
    "Strang (2016) Introduction to Linear Algebra §5.1"
);

/// Trace is the sum of eigenvalues: tr(A) = Σλ_i.
/// Horn & Johnson (2013) §1.2.
pub struct TraceEigenvalueSum;

impl Axiom for TraceEigenvalueSum {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for m in &canonical_symmetric_matrices() {
            let tr = m.trace();
            let evs = eigenvalue::eigenvalues_symmetric(m);
            let ev_sum: f64 = evs.iter().sum();
            if (tr - ev_sum).abs() > 1e-6 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "TraceEigenvalueSum",
        "tr(A) = sum of eigenvalues",
        "Horn & Johnson (2013) Matrix Analysis §1.2"
    );
}

pr4xis::register_axiom!(
    TraceEigenvalueSum,
    "Horn & Johnson (2013) Matrix Analysis §1.2"
);

/// Determinant is the product of eigenvalues: det(A) = Πλ_i.
/// Horn & Johnson (2013) §1.2.
pub struct DetEigenvalueProduct;

impl Axiom for DetEigenvalueProduct {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for m in &canonical_symmetric_matrices() {
            let d = determinant::det(m);
            let evs = eigenvalue::eigenvalues_symmetric(m);
            let ev_prod: f64 = evs.iter().product();
            if (d - ev_prod).abs() > 1e-4 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DetEigenvalueProduct",
        "det(A) = product of eigenvalues",
        "Horn & Johnson (2013) Matrix Analysis §1.2"
    );
}

pr4xis::register_axiom!(
    DetEigenvalueProduct,
    "Horn & Johnson (2013) Matrix Analysis §1.2"
);

/// Cholesky factorisation: every symmetric positive-definite matrix A
/// admits a unique decomposition A = LL^T with L lower-triangular and
/// having positive diagonal entries. Golub & Van Loan (2013) §4.2.
pub struct CholeskyFactorization;

impl Axiom for CholeskyFactorization {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for m in &canonical_pd_matrices() {
            let Some(l) = decomposition::cholesky(m) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            let reconstructed = l.multiply(&l.transpose());
            if !matrix::approx_eq(m, &reconstructed, 1e-10) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "CholeskyFactorization",
        "A = L L^T for symmetric positive-definite matrices (Cholesky factorisation)",
        "Golub & Van Loan (2013) Matrix Computations §4.2"
    );
}

pr4xis::register_axiom!(
    CholeskyFactorization,
    "Golub & Van Loan (2013) Matrix Computations §4.2"
);

/// Positive (semi-)definite quadratic form: x^T A x ≥ 0 for all x
/// when A is PSD. Horn & Johnson (2013) §7.1.
pub struct PsdQuadraticForm;

impl Axiom for PsdQuadraticForm {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let test_vectors = canonical_vectors();
        for m in &canonical_pd_matrices() {
            for x in &test_vectors {
                if x.dim() != m.rows {
                    continue;
                }
                let q = positive_definite::quadratic_form(m, x);
                if q < -1e-10 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PsdQuadraticForm",
        "x^T A x >= 0 for PSD matrices",
        "Horn & Johnson (2013) Matrix Analysis §7.1"
    );
}

pr4xis::register_axiom!(
    PsdQuadraticForm,
    "Horn & Johnson (2013) Matrix Analysis §7.1"
);

/// Joseph-form covariance update preserves positive-(semi-)definiteness.
/// Bucy & Joseph (1968) *Filtering for Stochastic Processes with
/// Applications to Guidance*, §4.2 — the numerically-stable Kalman
/// covariance update P_+ = (I - KH) P (I - KH)^T + K R K^T.
pub struct JosephPreservesPsd;

impl Axiom for JosephPreservesPsd {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let p = Matrix::new(2, 2, vec![4.0, 1.0, 1.0, 3.0]);
        let h = Matrix::new(1, 2, vec![1.0, 0.0]);
        let r = Matrix::new(1, 1, vec![1.0]);
        let pht = p.multiply(&h.transpose());
        let s = h.multiply(&pht).add(&r);
        let s_inv = 1.0 / s.get(0, 0);
        let k = pht.scale(s_inv);
        let p_new = positive_definite::joseph_update(&p, &k, &h, &r);
        if positive_definite::is_positive_semidefinite(&p_new) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "JosephPreservesPsd",
        "Joseph form covariance update preserves positive semi-definiteness",
        "Bucy & Joseph (1968) Filtering for Stochastic Processes with Applications to Guidance §4.2"
    );
}

pr4xis::register_axiom!(
    JosephPreservesPsd,
    "Bucy & Joseph (1968) Filtering for Stochastic Processes with Applications to Guidance §4.2"
);

// ---------------------------------------------------------------------------
// Canonical test data — small fixed sample sets for axiom verification.
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
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<LinearAlgebraCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        LinearAlgebraOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nine_concepts() {
        assert_eq!(LinearAlgebraConcept::variants().len(), 9);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn structure_dimension_total() {
        let q = StructureDimension;
        for c in LinearAlgebraConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing dimension", c);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn matrix_axioms_hold() {
        assert!(MultiplicationAssociativity.verify().is_ok());
        assert!(MultiplicationIdentity.verify().is_ok());
        assert!(TransposeInvolution.verify().is_ok());
        assert!(TransposeProduct.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn det_axioms_hold() {
        assert!(DetNormalization.verify().is_ok());
        assert!(DetMultiplicativity.verify().is_ok());
        assert!(DetTranspose.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn eigenvalue_axioms_hold() {
        assert!(TraceEigenvalueSum.verify().is_ok());
        assert!(DetEigenvalueProduct.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn pd_axioms_hold() {
        assert!(CholeskyFactorization.verify().is_ok());
        assert!(PsdQuadraticForm.verify().is_ok());
        assert!(JosephPreservesPsd.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = LinearAlgebraConcept> {
        proptest::sample::select(LinearAlgebraConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_structure_dimension_total(c in arb_concept()) {
            prop_assert!(StructureDimension.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            use pr4xis::category::{Arrow, Category};
            for m in LinearAlgebraCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in LinearAlgebraOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_structure_dimension_total, Verifiable);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
