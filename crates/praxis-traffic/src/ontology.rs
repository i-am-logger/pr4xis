use praxis_category::{Category, Entity, Relationship};
use praxis_ontology::{Axiom, Quality};

/// Traffic directions at an intersection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrafficDirection {
    North,
    South,
    East,
    West,
}

impl Entity for TrafficDirection {
    fn variants() -> Vec<Self> {
        vec![
            TrafficDirection::North,
            TrafficDirection::South,
            TrafficDirection::East,
            TrafficDirection::West,
        ]
    }
}

/// Conflict relationship: two directions that cannot both be green.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub a: TrafficDirection,
    pub b: TrafficDirection,
}

impl Relationship for Conflict {
    type Object = TrafficDirection;
    fn source(&self) -> TrafficDirection {
        self.a
    }
    fn target(&self) -> TrafficDirection {
        self.b
    }
}

pub struct TrafficCategory;

impl Category for TrafficCategory {
    type Object = TrafficDirection;
    type Morphism = Conflict;

    fn identity(obj: &TrafficDirection) -> Conflict {
        Conflict { a: *obj, b: *obj }
    }

    fn compose(f: &Conflict, g: &Conflict) -> Option<Conflict> {
        if f.b != g.a {
            return None;
        }
        Some(Conflict { a: f.a, b: g.b })
    }

    fn morphisms() -> Vec<Conflict> {
        let dirs = TrafficDirection::variants();
        dirs.iter()
            .flat_map(|&a| dirs.iter().map(move |&b| Conflict { a, b }))
            .collect()
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

impl Axiom<TrafficCategory> for OrthogonalConflicts {
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
        praxis_category::validate::check_category_laws::<TrafficCategory>().unwrap();
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
