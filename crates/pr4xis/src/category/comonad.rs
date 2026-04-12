use std::fmt::Debug;

// Comonad — dual of monad. Computation in context that you HAVE.
//
// Where a monad wraps a value in a context you need to sequence through,
// a comonad wraps a value WITH a context you can always extract from.
//
// Comonad operations:
//   extract: W(A) → A           (get the focused value)
//   extend: (W(A) → B) → W(A) → W(B)  (apply in context)
//   duplicate: W(A) → W(W(A))   (nest the context)
//
// The comonad laws:
//   1. Left identity:  extend(extract, w) = w
//   2. Right identity: extract(extend(f, w)) = f(w)
//   3. Associativity:  extend(f, extend(g, w)) = extend(|w| f(extend(g, w)), w)
//
// In pr4xis, comonads formalize:
//   - Cofree comonad: the trace schema T(C) = El(C) + O_obs
//     Every element carries its full context (trace history)
//   - Focused views: a concept in context of its neighbors
//   - Streams: infinite unfolding from a seed
//
// References:
// - Uustalu & Vene, "Comonadic Notions of Computation" (2008, ENTCS)
//   https://doi.org/10.1016/j.entcs.2008.05.029
// - Orchard & Yoshida, "Decomposing Comonadic Notions" (2019)
// - Mac Lane, "Categories for the Working Mathematician" (1971), Ch. VI
//   (comonads as dual of monads)

/// A comonad: a value focused within a context.
///
/// `Comonad<C, A>` = a value `A` with context `C`.
/// Unlike a monad where you sequence INTO context,
/// a comonad lets you extract FROM context.
///
/// ```
/// use pr4xis::category::comonad::Focused;
///
/// let f = Focused::new(42, vec![1, 2, 3]);
/// assert_eq!(f.extract(), &42);
/// assert_eq!(f.context(), &vec![1, 2, 3]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Focused<C, A> {
    /// The focused value.
    value: A,
    /// The surrounding context.
    context: C,
}

impl<C: Clone + Debug, A: Clone + Debug> Focused<C, A> {
    /// Create a focused value with context.
    pub fn new(value: A, context: C) -> Self {
        Self { value, context }
    }

    /// Extract the focused value (comonad extract).
    pub fn extract(&self) -> &A {
        &self.value
    }

    /// Get the context.
    pub fn context(&self) -> &C {
        &self.context
    }

    /// Extend: apply a function that sees the whole Focused to produce a new value.
    /// The context is preserved.
    pub fn extend<B: Clone + Debug>(&self, f: impl Fn(&Focused<C, A>) -> B) -> Focused<C, B> {
        Focused {
            value: f(self),
            context: self.context.clone(),
        }
    }

    /// Duplicate: nest the context.
    /// duplicate(w) = Focused(w, w.context)
    pub fn duplicate(&self) -> Focused<C, Focused<C, A>> {
        Focused {
            value: self.clone(),
            context: self.context.clone(),
        }
    }

    /// Map over the focused value (functor).
    pub fn map<B: Clone + Debug>(&self, f: impl Fn(&A) -> B) -> Focused<C, B> {
        Focused {
            value: f(&self.value),
            context: self.context.clone(),
        }
    }
}

/// Cofree comonad — the canonical comonad for building traced structures.
///
/// `Cofree<F, A>` = `(A, F(Cofree<F, A>))` — an infinite tree where
/// every node carries a value and branches via functor F.
///
/// In pr4xis, this models the trace schema: every ontology element
/// carries its full provenance chain.
///
/// References:
/// - Uustalu & Vene, "The Essence of Dataflow Programming" (2005, CATS)
/// - Kmett, "Cofree meets Free" (Haskell Libraries)
#[derive(Debug, Clone, PartialEq)]
pub struct Cofree<A> {
    /// The head value at this node.
    pub head: A,
    /// The tail — children in the cofree structure.
    pub tail: Vec<Cofree<A>>,
}

impl<A: Clone + Debug> Cofree<A> {
    /// Create a leaf (no children).
    pub fn leaf(value: A) -> Self {
        Self {
            head: value,
            tail: Vec::new(),
        }
    }

    /// Create a node with children.
    pub fn node(value: A, children: Vec<Cofree<A>>) -> Self {
        Self {
            head: value,
            tail: children,
        }
    }

    /// Extract the head value (comonad extract).
    pub fn extract(&self) -> &A {
        &self.head
    }

    /// Extend: apply a function over the whole tree structure.
    pub fn extend<B: Clone + Debug>(&self, f: &dyn Fn(&Cofree<A>) -> B) -> Cofree<B> {
        Cofree {
            head: f(self),
            tail: self.tail.iter().map(|child| child.extend(f)).collect(),
        }
    }

    /// Duplicate: every node gets the subtree rooted at it.
    pub fn duplicate(&self) -> Cofree<Cofree<A>> {
        Cofree {
            head: self.clone(),
            tail: self.tail.iter().map(|child| child.duplicate()).collect(),
        }
    }

    /// Fold the cofree structure bottom-up.
    pub fn fold<B>(&self, f: &dyn Fn(&A, &[B]) -> B) -> B {
        let children: Vec<B> = self.tail.iter().map(|c| c.fold(f)).collect();
        f(&self.head, &children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Focused comonad laws ---

    #[test]
    fn focused_extract_returns_value() {
        let f = Focused::new(42, "context");
        assert_eq!(f.extract(), &42);
    }

    #[test]
    fn focused_left_identity() {
        // extend(extract, w) = w
        let w = Focused::new(42, vec![1, 2, 3]);
        let result = w.extend(|fw| fw.extract().clone());
        assert_eq!(result.value, w.value);
        assert_eq!(result.context, w.context);
    }

    #[test]
    fn focused_right_identity() {
        // extract(extend(f, w)) = f(w)
        let w = Focused::new(10, "ctx");
        let f = |fw: &Focused<&str, i32>| fw.extract() * 2;
        let extended = w.extend(f);
        assert_eq!(*extended.extract(), f(&w));
    }

    #[test]
    fn focused_duplicate_extract() {
        let w = Focused::new(42, "ctx");
        let dup = w.duplicate();
        assert_eq!(dup.extract(), &w);
    }

    // --- Cofree comonad ---

    #[test]
    fn cofree_leaf_extract() {
        let leaf = Cofree::leaf(42);
        assert_eq!(cofree_leaf_extract_val(&leaf), &42);
    }

    fn cofree_leaf_extract_val(c: &Cofree<i32>) -> &i32 {
        c.extract()
    }

    #[test]
    fn cofree_tree_structure() {
        let tree = Cofree::node("root", vec![Cofree::leaf("left"), Cofree::leaf("right")]);
        assert_eq!(tree.extract(), &"root");
        assert_eq!(tree.tail.len(), 2);
        assert_eq!(tree.tail[0].extract(), &"left");
    }

    #[test]
    fn cofree_fold_counts_nodes() {
        let tree = Cofree::node(
            1,
            vec![Cofree::node(2, vec![Cofree::leaf(3)]), Cofree::leaf(4)],
        );
        let count = tree.fold(&|_, children: &[usize]| 1 + children.iter().sum::<usize>());
        assert_eq!(count, 4);
    }

    #[test]
    fn cofree_extend_sums_subtrees() {
        let tree = Cofree::node(1, vec![Cofree::leaf(2), Cofree::leaf(3)]);
        let summed = tree
            .extend(&|node| node.fold(&|val, children: &[i32]| val + children.iter().sum::<i32>()));
        assert_eq!(*summed.extract(), 6); // 1+2+3
        assert_eq!(*summed.tail[0].extract(), 2);
        assert_eq!(*summed.tail[1].extract(), 3);
    }

    // --- Practical: trace schema as cofree ---

    #[test]
    fn trace_schema_as_cofree() {
        // Each pipeline step carries its result + subtrace
        let trace = Cofree::node(
            ("tokenize", "5 tokens"),
            vec![
                Cofree::node(
                    ("parse", "S[dcl]"),
                    vec![Cofree::leaf(("interpret", "prop(dog, big)"))],
                ),
                Cofree::leaf(("metacognition", "KnownKnown")),
            ],
        );

        // Extract the root step
        assert_eq!(trace.extract().0, "tokenize");

        // Fold to collect all step names
        let steps = trace.fold(&|val, children: &[Vec<&str>]| {
            let mut all = vec![val.0];
            for c in children {
                all.extend(c.iter());
            }
            all
        });
        assert_eq!(
            steps,
            vec!["tokenize", "parse", "interpret", "metacognition"]
        );
    }
}
