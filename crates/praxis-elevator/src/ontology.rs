use praxis_category::{Axiom as CatAxiom, Category, Entity, FullyConnected, Relationship};
use praxis_ontology::Quality;

/// Floors are the entities of the elevator ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Floor(pub usize);

impl Floor {
    pub fn new(n: usize, max: usize) -> Self {
        assert!(n < max);
        Floor(n)
    }
}

/// Floor entity for a building with N floors.
/// Since Entity::variants() takes no args, we use a fixed 10-floor building.
const MAX_FLOORS: usize = 10;

impl Entity for Floor {
    fn variants() -> Vec<Self> {
        (0..MAX_FLOORS).map(Floor).collect()
    }
}

/// Travel between floors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Travel {
    pub from: Floor,
    pub to: Floor,
}

impl Relationship for Travel {
    type Object = Floor;
    fn source(&self) -> Floor {
        self.from
    }
    fn target(&self) -> Floor {
        self.to
    }
}

pub struct ElevatorCategory;

impl Category for ElevatorCategory {
    type Object = Floor;
    type Morphism = Travel;

    fn identity(obj: &Floor) -> Travel {
        Travel {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &Travel, g: &Travel) -> Option<Travel> {
        if f.to != g.from {
            return None;
        }
        Some(Travel {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<Travel> {
        let floors = Floor::variants();
        floors
            .iter()
            .flat_map(|&a| floors.iter().map(move |&b| Travel { from: a, to: b }))
            .collect()
    }
}

/// Quality: distance between a floor and ground.
#[derive(Debug, Clone)]
pub struct HeightFromGround;

impl Quality for HeightFromGround {
    type Individual = Floor;
    type Value = usize;

    fn get(&self, floor: &Floor) -> Option<usize> {
        Some(floor.0)
    }
}

/// Check: all floors are reachable (fully connected).
pub fn all_floors_reachable() -> bool {
    let axiom = FullyConnected::<ElevatorCategory>::new();
    CatAxiom::<ElevatorCategory>::holds(&axiom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10_floors() {
        assert_eq!(Floor::variants().len(), 10);
    }

    #[test]
    fn test_category_laws() {
        praxis_category::validate::check_category_laws::<ElevatorCategory>().unwrap();
    }

    #[test]
    fn test_height_quality() {
        assert_eq!(HeightFromGround.get(&Floor(0)), Some(0));
        assert_eq!(HeightFromGround.get(&Floor(5)), Some(5));
    }

    #[test]
    fn test_fully_connected() {
        assert!(all_floors_reachable());
    }
}
