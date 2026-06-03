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
//! [`crate::ontology::Ontology::axioms`] via [`category_law_axioms`], or verify them
//! directly:
//!
//! ```text
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

use super::adjunction::Adjunction;
use super::arrow::Arrow;
use super::category::Category;
use super::entity::FinitelyGenerated;
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
    // The identity law is checked by enumerating every object — a closed-world
    // (finite) verification.
    C::Object: FinitelyGenerated,
{
    fn verify(&self) -> Verdict {
        for obj in <C::Object as FinitelyGenerated>::variants() {
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
/// for splicing into [`crate::ontology::Ontology::axioms`].
pub fn category_law_axioms<C>() -> Vec<Box<dyn Axiom>>
where
    C: Category + 'static,
    C::Morphism: PartialEq + 'static,
    // IdentityLaw verifies by enumerating objects (closed-world).
    C::Object: FinitelyGenerated,
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
    C::Object: FinitelyGenerated,
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
    // Checked by enumerating every source object — a closed-world (finite)
    // verification over the functor's source category.
    <F::Source as Category>::Object: FinitelyGenerated,
{
    fn verify(&self) -> Verdict {
        for obj in <<F::Source as Category>::Object as FinitelyGenerated>::variants() {
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
    // FunctorIdentityLaw verifies by enumerating source objects (closed-world).
    <F::Source as Category>::Object: FinitelyGenerated,
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
    <F::Source as Category>::Object: FinitelyGenerated,
{
    for law in functor_law_axioms::<F>() {
        if let Err(c) = law.verify() {
            panic!("functor law failed: {}", c.meta().name.as_str());
        }
    }
}

// ---------------------------------------------------------------------------
// Full + faithful functor laws (Mac Lane 1971 CWM Ch. I §4)
// ---------------------------------------------------------------------------

/// Mac Lane (1971) CWM Ch. I §4: a functor is *faithful* when
/// `map_morphism` is injective on each hom-set — distinct source
/// morphisms `f, g : A → B` keep distinct images `F(f) ≠ F(g)`.
pub struct FunctorFaithfulLaw<F: Functor> {
    _marker: PhantomData<F>,
}

impl<F: Functor> FunctorFaithfulLaw<F> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<F: Functor> Default for FunctorFaithfulLaw<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F> Axiom for FunctorFaithfulLaw<F>
where
    F: Functor + 'static,
    <F::Source as Category>::Morphism: PartialEq + 'static,
    <F::Target as Category>::Morphism: PartialEq + 'static,
{
    fn verify(&self) -> Verdict {
        let ms = F::Source::morphisms();
        for (i, f) in ms.iter().enumerate() {
            for g in ms.iter().skip(i + 1) {
                // distinct morphisms in the same hom-set …
                if f.source() == g.source()
                    && f.target() == g.target()
                    && f != g
                    // … must not collapse to one image under F.
                    && F::map_morphism(f) == F::map_morphism(g)
                {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    fn name(&self) -> OntologyName {
        OntologyName::new_static("FunctorFaithfulLaw")
    }

    fn description(&self) -> Label {
        Label::new_static("map_morphism is injective on each hom-set (faithful)")
    }

    fn citation(&self) -> Citation {
        Citation::parse_static("Mac Lane (1971) Categories for the Working Mathematician Ch. I §4")
    }
}

/// Mac Lane (1971) CWM Ch. I §4: a functor is *full onto its image* when
/// every target morphism between image objects `F(A) → F(B)` is the image
/// of some source morphism in `Hom(A, B)`. Enumerated over the full
/// `Target::morphisms()` (including macro-emitted transitive-closure
/// edges), so a later edge introduced between two image objects cannot
/// silently break fullness. With [`FunctorFaithfulLaw`] this witnesses a
/// full-and-faithful embedding — an isomorphism onto the full subcategory
/// on the image.
pub struct FunctorFullOnImageLaw<F: Functor> {
    _marker: PhantomData<F>,
}

impl<F: Functor> FunctorFullOnImageLaw<F> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<F: Functor> Default for FunctorFullOnImageLaw<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F> Axiom for FunctorFullOnImageLaw<F>
where
    F: Functor + 'static,
    <F::Source as Category>::Morphism: PartialEq + 'static,
    <F::Target as Category>::Morphism: PartialEq + 'static,
    // Enumerates source objects to range over the hom-sets being checked for
    // fullness-on-image — a closed-world (finite) verification.
    <F::Source as Category>::Object: FinitelyGenerated,
{
    fn verify(&self) -> Verdict {
        let src_objs = <<F::Source as Category>::Object as FinitelyGenerated>::variants();
        let src_ms = F::Source::morphisms();
        let tgt_ms = F::Target::morphisms();
        for a in &src_objs {
            let fa = F::map_object(a);
            for b in &src_objs {
                let fb = F::map_object(b);
                for t in &tgt_ms {
                    if t.source() != fa || t.target() != fb {
                        continue;
                    }
                    // some source morphism in Hom(a, b) must map onto t.
                    let hit = src_ms
                        .iter()
                        .any(|f| f.source() == *a && f.target() == *b && F::map_morphism(f) == *t);
                    if !hit {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    fn name(&self) -> OntologyName {
        OntologyName::new_static("FunctorFullOnImageLaw")
    }

    fn description(&self) -> Label {
        Label::new_static(
            "every target morphism between image objects is the image of a source morphism (full onto image)",
        )
    }

    fn citation(&self) -> Citation {
        Citation::parse_static("Mac Lane (1971) Categories for the Working Mathematician Ch. I §4")
    }
}

/// The full-and-faithful embedding laws as `Box<dyn Axiom>` — combine with
/// [`functor_law_axioms`] to machine-prove `FunctorKind::FullyFaithful`
/// (identity + composition + faithful + full-onto-image).
pub fn fully_faithful_law_axioms<F>() -> Vec<Box<dyn Axiom>>
where
    F: Functor + 'static,
    <F::Source as Category>::Morphism: PartialEq + 'static,
    <F::Target as Category>::Morphism: PartialEq + 'static,
    // FunctorFullOnImageLaw verifies by enumerating source objects (closed-world);
    // FunctorFaithfulLaw ranges over morphisms() and needs no enumeration.
    <F::Source as Category>::Object: FinitelyGenerated,
{
    vec![
        Box::new(FunctorFaithfulLaw::<F>::new()),
        Box::new(FunctorFullOnImageLaw::<F>::new()),
    ]
}

// ---------------------------------------------------------------------------
// Adjunction triangle identities (Mac Lane 1971 CWM Ch. IV §1)
// ---------------------------------------------------------------------------

/// Mac Lane (1971) CWM Ch. IV §1: the *triangle identities* an adjunction
/// `F ⊣ G` (unit η, counit ε) must satisfy —
/// `ε_{F(A)} ∘ F(η_A) = id_{F(A)}` and `G(ε_B) ∘ η_{G(B)} = id_{G(B)}` —
/// checked by enumeration over the finite object sets of both categories.
pub struct AdjunctionTriangleLaw<A: Adjunction> {
    _marker: PhantomData<A>,
}

impl<A: Adjunction> AdjunctionTriangleLaw<A> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<A: Adjunction> Default for AdjunctionTriangleLaw<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Axiom for AdjunctionTriangleLaw<A>
where
    A: Adjunction + 'static,
    <<A::Left as Functor>::Source as Category>::Morphism: PartialEq + 'static,
    <<A::Left as Functor>::Target as Category>::Morphism: PartialEq + 'static,
    // Both triangle identities are checked by enumerating every object — the
    // left one ranges over C = F's source, the right over D = F's target — so
    // both object concepts must be finitely generated (closed-world).
    <<A::Left as Functor>::Source as Category>::Object: FinitelyGenerated,
    <<A::Left as Functor>::Target as Category>::Object: FinitelyGenerated,
{
    fn verify(&self) -> Verdict {
        // Left triangle: ε_{F(A)} ∘ F(η_A) = id_{F(A)}, for every A in C.
        for a in
            <<<A::Left as Functor>::Source as Category>::Object as FinitelyGenerated>::variants()
        {
            let eta_a = A::unit(&a); // C-morphism A → G(F(A))
            let f_eta = <A::Left as Functor>::map_morphism(&eta_a); // D: F(A) → F(G(F(A)))
            let fa = <A::Left as Functor>::map_object(&a); // D-object F(A)
            let eps_fa = A::counit(&fa); // D: F(G(F(A))) → F(A)
            let composed = <<A::Left as Functor>::Target as Category>::compose(&f_eta, &eps_fa);
            let id_fa = <<A::Left as Functor>::Target as Category>::identity(&fa);
            if composed.as_ref() != Some(&id_fa) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        // Right triangle: G(ε_B) ∘ η_{G(B)} = id_{G(B)}, for every B in D.
        for b in
            <<<A::Left as Functor>::Target as Category>::Object as FinitelyGenerated>::variants()
        {
            let gb = <A::Right as Functor>::map_object(&b); // C-object G(B)
            let eta_gb = A::unit(&gb); // C: G(B) → G(F(G(B)))
            let eps_b = A::counit(&b); // D: F(G(B)) → B
            let g_eps_b = <A::Right as Functor>::map_morphism(&eps_b); // C: G(F(G(B))) → G(B)
            let composed = <<A::Left as Functor>::Source as Category>::compose(&eta_gb, &g_eps_b);
            let id_gb = <<A::Left as Functor>::Source as Category>::identity(&gb);
            if composed.as_ref() != Some(&id_gb) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    fn name(&self) -> OntologyName {
        OntologyName::new_static("AdjunctionTriangleLaw")
    }

    fn description(&self) -> Label {
        Label::new_static("ε_{F(A)} ∘ F(η_A) = id_{F(A)} and G(ε_B) ∘ η_{G(B)} = id_{G(B)}")
    }

    fn citation(&self) -> Citation {
        Citation::parse_static("Mac Lane (1971) Categories for the Working Mathematician Ch. IV §1")
    }
}

/// The adjunction triangle identities as `Box<dyn Axiom>` instances.
pub fn adjunction_law_axioms<A>() -> Vec<Box<dyn Axiom>>
where
    A: Adjunction + 'static,
    <<A::Left as Functor>::Source as Category>::Morphism: PartialEq + 'static,
    <<A::Left as Functor>::Target as Category>::Morphism: PartialEq + 'static,
    // AdjunctionTriangleLaw enumerates both categories' objects (closed-world).
    <<A::Left as Functor>::Source as Category>::Object: FinitelyGenerated,
    <<A::Left as Functor>::Target as Category>::Object: FinitelyGenerated,
{
    vec![Box::new(AdjunctionTriangleLaw::<A>::new())]
}

/// Test convenience: verify the triangle identities for `A`, panicking
/// with the counterexample's meta name on failure.
pub fn assert_adjunction_laws<A>()
where
    A: Adjunction + 'static,
    <<A::Left as Functor>::Source as Category>::Morphism: PartialEq + 'static,
    <<A::Left as Functor>::Target as Category>::Morphism: PartialEq + 'static,
    <<A::Left as Functor>::Source as Category>::Object: FinitelyGenerated,
    <<A::Left as Functor>::Target as Category>::Object: FinitelyGenerated,
{
    for law in adjunction_law_axioms::<A>() {
        if let Err(c) = law.verify() {
            panic!("adjunction law failed: {}", c.meta().name.as_str());
        }
    }
}
