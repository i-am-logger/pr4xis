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

const MC_CITATION: &str = "Amarel (1968) On Representations of Problems of Reasoning About Actions, \
     Machine Intelligence 3:131-171";

/// Missionaries and cannibals. Boat holds 2.
/// Cannibals can't outnumber missionaries on either bank.
///
/// Source: Amarel (1968) — the canonical AI-planning formulation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub missionaries_left: u8,
    pub cannibals_left: u8,
    pub boat_left: bool, // true = boat on left bank
}

impl State {
    pub fn initial() -> Self {
        Self {
            missionaries_left: 3,
            cannibals_left: 3,
            boat_left: true,
        }
    }

    pub fn missionaries_right(&self) -> u8 {
        3 - self.missionaries_left
    }
    pub fn cannibals_right(&self) -> u8 {
        3 - self.cannibals_left
    }

    pub fn is_safe(&self) -> bool {
        // On left: missionaries >= cannibals (or no missionaries)
        let safe_left =
            self.missionaries_left == 0 || self.missionaries_left >= self.cannibals_left;
        let safe_right =
            self.missionaries_right() == 0 || self.missionaries_right() >= self.cannibals_right();
        safe_left && safe_right
    }

    /// Goal: every missionary and cannibal has crossed to the right bank.
    pub fn is_terminal(&self) -> bool {
        self.missionaries_left == 0 && self.cannibals_left == 0
    }
}

impl Situation for State {}

/// Move: (missionaries, cannibals) in the boat. At least 1, at most 2 total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crossing {
    pub missionaries: u8,
    pub cannibals: u8,
}

impl Crossing {
    pub fn new(m: u8, c: u8) -> Self {
        Self {
            missionaries: m,
            cannibals: c,
        }
    }
    pub fn total(&self) -> u8 {
        self.missionaries + self.cannibals
    }
}

impl Action for Crossing {
    type Sit = State;
}

struct ValidCrossing;
impl Precondition<Crossing> for ValidCrossing {
    fn check(&self, s: &State, a: &Crossing) -> Verdict {
        let meta = axiom_meta(
            "valid_crossing",
            "boat holds 1-2, people must be available",
            MC_CITATION,
        );
        if a.total() == 0 || a.total() > 2 {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        if s.boat_left {
            if a.missionaries > s.missionaries_left || a.cannibals > s.cannibals_left {
                return Err(Box::new(SimpleCounterexample::new(meta)));
            }
        } else if a.missionaries > s.missionaries_right() || a.cannibals > s.cannibals_right() {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

struct SafeResult;
impl Precondition<Crossing> for SafeResult {
    fn check(&self, s: &State, a: &Crossing) -> Verdict {
        let meta = axiom_meta(
            "safe_result",
            "cannibals can't outnumber missionaries on either bank",
            MC_CITATION,
        );
        let next = apply_mc(s, a).unwrap_or_else(|_| s.clone());
        if next.is_safe() {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

fn apply_mc(s: &State, a: &Crossing) -> Result<State, Box<dyn Counterexample>> {
    let mut n = s.clone();
    if s.boat_left {
        n.missionaries_left = n.missionaries_left.saturating_sub(a.missionaries);
        n.cannibals_left = n.cannibals_left.saturating_sub(a.cannibals);
    } else {
        n.missionaries_left = (n.missionaries_left + a.missionaries).min(3);
        n.cannibals_left = (n.cannibals_left + a.cannibals).min(3);
    }
    n.boat_left = !s.boat_left;
    Ok(n)
}

pub fn new_puzzle() -> Engine<Crossing> {
    Engine::new(
        State::initial(),
        vec![Box::new(ValidCrossing), Box::new(SafeResult)],
        apply_mc,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::engine::EngineError;
    use proptest::prelude::*;

    #[test]
    fn test_known_solution() {
        let e = new_puzzle()
            .next(Crossing::new(1, 1))
            .unwrap() // 2M2C | 1M1C
            .next(Crossing::new(1, 0))
            .unwrap() // 3M2C | 0M1C
            .next(Crossing::new(0, 2))
            .unwrap() // 3M0C | 0M3C
            .next(Crossing::new(0, 1))
            .unwrap() // 3M1C | 0M2C
            .next(Crossing::new(2, 0))
            .unwrap() // 1M1C | 2M2C
            .next(Crossing::new(1, 1))
            .unwrap() // 2M2C | 1M1C
            .next(Crossing::new(2, 0))
            .unwrap() // 0M2C | 3M1C
            .next(Crossing::new(0, 1))
            .unwrap() // 0M3C | 3M0C
            .next(Crossing::new(0, 2))
            .unwrap() // 0M1C | 3M2C
            .next(Crossing::new(0, 1))
            .unwrap() // 0M2C | 3M1C
            .next(Crossing::new(0, 2))
            .unwrap(); // 0M0C | 3M3C
        assert!(e.situation().is_terminal());
    }

    #[test]
    fn test_cannibals_outnumber_blocked() {
        // Send 2 missionaries, leaving 1M 3C on left
        assert!(new_puzzle().next(Crossing::new(2, 0)).is_err());
    }

    #[test]
    fn test_boat_too_full() {
        assert!(new_puzzle().next(Crossing::new(2, 1)).is_err());
    }

    #[test]
    fn test_empty_boat() {
        assert!(new_puzzle().next(Crossing::new(0, 0)).is_err());
    }

    proptest! {
        #[test]
        fn prop_always_safe(crossings in proptest::collection::vec((0..3u8, 0..3u8), 0..20)) {
            let mut e = new_puzzle();
            for (m, c) in crossings {
                match e.next(Crossing::new(m, c)) {
                    Ok(next) => { prop_assert!(next.situation().is_safe()); e = next; }
                    Err(EngineError::Violated { engine: prev, .. }) => { e = prev; }
                    Err(e) => panic!("{e:?}")
                }
            }
        }

        #[test]
        fn prop_total_preserved(crossings in proptest::collection::vec((0..3u8, 0..3u8), 0..20)) {
            let mut e = new_puzzle();
            for (m, c) in crossings {
                match e.next(Crossing::new(m, c)) {
                    Ok(next) => {
                        let s = next.situation();
                        prop_assert_eq!(s.missionaries_left + s.missionaries_right(), 3);
                        prop_assert_eq!(s.cannibals_left + s.cannibals_right(), 3);
                        e = next;
                    }
                    Err(EngineError::Violated { engine: prev, .. }) => { e = prev; }
                    Err(e) => panic!("{e:?}")
                }
            }
        }
    }
}
