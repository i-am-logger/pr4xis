use super::arrow::Arrow;
use super::category::Category;
use super::entity::FinitelyGenerated;
use super::terminal::TerminalTarget;
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

impl<C: Category> Axiom for NoDeadStates<C>
where
    // Enumerates every object to check each has an outgoing morphism — a
    // closed-world (finite) check.
    C::Object: FinitelyGenerated,
{
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

impl<C: Category> Axiom for FullyConnected<C>
where
    // Enumerates every object to seed the reachability BFS — a closed-world
    // (finite) check.
    C::Object: FinitelyGenerated,
{
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

/// A designated object `T` is **terminal**: every object has exactly one
/// morphism to `T`.
///
/// In a thin category this is "everything reaches a single sink." For an
/// interaction statechart whose objects are modes, `T` = the root mode and this
/// axiom is the **no-stuck guarantee**: from every mode there is exactly one way
/// back to root, so no input can strand the user (a terminal object is the limit
/// of the empty diagram — its defining universal property is the right adjoint to
/// the unique functor `! : C → 1`).
///
/// Literature:
/// - Awodey (2010) *Category Theory* Ch. 2 — initial and terminal objects.
/// - Mac Lane (1971) CWM III.4 — terminal object as the limit of the empty
///   diagram; right adjoint to `!`.
pub struct TerminalObject<C, T> {
    _marker: core::marker::PhantomData<(C, T)>,
}

impl<C, T> TerminalObject<C, T> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<C, T> Default for TerminalObject<C, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, T> Axiom for TerminalObject<C, T>
where
    C: Category,
    T: TerminalTarget<Category = C>,
    <C as Category>::Morphism: PartialEq,
    // Enumerates every object to check each has exactly one morphism to the
    // terminal — a closed-world (finite) check.
    C::Object: FinitelyGenerated,
{
    fn verify(&self) -> Verdict {
        let t = T::target();
        let terminal = <C::Object as FinitelyGenerated>::variants()
            .iter()
            .all(|a| {
                C::morphisms_from(a)
                    .into_iter()
                    .filter(|m| m.target() == t)
                    .count()
                    == 1
            });
        if terminal {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    crate::axiom_meta!(
        "TerminalObject",
        "every object has exactly one morphism to the terminal object (no-stuck root)",
        "Awodey (2010) Category Theory Ch. 2; Mac Lane (1971) CWM III.4 — terminal object as limit of the empty diagram"
    );
}
