//! ObservableProperty ontology (W3C SSN/SOSA grounded).
//!
//! The physical and geometric properties that sensors observe and actuators change.
//! Provides a shared home for Position, Velocity, Attitude, Heading, Range, Bearing,
//! Force, Temperature etc. that was previously missing — each domain had been
//! re-declaring these locally as a secondary enum with a manual `TaxonomyDef` impl.

pub mod ontology;
