//! Information — fundamental units of representation that bridge abstract
//! concepts (meanings, truths, quantities) and concrete representations
//! (bits, bytes, text, references).
//!
//! This ontology names WHAT information units ARE — Rust types like
//! `u8`, `u32`, `String` are implementations of these concepts.
//!
//! # Literature
//!
//! - **Shannon (1948)** "A Mathematical Theory of Communication", *Bell
//!   System Technical Journal* 27 — bit as the fundamental unit of
//!   information; entropy as the measure of representational content.
//! - **Turing (1936)** "On Computable Numbers, with an Application to
//!   the Entscheidungsproblem", *Proc. London Mathematical Society* 42 —
//!   addressing, sequences, the tape model of computation.
//! - **IEEE 1872-2015** *Standard Ontologies for Robotics and
//!   Automation* — has-a / is-a / equivalent relation conventions
//!   adopted here (SUMO/BFO-compatible).

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Info",
    source: "Shannon (1948) A Mathematical Theory of Communication, Bell System Technical Journal 27; Turing (1936) On Computable Numbers, Proc. London Mathematical Society 42; IEEE 1872-2015 Standard Ontologies for Robotics and Automation",

    concepts: [
        Bit,
        Byte,
        Word,
        Reference,
        Sequence,
        Text,
        TruthValue,
        Quantity,
    ],

    labels: {
        Bit: ("en", "Bit",
            "Shannon (1948): fundamental unit of binary information - two states 0 or 1; semantically equivalent to a single truth value."),
        Byte: ("en", "Byte",
            "Eight Bits - the standard addressable unit of information."),
        Word: ("en", "Word",
            "A fixed-size group of Bytes processed as a single unit. Architecture-dependent (2, 4, or 8 bytes typically)."),
        Reference: ("en", "Reference",
            "Turing (1936): an address pointing to an entity - a Word used as a location identifier (e.g., SynsetId IS a Reference)."),
        Sequence: ("en", "Sequence",
            "An ordered collection of information units."),
        Text: ("en", "Text",
            "A Sequence of characters - human-readable information."),
        TruthValue: ("en", "Truth value",
            "Shannon (1948): a true/false value - semantically equivalent to a single Bit."),
        Quantity: ("en", "Quantity",
            "A numeric value representing magnitude - a Sequence of digits in a numeral system."),
    },

    is_a: [
        // IEEE 1872 is-a: a Reference IS a Word (used as an address).
        (Reference, Word),
        // Text IS a Sequence (of characters).
        (Text, Sequence),
        // Quantity IS a Sequence (of digits).
        (Quantity, Sequence),
    ],

    has_a: [
        // Mereological: Byte has-a Bit (composed of 8 bits).
        (Byte, Bit),
        // Word has-a Byte (composed of bytes).
        (Word, Byte),
        // Text has-a Sequence (its underlying carrier).
        (Text, Sequence),
    ],

    edges: [
        // Shannon (1948): TruthValue and Bit are semantically equivalent.
        (TruthValue, Bit, Equivalence),
        (Bit, TruthValue, Equivalence),
    ],
}

/// Legacy alias for the generated concept enum — preserves the
/// `InfoUnit` name used by existing call-sites.
pub type InfoUnit = InfoConcept;

impl InfoConcept {
    /// Is this an atomic unit (no internal structure)?
    pub fn is_atomic(&self) -> bool {
        matches!(self, Self::Bit | Self::TruthValue | Self::Sequence)
    }

    /// Is this a structured unit (composed of or derived from other units)?
    pub fn is_structured(&self) -> bool {
        !self.is_atomic()
    }
}

/// Quality: size in bits of each information unit. Atomic units have a
/// fixed bit-width; structured units vary by architecture or content.
#[derive(Debug, Clone)]
pub struct BitSize;

impl Quality for BitSize {
    type Individual = InfoConcept;
    type Value = usize;

    fn get(&self, c: &InfoConcept) -> Option<usize> {
        match c {
            InfoConcept::Bit | InfoConcept::TruthValue => Some(1),
            InfoConcept::Byte => Some(8),
            _ => None,
        }
    }
}

impl Ontology for InfoOntology {
    type Cat = InfoCategory;
    type Qual = BitSize;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

/// A Reference — an address that points to an entity.
///
/// This is the value-level realisation of `InfoConcept::Reference`. A
/// Reference is a Word (fixed-size) used as a location identifier. The
/// referent (what it points to) gives it meaning; the Reference itself is
/// just an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ref<const BYTES: usize> {
    value: u64,
}

impl<const BYTES: usize> Ref<BYTES> {
    pub const fn new(value: u64) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> u64 {
        self.value
    }

    /// Size in bytes of this reference.
    pub fn size_bytes(&self) -> usize {
        BYTES
    }

    /// Maximum addressable entities.
    pub fn max_addressable(&self) -> u64 {
        if BYTES >= 8 {
            u64::MAX
        } else {
            (1u64 << (BYTES * 8)) - 1
        }
    }
}

/// A 4-byte reference — can address ~4 billion entities. Used by
/// SynsetId for any reasonable lexical database.
pub type Ref32 = Ref<4>;

/// An 8-byte reference — can address ~18 quintillion entities.
pub type Ref64 = Ref<8>;

// Re-export the legacy `Reference<N>` name to avoid breaking call-sites.
// The new concept-level `InfoConcept::Reference` and the value-level
// `Reference<N>` resolved through their type kind, so `Reference<N>`
// continues to refer to the value-level struct here.
pub type Reference<const N: usize> = Ref<N>;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<InfoCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        InfoOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn eight_concepts() {
        assert_eq!(InfoConcept::variants().len(), 8);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn byte_composed_of_bits() {
        // Mereological: a Bit is PART of a Byte (part→whole, BFO:0000050).
        let m = InfoCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == InfoConcept::Bit
            && r.target() == InfoConcept::Byte
            && r.kind() == InfoRelationKind::Parthood));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reference_is_a_word() {
        // Subsumption: Reference IS-A Word.
        let m = InfoCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == InfoConcept::Reference
            && r.target() == InfoConcept::Word
            && r.kind() == InfoRelationKind::Subsumption));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn truth_value_equivalent_to_bit() {
        // Shannon (1948): TruthValue and Bit are semantically equivalent.
        let m = InfoCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == InfoConcept::TruthValue
            && r.target() == InfoConcept::Bit
            && r.kind() == InfoRelationKind::Equivalence));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ref32_size() {
        let r: Ref32 = Ref::new(42);
        assert_eq!(r.size_bytes(), 4);
        assert_eq!(r.value(), 42);
        assert_eq!(r.max_addressable(), (1u64 << 32) - 1);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ref64_size() {
        let r: Ref64 = Ref::new(999);
        assert_eq!(r.size_bytes(), 8);
        assert_eq!(r.max_addressable(), u64::MAX);
    }

    fn arb_concept() -> impl Strategy<Value = InfoConcept> {
        proptest::sample::select(InfoConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in InfoCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in InfoOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_atomic_or_structured(c in arb_concept()) {
            prop_assert!(c.is_atomic() != c.is_structured());
        }

        #[test]
        fn prop_bitsize_partial(c in arb_concept()) {
            // Defined only on Bit, Byte, TruthValue.
            let v = BitSize.get(&c);
            let is_fixed = matches!(c, InfoConcept::Bit | InfoConcept::Byte | InfoConcept::TruthValue);
            prop_assert_eq!(v.is_some(), is_fixed);
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_atomic_or_structured, Verifiable);
    pr4xis::register_praxis_value!(prop_bitsize_partial, Verifiable);
}
