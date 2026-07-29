//! PropBank as typed Rust data — the frameset→predicate→roleset→alias shape
//! [`super::reader::read_propbank`] populates from the committed, archived
//! frame-file collection.
//!
//! Mirrors [`crate::cognitive::linguistics::verbnet::ontology`]'s recursive/
//! nested-collection shape: a plain, hand-written struct family describing
//! what the loaded per-lemma frame XML MEANS, populated by a hand-written
//! reader over the generic `XmlDocument` tree, not derived by any
//! schema-driven codegen (no source in this codebase gets one). An
//! instance-data loader, not a `pr4xis::ontology!` category — the same
//! reasoning as VerbNet/ConceptNet/FrameNet/SUMO: bounded third-party corpus
//! data, not a domain reasoning category.
//!
//! ## Only `<aliases>` is modeled — not roles/rolelinks/lexlinks/examples
//!
//! The real DTD (`propbank/propbank-frames` `dtds/v3.4/frameset.dtd`,
//! verified 2026-07-13) declares a much richer per-`<roleset>` tree:
//! `(aliases?, note*, roles, usagenotes*, (lexlinks | example | note)*)`.
//! This ontology carries only `<aliases>` — the numbered `<role>` argument
//! frame, `<rolelinks>` (VerbNet/FrameNet/AMR cross-resource pointers),
//! `<usagenotes>`, and the annotated `<example>` sentences are the
//! argument-structure PAYLOAD a semantic-role labeler would need, out of
//! scope for a lone-hit WORD-SENSE corroboration signal built entirely from
//! cross-POS alias co-membership. Mirrors VerbNet's own restraint: that
//! ontology carries `<MEMBERS>`/`<SUBCLASSES>` but not `<THEMROLES>`/
//! `<FRAMES>`/`<SEMANTICS>`, for the identical reason (unused by the
//! corroboration query, so not modeled).
//!
//! ## The `<alias pos="...">` POS alphabet
//!
//! The real DTD's `alias` `pos` attribute enumerates TEN codes:
//! `r | p | v | n | j | l | x | m | d | f`. Its own comment documents only
//! five (`r`=Adverb, `p`=Preposition, `v`=Verb, `n`=Noun, `j`=Adjective);
//! the other five (`l`, `x`, `m`, `d`, `f`) are UNDOCUMENTED in the source's
//! own comment (`l` is empirically a light-verb form — e.g. `trade.xml`'s
//! `<alias pos="l">make_trade</alias>` — but the DTD never says so). Only
//! the five documented codes have a defined [`LmfPos`] mapping;
//! [`propbank_pos_to_lmf`] returns `None` for the rest, and every
//! `RolesetAlias` still carries its raw `pos_code` string so the
//! undocumented codes are never silently coerced into a wrong `LmfPos` —
//! they are parsed, carried, and excluded from the corroboration index
//! (see [`super::store`]).
//!
//! References:
//! - Palmer, M., Gildea, D. & Kingsbury, P. (2005). "The Proposition Bank:
//!   An Annotated Corpus of Semantic Roles." Computational Linguistics
//!   31(1):71-106.
//! - Bonial, C., Bonn, J., Conger, K., Hwang, J. & Palmer, M. (2014).
//!   "PropBank: Semantics of New Predicate Types." LREC 2014.

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

use crate::social::software::markup::xml::lmf::LmfPos;

/// One `<alias pos="...">text</alias>` inside a roleset's `<aliases>` block —
/// a lemma+POS surface form sharing that roleset's argument-structure frame
/// (e.g. `trade.01`'s aliases include the verb `trade` and the noun
/// `trading`). `pos` is `Some` only for the five DTD-documented codes
/// (`v`/`n`/`j`/`r`/`p`); the five undocumented codes (`l`/`x`/`m`/`d`/`f`,
/// e.g. the light-verb form `make_trade`) are carried on `pos_code` but map
/// to `None` — see the module doc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RolesetAlias {
    /// The alias's surface text (e.g. `"trading"`, `"make_trade"`).
    pub text: String,
    /// The raw DTD `pos` attribute letter, unparsed (e.g. `"v"`, `"l"`).
    pub pos_code: String,
    /// The alias's part of speech, mapped to the shared [`LmfPos`]
    /// vocabulary — `None` for an undocumented DTD code (never silently
    /// coerced into a guessed [`LmfPos`]).
    pub pos: Option<LmfPos>,
}

/// One `<roleset id="..." name="...">` — a predicate's numbered
/// argument-structure sense (e.g. `trade.01`, "exchange"). Carries only its
/// identity and its `<aliases>` (see the module doc for what is
/// deliberately NOT modeled).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roleset {
    /// The roleset id, e.g. `"trade.01"` (lemma + a 2-digit sense number).
    pub id: String,
    /// The lemma+POS surface forms sharing this roleset's argument frame.
    pub aliases: Vec<RolesetAlias>,
}

/// One `<predicate lemma="...">` — a lemma's set of numbered rolesets (e.g.
/// `trade` carries `trade.01`; a distinct predicate entry like `trade_off`
/// carries its own `trade_off.03`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropBankPredicate {
    /// The predicate's lemma, e.g. `"trade"`, `"trade_off"`.
    pub lemma: String,
    pub rolesets: Vec<Roleset>,
}

/// One parsed frame file — the root `<frameset>` of one `frames/<lemma>.xml`
/// document, which may declare more than one `<predicate>` (light-verb and
/// multi-word variants of the same base lemma live in the same file, e.g.
/// `trade.xml` carries `trade`, `out_trade`, `trade_off`, and `trade_in`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PropBankFrameset {
    pub predicates: Vec<PropBankPredicate>,
}

/// The full loaded PropBank frame collection — one [`PropBankFrameset`] per
/// archived frame file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PropBank {
    pub framesets: Vec<PropBankFrameset>,
}

/// Map one PropBank DTD `<alias pos="...">` code letter to the shared
/// [`LmfPos`] vocabulary. Only the five codes the DTD's own comment
/// documents participate: `v`→Verb, `n`→Noun, `j`→Adjective, `r`→Adverb,
/// `p`→Preposition. `None` for any other code — including the five
/// DTD-ATTLIST-enumerated but comment-undocumented codes `l`/`x`/`m`/`d`/`f`
/// (never observed with a name in the source's own legend) — so an
/// undocumented code is never silently coerced into a guessed `LmfPos`.
#[must_use]
pub fn propbank_pos_to_lmf(code: &str) -> Option<LmfPos> {
    Some(match code {
        "v" => LmfPos::Verb,
        "n" => LmfPos::Noun,
        "j" => LmfPos::Adjective,
        "r" => LmfPos::Adverb,
        "p" => LmfPos::Preposition,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn propbank_pos_to_lmf_maps_the_five_documented_codes() {
        assert_eq!(propbank_pos_to_lmf("v"), Some(LmfPos::Verb));
        assert_eq!(propbank_pos_to_lmf("n"), Some(LmfPos::Noun));
        assert_eq!(propbank_pos_to_lmf("j"), Some(LmfPos::Adjective));
        assert_eq!(propbank_pos_to_lmf("r"), Some(LmfPos::Adverb));
        assert_eq!(propbank_pos_to_lmf("p"), Some(LmfPos::Preposition));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn propbank_pos_to_lmf_excludes_every_undocumented_code() {
        // The DTD's full ATTLIST alphabet is r|p|v|n|j|l|x|m|d|f — the last
        // five carry no comment-documented meaning (l is empirically a
        // light-verb marker, e.g. trade.xml's `pos="l"` on `make_trade`, but
        // the DTD never says so) and must never resolve to a guessed LmfPos.
        for code in ["l", "x", "m", "d", "f", "bogus", ""] {
            assert_eq!(propbank_pos_to_lmf(code), None, "code {code:?}");
        }
    }
}
