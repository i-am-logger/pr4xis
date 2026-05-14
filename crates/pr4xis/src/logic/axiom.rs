use crate::logic::proof::Verdict;
use crate::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

/// An axiom — a statement that must hold unconditionally.
///
/// # Verification returns a typed Proof / Counterexample, never a boolean (#160)
///
/// Under Martin-Löf (1984), a proof IS a term inhabiting its claim-type;
/// a refutation is a term of type `P → ⊥` (Curry & Feys 1958). These are
/// structurally distinct kinds of evidence — pr4xis core never collapses
/// them into a bool. [`verify`] returns [`Verdict`] = `Result<Box<dyn Proof>,
/// Box<dyn Counterexample>>`. Pattern-match the result; do not reach for
/// a boolean shortcut.
///
/// See `feedback_core_no_bool_api` — core's public API must never expose
/// `bool`-returning methods or `bool`-accepting helpers.
///
/// # Citation is required (#167)
///
/// Every axiom declares its literature citation by implementing
/// [`citation`]. There is no default. An axiom without a published
/// source is not an axiom — per `feedback_literature_or_remove`, if
/// you can't cite it, collapse it into a parent concept or remove it.
/// Absence at the type level is a compile error, not a runtime `None`.
///
/// Description and name have defaults derived from `type_name::<Self>()`
/// (an honest placeholder, not a back-compat shim) and may be overridden
/// when the Rust identifier differs from the human-readable axiom label.
///
/// Example:
///
/// ```ignore
/// impl Axiom for MyAxiom {
///     fn verify(&self) -> Verdict { ... }
///     fn citation(&self) -> Citation {
///         Citation::parse_static("Smith (1999) J. Foo")
///     }
/// }
/// ```
///
/// The [`axiom_meta!`] helper macro emits the three override methods
/// (`name`, `description`, `citation`) in one line for the common case
/// where all three are string literals.
///
/// Literature:
/// - Martin-Löf (1984) *Intuitionistic Type Theory*
/// - Curry & Feys (1958) *Combinatory Logic*
/// - Prawitz (1965) *Natural Deduction*
/// - Joyal-Street-Verity (1996) *Traced Monoidal Categories*
/// - Lambek (1968) "Deductive Systems and Categories"
pub trait Axiom {
    /// Verify this axiom and return a typed [`Verdict`] — `Ok` carrying
    /// a Proof witness, or `Err` carrying a Counterexample refutation.
    fn verify(&self) -> Verdict;

    /// Literature citation for this axiom. **Required.** No default —
    /// every axiom must trace to published work.
    fn citation(&self) -> Citation;

    /// Rust/display identifier for this axiom. Defaults to
    /// `type_name::<Self>()`. Override when the type name and the
    /// curated axiom label diverge (e.g. generic types rendered with
    /// their kind parameter: `"NoCyclesOnKind[Subsumption]"`).
    fn name(&self) -> OntologyName {
        OntologyName::new(core::any::type_name::<Self>().to_string())
    }

    /// Human-readable one-line label. Defaults to the type name.
    fn description(&self) -> Label {
        Label::new(core::any::type_name::<Self>().to_string())
    }

    /// Module path where this axiom lives. Defaults to the call-site
    /// module — override when axioms are constructed outside their
    /// home module and you need precise provenance.
    fn module_path(&self) -> ModulePath {
        ModulePath::new_static(module_path!())
    }

    /// Structured metadata — composed from [`name`], [`description`],
    /// [`citation`], and [`module_path`]. Typically not overridden.
    fn meta(&self) -> Provenance {
        Provenance {
            name: self.name(),
            description: self.description(),
            citation: self.citation(),
            module_path: self.module_path(),
        }
    }
}
