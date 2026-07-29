//! Conditional rule application — a fully-known, fully-cited statutory
//! [`LegalTerm`] applied to
//! one asker's turn, modeled as a Frame (Minsky 1974) over Frame Elements
//! (Fillmore 1976/1982): `LegalTerm.required_evidence` (a
//! `Vec<EvidenceRequirement>`) IS the frame-element set — no parallel
//! "Rule"/"Criterion" vocabulary is invented here. This module supplies only
//! what `LegalTerm`/`EvidenceRequirement` do not yet carry: per-turn SLOT
//! STATE (Bobrow, Kaplan, Norman, Thompson & Winograd 1977, "GUS, A
//! Frame-Driven Dialog System", *Artificial Intelligence* 8(2):155-173 — ask
//! for the unfilled slot, never guess) and the Hart (1961, *The Concept of
//! Law*, §V-VI) primary/secondary classification of the term's source.
//!
//! See `ontology.rs` for the checked Governance/Slot/Filler/Verdict category
//! and its axioms.
//!
//! Cite: Fillmore, C. J. (1976) "Frame semantics and the nature of
//! language", *Annals of the NY Academy of Sciences* 280(1):20-32; Fillmore,
//! C. J. (1982) "Frame Semantics", in *Linguistics in the Morning Calm*,
//! Hanshin, 111-137; Minsky, M. (1974) "A Framework for Representing
//! Knowledge", MIT-AI Memo 306; Sergot, M., Sadri, F., Kowalski, R.,
//! Kriwaczek, F., Hammond, P. & Cory, T. (1986) "The British Nationality Act
//! as a Logic Program", *CACM* 29(5):370-386; Codd, E. F. (1979) "Extending
//! the Database Relational Model to Capture More Meaning", *ACM TODS*
//! 4(4):397-434 (NULL as a distinct third truth value, not collapsed
//! negation-as-failure).

#[allow(unused_imports)]
use alloc::{vec, vec::Vec};

use pr4xis::category::NonEmpty;

use crate::cognitive::linguistics::english::ConceptId;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use crate::social::judicial::ontology::{EvidenceRequirement, LegalTerm, RequirementLevel};

pub mod ontology;
pub mod registry;

#[cfg(test)]
mod tests;

/// The value that fills a Slot once known (Bobrow et al. 1977: the GUS
/// "answer" to a slot-filling prompt). Typed per this codebase's
/// primitive-leak-sweep discipline — never a bare `bool`/`f64`/`String`
/// crossing a function boundary unwrapped. Deliberately three variants, not
/// a fourth free-text catch-all: a free-text fact has no narrower typed
/// shape yet identified in the corpus this design was built against (42
/// U.S.C. § 1396p / § 1396b(l)); add one, cited, when a real need surfaces
/// rather than stubbing it now.
#[derive(Debug, Clone, PartialEq)]
pub enum FactValue {
    Boolean(bool),
    /// A measured fact with units/dimension — income, an asset value, a
    /// duration. Reuses [`Quantity`], the same type the crate-wide
    /// primitive-leak sweep converted 223 prior leaks to.
    Amount(Quantity),
    /// A fact that is itself a choice among loaded concepts (e.g. which
    /// HCBS service).
    Concept(ConceptId),
}

/// The state of one [`AppliedElement`]'s Slot for THIS turn — three-valued
/// (Codd 1979: NULL is a distinct third value, never collapsed into
/// `false`). `Unfilled` is where every element of a freshly-loaded
/// [`ConditionalRule`] starts (see [`ConditionalRule::from_term`]).
/// Nothing in this module ever constructs `Satisfied`/`NotSatisfied` — that
/// is the multi-turn slot-filling layer's job. The type is complete now so
/// that layer has a correct shape to write into rather than one that has to
/// change again later.
#[derive(Debug, Clone, PartialEq)]
pub enum SlotState {
    Unfilled,
    Satisfied(FactValue),
    NotSatisfied(FactValue),
}

/// One [`EvidenceRequirement`] of a [`LegalTerm`], carrying this turn's
/// [`SlotState`] — Fillmore's (1976/1982) FRAME ELEMENT made evaluable.
/// `requirement` is the EXISTING, UNCHANGED `EvidenceRequirement` — nothing
/// about it is duplicated or renamed.
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedElement {
    pub requirement: EvidenceRequirement,
    pub state: SlotState,
}

impl AppliedElement {
    pub fn unfilled(requirement: EvidenceRequirement) -> Self {
        Self {
            requirement,
            state: SlotState::Unfilled,
        }
    }
}

/// Whether a [`ConditionalRule`] applies, for THIS asker, THIS turn.
#[derive(Debug, Clone, PartialEq)]
pub enum Applicability {
    Applies,
    /// At least one `Required`-tier element is `NotSatisfied` — decisive
    /// regardless of other unfilled elements (a rule with one failed
    /// condition does not apply; mirrors Sergot et al. 1986's conjunctive
    /// reduction). Unreachable in production until the slot-filling layer
    /// ever produces a `NotSatisfied` state — the type is complete now
    /// regardless.
    DoesNotApply,
    /// At least one `Required`-tier element is `Unfilled` and none is
    /// `NotSatisfied` — Sergot et al. (1986)'s reduction cannot complete
    /// without the missing fact(s). `missing` names EXACTLY the blocking
    /// elements. `NonEmpty` (not `Vec`) makes an empty `missing` list
    /// unrepresentable rather than an unchecked invariant — this variant is
    /// only ever constructed when at least one element genuinely blocks.
    Indeterminate {
        missing: NonEmpty<AppliedElement>,
    },
}

/// A fully-known, fully-cited [`LegalTerm`] (Sergot, Sadri, Kowalski,
/// Kriwaczek, Hammond & Cory 1986: a statute reduced to named, individually
/// evaluable conditions) applied to one asker's turn.
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalRule {
    pub term: LegalTerm,
    /// The Hart (1961 §V-VI) taxonomy leaf `term`'s source belongs to — set
    /// once, at extraction time, from the SAME [`SourceTaxonomyConcept`] the
    /// loading corpus already carries (e.g. `UsFederalStatute` for a
    /// Title-42 U.S. Code term, `Regulation` for a 42 C.F.R. term). Its
    /// binding-law floor is checked via the source taxonomy's
    /// `LegalCorpus`-vs-`Lexicon` split — see
    /// `ontology::ConditionalRuleGroundedInBindingLaw`. `Authority`
    /// itself is deliberately NOT embedded here (it derives only
    /// `PartialEq`, not `Eq`/`Hash`), following the SAME CURIE-resolution
    /// convention `judicial/mod.rs` already documents for every other
    /// synthesized-hierarchy field.
    pub source_kind: SourceTaxonomyConcept,
    /// One [`AppliedElement`] per `term.required_evidence` entry.
    pub elements: Vec<AppliedElement>,
}

impl ConditionalRule {
    /// Build a freshly-loaded `ConditionalRule` from a `LegalTerm` — every
    /// element starts `Unfilled`.
    pub fn from_term(term: LegalTerm, source_kind: SourceTaxonomyConcept) -> Self {
        let elements = term
            .required_evidence
            .iter()
            .cloned()
            .map(AppliedElement::unfilled)
            .collect();
        Self {
            term,
            source_kind,
            elements,
        }
    }

    /// The multi-turn slot-filling layer's ONE write path (task #17): a new
    /// `ConditionalRule` with the element whose `requirement.field.text`
    /// matches `field` moved from `Unfilled` to `state`, every other element
    /// unchanged. `field` is matched against the SAME text
    /// `Applicability::Indeterminate`'s `missing` already names (never a
    /// fresh string the caller invents), so a caller that only ever fills a
    /// field it was just asked about cannot target the wrong slot. A `field`
    /// matching nothing is a no-op (returns an unchanged clone) — the caller
    /// is expected to have taken `field` from this same rule's own
    /// `missing` list, so a non-match signals a caller bug, not a runtime
    /// condition to panic over.
    pub fn with_element_filled(&self, field: &str, state: SlotState) -> Self {
        let mut next = self.clone();
        for el in &mut next.elements {
            if el.requirement.field.text == field {
                el.state = state;
                break;
            }
        }
        next
    }

    /// Sergot et al. (1986)'s reduction, evaluated over this turn's
    /// [`SlotState`]s. Only `RequirementLevel::Required` elements block —
    /// `Recommended`/`Optional` elements never appear in `missing` and never
    /// prevent `Applies` (an eligibility rule with an unfilled optional
    /// supporting document is still evaluable).
    pub fn applicability(&self) -> Applicability {
        let mut missing = Vec::new();
        for el in &self.elements {
            if el.requirement.required != RequirementLevel::Required {
                continue;
            }
            match &el.state {
                SlotState::NotSatisfied(_) => return Applicability::DoesNotApply,
                SlotState::Unfilled => missing.push(el.clone()),
                SlotState::Satisfied(_) => {}
            }
        }
        match missing.split_first() {
            Some((head, tail)) => Applicability::Indeterminate {
                missing: NonEmpty::of(head.clone(), tail.to_vec()),
            },
            None => Applicability::Applies,
        }
    }
}
