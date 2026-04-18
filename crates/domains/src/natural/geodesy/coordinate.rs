#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Geodetic coordinates on an ellipsoid.
///
/// The natural coordinate system for positions on or near Earth's surface.
///
/// Source: Torge & Müller, *Geodesy* (2012), Chapter 5.
#[derive(Debug, Clone, PartialEq)]
pub struct Geodetic {
    /// Latitude in radians (positive north, -π/2 to π/2).
    pub lat: f64,
    /// Longitude in radians (positive east, -π to π).
    pub lon: f64,
    /// Height above ellipsoid in meters.
    pub alt: f64,
}

/// Earth-Centered Earth-Fixed (ECEF) Cartesian coordinates.
///
/// Origin at Earth's center of mass. X-axis toward prime meridian
/// at equator, Z-axis toward north pole, Y completes right-hand system.
///
/// Source: NIMA TR8350.2 (2000), Section 2.
#[derive(Debug, Clone, PartialEq)]
pub struct Ecef {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Local North-East-Down (NED) coordinates.
///
/// Tangent plane at a reference point. North = +X, East = +Y, Down = +Z.
/// Standard in aerospace and navigation.
///
/// Source: Groves, *Principles of GNSS, Inertial, and Multisensor
///         Integrated Navigation Systems* (2013), Section 2.4.
#[derive(Debug, Clone, PartialEq)]
pub struct Ned {
    pub north: f64,
    pub east: f64,
    pub down: f64,
}

/// Local East-North-Up (ENU) coordinates.
///
/// Tangent plane at a reference point. East = +X, North = +Y, Up = +Z.
/// Right-handed, Z-up convention.
#[derive(Debug, Clone, PartialEq)]
pub struct Enu {
    pub east: f64,
    pub north: f64,
    pub up: f64,
}

impl Geodetic {
    pub fn new(lat: f64, lon: f64, alt: f64) -> Self {
        Self { lat, lon, alt }
    }

    /// Latitude in degrees.
    pub fn lat_deg(&self) -> f64 {
        self.lat.to_degrees()
    }

    /// Longitude in degrees.
    pub fn lon_deg(&self) -> f64 {
        self.lon.to_degrees()
    }
}

impl Ned {
    /// Convert NED to ENU (simple axis swap).
    pub fn to_enu(&self) -> Enu {
        Enu {
            east: self.east,
            north: self.north,
            up: -self.down,
        }
    }

    /// Horizontal distance (north-east plane).
    pub fn horizontal_distance(&self) -> f64 {
        (self.north * self.north + self.east * self.east).sqrt()
    }
}

impl Enu {
    /// Convert ENU to NED.
    pub fn to_ned(&self) -> Ned {
        Ned {
            north: self.north,
            east: self.east,
            down: -self.up,
        }
    }
}
