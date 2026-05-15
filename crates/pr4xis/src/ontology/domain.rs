use crate::category::Category;
use crate::category::laws::category_law_axioms;
use crate::logic::Axiom;
use crate::logic::proof::{Verdict, combine_verdicts};
use crate::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::property::Quality;

/// An ontology: what exists, how things relate, and what rules govern them.
///
/// An `Ontology` ties together:
/// - A category (individuals + relations with composition)
/// - Qualities on individuals (attributes, capabilities)
/// - Axioms — all claims this ontology asserts
///
/// # One method for axioms (#168)
///
/// `Ontology` exposes a single required [`Ontology::axioms`] method. The prior
/// `structural_axioms()` / `domain_axioms()` split was coding convenience
/// — no source literature (Gruber 1993; Guarino 2009; Spivak 2012;
/// Smith et al. 2005 OBO-RO) distinguishes them as categories of
/// axiom. At the corpus level every axiom is a claim that must hold;
/// provenance (macro-generated structural property vs hand-authored
/// domain claim) is metadata, not kind.
///
/// Implementers combine whatever sources they use. The canonical
/// pattern uses [`crate::ontology::reasoning::structural_axioms_for`]
/// to inherit structural axioms from the Relations catalog (Smith et
/// al. 2005 OBO-RO), then appends hand-written domain axioms:
///
/// ```text
/// impl Ontology for FooOntology {
///     fn axioms() -> Vec<Box<dyn Axiom>> {
///         let mut all = structural_axioms_for::<Self::Cat>();
///         all.push(Box::new(MyDomainAxiom));
///         all
///     }
/// }
/// ```
///
/// Structural axioms are *inherited* — an ontology using the kind
/// `Subsumption` picks up `NoCyclesOnKind` and `AntisymmetricOnKind`
/// without re-emitting them. See the `catalog` module for the full
/// rule table.
///
/// The ontology validates itself — if it compiles and passes
/// validation, the domain model is mathematically sound.
pub trait Ontology {
    /// The underlying category (individuals + relations).
    type Cat: Category;

    /// Qualities that individuals can have.
    type Qual: Quality<Individual = <Self::Cat as Category>::Object>;

    /// All axioms this ontology asserts — merged from whatever sources
    /// (macro-generated structural + hand-authored domain) the
    /// implementer uses.
    fn axioms() -> Vec<Box<dyn Axiom>>;

    /// Validate the entire ontology: category laws + all axioms.
    ///
    /// Returns a typed [`Verdict`] (#160 / #162):
    /// - `Ok(Box<dyn Proof>)` — every category law and axiom discharges
    /// - `Err(Box<dyn Counterexample>)` — one or more sub-claims fail; the
    ///   counterexample carries the passing sub-proofs for context and the
    ///   specific failing sub-counterexamples
    ///
    /// Pattern-match the return; core does not expose any `is_valid() -> bool`
    /// convenience (see `feedback_core_no_bool_api`).
    fn validate() -> Verdict
    where
        Self::Cat: 'static,
        <Self::Cat as Category>::Morphism: PartialEq + 'static,
    {
        let mut subverdicts: Vec<Verdict> = Vec::new();

        for law in category_law_axioms::<Self::Cat>() {
            subverdicts.push(law.verify());
        }
        for axiom in Self::axioms() {
            subverdicts.push(axiom.verify());
        }

        let meta = Provenance {
            name: OntologyName::new_static("OntologyValidation"),
            description: Label::new_static("aggregate validation: category laws + ontology axioms"),
            citation: Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician; \
                 Barr & Wells (1999) CTCS §4 sketches; \
                 Spivak (2012) Functorial Data Model §§2–3",
            ),
            module_path: ModulePath::new_static(module_path!()),
        };
        combine_verdicts(meta, subverdicts)
    }
}
