use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};
use std::collections::HashSet;

/// Knight's Tour: visit every square on NxN board exactly once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub n: usize,
    pub position: (usize, usize),
    pub visited: HashSet<(usize, usize)>,
}

impl State {
    pub fn new(n: usize, start: (usize, usize)) -> Self {
        let mut visited = HashSet::new();
        visited.insert(start);
        Self {
            n,
            position: start,
            visited,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.visited.len() == self.n * self.n
    }

    pub fn knight_moves(&self) -> Vec<(usize, usize)> {
        let (x, y) = self.position;
        let deltas: [(i32, i32); 8] = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];
        deltas
            .iter()
            .filter_map(|&(dx, dy)| {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < self.n && (ny as usize) < self.n {
                    Some((nx as usize, ny as usize))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Situation for State {
    fn describe(&self) -> String {
        format!(
            "{}x{} at ({},{}) visited={}/{}",
            self.n,
            self.n,
            self.position.0,
            self.position.1,
            self.visited.len(),
            self.n * self.n
        )
    }
    fn is_terminal(&self) -> bool {
        self.is_complete()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnightMove {
    pub to: (usize, usize),
}

impl Action for KnightMove {
    type Sit = State;
    fn describe(&self) -> String {
        format!("move to ({},{})", self.to.0, self.to.1)
    }
}

struct ValidKnightMove;
impl Precondition<KnightMove> for ValidKnightMove {
    fn check(&self, s: &State, a: &KnightMove) -> PreconditionResult {
        if !s.knight_moves().contains(&a.to) {
            return PreconditionResult::violated(
                "valid_move",
                "not a valid knight move",
                &s.describe(),
                &a.describe(),
            );
        }
        if s.visited.contains(&a.to) {
            return PreconditionResult::violated(
                "valid_move",
                "square already visited",
                &s.describe(),
                &a.describe(),
            );
        }
        PreconditionResult::satisfied(
            "valid_move",
            &format!("({},{}) is valid and unvisited", a.to.0, a.to.1),
        )
    }
    fn describe(&self) -> &str {
        "must be L-shaped move to unvisited square"
    }
}

fn apply_knight(s: &State, a: &KnightMove) -> Result<State, String> {
    let mut n = s.clone();
    n.position = a.to;
    n.visited.insert(a.to);
    Ok(n)
}

pub fn new_puzzle(n: usize, start: (usize, usize)) -> Engine<KnightMove> {
    Engine::new(
        State::new(n, start),
        vec![Box::new(ValidKnightMove)],
        apply_knight,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_5x5_partial() {
        let e = new_puzzle(5, (0, 0))
            .next(KnightMove { to: (2, 1) })
            .unwrap()
            .next(KnightMove { to: (4, 0) })
            .unwrap();
        assert_eq!(e.situation().visited.len(), 3);
    }

    #[test]
    fn test_revisit_blocked() {
        let e = new_puzzle(5, (0, 0))
            .next(KnightMove { to: (2, 1) })
            .unwrap();
        assert!(e.next(KnightMove { to: (0, 0) }).is_err()); // already visited
    }

    #[test]
    fn test_non_knight_move_blocked() {
        assert!(
            new_puzzle(5, (0, 0))
                .next(KnightMove { to: (1, 0) })
                .is_err()
        );
    }
}
