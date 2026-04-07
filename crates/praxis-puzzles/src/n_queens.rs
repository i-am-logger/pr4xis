use praxis_engine::{Action, Engine, Precondition, PreconditionResult, Situation};

/// N-Queens: place N queens on NxN board with no attacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub n: usize,
    /// queens[row] = column (placed row by row from 0)
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
}

impl Situation for State {
    fn describe(&self) -> String {
        format!(
            "{}x{} queens={:?} placed={}/{}",
            self.n,
            self.n,
            self.queens,
            self.queens.len(),
            self.n
        )
    }
    fn is_terminal(&self) -> bool {
        self.queens.len() == self.n
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceQueen {
    pub col: usize,
}

impl Action for PlaceQueen {
    type Sit = State;
    fn describe(&self) -> String {
        format!("place queen at col {}", self.col)
    }
}

struct NoAttack;
impl Precondition<PlaceQueen> for NoAttack {
    fn check(&self, s: &State, a: &PlaceQueen) -> PreconditionResult {
        if a.col >= s.n {
            return PreconditionResult::violated(
                "no_attack",
                "column out of range",
                &s.describe(),
                &a.describe(),
            );
        }
        if s.queens.len() >= s.n {
            return PreconditionResult::violated(
                "no_attack",
                "board is full",
                &s.describe(),
                &a.describe(),
            );
        }
        let row = s.queens.len();
        if s.attacks(row, a.col) {
            PreconditionResult::violated(
                "no_attack",
                &format!("queen at ({},{}) attacks existing queen", row, a.col),
                &s.describe(),
                &a.describe(),
            )
        } else {
            PreconditionResult::satisfied("no_attack", &format!("({},{}) is safe", row, a.col))
        }
    }
    fn describe(&self) -> &str {
        "queen must not attack any existing queen"
    }
}

fn apply_queen(s: &State, a: &PlaceQueen) -> State {
    let mut n = s.clone();
    n.queens.push(a.col);
    n
}

pub fn new_puzzle(n: usize) -> Engine<PlaceQueen> {
    Engine::new(State::new(n), vec![Box::new(NoAttack)], apply_queen)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(e.is_terminal());
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
        assert!(e.is_terminal());
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
                    Err((prev, _)) => { e = prev; }
                }
            }
        }
    }
}
