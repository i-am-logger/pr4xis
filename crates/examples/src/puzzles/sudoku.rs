use praxis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

/// Sudoku: 9x9 grid, digits 1-9, no repeats in row/col/box.
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
}

impl Situation for State {
    fn describe(&self) -> String {
        format!("sudoku empty={}", self.empty_cells())
    }
    fn is_terminal(&self) -> bool {
        self.empty_cells() == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    pub row: usize,
    pub col: usize,
    pub val: u8,
}

impl Action for Place {
    type Sit = State;
    fn describe(&self) -> String {
        format!("place {} at ({},{})", self.val, self.row, self.col)
    }
}

struct SudokuRules;
impl Precondition<Place> for SudokuRules {
    fn check(&self, s: &State, a: &Place) -> PreconditionResult {
        if a.row >= 9 || a.col >= 9 {
            return PreconditionResult::violated(
                "sudoku",
                "out of range",
                &s.describe(),
                &a.describe(),
            );
        }
        if a.val == 0 || a.val > 9 {
            return PreconditionResult::violated(
                "sudoku",
                "value must be 1-9",
                &s.describe(),
                &a.describe(),
            );
        }
        if s.get(a.row, a.col) != 0 {
            return PreconditionResult::violated(
                "sudoku",
                "cell already filled",
                &s.describe(),
                &a.describe(),
            );
        }
        if !s.is_valid_placement(a.row, a.col, a.val) {
            // Find which constraint is violated
            if s.grid[a.row].contains(&a.val) {
                return PreconditionResult::violated(
                    "sudoku",
                    &format!("{} already in row {}", a.val, a.row),
                    &s.describe(),
                    &a.describe(),
                );
            }
            if (0..9).any(|r| s.grid[r][a.col] == a.val) {
                return PreconditionResult::violated(
                    "sudoku",
                    &format!("{} already in col {}", a.val, a.col),
                    &s.describe(),
                    &a.describe(),
                );
            }
            return PreconditionResult::violated(
                "sudoku",
                &format!("{} already in 3x3 box", a.val),
                &s.describe(),
                &a.describe(),
            );
        }
        PreconditionResult::satisfied(
            "sudoku",
            &format!("{} at ({},{}) is valid", a.val, a.row, a.col),
        )
    }
    fn describe(&self) -> &str {
        "no duplicate digits in row, column, or 3x3 box"
    }
}

fn apply_sudoku(s: &State, a: &Place) -> State {
    let mut n = s.clone();
    n.grid[a.row][a.col] = a.val;
    n
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
                    Err((prev, _)) => { e = prev; }
                }
            }
        }
    }
}
