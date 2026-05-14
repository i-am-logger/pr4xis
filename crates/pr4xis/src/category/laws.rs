//! Category and functor laws as first-class `Axiom` impls.
//!
//! # Laws are axioms (#168 / #169)
//!
//! Under Mac Lane (1971) CWM Ch. I §1, a category is defined by its
//! laws — identity and associativity hold as equations between
//! morphisms. In the computing setting (Barr & Wells 1999 CTCS §4
//! *sketches*; Spivak 2012 FDM §§2–3 *schema categories with path
//! equivalences*) those laws and any domain-specific equations are
//! verified uniformly — they are the same kind of thing.
//!
//! pr4xis follows Barr-Wells/Spivak: every law is an [`Axiom`] and
//! [`Axiom::verify`] returns a typed [`Verdict`] (proof or
//! counterexample, per Martin-Löf 1984). The prior `validate.rs`
//! module returned `Result<(), String>` — strings aren't proofs under
//! Martin-Löf, and the separate module privileged laws over axioms
//! with no ontological basis.
//!
//! # Use
//!
//! To verify a category's laws, compose them into an ontology's
//! [`Ontology::axioms`] via [`category_law_axioms`], or verify them
//! directly:
//!
//! ```ignore
//! for law in category_law_axioms::<FooCategory>() {
//!     law.verify().unwrap_or_else(|c| panic!("{}", c.meta().name.as_str()));
//! }
//! ```
//!
//! Functor laws ([`functor_law_axioms`]) work the same way.
//!
//! Literature:
//! - Mac Lane (1971) *Categories for the Working Mathematician* Ch. I §1
//!   (category laws), Ch. II §1 (functor laws)
//! - Barr & Wells (1999) *Category Theory for Computing Science* §4
//!   (sketches: laws and axioms uniform)
//! - Spivak (2012) *Functorial Data Model* §§2–3 (schema categories
//!   with path equivalences)
//! - Martin-Löf (1984) *Intuitionistic Type Theory* (typed proofs /
//!   counterexamples)

use std::marker::PhantomData;

use super::arrow::Arrow;
use super::category::Category;
use super::entity::Concept;
use super::functor::Functor;
use crate::logic::axiom::Axiom;
use crate::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use crate::ontology::meta::{Citation, Label, OntologyName};

// ---------------------------------------------------------------------------
// Category laws
// ---------------------------------------------------------------------------

/// Mac Lane (1971) CWM Ch. I §1: closure — whenever `compose(f, g)`
/// yields `Some(h)`, `h` must be in `morphisms()`.
///
/// Per OBO-RO partial composition (#166), `compose` may legitimately
/// return `None` for heterogeneous pairs with no declared rule. The
/// law is only asserted on the positive branch.
pub struct ClosureLaw<C: Category> {
    _marker: PhantomData<C>,
}

impl<C: Category> ClosureLaw<C> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<C: Category> Default for ClosureLaw<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Axiom for ClosureLaw<C>
where
    C: Category + 'static,
    C::Morphism: PartialEq + 'static,
{
    fn verify(&self) -> Verdict {
        let ms = C::morphisms();
        for f in &ms {
            for g in &ms {
                if f.target() != g.source() {
                    continue;
                }
                if let Some(h) = C::compose(f, g)
                    && !ms.contains(&h)
                {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    fn name(&self) -> OntologyName {
        OntologyName::new_static("ClosureLaw")
    }

    fn description(&self) -> Label {
        Label::new_static("compose(f, g) = Some(h) implies h ∈ morphisms()")
    }

    fn citation(&self) -> Citation {
        Citation::parse_static(
            "Mac Lane (1971) Categories for the Working Mathematician Ch. I §1; Barr & Wells (1999) CTCS §4 sketches",
        )
    }
}

/// Mac Lane (1971) CWM Ch. I §1: identity — `id_B ∘ f = f = f ∘ id_A`
/// for every morphism `f : A → B`.
pub struct IdentityLaw<C: Category> {
    _marker: PhantomData<C>,
}

impl<C: Category> IdentityLaw<C> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<C: Category> Default for IdentityLaw<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Axiom for IdentityLaw<C>
where
    C: Category + 'static,
    C::Morphism: PartialEq + 'static,
{
    fn verify(&self) -> Verdict {
        for obj in <C::Object as Concept>::variants() {
            let id = C::identity(&obj);
            for m in C::morphisms_from(&obj) {
                let left = C::compose(&id, &m);
                if left.as_ref() != Some(&m) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
            for m in C::morphisms_to(&obj) {
                let right = C::compose(&m, &id);
                if right.as_ref() != Some(&m) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    fn name(&self) -> OntologyName {
        OntologyName::new_static("IdentityLaw")
    }

    fn description(&self) -> Label {
        Label::new_static("id_B ∘ f = f = f ∘ id_A for every morphism f: A → B")
    }

    fn citation(&self) -> Citation {
        Citation::parse_static("Mac Lane (1971) Categories for the Working Mathematician Ch. I §1")
    }
}

/// Mac Lane (1971) CWM Ch. I §1: associativity — for composable
/// triples `f, g, h`, `(h ∘ g) ∘ f = h ∘ (g ∘ f)`.
pub struct AssociativityLaw<C: Category> {
    _marker: PhantomData<C>,
}

impl<C: Category> AssociativityLaw<C> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<C: Category> Default for AssociativityLaw<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Axiom for AssociativityLaw<C>
where
    C: Category + 'static,
    C::Morphism: PartialEq + 'static,
{
    fn verify(&self) -> Verdict {
        let ms = C::morphisms();
        for f in &ms {
            for g in &ms {
                if f.target() != g.source() {
                    continue;
                }
                for h in &ms {
                    if g.target() != h.source() {
                        continue;
                    }
                    let fg = C::compose(f, g);
                    let gh = C::compose(g, h);
                    let left = fg.as_ref().and_then(|fg| C::compose(fg, h));
                    let right = gh.as_ref().and_then(|gh| C::compose(f, gh));
                    if left != right {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    fn name(&self) -> OntologyName {
        OntologyName::new_static("AssociativityLaw")
    }

    fn description(&self) -> Label {
        Label::new_static("(h ∘ g) ∘ f = h ∘ (g ∘ f) for composable triples")
    }

    fn citation(&self) -> Citation {
        Citation::parse_static("Mac Lane (1971) Categories for the Working Mathematician Ch. I §1")
    }
}

/// The three category laws as `Box<dyn Axiom>` instances, suitable
/// for splicing into [`Ontology::axioms`].
pub fn category_law_axioms<C>() -> Vec<Box<dyn Axiom>>
where
    C: Category + 'static,
    C::Morphism: PartialEq + 'static,
{
    vec![
        Box::new(ClosureLaw::<C>::new()),
        Box::new(IdentityLaw::<C>::new()),
        Box::new(AssociativityLaw::<C>::new()),
    ]
}

/// Test convenience: verify every category law for `C`, panicking
/// with the counterexample's meta name on failure. Pattern-matches
/// the typed [`Verdict`] under the hood — core does not expose any
/// `bool`-returning shortcut.
pub fn assert_category_laws<C>()
where
    C: Category + 'static,
    C::Morphism: PartialEq + 'static,
{
    for law in category_law_axioms::<C>() {
        if let Err(c) = law.verify() {
            panic!("category law failed: {}", c.meta().name.as_str());
        }
    }
}

// ---------------------------------------------------------------------------
// Functor laws
// ---------------------------------------------------------------------------

/// Mac Lane (1971) CWM Ch. II §1: a functor preserves identities —
/// `F(id_A) = id_{F(A)}` for every source object A.
pub struct FunctorIdentityLaw<F: Functor> {
    _marker: PhantomData<F>,
}

impl<F: Functor> FunctorIdentityLaw<F> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<F: Functor> Default for FunctorIdentityLaw<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F> Axiom for FunctorIdentityLaw<F>
where
    F: Functor + 'static,
    <F::Target as Category>::Morphism: PartialEq + 'static,
{
    fn verify(&self) -> Verdict {
        for obj in <<F::Source as Category>::Object as Concept>::variants() {
            let id_source = F::Source::identity(&obj);
            let mapped_id = F::map_morphism(&id_source);
            let id_target = F::Target::identity(&F::map_object(&obj));
            if mapped_id != id_target {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    fn name(&self) -> OntologyName {
        OntologyName::new_static("FunctorIdentityLaw")
    }

    fn description(&self) -> Label {
        Label::new_static("F(id_A) = id_{F(A)} for every source object A")
    }

    fn citation(&self) -> Citation {
        Citation::parse_static("Mac Lane (1971) Categories for the Working Mathematician Ch. II §1")
    }
}

/// Mac Lane (1971) CWM Ch. II §1: a functor preserves composition —
/// `F(g ∘ f) = F(g) ∘ F(f)` for composable pairs.
pub struct FunctorCompositionLaw<F: Functor> {
    _marker: PhantomData<F>,
}

impl<F: Functor> FunctorCompositionLaw<F> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<F: Functor> Default for FunctorCompositionLaw<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F> Axiom for FunctorCompositionLaw<F>
where
    F: Functor + 'static,
    <F::Source as Category>::Morphism: PartialEq + 'static,
    <F::Target as Category>::Morphism: PartialEq + 'static,
{
    fn verify(&self) -> Verdict {
        let ms = F::Source::morphisms();
        for f in &ms {
            for g in &ms {
                if f.target() != g.source() {
                    continue;
                }
                if let Some(gf) = F::Source::compose(f, g) {
                    let f_mapped = F::map_morphism(&gf);
                    let composed = F::Target::compose(&F::map_morphism(f), &F::map_morphism(g));
                    if composed.as_ref() != Some(&f_mapped) {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    fn name(&self) -> OntologyName {
        OntologyName::new_static("FunctorCompositionLaw")
    }

    fn description(&self) -> Label {
        Label::new_static("F(g ∘ f) = F(g) ∘ F(f) for composable pairs")
    }

    fn citation(&self) -> Citation {
        Citation::parse_static("Mac Lane (1971) Categories for the Working Mathematician Ch. II §1")
    }
}

/// The two functor laws as `Box<dyn Axiom>` instances.
pub fn functor_law_axioms<F>() -> Vec<Box<dyn Axiom>>
where
    F: Functor + 'static,
    <F::Source as Category>::Morphism: PartialEq + 'static,
    <F::Target as Category>::Morphism: PartialEq + 'static,
{
    vec![
        Box::new(FunctorIdentityLaw::<F>::new()),
        Box::new(FunctorCompositionLaw::<F>::new()),
    ]
}

/// Test convenience: verify every functor law for `F`, panicking with
/// the counterexample's meta name on failure.
pub fn assert_functor_laws<F>()
where
    F: Functor + 'static,
    <F::Source as Category>::Morphism: PartialEq + 'static,
    <F::Target as Category>::Morphism: PartialEq + 'static,
{
    for law in functor_law_axioms::<F>() {
        if let Err(c) = law.verify() {
            panic!("functor law failed: {}", c.meta().name.as_str());
        }
    }
}
