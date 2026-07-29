//! The loaded PropBank roleset-alias index, indexed for the corroboration
//! query: do two WordNet concepts, occurring at DIFFERENT parts of speech,
//! share a PropBank roleset's argument-structure frame?
//!
//! ## LIVE resolution — FrameNet's precedent, not SUMO's offline crosswalk
//!
//! PropBank's `<alias pos="...">text</alias>` entries carry bare lemma+POS
//! strings only — no machine-readable WordNet sense-key or synset reference
//! anywhere in the schema (confirmed 2026-07-13: a grep across sample lemma
//! files found zero `wn=`/`wordnet` hits outside free-text `<note>` prose,
//! and the DTD's `rolelink`/`lexlink` cross-resource vocabulary observed in
//! the data is `{AMR, FrameNet, PropBank, VerbNet}`, never `WordNet`). This
//! is structurally identical to
//! [`crate::cognitive::linguistics::framenet::store`]'s situation (no native
//! WN link), not [`crate::cognitive::linguistics::sumo::store`]'s (native WN
//! synset offsets, wrong PWN version, needing offline sense-key
//! reconstruction). So `PropBankStore` is built exactly on FrameNet's
//! precedent: index `(lemma, LmfPos) → {RolesetId}`, resolved LIVE against
//! the loaded `LexicalReasoner` at query time — never a precomputed
//! crosswalk baked in by an offline regen step.
//!
//! ## Cross-POS scoping is the entire signal
//!
//! The prevalence research behind this build settled the design empirically —
//! though the real figure, re-derived directly from the committed corpus, is
//! more modest than the design-time sample suggested: a 15-common-verb sample
//! (trade, break, open, ...) found ~67% cross-POS co-occurrence, but across
//! the full committed data (8,804 verb-bearing rolesets), only 3,670 — 41.7% —
//! also carry a same-roleset noun/adjective alias (e.g. `trade.01` carries
//! both the verb `trade` and the noun `trading`; most single-sense technical
//! or rare verbs in the long tail do not). A substantial minority, not a
//! majority — still a real, non-trivial signal worth building, just not the
//! near-universal one the curated sample implied.
//! [`PropBankStore::shares_roleset`] therefore requires the TWO
//! concepts being compared to occur at DIFFERENT parts of speech — same-POS
//! matches (two verb aliases, or two noun aliases, in one roleset) are
//! excluded entirely: they would be redundant with VerbNet's existing
//! verb-verb signal and add no new information. The gate is total and
//! simple: look up each concept's own reachable rolesets AT ITS OWN POS,
//! then require BOTH a shared roleset AND a POS mismatch between the two
//! concepts — which, since each side's roleset set was only ever populated
//! from aliases matching that side's own POS, makes any surviving shared
//! roleset a genuine cross-POS witness by construction (never a
//! same-POS match slipping through disguised as one).

#[allow(unused_imports)]
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec::Vec,
};

use super::ontology::PropBank;
use crate::cognitive::linguistics::english::{ConceptId, LexicalReasoner};
use crate::social::software::markup::xml::lmf::LmfPos;

/// Normalize a surface lemma for lookup: lowercase, spaces and hyphens
/// folded to `_`. Mirrors
/// [`crate::cognitive::linguistics::framenet::store::normalize_lemma`]'s
/// exact behavior — kept as a small local copy rather than a cross-source
/// dependency (VerbNet, ConceptNet, FrameNet, SUMO, and PropBank are peer
/// instance-data loaders with no reason to import from one another; the
/// transform itself is a five-line generic surface-canonicalization, not
/// source-specific logic).
#[must_use]
pub fn normalize_lemma(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' | '-' => '_',
            c => c.to_ascii_lowercase(),
        })
        .collect()
}

/// The loaded, indexed PropBank data — the corroboration mechanism's query
/// surface. `(lemma, LmfPos) → {roleset id}`, populated ONLY from aliases
/// whose DTD `pos` code has a defined [`LmfPos`] mapping (see
/// [`super::ontology::propbank_pos_to_lmf`]) — the five undocumented codes
/// never enter the index, on either side of a query.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PropBankStore {
    lemma_pos_to_rolesets: BTreeMap<(String, LmfPos), BTreeSet<String>>,
}

impl PropBankStore {
    /// Build the indexed store from the typed, reader-produced [`PropBank`]
    /// data.
    #[must_use]
    pub fn from_propbank(pb: &PropBank) -> Self {
        let mut index: BTreeMap<(String, LmfPos), BTreeSet<String>> = BTreeMap::new();
        for frameset in &pb.framesets {
            for predicate in &frameset.predicates {
                for roleset in &predicate.rolesets {
                    for alias in &roleset.aliases {
                        let Some(pos) = alias.pos else {
                            continue; // undocumented code — excluded, never guessed
                        };
                        index
                            .entry((normalize_lemma(&alias.text), pos))
                            .or_default()
                            .insert(roleset.id.clone());
                    }
                }
            }
        }
        Self {
            lemma_pos_to_rolesets: index,
        }
    }

    /// The roleset ids `concept` reaches, keyed at its OWN `(lemma, pos)`
    /// pairs (a synset can have several lemmas; each lemma at this
    /// concept's POS may independently carry roleset membership).
    fn rolesets_for(&self, en: &dyn LexicalReasoner, concept: ConceptId) -> BTreeSet<&String> {
        let Some(view) = en.concept(concept) else {
            return BTreeSet::new();
        };
        let pos = view.pos();
        view.lemmas()
            .filter_map(|lemma| {
                self.lemma_pos_to_rolesets
                    .get(&(normalize_lemma(lemma), pos))
            })
            .flatten()
            .collect()
    }

    /// Does `concept` have ANY PropBank roleset membership at all (at its
    /// own POS)? The epistemic distinction [`PropBankStore::shares_roleset`]'s
    /// `false` alone can't make — mirrors
    /// [`crate::cognitive::linguistics::framenet::store::FrameNetStore::has_coverage`]'s
    /// same rationale.
    #[must_use]
    pub fn has_coverage(&self, en: &dyn LexicalReasoner, concept: ConceptId) -> bool {
        !self.rolesets_for(en, concept).is_empty()
    }

    /// Does `a` share a PropBank roleset with `b` — i.e. do they occur at
    /// DIFFERENT parts of speech AND reach at least one roleset id in
    /// common (each reached via aliases matching that concept's own POS)?
    ///
    /// The POS-mismatch gate is load-bearing, not incidental: two concepts
    /// at the SAME POS sharing a roleset (two verb senses, say) would be
    /// redundant with VerbNet's existing verb-verb signal and carries no
    /// information PropBank's data specifically contributes — exactly the
    /// scoping this build's prevalence research established. `false` if
    /// either concept is unresolvable, or the two share the same POS, or
    /// they reach no common roleset.
    #[must_use]
    pub fn shares_roleset(&self, en: &dyn LexicalReasoner, a: ConceptId, b: ConceptId) -> bool {
        let (Some(view_a), Some(view_b)) = (en.concept(a), en.concept(b)) else {
            return false;
        };
        if view_a.pos() == view_b.pos() {
            return false; // same-POS matches never count — see the module doc
        }
        let rolesets_a = self.rolesets_for(en, a);
        let rolesets_b = self.rolesets_for(en, b);
        rolesets_a.iter().any(|r| rolesets_b.contains(r))
    }
}

/// The process-wide loaded PropBank store — the committed `propbank@3.4.0`
/// `.prx` decoded, parsed, and indexed once. Mirrors
/// [`crate::cognitive::linguistics::framenet::store::framenet_loaded`]'s
/// caching shape: built lazily on first use, reused for the process
/// lifetime.
#[cfg(feature = "std")]
pub fn propbank_loaded() -> &'static PropBankStore {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<PropBankStore> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        use crate::applied::data_provisioning::decoders::propbank_frameset_collection;
        use crate::applied::data_provisioning::raw_source_prx::raw_source_bytes_embedded;

        const PROPBANK_PRX: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/propbank/propbank-3.4.0.prx"
        ));

        let bytes = raw_source_bytes_embedded("propbank", "3.4.0", PROPBANK_PRX);
        let collection = propbank_frameset_collection::decode(&bytes)
            .unwrap_or_else(|e| panic!("propbank committed .prx archive failed to decode: {e}"));
        let pb = super::reader::read_propbank(&collection);
        PropBankStore::from_propbank(&pb)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::english::English;
    use crate::cognitive::linguistics::propbank::ontology::{
        PropBank, PropBankFrameset, PropBankPredicate, Roleset, RolesetAlias,
    };
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    fn alias(text: &str, pos_code: &str, pos: Option<LmfPos>) -> RolesetAlias {
        RolesetAlias {
            text: text.to_string(),
            pos_code: pos_code.to_string(),
            pos,
        }
    }

    /// Fixture mirroring the real `trade.xml`/`out_trade` shape: `trade.01`
    /// is a cross-POS roleset (verb `trade` + noun `trading`), `eat.01` is a
    /// verb-only roleset (no cross-POS witness), and `bank.01` deliberately
    /// carries TWO VERB aliases in one roleset (a same-POS pair — not
    /// observed as a real pattern in PropBank, per the build spec, but not
    /// structurally impossible) to prove the same-POS exclusion is real.
    fn fixture_propbank() -> PropBank {
        PropBank {
            framesets: alloc::vec![PropBankFrameset {
                predicates: alloc::vec![
                    PropBankPredicate {
                        lemma: "trade".to_string(),
                        rolesets: alloc::vec![Roleset {
                            id: "trade.01".to_string(),
                            aliases: alloc::vec![
                                alias("trading", "n", Some(LmfPos::Noun)),
                                alias("trade", "v", Some(LmfPos::Verb)),
                                // undocumented code: excluded from the index
                                alias("make_trade", "l", None),
                            ],
                        }],
                    },
                    PropBankPredicate {
                        lemma: "eat".to_string(),
                        rolesets: alloc::vec![Roleset {
                            id: "eat.01".to_string(),
                            aliases: alloc::vec![alias("eat", "v", Some(LmfPos::Verb))],
                        }],
                    },
                    PropBankPredicate {
                        lemma: "bank".to_string(),
                        rolesets: alloc::vec![Roleset {
                            id: "bank.01".to_string(),
                            aliases: alloc::vec![
                                alias("bank", "v", Some(LmfPos::Verb)),
                                alias("bank_up", "v", Some(LmfPos::Verb)),
                            ],
                        }],
                    },
                ],
            },],
        }
    }

    /// A minimal WN-LMF fixture reasoner. Every lemma below is declared at
    /// EXACTLY ONE part of speech (no lemma is overloaded across POS), so
    /// `en.lookup(word)[0]` is unambiguous — `trade` (verb) and `trading`
    /// (noun) are two distinct WordNet concepts sharing NO synset, `eat`
    /// (verb) is an unrelated third, `bank`/`bank_up` (both verb) are the
    /// same-POS `bank.01` co-members, and `widget` (noun) is a concept
    /// PropBank never mentions at all — the genuine "no coverage" witness.
    fn fixture_reasoner() -> English {
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-trade-v"><Lemma writtenForm="trade" partOfSpeech="v"/><Sense id="trade-v-1" synset="s-trade-v"/></LexicalEntry>
    <LexicalEntry id="e-trading-n"><Lemma writtenForm="trading" partOfSpeech="n"/><Sense id="trading-n-1" synset="s-trading-n"/></LexicalEntry>
    <LexicalEntry id="e-eat-v"><Lemma writtenForm="eat" partOfSpeech="v"/><Sense id="eat-v-1" synset="s-eat-v"/></LexicalEntry>
    <LexicalEntry id="e-bank-v"><Lemma writtenForm="bank" partOfSpeech="v"/><Sense id="bank-v-1" synset="s-bank-v"/></LexicalEntry>
    <LexicalEntry id="e-bank-up-v"><Lemma writtenForm="bank_up" partOfSpeech="v"/><Sense id="bank-up-v-1" synset="s-bank-up-v"/></LexicalEntry>
    <LexicalEntry id="e-widget-n"><Lemma writtenForm="widget" partOfSpeech="n"/><Sense id="widget-n-1" synset="s-widget-n"/></LexicalEntry>
    <Synset id="s-trade-v" ili="i1" partOfSpeech="v"><Definition>engage in exchange</Definition></Synset>
    <Synset id="s-trading-n" ili="i2" partOfSpeech="n"><Definition>the act of trading</Definition></Synset>
    <Synset id="s-eat-v" ili="i4" partOfSpeech="v"><Definition>consume food</Definition></Synset>
    <Synset id="s-bank-v" ili="i5" partOfSpeech="v"><Definition>tilt an aircraft</Definition></Synset>
    <Synset id="s-bank-up-v" ili="i6" partOfSpeech="v"><Definition>pile up</Definition></Synset>
    <Synset id="s-widget-n" ili="i7" partOfSpeech="n"><Definition>a small manufactured item</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"))
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn normalize_folds_space_and_hyphen_to_underscore_and_lowercases() {
        assert_eq!(normalize_lemma("Well-Known"), "well_known");
        assert_eq!(normalize_lemma("some name"), "some_name");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn positive_match_cross_pos_roleset_sharing_is_found() {
        // The exact real-world case this corroboration mechanism exists
        // for: the VERB sense of "trade" and the NOUN sense of "trading"
        // both trace to trade.01 via aliases at their OWN differing POS.
        let store = PropBankStore::from_propbank(&fixture_propbank());
        let en = fixture_reasoner();
        let trade_v = en.lookup("trade")[0];
        let trading_n = en.lookup("trading")[0];
        assert!(store.shares_roleset(&en, trade_v, trading_n));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn negative_match_both_covered_but_no_shared_roleset() {
        // "trade" (verb) and "eat" (verb) are BOTH covered by PropBank, but
        // share no roleset — and are same-POS besides, so the gate must
        // reject them for either reason.
        let store = PropBankStore::from_propbank(&fixture_propbank());
        let en = fixture_reasoner();
        let trade_v = en.lookup("trade")[0];
        let eat_v = en.lookup("eat")[0];
        assert!(!store.shares_roleset(&en, trade_v, eat_v));
        assert!(store.has_coverage(&en, trade_v));
        assert!(store.has_coverage(&en, eat_v));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn has_coverage_distinguishes_no_data_from_no_connection() {
        let store = PropBankStore::from_propbank(&fixture_propbank());
        let en = fixture_reasoner();
        let trade_v = en.lookup("trade")[0];
        assert!(store.has_coverage(&en, trade_v));
        // "widget" (noun) is a WordNet concept PropBank never mentions at
        // all in the fixture data — genuinely uncovered, not merely
        // "queried, no connection".
        let widget_n = en.lookup("widget")[0];
        assert!(!store.has_coverage(&en, widget_n));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn same_pos_pair_in_one_roleset_does_not_count_as_a_match() {
        // TEETH for the cross-POS scoping: bank.01 carries TWO VERB aliases
        // (bank, bank_up) in the SAME roleset. Both are covered, both share
        // the roleset id in the raw index — but since they occur at the
        // SAME LmfPos, shares_roleset must reject the pair.
        let store = PropBankStore::from_propbank(&fixture_propbank());
        let en = fixture_reasoner();
        let bank_v = en.lookup("bank")[0];
        let bank_up_v = en.lookup("bank_up")[0];
        assert!(!store.shares_roleset(&en, bank_v, bank_up_v));
        // Both DO have coverage — a real "queried, no connection" (in fact
        // a same-POS-excluded connection), not "no data".
        assert!(store.has_coverage(&en, bank_v));
        assert!(store.has_coverage(&en, bank_up_v));
    }
}
