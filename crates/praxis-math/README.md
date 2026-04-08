# praxis-math

[![crates.io](https://img.shields.io/crates/v/praxis-math.svg)](https://crates.io/crates/praxis-math)
[![docs.rs](https://img.shields.io/docsrs/praxis-math)](https://docs.rs/praxis-math)

Mathematical foundations with axiom enforcement -- primes, Fibonacci, Pythagorean theorem, quadratics, sets, and path integrals.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models core mathematical structures as enforceable ontologies. The Pythagorean theorem and quadratic formula are engine-driven -- transformations are rejected if they would violate invariants (a^2 + b^2 = c^2, roots satisfying the equation). Set operations verify algebraic laws (commutativity, De Morgan's, inclusion-exclusion). The Feynman path integral models quantum amplitude accumulation with phase consistency enforcement. The ontology layer maps number domains (N, Z, Q, R, C) as a category with subset morphisms.

## Key Types

| Type | Description |
|---|---|
| `primes::is_prime` | Primality test |
| `primes::sieve` | Sieve of Eratosthenes |
| `primes::factorize` | Prime factorization (fundamental theorem of arithmetic) |
| `primes::goldbach` | Goldbach decomposition of even numbers |
| `fibonacci::fib` | Fibonacci sequence with golden ratio and Cassini's identity |
| `pythagorean::Triangle` | Right triangle with a^2 + b^2 = c^2 enforced on every action |
| `pythagorean::triples` | Generate Pythagorean triples up to a bound |
| `quadratic::Quadratic` | Quadratic equation with auto-computed roots (real, repeated, or complex) |
| `sets::{union, intersection, difference, symmetric_difference}` | Set operations with algebraic law verification |
| `feynman::PathIntegral` | Feynman path integral -- accumulate paths, sum amplitudes e^(iS/h-bar) |
| `ontology::MathDomain` | Number domains as category objects: N, Z, Q, R, C with subset morphisms |

## Example

```rust
use praxis_math::pythagorean;
use praxis_math::primes;

// Pythagorean theorem enforced as engine precondition
let engine = pythagorean::new_triangle(3.0, 4.0).unwrap();
assert!((engine.situation().c - 5.0).abs() < 1e-10);

// Prime factorization
assert_eq!(primes::factorize(60), vec![2, 2, 3, 5]);
assert!(primes::is_prime(97));

// Goldbach's conjecture
let (a, b) = primes::goldbach(28).unwrap();
assert_eq!(a + b, 28);
```

## License

CC BY-NC-SA 4.0
