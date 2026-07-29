#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// Dense vector over R (column vector).
///
/// Satisfies the 8 vector space axioms (Axler, *Linear Algebra Done Right*):
/// 1. Associativity of addition: (u+v)+w = u+(v+w)
/// 2. Commutativity of addition: u+v = v+u
/// 3. Additive identity: ∃0 s.t. v+0 = v
/// 4. Additive inverse: ∀v, ∃-v s.t. v+(-v) = 0
/// 5. Scalar compatibility: a(bv) = (ab)v
/// 6. Multiplicative identity: 1v = v
/// 7. Distributivity over vectors: a(u+v) = au + av
/// 8. Distributivity over scalars: (a+b)v = av + bv
#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    pub data: Vec<f64>,
}

impl Vector {
    pub fn new(data: Vec<f64>) -> Self {
        Self { data }
    }

    pub fn zeros(n: usize) -> Self {
        Self { data: vec![0.0; n] }
    }

    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Additive identity (axiom 3).
    pub fn zero(n: usize) -> Self {
        Self::zeros(n)
    }

    /// Vector addition (axioms 1, 2).
    pub fn add(&self, other: &Self) -> Self {
        assert_eq!(self.dim(), other.dim());
        Self {
            data: self
                .data
                .iter()
                .zip(&other.data)
                .map(|(a, b)| a + b)
                .collect(),
        }
    }

    /// Vector subtraction.
    pub fn sub(&self, other: &Self) -> Self {
        assert_eq!(self.dim(), other.dim());
        Self {
            data: self
                .data
                .iter()
                .zip(&other.data)
                .map(|(a, b)| a - b)
                .collect(),
        }
    }

    /// Additive inverse (axiom 4).
    pub fn negate(&self) -> Self {
        Self {
            data: self.data.iter().map(|x| -x).collect(),
        }
    }

    /// Scalar multiplication (axioms 5, 6, 7, 8).
    pub fn scale(&self, s: f64) -> Self {
        Self {
            data: self.data.iter().map(|x| x * s).collect(),
        }
    }

    /// Inner product (dot product).
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), never a bare
    /// `f64` — `Vector` is an abstract linear-algebra primitive (Axler,
    /// *Linear Algebra Done Right*) with no inherent physical unit at this
    /// generic layer, the same treatment as `Point3::distance_to`.
    pub fn dot(&self, other: &Self) -> Quantity {
        assert_eq!(self.dim(), other.dim());
        let sum: f64 = self.data.iter().zip(&other.data).map(|(a, b)| a * b).sum();
        Quantity::from_unit(sum, &unit::UNITLESS)
    }

    /// Euclidean norm (L2).
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), same
    /// reasoning as [`Vector::dot`].
    pub fn norm(&self) -> Quantity {
        Quantity::from_unit(self.dot(self).value.sqrt(), &unit::UNITLESS)
    }

    /// Squared norm.
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), same
    /// reasoning as [`Vector::dot`] and [`Vector::norm`].
    pub fn norm_squared(&self) -> Quantity {
        self.dot(self)
    }

    /// Element access.
    pub fn get(&self, i: usize) -> f64 {
        self.data[i]
    }
}
