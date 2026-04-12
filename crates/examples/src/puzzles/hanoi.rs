use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

/// Tower of Hanoi: move N disks from peg A to peg C.
/// Rule: never place a larger disk on a smaller one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// pegs[0..3], each is a stack of disk sizes (bottom to top, decreasing)
    pub pegs: [Vec<u8>; 3],
    pub num_disks: u8,
}

impl State {
    pub fn new(num_disks: u8) -> Self {
        Self {
            pegs: [(1..=num_disks).rev().collect(), vec![], vec![]],
            num_disks,
        }
    }
}

impl Situation for State {
    fn describe(&self) -> String {
        format!(
            "A:{:?} B:{:?} C:{:?}",
            self.pegs[0], self.pegs[1], self.pegs[2]
        )
    }

    fn is_terminal(&self) -> bool {
        self.pegs[0].is_empty()
            && self.pegs[1].is_empty()
            && self.pegs[2].len() == self.num_disks as usize
    }
}

/// Move top disk from one peg to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: usize,
    pub to: usize,
}

impl Move {
    pub fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
}

impl Action for Move {
    type Sit = State;
    fn describe(&self) -> String {
        let names = ['A', 'B', 'C'];
        format!("{} → {}", names[self.from], names[self.to])
    }
}

struct ValidMove;
impl Precondition<Move> for ValidMove {
    fn check(&self, state: &State, action: &Move) -> PreconditionResult {
        if action.from >= 3 || action.to >= 3 {
            return PreconditionResult::violated(
                "valid_move",
                "peg index out of range",
                &state.describe(),
                &action.describe(),
            );
        }
        if action.from == action.to {
            return PreconditionResult::violated(
                "valid_move",
                "same peg",
                &state.describe(),
                &action.describe(),
            );
        }
        let source = &state.pegs[action.from];
        if source.is_empty() {
            return PreconditionResult::violated(
                "valid_move",
                "source peg is empty",
                &state.describe(),
                &action.describe(),
            );
        }
        let disk = *source.last().unwrap();
        let target = &state.pegs[action.to];
        if let Some(&top) = target.last()
            && disk > top
        {
            return PreconditionResult::violated(
                "valid_move",
                &format!("disk {} cannot go on top of disk {}", disk, top),
                &state.describe(),
                &action.describe(),
            );
        }
        PreconditionResult::satisfied("valid_move", &format!("disk {} → peg", disk))
    }
    fn describe(&self) -> &str {
        "cannot place larger disk on smaller disk"
    }
}

fn apply_hanoi(state: &State, action: &Move) -> Result<State, String> {
    let mut next = state.clone();
    let disk = next.pegs[action.from]
        .pop()
        .ok_or_else(|| "source peg is empty".to_string())?;
    next.pegs[action.to].push(disk);
    Ok(next)
}

pub fn new_puzzle(num_disks: u8) -> Engine<Move> {
    Engine::new(
        State::new(num_disks),
        vec![Box::new(ValidMove)],
        apply_hanoi,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::engine::EngineError;
    use proptest::prelude::*;

    #[test]
    fn test_3_disk_solution() {
        let e = new_puzzle(3)
            .next(Move::new(0, 2))
            .unwrap()
            .next(Move::new(0, 1))
            .unwrap()
            .next(Move::new(2, 1))
            .unwrap()
            .next(Move::new(0, 2))
            .unwrap()
            .next(Move::new(1, 0))
            .unwrap()
            .next(Move::new(1, 2))
            .unwrap()
            .next(Move::new(0, 2))
            .unwrap();
        assert!(e.is_terminal());
        assert_eq!(e.trace().successful_steps(), 7); // 2^3 - 1
    }

    #[test]
    fn test_cant_place_larger_on_smaller() {
        let e = new_puzzle(3).next(Move::new(0, 2)).unwrap(); // move disk 1 to C
        // Try to move disk 2 onto disk 1
        assert!(e.next(Move::new(0, 2)).is_err());
    }

    #[test]
    fn test_cant_move_from_empty() {
        assert!(new_puzzle(3).next(Move::new(1, 2)).is_err());
    }

    proptest! {
        /// Pegs always contain valid decreasing sequences
        #[test]
        fn prop_pegs_always_sorted(moves in proptest::collection::vec((0..3usize, 0..3usize), 0..30)) {
            let mut e = new_puzzle(4);
            for (from, to) in moves {
                match e.next(Move::new(from, to)) {
                    Ok(next) => {
                        for peg in &next.situation().pegs {
                            for w in peg.windows(2) {
                                prop_assert!(w[0] > w[1], "peg not sorted: {:?}", peg);
                            }
                        }
                        e = next;
                    }
                    Err(EngineError::Violated { engine: prev, .. }) => { e = prev; }
                    Err(_) => unreachable!()
                }
            }
        }

        /// Total disk count is always preserved
        #[test]
        fn prop_disk_count_preserved(moves in proptest::collection::vec((0..3usize, 0..3usize), 0..30)) {
            let mut e = new_puzzle(4);
            let total = 4;
            for (from, to) in moves {
                match e.next(Move::new(from, to)) {
                    Ok(next) => {
                        let count: usize = next.situation().pegs.iter().map(|p| p.len()).sum();
                        prop_assert_eq!(count, total);
                        e = next;
                    }
                    Err(EngineError::Violated { engine: prev, .. }) => { e = prev; }
                    Err(_) => unreachable!()
                }
            }
        }
    }
}
