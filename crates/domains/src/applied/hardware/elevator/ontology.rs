use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::Quality;

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

define_ontology! {
    /// The elevator category: floors are objects, travel is the morphism.
    /// Fully connected — any floor can reach any floor.
    pub ElevatorOntology for ElevatorCategory {
        concepts: Floor,
        relation: Travel,
        being: SocialObject,
        source: "Mandel (1989); Barney & Dos Santos (1985)",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10_floors() {
        assert_eq!(Floor::variants().len(), 10);
    }

    #[test]
    fn test_category_laws() {
        pr4xis::category::validate::check_category_laws::<ElevatorCategory>().unwrap();
    }

    #[test]
    fn test_height_quality() {
        assert_eq!(HeightFromGround.get(&Floor(0)), Some(0));
        assert_eq!(HeightFromGround.get(&Floor(5)), Some(5));
    }
}
