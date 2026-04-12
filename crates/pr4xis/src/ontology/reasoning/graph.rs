use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Build an adjacency map from directed pairs: source → [targets].
pub(super) fn adjacency_map<E: Clone + Eq + Hash>(pairs: &[(E, E)]) -> HashMap<E, Vec<E>> {
    let mut map: HashMap<E, Vec<E>> = HashMap::new();
    for (from, to) in pairs {
        map.entry(from.clone()).or_default().push(to.clone());
    }
    map
}

/// Build a reverse adjacency map from directed pairs: target → [sources].
pub(super) fn reverse_adjacency_map<E: Clone + Eq + Hash>(pairs: &[(E, E)]) -> HashMap<E, Vec<E>> {
    let mut map: HashMap<E, Vec<E>> = HashMap::new();
    for (from, to) in pairs {
        map.entry(to.clone()).or_default().push(from.clone());
    }
    map
}

/// All nodes transitively reachable from `start` via the adjacency map.
/// Does NOT include `start` itself.
pub(super) fn reachable<E: Clone + Eq + Hash>(start: &E, adj: &HashMap<E, Vec<E>>) -> Vec<E> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    if let Some(neighbors) = adj.get(start) {
        for n in neighbors {
            if visited.insert(n.clone()) {
                queue.push_back(n.clone());
            }
        }
    }
    while let Some(current) = queue.pop_front() {
        result.push(current.clone());
        if let Some(neighbors) = adj.get(&current) {
            for n in neighbors {
                if visited.insert(n.clone()) {
                    queue.push_back(n.clone());
                }
            }
        }
    }
    result
}

/// Check if `start` is reachable from its own neighbors (cycle detection).
pub(super) fn has_cycle<E: Clone + Eq + Hash>(start: &E, adj: &HashMap<E, Vec<E>>) -> bool {
    reachable(start, adj).contains(start)
}
