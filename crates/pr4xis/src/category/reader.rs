#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::fmt::Debug;

// Reader monad — computation that depends on an environment.
//
// Reader<E, A> represents a computation that reads from environment E
// to produce value A. The environment is threaded implicitly —
// the computation doesn't modify it.
//
// In pr4xis, the Reader monad formalizes:
//   - ContextDef: (Entity, Signal) → Resolution (molecular functional context)
//   - Quality::get: Individual → Option<Value> (reading a property from an entity)
//   - Taxonomy queries: Entity → Vec<Entity> (reading ancestors from a taxonomy)
//
// The three monad laws:
//   1. Left identity:  bind(pure(a), f) = f(a)
//   2. Right identity: bind(m, pure) = m
//   3. Associativity:  bind(bind(m, f), g) = bind(m, |x| bind(f(x), g))
//
// References:
// - Moggi, "Notions of Computation and Monads" (1991, Inf. & Comp.)
//   https://doi.org/10.1016/0890-5401(91)90052-4
// - Wadler, "The Essence of Functional Programming" (1992, POPL)
//   https://doi.org/10.1145/143165.143169
// - Jones, "Functional Programming with Overloading and Higher-Order Polymorphism"
//   (1995, AFP Summer School) — Reader as environment-passing

/// A reader monad: computation that reads from an environment.
///
/// `Reader<E, A>` = `E → A`. The environment is available but immutable.
///
/// ```
/// use pr4xis::category::reader::Reader;
///
/// let double = Reader::new(|x: &i32| x * 2);
/// assert_eq!(double.run(&21), 42);
///
/// let add_then_double = Reader::new(|x: &i32| x + 1)
///     .map(|y| y * 2);
/// assert_eq!(add_then_double.run(&20), 42);
/// ```
pub struct Reader<E, A> {
    f: Box<dyn Fn(&E) -> A>,
}

impl<E: 'static, A: 'static> Reader<E, A> {
    /// Create a reader from a function.
    pub fn new(f: impl Fn(&E) -> A + 'static) -> Self {
        Self { f: Box::new(f) }
    }

    /// Lift a pure value (ignores environment).
    /// Monad law: pure is the left/right identity of bind.
    pub fn pure(a: A) -> Self
    where
        A: Clone,
    {
        Self {
            f: Box::new(move |_| a.clone()),
        }
    }

    /// Run the reader with an environment.
    pub fn run(&self, env: &E) -> A {
        (self.f)(env)
    }

    /// Functor map: transform the output.
    pub fn map<B: 'static>(self, g: impl Fn(A) -> B + 'static) -> Reader<E, B> {
        Reader::new(move |env| g((self.f)(env)))
    }

    /// Monadic bind: sequence with a function that returns a new Reader.
    /// bind(m, f) = Reader(|env| f(m.run(env)).run(env))
    pub fn bind<B: 'static>(self, g: impl Fn(A) -> Reader<E, B> + 'static) -> Reader<E, B> {
        Reader::new(move |env| {
            let a = (self.f)(env);
            g(a).run(env)
        })
    }

    /// Ask for the environment itself.
    pub fn ask() -> Reader<E, E>
    where
        E: Clone,
    {
        Reader::new(|env: &E| env.clone())
    }
}

impl<E, A: Debug> Debug for Reader<E, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Reader<_, _>")
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
        let f = |x: i32| Reader::new(move |env: &i32| x + env);

        let left = Reader::<i32, i32>::pure(a).bind(f);
        let right = (|x: i32| Reader::new(move |env: &i32| x + env))(a);

        assert_eq!(left.run(&10), right.run(&10));
    }

    #[test]
    fn right_identity() {
        // bind(m, pure) = m
        let m = Reader::new(|env: &i32| env * 2);
        let result = Reader::new(|env: &i32| env * 2).bind(|a| Reader::<i32, i32>::pure(a));

        assert_eq!(m.run(&21), result.run(&21));
    }

    #[test]
    fn associativity() {
        // bind(bind(m, f), g) = bind(m, |x| bind(f(x), g))
        let m = Reader::new(|env: &i32| env + 1);
        let f = |x: i32| Reader::new(move |env: &i32| x * env);
        let g = |x: i32| Reader::new(move |_env: &i32| x + 100);

        let left = Reader::new(|env: &i32| env + 1)
            .bind(|x| Reader::new(move |env: &i32| x * env))
            .bind(|x| Reader::new(move |_env: &i32| x + 100));

        let right = Reader::new(|env: &i32| env + 1).bind(|x| {
            (|x: i32| Reader::new(move |env: &i32| x * env))(x)
                .bind(|y| Reader::new(move |_env: &i32| y + 100))
        });

        let _ = (m, f, g); // suppress unused warnings
        assert_eq!(left.run(&10), right.run(&10));
    }

    // --- Practical usage ---

    #[test]
    fn ask_returns_environment() {
        let r = Reader::<String, String>::ask();
        assert_eq!(r.run(&"hello".to_string()), "hello");
    }

    #[test]
    fn map_transforms_output() {
        let r = Reader::new(|x: &i32| x + 1).map(|y| y * 2);
        assert_eq!(r.run(&20), 42);
    }

    #[test]
    fn context_resolution_example() {
        // Simulates ContextDef: (entity, signal) → resolution
        #[derive(Clone)]
        struct Context {
            therapeutic: bool,
        }

        let resolve = Reader::new(|ctx: &Context| {
            if ctx.therapeutic {
                "therapeutic_target"
            } else {
                "passive_homeostatic"
            }
        });

        assert_eq!(
            resolve.run(&Context { therapeutic: true }),
            "therapeutic_target"
        );
        assert_eq!(
            resolve.run(&Context { therapeutic: false }),
            "passive_homeostatic"
        );
    }
}
