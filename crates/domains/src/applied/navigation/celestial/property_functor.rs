//! Functor: Celestial → ObservableProperty.
//!
//! Each celestial sensor produces a property in the shared ObservableProperty
//! ontology: star trackers produce Attitude, sun sensors produce Bearing,
//! horizon sensors produce Elevation.
//!
//! Source: Wertz (2001); Bowditch (2002); Groves (2013).

use pr4xis::category::{Category, Functor, Relationship};

use crate::applied::navigation::celestial::ontology::{
    CelestialCategory, CelestialConcept, CelestialRelationKind,
};
use crate::applied::sensor_fusion::property::ontology::{
    ObservablePropertyCategory, ObservablePropertyConcept, ObservablePropertyRelation,
    ObservablePropertyRelationKind,
};

pub struct CelestialToProperty;

impl Functor for CelestialToProperty {
    type Source = CelestialCategory;
    type Target = ObservablePropertyCategory;

    fn map_object(obj: &CelestialConcept) -> ObservablePropertyConcept {
        match obj {
            // Abstract sensor → abstract observable property
            CelestialConcept::Sensor => ObservablePropertyConcept::ObservableProperty,
            // Star tracker produces full attitude estimates
            CelestialConcept::StarTracker => ObservablePropertyConcept::Attitude,
            // Sun sensor produces a bearing (to the sun)
            CelestialConcept::SunSensor => ObservablePropertyConcept::Bearing,
            // Horizon sensor produces elevation reference (to Earth's limb)
            CelestialConcept::HorizonSensor => ObservablePropertyConcept::Elevation,
        }
    }

    fn map_morphism(
        m: &<CelestialCategory as pr4xis::category::Category>::Morphism,
    ) -> ObservablePropertyRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Preserve source's Identity → target's Identity; everything else
        // maps to Composed so F(g∘f) == F(g)∘F(f) holds under collapse.
        match m.kind {
            CelestialRelationKind::Identity => ObservablePropertyCategory::identity(&from),
            _ => ObservablePropertyRelation {
                from,
                to,
                kind: ObservablePropertyRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(
    CelestialToProperty,
    "Wertz (2001); Bowditch (2002); Groves (2013)."
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws_hold() {
        check_functor_laws::<CelestialToProperty>().unwrap();
    }

    #[test]
    fn star_tracker_produces_attitude() {
        assert_eq!(
            CelestialToProperty::map_object(&CelestialConcept::StarTracker),
            ObservablePropertyConcept::Attitude
        );
    }

    #[test]
    fn sun_sensor_produces_bearing() {
        assert_eq!(
            CelestialToProperty::map_object(&CelestialConcept::SunSensor),
            ObservablePropertyConcept::Bearing
        );
    }

    #[test]
    fn horizon_sensor_produces_elevation() {
        assert_eq!(
            CelestialToProperty::map_object(&CelestialConcept::HorizonSensor),
            ObservablePropertyConcept::Elevation
        );
    }
}
