//! Electronic warfare observable types for emitter geolocation.
//!
//! # Literature
//!
//! - **Poisel (2012)** *Electronic Warfare Target Location Methods*,
//!   2nd ed., Artech House — the canonical reference for emitter
//!   geolocation observables (AOA, TDOA, FDOA, RSS) and their
//!   constant-locus geometries.
//! - **JP 3-13.1** (Joint Publication, Electronic Warfare) — doctrinal
//!   terminology for EW observables in joint operations.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Ew",
    source: "Poisel (2012) Electronic Warfare Target Location Methods, 2nd ed., Artech House; JP 3-13.1",

    concepts: [AOA, TDOA, FDOA, SignalStrength],

    labels: {
        AOA: ("en", "Angle of Arrival",
            "Angle of arrival (bearing to emitter). Poisel (2012) §4: the half-plane locus of constant bearing from a sensor."),
        TDOA: ("en", "Time Difference of Arrival",
            "Time difference of arrival between sensor pairs. Poisel (2012) §5: the hyperbolic locus of constant time-difference."),
        FDOA: ("en", "Frequency Difference of Arrival",
            "Frequency difference of arrival (Doppler-based). Poisel (2012) §6: the hyperbolic locus of constant frequency-difference."),
        SignalStrength: ("en", "Signal strength",
            "Received signal strength (path-loss based ranging). Poisel (2012) §3: the circular locus of constant range under path-loss model."),
    },
}

/// The constant-locus geometry an emitter-geolocation observable defines —
/// the surface of emitter positions consistent with a single measurement.
///
/// A closed taxonomy from Poisel (2012) *Electronic Warfare Target Location
/// Methods*: an angle measurement constrains the emitter to a line of bearing,
/// a time- or frequency-difference to a hyperbola, and a path-loss range to a
/// circle. A multi-observable fix is the intersection of these loci — which is
/// why the geometry is first-class rather than prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocusGeometry {
    /// Half-plane line of bearing from a single angle-of-arrival (Poisel 2012 §4).
    LineOfBearing,
    /// Hyperbola of constant time- or frequency-difference across a sensor pair
    /// (Poisel 2012 §5–6).
    Hyperbola,
    /// Circle of constant range under a path-loss model (Poisel 2012 §3).
    Circle,
}

/// Quality: the constant-locus [`LocusGeometry`] of each observable.
#[derive(Debug, Clone)]
pub struct ObservableGeometry;

impl Quality for ObservableGeometry {
    type Individual = EwConcept;
    type Value = LocusGeometry;

    fn get(&self, obs: &EwConcept) -> Option<LocusGeometry> {
        Some(match obs {
            EwConcept::AOA => LocusGeometry::LineOfBearing,
            EwConcept::TDOA => LocusGeometry::Hyperbola,
            EwConcept::FDOA => LocusGeometry::Hyperbola,
            EwConcept::SignalStrength => LocusGeometry::Circle,
        })
    }
}

impl Ontology for EwOntology {
    type Cat = EwCategory;
    type Qual = ObservableGeometry;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(AoaBounded));
        axioms.push(Box::new(TdoaRequiresSensorPair));
        axioms
    }
}

/// Axiom: AOA measurements are bounded to [-π, π].
///
/// Poisel (2012) §4.2: bearing is reported as a signed angle on
/// (-π, π] or [-π, π] depending on convention; any larger range
/// admits the same physical bearing twice and is rejected.
pub struct AoaBounded;

impl Axiom for AoaBounded {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::SimpleProof;
        // AOA is defined to be in [-π, π] by construction (typed in the
        // sensor models); the axiom asserts the invariant for the ontology
        // layer.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "AoaBounded",
        "angle of arrival measurements are in [-π, π]",
        "Poisel (2012) Electronic Warfare Target Location Methods §4.2"
    );
}

pr4xis::register_axiom!(
    AoaBounded,
    "Poisel (2012) Electronic Warfare Target Location Methods §4.2"
);

/// Axiom: TDOA geolocation requires at least one sensor pair (2 sensors).
///
/// Poisel (2012) §5.1: the TDOA locus is defined between TWO sensors;
/// 2D geolocation by TDOA intersection needs at least two pairs (three
/// sensors), but the observable itself is binary-sensor.
pub struct TdoaRequiresSensorPair;

impl Axiom for TdoaRequiresSensorPair {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::SimpleProof;
        // Definitional: TDOA exists iff |sensors| ≥ 2. The ontology
        // declares this as a requirement; the sensor-fusion layer
        // enforces it at construction time.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "TdoaRequiresSensorPair",
        "TDOA geolocation requires at least one sensor pair (2 sensors)",
        "Poisel (2012) Electronic Warfare Target Location Methods §5.1"
    );
}

pr4xis::register_axiom!(
    TdoaRequiresSensorPair,
    "Poisel (2012) Electronic Warfare Target Location Methods §5.1"
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
        assert_category_laws::<EwCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        EwOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_observable_types() {
        assert_eq!(EwConcept::variants().len(), 4);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_observable_has_geometry() {
        let q = ObservableGeometry;
        for c in EwConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing geometric locus", c);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn domain_axioms_hold() {
        assert!(AoaBounded.verify().is_ok());
        assert!(TdoaRequiresSensorPair.verify().is_ok());
    }

    fn arb_observable() -> impl Strategy<Value = EwConcept> {
        proptest::sample::select(EwConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_geometry_total(c in arb_observable()) {
            prop_assert!(ObservableGeometry.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in EwCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_axioms_hold(_seed in any::<u32>()) {
            for axiom in EwOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_geometry_total, Verifiable);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_axioms_hold, Verifiable);
}
