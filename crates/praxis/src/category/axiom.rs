use super::category::Category;
use super::entity::Entity;
use super::relationship::Relationship;
use crate::logic::Axiom;

/// Every object has at least one outgoing morphism (no dead states).
pub struct NoDeadStates<C: Category> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Category> NoDeadStates<C> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: Category> Default for NoDeadStates<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Category> Axiom for NoDeadStates<C> {
    fn description(&self) -> &str {
        "every object has at least one outgoing morphism"
    }

    fn holds(&self) -> bool {
        C::Object::variants()
            .iter()
            .all(|obj| !C::morphisms_from(obj).is_empty())
    }
}

/// Every object is reachable from every other object.
pub struct FullyConnected<C: Category> {
    _marker: std::marker::PhantomData<C>,
}

impl<C: Category> FullyConnected<C> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: Category> Default for FullyConnected<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Category> Axiom for FullyConnected<C> {
    fn description(&self) -> &str {
        "every object is reachable from every other object"
    }

    fn holds(&self) -> bool {
        use std::collections::{HashSet, VecDeque};

        let variants = C::Object::variants();
        if variants.is_empty() {
            return true;
        }

        let morphisms = C::morphisms();

        for start in &variants {
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            visited.insert(start.clone());
            queue.push_back(start.clone());

            while let Some(current) = queue.pop_front() {
                for m in &morphisms {
                    if m.source() == current && visited.insert(m.target()) {
                        queue.push_back(m.target());
                    }
                }
            }

            if visited.len() != variants.len() {
                return false;
            }
        }

        true
    }
}
