use std::fmt::Debug;

// Monoid — a set with an associative binary operation and identity element.
//
// The three monoid laws:
//   1. Associativity: combine(combine(a, b), c) = combine(a, combine(b, c))
//   2. Left identity: combine(empty(), a) = a
//   3. Right identity: combine(a, empty()) = a
//
// In pr4xis, monoids underlie:
//   - Trace accumulation: (Vec<TraceRecord>, concat, [])
//   - Morphism composition: (Morphisms, compose, identity)
//   - String building: (String, +, "")
//
// References:
// - Mac Lane, "Categories for the Working Mathematician" (1971), Ch. VII
// - Haskell Data.Monoid — the standard formalization

/// A monoid: a type with an associative binary operation and identity element.
pub trait Monoid: Clone + Debug {
    /// The identity element. For all a: combine(empty(), a) = a = combine(a, empty()).
    fn empty() -> Self;

    /// The associative binary operation. combine(combine(a, b), c) = combine(a, combine(b, c)).
    fn combine(&self, other: &Self) -> Self;
}

// --- Standard monoid instances ---

/// Vec<T> is a monoid under concatenation.
impl<T: Clone + Debug> Monoid for Vec<T> {
    fn empty() -> Self {
        Vec::new()
    }

    fn combine(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.extend(other.iter().cloned());
        result
    }
}

/// String is a monoid under concatenation.
impl Monoid for String {
    fn empty() -> Self {
        String::new()
    }

    fn combine(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.push_str(other);
        result
    }
}

/// () is the trivial monoid.
impl Monoid for () {
    fn empty() -> Self {}

    fn combine(&self, _other: &Self) -> Self {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_monoid_identity() {
        let v = vec![1, 2, 3];
        assert_eq!(Vec::<i32>::empty().combine(&v), v);
        assert_eq!(v.combine(&Vec::empty()), v);
    }

    #[test]
    fn vec_monoid_associativity() {
        let a = vec![1];
        let b = vec![2];
        let c = vec![3];
        assert_eq!(a.combine(&b).combine(&c), a.combine(&b.combine(&c)));
    }

    #[test]
    fn string_monoid_identity() {
        let s = "hello".to_string();
        assert_eq!(String::empty().combine(&s), s);
        assert_eq!(s.combine(&String::empty()), s);
    }

    #[test]
    fn string_monoid_associativity() {
        let a = "a".to_string();
        let b = "b".to_string();
        let c = "c".to_string();
        assert_eq!(a.combine(&b).combine(&c), a.combine(&b.combine(&c)));
    }

    #[test]
    fn unit_monoid() {
        assert_eq!(().combine(&()), ());
    }
}
