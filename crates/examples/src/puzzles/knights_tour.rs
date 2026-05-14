use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
use std::collections::HashSet;

fn axiom_meta(name: &'static str, description: &'static str, citation: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

const KNIGHTS_TOUR_CITATION: &str = "Euler (1759) Solution d'une question curieuse qui ne paroît soumise à aucune analyse, \
     Mémoires de l'Académie Royale des Sciences et Belles-Lettres de Berlin 15:310-337";

/// Knight's Tour: visit every square on NxN board exactly once.
///
/// Source: Euler (1759) — earliest systematic treatment of the knight's tour.
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

    /// The tour is terminal when every square has been visited exactly once.
    pub fn is_terminal(&self) -> bool {
        self.is_complete()
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

impl Situation for State {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnightMove {
    pub to: (usize, usize),
}

impl Action for KnightMove {
    type Sit = State;
}

struct ValidKnightMove;
impl Precondition<KnightMove> for ValidKnightMove {
    fn check(&self, s: &State, a: &KnightMove) -> Verdict {
        let meta = axiom_meta(
            "valid_move",
            "must be L-shaped move to unvisited square",
            KNIGHTS_TOUR_CITATION,
        );
        if !s.knight_moves().contains(&a.to) {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        if s.visited.contains(&a.to) {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

fn apply_knight(s: &State, a: &KnightMove) -> Result<State, Box<dyn Counterexample>> {
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
