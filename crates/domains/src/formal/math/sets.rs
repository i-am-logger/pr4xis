use std::collections::HashSet;
use std::hash::Hash;

/// Set operations with algebraic law enforcement.
pub fn union<T: Eq + Hash + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    a.union(b).cloned().collect()
}

pub fn intersection<T: Eq + Hash + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    a.intersection(b).cloned().collect()
}

pub fn difference<T: Eq + Hash + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    a.difference(b).cloned().collect()
}

pub fn symmetric_difference<T: Eq + Hash + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    a.symmetric_difference(b).cloned().collect()
}

pub fn is_subset<T: Eq + Hash>(a: &HashSet<T>, b: &HashSet<T>) -> bool {
    a.is_subset(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_set() -> impl Strategy<Value = HashSet<i32>> {
        proptest::collection::hash_set(-50..50i32, 0..20)
    }

    proptest! {
        /// Union is commutative: A ∪ B = B ∪ A
        #[test]
        fn prop_union_commutative(a in arb_set(), b in arb_set()) {
            prop_assert_eq!(union(&a, &b), union(&b, &a));
        }

        /// Intersection is commutative: A ∩ B = B ∩ A
        #[test]
        fn prop_intersection_commutative(a in arb_set(), b in arb_set()) {
            prop_assert_eq!(intersection(&a, &b), intersection(&b, &a));
        }

        /// Union with empty = self: A ∪ ∅ = A
        #[test]
        fn prop_union_identity(a in arb_set()) {
            let empty = HashSet::new();
            prop_assert_eq!(union(&a, &empty), a);
        }

        /// Intersection with empty = empty: A ∩ ∅ = ∅
        #[test]
        fn prop_intersection_empty(a in arb_set()) {
            let empty: HashSet<i32> = HashSet::new();
            prop_assert_eq!(intersection(&a, &empty), empty);
        }

        /// Union is idempotent: A ∪ A = A
        #[test]
        fn prop_union_idempotent(a in arb_set()) {
            prop_assert_eq!(union(&a, &a), a);
        }

        /// Intersection is idempotent: A ∩ A = A
        #[test]
        fn prop_intersection_idempotent(a in arb_set()) {
            prop_assert_eq!(intersection(&a, &a), a);
        }

        /// A ⊆ A ∪ B
        #[test]
        fn prop_subset_of_union(a in arb_set(), b in arb_set()) {
            let u = union(&a, &b);
            prop_assert!(is_subset(&a, &u));
            prop_assert!(is_subset(&b, &u));
        }

        /// A ∩ B ⊆ A
        #[test]
        fn prop_intersection_subset(a in arb_set(), b in arb_set()) {
            let i = intersection(&a, &b);
            prop_assert!(is_subset(&i, &a));
            prop_assert!(is_subset(&i, &b));
        }

        /// De Morgan's: A \ (B ∪ C) = (A \ B) ∩ (A \ C)
        #[test]
        fn prop_de_morgan(a in arb_set(), b in arb_set(), c in arb_set()) {
            let left = difference(&a, &union(&b, &c));
            let right = intersection(&difference(&a, &b), &difference(&a, &c));
            prop_assert_eq!(left, right);
        }

        /// Distributive: A ∩ (B ∪ C) = (A ∩ B) ∪ (A ∩ C)
        #[test]
        fn prop_distributive(a in arb_set(), b in arb_set(), c in arb_set()) {
            let left = intersection(&a, &union(&b, &c));
            let right = union(&intersection(&a, &b), &intersection(&a, &c));
            prop_assert_eq!(left, right);
        }

        /// |A ∪ B| = |A| + |B| - |A ∩ B| (inclusion-exclusion)
        #[test]
        fn prop_inclusion_exclusion(a in arb_set(), b in arb_set()) {
            let u = union(&a, &b);
            let i = intersection(&a, &b);
            prop_assert_eq!(u.len(), a.len() + b.len() - i.len());
        }

        /// Symmetric difference is commutative
        #[test]
        fn prop_symmetric_diff_commutative(a in arb_set(), b in arb_set()) {
            prop_assert_eq!(symmetric_difference(&a, &b), symmetric_difference(&b, &a));
        }

        /// A △ A = ∅ (symmetric difference with self is empty)
        #[test]
        fn prop_symmetric_diff_self_empty(a in arb_set()) {
            let empty: HashSet<i32> = HashSet::new();
            prop_assert_eq!(symmetric_difference(&a, &a), empty);
        }
    }
}
