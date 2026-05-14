//! Functor: Gnss → ObservableProperty.
//!
//! Each GNSS observable type ultimately measures an observable property:
//! Pseudorange and CarrierPhase measure Range, Doppler measures Velocity
//! (radial, via frequency shift), NavigationMessage carries Time (ephemeris).
//!
//! Source: IS-GPS-200 (2022); Groves (2013) Chapter 8.

use pr4xis::category::{Arrow, Category, Functor};

use crate::applied::navigation::gnss::ontology::{GnssCategory, GnssConcept, GnssRelationKind};
use crate::applied::sensor_fusion::property::ontology::{
    ObservablePropertyCategory, ObservablePropertyConcept, ObservablePropertyRelation,
    ObservablePropertyRelationKind,
};

pub struct GnssToProperty;

impl Functor for GnssToProperty {
    type Source = GnssCategory;
    type Target = ObservablePropertyCategory;

    fn map_object(obj: &GnssConcept) -> ObservablePropertyConcept {
        match obj {
            GnssConcept::Observable => ObservablePropertyConcept::ObservableProperty,
            // Pseudorange and CarrierPhase both yield range (distance to satellite).
            GnssConcept::Pseudorange => ObservablePropertyConcept::Range,
            GnssConcept::CarrierPhase => ObservablePropertyConcept::Range,
            // Doppler is a frequency shift → yields velocity (radial component).
            GnssConcept::Doppler => ObservablePropertyConcept::Velocity,
            // NavigationMessage carries satellite clock/ephemeris info → Time.
            GnssConcept::NavigationMessage => ObservablePropertyConcept::Time,
        }
    }

    fn map_morphism(
        m: &<GnssCategory as pr4xis::category::Category>::Morphism,
    ) -> ObservablePropertyRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Preserve source's Identity → target's Identity; everything else
        // collapses to Subsumption (#166 removed `Composed` from auto-generated
        // kinds — heterogeneous composition is now expressed as the most
        // general structural kind, OBO-RO Subsumption).
        match m.kind {
            GnssRelationKind::Identity => ObservablePropertyCategory::identity(&from),
            _ => ObservablePropertyRelation {
                from,
                to,
                kind: ObservablePropertyRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(
    GnssToProperty,
    "IS-GPS-200 (2022); Groves (2013) Chapter 8."
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<GnssToProperty>();
    }

    #[test]
    fn pseudorange_measures_range() {
        assert_eq!(
            GnssToProperty::map_object(&GnssConcept::Pseudorange),
            ObservablePropertyConcept::Range
        );
    }

    #[test]
    fn doppler_measures_velocity() {
        assert_eq!(
            GnssToProperty::map_object(&GnssConcept::Doppler),
            ObservablePropertyConcept::Velocity
        );
    }
}
