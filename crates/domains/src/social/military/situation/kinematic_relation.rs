//! Inter-entity kinematic-relationship classification as an ONTOLOGY, not
//! magic-number thresholds.
//!
//! Two tracked entities stand in a relationship determined by their *relative
//! motion* — proximity, relative speed, and range-rate. Previously
//! `classify_relationship` decided that relationship from inline literals
//! (`dist < 100.0`, `rel_speed < 0.5`, `closing_rate < -1.0`): domain knowledge
//! smuggled into imperative code as magic numbers, un-cited and un-checkable.
//!
//! This module makes the decision ontological. The relationship kinds are a
//! cited `ontology!`; the thresholds are typed, cited [`Quantity`] criteria
//! ([`RelationCriteria`]) — *load, don't encode* — that a deployment tunes; a
//! measurement is a typed [`RelativeKinematics`]; and [`classify`] matches the
//! measurement against each concept's declared criterion, so the classification
//! reads the ontology instead of hardcoding it. The semantics of the result are
//! discharged as [`Axiom`]s.
//!
//! # Literature
//!
//! - **Laube, Imfeld & Weibel (2005)** "Finding REMO — Detecting Relative
//!   Motion Patterns in Geospatial Lifelines", in *Developments in Spatial Data
//!   Handling* (SDH 2004), Springer, pp. 201–215 — the REMO framework: relative
//!   motion patterns (flock, leadership, convergence, encounter) among moving
//!   point objects, parameterised by proximity, speed, and direction
//!   similarity. Grounds Formation (flock), Following (leadership), and
//!   Converging (convergence).
//! - **Jeung, Yiu, Zhou, Jensen & Shen (2008)** "Discovery of Convoys in
//!   Trajectory Databases", *Proc. VLDB Endowment* 1(1):1068–1080 — a convoy is
//!   a group of objects travelling together over time; grounds Following.
//! - **Blackman & Popoli (1999)** *Design and Analysis of Modern Tracking
//!   Systems*, Artech House — group / formation tracking, and range-rate as the
//!   radial component of relative velocity.

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use crate::applied::sensor_fusion::frame::reference::ReferenceFrame;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::quantity::unit::{METER, METER_PER_SECOND};
use crate::formal::math::quantity::value::Quantity;

pr4xis::ontology! {
    name: "KinematicRelation",
    source: "Laube, Imfeld & Weibel (2005) Finding REMO — Detecting Relative Motion Patterns in Geospatial Lifelines, in Developments in Spatial Data Handling (SDH 2004), Springer, pp. 201-215; Jeung, Yiu, Zhou, Jensen & Shen (2008) Discovery of Convoys in Trajectory Databases, Proc. VLDB Endowment 1(1):1068-1080; Blackman & Popoli (1999) Design and Analysis of Modern Tracking Systems, Artech House",

    concepts: [
        Formation,
        Following,
        Converging,
        Diverging,
        Unrelated,
    ],

    labels: {
        Formation: ("en", "Formation",
            "Laube et al. (2005) flock pattern: two entities co-moving (low relative speed) in close proximity — a tight group holding station."),
        Following: ("en", "Following",
            "Laube et al. (2005) leadership / Jeung et al. (2008) convoy: co-moving (low relative speed) beyond formation proximity, neither closing nor opening — a trailing / convoy relation."),
        Converging: ("en", "Converging",
            "Laube et al. (2005) convergence: entities on approaching paths — range-rate significantly negative (Blackman & Popoli 1999)."),
        Diverging: ("en", "Diverging",
            "The dual of convergence: entities on separating paths — range-rate significantly positive."),
        Unrelated: ("en", "Unrelated",
            "No significant relative-motion pattern under the criteria."),
    },

    opposes: [
        // Approaching vs separating are polar relative-motion patterns.
        (Converging, Diverging),
        (Diverging, Converging),
    ],
}

/// The closure tendency of a kinematic relationship — whether the entities are
/// drawing together, apart, holding station, or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClosureTendency {
    /// Range decreasing (Converging).
    Approaching,
    /// Range increasing (Diverging).
    Separating,
    /// Range roughly constant, moving together (Formation / Following).
    CoMoving,
    /// No significant tendency (Unrelated).
    Indeterminate,
}

/// Quality: the [`ClosureTendency`] of each relationship kind.
#[derive(Debug, Clone)]
pub struct ClosureTendencyOf;

impl Quality for ClosureTendencyOf {
    type Individual = KinematicRelationConcept;
    type Value = ClosureTendency;
    const KIND: QualityKind = QualityKind::Physical;

    fn get(&self, r: &KinematicRelationConcept) -> Option<ClosureTendency> {
        use KinematicRelationConcept as R;
        Some(match r {
            R::Formation | R::Following => ClosureTendency::CoMoving,
            R::Converging => ClosureTendency::Approaching,
            R::Diverging => ClosureTendency::Separating,
            R::Unrelated => ClosureTendency::Indeterminate,
        })
    }
}

impl Ontology for KinematicRelationOntology {
    type Cat = KinematicRelationCategory;
    type Qual = ClosureTendencyOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ClassificationIsSound));
        axioms.push(Box::new(RelationSemanticsRespectRangeRate));
        axioms.push(Box::new(CoMovingRelationsAreSlow));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Typed criteria (the thresholds, as cited ontological parameters)
// ---------------------------------------------------------------------------

/// The relative-motion thresholds that operationalise the relationship
/// concepts — typed, cited [`Quantity`] parameters, not magic numbers.
///
/// In the REMO framework (Laube et al. 2005) proximity, speed-similarity, and
/// direction are *analysis parameters*, not universal constants; a deployment
/// supplies its own values. [`RelationCriteria::standard`] provides illustrative
/// defaults so the classification is runnable, but the point is that the
/// decision reads these named quantities rather than inlining a literal.
#[derive(Debug, Clone)]
pub struct RelationCriteria {
    /// Max separation for a flock/formation (proximity radius). LENGTH.
    pub formation_radius: Quantity,
    /// Max relative speed for two tracks to count as co-moving (speed/direction
    /// similarity, projected onto the relative-velocity magnitude). VELOCITY.
    pub co_motion_speed: Quantity,
    /// Range-rate magnitude above which approach/separation is a significant
    /// convergence/divergence. VELOCITY.
    pub significant_range_rate: Quantity,
}

impl RelationCriteria {
    /// Illustrative defaults grounded in the REMO / group-tracking parameters
    /// (Laube et al. 2005; Blackman & Popoli 1999). A deployment overrides these
    /// for its scenario — they are the tuning surface, declared as typed
    /// quantities rather than hardcoded in the classifier.
    pub fn standard() -> Self {
        Self {
            formation_radius: Quantity::from_unit(100.0, &METER),
            co_motion_speed: Quantity::from_unit(0.5, &METER_PER_SECOND),
            significant_range_rate: Quantity::from_unit(1.0, &METER_PER_SECOND),
        }
    }
}

// ---------------------------------------------------------------------------
// Typed measurement
// ---------------------------------------------------------------------------

/// The relative kinematics between two entities, as typed quantities.
#[derive(Debug, Clone)]
pub struct RelativeKinematics {
    /// Separation `|p_b − p_a|`. LENGTH, ≥ 0.
    pub separation: Quantity,
    /// Relative speed `|v_b − v_a|`. VELOCITY, ≥ 0.
    pub relative_speed: Quantity,
    /// Range-rate `d/dt |p_b − p_a|` — the radial component of the relative
    /// velocity. VELOCITY, signed (negative = approaching). By Cauchy–Schwarz
    /// `|range_rate| ≤ relative_speed` (Blackman & Popoli 1999).
    pub range_rate: Quantity,
}

impl RelativeKinematics {
    /// Derive the relative kinematics of `b` with respect to `a` from their
    /// kinematic states — dimension-general position/velocity [`Vector`]s, each
    /// expressed in a [`ReferenceFrame`].
    ///
    /// Returns `None` unless both entities are in the **same** reference frame
    /// (relative motion between differently-framed vectors is undefined — a
    /// frame transform must precede this) and their position/velocity vectors
    /// share one dimension. Works in 2-D, 3-D, or any dimension; nothing here is
    /// hardcoded to a plane.
    ///
    /// `range_rate = (Δp · Δv) / |Δp|` (the radial projection); zero at zero
    /// separation, where the range-rate is undefined.
    pub fn from_states(
        frame_a: ReferenceFrame,
        position_a: &Vector,
        velocity_a: &Vector,
        frame_b: ReferenceFrame,
        position_b: &Vector,
        velocity_b: &Vector,
    ) -> Option<Self> {
        if frame_a != frame_b
            || position_a.dim() != position_b.dim()
            || velocity_a.dim() != velocity_b.dim()
            || position_a.dim() != velocity_a.dim()
        {
            return None;
        }
        let dp = position_b.sub(position_a);
        let dv = velocity_b.sub(velocity_a);
        let sep = dp.norm().value;
        let rel_speed = dv.norm().value;
        let range_rate = if sep > 0.0 {
            dp.dot(&dv).value / sep
        } else {
            0.0
        };
        Some(Self {
            separation: Quantity::from_unit(sep, &METER),
            relative_speed: Quantity::from_unit(rel_speed, &METER_PER_SECOND),
            range_rate: Quantity::from_unit(range_rate, &METER_PER_SECOND),
        })
    }
}

// ---------------------------------------------------------------------------
// Classification — matches the measurement against each concept's criterion
// ---------------------------------------------------------------------------

impl KinematicRelationConcept {
    /// Does `k` satisfy THIS relation's defining criterion under `c`?
    ///
    /// Every threshold comparison reads a named [`RelationCriteria`] quantity;
    /// there are no inline literals. `Unrelated` is the catch-all.
    pub fn matches(&self, k: &RelativeKinematics, c: &RelationCriteria) -> bool {
        use KinematicRelationConcept as R;
        let co_moving = k.relative_speed.value < c.co_motion_speed.value;
        let proximate = k.separation.value < c.formation_radius.value;
        let approaching = k.range_rate.value < -c.significant_range_rate.value;
        let separating = k.range_rate.value > c.significant_range_rate.value;
        match self {
            R::Formation => co_moving && proximate,
            R::Following => co_moving && !proximate && !approaching && !separating,
            R::Converging => approaching,
            R::Diverging => separating,
            R::Unrelated => true,
        }
    }
}

/// Priority order: specific relations before the `Unrelated` catch-all. Because
/// `|range_rate| ≤ relative_speed`, a co-moving pair is never approaching or
/// separating, so the co-moving relations (Formation, Following) and the
/// closure relations (Converging, Diverging) are mutually exclusive; the order
/// only ensures `Unrelated` is last.
const CLASSIFICATION_PRIORITY: [KinematicRelationConcept; 5] = [
    KinematicRelationConcept::Formation,
    KinematicRelationConcept::Converging,
    KinematicRelationConcept::Diverging,
    KinematicRelationConcept::Following,
    KinematicRelationConcept::Unrelated,
];

/// Classify relative kinematics by matching against the ontology's declared,
/// cited criteria — the ontological replacement for a magic-number `if` cascade.
pub fn classify(k: &RelativeKinematics, c: &RelationCriteria) -> KinematicRelationConcept {
    CLASSIFICATION_PRIORITY
        .into_iter()
        .find(|r| r.matches(k, c))
        .unwrap_or(KinematicRelationConcept::Unrelated)
}

// ---------------------------------------------------------------------------
// Axioms — the classification's semantics, made checkable
// ---------------------------------------------------------------------------

/// Physically valid relative-kinematics fixtures: `|range_rate| ≤ relative_speed`
/// (the range-rate is the radial component of the relative velocity).
fn physical_fixtures() -> Vec<RelativeKinematics> {
    let mut v: Vec<RelativeKinematics> = Vec::new();
    for &sep in &[5.0_f64, 60.0, 200.0, 1000.0] {
        for &spd in &[0.05_f64, 0.4, 2.0, 8.0] {
            for &frac in &[-1.0_f64, -0.5, 0.0, 0.5, 1.0] {
                v.push(RelativeKinematics {
                    separation: Quantity::from_unit(sep, &METER),
                    relative_speed: Quantity::from_unit(spd, &METER_PER_SECOND),
                    range_rate: Quantity::from_unit(spd * frac, &METER_PER_SECOND),
                });
            }
        }
    }
    v
}

/// Axiom: the classification is *sound* — the concept it returns genuinely
/// satisfies its own criterion for the input.
///
/// A priority resolution that returned a concept whose criterion the input did
/// not meet would be a bug; this folds every physical fixture and checks
/// `classify(k).matches(k)`.
pub struct ClassificationIsSound;

impl Axiom for ClassificationIsSound {
    fn verify(&self) -> Verdict {
        let c = RelationCriteria::standard();
        let sound = physical_fixtures()
            .iter()
            .all(|k| classify(k, &c).matches(k, &c));
        if sound {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ClassificationIsSound",
        "classify(k) returns a relation whose own criterion k satisfies (sound priority resolution)",
        "Laube, Imfeld & Weibel (2005) Finding REMO, in Developments in Spatial Data Handling, Springer pp. 201-215"
    );
}
pr4xis::register_axiom!(
    ClassificationIsSound,
    "Laube, Imfeld & Weibel (2005) Finding REMO, in Developments in Spatial Data Handling, Springer pp. 201-215"
);

/// Axiom: convergence means approaching, divergence means separating.
///
/// For every physical fixture, a `Converging` classification implies a negative
/// range-rate and a `Diverging` classification implies a positive one — the
/// range-rate sign IS the convergence/divergence semantics (Laube et al. 2005;
/// Blackman & Popoli 1999). A classification that labelled a receding pair
/// `Converging` would be refuted here.
pub struct RelationSemanticsRespectRangeRate;

impl Axiom for RelationSemanticsRespectRangeRate {
    fn verify(&self) -> Verdict {
        use KinematicRelationConcept as R;
        let c = RelationCriteria::standard();
        let ok = physical_fixtures().iter().all(|k| match classify(k, &c) {
            R::Converging => k.range_rate.value < 0.0,
            R::Diverging => k.range_rate.value > 0.0,
            _ => true,
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RelationSemanticsRespectRangeRate",
        "Converging implies negative range-rate (approaching); Diverging implies positive (separating)",
        "Laube, Imfeld & Weibel (2005) Finding REMO; Blackman & Popoli (1999) Design and Analysis of Modern Tracking Systems"
    );
}
pr4xis::register_axiom!(
    RelationSemanticsRespectRangeRate,
    "Laube, Imfeld & Weibel (2005) Finding REMO; Blackman & Popoli (1999) Design and Analysis of Modern Tracking Systems"
);

/// Axiom: the co-moving relations are slow.
///
/// Formation and Following are the REMO co-moving patterns (flock / leadership);
/// for every physical fixture so classified, the relative speed is below the
/// co-motion threshold. A fast-relative pair can never be a formation.
pub struct CoMovingRelationsAreSlow;

impl Axiom for CoMovingRelationsAreSlow {
    fn verify(&self) -> Verdict {
        use KinematicRelationConcept as R;
        let c = RelationCriteria::standard();
        let ok = physical_fixtures().iter().all(|k| match classify(k, &c) {
            R::Formation | R::Following => k.relative_speed.value < c.co_motion_speed.value,
            _ => true,
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CoMovingRelationsAreSlow",
        "Formation and Following imply relative speed below the co-motion threshold (the REMO flock/leadership speed-similarity criterion)",
        "Laube, Imfeld & Weibel (2005) Finding REMO, in Developments in Spatial Data Handling, Springer pp. 201-215"
    );
}
pr4xis::register_axiom!(
    CoMovingRelationsAreSlow,
    "Laube, Imfeld & Weibel (2005) Finding REMO, in Developments in Spatial Data Handling, Springer pp. 201-215"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<KinematicRelationCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        KinematicRelationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn converging_diverging_oppose() {
        let opp: Vec<_> = KinematicRelationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == KinematicRelationRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(
            KinematicRelationConcept::Converging,
            KinematicRelationConcept::Diverging
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn classification_axioms_hold() {
        assert!(ClassificationIsSound.verify().is_ok());
        assert!(RelationSemanticsRespectRangeRate.verify().is_ok());
        assert!(CoMovingRelationsAreSlow.verify().is_ok());
    }

    /// Build a same-frame `RelativeKinematics` from planar (x,y) pairs (test helper).
    fn planar(
        pa: (f64, f64),
        va: (f64, f64),
        pb: (f64, f64),
        vb: (f64, f64),
    ) -> RelativeKinematics {
        let v = |x: f64, y: f64| Vector::new(vec![x, y]);
        RelativeKinematics::from_states(
            ReferenceFrame::NED,
            &v(pa.0, pa.1),
            &v(va.0, va.1),
            ReferenceFrame::NED,
            &v(pb.0, pb.1),
            &v(vb.0, vb.1),
        )
        .expect("common NED frame, equal dimension → defined")
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn known_geometries_classify_as_expected() {
        let c = RelationCriteria::standard();
        // Close + co-moving → Formation (a flock).
        assert_eq!(
            classify(
                &planar((0.0, 0.0), (10.0, 0.0), (20.0, 5.0), (10.1, 0.0)),
                &c
            ),
            KinematicRelationConcept::Formation
        );
        // Head-on approach → Converging.
        assert_eq!(
            classify(
                &planar((0.0, 0.0), (5.0, 0.0), (1000.0, 0.0), (-5.0, 0.0)),
                &c
            ),
            KinematicRelationConcept::Converging
        );
        // Receding → Diverging.
        assert_eq!(
            classify(
                &planar((0.0, 0.0), (-5.0, 0.0), (1000.0, 0.0), (5.0, 0.0)),
                &c
            ),
            KinematicRelationConcept::Diverging
        );
        // Co-moving but far apart → Following (a convoy).
        assert_eq!(
            classify(
                &planar((0.0, 0.0), (10.0, 0.0), (500.0, 0.0), (10.05, 0.0)),
                &c
            ),
            KinematicRelationConcept::Following
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn classifies_in_three_dimensions() {
        // Nothing is hardwired to a plane: a 3-D head-on approach along the
        // z-axis classifies as Converging, using the same dimension-general path.
        let c = RelationCriteria::standard();
        let f = ReferenceFrame::ECEF;
        let k = RelativeKinematics::from_states(
            f,
            &Vector::new(vec![0.0, 0.0, 0.0]),
            &Vector::new(vec![0.0, 0.0, 5.0]),
            f,
            &Vector::new(vec![0.0, 0.0, 1000.0]),
            &Vector::new(vec![0.0, 0.0, -5.0]),
        )
        .expect("common frame, 3-D, equal dimension → defined");
        assert_eq!(classify(&k, &c), KinematicRelationConcept::Converging);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn from_states_requires_common_frame_and_dimension() {
        let v2 = |x: f64, y: f64| Vector::new(vec![x, y]);
        // Different frames → undefined (None).
        assert!(
            RelativeKinematics::from_states(
                ReferenceFrame::NED,
                &v2(0.0, 0.0),
                &v2(1.0, 0.0),
                ReferenceFrame::ECEF,
                &v2(10.0, 0.0),
                &v2(1.0, 0.0),
            )
            .is_none()
        );
        // Mismatched dimensions → undefined (None).
        assert!(
            RelativeKinematics::from_states(
                ReferenceFrame::NED,
                &v2(0.0, 0.0),
                &v2(1.0, 0.0),
                ReferenceFrame::NED,
                &Vector::new(vec![10.0, 0.0, 0.0]),
                &Vector::new(vec![1.0, 0.0, 0.0]),
            )
            .is_none()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn range_rate_bounded_by_relative_speed() {
        // The engine's derived range-rate is the radial component of the
        // relative velocity: |range_rate| ≤ relative_speed (Cauchy–Schwarz).
        let k = planar((0.0, 0.0), (3.0, 4.0), (100.0, 50.0), (-2.0, 7.0));
        assert!(k.range_rate.value.abs() <= k.relative_speed.value + 1e-9);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn closure_tendency_total() {
        let q = ClosureTendencyOf;
        for r in KinematicRelationConcept::variants() {
            assert!(q.get(&r).is_some(), "{r:?} missing closure tendency");
        }
    }

    proptest! {
        #[test]
        fn prop_classification_is_sound(
            sep in 0.0_f64..2000.0,
            spd in 0.0_f64..20.0,
            frac in -1.0_f64..=1.0,
        ) {
            // Any physical measurement (|range_rate| ≤ relative_speed) is
            // classified into a relation whose criterion it satisfies.
            let c = RelationCriteria::standard();
            let k = RelativeKinematics {
                separation: Quantity::from_unit(sep, &METER),
                relative_speed: Quantity::from_unit(spd, &METER_PER_SECOND),
                range_rate: Quantity::from_unit(spd * frac, &METER_PER_SECOND),
            };
            prop_assert!(classify(&k, &c).matches(&k, &c));
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in KinematicRelationCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
    }

    pr4xis::register_praxis_value!(prop_classification_is_sound, Verifiable);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
}
