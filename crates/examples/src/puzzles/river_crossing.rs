use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

fn axiom_meta(name: &'static str, description: &'static str, citation: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

const RIVER_CITATION: &str = "Alcuin of York (c. 800) Propositiones ad acuendos juvenes, Problem 18: \
     Propositio de homine et capra et lupo";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bank {
    Left,
    Right,
}

impl Bank {
    pub fn opposite(&self) -> Bank {
        match self {
            Bank::Left => Bank::Right,
            Bank::Right => Bank::Left,
        }
    }
}

/// Wolf, goat, and cabbage river-crossing.
///
/// Source: Alcuin of York (c. 800), Problem 18.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub farmer: Bank,
    pub wolf: Bank,
    pub goat: Bank,
    pub cabbage: Bank,
}

impl State {
    pub fn initial() -> Self {
        Self {
            farmer: Bank::Left,
            wolf: Bank::Left,
            goat: Bank::Left,
            cabbage: Bank::Left,
        }
    }

    pub fn is_safe(&self) -> bool {
        (self.wolf != self.goat || self.farmer == self.wolf)
            && (self.goat != self.cabbage || self.farmer == self.goat)
    }

    /// Goal: every entity has crossed to the right bank.
    pub fn is_terminal(&self) -> bool {
        self.farmer == Bank::Right
            && self.wolf == Bank::Right
            && self.goat == Bank::Right
            && self.cabbage == Bank::Right
    }
}

impl Situation for State {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crossing {
    Alone,
    WithWolf,
    WithGoat,
    WithCabbage,
}

impl Action for Crossing {
    type Sit = State;
}

struct ItemWithFarmer;
impl Precondition<Crossing> for ItemWithFarmer {
    fn check(&self, s: &State, a: &Crossing) -> Verdict {
        let meta = axiom_meta(
            "item_with_farmer",
            "item must be on farmer's bank",
            RIVER_CITATION,
        );
        let bank = match a {
            Crossing::Alone => return Ok(Box::new(SimpleProof::new(meta))),
            Crossing::WithWolf => s.wolf,
            Crossing::WithGoat => s.goat,
            Crossing::WithCabbage => s.cabbage,
        };
        if bank == s.farmer {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

struct SafeResult;
impl Precondition<Crossing> for SafeResult {
    fn check(&self, s: &State, a: &Crossing) -> Verdict {
        let meta = axiom_meta(
            "safe_result",
            "result must be safe — wolf cannot eat goat and goat cannot eat cabbage",
            RIVER_CITATION,
        );
        let next = apply_crossing(s, a).unwrap_or_else(|_| s.clone());
        if next.is_safe() {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

fn apply_crossing(s: &State, a: &Crossing) -> Result<State, Box<dyn Counterexample>> {
    let mut n = s.clone();
    let dest = s.farmer.opposite();
    n.farmer = dest;
    match a {
        Crossing::Alone => {}
        Crossing::WithWolf => n.wolf = dest,
        Crossing::WithGoat => n.goat = dest,
        Crossing::WithCabbage => n.cabbage = dest,
    }
    Ok(n)
}

pub fn new_puzzle() -> Engine<Crossing> {
    Engine::new(
        State::initial(),
        vec![Box::new(ItemWithFarmer), Box::new(SafeResult)],
        apply_crossing,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::engine::EngineError;
    use proptest::prelude::*;

    fn arb_crossing() -> impl Strategy<Value = Crossing> {
        prop_oneof![
            Just(Crossing::Alone),
            Just(Crossing::WithWolf),
            Just(Crossing::WithGoat),
            Just(Crossing::WithCabbage)
        ]
    }

    #[test]
    fn test_known_solution() {
        let e = new_puzzle()
            .next(Crossing::WithGoat)
            .unwrap()
            .next(Crossing::Alone)
            .unwrap()
            .next(Crossing::WithWolf)
            .unwrap()
            .next(Crossing::WithGoat)
            .unwrap()
            .next(Crossing::WithCabbage)
            .unwrap()
            .next(Crossing::Alone)
            .unwrap()
            .next(Crossing::WithGoat)
            .unwrap();
        assert!(e.situation().is_terminal());
    }

    #[test]
    fn test_wolf_eats_goat_blocked() {
        assert!(new_puzzle().next(Crossing::WithCabbage).is_err());
    }

    #[test]
    fn test_cant_take_from_other_bank() {
        let e = new_puzzle()
            .next(Crossing::WithGoat)
            .unwrap()
            .next(Crossing::Alone)
            .unwrap();
        assert!(e.next(Crossing::WithGoat).is_err());
    }

    proptest! {
        #[test]
        fn prop_always_safe(crossings in proptest::collection::vec(arb_crossing(), 0..20)) {
            let mut e = new_puzzle();
            for c in crossings {
                match e.next(c) {
                    Ok(next) => { prop_assert!(next.situation().is_safe()); e = next; }
                    Err(EngineError::Violated { engine: prev, .. }) => { e = prev; }
                    Err(e) => panic!("{e:?}")
                }
            }
        }
    }
}
