//! The fold-on-miss secondary index builder (Slice D,
//! `.notes/chat-fix-c-build-state.md`) — derives an `English` instance's
//! [`WordIndex`] keyed by CASE-FOLDED surface, unioning concept ids across
//! every original-cased word in the source index that folds to the same
//! key, via the loaded Unicode simple case-folding table
//! ([`case_folding`] module).
//!
//! Only entries whose OWN fold DIFFERS from their exact-case key are
//! included — an already-lowercase lemma ("dog") needs no fold-index entry,
//! since [`lookup_case_folded`](super::ontology::LexicalReasoner::lookup_case_folded)
//! tries the folded query against the ordinary exact index FIRST (covering
//! an all-caps/title-case typo of an already-lowercase lemma for free)
//! before consulting this index for the genuinely case-marked population
//! ("Section Eight", "O.K.", "Turkish bath").

#[allow(unused_imports)]
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::HashMap;

use super::ontology::ConceptId;
use super::word_index::WordIndex;
use crate::cognitive::linguistics::orthography::case_folding;

/// Build the fold index from an already-packed [`WordIndex`] — every word,
/// its own fold (if it differs from itself), unioning concept ids per fold
/// key.
pub fn build(word_index: &WordIndex) -> WordIndex {
    let folder = case_folding::table();
    let mut map: HashMap<String, Vec<ConceptId>> = HashMap::new();
    for word in word_index.words() {
        let folded = folder.fold(word);
        if folded == word {
            continue;
        }
        let entry = map.entry(folded).or_default();
        for &id in word_index.lookup(word) {
            if !entry.contains(&id) {
                entry.push(id);
            }
        }
    }
    WordIndex::build(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }

    fn source() -> WordIndex {
        let mut map: HashMap<String, Vec<ConceptId>> = HashMap::new();
        map.insert(String::from("dog"), alloc::vec![cid(1)]);
        map.insert(String::from("Section Eight"), alloc::vec![cid(2), cid(3)]);
        map.insert(String::from("O.K."), alloc::vec![cid(4)]);
        WordIndex::build(map)
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn only_case_marked_entries_are_indexed() {
        let idx = build(&source());
        // "dog" folds to itself — no entry.
        assert!(idx.lookup("dog").is_empty());
        // "Section Eight" folds to "section eight" — a fresh entry.
        assert_eq!(idx.lookup("section eight"), &[cid(2), cid(3)]);
        assert_eq!(idx.lookup("o.k."), &[cid(4)]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn distinct_original_casings_union_under_the_same_fold() {
        let mut map: HashMap<String, Vec<ConceptId>> = HashMap::new();
        map.insert(String::from("Bass"), alloc::vec![cid(1)]);
        map.insert(String::from("BASS"), alloc::vec![cid(2)]);
        let idx = build(&WordIndex::build(map));
        let ids = idx.lookup("bass");
        assert_eq!(
            ids.len(),
            2,
            "both casings' ids are unioned under one fold key"
        );
        assert!(ids.contains(&cid(1)));
        assert!(ids.contains(&cid(2)));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn an_all_lowercase_source_produces_an_empty_fold_index() {
        let mut map: HashMap<String, Vec<ConceptId>> = HashMap::new();
        map.insert(String::from("dog"), alloc::vec![cid(1)]);
        map.insert(String::from("cat"), alloc::vec![cid(2)]);
        let idx = build(&WordIndex::build(map));
        assert!(idx.is_empty(), "nothing folds away from itself");
    }
}
