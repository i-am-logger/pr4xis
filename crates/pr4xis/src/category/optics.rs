use std::sync::Arc;

// Optics — composable bidirectional accessors.
//
// An optic is a first-class accessor that can get, set, or modify
// a part of a whole. Optics compose: if you can access B inside A,
// and C inside B, you can access C inside A.
//
// The optics hierarchy:
//   Lens: access exactly one part (get + set)
//   Prism: access a possible part (preview + review)
//   Iso: bidirectional isomorphism (there + back)
//
// In pr4xis, optics formalize:
//   - Ontology views: "show me just the taxonomy" = Lens into ontology
//   - Entity projection: focus on one aspect of a concept
//   - Functor composition: functors ARE optics between categories
//
// References:
// - van Laarhoven, "CPS-based functional references" (2009, blog)
// - Kmett, "lens" library (2012-present, Haskell)
// - Pickering, Gibbons & Wu, "Profunctor Optics: Modular Data Accessors"
//   (2017, The Art, Science, and Engineering of Programming)
//   https://doi.org/10.22152/programming-journal.org/2017/1/7
// - Foster et al., "Combinators for Bidirectional Tree Transformations"
//   (2007, POPL) https://doi.org/10.1145/1190216.1190231

/// A lens: a composable accessor for a part of a whole.
///
/// `Lens<S, A>` can get an `A` from an `S`, and set a new `A` inside `S`.
/// Uses Arc internally so lenses can be composed without move issues.
///
/// ```
/// use pr4xis::category::optics::Lens;
///
/// #[derive(Clone, Debug)]
/// struct Person { name: String, age: u32 }
///
/// let age_lens = Lens::new(
///     |p: &Person| p.age,
///     |p: &Person, a: u32| Person { age: a, ..p.clone() },
/// );
///
/// let alice = Person { name: "Alice".into(), age: 30 };
/// assert_eq!(age_lens.get(&alice), 30);
///
/// let older = age_lens.set(&alice, 31);
/// assert_eq!(older.age, 31);
/// assert_eq!(older.name, "Alice");
/// ```
#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub struct Lens<S, A> {
    getter: Arc<dyn Fn(&S) -> A>,
    setter: Arc<dyn Fn(&S, A) -> S>,
}

impl<S: 'static, A: 'static> Lens<S, A> {
    pub fn new(get: impl Fn(&S) -> A + 'static, set: impl Fn(&S, A) -> S + 'static) -> Self {
        Self {
            getter: Arc::new(get),
            setter: Arc::new(set),
        }
    }

    /// Get the focused value.
    pub fn get(&self, whole: &S) -> A {
        (self.getter)(whole)
    }

    /// Set a new focused value, returning the modified whole.
    pub fn set(&self, whole: &S, value: A) -> S {
        (self.setter)(whole, value)
    }

    /// Modify the focused value with a function.
    pub fn modify(&self, whole: &S, f: impl FnOnce(A) -> A) -> S {
        let current = self.get(whole);
        self.set(whole, f(current))
    }

    /// Compose two lenses: Lens<S, A> ∘ Lens<A, B> = Lens<S, B>.
    ///
    /// If you can focus on A inside S, and B inside A,
    /// then you can focus on B inside S.
    pub fn compose<B: 'static>(&self, inner: &Lens<A, B>) -> Lens<S, B>
    where
        A: Clone,
        S: Clone,
    {
        let outer_get1 = self.getter.clone();
        let outer_get2 = self.getter.clone();
        let outer_set = self.setter.clone();
        let inner_get = inner.getter.clone();
        let inner_set = inner.setter.clone();

        Lens::new(
            move |s: &S| inner_get(&outer_get1(s)),
            move |s: &S, b: B| {
                let a = outer_get2(s);
                let new_a = inner_set(&a, b);
                outer_set(s, new_a)
            },
        )
    }
}

/// A prism: an accessor for a possible part (like enum variants).
///
/// `Prism<S, A>` can try to get an `A` from `S` (may fail),
/// and can always construct an `S` from an `A`.
#[allow(clippy::type_complexity)]
pub struct Prism<S, A> {
    preview: Box<dyn Fn(&S) -> Option<A>>,
    review: Box<dyn Fn(A) -> S>,
}

impl<S: 'static, A: 'static> Prism<S, A> {
    pub fn new(
        preview: impl Fn(&S) -> Option<A> + 'static,
        review: impl Fn(A) -> S + 'static,
    ) -> Self {
        Self {
            preview: Box::new(preview),
            review: Box::new(review),
        }
    }

    /// Try to extract the focused value.
    pub fn preview(&self, whole: &S) -> Option<A> {
        (self.preview)(whole)
    }

    /// Construct the whole from the part.
    pub fn review(&self, part: A) -> S {
        (self.review)(part)
    }
}

/// An isomorphism: a bidirectional conversion that loses no information.
///
/// `Iso<A, B>`: A → B and B → A, where roundtrip is identity.
pub struct Iso<A, B> {
    there: Box<dyn Fn(&A) -> B>,
    back: Box<dyn Fn(&B) -> A>,
}

impl<A: 'static, B: 'static> Iso<A, B> {
    pub fn new(there: impl Fn(&A) -> B + 'static, back: impl Fn(&B) -> A + 'static) -> Self {
        Self {
            there: Box::new(there),
            back: Box::new(back),
        }
    }

    pub fn there(&self, a: &A) -> B {
        (self.there)(a)
    }

    pub fn back(&self, b: &B) -> A {
        (self.back)(b)
    }

    /// Reverse the isomorphism.
    pub fn reverse(self) -> Iso<B, A> {
        Iso {
            there: self.back,
            back: self.there,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    fn age_lens() -> Lens<Person, u32> {
        Lens::new(
            |p: &Person| p.age,
            |p: &Person, a: u32| Person {
                age: a,
                ..p.clone()
            },
        )
    }

    // --- Lens laws ---

    #[test]
    fn get_set() {
        // set(s, get(s)) = s
        let alice = Person {
            name: "Alice".into(),
            age: 30,
        };
        let l = age_lens();
        let age = l.get(&alice);
        let result = l.set(&alice, age);
        assert_eq!(result, alice);
    }

    #[test]
    fn set_get() {
        // get(set(s, a)) = a
        let alice = Person {
            name: "Alice".into(),
            age: 30,
        };
        let l = age_lens();
        let result = l.set(&alice, 99);
        assert_eq!(l.get(&result), 99);
    }

    #[test]
    fn set_set() {
        // set(set(s, a), b) = set(s, b)
        let alice = Person {
            name: "Alice".into(),
            age: 30,
        };
        let l = age_lens();
        let r1 = l.set(&l.set(&alice, 40), 50);
        let r2 = l.set(&alice, 50);
        assert_eq!(r1, r2);
    }

    #[test]
    fn lens_modify() {
        let alice = Person {
            name: "Alice".into(),
            age: 30,
        };
        let older = age_lens().modify(&alice, |a| a + 1);
        assert_eq!(older.age, 31);
    }

    #[test]
    fn lens_compose() {
        #[derive(Clone, Debug, PartialEq)]
        struct Company {
            ceo: Person,
        }

        let ceo_lens = Lens::new(
            |c: &Company| c.ceo.clone(),
            |_c: &Company, p: Person| Company { ceo: p },
        );

        let ceo_age = ceo_lens.compose(&age_lens());
        let co = Company {
            ceo: Person {
                name: "Bob".into(),
                age: 50,
            },
        };

        assert_eq!(ceo_age.get(&co), 50);
        let updated = ceo_age.set(&co, 51);
        assert_eq!(updated.ceo.age, 51);
        assert_eq!(updated.ceo.name, "Bob"); // preserved
    }

    // --- Iso ---

    #[test]
    fn iso_roundtrip() {
        let celsius_fahrenheit = Iso::new(
            |c: &f64| c * 9.0 / 5.0 + 32.0,
            |f: &f64| (f - 32.0) * 5.0 / 9.0,
        );
        let c = 100.0;
        let f = celsius_fahrenheit.there(&c);
        let back = celsius_fahrenheit.back(&f);
        assert!((back - c).abs() < 1e-10);
    }

    #[test]
    fn iso_reverse() {
        let c_to_f = Iso::new(
            |c: &f64| c * 9.0 / 5.0 + 32.0,
            |f: &f64| (f - 32.0) * 5.0 / 9.0,
        );
        let f_to_c = c_to_f.reverse();
        assert!((f_to_c.there(&212.0) - 100.0).abs() < 1e-10);
    }

    // --- Prism ---

    #[test]
    fn prism_enum_variant() {
        #[derive(Clone, Debug, PartialEq)]
        enum Shape {
            Circle(f64),
            Rect(f64, f64),
        }

        let circle_prism = Prism::new(
            |s: &Shape| match s {
                Shape::Circle(r) => Some(*r),
                _ => None,
            },
            Shape::Circle,
        );

        assert_eq!(circle_prism.preview(&Shape::Circle(5.0)), Some(5.0));
        assert_eq!(circle_prism.preview(&Shape::Rect(3.0, 4.0)), None);

        // Rect prism — exercises the other variant
        let rect_prism = Prism::new(
            |s: &Shape| match s {
                Shape::Rect(w, h) => Some((*w, *h)),
                _ => None,
            },
            |(w, h)| Shape::Rect(w, h),
        );
        assert_eq!(rect_prism.preview(&Shape::Rect(3.0, 4.0)), Some((3.0, 4.0)));
        assert_eq!(rect_prism.review((5.0, 6.0)), Shape::Rect(5.0, 6.0));
    }

    // --- Practical: ontology taxonomy view ---

    #[test]
    fn ontology_taxonomy_lens() {
        #[derive(Clone, Debug, PartialEq)]
        struct SimpleOntology {
            name: String,
            taxonomy_edges: Vec<(String, String)>,
        }

        let taxonomy_lens = Lens::new(
            |ont: &SimpleOntology| ont.taxonomy_edges.clone(),
            |ont: &SimpleOntology, edges: Vec<(String, String)>| SimpleOntology {
                taxonomy_edges: edges,
                ..ont.clone()
            },
        );

        let bio = SimpleOntology {
            name: "Biology".into(),
            taxonomy_edges: vec![("Dog".into(), "Mammal".into())],
        };

        assert_eq!(taxonomy_lens.get(&bio).len(), 1);

        let updated = taxonomy_lens.modify(&bio, |mut e| {
            e.push(("Cat".into(), "Mammal".into()));
            e
        });
        assert_eq!(updated.taxonomy_edges.len(), 2);
        assert_eq!(updated.name, "Biology"); // unchanged
    }
}
