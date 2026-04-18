//! Functor: Ahrs → ObservableProperty.
//!
//! Every attitude filter produces the same observable property (Attitude,
//! composed of Roll/Pitch/Yaw). Prior to #118, AHRS declared its own
//! `AttitudeState` enum; those concepts now live in the shared
//! `ObservableProperty` ontology and this functor expresses the production
//! relation.
//!
//! Source: Madgwick (2010); Mahony et al. (2008); Titterton & Weston (2004).

use pr4xis::category::{Category, Functor, Relationship};

use crate::applied::navigation::ahrs::ontology::{AhrsCategory, AhrsConcept, AhrsRelationKind};
use crate::applied::sensor_fusion::property::ontology::{
    ObservablePropertyCategory, ObservablePropertyConcept, ObservablePropertyRelation,
    ObservablePropertyRelationKind,
};

/// Maps each AHRS filter to the observable property it produces.
/// All AHRS filters produce Attitude — the Roll×Pitch×Yaw Euler decomposition
/// is handled by ObservableProperty's internal taxonomy (Roll/Pitch/Yaw is_a Attitude).
pub struct AhrsToProperty;

impl Functor for AhrsToProperty {
    type Source = AhrsCategory;
    type Target = ObservablePropertyCategory;

    fn map_object(obj: &AhrsConcept) -> ObservablePropertyConcept {
        match obj {
            // The abstract filter produces attitude.
            AhrsConcept::Filter => ObservablePropertyConcept::Attitude,
            // Every concrete filter produces attitude — they differ in accuracy
            // and cost (captured as Qualities on the AHRS ontology), not in
            // what they produce.
            AhrsConcept::ComplementaryFilter
            | AhrsConcept::MahonyFilter
            | AhrsConcept::MadgwickFilter
            | AhrsConcept::ExtendedKalmanFilter => ObservablePropertyConcept::Attitude,
        }
    }

    fn map_morphism(
        m: &<AhrsCategory as pr4xis::category::Category>::Morphism,
    ) -> ObservablePropertyRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Preserve source's Identity → target's Identity; everything else
        // maps to Composed so F(g∘f) == F(g)∘F(f) holds under collapse.
        match m.kind {
            AhrsRelationKind::Identity => ObservablePropertyCategory::identity(&from),
            _ => ObservablePropertyRelation {
                from,
                to,
                kind: ObservablePropertyRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(
    AhrsToProperty,
    "Madgwick (2010); Mahony et al. (2008); Titterton & Weston (2004)."
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws_hold() {
        check_functor_laws::<AhrsToProperty>().unwrap();
    }

    #[test]
    fn every_filter_produces_attitude() {
        for filter in [
            AhrsConcept::Filter,
            AhrsConcept::ComplementaryFilter,
            AhrsConcept::MahonyFilter,
            AhrsConcept::MadgwickFilter,
            AhrsConcept::ExtendedKalmanFilter,
        ] {
            assert_eq!(
                AhrsToProperty::map_object(&filter),
                ObservablePropertyConcept::Attitude,
                "{filter:?} should produce Attitude"
            );
        }
    }
}
