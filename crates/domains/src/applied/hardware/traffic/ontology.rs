use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::{Axiom, Quality};

/// Traffic directions at an intersection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum TrafficDirection {
    North,
    South,
    East,
    West,
}

define_dense_category! {
    pub TrafficCategory {
        entity: TrafficDirection,
        relation: Conflict,
    }
}

/// Quality: does this direction conflict with North?
#[derive(Debug, Clone)]
pub struct ConflictsWithNorth;

impl Quality for ConflictsWithNorth {
    type Individual = TrafficDirection;
    type Value = ();

    fn get(&self, dir: &TrafficDirection) -> Option<()> {
        match dir {
            TrafficDirection::East | TrafficDirection::West => Some(()),
            _ => None,
        }
    }
}

/// Axiom: NS and EW are conflict pairs (not same-direction).
pub struct OrthogonalConflicts;

impl Axiom for OrthogonalConflicts {
    fn description(&self) -> &str {
        "north-south and east-west are orthogonal conflict pairs"
    }
    fn holds(&self) -> bool {
        // NS don't conflict with each other, EW don't conflict with each other
        // NS conflicts with EW
        ConflictsWithNorth.get(&TrafficDirection::East).is_some()
            && ConflictsWithNorth.get(&TrafficDirection::West).is_some()
            && ConflictsWithNorth.get(&TrafficDirection::South).is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_4_directions() {
        assert_eq!(TrafficDirection::variants().len(), 4);
    }

    #[test]
    fn test_category_laws() {
        pr4xis::category::validate::check_category_laws::<TrafficCategory>().unwrap();
    }

    #[test]
    fn test_conflicts_with_north() {
        assert_eq!(ConflictsWithNorth.individuals_with().len(), 2); // East, West
    }

    #[test]
    fn test_orthogonal_conflicts() {
        assert!(OrthogonalConflicts.holds());
    }
}
