use praxis_category::Category;

/// An axiom that an ontology must satisfy.
///
/// Axioms go beyond category laws to express domain-specific invariants.
/// They define what makes an ontology valid for YOUR domain.
///
/// The category is available via `C::Object::variants()` and `C::morphisms()`
/// so axioms can inspect the full structure.
///
/// Examples:
/// - "Every mode must have an exit transition"
/// - "Hardware can only receive commands within its capabilities"
/// - "A theme change must propagate to all surfaces"
pub trait Axiom<C: Category> {
    /// Human-readable description of this axiom.
    fn description(&self) -> &str;

    /// Check whether this axiom holds for the category.
    fn holds(&self) -> bool;
}
