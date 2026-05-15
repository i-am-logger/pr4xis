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

const N_QUEENS_CITATION: &str =
    "Bezzel (1848) Proposal of the Eight Queens Problem, Schachzeitung 3:363";

/// N-Queens: place N queens on NxN board with no attacks.
///
/// Source: Bezzel (1848) — earliest published statement of the
/// eight-queens problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub n: usize,
    /// `queens[row]` = column (placed row by row from 0)
    pub queens: Vec<usize>,
}

impl State {
    pub fn new(n: usize) -> Self {
        Self { n, queens: vec![] }
    }

    pub fn attacks(&self, row: usize, col: usize) -> bool {
        for (r, &c) in self.queens.iter().enumerate() {
            if c == col {
                return true;
            }
            if (r as i32 - row as i32).unsigned_abs() as usize
                == (c as i32 - col as i32).unsigned_abs() as usize
            {
                return true;
            }
        }
        false
    }

    /// Board is complete when N queens have been placed.
    pub fn is_terminal(&self) -> bool {
        self.queens.len() == self.n
    }
}

impl Situation for State {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceQueen {
    pub col: usize,
}

impl Action for PlaceQueen {
    type Sit = State;
}

struct NoAttack;
impl Precondition<PlaceQueen> for NoAttack {
    fn check(&self, s: &State, a: &PlaceQueen) -> Verdict {
        let meta = axiom_meta(
            "no_attack",
            "queen must not attack any existing queen",
            N_QUEENS_CITATION,
        );
        if a.col >= s.n {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        if s.queens.len() >= s.n {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        let row = s.queens.len();
        if s.attacks(row, a.col) {
            Err(Box::new(SimpleCounterexample::new(meta)))
        } else {
            Ok(Box::new(SimpleProof::new(meta)))
        }
    }
}

fn apply_queen(s: &State, a: &PlaceQueen) -> Result<State, Box<dyn Counterexample>> {
    let mut n = s.clone();
    n.queens.push(a.col);
    Ok(n)
}

pub fn new_puzzle(n: usize) -> Engine<PlaceQueen> {
    Engine::new(State::new(n), vec![Box::new(NoAttack)], apply_queen)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::engine::EngineError;
    use proptest::prelude::*;

    #[test]
    fn test_4_queens_solution() {
        let e = new_puzzle(4)
            .next(PlaceQueen { col: 1 })
            .unwrap()
            .next(PlaceQueen { col: 3 })
            .unwrap()
            .next(PlaceQueen { col: 0 })
            .unwrap()
            .next(PlaceQueen { col: 2 })
            .unwrap();
        assert!(e.situation().is_terminal());
    }

    #[test]
    fn test_8_queens_solution() {
        let e = new_puzzle(8)
            .next(PlaceQueen { col: 0 })
            .unwrap()
            .next(PlaceQueen { col: 4 })
            .unwrap()
            .next(PlaceQueen { col: 7 })
            .unwrap()
            .next(PlaceQueen { col: 5 })
            .unwrap()
            .next(PlaceQueen { col: 2 })
            .unwrap()
            .next(PlaceQueen { col: 6 })
            .unwrap()
            .next(PlaceQueen { col: 1 })
            .unwrap()
            .next(PlaceQueen { col: 3 })
            .unwrap();
        assert!(e.situation().is_terminal());
    }

    #[test]
    fn test_same_column_blocked() {
        let e = new_puzzle(4).next(PlaceQueen { col: 0 }).unwrap();
        assert!(e.next(PlaceQueen { col: 0 }).is_err());
    }

    #[test]
    fn test_diagonal_blocked() {
        let e = new_puzzle(4).next(PlaceQueen { col: 0 }).unwrap();
        assert!(e.next(PlaceQueen { col: 1 }).is_err()); // diagonal
    }

    proptest! {
        #[test]
        fn prop_no_attacks_ever(cols in proptest::collection::vec(0..8usize, 0..8)) {
            let mut e = new_puzzle(8);
            for col in cols {
                match e.next(PlaceQueen { col }) {
                    Ok(next) => {
                        let queens = &next.situation().queens;
                        for (i, &ci) in queens.iter().enumerate() {
                            for (j, &cj) in queens.iter().enumerate() {
                                if i != j {
                                    prop_assert_ne!(ci, cj, "same column");
                                    prop_assert_ne!((i as i32 - j as i32).unsigned_abs(), (ci as i32 - cj as i32).unsigned_abs(), "diagonal");
                                }
                            }
                        }
                        e = next;
                    }
                    Err(EngineError::Violated { engine: prev, .. }) => { e = prev; }
                    Err(e) => panic!("{e:?}")
                }
            }
        }
    }
}
