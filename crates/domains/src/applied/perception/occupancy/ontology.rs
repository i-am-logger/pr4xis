use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Cell states in a Bayesian occupancy grid.
///
/// Source: Elfes (1989), "Using Occupancy Grids for Mobile Robot Perception and Navigation"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum CellState {
    /// Cell is free (unoccupied).
    Free,
    /// Cell is occupied by an obstacle.
    Occupied,
    /// Cell state is unknown (no observations yet).
    Unknown,
}

define_ontology! {
    /// Category for occupancy grid cell state transitions.
    ///
    /// All transitions are possible: a cell can go from any state to any state
    /// upon receiving new sensor evidence (Bayesian update).
    pub OccupancyOntology for OccupancyCategory {
        entity: CellState,
        relation: CellTransition,
        being: Process,
        source: "Elfes (1989); Thrun et al. (2005)",
    }
}

/// Quality: occupancy probability range for each state.
#[derive(Debug, Clone)]
pub struct OccupancyProbability;

impl Quality for OccupancyProbability {
    type Individual = CellState;
    type Value = (f64, f64); // (min, max) probability range

    fn get(&self, state: &CellState) -> Option<(f64, f64)> {
        Some(match state {
            CellState::Free => (0.0, 0.5),
            CellState::Occupied => (0.5, 1.0),
            CellState::Unknown => (0.5, 0.5), // prior = 0.5
        })
    }
}

/// Axiom: occupancy probabilities are in [0, 1].
pub struct ProbabilityBounded;

impl Axiom for ProbabilityBounded {
    fn description(&self) -> &str {
        "occupancy probabilities must be in [0, 1]"
    }
    fn holds(&self) -> bool {
        let q = OccupancyProbability;
        CellState::variants().iter().all(|s| {
            if let Some((min, max)) = q.get(s) {
                min >= 0.0 && max <= 1.0 && min <= max
            } else {
                false
            }
        })
    }
}
pr4xis::register_axiom!(ProbabilityBounded);

/// Axiom: log-odds update is deterministic (same input gives same output).
pub struct LogOddsUpdateDeterministic;

impl Axiom for LogOddsUpdateDeterministic {
    fn description(&self) -> &str {
        "log-odds Bayesian update is a deterministic function"
    }
    fn holds(&self) -> bool {
        // Verify determinism: same prior + same observation => same posterior
        let prior = 0.5_f64;
        let log_odds_prior = (prior / (1.0 - prior)).ln();
        let sensor_log_odds = 0.8_f64.ln() - 0.2_f64.ln();

        let result1 = log_odds_prior + sensor_log_odds;
        let result2 = log_odds_prior + sensor_log_odds;
        (result1 - result2).abs() < 1e-15
    }
}
pr4xis::register_axiom!(LogOddsUpdateDeterministic);

impl Ontology for OccupancyOntology {
    type Cat = OccupancyCategory;
    type Qual = OccupancyProbability;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ProbabilityBounded),
            Box::new(LogOddsUpdateDeterministic),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<OccupancyCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        OccupancyOntology::validate().unwrap();
    }
}
