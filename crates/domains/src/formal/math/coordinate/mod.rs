//! Typed coordinate representations — polar, spherical, and geodetic positions
//! as first-class structs, not `(f64, f64[, f64])` tuples.
//!
//! A Cartesian coordinate is a Euclidean vector, so it is a
//! [`Vector`](crate::formal::math::linear_algebra::vector_space::Vector); the
//! *non-Euclidean* coordinate systems (where component-wise vector arithmetic is
//! meaningless) get their own types here, each with conversions to and from the
//! Cartesian `Vector`.
//!
//! # Literature
//!
//! - **ISO 19111:2019** *Geographic information — Referencing by coordinates*
//!   (= OGC Abstract Specification Topic 2) — defines the coordinate-system
//!   types: Cartesian, spherical, ellipsoidal (geodetic), polar, cylindrical.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pub mod ontology;

use crate::formal::math::angle::Angle;
use crate::formal::math::linear_algebra::vector_space::Vector;

/// Range magnitude below which a point is treated as at the origin (where the
/// elevation angle is geometrically undefined). Just above the IEEE-754
/// double-precision noise floor at unit scale.
const ORIGIN_RANGE_EPS: f64 = 1e-15;

/// A 2-D polar coordinate — radial distance + azimuth (ISO 19111 polar CS).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolarCoordinate {
    /// Radial distance from the origin.
    pub range: f64,
    /// Azimuth, measured clockwise from north — a typed [`Angle`].
    pub azimuth: Angle,
}

impl PolarCoordinate {
    /// Construct from a range and an [`Angle`] azimuth.
    pub fn new(range: f64, azimuth: Angle) -> Self {
        Self { range, azimuth }
    }

    /// To 2-D Cartesian `[x = east, y = north]`: `x = r·sin(az)`, `y = r·cos(az)`.
    pub fn to_cartesian(&self) -> Vector {
        Vector::new(vec![
            self.range * self.azimuth.sin(),
            self.range * self.azimuth.cos(),
        ])
    }

    /// From a 2-D Cartesian `[x = east, y = north]` vector.
    pub fn from_cartesian(p: &Vector) -> Self {
        let (x, y) = (p.get(0), p.get(1));
        Self {
            range: (x * x + y * y).sqrt(),
            azimuth: Angle::from_radians(x.atan2(y)),
        }
    }
}

/// A 3-D spherical coordinate — range + azimuth + elevation (ISO 19111 spherical CS).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphericalCoordinate {
    /// Radial distance from the origin.
    pub range: f64,
    /// Azimuth, clockwise from north — a typed [`Angle`].
    pub azimuth: Angle,
    /// Elevation above the horizontal plane — a typed [`Angle`].
    pub elevation: Angle,
}

impl SphericalCoordinate {
    /// Construct from a range and [`Angle`] azimuth/elevation.
    pub fn new(range: f64, azimuth: Angle, elevation: Angle) -> Self {
        Self {
            range,
            azimuth,
            elevation,
        }
    }

    /// To 3-D Cartesian `[x = east, y = north, z = up]`.
    pub fn to_cartesian(&self) -> Vector {
        let cos_el = self.elevation.cos();
        Vector::new(vec![
            self.range * cos_el * self.azimuth.sin(),
            self.range * cos_el * self.azimuth.cos(),
            self.range * self.elevation.sin(),
        ])
    }

    /// From a 3-D Cartesian vector. Elevation is `0` at the origin (undefined).
    pub fn from_cartesian(p: &Vector) -> Self {
        let (x, y, z) = (p.get(0), p.get(1), p.get(2));
        let range = (x * x + y * y + z * z).sqrt();
        let elevation = if range > ORIGIN_RANGE_EPS {
            (z / range).asin()
        } else {
            0.0
        };
        Self {
            range,
            azimuth: Angle::from_radians(x.atan2(y)),
            elevation: Angle::from_radians(elevation),
        }
    }
}

/// A geodetic (ellipsoidal) horizontal position — latitude + longitude
/// (ISO 19111 ellipsoidal CS). Angles are stored in **radians**.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeticPosition {
    /// Geodetic latitude — a typed [`Angle`].
    pub latitude: Angle,
    /// Geodetic longitude — a typed [`Angle`].
    pub longitude: Angle,
}

impl GeodeticPosition {
    /// Construct from latitude / longitude [`Angle`]s.
    pub fn new(latitude: Angle, longitude: Angle) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    /// Construct from latitude / longitude in **degrees**.
    pub fn from_degrees(latitude_deg: f64, longitude_deg: f64) -> Self {
        Self {
            latitude: Angle::from_degrees(latitude_deg),
            longitude: Angle::from_degrees(longitude_deg),
        }
    }

    /// Latitude in degrees.
    pub fn latitude_degrees(&self) -> f64 {
        self.latitude.degrees()
    }

    /// Longitude in degrees.
    pub fn longitude_degrees(&self) -> f64 {
        self.longitude.degrees()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn polar_cartesian_round_trip() {
        let p = PolarCoordinate::new(500.0, Angle::from_radians(0.7));
        let back = PolarCoordinate::from_cartesian(&p.to_cartesian());
        assert!((p.range - back.range).abs() < 1e-9);
        assert!(p.azimuth.difference(&back.azimuth).radians().abs() < 1e-9);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn spherical_cartesian_round_trip() {
        let s =
            SphericalCoordinate::new(1000.0, Angle::from_radians(1.2), Angle::from_radians(0.3));
        let back = SphericalCoordinate::from_cartesian(&s.to_cartesian());
        assert!((s.range - back.range).abs() < 1e-9);
        assert!(s.azimuth.difference(&back.azimuth).radians().abs() < 1e-9);
        assert!(s.elevation.difference(&back.elevation).radians().abs() < 1e-9);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn spherical_from_origin_has_zero_elevation() {
        let s = SphericalCoordinate::from_cartesian(&Vector::zeros(3));
        assert_eq!(s.range, 0.0);
        assert_eq!(s.elevation.radians(), 0.0);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn geodetic_degree_radian() {
        let g = GeodeticPosition::from_degrees(45.0, -120.0);
        assert!((g.latitude_degrees() - 45.0).abs() < 1e-9);
        assert!((g.longitude_degrees() - (-120.0)).abs() < 1e-9);
    }
}
