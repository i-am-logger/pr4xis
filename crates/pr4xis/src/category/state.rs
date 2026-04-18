#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::fmt::Debug;

// State monad — computation that reads and modifies state.
//
// State<S, A> represents a computation that takes an initial state S,
// produces a value A, and returns a new state S. State is threaded
// through sequenced computations automatically.
//
// In pr4xis, the State monad formalizes:
//   - Engine: Situation → (Action, NewSituation) — the game/system engine
//   - Pipeline: PipelineState → (Result, NewPipelineState) — chat processing
//   - Transition: SystemState → (Event, NewState) — any state machine
//
// The three monad laws:
//   1. Left identity:  bind(pure(a), f) = f(a)
//   2. Right identity: bind(m, pure) = m
//   3. Associativity:  bind(bind(m, f), g) = bind(m, |x| bind(f(x), g))
//
// References:
// - Moggi, "Notions of Computation and Monads" (1991, Inf. & Comp.)
//   https://doi.org/10.1016/0890-5401(91)90052-4
// - Wadler, "Monads for Functional Programming" (1995)
// - Peyton Jones & Wadler, "Imperative Functional Programming" (1993, POPL)
//   https://doi.org/10.1145/158511.158524
// - Moggi, "Computational lambda-calculus and monads" (1989, LICS)
//   https://doi.org/10.1109/LICS.1989.39155

/// A state monad: computation that threads state through sequenced operations.
///
/// `State<S, A>` = `S → (A, S)`. Each step reads state, produces a value,
/// and returns modified state.
///
/// ```
/// use pr4xis::category::state::State;
///
/// let increment = State::new(|s: i32| (s, s + 1));
/// let (val, new_state) = increment.run(0);
/// assert_eq!(val, 0);  // value is the old state
/// assert_eq!(new_state, 1);  // state incremented
/// ```
pub struct State<S, A> {
    f: Box<dyn FnOnce(S) -> (A, S)>,
}

impl<S: 'static, A: 'static> State<S, A> {
    /// Create a state computation from a function.
    pub fn new(f: impl FnOnce(S) -> (A, S) + 'static) -> Self {
        Self { f: Box::new(f) }
    }

    /// Lift a pure value (state passes through unchanged).
    pub fn pure(a: A) -> Self {
        Self {
            f: Box::new(move |s| (a, s)),
        }
    }

    /// Run the state computation with an initial state.
    pub fn run(self, initial: S) -> (A, S) {
        (self.f)(initial)
    }

    /// Functor map: transform the output value, state unchanged.
    pub fn map<B: 'static>(self, g: impl FnOnce(A) -> B + 'static) -> State<S, B> {
        State::new(move |s| {
            let (a, s2) = (self.f)(s);
            (g(a), s2)
        })
    }

    /// Monadic bind: sequence with a function that returns a new State.
    /// bind(m, f) = State(|s| let (a, s') = m.run(s) in f(a).run(s'))
    pub fn bind<B: 'static>(self, g: impl FnOnce(A) -> State<S, B> + 'static) -> State<S, B> {
        State::new(move |s| {
            let (a, s2) = (self.f)(s);
            g(a).run(s2)
        })
    }

    /// Get the current state as the value.
    pub fn get() -> State<S, S>
    where
        S: Clone,
    {
        State::new(|s: S| (s.clone(), s))
    }

    /// Replace the state.
    pub fn put(new_state: S) -> State<S, ()> {
        State::new(move |_| ((), new_state))
    }

    /// Modify the state with a function.
    pub fn modify(f: impl FnOnce(S) -> S + 'static) -> State<S, ()> {
        State::new(move |s| ((), f(s)))
    }
}

impl<S, A: Debug> Debug for State<S, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "State<_, _>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Monad laws ---

    #[test]
    fn left_identity() {
        // bind(pure(a), f) = f(a)
        let a = 42;
        let f = |x: i32| State::new(move |s: i32| (x + s, s + 1));

        let left = State::<i32, i32>::pure(a).bind(f);
        let right = (|x: i32| State::new(move |s: i32| (x + s, s + 1)))(a);

        assert_eq!(left.run(10), right.run(10));
    }

    #[test]
    fn right_identity() {
        // bind(m, pure) = m
        let (val1, state1) = State::new(|s: i32| (s * 2, s + 1)).run(10);
        let (val2, state2) = State::new(|s: i32| (s * 2, s + 1))
            .bind(State::<i32, i32>::pure)
            .run(10);

        assert_eq!(val1, val2);
        assert_eq!(state1, state2);
    }

    #[test]
    fn associativity() {
        // bind(bind(m, f), g) = bind(m, |x| bind(f(x), g))
        let m = || State::new(|s: i32| (s, s + 1));
        let f = |x: i32| State::new(move |s: i32| (x * 2, s + 10));
        let g = |x: i32| State::new(move |s: i32| (x + 100, s * 2));

        let left = m().bind(f).bind(g).run(0);
        let right = m()
            .bind(|x| {
                (|x: i32| State::new(move |s: i32| (x * 2, s + 10)))(x)
                    .bind(|y| State::new(move |s: i32| (y + 100, s * 2)))
            })
            .run(0);

        assert_eq!(left, right);
    }

    // --- State operations ---

    #[test]
    fn get_returns_state() {
        let (val, state) = State::<i32, i32>::get().run(42);
        assert_eq!(val, 42);
        assert_eq!(state, 42);
    }

    #[test]
    fn put_replaces_state() {
        let (_, state) = State::<i32, ()>::put(99).run(0);
        assert_eq!(state, 99);
    }

    #[test]
    fn modify_transforms_state() {
        let (_, state) = State::<i32, ()>::modify(|s| s + 10).run(32);
        assert_eq!(state, 42);
    }

    // --- Practical: engine pattern ---

    #[test]
    fn engine_as_state_monad() {
        // Engine: process input through pipeline, accumulate state
        #[derive(Clone, Debug, PartialEq)]
        struct PipelineState {
            tokens: Vec<String>,
            parsed: bool,
        }

        let tokenize = State::new(|mut s: PipelineState| {
            s.tokens = vec!["hello".into(), "world".into()];
            (s.tokens.len(), s)
        });

        let parse = |token_count: usize| {
            State::new(move |mut s: PipelineState| {
                s.parsed = token_count > 0;
                (s.parsed, s)
            })
        };

        let initial = PipelineState {
            tokens: vec![],
            parsed: false,
        };

        let (result, final_state) = tokenize.bind(parse).run(initial);

        assert!(result); // parsed successfully
        assert_eq!(final_state.tokens, vec!["hello", "world"]);
        assert!(final_state.parsed);
    }

    // --- Property: composing get/put is identity ---

    #[test]
    fn get_then_put_is_identity() {
        let (_, state) = State::<i32, i32>::get()
            .bind(|s| State::<i32, ()>::put(s))
            .run(42);
        assert_eq!(state, 42);
    }

    #[test]
    fn put_then_get_returns_new_state() {
        let (val, state) = State::<i32, ()>::put(99)
            .bind(|_| State::<i32, i32>::get())
            .run(0);
        assert_eq!(val, 99);
        assert_eq!(state, 99);
    }
}
