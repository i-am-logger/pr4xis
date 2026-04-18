use core::fmt::Debug;

use super::monoid::Monoid;

// Monad — a computation context with sequencing.
//
// The three monad laws:
//   1. Left identity:  bind(pure(a), f) = f(a)
//   2. Right identity: bind(m, pure) = m
//   3. Associativity:  bind(bind(m, f), g) = bind(m, |x| bind(f(x), g))
//
// In pr4xis, monads model computational effects:
//   - Writer<W, A>: computation that accumulates a log W (trace, provenance)
//   - Option<A>: computation that may fail (partial morphisms)
//   - Vec<A>: nondeterministic computation (multiple parses, ambiguity)
//
// References:
// - Moggi, "Notions of Computation and Monads" (1991, Inf. & Comp.)
// - Wadler, "Monads for Functional Programming" (1995)
// - Mac Lane, "Categories for the Working Mathematician" (1971), Ch. VI

/// A writer monad: pairs a value with an accumulated log.
///
/// Writer<W, A> = (A, W) where W: Monoid.
///
/// - `pure(a)` = `(a, W::empty())`
/// - `bind((a, w), f)` = let `(b, w')` = f(a) in `(b, w.combine(w'))`
///
/// The trace accumulates automatically through composition.
/// This is how TracedCategory works: morphisms carry trace records,
/// and composition concatenates them (Vec<TraceRecord> is the monoid).
#[derive(Debug, Clone)]
pub struct Writer<W: Monoid, A: Clone + Debug> {
    /// The computed value.
    pub value: A,
    /// The accumulated log.
    pub log: W,
}

impl<W: Monoid, A: Clone + Debug> Writer<W, A> {
    /// Lift a pure value into the writer monad (no log).
    /// Monad law: pure is the left/right identity of bind.
    pub fn pure(value: A) -> Self {
        Self {
            value,
            log: W::empty(),
        }
    }

    /// Create a writer with an initial log entry.
    pub fn new(value: A, log: W) -> Self {
        Self { value, log }
    }

    /// Monadic bind: sequence this computation with f, accumulating logs.
    ///
    /// bind((a, w), f) = let (b, w') = f(a) in (b, w ++ w')
    pub fn bind<B: Clone + Debug>(self, f: impl FnOnce(A) -> Writer<W, B>) -> Writer<W, B> {
        let Writer {
            value: b,
            log: w_prime,
        } = f(self.value);
        Writer {
            value: b,
            log: self.log.combine(&w_prime),
        }
    }

    /// Map a function over the value, preserving the log.
    /// Functor map: fmap f (a, w) = (f(a), w)
    pub fn map<B: Clone + Debug>(self, f: impl FnOnce(A) -> B) -> Writer<W, B> {
        Writer {
            value: f(self.value),
            log: self.log,
        }
    }

    /// Append to the log without changing the value.
    pub fn tell(self, entry: W) -> Self {
        Self {
            value: self.value,
            log: self.log.combine(&entry),
        }
    }
}

impl<W: Monoid + PartialEq, A: Clone + Debug + PartialEq> PartialEq for Writer<W, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.log == other.log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Writer Monad Laws ---

    #[test]
    fn left_identity() {
        // bind(pure(a), f) = f(a)
        let a = 42;
        let f = |x: i32| Writer::new(x * 2, vec!["doubled"]);

        let left = Writer::<Vec<&str>, i32>::pure(a).bind(f);
        let right = Writer::new(a, Vec::new()).bind(|x| Writer::new(x * 2, vec!["doubled"]));

        assert_eq!(left.value, right.value);
        assert_eq!(left.log, right.log);
    }

    #[test]
    fn right_identity() {
        // bind(m, pure) = m
        let m = Writer::new(42, vec!["initial"]);
        let result = m.clone().bind(Writer::<Vec<&str>, i32>::pure);

        assert_eq!(result.value, m.value);
        assert_eq!(result.log, m.log);
    }

    #[test]
    fn associativity() {
        // bind(bind(m, f), g) = bind(m, |x| bind(f(x), g))
        let m = Writer::new(10, vec!["start"]);
        let f = |x: i32| Writer::new(x + 5, vec!["added 5"]);
        let g = |x: i32| Writer::new(x * 2, vec!["doubled"]);

        let left = m.clone().bind(f).bind(g);
        let right = m.bind(|x| {
            let fx = Writer::new(x + 5, vec!["added 5"]);
            fx.bind(|y| Writer::new(y * 2, vec!["doubled"]))
        });

        assert_eq!(left.value, right.value);
        assert_eq!(left.log, right.log);
    }

    // --- Practical usage ---

    #[test]
    fn trace_accumulates_through_bind() {
        let result = Writer::new(1, vec!["parsed"])
            .bind(|x| Writer::new(x + 1, vec!["interpreted"]))
            .bind(|x| Writer::new(x * 10, vec!["generated"]));

        assert_eq!(result.value, 20);
        assert_eq!(result.log, vec!["parsed", "interpreted", "generated"]);
    }

    #[test]
    fn tell_appends_log() {
        let result = Writer::<Vec<&str>, i32>::pure(42).tell(vec!["traced"]);

        assert_eq!(result.value, 42);
        assert_eq!(result.log, vec!["traced"]);
    }

    #[test]
    fn map_preserves_log() {
        let result = Writer::new(21, vec!["initial"]).map(|x| x * 2);

        assert_eq!(result.value, 42);
        assert_eq!(result.log, vec!["initial"]);
    }

    mod prop {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Writer left identity: bind(pure(a), f) = f(a)
            #[test]
            fn prop_writer_left_identity(a in -100..100i32) {
                let f = |x: i32| Writer::new(x * 2, vec![x]);
                let left = Writer::<Vec<i32>, i32>::pure(a).bind(&f);
                let right = f(a);
                prop_assert_eq!(left.value, right.value);
                prop_assert_eq!(left.log, right.log);
            }

            /// Writer right identity: bind(m, pure) = m
            #[test]
            fn prop_writer_right_identity(a in -100..100i32, log_val in -100..100i32) {
                let m = Writer::new(a, vec![log_val]);
                let result = Writer::new(a, vec![log_val]).bind(Writer::<Vec<i32>, i32>::pure);
                prop_assert_eq!(m.value, result.value);
                prop_assert_eq!(m.log, result.log);
            }

            /// Monoid identity: pure produces empty log
            #[test]
            fn prop_writer_pure_empty_log(a in -100..100i32) {
                let w = Writer::<Vec<i32>, i32>::pure(a);
                prop_assert!(w.log.is_empty());
                prop_assert_eq!(w.value, a);
            }

            /// Tell only appends, doesn't change value
            #[test]
            fn prop_writer_tell_preserves_value(a in -100..100i32, entry in -100..100i32) {
                let w = Writer::<Vec<i32>, i32>::pure(a).tell(vec![entry]);
                prop_assert_eq!(w.value, a);
                prop_assert_eq!(w.log, vec![entry]);
            }

            /// Map preserves log
            #[test]
            fn prop_writer_map_preserves_log(a in -100..100i32, log_val in -100..100i32) {
                let w = Writer::new(a, vec![log_val]).map(|x| x + 1);
                prop_assert_eq!(w.value, a + 1);
                prop_assert_eq!(w.log, vec![log_val]);
            }
        }
    }
}
