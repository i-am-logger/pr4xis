use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::science::math::linear_algebra::ontology::*;

#[test]
fn linear_algebra_category_laws() {
    check_category_laws::<LinearAlgebraCategory>().unwrap();
}

#[test]
fn linear_algebra_ontology_validates() {
    LinearAlgebraOntology::validate().unwrap();
}

#[test]
fn multiplication_associativity() {
    assert!(MultiplicationAssociativity.holds());
}

#[test]
fn multiplication_identity() {
    assert!(MultiplicationIdentity.holds());
}

#[test]
fn transpose_involution() {
    assert!(TransposeInvolution.holds());
}

#[test]
fn transpose_product() {
    assert!(TransposeProduct.holds());
}

#[test]
fn det_normalization() {
    assert!(DetNormalization.holds());
}

#[test]
fn det_multiplicativity() {
    assert!(DetMultiplicativity.holds());
}

#[test]
fn det_transpose() {
    assert!(DetTranspose.holds());
}

#[test]
fn trace_eigenvalue_sum() {
    assert!(TraceEigenvalueSum.holds());
}

#[test]
fn det_eigenvalue_product() {
    assert!(DetEigenvalueProduct.holds());
}

#[test]
fn cholesky_factorization() {
    assert!(CholeskyFactorization.holds());
}

#[test]
fn psd_quadratic_form() {
    assert!(PsdQuadraticForm.holds());
}

#[test]
fn joseph_preserves_psd() {
    assert!(JosephPreservesPsd.holds());
}

// ---------------------------------------------------------------------------
// H4: solve_lower/upper_triangular return None on zero diagonal
// ---------------------------------------------------------------------------

#[test]
fn solve_lower_triangular_zero_diagonal_returns_none() {
    use crate::science::math::linear_algebra::decomposition;
    use crate::science::math::linear_algebra::matrix::Matrix;
    // Lower triangular with a zero on the diagonal
    let l = Matrix::new(2, 2, vec![1.0, 0.0, 1.0, 0.0]);
    let b = vec![1.0, 1.0];
    let result = decomposition::solve_lower_triangular(&l, &b);
    assert!(result.is_none(), "zero diagonal should return None");
}

#[test]
fn solve_upper_triangular_zero_diagonal_returns_none() {
    use crate::science::math::linear_algebra::decomposition;
    use crate::science::math::linear_algebra::matrix::Matrix;
    // Lower triangular with a zero on the diagonal (used as L^T)
    let l = Matrix::new(2, 2, vec![0.0, 0.0, 1.0, 1.0]);
    let b = vec![1.0, 1.0];
    let result = decomposition::solve_upper_triangular(&l, &b);
    assert!(result.is_none(), "zero diagonal should return None");
}

#[test]
fn solve_spd_singular_returns_none() {
    use crate::science::math::linear_algebra::decomposition;
    use crate::science::math::linear_algebra::matrix::Matrix;
    // Singular matrix (not PD)
    let a = Matrix::new(2, 2, vec![1.0, 1.0, 1.0, 1.0]);
    let b = vec![1.0, 1.0];
    let result = decomposition::solve_spd(&a, &b);
    assert!(result.is_none(), "singular matrix should return None");
}

// ---------------------------------------------------------------------------
// H5: QR eigenvalue non-convergence returns NaN
// ---------------------------------------------------------------------------

#[test]
fn eigenvalues_symmetric_converges_for_well_conditioned() {
    use crate::science::math::linear_algebra::eigenvalue;
    use crate::science::math::linear_algebra::matrix::Matrix;
    let m = Matrix::new(3, 3, vec![4.0, 1.0, 0.0, 1.0, 3.0, 1.0, 0.0, 1.0, 2.0]);
    let evs = eigenvalue::eigenvalues_symmetric(&m);
    // Should converge — no NaN
    for ev in &evs {
        assert!(
            !ev.is_nan(),
            "eigenvalue should not be NaN for well-conditioned matrix"
        );
    }
    // Trace should equal sum of eigenvalues
    let trace = m.trace();
    let ev_sum: f64 = evs.iter().sum();
    assert!(
        (trace - ev_sum).abs() < 1e-6,
        "trace={} vs ev_sum={}",
        trace,
        ev_sum
    );
}

#[test]
fn is_positive_definite_uses_cholesky() {
    use crate::science::math::linear_algebra::matrix::Matrix;
    use crate::science::math::linear_algebra::positive_definite;
    // This matrix is M^T M + 0.1I and is guaranteed PD.
    // QR eigenvalue iteration may not converge for it, but Cholesky-based
    // PD check handles it correctly.
    let m = Matrix::new(
        3,
        3,
        vec![
            37.694766390947535,
            7.93367450966257,
            4.664130987929606,
            7.93367450966257,
            13.917510803132062,
            3.737703167011621,
            4.664130987929606,
            3.737703167011621,
            29.151709686707342,
        ],
    );
    assert!(
        positive_definite::is_positive_definite(&m),
        "M^T M + 0.1I must be PD"
    );
}

#[cfg(test)]
mod proptest_proofs {
    use crate::science::math::linear_algebra::decomposition;
    use crate::science::math::linear_algebra::determinant;
    use crate::science::math::linear_algebra::matrix::{self, Matrix};
    use crate::science::math::linear_algebra::positive_definite;
    use crate::science::math::linear_algebra::vector_space::Vector;
    use proptest::prelude::*;

    fn arb_vec(n: usize) -> impl Strategy<Value = Vector> {
        proptest::collection::vec(-10.0..10.0_f64, n).prop_map(Vector::new)
    }

    /// Generate a random PD matrix: A = M^T M + εI (always PD for ε > 0).
    fn arb_pd_matrix(n: usize) -> impl Strategy<Value = Matrix> {
        proptest::collection::vec(-5.0..5.0_f64, n * n).prop_map(move |data| {
            let m = Matrix::new(n, n, data);
            let mt = m.transpose();
            mt.multiply(&m).add(&Matrix::identity(n).scale(0.1))
        })
    }

    fn arb_square_matrix(n: usize) -> impl Strategy<Value = Matrix> {
        proptest::collection::vec(-10.0..10.0_f64, n * n)
            .prop_map(move |data| Matrix::new(n, n, data))
    }

    proptest! {
        // --- Vector space axioms ---

        #[test]
        fn vec_addition_commutativity(u in arb_vec(3), v in arb_vec(3)) {
            let uv = u.add(&v);
            let vu = v.add(&u);
            prop_assert!(uv == vu);
        }

        #[test]
        fn vec_addition_associativity(u in arb_vec(3), v in arb_vec(3), w in arb_vec(3)) {
            let lhs = u.add(&v).add(&w);
            let rhs = u.add(&v.add(&w));
            for i in 0..3 {
                prop_assert!((lhs.get(i) - rhs.get(i)).abs() < 1e-10);
            }
        }

        #[test]
        fn vec_additive_identity(v in arb_vec(3)) {
            let sum = v.add(&Vector::zero(3));
            prop_assert!(sum == v);
        }

        #[test]
        fn vec_additive_inverse(v in arb_vec(3)) {
            let sum = v.add(&v.negate());
            prop_assert!(sum.norm() < 1e-12);
        }

        #[test]
        fn vec_scalar_compatibility(v in arb_vec(3), a in -10.0..10.0_f64, b in -10.0..10.0_f64) {
            let lhs = v.scale(a * b);
            let rhs = v.scale(a).scale(b);
            for i in 0..3 {
                prop_assert!((lhs.get(i) - rhs.get(i)).abs() < 1e-8);
            }
        }

        #[test]
        fn vec_multiplicative_identity(v in arb_vec(3)) {
            let scaled = v.scale(1.0);
            prop_assert!(scaled == v);
        }

        #[test]
        fn vec_distributivity_vectors(u in arb_vec(3), v in arb_vec(3), a in -10.0..10.0_f64) {
            let lhs = u.add(&v).scale(a);
            let rhs = u.scale(a).add(&v.scale(a));
            for i in 0..3 {
                prop_assert!((lhs.get(i) - rhs.get(i)).abs() < 1e-8);
            }
        }

        #[test]
        fn vec_distributivity_scalars(v in arb_vec(3), a in -10.0..10.0_f64, b in -10.0..10.0_f64) {
            let lhs = v.scale(a + b);
            let rhs = v.scale(a).add(&v.scale(b));
            for i in 0..3 {
                prop_assert!((lhs.get(i) - rhs.get(i)).abs() < 1e-8);
            }
        }

        // --- Matrix axioms ---

        #[test]
        fn matrix_transpose_involution(m in arb_square_matrix(3)) {
            let mtt = m.transpose().transpose();
            prop_assert!(matrix::approx_eq(&m, &mtt, 1e-15));
        }

        #[test]
        fn matrix_transpose_product(a in arb_square_matrix(2), b in arb_square_matrix(2)) {
            let lhs = a.multiply(&b).transpose();
            let rhs = b.transpose().multiply(&a.transpose());
            prop_assert!(matrix::approx_eq(&lhs, &rhs, 1e-8));
        }

        #[test]
        fn det_of_identity_is_one(n in 1..5_usize) {
            let i = Matrix::identity(n);
            prop_assert!((determinant::det(&i) - 1.0).abs() < 1e-12);
        }

        #[test]
        fn det_transpose_invariance(m in arb_square_matrix(3)) {
            let d = determinant::det(&m);
            let dt = determinant::det(&m.transpose());
            prop_assert!((d - dt).abs() < 1e-6,
                "det={}, det(T)={}", d, dt);
        }

        #[test]
        fn det_multiplicativity(a in arb_square_matrix(2), b in arb_square_matrix(2)) {
            let lhs = determinant::det(&a.multiply(&b));
            let rhs = determinant::det(&a) * determinant::det(&b);
            prop_assert!((lhs - rhs).abs() < 1e-4,
                "det(AB)={}, det(A)*det(B)={}", lhs, rhs);
        }

        // --- PSD / PD ---

        #[test]
        fn pd_matrix_has_positive_eigenvalues(m in arb_pd_matrix(3)) {
            prop_assert!(positive_definite::is_positive_definite(&m));
        }

        #[test]
        fn pd_quadratic_form_is_positive(m in arb_pd_matrix(2), x in arb_vec(2)) {
            if x.norm() > 1e-10 {
                let q = positive_definite::quadratic_form(&m, &x);
                prop_assert!(q > -1e-10, "x^T A x = {} for PD matrix", q);
            }
        }

        #[test]
        fn cholesky_roundtrip(m in arb_pd_matrix(3)) {
            let l = decomposition::cholesky(&m).unwrap();
            let reconstructed = l.multiply(&l.transpose());
            prop_assert!(matrix::approx_eq(&m, &reconstructed, 1e-8));
        }

        #[test]
        fn cholesky_is_lower_triangular(m in arb_pd_matrix(3)) {
            let l = decomposition::cholesky(&m).unwrap();
            for i in 0..3 {
                for j in (i + 1)..3 {
                    prop_assert!(l.get(i, j).abs() < 1e-12,
                        "L[{},{}] = {} (should be 0)", i, j, l.get(i, j));
                }
            }
        }

        #[test]
        fn symmetrize_produces_symmetric(m in arb_square_matrix(3)) {
            let s = positive_definite::symmetrize(&m);
            prop_assert!(s.is_symmetric(1e-15));
        }

        #[test]
        fn trace_is_linear(a in arb_square_matrix(3), b in arb_square_matrix(3)) {
            let lhs = a.add(&b).trace();
            let rhs = a.trace() + b.trace();
            prop_assert!((lhs - rhs).abs() < 1e-10);
        }
    }
}
