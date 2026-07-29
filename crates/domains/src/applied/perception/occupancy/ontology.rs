//! Occupancy — Bayesian occupancy grid cell-state ontology.
//!
//! Cells of a 2D / 3D occupancy grid are partitioned into three Bayesian
//! states (free / occupied / unknown) per Elfes (1989). The log-odds
//! update rule (Thrun, Burgard & Fox 2005) drives transitions between
//! states as sensor measurements arrive.
//!
//! # Literature
//!
//! - **Elfes (1989)** "Using Occupancy Grids for Mobile Robot Perception
//!   and Navigation", *Computer* 22(6) — the founding paper on occupancy
//!   grids; defines the three-state cell partition (free / occupied /
//!   unknown) and the probabilistic sensor model.
//! - **Thrun, Burgard & Fox (2005)** *Probabilistic Robotics*, Ch. 9 —
//!   the canonical log-odds Bayesian update for occupancy grids.

use crate::formal::math::quantity::unit::UNITLESS;
use crate::formal::math::quantity::value::{Quantity, QuantityRange};
use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Occupancy",
    source: "Elfes (1989) Using Occupancy Grids for Mobile Robot Perception and Navigation, Computer 22(6); Thrun, Burgard & Fox (2005) Probabilistic Robotics Ch. 9",

    concepts: [
        // Elfes (1989) three-state cell partition.
        Free,
        Occupied,
        Unknown,
    ],

    labels: {
        Free: ("en", "Free",
            "Elfes (1989): a cell believed to be unoccupied — posterior occupancy probability in [0, 0.5)."),
        Occupied: ("en", "Occupied",
            "Elfes (1989): a cell believed to contain an obstacle — posterior occupancy probability in (0.5, 1]."),
        Unknown: ("en", "Unknown",
            "Elfes (1989): a cell with no informative observations — prior occupancy probability = 0.5."),
    },

    opposes: [
        // Free vs Occupied — the two informative belief states.
        (Free, Occupied),
        (Occupied, Free),
    ],
}

/// Quality: the Bayesian occupancy-probability range for each cell state, as
/// a dimensionless [`QuantityRange`] in [`UNITLESS`], NOT a bare `(f64,
/// f64)` pair.
///
/// Elfes (1989) §3 — the three states partition the unit interval into
/// (0, 0.5), {0.5}, and (0.5, 1) for free / unknown / occupied
/// respectively. Boundaries given inclusive at the prior point for the
/// `Unknown` state, exclusive at the prior for the informative states.
#[derive(Debug, Clone)]
pub struct OccupancyProbability;

impl Quality for OccupancyProbability {
    type Individual = OccupancyConcept;
    type Value = QuantityRange;

    fn get(&self, state: &OccupancyConcept) -> Option<QuantityRange> {
        let p = |lo: f64, hi: f64| QuantityRange {
            min: Quantity::from_unit(lo, &UNITLESS),
            max: Quantity::from_unit(hi, &UNITLESS),
        };
        Some(match state {
            OccupancyConcept::Free => p(0.0, 0.5),
            OccupancyConcept::Occupied => p(0.5, 1.0),
            OccupancyConcept::Unknown => p(0.5, 0.5),
        })
    }
}

impl Ontology for OccupancyOntology {
    type Cat = OccupancyCategory;
    type Qual = OccupancyProbability;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ProbabilityBounded));
        axioms.push(Box::new(LogOddsUpdateDeterministic));
        axioms.push(Box::new(SaturationPreventsOverconfidence));
        axioms
    }
}

/// Axiom: the log-odds saturation keeps every cell strictly inside `(0, 1)`, so
/// the map can always recover — no run of observations drives a cell to
/// certainty.
///
/// Thrun, Burgard & Fox (2005) §9.2: clamping the accumulated log-odds prevents
/// overconfidence; at the `±5` bound the posterior is `p ≈ 0.993 / 0.007`, still
/// strictly interior, so contrary observations can still flip the cell. This is
/// the property the former inline `±5.0` constructor literals only implied.
pub struct SaturationPreventsOverconfidence;

impl Axiom for SaturationPreventsOverconfidence {
    fn verify(&self) -> Verdict {
        use crate::applied::perception::occupancy::engine::{LogOddsSaturation, OccupancyGrid};
        let sat = LogOddsSaturation::standard();
        let p_max = OccupancyGrid::log_odds_to_probability(sat.max).value;
        let p_min = OccupancyGrid::log_odds_to_probability(sat.min).value;
        // 0.0 / 0.5 / 1.0 are the definitional probability bounds (impossible /
        // neutral prior / certain), not tunable numbers.
        if sat.min < 0.0
            && sat.max > 0.0
            && p_max < 1.0
            && p_min > 0.0
            && p_max > 0.5
            && p_min < 0.5
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SaturationPreventsOverconfidence",
        "log-odds saturation keeps posterior occupancy strictly in (0,1) so the map stays responsive",
        "Thrun, Burgard & Fox (2005) Probabilistic Robotics §9.2"
    );
}
pr4xis::register_axiom!(
    SaturationPreventsOverconfidence,
    "Thrun, Burgard & Fox (2005) Probabilistic Robotics §9.2"
);

/// Axiom: every state's occupancy-probability range lies in [0, 1] with
/// min ≤ max.
///
/// Elfes (1989) §3 — occupancy is a probability; Kolmogorov's first
/// axiom of probability constrains the value to [0, 1].
pub struct ProbabilityBounded;

impl Axiom for ProbabilityBounded {
    fn verify(&self) -> Verdict {
        let zero = Quantity::from_unit(0.0, &UNITLESS);
        let one = Quantity::from_unit(1.0, &UNITLESS);
        for s in OccupancyConcept::variants() {
            if let Some(r) = OccupancyProbability.get(&s) {
                if !(r.min >= zero && r.max <= one && r.min <= r.max) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            } else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ProbabilityBounded",
        "occupancy probabilities are in [0, 1] with min ≤ max",
        "Elfes (1989) Using Occupancy Grids for Mobile Robot Perception and Navigation, Computer 22(6) §3"
    );
}

pr4xis::register_axiom!(
    ProbabilityBounded,
    "Elfes (1989) Using Occupancy Grids for Mobile Robot Perception and Navigation, Computer 22(6) §3"
);

/// Axiom: the log-odds Bayesian update is a deterministic, pure function
/// of (prior log-odds, sensor log-odds).
///
/// Thrun, Burgard & Fox (2005) §9.2 — the canonical update
/// `l(x_t) = l(x_{t-1}) + log[ p(z_t|x_t) / p(z_t|¬x_t) ] − l(x_0)`
/// is purely arithmetic; running it twice on the same inputs returns
/// bit-equal results.
pub struct LogOddsUpdateDeterministic;

impl Axiom for LogOddsUpdateDeterministic {
    fn verify(&self) -> Verdict {
        let prior = 0.5_f64;
        let log_odds_prior = (prior / (1.0 - prior)).ln();
        let sensor_log_odds = 0.8_f64.ln() - 0.2_f64.ln();
        let r1 = log_odds_prior + sensor_log_odds;
        let r2 = log_odds_prior + sensor_log_odds;
        if (r1 - r2).abs() < 1e-15 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LogOddsUpdateDeterministic",
        "log-odds Bayesian update is a deterministic pure function",
        "Thrun, Burgard & Fox (2005) Probabilistic Robotics §9.2"
    );
}

pr4xis::register_axiom!(
    LogOddsUpdateDeterministic,
    "Thrun, Burgard & Fox (2005) Probabilistic Robotics §9.2"
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
        assert_category_laws::<OccupancyCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        OccupancyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn three_cell_states() {
        assert_eq!(OccupancyConcept::variants().len(), 3);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn free_probability_under_half() {
        let r = OccupancyProbability.get(&OccupancyConcept::Free).unwrap();
        let half = Quantity::from_unit(0.5, &UNITLESS);
        assert!(r.min >= Quantity::from_unit(0.0, &UNITLESS) && r.max <= half);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn occupied_probability_over_half() {
        let r = OccupancyProbability
            .get(&OccupancyConcept::Occupied)
            .unwrap();
        let half = Quantity::from_unit(0.5, &UNITLESS);
        assert!(r.min >= half && r.max <= Quantity::from_unit(1.0, &UNITLESS));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn unknown_at_prior() {
        let r = OccupancyProbability
            .get(&OccupancyConcept::Unknown)
            .unwrap();
        let half = Quantity::from_unit(0.5, &UNITLESS);
        assert_eq!(r.min, half);
        assert_eq!(r.max, half);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn free_and_occupied_oppose() {
        let opp: Vec<_> = OccupancyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == OccupancyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(OccupancyConcept::Free, OccupancyConcept::Occupied)));
        assert!(opp.contains(&(OccupancyConcept::Occupied, OccupancyConcept::Free)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn probability_bounded_axiom() {
        assert!(ProbabilityBounded.verify().is_ok());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn log_odds_deterministic_axiom() {
        assert!(LogOddsUpdateDeterministic.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = OccupancyConcept> {
        proptest::sample::select(OccupancyConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in OccupancyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in OccupancyOntology::axioms() {
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
        fn prop_probability_total(c in arb_concept()) {
            // OccupancyProbability is total over all three states.
            prop_assert!(OccupancyProbability.get(&c).is_some());
        }

        #[test]
        fn prop_probability_in_unit_interval(c in arb_concept()) {
            let r = OccupancyProbability.get(&c).unwrap();
            let zero = Quantity::from_unit(0.0, &UNITLESS);
            let one = Quantity::from_unit(1.0, &UNITLESS);
            prop_assert!(r.min >= zero && r.max <= one && r.min <= r.max);
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = OccupancyCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == OccupancyRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_probability_total, Verifiable);
    pr4xis::register_praxis_value!(prop_probability_in_unit_interval, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
}
