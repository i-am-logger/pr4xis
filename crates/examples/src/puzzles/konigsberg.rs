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

const EULER_CITATION: &str = "Euler (1736) Solutio problematis ad geometriam situs pertinentis, \
     Commentarii Academiae Scientiarum Petropolitanae 8:128-140";

/// Bridges of Königsberg: traverse all edges exactly once.
/// Classic result: impossible if more than 2 nodes have odd degree.
///
/// Source: Euler (1736) — the founding paper of graph theory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    pub nodes: usize,
    pub edges: Vec<(usize, usize)>, // undirected
}

impl Graph {
    /// Classic Königsberg: 4 landmasses (A,B,C,D), 7 bridges
    /// Degrees: A=3, B=5, C=3, D=3 — all odd → no Euler path
    pub fn konigsberg() -> Self {
        Self {
            nodes: 4, // A=0, B=1, C=2, D=3
            edges: vec![
                (0, 1), // bridge A-B (1)
                (0, 2), // bridge A-C (1)
                (0, 3), // bridge A-D (1)  → A degree = 3
                (1, 2),
                (1, 2), // 2 bridges B-C
                (1, 3),
                (1, 3), // 2 bridges B-D   → B degree = 5
                        // C degree: A-C + B-C×2 = 3
                        // D degree: A-D + B-D×2 = 3
            ],
        }
    }

    /// Simple graph that HAS an Euler circuit (all even degree)
    pub fn eulerian_circuit() -> Self {
        // Triangle: each node has degree 2 (even)
        Self {
            nodes: 3,
            edges: vec![(0, 1), (1, 2), (2, 0)],
        }
    }

    pub fn degree(&self, node: usize) -> usize {
        self.edges
            .iter()
            .filter(|(a, b)| *a == node || *b == node)
            .count()
    }

    pub fn odd_degree_count(&self) -> usize {
        (0..self.nodes)
            .filter(|&n| !self.degree(n).is_multiple_of(2))
            .count()
    }

    /// Euler path exists iff 0 or 2 nodes have odd degree.
    pub fn has_euler_path(&self) -> bool {
        let odd = self.odd_degree_count();
        odd == 0 || odd == 2
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub graph: Graph,
    pub position: usize,
    pub traversed: HashSet<usize>, // indices of traversed edges
}

impl State {
    pub fn new(graph: Graph, start: usize) -> Self {
        Self {
            graph,
            position: start,
            traversed: HashSet::new(),
        }
    }

    pub fn all_traversed(&self) -> bool {
        self.traversed.len() == self.graph.edges.len()
    }

    /// The Euler-path search succeeds when every edge has been crossed.
    pub fn is_terminal(&self) -> bool {
        self.all_traversed()
    }

    pub fn available_edges(&self) -> Vec<(usize, usize, usize)> {
        self.graph
            .edges
            .iter()
            .enumerate()
            .filter(|(i, (a, b))| {
                !self.traversed.contains(i) && (*a == self.position || *b == self.position)
            })
            .map(|(i, (a, b))| (i, *a, *b))
            .collect()
    }
}

impl Situation for State {}

/// Cross a bridge (by edge index).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrossBridge {
    pub edge_index: usize,
}

impl Action for CrossBridge {
    type Sit = State;
}

struct ValidCrossing;
impl Precondition<CrossBridge> for ValidCrossing {
    fn check(&self, s: &State, a: &CrossBridge) -> Verdict {
        let meta = axiom_meta(
            "valid_crossing",
            "must cross adjacent, untraversed bridge",
            EULER_CITATION,
        );
        if a.edge_index >= s.graph.edges.len() {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        if s.traversed.contains(&a.edge_index) {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        let (a_node, b_node) = s.graph.edges[a.edge_index];
        if a_node != s.position && b_node != s.position {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

fn apply_crossing(s: &State, a: &CrossBridge) -> Result<State, Box<dyn Counterexample>> {
    let mut n = s.clone();
    let (a_node, b_node) = s.graph.edges[a.edge_index];
    n.position = if a_node == s.position { b_node } else { a_node };
    n.traversed.insert(a.edge_index);
    Ok(n)
}

pub fn new_konigsberg(start: usize) -> Engine<CrossBridge> {
    Engine::new(
        State::new(Graph::konigsberg(), start),
        vec![Box::new(ValidCrossing)],
        apply_crossing,
    )
}

pub fn new_graph(graph: Graph, start: usize) -> Engine<CrossBridge> {
    Engine::new(
        State::new(graph, start),
        vec![Box::new(ValidCrossing)],
        apply_crossing,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::engine::EngineError;
    use proptest::prelude::*;

    #[test]
    fn test_konigsberg_impossible() {
        let g = Graph::konigsberg();
        assert!(!g.has_euler_path()); // 4 odd-degree nodes
        assert_eq!(g.odd_degree_count(), 4);
    }

    #[test]
    fn test_eulerian_graph_possible() {
        let g = Graph::eulerian_circuit();
        assert!(g.has_euler_path());
    }

    #[test]
    fn test_cant_reuse_bridge() {
        let e = new_konigsberg(0)
            .next(CrossBridge { edge_index: 0 })
            .unwrap();
        assert!(e.next(CrossBridge { edge_index: 0 }).is_err());
    }

    #[test]
    fn test_must_be_adjacent() {
        // Start at A(0). Edges 3,4 are B-C — not adjacent to A
        let e = new_konigsberg(0);
        assert!(e.next(CrossBridge { edge_index: 3 }).is_err());
    }

    #[test]
    fn test_konigsberg_gets_stuck() {
        let mut e = new_konigsberg(0);
        let mut count = 0;
        for i in 0..7 {
            match e.next(CrossBridge { edge_index: i }) {
                Ok(next) => {
                    count += 1;
                    e = next;
                }
                Err(EngineError::Violated { engine: prev, .. }) => {
                    e = prev;
                }
                Err(e) => panic!("{e:?}"),
            }
        }
        assert!(count < 7);
    }

    #[test]
    fn test_konigsberg_exhaustive_impossibility() {
        // Prove that NO starting node and NO sequence of moves can traverse all 7 bridges
        fn try_all(state: &State, depth: usize, max_depth: usize) -> bool {
            if state.all_traversed() {
                return true;
            } // found a solution!
            if depth >= max_depth {
                return false;
            }
            for (idx, _, _) in state.available_edges() {
                let mut next = state.clone();
                let (a, b) = next.graph.edges[idx];
                next.position = if a == next.position { b } else { a };
                next.traversed.insert(idx);
                if try_all(&next, depth + 1, max_depth) {
                    return true;
                }
            }
            false
        }

        let g = Graph::konigsberg();
        for start in 0..g.nodes {
            let state = State::new(g.clone(), start);
            assert!(
                !try_all(&state, 0, g.edges.len()),
                "Euler path should be impossible from node {}",
                start
            );
        }
    }

    proptest! {
        /// Can never traverse the same bridge twice
        #[test]
        fn prop_no_reuse(edges in proptest::collection::vec(0..7usize, 0..10)) {
            let mut e = new_konigsberg(0);
            let mut used = HashSet::new();
            for edge in edges {
                match e.next(CrossBridge { edge_index: edge }) {
                    Ok(next) => {
                        prop_assert!(!used.contains(&edge));
                        used.insert(edge);
                        e = next;
                    }
                    Err(EngineError::Violated { engine: prev, .. }) => { e = prev; }
                    Err(e) => panic!("{e:?}")
                }
            }
        }

        /// Traversed count matches successful crossings
        #[test]
        fn prop_traversed_count(edges in proptest::collection::vec(0..7usize, 0..10)) {
            let mut e = new_konigsberg(0);
            let mut successes = 0;
            for edge in edges {
                match e.next(CrossBridge { edge_index: edge }) {
                    Ok(next) => { successes += 1; e = next; }
                    Err(EngineError::Violated { engine: prev, .. }) => { e = prev; }
                    Err(e) => panic!("{e:?}")
                }
            }
            prop_assert_eq!(e.situation().traversed.len(), successes);
        }
    }
}
