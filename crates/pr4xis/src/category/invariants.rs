use super::category::Category;
use super::entity::Concept;
use super::relationship::Relationship;
use crate::logic::Axiom;
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Every object has at least one outgoing morphism (no dead states).
pub struct NoDeadStates<C: Category> {
    _marker: core::marker::PhantomData<C>,
}

impl<C: Category> NoDeadStates<C> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
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

    crate::axiom_meta!(
        "NoDeadStates",
        "every object has at least one outgoing morphism",
        "Mac Lane (1971) 'Categories for the Working Mathematician' Ch. I"
    );
}

/// Every object is reachable from every other object.
pub struct FullyConnected<C: Category> {
    _marker: core::marker::PhantomData<C>,
}

impl<C: Category> FullyConnected<C> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
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
        use alloc::collections::VecDeque;
        use hashbrown::HashSet;

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

    crate::axiom_meta!(
        "FullyConnected",
        "every object is reachable from every other object",
        "Graph connectivity invariant on a category"
    );
}
