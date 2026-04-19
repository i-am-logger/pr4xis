use super::arrow::Arrow;
use super::category::Category;
use super::entity::Concept;
use crate::logic::Axiom;
use crate::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};

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
    fn verify(&self) -> Verdict {
        if C::Object::variants()
            .iter()
            .all(|obj| !C::morphisms_from(obj).is_empty())
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
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
    fn verify(&self) -> Verdict {
        use std::collections::{HashSet, VecDeque};

        let variants = C::Object::variants();
        let connected = if variants.is_empty() {
            true
        } else {
            let morphisms = C::morphisms();
            variants.iter().all(|start| {
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
                visited.len() == variants.len()
            })
        };
        if connected {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    crate::axiom_meta!(
        "FullyConnected",
        "every object is reachable from every other object",
        "Graph connectivity invariant on a category"
    );
}
