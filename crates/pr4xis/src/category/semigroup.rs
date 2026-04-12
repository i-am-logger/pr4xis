use std::fmt::Debug;

// Semigroup — a set with an associative binary operation (no identity required).
//
// Semigroup is Monoid without the identity element. Every Monoid is a
// Semigroup, but not vice versa. Useful for NonEmpty collections and
// partial accumulations.
//
// The semigroup law:
//   Associativity: combine(combine(a, b), c) = combine(a, combine(b, c))
//
// References:
// - Clifford & Preston, "The Algebraic Theory of Semigroups" (1961, AMS)
// - Howie, "Fundamentals of Semigroup Theory" (1995, Oxford)
// - Brent Yorgey, "Typeclassopedia" (2009, Haskell Wiki)

/// A semigroup: a type with an associative binary operation.
/// Unlike Monoid, no identity element is required.
pub trait Semigroup: Clone + Debug {
    /// Associative binary operation.
    fn combine(&self, other: &Self) -> Self;
}

// Every Monoid is a Semigroup.
impl<T: super::monoid::Monoid> Semigroup for T {
    fn combine(&self, other: &Self) -> Self {
        super::monoid::Monoid::combine(self, other)
    }
}

/// A non-empty collection — guaranteed to have at least one element.
/// Forms a semigroup under concatenation (no empty identity needed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmpty<T> {
    pub head: T,
    pub tail: Vec<T>,
}

impl<T: Clone + Debug> NonEmpty<T> {
    pub fn new(head: T) -> Self {
        Self {
            head,
            tail: Vec::new(),
        }
    }

    pub fn of(head: T, tail: Vec<T>) -> Self {
        Self { head, tail }
    }

    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.head).chain(self.tail.iter())
    }

    pub fn push(&mut self, item: T) {
        self.tail.push(item);
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut v = vec![self.head.clone()];
        v.extend(self.tail.iter().cloned());
        v
    }
}

impl<T: Clone + Debug> Semigroup for NonEmpty<T> {
    fn combine(&self, other: &Self) -> Self {
        let mut tail = self.tail.clone();
        tail.push(other.head.clone());
        tail.extend(other.tail.iter().cloned());
        Self {
            head: self.head.clone(),
            tail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonempty_semigroup_associativity() {
        let a = NonEmpty::new(1);
        let b = NonEmpty::of(2, vec![3]);
        let c = NonEmpty::new(4);
        assert_eq!(a.combine(&b).combine(&c), a.combine(&b.combine(&c)));
    }

    #[test]
    fn nonempty_len() {
        let ne = NonEmpty::of(1, vec![2, 3]);
        assert_eq!(ne.len(), 3);
    }

    #[test]
    fn nonempty_to_vec() {
        let ne = NonEmpty::of(1, vec![2, 3]);
        assert_eq!(ne.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn vec_is_semigroup_via_monoid() {
        let a = vec![1, 2];
        let b = vec![3];
        assert_eq!(Semigroup::combine(&a, &b), vec![1, 2, 3]);
    }

    mod prop {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_nonempty_associativity(
                a in any::<i32>(),
                b in any::<i32>(),
                c in any::<i32>(),
            ) {
                let na = NonEmpty::new(a);
                let nb = NonEmpty::new(b);
                let nc = NonEmpty::new(c);
                prop_assert_eq!(
                    na.combine(&nb).combine(&nc),
                    na.combine(&nb.combine(&nc))
                );
            }

            #[test]
            fn prop_nonempty_combine_length(a in any::<i32>(), b in any::<i32>()) {
                let na = NonEmpty::new(a);
                let nb = NonEmpty::new(b);
                prop_assert_eq!(na.combine(&nb).len(), 2);
            }
        }
    }
}
