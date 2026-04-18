//! Functor: Odometry → ObservableProperty.
//!
//! Each odometry method measures specific properties. This functor makes that
//! relationship first-class instead of encoding it in docstrings or axiom
//! bodies. Prior to #118, odometry's module declared a local `OdometryState`
//! enum (Position2D, Heading, Velocity) paired with a manual `TaxonomyDef`;
//! those concepts now live in the shared `ObservableProperty` ontology and
//! this functor expresses the measurement relation.
//!
//! Source: Borenstein et al. (1996); Thrun, Burgard & Fox (2005); Scaramuzza
//!         & Fraundorfer (2011).

use pr4xis::category::{Category, Functor, Relationship};

use crate::applied::navigation::odometry::ontology::{
    OdometryCategory, OdometryConcept, OdometryRelationKind,
};
use crate::applied::sensor_fusion::property::ontology::{
    ObservablePropertyCategory, ObservablePropertyConcept, ObservablePropertyRelation,
    ObservablePropertyRelationKind,
};

/// Maps each odometry method to the primary observable property it estimates.
///
/// Many-to-one: all methods collapse to their dominant observable. The
/// richer many-to-many picture (a single method measures multiple properties)
/// is not expressible by a bare Functor — it needs a relation, i.e. a profunctor
/// Odometry ↛ ObservableProperty, which is a future refinement.
pub struct OdometryToProperty;

impl Functor for OdometryToProperty {
    type Source = OdometryCategory;
    type Target = ObservablePropertyCategory;

    fn map_object(obj: &OdometryConcept) -> ObservablePropertyConcept {
        match obj {
            // The abstract odometry source measures position in general.
            OdometryConcept::Source => ObservablePropertyConcept::Position,
            // Wheel encoders count rotations to estimate distance → position.
            OdometryConcept::WheelEncoder => ObservablePropertyConcept::Position,
            // Visual odometry estimates both position and heading from feature tracks.
            // We pick Position as primary; a finer-grained mapping would split
            // the method into separate morphisms for each output.
            OdometryConcept::VisualOdometry => ObservablePropertyConcept::Position,
            // Inertial odometry integrates acceleration to velocity to position,
            // and angular velocity to attitude. Primary output: velocity.
            OdometryConcept::InertialOdometry => ObservablePropertyConcept::Velocity,
            // Laser odometry matches scans to produce pose → position.
            OdometryConcept::LaserOdometry => ObservablePropertyConcept::Position,
        }
    }

    fn map_morphism(
        m: &<OdometryCategory as pr4xis::category::Category>::Morphism,
    ) -> ObservablePropertyRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Preserve source's Identity → target's Identity; everything else
        // maps to Composed so F(g∘f) == F(g)∘F(f) holds under collapse.
        match m.kind {
            OdometryRelationKind::Identity => ObservablePropertyCategory::identity(&from),
            _ => ObservablePropertyRelation {
                from,
                to,
                kind: ObservablePropertyRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(
    OdometryToProperty,
    "Borenstein et al. (1996); Thrun, Burgard & Fox (2005); Scaramuzza"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws_hold() {
        check_functor_laws::<OdometryToProperty>().unwrap();
    }

    #[test]
    fn wheel_encoder_measures_position() {
        assert_eq!(
            OdometryToProperty::map_object(&OdometryConcept::WheelEncoder),
            ObservablePropertyConcept::Position
        );
    }

    #[test]
    fn inertial_odometry_primary_output_is_velocity() {
        assert_eq!(
            OdometryToProperty::map_object(&OdometryConcept::InertialOdometry),
            ObservablePropertyConcept::Velocity
        );
    }
}
