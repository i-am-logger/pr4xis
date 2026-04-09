use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

// Information ontology — the science of how knowledge is represented.
//
// Information is the bridge between abstract concepts (meanings, truths,
// quantities) and concrete representations (bits, bytes, text, references).
//
// This ontology defines WHAT information units ARE — not Rust types,
// but the concepts that Rust types implement.
//
// References:
// - Claude Shannon, A Mathematical Theory of Communication (1948)
// - Alan Turing, On Computable Numbers (1936)

/// Fundamental units of information representation.
///
/// These are the ontological concepts — what things ARE.
/// Rust types (u8, u32, String) are implementations of these concepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfoUnit {
    /// The fundamental unit of binary information. Two states: 0 or 1.
    /// Connects to logic: a Bit IS a truth value.
    Bit,

    /// 8 bits. The standard addressable unit of information.
    Byte,

    /// A fixed-size group of bytes processed as a single unit.
    /// Size depends on architecture (2, 4, or 8 bytes typically).
    Word,

    /// An address that points to an entity. A Word used as a location.
    /// SynsetId IS a Reference — it points to a concept.
    Reference,

    /// An ordered collection of information units.
    Sequence,

    /// A sequence of characters — human-readable information.
    /// Connects to linguistics/symbols: Text has-a Characters.
    Text,

    /// A true/false value. Semantically equivalent to a single Bit.
    /// Connects to praxis::logic: TruthValue IS a proposition's result.
    TruthValue,

    /// A quantity — a numeric value representing magnitude.
    /// Connects to science/math.
    Quantity,
}

impl Entity for InfoUnit {
    fn variants() -> Vec<Self> {
        vec![
            Self::Bit,
            Self::Byte,
            Self::Word,
            Self::Reference,
            Self::Sequence,
            Self::Text,
            Self::TruthValue,
            Self::Quantity,
        ]
    }
}

impl InfoUnit {
    /// Is this an atomic unit (no internal structure)?
    pub fn is_atomic(&self) -> bool {
        matches!(self, Self::Bit | Self::TruthValue | Self::Sequence)
    }

    /// Is this a structured unit (composed of or derived from other units)?
    pub fn is_structured(&self) -> bool {
        !self.is_atomic()
    }
}

/// Relationships between information units.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InfoRelation {
    pub from: InfoUnit,
    pub to: InfoUnit,
    pub kind: InfoRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfoRelationKind {
    Identity,
    /// Mereological: A has-a B (Byte has-a Bits).
    ComposedOf,
    /// Taxonomic: A is-a B (Reference is-a Word).
    IsA,
    /// Semantic equivalence (TruthValue is-like Bit).
    Equivalent,
    /// Composed (transitive).
    Composed,
}

impl Relationship for InfoRelation {
    type Object = InfoUnit;
    fn source(&self) -> InfoUnit {
        self.from
    }
    fn target(&self) -> InfoUnit {
        self.to
    }
}

/// The information category.
pub struct InfoCategory;

impl Category for InfoCategory {
    type Object = InfoUnit;
    type Morphism = InfoRelation;

    fn identity(obj: &InfoUnit) -> InfoRelation {
        InfoRelation {
            from: *obj,
            to: *obj,
            kind: InfoRelationKind::Identity,
        }
    }

    fn compose(f: &InfoRelation, g: &InfoRelation) -> Option<InfoRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == InfoRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == InfoRelationKind::Identity {
            return Some(f.clone());
        }
        Some(InfoRelation {
            from: f.from,
            to: g.to,
            kind: InfoRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<InfoRelation> {
        use InfoRelationKind::*;
        use InfoUnit::*;

        let mut m = Vec::new();

        // Identity
        for u in InfoUnit::variants() {
            m.push(InfoRelation {
                from: u,
                to: u,
                kind: Identity,
            });
        }

        // Composition (has-a / mereology)
        // Byte is composed of Bits
        m.push(InfoRelation {
            from: Byte,
            to: Bit,
            kind: ComposedOf,
        });
        // Word is composed of Bytes
        m.push(InfoRelation {
            from: Word,
            to: Byte,
            kind: ComposedOf,
        });
        // Text is composed of a Sequence
        m.push(InfoRelation {
            from: Text,
            to: Sequence,
            kind: ComposedOf,
        });

        // Taxonomy (is-a)
        // Reference is-a Word (a word used as an address)
        m.push(InfoRelation {
            from: Reference,
            to: Word,
            kind: IsA,
        });
        // Text is-a Sequence (of characters)
        m.push(InfoRelation {
            from: Text,
            to: Sequence,
            kind: IsA,
        });
        // Quantity is-a Sequence (of digits)
        m.push(InfoRelation {
            from: Quantity,
            to: Sequence,
            kind: IsA,
        });
        // TruthValue equivalent to Bit (semantically)
        m.push(InfoRelation {
            from: TruthValue,
            to: Bit,
            kind: Equivalent,
        });

        // Transitive closure
        // Word composed of Bits (via Bytes)
        m.push(InfoRelation {
            from: Word,
            to: Bit,
            kind: Composed,
        });
        // Reference composed of Bytes (via Word)
        m.push(InfoRelation {
            from: Reference,
            to: Byte,
            kind: Composed,
        });
        // Reference composed of Bits
        m.push(InfoRelation {
            from: Reference,
            to: Bit,
            kind: Composed,
        });

        m
    }
}

/// A Reference — an address that points to an entity.
///
/// This is the ontological definition of what SynsetId, NodeId, etc. ARE.
/// A Reference is a Word (fixed-size) used as a location identifier.
/// The referent (what it points to) gives it meaning — the Reference
/// itself is just an address.
///
/// In category theory terms: a Reference's identity comes from its
/// morphisms (what it relates to), not from its value (Yoneda lemma).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reference<const BYTES: usize> {
    value: u64,
}

impl<const BYTES: usize> Reference<BYTES> {
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    pub fn value(&self) -> u64 {
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

/// A 4-byte reference — can address up to ~4 billion entities.
/// This is what SynsetId uses: efficient for any lexical database.
pub type Ref32 = Reference<4>;

/// An 8-byte reference — can address up to ~18 quintillion entities.
pub type Ref64 = Reference<8>;
