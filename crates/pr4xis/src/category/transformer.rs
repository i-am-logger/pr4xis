use std::fmt::Debug;

use super::monoid::Monoid;

// Monad transformers — composing monadic effects.
//
// A monad transformer T takes a monad M and produces a new monad T(M)
// that combines both effects. The chat pipeline IS:
//   WriterT<Vec<TraceRecord>,
//     StateT<PipelineState,
//       Reader<Language, Response>>>
//
// Transformers stack effects without manual plumbing.
//
// References:
// - Liang, Hudak & Jones, "Monad Transformers and Modular Interpreters"
//   (1995, POPL) https://doi.org/10.1145/199448.199528
// - Moggi, "Notions of Computation and Monads" (1991)
// - Jones, "Functional Programming with Overloading and Higher-Order
//   Polymorphism" (1995, AFP)

/// WriterT: adds logging/tracing to any computation.
///
/// `WriterT<W, A>` wraps a computation that produces `(A, W)`.
/// W is a Monoid (accumulates through bind).
///
/// This is the transformer that gives TracedCategory its power.
#[derive(Debug, Clone)]
pub struct WriterT<W: Monoid, A: Clone + Debug> {
    pub value: A,
    pub log: W,
}

impl<W: Monoid, A: Clone + Debug> WriterT<W, A> {
    pub fn pure(a: A) -> Self {
        Self {
            value: a,
            log: W::empty(),
        }
    }

    pub fn new(value: A, log: W) -> Self {
        Self { value, log }
    }

    pub fn bind<B: Clone + Debug>(self, f: impl FnOnce(A) -> WriterT<W, B>) -> WriterT<W, B> {
        let WriterT { value, log } = f(self.value);
        WriterT {
            value,
            log: self.log.combine(&log),
        }
    }

    pub fn tell(self, w: W) -> Self {
        Self {
            value: self.value,
            log: self.log.combine(&w),
        }
    }

    pub fn map<B: Clone + Debug>(self, f: impl FnOnce(A) -> B) -> WriterT<W, B> {
        WriterT {
            value: f(self.value),
            log: self.log,
        }
    }
}

/// StateT: adds mutable state to any computation.
///
/// `StateT<S, A>` wraps `S → (A, S)` — threading state.
pub struct StateT<S, A> {
    run: Box<dyn FnOnce(S) -> (A, S)>,
}

impl<S: 'static, A: 'static> StateT<S, A> {
    pub fn new(f: impl FnOnce(S) -> (A, S) + 'static) -> Self {
        Self { run: Box::new(f) }
    }

    pub fn pure(a: A) -> Self {
        Self::new(move |s| (a, s))
    }

    pub fn run(self, s: S) -> (A, S) {
        (self.run)(s)
    }

    pub fn bind<B: 'static>(self, f: impl FnOnce(A) -> StateT<S, B> + 'static) -> StateT<S, B> {
        StateT::new(move |s| {
            let (a, s2) = (self.run)(s);
            f(a).run(s2)
        })
    }

    pub fn get() -> StateT<S, S>
    where
        S: Clone,
    {
        StateT::new(|s: S| (s.clone(), s))
    }

    pub fn put(new_s: S) -> StateT<S, ()> {
        StateT::new(move |_| ((), new_s))
    }

    pub fn modify(f: impl FnOnce(S) -> S + 'static) -> StateT<S, ()> {
        StateT::new(move |s| ((), f(s)))
    }
}

/// ReaderT: adds environment access to any computation.
///
/// `ReaderT<E, A>` wraps `E → A` — reading from environment.
pub struct ReaderT<E, A> {
    run: Box<dyn Fn(&E) -> A>,
}

impl<E: 'static, A: 'static> ReaderT<E, A> {
    pub fn new(f: impl Fn(&E) -> A + 'static) -> Self {
        Self { run: Box::new(f) }
    }

    pub fn pure(a: A) -> Self
    where
        A: Clone,
    {
        Self::new(move |_| a.clone())
    }

    pub fn run(&self, env: &E) -> A {
        (self.run)(env)
    }

    pub fn map<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> ReaderT<E, B> {
        ReaderT::new(move |env| f((self.run)(env)))
    }

    pub fn bind<B: 'static>(self, f: impl Fn(A) -> ReaderT<E, B> + 'static) -> ReaderT<E, B> {
        ReaderT::new(move |env| {
            let a = (self.run)(env);
            f(a).run(env)
        })
    }

    pub fn ask() -> ReaderT<E, E>
    where
        E: Clone,
    {
        ReaderT::new(|env: &E| env.clone())
    }
}

/// Composed transformer: WriterT over StateT.
/// Combines tracing (Writer) with state threading (State).
///
/// This IS the chat pipeline computation model:
///   - State: the pipeline accumulates results
///   - Writer: each step logs a trace entry
pub struct TracedState<W: Monoid, S, A> {
    run: Box<dyn FnOnce(S) -> (A, S, W)>,
}

impl<W: Monoid + 'static, S: 'static, A: 'static> TracedState<W, S, A> {
    pub fn new(f: impl FnOnce(S) -> (A, S, W) + 'static) -> Self {
        Self { run: Box::new(f) }
    }

    pub fn pure(a: A) -> Self {
        Self::new(move |s| (a, s, W::empty()))
    }

    pub fn run(self, s: S) -> (A, S, W) {
        (self.run)(s)
    }

    pub fn bind<B: 'static>(
        self,
        f: impl FnOnce(A) -> TracedState<W, S, B> + 'static,
    ) -> TracedState<W, S, B> {
        TracedState::new(move |s| {
            let (a, s2, w1) = (self.run)(s);
            let (b, s3, w2) = f(a).run(s2);
            (b, s3, w1.combine(&w2))
        })
    }

    pub fn tell(w: W) -> TracedState<W, S, ()> {
        TracedState::new(move |s| ((), s, w))
    }

    pub fn get() -> TracedState<W, S, S>
    where
        S: Clone,
    {
        TracedState::new(move |s: S| (s.clone(), s, W::empty()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_t_accumulates() {
        let result = WriterT::new(1, vec!["start"])
            .bind(|x| WriterT::new(x + 1, vec!["incremented"]))
            .bind(|x| WriterT::new(x * 10, vec!["scaled"]));

        assert_eq!(result.value, 20);
        assert_eq!(result.log, vec!["start", "incremented", "scaled"]);
    }

    #[test]
    fn state_t_threads_state() {
        let (result, final_state) = StateT::new(|s: i32| (s * 2, s + 1))
            .bind(|x| StateT::new(move |s: i32| (x + s, s + 10)))
            .run(5);

        // Step 1: s=5, result=10, new_s=6
        // Step 2: s=6, result=10+6=16, new_s=16
        assert_eq!(result, 16);
        assert_eq!(final_state, 16);
    }

    #[test]
    fn reader_t_reads_env() {
        let computation =
            ReaderT::new(|env: &i32| env * 2).bind(|x| ReaderT::new(move |env: &i32| x + env));

        assert_eq!(computation.run(&10), 30); // 10*2 + 10
    }

    #[test]
    fn traced_state_pipeline() {
        // Simulate chat pipeline: state = token count, trace = step log
        let pipeline = TracedState::<Vec<&str>, usize, &str>::pure("hello world")
            .bind(|input| {
                TracedState::new(move |_s| {
                    let tokens = input.split_whitespace().count();
                    (tokens, tokens, vec!["tokenized"])
                })
            })
            .bind(|count| {
                TracedState::new(move |s| {
                    let parsed = count > 0;
                    (parsed, s, vec!["parsed"])
                })
            });

        let (result, state, trace) = pipeline.run(0);
        assert!(result); // parsed = true
        assert_eq!(state, 2); // 2 tokens
        assert_eq!(trace, vec!["tokenized", "parsed"]);
    }

    #[test]
    fn traced_state_tell() {
        let ((), state, trace) = TracedState::<Vec<&str>, i32, ()>::tell(vec!["logged"]).run(42);
        assert_eq!(state, 42); // unchanged
        assert_eq!(trace, vec!["logged"]);
    }
}
