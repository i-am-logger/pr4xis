//! Functor: Sensor → Driver (every sensor is a device a driver drives).
//!
//! From the operating system's side of the seam, a sensor — inertial,
//! optical, acoustic, or composite — is simply a hardware peripheral
//! reached through its driver (Corbet, Rubini & Kroah-Hartman 2005,
//! Ch. 1). The whole Groves (2013) modality taxonomy — proprioceptive
//! vs exteroceptive, active vs passive, and the composite IMU/AHRS/INS
//! aggregates — is *below the driver layer's resolution*: this is the
//! constant functor onto `DriverConcept::Device`, collapsing all
//! sensor concepts (and all their modality/parthood structure) into
//! device-hood.
//!
//! Because every object lands on `Device`, every morphism — identity
//! or otherwise — lands on `id_Device`, which is identity- and
//! composition-preserving by construction (the same reading as the
//! workspace's other constant functors).
//!
//! **Deferred follow-up**: the sensor/driver *adjunction* that would
//! expose this collapse gap (which sensor distinctions the driver
//! layer cannot represent, and what a right adjoint would have to
//! reconstruct) is not built here.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Category, Functor};

use super::ontology::{SensorCategory, SensorConcept};
use crate::applied::operating_system::driver::ontology::{DriverCategory, DriverConcept};

/// The constant functor: every sensor concept is a device the driver
/// drives (Corbet et al. 2005, Ch. 1; Groves 2013).
pub struct SensorToDriver;

impl Functor for SensorToDriver {
    type Source = SensorCategory;
    type Target = DriverCategory;

    fn map_object(_sensor: &SensorConcept) -> DriverConcept {
        // Every sensor — including the composite IMU/AHRS/INS — is a
        // hardware peripheral from the OS side: a Device.
        DriverConcept::Device
    }

    fn map_morphism(
        _m: &<SensorCategory as Category>::Morphism,
    ) -> <DriverCategory as Category>::Morphism {
        // Both endpoints of every sensor morphism map to `Device`, so
        // the image of every morphism (identity, subsumption,
        // parthood, opposition) is the identity on `Device`: F(id) =
        // id_Device and F(g . f) = id_Device = id_Device . id_Device.
        DriverCategory::identity(&DriverConcept::Device)
    }
}
pr4xis::register_functor!(
    SensorToDriver,
    "Corbet, Rubini & Kroah-Hartman (2005) Ch. 1; Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2e"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<SensorToDriver>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn every_sensor_is_a_device() {
        for sensor in SensorConcept::variants() {
            assert_eq!(
                SensorToDriver::map_object(&sensor),
                DriverConcept::Device,
                "{sensor:?} should collapse to Device"
            );
        }
    }

    /// The composite sensors collapse too: the driver layer cannot see
    /// the IMU/AHRS/INS aggregation structure — the gap the deferred
    /// sensor/driver adjunction would expose.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn composite_sensors_also_collapse() {
        for composite in [SensorConcept::IMU, SensorConcept::AHRS, SensorConcept::INS] {
            assert_eq!(
                SensorToDriver::map_object(&composite),
                DriverConcept::Device,
                "{composite:?} should collapse to Device"
            );
        }
    }
}
