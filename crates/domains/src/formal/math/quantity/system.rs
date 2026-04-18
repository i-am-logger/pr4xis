#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::quantity::dimension::Dimension;

/// Measurement system classification.
///
/// Source: BIPM SI Brochure (2019).
///         QUDT (qudt.org).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeasurementSystem {
    /// International System of Units.
    SI,
    /// Imperial / US customary.
    Imperial,
    /// CGS (centimeter-gram-second).
    CGS,
    /// Natural units (c = ℏ = 1).
    Natural,
}

/// Verify that a derived dimension equals its definition.
///
/// Example: velocity = length / time
///   [v] = [L] · [T]⁻¹ = L¹·T⁻¹ ✓
pub fn verify_derived(derived: Dimension, definition: Dimension) -> bool {
    derived == definition
}
