//! Radar coordinate conversions.
//!
//! The polar/spherical ↔ Cartesian conversions that used to live here as free
//! tuple functions now live in the typed coordinate module
//! [`crate::formal::math::coordinate`], as [`PolarCoordinate`] and
//! [`SphericalCoordinate`] with `to_cartesian` / `from_cartesian` against the
//! Cartesian [`Vector`]. Radar code uses those typed representations directly.
//!
//! [`PolarCoordinate`]: crate::formal::math::coordinate::PolarCoordinate
//! [`SphericalCoordinate`]: crate::formal::math::coordinate::SphericalCoordinate
//! [`Vector`]: crate::formal::math::linear_algebra::vector_space::Vector
