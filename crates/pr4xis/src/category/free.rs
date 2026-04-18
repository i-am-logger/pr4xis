#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::fmt::Debug;

// Free monad — builds a computation DSL from a functor.
//
// Free<F, A> is either:
//   Pure(A) — a completed computation
//   Free(F<Free<F, A>>) — a suspended computation with more to do
//
// The free monad gives you monadic sequencing "for free" from any functor.
// You describe WHAT to compute (the DSL), then interpret it separately.
//
// In pr4xis, the free monad formalizes:
//   - define_ontology! — the macro IS an interpreter of a free monad DSL
//     (the user declares operations, the macro interprets them into code)
//   - Pipeline steps — each step is a "command" that gets interpreted
//   - Deferred computation — build up a description, run it later
//
// References:
// - Swierstra, "Data types à la carte" (2008, JFP)
//   https://doi.org/10.1017/S0956796808006758
// - Kiselyov & Ishii, "Freer Monads, More Extensible Effects" (2015, Haskell)
//   https://doi.org/10.1145/2887747.2804319
// - Kmett, "Free Monads for Less" (2012, Comonad.Reader blog)
// - Mac Lane, "Categories for the Working Mathematician" (1971), Ch. IV
//   — free constructions

/// A free monad over a command type F.
///
/// Builds a computation as a tree of commands that can be interpreted
/// by different interpreters (pure, IO, traced, etc.).
#[derive(Debug, Clone)]
pub enum Free<F: Clone + Debug, A: Clone + Debug> {
    /// A completed computation with a final value.
    Pure(A),
    /// A suspended computation: a command F with a continuation.
    Suspend(F, Box<dyn CloneFn<F, A>>),
}

/// Trait for clonable continuations (workaround for Box<dyn FnOnce>).
pub trait CloneFn<F: Clone + Debug, A: Clone + Debug>: Debug {
    fn call(&self, f: F) -> Free<F, A>;
    fn clone_box(&self) -> Box<dyn CloneFn<F, A>>;
}

impl<F: Clone + Debug, A: Clone + Debug> Clone for Box<dyn CloneFn<F, A>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<F: Clone + Debug + 'static, A: Clone + Debug + 'static> Free<F, A> {
    /// Lift a pure value into the free monad.
    pub fn pure(a: A) -> Self {
        Free::Pure(a)
    }

    /// Lift a command into the free monad (suspend it).
    pub fn lift(cmd: F) -> Free<F, F> {
        #[derive(Debug, Clone)]
        struct IdCont<F: Clone + Debug>(core::marker::PhantomData<F>);
        impl<F: Clone + Debug + 'static> CloneFn<F, F> for IdCont<F> {
            fn call(&self, f: F) -> Free<F, F> {
                Free::Pure(f)
            }
            fn clone_box(&self) -> Box<dyn CloneFn<F, F>> {
                Box::new(self.clone())
            }
        }
        Free::Suspend(cmd, Box::new(IdCont(core::marker::PhantomData)))
    }

    /// Run a pure free monad (no real effects, just unwrap).
    pub fn run(self) -> A
    where
        F: Into<A>,
    {
        match self {
            Free::Pure(a) => a,
            Free::Suspend(cmd, k) => k.call(cmd).run(),
        }
    }
}

// Simplified free monad for practical use: just a chain of operations.

/// A computation chain — simplified free monad for pr4xis.
///
/// Each step produces a value that feeds into the next step.
/// The chain can be inspected, traced, or interpreted differently.
#[derive(Debug, Clone)]
pub struct Chain<A: Clone + Debug> {
    steps: Vec<String>,
    value: A,
}

impl<A: Clone + Debug> Chain<A> {
    /// Start a chain with an initial value.
    pub fn start(value: A) -> Self {
        Self {
            steps: Vec::new(),
            value,
        }
    }

    /// Add a step to the chain.
    pub fn then<B: Clone + Debug>(self, name: &str, f: impl FnOnce(A) -> B) -> Chain<B> {
        let mut steps = self.steps;
        steps.push(name.to_string());
        Chain {
            steps,
            value: f(self.value),
        }
    }

    /// Get the final value.
    pub fn value(&self) -> &A {
        &self.value
    }

    /// Get the step names (for tracing).
    pub fn steps(&self) -> &[String] {
        &self.steps
    }

    /// Run and extract value + trace.
    pub fn run(self) -> (A, Vec<String>) {
        (self.value, self.steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_accumulates_steps() {
        let (result, steps) = Chain::start(10)
            .then("double", |x| x * 2)
            .then("add_one", |x| x + 1)
            .then("to_string", |x| format!("{x}"))
            .run();

        assert_eq!(result, "21");
        assert_eq!(steps, vec!["double", "add_one", "to_string"]);
    }

    #[test]
    fn chain_empty() {
        let chain = Chain::start(42);
        assert_eq!(*chain.value(), 42);
        assert!(chain.steps().is_empty());
    }

    #[test]
    fn chain_single_step() {
        let (val, steps) = Chain::start("hello").then("length", |s| s.len()).run();
        assert_eq!(val, 5);
        assert_eq!(steps, vec!["length"]);
    }

    // --- Pipeline as free monad ---

    #[test]
    fn pipeline_as_chain() {
        // Simulate the chat pipeline as a computation chain
        let (response, trace) = Chain::start("is a dog an animal")
            .then("tokenize", |input| input.split_whitespace().count())
            .then("parse", |token_count| token_count > 0)
            .then(
                "interpret",
                |parsed| {
                    if parsed { "question" } else { "unknown" }
                },
            )
            .then("respond", |intent| format!("Understood: {intent}"))
            .run();

        assert_eq!(response, "Understood: question");
        assert_eq!(trace, vec!["tokenize", "parse", "interpret", "respond"]);
    }

    mod prop {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Chain preserves step count
            #[test]
            fn prop_chain_step_count(n in 0..10usize) {
                let mut chain = Chain::start(0);
                for i in 0..n {
                    chain = chain.then(&format!("step{i}"), |x| x + 1);
                }
                prop_assert_eq!(chain.steps().len(), n);
                prop_assert_eq!(*chain.value(), n as i32);
            }
        }
    }
}
