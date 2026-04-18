use crate::ontology::meta::{Citation, ModulePath, OntologyName};

/// Lemon-style lexical metadata for an axiom — its identity, citation,
/// and module path. Matches the `OntologyMeta` shape so the lexicon
/// treats ontologies and axioms uniformly (issue #148: "every structural
/// entity announces itself lexically").
///
/// Construct via the `ontology!` macro's `axioms:` clause, which emits
/// these values from compile-time constants. New axioms outside that
/// clause fall back to the `Axiom` trait's default `meta()` (derived
/// from `type_name`).
#[derive(Debug, Clone)]
pub struct AxiomMeta {
    pub name: OntologyName,
    pub citation: Citation,
    pub module_path: ModulePath,
}

/// An axiom — a statement that must hold unconditionally.
///
/// Axioms are foundational truths about a domain. `holds()` verifies
/// the system is consistent with the axiom — the system cannot lie.
///
/// Used by both category-level structural checks (e.g. "no dead states")
/// and domain-level invariants (e.g. "energy is conserved").
///
/// Every axiom announces itself via `meta()` — its name, citation, and
/// module path, carried in the same Lemon-style wrappers as ontologies.
/// `description()` remains as an English fallback until the lexicon
/// resolves `meta().name` into per-language labels.
pub trait Axiom {
    /// English fallback — will be replaced by Lemon lexicon lookup of `meta().name`.
    fn description(&self) -> &str;

    /// Verify this axiom holds.
    fn holds(&self) -> bool;

    /// Structured metadata — name, citation, module path.
    ///
    /// Default implementation derives a sensible identity from
    /// `std::any::type_name` so every existing axiom keeps compiling
    /// without migration. Axioms declared inside `ontology!`'s
    /// `axioms:` clause override this with the literature citation
    /// captured at the declaration site.
    fn meta(&self) -> AxiomMeta {
        AxiomMeta {
            name: OntologyName::new(std::any::type_name::<Self>().to_string()),
            citation: Citation::EMPTY,
            module_path: ModulePath::new(module_path!().to_string()),
        }
    }
}
