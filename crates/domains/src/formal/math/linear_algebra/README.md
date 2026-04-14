# Linear Algebra -- Vector spaces, matrices, determinants, eigenvalues

Models the standard algebraic structures of finite-dimensional linear algebra as a discrete category. Axioms cover matrix multiplication, transpose, determinant, eigenvalues, Cholesky factorization, and positive (semi-)definiteness — each verified numerically against canonical matrices at test time.

Key references:
- Strang: *Introduction to Linear Algebra* (the standard undergraduate text)
- Golub & Van Loan: *Matrix Computations* (numerical reference)
- Horn & Johnson: *Matrix Analysis* (theoretical reference)

## Entities (9)

| Category | Entities |
|---|---|
| Primitives (3) | Scalar, Vector, Matrix |
| Structured matrices (4) | SymmetricMatrix, PositiveDefiniteMatrix, DiagonalMatrix, IdentityMatrix |
| Triangular matrices (2) | LowerTriangular, UpperTriangular |

## Category

Discrete category over the nine algebraic structures — objects only, identity morphisms only. The mathematical content lives in the axioms over canonical matrix data, not in the morphism structure.

## Qualities

| Quality | Type | Description |
|---|---|---|
| StructureDimension | &'static str | Shape and free-parameter count: Scalar=0, Vector=n, Matrix=n×m, SymmetricMatrix=n(n+1)/2 free, PositiveDefiniteMatrix=n(n+1)/2 free with all λ>0, etc. |

## Axioms (12)

| Axiom | Description | Source |
|---|---|---|
| MultiplicationAssociativity | (AB)C = A(BC) | standard |
| MultiplicationIdentity | AI = IA = A | standard |
| TransposeInvolution | (A^T)^T = A | standard |
| TransposeProduct | (AB)^T = B^T A^T | standard |
| DetNormalization | det(I) = 1 | determinant axioms |
| DetMultiplicativity | det(AB) = det(A)·det(B) | determinant axioms |
| DetTranspose | det(A^T) = det(A) | determinant axioms |
| TraceEigenvalueSum | tr(A) = Σλᵢ | eigenvalue theorem |
| DetEigenvalueProduct | det(A) = Πλᵢ | eigenvalue theorem |
| CholeskyFactorization | A = L L^T for positive definite A | Cholesky 1910 |
| PsdQuadraticForm | x^T A x ≥ 0 for PSD A | PSD definition |
| JosephPreservesPsd | Joseph-form covariance update preserves PSD | Kalman filtering |

Plus the auto-generated structural axioms from `define_ontology!` (category laws on the discrete category).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. Linear algebra is a foundational ontology that other domains (control theory, signal processing, statistics, rotation) compose against; the functors will land as those other domains gain explicit linear-algebraic morphisms.

## Files

- `ontology.rs` -- Entity, discrete category, 12 axioms, tests
- `matrix.rs` -- `Matrix` type and core operations (multiply, transpose, trace, add, scale)
- `vector_space.rs` -- `Vector` type and vector-space operations
- `determinant.rs` -- determinant computation
- `eigenvalue.rs` -- eigenvalue computation for symmetric matrices
- `decomposition.rs` -- Cholesky and related factorizations
- `positive_definite.rs` -- quadratic form, Joseph-form update, PSD check
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
