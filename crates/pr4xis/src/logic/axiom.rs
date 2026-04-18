use crate::ontology::meta::{Citation, ModulePath, OntologyName};

/// Lemon-style lexical metadata for an axiom — its identity, citation,
/// and module path. Matches the `OntologyMeta` shape so the lexicon
/// treats ontologies and axioms uniformly (issue #148: "every structural
/// entity announces itself lexically").
///
/// Construct via the `ontology!` macro's `axioms:` clause, which emits
/// these values from compile-time constants, or via the
/// [`axiom_meta!`](crate::axiom_meta!) helper inline inside a manual
/// `impl Axiom for T` block.
#[derive(Debug, Clone)]
pub struct AxiomMeta {
    pub name: OntologyName,
    /// English-language label / description of the axiom (Lemon Form).
    /// For most axioms this is the same string returned by `description()`.
    pub description: crate::ontology::meta::Label,
    pub citation: Citation,
    pub module_path: ModulePath,
}

/// Helper: write the `meta()` method for a hand-written `impl Axiom`
/// with a literature citation in one line. Ensures every axiom announces
/// itself without boilerplate.
///
/// # Example
///
/// ```ignore
/// impl Axiom for MyAxiom {
///     fn description(&self) -> &str { "..." }
///     fn holds(&self) -> bool { ... }
///     pr4xis::axiom_meta!("MyAxiom", "Smith (1999) Journal of X");
/// }
/// ```
#[macro_export]
macro_rules! axiom_meta {
    // Three-argument form — name, description (English label), citation.
    ($name:literal, $description:literal, $citation:literal) => {
        fn meta(&self) -> $crate::logic::axiom::AxiomMeta {
            $crate::logic::axiom::AxiomMeta {
                name: $crate::ontology::meta::OntologyName::new_static($name),
                description: $crate::ontology::meta::Label::new_static($description),
                citation: $crate::ontology::meta::Citation::parse_static($citation),
                module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
            }
        }
    };
    // Two-argument form — name + citation, description defaults to name.
    // Convenience for axioms where the struct name is itself the English label.
    ($name:literal, $citation:literal) => {
        fn meta(&self) -> $crate::logic::axiom::AxiomMeta {
            $crate::logic::axiom::AxiomMeta {
                name: $crate::ontology::meta::OntologyName::new_static($name),
                description: $crate::ontology::meta::Label::new_static($name),
                citation: $crate::ontology::meta::Citation::parse_static($citation),
                module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
            }
        }
    };
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
    /// Associated function (no `&self`) so it can be registered into the
    /// global AXIOMS distributed slice without constructing an instance —
    /// works for unit structs and structs-with-fields alike.
    ///
    /// The default is an **honest placeholder** using `std::any::type_name`
    /// and an empty citation — "this axiom hasn't declared its literature
    /// citation yet"; consumers can detect and flag via `citation.is_empty()`.
    ///
    /// Axioms declared via `ontology!`'s `axioms:` clause or with the
    /// [`axiom_meta!`](crate::axiom_meta!) helper inline override the
    /// default with the actual literature reference.
    fn meta(&self) -> AxiomMeta {
        let tn = std::any::type_name::<Self>().to_string();
        AxiomMeta {
            name: OntologyName::new(tn.clone()),
            description: crate::ontology::meta::Label::new(self.description().to_string()),
            citation: Citation::EMPTY,
            module_path: ModulePath::new(module_path!().to_string()),
        }
    }
}
