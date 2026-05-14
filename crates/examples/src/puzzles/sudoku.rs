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

const SUDOKU_CITATION: &str = "Yato & Seta (2003) Complexity and Completeness of Finding Another Solution \
     and Its Application to Puzzles, IEICE Trans. on Fundamentals E86-A(5):1052-1060";

/// Sudoku: 9x9 grid, digits 1-9, no repeats in row/col/box.
///
/// Source: Yato & Seta (2003) — formal complexity treatment of the
/// constraints (NP-completeness of generalised Sudoku).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub grid: [[u8; 9]; 9], // 0 = empty
}

impl State {
    pub fn empty() -> Self {
        Self { grid: [[0; 9]; 9] }
    }

    pub fn from_grid(grid: [[u8; 9]; 9]) -> Self {
        Self { grid }
    }

    pub fn get(&self, row: usize, col: usize) -> u8 {
        self.grid[row][col]
    }

    pub fn is_valid_placement(&self, row: usize, col: usize, val: u8) -> bool {
        if val == 0 || val > 9 {
            return false;
        }
        // Row check
        if self.grid[row].contains(&val) {
            return false;
        }
        // Column check
        if (0..9).any(|r| self.grid[r][col] == val) {
            return false;
        }
        // 3x3 box check
        let (br, bc) = (row / 3 * 3, col / 3 * 3);
        for r in br..br + 3 {
            for c in bc..bc + 3 {
                if self.grid[r][c] == val {
                    return false;
                }
            }
        }
        true
    }

    pub fn empty_cells(&self) -> usize {
        self.grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&v| v == 0)
            .count()
    }

    /// The puzzle is solved when every cell is filled.
    pub fn is_terminal(&self) -> bool {
        self.empty_cells() == 0
    }
}

impl Situation for State {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    pub row: usize,
    pub col: usize,
    pub val: u8,
}

impl Action for Place {
    type Sit = State;
}

struct SudokuRules;
impl Precondition<Place> for SudokuRules {
    fn check(&self, s: &State, a: &Place) -> Verdict {
        let meta = axiom_meta(
            "sudoku",
            "no duplicate digits in row, column, or 3x3 box",
            SUDOKU_CITATION,
        );
        if a.row >= 9 || a.col >= 9 {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        if a.val == 0 || a.val > 9 {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        if s.get(a.row, a.col) != 0 {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        if !s.is_valid_placement(a.row, a.col, a.val) {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

fn apply_sudoku(s: &State, a: &Place) -> Result<State, Box<dyn Counterexample>> {
    let mut n = s.clone();
    n.grid[a.row][a.col] = a.val;
    Ok(n)
}

pub fn new_puzzle(initial: [[u8; 9]; 9]) -> Engine<Place> {
    Engine::new(
        State::from_grid(initial),
        vec![Box::new(SudokuRules)],
        apply_sudoku,
    )
}

pub fn new_empty() -> Engine<Place> {
    Engine::new(State::empty(), vec![Box::new(SudokuRules)], apply_sudoku)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::engine::EngineError;
    use proptest::prelude::*;

    #[test]
    fn test_valid_placement() {
        let e = new_empty()
            .next(Place {
                row: 0,
                col: 0,
                val: 1,
            })
            .unwrap()
            .next(Place {
                row: 0,
                col: 1,
                val: 2,
            })
            .unwrap();
        assert_eq!(e.situation().get(0, 0), 1);
        assert_eq!(e.situation().get(0, 1), 2);
    }

    #[test]
    fn test_same_row_blocked() {
        let e = new_empty()
            .next(Place {
                row: 0,
                col: 0,
                val: 1,
            })
            .unwrap();
        assert!(
            e.next(Place {
                row: 0,
                col: 5,
                val: 1
            })
            .is_err()
        );
    }

    #[test]
    fn test_same_col_blocked() {
        let e = new_empty()
            .next(Place {
                row: 0,
                col: 0,
                val: 1,
            })
            .unwrap();
        assert!(
            e.next(Place {
                row: 5,
                col: 0,
                val: 1
            })
            .is_err()
        );
    }

    #[test]
    fn test_same_box_blocked() {
        let e = new_empty()
            .next(Place {
                row: 0,
                col: 0,
                val: 1,
            })
            .unwrap();
        assert!(
            e.next(Place {
                row: 1,
                col: 1,
                val: 1
            })
            .is_err()
        ); // same 3x3 box
    }

    #[test]
    fn test_cell_already_filled() {
        let e = new_empty()
            .next(Place {
                row: 0,
                col: 0,
                val: 1,
            })
            .unwrap();
        assert!(
            e.next(Place {
                row: 0,
                col: 0,
                val: 2
            })
            .is_err()
        );
    }

    #[test]
    fn test_value_out_of_range() {
        assert!(
            new_empty()
                .next(Place {
                    row: 0,
                    col: 0,
                    val: 0
                })
                .is_err()
        );
        assert!(
            new_empty()
                .next(Place {
                    row: 0,
                    col: 0,
                    val: 10
                })
                .is_err()
        );
    }

    proptest! {
        /// Valid placements never violate sudoku constraints
        #[test]
        fn prop_no_constraint_violations(placements in proptest::collection::vec((0..9usize, 0..9usize, 1..10u8), 0..20)) {
            let mut e = new_empty();
            for (row, col, val) in placements {
                match e.next(Place { row, col, val }) {
                    Ok(next) => {
                        // Verify no row duplicates
                        for r in 0..9 {
                            let vals: Vec<u8> = (0..9).map(|c| next.situation().get(r, c)).filter(|&v| v != 0).collect();
                            let unique: std::collections::HashSet<u8> = vals.iter().copied().collect();
                            prop_assert_eq!(vals.len(), unique.len(), "row {} has duplicates", r);
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
