//! The geodesy ontology — WGS84 ellipsoid, geodetic/ECEF/NED coordinate conversions
pub mod conversion;
pub mod coordinate;
pub mod ellipsoid;
pub mod ontology;

#[cfg(test)]
mod tests;
