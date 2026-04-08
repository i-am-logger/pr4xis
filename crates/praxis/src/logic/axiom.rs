/// An axiom — a statement that must hold unconditionally.
///
/// Axioms are foundational truths about a domain. `holds()` verifies
/// the system is consistent with the axiom — the system cannot lie.
///
/// Used by both category-level structural checks (e.g. "no dead states")
/// and domain-level invariants (e.g. "energy is conserved").
pub trait Axiom {
    /// Human-readable description of this axiom.
    fn description(&self) -> &str;

    /// Verify this axiom holds.
    fn holds(&self) -> bool;
}
