//! ObservableProperty — physical / geometric properties that sensors observe
//! and actuators change.
//!
//! Grounded in the W3C SSN/SOSA standard (Semantic Sensor Network /
//! Sensor, Observation, Sample, Actuator). The abstract SSN hierarchy is
//! `Property → {ObservableProperty, ActuatableProperty}`; this ontology
//! adds the concrete properties common in robotics, navigation, and
//! physics (Position, Velocity, Attitude, Heading, Range, Bearing,
//! Acceleration, AngularVelocity, Force, Torque, Temperature, Pressure,
//! MagneticField, etc.).
//!
//! Fills the gap identified in #118: domains such as odometry and AHRS
//! previously re-declared these property enums locally; with this
//! ontology in place they compose from here via functors (`SensorToProperty`,
//! `AhrsToProperty`, …) and keep their own concepts scoped to sensors and
//! methods, not properties.
//!
//! # Literature
//!
//! - **Haller, Janowicz, Cox, Lefrançois, Taylor, Le Phuoc, Lieberman,
//!   García-Castro, Atkinson, Stadler (2019)** "The modular SSN ontology:
//!   A joint W3C and OGC standard specifying the semantics of sensors,
//!   observations, sampling, and actuation", *Semantic Web Journal*
//!   10(1), pp. 9–32. DOI: 10.3233/SW-180320.
//! - **W3C Recommendation (2017)** *Semantic Sensor Network Ontology*,
//!   <https://www.w3.org/TR/vocab-ssn/>.
//! - **Compton et al. (2012)** "The SSN ontology of the W3C semantic
//!   sensor network incubator group", *Journal of Web Semantics* 17,
//!   pp. 25–32.

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::quantity::dimension::Dimension;

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pr4xis::ontology! {
    name: "ObservableProperty",
    source: "Haller et al. (2019) The modular SSN ontology, Semantic Web Journal 10(1); W3C Recommendation (2017) Semantic Sensor Network Ontology; Compton et al. (2012) The SSN ontology of the W3C semantic sensor network incubator group, Journal of Web Semantics 17",

    concepts: [
        // --- SSN/SOSA abstract hierarchy ---
        Property,
        ObservableProperty,
        ActuatableProperty,

        // --- Kinematic properties (linear) ---
        Position,
        Velocity,
        Acceleration,
        Jerk,

        // --- Kinematic properties (angular) ---
        Attitude,
        AngularVelocity,
        AngularAcceleration,
        Orientation,

        // --- Attitude components ---
        Roll,
        Pitch,
        Yaw,
        Heading,

        // --- Geometric relations ---
        Range,
        Bearing,
        Elevation,

        // --- Dynamics (force/torque) ---
        Force,
        Torque,
        Mass,
        MomentOfInertia,

        // --- Field properties ---
        MagneticField,
        GravitationalField,
        ElectricField,

        // --- Thermodynamic properties ---
        Temperature,
        Pressure,
        Humidity,

        // --- Time properties ---
        Time,
        Duration,
        Frequency,
    ],

    labels: {
        Property: ("en", "Property", "A quality of a FeatureOfInterest that is intrinsic to and cannot exist without the entity. SSN ssn:Property."),
        ObservableProperty: ("en", "Observable property", "An observable quality (property, characteristic) of a FeatureOfInterest that can be measured by a Sensor. SOSA sosa:ObservableProperty."),
        ActuatableProperty: ("en", "Actuatable property", "An actuatable quality (property, characteristic) of a FeatureOfInterest that can be changed by an Actuator. SOSA sosa:ActuatableProperty."),

        Position: ("en", "Position", "Spatial location of a feature of interest, typically as a vector in a reference frame."),
        Velocity: ("en", "Velocity", "Rate of change of position, first derivative dx/dt."),
        Acceleration: ("en", "Acceleration", "Rate of change of velocity, second derivative d²x/dt²."),
        Jerk: ("en", "Jerk", "Rate of change of acceleration, third derivative d³x/dt³."),

        Attitude: ("en", "Attitude", "Rotational state of a rigid body relative to a reference frame; an element of SO(3)."),
        AngularVelocity: ("en", "Angular velocity", "Rate of change of attitude, ω (rad/s)."),
        AngularAcceleration: ("en", "Angular acceleration", "Rate of change of angular velocity."),
        Orientation: ("en", "Orientation", "Synonym for Attitude in many literatures; the pointing direction of a rigid body."),

        Roll: ("en", "Roll", "Rotation about the longitudinal (x, forward) axis; one of the three Euler angles."),
        Pitch: ("en", "Pitch", "Rotation about the lateral (y, right) axis; one of the three Euler angles."),
        Yaw: ("en", "Yaw", "Rotation about the vertical (z, down) axis; one of the three Euler angles. Heading is yaw relative to a geographic reference."),
        Heading: ("en", "Heading", "Angle between the body's forward direction and a geographic reference (north); yaw with geographic meaning."),

        Range: ("en", "Range", "Scalar distance from sensor to feature of interest."),
        Bearing: ("en", "Bearing", "Horizontal angle from sensor to feature of interest."),
        Elevation: ("en", "Elevation", "Vertical angle from sensor to feature of interest."),

        Force: ("en", "Force", "Vector quantity causing acceleration; F = ma."),
        Torque: ("en", "Torque", "Rotational analog of force; τ = Iα."),
        Mass: ("en", "Mass", "Inertial or gravitational mass of a body."),
        MomentOfInertia: ("en", "Moment of inertia", "Rotational analog of mass; I in τ = Iα."),

        MagneticField: ("en", "Magnetic field", "Vector field B, measured by magnetometer."),
        GravitationalField: ("en", "Gravitational field", "Vector field g; on Earth's surface typically ~9.8 m/s²."),
        ElectricField: ("en", "Electric field", "Vector field E, measured by E-field sensor."),

        Temperature: ("en", "Temperature", "Thermodynamic temperature of a feature of interest."),
        Pressure: ("en", "Pressure", "Scalar pressure of a fluid or gas."),
        Humidity: ("en", "Humidity", "Water vapor content of air."),

        Time: ("en", "Time", "Temporal coordinate; position on the time axis."),
        Duration: ("en", "Duration", "Interval between two time points."),
        Frequency: ("en", "Frequency", "Rate of periodic occurrence; 1/period."),
    },

    // The SSN abstract hierarchy: both observable and actuatable are Properties.
    is_a: [
        (ObservableProperty, Property),
        (ActuatableProperty, Property),

        // All concrete properties are observable (some are also actuatable,
        // but by default we classify them as observable here).
        (Position, ObservableProperty),
        (Velocity, ObservableProperty),
        (Acceleration, ObservableProperty),
        (Jerk, ObservableProperty),
        (Attitude, ObservableProperty),
        (AngularVelocity, ObservableProperty),
        (AngularAcceleration, ObservableProperty),
        (Orientation, ObservableProperty),
        (Roll, ObservableProperty),
        (Pitch, ObservableProperty),
        (Yaw, ObservableProperty),
        (Heading, ObservableProperty),
        (Range, ObservableProperty),
        (Bearing, ObservableProperty),
        (Elevation, ObservableProperty),
        (Force, ObservableProperty),
        (Torque, ObservableProperty),
        (Mass, ObservableProperty),
        (MomentOfInertia, ObservableProperty),
        (MagneticField, ObservableProperty),
        (GravitationalField, ObservableProperty),
        (ElectricField, ObservableProperty),
        (Temperature, ObservableProperty),
        (Pressure, ObservableProperty),
        (Humidity, ObservableProperty),
        (Time, ObservableProperty),
        (Duration, ObservableProperty),
        (Frequency, ObservableProperty),

        // Attitude component relationships
        (Roll, Attitude),
        (Pitch, Attitude),
        (Yaw, Attitude),
        (Heading, Yaw),
    ],

    // Causal/derivational relationships: differentiation chain.
    causes: [
        // Time-differentiation chain: Position → Velocity → Acceleration → Jerk
        (Position, Velocity),
        (Velocity, Acceleration),
        (Acceleration, Jerk),
        // Angular chain
        (Attitude, AngularVelocity),
        (AngularVelocity, AngularAcceleration),
        // Newton's second law: Force causes Acceleration (given Mass)
        (Force, Acceleration),
        (Torque, AngularAcceleration),
    ],
}

/// Quality: the physical [`Dimension`] of each observable property.
///
/// The value is the `quantity` ontology's own typed [`Dimension`] (the seven
/// SI exponents), NOT a prose spelling of it. This is the SSN/SOSA
/// observation-model link "an `ObservableProperty` has a `QuantityKind`"
/// (Haller et al. 2019) realised against the SI dimension algebra
/// (`formal::math::quantity`), so dimensional relationships between properties
/// are machine-checkable rather than string-matched — see
/// [`DifferentiationChainDimensionallyConsistent`]. `Dimension`'s `Display`
/// still renders the familiar `L^1·T^-1` form when a string is wanted.
#[derive(Debug, Clone)]
pub struct PropertyDimension;

impl Quality for PropertyDimension {
    type Individual = ObservablePropertyConcept;
    type Value = Dimension;
    // A dimension symbol is an abstract measure (DOLCE): it classifies the
    // quality space, it is not itself a physical endurant's quality.

    fn get(&self, p: &ObservablePropertyConcept) -> Option<Dimension> {
        use ObservablePropertyConcept as P;
        Some(match p {
            // Abstract SSN roles — no dimension.
            P::Property | P::ObservableProperty | P::ActuatableProperty => return None,

            // Kinematic linear (differentiation chain L, L·T⁻¹, L·T⁻², L·T⁻³).
            P::Position => Dimension::LENGTH,
            P::Velocity => Dimension::VELOCITY,
            P::Acceleration => Dimension::ACCELERATION,
            P::Jerk => Dimension {
                length: 1,
                time: -3,
                ..Dimension::DIMENSIONLESS
            },

            // Kinematic angular. Radian is L/L, so ANGLE == DIMENSIONLESS in SI
            // (Haller 2019 / BIPM 2019) — angles share the dimensionless space.
            P::Attitude | P::Orientation | P::Roll | P::Pitch | P::Yaw | P::Heading => {
                Dimension::ANGLE
            }
            P::AngularVelocity => Dimension::ANGULAR_VELOCITY,
            P::AngularAcceleration => Dimension::TIME.power(-2),

            // Geometric
            P::Range => Dimension::LENGTH,
            P::Bearing | P::Elevation => Dimension::ANGLE,

            // Dynamics
            P::Force => Dimension::FORCE,
            // Torque shares energy's dimension (force × length).
            P::Torque => Dimension {
                length: 2,
                mass: 1,
                time: -2,
                ..Dimension::DIMENSIONLESS
            },
            P::Mass => Dimension::MASS,
            P::MomentOfInertia => Dimension {
                length: 2,
                mass: 1,
                ..Dimension::DIMENSIONLESS
            },

            // Fields
            P::MagneticField => Dimension {
                mass: 1,
                time: -2,
                current: -1,
                ..Dimension::DIMENSIONLESS
            },
            P::GravitationalField => Dimension::ACCELERATION,
            P::ElectricField => Dimension {
                length: 1,
                mass: 1,
                time: -3,
                current: -1,
                ..Dimension::DIMENSIONLESS
            },

            // Thermodynamic
            P::Temperature => Dimension::TEMPERATURE,
            P::Pressure => Dimension {
                length: -1,
                mass: 1,
                time: -2,
                ..Dimension::DIMENSIONLESS
            },
            P::Humidity => Dimension::DIMENSIONLESS,

            // Time
            P::Time | P::Duration => Dimension::TIME,
            P::Frequency => Dimension::FREQUENCY,
        })
    }
}

/// Axiom: the time-differentiation chains are dimensionally consistent —
/// each derivative property's dimension is its antiderivative's dimension
/// divided by time.
///
/// This claim was *inexpressible* while `PropertyDimension` returned prose
/// strings; with the value typed as [`Dimension`] it is a machine-checkable
/// theorem. For every differentiation edge the `causes:` graph declares
/// (Position → Velocity → Acceleration → Jerk, and
/// Attitude → AngularVelocity → AngularAcceleration), it verifies
/// `dim(derivative) = dim(antiderivative) ÷ Time` using the SI dimension
/// algebra (`Dimension::divide`). The Newtonian edges (Force → Acceleration,
/// Torque → AngularAcceleration) are *not* differentiations and are excluded.
///
/// Grounded in the SI dimensional-analysis calculus (BIPM 2019; Tao 2012):
/// differentiation with respect to time lowers the time exponent by one.
pub struct DifferentiationChainDimensionallyConsistent;

impl Axiom for DifferentiationChainDimensionallyConsistent {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use ObservablePropertyConcept as P;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        // (antiderivative, time-derivative) pairs — the differentiation edges.
        let chains = [
            (P::Position, P::Velocity),
            (P::Velocity, P::Acceleration),
            (P::Acceleration, P::Jerk),
            (P::Attitude, P::AngularVelocity),
            (P::AngularVelocity, P::AngularAcceleration),
        ];
        let q = PropertyDimension;
        let consistent =
            chains.iter().all(
                |(antideriv, deriv)| match (q.get(antideriv), q.get(deriv)) {
                    (Some(base), Some(rate)) => rate == base.divide(&Dimension::TIME),
                    _ => false,
                },
            );
        if consistent {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DifferentiationChainDimensionallyConsistent",
        "each time-derivative property's dimension equals its antiderivative's dimension divided by time (dim(v)=dim(x)/T, dim(a)=dim(v)/T, ...)",
        "BIPM (2019) SI Brochure; Tao (2012) A mathematical formalization of dimensional analysis"
    );
}
pr4xis::register_axiom!(
    DifferentiationChainDimensionallyConsistent,
    "BIPM (2019) SI Brochure; Tao (2012) A mathematical formalization of dimensional analysis"
);

impl Ontology for ObservablePropertyOntology {
    type Cat = ObservablePropertyCategory;
    type Qual = PropertyDimension;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(DifferentiationChainDimensionallyConsistent));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn concept_count() {
        // 3 SSN + 4 linear kin + 4 angular kin + 4 attitude components +
        // 3 geometric + 4 dynamics + 3 fields + 3 thermodynamic + 3 time = 31.
        assert_eq!(ObservablePropertyConcept::variants().len(), 31);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ObservablePropertyCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ObservablePropertyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn position_is_observable_property() {
        let sub: Vec<_> = ObservablePropertyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ObservablePropertyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(sub.contains(&(
            ObservablePropertyConcept::Position,
            ObservablePropertyConcept::ObservableProperty
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn heading_is_a_yaw() {
        // Heading is yaw with geographic meaning — Heading is_a Yaw.
        let sub: Vec<_> = ObservablePropertyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ObservablePropertyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(sub.contains(&(
            ObservablePropertyConcept::Heading,
            ObservablePropertyConcept::Yaw
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn roll_pitch_yaw_are_attitude() {
        // The classic Euler-angle decomposition: Attitude is composed of Roll, Pitch, Yaw.
        let sub: Vec<_> = ObservablePropertyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ObservablePropertyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for component in [
            ObservablePropertyConcept::Roll,
            ObservablePropertyConcept::Pitch,
            ObservablePropertyConcept::Yaw,
        ] {
            assert!(
                sub.contains(&(component, ObservablePropertyConcept::Attitude)),
                "{component:?} should be_a Attitude"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn position_causes_velocity_via_differentiation() {
        let caus: Vec<_> = ObservablePropertyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ObservablePropertyRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(caus.contains(&(
            ObservablePropertyConcept::Position,
            ObservablePropertyConcept::Velocity
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn force_causes_acceleration() {
        // Newton's second law: F = ma — Force causes Acceleration given Mass.
        let caus: Vec<_> = ObservablePropertyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ObservablePropertyRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(caus.contains(&(
            ObservablePropertyConcept::Force,
            ObservablePropertyConcept::Acceleration
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn differentiation_chain_dimensionally_consistent() {
        // dim(velocity) = dim(position)/T, etc. — a check the prose-string
        // form of PropertyDimension could not express.
        assert!(
            DifferentiationChainDimensionallyConsistent.verify().is_ok(),
            "time-differentiation chain is not dimensionally consistent",
        );
        // Spot-check the SI identity directly.
        let q = PropertyDimension;
        assert_eq!(
            q.get(&ObservablePropertyConcept::Velocity).unwrap(),
            q.get(&ObservablePropertyConcept::Position)
                .unwrap()
                .divide(&Dimension::TIME),
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn dimension_total_on_concrete_properties() {
        // PropertyDimension is None on the abstract trio, Some on every
        // concrete property.
        let q = PropertyDimension;
        let abstract_concepts = [
            ObservablePropertyConcept::Property,
            ObservablePropertyConcept::ObservableProperty,
            ObservablePropertyConcept::ActuatableProperty,
        ];
        for c in ObservablePropertyConcept::variants() {
            let v = q.get(&c);
            if abstract_concepts.contains(&c) {
                assert!(v.is_none(), "{:?} abstract should have None dimension", c);
            } else {
                assert!(v.is_some(), "{:?} concrete should have Some dimension", c);
            }
        }
    }

    fn arb_concept() -> impl Strategy<Value = ObservablePropertyConcept> {
        proptest::sample::select(ObservablePropertyConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in ObservablePropertyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ObservablePropertyOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_dimension_total_on_concrete(c in arb_concept()) {
            // Total on every non-abstract concept; None on the three SSN
            // abstract concepts (Property, ObservableProperty, ActuatableProperty).
            let v = PropertyDimension.get(&c);
            let is_abstract = matches!(
                c,
                ObservablePropertyConcept::Property
                | ObservablePropertyConcept::ObservableProperty
                | ObservablePropertyConcept::ActuatableProperty
            );
            prop_assert_eq!(v.is_some(), !is_abstract);
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = ObservablePropertyConcept::variants();
            for m in ObservablePropertyCategory::morphisms() {
                if m.kind() == ObservablePropertyRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_dimension_total_on_concrete, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
}
