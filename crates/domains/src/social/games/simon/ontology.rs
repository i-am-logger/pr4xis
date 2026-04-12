use super::color::SimonColor;
use pr4xis::category::Entity;
use pr4xis::ontology::Quality;

impl Entity for SimonColor {
    fn variants() -> Vec<Self> {
        SimonColor::all().to_vec()
    }
}

/// Quality: index position in the button array.
#[derive(Debug, Clone)]
pub struct ButtonIndex;

impl Quality for ButtonIndex {
    type Individual = SimonColor;
    type Value = usize;

    fn get(&self, color: &SimonColor) -> Option<usize> {
        Some(match color {
            SimonColor::Red => 0,
            SimonColor::Blue => 1,
            SimonColor::Green => 2,
            SimonColor::Yellow => 3,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_4_colors() {
        assert_eq!(SimonColor::variants().len(), 4);
    }

    #[test]
    fn test_button_index() {
        assert_eq!(ButtonIndex.get(&SimonColor::Red), Some(0));
        assert_eq!(ButtonIndex.get(&SimonColor::Yellow), Some(3));
        assert_eq!(ButtonIndex.individuals_with().len(), 4);
    }
}
