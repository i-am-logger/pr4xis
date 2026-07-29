//! SUMO's WordNet↔SUMO mappings as typed Rust data — the RESOLVED shape
//! [`super::reader::read_sumo`] populates from the committed, extracted TSV.
//!
//! Mirrors [`crate::cognitive::linguistics::framenet::ontology`]'s plain
//! struct-family shape: an instance-data loader, not a `pr4xis::ontology!`
//! category (same reasoning as VerbNet/ConceptNet/FrameNet — no schema-driven
//! codegen exists anywhere in this codebase, and this is bounded third-party
//! corpus data, not a domain reasoning category).
//!
//! ## The committed table is already RESOLVED to `ConceptId`
//!
//! SUMO's `WordNetMappings30-{noun,verb,adj,adv}.txt` files annotate Princeton
//! WordNet **3.0** synsets (by their 3.0 offset). Open English WordNet 2025 —
//! the WordNet this project loads — does NOT preserve PWN 3.0 offsets as its
//! synset ids (it renumbered), so a live `oewn-<offset>-<pos>` lookup resolves
//! almost nothing. The version-stable bridge is the WordNet SENSE KEY (lemma +
//! `ss_type:lex_filenum:lex_id`), which each SUMO WNDB record carries in full.
//! So — exactly like VerbNet — the synset→`ConceptId` resolution is done ONCE,
//! OFFLINE, by `super::regenerate` (which can afford the ~89 MB OEWN XML
//! parse a runtime load is built to avoid), and the committed TSV is emitted
//! ALREADY RESOLVED: one `concept_value<TAB>term<TAB>relation_code` row per
//! (resolved synset, SUMO term, relation). A SUMO synset none of whose sense
//! keys resolve against this WordNet build is dropped offline. This targets the
//! numeric `ConceptId` VALUE, which VerbNet's store doc establishes is IDENTICAL
//! across the raw-XML and compact/store-bundle load paths — so the value baked
//! at regen time is the same value `english_loaded()` serves at runtime. The
//! store therefore needs NO WordNet at load and NO live resolution.
//!
//! ## The suffix → relation-kind legend (source-of-truth for the relation column)
//!
//! Each raw SUMO annotation is `&%<SumoTerm><suffix>`; the source file's own
//! header legend states the suffix semantics verbatim (WordNetMappings30-noun.txt,
//! lines 5-13):
//!
//! > Each SUMO concept is designated with the prefix '&%'. Note that each
//! > concept also has a suffix, '=', ':', '+', '[', ']' or '@', which
//! > indicates the precise relationship between the SUMO concept and the
//! > WordNet synset. The symbols '=', '+', and '@' mean, respectively, that
//! > the WordNet synset is equivalent in meaning to the SUMO concept, is
//! > subsumed by the SUMO concept or is an instance of the SUMO concept.
//! > ':', '[', and ']' are the complements of those relations. For example, a
//! > mapping expressed as (%ComplementFn &%Motion)+ now appears as &%Motion[.
//! > Note also that ']' has not currently been needed.
//!
//! A complement (`:`/`[`/`]`) means the synset is explicitly NOT that SUMO
//! class — the reason the `Complement*` variants are carried yet EXCLUDED from
//! the positive corroboration signal (see
//! [`super::store::SumoStore::shares_sumo_class`]).
//!
//! References:
//! - Niles, I. & Pease, A. (2001). "Towards a Standard Upper Ontology."
//!   Proceedings of the 2nd International Conference on Formal Ontology in
//!   Information Systems (FOIS 2001), pp. 2-9.
//! - Niles, I. & Pease, A. (2003). "Linking Lexicons and Ontologies: Mapping
//!   WordNet to the Suggested Upper Merged Ontology." Proceedings of the IEEE
//!   International Conference on Information and Knowledge Engineering (IKE
//!   2003), pp. 412-416.

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

use crate::cognitive::linguistics::english::ConceptId;

/// The relationship a SUMO annotation asserts between a WordNet synset and a
/// SUMO term — the six suffix codes of the source's own legend (see the module
/// doc). The three `Complement*` variants assert the synset is explicitly NOT
/// that SUMO class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SumoRelationKind {
    /// `=` — the synset is EQUIVALENT in meaning to the SUMO term.
    Equivalence,
    /// `+` — the synset is SUBSUMED by (is-a, a subclass of) the SUMO term.
    Subsumption,
    /// `@` — the synset is an INSTANCE of the SUMO term.
    Instance,
    /// `:` — complement of [`Equivalence`](Self::Equivalence): the synset is
    /// explicitly NOT equivalent to the SUMO term.
    ComplementEquivalence,
    /// `[` — complement of [`Subsumption`](Self::Subsumption): the synset is
    /// explicitly NOT subsumed by the SUMO term.
    ComplementSubsumption,
    /// `]` — complement of [`Instance`](Self::Instance): the synset is
    /// explicitly NOT an instance of the SUMO term. The source's legend notes
    /// this suffix "has not currently been needed" (zero occurrences observed);
    /// carried for completeness so the suffix alphabet is total.
    ComplementInstance,
}

impl SumoRelationKind {
    /// Is this a `Complement*` relation — an assertion the synset is NOT the
    /// SUMO class? The three positive relations (Equivalence/Subsumption/
    /// Instance) are same-class witnesses; the three complements are not (see
    /// [`super::store::SumoStore::shares_sumo_class`]).
    #[must_use]
    pub fn is_complement(self) -> bool {
        matches!(
            self,
            Self::ComplementEquivalence | Self::ComplementSubsumption | Self::ComplementInstance
        )
    }

    /// Parse a SUMO annotation SUFFIX character (the source's own legend — see
    /// the module doc) into its relation kind. `None` for any character that is
    /// not one of the six legend suffixes, so the extractor fails closed
    /// per-annotation rather than inventing a relation.
    #[must_use]
    pub fn from_suffix(c: char) -> Option<Self> {
        Some(match c {
            '=' => Self::Equivalence,
            '+' => Self::Subsumption,
            '@' => Self::Instance,
            ':' => Self::ComplementEquivalence,
            '[' => Self::ComplementSubsumption,
            ']' => Self::ComplementInstance,
            _ => return None,
        })
    }

    /// The stable derived-TSV relation CODE — the token
    /// `super::regenerate` writes into the extracted table and
    /// [`super::reader::read_sumo`] parses back. Inverse of
    /// [`from_code`](Self::from_code). Kept distinct from the raw source
    /// suffix character so the committed TSV is self-describing rather than
    /// carrying a bare punctuation glyph.
    #[must_use]
    pub fn to_code(self) -> &'static str {
        match self {
            Self::Equivalence => "EQ",
            Self::Subsumption => "SUB",
            Self::Instance => "INST",
            Self::ComplementEquivalence => "CEQ",
            Self::ComplementSubsumption => "CSUB",
            Self::ComplementInstance => "CINST",
        }
    }

    /// Parse a derived-TSV relation code back into its relation kind. `None`
    /// for any unrecognized code, so [`super::reader::read_sumo`] skips a
    /// malformed row fail-closed. Exact inverse of [`to_code`](Self::to_code).
    #[must_use]
    pub fn from_code(s: &str) -> Option<Self> {
        Some(match s {
            "EQ" => Self::Equivalence,
            "SUB" => Self::Subsumption,
            "INST" => Self::Instance,
            "CEQ" => Self::ComplementEquivalence,
            "CSUB" => Self::ComplementSubsumption,
            "CINST" => Self::ComplementInstance,
            _ => return None,
        })
    }
}

/// One RESOLVED SUMO mapping — a WordNet concept, the SUMO upper-ontology term
/// it maps to, and the relation kind. The synset→`ConceptId` resolution was
/// done offline by `super::regenerate` (see the module doc); the committed
/// table carries the concept directly, so no WordNet lookup happens at load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumoMapping {
    pub concept: ConceptId,
    pub term: String,
    pub relation: SumoRelationKind,
    /// The REAL, external Open English WordNet synset id (e.g.
    /// `oewn-00001740-n`) that `concept` resolves to — distinct from `concept`
    /// itself, which is this project's internal, load-path-stable numeric
    /// `ConceptId` value. Carried through unparsed from the committed TSV's 4th
    /// column (`super::regenerate`); consumed by [`super::sssom`] as the SSSOM
    /// `subject_id` (a real, dereferenceable `https://en-word.net/id/...` URI).
    pub oewn_synset_id: String,
}

/// The full loaded, resolved WordNet-concept↔SUMO-term mapping table.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Sumo {
    pub mappings: Vec<SumoMapping>,
}
